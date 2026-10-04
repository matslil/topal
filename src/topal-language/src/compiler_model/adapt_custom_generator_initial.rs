fn adapt_custom_generator_initial(
    source: &SourceText,
    parameter: &CompilerParameter,
    argument: &CompilerExpression,
    generator_name: &str,
    call_span: Span,
) -> Result<CompilerExpression, Diagnostic> {
    validate_list_custom_generator_initial(source, parameter, argument, generator_name)?;
    if parameter.value_type == nested_result_product_type()
        && !matches!(&argument.kind, CompilerExpressionKind::Tuple(_))
    {
        return Err(unsupported(
            source,
            argument.span,
            "nested Result custom generator input outside exact (7, \"item\")",
        ));
    }
    if parameter.value_type == nested_result_product_type() {
        let wrapped = CompilerExpression {
            kind: CompilerExpressionKind::ResultSuccess(Box::new(argument.clone())),
            value_type: parameter.value_type.clone(),
            int_range: None,
            rational_value: None,
            span: argument.span,
        };
        if !exact_nested_result_product_value(&wrapped, 7, "item") {
            return Err(unsupported(
                source,
                argument.span,
                "nested Result custom generator input outside exact (7, \"item\")",
            ));
        }
    }
    if parameter.value_type
        == CompilerType::Optional(Box::new(CompilerType::Tuple(vec![
            CompilerType::Int,
            CompilerType::String,
        ])))
        && !exact_optional_int_string_value(argument, 7, "item")
        && !exact_optional_int_string_none_value(argument)
        && !matches!(&argument.kind, CompilerExpressionKind::Local(name)
            if generator_name == "pairs" && name == "initial")
    {
        return Err(unsupported(
            source,
            argument.span,
            "nested Optional custom generator input outside exact Some (7, \"item\") or None (Int, String)",
        ));
    }
    if recursive_nominal_enumeration(&parameter.value_type).is_some()
        && !exact_recursive_nominal_value(argument, 0)
    {
        return Err(unsupported(
            source,
            argument.span,
            "recursive nominal custom generator input outside exact (Some First, First)",
        ));
    }
    if parameter.value_type == CompilerType::Comparison
        && !exact_int_comparison_value(argument, 1, 2)
    {
        return Err(unsupported(
            source,
            argument.span,
            "Comparison custom generator input outside the exact 1 <=> 2 subset",
        ));
    }
    if parameter.value_type == CompilerType::Result(Box::new(CompilerType::Rational))
        && argument.rational_value.as_ref() != Some(&BigRational::from_integer(BigInt::from(1)))
    {
        return Err(unsupported(
            source,
            argument.span,
            "Result-Rational custom generator input outside the exact proven-success Rational 1 subset",
        ));
    }
    adapt_call_argument(&parameter.value_type, argument)
        .or_else(|| {
            let CompilerType::Result(success) = &parameter.value_type else {
                return None;
            };
            let success = adapt_call_argument(success, argument)?;
            Some(CompilerExpression {
                kind: CompilerExpressionKind::ResultSuccess(Box::new(success)),
                value_type: parameter.value_type.clone(),
                int_range: None,
                rational_value: None,
                span: argument.span,
            })
        })
        .ok_or_else(|| {
            source_diagnostic(
                source,
                "E-NO-APPLICABLE-GENERATOR-OVERLOAD",
                call_span,
                format!(
                    "no `{generator_name}` generator overload accepts `{}`",
                    argument.value_type.name()
                ),
            )
        })
}

fn flattened_product_arguments(
    sources: &[&Expression],
    arguments: &[CompilerExpression],
) -> Option<Vec<CompilerExpression>> {
    if let [Expression::Product { fields, .. }] = sources
        && let [
            CompilerExpression {
                kind: CompilerExpressionKind::Tuple(values),
                ..
            },
        ] = arguments
    {
        return fields
            .iter()
            .all(|field| field.label.is_none())
            .then(|| values.clone());
    }
    let [tuple] = arguments else {
        return None;
    };
    let CompilerType::Tuple(fields) = &tuple.value_type else {
        return None;
    };
    Some(
        fields
            .iter()
            .enumerate()
            .map(|(index, value_type)| CompilerExpression {
                kind: CompilerExpressionKind::TupleField {
                    tuple: Box::new(tuple.clone()),
                    index,
                },
                value_type: value_type.clone(),
                int_range: None,
                rational_value: None,
                span: tuple.span,
            })
            .collect(),
    )
}

fn rational_absolute(value: &BigRational) -> BigRational {
    if value.numer() < &BigInt::from(0) {
        -value.clone()
    } else {
        value.clone()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CompilerInfinityOperand {
    NegativeInfinity,
    Finite(Option<Ordering>),
    PositiveInfinity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CompilerInfinityArithmeticOutcome {
    Direction(bool),
    Indeterminate,
    NeedsDynamicResult,
}

fn compiler_finite_numeric_sign(expression: &CompilerExpression) -> Option<Ordering> {
    if let Some(value) = expression.rational_value.as_ref() {
        return Some(value.cmp(&BigRational::from_integer(BigInt::from(0))));
    }
    if let Some(value) = exact_int(expression) {
        return Some(value.cmp(&BigInt::from(0)));
    }
    expression.int_range.as_ref().and_then(|range| {
        if range.lower > BigInt::from(0) {
            Some(Ordering::Greater)
        } else if range.upper < BigInt::from(0) {
            Some(Ordering::Less)
        } else if range.lower == BigInt::from(0) && range.upper == BigInt::from(0) {
            Some(Ordering::Equal)
        } else {
            None
        }
    })
}

fn compiler_infinity_operand(expression: &CompilerExpression) -> CompilerInfinityOperand {
    match compiler_infinity_direction(expression) {
        Some(true) => CompilerInfinityOperand::NegativeInfinity,
        Some(false) => CompilerInfinityOperand::PositiveInfinity,
        None => CompilerInfinityOperand::Finite(compiler_finite_numeric_sign(expression)),
    }
}

fn compiler_infinity_binary_outcome(
    operation: CompilerBinary,
    left: &CompilerExpression,
    right: &CompilerExpression,
) -> CompilerInfinityArithmeticOutcome {
    use CompilerInfinityArithmeticOutcome::{Direction, Indeterminate, NeedsDynamicResult};
    use CompilerInfinityOperand::{Finite, NegativeInfinity, PositiveInfinity};
    let left = compiler_infinity_operand(left);
    let right = compiler_infinity_operand(right);
    match operation {
        CompilerBinary::Add => match (left, right) {
            (NegativeInfinity, PositiveInfinity) | (PositiveInfinity, NegativeInfinity) => {
                Indeterminate
            }
            (NegativeInfinity, _) | (_, NegativeInfinity) => Direction(true),
            (PositiveInfinity, _) | (_, PositiveInfinity) => Direction(false),
            (Finite(_), Finite(_)) => unreachable!("infinity arithmetic has an infinite operand"),
        },
        CompilerBinary::Subtract => match (left, right) {
            (NegativeInfinity, NegativeInfinity) | (PositiveInfinity, PositiveInfinity) => {
                Indeterminate
            }
            (NegativeInfinity, _) | (_, PositiveInfinity) => Direction(true),
            (PositiveInfinity, _) | (_, NegativeInfinity) => Direction(false),
            (Finite(_), Finite(_)) => unreachable!("infinity arithmetic has an infinite operand"),
        },
        CompilerBinary::Multiply => {
            let negative = |operand| match operand {
                NegativeInfinity | Finite(Some(Ordering::Less)) => Ok(true),
                PositiveInfinity | Finite(Some(Ordering::Greater)) => Ok(false),
                Finite(Some(Ordering::Equal)) => Err(Indeterminate),
                Finite(None) => Err(NeedsDynamicResult),
            };
            match (negative(left), negative(right)) {
                (Ok(left), Ok(right)) => Direction(left ^ right),
                (Err(Indeterminate), _) | (_, Err(Indeterminate)) => Indeterminate,
                (Err(NeedsDynamicResult), _) | (_, Err(NeedsDynamicResult)) => NeedsDynamicResult,
                (Err(Direction(_)), _) | (_, Err(Direction(_))) => {
                    unreachable!("direction is never an operand-sign error")
                }
            }
        }
        _ => NeedsDynamicResult,
    }
}

fn compiler_infinity_direction(expression: &CompilerExpression) -> Option<bool> {
    match &expression.kind {
        CompilerExpressionKind::Infinity { negative }
        | CompilerExpressionKind::InfinityLocal { negative, .. } => Some(*negative),
        CompilerExpressionKind::Negate(value) => Some(!compiler_infinity_direction(value)?),
        CompilerExpressionKind::Absolute(value) => {
            compiler_infinity_direction(value).map(|_| false)
        }
        CompilerExpressionKind::IntToRational(value)
        | CompilerExpressionKind::IntToNat(value)
        | CompilerExpressionKind::IntToNatBoundary(value)
        | CompilerExpressionKind::RationalToInt(value) => compiler_infinity_direction(value),
        CompilerExpressionKind::Binary {
            operation,
            left,
            right,
        } if matches!(
            operation,
            CompilerBinary::Add | CompilerBinary::Subtract | CompilerBinary::Multiply
        ) =>
        {
            if compiler_infinity_direction(left).is_none()
                && compiler_infinity_direction(right).is_none()
            {
                return None;
            }
            match compiler_infinity_binary_outcome(*operation, left, right) {
                CompilerInfinityArithmeticOutcome::Direction(negative) => Some(negative),
                CompilerInfinityArithmeticOutcome::Indeterminate
                | CompilerInfinityArithmeticOutcome::NeedsDynamicResult => None,
            }
        }
        _ => None,
    }
}

fn is_proven_zero_numeric(expression: &CompilerExpression) -> bool {
    match expression.value_type {
        CompilerType::Int => exact_int(expression).is_some_and(|value| value == BigInt::from(0)),
        CompilerType::Rational => expression
            .rational_value
            .as_ref()
            .is_some_and(|value| value.numer() == &BigInt::from(0)),
        _ => false,
    }
}

fn equality_zero_local(expression: &CompilerExpression) -> Option<&str> {
    let CompilerExpressionKind::Binary {
        operation: CompilerBinary::Equal,
        left,
        right,
    } = &expression.kind
    else {
        return None;
    };
    if exact_int(left).is_some_and(|value| value == BigInt::from(0))
        && let CompilerExpressionKind::Local(name) = &right.kind
    {
        return Some(name);
    }
    if exact_int(right).is_some_and(|value| value == BigInt::from(0))
        && let CompilerExpressionKind::Local(name) = &left.kind
    {
        return Some(name);
    }
    None
}

fn equality_zero_call(expression: &CompilerExpression) -> Option<&CompilerExpression> {
    let CompilerExpressionKind::Binary {
        operation: CompilerBinary::Equal,
        left,
        right,
    } = &expression.kind
    else {
        return None;
    };
    (matches!(left.kind, CompilerExpressionKind::Call { .. })
        && exact_int(right).is_some_and(|value| value == BigInt::from(0)))
    .then_some(left)
}

fn less_zero_local(expression: &CompilerExpression) -> Option<&str> {
    let CompilerExpressionKind::Binary {
        operation: CompilerBinary::Less,
        left,
        right,
    } = &expression.kind
    else {
        return None;
    };
    let CompilerExpressionKind::Local(name) = &left.kind else {
        return None;
    };
    exact_int(right)
        .is_some_and(|value| value == BigInt::from(0))
        .then_some(name)
}

fn false_branch_nonnegative_locals(expression: &CompilerExpression) -> Vec<String> {
    if let CompilerExpressionKind::Binary {
        operation: CompilerBinary::Or,
        left,
        right,
    } = &expression.kind
    {
        let mut names = false_branch_nonnegative_locals(left);
        names.extend(false_branch_nonnegative_locals(right));
        names.sort();
        names.dedup();
        return names;
    }
    less_zero_local(expression)
        .map(ToOwned::to_owned)
        .into_iter()
        .collect()
}

fn extend_false_branch_nonnegative_locals(
    expression: &CompilerExpression,
    nonnegative: &mut Vec<String>,
) {
    if let CompilerExpressionKind::Binary {
        operation: CompilerBinary::Or,
        left,
        right,
    } = &expression.kind
    {
        extend_false_branch_nonnegative_locals(left, nonnegative);
        extend_false_branch_nonnegative_locals(right, nonnegative);
        return;
    }
    let CompilerExpressionKind::Binary {
        operation: CompilerBinary::Less,
        left,
        right,
    } = &expression.kind
    else {
        return;
    };
    let (CompilerExpressionKind::Local(left), CompilerExpressionKind::Local(right)) =
        (&left.kind, &right.kind)
    else {
        return;
    };
    if nonnegative.contains(right) && !nonnegative.contains(left) {
        nonnegative.push(left.clone());
    }
}

fn less_equal_zero_nat_local<'a>(
    expression: &'a CompilerExpression,
    environment: &'a BTreeMap<String, BindingFacts>,
) -> Option<&'a str> {
    let CompilerExpressionKind::Binary {
        operation: CompilerBinary::LessEqual,
        left,
        right,
    } = &expression.kind
    else {
        return None;
    };
    let CompilerExpressionKind::Local(name) = &left.kind else {
        return None;
    };
    (exact_int(right).is_some_and(|value| value == BigInt::from(0))
        && binding_facts_by_storage(environment, name)
            .is_some_and(|facts| facts.value_type == CompilerType::Nat))
    .then_some(name)
}

fn less_equal_nat_local_false_bound(
    expression: &CompilerExpression,
    environment: &BTreeMap<String, BindingFacts>,
) -> Option<(String, BigInt)> {
    let CompilerExpressionKind::Binary {
        operation: CompilerBinary::LessEqual,
        left,
        right,
    } = &expression.kind
    else {
        return None;
    };
    let CompilerExpressionKind::Local(name) = &left.kind else {
        return None;
    };
    let bound = exact_int(right)?;
    (binding_facts_by_storage(environment, name)
        .is_some_and(|facts| facts.value_type == CompilerType::Nat)
        && bound >= BigInt::from(0))
    .then(|| (name.clone(), bound + BigInt::from(1)))
}

fn greater_nat_local<'a>(
    expression: &'a CompilerExpression,
    environment: &'a BTreeMap<String, BindingFacts>,
) -> Option<&'a str> {
    let CompilerExpressionKind::Binary {
        operation: CompilerBinary::Greater,
        left,
        right,
    } = &expression.kind
    else {
        return None;
    };
    let CompilerExpressionKind::Local(left_name) = &left.kind else {
        return None;
    };
    let left_nat = binding_facts_by_storage(environment, left_name)
        .is_some_and(|facts| facts.value_type == CompilerType::Nat);
    if exact_int(right).is_some_and(|value| value == BigInt::from(0)) {
        return left_nat.then_some(left_name);
    }
    let CompilerExpressionKind::Local(right_name) = &right.kind else {
        return None;
    };
    let right_nat = binding_facts_by_storage(environment, right_name)
        .is_some_and(|facts| facts.value_type == CompilerType::Nat);
    (left_nat && right_nat).then_some(left_name)
}

fn greater_nat_local_bound(
    expression: &CompilerExpression,
    environment: &BTreeMap<String, BindingFacts>,
) -> Option<(String, BigInt)> {
    let CompilerExpressionKind::Binary {
        operation: CompilerBinary::Greater,
        left,
        right,
    } = &expression.kind
    else {
        return None;
    };
    let CompilerExpressionKind::Local(name) = &left.kind else {
        return None;
    };
    let bound = exact_int(right)?;
    (binding_facts_by_storage(environment, name)
        .is_some_and(|facts| facts.value_type == CompilerType::Nat)
        && bound >= BigInt::from(0))
    .then(|| (name.clone(), bound + BigInt::from(1)))
}

fn call_true_nonzero_local<'a>(
    expression: &'a CompilerExpression,
    summaries: &BTreeMap<String, BTreeSet<usize>>,
) -> Option<&'a str> {
    let CompilerExpressionKind::Call { symbol, arguments } = &expression.kind else {
        return None;
    };
    summaries.get(symbol)?.iter().find_map(|index| {
        let argument = arguments.get(*index)?;
        let CompilerExpressionKind::Local(name) = &argument.kind else {
            return None;
        };
        Some(name.as_str())
    })
}

fn boolean_true_nonzero_parameters(
    expression: &CompilerExpression,
    parameters: &[CompilerParameter],
) -> BTreeSet<usize> {
    let CompilerExpressionKind::BooleanDecision {
        subject,
        when_false,
        ..
    } = &expression.kind
    else {
        return BTreeSet::new();
    };
    if !matches!(when_false.kind, CompilerExpressionKind::Boolean(false)) {
        return BTreeSet::new();
    }
    let CompilerExpressionKind::Binary {
        operation: CompilerBinary::Greater,
        left,
        right,
    } = &subject.kind
    else {
        return BTreeSet::new();
    };
    let (CompilerExpressionKind::Local(left), CompilerExpressionKind::Local(right)) =
        (&left.kind, &right.kind)
    else {
        return BTreeSet::new();
    };
    let left_index = parameters
        .iter()
        .position(|parameter| parameter.name == *left && parameter.value_type == CompilerType::Nat);
    let right_is_nat = parameters
        .iter()
        .any(|parameter| parameter.name == *right && parameter.value_type == CompilerType::Nat);
    left_index.filter(|_| right_is_nat).into_iter().collect()
}

fn is_proven_nonzero_numeric(expression: &CompilerExpression) -> bool {
    match expression.value_type {
        CompilerType::Int | CompilerType::Nat => expression
            .int_range
            .as_ref()
            .is_some_and(|range| range.upper < BigInt::from(0) || range.lower > BigInt::from(0)),
        CompilerType::Rational => expression
            .rational_value
            .as_ref()
            .is_some_and(|value| value.numer() != &BigInt::from(0)),
        _ => false,
    }
}

fn division_by_zero(source: &SourceText, span: Span) -> Diagnostic {
    source_diagnostic(
        source,
        "E-DIVISION-BY-ZERO",
        span,
        "exact division requires a nonzero divisor",
    )
}

fn modular_range(modular: &CompilerModularType) -> IntRange {
    IntRange {
        lower: modular.lower.clone(),
        upper: modular.upper.clone(),
    }
}

fn reduce_modular(value: BigInt, modular: &CompilerModularType) -> BigInt {
    let modulus = &modular.upper - &modular.lower + BigInt::from(1);
    let mut residue = (value - &modular.lower) % &modulus;
    if residue < BigInt::from(0) {
        residue += &modulus;
    }
    &modular.lower + residue
}

fn modulo_range(dividend: &CompilerExpression, divisor: &CompilerExpression) -> Option<IntRange> {
    if let (Some(dividend), Some(divisor)) = (exact_int(dividend), exact_int(divisor)) {
        let magnitude = if divisor < BigInt::from(0) {
            -divisor
        } else {
            divisor
        };
        let mut remainder = dividend % &magnitude;
        if remainder < BigInt::from(0) {
            remainder += magnitude;
        }
        return Some(IntRange::exact(remainder));
    }
    let range = divisor.int_range.as_ref()?;
    let magnitude = (-range.lower.clone()).max(range.upper.clone());
    Some(IntRange {
        lower: BigInt::from(0),
        upper: magnitude - 1,
    })
}

fn power_range(base: &CompilerExpression, exponent: &CompilerExpression) -> Option<IntRange> {
    let base = exact_int(base)?;
    let exponent = exact_int(exponent)?.to_string().parse::<u32>().ok()?;
    Some(IntRange::exact(base.pow(exponent)))
}

fn exact_rational_binary(
    operation: CompilerBinary,
    left: &CompilerExpression,
    right: &CompilerExpression,
) -> Option<BigRational> {
    let left = left.rational_value.as_ref()?;
    if operation == CompilerBinary::Power {
        let exponent = exact_int(right)?.to_string().parse::<i32>().ok()?;
        Some(left.pow(exponent))
    } else {
        let right = right.rational_value.as_ref()?;
        match operation {
            CompilerBinary::Add => Some(left + right),
            CompilerBinary::Subtract => Some(left - right),
            CompilerBinary::Multiply => Some(left * right),
            CompilerBinary::Divide => Some(left / right),
            _ => None,
        }
    }
}

fn combine_ranges(
    left: &CompilerExpression,
    right: &CompilerExpression,
    operation: impl FnOnce(&IntRange, &IntRange) -> IntRange,
) -> Option<IntRange> {
    Some(operation(
        left.int_range.as_ref()?,
        right.int_range.as_ref()?,
    ))
}

fn multiply_range(left: &IntRange, right: &IntRange) -> IntRange {
    let products = [
        &left.lower * &right.lower,
        &left.lower * &right.upper,
        &left.upper * &right.lower,
        &left.upper * &right.upper,
    ];
    IntRange {
        lower: products.iter().min().expect("four products").clone(),
        upper: products.iter().max().expect("four products").clone(),
    }
}

fn absolute_range(range: &IntRange) -> IntRange {
    let zero = BigInt::from(0);
    if range.lower >= zero {
        range.clone()
    } else if range.upper <= zero {
        IntRange {
            lower: -range.upper.clone(),
            upper: -range.lower.clone(),
        }
    } else {
        IntRange {
            lower: zero,
            upper: (-range.lower.clone()).max(range.upper.clone()),
        }
    }
}

fn require_type(
    source: &SourceText,
    span: Span,
    expected: &CompilerType,
    actual: &CompilerType,
) -> Result<(), Diagnostic> {
    if expected == &CompilerType::String && actual == &CompilerType::Character {
        return Ok(());
    }
    require_same_type(source, span, expected, actual)
}

fn require_int_list(
    source: &SourceText,
    value: &CompilerExpression,
    operation: &str,
) -> Result<(), Diagnostic> {
    let CompilerType::List(element) = &value.value_type else {
        return Err(unsupported(source, value.span, operation));
    };
    if element.as_ref() == &CompilerType::Int {
        Ok(())
    } else {
        Err(unsupported(
            source,
            value.span,
            &format!("{operation} for this List element type"),
        ))
    }
}

fn require_int_or_int_pair_list(
    source: &SourceText,
    value: &CompilerExpression,
    operation: &str,
) -> Result<CompilerType, Diagnostic> {
    let CompilerType::List(element) = &value.value_type else {
        return Err(unsupported(source, value.span, operation));
    };
    if element.as_ref() == &CompilerType::Int
        || matches!(
            element.as_ref(),
            CompilerType::Tuple(fields)
                if fields.as_slice() == [CompilerType::Int, CompilerType::Int]
        )
    {
        Ok(element.as_ref().clone())
    } else {
        Err(unsupported(
            source,
            value.span,
            &format!("{operation} for this List element type"),
        ))
    }
}

fn require_int_fold_result(
    source: &SourceText,
    result: &CompilerExpression,
) -> Result<(), Diagnostic> {
    if result.value_type == CompilerType::Int
        || result.value_type == CompilerType::TraversalControl(Box::new(CompilerType::Int))
    {
        Ok(())
    } else {
        Err(source_diagnostic(
            source,
            "E-TYPE-MISMATCH",
            result.span,
            format!(
                "expected Int or TraversalControl Int, found {}",
                result.value_type.name()
            ),
        ))
    }
}

fn require_same_type(
    source: &SourceText,
    span: Span,
    expected: &CompilerType,
    actual: &CompilerType,
) -> Result<(), Diagnostic> {
    if expected == actual {
        Ok(())
    } else {
        Err(source_diagnostic(
            source,
            "E-TYPE-MISMATCH",
            span,
            format!("expected {}, found {}", expected.name(), actual.name()),
        ))
    }
}

fn unit_expression(span: Span) -> CompilerExpression {
    CompilerExpression {
        kind: CompilerExpressionKind::Unit,
        value_type: CompilerType::Unit,
        int_range: None,
        rational_value: None,
        span,
    }
}

fn mangle(name: &str) -> String {
    name.bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || byte == b'_' {
                char::from(byte).to_string()
            } else {
                format!("_{byte:02x}")
            }
        })
        .collect()
}

fn unsupported(source: &SourceText, span: Span, construct: &str) -> Diagnostic {
    source_diagnostic(
        source,
        "E-COMPILER-UNSUPPORTED",
        span,
        format!("the current native compiler increment does not yet support {construct}"),
    )
}

fn source_diagnostic(
    source: &SourceText,
    code: impl Into<String>,
    span: Span,
    message: impl Into<String>,
) -> Diagnostic {
    let start = span.start.min(source.as_str().len());
    let end = span.end.min(source.as_str().len()).max(start);
    let position = source.position(start);
    let line = source
        .as_str()
        .lines()
        .nth(position.line.saturating_sub(1))
        .map(ToOwned::to_owned);
    let width = source.slice(Span::new(start, end)).chars().count().max(1);
    Diagnostic::error(code, position.line, position.column, message)
        .with_source_span(span)
        .with_source_excerpt(line, width)
}

fn statement_span(statement: &Statement) -> Span {
    match statement {
        Statement::LanguageSelection { span, .. }
        | Statement::LibrarySelection { span, .. }
        | Statement::Published { span, .. }
        | Statement::DiagnosticControl { span, .. }
        | Statement::Implementation { span, .. }
        | Statement::StateField { name: span, .. }
        | Statement::ContextAssignment { span, .. }
        | Statement::Function { span, .. }
        | Statement::Generator { span, .. }
        | Statement::Union { span, .. }
        | Statement::Interface { span, .. }
        | Statement::InterfaceImplementation { span, .. }
        | Statement::Foreach { span, .. }
        | Statement::Discard { span, .. } => *span,
        Statement::Binding { name, value, .. } => Span::new(name.start, value.span().end),
        Statement::Return { keyword, value } => Span::new(keyword.start, value.span().end),
        Statement::Expression(expression) => expression.span(),
    }
}
