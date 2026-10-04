#[allow(clippy::too_many_arguments)] // Every existing nominal root namespace is a collision boundary.
fn collect_interface_sources(
    source: &SourceText,
    statements: &[Statement],
    enums: &EnumTypes,
    enum_alternatives: &EnumAlternativeBindings,
    sums: &SumTypes,
    sum_alternatives: &SumAlternativeBindings,
    modulars: &[ModularSource],
) -> Result<BTreeMap<String, InterfaceSource>, Diagnostic> {
    let modular_names = modulars
        .iter()
        .map(|declaration| source.slice(declaration.name))
        .collect::<BTreeSet<_>>();
    let mut interfaces = BTreeMap::new();
    for statement in statements {
        let Some(declaration) = interface_declaration(statement) else {
            continue;
        };
        let name = source.slice(declaration.name).to_owned();
        if name == "root"
            || enums.contains_key(&name)
            || enum_alternatives.contains_key(&name)
            || sums.contains_key(&name)
            || sum_alternatives.contains_key(&name)
            || modular_names.contains(name.as_str())
            || interfaces.contains_key(&name)
        {
            return Err(source_diagnostic(
                source,
                "E-DUPLICATE-DECLARATION",
                declaration.name,
                format!("`{name}` is already declared"),
            ));
        }
        let mut operations = BTreeSet::new();
        for function in &declaration.functions {
            let operation = source.slice(function.name);
            if !operations.insert(operation) {
                return Err(source_diagnostic(
                    source,
                    "E-DUPLICATE-INTERFACE-OPERATION",
                    function.name,
                    format!("interface operation `{operation}` is declared twice"),
                ));
            }
        }
        interfaces.insert(name, declaration);
    }
    Ok(interfaces)
}

fn collect_modular_sources(
    source: &SourceText,
    statements: &[Statement],
    enums: &EnumTypes,
    enum_alternatives: &EnumAlternativeBindings,
    sums: &SumTypes,
    sum_alternatives: &SumAlternativeBindings,
) -> Result<Vec<ModularSource>, Diagnostic> {
    let mut declarations = Vec::new();
    let mut names = BTreeSet::new();
    for statement in statements {
        let Some(mut declaration) = modular_declaration(source, statement) else {
            continue;
        };
        let name = source.slice(declaration.name);
        if name == "root"
            || enums.contains_key(name)
            || enum_alternatives.contains_key(name)
            || sums.contains_key(name)
            || sum_alternatives.contains_key(name)
            || !names.insert(name.to_owned())
        {
            return Err(source_diagnostic(
                source,
                "E-DUPLICATE-BINDING",
                declaration.name,
                format!("`{name}` is already declared in this scope"),
            ));
        }
        declaration.range = resolve_modular_range_source(
            source,
            statements,
            &declaration.range,
            declaration.name.start,
        );
        declarations.push(declaration);
    }
    Ok(declarations)
}

fn resolve_modular_range_source(
    source: &SourceText,
    statements: &[Statement],
    range: &Expression,
    mut before: usize,
) -> Expression {
    let mut range = range.clone();
    let mut seen = BTreeSet::new();
    while let Expression::Identifier(name) = &range {
        let name = source.slice(*name);
        if !seen.insert(name.to_owned()) {
            break;
        }
        let Some((value, declaration_start)) = statements.iter().rev().find_map(|statement| {
            let statement = match statement {
                Statement::Published { declaration, .. } => declaration.as_ref(),
                statement => statement,
            };
            let Statement::Binding {
                name: candidate,
                value,
                ..
            } = statement
            else {
                return None;
            };
            (candidate.start < before && source.slice(*candidate) == name)
                .then_some((value.clone(), candidate.start))
        }) else {
            break;
        };
        range = value;
        before = declaration_start;
    }
    range
}

fn collect_sums(
    source: &SourceText,
    statements: &[Statement],
    enums: &EnumTypes,
    enum_alternatives: &EnumAlternativeBindings,
) -> Result<(SumTypes, SumAlternativeBindings), Diagnostic> {
    let mut sums: SumTypes = BTreeMap::new();
    let mut alternatives: SumAlternativeBindings = BTreeMap::new();
    for statement in statements {
        let Some(declaration) = sum_declaration(source, statement) else {
            continue;
        };
        let name = source.slice(declaration.name).to_owned();
        if name == "root"
            || enums.contains_key(&name)
            || enum_alternatives.contains_key(&name)
            || sums.contains_key(&name)
            || alternatives.contains_key(&name)
        {
            return Err(source_diagnostic(
                source,
                "E-DUPLICATE-UNION",
                declaration.name,
                format!("`{name}` is already declared in this scope"),
            ));
        }
        let mut local = BTreeSet::new();
        let mut lowered = Vec::with_capacity(declaration.alternatives.len());
        for (label, classifier, alternative_span) in &declaration.alternatives {
            if !local.insert(label.clone()) {
                return Err(source_diagnostic(
                    source,
                    "E-DUPLICATE-UNION-ALTERNATIVE",
                    *alternative_span,
                    format!("sum alternative `{label}` occurs more than once"),
                ));
            }
            if !declaration.positional
                && (label == "root"
                    || label == &name
                    || enums.contains_key(label)
                    || enum_alternatives.contains_key(label)
                    || sums.contains_key(label)
                    || alternatives.contains_key(label))
            {
                return Err(source_diagnostic(
                    source,
                    "E-DUPLICATE-UNION-ALTERNATIVE",
                    *alternative_span,
                    format!("sum alternative `{label}` is already declared in this scope"),
                ));
            }
            let payload = classifier
                .map(|classifier| {
                    let classifier_text = compact_classifier(source.slice(classifier));
                    enums
                        .get(&classifier_text)
                        .filter(|(_, declaration)| declaration.end <= classifier.start)
                        .map(|(enumeration, _)| CompilerType::Enum(enumeration.clone()))
                        .or_else(|| {
                            sums.get(&classifier_text)
                                .filter(|(_, declaration)| declaration.end <= classifier.start)
                                .map(|(sum, _)| CompilerType::Sum(sum.clone()))
                        })
                        .or_else(|| parse_compact_classifier(&classifier_text))
                        .ok_or_else(|| unsupported(source, classifier, "sum payload classifier"))
                })
                .transpose()?;
            if payload.as_ref().is_some_and(|payload| {
                payload != &CompilerType::Function && !compiler_function_result_supported(payload)
            }) {
                return Err(unsupported(
                    source,
                    classifier.expect("unsupported payload has a classifier"),
                    "sum payload without an admitted private representation",
                ));
            }
            lowered.push(CompilerSumAlternative {
                name: label.clone(),
                payload,
            });
        }
        let sum = CompilerSumType {
            name: name.clone(),
            positional: declaration.positional,
            alternatives: lowered,
        };
        if !sum.positional {
            for (index, alternative) in sum.alternatives.iter().enumerate() {
                let index = u32::try_from(index).map_err(|_| {
                    unsupported(source, declaration.span, "native Union alternative tag")
                })?;
                alternatives.insert(
                    alternative.name.clone(),
                    (sum.clone(), index, declaration.span),
                );
            }
        }
        sums.insert(name, (sum, declaration.span));
    }
    Ok((sums, alternatives))
}

fn constraint_definition<'a>(
    source: &SourceText,
    expression: &'a Expression,
) -> Option<(Span, &'a [AnonymousPattern], &'a Expression, Span)> {
    let Expression::Application { items, span } = expression else {
        return None;
    };
    let [
        Expression::Identifier(base),
        Expression::Identifier(operation),
        Expression::AnonymousFunction {
            parameters, body, ..
        },
    ] = items.as_slice()
    else {
        return None;
    };
    (source.slice(*operation) == "constraint").then_some((
        *base,
        parameters.as_slice(),
        body.as_ref(),
        *span,
    ))
}

fn nonempty_string_constraint_predicate(
    source: &SourceText,
    predicate: &Expression,
    parameter: &str,
) -> bool {
    let Expression::Application { items, .. } = predicate else {
        return false;
    };
    let [
        count,
        Expression::Callable {
            kind: CallableKind::Greater,
            ..
        },
        Expression::Integer(zero),
    ] = items.as_slice()
    else {
        return false;
    };
    if parse_integer(source.slice(*zero)).as_ref() != Some(&BigInt::from(0)) {
        return false;
    }
    let Expression::Application { items, .. } = count else {
        return false;
    };
    let [Expression::Identifier(entry_count), collected] = items.as_slice() else {
        return false;
    };
    if source.slice(*entry_count) != "entry-count" {
        return false;
    }
    let Expression::Application { items, .. } = collected else {
        return false;
    };
    let [Expression::Identifier(collect), generated] = items.as_slice() else {
        return false;
    };
    if source.slice(*collect) != "collect" {
        return false;
    }
    let Expression::Application { items, .. } = generated else {
        return false;
    };
    matches!(items.as_slice(), [Expression::Identifier(characters), Expression::Identifier(value)]
        if source.slice(*characters) == "characters" && source.slice(*value) == parameter)
}

fn unicode_white_space(character: char) -> bool {
    matches!(
        character,
        '\u{0009}'..='\u{000D}'
            | '\u{0020}'
            | '\u{0085}'
            | '\u{00A0}'
            | '\u{1680}'
            | '\u{2000}'..='\u{200A}'
            | '\u{2028}'
            | '\u{2029}'
            | '\u{202F}'
            | '\u{205F}'
            | '\u{3000}'
    )
}

fn unicode_character_predicate(operation: &str, candidate: &str) -> bool {
    let mut scalars = candidate.chars();
    let Some(character) = scalars.next() else {
        return false;
    };
    if scalars.next().is_some() {
        return false;
    }
    match operation {
        "unicode-carriage-return-character" => character == '\r',
        "unicode-line-feed-character" => character == '\n',
        "unicode-whitespace-character" => unicode_white_space(character),
        "unicode-decimal-digit-character" => is_decimal_digit(character),
        "unicode-word-character" => is_regex_word(character),
        _ => false,
    }
}

fn callback_state_classifier(
    parameters: &[FunctionParameter],
    arguments: &[Expression],
    state: &str,
    source: &SourceText,
) -> Option<Span> {
    let arguments = if let [Expression::Product { fields, .. }] = arguments {
        fields.iter().map(|field| &field.value).collect::<Vec<_>>()
    } else {
        arguments.iter().collect::<Vec<_>>()
    };
    let parameters = if let [parameter] = parameters
        && !parameter.fields.is_empty()
    {
        parameter.fields.as_slice()
    } else {
        parameters
    };
    if parameters.len() != arguments.len() {
        return None;
    }
    parameters
        .iter()
        .zip(arguments)
        .find_map(|(parameter, argument)| {
            if matches!(argument, Expression::Identifier(name) if source.slice(*name) == state) {
                return Some(parameter.classifier);
            }
            let Expression::Product { fields, .. } = argument else {
                return None;
            };
            callback_state_classifier(
                &parameter.fields,
                &fields
                    .iter()
                    .map(|field| field.value.clone())
                    .collect::<Vec<_>>(),
                state,
                source,
            )
        })
}

#[allow(clippy::too_many_lines)] // Published and ordinary declaration validation remains one ordered pass.
fn collect_functions(
    source: &SourceText,
    statements: &[Statement],
    reserved_names: &BTreeSet<String>,
    functions: &mut BTreeMap<String, Vec<FunctionSource>>,
    validate_effects: bool,
) -> Result<(), Diagnostic> {
    for statement in statements {
        if let Statement::InterfaceImplementation { declarations, .. } = statement {
            collect_functions(
                source,
                declarations,
                reserved_names,
                functions,
                validate_effects,
            )?;
            continue;
        }
        let declaration = match statement {
            Statement::Function {
                name,
                parameters,
                result,
                body,
                span,
                is_static,
                effect_bound,
                clauses,
            } if **clauses == FunctionClauses::default() => Some(FunctionSource {
                name: *name,
                parameters: parameters.clone(),
                result: *result,
                effect_bound: *effect_bound,
                declared_effects: None,
                body: body.clone(),
                span: *span,
                is_static: *is_static,
                published: false,
                module_identity: None,
            }),
            Statement::Published { declaration, .. } => {
                if let Statement::Function {
                    name,
                    parameters,
                    result,
                    body,
                    span,
                    is_static,
                    effect_bound,
                    clauses,
                } = declaration.as_ref()
                    && **clauses == FunctionClauses::default()
                {
                    Some(FunctionSource {
                        name: *name,
                        parameters: parameters.clone(),
                        result: *result,
                        effect_bound: *effect_bound,
                        declared_effects: None,
                        body: body.clone(),
                        span: *span,
                        is_static: *is_static,
                        published: true,
                        module_identity: None,
                    })
                } else if interface_declaration(statement).is_some()
                    || enum_declaration(source, statement).is_some()
                    || sum_declaration(source, statement).is_some()
                    || matches!(declaration.as_ref(), Statement::Binding { .. })
                {
                    None
                } else {
                    return Err(unsupported(
                        source,
                        statement_span(statement),
                        "published declaration",
                    ));
                }
            }
            Statement::Function { .. } => {
                return Err(unsupported(
                    source,
                    statement_span(statement),
                    "constrained or effectful function",
                ));
            }
            _ => None,
        };
        if let Some(mut function) = declaration {
            if validate_effects {
                function.declared_effects =
                    compiler_declared_effect_row(source, function.effect_bound)?;
            }
            let name = source.slice(function.name).to_owned();
            if reserved_names.contains(&name) {
                return Err(source_diagnostic(
                    source,
                    "E-DUPLICATE-BINDING",
                    function.name,
                    format!("`{name}` is already declared in this scope"),
                ));
            }
            let overloads = functions.entry(name.clone()).or_default();
            if overloads
                .iter()
                .any(|candidate| same_function_input_header(source, candidate, &function))
            {
                return Err(source_diagnostic(
                    source,
                    "E-DUPLICATE-FUNCTION-OVERLOAD",
                    statement_span(statement),
                    format!("function `{name}` repeats an input signature and staticness"),
                ));
            }
            overloads.push(function);
        }
    }
    Ok(())
}

fn exact_character_generator_close_handler(
    source: &SourceText,
    initial: Span,
    body: &[Statement],
) -> Option<CompilerGeneratorCloseHandler> {
    let [
        Statement::Binding {
            name: result_binding,
            classifier: None,
            value: Expression::Application {
                items: yield_items, ..
            },
        },
        Statement::Expression(Expression::DecisionTable {
            subject,
            rules,
            span,
        }),
    ] = body
    else {
        return None;
    };
    let [
        Expression::Identifier(yield_operation),
        Expression::Identifier(yielded),
    ] = yield_items.as_slice()
    else {
        return None;
    };
    let Expression::Identifier(subject) = subject.as_ref() else {
        return None;
    };
    if source.slice(*yield_operation) != "yield"
        || source.slice(*yielded) != source.slice(initial)
        || source.slice(*subject) != source.slice(*result_binding)
        || source.slice(*result_binding) == "_"
        || source.slice(*result_binding) == source.slice(initial)
    {
        return None;
    }
    let mut error = None;
    let mut ok = None;
    let mut error_codes = Vec::new();
    for rule in rules {
        let Expression::Unit(action_span) = &rule.action else {
            return None;
        };
        let action = unit_expression(*action_span);
        match &rule.matcher {
            DecisionMatcher::Result {
                error: true,
                binding,
                ..
            } if error.is_none() && source.slice(*binding) != "_" => {
                error = Some((*binding, action));
            }
            DecisionMatcher::Result {
                error: false,
                binding,
                ..
            } if ok.is_none() && source.slice(*binding) != "_" => {
                ok = Some((*binding, action));
            }
            DecisionMatcher::ErrorCode {
                namespace,
                vocabulary,
                code,
                ..
            } if error.is_none() && error_codes.is_empty() => {
                let (_, code) = generator_error_code(
                    source.slice(*namespace),
                    source.slice(*vocabulary),
                    source.slice(*code),
                )?;
                error_codes.push(CompilerErrorCodeRule {
                    code,
                    action,
                    span: rule.span,
                });
            }
            _ => return None,
        }
    }
    let (error_binding, error_action) = error?;
    let (ok_binding, ok_action) = ok?;
    let (error_code_type, _) = generator_error_code("lang", "generator", "generator-closed")?;
    Some(CompilerGeneratorCloseHandler {
        result_binding: source.slice(*result_binding).to_owned(),
        result_binding_span: *result_binding,
        error_codes,
        error_binding: source.slice(error_binding).to_owned(),
        error_binding_span: error_binding,
        error_action: Box::new(error_action),
        ok_binding: source.slice(ok_binding).to_owned(),
        ok_binding_span: ok_binding,
        ok_action: Box::new(ok_action),
        error_code_type,
        span: *span,
    })
}

fn exact_generator_local_close_call(
    source: &SourceText,
    expression: &Expression,
    alternative: &str,
    value: u32,
    enumeration: &CompilerEnumType,
) -> Option<CompilerExpression> {
    let Expression::Application { items, span } = expression else {
        return None;
    };
    let [
        Expression::Identifier(function),
        Expression::Identifier(argument),
    ] = items.as_slice()
    else {
        return None;
    };
    if source.slice(*function) != "cleanup" || source.slice(*argument) != alternative {
        return None;
    }
    Some(CompilerExpression {
        kind: CompilerExpressionKind::Call {
            symbol: String::new(),
            arguments: vec![CompilerExpression {
                kind: CompilerExpressionKind::Enum(value),
                value_type: CompilerType::Enum(enumeration.clone()),
                int_range: None,
                rational_value: None,
                span: *argument,
            }],
        },
        value_type: CompilerType::Unit,
        int_range: None,
        rational_value: None,
        span: *span,
    })
}

#[allow(clippy::too_many_lines)] // Exact restored declaration and close branches form one proof.
fn exact_character_generator_local_close_handler(
    source: &SourceText,
    initial: Span,
    body: &[Statement],
    span: Span,
) -> Result<(CompilerGeneratorCloseHandler, CompilerFunction), Diagnostic> {
    let [
        enum_statement,
        function_statement,
        binding_statement,
        decision_statement,
    ] = body
    else {
        return Err(unsupported(
            source,
            span,
            "generator-local close body outside one enum, function, bound yield, and Result decision",
        ));
    };
    let Some(EnumSource {
        name: enum_name,
        alternatives,
        ..
    }) = enum_declaration(source, enum_statement)
    else {
        return Err(unsupported(source, span, "generator-local close enum"));
    };
    if source.slice(enum_name) != "CloseChoice"
        || alternatives.len() != 2
        || alternatives[0].0 != "Closed"
        || alternatives[1].0 != "Continued"
    {
        return Err(unsupported(
            source,
            enum_name,
            "generator-local close enum outside CloseChoice (Closed, Continued)",
        ));
    }
    let enumeration = CompilerEnumType {
        name: String::from("CloseChoice"),
        alternatives: vec![String::from("Closed"), String::from("Continued")],
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
            "generator-local close function declaration",
        ));
    };
    let [function_parameter] = parameters.as_slice() else {
        return Err(unsupported(
            source,
            *function_span,
            "generator-local close function arity",
        ));
    };
    if source.slice(*function_name) != "cleanup"
        || !function_parameter.fields.is_empty()
        || function_parameter.default.is_some()
        || function_parameter.qualifier.is_some()
        || source.slice(function_parameter.name) != "choice"
        || source.slice(function_parameter.classifier) != "CloseChoice"
        || source.slice(*result) != "Unit"
        || **clauses != FunctionClauses::default()
    {
        return Err(unsupported(
            source,
            *function_span,
            "generator-local close function outside cleanup (choice : CloseChoice) -> Unit",
        ));
    }
    let [Statement::Expression(Expression::Unit(function_result_span))] = function_body.as_slice()
    else {
        return Err(unsupported(
            source,
            *function_span,
            "generator-local cleanup body outside Unit",
        ));
    };

    let Statement::Binding {
        name: result_binding,
        classifier: None,
        value: Expression::Application {
            items: yield_items, ..
        },
    } = binding_statement
    else {
        return Err(unsupported(
            source,
            statement_span(binding_statement),
            "generator-local close result binding",
        ));
    };
    let [
        Expression::Identifier(yield_operation),
        Expression::Identifier(yielded),
    ] = yield_items.as_slice()
    else {
        return Err(unsupported(
            source,
            statement_span(binding_statement),
            "generator-local close yield",
        ));
    };
    if source.slice(*yield_operation) != "yield"
        || source.slice(*yielded) != source.slice(initial)
        || source.slice(*result_binding) != "resume-result"
        || source.slice(*result_binding) == source.slice(initial)
    {
        return Err(unsupported(
            source,
            statement_span(binding_statement),
            "generator-local close outside resume-result is yield initial",
        ));
    }

    let Statement::Expression(Expression::DecisionTable {
        subject,
        rules,
        span: decision_span,
    }) = decision_statement
    else {
        return Err(unsupported(
            source,
            statement_span(decision_statement),
            "generator-local close Result decision",
        ));
    };
    let Expression::Identifier(subject_name) = subject.as_ref() else {
        return Err(unsupported(
            source,
            subject.span(),
            "generator-local close Result subject",
        ));
    };
    let [closed_rule, error_rule, ok_rule] = rules.as_slice() else {
        return Err(unsupported(
            source,
            *decision_span,
            "generator-local close Result branches",
        ));
    };
    let (
        DecisionMatcher::ErrorCode {
            namespace,
            vocabulary,
            code,
            ..
        },
        DecisionMatcher::Result {
            error: true,
            binding: error_binding,
            ..
        },
        DecisionMatcher::Result {
            error: false,
            binding: ok_binding,
            ..
        },
    ) = (&closed_rule.matcher, &error_rule.matcher, &ok_rule.matcher)
    else {
        return Err(unsupported(
            source,
            *decision_span,
            "generator-local close Result branch order",
        ));
    };
    if source.slice(*subject_name) != source.slice(*result_binding)
        || source.slice(*namespace) != "lang"
        || source.slice(*vocabulary) != "generator"
        || source.slice(*code) != "generator-closed"
        || source.slice(*error_binding) != "problem"
        || source.slice(*ok_binding) != "resumed"
    {
        return Err(unsupported(
            source,
            *decision_span,
            "generator-local close outside the retained qualified Result branches",
        ));
    }
    let closed_action =
        exact_generator_local_close_call(source, &closed_rule.action, "Closed", 0, &enumeration)
            .ok_or_else(|| {
                unsupported(
                    source,
                    closed_rule.action.span(),
                    "generator-local close action outside cleanup Closed",
                )
            })?;
    let Expression::Unit(error_action_span) = &error_rule.action else {
        return Err(unsupported(
            source,
            error_rule.action.span(),
            "generator-local fallback close action outside Unit",
        ));
    };
    let ok_action =
        exact_generator_local_close_call(source, &ok_rule.action, "Continued", 1, &enumeration)
            .ok_or_else(|| {
                unsupported(
                    source,
                    ok_rule.action.span(),
                    "generator-local successful-resume action outside cleanup Continued",
                )
            })?;
    let (error_code_type, code) = generator_error_code("lang", "generator", "generator-closed")
        .expect("the intrinsic generator close code exists");
    let local_parameter = CompilerParameter {
        name: String::from("choice"),
        discarded: false,
        source_visible: true,
        value_type: CompilerType::Enum(enumeration),
        int_range: None,
        span: function_parameter.name,
    };
    Ok((
        CompilerGeneratorCloseHandler {
            result_binding: String::from("resume-result"),
            result_binding_span: *result_binding,
            error_codes: vec![CompilerErrorCodeRule {
                code,
                action: closed_action,
                span: closed_rule.span,
            }],
            error_binding: String::from("problem"),
            error_binding_span: *error_binding,
            error_action: Box::new(unit_expression(*error_action_span)),
            ok_binding: String::from("resumed"),
            ok_binding_span: *ok_binding,
            ok_action: Box::new(ok_action),
            error_code_type,
            span: *decision_span,
        },
        CompilerFunction {
            source_name: String::from("cleanup"),
            module_identity: None,
            symbol: String::new(),
            parameters: vec![local_parameter],
            pattern_identities: Vec::new(),
            result_type: CompilerType::Unit,
            result_captures: Vec::new(),
            body: CompilerBlock {
                statements: Vec::new(),
                result: unit_expression(*function_result_span),
            },
            span: *function_span,
            is_static: false,
            declared_effects: None,
            foreign: None,
        },
    ))
}

#[allow(clippy::too_many_lines)] // The exact prefix and yield shape is rejected as one atomic proof.
fn exact_string_input_generator_body(
    source: &SourceText,
    parameter: &FunctionParameter,
    body: &[Statement],
    span: Span,
) -> Result<(CompilerBlock, Vec<String>), Diagnostic> {
    let [
        Statement::Binding {
            name: binding_name,
            classifier: Some(binding_classifier),
            value:
                Expression::Application {
                    items: predicate_items,
                    span: predicate_span,
                },
        },
        Statement::Discard {
            value: Expression::Application {
                items: yield_items, ..
            },
            ..
        },
        Statement::Expression(Expression::Unit(final_span)),
    ] = body
    else {
        return Err(unsupported(
            source,
            span,
            "String-input custom generator outside one empty? binding followed by one Character yield",
        ));
    };
    let [
        Expression::Identifier(predicate),
        Expression::Identifier(predicate_operand),
    ] = predicate_items.as_slice()
    else {
        return Err(unsupported(
            source,
            span,
            "String-input custom generator predicate outside empty? initial",
        ));
    };
    let [
        Expression::Identifier(yield_operation),
        Expression::String(yield_literal),
    ] = yield_items.as_slice()
    else {
        return Err(unsupported(
            source,
            span,
            "String-input custom generator yield outside one Character literal",
        ));
    };
    if source.slice(*binding_name) == "_"
        || source.slice(*binding_name) == source.slice(parameter.name)
        || source.slice(*binding_classifier) != "Boolean"
        || source.slice(*predicate) != "empty?"
        || source.slice(*predicate_operand) != source.slice(parameter.name)
        || source.slice(*yield_operation) != "yield"
    {
        return Err(unsupported(
            source,
            span,
            "String-input custom generator outside one named Boolean empty? initial binding",
        ));
    }
    let character = parse_string(source.slice(*yield_literal)).ok_or_else(|| {
        source_diagnostic(
            source,
            "E-STRING-LITERAL",
            *yield_literal,
            "invalid string literal delimiter",
        )
    })?;
    let count = character_count(character);
    if count != 1 {
        return Err(source_diagnostic(
            source,
            "E-CHARACTER-CLASSIFIER",
            *yield_literal,
            format!(
                "Character requires exactly one user-perceived character, but this String contains {count}"
            ),
        ));
    }
    let initial = CompilerExpression {
        kind: CompilerExpressionKind::Local(source.slice(parameter.name).to_owned()),
        value_type: CompilerType::String,
        int_range: None,
        rational_value: None,
        span: parameter.name,
    };
    let predicate = CompilerExpression {
        kind: CompilerExpressionKind::StringEmptyPredicate(Box::new(initial)),
        value_type: CompilerType::Boolean,
        int_range: None,
        rational_value: None,
        span: *predicate_span,
    };
    let binding_name_text = source.slice(*binding_name).to_owned();
    Ok((
        CompilerBlock {
            statements: vec![CompilerStatement::Binding(CompilerBinding {
                name: binding_name_text.clone(),
                storage_name: binding_name_text,
                value: predicate,
                span: *binding_name,
            })],
            result: unit_expression(*final_span),
        },
        vec![character.to_owned()],
    ))
}

fn exact_string_literal_expression(
    source: &SourceText,
    literal: Span,
) -> Result<CompilerExpression, Diagnostic> {
    let value = parse_string(source.slice(literal)).ok_or_else(|| {
        source_diagnostic(
            source,
            "E-STRING-LITERAL",
            literal,
            "invalid string literal delimiter",
        )
    })?;
    Ok(CompilerExpression {
        kind: CompilerExpressionKind::String(value.to_owned()),
        value_type: CompilerType::String,
        int_range: None,
        rational_value: None,
        span: literal,
    })
}

fn exact_string_empty_discard(
    source: &SourceText,
    parameter: &FunctionParameter,
    statement: &Statement,
) -> Option<CompilerStatement> {
    let Statement::Discard {
        value:
            Expression::Application {
                items,
                span: predicate_span,
            },
        ..
    } = statement
    else {
        return None;
    };
    let [
        Expression::Identifier(operation),
        Expression::Identifier(operand),
    ] = items.as_slice()
    else {
        return None;
    };
    if source.slice(*operation) != "empty?"
        || source.slice(*operand) != source.slice(parameter.name)
    {
        return None;
    }
    Some(CompilerStatement::Discard(CompilerExpression {
        kind: CompilerExpressionKind::StringEmptyPredicate(Box::new(CompilerExpression {
            kind: CompilerExpressionKind::Local(source.slice(*operand).to_owned()),
            value_type: CompilerType::String,
            int_range: None,
            rational_value: None,
            span: *operand,
        })),
        value_type: CompilerType::Boolean,
        int_range: None,
        rational_value: None,
        span: *predicate_span,
    }))
}

struct ExactValueGeneratorBody {
    yields: Vec<CompilerGeneratorYield>,
    continuations: Vec<CompilerGeneratorContinuation>,
    explicit_return: Option<Span>,
    result: CompilerExpression,
}

#[allow(clippy::too_many_lines)] // Exact yield/continuation phase proof stays fail-closed together.
fn exact_string_value_generator_body(
    source: &SourceText,
    parameter: &FunctionParameter,
    result_classifier: Span,
    body: &[Statement],
    span: Span,
) -> Result<ExactValueGeneratorBody, Diagnostic> {
    let Some((final_statement, yield_statements)) = body.split_last() else {
        return Err(unsupported(
            source,
            span,
            "String-value custom generator without a final expression",
        ));
    };
    let (result, explicit_return) = match (source.slice(result_classifier), final_statement) {
        ("Unit", Statement::Expression(Expression::Unit(result_span))) => {
            (unit_expression(*result_span), None)
        }
        ("String", Statement::Expression(Expression::String(literal))) => {
            (exact_string_literal_expression(source, *literal)?, None)
        }
        (
            "String",
            Statement::Return {
                keyword,
                value: Expression::String(literal),
            },
        ) => (
            exact_string_literal_expression(source, *literal)?,
            Some(*keyword),
        ),
        _ => {
            return Err(unsupported(
                source,
                statement_span(final_statement),
                "String-value custom generator final expression outside its declared exact Unit or String result",
            ));
        }
    };
    if explicit_return.is_some() && yield_statements.is_empty() {
        return Ok(ExactValueGeneratorBody {
            yields: Vec::new(),
            continuations: Vec::new(),
            explicit_return,
            result,
        });
    }
    if yield_statements.is_empty() {
        return Err(unsupported(
            source,
            span,
            "String-yield custom generator without a suspension",
        ));
    }
    let mut yields = Vec::with_capacity(yield_statements.len());
    let mut continuations = Vec::new();
    for statement in yield_statements {
        if let Some(discard) = exact_string_empty_discard(source, parameter, statement) {
            if result.value_type != CompilerType::Unit
                || yields.is_empty()
                || !continuations.is_empty()
            {
                return Err(unsupported(
                    source,
                    statement_span(statement),
                    "String-yield custom generator outside one discarded empty? initial computation after a suspension",
                ));
            }
            let continuation_span = statement_span(statement);
            continuations.push(CompilerGeneratorContinuation {
                after_resumptions: yields.len(),
                body: CompilerBlock {
                    statements: vec![discard],
                    result: unit_expression(continuation_span),
                },
            });
            continue;
        }
        let Statement::Discard {
            value:
                Expression::Application {
                    items,
                    span: yield_span,
                },
            ..
        } = statement
        else {
            return Err(unsupported(
                source,
                span,
                "String-yield custom generator outside consecutive discarded yields",
            ));
        };
        let [Expression::Identifier(operation), value] = items.as_slice() else {
            return Err(unsupported(
                source,
                *yield_span,
                "String-yield custom generator yield expression",
            ));
        };
        if source.slice(*operation) != "yield" {
            return Err(unsupported(
                source,
                *yield_span,
                "String-yield custom generator outside consecutive discarded yields",
            ));
        }
        match value {
            Expression::Identifier(name) if source.slice(*name) == source.slice(parameter.name) => {
                yields.push(CompilerGeneratorYield::Initial(*yield_span));
            }
            Expression::String(literal) => {
                yields.push(CompilerGeneratorYield::Value(Box::new(
                    exact_string_literal_expression(source, *literal)?,
                )));
            }
            _ => {
                return Err(unsupported(
                    source,
                    value.span(),
                    "String-yield custom generator value outside its initial input or an exact literal",
                ));
            }
        }
    }
    if continuations
        .last()
        .is_some_and(|continuation| continuation.after_resumptions == yields.len())
    {
        return Err(unsupported(
            source,
            span,
            "String-yield custom generator discarded computation without a following suspension",
        ));
    }
    if explicit_return.is_some()
        && !matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
    {
        return Err(unsupported(
            source,
            span,
            "explicit String return outside exactly one initial-parameter suspension",
        ));
    }
    Ok(ExactValueGeneratorBody {
        yields,
        continuations,
        explicit_return,
        result,
    })
}

fn exact_boolean_value_generator_body(
    source: &SourceText,
    parameter: &FunctionParameter,
    result_classifier: Span,
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
        Statement::Expression(result_expression),
    ] = body
    else {
        return Err(unsupported(
            source,
            span,
            "Boolean-yield custom generator outside one initial yield and an admitted final expression",
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
            "Boolean-value custom generator yield outside its initial parameter",
        ));
    };
    if source.slice(*yield_operation) != "yield"
        || source.slice(*yield_value) != source.slice(parameter.name)
    {
        return Err(unsupported(
            source,
            span,
            "Boolean-yield custom generator outside yield initial",
        ));
    }
    let initial = CompilerExpression {
        kind: CompilerExpressionKind::Local(source.slice(parameter.name).to_owned()),
        value_type: CompilerType::Boolean,
        int_range: None,
        rational_value: None,
        span: parameter.name,
    };
    let result = exact_boolean_generator_result(
        source,
        parameter,
        result_classifier,
        result_expression,
        initial,
    )?;
    Ok(ExactValueGeneratorBody {
        yields: vec![CompilerGeneratorYield::Initial(*yield_span)],
        continuations: Vec::new(),
        explicit_return: None,
        result,
    })
}
