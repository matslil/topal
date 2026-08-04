use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt::{self, Write as _};

use num_bigint::BigInt;
use num_rational::BigRational;
use topal_source::{SourceText, Span};
use topal_syntax::{CallableKind, Expression, Statement, lex, parse};

use crate::{ExecutionSnapshot, TraceEvent, TraceSink};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    Boolean(bool),
    Int(BigInt),
    Rational(BigRational),
    String(String),
    Tuple(Vec<Self>),
    Unit,
}

impl fmt::Display for Value {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(value) => value.fmt(formatter),
            Self::Int(value) => value.fmt(formatter),
            Self::Rational(value) => {
                write!(
                    formatter,
                    "Rational ( {}, {} )",
                    value.numer(),
                    value.denom()
                )
            }
            Self::String(value) => formatter.write_str(&display_string(value)),
            Self::Tuple(items) => {
                formatter.write_str("(")?;
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        formatter.write_str(", ")?;
                    }
                    item.fmt(formatter)?;
                }
                if items.len() == 1 {
                    formatter.write_str(",")?;
                }
                formatter.write_str(")")
            }
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
    source_line: Option<String>,
    marker_width: usize,
    help: Option<String>,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.render("<input>"))
    }
}

impl std::error::Error for Diagnostic {}

impl Diagnostic {
    #[must_use]
    pub fn render(&self, source_name: &str) -> String {
        let mut rendered = format!(
            "error[{}]: {}\n --> {source_name}:{}:{}",
            self.code, self.message, self.line, self.column
        );
        if let Some(source_line) = &self.source_line {
            let gutter_width = self.line.to_string().len();
            let _ = write!(
                rendered,
                "\n{empty:>gutter_width$} |\n{line:>gutter_width$} | {source_line}\n{empty:>gutter_width$} | {padding}{markers}",
                empty = "",
                line = self.line,
                padding = " ".repeat(self.column.saturating_sub(1)),
                markers = "^".repeat(self.marker_width.max(1)),
            );
        }
        if let Some(help) = &self.help {
            let _ = write!(
                rendered,
                "\n{empty:>width$} |\n{empty:>width$} = help: {help}",
                empty = "",
                width = self.line.to_string().len()
            );
        }
        rendered
    }

    fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }
}

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
        self.checkpoint(trace, None);
        trace.record(TraceEvent {
            event: "context.selected",
            rule: "TOPAL-SYN-UNICODE-001",
            detail: "design-0;Unicode=17.0.0",
        });
        let source = SourceText::new(input).map_err(|error| {
            let (line, column) = raw_position(input, error.span.start);
            Diagnostic {
                code: error.code,
                line,
                column,
                message: error.message.into(),
                source_line: raw_source_line(input, line),
                marker_width: marker_width(input, error.span),
                help: diagnostic_help(error.code).map(str::to_owned),
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
                source_line: raw_source_line(input, 1),
                marker_width: 1,
                help: diagnostic_help("E-EXPECTED-EXPRESSION").map(str::to_owned),
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
                    self.checkpoint(trace, result.as_ref());
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
            source_line: raw_source_line(input, 1),
            marker_width: 1,
            help: diagnostic_help("E-EXPECTED-EXPRESSION").map(str::to_owned),
        })?;
        trace.record(TraceEvent {
            event: "evaluation.result",
            rule: "TOPAL-SYN-GRAMMAR-001",
            detail: match &value {
                Value::Boolean(_) => "Boolean",
                Value::Int(_) => "Int",
                Value::Rational(_) => "Rational",
                Value::String(_) => "String",
                Value::Tuple(_) => "Tuple",
                Value::Unit => "Unit",
            },
        });
        self.checkpoint(trace, Some(&value));
        Ok(value)
    }

    fn checkpoint(&self, trace: &mut impl TraceSink, value: Option<&Value>) {
        trace.checkpoint(ExecutionSnapshot {
            bindings: &self.bindings,
            value,
        });
    }

    fn evaluate_expression(
        &self,
        source: &SourceText,
        expression: &Expression,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        match expression {
            Expression::Boolean(span) => Ok(evaluate_boolean_literal(source, *span, trace)),
            Expression::Unit(_) => {
                trace.record(TraceEvent {
                    event: "product.unit",
                    rule: "TOPAL-TYPE-PRODUCT-001",
                    detail: "Tuple()",
                });
                Ok(Value::Unit)
            }
            Expression::Tuple { items, .. } => {
                let values = items
                    .iter()
                    .map(|item| self.evaluate_expression(source, item, trace))
                    .collect::<Result<Vec<_>, _>>()?;
                let detail = format!("fields={}", values.len());
                trace.record(TraceEvent {
                    event: "product.tuple",
                    rule: "TOPAL-TYPE-PRODUCT-001",
                    detail: &detail,
                });
                Ok(Value::Tuple(values))
            }
            Expression::Integer(span) => evaluate_integer_literal(source, *span, trace),
            Expression::Rational(span) => evaluate_rational_literal(source, *span, trace),
            Expression::String(span) => evaluate_string_literal(source, *span, trace),
            Expression::Identifier(span) => {
                let name = source.slice(*span);
                let value = self.bindings.get(name).cloned().ok_or_else(|| {
                    let error = diagnostic(source, "E-UNBOUND-NAME", *span, "name is not bound");
                    closest_name(name, self.bindings.keys()).map_or(error.clone(), |candidate| {
                        error.with_help(format!("did you mean `{candidate}`?"))
                    })
                })?;
                trace.record(TraceEvent {
                    event: "binding.resolved",
                    rule: "TOPAL-SYN-BIND-001",
                    detail: name,
                });
                Ok(value)
            }
            Expression::Callable { span, .. } => Err(diagnostic(
                source,
                "E-UNSUPPORTED-CALLABLE-VALUE",
                *span,
                "callable values are not yet executable in isolation",
            )),
            Expression::Application { items, span } => {
                let (mut result, mut index) = if matches!(
                    items.first(),
                    Some(Expression::Callable {
                        kind: CallableKind::Minus,
                        ..
                    })
                ) {
                    let Some(operand) = items.get(1) else {
                        return Err(diagnostic(
                            source,
                            "E-EXPECTED-OPERAND",
                            *span,
                            "expected an operand after prefix -",
                        ));
                    };
                    let operand = self.evaluate_expression(source, operand, trace)?;
                    (apply_negate(source, operand, *span, trace)?, 2)
                } else {
                    (self.evaluate_expression(source, &items[0], trace)?, 1)
                };
                while index < items.len() {
                    let Expression::Callable {
                        kind,
                        span: operator_span,
                    } = &items[index]
                    else {
                        return Err(diagnostic(
                            source,
                            "E-UNSUPPORTED-APPLICATION",
                            items[index].span(),
                            "the implemented subset requires a symbolic callable",
                        ));
                    };
                    let Some(right) = items.get(index + 1) else {
                        return Err(diagnostic(
                            source,
                            "E-EXPECTED-OPERAND",
                            Span::new(operator_span.end, operator_span.end),
                            "expected an operand after callable",
                        ));
                    };
                    let right_span = right.span();
                    let right = self.evaluate_expression(source, right, trace)?;
                    result = apply_binary(source, *kind, result, right, *span, right_span, trace)?;
                    index += 2;
                }
                Ok(result)
            }
        }
    }
}

fn evaluate_boolean_literal(source: &SourceText, span: Span, trace: &mut impl TraceSink) -> Value {
    let lexeme = source.slice(span);
    trace.record(TraceEvent {
        event: "token.boolean",
        rule: "TOPAL-TYPE-BOOLEAN-001",
        detail: lexeme,
    });
    Value::Boolean(lexeme == "true")
}

fn evaluate_integer_literal(
    source: &SourceText,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let text = source.slice(span);
    let value = parse_integer(text)
        .ok_or_else(|| diagnostic(source, "E-NUMERIC-LITERAL", span, "invalid integer literal"))?;
    trace.record(TraceEvent {
        event: "token.integer",
        rule: "TOPAL-NUM-LITERAL-001",
        detail: text,
    });
    Ok(Value::Int(value))
}

fn evaluate_rational_literal(
    source: &SourceText,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let text = source.slice(span);
    let value = parse_rational(text).ok_or_else(|| {
        diagnostic(
            source,
            "E-NUMERIC-LITERAL",
            span,
            "invalid rational literal",
        )
    })?;
    trace.record(TraceEvent {
        event: "token.rational",
        rule: "TOPAL-NUM-RATIONAL-LITERAL-001",
        detail: text,
    });
    Ok(Value::Rational(value))
}

fn evaluate_string_literal(
    source: &SourceText,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let lexeme = source.slice(span);
    let value = parse_string(lexeme).ok_or_else(|| {
        diagnostic(
            source,
            "E-STRING-LITERAL",
            span,
            "invalid string literal delimiter",
        )
    })?;
    trace.record(TraceEvent {
        event: "token.string",
        rule: "TOPAL-SYN-STRING-001",
        detail: lexeme,
    });
    Ok(Value::String(value.to_owned()))
}

fn parse_string(lexeme: &str) -> Option<&str> {
    let opening = lexeme.find('"')?;
    let closing_length = opening + 1;
    (lexeme.len() >= opening + 1 + closing_length)
        .then(|| &lexeme[opening + 1..lexeme.len() - closing_length])
}

fn display_string(value: &str) -> String {
    if !value.contains('"') {
        return format!("\"{value}\"");
    }
    let mut tag = "text".to_owned();
    while value.contains(&format!("\"{tag}")) {
        tag.push('_');
    }
    format!("{tag}\"{value}\"{tag}")
}

fn apply_binary(
    source: &SourceText,
    kind: CallableKind,
    left: Value,
    right: Value,
    span: Span,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if matches!(kind, CallableKind::Equal | CallableKind::NotEqual) {
        return apply_equality(source, kind, left, right, span, trace);
    }
    if matches!(
        kind,
        CallableKind::Less
            | CallableKind::Greater
            | CallableKind::LessEqual
            | CallableKind::GreaterEqual
    ) {
        return apply_comparison(source, kind, left, right, span, trace);
    }
    match (left, right) {
        (Value::Int(left), Value::Int(right)) => {
            apply_int_binary(source, kind, left, right, right_span, trace)
        }
        (Value::Rational(left), Value::Rational(right)) => {
            apply_rational_binary(source, kind, left, right, span, right_span, trace)
        }
        (Value::Int(left), Value::Rational(right)) if kind != CallableKind::Power => {
            trace_conversion(trace, "Int->Rational:left");
            apply_rational_binary(
                source,
                kind,
                BigRational::from_integer(left),
                right,
                span,
                right_span,
                trace,
            )
        }
        (Value::Rational(left), Value::Int(right)) if kind != CallableKind::Power => {
            trace_conversion(trace, "Int->Rational:right");
            apply_rational_binary(
                source,
                kind,
                left,
                BigRational::from_integer(right),
                span,
                right_span,
                trace,
            )
        }
        _ => Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            span,
            "the implemented subset requires operands from one exact numeric domain",
        )),
    }
}

fn apply_comparison(
    source: &SourceText,
    kind: CallableKind,
    left: Value,
    right: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let tuple = matches!((&left, &right), (Value::Tuple(_), Value::Tuple(_)));
    let Some(ordering) = values_compare(left, right, trace) else {
        return Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            span,
            "ordering requires operands with shared TotalOrder evidence",
        ));
    };
    let (callable, result) = match kind {
        CallableKind::Less => ("root.<(TotalOrder,TotalOrder)", ordering == Ordering::Less),
        CallableKind::Greater => (
            "root.>(TotalOrder,TotalOrder)",
            ordering == Ordering::Greater,
        ),
        CallableKind::LessEqual => (
            "root.<=(TotalOrder,TotalOrder)",
            ordering != Ordering::Greater,
        ),
        CallableKind::GreaterEqual => {
            ("root.>=(TotalOrder,TotalOrder)", ordering != Ordering::Less)
        }
        _ => unreachable!("comparison dispatch accepts only ordering predicates"),
    };
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: callable,
    });
    trace.record(TraceEvent {
        event: "comparison.result",
        rule: if tuple {
            "TOPAL-TYPE-ORDERING-001"
        } else {
            "TOPAL-NUM-COMPARE-001"
        },
        detail: match ordering {
            Ordering::Less => "Less",
            Ordering::Equal => "Equal",
            Ordering::Greater => "Greater",
        },
    });
    Ok(Value::Boolean(result))
}

fn values_compare(left: Value, right: Value, trace: &mut impl TraceSink) -> Option<Ordering> {
    match (left, right) {
        (Value::Int(left), Value::Int(right)) => Some(left.cmp(&right)),
        (Value::Rational(left), Value::Rational(right)) => Some(left.cmp(&right)),
        (Value::Int(left), Value::Rational(right)) => {
            trace_conversion(trace, "Int->Rational:left");
            Some(BigRational::from_integer(left).cmp(&right))
        }
        (Value::Rational(left), Value::Int(right)) => {
            trace_conversion(trace, "Int->Rational:right");
            Some(left.cmp(&BigRational::from_integer(right)))
        }
        (Value::Tuple(left), Value::Tuple(right)) if left.len() == right.len() => {
            for (left, right) in left.into_iter().zip(right) {
                let ordering = values_compare(left, right, trace)?;
                if ordering != Ordering::Equal {
                    return Some(ordering);
                }
            }
            Some(Ordering::Equal)
        }
        _ => None,
    }
}

fn apply_equality(
    source: &SourceText,
    kind: CallableKind,
    left: Value,
    right: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Some(equal) = values_equal(left, right, trace) else {
        return Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            span,
            "the operand types do not share an applicable Equality operation",
        ));
    };
    let equal = if kind == CallableKind::NotEqual {
        !equal
    } else {
        equal
    };
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: if kind == CallableKind::NotEqual {
            "root.!=(Equality,Equality)"
        } else {
            "root.=(Equality,Equality)"
        },
    });
    trace.record(TraceEvent {
        event: "evaluation.equal",
        rule: "TOPAL-TYPE-EQUALITY-001",
        detail: if equal { "true" } else { "false" },
    });
    Ok(Value::Boolean(equal))
}

fn values_equal(left: Value, right: Value, trace: &mut impl TraceSink) -> Option<bool> {
    match (left, right) {
        (Value::Boolean(left), Value::Boolean(right)) => Some(left == right),
        (Value::Int(left), Value::Int(right)) => Some(left == right),
        (Value::Rational(left), Value::Rational(right)) => Some(left == right),
        (Value::Int(left), Value::Rational(right)) => {
            trace_conversion(trace, "Int->Rational:left");
            Some(BigRational::from_integer(left) == right)
        }
        (Value::Rational(left), Value::Int(right)) => {
            trace_conversion(trace, "Int->Rational:right");
            Some(left == BigRational::from_integer(right))
        }
        (Value::String(left), Value::String(right)) => Some(left == right),
        (Value::Unit, Value::Unit) => Some(true),
        (Value::Tuple(left), Value::Tuple(right)) if left.len() == right.len() => left
            .into_iter()
            .zip(right)
            .try_fold(true, |equal, (left, right)| {
                values_equal(left, right, trace).map(|field_equal| equal && field_equal)
            }),
        _ => None,
    }
}

fn trace_conversion(trace: &mut impl TraceSink, detail: &'static str) {
    trace.record(TraceEvent {
        event: "conversion.applied",
        rule: "TOPAL-TYPE-CONVERT-001",
        detail,
    });
}

fn apply_int_binary(
    source: &SourceText,
    kind: CallableKind,
    left: BigInt,
    right: BigInt,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    match kind {
        CallableKind::Equal
        | CallableKind::NotEqual
        | CallableKind::Less
        | CallableKind::Greater
        | CallableKind::LessEqual
        | CallableKind::GreaterEqual => {
            unreachable!("comparison is dispatched before numeric operations")
        }
        CallableKind::Plus => {
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
        CallableKind::Minus => {
            trace.record(TraceEvent {
                event: "operator.selected",
                rule: "TOPAL-TYPE-CALL-001",
                detail: "root.-(Int,Int)",
            });
            trace.record(TraceEvent {
                event: "evaluation.subtract",
                rule: "TOPAL-NUM-SUB-001",
                detail: "Int",
            });
            Ok(Value::Int(left - right))
        }
        CallableKind::Multiply => {
            trace.record(TraceEvent {
                event: "operator.selected",
                rule: "TOPAL-TYPE-CALL-001",
                detail: "root.*(Int,Int)",
            });
            trace.record(TraceEvent {
                event: "evaluation.multiply",
                rule: "TOPAL-NUM-MUL-001",
                detail: "Int",
            });
            Ok(Value::Int(left * right))
        }
        CallableKind::Divide => apply_divide(source, left, right, right_span, trace),
        CallableKind::Power => apply_power(source, left, right, right_span, trace),
    }
}

fn apply_rational_binary(
    source: &SourceText,
    kind: CallableKind,
    left: BigRational,
    right: BigRational,
    span: Span,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let (callable, event, rule, result) = match kind {
        CallableKind::Equal
        | CallableKind::NotEqual
        | CallableKind::Less
        | CallableKind::Greater
        | CallableKind::LessEqual
        | CallableKind::GreaterEqual => {
            unreachable!("comparison is dispatched before numeric operations")
        }
        CallableKind::Plus => (
            "root.+(Rational,Rational)",
            "evaluation.add",
            "TOPAL-NUM-RAT-ADD-001",
            left + right,
        ),
        CallableKind::Minus => (
            "root.-(Rational,Rational)",
            "evaluation.subtract",
            "TOPAL-NUM-RAT-SUB-001",
            left - right,
        ),
        CallableKind::Multiply => (
            "root.*(Rational,Rational)",
            "evaluation.multiply",
            "TOPAL-NUM-RAT-MUL-001",
            left * right,
        ),
        CallableKind::Divide => {
            if right.numer() == &BigInt::from(0) {
                trace.record(TraceEvent {
                    event: "obligation.refuted",
                    rule: "TOPAL-NUM-DIVZERO-001",
                    detail: "divisor.nonzero",
                });
                return Err(diagnostic(
                    source,
                    "E-DIVISION-BY-ZERO",
                    right_span,
                    "statically evident division by zero",
                ));
            }
            trace.record(TraceEvent {
                event: "obligation.proved",
                rule: "TOPAL-NUM-DIVZERO-001",
                detail: "divisor.nonzero",
            });
            (
                "root./(Rational,Rational)",
                "evaluation.divide",
                "TOPAL-NUM-RAT-DIV-001",
                left / right,
            )
        }
        CallableKind::Power => {
            return Err(diagnostic(
                source,
                "E-NO-APPLICABLE-OVERLOAD",
                span,
                "Rational exponentiation is not in the implemented subset",
            ));
        }
    };
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: callable,
    });
    trace.record(TraceEvent {
        event,
        rule,
        detail: "Rational",
    });
    Ok(Value::Rational(result))
}

fn apply_divide(
    source: &SourceText,
    left: BigInt,
    right: BigInt,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if right == BigInt::from(0) {
        trace.record(TraceEvent {
            event: "obligation.refuted",
            rule: "TOPAL-NUM-DIVZERO-001",
            detail: "divisor.nonzero",
        });
        return Err(diagnostic(
            source,
            "E-DIVISION-BY-ZERO",
            right_span,
            "statically evident division by zero",
        ));
    }
    trace.record(TraceEvent {
        event: "obligation.proved",
        rule: "TOPAL-NUM-DIVZERO-001",
        detail: "divisor.nonzero",
    });
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: "root./(Int,Int)",
    });
    trace.record(TraceEvent {
        event: "evaluation.divide",
        rule: "TOPAL-NUM-DIV-001",
        detail: "Rational",
    });
    Ok(Value::Rational(BigRational::new(left, right)))
}

fn apply_power(
    source: &SourceText,
    left: BigInt,
    right: BigInt,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if right < BigInt::from(0) {
        trace.record(TraceEvent {
            event: "obligation.refuted",
            rule: "TOPAL-NUM-POW-001",
            detail: "exponent.finite-nat",
        });
        return Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            right_span,
            "Int exponentiation requires a finite Nat exponent",
        ));
    }
    trace.record(TraceEvent {
        event: "obligation.proved",
        rule: "TOPAL-NUM-POW-001",
        detail: "exponent.finite-nat",
    });
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: "root.^(Int,Nat)",
    });
    trace.record(TraceEvent {
        event: "evaluation.power",
        rule: "TOPAL-NUM-POW-001",
        detail: "Int",
    });
    Ok(Value::Int(pow_int(left, right)))
}

fn pow_int(mut base: BigInt, mut exponent: BigInt) -> BigInt {
    let zero = BigInt::from(0);
    let one = BigInt::from(1);
    let two = BigInt::from(2);
    let mut result = one.clone();
    while exponent > zero {
        if &exponent % &two == one {
            result *= &base;
        }
        exponent /= &two;
        if exponent > zero {
            base = &base * &base;
        }
    }
    result
}

fn apply_negate(
    source: &SourceText,
    operand: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    match operand {
        Value::Int(operand) => {
            trace.record(TraceEvent {
                event: "operator.selected",
                rule: "TOPAL-TYPE-CALL-001",
                detail: "root.-(Int)",
            });
            trace.record(TraceEvent {
                event: "evaluation.negate",
                rule: "TOPAL-NUM-NEG-001",
                detail: "Int",
            });
            Ok(Value::Int(-operand))
        }
        Value::Rational(operand) => {
            trace.record(TraceEvent {
                event: "operator.selected",
                rule: "TOPAL-TYPE-CALL-001",
                detail: "root.-(Rational)",
            });
            trace.record(TraceEvent {
                event: "evaluation.negate",
                rule: "TOPAL-NUM-RAT-NEG-001",
                detail: "Rational",
            });
            Ok(Value::Rational(-operand))
        }
        Value::Boolean(_) | Value::String(_) | Value::Tuple(_) | Value::Unit => Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            span,
            "prefix - requires an exact numeric operand",
        )),
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
        source_line: source
            .as_str()
            .lines()
            .nth(position.line - 1)
            .map(str::to_owned),
        marker_width: marker_width(source.as_str(), span),
        help: diagnostic_help(code).map(str::to_owned),
    }
}

fn closest_name<'a>(name: &str, candidates: impl Iterator<Item = &'a String>) -> Option<&'a str> {
    let maximum = 2.max(name.chars().count() / 3);
    candidates
        .map(|candidate| (edit_distance(name, candidate), candidate.as_str()))
        .filter(|(distance, _)| *distance <= maximum)
        .min()
        .map(|(_, candidate)| candidate)
}

fn edit_distance(left: &str, right: &str) -> usize {
    let right = right.chars().collect::<Vec<_>>();
    let mut previous = (0..=right.len()).collect::<Vec<_>>();
    for (left_index, left_character) in left.chars().enumerate() {
        let mut current = vec![left_index + 1];
        for (right_index, right_character) in right.iter().enumerate() {
            current.push(
                (current[right_index] + 1)
                    .min(previous[right_index + 1] + 1)
                    .min(previous[right_index] + usize::from(left_character != *right_character)),
            );
        }
        previous = current;
    }
    previous[right.len()]
}

fn diagnostic_help(code: &str) -> Option<&'static str> {
    match code {
        "E-UNKNOWN-TOKEN" => Some("remove this character or use a symbol declared by design-0"),
        "E-UNBOUND-NAME" => Some("declare this name earlier in the same source session"),
        "E-EXPECTED-RPAREN" => Some("add a closing `)` for this parenthesized expression"),
        "E-UNTERMINATED-STRING" => Some("add the literal's matching closing quote and tag"),
        "E-DIVISION-BY-ZERO" => Some("use a divisor that is provably nonzero"),
        "E-NO-APPLICABLE-OVERLOAD" => {
            Some("use operands supported by one overload or apply an explicit conversion")
        }
        "E-RESERVED-BOOLEAN-LITERAL" => {
            Some("choose an identifier other than the reserved literals `true` and `false`")
        }
        _ => None,
    }
}

fn raw_source_line(source: &str, line: usize) -> Option<String> {
    source.lines().nth(line - 1).map(str::to_owned)
}

fn marker_width(source: &str, span: Span) -> usize {
    source
        .get(span.start..span.end)
        .unwrap_or("")
        .split(['\r', '\n'])
        .next()
        .unwrap_or("")
        .chars()
        .count()
        .max(1)
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

fn parse_rational(token: &str) -> Option<BigRational> {
    if let Some(unsigned) = token.strip_prefix('-') {
        return parse_unsigned_rational(unsigned).map(std::ops::Neg::neg);
    }
    parse_unsigned_rational(token)
}

fn parse_unsigned_rational(token: &str) -> Option<BigRational> {
    let (mantissa, exponent) = if let Some(offset) = token.find(['e', 'E']) {
        (
            &token[..offset],
            parse_signed_decimal_integer(&token[offset + 1..])?,
        )
    } else {
        (token, BigInt::from(0))
    };
    let (integer, fractional) = mantissa
        .split_once('.')
        .map_or((mantissa, ""), |parts| parts);
    if !valid_decimal_integer(integer) || (!fractional.is_empty() && !valid_fractional(fractional))
    {
        return None;
    }
    if fractional.is_empty() && !token.contains(['e', 'E']) {
        return None;
    }
    let integer_digits = integer.replace('_', "");
    let fractional_digits = fractional.replace('_', "");
    let numerator = format!("{integer_digits}{fractional_digits}")
        .parse::<BigInt>()
        .ok()?;
    let scale = BigInt::from(fractional_digits.len()) - exponent;
    if scale >= BigInt::from(0) {
        Some(BigRational::new(
            numerator,
            pow_int(BigInt::from(10), scale),
        ))
    } else {
        Some(BigRational::from_integer(
            numerator * pow_int(BigInt::from(10), -scale),
        ))
    }
}

fn parse_signed_decimal_integer(token: &str) -> Option<BigInt> {
    let (negative, unsigned) = token
        .strip_prefix('-')
        .map_or((false, token), |value| (true, value));
    let unsigned = unsigned.strip_prefix('+').unwrap_or(unsigned);
    if !valid_decimal_integer(unsigned) {
        return None;
    }
    let value = unsigned.replace('_', "").parse::<BigInt>().ok()?;
    Some(if negative { -value } else { value })
}

fn valid_fractional(token: &str) -> bool {
    if !token.contains('_') {
        return token.bytes().all(|byte| byte.is_ascii_digit());
    }
    let groups = token.split('_').collect::<Vec<_>>();
    groups
        .first()
        .is_some_and(|group| group.len() == 3 && group.bytes().all(|byte| byte.is_ascii_digit()))
        && groups.iter().skip(1).enumerate().all(|(index, group)| {
            let final_group = index + 2 == groups.len();
            (group.len() == 3 || (final_group && (1..=2).contains(&group.len())))
                && group.bytes().all(|byte| byte.is_ascii_digit())
        })
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
    fn negates_and_subtracts_exact_integers() {
        assert_eq!(evaluate("- 42").unwrap().to_string(), "-42");
        assert_eq!(evaluate("10 - 3 - 2").unwrap().to_string(), "5");
        assert_eq!(evaluate("10 - -2").unwrap().to_string(), "12");
    }

    #[test]
    fn multiplies_without_hidden_precedence() {
        assert_eq!(evaluate("2 + 3 * 4").unwrap().to_string(), "20");
        assert_eq!(evaluate("2 + (3 * 4)").unwrap().to_string(), "14");
        assert_eq!(
            evaluate("99999999999999999999 * 99999999999999999999")
                .unwrap()
                .to_string(),
            "9999999999999999999800000000000000000001"
        );
    }

    #[test]
    fn divides_to_canonical_rational() {
        assert_eq!(evaluate("6 / 8").unwrap().to_string(), "Rational ( 3, 4 )");
        assert_eq!(
            evaluate("6 / -8").unwrap().to_string(),
            "Rational ( -3, 4 )"
        );
        assert_eq!(evaluate("6 / 3").unwrap().to_string(), "Rational ( 2, 1 )");
    }

    #[test]
    fn rejects_statically_evident_zero_divisor() {
        assert_eq!(evaluate("1 / 0").unwrap_err().code, "E-DIVISION-BY-ZERO");
    }

    #[test]
    fn renders_unicode_aligned_actionable_diagnostics() {
        let error = evaluate("α is 1\nα + missing").unwrap_err();
        assert_eq!(
            error.render("example.t"),
            "error[E-UNBOUND-NAME]: name is not bound\n --> example.t:2:5\n  |\n2 | α + missing\n  |     ^^^^^^^\n  |\n  = help: declare this name earlier in the same source session"
        );
    }

    #[test]
    fn edit_distance_counts_unicode_scalars() {
        assert_eq!(edit_distance("räknare", "räknaren"), 1);
        assert_eq!(edit_distance("αβ", "βα"), 2);
    }

    #[test]
    fn raises_integer_to_natural_power_exactly() {
        assert_eq!(
            evaluate("2 ^ 100").unwrap().to_string(),
            "1267650600228229401496703205376"
        );
        assert_eq!(evaluate("0 ^ 0").unwrap().to_string(), "1");
        assert_eq!(evaluate("-2 ^ 3").unwrap().to_string(), "-8");
    }

    #[test]
    fn exponentiation_uses_ordinary_left_association() {
        assert_eq!(evaluate("2 + 3 ^ 2").unwrap().to_string(), "25");
        assert_eq!(evaluate("2 + (3 ^ 2)").unwrap().to_string(), "11");
    }

    #[test]
    fn rejects_negative_integer_exponent() {
        assert_eq!(
            evaluate("2 ^ -1").unwrap_err().code,
            "E-NO-APPLICABLE-OVERLOAD"
        );
    }

    #[test]
    fn constructs_exact_rational_literals() {
        assert_eq!(evaluate("0.1").unwrap().to_string(), "Rational ( 1, 10 )");
        assert_eq!(
            evaluate("1.25e3").unwrap().to_string(),
            "Rational ( 1250, 1 )"
        );
        assert_eq!(
            evaluate("-6.022e-24").unwrap().to_string(),
            "Rational ( -3011, 500000000000000000000000000 )"
        );
        assert_eq!(
            evaluate("1_000.000_125").unwrap().to_string(),
            "Rational ( 8000001, 8000 )"
        );
    }

    #[test]
    fn rejects_malformed_rational_literal() {
        assert_eq!(evaluate("1.2e").unwrap_err().code, "E-NUMERIC-LITERAL");
    }

    #[test]
    fn evaluates_exact_rational_arithmetic() {
        assert_eq!(
            evaluate("0.5 + 0.25").unwrap().to_string(),
            "Rational ( 3, 4 )"
        );
        assert_eq!(
            evaluate("- 1.5 - 0.25").unwrap().to_string(),
            "Rational ( -7, 4 )"
        );
        assert_eq!(
            evaluate("1.5 * 0.5").unwrap().to_string(),
            "Rational ( 3, 4 )"
        );
        assert_eq!(
            evaluate("1.5 / 0.25").unwrap().to_string(),
            "Rational ( 6, 1 )"
        );
    }

    #[test]
    fn converts_int_for_mixed_exact_arithmetic() {
        assert_eq!(
            evaluate("1 + 0.5").unwrap().to_string(),
            "Rational ( 3, 2 )"
        );
        assert_eq!(
            evaluate("0.5 * 2").unwrap().to_string(),
            "Rational ( 1, 1 )"
        );
        assert_eq!(
            evaluate("1 / 0.5").unwrap().to_string(),
            "Rational ( 2, 1 )"
        );
    }

    #[test]
    fn does_not_promote_natural_exponent_position() {
        assert_eq!(
            evaluate("2.0 ^ 2").unwrap_err().code,
            "E-NO-APPLICABLE-OVERLOAD"
        );
    }

    #[test]
    fn rejects_rational_zero_divisor() {
        assert_eq!(
            evaluate("1.0 / 0.0").unwrap_err().code,
            "E-DIVISION-BY-ZERO"
        );
    }

    #[test]
    fn preserves_ordinary_and_tagged_string_contents() {
        assert_eq!(
            evaluate(r#""plain\n{value}""#).unwrap().to_string(),
            r#""plain\n{value}""#
        );
        assert_eq!(
            evaluate(r#"text"He said "hello"."text"#)
                .unwrap()
                .to_string(),
            r#"text"He said "hello"."text"#
        );
        assert_eq!(
            evaluate("\"first\nsecond\"").unwrap().to_string(),
            "\"first\nsecond\""
        );
    }

    #[test]
    fn display_extends_colliding_string_tag() {
        assert_eq!(
            evaluate(r#"tag"contains "text closing"tag"#)
                .unwrap()
                .to_string(),
            r#"text_"contains "text closing"text_"#
        );
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
