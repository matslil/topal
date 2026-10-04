#[allow(clippy::too_many_lines)] // Exact local declarations and fail-closed shape checks stay together.
fn exact_boolean_local_function_generator_body(
    source: &SourceText,
    parameter: &FunctionParameter,
    body: &[Statement],
    span: Span,
) -> Result<(ExactValueGeneratorBody, CompilerFunction), Diagnostic> {
    let [
        enum_statement,
        function_statement,
        yield_statement,
        final_statement,
    ] = body
    else {
        return Err(unsupported(
            source,
            span,
            "Boolean-yield generator-local declaration body outside one enum, function, initial yield, and final call",
        ));
    };
    let Some(EnumSource {
        name: enum_name,
        alternatives,
        ..
    }) = enum_declaration(source, enum_statement)
    else {
        return Err(unsupported(
            source,
            span,
            "generator-local enum declaration",
        ));
    };
    if source.slice(enum_name) != "Choice"
        || alternatives.len() != 2
        || alternatives[0].0 != "Accepted"
        || alternatives[1].0 != "Rejected"
    {
        return Err(unsupported(
            source,
            enum_name,
            "generator-local enum outside Choice (Accepted, Rejected)",
        ));
    }
    let enumeration = CompilerEnumType {
        name: String::from("Choice"),
        alternatives: vec![String::from("Accepted"), String::from("Rejected")],
    };
    let Statement::Function {
        name: function_name,
        is_static: false,
        parameters,
        result,
        effect_bound: None,
        clauses,
        body: function_body,
        span: function_span,
    } = function_statement
    else {
        return Err(unsupported(
            source,
            statement_span(function_statement),
            "generator-local function declaration",
        ));
    };
    let [function_parameter] = parameters.as_slice() else {
        return Err(unsupported(
            source,
            *function_span,
            "generator-local function arity",
        ));
    };
    if source.slice(*function_name) != "label"
        || !function_parameter.fields.is_empty()
        || function_parameter.default.is_some()
        || function_parameter.qualifier.is_some()
        || source.slice(function_parameter.name) != "value"
        || source.slice(function_parameter.classifier) != "Choice"
        || source.slice(*result) != "String"
        || **clauses != FunctionClauses::default()
    {
        return Err(unsupported(
            source,
            *function_span,
            "generator-local function outside label (value : Choice) -> String",
        ));
    }
    let [
        Statement::Expression(Expression::DecisionTable {
            subject,
            rules,
            span: decision_span,
        }),
    ] = function_body.as_slice()
    else {
        return Err(unsupported(
            source,
            *function_span,
            "generator-local label body",
        ));
    };
    let Expression::Identifier(subject_name) = subject.as_ref() else {
        return Err(unsupported(
            source,
            subject.span(),
            "generator-local label decision subject",
        ));
    };
    let [accepted_rule, rejected_rule] = rules.as_slice() else {
        return Err(unsupported(
            source,
            *decision_span,
            "generator-local label decision rules",
        ));
    };
    let (
        DecisionMatcher::Identifier(accepted_matcher),
        Expression::String(accepted_literal),
        DecisionMatcher::Identifier(rejected_matcher),
        Expression::String(rejected_literal),
    ) = (
        &accepted_rule.matcher,
        &accepted_rule.action,
        &rejected_rule.matcher,
        &rejected_rule.action,
    )
    else {
        return Err(unsupported(
            source,
            *decision_span,
            "generator-local label decision shape",
        ));
    };
    let accepted = exact_string_literal_expression(source, *accepted_literal)?;
    let rejected = exact_string_literal_expression(source, *rejected_literal)?;
    if source.slice(*subject_name) != "value"
        || source.slice(*accepted_matcher) != "Accepted"
        || source.slice(*rejected_matcher) != "Rejected"
        || exact_string(&accepted).as_deref() != Some("accepted")
        || exact_string(&rejected).as_deref() != Some("rejected")
    {
        return Err(unsupported(
            source,
            *decision_span,
            "generator-local label decision outside the retained Choice mapping",
        ));
    }
    let Statement::Discard {
        value:
            Expression::Application {
                items: yield_items,
                span: yield_span,
            },
        ..
    } = yield_statement
    else {
        return Err(unsupported(
            source,
            statement_span(yield_statement),
            "generator-local initial yield",
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
            "generator-local initial yield",
        ));
    };
    if source.slice(*yield_operation) != "yield"
        || source.slice(*yield_value) != source.slice(parameter.name)
    {
        return Err(unsupported(
            source,
            *yield_span,
            "generator-local initial yield",
        ));
    }
    let Statement::Expression(Expression::Application {
        items: final_items,
        span: final_span,
    }) = final_statement
    else {
        return Err(unsupported(
            source,
            statement_span(final_statement),
            "generator-local final call",
        ));
    };
    let [
        Expression::Identifier(final_function),
        Expression::Identifier(final_argument),
    ] = final_items.as_slice()
    else {
        return Err(unsupported(
            source,
            *final_span,
            "generator-local final call",
        ));
    };
    if source.slice(*final_function) != "label" || source.slice(*final_argument) != "Accepted" {
        return Err(unsupported(
            source,
            *final_span,
            "generator-local final call outside label Accepted",
        ));
    }
    let local_parameter = CompilerParameter {
        name: String::from("value"),
        discarded: false,
        source_visible: true,
        value_type: CompilerType::Enum(enumeration.clone()),
        int_range: None,
        span: function_parameter.name,
    };
    let function = CompilerFunction {
        source_name: String::from("label"),
        module_identity: None,
        symbol: String::new(),
        parameters: vec![local_parameter.clone()],
        pattern_identities: Vec::new(),
        result_type: CompilerType::String,
        result_captures: Vec::new(),
        body: CompilerBlock {
            statements: Vec::new(),
            result: CompilerExpression {
                kind: CompilerExpressionKind::EnumDecision {
                    subject: Box::new(CompilerExpression {
                        kind: CompilerExpressionKind::Local(local_parameter.name.clone()),
                        value_type: local_parameter.value_type.clone(),
                        int_range: None,
                        rational_value: None,
                        span: *subject_name,
                    }),
                    rules: vec![
                        CompilerEnumRule {
                            value: 0,
                            action: accepted,
                            span: accepted_rule.span,
                        },
                        CompilerEnumRule {
                            value: 1,
                            action: rejected,
                            span: rejected_rule.span,
                        },
                    ],
                    otherwise: None,
                },
                value_type: CompilerType::String,
                int_range: None,
                rational_value: None,
                span: *decision_span,
            },
        },
        span: *function_span,
        is_static: false,
        declared_effects: None,
        foreign: None,
    };
    let result = CompilerExpression {
        kind: CompilerExpressionKind::Call {
            symbol: String::new(),
            arguments: vec![CompilerExpression {
                kind: CompilerExpressionKind::Enum(0),
                value_type: CompilerType::Enum(enumeration),
                int_range: None,
                rational_value: None,
                span: *final_argument,
            }],
        },
        value_type: CompilerType::String,
        int_range: None,
        rational_value: None,
        span: *final_span,
    };
    Ok((
        ExactValueGeneratorBody {
            yields: vec![CompilerGeneratorYield::Initial(*yield_span)],
            continuations: Vec::new(),
            explicit_return: None,
            result,
        },
        function,
    ))
}

fn exact_boolean_generator_result(
    source: &SourceText,
    parameter: &FunctionParameter,
    result_classifier: Span,
    result_expression: &Expression,
    initial: CompilerExpression,
) -> Result<CompilerExpression, Diagnostic> {
    match source.slice(result_classifier) {
        "Boolean" => {
            let Expression::Application {
                items: result_items,
                span: result_span,
            } = result_expression
            else {
                return Err(unsupported(
                    source,
                    result_expression.span(),
                    "Boolean-value custom generator final value outside not initial",
                ));
            };
            let [
                Expression::Identifier(not_operation),
                Expression::Identifier(result_value),
            ] = result_items.as_slice()
            else {
                return Err(unsupported(
                    source,
                    *result_span,
                    "Boolean-value custom generator final value outside not initial",
                ));
            };
            if source.slice(*not_operation) != "not"
                || source.slice(*result_value) != source.slice(parameter.name)
            {
                return Err(unsupported(
                    source,
                    *result_span,
                    "Boolean-value custom generator outside yield initial followed by not initial",
                ));
            }
            Ok(CompilerExpression {
                kind: CompilerExpressionKind::Not(Box::new(initial)),
                value_type: CompilerType::Boolean,
                int_range: None,
                rational_value: None,
                span: *result_span,
            })
        }
        "String" => {
            exact_boolean_string_generator_decision(source, parameter, result_expression, initial)
        }
        _ => Err(unsupported(
            source,
            result_classifier,
            "Boolean-yield custom generator result outside Boolean or String",
        )),
    }
}

fn exact_boolean_string_generator_decision(
    source: &SourceText,
    parameter: &FunctionParameter,
    expression: &Expression,
    initial: CompilerExpression,
) -> Result<CompilerExpression, Diagnostic> {
    let Expression::DecisionTable {
        subject,
        rules,
        span,
    } = expression
    else {
        return Err(unsupported(
            source,
            expression.span(),
            "Boolean-yield custom generator final String outside its exact decision",
        ));
    };
    let Expression::Identifier(subject_name) = subject.as_ref() else {
        return Err(unsupported(
            source,
            subject.span(),
            "Boolean-yield custom generator final decision subject",
        ));
    };
    let [when_true_rule, otherwise_rule] = rules.as_slice() else {
        return Err(unsupported(
            source,
            *span,
            "Boolean-yield custom generator final decision rules",
        ));
    };
    let (
        DecisionMatcher::Boolean { value: true, .. },
        Expression::String(when_true_literal),
        DecisionMatcher::Otherwise(_),
        Expression::String(when_false_literal),
    ) = (
        &when_true_rule.matcher,
        &when_true_rule.action,
        &otherwise_rule.matcher,
        &otherwise_rule.action,
    )
    else {
        return Err(unsupported(
            source,
            *span,
            "Boolean-yield custom generator final decision outside true then String followed by otherwise String",
        ));
    };
    let when_true = exact_string_literal_expression(source, *when_true_literal)?;
    let when_false = exact_string_literal_expression(source, *when_false_literal)?;
    if source.slice(*subject_name) != source.slice(parameter.name)
        || exact_string(&when_true).as_deref() != Some("accepted")
        || exact_string(&when_false).as_deref() != Some("rejected")
    {
        return Err(unsupported(
            source,
            *span,
            "Boolean-yield custom generator final decision outside initial, accepted, and rejected",
        ));
    }
    Ok(CompilerExpression {
        kind: CompilerExpressionKind::BooleanDecision {
            subject: Box::new(CompilerExpression {
                span: *subject_name,
                ..initial
            }),
            when_true: Box::new(when_true),
            when_false: Box::new(when_false),
        },
        value_type: CompilerType::String,
        int_range: None,
        rational_value: None,
        span: *span,
    })
}

fn exact_int_comparison_expression(
    source: &SourceText,
    expression: &Expression,
    expected_left: i64,
    expected_right: i64,
    feature: &str,
) -> Result<CompilerExpression, Diagnostic> {
    let Expression::Application { items, span } = expression else {
        return Err(unsupported(source, expression.span(), feature));
    };
    let [
        Expression::Integer(left_span),
        Expression::Callable {
            kind: CallableKind::Compare,
            ..
        },
        Expression::Integer(right_span),
    ] = items.as_slice()
    else {
        return Err(unsupported(source, *span, feature));
    };
    let left_value = parse_integer(source.slice(*left_span));
    let right_value = parse_integer(source.slice(*right_span));
    if left_value.as_ref() != Some(&BigInt::from(expected_left))
        || right_value.as_ref() != Some(&BigInt::from(expected_right))
    {
        return Err(unsupported(source, *span, feature));
    }
    let left = CompilerExpression {
        kind: CompilerExpressionKind::Int(left_value.expect("checked integer literal exists")),
        value_type: CompilerType::Int,
        int_range: Some(IntRange::exact(BigInt::from(expected_left))),
        rational_value: None,
        span: *left_span,
    };
    let right = CompilerExpression {
        kind: CompilerExpressionKind::Int(right_value.expect("checked integer literal exists")),
        value_type: CompilerType::Int,
        int_range: Some(IntRange::exact(BigInt::from(expected_right))),
        rational_value: None,
        span: *right_span,
    };
    Ok(CompilerExpression {
        kind: CompilerExpressionKind::Binary {
            operation: CompilerBinary::Compare,
            left: Box::new(left),
            right: Box::new(right),
        },
        value_type: CompilerType::Comparison,
        int_range: None,
        rational_value: None,
        span: *span,
    })
}

fn exact_comparison_value_generator_body(
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
        Statement::Expression(result),
    ] = body
    else {
        return Err(unsupported(
            source,
            span,
            "Comparison-value custom generator outside one initial yield and final 3 <=> 2",
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
            "Comparison-value custom generator yield outside its initial parameter",
        ));
    };
    if source.slice(*yield_operation) != "yield"
        || source.slice(*yield_value) != source.slice(parameter.name)
    {
        return Err(unsupported(
            source,
            span,
            "Comparison-value custom generator outside yield initial followed by 3 <=> 2",
        ));
    }
    let result = exact_int_comparison_expression(
        source,
        result,
        3,
        2,
        "Comparison-value custom generator final expression outside 3 <=> 2",
    )?;
    Ok(ExactValueGeneratorBody {
        yields: vec![CompilerGeneratorYield::Initial(*yield_span)],
        continuations: Vec::new(),
        explicit_return: None,
        result,
    })
}

fn exact_int_comparison_value(expression: &CompilerExpression, left: i64, right: i64) -> bool {
    expression.value_type == CompilerType::Comparison
        && matches!(
            &expression.kind,
            CompilerExpressionKind::Binary {
                operation: CompilerBinary::Compare,
                left: actual_left,
                right: actual_right,
            } if exact_int(actual_left).as_ref() == Some(&BigInt::from(left))
                && exact_int(actual_right).as_ref() == Some(&BigInt::from(right))
        )
}

fn exact_comparison_value_generator_action(
    parameter: &CompilerParameter,
    body: &CompilerBlock,
) -> bool {
    parameter.value_type == CompilerType::Comparison
        && !parameter.discarded
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
                && left.value_type == CompilerType::Comparison
                && exact_int_comparison_value(right, 1, 2)
        )
        && matches!(body.result.kind, CompilerExpressionKind::Unit)
}

fn exact_enum_value_generator_body(
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
        Statement::Expression(Expression::Identifier(result_value)),
    ] = body
    else {
        return Err(unsupported(
            source,
            span,
            "Choice-value custom generator outside one initial yield and final Second",
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
            "Choice-value custom generator yield outside its initial parameter",
        ));
    };
    if enumeration.name != "Choice"
        || enumeration.alternatives != ["First", "Second"]
        || source.slice(*yield_operation) != "yield"
        || source.slice(*yield_value) != source.slice(parameter.name)
        || source.slice(*result_value) != "Second"
    {
        return Err(unsupported(
            source,
            span,
            "Choice-value custom generator outside yield initial followed by Second",
        ));
    }
    Ok(ExactValueGeneratorBody {
        yields: vec![CompilerGeneratorYield::Initial(*yield_span)],
        continuations: Vec::new(),
        explicit_return: None,
        result: CompilerExpression {
            kind: CompilerExpressionKind::Enum(1),
            value_type: CompilerType::Enum(enumeration.clone()),
            int_range: None,
            rational_value: None,
            span: *result_value,
        },
    })
}

fn exact_enum_value_generator_action(parameter: &CompilerParameter, body: &CompilerBlock) -> bool {
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
                && matches!(right.kind, CompilerExpressionKind::Enum(0))
                && right.value_type == parameter.value_type
        )
        && matches!(body.result.kind, CompilerExpressionKind::Unit)
}

fn exact_int_string_product_value_generator_body(
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
        Statement::Expression(Expression::Product {
            fields: result_fields,
            span: result_span,
        }),
    ] = body
    else {
        return Err(unsupported(
            source,
            span,
            "(Int, String)-value custom generator outside one initial yield and final (8, \"done\")",
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
            "(Int, String)-value custom generator yield outside its initial parameter",
        ));
    };
    let [int_field, text_field] = result_fields.as_slice() else {
        return Err(unsupported(
            source,
            *result_span,
            "(Int, String)-value custom generator final product arity",
        ));
    };
    let (Expression::Integer(int_literal), Expression::String(text_literal)) =
        (&int_field.value, &text_field.value)
    else {
        return Err(unsupported(
            source,
            *result_span,
            "(Int, String)-value custom generator final product fields",
        ));
    };
    let text = exact_string_literal_expression(source, *text_literal)?;
    if source.slice(*yield_operation) != "yield"
        || source.slice(*yield_value) != source.slice(parameter.name)
        || int_field.label.is_some()
        || text_field.label.is_some()
        || source.slice(*int_literal) != "8"
        || exact_string(&text).as_deref() != Some("done")
    {
        return Err(unsupported(
            source,
            span,
            "(Int, String)-value custom generator outside yield initial followed by (8, \"done\")",
        ));
    }
    let int = CompilerExpression {
        kind: CompilerExpressionKind::Int(BigInt::from(8)),
        value_type: CompilerType::Int,
        int_range: Some(IntRange::exact(BigInt::from(8))),
        rational_value: None,
        span: *int_literal,
    };
    let value_type = CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String]);
    Ok(ExactValueGeneratorBody {
        yields: vec![CompilerGeneratorYield::Initial(*yield_span)],
        continuations: Vec::new(),
        explicit_return: None,
        result: CompilerExpression {
            kind: CompilerExpressionKind::Tuple(vec![int, text]),
            value_type,
            int_range: None,
            rational_value: None,
            span: *result_span,
        },
    })
}

fn exact_int_string_product_value_generator_action(
    parameter: &CompilerParameter,
    body: &CompilerBlock,
) -> bool {
    parameter.value_type == CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String])
        && !parameter.discarded
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
                && left.value_type == parameter.value_type
                && matches!(
                    &right.kind,
                    CompilerExpressionKind::Tuple(fields)
                        if matches!(fields.as_slice(), [int, text]
                            if exact_int(int).as_ref() == Some(&BigInt::from(7))
                                && exact_string(text).as_deref() == Some("item"))
                )
                && right.value_type == parameter.value_type
        )
        && matches!(body.result.kind, CompilerExpressionKind::Unit)
}

fn exact_int_string_product(
    expression: &CompilerExpression,
    expected_int: i64,
    expected_text: &str,
) -> bool {
    matches!(
        &expression.kind,
        CompilerExpressionKind::Tuple(fields)
            if matches!(fields.as_slice(), [int, text]
                if exact_int(int) == Some(BigInt::from(expected_int))
                    && exact_string(text).as_deref() == Some(expected_text))
    ) && expression.value_type == CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String])
}

fn exact_singleton_int_list(expression: &CompilerExpression, expected: i64) -> bool {
    expression.value_type == int_list_type()
        && matches!(
            &expression.kind,
            CompilerExpressionKind::ListEntry { value, remaining }
                if exact_int(value) == Some(BigInt::from(expected))
                    && remaining.value_type == int_list_type()
                    && matches!(remaining.kind, CompilerExpressionKind::ListEmpty)
        )
}

fn exact_int_list_append(expression: &CompilerExpression, list_name: &str, expected: i64) -> bool {
    expression.value_type == int_list_type()
        && matches!(
            &expression.kind,
            CompilerExpressionKind::ListAppend { list, value }
                if list.value_type == int_list_type()
                    && matches!(&list.kind, CompilerExpressionKind::Local(name)
                        if name == list_name)
                    && exact_int(value) == Some(BigInt::from(expected))
        )
}

fn exact_list_value_generator_body(
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
            "List-Int-value custom generator outside one initial yield and final initial append 9",
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
            "List-Int-value custom generator yield outside its initial parameter",
        ));
    };
    let [
        Expression::Identifier(result_list),
        Expression::Identifier(append),
        Expression::Integer(appended),
    ] = result_items.as_slice()
    else {
        return Err(unsupported(
            source,
            *result_span,
            "List-Int-value custom generator final expression outside initial append 9",
        ));
    };
    let parameter_name = source.slice(parameter.name);
    if source.slice(*yield_operation) != "yield"
        || source.slice(*yield_value) != parameter_name
        || source.slice(*result_list) != parameter_name
        || source.slice(*append) != "append"
        || source.slice(*appended) != "9"
    {
        return Err(unsupported(
            source,
            span,
            "List-Int-value custom generator outside yield initial followed by initial append 9",
        ));
    }
    let list_type = int_list_type();
    let list = CompilerExpression {
        kind: CompilerExpressionKind::Local(parameter_name.to_owned()),
        value_type: list_type.clone(),
        int_range: None,
        rational_value: None,
        span: *result_list,
    };
    let value = CompilerExpression {
        kind: CompilerExpressionKind::Int(BigInt::from(9)),
        value_type: CompilerType::Int,
        int_range: Some(IntRange::exact(BigInt::from(9))),
        rational_value: None,
        span: *appended,
    };
    Ok(ExactValueGeneratorBody {
        yields: vec![CompilerGeneratorYield::Initial(*yield_span)],
        continuations: Vec::new(),
        explicit_return: None,
        result: CompilerExpression {
            kind: CompilerExpressionKind::ListAppend {
                list: Box::new(list),
                value: Box::new(value),
            },
            value_type: list_type,
            int_range: None,
            rational_value: None,
            span: *result_span,
        },
    })
}

fn exact_list_value_generator_action(parameter: &CompilerParameter, body: &CompilerBlock) -> bool {
    parameter.value_type == int_list_type()
        && !parameter.discarded
        && matches!(
            body.statements.as_slice(),
            [CompilerStatement::Discard(CompilerExpression {
                kind: CompilerExpressionKind::ListEntryCount(value),
                ..
            })] if value.value_type == parameter.value_type
                && matches!(&value.kind, CompilerExpressionKind::Local(name)
                    if name == &parameter.name)
        )
        && matches!(body.result.kind, CompilerExpressionKind::Unit)
}

fn exact_int_string_product_literal(
    source: &SourceText,
    expression: &Expression,
    expected_int: i64,
    expected_text: &str,
) -> Option<CompilerExpression> {
    let Expression::Product { fields, span } = expression else {
        return None;
    };
    let [int_field, text_field] = fields.as_slice() else {
        return None;
    };
    let (Expression::Integer(int_span), Expression::String(text_span)) =
        (&int_field.value, &text_field.value)
    else {
        return None;
    };
    let int_value = BigInt::from(expected_int);
    let text = parse_string(source.slice(*text_span))?;
    if int_field.label.is_some()
        || text_field.label.is_some()
        || parse_integer(source.slice(*int_span)).as_ref() != Some(&int_value)
        || text != expected_text
    {
        return None;
    }
    Some(CompilerExpression {
        kind: CompilerExpressionKind::Tuple(vec![
            CompilerExpression {
                kind: CompilerExpressionKind::Int(int_value.clone()),
                value_type: CompilerType::Int,
                int_range: Some(IntRange::exact(int_value)),
                rational_value: None,
                span: *int_span,
            },
            CompilerExpression {
                kind: CompilerExpressionKind::String(text.to_owned()),
                value_type: CompilerType::String,
                int_range: None,
                rational_value: None,
                span: *text_span,
            },
        ]),
        value_type: CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String]),
        int_range: None,
        rational_value: None,
        span: *span,
    })
}

fn exact_optional_int_string_literal(
    source: &SourceText,
    expression: &Expression,
    expected_int: i64,
    expected_text: &str,
) -> Option<CompilerExpression> {
    let Expression::Application { items, span } = expression else {
        return None;
    };
    let [Expression::Identifier(constructor), payload] = items.as_slice() else {
        return None;
    };
    if source.slice(*constructor) != "Some" {
        return None;
    }
    let payload = exact_int_string_product_literal(source, payload, expected_int, expected_text)?;
    Some(CompilerExpression {
        kind: CompilerExpressionKind::OptionalSome(Box::new(payload)),
        value_type: CompilerType::Optional(Box::new(CompilerType::Tuple(vec![
            CompilerType::Int,
            CompilerType::String,
        ]))),
        int_range: None,
        rational_value: None,
        span: *span,
    })
}

fn exact_optional_int_string_none_literal(
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
    let [int_field, string_field] = fields.as_slice() else {
        return None;
    };
    let (Expression::Identifier(int_type), Expression::Identifier(string_type)) =
        (&int_field.value, &string_field.value)
    else {
        return None;
    };
    if source.slice(*constructor) != "None"
        || int_field.label.is_some()
        || string_field.label.is_some()
        || source.slice(*int_type) != "Int"
        || source.slice(*string_type) != "String"
    {
        return None;
    }
    Some(CompilerExpression {
        kind: CompilerExpressionKind::OptionalNone,
        value_type: CompilerType::Optional(Box::new(CompilerType::Tuple(vec![
            CompilerType::Int,
            CompilerType::String,
        ]))),
        int_range: None,
        rational_value: None,
        span: *span,
    })
}

fn exact_optional_int_string_value(
    expression: &CompilerExpression,
    expected_int: i64,
    expected_text: &str,
) -> bool {
    expression.value_type
        == CompilerType::Optional(Box::new(CompilerType::Tuple(vec![
            CompilerType::Int,
            CompilerType::String,
        ])))
        && matches!(&expression.kind, CompilerExpressionKind::OptionalSome(payload)
            if matches!(&payload.kind, CompilerExpressionKind::Tuple(fields)
                if matches!(fields.as_slice(), [int, text]
                    if exact_int(int).as_ref() == Some(&BigInt::from(expected_int))
                        && exact_string(text).as_deref() == Some(expected_text))))
}

fn exact_optional_int_string_none_value(expression: &CompilerExpression) -> bool {
    expression.value_type
        == CompilerType::Optional(Box::new(CompilerType::Tuple(vec![
            CompilerType::Int,
            CompilerType::String,
        ])))
        && matches!(expression.kind, CompilerExpressionKind::OptionalNone)
}

fn exact_nested_optional_value_generator_body(
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
        Statement::Expression(result),
    ] = body
    else {
        return Err(unsupported(
            source,
            span,
            "nested Optional-value custom generator outside one initial yield and final Some (8, \"done\")",
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
            "nested Optional-value custom generator yield outside its initial parameter",
        ));
    };
    if source.slice(*yield_operation) != "yield"
        || source.slice(*yield_value) != source.slice(parameter.name)
    {
        return Err(unsupported(
            source,
            span,
            "nested Optional-value custom generator outside yield initial followed by Some (8, \"done\")",
        ));
    }
    let result = exact_optional_int_string_literal(source, result, 8, "done")
        .or_else(|| exact_optional_int_string_none_literal(source, result))
        .ok_or_else(|| {
            unsupported(
                source,
                result.span(),
                "nested Optional-value custom generator final expression outside Some (8, \"done\") or None (Int, String)",
            )
        })?;
    Ok(ExactValueGeneratorBody {
        yields: vec![CompilerGeneratorYield::Initial(*yield_span)],
        continuations: Vec::new(),
        explicit_return: None,
        result,
    })
}

fn exact_nested_optional_value_generator_action(
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
                && left.value_type == parameter.value_type
                && exact_optional_int_string_value(right, 7, "item")
        )
        && matches!(body.result.kind, CompilerExpressionKind::Unit)
}

fn exact_nested_optional_value_generator_foreach_body(
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
            "nested Optional-value custom generator foreach action",
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
            "nested Optional-value custom generator foreach equality action",
        ));
    };
    if parameter.discarded || source.slice(*left) != parameter.name {
        return Err(unsupported(
            source,
            *expression_span,
            "nested Optional-value custom generator foreach named candidate action",
        ));
    }
    let right = exact_optional_int_string_literal(source, right, 7, "item")
        .or_else(|| exact_optional_int_string_none_literal(source, right))
        .ok_or_else(|| {
            unsupported(
                source,
                right.span(),
                "nested Optional-value custom generator foreach comparison operand",
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

fn exact_nested_none_value_generator_action(
    parameter: &CompilerParameter,
    body: &CompilerBlock,
) -> bool {
    !parameter.discarded
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
            && exact_optional_int_string_none_value(right))
        && matches!(body.result.kind, CompilerExpressionKind::Unit)
}

fn nested_result_product_type() -> CompilerType {
    CompilerType::Result(Box::new(CompilerType::Tuple(vec![
        CompilerType::Int,
        CompilerType::String,
    ])))
}

fn exact_nested_result_product_literal(
    source: &SourceText,
    expression: &Expression,
    expected_int: i64,
    expected_text: &str,
) -> Option<CompilerExpression> {
    let span = expression.span();
    let payload =
        exact_int_string_product_literal(source, expression, expected_int, expected_text)?;
    Some(CompilerExpression {
        kind: CompilerExpressionKind::ResultSuccess(Box::new(payload)),
        value_type: nested_result_product_type(),
        int_range: None,
        rational_value: None,
        span,
    })
}

fn exact_nested_result_product_value(
    expression: &CompilerExpression,
    expected_int: i64,
    expected_text: &str,
) -> bool {
    expression.value_type == nested_result_product_type()
        && matches!(&expression.kind, CompilerExpressionKind::ResultSuccess(payload)
            if matches!(&payload.kind, CompilerExpressionKind::Tuple(fields)
                if matches!(fields.as_slice(), [int, text]
                    if exact_int(int).as_ref() == Some(&BigInt::from(expected_int))
                        && exact_string(text).as_deref() == Some(expected_text))))
}

fn exact_nested_result_value_generator_body(
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
        Statement::Expression(result),
    ] = body
    else {
        return Err(unsupported(
            source,
            span,
            "nested Result-value custom generator outside one initial yield and final (8, \"done\")",
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
            "nested Result-value custom generator yield outside its initial parameter",
        ));
    };
    if source.slice(*yield_operation) != "yield"
        || source.slice(*yield_value) != source.slice(parameter.name)
    {
        return Err(unsupported(
            source,
            span,
            "nested Result-value custom generator outside yield initial followed by (8, \"done\")",
        ));
    }
    let result =
        exact_nested_result_product_literal(source, result, 8, "done").ok_or_else(|| {
            unsupported(
                source,
                result.span(),
                "nested Result-value custom generator final expression outside (8, \"done\")",
            )
        })?;
    Ok(ExactValueGeneratorBody {
        yields: vec![CompilerGeneratorYield::Initial(*yield_span)],
        continuations: Vec::new(),
        explicit_return: None,
        result,
    })
}

fn exact_nested_result_value_generator_foreach_body(
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
            "nested Result-value custom generator foreach action outside discarded candidate = (7, \"item\")",
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
            "nested Result-value custom generator foreach action outside discarded candidate = (7, \"item\")",
        ));
    };
    if parameter.discarded || source.slice(*left) != parameter.name {
        return Err(unsupported(
            source,
            *expression_span,
            "nested Result-value custom generator foreach action outside discarded candidate = (7, \"item\")",
        ));
    }
    let right = exact_nested_result_product_literal(source, right, 7, "item").ok_or_else(|| {
        unsupported(
            source,
            right.span(),
            "nested Result-value custom generator foreach comparison outside (7, \"item\")",
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

fn exact_nested_result_value_generator_action(
    parameter: &CompilerParameter,
    body: &CompilerBlock,
) -> bool {
    parameter.value_type == nested_result_product_type()
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
            && exact_nested_result_product_value(right, 7, "item"))
        && matches!(body.result.kind, CompilerExpressionKind::Unit)
}

const RECURSIVE_NOMINAL_GENERATOR_CLASSIFIER: &str =
    "(OptionalChoice,Result(Choice,langarithmeticArithmeticErrorCode))";

fn recursive_nominal_generator_type(enumeration: &CompilerEnumType) -> CompilerType {
    CompilerType::Tuple(vec![
        CompilerType::Optional(Box::new(CompilerType::Enum(enumeration.clone()))),
        CompilerType::Result(Box::new(CompilerType::Enum(enumeration.clone()))),
    ])
}

fn recursive_nominal_enumeration(value_type: &CompilerType) -> Option<&CompilerEnumType> {
    let CompilerType::Tuple(fields) = value_type else {
        return None;
    };
    let [
        CompilerType::Optional(optional),
        CompilerType::Result(result),
    ] = fields.as_slice()
    else {
        return None;
    };
    let (CompilerType::Enum(optional), CompilerType::Enum(result)) =
        (optional.as_ref(), result.as_ref())
    else {
        return None;
    };
    (optional == result
        && optional.name == "Choice"
        && optional.alternatives == ["First", "Second"])
    .then_some(optional)
}

fn exact_recursive_nominal_literal(
    source: &SourceText,
    expression: &Expression,
    enumeration: &CompilerEnumType,
    expected_alternative: &str,
) -> Option<CompilerExpression> {
    let Expression::Product { fields, span } = expression else {
        return None;
    };
    let [optional_field, result_field] = fields.as_slice() else {
        return None;
    };
    let Expression::Application {
        items: optional_items,
        span: optional_span,
    } = &optional_field.value
    else {
        return None;
    };
    let [
        Expression::Identifier(some),
        Expression::Identifier(optional_alternative),
    ] = optional_items.as_slice()
    else {
        return None;
    };
    let Expression::Identifier(result_alternative) = result_field.value else {
        return None;
    };
    let alternative = u32::try_from(
        enumeration
            .alternatives
            .iter()
            .position(|alternative| alternative == expected_alternative)?,
    )
    .ok()?;
    if enumeration.name != "Choice"
        || enumeration.alternatives != ["First", "Second"]
        || optional_field.label.is_some()
        || result_field.label.is_some()
        || source.slice(*some) != "Some"
        || source.slice(*optional_alternative) != expected_alternative
        || source.slice(result_alternative) != expected_alternative
    {
        return None;
    }
    let optional_payload = CompilerExpression {
        kind: CompilerExpressionKind::Enum(alternative),
        value_type: CompilerType::Enum(enumeration.clone()),
        int_range: None,
        rational_value: None,
        span: *optional_alternative,
    };
    let optional = CompilerExpression {
        kind: CompilerExpressionKind::OptionalSome(Box::new(optional_payload)),
        value_type: CompilerType::Optional(Box::new(CompilerType::Enum(enumeration.clone()))),
        int_range: None,
        rational_value: None,
        span: *optional_span,
    };
    let result_payload = CompilerExpression {
        kind: CompilerExpressionKind::Enum(alternative),
        value_type: CompilerType::Enum(enumeration.clone()),
        int_range: None,
        rational_value: None,
        span: result_alternative,
    };
    let result = CompilerExpression {
        kind: CompilerExpressionKind::ResultSuccess(Box::new(result_payload)),
        value_type: CompilerType::Result(Box::new(CompilerType::Enum(enumeration.clone()))),
        int_range: None,
        rational_value: None,
        span: result_alternative,
    };
    Some(CompilerExpression {
        kind: CompilerExpressionKind::Tuple(vec![optional, result]),
        value_type: recursive_nominal_generator_type(enumeration),
        int_range: None,
        rational_value: None,
        span: *span,
    })
}

fn exact_recursive_nominal_value(expression: &CompilerExpression, alternative: u32) -> bool {
    recursive_nominal_enumeration(&expression.value_type).is_some()
        && matches!(&expression.kind, CompilerExpressionKind::Tuple(fields)
            if matches!(fields.as_slice(), [optional, result]
                if matches!(&optional.kind, CompilerExpressionKind::OptionalSome(payload)
                    if matches!(payload.kind, CompilerExpressionKind::Enum(value)
                        if value == alternative))
                && matches!(&result.kind, CompilerExpressionKind::ResultSuccess(payload)
                    if matches!(payload.kind, CompilerExpressionKind::Enum(value)
                        if value == alternative))))
}
