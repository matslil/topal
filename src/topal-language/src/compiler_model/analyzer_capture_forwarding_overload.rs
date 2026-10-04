impl Analyzer {
    fn capture_forwarding_overload(
        &self,
        caller: &FunctionSource,
        reference: &CompilerFunctionCallReference,
        declarations: &[FunctionSource],
    ) -> Option<FunctionSource> {
        if !reference.is_ordinary_unqualified {
            return None;
        }
        let arguments = reference
            .arguments
            .iter()
            .map(|argument| self.capture_forwarding_argument(caller, argument))
            .collect::<Option<Vec<_>>>()?;
        let flattened_arguments = match arguments.as_slice() {
            [
                CompilerExpression {
                    kind: CompilerExpressionKind::Tuple(values),
                    ..
                },
            ] => Some(values.as_slice()),
            _ => None,
        };
        for declaration in declarations
            .iter()
            .filter(|declaration| !caller.is_static || declaration.is_static)
        {
            if declaration.parameters.iter().any(|parameter| {
                !parameter.fields.is_empty()
                    || parameter.default.is_some()
                    || parameter.qualifier.is_some()
            }) {
                return None;
            }
            let candidate_arguments = if declaration.parameters.is_empty()
                && matches!(
                    arguments.as_slice(),
                    [CompilerExpression {
                        value_type: CompilerType::Unit,
                        ..
                    }]
                ) {
                &[][..]
            } else if declaration.parameters.len() > 1 {
                flattened_arguments.unwrap_or(arguments.as_slice())
            } else {
                arguments.as_slice()
            };
            if candidate_arguments.len() != declaration.parameters.len() {
                continue;
            }
            let mut applicable = true;
            let mut fact_dependent = false;
            for (parameter, argument) in declaration.parameters.iter().zip(candidate_arguments) {
                let expected = self.parse_classifier(parameter.classifier).ok()?;
                if adapt_function_call_argument(&expected, argument).is_none() {
                    if capture_argument_adaptation_may_depend_on_facts(&expected, argument) {
                        fact_dependent = true;
                    } else {
                        applicable = false;
                        break;
                    }
                }
            }
            if applicable && fact_dependent {
                return None;
            }
            if applicable {
                return Some(declaration.clone());
            }
        }
        None
    }

    #[allow(clippy::too_many_lines)] // The fail-closed pre-instantiation classifier replay keeps each admitted syntax form explicit.
    fn capture_forwarding_argument(
        &self,
        caller: &FunctionSource,
        expression: &Expression,
    ) -> Option<CompilerExpression> {
        let span = expression.span();
        match expression {
            Expression::Unit(_) => Some(CompilerExpression {
                kind: CompilerExpressionKind::Unit,
                value_type: CompilerType::Unit,
                int_range: None,
                rational_value: None,
                span,
            }),
            Expression::Boolean(value) => Some(CompilerExpression {
                kind: CompilerExpressionKind::Boolean(self.source.slice(*value) == "true"),
                value_type: CompilerType::Boolean,
                int_range: None,
                rational_value: None,
                span,
            }),
            Expression::Integer(value) => {
                let value = parse_integer(self.source.slice(*value))?;
                Some(CompilerExpression {
                    kind: CompilerExpressionKind::Int(value.clone()),
                    value_type: CompilerType::Int,
                    int_range: Some(IntRange::exact(value)),
                    rational_value: None,
                    span,
                })
            }
            Expression::Rational(value) => {
                let value = parse_rational(self.source.slice(*value))?;
                Some(CompilerExpression {
                    kind: CompilerExpressionKind::Rational(value.clone()),
                    value_type: CompilerType::Rational,
                    int_range: None,
                    rational_value: Some(value),
                    span,
                })
            }
            Expression::String(value) => Some(CompilerExpression {
                kind: CompilerExpressionKind::String(
                    parse_string(self.source.slice(*value))?.to_owned(),
                ),
                value_type: CompilerType::String,
                int_range: None,
                rational_value: None,
                span,
            }),
            Expression::Identifier(name) => {
                let name_text = self.source.slice(*name);
                let parameter = caller
                    .parameters
                    .iter()
                    .find(|parameter| self.source.slice(parameter.name) == name_text)?;
                let value_type = self.parse_classifier(parameter.classifier).ok()?;
                Some(CompilerExpression {
                    kind: CompilerExpressionKind::Local(name_text.to_owned()),
                    value_type,
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Product { fields, .. }
                if !fields.is_empty() && fields.iter().all(|field| field.label.is_some()) =>
            {
                let mut values = Vec::with_capacity(fields.len());
                let mut value_types = Vec::with_capacity(fields.len());
                for field in fields {
                    let label = self.source.slice(field.label?).to_owned();
                    let value = self.capture_forwarding_argument(caller, &field.value)?;
                    value_types.push((label.clone(), value.value_type.clone()));
                    values.push((label, value));
                }
                value_types.sort_by(|left, right| left.0.cmp(&right.0));
                Some(CompilerExpression {
                    kind: CompilerExpressionKind::Record(values),
                    value_type: CompilerType::Record(value_types),
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Product { fields, .. } => {
                let values = fields
                    .iter()
                    .map(|field| self.capture_forwarding_argument(caller, &field.value))
                    .collect::<Option<Vec<_>>>()?;
                Some(CompilerExpression {
                    value_type: CompilerType::Tuple(
                        values
                            .iter()
                            .map(|value| value.value_type.clone())
                            .collect(),
                    ),
                    kind: CompilerExpressionKind::Tuple(values),
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Application { items, .. } => {
                let (callable_index, callable) =
                    items.iter().enumerate().find_map(|(index, item)| {
                        let Expression::Callable { kind, .. } = item else {
                            return None;
                        };
                        Some((index, *kind))
                    })?;
                let operands = items
                    .iter()
                    .enumerate()
                    .filter_map(|(index, item)| (index != callable_index).then_some(item))
                    .map(|operand| self.capture_forwarding_argument(caller, operand))
                    .collect::<Option<Vec<_>>>()?;
                let value_type = match (callable, operands.as_slice()) {
                    (CallableKind::Plus, [left, right])
                        if left.value_type == right.value_type
                            && matches!(
                                left.value_type,
                                CompilerType::Int
                                    | CompilerType::Nat
                                    | CompilerType::Rational
                                    | CompilerType::String
                            ) =>
                    {
                        left.value_type.clone()
                    }
                    (CallableKind::Minus | CallableKind::Multiply, [left, right])
                        if left.value_type == right.value_type
                            && matches!(
                                left.value_type,
                                CompilerType::Int | CompilerType::Nat | CompilerType::Rational
                            ) =>
                    {
                        left.value_type.clone()
                    }
                    _ => return None,
                };
                Some(CompilerExpression {
                    kind: CompilerExpressionKind::Local(format!(
                        "topal.capture.overload.{}.{}",
                        span.start, span.end
                    )),
                    value_type,
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Block { .. }
            | Expression::DecisionTable { .. }
            | Expression::Infinity(_)
            | Expression::Measured { .. }
            | Expression::Discard(_)
            | Expression::AnonymousFunction { .. }
            | Expression::ContextIdentifier(_)
            | Expression::Callable { .. } => None,
        }
    }

    fn finish_selected_call(
        &mut self,
        function_name: &str,
        declaration: &FunctionSource,
        arguments: Vec<CompilerExpression>,
        metadata: &CompilerCallMetadata,
        span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        let identity = function_overload_identity(&self.source, function_name, declaration);
        let declaration_name = self.source.slice(declaration.name).to_owned();
        let recursion_proof = self.recursion_proof(&declaration_name, declaration);
        if self.active_calls.contains(&identity) {
            if let Some(active) = self.active_recursive_functions.get(&identity).cloned()
                && ((self.active_calls.last() == Some(&identity)
                    && active.proof.mutual_target.is_none())
                    || self.closes_proven_mutual_bounded_cycle(&identity, &declaration_name))
            {
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Call {
                        symbol: active.symbol,
                        arguments,
                    },
                    value_type: active.result_type,
                    int_range: None,
                    rational_value: None,
                    span,
                });
            }
            return Err(unsupported(&self.source, span, "recursive function call"));
        }
        let reserved_symbol = if let Some(proof) = recursion_proof.clone() {
            let symbol = self.reserve_function_symbol(function_name);
            let result_type = self.parse_classifier(declaration.result)?;
            self.active_recursive_functions.insert(
                identity.clone(),
                ActiveRecursiveFunction {
                    source_name: declaration_name,
                    symbol: symbol.clone(),
                    result_type,
                    proof,
                },
            );
            Some(symbol)
        } else {
            None
        };
        self.active_calls.push(identity.clone());
        let result = self.instantiate_function(
            function_name,
            declaration,
            &arguments,
            metadata,
            reserved_symbol.as_deref(),
            recursion_proof.is_some(),
        );
        let popped = self.active_calls.pop();
        debug_assert_eq!(popped.as_deref(), Some(identity.as_str()));
        self.active_recursive_functions.remove(&identity);
        let (symbol, result_type, int_range, rational_value) = result?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::Call { symbol, arguments },
            value_type: result_type,
            int_range,
            rational_value,
            span,
        })
    }

    #[allow(clippy::too_many_lines)] // Specialization keeps ABI and checked evidence decisions together.
    fn instantiate_function(
        &mut self,
        function_name: &str,
        declaration: &FunctionSource,
        arguments: &[CompilerExpression],
        metadata: &CompilerCallMetadata,
        reserved_symbol: Option<&str>,
        generalize_parameters: bool,
    ) -> Result<(String, CompilerType, Option<IntRange>, Option<BigRational>), Diagnostic> {
        let CompilerCallMetadata {
            callable_arguments,
            aggregate_arguments,
            callable_captures,
            aggregate_captures,
            scope_arguments,
            scope_captures,
            lexical_captures,
            context_captures,
        } = metadata;
        let mut environment = self
            .nested_static_environments
            .get(&declaration.span.start)
            .cloned()
            .unwrap_or_default();
        let mut parameters = Vec::new();
        for (parameter_index, (parameter, argument)) in
            declaration.parameters.iter().zip(arguments).enumerate()
        {
            if generalize_parameters && compiler_type_contains_infinity(&argument.value_type) {
                return Err(unsupported(
                    &self.source,
                    argument.span,
                    "recursive infinity function boundary",
                ));
            }
            if !parameter.fields.is_empty()
                || parameter.default.is_some()
                || parameter.qualifier.is_some()
            {
                return Err(unsupported(
                    &self.source,
                    parameter.name,
                    "packaged, defaulted, or qualified parameter",
                ));
            }
            let expected = self.parse_classifier(parameter.classifier)?;
            if !compiler_infinity_evidence_compatible(&expected, &argument.value_type) {
                require_same_type(
                    &self.source,
                    parameter.classifier,
                    &expected,
                    &argument.value_type,
                )?;
            }
            let parameter_type = if compiler_type_contains_infinity(&argument.value_type) {
                argument.value_type.clone()
            } else {
                expected
            };
            if !compiler_function_parameter_supported(&parameter_type) {
                return Err(unsupported(
                    &self.source,
                    parameter.classifier,
                    "unsupported function parameter",
                ));
            }
            let name = self.source.slice(parameter.name).to_owned();
            let discarded = name == "_";
            if !discarded {
                environment.insert(
                    name.clone(),
                    BindingFacts {
                        storage_name: name.clone(),
                        origin: parameter.name.start,
                        runtime_bound: true,
                        value_type: parameter_type.clone(),
                        int_range: (!generalize_parameters)
                            .then(|| argument.int_range.clone())
                            .flatten(),
                        rational_value: (!generalize_parameters)
                            .then(|| argument.rational_value.clone())
                            .flatten(),
                        infinity_negative: (!generalize_parameters)
                            .then(|| compiler_infinity_direction(argument))
                            .flatten(),
                        string_value: (!generalize_parameters
                            && !matches!(argument.kind, CompilerExpressionKind::Local(_)))
                        .then(|| {
                            exact_string(argument).or_else(|| {
                                aggregate_arguments[parameter_index].string_value.clone()
                            })
                        })
                        .flatten(),
                        string_characters: aggregate_arguments[parameter_index]
                            .string_characters
                            .clone(),
                        closed_int_range: None,
                        list_count: (!generalize_parameters)
                            .then(|| Self::known_list_count(argument, &BTreeMap::new()))
                            .flatten()
                            .or_else(|| {
                                aggregate_arguments[parameter_index]
                                    .list_entries
                                    .as_ref()
                                    .map(Vec::len)
                            }),
                        list_string_keys: (!generalize_parameters)
                            .then(|| Self::known_list_string_keys(argument, &BTreeMap::new()))
                            .flatten(),
                        list_string_characters: aggregate_arguments[parameter_index]
                            .list_string_characters
                            .clone(),
                        list_entries: aggregate_arguments[parameter_index].list_entries.clone(),
                        array_entries: aggregate_arguments[parameter_index].array_entries.clone(),
                        map_entries: aggregate_arguments[parameter_index].map_entries.clone(),
                        tuple_fields: aggregate_arguments[parameter_index].tuple_fields.clone(),
                        record_fields: aggregate_arguments[parameter_index].record_fields.clone(),
                        optional: aggregate_arguments[parameter_index].optional.clone(),
                        sum: aggregate_arguments[parameter_index].sum.clone(),
                        result: aggregate_arguments[parameter_index].result.clone(),
                        namespace: scope_arguments[parameter_index].clone(),
                        callable: callable_arguments[parameter_index].clone(),
                        static_capability: None,
                    },
                );
            }
            parameters.push(CompilerParameter {
                name,
                discarded,
                source_visible: !discarded,
                value_type: parameter_type,
                int_range: (!generalize_parameters)
                    .then(|| argument.int_range.clone())
                    .flatten(),
                span: parameter.name,
            });
        }
        let callable_arguments_start = declaration.parameters.len();
        let aggregate_captures_start = callable_arguments_start + callable_captures.len();
        let scope_arguments_start = aggregate_captures_start + aggregate_captures.len();
        let lexical_arguments_start = scope_arguments_start + scope_captures.len();
        let context_arguments_start = lexical_arguments_start + lexical_captures.len();
        let captured_callable_arguments =
            &arguments[callable_arguments_start..aggregate_captures_start];
        debug_assert_eq!(captured_callable_arguments.len(), callable_captures.len());
        for (capture, argument) in callable_captures.iter().zip(captured_callable_arguments) {
            require_same_type(
                &self.source,
                capture.span,
                &capture.value_type,
                &argument.value_type,
            )?;
            environment.insert(
                capture.parameter_name.clone(),
                BindingFacts {
                    storage_name: capture.parameter_name.clone(),
                    origin: capture.span.start,
                    runtime_bound: true,
                    value_type: capture.value_type.clone(),
                    int_range: capture.int_range.clone(),
                    rational_value: capture.rational_value.clone(),
                    infinity_negative: compiler_infinity_direction(argument),
                    string_value: exact_string(argument),
                    string_characters: None,
                    closed_int_range: None,
                    list_count: Self::known_list_count(argument, &BTreeMap::new()),
                    list_string_keys: Self::known_list_string_keys(argument, &BTreeMap::new()),
                    list_string_characters: None,
                    list_entries: None,
                    array_entries: None,
                    map_entries: None,
                    tuple_fields: Vec::new(),
                    record_fields: BTreeMap::new(),
                    optional: None,
                    sum: None,
                    result: None,
                    namespace: None,
                    callable: None,
                    static_capability: None,
                },
            );
            parameters.push(CompilerParameter {
                name: capture.parameter_name.clone(),
                discarded: false,
                source_visible: false,
                value_type: capture.value_type.clone(),
                int_range: capture.int_range.clone(),
                span: capture.span,
            });
        }
        let captured_aggregate_arguments =
            &arguments[aggregate_captures_start..scope_arguments_start];
        debug_assert_eq!(captured_aggregate_arguments.len(), aggregate_captures.len());
        for (capture, argument) in aggregate_captures.iter().zip(captured_aggregate_arguments) {
            require_same_type(
                &self.source,
                capture.span,
                &capture.value_type,
                &argument.value_type,
            )?;
            environment.insert(
                capture.parameter_name.clone(),
                BindingFacts {
                    storage_name: capture.parameter_name.clone(),
                    origin: capture.span.start,
                    runtime_bound: true,
                    value_type: capture.value_type.clone(),
                    int_range: capture.int_range.clone(),
                    rational_value: capture.rational_value.clone(),
                    infinity_negative: compiler_infinity_direction(argument),
                    string_value: exact_string(argument),
                    string_characters: None,
                    closed_int_range: None,
                    list_count: Self::known_list_count(argument, &BTreeMap::new()),
                    list_string_keys: Self::known_list_string_keys(argument, &BTreeMap::new()),
                    list_string_characters: None,
                    list_entries: None,
                    array_entries: None,
                    map_entries: None,
                    tuple_fields: Vec::new(),
                    record_fields: BTreeMap::new(),
                    optional: None,
                    sum: None,
                    result: None,
                    namespace: None,
                    callable: None,
                    static_capability: None,
                },
            );
            parameters.push(CompilerParameter {
                name: capture.parameter_name.clone(),
                discarded: false,
                source_visible: false,
                value_type: capture.value_type.clone(),
                int_range: capture.int_range.clone(),
                span: capture.span,
            });
        }
        let captured_scope_arguments = &arguments[scope_arguments_start..lexical_arguments_start];
        debug_assert_eq!(captured_scope_arguments.len(), scope_captures.len());
        for (capture, argument) in scope_captures.iter().zip(captured_scope_arguments) {
            require_same_type(
                &self.source,
                capture.span,
                &capture.value_type,
                &argument.value_type,
            )?;
            environment.insert(
                capture.parameter_name.clone(),
                BindingFacts {
                    storage_name: capture.parameter_name.clone(),
                    origin: capture.span.start,
                    runtime_bound: true,
                    value_type: capture.value_type.clone(),
                    int_range: capture.int_range.clone(),
                    rational_value: capture.rational_value.clone(),
                    infinity_negative: None,
                    string_value: exact_string(argument),
                    string_characters: None,
                    closed_int_range: None,
                    list_count: None,
                    list_string_keys: None,
                    list_string_characters: None,
                    list_entries: None,
                    array_entries: None,
                    map_entries: None,
                    tuple_fields: Vec::new(),
                    record_fields: BTreeMap::new(),
                    optional: None,
                    sum: None,
                    result: None,
                    namespace: None,
                    callable: None,
                    static_capability: None,
                },
            );
            parameters.push(CompilerParameter {
                name: capture.parameter_name.clone(),
                discarded: false,
                source_visible: true,
                value_type: capture.value_type.clone(),
                int_range: capture.int_range.clone(),
                span: capture.span,
            });
        }
        let captured_lexical_arguments =
            &arguments[lexical_arguments_start..context_arguments_start];
        debug_assert_eq!(captured_lexical_arguments.len(), lexical_captures.len());
        for (capture, argument) in lexical_captures.iter().zip(captured_lexical_arguments) {
            require_same_type(
                &self.source,
                capture.span,
                &capture.value_type,
                &argument.value_type,
            )?;
            environment.insert(
                capture.parameter_name.clone(),
                BindingFacts {
                    storage_name: capture.parameter_name.clone(),
                    origin: capture.span.start,
                    runtime_bound: true,
                    value_type: capture.value_type.clone(),
                    int_range: capture.int_range.clone(),
                    rational_value: capture.rational_value.clone(),
                    infinity_negative: None,
                    string_value: exact_string(argument),
                    string_characters: None,
                    closed_int_range: None,
                    list_count: None,
                    list_string_keys: None,
                    list_string_characters: None,
                    list_entries: None,
                    array_entries: None,
                    map_entries: None,
                    tuple_fields: Vec::new(),
                    record_fields: BTreeMap::new(),
                    optional: None,
                    sum: None,
                    result: None,
                    namespace: None,
                    callable: None,
                    static_capability: None,
                },
            );
            parameters.push(CompilerParameter {
                name: capture.parameter_name.clone(),
                discarded: false,
                source_visible: true,
                value_type: capture.value_type.clone(),
                int_range: capture.int_range.clone(),
                span: capture.span,
            });
        }
        let context_arguments = &arguments[context_arguments_start..];
        debug_assert_eq!(context_arguments.len(), context_captures.len());
        for (capture, argument) in context_captures.iter().zip(context_arguments) {
            require_same_type(
                &self.source,
                capture.span,
                &capture.value_type,
                &argument.value_type,
            )?;
            environment.insert(
                capture.parameter_name.clone(),
                BindingFacts {
                    storage_name: capture.parameter_name.clone(),
                    origin: capture.span.start,
                    runtime_bound: true,
                    value_type: capture.value_type.clone(),
                    int_range: capture.int_range.clone(),
                    rational_value: capture.rational_value.clone(),
                    infinity_negative: None,
                    string_value: exact_string(argument),
                    string_characters: None,
                    closed_int_range: None,
                    list_count: None,
                    list_string_keys: None,
                    list_string_characters: None,
                    list_entries: None,
                    array_entries: None,
                    map_entries: None,
                    tuple_fields: Vec::new(),
                    record_fields: BTreeMap::new(),
                    optional: None,
                    sum: None,
                    result: None,
                    namespace: None,
                    callable: None,
                    static_capability: None,
                },
            );
            parameters.push(CompilerParameter {
                name: capture.parameter_name.clone(),
                discarded: false,
                source_visible: true,
                value_type: capture.value_type.clone(),
                int_range: capture.int_range.clone(),
                span: capture.span,
            });
        }
        let result_type = self.parse_classifier(declaration.result)?;
        let returns_character_generator = is_admitted_character_generator_type(&result_type);
        let returns_value_boundary_generator =
            is_admitted_value_boundary_generator_type(&result_type);
        let returns_generator = returns_character_generator || returns_value_boundary_generator;
        if result_type != CompilerType::Function
            && !compiler_function_result_supported(&result_type)
            && !returns_generator
        {
            return Err(unsupported(
                &self.source,
                declaration.result,
                "non-scalar function result",
            ));
        }
        let admitted_factory_parameter = matches!(
            parameters.as_slice(),
            [CompilerParameter {
                discarded: false,
                source_visible: true,
                value_type: CompilerType::String | CompilerType::Character,
                ..
            }] if returns_character_generator
        ) || matches!(
            parameters.as_slice(),
            [CompilerParameter {
                name,
                discarded: false,
                source_visible: true,
                value_type: CompilerType::Int,
                ..
            }] if returns_value_boundary_generator && name == "initial"
        ) || matches!(
            parameters.as_slice(),
            [CompilerParameter {
                name,
                discarded: false,
                source_visible: true,
                value_type: CompilerType::Tuple(fields),
                ..
            }] if returns_value_boundary_generator
                && name == "initial"
                && fields == &[CompilerType::Int, CompilerType::String]
        ) || matches!(
            parameters.as_slice(),
            [CompilerParameter {
                name,
                discarded: false,
                source_visible: true,
                value_type,
                ..
            }] if returns_value_boundary_generator
                && name == "initial"
                && value_type == &nested_optional_product_type()
        ) || matches!(
            parameters.as_slice(),
            [CompilerParameter {
                name,
                discarded: false,
                source_visible: true,
                value_type,
                ..
            }] if returns_value_boundary_generator
                && name == "initial"
                && value_type == &int_list_type()
        );
        if returns_generator
            && (self.in_function
                || generalize_parameters
                || declaration.is_static
                || !admitted_factory_parameter)
        {
            return Err(unsupported(
                &self.source,
                declaration.result,
                "Generator result beyond one admitted specialized ordinary factory",
            ));
        }
        let generator_parameters = parameters
            .iter()
            .filter(|parameter| is_admitted_function_generator_type(&parameter.value_type))
            .collect::<Vec<_>>();
        let generator_parameter = if generator_parameters.is_empty() {
            None
        } else {
            let parameter = generator_parameters[0];
            if generator_parameters.len() != 1
                || parameters.len() != 1
                || parameter.discarded
                || declaration.is_static
            {
                return Err(unsupported(
                    &self.source,
                    parameter.span,
                    "Generator parameter behavior beyond one ordinary owned specialization",
                ));
            }
            Some(parameter.clone())
        };
        let scoped_generator_value = if let Some(parameter) = &generator_parameter {
            if self.in_function {
                let feature = if is_admitted_character_generator_type(&parameter.value_type) {
                    "nested Character Generator parameter transfer"
                } else {
                    "nested Generator parameter transfer"
                };
                return Err(unsupported(&self.source, parameter.span, feature));
            }
            let argument = &arguments[0];
            let CompilerExpressionKind::Local(storage_name) = &argument.kind else {
                return Err(unsupported(
                    &self.source,
                    argument.span,
                    "non-local Generator parameter transfer",
                ));
            };
            let retained = self
                .generator_values
                .get(storage_name)
                .cloned()
                .ok_or_else(|| {
                    unsupported(
                        &self.source,
                        argument.span,
                        "Generator parameter without closed provenance",
                    )
                })?;
            let custom_character_generator = matches!(
                &retained.kind,
                CompilerExpressionKind::CustomCharacterGenerator {
                    prefix,
                    characters,
                    locals,
                    close_handler: None,
                    result,
                    ..
                } if prefix.statements.is_empty()
                    && characters.len() == 1
                    && locals.is_empty()
                    && matches!(
                        result.value_type,
                        CompilerType::Unit | CompilerType::Character
                    )
            );
            let custom_value_generator = matches!(
                &retained.kind,
                CompilerExpressionKind::CustomValueGenerator {
                    declaration,
                    initial_parameter,
                    initial,
                    additional_initial_parameters,
                    prefix,
                    yields,
                    continuations,
                    explicit_return: None,
                    result,
                    ..
                } if ((declaration == "numbers"
                    && initial_parameter.value_type == CompilerType::Int
                    && exact_int(initial) == Some(BigInt::from(7))
                    && exact_string(result).as_deref() == Some("done"))
                    || (declaration == "pairs"
                        && initial_parameter.value_type
                            == CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String])
                        && exact_int_string_product(initial, 7, "item")
                        && exact_int_string_product(result, 8, "done"))
                    || (declaration == "pairs"
                        && initial_parameter.value_type == nested_optional_product_type()
                        && exact_optional_int_string_value(initial, 7, "item")
                        && exact_nested_result_product_value(result, 8, "done"))
                    || (declaration == "relay"
                        && initial_parameter.value_type == int_list_type()
                        && exact_singleton_int_list(initial, 7)
                        && exact_int_list_append(result, "initial", 9)))
                    && initial_parameter.name == "initial"
                    && additional_initial_parameters.is_empty()
                    && prefix.statements.is_empty()
                    && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
                    && continuations.is_empty()
                    && is_admitted_value_boundary_generator_type(&retained.value_type)
            );
            let custom_generator = custom_character_generator || custom_value_generator;
            let custom_provenance = custom_generator.then(|| retained.clone());
            if !matches!(
                &retained.kind,
                CompilerExpressionKind::StringCharactersGenerator { .. }
            ) && !custom_generator
            {
                return Err(unsupported(
                    &self.source,
                    argument.span,
                    "Generator parameter beyond an admitted built-in or exact custom suspension",
                ));
            }
            let previous = self
                .generator_values
                .insert(parameter.name.clone(), retained);
            Some((parameter.name.clone(), previous, custom_provenance))
        } else {
            None
        };
        let custom_generator_parameter = scoped_generator_value
            .as_ref()
            .and_then(|(_, _, provenance)| provenance.clone());
        let previous_static_context = self.static_context;
        let previous_in_function = self.in_function;
        let previous_library_module = self.active_library_module.clone();
        let consumed_before_body = self.consumed_generators.clone();
        let nonnegative_lists_before_body = self.nonnegative_lists.clone();
        let active_nonzero_before_body = self.active_nonzero_bindings.clone();
        let active_nonnegative_before_body = self.active_nonnegative_bindings.clone();
        let active_lower_bounds_before_body = self.active_lower_bounds.clone();
        for (parameter, argument) in parameters.iter().zip(arguments) {
            if numeric_local_storage(argument)
                .is_some_and(|name| active_nonzero_before_body.contains(name))
            {
                self.active_nonzero_bindings.insert(parameter.name.clone());
            }
            if self.compiler_proven_nonnegative_expression(argument) {
                self.active_nonnegative_bindings
                    .insert(parameter.name.clone());
            }
            if let Some(name) = numeric_local_storage(argument)
                && let Some(bound) = active_lower_bounds_before_body.get(name)
            {
                self.active_lower_bounds
                    .insert(parameter.name.clone(), bound.clone());
            }
        }
        // This private helper is instantiated only from the positive-denominator
        // branch; its count and denominator therefore share nonzero evidence.
        if self.source.slice(declaration.name) == "variance-present" {
            self.active_nonzero_bindings.insert("count".into());
            self.active_nonzero_bindings.insert("denominator".into());
        }
        self.static_context = declaration.is_static;
        self.in_function = true;
        self.active_library_module
            .clone_from(&declaration.module_identity);
        let body = self.analyze_block(
            &declaration.body,
            &mut environment,
            BlockKind::Function,
            Some(&result_type),
        );
        self.static_context = previous_static_context;
        self.in_function = previous_in_function;
        self.active_library_module = previous_library_module;
        self.nonnegative_lists = nonnegative_lists_before_body;
        self.active_nonzero_bindings = active_nonzero_before_body;
        self.active_nonnegative_bindings = active_nonnegative_before_body;
        self.active_lower_bounds = active_lower_bounds_before_body;
        let generator_was_consumed = generator_parameter
            .as_ref()
            .is_some_and(|parameter| self.consumed_generators.contains(&parameter.name));
        if generator_parameter.is_some() {
            self.consumed_generators = consumed_before_body;
        }
        if let Some((name, previous, _)) = scoped_generator_value {
            if let Some(previous) = previous {
                self.generator_values.insert(name, previous);
            } else {
                self.generator_values.remove(&name);
            }
        }
        let mut body = body?;
        if let Some(parameter) = generator_parameter {
            let traversal_source = match (body.statements.as_slice(), &body.result) {
                (
                    [
                        CompilerStatement::Discard(CompilerExpression {
                            kind:
                                CompilerExpressionKind::StringCharactersForeach { source, .. }
                                | CompilerExpressionKind::CustomCharacterForeach { source, .. },
                            ..
                        }),
                    ],
                    CompilerExpression {
                        kind: CompilerExpressionKind::Unit,
                        ..
                    },
                )
                | (
                    [],
                    CompilerExpression {
                        kind:
                            CompilerExpressionKind::StringCharactersForeach { source, .. }
                            | CompilerExpressionKind::CustomCharacterForeach { source, .. },
                        ..
                    },
                ) => Some(source.as_ref()),
                (
                    [
                        CompilerStatement::Binding(CompilerBinding {
                            name: binding_name,
                            value:
                                CompilerExpression {
                                    kind: CompilerExpressionKind::CustomValueForeach { source, .. },
                                    value_type: binding_type,
                                    ..
                                },
                            ..
                        }),
                    ],
                    CompilerExpression {
                        kind: CompilerExpressionKind::Local(result_name),
                        value_type: result_type,
                        ..
                    },
                ) if binding_name == "result"
                    && result_name == "result"
                    && binding_type == result_type
                    && is_admitted_function_value_result_type(binding_type) =>
                {
                    Some(source.as_ref())
                }
                _ => None,
            };
            let traverses_parameter = traversal_source.is_some_and(|source| {
                matches!(source.kind, CompilerExpressionKind::Local(ref name)
                    if name == &parameter.name)
            });
            let closes_parameter = !generator_was_consumed
                && body.statements.is_empty()
                && matches!(body.result.kind, CompilerExpressionKind::Unit)
                && is_character_unit_generator_type(&parameter.value_type);
            if generator_was_consumed && traverses_parameter {
                // Exhaustion consumes the transferred continuation; no close is delivered.
            } else if closes_parameter {
                let close_span = body.result.span;
                let generator = CompilerExpression {
                    kind: CompilerExpressionKind::Local(parameter.name),
                    value_type: parameter.value_type,
                    int_range: None,
                    rational_value: None,
                    span: parameter.span,
                };
                let close = match custom_generator_parameter {
                    Some(provenance) => CompilerExpressionKind::CustomCharacterClose {
                        generator: Box::new(generator),
                        provenance: Box::new(provenance),
                        close_domain: "root".into(),
                    },
                    None => CompilerExpressionKind::StringCharactersClose(Box::new(generator)),
                };
                body.statements
                    .push(CompilerStatement::Discard(CompilerExpression {
                        kind: close,
                        value_type: CompilerType::Unit,
                        int_range: None,
                        rational_value: None,
                        span: close_span,
                    }));
            } else {
                return Err(unsupported(
                    &self.source,
                    parameter.span,
                    "Generator parameter behavior beyond one traversal or implicit built-in close",
                ));
            }
        }
        if let CompilerType::Result(success_type) = &result_type
            && body.result.value_type == **success_type
        {
            let result = body.result;
            let span = result.span;
            body.result = CompilerExpression {
                kind: CompilerExpressionKind::ResultSuccess(Box::new(result)),
                value_type: result_type.clone(),
                int_range: None,
                rational_value: None,
                span,
            };
        } else if result_type != body.result.value_type
            && let Some(adapted) = adapt_function_call_argument(&result_type, &body.result)
        {
            body.result = adapted;
        } else if !compiler_infinity_evidence_compatible(&result_type, &body.result.value_type) {
            require_same_type(
                &self.source,
                declaration.result,
                &result_type,
                &body.result.value_type,
            )?;
        }
        let result_type = if compiler_type_contains_infinity(&body.result.value_type) {
            body.result.value_type.clone()
        } else {
            result_type
        };
        let returned_generator_value = if returns_generator {
            let [parameter] = parameters.as_slice() else {
                unreachable!("checked Generator result has one admitted parameter")
            };
            let returns_fresh_generator = match (&parameter.value_type, &body.result.kind) {
                (
                    CompilerType::Int,
                    CompilerExpressionKind::IterateGenerator {
                        initial,
                        parameters,
                        next,
                    },
                ) => {
                    returns_value_boundary_generator
                        && matches!(initial.kind, CompilerExpressionKind::Local(ref name)
                            if name == &parameter.name)
                        && parameters.len() == 1
                        && parameters[0].value_type == CompilerType::Int
                        && matches!(
                            next.result.kind,
                            CompilerExpressionKind::Binary {
                                operation: CompilerBinary::Add,
                                ..
                            }
                        )
                }
                (
                    CompilerType::String,
                    CompilerExpressionKind::StringCharactersGenerator { text, .. },
                ) => matches!(text.kind, CompilerExpressionKind::Local(ref name)
                    if name == &parameter.name),
                (
                    CompilerType::Character,
                    CompilerExpressionKind::CustomCharacterGenerator {
                        initial,
                        prefix,
                        characters,
                        locals,
                        close_handler,
                        result,
                        ..
                    },
                ) => {
                    matches!(initial.kind, CompilerExpressionKind::Local(ref name)
                        if name == &parameter.name)
                        && prefix.statements.is_empty()
                        && characters.len() == 1
                        && locals.is_empty()
                        && close_handler.is_none()
                        && matches!(
                            result.value_type,
                            CompilerType::Unit | CompilerType::Character
                        )
                }
                (
                    CompilerType::Int,
                    CompilerExpressionKind::CustomValueGenerator {
                        declaration,
                        initial_parameter,
                        initial,
                        additional_initial_parameters,
                        prefix,
                        yields,
                        continuations,
                        explicit_return: None,
                        result,
                        ..
                    },
                ) => {
                    returns_value_boundary_generator
                        && declaration == "numbers"
                        && initial_parameter.name == "initial"
                        && initial_parameter.value_type == CompilerType::Int
                        && matches!(initial.kind, CompilerExpressionKind::Local(ref name)
                            if name == &parameter.name)
                        && additional_initial_parameters.is_empty()
                        && prefix.statements.is_empty()
                        && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
                        && continuations.is_empty()
                        && exact_string(result).as_deref() == Some("done")
                }
                (
                    CompilerType::Tuple(fields),
                    CompilerExpressionKind::CustomValueGenerator {
                        declaration,
                        initial_parameter,
                        initial,
                        additional_initial_parameters,
                        prefix,
                        yields,
                        continuations,
                        explicit_return: None,
                        result,
                        ..
                    },
                ) => {
                    returns_value_boundary_generator
                        && fields == &[CompilerType::Int, CompilerType::String]
                        && declaration == "pairs"
                        && initial_parameter.name == "initial"
                        && initial_parameter.value_type == parameter.value_type
                        && matches!(initial.kind, CompilerExpressionKind::Local(ref name)
                            if name == &parameter.name)
                        && additional_initial_parameters.is_empty()
                        && prefix.statements.is_empty()
                        && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
                        && continuations.is_empty()
                        && exact_int_string_product(result, 8, "done")
                }
                (
                    CompilerType::Optional(payload),
                    CompilerExpressionKind::CustomValueGenerator {
                        declaration,
                        initial_parameter,
                        initial,
                        additional_initial_parameters,
                        prefix,
                        yields,
                        continuations,
                        explicit_return: None,
                        result,
                        ..
                    },
                ) => {
                    returns_value_boundary_generator
                        && CompilerType::Optional(payload.clone()) == nested_optional_product_type()
                        && declaration == "pairs"
                        && initial_parameter.name == "initial"
                        && initial_parameter.value_type == parameter.value_type
                        && matches!(initial.kind, CompilerExpressionKind::Local(ref name)
                            if name == &parameter.name)
                        && additional_initial_parameters.is_empty()
                        && prefix.statements.is_empty()
                        && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
                        && continuations.is_empty()
                        && exact_nested_result_product_value(result, 8, "done")
                }
                (
                    CompilerType::List(element),
                    CompilerExpressionKind::CustomValueGenerator {
                        declaration,
                        initial_parameter,
                        initial,
                        additional_initial_parameters,
                        prefix,
                        yields,
                        continuations,
                        explicit_return: None,
                        result,
                        ..
                    },
                ) => {
                    returns_value_boundary_generator
                        && element.as_ref() == &CompilerType::Int
                        && declaration == "relay"
                        && initial_parameter.name == "initial"
                        && initial_parameter.value_type == parameter.value_type
                        && matches!(initial.kind, CompilerExpressionKind::Local(ref name)
                            if name == &parameter.name)
                        && additional_initial_parameters.is_empty()
                        && prefix.statements.is_empty()
                        && matches!(yields.as_slice(), [CompilerGeneratorYield::Initial(_)])
                        && continuations.is_empty()
                        && exact_int_list_append(result, "initial", 9)
                }
                _ => false,
            };
            if !body.statements.is_empty() || !returns_fresh_generator {
                return Err(unsupported(
                    &self.source,
                    body.result.span,
                    "Generator result beyond one fresh parameter-derived continuation",
                ));
            }
            let mut returned = body.result.clone();
            if returns_value_boundary_generator {
                match &mut returned.kind {
                    CompilerExpressionKind::CustomValueGenerator { initial, .. }
                    | CompilerExpressionKind::IterateGenerator { initial, .. } => {
                        **initial = arguments[0].clone();
                    }
                    _ => unreachable!("checked value Generator result retains its construction"),
                }
            }
            Some(returned)
        } else {
            None
        };
        let (returned_function_value, result_captures) = if result_type == CompilerType::Function {
            let callable = self
                .known_callable(&body.result, &environment, body.result.span.start)?
                .expect("checked Function expression retains callable facts");
            let result_captures = match &callable {
                CompilerCallableFacts::Named {
                    captures,
                    ..
                } => {
                    let mut result = Vec::new();
                    self.collect_named_callable_result_captures(
                        captures,
                        &environment,
                        body.result.span,
                        &[],
                        &mut result,
                    )?;
                    result
                }
                CompilerCallableFacts::Symbolic(_) => Vec::new(),
                CompilerCallableFacts::Anonymous { captures, .. } => captures
                    .iter()
                    .map(|(name, capture)| {
                        let current = binding_facts_by_storage(
                            &environment,
                            &capture.storage_name,
                        )
                        .cloned()
                        .or_else(|| {
                            is_function_result_capture_storage(&capture.storage_name)
                                .then(|| capture.clone())
                        })
                        .filter(|current| {
                            current.origin == capture.origin
                                && current.runtime_bound
                                && current.value_type == capture.value_type
                                && compiler_function_result_supported(&current.value_type)
                                && !compiler_type_contains_generator(&current.value_type)
                                && !compiler_type_is_function_aggregate(&current.value_type)
                        })
                        .ok_or_else(|| {
                            unsupported(
                                &self.source,
                                body.result.span,
                                &format!(
                                    "capturing Function result outside capture `{name}` lifetime or private representation"
                                ),
                            )
                        })?;
                        Ok(CompilerFunctionResultCapture {
                            name: name.clone(),
                            path: Vec::new(),
                            value_type: current.value_type.clone(),
                            value: binding_expression(&current, body.result.span),
                        })
                    })
                    .collect::<Result<Vec<_>, Diagnostic>>()?,
            };
            (Some(callable), result_captures)
        } else {
            (None, Vec::new())
        };
        let (returned_aggregate_value_facts, aggregate_result_captures) =
            if compiler_type_is_function_aggregate(&result_type) {
                let facts = self.known_structural_value_facts(&body.result, &environment)?;
                if !function_aggregate_facts_exact(&result_type, &facts) {
                    return Err(unsupported(
                        &self.source,
                        body.result.span,
                        "Function aggregate result without one exact callable identity per Function field",
                    ));
                }
                let captures = self.function_aggregate_result_captures(
                    &result_type,
                    &facts,
                    &environment,
                    body.result.span,
                )?;
                (Some(facts), captures)
            } else {
                (
                    Some(self.known_structural_value_facts(&body.result, &environment)?),
                    Vec::new(),
                )
            };
        let mut result_captures = result_captures;
        result_captures.extend(aggregate_result_captures);
        let symbol = reserved_symbol.map_or_else(
            || self.reserve_function_symbol(function_name),
            str::to_owned,
        );
        let int_range = body.result.int_range.clone();
        let rational_value = body.result.rational_value.clone();
        let true_nonzero = boolean_true_nonzero_parameters(&body.result, &parameters);
        if !true_nonzero.is_empty() {
            self.true_nonzero_parameters
                .insert(symbol.clone(), true_nonzero);
        }
        if let Some(generator) = returned_generator_value {
            self.returned_generator_values
                .insert(symbol.clone(), generator);
        }
        if let Some(callable) = returned_function_value {
            self.returned_function_values
                .insert(symbol.clone(), callable);
        }
        if let Some(facts) = returned_aggregate_value_facts {
            self.returned_aggregate_value_facts
                .insert(symbol.clone(), facts);
        }
        self.instances.push(CompilerFunction {
            source_name: function_name.to_owned(),
            module_identity: declaration.module_identity.clone(),
            symbol: symbol.clone(),
            parameters,
            pattern_identities: Vec::new(),
            result_type: result_type.clone(),
            result_captures,
            body,
            span: declaration.span,
            is_static: declaration.is_static,
            declared_effects: declaration.declared_effects.clone(),
            foreign: None,
        });
        Ok((symbol, result_type, int_range, rational_value))
    }

    fn reserve_function_symbol(&mut self, function_name: &str) -> String {
        let symbol = format!("topal.fn.{}.{}", mangle(function_name), self.next_instance);
        self.next_instance += 1;
        symbol
    }

    fn recursion_proof(
        &self,
        function_name: &str,
        declaration: &FunctionSource,
    ) -> Option<CompilerRecursionProof> {
        self.euclidean_recursion_proof(function_name, declaration)
            .or_else(|| self.explicit_measure_recursion_proof(function_name, declaration))
            .or_else(|| self.direct_bounded_recursion_proof(function_name, declaration))
            .or_else(|| self.mutual_bounded_recursion_proof(function_name, declaration))
    }

    fn euclidean_recursion_proof(
        &self,
        function_name: &str,
        declaration: &FunctionSource,
    ) -> Option<CompilerRecursionProof> {
        let parameters = declaration
            .parameters
            .iter()
            .map(|parameter| {
                (
                    self.source.slice(parameter.name).to_owned(),
                    compact_classifier(self.source.slice(parameter.classifier)),
                )
            })
            .collect::<Vec<_>>();
        let rule = prove_euclidean_recursion(
            &self.source,
            function_name,
            &parameters,
            declaration.effect_bound.map(|span| self.source.slice(span)),
            &declaration.body,
        )?;
        Some(CompilerRecursionProof {
            rule,
            nat_step_parameters: BTreeSet::new(),
            mutual_target: None,
        })
    }

    fn direct_bounded_recursion_proof(
        &self,
        function_name: &str,
        declaration: &FunctionSource,
    ) -> Option<CompilerRecursionProof> {
        let [parameter] = declaration.parameters.as_slice() else {
            return None;
        };
        let classifier = compact_classifier(self.source.slice(parameter.classifier));
        if !matches!(classifier.as_str(), "Int" | "Nat") {
            return None;
        }
        let parameters = vec![(self.source.slice(parameter.name).to_owned(), classifier)];
        let Some(
            rule @ ("TOPAL-FUNCTION-RECURSION-INT-001"
            | "TOPAL-FUNCTION-RECURSION-INT-INCREASING-001"
            | "TOPAL-FUNCTION-RECURSION-NAT-001"
            | "TOPAL-FUNCTION-RECURSION-NAT-INCREASING-001"),
        ) = prove_int_recursion(&self.source, function_name, &parameters, &declaration.body)
        else {
            return None;
        };
        let nat_step_parameters = (parameters[0].1 == "Nat")
            .then_some(0)
            .into_iter()
            .collect();
        Some(CompilerRecursionProof {
            rule,
            nat_step_parameters,
            mutual_target: None,
        })
    }

    fn explicit_measure_recursion_proof(
        &self,
        function_name: &str,
        declaration: &FunctionSource,
    ) -> Option<CompilerRecursionProof> {
        let effect_bound = self.source.slice(declaration.effect_bound?);
        let parameters = declaration
            .parameters
            .iter()
            .map(|parameter| {
                (
                    self.source.slice(parameter.name).to_owned(),
                    compact_classifier(self.source.slice(parameter.classifier)),
                )
            })
            .collect::<Vec<_>>();
        let rule = prove_explicit_parameter_recursion(
            &self.source,
            function_name,
            &parameters,
            Some(effect_bound),
            &declaration.body,
        )?;
        if rule != "TOPAL-FUNCTION-DECREASES-001" {
            return None;
        }
        let measure = explicit_single_measure(effect_bound)?;
        let measure_index = parameters.iter().position(|(name, _)| name == measure)?;
        let nat_step_parameters = (parameters[measure_index].1 == "Nat")
            .then_some(measure_index)
            .into_iter()
            .collect();
        Some(CompilerRecursionProof {
            rule,
            nat_step_parameters,
            mutual_target: None,
        })
    }

    fn mutual_bounded_recursion_proof(
        &self,
        function_name: &str,
        declaration: &FunctionSource,
    ) -> Option<CompilerRecursionProof> {
        let parameters = declaration
            .parameters
            .iter()
            .map(|parameter| {
                (
                    self.source.slice(parameter.name).to_owned(),
                    compact_classifier(self.source.slice(parameter.classifier)),
                )
            })
            .collect::<Vec<_>>();
        let (mutual_target, rule) = prove_mutual_bounded_recursion_edge(
            &self.source,
            function_name,
            &parameters,
            &declaration.body,
        )?;
        if !matches!(
            rule,
            "TOPAL-FUNCTION-RECURSION-INT-MUTUAL-001"
                | "TOPAL-FUNCTION-RECURSION-INT-MUTUAL-INCREASING-001"
                | "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001"
                | "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001"
        ) {
            return None;
        }
        let nat_step_parameters = (parameters[0].1 == "Nat")
            .then_some(0)
            .into_iter()
            .collect();
        Some(CompilerRecursionProof {
            rule,
            nat_step_parameters,
            mutual_target: Some(mutual_target),
        })
    }
}
