use std::collections::BTreeMap;
use std::fmt;

use num_bigint::BigInt;
use topal_source::{SourceText, Span};
use topal_syntax::{Expression, Statement, lex, parse};

use crate::{TraceEvent, TraceSink};

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
    /// Returns a source, syntax, name-resolution, or evaluation diagnostic.
    pub fn evaluate(
        &mut self,
        input: &str,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let source = SourceText::new(input).map_err(|error| {
            let (line, column) = raw_position(input, error.span.start);
            Diagnostic {
                code: error.code,
                line,
                column,
                message: error.message.into(),
            }
        })?;
        trace.record(TraceEvent {
            event: "source.accepted",
            rule: "TOPAL-SYN-SOURCE-001",
            detail: "unicode source normalized",
        });

        let parsed = parse(&source, &lex(&source));
        if let Some(error) = parsed.diagnostics.first() {
            return Err(diagnostic(&source, error.code, error.span, error.message));
        }
        if parsed.statements.is_empty() {
            return Err(Diagnostic {
                code: "E-EXPECTED-EXPRESSION",
                line: 1,
                column: 1,
                message: "expected a statement".into(),
            });
        }

        let mut result = None;
        for (index, statement) in parsed.statements.iter().enumerate() {
            match statement {
                Statement::Binding { name, value } => {
                    let name_text = source.slice(*name);
                    if self.bindings.contains_key(name_text) {
                        return Err(diagnostic(
                            &source,
                            "E-DUPLICATE-BINDING",
                            *name,
                            "name is already bound in this scope",
                        ));
                    }
                    let evaluated = self.evaluate_expression(&source, value, trace)?;
                    self.bindings.insert(name_text.to_owned(), evaluated);
                    trace.record(TraceEvent {
                        event: "binding.created",
                        rule: "TOPAL-SYN-BIND-001",
                        detail: name_text,
                    });
                    result = Some(Value::Unit);
                }
                Statement::Expression(expression) => {
                    if index + 1 != parsed.statements.len() {
                        return Err(diagnostic(
                            &source,
                            "E-DISCARDED-VALUE",
                            expression.span(),
                            "a non-final expression value cannot be discarded",
                        ));
                    }
                    result = Some(self.evaluate_expression(&source, expression, trace)?);
                }
            }
        }
        let value = result.ok_or_else(|| Diagnostic {
            code: "E-EXPECTED-EXPRESSION",
            line: 1,
            column: 1,
            message: "expected a statement".into(),
        })?;
        trace.record(TraceEvent {
            event: "evaluation.result",
            rule: "TOPAL-SYN-GRAMMAR-001",
            detail: match &value {
                Value::Int(_) => "Int",
                Value::Unit => "Unit",
            },
        });
        Ok(value)
    }

    fn evaluate_expression(
        &self,
        source: &SourceText,
        expression: &Expression,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        match expression {
            Expression::Integer(span) => {
                let text = source.slice(*span);
                let value = parse_integer(text).ok_or_else(|| {
                    diagnostic(
                        source,
                        "E-NUMERIC-LITERAL",
                        *span,
                        "invalid integer literal",
                    )
                })?;
                trace.record(TraceEvent {
                    event: "token.integer",
                    rule: "TOPAL-NUM-LITERAL-001",
                    detail: text,
                });
                Ok(Value::Int(value))
            }
            Expression::Identifier(span) => {
                let name = source.slice(*span);
                let value = self.bindings.get(name).cloned().ok_or_else(|| {
                    diagnostic(source, "E-UNBOUND-NAME", *span, "name is not bound")
                })?;
                trace.record(TraceEvent {
                    event: "binding.resolved",
                    rule: "TOPAL-SYN-BIND-001",
                    detail: name,
                });
                Ok(value)
            }
            Expression::Add { left, right, span } => {
                let left = self.evaluate_expression(source, left, trace)?;
                let right = self.evaluate_expression(source, right, trace)?;
                let (Value::Int(left), Value::Int(right)) = (left, right) else {
                    return Err(diagnostic(
                        source,
                        "E-NO-APPLICABLE-OVERLOAD",
                        *span,
                        "+ requires two Int operands in the implemented subset",
                    ));
                };
                trace.record(TraceEvent {
                    event: "operator.selected",
                    rule: "TOPAL-TYPE-CALL-001",
                    detail: "root.+(Int,Int)",
                });
                trace.record(TraceEvent {
                    event: "evaluation.add",
                    rule: "TOPAL-NUM-ADD-001",
                    detail: "Int",
                });
                Ok(Value::Int(left + right))
            }
        }
    }
}

fn diagnostic(
    source: &SourceText,
    code: &'static str,
    span: Span,
    message: impl Into<String>,
) -> Diagnostic {
    let position = source.position(span.start);
    Diagnostic {
        code,
        line: position.line,
        column: position.column,
        message: message.into(),
    }
}

fn raw_position(source: &str, offset: usize) -> (usize, usize) {
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

fn parse_integer(token: &str) -> Option<BigInt> {
    if let Some(unsigned) = token.strip_prefix('-') {
        return parse_unsigned_integer(unsigned).map(std::ops::Neg::neg);
    }
    parse_unsigned_integer(token)
}

fn parse_unsigned_integer(token: &str) -> Option<BigInt> {
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
    valid_based_digits(digits, radix)
        .then(|| BigInt::parse_bytes(digits.replace('_', "").as_bytes(), radix))
        .flatten()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn evaluate(source: &str) -> Result<Value, Diagnostic> {
        Session::new().evaluate(source, &mut std::io::sink())
    }

    #[test]
    fn adds_signed_arbitrary_precision_integers() {
        assert_eq!(
            evaluate("-1 + 123456789012345678901234567890")
                .unwrap()
                .to_string(),
            "123456789012345678901234567889"
        );
    }

    #[test]
    fn follows_left_association() {
        assert_eq!(evaluate("1 + 2 + 3").unwrap().to_string(), "6");
    }

    #[test]
    fn parentheses_group_addition() {
        assert_eq!(evaluate("1 + (2 + 3)").unwrap().to_string(), "6");
    }

    #[test]
    fn evaluates_binding_and_lookup() {
        assert_eq!(
            evaluate("answer is 40 + 2\nanswer").unwrap().to_string(),
            "42"
        );
    }

    #[test]
    fn rejects_incomplete_grouping() {
        assert_eq!(evaluate("12_34").unwrap_err().code, "E-NUMERIC-LITERAL");
    }
}
