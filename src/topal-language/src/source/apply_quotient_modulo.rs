fn apply_quotient_modulo(
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
        if parse_integer(source.slice(right_span)).is_none() {
            let position = source.position(right_span.start);
            trace.record(TraceEvent {
                event: "result.error.constructed",
                rule: "TOPAL-TYPE-RESULT-001",
                detail: "root./%(Int,Int);division-by-zero",
            });
            return Ok(Value::Error {
                domain: "root./%(Int,Int)".to_owned(),
                code: "division-by-zero".to_owned(),
                line: position.line,
                column: position.column,
            });
        }
        return Err(diagnostic(
            source,
            "E-DIVISION-BY-ZERO",
            right_span,
            "statically evident quotient/modulo by zero",
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
        detail: "root./%(Int,Int)",
    });
    let remainder = euclidean_remainder(left.clone(), &right);
    let quotient = (left - &remainder) / right;
    trace.record(TraceEvent {
        event: "evaluation.quotient-modulo",
        rule: "TOPAL-NUM-INT-QUOTIENT-MODULO-001",
        detail: "Euclidean",
    });
    Ok(Value::Tuple(vec![
        Value::Int(quotient),
        Value::Int(remainder),
    ]))
}

fn euclidean_remainder(left: BigInt, right: &BigInt) -> BigInt {
    let mut remainder = left % right;
    if remainder < BigInt::from(0) {
        remainder += if right < &BigInt::from(0) {
            -right
        } else {
            right.clone()
        };
    }
    remainder
}

#[allow(clippy::too_many_lines)] // Numeric operations retain explicit diagnostic and trace branches.
fn apply_rational_binary(
    source: &SourceText,
    kind: CallableKind,
    left: BigRational,
    right: BigRational,
    span: Span,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if matches!(kind, CallableKind::Modulo | CallableKind::QuotientModulo) {
        return Err(discrete_operand_diagnostic(source, span));
    }
    let (callable, event, rule, result) = match kind {
        CallableKind::Equal
        | CallableKind::NotEqual
        | CallableKind::Compare
        | CallableKind::Less
        | CallableKind::Greater
        | CallableKind::LessEqual
        | CallableKind::GreaterEqual => {
            unreachable!("comparison is dispatched before numeric operations")
        }
        CallableKind::Range
        | CallableKind::RangeOpen
        | CallableKind::RangeInclusive
        | CallableKind::RangeOpenInclusive => {
            unreachable!("range is dispatched before numeric operations")
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
        CallableKind::Modulo | CallableKind::QuotientModulo => {
            unreachable!("discrete operations are rejected before Rational dispatch")
        }
        CallableKind::Divide => {
            if right.numer() == &BigInt::from(0) {
                trace.record(TraceEvent {
                    event: "obligation.refuted",
                    rule: "TOPAL-NUM-DIVZERO-001",
                    detail: "divisor.nonzero",
                });
                if parse_rational(source.slice(right_span)).is_none()
                    && parse_integer(source.slice(right_span)).is_none()
                {
                    let position = source.position(right_span.start);
                    trace.record(TraceEvent {
                        event: "result.error.constructed",
                        rule: "TOPAL-TYPE-RESULT-001",
                        detail: "root./(Rational,Rational);division-by-zero",
                    });
                    return Ok(Value::Error {
                        domain: "root./(Rational,Rational)".to_owned(),
                        code: "division-by-zero".to_owned(),
                        line: position.line,
                        column: position.column,
                    });
                }
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

fn discrete_operand_diagnostic(source: &SourceText, span: Span) -> Diagnostic {
    diagnostic(
        source,
        "E-NO-APPLICABLE-OVERLOAD",
        span,
        "Euclidean modulo requires discrete Int operands",
    )
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

fn apply_rational_power(
    source: &SourceText,
    left: BigRational,
    right: BigInt,
    left_span: Span,
    right_span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    if right < BigInt::from(0) {
        if left.numer() == &BigInt::from(0) {
            trace.record(TraceEvent {
                event: "obligation.refuted",
                rule: "TOPAL-NUM-RAT-NEG-POW-001",
                detail: "base.nonzero",
            });
            if parse_rational(source.slice(left_span)).is_none()
                && parse_integer(source.slice(left_span)).is_none()
            {
                let position = source.position(left_span.start);
                trace.record(TraceEvent {
                    event: "result.error.constructed",
                    rule: "TOPAL-TYPE-RESULT-001",
                    detail: "root.^(Rational,Int);division-by-zero",
                });
                return Ok(Value::Error {
                    domain: "root.^(Rational,Int)".to_owned(),
                    code: "division-by-zero".to_owned(),
                    line: position.line,
                    column: position.column,
                });
            }
            return Err(diagnostic(
                source,
                "E-DIVISION-BY-ZERO",
                right_span,
                "a zero Rational base cannot be raised to a negative exponent",
            ));
        }
        trace.record(TraceEvent {
            event: "obligation.proved",
            rule: "TOPAL-NUM-RAT-NEG-POW-001",
            detail: "base.nonzero",
        });
        trace.record(TraceEvent {
            event: "operator.selected",
            rule: "TOPAL-TYPE-CALL-001",
            detail: "root.^(Rational,Int)",
        });
        trace.record(TraceEvent {
            event: "evaluation.power",
            rule: "TOPAL-NUM-RAT-NEG-POW-001",
            detail: "Rational",
        });
        let power = pow_rational(left, -right);
        return Ok(Value::Rational(BigRational::new(
            power.denom().clone(),
            power.numer().clone(),
        )));
    }
    trace.record(TraceEvent {
        event: "obligation.proved",
        rule: "TOPAL-NUM-RAT-POW-001",
        detail: "exponent.finite-nat",
    });
    trace.record(TraceEvent {
        event: "operator.selected",
        rule: "TOPAL-TYPE-CALL-001",
        detail: "root.^(Rational,Nat)",
    });
    trace.record(TraceEvent {
        event: "evaluation.power",
        rule: "TOPAL-NUM-RAT-POW-001",
        detail: "Rational",
    });
    Ok(Value::Rational(pow_rational(left, right)))
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

fn pow_rational(mut base: BigRational, mut exponent: BigInt) -> BigRational {
    let zero = BigInt::from(0);
    let one = BigInt::from(1);
    let two = BigInt::from(2);
    let mut result = BigRational::from_integer(one.clone());
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

#[allow(clippy::too_many_lines)] // Rejection remains exhaustive across every nonnumeric value kind.
fn apply_negate(
    source: &SourceText,
    operand: Value,
    span: Span,
    trace: &mut impl TraceSink,
) -> Result<Value, Diagnostic> {
    match operand {
        Value::Infinity {
            negative,
            classifier,
        } => {
            let rational = classifier == "Rational";
            trace.record(TraceEvent {
                event: "operator.selected",
                rule: "TOPAL-TYPE-CALL-001",
                detail: if rational {
                    "root.-(Rational)"
                } else {
                    "root.-(Int)"
                },
            });
            trace.record(TraceEvent {
                event: "evaluation.negate",
                rule: "TOPAL-NUM-INFINITY-ARITHMETIC-001",
                detail: if rational { "Rational" } else { "Int" },
            });
            Ok(Value::Infinity {
                negative: !negative,
                classifier: if classifier == "Nat" {
                    "Int".to_owned()
                } else {
                    classifier
                },
            })
        }
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
        Value::SizeBits(_) => Err(diagnostic(
            source,
            "E-NEGATE-OPERAND",
            span,
            "a storage size cannot be negated",
        )),
        Value::Modular {
            type_name,
            lower,
            upper,
            value,
        } => {
            let value = reduce_modular(-value, &lower, &upper);
            trace.record(TraceEvent {
                event: "numeric.modular.wrapped",
                rule: "TOPAL-NUM-MODULAR-ARITHMETIC-001",
                detail: &type_name,
            });
            Ok(Value::Modular {
                type_name,
                lower,
                upper,
                value,
            })
        }
        Value::Boolean(_)
        | Value::Version(_)
        | Value::NativeSerializer(_)
        | Value::SerializationStream(_)
        | Value::TaskType(_)
        | Value::TaskDefinition(_)
        | Value::TaskInstance(_)
        | Value::Type(_)
        | Value::Effects(_)
        | Value::IntRange { .. }
        | Value::InfiniteIntRange { .. }
        | Value::RationalRange { .. }
        | Value::InfiniteRationalRange { .. }
        | Value::Optional { .. }
        | Value::List { .. }
        | Value::Callable(_)
        | Value::NamedFunction(_)
        | Value::Namespace(_)
        | Value::AnonymousFunction(_)
        | Value::Array { .. }
        | Value::Set { .. }
        | Value::Bag { .. }
        | Value::Map { .. }
        | Value::CharacterGenerator { .. }
        | Value::CharacterReturningGenerator { .. }
        | Value::IterateGenerator { .. }
        | Value::UnfoldGenerator { .. }
        | Value::SuspendedGenerator { .. }
        | Value::String(_)
        | Value::Tuple(_)
        | Value::Record(_)
        | Value::Enum { .. }
        | Value::Union(_)
        | Value::Constraint(_)
        | Value::Capability(_)
        | Value::Interface(_)
        | Value::Introspection(_)
        | Value::ObjectDescription { .. }
        | Value::Refined { .. }
        | Value::ModularType(_)
        | Value::AddressRangeType(_)
        | Value::AddressRange { .. }
        | Value::AddressOffsetType(_)
        | Value::AddressOffset { .. }
        | Value::LayoutType(_)
        | Value::LayoutFactory(_)
        | Value::LayoutBacked { .. }
        | Value::LocationType(_)
        | Value::Location { .. }
        | Value::ErrorDomain(_)
        | Value::Error { .. }
        | Value::Continue(_)
        | Value::Finish(_)
        | Value::Completed
        | Value::Unit => Err(diagnostic(
            source,
            "E-NO-APPLICABLE-OVERLOAD",
            span,
            "prefix - requires an exact numeric operand",
        )),
    }
}

const fn callable_name(kind: CallableKind) -> &'static str {
    match kind {
        CallableKind::Equal => "=",
        CallableKind::NotEqual => "/=",
        CallableKind::Less => "<",
        CallableKind::Greater => ">",
        CallableKind::LessEqual => "<=",
        CallableKind::Compare => "<=>",
        CallableKind::Range => "..",
        CallableKind::RangeOpen => "<..",
        CallableKind::RangeInclusive => "..=",
        CallableKind::RangeOpenInclusive => "<..=",
        CallableKind::GreaterEqual => ">=",
        CallableKind::Plus => "+",
        CallableKind::Minus => "-",
        CallableKind::Multiply => "*",
        CallableKind::Divide => "/",
        CallableKind::QuotientModulo => "/%",
        CallableKind::Modulo => "%",
        CallableKind::Power => "^",
    }
}

fn diagnostic(
    source: &SourceText,
    code: &'static str,
    span: Span,
    message: impl Into<String>,
) -> Diagnostic {
    let position = source.position(span.start);
    let mut diagnostic = Diagnostic::error(code, position.line, position.column, message)
        .with_source_excerpt(
            source
                .as_str()
                .lines()
                .nth(position.line - 1)
                .map(str::to_owned),
            marker_width(source.as_str(), span),
        );
    if let Some(help) = diagnostic_help(code) {
        diagnostic = diagnostic.with_help(help);
    }
    diagnostic
}

fn closest_name<'a>(name: &str, candidates: impl Iterator<Item = &'a String>) -> Option<&'a str> {
    let maximum = 2.max(name.chars().count() / 3);
    candidates
        .map(|candidate| (edit_distance(name, candidate), candidate.as_str()))
        .filter(|(distance, _)| *distance <= maximum)
        .min()
        .map(|(_, candidate)| candidate)
}

const ROOT_OPERATIONS: [&str; 36] = [
    "absolute",
    "ascii-decimal-digit",
    "ascii-decimal-text?",
    "byte-count",
    "case-fold",
    "canonically-equals",
    "characters",
    "character-count",
    "concat",
    "collect",
    "empty",
    "entry-count",
    "first",
    "lower",
    "normalize",
    "range-lower",
    "range-lower-inclusive?",
    "range-upper",
    "range-upper-inclusive?",
    "upper",
    "unicode-whitespace-character",
    "unicode-line-feed-character",
    "unicode-carriage-return-character",
    "unicode-decimal-digit-character",
    "unicode-scalar-characters",
    "unicode-scalar-value",
    "unicode-word-character",
    "uncons",
    "not",
    "negate",
    "one",
    "rest",
    "reverse",
    "stable-sort",
    "stable-sort-descending",
    "zero",
];

fn closest_root_operation(name: &str) -> Option<&'static str> {
    if name == "concatenate" {
        return Some("concat");
    }
    let maximum = 2.max(name.chars().count() / 3);
    ROOT_OPERATIONS
        .into_iter()
        .map(|candidate| (edit_distance(name, candidate), candidate))
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
        "E-MIXED-PRODUCT-FIELDS" => {
            Some("nest a tuple in a labeled field, or place a record inside a tuple")
        }
        "E-RESULT-PROJECTION-INFALLIBLE" => {
            Some("change the function result to `Result (T, Codes)`, or match the Error explicitly")
        }
        "E-RESULT-PROJECTION-OUTSIDE-FUNCTION" => Some("match the Result explicitly at top level"),
        "E-INCOMPLETE-ERROR-CODE-DECISION" => {
            Some("add each missing qualified code pattern, or add an `Error problem` fallback")
        }
        "E-DUPLICATE-ERROR-CODE-PATTERN" => {
            Some("remove the later duplicate pattern or replace it with a missing alternative")
        }
        "E-ANONYMOUS-PATTERN-IDENTITY" => {
            Some("pass the same exact value at every occurrence of the repeated pattern name")
        }
        "E-UNREACHABLE-ERROR-CODE-PATTERN" => {
            Some("move qualified code patterns before the generic `Error problem` fallback")
        }
        "E-UNREACHABLE-DECISION-RULE" => Some("move `otherwise` after every specific matcher"),
        "E-CHARACTER-CLASSIFIER" => {
            Some("use a String containing exactly one Unicode grapheme cluster")
        }
        "E-STRING-CONSTRUCTOR-CHARACTER" => {
            Some("classify a one-character String as Character before construction")
        }
        "E-RATIONAL-NOT-EXACT-INT" => {
            Some("use an exactly divisible expression or keep the result classified as Rational")
        }
        "E-NAT-OUT-OF-RANGE" => Some("use a provably nonnegative Int or handle dynamic validation"),
        "E-INDETERMINATE-RATIONAL" => {
            Some("use a nonzero denominator or handle dynamic Rational construction")
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

pub(crate) fn parse_integer(token: &str) -> Option<BigInt> {
    if let Some(unsigned) = token.strip_prefix('-') {
        return parse_unsigned_integer(unsigned).map(std::ops::Neg::neg);
    }
    parse_unsigned_integer(token)
}

pub(crate) fn parse_rational(token: &str) -> Option<BigRational> {
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
