fn function_body_context_member_span(
    source: &SourceText,
    statements: &[Statement],
    member_name: &str,
) -> Option<Span> {
    statements
        .iter()
        .find_map(|statement| statement_context_member_span(source, statement, member_name))
}

fn statement_context_member_span(
    source: &SourceText,
    statement: &Statement,
    member_name: &str,
) -> Option<Span> {
    match statement {
        Statement::Published { declaration, .. } => {
            statement_context_member_span(source, declaration, member_name)
        }
        Statement::Binding { value, .. }
        | Statement::ContextAssignment { value, .. }
        | Statement::Discard { value, .. }
        | Statement::Return { value, .. }
        | Statement::Expression(value) => {
            expression_context_member_span(source, value, member_name)
        }
        _ => None,
    }
}

fn expression_context_member_span(
    source: &SourceText,
    expression: &Expression,
    member_name: &str,
) -> Option<Span> {
    match expression {
        Expression::ContextIdentifier(span) if source.slice(*span) == member_name => Some(*span),
        Expression::Block { statements, .. } => {
            function_body_context_member_span(source, statements, member_name)
        }
        Expression::Product { fields, .. } => fields
            .iter()
            .find_map(|field| expression_context_member_span(source, &field.value, member_name)),
        Expression::DecisionTable { subject, rules, .. } => {
            expression_context_member_span(source, subject, member_name).or_else(|| {
                rules.iter().find_map(|rule| {
                    let matcher = match &rule.matcher {
                        DecisionMatcher::Comparison { operand, .. } => {
                            expression_context_member_span(source, operand, member_name)
                        }
                        _ => None,
                    };
                    matcher.or_else(|| {
                        expression_context_member_span(source, &rule.action, member_name)
                    })
                })
            })
        }
        Expression::Application { items, .. } => items
            .iter()
            .find_map(|item| expression_context_member_span(source, item, member_name)),
        Expression::AnonymousFunction { body, .. } => {
            expression_context_member_span(source, body, member_name)
        }
        Expression::Unit(_)
        | Expression::Boolean(_)
        | Expression::Integer(_)
        | Expression::Infinity(_)
        | Expression::Measured { .. }
        | Expression::Rational(_)
        | Expression::String(_)
        | Expression::Identifier(_)
        | Expression::ContextIdentifier(_)
        | Expression::Discard(_)
        | Expression::Callable { .. } => None,
    }
}

fn function_body_root_member_span(
    source: &SourceText,
    statements: &[Statement],
    member_name: &str,
) -> Option<Span> {
    statements
        .iter()
        .find_map(|statement| statement_root_member_span(source, statement, member_name))
}

fn statement_root_member_span(
    source: &SourceText,
    statement: &Statement,
    member_name: &str,
) -> Option<Span> {
    match statement {
        Statement::Published { declaration, .. } => {
            statement_root_member_span(source, declaration, member_name)
        }
        Statement::Binding { value, .. }
        | Statement::ContextAssignment { value, .. }
        | Statement::Discard { value, .. }
        | Statement::Return { value, .. }
        | Statement::Expression(value) => expression_root_member_span(source, value, member_name),
        _ => None,
    }
}

fn expression_root_member_span(
    source: &SourceText,
    expression: &Expression,
    member_name: &str,
) -> Option<Span> {
    match expression {
        Expression::Block { statements, .. } => {
            function_body_root_member_span(source, statements, member_name)
        }
        Expression::Product { fields, .. } => fields
            .iter()
            .find_map(|field| expression_root_member_span(source, &field.value, member_name)),
        Expression::DecisionTable { subject, rules, .. } => {
            expression_root_member_span(source, subject, member_name).or_else(|| {
                rules.iter().find_map(|rule| {
                    let matcher = match &rule.matcher {
                        DecisionMatcher::Comparison { operand, .. } => {
                            expression_root_member_span(source, operand, member_name)
                        }
                        _ => None,
                    };
                    matcher
                        .or_else(|| expression_root_member_span(source, &rule.action, member_name))
                })
            })
        }
        Expression::Application { items, .. } => {
            if let [Expression::Identifier(root), Expression::Identifier(member)] = items.as_slice()
                && source.slice(*root) == "root"
                && source.slice(*member) == member_name
            {
                return Some(*member);
            }
            items
                .iter()
                .find_map(|item| expression_root_member_span(source, item, member_name))
        }
        Expression::AnonymousFunction { body, .. } => {
            expression_root_member_span(source, body, member_name)
        }
        Expression::Unit(_)
        | Expression::Boolean(_)
        | Expression::Integer(_)
        | Expression::Infinity(_)
        | Expression::Measured { .. }
        | Expression::Rational(_)
        | Expression::String(_)
        | Expression::Identifier(_)
        | Expression::ContextIdentifier(_)
        | Expression::Discard(_)
        | Expression::Callable { .. } => None,
    }
}

fn function_body_called_function_references(
    source: &SourceText,
    functions: &BTreeMap<String, Vec<FunctionSource>>,
    declaration: &FunctionSource,
) -> Vec<CompilerFunctionCallReference> {
    let mut local_functions = BTreeMap::new();
    for parameter in &declaration.parameters {
        collect_parameter_function_bindings(source, parameter, &mut local_functions);
    }
    let mut references = Vec::new();
    collect_statement_function_calls(
        source,
        functions,
        &declaration.body,
        &mut local_functions,
        &mut references,
    );
    references
}

fn collect_parameter_function_bindings(
    source: &SourceText,
    parameter: &FunctionParameter,
    local_functions: &mut BTreeMap<String, Option<CompilerNamedCallTarget>>,
) {
    local_functions.insert(source.slice(parameter.name).to_owned(), None);
    for field in &parameter.fields {
        collect_parameter_function_bindings(source, field, local_functions);
    }
}

fn nested_named_call_target(statement: &Statement) -> Option<CompilerNamedCallTarget> {
    let Statement::Function {
        name,
        is_static,
        parameters,
        result,
        effect_bound,
        clauses,
        body,
        span,
    } = statement
    else {
        return None;
    };
    (!is_static && effect_bound.is_none() && **clauses == FunctionClauses::default()).then(|| {
        CompilerNamedCallTarget {
            declarations: vec![FunctionSource {
                name: *name,
                parameters: parameters.clone(),
                result: *result,
                effect_bound: None,
                declared_effects: None,
                body: body.clone(),
                span: *span,
                is_static: false,
                published: false,
                module_identity: None,
            }],
        }
    })
}

fn retained_named_call_target(
    source: &SourceText,
    functions: &BTreeMap<String, Vec<FunctionSource>>,
    local_functions: &BTreeMap<String, Option<CompilerNamedCallTarget>>,
    expression: &Expression,
) -> Option<CompilerNamedCallTarget> {
    let Expression::Identifier(name) = expression else {
        return None;
    };
    let name_text = source.slice(*name);
    if let Some(target) = local_functions.get(name_text) {
        return target.clone();
    }
    let declarations = functions
        .get(name_text)?
        .iter()
        .filter(|declaration| declaration.span.end <= name.start)
        .cloned()
        .collect::<Vec<_>>();
    (!declarations.is_empty()).then_some(CompilerNamedCallTarget { declarations })
}

fn collect_statement_function_calls(
    source: &SourceText,
    functions: &BTreeMap<String, Vec<FunctionSource>>,
    statements: &[Statement],
    local_functions: &mut BTreeMap<String, Option<CompilerNamedCallTarget>>,
    references: &mut Vec<CompilerFunctionCallReference>,
) {
    for statement in statements {
        match statement {
            Statement::Function { name, .. } => {
                local_functions.insert(
                    source.slice(*name).to_owned(),
                    nested_named_call_target(statement),
                );
            }
            Statement::Published { declaration, .. } => {
                if let Statement::Function { name, .. } = declaration.as_ref() {
                    local_functions.insert(source.slice(*name).to_owned(), None);
                }
            }
            _ => {}
        }
    }
    for statement in statements {
        match statement {
            Statement::Published { declaration, .. } => {
                if !matches!(declaration.as_ref(), Statement::Function { .. }) {
                    collect_statement_function_calls(
                        source,
                        functions,
                        std::slice::from_ref(declaration.as_ref()),
                        local_functions,
                        references,
                    );
                }
            }
            Statement::Binding { name, value, .. } => {
                if !matches!(value, Expression::Identifier(_)) {
                    collect_expression_function_calls(
                        source,
                        functions,
                        value,
                        local_functions,
                        references,
                    );
                }
                let target = retained_named_call_target(source, functions, local_functions, value);
                local_functions.insert(source.slice(*name).to_owned(), target);
            }
            Statement::ContextAssignment { value, .. }
            | Statement::Discard { value, .. }
            | Statement::Return { value, .. }
            | Statement::Expression(value) => {
                collect_expression_function_calls(
                    source,
                    functions,
                    value,
                    local_functions,
                    references,
                );
            }
            Statement::Foreach {
                result,
                source: iterated,
                binding,
                body,
                ..
            } => {
                collect_expression_function_calls(
                    source,
                    functions,
                    iterated,
                    local_functions,
                    references,
                );
                let mut body_local_functions = local_functions.clone();
                body_local_functions.insert(source.slice(*binding).to_owned(), None);
                if let Some((name, _)) = result {
                    body_local_functions.insert(source.slice(*name).to_owned(), None);
                }
                collect_statement_function_calls(
                    source,
                    functions,
                    body,
                    &mut body_local_functions,
                    references,
                );
            }
            Statement::Function { .. }
            | Statement::LanguageSelection { .. }
            | Statement::LibrarySelection { .. }
            | Statement::DiagnosticControl { .. }
            | Statement::Implementation { .. }
            | Statement::StateField { .. }
            | Statement::Generator { .. }
            | Statement::Union { .. }
            | Statement::Interface { .. }
            | Statement::InterfaceImplementation { .. } => {}
        }
    }
}

#[allow(clippy::too_many_lines)] // Recursive syntax coverage and exact call arguments stay co-located for capture discovery.
fn collect_expression_function_calls(
    source: &SourceText,
    functions: &BTreeMap<String, Vec<FunctionSource>>,
    expression: &Expression,
    local_functions: &BTreeMap<String, Option<CompilerNamedCallTarget>>,
    references: &mut Vec<CompilerFunctionCallReference>,
) {
    match expression {
        Expression::Block { statements, .. } => {
            let mut block_local_functions = local_functions.clone();
            collect_statement_function_calls(
                source,
                functions,
                statements,
                &mut block_local_functions,
                references,
            );
        }
        Expression::Product { fields, .. } => {
            for field in fields {
                collect_expression_function_calls(
                    source,
                    functions,
                    &field.value,
                    local_functions,
                    references,
                );
            }
        }
        Expression::DecisionTable { subject, rules, .. } => {
            collect_expression_function_calls(
                source,
                functions,
                subject,
                local_functions,
                references,
            );
            for rule in rules {
                let mut action_local_functions = local_functions.clone();
                match &rule.matcher {
                    DecisionMatcher::Union { binding, .. }
                    | DecisionMatcher::Variant { binding, .. }
                    | DecisionMatcher::Result { binding, .. }
                    | DecisionMatcher::Optional {
                        binding: Some(binding),
                        ..
                    } => {
                        action_local_functions.insert(source.slice(*binding).to_owned(), None);
                    }
                    DecisionMatcher::ListEntry { first, rest, .. } => {
                        action_local_functions.insert(source.slice(*first).to_owned(), None);
                        action_local_functions.insert(source.slice(*rest).to_owned(), None);
                    }
                    DecisionMatcher::Comparison { operand, .. } => {
                        collect_expression_function_calls(
                            source,
                            functions,
                            operand,
                            local_functions,
                            references,
                        );
                    }
                    DecisionMatcher::Boolean { .. }
                    | DecisionMatcher::Identifier(_)
                    | DecisionMatcher::Optional { binding: None, .. }
                    | DecisionMatcher::ListEmpty(_)
                    | DecisionMatcher::ErrorCode { .. }
                    | DecisionMatcher::Otherwise(_) => {}
                }
                collect_expression_function_calls(
                    source,
                    functions,
                    &rule.action,
                    &action_local_functions,
                    references,
                );
            }
        }
        Expression::Application { items, .. } => {
            let mut callable_indexes = BTreeSet::new();
            if let [
                Expression::Identifier(root),
                Expression::Identifier(member),
                ..,
            ] = items.as_slice()
                && source.slice(*root) == "root"
                && functions.contains_key(source.slice(*member))
            {
                let declarations = functions
                    .get(source.slice(*member))
                    .expect("checked root function exists")
                    .clone();
                references.push(CompilerFunctionCallReference {
                    declarations,
                    span: *member,
                    arguments: items[2..].to_vec(),
                    is_ordinary_unqualified: false,
                    is_retained_value: false,
                });
                callable_indexes.extend([0, 1]);
            } else if let Some(Expression::Identifier(alias)) = items.first()
                && let Some(Some(target)) = local_functions.get(source.slice(*alias))
            {
                references.push(CompilerFunctionCallReference {
                    declarations: target.declarations.clone(),
                    span: *alias,
                    arguments: items[1..].to_vec(),
                    is_ordinary_unqualified: true,
                    is_retained_value: false,
                });
                callable_indexes.insert(0);
            } else if let Some((function_index, span, declarations)) =
                items.iter().enumerate().find_map(|(index, item)| {
                    let Expression::Identifier(name) = item else {
                        return None;
                    };
                    let name_text = source.slice(*name);
                    (!local_functions.contains_key(name_text))
                        .then(|| functions.get(name_text))
                        .flatten()
                        .map(|declarations| (index, *name, declarations.clone()))
                })
            {
                callable_indexes.insert(function_index);
                references.push(CompilerFunctionCallReference {
                    declarations,
                    span,
                    arguments: items
                        .iter()
                        .enumerate()
                        .filter_map(|(index, item)| {
                            (index != function_index).then_some(item.clone())
                        })
                        .collect(),
                    is_ordinary_unqualified: true,
                    is_retained_value: false,
                });
            }
            for (index, item) in items.iter().enumerate() {
                if callable_indexes.contains(&index) {
                    continue;
                }
                collect_expression_function_calls(
                    source,
                    functions,
                    item,
                    local_functions,
                    references,
                );
            }
        }
        Expression::AnonymousFunction {
            parameters, body, ..
        } => {
            let mut anonymous_local_functions = local_functions.clone();
            let mut parameter_names = BTreeSet::new();
            for parameter in parameters {
                collect_anonymous_pattern_names(source, parameter, &mut parameter_names);
            }
            for parameter_name in parameter_names {
                anonymous_local_functions.insert(parameter_name.to_owned(), None);
            }
            collect_expression_function_calls(
                source,
                functions,
                body,
                &anonymous_local_functions,
                references,
            );
        }
        Expression::Identifier(name) => {
            if let Some(target) =
                retained_named_call_target(source, functions, local_functions, expression)
            {
                references.push(CompilerFunctionCallReference {
                    declarations: target.declarations,
                    span: *name,
                    arguments: Vec::new(),
                    is_ordinary_unqualified: true,
                    is_retained_value: true,
                });
            }
        }
        Expression::Unit(_)
        | Expression::Boolean(_)
        | Expression::Integer(_)
        | Expression::Infinity(_)
        | Expression::Measured { .. }
        | Expression::Rational(_)
        | Expression::String(_)
        | Expression::ContextIdentifier(_)
        | Expression::Discard(_)
        | Expression::Callable { .. } => {}
    }
}

fn compact_classifier(classifier: &str) -> String {
    classifier
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn function_overload_identity(
    source: &SourceText,
    name: &str,
    declaration: &FunctionSource,
) -> String {
    let inputs = declaration
        .parameters
        .iter()
        .map(|parameter| compact_classifier(source.slice(parameter.classifier)))
        .collect::<Vec<_>>()
        .join(",");
    let staticness = if declaration.is_static {
        "static"
    } else {
        "ordinary"
    };
    format!("{name}:{staticness}({inputs})")
}

fn fundamental_type_value(name: &str) -> Option<u32> {
    match name {
        "Boolean" => Some(0),
        "Int" => Some(1),
        "Nat" => Some(2),
        "Rational" => Some(3),
        "String" => Some(4),
        "Unit" => Some(5),
        "Scope" => Some(6),
        _ => None,
    }
}

fn fundamental_capability(name: &str) -> bool {
    matches!(
        name,
        "Equality" | "Ordering" | "Foldable" | "Membership" | "Indexed" | "Keyed"
    )
}

fn compiler_layout_policy(name: &str) -> Option<(CompilerEnumType, u32)> {
    let (type_name, alternatives): (&str, &[&str]) = match name {
        "Little" | "Big" => ("Endian", &["Little", "Big"]),
        "ReadWrite" | "ReadOnly" | "WriteOnly" | "Reserved" => (
            "Access",
            &["ReadWrite", "ReadOnly", "WriteOnly", "Reserved"],
        ),
        "MostSignificantFirst" | "LeastSignificantFirst" => (
            "BitOrder",
            &["MostSignificantFirst", "LeastSignificantFirst"],
        ),
        "Natural" | "Packed" => ("Packing", &["Natural", "Packed"]),
        "Declared" => ("FieldOrder", &["Declared"]),
        "AfterTag" | "Overlay" => ("PayloadPlacement", &["AfterTag", "Overlay"]),
        "NoLength" | "NoTerminator" => ("LayoutPolicy", &["NoLength", "NoTerminator"]),
        _ => return None,
    };
    let value = alternatives
        .iter()
        .position(|candidate| *candidate == name)?;
    Some((
        CompilerEnumType {
            name: type_name.to_owned(),
            alternatives: alternatives
                .iter()
                .map(|value| (*value).to_owned())
                .collect(),
        },
        u32::try_from(value).expect("fundamental layout policy count fits u32"),
    ))
}

fn compiler_int_string_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if matches!(fields.as_slice(), [
                CompilerType::Int | CompilerType::Nat,
                CompilerType::Character | CompilerType::String
            ])
    )
}

fn compiler_int_list_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if matches!(fields.as_slice(), [
                CompilerType::Int | CompilerType::Nat,
                CompilerType::List(element),
            ] if matches!(element.as_ref(), CompilerType::Int | CompilerType::Nat)
                || compiler_integer_pair(element.as_ref()))
    )
}

fn compiler_string_int_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if fields.as_slice() == [CompilerType::String, CompilerType::Int]
    )
}

fn compiler_string_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if fields.as_slice() == [CompilerType::String, CompilerType::String]
    )
}

fn compiler_boolean_int_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if fields.as_slice() == [CompilerType::Boolean, CompilerType::Int]
    )
}

fn compiler_boolean_string_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if fields.as_slice() == [CompilerType::Boolean, CompilerType::String]
    )
}

fn compiler_string_function_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if fields.as_slice() == [CompilerType::String, CompilerType::Function]
    )
}

fn compiler_int_triple(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if fields.as_slice()
                == [CompilerType::Int, CompilerType::Int, CompilerType::Int]
    )
}

fn compiler_string_string_rational_triple(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if fields.as_slice()
                == [CompilerType::String, CompilerType::String, CompilerType::Rational]
    )
}

fn compiler_int_int_list_triple(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if matches!(fields.as_slice(), [CompilerType::Int, CompilerType::Int, CompilerType::List(element)]
                if matches!(element.as_ref(), CompilerType::Int | CompilerType::Nat))
    )
}

fn compiler_string_rational_string_list_triple(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if matches!(fields.as_slice(), [CompilerType::String, CompilerType::Rational, CompilerType::List(element)]
                if element.as_ref() == &CompilerType::String)
    )
}

fn compiler_string_string_list_int_triple(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if matches!(fields.as_slice(), [CompilerType::String, CompilerType::List(element), CompilerType::Int]
                if element.as_ref() == &CompilerType::String)
    )
}

fn compiler_int_int_boolean_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if fields.as_slice()
                == [
                    CompilerType::Int,
                    CompilerType::Int,
                    CompilerType::Boolean,
                    CompilerType::Boolean,
                ]
    )
}

fn compiler_int_int_string_int_tuple(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if matches!(fields.as_slice(), [
                CompilerType::Int | CompilerType::Nat,
                CompilerType::Int | CompilerType::Nat,
                CompilerType::String,
                CompilerType::Int | CompilerType::Nat,
            ] | [
                CompilerType::Int | CompilerType::Nat,
                CompilerType::String,
                CompilerType::Int | CompilerType::Nat,
                CompilerType::Int | CompilerType::Nat,
            ])
    )
}

fn compiler_nested_int_string_list_element(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::List(element) if compiler_int_string_pair(element)
    )
}

fn compiler_nested_int_list_element(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::List(element)
            if matches!(element.as_ref(), CompilerType::Int | CompilerType::Nat)
                || compiler_nested_int_list_element(element.as_ref())
    )
}

fn compiler_nested_string_list_element(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::List(element)
            if element.as_ref() == &CompilerType::String
                || compiler_nested_string_list_element(element.as_ref())
    )
}

fn compiler_nested_integer_pair_list_element(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::List(element) if compiler_integer_pair(element.as_ref())
    )
}

fn compiler_nonnegative_iterate_collect(value: &CompilerExpression) -> bool {
    matches!(
        &value.kind,
        CompilerExpressionKind::GeneratorCollect(generator)
            if matches!(
                &generator.kind,
                CompilerExpressionKind::GeneratorTakeWhile { generator, .. }
                    if matches!(
                        &generator.kind,
                        CompilerExpressionKind::IterateGenerator { initial, .. }
                            if initial.int_range.as_ref().is_some_and(|range|
                                range.lower >= BigInt::from(0_u8))
                    )
            )
    )
}

fn compiler_list_observation_element_supported(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Unit
            | CompilerType::Completed
            | CompilerType::Effect
            | CompilerType::Type
            | CompilerType::Boolean
            | CompilerType::Character
            | CompilerType::Comparison
            | CompilerType::ErrorCode
            | CompilerType::Enum(_)
            | CompilerType::Modular(_)
            | CompilerType::Int
            | CompilerType::Nat
            | CompilerType::Rational
            | CompilerType::String
    ) || matches!(value_type, CompilerType::Range(endpoint)
        if endpoint.as_ref() == &CompilerType::Int)
        || matches!(
            value_type,
            CompilerType::Tuple(fields)
                if matches!(fields.as_slice(),
                    [
                        CompilerType::Int | CompilerType::Nat | CompilerType::Character | CompilerType::String | CompilerType::Rational,
                        CompilerType::Int | CompilerType::Nat | CompilerType::Character | CompilerType::String | CompilerType::Rational
                    ])
        )
        || compiler_int_triple(value_type)
        || compiler_string_string_rational_triple(value_type)
        || compiler_int_int_list_triple(value_type)
        || compiler_string_rational_string_list_triple(value_type)
        || compiler_string_string_list_int_triple(value_type)
        || compiler_int_int_boolean_pair(value_type)
        || compiler_int_int_string_int_tuple(value_type)
        || compiler_int_list_pair(value_type)
        || compiler_list_string_rational_pair(value_type)
        || compiler_list_integer_pair(value_type)
        || compiler_boolean_int_pair(value_type)
        || compiler_boolean_string_pair(value_type)
        || compiler_nested_string_list_element(value_type)
        || compiler_nested_integer_pair_list_element(value_type)
        || matches!(value_type, CompilerType::Optional(payload)
        if matches!(payload.as_ref(), CompilerType::Int | CompilerType::Rational | CompilerType::String))
}

fn compiler_list_node_element_supported(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Unit
            | CompilerType::Completed
            | CompilerType::Effect
            | CompilerType::Type
            | CompilerType::Boolean
            | CompilerType::Character
            | CompilerType::Comparison
            | CompilerType::ErrorCode
            | CompilerType::Enum(_)
            | CompilerType::Modular(_)
            | CompilerType::Int
            | CompilerType::Nat
            | CompilerType::Rational
            | CompilerType::String
            | CompilerType::Function
    ) || matches!(value_type, CompilerType::Range(endpoint)
        if endpoint.as_ref() == &CompilerType::Int)
        || matches!(value_type, CompilerType::Optional(payload)
        if matches!(payload.as_ref(), CompilerType::Int | CompilerType::Rational | CompilerType::String))
        || matches!(
            value_type,
            CompilerType::Tuple(fields)
                if matches!(fields.as_slice(),
                    [
                        CompilerType::Int | CompilerType::Nat | CompilerType::Rational,
                        CompilerType::Int | CompilerType::Nat | CompilerType::Rational
                    ])
        )
        || compiler_int_string_pair(value_type)
        || compiler_int_list_pair(value_type)
        || compiler_list_string_rational_pair(value_type)
        || compiler_string_int_pair(value_type)
        || compiler_string_pair(value_type)
        || compiler_boolean_int_pair(value_type)
        || compiler_boolean_string_pair(value_type)
        || compiler_list_integer_pair(value_type)
        || compiler_string_function_pair(value_type)
        || compiler_int_triple(value_type)
        || compiler_string_string_rational_triple(value_type)
        || compiler_int_int_list_triple(value_type)
        || compiler_string_rational_string_list_triple(value_type)
        || compiler_string_string_list_int_triple(value_type)
        || compiler_int_int_boolean_pair(value_type)
        || compiler_int_int_string_int_tuple(value_type)
        || compiler_nested_int_list_element(value_type)
        || compiler_nested_string_list_element(value_type)
        || compiler_nested_integer_pair_list_element(value_type)
        || compiler_nested_int_string_list_element(value_type)
}

fn compiler_nat_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if fields.as_slice() == [CompilerType::Nat, CompilerType::Nat]
    )
}

fn compiler_integer_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if matches!(fields.as_slice(), [
                CompilerType::Int | CompilerType::Nat,
                CompilerType::Int | CompilerType::Nat
            ])
    )
}

fn compiler_rational_nat_pair(value_type: &CompilerType) -> bool {
    matches!(value_type, CompilerType::Tuple(fields)
        if fields.as_slice() == [CompilerType::Rational, CompilerType::Nat])
}

fn compiler_list_integer_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if matches!(fields.as_slice(), [CompilerType::List(element), CompilerType::Int | CompilerType::Nat]
                if matches!(element.as_ref(), CompilerType::Int | CompilerType::Nat)
                    || compiler_nested_int_list_element(element.as_ref()))
    )
}

fn compiler_list_string_rational_pair(value_type: &CompilerType) -> bool {
    matches!(
        value_type,
        CompilerType::Tuple(fields)
            if matches!(fields.as_slice(), [CompilerType::List(element), CompilerType::Rational]
                if element.as_ref() == &CompilerType::String)
    )
}

fn parse_compact_classifier(classifier: &str) -> Option<CompilerType> {
    parse_compact_classifier_with(classifier, &|_| None)
}

fn split_compact_function_classifier(classifier: &str) -> Option<(&str, &str)> {
    let body = classifier.strip_prefix("fn(")?;
    let separator = body.rfind(")->")?;
    let input = &body[..separator];
    let output = &body[separator + 3..];
    (!input.is_empty() && !output.is_empty()).then_some((input, output))
}

fn split_classifier_constraint(classifier: &str) -> Option<(&str, &str)> {
    let inner = classifier.strip_prefix('(')?.strip_suffix(')')?;
    if split_classifier_fields(inner)?.len() != 1 {
        return None;
    }
    let mut depth = 0_u32;
    for (index, byte) in inner.bytes().enumerate() {
        match byte {
            b'(' => depth += 1,
            b')' => depth = depth.checked_sub(1)?,
            b':' if depth == 0 => {
                let name = &inner[..index];
                let capability = &inner[index + 1..];
                return (!name.is_empty() && !capability.is_empty()).then_some((name, capability));
            }
            _ => {}
        }
    }
    None
}

fn generic_capability_accepts(capability: &str, value_type: &CompilerType) -> bool {
    match capability {
        "Type" => !compiler_type_contains_static_only(value_type),
        "Equality" => compiler_equality_supported(value_type),
        "TotalOrder" => compiler_ordering_supported(value_type),
        "ErrorCode" => value_type == &CompilerType::ErrorCode,
        _ => false,
    }
}

fn classifier_uses_substitution(
    classifier: &str,
    substitutions: &BTreeMap<String, CompilerType>,
) -> bool {
    if split_classifier_constraint(classifier).is_some() || substitutions.contains_key(classifier) {
        return true;
    }
    for prefix in ["List", "Optional", "Range", "Set", "Bag"] {
        if let Some(element) = classifier.strip_prefix(prefix) {
            return classifier_uses_substitution(element, substitutions);
        }
    }
    if let Some(fields) = classifier
        .strip_prefix("Result(")
        .and_then(|value| value.strip_suffix(')'))
        .and_then(split_classifier_fields)
    {
        return fields
            .iter()
            .any(|field| classifier_uses_substitution(field, substitutions));
    }
    classifier
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .and_then(split_classifier_fields)
        .is_some_and(|fields| {
            fields
                .iter()
                .any(|field| classifier_uses_substitution(field, substitutions))
        })
}

fn infer_classifier_substitutions(
    classifier: &str,
    actual: &CompilerType,
    substitutions: &mut BTreeMap<String, CompilerType>,
    resolve_nominal: &impl Fn(&str) -> Option<CompilerType>,
) -> bool {
    if let Some((name, capability)) = split_classifier_constraint(classifier) {
        if !generic_capability_accepts(capability, actual) {
            return false;
        }
        return substitutions
            .get(name)
            .is_none_or(|existing| existing == actual)
            && {
                substitutions.insert(name.to_owned(), actual.clone());
                true
            };
    }
    if let Some(expected) = substitutions.get(classifier) {
        return expected == actual;
    }
    if let Some(fields) = classifier
        .strip_prefix("Result(")
        .and_then(|value| value.strip_suffix(')'))
        .and_then(split_classifier_fields)
        && let [success, codes] = fields.as_slice()
    {
        let CompilerType::Result(actual_success) = actual else {
            return false;
        };
        return infer_classifier_substitutions(
            success,
            actual_success,
            substitutions,
            resolve_nominal,
        ) && infer_classifier_substitutions(
            codes,
            &CompilerType::ErrorCode,
            substitutions,
            resolve_nominal,
        );
    }
    if let Some(fields) = classifier
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .and_then(split_classifier_fields)
        && let [field] = fields.as_slice()
    {
        return infer_classifier_substitutions(field, actual, substitutions, resolve_nominal);
    }
    if let Some(element) = classifier.strip_prefix("List") {
        return matches!(actual, CompilerType::List(actual) if infer_classifier_substitutions(element, actual, substitutions, resolve_nominal));
    }
    if let Some(payload) = classifier.strip_prefix("Optional") {
        return matches!(actual, CompilerType::Optional(actual) if infer_classifier_substitutions(payload, actual, substitutions, resolve_nominal));
    }
    if let Some(element) = classifier.strip_prefix("Range") {
        return matches!(actual, CompilerType::Range(actual) if infer_classifier_substitutions(element, actual, substitutions, resolve_nominal));
    }
    if let Some(fields) = classifier
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        && let Some(patterns) = split_classifier_fields(fields)
        && patterns.len() > 1
    {
        let CompilerType::Tuple(actual_fields) = actual else {
            return false;
        };
        return patterns.len() == actual_fields.len()
            && patterns.iter().zip(actual_fields).all(|(pattern, actual)| {
                infer_classifier_substitutions(pattern, actual, substitutions, resolve_nominal)
            });
    }
    let expected = parse_compact_classifier_with(classifier, resolve_nominal);
    if expected.is_none()
        && classifier
            .chars()
            .all(|character| character == '-' || character.is_alphanumeric())
    {
        return substitutions
            .get(classifier)
            .is_none_or(|existing| existing == actual)
            && {
                substitutions.insert(classifier.to_owned(), actual.clone());
                true
            };
    }
    expected.as_ref() == Some(actual)
        || matches!(actual, CompilerType::Refined { base, .. }
            if expected.as_ref() == Some(base.as_ref()))
}

fn parse_substituted_classifier(
    classifier: &str,
    substitutions: &BTreeMap<String, CompilerType>,
    resolve_nominal: &impl Fn(&str) -> Option<CompilerType>,
) -> Option<CompilerType> {
    if classifier.starts_with("fn(") && classifier.contains(")->") {
        return Some(CompilerType::Function);
    }
    if let Some((name, capability)) = split_classifier_constraint(classifier) {
        let value_type = substitutions.get(name)?.clone();
        return generic_capability_accepts(capability, &value_type).then_some(value_type);
    }
    if let Some(value_type) = substitutions.get(classifier) {
        return Some(value_type.clone());
    }
    if let Some(fields) = classifier
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .and_then(split_classifier_fields)
        && let [field] = fields.as_slice()
    {
        return parse_substituted_classifier(field, substitutions, resolve_nominal);
    }
    if let Some(element) = classifier.strip_prefix("List") {
        let element = parse_substituted_classifier(element, substitutions, resolve_nominal)?;
        return compiler_list_node_element_supported(&element)
            .then(|| CompilerType::List(Box::new(element)));
    }
    if let Some(payload) = classifier.strip_prefix("Optional") {
        return Some(CompilerType::Optional(Box::new(
            parse_substituted_classifier(payload, substitutions, resolve_nominal)?,
        )));
    }
    if let Some(fields) = classifier
        .strip_prefix("Result(")
        .and_then(|value| value.strip_suffix(')'))
        .and_then(split_classifier_fields)
        && let [success, codes] = fields.as_slice()
    {
        let success = parse_substituted_classifier(success, substitutions, resolve_nominal)?;
        let codes = parse_substituted_classifier(codes, substitutions, resolve_nominal)?;
        return (codes == CompilerType::ErrorCode).then(|| CompilerType::Result(Box::new(success)));
    }
    if let Some(element) = classifier.strip_prefix("Range") {
        return Some(CompilerType::Range(Box::new(parse_substituted_classifier(
            element,
            substitutions,
            resolve_nominal,
        )?)));
    }
    if let Some(fields) = classifier
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        && let Some(fields) = split_classifier_fields(fields)
        && fields.len() > 1
    {
        return Some(CompilerType::Tuple(
            fields
                .into_iter()
                .map(|field| parse_substituted_classifier(field, substitutions, resolve_nominal))
                .collect::<Option<Vec<_>>>()?,
        ));
    }
    parse_compact_classifier_with(classifier, resolve_nominal)
}

#[allow(clippy::too_many_lines)] // Compact structural classifiers are parsed in one recursive grammar walk.
fn parse_compact_classifier_with(
    classifier: &str,
    resolve_nominal: &impl Fn(&str) -> Option<CompilerType>,
) -> Option<CompilerType> {
    if classifier.starts_with("fn(") && classifier.contains(")->") {
        return Some(CompilerType::Function);
    }
    if let Some(fields) = classifier
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .and_then(split_classifier_fields)
        && let [field] = fields.as_slice()
    {
        return parse_compact_classifier_with(field, resolve_nominal);
    }
    if let Some(fields) = classifier
        .strip_prefix("Array(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let (count, element) = split_classifier_once(fields)?;
        return Some(CompilerType::Array {
            count: count.parse().ok()?,
            element: Box::new(parse_compact_classifier_with(element, resolve_nominal)?),
        });
    }
    if let Some(fields) = classifier
        .strip_prefix("Map(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let (key, value) = split_classifier_once(fields)?;
        return Some(CompilerType::Map {
            key: Box::new(parse_compact_classifier_with(key, resolve_nominal)?),
            value: Box::new(parse_compact_classifier_with(value, resolve_nominal)?),
        });
    }
    if let Some(element) = classifier.strip_prefix("Set") {
        return Some(CompilerType::Set(Box::new(parse_compact_classifier_with(
            element,
            resolve_nominal,
        )?)));
    }
    if let Some(element) = classifier.strip_prefix("Bag") {
        return Some(CompilerType::Bag(Box::new(parse_compact_classifier_with(
            element,
            resolve_nominal,
        )?)));
    }
    if let Some(element) = classifier.strip_prefix("List") {
        let element = parse_compact_classifier_with(element, resolve_nominal)?;
        if compiler_list_node_element_supported(&element) {
            return Some(CompilerType::List(Box::new(element)));
        }
        return None;
    }
    if let Some(payload) = classifier.strip_prefix("Optional") {
        return Some(CompilerType::Optional(Box::new(
            parse_compact_classifier_with(payload, resolve_nominal)?,
        )));
    }
    if let Some(success_and_codes) = classifier
        .strip_prefix("Result(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let (success, codes) = split_classifier_once(success_and_codes)?;
        if codes != "langarithmeticArithmeticErrorCode" {
            return None;
        }
        return Some(CompilerType::Result(Box::new(
            parse_compact_classifier_with(success, resolve_nominal)?,
        )));
    }
    if let Some(fields) = classifier
        .strip_prefix("Record(")
        .and_then(|value| value.strip_suffix(')'))
    {
        let mut parsed = split_classifier_fields(fields)?
            .into_iter()
            .map(|field| {
                let (label, classifier) = split_record_classifier_field(field)?;
                Some((
                    label.to_owned(),
                    parse_compact_classifier_with(classifier, resolve_nominal)?,
                ))
            })
            .collect::<Option<Vec<_>>>()?;
        parsed.sort_by(|left, right| left.0.cmp(&right.0));
        return parsed
            .windows(2)
            .all(|fields| fields[0].0 != fields[1].0)
            .then_some(CompilerType::Record(parsed));
    }
    if let Some(fields) = classifier
        .strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
    {
        let fields = split_classifier_fields(fields)?;
        if fields.len() <= 1 {
            return None;
        }
        return Some(CompilerType::Tuple(
            fields
                .into_iter()
                .map(|field| parse_compact_classifier_with(field, resolve_nominal))
                .collect::<Option<Vec<_>>>()?,
        ));
    }
    parse_compact_scalar_classifier(classifier).or_else(|| resolve_nominal(classifier))
}

fn parse_compact_scalar_classifier(classifier: &str) -> Option<CompilerType> {
    match classifier {
        "RangeInt" => Some(CompilerType::Range(Box::new(CompilerType::Int))),
        "RangeRational" => Some(CompilerType::Range(Box::new(CompilerType::Rational))),
        "Unit" => Some(CompilerType::Unit),
        "Completed" => Some(CompilerType::Completed),
        "Effect" => Some(CompilerType::Effect),
        "Type" => Some(CompilerType::Type),
        "Scope" => Some(CompilerType::Scope),
        "Function" => Some(CompilerType::Function),
        "Capability" => Some(CompilerType::Capability),
        "Constraint" => Some(CompilerType::Constraint),
        "Boolean" => Some(CompilerType::Boolean),
        "Int" => Some(CompilerType::Int),
        "Nat" => Some(CompilerType::Nat),
        "Rational" => Some(CompilerType::Rational),
        "Comparison" => Some(CompilerType::Comparison),
        "Error" => Some(CompilerType::Error),
        "ErrorCode" | "langarithmeticArithmeticErrorCode" => Some(CompilerType::ErrorCode),
        "ErrorDomain" => Some(CompilerType::ErrorDomain),
        "SourceLocation" => Some(CompilerType::SourceLocation),
        "Character" => Some(CompilerType::Character),
        "String" => Some(CompilerType::String),
        "GeneratorCharacterUnitUnit" => Some(character_unit_generator_type()),
        "GeneratorCharacterUnitCharacter" => {
            Some(character_generator_type(CompilerType::Character))
        }
        "GeneratorIntUnitUnit" => Some(int_unit_generator_type()),
        "GeneratorIntUnitString" => Some(value_boundary_generator_type()),
        "Generator(Int,String)Unit(Int,String)" => Some(product_boundary_generator_type()),
        "GeneratorOptional(Int,String)UnitResult((Int,String),langarithmeticArithmeticErrorCode)" => {
            Some(nested_boundary_generator_type())
        }
        "GeneratorListIntUnitListInt" => Some(list_boundary_generator_type()),
        _ => None,
    }
}

fn split_record_classifier_field(field: &str) -> Option<(&str, &str)> {
    let mut depth = 0_u32;
    for (index, byte) in field.bytes().enumerate() {
        match byte {
            b'(' => depth += 1,
            b')' => depth = depth.checked_sub(1)?,
            b':' if depth == 0 => {
                let label = &field[..index];
                let classifier = &field[index + 1..];
                return (!label.is_empty() && !classifier.is_empty())
                    .then_some((label, classifier));
            }
            _ => {}
        }
    }
    None
}

fn split_classifier_once(classifier: &str) -> Option<(&str, &str)> {
    let mut depth = 0_u32;
    for (index, byte) in classifier.bytes().enumerate() {
        match byte {
            b'(' => depth += 1,
            b')' => depth = depth.checked_sub(1)?,
            b',' if depth == 0 => return Some((&classifier[..index], &classifier[index + 1..])),
            _ => {}
        }
    }
    None
}

fn split_classifier_fields(classifier: &str) -> Option<Vec<&str>> {
    let mut fields = Vec::new();
    let mut remaining = classifier;
    while let Some((field, rest)) = split_classifier_once(remaining) {
        fields.push(field);
        remaining = rest;
    }
    fields.push(remaining);
    fields
        .iter()
        .all(|field| !field.is_empty())
        .then_some(fields)
}

fn is_range_construction(operation: CompilerBinary) -> bool {
    matches!(
        operation,
        CompilerBinary::Range
            | CompilerBinary::RangeOpen
            | CompilerBinary::RangeInclusive
            | CompilerBinary::RangeOpenInclusive
    )
}

#[allow(clippy::too_many_lines)] // ABI admission keeps every private aggregate layout explicit.
fn compiler_abi_type_supported(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::TraversalControl(_)
        | CompilerType::Generator(_)
        | CompilerType::SerializationStream(_)
        | CompilerType::TaskResponse(_)
        | CompilerType::Task(_)
        | CompilerType::ExternalLocation(_) => false,
        CompilerType::List(element) => {
            matches!(
                element.as_ref(),
                CompilerType::Unit
                    | CompilerType::Completed
                    | CompilerType::Effect
                    | CompilerType::Type
                    | CompilerType::Boolean
                    | CompilerType::Character
                    | CompilerType::Comparison
                    | CompilerType::ErrorCode
                    | CompilerType::Enum(_)
                    | CompilerType::Modular(_)
                    | CompilerType::Int
                    | CompilerType::Nat
                    | CompilerType::Rational
                    | CompilerType::String
                    | CompilerType::Function
            ) || compiler_nested_int_list_element(element.as_ref())
                || compiler_nested_string_list_element(element.as_ref())
                || compiler_nested_integer_pair_list_element(element.as_ref())
                || compiler_nested_int_string_list_element(element.as_ref())
                || matches!(element.as_ref(), CompilerType::Tuple(fields)
                if matches!(fields.as_slice(), [
                    CompilerType::Int | CompilerType::Nat | CompilerType::Character | CompilerType::String | CompilerType::Rational,
                    CompilerType::Int | CompilerType::Nat | CompilerType::Character | CompilerType::String | CompilerType::Rational
                ]))
                || compiler_int_triple(element.as_ref())
                || compiler_boolean_int_pair(element.as_ref())
                || compiler_boolean_string_pair(element.as_ref())
                || compiler_string_string_rational_triple(element.as_ref())
                || compiler_int_int_list_triple(element.as_ref())
                || compiler_string_rational_string_list_triple(element.as_ref())
                || compiler_string_string_list_int_triple(element.as_ref())
                || matches!(element.as_ref(), CompilerType::Optional(payload)
                    if matches!(payload.as_ref(), CompilerType::Int | CompilerType::Rational | CompilerType::String))
                || compiler_string_function_pair(element.as_ref())
                || compiler_int_int_boolean_pair(element.as_ref())
                || compiler_int_int_string_int_tuple(element.as_ref())
                || compiler_int_list_pair(element.as_ref())
                || compiler_list_integer_pair(element.as_ref())
                || compiler_list_string_rational_pair(element.as_ref())
                || matches!(element.as_ref(), CompilerType::Range(endpoint)
                    if endpoint.as_ref() == &CompilerType::Int)
        }
        CompilerType::Optional(payload) => {
            matches!(
                payload.as_ref(),
                CompilerType::Boolean
                    | CompilerType::Int
                    | CompilerType::Nat
                    | CompilerType::Rational
                    | CompilerType::Character
                    | CompilerType::String
                    | CompilerType::Error
                    | CompilerType::SourceLocation
                    | CompilerType::Function
            ) || compiler_integer_pair(payload)
                || compiler_list_integer_pair(payload)
                || compiler_list_string_rational_pair(payload)
                || compiler_int_triple(payload)
                || compiler_string_string_rational_triple(payload)
                || compiler_string_rational_string_list_triple(payload)
                || compiler_string_string_list_int_triple(payload)
                || compiler_int_string_pair(payload)
                || compiler_int_list_pair(payload)
                || compiler_int_int_string_int_tuple(payload)
                || matches!(payload.as_ref(), CompilerType::Tuple(fields)
                    if matches!(fields.as_slice(), [CompilerType::Int, CompilerType::List(element)]
                        if element.as_ref() == &CompilerType::Int))
                || matches!(payload.as_ref(), CompilerType::List(_))
        }
        CompilerType::Result(success) => {
            matches!(
                success.as_ref(),
                CompilerType::Boolean
                    | CompilerType::Int
                    | CompilerType::Nat
                    | CompilerType::Rational
                    | CompilerType::String
                    | CompilerType::Modular(_)
                    | CompilerType::Function
            ) || matches!(
                success.as_ref(),
                CompilerType::Tuple(fields)
                    if matches!(fields.as_slice(), [CompilerType::Int, CompilerType::Int | CompilerType::String])
            )
        }
        CompilerType::Range(endpoint) => {
            matches!(
                endpoint.as_ref(),
                CompilerType::Int | CompilerType::Rational
            )
        }
        CompilerType::Array { element, .. } => {
            matches!(element.as_ref(), CompilerType::Int | CompilerType::Function)
        }
        CompilerType::Set(element) | CompilerType::Bag(element) => {
            element.as_ref() == &CompilerType::Int
        }
        CompilerType::Map { key, value } => {
            key.as_ref() == &CompilerType::String
                && matches!(value.as_ref(), CompilerType::Int | CompilerType::Function)
        }
        _ => true,
    }
}

fn compiler_type_contains_infinity(value_type: &CompilerType) -> bool {
    match value_type {
        CompilerType::InfiniteInt | CompilerType::InfiniteNat | CompilerType::InfiniteRational => {
            true
        }
        CompilerType::SerializationStream(value)
        | CompilerType::Range(value)
        | CompilerType::Result(value)
        | CompilerType::TaskResponse(value)
        | CompilerType::Optional(value)
        | CompilerType::List(value)
        | CompilerType::Set(value)
        | CompilerType::Bag(value)
        | CompilerType::TraversalControl(value)
        | CompilerType::Refined { base: value, .. } => compiler_type_contains_infinity(value),
        CompilerType::Array { element, .. } => compiler_type_contains_infinity(element),
        CompilerType::Map { key, value } => {
            compiler_type_contains_infinity(key) || compiler_type_contains_infinity(value)
        }
        CompilerType::Generator(generator) => {
            compiler_type_contains_infinity(&generator.yield_type)
                || compiler_type_contains_infinity(&generator.resume_type)
                || compiler_type_contains_infinity(&generator.result_type)
        }
        CompilerType::Task(task) => {
            compiler_type_contains_infinity(&task.state_type)
                || task.handlers.iter().any(|handler| {
                    compiler_type_contains_infinity(&handler.payload_type)
                        || compiler_type_contains_infinity(&handler.response_type)
                        || handler.stream_type.as_ref().is_some_and(|stream| {
                            compiler_type_contains_infinity(&stream.yield_type)
                                || compiler_type_contains_infinity(&stream.resume_type)
                                || compiler_type_contains_infinity(&stream.result_type)
                        })
                })
        }
        CompilerType::Sum(sum) => sum.alternatives.iter().any(|alternative| {
            alternative
                .payload
                .as_ref()
                .is_some_and(compiler_type_contains_infinity)
        }),
        CompilerType::Tuple(fields) => fields.iter().any(compiler_type_contains_infinity),
        CompilerType::Record(fields) => fields
            .iter()
            .any(|(_, value)| compiler_type_contains_infinity(value)),
        _ => false,
    }
}

fn external_metadata_expression(
    metadata: CompilerExternalMetadata,
    span: Span,
) -> CompilerExpression {
    CompilerExpression {
        kind: CompilerExpressionKind::ExternalMetadata(metadata),
        value_type: CompilerType::ExternalMetadata,
        int_range: None,
        rational_value: None,
        span,
    }
}

fn assign_external_metadata_identity(metadata: &mut CompilerExternalMetadata, identity: &str) {
    match metadata {
        CompilerExternalMetadata::Layout(layout) => layout.identity = identity.into(),
        CompilerExternalMetadata::AddressRangeType(range) => range.identity = identity.into(),
        CompilerExternalMetadata::AddressRange(range) => range.identity = identity.into(),
        CompilerExternalMetadata::AddressOffsetType(offset) => offset.identity = identity.into(),
        CompilerExternalMetadata::AddressOffset(offset) => offset.identity = identity.into(),
        CompilerExternalMetadata::LocationType(location) => location.identity = identity.into(),
    }
}

fn external_layout_value_type(layout: &CompilerExternalLayout) -> Option<CompilerType> {
    matches!(layout.family, CompilerExternalLayoutFamily::UnsignedNat).then(|| {
        CompilerType::Refined {
            constraint: layout.identity.clone(),
            base: Box::new(CompilerType::Nat),
        }
    })
}
