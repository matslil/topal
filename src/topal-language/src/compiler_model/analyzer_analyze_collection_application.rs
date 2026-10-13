impl Analyzer {
    #[allow(clippy::too_many_lines)] // Collection operations and final call resolution remain ordered.
    fn analyze_collection_application(
        &mut self,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        if let [
            list,
            Expression::Identifier(operation),
            Expression::AnonymousFunction {
                parameters,
                body,
                span: function_span,
            },
        ] = items
            && matches!(self.source.slice(*operation), "map" | "select")
        {
            let operation = self.source.slice(*operation).to_owned();
            let list = self.analyze_expression(list, environment)?;
            if operation == "map"
                && let CompilerExpressionKind::Call { symbol, arguments } = &list.kind
                && self.instances.iter().any(|function| {
                    function.symbol == *symbol && function.source_name.ends_with(".positions")
                })
                && let [
                    CompilerExpression {
                        kind: CompilerExpressionKind::ListEntryCount(source),
                        ..
                    },
                ] = arguments.as_slice()
                && let [AnonymousPattern::Binding(index)] = parameters.as_slice()
                && let Expression::Application {
                    items: insertion, ..
                } = &**body
                && let [
                    body_list,
                    Expression::Identifier(insert_at),
                    Expression::Identifier(boundary),
                    inserted,
                ] = insertion.as_slice()
                && self.source.slice(*insert_at) == "insert-at"
                && self.source.slice(*boundary) == self.source.slice(*index)
            {
                let body_list = self.analyze_expression(body_list, environment)?;
                let same_source = matches!(
                    (&source.kind, &body_list.kind),
                    (CompilerExpressionKind::Local(left), CompilerExpressionKind::Local(right))
                        if left == right
                );
                if same_source {
                    require_int_list(&self.source, &body_list, "insert-everywhere source")?;
                    let inserted = self.analyze_expression(inserted, environment)?;
                    require_type(
                        &self.source,
                        inserted.span,
                        &CompilerType::Int,
                        &inserted.value_type,
                    )?;
                    return Ok(CompilerExpression {
                        kind: CompilerExpressionKind::ListInsertEverywhere {
                            list: Box::new(body_list),
                            value: Box::new(inserted),
                        },
                        value_type: CompilerType::List(Box::new(int_list_type())),
                        int_range: None,
                        rational_value: None,
                        span,
                    });
                }
            }
            let nested_int_map = operation == "map"
                && matches!(&list.value_type, CompilerType::List(element)
                    if compiler_nested_int_list_element(element.as_ref()));
            let nested_int_select = operation == "select"
                && matches!(&list.value_type, CompilerType::List(element)
                    if compiler_nested_int_list_element(element.as_ref()));
            let parameter_type = if nested_int_map {
                int_list_type()
            } else if operation == "map" {
                require_int_or_int_pair_list(&self.source, &list, "map subject")?
            } else if nested_int_select {
                int_list_type()
            } else {
                require_int_list(&self.source, &list, "select subject")?;
                CompilerType::Int
            };
            let (parameters, body) = self.analyze_collection_function(
                parameters,
                body,
                &[parameter_type],
                environment,
                self.static_context,
                *function_span,
            )?;
            let expected = if nested_int_map {
                int_list_type()
            } else if operation == "map" {
                CompilerType::Int
            } else {
                CompilerType::Boolean
            };
            require_type(
                &self.source,
                body.result.span,
                &expected,
                &body.result.value_type,
            )?;
            let value_type = if operation == "select" {
                list.value_type.clone()
            } else {
                CompilerType::List(Box::new(expected))
            };
            let kind = if operation == "map" {
                CompilerExpressionKind::ListMap {
                    list: Box::new(list),
                    parameters,
                    body: Box::new(body),
                }
            } else {
                CompilerExpressionKind::ListSelect {
                    list: Box::new(list),
                    parameters,
                    body: Box::new(body),
                }
            };
            return Ok(CompilerExpression {
                kind,
                value_type,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(operation), generator] = items
            && self.source.slice(*operation) == "collect"
        {
            let retained_generator = if let Expression::Identifier(name) = generator {
                environment
                    .get(self.source.slice(*name))
                    .and_then(|facts| self.generator_values.get(&facts.storage_name))
                    .cloned()
            } else {
                None
            };
            let direct_bounded = direct_bounded_iterate_functions(&self.source, generator);
            if let Some((next, predicate)) = direct_bounded {
                for (parameters, body, function_span, role) in [
                    (next.0, next.1, next.2, "iterate operation"),
                    (
                        predicate.0,
                        predicate.1,
                        predicate.2,
                        "take-while predicate",
                    ),
                ] {
                    if let Some(capture) =
                        anonymous_body_capture(&self.source, parameters, body, environment)
                    {
                        let admitted_predicate_bound = role == "take-while predicate"
                            && environment.get(&capture).is_some_and(|facts| {
                                facts.runtime_bound
                                    && (facts.value_type == CompilerType::Nat
                                        || (self.in_function
                                            && facts.value_type == CompilerType::Int)
                                        || matches!(
                                            &facts.value_type,
                                            CompilerType::List(element)
                                                if matches!(
                                                    element.as_ref(),
                                                    CompilerType::Int | CompilerType::Nat
                                                ) || compiler_nested_int_list_element(
                                                    element.as_ref()
                                                )
                                        ))
                            });
                        if admitted_predicate_bound {
                            continue;
                        }
                        return Err(unsupported(
                            &self.source,
                            function_span,
                            &format!("captured {role} value `{capture}`"),
                        ));
                    }
                }
            }
            let generator = self.analyze_expression(generator, environment)?;
            if let CompilerExpressionKind::StringRangeCharactersGenerator {
                text,
                characters,
                range,
            } = generator.kind
            {
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::StringRangeCharactersCollect {
                        text,
                        characters,
                        range,
                    },
                    value_type: CompilerType::List(Box::new(CompilerType::Character)),
                    int_range: None,
                    rational_value: None,
                    span,
                });
            }
            if let CompilerExpressionKind::StringProvenanceCharactersGenerator {
                text,
                characters,
            } = generator.kind
            {
                if characters.is_empty() {
                    return Ok(CompilerExpression {
                        kind: CompilerExpressionKind::StringDynamicScalarCharactersCollect(text),
                        value_type: CompilerType::List(Box::new(CompilerType::Character)),
                        int_range: None,
                        rational_value: None,
                        span,
                    });
                }
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::StringProvenanceCharactersCollect {
                        text,
                        characters,
                    },
                    value_type: CompilerType::List(Box::new(CompilerType::Character)),
                    int_range: None,
                    rational_value: None,
                    span,
                });
            }
            if let CompilerExpressionKind::StringCharactersGenerator { characters, .. } =
                &generator.kind
            {
                let list_type = CompilerType::List(Box::new(CompilerType::Character));
                let mut list = CompilerExpression {
                    kind: CompilerExpressionKind::ListEmpty,
                    value_type: list_type.clone(),
                    int_range: None,
                    rational_value: None,
                    span,
                };
                for character in characters.iter().rev() {
                    let value = CompilerExpression {
                        kind: CompilerExpressionKind::String(character.clone()),
                        value_type: CompilerType::Character,
                        int_range: None,
                        rational_value: None,
                        span: generator.span,
                    };
                    list = CompilerExpression {
                        kind: CompilerExpressionKind::ListEntry {
                            value: Box::new(value),
                            remaining: Box::new(list),
                        },
                        value_type: list_type.clone(),
                        int_range: None,
                        rational_value: None,
                        span,
                    };
                }
                return Ok(list);
            }
            require_type(
                &self.source,
                generator.span,
                &int_unit_generator_type(),
                &generator.value_type,
            )?;
            let collection_generator = retained_generator.as_ref().unwrap_or(&generator);
            match &collection_generator.kind {
                CompilerExpressionKind::GeneratorTakeWhile {
                    generator: source, ..
                } if retained_generator.is_none()
                    && matches!(source.kind, CompilerExpressionKind::IterateGenerator { .. }) =>
                {
                    return Ok(CompilerExpression {
                        kind: CompilerExpressionKind::GeneratorCollect(Box::new(
                            collection_generator.clone(),
                        )),
                        value_type: CompilerType::List(Box::new(CompilerType::Int)),
                        int_range: None,
                        rational_value: None,
                        span,
                    });
                }
                CompilerExpressionKind::UnfoldGenerator { .. }
                    if directly_collectable_list_uncons_unfold(
                        collection_generator,
                        environment,
                    ) =>
                {
                    return Ok(CompilerExpression {
                        kind: CompilerExpressionKind::GeneratorCollect(Box::new(
                            collection_generator.clone(),
                        )),
                        value_type: CompilerType::List(Box::new(CompilerType::Int)),
                        int_range: None,
                        rational_value: None,
                        span,
                    });
                }
                CompilerExpressionKind::IterateGenerator { .. } => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-UNBOUNDED-GENERATOR-COLLECT",
                        generator.span,
                        "collect requires a statically finite generated traversal",
                    ));
                }
                _ => {
                    return Err(unsupported(
                        &self.source,
                        generator.span,
                        "collection through a non-direct Generator value",
                    ));
                }
            }
        }
        if let [
            initial,
            Expression::Identifier(iterate),
            Expression::AnonymousFunction {
                parameters: next_parameters,
                body: next_body,
                span: next_span,
            },
            Expression::Identifier(take_while),
            Expression::AnonymousFunction {
                parameters: predicate_parameters,
                body: predicate_body,
                span: predicate_span,
            },
        ] = items
            && self.source.slice(*iterate) == "iterate"
            && self.source.slice(*take_while) == "take-while"
        {
            let generator = self.analyze_int_iterate_generator(
                initial,
                next_parameters,
                next_body,
                *next_span,
                span,
                environment,
            )?;
            return self.analyze_int_generator_take_while(
                generator,
                predicate_parameters,
                predicate_body,
                *predicate_span,
                span,
                environment,
            );
        }
        if let [
            initial,
            Expression::Identifier(operation),
            Expression::AnonymousFunction {
                parameters,
                body,
                span: function_span,
            },
        ] = items
            && self.source.slice(*operation) == "iterate"
        {
            return self.analyze_int_iterate_generator(
                initial,
                parameters,
                body,
                *function_span,
                span,
                environment,
            );
        }
        if let [
            generator,
            Expression::Identifier(operation),
            Expression::AnonymousFunction {
                parameters,
                body,
                span: function_span,
            },
        ] = items
            && self.source.slice(*operation) == "take-while"
        {
            let generator = self.analyze_expression(generator, environment)?;
            return self.analyze_int_generator_take_while(
                generator,
                parameters,
                body,
                *function_span,
                span,
                environment,
            );
        }
        if let [
            seed,
            Expression::Identifier(operation),
            Expression::AnonymousFunction {
                parameters,
                body,
                span: function_span,
            },
        ] = items
            && self.source.slice(*operation) == "unfold"
        {
            return self.analyze_int_list_unfold_generator(
                seed,
                parameters,
                body,
                *function_span,
                span,
                environment,
            );
        }
        if let [
            list,
            Expression::Identifier(operation),
            initial,
            Expression::AnonymousFunction {
                parameters,
                body,
                span: function_span,
            },
        ] = items
            && self.source.slice(*operation) == "fold"
        {
            let list = self.analyze_expression(list, environment)?;
            let mut initial = self.analyze_expression(initial, environment)?;
            if let Some(expected) = self.fold_callback_state_type(parameters, body)
                && let Some(adapted) = adapt_function_call_argument(&expected, &initial)
            {
                initial = adapted;
            }
            if matches!(&list.value_type, CompilerType::List(element)
                if element.as_ref() == &CompilerType::String)
                && matches!(&initial.value_type, CompilerType::List(element)
                    if compiler_string_int_pair(element.as_ref()))
                && let [
                    AnonymousPattern::Binding(state),
                    AnonymousPattern::Binding(candidate),
                ] = parameters.as_slice()
                && let Expression::Application { items: concat, .. } = &**body
                && let [
                    Expression::Identifier(state_use),
                    Expression::Identifier(concat_name),
                    pair_call,
                ] = concat.as_slice()
                && self.source.slice(*state_use) == self.source.slice(*state)
                && self.source.slice(*concat_name) == "concat"
                && let Expression::Application {
                    items: pair_items, ..
                } = pair_call
                && let [
                    Expression::Identifier(pair_with),
                    Expression::Product { fields, .. },
                ] = pair_items.as_slice()
                && self.source.slice(*pair_with) == "pair-with"
                && let [candidate_field, right_field] = fields.as_slice()
                && candidate_field.label.is_none()
                && right_field.label.is_none()
                && matches!(&candidate_field.value, Expression::Identifier(name)
                    if self.source.slice(*name) == self.source.slice(*candidate))
            {
                let right = self.analyze_expression(&right_field.value, environment)?;
                require_type(
                    &self.source,
                    right.span,
                    &int_list_type(),
                    &right.value_type,
                )?;
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::ListCartesianStringInt {
                        left: Box::new(list),
                        right: Box::new(right),
                    },
                    value_type: initial.value_type,
                    int_range: None,
                    rational_value: None,
                    span,
                });
            }
            if matches!(&list.value_type, CompilerType::List(element)
                if compiler_nat_pair(element.as_ref()))
                && initial.value_type == CompilerType::Int
            {
                let initial_span = initial.span;
                initial = self.finish_nat_conversion(initial, span, initial_span)?;
            }
            let (state_type, mut element_type) = match (&list.value_type, &initial.value_type) {
                (CompilerType::List(element), CompilerType::Int)
                    if matches!(element.as_ref(), CompilerType::Int | CompilerType::Nat)
                        || compiler_integer_pair(element.as_ref())
                        || compiler_int_triple(element.as_ref())
                        || compiler_int_int_list_triple(element.as_ref()) =>
                {
                    (CompilerType::Int, element.as_ref().clone())
                }
                (CompilerType::List(element), CompilerType::Optional(payload))
                    if element.as_ref()
                        == &CompilerType::Tuple(vec![
                            CompilerType::String,
                            CompilerType::String,
                        ])
                        && payload.as_ref() == &CompilerType::String =>
                {
                    (
                        CompilerType::Optional(Box::new(CompilerType::String)),
                        element.as_ref().clone(),
                    )
                }
                (CompilerType::List(element), CompilerType::Optional(payload))
                    if compiler_integer_pair(element.as_ref())
                        && payload.as_ref() == &CompilerType::Nat =>
                {
                    (
                        CompilerType::Optional(Box::new(CompilerType::Nat)),
                        element.as_ref().clone(),
                    )
                }
                (CompilerType::List(element), CompilerType::Optional(payload))
                    if compiler_nested_int_list_element(element.as_ref())
                        && payload.as_ref() == &CompilerType::Nat =>
                {
                    (
                        CompilerType::Optional(Box::new(CompilerType::Nat)),
                        element.as_ref().clone(),
                    )
                }
                (CompilerType::List(element), CompilerType::Optional(payload))
                    if element.as_ref() == payload.as_ref()
                        && compiler_list_node_element_supported(element.as_ref()) =>
                {
                    (
                        CompilerType::Optional(payload.clone()),
                        element.as_ref().clone(),
                    )
                }
                (CompilerType::List(element), CompilerType::Nat)
                    if matches!(
                        element.as_ref(),
                        CompilerType::Int | CompilerType::Nat | CompilerType::Rational
                    ) || compiler_integer_pair(element.as_ref())
                        || compiler_rational_nat_pair(element.as_ref())
                        || compiler_string_pair(element.as_ref())
                        || compiler_int_triple(element.as_ref())
                        || compiler_list_node_element_supported(element.as_ref()) =>
                {
                    (CompilerType::Nat, element.as_ref().clone())
                }
                (CompilerType::List(element), CompilerType::Rational)
                    if element.as_ref() == &CompilerType::Rational =>
                {
                    (CompilerType::Rational, CompilerType::Rational)
                }
                (CompilerType::List(element), CompilerType::Boolean)
                    if element.as_ref() == &CompilerType::Nat
                        || element.as_ref() == &CompilerType::String
                        || element.as_ref() == &CompilerType::Int
                        || compiler_nested_int_list_element(element.as_ref())
                        || compiler_nested_integer_pair_list_element(element.as_ref())
                        || compiler_integer_pair(element.as_ref())
                        || compiler_int_string_pair(element.as_ref())
                        || compiler_int_list_pair(element.as_ref()) =>
                {
                    let entry_type = if element.as_ref() == &CompilerType::Int
                        && self.compiler_proven_nonnegative_list(&list)
                    {
                        CompilerType::Nat
                    } else if compiler_nested_int_list_element(element.as_ref())
                        || compiler_nested_integer_pair_list_element(element.as_ref())
                        || compiler_integer_pair(element.as_ref())
                        || compiler_int_string_pair(element.as_ref())
                        || compiler_int_list_pair(element.as_ref())
                        || element.as_ref() == &CompilerType::String
                    {
                        element.as_ref().clone()
                    } else {
                        CompilerType::Nat
                    };
                    (CompilerType::Boolean, entry_type)
                }
                (CompilerType::List(element), CompilerType::Int)
                    if element.as_ref() == &CompilerType::String =>
                {
                    (CompilerType::Int, CompilerType::String)
                }
                (CompilerType::List(element), CompilerType::Int)
                    if compiler_string_string_list_int_triple(element.as_ref()) =>
                {
                    (CompilerType::Int, element.as_ref().clone())
                }
                (CompilerType::List(element), CompilerType::String)
                    if element.as_ref() == &CompilerType::Character =>
                {
                    (CompilerType::String, CompilerType::Character)
                }
                (CompilerType::List(element), CompilerType::List(state_element))
                    if state_element.as_ref() == &CompilerType::String
                        && (matches!(
                            element.as_ref(),
                            CompilerType::Character | CompilerType::String
                        ) || compiler_string_pair(element.as_ref())) =>
                {
                    (
                        CompilerType::List(Box::new(CompilerType::String)),
                        element.as_ref().clone(),
                    )
                }
                (CompilerType::List(element), CompilerType::List(state_element))
                    if matches!(
                        element.as_ref(),
                        CompilerType::Character | CompilerType::Int
                    ) && state_element.as_ref() == &CompilerType::Nat =>
                {
                    (
                        CompilerType::List(Box::new(CompilerType::Nat)),
                        element.as_ref().clone(),
                    )
                }
                (CompilerType::List(element), CompilerType::List(state_element))
                    if (matches!(
                        element.as_ref(),
                        CompilerType::Int | CompilerType::Nat | CompilerType::Rational
                    ) || compiler_integer_pair(element.as_ref())
                        || compiler_rational_nat_pair(element.as_ref()))
                        && matches!(
                            state_element.as_ref(),
                            CompilerType::Int | CompilerType::Nat | CompilerType::Rational
                        ) =>
                {
                    (
                        CompilerType::List(Box::new(state_element.as_ref().clone())),
                        element.as_ref().clone(),
                    )
                }
                (CompilerType::List(element), CompilerType::List(state_element))
                    if compiler_nested_int_list_element(element.as_ref())
                        && matches!(
                            state_element.as_ref(),
                            CompilerType::Int | CompilerType::Nat
                        ) =>
                {
                    (
                        CompilerType::List(Box::new(state_element.as_ref().clone())),
                        element.as_ref().clone(),
                    )
                }
                (CompilerType::List(element), CompilerType::List(state_element))
                    if compiler_list_node_element_supported(element.as_ref())
                        && compiler_list_node_element_supported(state_element.as_ref()) =>
                {
                    (
                        CompilerType::List(Box::new(state_element.as_ref().clone())),
                        element.as_ref().clone(),
                    )
                }
                (CompilerType::List(element), CompilerType::List(state_element))
                    if compiler_int_triple(element.as_ref())
                        && state_element.as_ref() == &CompilerType::Int =>
                {
                    (int_list_type(), element.as_ref().clone())
                }
                (CompilerType::List(element), CompilerType::List(state_element))
                    if compiler_list_node_element_supported(element.as_ref())
                        && matches!(state_element.as_ref(), CompilerType::Range(endpoint)
                            if endpoint.as_ref() == &CompilerType::Int) =>
                {
                    (
                        CompilerType::List(Box::new(state_element.as_ref().clone())),
                        element.as_ref().clone(),
                    )
                }
                (CompilerType::List(element), state @ CompilerType::Tuple(_))
                    if compiler_list_node_element_supported(element.as_ref()) =>
                {
                    (state.clone(), element.as_ref().clone())
                }
                (CompilerType::List(element), CompilerType::List(state_element))
                    if (compiler_nested_int_list_element(state_element.as_ref())
                        || compiler_nested_integer_pair_list_element(state_element.as_ref()))
                        && (element.as_ref() == &CompilerType::Int
                            || compiler_nested_int_list_element(element.as_ref())
                            || compiler_nested_integer_pair_list_element(element.as_ref())
                            || compiler_boolean_int_pair(element.as_ref())) =>
                {
                    (
                        CompilerType::List(Box::new(state_element.as_ref().clone())),
                        element.as_ref().clone(),
                    )
                }
                (CompilerType::List(_), _) => {
                    return Err(unsupported(
                        &self.source,
                        list.span,
                        "fold subject and state classifier combination",
                    ));
                }
                _ => {
                    return Err(unsupported(
                        &self.source,
                        list.span,
                        "fold subject for this List element type",
                    ));
                }
            };
            if element_type == CompilerType::Int && self.compiler_proven_nonnegative_list(&list) {
                element_type = CompilerType::Nat;
            }
            if parameters
                .iter()
                .all(|parameter| matches!(parameter, AnonymousPattern::Binding(_)))
            {
                let initial_facts = self
                    .known_structural_value_facts(&initial, environment)
                    .unwrap_or_default();
                let state_facts = merge_static_provenance(initial_facts.clone(), initial_facts);
                let list_facts = self
                    .known_structural_value_facts(&list, environment)
                    .unwrap_or_default();
                let mut entry_facts = StaticValueFacts::default();
                if element_type == CompilerType::String || element_type == CompilerType::Character {
                    entry_facts.string_characters = list_facts.list_string_characters;
                } else if compiler_type_contains_string(&element_type) {
                    retain_string_character_provenance_for_type(
                        &mut entry_facts,
                        &element_type,
                        list_facts.list_string_characters,
                    );
                }
                self.collection_parameter_facts = vec![state_facts, entry_facts];
            }
            let (parameters, body) = self.analyze_collection_function(
                parameters,
                body,
                &[state_type.clone(), element_type],
                environment,
                self.static_context,
                *function_span,
            )?;
            if state_type == CompilerType::Int {
                require_int_fold_result(&self.source, &body.result)?;
            } else {
                require_type(
                    &self.source,
                    body.result.span,
                    &state_type,
                    &body.result.value_type,
                )?;
            }
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListFold {
                    list: Box::new(list),
                    initial: Box::new(initial),
                    parameters,
                    body: Box::new(body),
                },
                value_type: state_type,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [
            list,
            Expression::Identifier(operation),
            Expression::Identifier(function),
        ] = items
            && matches!(self.source.slice(*operation), "map" | "select")
            && let Some(CompilerCallableFacts::Anonymous {
                parameters,
                body,
                captures,
                static_context,
                span: function_span,
            }) = environment
                .get(self.source.slice(*function))
                .and_then(|facts| facts.callable.as_ref())
                .cloned()
        {
            let operation = self.source.slice(*operation).to_owned();
            let list = self.analyze_expression(list, environment)?;
            let parameter_type = if operation == "map" {
                require_int_or_int_pair_list(&self.source, &list, "map subject")?
            } else {
                require_int_list(&self.source, &list, "select subject")?;
                CompilerType::Int
            };
            let (parameters, body) = self.analyze_collection_function(
                &parameters,
                &body,
                &[parameter_type],
                &captures,
                static_context,
                function_span,
            )?;
            let expected = if operation == "map" {
                CompilerType::Int
            } else {
                CompilerType::Boolean
            };
            require_type(
                &self.source,
                body.result.span,
                &expected,
                &body.result.value_type,
            )?;
            let kind = if operation == "map" {
                CompilerExpressionKind::ListMap {
                    list: Box::new(list),
                    parameters,
                    body: Box::new(body),
                }
            } else {
                CompilerExpressionKind::ListSelect {
                    list: Box::new(list),
                    parameters,
                    body: Box::new(body),
                }
            };
            return Ok(CompilerExpression {
                kind,
                value_type: CompilerType::List(Box::new(CompilerType::Int)),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [
            list,
            Expression::Identifier(operation),
            initial,
            Expression::Identifier(function),
        ] = items
            && self.source.slice(*operation) == "fold"
            && let Some(CompilerCallableFacts::Anonymous {
                parameters,
                body,
                captures,
                static_context,
                span: function_span,
            }) = environment
                .get(self.source.slice(*function))
                .and_then(|facts| facts.callable.as_ref())
                .cloned()
        {
            let list = self.analyze_expression(list, environment)?;
            require_int_list(&self.source, &list, "fold subject")?;
            let initial = self.analyze_expression(initial, environment)?;
            require_type(
                &self.source,
                initial.span,
                &CompilerType::Int,
                &initial.value_type,
            )?;
            let (parameters, body) = self.analyze_collection_function(
                &parameters,
                &body,
                &[CompilerType::Int, CompilerType::Int],
                &captures,
                static_context,
                function_span,
            )?;
            require_int_fold_result(&self.source, &body.result)?;
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListFold {
                    list: Box::new(list),
                    initial: Box::new(initial),
                    parameters,
                    body: Box::new(body),
                },
                value_type: CompilerType::Int,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [collection, Expression::Identifier(operation), selector] = items
            && matches!(self.source.slice(*operation), "select" | "select-index")
        {
            let operation = self.source.slice(*operation).to_owned();
            let collection = self.analyze_expression(collection, environment)?;
            let selector = self.analyze_expression(selector, environment)?;
            require_type(
                &self.source,
                selector.span,
                &CompilerType::Range(Box::new(CompilerType::Int)),
                &selector.value_type,
            )?;
            match collection.value_type.clone() {
                CompilerType::List(element)
                    if matches!(element.as_ref(), CompilerType::Int | CompilerType::Nat)
                        || (operation == "select-index"
                            && (matches!(
                                element.as_ref(),
                                CompilerType::Boolean
                                    | CompilerType::Character
                                    | CompilerType::String
                                    | CompilerType::Rational
                            ) || compiler_nested_int_list_element(element.as_ref())
                                || compiler_nested_string_list_element(element.as_ref())
                                || compiler_nested_integer_pair_list_element(
                                    element.as_ref(),
                                )
                                || compiler_integer_pair(element.as_ref())
                                || compiler_int_string_pair(element.as_ref())
                                || compiler_int_list_pair(element.as_ref())
                                || compiler_int_int_string_int_tuple(element.as_ref())
                                || compiler_list_integer_pair(element.as_ref())
                                || compiler_int_triple(element.as_ref()))) =>
                {
                    let value_type = CompilerType::List(element);
                    return Ok(CompilerExpression {
                        kind: CompilerExpressionKind::ListRangeSelect {
                            list: Box::new(collection),
                            range: Box::new(selector),
                            indexes: operation == "select-index",
                        },
                        value_type,
                        int_range: None,
                        rational_value: None,
                        span,
                    });
                }
                CompilerType::String if operation == "select-index" => {
                    let text = Self::known_string_value(&collection, environment);
                    let text_characters = text
                        .as_deref()
                        .map(|text| characters(text).map(str::to_owned).collect::<Vec<_>>())
                        .unwrap_or_default();
                    if text.is_some()
                        && let Some(range) = Self::known_closed_int_range(&selector, environment)
                    {
                        let selected = text_characters
                            .into_iter()
                            .enumerate()
                            .filter_map(|(index, character)| {
                                range.contains(&BigInt::from(index)).then_some(character)
                            })
                            .collect();
                        return Ok(CompilerExpression {
                            kind: CompilerExpressionKind::String(selected),
                            value_type: CompilerType::String,
                            int_range: None,
                            rational_value: None,
                            span,
                        });
                    }
                    return Ok(CompilerExpression {
                        kind: CompilerExpressionKind::StringRangeSelect {
                            text: Box::new(collection),
                            characters: text_characters,
                            range: Box::new(selector),
                        },
                        value_type: CompilerType::String,
                        int_range: None,
                        rational_value: None,
                        span,
                    });
                }
                CompilerType::List(_) => {
                    return Err(unsupported(
                        &self.source,
                        collection.span,
                        "range selection for this List element type",
                    ));
                }
                CompilerType::String => {
                    return Err(unsupported(
                        &self.source,
                        collection.span,
                        "Range Int value selection for String",
                    ));
                }
                _ => {
                    return Err(unsupported(
                        &self.source,
                        collection.span,
                        "range selection source",
                    ));
                }
            }
        }
        if let [list, Expression::Identifier(operation), operand] = items
            && matches!(
                self.source.slice(*operation),
                "contains-entry" | "contains-sequence" | "contains-subsequence"
            )
        {
            let operation = self.source.slice(*operation).to_owned();
            let list = self.analyze_expression(list, environment)?;
            let CompilerType::List(element) = &list.value_type else {
                return Err(unsupported(
                    &self.source,
                    list.span,
                    "List containment subject",
                ));
            };
            let element = element.as_ref().clone();
            let list_type = list.value_type.clone();
            if !(matches!(
                element,
                CompilerType::Int | CompilerType::Nat | CompilerType::Rational
            ) || matches!(element, CompilerType::Character | CompilerType::String)
                || compiler_integer_pair(&element)
                    && matches!(operation.as_str(), "contains-entry" | "contains-sequence"))
            {
                return Err(unsupported(
                    &self.source,
                    span,
                    "containment for this List element type",
                ));
            }
            let operand = if operation == "contains-entry" {
                self.analyze_expression_with_expected(operand, environment, Some(&element))?
            } else {
                self.analyze_expression(operand, environment)?
            };
            let kind = match operation.as_str() {
                "contains-entry" => {
                    require_same_type(&self.source, operand.span, &element, &operand.value_type)?;
                    CompilerExpressionKind::ListContainsEntry {
                        list: Box::new(list),
                        value: Box::new(operand),
                    }
                }
                "contains-sequence" => {
                    require_same_type(&self.source, operand.span, &list_type, &operand.value_type)?;
                    CompilerExpressionKind::ListContainsSequence {
                        list: Box::new(list),
                        pattern: Box::new(operand),
                    }
                }
                "contains-subsequence" => {
                    require_same_type(&self.source, operand.span, &list_type, &operand.value_type)?;
                    CompilerExpressionKind::ListContainsSubsequence {
                        list: Box::new(list),
                        pattern: Box::new(operand),
                    }
                }
                _ => unreachable!(),
            };
            return Ok(CompilerExpression {
                kind,
                value_type: CompilerType::Boolean,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [list, Expression::Identifier(operation), value] = items
            && matches!(self.source.slice(*operation), "remove-first" | "remove-all")
        {
            let operation = self.source.slice(*operation).to_owned();
            let list = self.analyze_expression(list, environment)?;
            let CompilerType::List(element) = &list.value_type else {
                return Err(unsupported(&self.source, list.span, "List removal subject"));
            };
            let element = element.as_ref().clone();
            let list_type = list.value_type.clone();
            if element != CompilerType::Int {
                return Err(unsupported(
                    &self.source,
                    span,
                    "removal for this List element type",
                ));
            }
            let value = self.analyze_expression(value, environment)?;
            require_same_type(&self.source, value.span, &element, &value.value_type)?;
            let kind = if operation == "remove-first" {
                CompilerExpressionKind::ListRemoveFirst {
                    list: Box::new(list),
                    value: Box::new(value),
                }
            } else {
                CompilerExpressionKind::ListRemoveAll {
                    list: Box::new(list),
                    value: Box::new(value),
                }
            };
            return Ok(CompilerExpression {
                kind,
                value_type: list_type,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(operation), operands] = items
            && matches!(
                self.source.slice(*operation),
                "array-at?" | "map-lookup" | "set-contains?" | "bag-multiplicity"
            )
        {
            let operation = self.source.slice(*operation).to_owned();
            let operands = self.analyze_expression(operands, environment)?;
            let CompilerExpressionKind::Tuple(mut fields) = operands.kind else {
                return Err(source_diagnostic(
                    &self.source,
                    "E-COLLECTION-QUERY-ARGUMENT",
                    operands.span,
                    format!("{operation} requires one two-field product"),
                ));
            };
            if fields.len() != 2 {
                return Err(source_diagnostic(
                    &self.source,
                    "E-COLLECTION-QUERY-ARGUMENT",
                    operands.span,
                    format!("{operation} requires one two-field product"),
                ));
            }
            let query = fields.pop().expect("checked two query fields");
            let collection = fields.pop().expect("checked two query fields");
            let (kind, value_type) = match operation.as_str() {
                "array-at?" => {
                    let CompilerType::Array { element, .. } = &collection.value_type else {
                        return Err(unsupported(
                            &self.source,
                            collection.span,
                            "array-at? collection kind",
                        ));
                    };
                    let element = element.clone();
                    require_type(
                        &self.source,
                        query.span,
                        &CompilerType::Int,
                        &query.value_type,
                    )?;
                    let index = Self::exact_usize(&query).ok_or_else(|| {
                        unsupported(&self.source, query.span, "dynamic or negative Array index")
                    })?;
                    (
                        CompilerExpressionKind::ArrayAt {
                            array: Box::new(collection),
                            index,
                        },
                        CompilerType::Optional(element),
                    )
                }
                "map-lookup" => {
                    let CompilerType::Map { key, value } = &collection.value_type else {
                        return Err(unsupported(
                            &self.source,
                            collection.span,
                            "map-lookup collection kind",
                        ));
                    };
                    let key_type = key.as_ref().clone();
                    let value = value.clone();
                    require_same_type(&self.source, query.span, &key_type, &query.value_type)?;
                    let exact_key = Self::known_string_expression(&query, environment);
                    if value.as_ref() == &CompilerType::Function && exact_key.is_none() {
                        return Err(unsupported(
                            &self.source,
                            query.span,
                            "dynamic Map Function lookup key",
                        ));
                    }
                    (
                        CompilerExpressionKind::MapLookup {
                            mapping: Box::new(collection),
                            key: Box::new(query),
                            exact_key,
                        },
                        CompilerType::Optional(value),
                    )
                }
                "set-contains?" => {
                    let CompilerType::Set(element) = &collection.value_type else {
                        return Err(unsupported(
                            &self.source,
                            collection.span,
                            "set-contains? collection kind",
                        ));
                    };
                    require_same_type(&self.source, query.span, element, &query.value_type)?;
                    (
                        CompilerExpressionKind::SetContains {
                            set: Box::new(collection),
                            value: Box::new(query),
                        },
                        CompilerType::Boolean,
                    )
                }
                "bag-multiplicity" => {
                    let CompilerType::Bag(element) = &collection.value_type else {
                        return Err(unsupported(
                            &self.source,
                            collection.span,
                            "bag-multiplicity collection kind",
                        ));
                    };
                    require_same_type(&self.source, query.span, element, &query.value_type)?;
                    (
                        CompilerExpressionKind::BagMultiplicity {
                            bag: Box::new(collection),
                            value: Box::new(query),
                        },
                        CompilerType::Int,
                    )
                }
                _ => unreachable!("collection query spelling selected above"),
            };
            return Ok(CompilerExpression {
                kind,
                value_type,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(constructor), value] = items
            && self.source.slice(*constructor) == "Some"
        {
            let value = self.analyze_expression(value, environment)?;
            require_optional_payload(&self.source, value.span, &value.value_type)?;
            return Ok(CompilerExpression {
                value_type: CompilerType::Optional(Box::new(value.value_type.clone())),
                kind: CompilerExpressionKind::OptionalSome(Box::new(value)),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(operation), operand] = items
            && matches!(
                self.source.slice(*operation),
                "upper" | "lower" | "case-fold"
            )
        {
            let operation = self.source.slice(*operation).to_owned();
            return self.analyze_static_unicode_transform(&operation, operand, span, environment);
        }
        if let [text, Expression::Identifier(operation), index] = items
            && self.source.slice(*operation) == "character-at"
        {
            return self.analyze_static_character_at(text, index, span, environment);
        }
        if let [
            base,
            Expression::Identifier(operation),
            Expression::Product { fields, .. },
        ] = items
            && self.source.slice(*operation) == "with"
            && !fields.is_empty()
            && fields.iter().all(|field| field.label.is_some())
        {
            return self.analyze_record_reconstruction(base, fields, span, environment);
        }
        if let [
            text,
            Expression::Identifier(operation),
            Expression::Identifier(form),
        ] = items
            && self.source.slice(*operation) == "normalize"
        {
            let form = self.source.slice(*form).to_owned();
            return self.analyze_static_normalization(text, &form, span, environment);
        }
        if let [left, Expression::Identifier(operation), right] = items
            && self.source.slice(*operation) == "canonically-equals"
        {
            return self.analyze_static_canonical_equality(left, right, span, environment);
        }
        if let [
            Expression::Identifier(constructor),
            Expression::Identifier(payload),
        ] = items
            && self.source.slice(*constructor) == "None"
        {
            let payload = self.parse_classifier(*payload)?;
            return self.finish_optional_none(payload, span);
        }
        if let [
            Expression::Identifier(constructor),
            Expression::Product { fields, .. },
        ] = items
            && self.source.slice(*constructor) == "None"
            && fields.iter().all(|field| field.label.is_none())
        {
            let mut payload = Vec::with_capacity(fields.len());
            for field in fields {
                let Expression::Identifier(classifier) = &field.value else {
                    return Err(unsupported(
                        &self.source,
                        field.value.span(),
                        "Optional product payload classifier",
                    ));
                };
                payload.push(self.parse_classifier(*classifier)?);
            }
            return self.finish_optional_none(CompilerType::Tuple(payload), span);
        }
        if let [
            value,
            Expression::Identifier(operation),
            Expression::Identifier(encoding),
        ] = items
            && self.source.slice(*operation) == "byte-count"
        {
            if self.source.slice(*encoding) != "Utf8" {
                return Err(source_diagnostic(
                    &self.source,
                    "E-NO-APPLICABLE-OVERLOAD",
                    *encoding,
                    "the compiler String byte-count operation requires Utf8",
                ));
            }
            let value = self.analyze_expression(value, environment)?;
            require_type(
                &self.source,
                value.span,
                &CompilerType::String,
                &value.value_type,
            )?;
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::StringUtf8ByteCount(Box::new(value)),
                value_type: CompilerType::Int,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [
            Expression::Identifier(namespace),
            Expression::Identifier(vocabulary),
            Expression::Identifier(code),
        ] = items
            && let Some(value) = arithmetic_error_code(
                self.source.slice(*namespace),
                self.source.slice(*vocabulary),
                self.source.slice(*code),
            )
        {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ErrorCode(value),
                value_type: CompilerType::ErrorCode,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [
            Expression::Identifier(namespace),
            Expression::Identifier(vocabulary),
            Expression::Identifier(code),
        ] = items
            && let Some((enumeration, value)) = generator_error_code(
                self.source.slice(*namespace),
                self.source.slice(*vocabulary),
                self.source.slice(*code),
            )
        {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::Enum(value),
                value_type: CompilerType::Enum(enumeration),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [
            Expression::Identifier(operation),
            Expression::Identifier(domain),
        ] = items
            && matches!(self.source.slice(*operation), "zero" | "one")
        {
            let one = self.source.slice(*operation) == "one";
            return match self.source.slice(*domain) {
                "Int" | "Nat" => {
                    let value = BigInt::from(u8::from(one));
                    let value_type = if self.source.slice(*domain) == "Nat" {
                        CompilerType::Nat
                    } else {
                        CompilerType::Int
                    };
                    Ok(CompilerExpression {
                        kind: CompilerExpressionKind::Int(value.clone()),
                        value_type,
                        int_range: Some(IntRange::exact(value)),
                        rational_value: None,
                        span,
                    })
                }
                "Rational" => {
                    let value = BigRational::from_integer(BigInt::from(u8::from(one)));
                    Ok(CompilerExpression {
                        kind: CompilerExpressionKind::Rational(value.clone()),
                        value_type: CompilerType::Rational,
                        int_range: None,
                        rational_value: Some(value),
                        span,
                    })
                }
                _ => Err(unsupported(
                    &self.source,
                    *domain,
                    "numeric identity domain",
                )),
            };
        }
        if let [Expression::Identifier(constructor), argument] = items {
            match self.source.slice(*constructor) {
                "Int" => return self.analyze_int_constructor(argument, span, environment),
                "Nat" => return self.analyze_nat_constructor(argument, span, environment),
                "Character" => {
                    let value = self.analyze_expression(argument, environment)?;
                    return self.finish_character_conversion(value, argument.span());
                }
                "String" => {
                    let value = self.analyze_expression(argument, environment)?;
                    let value = self.finish_character_conversion(value, argument.span())?;
                    return Ok(forget_character_evidence(value));
                }
                _ => {}
            }
        }
        if let [Expression::Identifier(constructor), argument] = items
            && self.source.slice(*constructor) == "Rational"
        {
            return self.analyze_rational_constructor(argument, span, environment);
        }
        if let [Expression::Identifier(operation), operand] = items
            && matches!(
                self.source.slice(*operation),
                "empty?"
                    | "range-lower"
                    | "range-upper"
                    | "range-lower-inclusive?"
                    | "range-upper-inclusive?"
            )
        {
            let operation = self.source.slice(*operation).to_owned();
            return self.analyze_range_observation(&operation, operand, span, environment);
        }
        if let [record, Expression::Identifier(field)] = items
            && self.record_selection_candidate(record, environment)
        {
            return self.analyze_record_field(record, *field, span, environment);
        }
        if let [error, Expression::Identifier(field)] = items
            && matches!(
                self.source.slice(*field),
                "code" | "domain" | "detail" | "cause" | "source"
            )
            && !matches!(error, Expression::Identifier(name)
                if !environment.contains_key(self.source.slice(*name)))
        {
            return self.analyze_error_field(error, *field, span, environment);
        }
        if let [
            Expression::Callable {
                kind: CallableKind::Minus,
                ..
            },
            operand,
        ] = items
        {
            let operand = self.analyze_expression(operand, environment)?;
            if let CompilerType::Modular(modular) = operand.value_type.clone() {
                let int_range = exact_int(&operand)
                    .map(|value| IntRange::exact(reduce_modular(-value, &modular)))
                    .or_else(|| Some(modular_range(&modular)));
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Negate(Box::new(operand)),
                    value_type: CompilerType::Modular(modular),
                    int_range,
                    rational_value: None,
                    span,
                });
            }
            require_exact_numeric(&self.source, operand.span, &operand.value_type)?;
            let range = (operand.value_type == CompilerType::Int)
                .then_some(operand.int_range.as_ref())
                .flatten()
                .map(|range| IntRange {
                    lower: -range.upper.clone(),
                    upper: -range.lower.clone(),
                });
            let rational_value = operand.rational_value.as_ref().map(|value| -value.clone());
            let value_type = if operand.value_type == CompilerType::InfiniteNat {
                CompilerType::InfiniteInt
            } else {
                operand.value_type.clone()
            };
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::Negate(Box::new(operand)),
                value_type,
                int_range: range,
                rational_value,
                span,
            });
        }
        if let [Expression::Identifier(operation), operand] = items
            && matches!(self.source.slice(*operation), "negate" | "absolute")
        {
            let negate = self.source.slice(*operation) == "negate";
            let operand = self.analyze_expression(operand, environment)?;
            if let CompilerType::Modular(modular) = operand.value_type.clone() {
                if !negate {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "absolute value over a modular value",
                    ));
                }
                let int_range = exact_int(&operand)
                    .map(|value| IntRange::exact(reduce_modular(-value, &modular)))
                    .or_else(|| Some(modular_range(&modular)));
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Negate(Box::new(operand)),
                    value_type: CompilerType::Modular(modular),
                    int_range,
                    rational_value: None,
                    span,
                });
            }
            require_exact_numeric(&self.source, operand.span, &operand.value_type)?;
            let range = (operand.value_type == CompilerType::Int)
                .then_some(operand.int_range.as_ref())
                .flatten()
                .map(|range| {
                    if negate {
                        IntRange {
                            lower: -range.upper.clone(),
                            upper: -range.lower.clone(),
                        }
                    } else {
                        absolute_range(range)
                    }
                });
            let rational_value = operand.rational_value.as_ref().map(|value| {
                if negate {
                    -value.clone()
                } else {
                    rational_absolute(value)
                }
            });
            let value_type = if negate && operand.value_type == CompilerType::InfiniteNat {
                CompilerType::InfiniteInt
            } else {
                operand.value_type.clone()
            };
            return Ok(CompilerExpression {
                kind: if negate {
                    CompilerExpressionKind::Negate(Box::new(operand))
                } else {
                    CompilerExpressionKind::Absolute(Box::new(operand))
                },
                value_type,
                int_range: range,
                rational_value,
                span,
            });
        }
        if let [Expression::Identifier(operation), operand] = items
            && self.source.slice(*operation) == "not"
        {
            let operand = self.analyze_expression(operand, environment)?;
            require_type(
                &self.source,
                operand.span,
                &CompilerType::Boolean,
                &operand.value_type,
            )?;
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::Not(Box::new(operand)),
                value_type: CompilerType::Boolean,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if items.len() >= 5
            && items.len() % 2 == 1
            && items
                .iter()
                .skip(1)
                .step_by(2)
                .all(|item| matches!(item, Expression::Callable { .. }))
        {
            let mut grouped = Expression::Application {
                items: items[..3].to_vec(),
                span: Span::new(items[0].span().start, items[2].span().end),
            };
            for pair in items[3..].chunks_exact(2) {
                grouped = Expression::Application {
                    items: vec![grouped, pair[0].clone(), pair[1].clone()],
                    span: Span::new(items[0].span().start, pair[1].span().end),
                };
            }
            return self
                .analyze_expression(&grouped, environment)
                .map(|mut value| {
                    value.span = span;
                    value
                });
        }
        if items.len() >= 5
            && items.len() % 2 == 1
            && items.iter().skip(1).step_by(2).all(|item| {
                matches!(item, Expression::Identifier(operation)
                    if matches!(self.source.slice(*operation), "and" | "or" | "xor" | "in" | "contains"))
            })
        {
            let mut grouped = Expression::Application {
                items: items[..3].to_vec(),
                span: Span::new(items[0].span().start, items[2].span().end),
            };
            for pair in items[3..].chunks_exact(2) {
                grouped = Expression::Application {
                    items: vec![grouped, pair[0].clone(), pair[1].clone()],
                    span: Span::new(items[0].span().start, pair[1].span().end),
                };
            }
            return self.analyze_expression(&grouped, environment).map(|mut value| {
                value.span = span;
                value
            });
        }
        if let [
            record,
            Expression::Identifier(field),
            Expression::Callable { kind, .. },
            right,
        ] = items
            && self.record_selection_candidate(record, environment)
        {
            let left = Expression::Application {
                items: vec![record.clone(), Expression::Identifier(*field)],
                span: Span::new(record.span().start, field.end),
            };
            return self.analyze_symbolic_binary(*kind, &left, right, span, environment);
        }
        if let [left, Expression::Callable { kind, .. }, right] = items {
            return self.analyze_symbolic_binary(*kind, left, right, span, environment);
        }
        if let [left, Expression::Identifier(operation), right] = items
            && matches!(
                self.source.slice(*operation),
                "and" | "or" | "xor" | "in" | "contains"
            )
        {
            let operation = self.source.slice(*operation).to_owned();
            return self.analyze_identifier_binary(&operation, left, right, span, environment);
        }
        if let Some(value) = self.analyze_function_value_chain(items, span, environment)? {
            return Ok(value);
        }
        self.analyze_call(items, span, environment)
    }
}
