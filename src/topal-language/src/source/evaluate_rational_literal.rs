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

pub(crate) fn parse_string(lexeme: &str) -> Option<&str> {
    let opening = lexeme.find('"')?;
    let closing_length = opening + 1;
    (lexeme.len() >= opening + 1 + closing_length)
        .then(|| &lexeme[opening + 1..lexeme.len() - closing_length])
}

#[must_use]
pub fn display_string_literal(value: &str) -> String {
    if !value.contains('"') {
        return format!("\"{value}\"");
    }
    let mut tag = "text".to_owned();
    while value.contains(&format!("\"{tag}")) {
        tag.push('_');
    }
    format!("{tag}\"{value}\"{tag}")
}

#[allow(clippy::too_many_lines)] // Numeric domains keep explicit, non-coercing dispatch arms.
fn apply_binary(
    source: &SourceText,
    kind: CallableKind,
    left: Value,
    right: Value,
    spans: (Span, Span, Span),
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let (span, left_span, right_span) = spans;
    let left = forget_refinement(left, trace, "constraint->base:left");
    let right = forget_refinement(right, trace, "constraint->base:right");
    if matches!(kind, CallableKind::Equal | CallableKind::NotEqual) {
        return apply_equality(source, kind, left, right, span, trace);
    }
    if kind == CallableKind::Compare {
        let Some(ordering) = values_compare(left, right, trace) else {
            return Err(diagnostic(
                source,
                "E-NO-APPLICABLE-OVERLOAD",
                span,
                "the operand types do not share an applicable TotalOrder",
            ));
        };
        let alternative = match ordering {
            Ordering::Less => "Less",
            Ordering::Equal => "Equal",
            Ordering::Greater => "Greater",
        };
        trace.record(TraceEvent {
            event: "operator.selected",
            rule: "TOPAL-TYPE-CALL-001",
            detail: "root.<=>(TotalOrder,TotalOrder)",
        });
        trace.record(TraceEvent {
            event: "comparison.result",
            rule: "TOPAL-NUM-THREE-WAY-COMPARE-001",
            detail: alternative,
        });
        return Ok(Value::Enum {
            type_name: "Comparison".to_owned(),
            alternative: alternative.to_owned(),
        });
    }
    if matches!(
        kind,
        CallableKind::Range
            | CallableKind::RangeOpen
            | CallableKind::RangeInclusive
            | CallableKind::RangeOpenInclusive
    ) {
        return apply_range(source, kind, left, right, span, trace);
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
    if let Some(result) = apply_infinity_binary(
        source,
        kind,
        &left,
        &right,
        span,
        (left_span, right_span),
        trace,
    ) {
        return result;
    }
    match (left, right) {
        (
            Value::Modular {
                type_name: left_type,
                lower,
                upper,
                value: left,
            },
            Value::Modular {
                type_name: right_type,
                value: right,
                ..
            },
        ) if left_type == right_type => {
            let raw = match kind {
                CallableKind::Plus => left + right,
                CallableKind::Minus => left - right,
                CallableKind::Multiply => left * right,
                _ => {
                    return Err(diagnostic(
                        source,
                        "E-NO-APPLICABLE-OVERLOAD",
                        span,
                        "modular values support settled wrapping +, -, and * operations",
                    ));
                }
            };
            let value = reduce_modular(raw, &lower, &upper);
            trace.record(TraceEvent {
                event: "numeric.modular.wrapped",
                rule: "TOPAL-NUM-MODULAR-ARITHMETIC-001",
                detail: &left_type,
            });
            Ok(Value::Modular {
                type_name: left_type,
                lower,
                upper,
                value,
            })
        }
        (Value::Int(left), Value::Int(right)) => {
            apply_int_binary(source, kind, left, right, right_span, trace)
        }
        (Value::Rational(left), Value::Rational(right)) => {
            apply_rational_binary(source, kind, left, right, span, right_span, trace)
        }
        (Value::Rational(left), Value::Int(right)) if kind == CallableKind::Power => {
            apply_rational_power(source, left, right, left_span, right_span, trace)
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

fn reduce_modular(value: BigInt, lower: &BigInt, upper: &BigInt) -> BigInt {
    let modulus = upper - lower + BigInt::from(1);
    let offset = value - lower;
    let reduced = ((offset % &modulus) + &modulus) % &modulus;
    reduced + lower
}

fn forget_refinement(value: Value, trace: &mut impl TraceSink, detail: &'static str) -> Value {
    if let Value::Refined { value, .. } = value {
        trace_conversion(trace, detail);
        *value
    } else {
        value
    }
}

fn extended_int(value: &Value) -> Option<ExtendedInt> {
    match value {
        Value::Int(value) => Some(ExtendedInt::Finite(value.clone())),
        Value::Infinity {
            negative: true,
            classifier,
        } if classifier != "Rational" => Some(ExtendedInt::NegativeInfinity),
        Value::Infinity {
            negative: false,
            classifier,
        } if classifier != "Rational" => Some(ExtendedInt::PositiveInfinity),
        _ => None,
    }
}

fn extended_rational(value: &Value) -> Option<ExtendedRational> {
    match value {
        Value::Rational(value) => Some(ExtendedRational::Finite(value.clone())),
        Value::Infinity {
            negative: true,
            classifier,
        } if classifier == "Rational" => Some(ExtendedRational::NegativeInfinity),
        Value::Infinity {
            negative: false,
            classifier,
        } if classifier == "Rational" => Some(ExtendedRational::PositiveInfinity),
        _ => None,
    }
}

fn exact_to_extended_rational(value: &Value) -> Option<ExtendedRational> {
    extended_rational(value).or_else(|| {
        let Value::Int(value) = value else {
            return None;
        };
        Some(ExtendedRational::Finite(BigRational::from_integer(
            value.clone(),
        )))
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InfinityArithmeticOperand {
    NegativeInfinity,
    Finite(Ordering),
    PositiveInfinity,
}

fn extended_int_arithmetic_operand(value: &Value) -> Option<InfinityArithmeticOperand> {
    match extended_int(value)? {
        ExtendedInt::NegativeInfinity => Some(InfinityArithmeticOperand::NegativeInfinity),
        ExtendedInt::Finite(value) => Some(InfinityArithmeticOperand::Finite(
            value.cmp(&BigInt::from(0)),
        )),
        ExtendedInt::PositiveInfinity => Some(InfinityArithmeticOperand::PositiveInfinity),
    }
}

fn extended_rational_arithmetic_operand(value: &Value) -> Option<InfinityArithmeticOperand> {
    match exact_to_extended_rational(value)? {
        ExtendedRational::NegativeInfinity => Some(InfinityArithmeticOperand::NegativeInfinity),
        ExtendedRational::Finite(value) => Some(InfinityArithmeticOperand::Finite(
            value.cmp(&BigRational::from_integer(BigInt::from(0))),
        )),
        ExtendedRational::PositiveInfinity => Some(InfinityArithmeticOperand::PositiveInfinity),
    }
}

fn infinity_arithmetic_direction(
    kind: CallableKind,
    left: InfinityArithmeticOperand,
    right: InfinityArithmeticOperand,
) -> Option<bool> {
    use InfinityArithmeticOperand::{Finite, NegativeInfinity, PositiveInfinity};
    match kind {
        CallableKind::Plus => match (left, right) {
            (NegativeInfinity, PositiveInfinity) | (PositiveInfinity, NegativeInfinity) => None,
            (NegativeInfinity, _) | (_, NegativeInfinity) => Some(true),
            (PositiveInfinity, _) | (_, PositiveInfinity) => Some(false),
            (Finite(_), Finite(_)) => unreachable!("infinity arithmetic has an infinite operand"),
        },
        CallableKind::Minus => match (left, right) {
            (NegativeInfinity, NegativeInfinity) | (PositiveInfinity, PositiveInfinity) => None,
            (NegativeInfinity, _) | (_, PositiveInfinity) => Some(true),
            (PositiveInfinity, _) | (_, NegativeInfinity) => Some(false),
            (Finite(_), Finite(_)) => unreachable!("infinity arithmetic has an infinite operand"),
        },
        CallableKind::Multiply => {
            let negative = |operand| match operand {
                NegativeInfinity | Finite(Ordering::Less) => Some(true),
                PositiveInfinity | Finite(Ordering::Greater) => Some(false),
                Finite(Ordering::Equal) => None,
            };
            Some(negative(left)? ^ negative(right)?)
        }
        _ => None,
    }
}

fn infinity_operator_selection(kind: CallableKind, rational: bool) -> &'static str {
    match (kind, rational) {
        (CallableKind::Plus, false) => "root.+(Int,Int)",
        (CallableKind::Minus, false) => "root.-(Int,Int)",
        (CallableKind::Multiply, false) => "root.*(Int,Int)",
        (CallableKind::Plus, true) => "root.+(Rational,Rational)",
        (CallableKind::Minus, true) => "root.-(Rational,Rational)",
        (CallableKind::Multiply, true) => "root.*(Rational,Rational)",
        _ => unreachable!("selection is requested only for admitted infinity arithmetic"),
    }
}

fn closed_numeric_literal(source: &SourceText, span: Span) -> bool {
    let lexeme = source.slice(span);
    if parse_integer(lexeme).is_some() || parse_rational(lexeme).is_some() {
        return true;
    }
    let Some(operand) = lexeme.strip_prefix("Rational").map(str::trim) else {
        return false;
    };
    if parse_integer(operand).is_some() {
        return true;
    }
    let Some(components) = operand
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
    else {
        return false;
    };
    let Some((numerator, denominator)) = components.split_once(',') else {
        return false;
    };
    parse_integer(numerator.trim()).is_some()
        && parse_integer(denominator.trim()).is_some_and(|value| value != BigInt::from(0))
}

fn dynamic_infinity_zero_span(
    source: &SourceText,
    kind: CallableKind,
    operands: (InfinityArithmeticOperand, InfinityArithmeticOperand),
    operand_spans: (Span, Span),
) -> Option<Span> {
    if kind != CallableKind::Multiply {
        return None;
    }
    match operands {
        (InfinityArithmeticOperand::Finite(Ordering::Equal), _) => Some(operand_spans.0),
        (_, InfinityArithmeticOperand::Finite(Ordering::Equal)) => Some(operand_spans.1),
        _ => None,
    }
    .filter(|zero_span| !closed_numeric_literal(source, *zero_span))
}

fn dynamic_infinity_error(
    source: &SourceText,
    rational: bool,
    selection: &str,
    zero_span: Span,
    trace: &mut impl TraceSink,
) -> Value {
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: selection,
    });
    trace.record(TraceEvent {
        event: "numeric.infinity.arithmetic",
        rule: "TOPAL-NUM-INFINITY-ARITHMETIC-001",
        detail: callable_name(CallableKind::Multiply),
    });
    trace.record(TraceEvent {
        event: "result.error.constructed",
        rule: "TOPAL-TYPE-RESULT-001",
        detail: if rational {
            "root.*(Rational,Rational);indeterminate"
        } else {
            "root.*(Int,Int);indeterminate"
        },
    });
    let position = source.position(zero_span.start);
    Value::Error {
        domain: selection.to_owned(),
        code: "indeterminate".to_owned(),
        line: position.line,
        column: position.column,
    }
}

fn apply_infinity_binary(
    source: &SourceText,
    kind: CallableKind,
    left: &Value,
    right: &Value,
    span: Span,
    operand_spans: (Span, Span),
    trace: &mut impl TraceSink,
) -> Option<Result<Value, Diagnostic>> {
    if !matches!(left, Value::Infinity { .. }) && !matches!(right, Value::Infinity { .. }) {
        return None;
    }
    if !matches!(
        kind,
        CallableKind::Plus | CallableKind::Minus | CallableKind::Multiply
    ) {
        return Some(Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            span,
            "this exact infinity increment supports only +, -, and * arithmetic",
        )));
    }
    let rational = matches!(left, Value::Rational(_))
        || matches!(right, Value::Rational(_))
        || matches!(left, Value::Infinity { classifier, .. } if classifier == "Rational")
        || matches!(right, Value::Infinity { classifier, .. } if classifier == "Rational");
    let operands = if rational {
        let Some(left) = extended_rational_arithmetic_operand(left) else {
            return Some(Err(diagnostic(
                source,
                "E-NO-APPLICABLE-OVERLOAD",
                span,
                "an Int infinity is not implicitly converted into Rational infinity",
            )));
        };
        let Some(right) = extended_rational_arithmetic_operand(right) else {
            return Some(Err(diagnostic(
                source,
                "E-NO-APPLICABLE-OVERLOAD",
                span,
                "an Int infinity is not implicitly converted into Rational infinity",
            )));
        };
        (left, right)
    } else {
        let (Some(left), Some(right)) = (
            extended_int_arithmetic_operand(left),
            extended_int_arithmetic_operand(right),
        ) else {
            return Some(Err(diagnostic(
                source,
                "E-NO-APPLICABLE-OVERLOAD",
                span,
                "exact infinity arithmetic requires one shared numeric domain",
            )));
        };
        (left, right)
    };
    let selection = infinity_operator_selection(kind, rational);
    let Some(negative) = infinity_arithmetic_direction(kind, operands.0, operands.1) else {
        if let Some(zero_span) = dynamic_infinity_zero_span(source, kind, operands, operand_spans) {
            return Some(Ok(dynamic_infinity_error(
                source, rational, selection, zero_span, trace,
            )));
        }
        return Some(Err(diagnostic(
            source,
            "E-INDETERMINATE-INFINITY",
            span,
            "this infinity arithmetic expression does not determine one exact numeric value",
        )));
    };
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: selection,
    });
    trace.record(TraceEvent {
        event: "numeric.infinity.arithmetic",
        rule: "TOPAL-NUM-INFINITY-ARITHMETIC-001",
        detail: callable_name(kind),
    });
    Some(Ok(Value::Infinity {
        negative,
        classifier: if rational { "Rational" } else { "Int" }.to_owned(),
    }))
}

fn extended_int_value(value: ExtendedInt) -> Value {
    match value {
        ExtendedInt::NegativeInfinity => Value::Infinity {
            negative: true,
            classifier: "Int".into(),
        },
        ExtendedInt::Finite(value) => Value::Int(value),
        ExtendedInt::PositiveInfinity => Value::Infinity {
            negative: false,
            classifier: "Int".into(),
        },
    }
}

fn extended_rational_value(value: ExtendedRational) -> Value {
    match value {
        ExtendedRational::NegativeInfinity => Value::Infinity {
            negative: true,
            classifier: "Rational".into(),
        },
        ExtendedRational::Finite(value) => Value::Rational(value),
        ExtendedRational::PositiveInfinity => Value::Infinity {
            negative: false,
            classifier: "Rational".into(),
        },
    }
}

fn extended_int_range(value: &Value) -> Option<(ExtendedInt, ExtendedInt, bool, bool, bool)> {
    match value {
        Value::IntRange {
            lower,
            upper,
            lower_inclusive,
            upper_inclusive,
        } => Some((
            ExtendedInt::Finite(lower.clone()),
            ExtendedInt::Finite(upper.clone()),
            *lower_inclusive,
            *upper_inclusive,
            false,
        )),
        Value::InfiniteIntRange {
            lower,
            upper,
            lower_inclusive,
            upper_inclusive,
        } => Some((
            lower.clone(),
            upper.clone(),
            *lower_inclusive,
            *upper_inclusive,
            true,
        )),
        _ => None,
    }
}

fn extended_rational_range(
    value: &Value,
) -> Option<(ExtendedRational, ExtendedRational, bool, bool, bool)> {
    match value {
        Value::RationalRange {
            lower,
            upper,
            lower_inclusive,
            upper_inclusive,
        } => Some((
            ExtendedRational::Finite(lower.clone()),
            ExtendedRational::Finite(upper.clone()),
            *lower_inclusive,
            *upper_inclusive,
            false,
        )),
        Value::InfiniteRationalRange {
            lower,
            upper,
            lower_inclusive,
            upper_inclusive,
        } => Some((
            lower.clone(),
            upper.clone(),
            *lower_inclusive,
            *upper_inclusive,
            true,
        )),
        _ => None,
    }
}

fn apply_range(
    source: &SourceText,
    kind: CallableKind,
    left: Value,
    right: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let (lower_inclusive, upper_inclusive) = match kind {
        CallableKind::Range => (true, false),
        CallableKind::RangeOpen => (false, false),
        CallableKind::RangeInclusive => (true, true),
        CallableKind::RangeOpenInclusive => (false, true),
        _ => unreachable!("range operator dispatched with range kind"),
    };
    let (range, nonempty) = match (left, right) {
        (Value::Int(lower), Value::Int(upper)) => {
            let nonempty = lower < upper || (lower == upper && lower_inclusive && upper_inclusive);
            (
                Value::IntRange {
                    lower,
                    upper,
                    lower_inclusive,
                    upper_inclusive,
                },
                nonempty,
            )
        }
        (left, right)
            if extended_int(&left).is_some()
                && extended_int(&right).is_some()
                && (matches!(left, Value::Infinity { .. })
                    || matches!(right, Value::Infinity { .. })) =>
        {
            let lower = extended_int(&left).expect("guard retained an extended Int");
            let upper = extended_int(&right).expect("guard retained an extended Int");
            let nonempty = lower < upper || (lower == upper && lower_inclusive && upper_inclusive);
            (
                Value::InfiniteIntRange {
                    lower,
                    upper,
                    lower_inclusive,
                    upper_inclusive,
                },
                nonempty,
            )
        }
        (left, right)
            if (extended_rational(&left).is_some() || extended_rational(&right).is_some())
                && exact_to_extended_rational(&left).is_some()
                && exact_to_extended_rational(&right).is_some() =>
        {
            if extended_rational(&left).is_none() {
                trace_conversion(trace, "Int->Rational:left");
            }
            if extended_rational(&right).is_none() {
                trace_conversion(trace, "Int->Rational:right");
            }
            let lower = exact_to_extended_rational(&left)
                .expect("guard retained an exact Rational-convertible lower endpoint");
            let upper = exact_to_extended_rational(&right)
                .expect("guard retained an exact Rational-convertible upper endpoint");
            let nonempty = lower < upper || (lower == upper && lower_inclusive && upper_inclusive);
            let range = match (lower, upper) {
                (ExtendedRational::Finite(lower), ExtendedRational::Finite(upper)) => {
                    Value::RationalRange {
                        lower,
                        upper,
                        lower_inclusive,
                        upper_inclusive,
                    }
                }
                (lower, upper) => Value::InfiniteRationalRange {
                    lower,
                    upper,
                    lower_inclusive,
                    upper_inclusive,
                },
            };
            (range, nonempty)
        }
        _ => {
            return Err(diagnostic(
                source,
                "E-RANGE-ENDPOINTS",
                span,
                "range endpoints require compatible Int or Rational values",
            ));
        }
    };
    trace.record(TraceEvent {
        event: "range.constructed",
        rule: "TOPAL-RANGE-BOUNDS-001",
        detail: if nonempty { "nonempty" } else { "empty" },
    });
    Ok(range)
}

fn apply_range_bound(
    source: &SourceText,
    operation: &str,
    range: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let value = match (operation, range) {
        ("range-lower", Value::IntRange { lower, .. }) => Value::Int(lower),
        ("range-upper", Value::IntRange { upper, .. }) => Value::Int(upper),
        ("range-lower", Value::InfiniteIntRange { lower, .. }) => extended_int_value(lower),
        ("range-upper", Value::InfiniteIntRange { upper, .. }) => extended_int_value(upper),
        ("range-lower", Value::RationalRange { lower, .. }) => Value::Rational(lower),
        ("range-upper", Value::RationalRange { upper, .. }) => Value::Rational(upper),
        ("range-lower", Value::InfiniteRationalRange { lower, .. }) => {
            extended_rational_value(lower)
        }
        ("range-upper", Value::InfiniteRationalRange { upper, .. }) => {
            extended_rational_value(upper)
        }
        (
            "range-lower-inclusive?",
            Value::IntRange {
                lower_inclusive, ..
            }
            | Value::InfiniteIntRange {
                lower_inclusive, ..
            }
            | Value::RationalRange {
                lower_inclusive, ..
            }
            | Value::InfiniteRationalRange {
                lower_inclusive, ..
            },
        ) => Value::Boolean(lower_inclusive),
        (
            "range-upper-inclusive?",
            Value::IntRange {
                upper_inclusive, ..
            }
            | Value::InfiniteIntRange {
                upper_inclusive, ..
            }
            | Value::RationalRange {
                upper_inclusive, ..
            }
            | Value::InfiniteRationalRange {
                upper_inclusive, ..
            },
        ) => Value::Boolean(upper_inclusive),
        _ => {
            return Err(diagnostic(
                source,
                "E-RANGE-BOUND-OPERAND",
                span,
                format!("{operation} requires a bounded exact Range operand"),
            ));
        }
    };
    trace.record(TraceEvent {
        event: "range.bound.observed",
        rule: "TOPAL-RANGE-BOUND-001",
        detail: operation,
    });
    Ok(value)
}

fn apply_range_membership(
    source: &SourceText,
    callable: &str,
    left: Value,
    right: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let extended_operands = int_range_membership_operands(callable, &left, &right);
    if let Some((value, lower, upper, lower_inclusive, upper_inclusive)) = extended_operands {
        let accepted = bound_contains(&value, &lower, &upper, lower_inclusive, upper_inclusive);
        return Ok(record_range_membership(accepted, trace));
    }
    if let Some((accepted, converted)) = rational_range_membership(callable, &left, &right) {
        if converted {
            trace_conversion(trace, "Int->Rational:membership");
        }
        return Ok(record_range_membership(accepted, trace));
    }
    let operands = match (callable, left, right) {
        (
            "in",
            Value::Int(value),
            Value::IntRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            },
        )
        | (
            "contains",
            Value::IntRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            },
            Value::Int(value),
        ) => Some((
            BigRational::from_integer(value),
            BigRational::from_integer(lower),
            BigRational::from_integer(upper),
            lower_inclusive,
            upper_inclusive,
        )),
        (
            "in",
            Value::Rational(value),
            Value::RationalRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            },
        )
        | (
            "contains",
            Value::RationalRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            },
            Value::Rational(value),
        ) => Some((value, lower, upper, lower_inclusive, upper_inclusive)),
        (
            "in",
            Value::Int(value),
            Value::RationalRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            },
        )
        | (
            "contains",
            Value::RationalRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            },
            Value::Int(value),
        ) => {
            trace_conversion(trace, "Int->Rational:membership");
            Some((
                BigRational::from_integer(value),
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            ))
        }
        _ => None,
    };
    let Some((value, lower, upper, lower_inclusive, upper_inclusive)) = operands else {
        return Err(diagnostic(
            source,
            "E-RANGE-MEMBERSHIP-OPERANDS",
            span,
            "range membership requires compatible exact numeric operands",
        ));
    };
    let accepted = bound_contains(&value, &lower, &upper, lower_inclusive, upper_inclusive);
    Ok(record_range_membership(accepted, trace))
}

fn record_range_membership(accepted: bool, trace: &mut impl TraceSink) -> Value {
    trace.record(TraceEvent {
        event: "range.membership.tested",
        rule: "TOPAL-RANGE-MEMBERSHIP-001",
        detail: if accepted { "accepted" } else { "rejected" },
    });
    Value::Boolean(accepted)
}

fn int_range_membership_operands(
    callable: &str,
    left: &Value,
    right: &Value,
) -> Option<(ExtendedInt, ExtendedInt, ExtendedInt, bool, bool)> {
    let (value, range) = match callable {
        "in" => (left, right),
        "contains" => (right, left),
        _ => return None,
    };
    let value = extended_int(value)?;
    let (lower, upper, lower_inclusive, upper_inclusive, _) = extended_int_range(range)?;
    Some((value, lower, upper, lower_inclusive, upper_inclusive))
}

fn rational_range_membership(callable: &str, left: &Value, right: &Value) -> Option<(bool, bool)> {
    let (value, range) = match callable {
        "in" => (left, right),
        "contains" => (right, left),
        _ => return None,
    };
    let (lower, upper, lower_inclusive, upper_inclusive, _) = extended_rational_range(range)?;
    let converted = extended_rational(value).is_none();
    let value = exact_to_extended_rational(value)?;
    Some((
        bound_contains(&value, &lower, &upper, lower_inclusive, upper_inclusive),
        converted,
    ))
}

#[allow(clippy::too_many_lines)] // Each supported conjunction kind has an explicit trace path.
fn apply_and(
    source: &SourceText,
    left: Value,
    right: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if let (Value::Boolean(left), Value::Boolean(right)) = (&left, &right) {
        trace.record(TraceEvent {
            event: "operator.selected",
            rule: "TOPAL-TYPE-CALL-001",
            detail: "root.and(Boolean,Boolean)",
        });
        trace.record(TraceEvent {
            event: "evaluation.logical",
            rule: "TOPAL-TYPE-BOOLEAN-LOGIC-001",
            detail: "and:eager",
        });
        return Ok(Value::Boolean(*left && *right));
    }
    if let (Value::Capability(left), Value::Capability(right)) = (&left, &right) {
        let mut alternatives: Vec<BTreeSet<String>> = left
            .iter()
            .flat_map(|left| {
                right.iter().map(move |right| {
                    let mut combined = left.clone();
                    combined.extend(right.iter().cloned());
                    combined
                })
            })
            .collect();
        alternatives.sort();
        alternatives.dedup();
        trace.record(TraceEvent {
            event: "capability.composed",
            rule: "TOPAL-CAPABILITY-EVIDENCE-001",
            detail: "and",
        });
        return Ok(Value::Capability(alternatives));
    }
    if let (Some(left), Some(right)) = (extended_int_range(&left), extended_int_range(&right)) {
        let (left_lower, left_upper, left_lower_inclusive, left_upper_inclusive, left_infinite) =
            left;
        let (
            right_lower,
            right_upper,
            right_lower_inclusive,
            right_upper_inclusive,
            right_infinite,
        ) = right;
        let (lower, lower_inclusive) = stricter_lower(
            left_lower,
            left_lower_inclusive,
            right_lower,
            right_lower_inclusive,
        );
        let (upper, upper_inclusive) = stricter_upper(
            left_upper,
            left_upper_inclusive,
            right_upper,
            right_upper_inclusive,
        );
        let result = if left_infinite || right_infinite {
            Value::InfiniteIntRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            }
        } else {
            let (ExtendedInt::Finite(lower), ExtendedInt::Finite(upper)) = (lower, upper) else {
                unreachable!("finite Int ranges retain finite endpoints")
            };
            Value::IntRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            }
        };
        trace.record(TraceEvent {
            event: "range.intersection.constructed",
            rule: "TOPAL-RANGE-INTERSECTION-001",
            detail: "conjunction",
        });
        return Ok(result);
    }
    if let (Some(left), Some(right)) = (
        extended_rational_range(&left),
        extended_rational_range(&right),
    ) {
        let (left_lower, left_upper, left_lower_inclusive, left_upper_inclusive, left_infinite) =
            left;
        let (
            right_lower,
            right_upper,
            right_lower_inclusive,
            right_upper_inclusive,
            right_infinite,
        ) = right;
        let (lower, lower_inclusive) = stricter_lower(
            left_lower,
            left_lower_inclusive,
            right_lower,
            right_lower_inclusive,
        );
        let (upper, upper_inclusive) = stricter_upper(
            left_upper,
            left_upper_inclusive,
            right_upper,
            right_upper_inclusive,
        );
        let result = if left_infinite || right_infinite {
            Value::InfiniteRationalRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            }
        } else {
            let (ExtendedRational::Finite(lower), ExtendedRational::Finite(upper)) = (lower, upper)
            else {
                unreachable!("finite Rational ranges retain finite endpoints")
            };
            Value::RationalRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            }
        };
        trace.record(TraceEvent {
            event: "range.intersection.constructed",
            rule: "TOPAL-RANGE-INTERSECTION-001",
            detail: "conjunction",
        });
        return Ok(result);
    }
    let result = match (left, right) {
        (
            Value::RationalRange {
                lower: left_lower,
                upper: left_upper,
                lower_inclusive: left_lower_inclusive,
                upper_inclusive: left_upper_inclusive,
            },
            Value::RationalRange {
                lower: right_lower,
                upper: right_upper,
                lower_inclusive: right_lower_inclusive,
                upper_inclusive: right_upper_inclusive,
            },
        ) => {
            let (lower, lower_inclusive) = stricter_lower(
                left_lower,
                left_lower_inclusive,
                right_lower,
                right_lower_inclusive,
            );
            let (upper, upper_inclusive) = stricter_upper(
                left_upper,
                left_upper_inclusive,
                right_upper,
                right_upper_inclusive,
            );
            Value::RationalRange {
                lower,
                upper,
                lower_inclusive,
                upper_inclusive,
            }
        }
        _ => {
            return Err(diagnostic(
                source,
                "E-RANGE-INTERSECTION-OPERANDS",
                span,
                "and requires two Booleans or ranges from the same endpoint domain",
            ));
        }
    };
    trace.record(TraceEvent {
        event: "range.intersection.constructed",
        rule: "TOPAL-RANGE-INTERSECTION-001",
        detail: "conjunction",
    });
    Ok(result)
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
    if (extended_rational(&left).is_some() || extended_rational(&right).is_some())
        && exact_to_extended_rational(&left).is_some()
        && exact_to_extended_rational(&right).is_some()
    {
        if extended_rational(&left).is_none() {
            trace_conversion(trace, "Int->Rational:left");
        }
        if extended_rational(&right).is_none() {
            trace_conversion(trace, "Int->Rational:right");
        }
        return Some(exact_to_extended_rational(&left)?.cmp(&exact_to_extended_rational(&right)?));
    }
    if let (Some(left), Some(right)) = (extended_int(&left), extended_int(&right))
        && (matches!(
            left,
            ExtendedInt::NegativeInfinity | ExtendedInt::PositiveInfinity
        ) || matches!(
            right,
            ExtendedInt::NegativeInfinity | ExtendedInt::PositiveInfinity
        ))
    {
        return Some(left.cmp(&right));
    }
    match (left, right) {
        (Value::Refined { value, .. }, right) => values_compare(*value, right, trace),
        (left, Value::Refined { value, .. }) => values_compare(left, *value, trace),
        (
            Value::Modular {
                type_name: left_type,
                value: left,
                ..
            },
            Value::Modular {
                type_name: right_type,
                value: right,
                ..
            },
        ) if left_type == right_type => Some(left.cmp(&right)),
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

fn is_singleton_list_construction(source: &SourceText, items: &[Expression]) -> bool {
    matches!(items, [Expression::Identifier(callable), _] if source.slice(*callable) == "one")
}

fn evaluate_singleton_list(
    source: &SourceText,
    session: &Session,
    items: &[Expression],
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let [_, entry] = items else {
        unreachable!("singleton construction shape checked")
    };
    let entry = session.evaluate_expression(source, entry, trace)?;
    Ok(construct_singleton_list(entry, trace))
}

fn is_explicit_empty_list_construction(source: &SourceText, items: &[Expression]) -> bool {
    matches!(items, [Expression::Identifier(empty), Expression::Identifier(list), _]
        if source.slice(*empty) == "empty" && source.slice(*list) == "List")
}

fn evaluate_empty_list(
    source: &SourceText,
    items: &[Expression],
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let [
        Expression::Identifier(empty),
        Expression::Identifier(list),
        element,
    ] = items
    else {
        unreachable!("empty List construction shape checked")
    };
    debug_assert_eq!(source.slice(*empty), "empty");
    debug_assert_eq!(source.slice(*list), "List");
    let Some(element_classifier) = classifier_expression(source, element) else {
        return Err(diagnostic(
            source,
            "E-LIST-ELEMENT-CLASSIFIER",
            element.span(),
            "empty List requires a supported element classifier",
        ));
    };
    Ok(construct_empty_list(element_classifier, trace))
}

fn construct_singleton_list(entry: Value, trace: &mut impl TraceSink) -> Value {
    let element_classifier = structural_value_classifier(&entry);
    let selection = format!("root.one({element_classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    trace.record(TraceEvent {
        event: "list.singleton.constructed",
        rule: "TOPAL-LIST-ONE-001",
        detail: &element_classifier,
    });
    Value::List {
        element_classifier,
        entries: vec![entry],
    }
}

fn construct_empty_list(element_classifier: String, trace: &mut impl TraceSink) -> Value {
    let classifier = format!("List {element_classifier}");
    let selection = format!("root.empty({classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    trace.record(TraceEvent {
        event: "list.empty.constructed",
        rule: "TOPAL-LIST-EMPTY-001",
        detail: &element_classifier,
    });
    Value::List {
        element_classifier,
        entries: Vec::new(),
    }
}

fn apply_empty_predicate(
    source: &SourceText,
    operand: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let (is_empty, classifier, event, rule) = match operand {
        Value::String(text) => (
            text.is_empty(),
            "String".to_owned(),
            "string.empty.tested",
            "TOPAL-STRING-EMPTY-PREDICATE-001",
        ),
        Value::List {
            element_classifier,
            entries,
        } => (
            entries.is_empty(),
            format!("List {element_classifier}"),
            "list.empty.tested",
            "TOPAL-LIST-EMPTY-PREDICATE-001",
        ),
        Value::Array { entries, .. } | Value::Set { entries, .. } => (
            entries.is_empty(),
            "Collection".into(),
            "collection.empty.tested",
            "TOPAL-COLLECTION-EMPTY-PREDICATE-001",
        ),
        Value::Bag { entries, .. } => (
            entries.is_empty(),
            "Collection".into(),
            "collection.empty.tested",
            "TOPAL-COLLECTION-EMPTY-PREDICATE-001",
        ),
        Value::Map { entries, .. } => (
            entries.is_empty(),
            "Collection".into(),
            "collection.empty.tested",
            "TOPAL-COLLECTION-EMPTY-PREDICATE-001",
        ),
        Value::IntRange {
            lower,
            upper,
            lower_inclusive,
            upper_inclusive,
        } => (
            lower > upper || (lower == upper && !(lower_inclusive && upper_inclusive)),
            "Range Int".into(),
            "range.empty.tested",
            "TOPAL-RANGE-EMPTY-001",
        ),
        Value::InfiniteIntRange {
            lower,
            upper,
            lower_inclusive,
            upper_inclusive,
        } => (
            lower > upper || (lower == upper && !(lower_inclusive && upper_inclusive)),
            "Range Int".into(),
            "range.empty.tested",
            "TOPAL-RANGE-EMPTY-001",
        ),
        Value::RationalRange {
            lower,
            upper,
            lower_inclusive,
            upper_inclusive,
        } => (
            lower > upper || (lower == upper && !(lower_inclusive && upper_inclusive)),
            "Range Rational".into(),
            "range.empty.tested",
            "TOPAL-RANGE-EMPTY-001",
        ),
        Value::InfiniteRationalRange {
            lower,
            upper,
            lower_inclusive,
            upper_inclusive,
        } => (
            lower > upper || (lower == upper && !(lower_inclusive && upper_inclusive)),
            "Range Rational".into(),
            "range.empty.tested",
            "TOPAL-RANGE-EMPTY-001",
        ),
        value => {
            let found = structural_value_classifier(&value);
            return Err(diagnostic(
                source,
                "E-NO-APPLICABLE-OVERLOAD",
                span,
                format!("empty? requires a collection, text, or Range operand, found `{found}`"),
            ));
        }
    };
    let selection = format!("root.empty?({classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    trace.record(TraceEvent {
        event,
        rule,
        detail: if is_empty { "true" } else { "false" },
    });
    Ok(Value::Boolean(is_empty))
}

fn is_list_uncons(source: &SourceText, items: &[Expression]) -> bool {
    matches!(items, [Expression::Identifier(name), _] if source.slice(*name) == "uncons")
}

fn evaluate_list_uncons(
    source: &SourceText,
    session: &Session,
    items: &[Expression],
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let [_, operand] = items else {
        unreachable!("uncons expression shape checked")
    };
    let operand_span = operand.span();
    let operand = session.evaluate_expression(source, operand, trace)?;
    let value = apply_list_uncons(source, operand, operand_span, trace)?;
    session.checkpoint(trace, Some(&value), Some(span));
    Ok(value)
}

fn apply_list_uncons(
    source: &SourceText,
    operand: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let Value::List {
        element_classifier,
        mut entries,
    } = operand
    else {
        return Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            span,
            "uncons requires a List operand",
        ));
    };
    let payload_classifier = format!("({element_classifier}, List {element_classifier})");
    let payload = if entries.is_empty() {
        None
    } else {
        let first = entries.remove(0);
        Some(Box::new(Value::Tuple(vec![
            first,
            Value::List {
                element_classifier: element_classifier.clone(),
                entries,
            },
        ])))
    };
    let selection = format!("root.uncons(List {element_classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    trace.record(TraceEvent {
        event: "list.uncons",
        rule: "TOPAL-LIST-UNCONS-001",
        detail: if payload.is_some() { "Some" } else { "None" },
    });
    Ok(Value::Optional {
        payload_classifier,
        payload,
    })
}

fn is_list_projection(source: &SourceText, items: &[Expression]) -> bool {
    matches!(items, [Expression::Identifier(name), _]
        if matches!(source.slice(*name), "first" | "rest"))
}

fn evaluate_list_projection(
    source: &SourceText,
    session: &Session,
    items: &[Expression],
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    let [Expression::Identifier(operation), operand] = items else {
        unreachable!("List projection expression shape checked")
    };
    let operation = source.slice(*operation);
    let operand_span = operand.span();
    let operand = session.evaluate_expression(source, operand, trace)?;
    let Value::List {
        element_classifier,
        mut entries,
    } = operand
    else {
        return Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            operand_span,
            format!("{operation} requires a List operand"),
        ));
    };
    let (payload_classifier, payload) = if operation == "first" {
        (
            element_classifier.clone(),
            (!entries.is_empty()).then(|| Box::new(entries.remove(0))),
        )
    } else {
        let payload_classifier = format!("List {element_classifier}");
        let payload = if entries.is_empty() {
            None
        } else {
            entries.remove(0);
            Some(Box::new(Value::List {
                element_classifier: element_classifier.clone(),
                entries,
            }))
        };
        (payload_classifier, payload)
    };
    let selection = format!("root.{operation}(List {element_classifier})");
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: &selection,
    });
    trace.record(TraceEvent {
        event: if operation == "first" {
            "list.first"
        } else {
            "list.rest"
        },
        rule: if operation == "first" {
            "TOPAL-LIST-FIRST-001"
        } else {
            "TOPAL-LIST-REST-001"
        },
        detail: if payload.is_some() { "Some" } else { "None" },
    });
    let value = Value::Optional {
        payload_classifier,
        payload,
    };
    session.checkpoint(trace, Some(&value), Some(span));
    Ok(value)
}
