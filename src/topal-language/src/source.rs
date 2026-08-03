use std::collections::BTreeMap;
use std::fmt;

use num_bigint::BigInt;

use crate::{TraceEvent, TraceSink};

/// A value produced by the implemented language subset.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    Int(BigInt),
    Unit,
}

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(value) => value.fmt(formatter),
            Self::Unit => formatter.write_str("()"),
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
pub struct Session {
    bindings: BTreeMap<String, Value>,
}

impl Session {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
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

        let statements = scan_statements(&normalized)?;
        let mut result = None;
        for (index, statement) in statements.iter().enumerate() {
            match statement {
                Statement::Binding { name, expression } => {
                    if self.bindings.contains_key(*name) {
                        return Err(Diagnostic {
                            code: "E-DUPLICATE-BINDING",
                            line: expression.line,
                            column: 1,
                            message: format!("{name} is already bound in this scope"),
                        });
                    }
                    let value = self.evaluate_expression(expression, trace)?;
                    self.bindings.insert((*name).to_owned(), value);
                    trace.record(TraceEvent {
                        event: "binding.created",
                        rule: "TOPAL-SYN-BIND-001",
                        detail: name,
                    });
                    result = Some(Value::Unit);
                }
                Statement::Expression(expression) => {
                    if index + 1 != statements.len() {
                        return Err(Diagnostic {
                            code: "E-DISCARDED-VALUE",
                            line: expression.line,
                            column: expression.column,
                            message: "a non-final expression value cannot be discarded".into(),
                        });
                    }
                    result = Some(self.evaluate_expression(expression, trace)?);
                }
            }
        }
        let value = result.ok_or_else(|| Diagnostic {
            code: "E-EXPECTED-EXPRESSION",
            line: 1,
            column: 1,
            message: "expected a statement".into(),
        })?;
        let classifier = match &value {
            Value::Int(_) => "Int",
            Value::Unit => "Unit",
        };
        trace.record(TraceEvent {
            event: "evaluation.result",
            rule: "TOPAL-SYN-GRAMMAR-001",
            detail: classifier,
        });
        Ok(value)
    }

    fn evaluate_expression(
        &self,
        expression: &Expression<'_>,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        if let Some(value) = parse_integer(expression.text) {
            trace.record(TraceEvent {
                event: "token.integer",
                rule: "TOPAL-SYN-NUM-001",
                detail: expression.text,
            });
            return Ok(Value::Int(value));
        }
        if valid_identifier(expression.text) {
            let value = self
                .bindings
                .get(expression.text)
                .cloned()
                .ok_or_else(|| Diagnostic {
                    code: "E-UNBOUND-NAME",
                    line: expression.line,
                    column: expression.column,
                    message: format!("{} is not bound", expression.text),
                })?;
            trace.record(TraceEvent {
                event: "binding.resolved",
                rule: "TOPAL-SYN-BIND-001",
                detail: expression.text,
            });
            return Ok(value);
        }
        Err(Diagnostic {
            code: "E-UNSUPPORTED-SYNTAX",
            line: expression.line,
            column: expression.column,
            message: "the implemented subset accepts integer literals and bound names".into(),
        })
    }
}

enum Statement<'a> {
    Binding {
        name: &'a str,
        expression: Expression<'a>,
    },
    Expression(Expression<'a>),
}

struct Expression<'a> {
    text: &'a str,
    line: usize,
    column: usize,
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

fn scan_statements(source: &str) -> Result<Vec<Statement<'_>>, Diagnostic> {
    let mut statements = Vec::new();
    for (line_index, line) in source.lines().enumerate() {
        let code = line.split_once('#').map_or(line, |(before, _)| before);
        let trimmed = code.trim();
        if trimmed.is_empty() {
            continue;
        }
        let line_number = line_index + 1;
        let column = code.find(trimmed).unwrap_or(0) + 1;
        let words = trimmed.split_whitespace().collect::<Vec<_>>();
        let statement = match words.as_slice() {
            [expression] => Statement::Expression(Expression {
                text: expression,
                line: line_number,
                column,
            }),
            [name, "is", expression] if valid_identifier(name) => Statement::Binding {
                name,
                expression: Expression {
                    text: expression,
                    line: line_number,
                    column: column + name.chars().count() + 4,
                },
            },
            _ => {
                return Err(Diagnostic {
                    code: "E-UNSUPPORTED-SYNTAX",
                    line: line_number,
                    column,
                    message: "the implemented subset accepts `name is expression` or one expression per line".into(),
                });
            }
        };
        statements.push(statement);
    }
    if statements.is_empty() {
        return Err(Diagnostic {
            code: "E-EXPECTED-EXPRESSION",
            line: 1,
            column: 1,
            message: "expected an integer expression".into(),
        });
    }
    Ok(statements)
}

fn valid_identifier(token: &str) -> bool {
    if token == "_" {
        return false;
    }
    let mut characters = token.chars().peekable();
    if !characters.next().is_some_and(unicode_ident::is_xid_start) {
        return false;
    }
    while let Some(character) = characters.next() {
        if character == '-' {
            if !characters
                .peek()
                .is_some_and(|next| unicode_ident::is_xid_continue(*next))
            {
                return false;
            }
        } else if !unicode_ident::is_xid_continue(character) {
            return false;
        }
    }
    true
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

fn parse_integer(token: &str) -> Option<BigInt> {
    if valid_decimal_integer(token) {
        return token.replace('_', "").parse().ok();
    }
    let (radix, digits) = if let Some(digits) = token.strip_prefix("0b") {
        (2, digits)
    } else if let Some(digits) = token.strip_prefix("0o") {
        (8, digits)
    } else {
        (16, token.strip_prefix("0x")?)
    };
    if !valid_based_digits(digits, radix) {
        return None;
    }
    BigInt::parse_bytes(digits.replace('_', "").as_bytes(), radix)
}

fn valid_based_digits(digits: &str, radix: u32) -> bool {
    if digits.is_empty() {
        return false;
    }
    let valid_group =
        |group: &str| !group.is_empty() && group.chars().all(|character| character.is_digit(radix));
    if !digits.contains('_') {
        return valid_group(digits);
    }
    let mut groups = digits.split('_');
    let first = groups.next().unwrap_or_default();
    (1..=4).contains(&first.len())
        && valid_group(first)
        && groups.all(|group| group.len() == 4 && valid_group(group))
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
    fn evaluates_based_integers() {
        assert_eq!(evaluate("0b1010").unwrap().to_string(), "10");
        assert_eq!(evaluate("0o755").unwrap().to_string(), "493");
        assert_eq!(evaluate("0xCAFE_BABE").unwrap().to_string(), "3405691582");
    }

    #[test]
    fn rejects_incomplete_based_grouping() {
        assert_eq!(
            evaluate("0xCA_FEBABE").unwrap_err().code,
            "E-UNSUPPORTED-SYNTAX"
        );
    }

    #[test]
    fn rejects_incomplete_grouping() {
        assert_eq!(evaluate("12_34").unwrap_err().code, "E-UNSUPPORTED-SYNTAX");
    }

    #[test]
    fn rejects_bare_carriage_return() {
        assert_eq!(evaluate("1\r").unwrap_err().code, "E-SOURCE-LINE-END");
    }

    #[test]
    fn evaluates_binding_and_lookup() {
        assert_eq!(evaluate("answer is 42\nanswer").unwrap().to_string(), "42");
    }

    #[test]
    fn rejects_duplicate_binding() {
        assert_eq!(
            evaluate("answer is 1\nanswer is 2\nanswer")
                .unwrap_err()
                .code,
            "E-DUPLICATE-BINDING"
        );
    }

    #[test]
    fn rejects_unbound_name() {
        assert_eq!(evaluate("missing").unwrap_err().code, "E-UNBOUND-NAME");
    }

    #[test]
    fn accepts_unicode_and_hyphenated_identifiers() {
        assert_eq!(
            evaluate("värde-1 is 42\nvärde-1").unwrap().to_string(),
            "42"
        );
    }
}
