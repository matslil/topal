use std::fmt;

use num_bigint::BigInt;

use crate::{TraceEvent, TraceSink};

/// A value produced by the implemented language subset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    Integer(BigInt),
}

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => value.fmt(formatter),
        }
    }
}

/// A source diagnostic with a stable category and source position.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}:{}: {}",
            self.code, self.line, self.column, self.message
        )
    }
}

impl std::error::Error for Diagnostic {}

/// Persistent evaluation context used by scripts and interactive sessions.
#[derive(Default)]
pub struct Session;

impl Session {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Evaluate one source unit and return its final value.
    ///
    /// # Errors
    ///
    /// Returns a source diagnostic when decoding, lexing, or parsing fails.
    pub fn evaluate(
        &mut self,
        source: &str,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let normalized = normalize(source)?;
        trace.record(TraceEvent {
            event: "source.accepted",
            rule: "TOPAL-SYN-SOURCE-001",
            detail: "unicode source normalized",
        });

        let token = scan_single_integer(&normalized)?;
        trace.record(TraceEvent {
            event: "token.integer",
            rule: "TOPAL-SYN-NUM-001",
            detail: token,
        });

        let digits = token.replace('_', "");
        let value = digits.parse::<BigInt>().map_err(|_| Diagnostic {
            code: "E-NUMERIC-LITERAL",
            line: 1,
            column: 1,
            message: "invalid decimal integer literal".into(),
        })?;
        trace.record(TraceEvent {
            event: "evaluation.result",
            rule: "TOPAL-SYN-GRAMMAR-001",
            detail: "Integer",
        });
        Ok(Value::Integer(value))
    }
}

fn normalize(source: &str) -> Result<String, Diagnostic> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let source = source.strip_prefix("#!").map_or(source, |rest| {
        rest.find('\n').map_or("", |newline| &rest[newline + 1..])
    });

    for (offset, character) in source.char_indices() {
        if character == '\0' || is_noncharacter(character) {
            let (line, column) = position(source, offset);
            return Err(Diagnostic {
                code: "E-SOURCE-DECODE",
                line,
                column,
                message: "source contains a forbidden Unicode scalar".into(),
            });
        }
        if character == '\r' && !source[offset..].starts_with("\r\n") {
            let (line, column) = position(source, offset);
            return Err(Diagnostic {
                code: "E-SOURCE-LINE-END",
                line,
                column,
                message: "bare carriage return is not a valid line ending".into(),
            });
        }
    }
    Ok(source.replace("\r\n", "\n"))
}

fn scan_single_integer(source: &str) -> Result<&str, Diagnostic> {
    let mut token = None;
    for (line_index, line) in source.lines().enumerate() {
        let code = line.split_once('#').map_or(line, |(before, _)| before);
        let trimmed = code.trim();
        if trimmed.is_empty() {
            continue;
        }
        if token.is_some() {
            return Err(Diagnostic {
                code: "E-UNSUPPORTED-SYNTAX",
                line: line_index + 1,
                column: code.find(trimmed).unwrap_or(0) + 1,
                message: "the initial subset accepts one integer expression".into(),
            });
        }
        token = Some((trimmed, line_index + 1, code.find(trimmed).unwrap_or(0) + 1));
    }

    let Some((token, line, column)) = token else {
        return Err(Diagnostic {
            code: "E-EXPECTED-EXPRESSION",
            line: 1,
            column: 1,
            message: "expected an integer expression".into(),
        });
    };
    if valid_decimal_integer(token) {
        Ok(token)
    } else {
        Err(Diagnostic {
            code: "E-UNSUPPORTED-SYNTAX",
            line,
            column,
            message: "the initial subset accepts one decimal integer literal".into(),
        })
    }
}

fn valid_decimal_integer(token: &str) -> bool {
    if token == "0" {
        return true;
    }
    if token.starts_with('0')
        || !token
            .bytes()
            .all(|byte| byte.is_ascii_digit() || byte == b'_')
    {
        return false;
    }
    if !token.contains('_') {
        return true;
    }
    let mut groups = token.split('_');
    let first = groups.next().unwrap_or_default();
    (1..=3).contains(&first.len())
        && first.bytes().all(|byte| byte.is_ascii_digit())
        && groups.all(|group| group.len() == 3 && group.bytes().all(|byte| byte.is_ascii_digit()))
}

const fn is_noncharacter(character: char) -> bool {
    let scalar = character as u32;
    (scalar >= 0xfdd0 && scalar <= 0xfdef) || scalar & 0xffff >= 0xfffe
}

fn position(source: &str, offset: usize) -> (usize, usize) {
    let before = &source[..offset];
    let line = before.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = before
        .rsplit_once('\n')
        .map_or(before, |(_, tail)| tail)
        .chars()
        .count()
        + 1;
    (line, column)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evaluate(source: &str) -> Result<Value, Diagnostic> {
        Session::new().evaluate(source, &mut std::io::sink())
    }

    #[test]
    fn evaluates_arbitrary_precision_integer() {
        let value = evaluate("123456789012345678901234567890").unwrap();
        assert_eq!(value.to_string(), "123456789012345678901234567890");
    }

    #[test]
    fn accepts_complete_grouping() {
        assert_eq!(evaluate("12_345_678").unwrap().to_string(), "12345678");
    }

    #[test]
    fn rejects_incomplete_grouping() {
        assert_eq!(evaluate("12_34").unwrap_err().code, "E-UNSUPPORTED-SYNTAX");
    }

    #[test]
    fn rejects_bare_carriage_return() {
        assert_eq!(evaluate("1\r").unwrap_err().code, "E-SOURCE-LINE-END");
    }
}
