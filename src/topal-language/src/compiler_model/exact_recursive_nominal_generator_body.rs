fn exact_recursive_nominal_generator_body(
    source: &SourceText,
    parameter: &FunctionParameter,
    enumeration: &CompilerEnumType,
    body: &[Statement],
    span: Span,
) -> Result<ExactValueGeneratorBody, Diagnostic> {
    let [
        Statement::Discard {
            value:
                Expression::Application {
                    items: yield_items,
                    span: yield_span,
                },
            ..
        },
        Statement::Expression(result),
    ] = body
    else {
        return Err(unsupported(
            source,
            span,
            "recursive nominal custom generator outside one initial yield and final (Some Second, Second)",
        ));
    };
    let [
        Expression::Identifier(yield_operation),
        Expression::Identifier(yield_value),
    ] = yield_items.as_slice()
    else {
        return Err(unsupported(
            source,
            *yield_span,
            "recursive nominal custom generator yield outside its initial parameter",
        ));
    };
    if source.slice(*yield_operation) != "yield"
        || source.slice(*yield_value) != source.slice(parameter.name)
    {
        return Err(unsupported(
            source,
            span,
            "recursive nominal custom generator outside yield initial followed by (Some Second, Second)",
        ));
    }
    let result = exact_recursive_nominal_literal(source, result, enumeration, "Second")
        .ok_or_else(|| {
            unsupported(
                source,
                result.span(),
                "recursive nominal custom generator final expression outside (Some Second, Second)",
            )
        })?;
    Ok(ExactValueGeneratorBody {
        yields: vec![CompilerGeneratorYield::Initial(*yield_span)],
        continuations: Vec::new(),
        explicit_return: None,
        result,
    })
}

fn exact_recursive_nominal_generator_foreach_body(
    source: &SourceText,
    parameter: &CompilerParameter,
    enumeration: &CompilerEnumType,
    statements: &[Statement],
) -> Result<CompilerBlock, Diagnostic> {
    let [
        Statement::Discard {
            span,
            value:
                Expression::Application {
                    items,
                    span: expression_span,
                },
        },
    ] = statements
    else {
        return Err(unsupported(
            source,
            parameter.span,
            "recursive nominal custom generator foreach action",
        ));
    };
    let [
        Expression::Identifier(left),
        Expression::Callable {
            kind: CallableKind::Equal,
            ..
        },
        right,
    ] = items.as_slice()
    else {
        return Err(unsupported(
            source,
            *expression_span,
            "recursive nominal custom generator foreach equality action",
        ));
    };
    if parameter.discarded || source.slice(*left) != parameter.name {
        return Err(unsupported(
            source,
            *expression_span,
            "recursive nominal custom generator foreach named candidate action",
        ));
    }
    let right =
        exact_recursive_nominal_literal(source, right, enumeration, "First").ok_or_else(|| {
            unsupported(
                source,
                right.span(),
                "recursive nominal custom generator foreach operand outside (Some First, First)",
            )
        })?;
    Ok(CompilerBlock {
        statements: vec![CompilerStatement::Discard(CompilerExpression {
            kind: CompilerExpressionKind::Binary {
                operation: CompilerBinary::Equal,
                left: Box::new(CompilerExpression {
                    kind: CompilerExpressionKind::Local(parameter.name.clone()),
                    value_type: parameter.value_type.clone(),
                    int_range: None,
                    rational_value: None,
                    span: *left,
                }),
                right: Box::new(right),
            },
            value_type: CompilerType::Boolean,
            int_range: None,
            rational_value: None,
            span: *expression_span,
        })],
        result: unit_expression(*span),
    })
}

fn exact_recursive_nominal_generator_action(
    parameter: &CompilerParameter,
    body: &CompilerBlock,
) -> bool {
    recursive_nominal_enumeration(&parameter.value_type).is_some()
        && !parameter.discarded
        && matches!(body.statements.as_slice(), [CompilerStatement::Discard(
            CompilerExpression {
                kind: CompilerExpressionKind::Binary {
                    operation: CompilerBinary::Equal,
                    left,
                    right,
                },
                ..
            }
        )] if matches!(left.kind, CompilerExpressionKind::Local(ref name)
            if name == &parameter.name)
            && left.value_type == parameter.value_type
            && exact_recursive_nominal_value(right, 0))
        && matches!(body.result.kind, CompilerExpressionKind::Unit)
}

#[allow(clippy::too_many_lines)] // Exact syntax, provenance, and every rejection stay together.
fn exact_result_rational_value_generator_body(
    source: &SourceText,
    parameter: &FunctionParameter,
    body: &[Statement],
    span: Span,
) -> Result<ExactValueGeneratorBody, Diagnostic> {
    let [
        Statement::Discard {
            value:
                Expression::Application {
                    items: yield_items,
                    span: yield_span,
                },
            ..
        },
        Statement::Expression(Expression::Application {
            items: result_items,
            span: result_span,
        }),
    ] = body
    else {
        return Err(unsupported(
            source,
            span,
            "Result-Rational-value custom generator outside one initial yield and final division by Rational 0",
        ));
    };
    let [
        Expression::Identifier(yield_operation),
        Expression::Identifier(yield_value),
    ] = yield_items.as_slice()
    else {
        return Err(unsupported(
            source,
            *yield_span,
            "Result-Rational-value custom generator yield outside its initial parameter",
        ));
    };
    let [
        Expression::Identifier(result_value),
        Expression::Callable {
            kind: CallableKind::Divide,
            ..
        },
        Expression::Application {
            items: zero_items,
            span: zero_span,
        },
    ] = result_items.as_slice()
    else {
        return Err(unsupported(
            source,
            *result_span,
            "Result-Rational-value custom generator final expression",
        ));
    };
    let [
        Expression::Identifier(constructor),
        Expression::Integer(zero_literal),
    ] = zero_items.as_slice()
    else {
        return Err(unsupported(
            source,
            *zero_span,
            "Result-Rational-value custom generator divisor",
        ));
    };
    if source.slice(*yield_operation) != "yield"
        || source.slice(*yield_value) != source.slice(parameter.name)
        || source.slice(*result_value) != source.slice(parameter.name)
        || source.slice(*constructor) != "Rational"
        || parse_integer(source.slice(*zero_literal)).as_ref() != Some(&BigInt::from(0))
    {
        return Err(unsupported(
            source,
            span,
            "Result-Rational-value custom generator outside yield initial followed by initial / (Rational 0)",
        ));
    }
    let result_type = CompilerType::Result(Box::new(CompilerType::Rational));
    let one_int = CompilerExpression {
        kind: CompilerExpressionKind::Int(BigInt::from(1)),
        value_type: CompilerType::Int,
        int_range: Some(IntRange::exact(BigInt::from(1))),
        rational_value: None,
        span: *result_value,
    };
    let one = CompilerExpression {
        kind: CompilerExpressionKind::IntToRational(Box::new(one_int)),
        value_type: CompilerType::Rational,
        int_range: None,
        rational_value: Some(BigRational::from_integer(BigInt::from(1))),
        span: *result_value,
    };
    let zero_int = CompilerExpression {
        kind: CompilerExpressionKind::Int(BigInt::from(0)),
        value_type: CompilerType::Int,
        int_range: Some(IntRange::exact(BigInt::from(0))),
        rational_value: None,
        span: *zero_literal,
    };
    let zero = CompilerExpression {
        kind: CompilerExpressionKind::IntToRational(Box::new(zero_int)),
        value_type: CompilerType::Rational,
        int_range: None,
        rational_value: Some(BigRational::from_integer(BigInt::from(0))),
        span: *zero_span,
    };
    Ok(ExactValueGeneratorBody {
        yields: vec![CompilerGeneratorYield::Initial(*yield_span)],
        continuations: Vec::new(),
        explicit_return: None,
        result: CompilerExpression {
            kind: CompilerExpressionKind::Fallible {
                operation: CompilerFallible::RationalDivide,
                left: Box::new(one),
                right: Box::new(zero),
                error_span: *zero_span,
            },
            value_type: result_type,
            int_range: None,
            rational_value: None,
            span: *result_span,
        },
    })
}

fn exact_result_rational_value_generator_foreach_body(
    source: &SourceText,
    parameter: &CompilerParameter,
    statements: &[Statement],
) -> Result<CompilerBlock, Diagnostic> {
    let [
        Statement::Discard {
            span,
            value:
                Expression::Application {
                    items,
                    span: expression_span,
                },
        },
    ] = statements
    else {
        return Err(unsupported(
            source,
            parameter.span,
            "Result-Rational-value custom generator foreach action outside discarded candidate = candidate",
        ));
    };
    let [
        Expression::Identifier(left),
        Expression::Callable {
            kind: CallableKind::Equal,
            ..
        },
        Expression::Identifier(right),
    ] = items.as_slice()
    else {
        return Err(unsupported(
            source,
            *expression_span,
            "Result-Rational-value custom generator foreach action outside discarded candidate = candidate",
        ));
    };
    if parameter.discarded
        || source.slice(*left) != parameter.name
        || source.slice(*right) != parameter.name
    {
        return Err(unsupported(
            source,
            *expression_span,
            "Result-Rational-value custom generator foreach action outside discarded candidate = candidate",
        ));
    }
    Ok(CompilerBlock {
        statements: vec![CompilerStatement::Discard(CompilerExpression {
            kind: CompilerExpressionKind::Boolean(true),
            value_type: CompilerType::Boolean,
            int_range: None,
            rational_value: None,
            span: *expression_span,
        })],
        result: unit_expression(*span),
    })
}

fn exact_result_rational_value_generator_action(
    parameter: &CompilerParameter,
    body: &CompilerBlock,
) -> bool {
    parameter.value_type == CompilerType::Result(Box::new(CompilerType::Rational))
        && !parameter.discarded
        && matches!(
            body.statements.as_slice(),
            [CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::Boolean(true),
                ..
            })]
        )
        && matches!(body.result.kind, CompilerExpressionKind::Unit)
}

fn exact_int_value_generator_body(
    source: &SourceText,
    parameter: &FunctionParameter,
    body: &[Statement],
    span: Span,
) -> Result<ExactValueGeneratorBody, Diagnostic> {
    let [
        Statement::Discard {
            value:
                Expression::Application {
                    items: yield_items,
                    span: yield_span,
                },
            ..
        },
        Statement::Expression(Expression::Application {
            items: result_items,
            span: result_span,
        }),
    ] = body
    else {
        return Err(unsupported(
            source,
            span,
            "Int-value custom generator outside one initial yield and final increment",
        ));
    };
    let [
        Expression::Identifier(yield_operation),
        Expression::Identifier(yield_value),
    ] = yield_items.as_slice()
    else {
        return Err(unsupported(
            source,
            *yield_span,
            "Int-value custom generator yield outside its initial parameter",
        ));
    };
    let [
        Expression::Identifier(result_value),
        Expression::Callable {
            kind: CallableKind::Plus,
            ..
        },
        Expression::Integer(increment),
    ] = result_items.as_slice()
    else {
        return Err(unsupported(
            source,
            *result_span,
            "Int-value custom generator final value outside initial + 1",
        ));
    };
    if source.slice(*yield_operation) != "yield"
        || source.slice(*yield_value) != source.slice(parameter.name)
        || source.slice(*result_value) != source.slice(parameter.name)
        || parse_integer(source.slice(*increment)).as_ref() != Some(&BigInt::from(1))
    {
        return Err(unsupported(
            source,
            span,
            "Int-value custom generator outside yield initial followed by initial + 1",
        ));
    }
    let initial = CompilerExpression {
        kind: CompilerExpressionKind::Local(source.slice(parameter.name).to_owned()),
        value_type: CompilerType::Int,
        int_range: None,
        rational_value: None,
        span: *result_value,
    };
    let one = CompilerExpression {
        kind: CompilerExpressionKind::Int(BigInt::from(1)),
        value_type: CompilerType::Int,
        int_range: Some(IntRange::exact(BigInt::from(1))),
        rational_value: None,
        span: *increment,
    };
    Ok(ExactValueGeneratorBody {
        yields: vec![CompilerGeneratorYield::Initial(*yield_span)],
        continuations: Vec::new(),
        explicit_return: None,
        result: CompilerExpression {
            kind: CompilerExpressionKind::Binary {
                operation: CompilerBinary::Add,
                left: Box::new(initial),
                right: Box::new(one),
            },
            value_type: CompilerType::Int,
            int_range: None,
            rational_value: None,
            span: *result_span,
        },
    })
}

fn exact_int_value_generator_action(parameter: &CompilerParameter, body: &CompilerBlock) -> bool {
    !parameter.discarded
        && matches!(
            body.statements.as_slice(),
            [CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::Binary {
                    operation: CompilerBinary::Add,
                    left,
                    right,
                },
                ..
            })] if matches!(left.kind, CompilerExpressionKind::Local(ref name)
                if name == &parameter.name)
                && matches!(right.kind, CompilerExpressionKind::Int(ref value)
                    if value == &BigInt::from(1))
        )
        && matches!(body.result.kind, CompilerExpressionKind::Unit)
}

fn exact_nat_value_generator_body(
    source: &SourceText,
    parameter: &FunctionParameter,
    body: &[Statement],
    span: Span,
) -> Result<ExactValueGeneratorBody, Diagnostic> {
    let [
        Statement::Discard {
            value:
                Expression::Application {
                    items: yield_items,
                    span: yield_span,
                },
            ..
        },
        Statement::Expression(Expression::Application {
            items: result_items,
            span: result_span,
        }),
    ] = body
    else {
        return Err(unsupported(
            source,
            span,
            "Nat-value custom generator outside one initial yield and final increment",
        ));
    };
    let [
        Expression::Identifier(yield_operation),
        Expression::Identifier(yield_value),
    ] = yield_items.as_slice()
    else {
        return Err(unsupported(
            source,
            *yield_span,
            "Nat-value custom generator yield outside its initial parameter",
        ));
    };
    let [
        Expression::Identifier(result_value),
        Expression::Callable {
            kind: CallableKind::Plus,
            ..
        },
        Expression::Integer(increment),
    ] = result_items.as_slice()
    else {
        return Err(unsupported(
            source,
            *result_span,
            "Nat-value custom generator final value outside initial + 1",
        ));
    };
    if source.slice(*yield_operation) != "yield"
        || source.slice(*yield_value) != source.slice(parameter.name)
        || source.slice(*result_value) != source.slice(parameter.name)
        || parse_integer(source.slice(*increment)).as_ref() != Some(&BigInt::from(1))
    {
        return Err(unsupported(
            source,
            span,
            "Nat-value custom generator outside yield initial followed by initial + 1",
        ));
    }
    let initial = CompilerExpression {
        kind: CompilerExpressionKind::Local(source.slice(parameter.name).to_owned()),
        value_type: CompilerType::Nat,
        int_range: None,
        rational_value: None,
        span: *result_value,
    };
    let one = CompilerExpression {
        kind: CompilerExpressionKind::Int(BigInt::from(1)),
        value_type: CompilerType::Int,
        int_range: Some(IntRange::exact(BigInt::from(1))),
        rational_value: None,
        span: *increment,
    };
    Ok(ExactValueGeneratorBody {
        yields: vec![CompilerGeneratorYield::Initial(*yield_span)],
        continuations: Vec::new(),
        explicit_return: None,
        result: CompilerExpression {
            kind: CompilerExpressionKind::Binary {
                operation: CompilerBinary::Add,
                left: Box::new(initial),
                right: Box::new(one),
            },
            value_type: CompilerType::Nat,
            int_range: None,
            rational_value: None,
            span: *result_span,
        },
    })
}

fn exact_nat_value_generator_action(parameter: &CompilerParameter, body: &CompilerBlock) -> bool {
    !parameter.discarded
        && matches!(
            body.statements.as_slice(),
            [CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::Binary {
                    operation: CompilerBinary::Add,
                    left,
                    right,
                },
                ..
            })] if matches!(left.kind, CompilerExpressionKind::Local(ref name)
                if name == &parameter.name)
                && matches!(right.kind, CompilerExpressionKind::Int(ref value)
                    if value == &BigInt::from(1))
        )
        && matches!(body.result.kind, CompilerExpressionKind::Unit)
}

fn exact_nat_value_generator_foreach_body(
    source: &SourceText,
    parameter: &CompilerParameter,
    statements: &[Statement],
) -> Result<CompilerBlock, Diagnostic> {
    let [
        Statement::Discard {
            span,
            value:
                Expression::Application {
                    items,
                    span: expression_span,
                },
        },
    ] = statements
    else {
        return Err(unsupported(
            source,
            parameter.span,
            "Nat-value custom generator foreach action outside discarded value + 1",
        ));
    };
    let [
        Expression::Identifier(value),
        Expression::Callable {
            kind: CallableKind::Plus,
            ..
        },
        Expression::Integer(increment),
    ] = items.as_slice()
    else {
        return Err(unsupported(
            source,
            *expression_span,
            "Nat-value custom generator foreach action outside discarded value + 1",
        ));
    };
    if parameter.discarded
        || source.slice(*value) != parameter.name
        || parse_integer(source.slice(*increment)).as_ref() != Some(&BigInt::from(1))
    {
        return Err(unsupported(
            source,
            *expression_span,
            "Nat-value custom generator foreach action outside discarded value + 1",
        ));
    }
    let left = CompilerExpression {
        kind: CompilerExpressionKind::Local(parameter.name.clone()),
        value_type: CompilerType::Nat,
        int_range: None,
        rational_value: None,
        span: *value,
    };
    let right = CompilerExpression {
        kind: CompilerExpressionKind::Int(BigInt::from(1)),
        value_type: CompilerType::Int,
        int_range: Some(IntRange::exact(BigInt::from(1))),
        rational_value: None,
        span: *increment,
    };
    Ok(CompilerBlock {
        statements: vec![CompilerStatement::Discard(CompilerExpression {
            kind: CompilerExpressionKind::Binary {
                operation: CompilerBinary::Add,
                left: Box::new(left),
                right: Box::new(right),
            },
            value_type: CompilerType::Nat,
            int_range: None,
            rational_value: None,
            span: *expression_span,
        })],
        result: unit_expression(*span),
    })
}

fn exact_one_third_rational_expression(
    source: &SourceText,
    expression: &Expression,
) -> Option<CompilerExpression> {
    let Expression::Application { items, span } = expression else {
        return None;
    };
    let [
        Expression::Identifier(constructor),
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return None;
    };
    let [numerator, denominator] = fields.as_slice() else {
        return None;
    };
    let (Expression::Integer(numerator_span), Expression::Integer(denominator_span)) =
        (&numerator.value, &denominator.value)
    else {
        return None;
    };
    if source.slice(*constructor) != "Rational"
        || numerator.label.is_some()
        || denominator.label.is_some()
        || parse_integer(source.slice(*numerator_span)).as_ref() != Some(&BigInt::from(1))
        || parse_integer(source.slice(*denominator_span)).as_ref() != Some(&BigInt::from(3))
    {
        return None;
    }
    let numerator = CompilerExpression {
        kind: CompilerExpressionKind::Int(BigInt::from(1)),
        value_type: CompilerType::Int,
        int_range: Some(IntRange::exact(BigInt::from(1))),
        rational_value: None,
        span: *numerator_span,
    };
    let denominator = CompilerExpression {
        kind: CompilerExpressionKind::Int(BigInt::from(3)),
        value_type: CompilerType::Int,
        int_range: Some(IntRange::exact(BigInt::from(3))),
        rational_value: None,
        span: *denominator_span,
    };
    Some(CompilerExpression {
        kind: CompilerExpressionKind::RationalConstruct {
            numerator: Box::new(numerator),
            denominator: Box::new(denominator),
        },
        value_type: CompilerType::Rational,
        int_range: None,
        rational_value: Some(BigRational::new(BigInt::from(1), BigInt::from(3))),
        span: *span,
    })
}

fn exact_rational_value_generator_body(
    source: &SourceText,
    parameter: &FunctionParameter,
    body: &[Statement],
    span: Span,
) -> Result<ExactValueGeneratorBody, Diagnostic> {
    let [
        Statement::Discard {
            value:
                Expression::Application {
                    items: yield_items,
                    span: yield_span,
                },
            ..
        },
        Statement::Expression(Expression::Application {
            items: result_items,
            span: result_span,
        }),
    ] = body
    else {
        return Err(unsupported(
            source,
            span,
            "Rational-value custom generator outside one initial yield and final addition",
        ));
    };
    let [
        Expression::Identifier(yield_operation),
        Expression::Identifier(yield_value),
    ] = yield_items.as_slice()
    else {
        return Err(unsupported(
            source,
            *yield_span,
            "Rational-value custom generator yield outside its initial parameter",
        ));
    };
    let [
        Expression::Identifier(result_value),
        Expression::Callable {
            kind: CallableKind::Plus,
            ..
        },
        increment,
    ] = result_items.as_slice()
    else {
        return Err(unsupported(
            source,
            *result_span,
            "Rational-value custom generator final value outside initial + Rational (1, 3)",
        ));
    };
    let Some(increment) = exact_one_third_rational_expression(source, increment) else {
        return Err(unsupported(
            source,
            increment.span(),
            "Rational-value custom generator final addend outside Rational (1, 3)",
        ));
    };
    if source.slice(*yield_operation) != "yield"
        || source.slice(*yield_value) != source.slice(parameter.name)
        || source.slice(*result_value) != source.slice(parameter.name)
    {
        return Err(unsupported(
            source,
            span,
            "Rational-value custom generator outside yield initial followed by its exact final addition",
        ));
    }
    let initial = CompilerExpression {
        kind: CompilerExpressionKind::Local(source.slice(parameter.name).to_owned()),
        value_type: CompilerType::Rational,
        int_range: None,
        rational_value: None,
        span: *result_value,
    };
    Ok(ExactValueGeneratorBody {
        yields: vec![CompilerGeneratorYield::Initial(*yield_span)],
        continuations: Vec::new(),
        explicit_return: None,
        result: CompilerExpression {
            kind: CompilerExpressionKind::Binary {
                operation: CompilerBinary::Add,
                left: Box::new(initial),
                right: Box::new(increment),
            },
            value_type: CompilerType::Rational,
            int_range: None,
            rational_value: None,
            span: *result_span,
        },
    })
}

fn exact_rational_value_generator_action(
    parameter: &CompilerParameter,
    body: &CompilerBlock,
) -> bool {
    !parameter.discarded
        && matches!(
            body.statements.as_slice(),
            [CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::Binary {
                    operation: CompilerBinary::Add,
                    left,
                    right,
                },
                ..
            })] if matches!(left.kind, CompilerExpressionKind::Local(ref name)
                if name == &parameter.name)
                && exact_one_third_rational_construct(right)
        )
        && matches!(body.result.kind, CompilerExpressionKind::Unit)
}

fn exact_one_third_rational_construct(expression: &CompilerExpression) -> bool {
    let CompilerExpressionKind::RationalConstruct {
        numerator,
        denominator,
    } = &expression.kind
    else {
        return false;
    };
    matches!(&numerator.kind, CompilerExpressionKind::Int(value)
        if value == &BigInt::from(1))
        && matches!(&denominator.kind, CompilerExpressionKind::Int(value)
            if value == &BigInt::from(3))
        && expression.rational_value.as_ref()
            == Some(&BigRational::new(BigInt::from(1), BigInt::from(3)))
}

fn exact_unit_value_generator_body(
    source: &SourceText,
    parameter: &FunctionParameter,
    body: &[Statement],
    span: Span,
) -> Result<ExactValueGeneratorBody, Diagnostic> {
    let [
        Statement::Discard {
            value:
                Expression::Application {
                    items: yield_items,
                    span: yield_span,
                },
            ..
        },
        Statement::Expression(Expression::Unit(result_span)),
    ] = body
    else {
        return Err(unsupported(
            source,
            span,
            "Unit-value custom generator outside one initial yield and final Unit",
        ));
    };
    let [
        Expression::Identifier(yield_operation),
        Expression::Identifier(yield_value),
    ] = yield_items.as_slice()
    else {
        return Err(unsupported(
            source,
            *yield_span,
            "Unit-value custom generator yield outside its initial parameter",
        ));
    };
    if source.slice(*yield_operation) != "yield"
        || source.slice(*yield_value) != source.slice(parameter.name)
    {
        return Err(unsupported(
            source,
            span,
            "Unit-value custom generator outside yield initial followed by ()",
        ));
    }
    Ok(ExactValueGeneratorBody {
        yields: vec![CompilerGeneratorYield::Initial(*yield_span)],
        continuations: Vec::new(),
        explicit_return: None,
        result: unit_expression(*result_span),
    })
}

fn exact_unit_value_generator_action(parameter: &CompilerParameter, body: &CompilerBlock) -> bool {
    !parameter.discarded
        && body.statements.is_empty()
        && matches!(
            body.result,
            CompilerExpression {
                kind: CompilerExpressionKind::Local(ref name),
                value_type: CompilerType::Unit,
                ..
            } if name == &parameter.name
        )
}

fn exact_optional_int_value_generator_body(
    source: &SourceText,
    parameter: &FunctionParameter,
    body: &[Statement],
    span: Span,
) -> Result<ExactValueGeneratorBody, Diagnostic> {
    let [
        Statement::Discard {
            value:
                Expression::Application {
                    items: yield_items,
                    span: yield_span,
                },
            ..
        },
        Statement::Expression(Expression::Application {
            items: result_items,
            span: result_span,
        }),
    ] = body
    else {
        return Err(unsupported(
            source,
            span,
            "Optional-Int-value custom generator outside one initial yield and final None Int",
        ));
    };
    let [
        Expression::Identifier(yield_operation),
        Expression::Identifier(yield_value),
    ] = yield_items.as_slice()
    else {
        return Err(unsupported(
            source,
            *yield_span,
            "Optional-Int-value custom generator yield outside its initial parameter",
        ));
    };
    let [
        Expression::Identifier(none_constructor),
        Expression::Identifier(none_payload),
    ] = result_items.as_slice()
    else {
        return Err(unsupported(
            source,
            *result_span,
            "Optional-Int-value custom generator final value outside None Int",
        ));
    };
    if source.slice(*yield_operation) != "yield"
        || source.slice(*yield_value) != source.slice(parameter.name)
        || source.slice(*none_constructor) != "None"
        || source.slice(*none_payload) != "Int"
    {
        return Err(unsupported(
            source,
            span,
            "Optional-Int-value custom generator outside yield initial followed by None Int",
        ));
    }
    Ok(ExactValueGeneratorBody {
        yields: vec![CompilerGeneratorYield::Initial(*yield_span)],
        continuations: Vec::new(),
        explicit_return: None,
        result: CompilerExpression {
            kind: CompilerExpressionKind::OptionalNone,
            value_type: CompilerType::Optional(Box::new(CompilerType::Int)),
            int_range: None,
            rational_value: None,
            span: *result_span,
        },
    })
}

fn exact_optional_int_value_generator_action(
    parameter: &CompilerParameter,
    body: &CompilerBlock,
) -> bool {
    !parameter.discarded
        && matches!(
            body.statements.as_slice(),
            [CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::Binary {
                    operation: CompilerBinary::Equal,
                    left,
                    right,
                },
                ..
            })] if matches!(left.kind, CompilerExpressionKind::Local(ref name)
                if name == &parameter.name)
                && matches!(right.kind, CompilerExpressionKind::OptionalSome(ref payload)
                    if matches!(payload.kind, CompilerExpressionKind::Int(ref value)
                        if value == &BigInt::from(7)))
        )
        && matches!(body.result.kind, CompilerExpressionKind::Unit)
}

fn exact_int_range_literal(
    source: &SourceText,
    expression: &Expression,
    lower_value: i64,
    upper_value: i64,
) -> Option<CompilerExpression> {
    let Expression::Application { items, span } = expression else {
        return None;
    };
    let [
        Expression::Integer(lower_span),
        Expression::Callable {
            kind: CallableKind::RangeInclusive,
            ..
        },
        Expression::Integer(upper_span),
    ] = items.as_slice()
    else {
        return None;
    };
    let lower_value = BigInt::from(lower_value);
    let upper_value = BigInt::from(upper_value);
    if parse_integer(source.slice(*lower_span)).as_ref() != Some(&lower_value)
        || parse_integer(source.slice(*upper_span)).as_ref() != Some(&upper_value)
    {
        return None;
    }
    let lower = CompilerExpression {
        kind: CompilerExpressionKind::Int(lower_value.clone()),
        value_type: CompilerType::Int,
        int_range: Some(IntRange::exact(lower_value)),
        rational_value: None,
        span: *lower_span,
    };
    let upper = CompilerExpression {
        kind: CompilerExpressionKind::Int(upper_value.clone()),
        value_type: CompilerType::Int,
        int_range: Some(IntRange::exact(upper_value)),
        rational_value: None,
        span: *upper_span,
    };
    Some(CompilerExpression {
        kind: CompilerExpressionKind::Binary {
            operation: CompilerBinary::RangeInclusive,
            left: Box::new(lower),
            right: Box::new(upper),
        },
        value_type: CompilerType::Range(Box::new(CompilerType::Int)),
        int_range: None,
        rational_value: None,
        span: *span,
    })
}

fn exact_int_range_value_generator_body(
    source: &SourceText,
    parameter: &FunctionParameter,
    body: &[Statement],
    span: Span,
) -> Result<ExactValueGeneratorBody, Diagnostic> {
    let [
        Statement::Discard {
            value:
                Expression::Application {
                    items: yield_items,
                    span: yield_span,
                },
            ..
        },
        Statement::Expression(Expression::Application {
            items: result_items,
            span: result_span,
        }),
    ] = body
    else {
        return Err(unsupported(
            source,
            span,
            "Range-Int-value custom generator outside one initial yield and final intersection",
        ));
    };
    let [
        Expression::Identifier(yield_operation),
        Expression::Identifier(yield_value),
    ] = yield_items.as_slice()
    else {
        return Err(unsupported(
            source,
            *yield_span,
            "Range-Int-value custom generator yield outside its initial parameter",
        ));
    };
    let [
        Expression::Identifier(result_value),
        Expression::Identifier(intersection),
        retained_range,
    ] = result_items.as_slice()
    else {
        return Err(unsupported(
            source,
            *result_span,
            "Range-Int-value custom generator final value outside initial and (5 ..= 15)",
        ));
    };
    let Some(retained_range) = exact_int_range_literal(source, retained_range, 5, 15) else {
        return Err(unsupported(
            source,
            retained_range.span(),
            "Range-Int-value custom generator final intersection outside 5 ..= 15",
        ));
    };
    if source.slice(*yield_operation) != "yield"
        || source.slice(*yield_value) != source.slice(parameter.name)
        || source.slice(*result_value) != source.slice(parameter.name)
        || source.slice(*intersection) != "and"
    {
        return Err(unsupported(
            source,
            span,
            "Range-Int-value custom generator outside yield initial followed by its exact final intersection",
        ));
    }
    let range_int = CompilerType::Range(Box::new(CompilerType::Int));
    let initial = CompilerExpression {
        kind: CompilerExpressionKind::Local(source.slice(parameter.name).to_owned()),
        value_type: range_int.clone(),
        int_range: None,
        rational_value: None,
        span: *result_value,
    };
    Ok(ExactValueGeneratorBody {
        yields: vec![CompilerGeneratorYield::Initial(*yield_span)],
        continuations: Vec::new(),
        explicit_return: None,
        result: CompilerExpression {
            kind: CompilerExpressionKind::Binary {
                operation: CompilerBinary::And,
                left: Box::new(initial),
                right: Box::new(retained_range),
            },
            value_type: range_int,
            int_range: None,
            rational_value: None,
            span: *result_span,
        },
    })
}

fn exact_int_range_value_generator_action(
    parameter: &CompilerParameter,
    body: &CompilerBlock,
) -> bool {
    !parameter.discarded
        && matches!(
            body.statements.as_slice(),
            [CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::Binary {
                    operation: CompilerBinary::In,
                    left,
                    right,
                },
                ..
            })] if matches!(left.kind, CompilerExpressionKind::Int(ref value)
                if value == &BigInt::from(5))
                && matches!(right.kind, CompilerExpressionKind::Local(ref name)
                    if name == &parameter.name)
        )
        && matches!(body.result.kind, CompilerExpressionKind::Unit)
}

fn exact_string_overload_generator_action(
    parameter: &CompilerParameter,
    body: &CompilerBlock,
) -> bool {
    parameter.value_type == CompilerType::String
        && !parameter.discarded
        && matches!(
            body.statements.as_slice(),
            [CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::StringEmptyPredicate(value),
                ..
            })] if matches!(value.kind, CompilerExpressionKind::Local(ref name)
                if name == &parameter.name)
        )
        && matches!(body.result.kind, CompilerExpressionKind::Unit)
}

fn generator_input_types(generator: &GeneratorSource) -> Vec<&CompilerType> {
    std::iter::once(&generator.initial_parameter.value_type)
        .chain(
            generator
                .additional_initial_parameters
                .iter()
                .map(|parameter| &parameter.value_type),
        )
        .collect()
}

fn exact_value_boundary_generator_source(source: &SourceText, generator: &GeneratorSource) -> bool {
    let scalar = source.slice(generator.name) == "numbers"
        && generator.initial_parameter.value_type == CompilerType::Int
        && exact_string(&generator.result).as_deref() == Some("done");
    let product = source.slice(generator.name) == "pairs"
        && generator.initial_parameter.value_type
            == CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String])
        && exact_int_string_product(&generator.result, 8, "done");
    let nested = source.slice(generator.name) == "pairs"
        && generator.initial_parameter.value_type == nested_optional_product_type()
        && exact_nested_result_product_value(&generator.result, 8, "done");
    let list = source.slice(generator.name) == "relay"
        && generator.initial_parameter.value_type == int_list_type()
        && exact_int_list_append(&generator.result, "initial", 9);
    (scalar || product || nested || list)
        && generator.initial_parameter.name == "initial"
        && generator.additional_initial_parameters.is_empty()
        && generator.prefix.statements.is_empty()
        && matches!(
            generator.value_yields.as_deref(),
            Some([CompilerGeneratorYield::Initial(_)])
        )
        && generator.value_continuations.is_empty()
        && generator.explicit_return.is_none()
}

fn register_generator(
    source: &SourceText,
    generators: &mut BTreeMap<String, Vec<GeneratorSource>>,
    name: &str,
    generator: GeneratorSource,
) -> Result<(), Diagnostic> {
    let signature = generator_input_types(&generator);
    let overloads = generators.entry(name.to_owned()).or_default();
    if overloads
        .iter()
        .any(|candidate| generator_input_types(candidate) == signature)
    {
        return Err(source_diagnostic(
            source,
            "E-DUPLICATE-GENERATOR-OVERLOAD",
            generator.span,
            format!("generator overload `{name}` has the same input classifiers"),
        ));
    }
    overloads.push(generator);
    Ok(())
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)] // The admitted declaration and complete retained body graph are validated together.
fn exact_int_string_generator_overload(
    source: &SourceText,
    name: Span,
    parameters: &[FunctionParameter],
    yielded: Span,
    resumed: Span,
    result: Span,
    body: &[Statement],
    span: Span,
) -> Result<GeneratorSource, Diagnostic> {
    let [value_parameter, suffix_parameter] = parameters else {
        return Err(unsupported(
            source,
            span,
            "multi-input custom generator outside two ordered operands",
        ));
    };
    if !value_parameter.fields.is_empty()
        || value_parameter.default.is_some()
        || value_parameter.qualifier.is_some()
        || !suffix_parameter.fields.is_empty()
        || suffix_parameter.default.is_some()
        || suffix_parameter.qualifier.is_some()
        || source.slice(value_parameter.name) != "value"
        || source.slice(value_parameter.classifier) != "Int"
        || source.slice(suffix_parameter.name) != "suffix"
        || source.slice(suffix_parameter.classifier) != "String"
        || source.slice(yielded) != "String"
        || source.slice(resumed) != "Unit"
        || source.slice(result) != "String"
    {
        return Err(unsupported(
            source,
            span,
            "multi-input custom generator outside (value : Int, suffix : String) yielding String, resuming Unit, and returning String",
        ));
    }
    let [
        Statement::Discard {
            value:
                Expression::Application {
                    items: prefix_items,
                    span: prefix_span,
                },
            ..
        },
        Statement::Discard {
            value:
                Expression::Application {
                    items: yield_items,
                    span: yield_span,
                },
            ..
        },
        Statement::Expression(final_expression),
    ] = body
    else {
        return Err(unsupported(
            source,
            span,
            "multi-input custom generator outside one prefix increment, suffix yield, and final String",
        ));
    };
    let [
        Expression::Identifier(prefix_value),
        Expression::Callable {
            kind: CallableKind::Plus,
            ..
        },
        Expression::Integer(increment),
    ] = prefix_items.as_slice()
    else {
        return Err(unsupported(
            source,
            *prefix_span,
            "multi-input custom generator prefix outside value + 1",
        ));
    };
    let [
        Expression::Identifier(yield_operation),
        Expression::Identifier(yield_value),
    ] = yield_items.as_slice()
    else {
        return Err(unsupported(
            source,
            *yield_span,
            "multi-input custom generator yield outside suffix",
        ));
    };
    let final_value = exact_string_literal_expression(source, final_expression.span())?;
    if source.slice(*prefix_value) != "value"
        || parse_integer(source.slice(*increment)).as_ref() != Some(&BigInt::from(1))
        || source.slice(*yield_operation) != "yield"
        || source.slice(*yield_value) != "suffix"
        || exact_string(&final_value).as_deref() != Some("binary")
    {
        return Err(unsupported(
            source,
            span,
            "multi-input custom generator outside the retained value/suffix graph",
        ));
    }
    let value = CompilerParameter {
        name: String::from("value"),
        discarded: false,
        source_visible: true,
        value_type: CompilerType::Int,
        int_range: None,
        span: value_parameter.name,
    };
    let suffix = CompilerParameter {
        name: String::from("suffix"),
        discarded: false,
        source_visible: true,
        value_type: CompilerType::String,
        int_range: None,
        span: suffix_parameter.name,
    };
    let prefix_value = CompilerExpression {
        kind: CompilerExpressionKind::Local(value.name.clone()),
        value_type: CompilerType::Int,
        int_range: None,
        rational_value: None,
        span: *prefix_value,
    };
    let one = CompilerExpression {
        kind: CompilerExpressionKind::Int(BigInt::from(1)),
        value_type: CompilerType::Int,
        int_range: Some(IntRange::exact(BigInt::from(1))),
        rational_value: None,
        span: *increment,
    };
    Ok(GeneratorSource {
        name,
        span,
        initial_parameter: value,
        additional_initial_parameters: vec![suffix.clone()],
        yield_parameter: 1,
        prefix: CompilerBlock {
            statements: vec![CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::Binary {
                    operation: CompilerBinary::Add,
                    left: Box::new(prefix_value),
                    right: Box::new(one),
                },
                value_type: CompilerType::Int,
                int_range: None,
                rational_value: None,
                span: *prefix_span,
            })],
            result: unit_expression(*prefix_span),
        },
        literal_characters: None,
        value_yields: Some(vec![CompilerGeneratorYield::Value(Box::new(
            CompilerExpression {
                kind: CompilerExpressionKind::Local(suffix.name.clone()),
                value_type: CompilerType::String,
                int_range: None,
                rational_value: None,
                span: *yield_value,
            },
        ))]),
        value_continuations: Vec::new(),
        explicit_return: None,
        yield_count: 1,
        local: None,
        local_function: None,
        close_handler: None,
        result: final_value,
    })
}

fn exact_unary_int_string_generator_overload(
    source: &SourceText,
    name: Span,
    parameter: &FunctionParameter,
    body: &[Statement],
    span: Span,
) -> Result<GeneratorSource, Diagnostic> {
    let [
        Statement::Discard {
            value:
                Expression::Application {
                    items: yield_items,
                    span: yield_span,
                },
            ..
        },
        Statement::Expression(final_expression),
    ] = body
    else {
        return Err(unsupported(
            source,
            span,
            "unary overloaded generator outside one value yield and final String",
        ));
    };
    let [
        Expression::Identifier(yield_operation),
        Expression::Identifier(yield_value),
    ] = yield_items.as_slice()
    else {
        return Err(unsupported(
            source,
            *yield_span,
            "unary overloaded generator yield outside value",
        ));
    };
    let final_value = exact_string_literal_expression(source, final_expression.span())?;
    let final_string = exact_string(&final_value);
    let exact_shape = (source.slice(name) == "select"
        && source.slice(parameter.name) == "value"
        && final_string.as_deref() == Some("unary"))
        || (source.slice(name) == "numbers"
            && source.slice(parameter.name) == "initial"
            && final_string.as_deref() == Some("done"));
    if source.slice(*yield_operation) != "yield"
        || source.slice(*yield_value) != source.slice(parameter.name)
        || !exact_shape
    {
        return Err(unsupported(
            source,
            span,
            "unary Int-to-String generator outside an admitted overload or function-boundary graph",
        ));
    }
    Ok(GeneratorSource {
        name,
        span,
        initial_parameter: CompilerParameter {
            name: source.slice(parameter.name).to_owned(),
            discarded: false,
            source_visible: true,
            value_type: CompilerType::Int,
            int_range: None,
            span: parameter.name,
        },
        additional_initial_parameters: Vec::new(),
        yield_parameter: 0,
        prefix: CompilerBlock {
            statements: Vec::new(),
            result: unit_expression(span),
        },
        literal_characters: None,
        value_yields: Some(vec![CompilerGeneratorYield::Initial(*yield_span)]),
        value_continuations: Vec::new(),
        explicit_return: None,
        yield_count: 1,
        local: None,
        local_function: None,
        close_handler: None,
        result: final_value,
    })
}
