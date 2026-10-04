impl Analyzer {
    fn analyze_range_observation(
        &mut self,
        operation: &str,
        operand: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let operand = self.analyze_expression(operand, environment)?;
        if operation == "empty?" && operand.value_type == CompilerType::String {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::StringEmptyPredicate(Box::new(operand)),
                value_type: CompilerType::Boolean,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if operation == "empty?"
            && let CompilerType::List(element) = &operand.value_type
        {
            if !compiler_list_observation_element_supported(element.as_ref()) {
                return Err(unsupported(
                    &self.source,
                    operand.span,
                    "empty? for this List element type",
                ));
            }
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListEmptyPredicate(Box::new(operand)),
                value_type: CompilerType::Boolean,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if operation == "empty?"
            && matches!(
                operand.value_type,
                CompilerType::Array { .. }
                    | CompilerType::Set(_)
                    | CompilerType::Bag(_)
                    | CompilerType::Map { .. }
            )
        {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ContainerEmpty(Box::new(operand)),
                value_type: CompilerType::Boolean,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        let CompilerType::Range(endpoint) = &operand.value_type else {
            if operation == "empty?" {
                return Err(unsupported(
                    &self.source,
                    operand.span,
                    "empty? for this value type",
                ));
            }
            return Err(source_diagnostic(
                &self.source,
                "E-TYPE-MISMATCH",
                operand.span,
                format!("{operation} requires an exact Range operand"),
            ));
        };
        let endpoint = endpoint.as_ref().clone();
        let (kind, value_type) = match operation {
            "range-lower" => (
                CompilerExpressionKind::RangeLower(Box::new(operand)),
                endpoint,
            ),
            "range-upper" => (
                CompilerExpressionKind::RangeUpper(Box::new(operand)),
                endpoint,
            ),
            "range-lower-inclusive?" => (
                CompilerExpressionKind::RangeLowerInclusive(Box::new(operand)),
                CompilerType::Boolean,
            ),
            "range-upper-inclusive?" => (
                CompilerExpressionKind::RangeUpperInclusive(Box::new(operand)),
                CompilerType::Boolean,
            ),
            "empty?" => (
                CompilerExpressionKind::RangeEmpty(Box::new(operand)),
                CompilerType::Boolean,
            ),
            _ => unreachable!("range observation spelling selected above"),
        };
        Ok(CompilerExpression {
            kind,
            value_type,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_identifier_binary(
        &mut self,
        operation: &str,
        left: &Expression,
        right: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let left = self.analyze_expression(left, environment)?;
        let right = self.analyze_expression(right, environment)?;
        let binary = match operation {
            "and" => CompilerBinary::And,
            "or" => CompilerBinary::Or,
            "xor" => CompilerBinary::Xor,
            "in" => CompilerBinary::In,
            "contains" => CompilerBinary::Contains,
            _ => unreachable!("identifier binary spelling selected above"),
        };
        if matches!(binary, CompilerBinary::In | CompilerBinary::Contains) {
            return self.analyze_range_membership_binary(binary, left, right, span);
        }
        if binary == CompilerBinary::And
            && let CompilerType::Range(left_endpoint) = &left.value_type
        {
            let CompilerType::Range(right_endpoint) = &right.value_type else {
                return Err(source_diagnostic(
                    &self.source,
                    "E-TYPE-MISMATCH",
                    span,
                    format!(
                        "expected {}, found {}",
                        left.value_type.name(),
                        right.value_type.name()
                    ),
                ));
            };
            let value_type =
                if is_int_range_endpoint(left_endpoint) && is_int_range_endpoint(right_endpoint) {
                    let endpoint = if left_endpoint.as_ref() == &CompilerType::InfiniteInt
                        || right_endpoint.as_ref() == &CompilerType::InfiniteInt
                    {
                        CompilerType::InfiniteInt
                    } else {
                        CompilerType::Int
                    };
                    CompilerType::Range(Box::new(endpoint))
                } else if is_rational_range_endpoint(left_endpoint)
                    && is_rational_range_endpoint(right_endpoint)
                {
                    let endpoint = if left_endpoint.as_ref() == &CompilerType::InfiniteRational
                        || right_endpoint.as_ref() == &CompilerType::InfiniteRational
                    {
                        CompilerType::InfiniteRational
                    } else {
                        CompilerType::Rational
                    };
                    CompilerType::Range(Box::new(endpoint))
                } else {
                    require_same_type(&self.source, span, &left.value_type, &right.value_type)?;
                    left.value_type.clone()
                };
            return Ok(Self::finish_binary(binary, left, right, value_type, span));
        }
        if let (
            CompilerExpressionKind::Capability(left),
            CompilerExpressionKind::Capability(right),
        ) = (&left.kind, &right.kind)
        {
            let capability = match binary {
                CompilerBinary::And => left.and(right),
                CompilerBinary::Or => left.or(right),
                _ => {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "Capability operator other than and/or",
                    ));
                }
            };
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::Capability(capability),
                value_type: CompilerType::Capability,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        require_type(
            &self.source,
            left.span,
            &CompilerType::Boolean,
            &left.value_type,
        )?;
        require_type(
            &self.source,
            right.span,
            &CompilerType::Boolean,
            &right.value_type,
        )?;
        Ok(Self::finish_binary(
            binary,
            left,
            right,
            CompilerType::Boolean,
            span,
        ))
    }

    fn analyze_range_membership_binary(
        &self,
        binary: CompilerBinary,
        mut left: CompilerExpression,
        mut right: CompilerExpression,
        span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        let (range, value) = if binary == CompilerBinary::In {
            (&right.value_type, &mut left)
        } else {
            (&left.value_type, &mut right)
        };
        let CompilerType::Range(endpoint) = range else {
            return Err(source_diagnostic(
                &self.source,
                "E-RANGE-MEMBERSHIP-OPERANDS",
                span,
                "range membership requires an exact Range operand",
            ));
        };
        require_exact_numeric(&self.source, value.span, &value.value_type)?;
        if is_int_range_endpoint(endpoint)
            && matches!(
                value.value_type,
                CompilerType::Int | CompilerType::InfiniteInt | CompilerType::InfiniteNat
            )
        {
            *value = forget_nat_evidence(value.clone());
        } else if is_rational_range_endpoint(endpoint) {
            if matches!(
                value.value_type,
                CompilerType::InfiniteInt | CompilerType::InfiniteNat
            ) {
                return Err(unsupported(
                    &self.source,
                    value.span,
                    "cross-domain Int/Rational infinity membership conversion",
                ));
            }
            if matches!(value.value_type, CompilerType::Int | CompilerType::Nat) {
                *value = into_rational(value.clone());
            }
            if !matches!(
                value.value_type,
                CompilerType::Rational | CompilerType::InfiniteRational
            ) {
                require_same_type(&self.source, value.span, endpoint, &value.value_type)?;
            }
        } else {
            require_same_type(&self.source, value.span, endpoint, &value.value_type)?;
        }
        Ok(Self::finish_binary(
            binary,
            left,
            right,
            CompilerType::Boolean,
            span,
        ))
    }

    fn analyze_bound_symbolic_callable(
        &mut self,
        kind: CallableKind,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let [alias, argument] = items else {
            return Err(source_diagnostic(
                &self.source,
                "E-NO-APPLICABLE-OVERLOAD",
                span,
                "a symbolic Function value accepts one direct or product operand",
            ));
        };
        if let Expression::Product { fields, .. } = argument
            && let [left, right] = fields.as_slice()
            && left.label.is_none()
            && right.label.is_none()
        {
            return self.analyze_symbolic_binary(
                kind,
                &left.value,
                &right.value,
                span,
                environment,
            );
        }
        if kind == CallableKind::Minus {
            let application = [
                Expression::Callable {
                    kind,
                    span: alias.span(),
                },
                argument.clone(),
            ];
            return self.analyze_application(&application, span, environment);
        }
        Err(source_diagnostic(
            &self.source,
            "E-NO-APPLICABLE-OVERLOAD",
            argument.span(),
            "a binary symbolic Function value requires a two-field positional product",
        ))
    }

    fn analyze_callable_application(
        &mut self,
        callable: CompilerCallableFacts,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        match callable {
            CompilerCallableFacts::Named {
                name,
                declarations,
                captures,
            } => {
                let mut call_environment = environment.clone();
                let mut lexical_captures = Vec::new();
                let mut carries_environment = false;
                for capture in captures {
                    if capture.parameter_name.starts_with("@ ")
                        || capture.parameter_name.starts_with("root ")
                    {
                        carries_environment = true;
                        let (CompilerExpressionKind::Local(storage_name)
                        | CompilerExpressionKind::InfinityLocal { storage_name, .. }) =
                            &capture.argument.kind
                        else {
                            return Err(unsupported(
                                &self.source,
                                capture.span,
                                "named Function environment without retained private storage",
                            ));
                        };
                        let facts = binding_facts_by_storage(environment, storage_name)
                            .cloned()
                            .or_else(|| named_callable_capture_binding(&capture))
                            .ok_or_else(|| {
                                unsupported(
                                    &self.source,
                                    capture.span,
                                    "named Function environment outside its retained lifetime",
                                )
                            })?;
                        call_environment.insert(capture.parameter_name, facts);
                    } else {
                        lexical_captures.push(capture);
                    }
                }
                self.analyze_resolved_call_from(
                    items,
                    span,
                    &call_environment,
                    0,
                    &name,
                    &declarations,
                    &lexical_captures,
                    carries_environment,
                )
            }
            CompilerCallableFacts::Symbolic(kind) => {
                self.analyze_bound_symbolic_callable(kind, items, span, environment)
            }
            CompilerCallableFacts::Anonymous {
                parameters,
                body,
                captures,
                static_context,
                span: declaration_span,
            } => self.analyze_bound_anonymous_function(
                &parameters,
                &body,
                &captures,
                static_context,
                declaration_span,
                items,
                span,
                environment,
            ),
        }
    }

    fn flatten_anonymous_argument(
        &self,
        pattern: &AnonymousPattern,
        argument: &CompilerExpression,
        argument_facts: Option<&CompilerExpression>,
        flattened: &mut Vec<(Span, CompilerExpression, Option<CompilerExpression>)>,
    ) -> Result<(), Diagnostic> {
        match pattern {
            AnonymousPattern::Binding(name) => {
                flattened.push((*name, argument.clone(), argument_facts.cloned()));
            }
            AnonymousPattern::Product {
                fields,
                span: pattern_span,
            } => {
                let CompilerType::Tuple(field_types) = &argument.value_type else {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-ANONYMOUS-PRODUCT-PATTERN",
                        argument.span,
                        "anonymous product pattern requires a positional product",
                    ));
                };
                if fields.len() != field_types.len() {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-ANONYMOUS-PRODUCT-PATTERN",
                        *pattern_span,
                        format!(
                            "anonymous product pattern expects {} fields, found {}",
                            fields.len(),
                            field_types.len()
                        ),
                    ));
                }
                let literal_fields = argument_facts.and_then(|facts| {
                    if let CompilerExpressionKind::Tuple(fields) = &facts.kind {
                        Some(fields)
                    } else {
                        None
                    }
                });
                for (index, (field, value_type)) in fields.iter().zip(field_types).enumerate() {
                    let projected_facts = literal_fields
                        .and_then(|values| values.get(index).cloned())
                        .or_else(|| {
                            argument_facts.map(|facts| CompilerExpression {
                                kind: CompilerExpressionKind::TupleField {
                                    tuple: Box::new(facts.clone()),
                                    index,
                                },
                                value_type: value_type.clone(),
                                int_range: None,
                                rational_value: None,
                                span: facts.span,
                            })
                        });
                    let projected = CompilerExpression {
                        kind: CompilerExpressionKind::TupleField {
                            tuple: Box::new(argument.clone()),
                            index,
                        },
                        value_type: value_type.clone(),
                        int_range: projected_facts
                            .as_ref()
                            .and_then(|value| value.int_range.clone()),
                        rational_value: projected_facts
                            .as_ref()
                            .and_then(|value| value.rational_value.clone()),
                        span: argument.span,
                    };
                    self.flatten_anonymous_argument(
                        field,
                        &projected,
                        projected_facts.as_ref(),
                        flattened,
                    )?;
                }
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)] // Retained source identity and call-site evidence are intentionally explicit.
    fn analyze_bound_anonymous_function(
        &mut self,
        parameters: &[AnonymousPattern],
        body: &Expression,
        captures: &BTreeMap<String, BindingFacts>,
        static_context: bool,
        declaration_span: Span,
        items: &[Expression],
        call_span: Span,
        call_environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let [_, argument_source] = items else {
            return Err(source_diagnostic(
                &self.source,
                "E-ANONYMOUS-FUNCTION-ARITY",
                call_span,
                "an anonymous Function value accepts exactly one direct or product operand",
            ));
        };
        let argument = self.analyze_expression(argument_source, call_environment)?;
        let destructures_product = parameters.iter().any(anonymous_pattern_contains_product);
        let materialize_argument = parameters.len() != 1 || destructures_product;
        let literal_fields = match &argument.kind {
            CompilerExpressionKind::Tuple(fields) => Some(fields.clone()),
            _ => None,
        };
        let (parameter_arguments, parameter_facts, private_argument) = if materialize_argument {
            let storage_name = format!("topal.anonymous.argument.{}", call_span.start);
            let local = CompilerExpression {
                kind: CompilerExpressionKind::Local(storage_name.clone()),
                value_type: argument.value_type.clone(),
                int_range: argument.int_range.clone(),
                rational_value: argument.rational_value.clone(),
                span: argument.span,
            };
            let (values, facts) = if parameters.len() == 1 {
                (vec![local], vec![Some(argument.clone())])
            } else if let CompilerType::Tuple(fields) = &argument.value_type {
                if fields.len() != parameters.len() {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-ANONYMOUS-FUNCTION-ARITY",
                        argument_source.span(),
                        format!(
                            "anonymous function expects {} arguments, found {}",
                            parameters.len(),
                            fields.len()
                        ),
                    ));
                }
                let values = fields
                    .iter()
                    .enumerate()
                    .map(|(index, value_type)| CompilerExpression {
                        kind: CompilerExpressionKind::TupleField {
                            tuple: Box::new(local.clone()),
                            index,
                        },
                        value_type: value_type.clone(),
                        int_range: literal_fields
                            .as_ref()
                            .and_then(|values| values[index].int_range.clone()),
                        rational_value: literal_fields
                            .as_ref()
                            .and_then(|values| values[index].rational_value.clone()),
                        span: argument.span,
                    })
                    .collect();
                let facts = (0..fields.len())
                    .map(|index| literal_fields.as_ref().map(|values| values[index].clone()))
                    .collect();
                (values, facts)
            } else {
                return Err(source_diagnostic(
                    &self.source,
                    "E-ANONYMOUS-ARGUMENT-PACKAGE",
                    argument_source.span(),
                    format!(
                        "anonymous function expects {} arguments packaged as a tuple, found `{}`",
                        parameters.len(),
                        argument.value_type.name()
                    ),
                ));
            };
            (values, facts, Some((storage_name, argument)))
        } else {
            (vec![argument], vec![None], None)
        };

        let mut flattened_arguments = Vec::new();
        for ((parameter, argument), argument_facts) in parameters
            .iter()
            .zip(parameter_arguments)
            .zip(parameter_facts)
        {
            self.flatten_anonymous_argument(
                parameter,
                &argument,
                argument_facts.as_ref(),
                &mut flattened_arguments,
            )?;
        }

        let source_parameter_count = flattened_arguments.len();
        let mut environment: BTreeMap<String, BindingFacts> = BTreeMap::new();
        let mut lowered_parameters: Vec<CompilerParameter> =
            Vec::with_capacity(flattened_arguments.len());
        let mut arguments = Vec::with_capacity(flattened_arguments.len() + captures.len());
        let mut pattern_identities = Vec::new();
        let mut parameter_callable_captures: Vec<CompilerContextCapture> = Vec::new();
        let mut parameter_callable_capture_ranges: BTreeMap<usize, (usize, usize)> =
            BTreeMap::new();
        for (name_span, argument, argument_facts) in flattened_arguments {
            let facts = argument_facts.as_ref().unwrap_or(&argument);
            let name = self.source.slice(name_span).to_owned();
            let discarded = name == "_";
            if argument.value_type == CompilerType::Scope
                || !compiler_function_parameter_supported(&argument.value_type)
            {
                return Err(unsupported(
                    &self.source,
                    argument.span,
                    "unsupported inferred anonymous-function parameter",
                ));
            }
            let mut structural_facts = compiler_type_is_function_aggregate(&argument.value_type)
                .then(|| self.known_structural_value_facts(facts, call_environment))
                .transpose()?;
            let repeated = validate_repeated_anonymous_pattern_parameter(
                &self.source,
                &lowered_parameters,
                &name,
                discarded,
                &argument.value_type,
                name_span,
            )?;
            if let Some(first_parameter) = repeated {
                pattern_identities.push(CompilerPatternIdentity {
                    first_parameter,
                    repeated_parameter: lowered_parameters.len(),
                    span: name_span,
                });
                if argument.value_type == CompilerType::Function {
                    let first_callable = environment
                        .get(&name)
                        .and_then(|facts| facts.callable.as_ref())
                        .expect("first Function pattern occurrence retains callable facts");
                    let mut current_callable = self
                        .known_callable(facts, call_environment, facts.span.start)?
                        .expect("repeated Function pattern occurrence retains callable facts");
                    if same_callable_identity(first_callable, &current_callable) {
                        let capture_start = parameter_callable_captures.len();
                        self.forward_callable_captures(
                            &format!("{name} repeated {}", lowered_parameters.len()),
                            name_span,
                            &mut current_callable,
                            call_environment,
                            &mut parameter_callable_captures,
                        )?;
                        let capture_end = parameter_callable_captures.len();
                        let (first_start, first_end) = parameter_callable_capture_ranges
                            .get(&first_parameter)
                            .copied()
                            .expect("first captured Function occurrence retains capture range");
                        append_repeated_capture_identities(
                            &self.source,
                            name_span,
                            source_parameter_count,
                            (first_start, first_end),
                            (capture_start, capture_end),
                            &parameter_callable_captures,
                            &mut pattern_identities,
                        )?;
                    }
                }
                if compiler_type_is_function_aggregate(&argument.value_type) {
                    let current = structural_facts
                        .as_mut()
                        .expect("Function aggregate argument retains structural facts");
                    let first = environment
                        .get(&name)
                        .expect("first repeated pattern occurrence retains binding facts");
                    if !function_aggregate_binding_facts_exact(&argument.value_type, first)
                        || !function_aggregate_facts_exact(&argument.value_type, current)
                    {
                        return Err(unsupported(
                            &self.source,
                            name_span,
                            "repeated anonymous Function aggregate identity requires exact callable facts",
                        ));
                    }
                    if function_aggregate_binding_facts_same_callable_identity(
                        &argument.value_type,
                        first,
                        current,
                    ) {
                        let capture_start = parameter_callable_captures.len();
                        self.forward_function_aggregate_captures(
                            &argument.value_type,
                            current,
                            &format!("{name} repeated {}", lowered_parameters.len()),
                            name_span,
                            call_environment,
                            &mut parameter_callable_captures,
                        )?;
                        let capture_end = parameter_callable_captures.len();
                        let (first_start, first_end) = parameter_callable_capture_ranges
                            .get(&first_parameter)
                            .copied()
                            .expect("first Function aggregate occurrence retains capture range");
                        append_repeated_capture_identities(
                            &self.source,
                            name_span,
                            source_parameter_count,
                            (first_start, first_end),
                            (capture_start, capture_end),
                            &parameter_callable_captures,
                            &mut pattern_identities,
                        )?;
                    }
                }
            } else if !discarded {
                let mut callable =
                    self.known_callable(facts, call_environment, facts.span.start)?;
                let capture_start = parameter_callable_captures.len();
                if let Some(callable) = &mut callable {
                    self.forward_callable_captures(
                        &name,
                        name_span,
                        callable,
                        call_environment,
                        &mut parameter_callable_captures,
                    )?;
                }
                if compiler_type_is_function_aggregate(&argument.value_type) {
                    self.forward_function_aggregate_captures(
                        &argument.value_type,
                        structural_facts
                            .as_mut()
                            .expect("Function aggregate argument retains structural facts"),
                        &name,
                        name_span,
                        call_environment,
                        &mut parameter_callable_captures,
                    )?;
                }
                let capture_end = parameter_callable_captures.len();
                if argument.value_type == CompilerType::Function
                    || compiler_type_is_function_aggregate(&argument.value_type)
                {
                    parameter_callable_capture_ranges
                        .insert(lowered_parameters.len(), (capture_start, capture_end));
                }
                environment.insert(
                    name.clone(),
                    BindingFacts {
                        storage_name: name.clone(),
                        origin: name_span.start,
                        runtime_bound: true,
                        value_type: argument.value_type.clone(),
                        int_range: facts.int_range.clone(),
                        rational_value: facts.rational_value.clone(),
                        infinity_negative: compiler_infinity_direction(facts),
                        string_value: exact_string(facts),
                        string_characters: structural_facts
                            .as_ref()
                            .and_then(|facts| facts.string_characters.clone()),
                        closed_int_range: None,
                        list_count: Self::known_list_count(facts, call_environment),
                        list_string_keys: Self::known_list_string_keys(facts, call_environment),
                        list_string_characters: structural_facts
                            .as_ref()
                            .and_then(|facts| facts.list_string_characters.clone()),
                        list_entries: structural_facts
                            .as_ref()
                            .and_then(|facts| facts.list_entries.clone()),
                        array_entries: structural_facts
                            .as_ref()
                            .and_then(|facts| facts.array_entries.clone()),
                        map_entries: structural_facts
                            .as_ref()
                            .and_then(|facts| facts.map_entries.clone()),
                        tuple_fields: structural_facts
                            .as_ref()
                            .map(|facts| facts.tuple_fields.clone())
                            .unwrap_or_default(),
                        record_fields: structural_facts
                            .as_ref()
                            .map(|facts| facts.record_fields.clone())
                            .unwrap_or_default(),
                        optional: structural_facts
                            .as_ref()
                            .and_then(|facts| facts.optional.clone()),
                        sum: structural_facts
                            .as_ref()
                            .and_then(|facts| facts.sum.clone()),
                        result: structural_facts
                            .as_ref()
                            .and_then(|facts| facts.result.clone()),
                        namespace: None,
                        callable,
                        static_capability: None,
                    },
                );
            }
            arguments.push(argument.clone());
            lowered_parameters.push(CompilerParameter {
                name,
                discarded: discarded || repeated.is_some(),
                source_visible: !discarded && repeated.is_none(),
                value_type: argument.value_type.clone(),
                int_range: facts.int_range.clone(),
                span: name_span,
            });
        }

        for capture in parameter_callable_captures {
            arguments.push(capture.argument.clone());
            environment.insert(
                capture.parameter_name.clone(),
                BindingFacts {
                    storage_name: capture.parameter_name.clone(),
                    origin: capture.span.start,
                    runtime_bound: true,
                    value_type: capture.value_type.clone(),
                    int_range: capture.int_range.clone(),
                    rational_value: capture.rational_value.clone(),
                    infinity_negative: compiler_infinity_direction(&capture.argument),
                    string_value: exact_string(&capture.argument),
                    string_characters: None,
                    closed_int_range: None,
                    list_count: Self::known_list_count(&capture.argument, call_environment),
                    list_string_keys: Self::known_list_string_keys(
                        &capture.argument,
                        call_environment,
                    ),
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
            lowered_parameters.push(CompilerParameter {
                name: capture.parameter_name,
                discarded: false,
                source_visible: false,
                value_type: capture.value_type,
                int_range: capture.int_range,
                span: capture.span,
            });
        }

        for (name, capture) in captures {
            if !capture.runtime_bound
                || !compiler_function_result_supported(&capture.value_type)
                || compiler_type_contains_generator(&capture.value_type)
                || compiler_type_is_function_aggregate(&capture.value_type)
            {
                return Err(unsupported(
                    &self.source,
                    declaration_span,
                    &format!(
                        "anonymous Function capture `{name}` without an admitted private representation"
                    ),
                ));
            }
            let current = binding_facts_by_storage(call_environment, &capture.storage_name)
                .cloned()
                .or_else(|| {
                    is_function_result_capture_storage(&capture.storage_name)
                        .then(|| capture.clone())
                })
                .filter(|current| {
                    current.origin == capture.origin
                        && current.runtime_bound
                        && current.value_type == capture.value_type
                })
                .ok_or_else(|| {
                    unsupported(
                        &self.source,
                        declaration_span,
                        &format!(
                            "anonymous Function capture `{name}` outside its defining invocation"
                        ),
                    )
                })?;
            arguments.push(CompilerExpression {
                kind: current.infinity_negative.map_or_else(
                    || CompilerExpressionKind::Local(current.storage_name.clone()),
                    |negative| CompilerExpressionKind::InfinityLocal {
                        storage_name: current.storage_name.clone(),
                        negative,
                    },
                ),
                value_type: current.value_type.clone(),
                int_range: current.int_range.clone(),
                rational_value: current.rational_value.clone(),
                span: declaration_span,
            });

            let mut capture_parameter_facts = capture.clone();
            capture_parameter_facts.storage_name.clone_from(name);
            environment.insert(name.clone(), capture_parameter_facts);
            lowered_parameters.push(CompilerParameter {
                name: name.clone(),
                discarded: false,
                source_visible: true,
                value_type: capture.value_type.clone(),
                int_range: capture.int_range.clone(),
                span: declaration_span,
            });
        }

        let previous_static_context = self.static_context;
        let previous_in_function = self.in_function;
        self.static_context = static_context;
        self.in_function = true;
        let analyzed_body = match body {
            Expression::Block { statements, .. } => {
                self.analyze_block(statements, &mut environment, BlockKind::Function, None)
            }
            expression => self
                .analyze_expression(expression, &environment)
                .map(|result| CompilerBlock {
                    statements: Vec::new(),
                    result,
                }),
        };
        self.static_context = previous_static_context;
        self.in_function = previous_in_function;
        let analyzed_body = analyzed_body?;
        if !compiler_function_result_supported(&analyzed_body.result.value_type) {
            return Err(unsupported(
                &self.source,
                analyzed_body.result.span,
                "unsupported inferred anonymous-function result",
            ));
        }

        let source_name = format!("<anonymous fn/{}>", parameters.len());
        let symbol = self.reserve_function_symbol("anonymous");
        let result_type = analyzed_body.result.value_type.clone();
        let int_range = analyzed_body.result.int_range.clone();
        let rational_value = analyzed_body.result.rational_value.clone();
        self.instances.push(CompilerFunction {
            source_name,
            module_identity: None,
            symbol: symbol.clone(),
            parameters: lowered_parameters,
            pattern_identities,
            result_type: result_type.clone(),
            result_captures: Vec::new(),
            body: analyzed_body,
            span: declaration_span,
            is_static: static_context,
            declared_effects: None,
            foreign: None,
        });
        let call = CompilerExpression {
            kind: CompilerExpressionKind::Call { symbol, arguments },
            value_type: result_type.clone(),
            int_range: int_range.clone(),
            rational_value: rational_value.clone(),
            span: call_span,
        };
        Ok(if let Some((storage_name, value)) = private_argument {
            CompilerExpression {
                kind: CompilerExpressionKind::PrivateBinding {
                    storage_name,
                    value: Box::new(value),
                    body: Box::new(call),
                },
                value_type: result_type,
                int_range,
                rational_value,
                span: call_span,
            }
        } else {
            call
        })
    }

    #[allow(clippy::too_many_lines)] // Numeric coercion and fail-closed obligations stay in one selection path.
    fn analyze_symbolic_binary(
        &mut self,
        kind: CallableKind,
        left: &Expression,
        right: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let operation = match kind {
            CallableKind::Plus => CompilerBinary::Add,
            CallableKind::Minus => CompilerBinary::Subtract,
            CallableKind::Multiply => CompilerBinary::Multiply,
            CallableKind::Divide => CompilerBinary::Divide,
            CallableKind::Modulo => CompilerBinary::Modulo,
            CallableKind::QuotientModulo => CompilerBinary::QuotientModulo,
            CallableKind::Power => CompilerBinary::Power,
            CallableKind::Compare => CompilerBinary::Compare,
            CallableKind::Range => CompilerBinary::Range,
            CallableKind::RangeOpen => CompilerBinary::RangeOpen,
            CallableKind::RangeInclusive => CompilerBinary::RangeInclusive,
            CallableKind::RangeOpenInclusive => CompilerBinary::RangeOpenInclusive,
            CallableKind::Equal => CompilerBinary::Equal,
            CallableKind::NotEqual => CompilerBinary::NotEqual,
            CallableKind::Less => CompilerBinary::Less,
            CallableKind::Greater => CompilerBinary::Greater,
            CallableKind::LessEqual => CompilerBinary::LessEqual,
            CallableKind::GreaterEqual => CompilerBinary::GreaterEqual,
        };
        let mut left_value = self.analyze_expression(left, environment)?;
        let mut right_value = self.analyze_expression(right, environment)?;
        if matches!(left_value.value_type, CompilerType::Refined { .. }) {
            left_value = forget_refined_evidence(left_value);
        }
        if matches!(right_value.value_type, CompilerType::Refined { .. }) {
            right_value = forget_refined_evidence(right_value);
        }

        if matches!(left_value.value_type, CompilerType::Modular(_))
            || matches!(right_value.value_type, CompilerType::Modular(_))
        {
            return self.finish_modular_binary(operation, left_value, right_value, span);
        }

        if is_range_construction(operation) {
            if left_value.value_type == CompilerType::Nat {
                left_value = forget_nat_evidence(left_value);
            }
            if right_value.value_type == CompilerType::Nat {
                right_value = forget_nat_evidence(right_value);
            }
            require_exact_numeric(&self.source, left_value.span, &left_value.value_type)?;
            require_exact_numeric(&self.source, right_value.span, &right_value.value_type)?;
            let infinite = matches!(
                left_value.value_type,
                CompilerType::InfiniteInt
                    | CompilerType::InfiniteNat
                    | CompilerType::InfiniteRational
            ) || matches!(
                right_value.value_type,
                CompilerType::InfiniteInt
                    | CompilerType::InfiniteNat
                    | CompilerType::InfiniteRational
            );
            let rational = matches!(
                left_value.value_type,
                CompilerType::Rational | CompilerType::InfiniteRational
            ) || matches!(
                right_value.value_type,
                CompilerType::Rational | CompilerType::InfiniteRational
            );
            if rational
                && (matches!(
                    left_value.value_type,
                    CompilerType::InfiniteInt | CompilerType::InfiniteNat
                ) || matches!(
                    right_value.value_type,
                    CompilerType::InfiniteInt | CompilerType::InfiniteNat
                ))
            {
                return Err(unsupported(
                    &self.source,
                    span,
                    "cross-domain Int/Rational infinity range endpoint conversion",
                ));
            }
            let endpoint = if rational {
                left_value = into_rational(left_value);
                right_value = into_rational(right_value);
                if infinite {
                    CompilerType::InfiniteRational
                } else {
                    CompilerType::Rational
                }
            } else if infinite {
                left_value = forget_nat_evidence(left_value);
                right_value = forget_nat_evidence(right_value);
                CompilerType::InfiniteInt
            } else {
                CompilerType::Int
            };
            return Ok(Self::finish_binary(
                operation,
                left_value,
                right_value,
                CompilerType::Range(Box::new(endpoint)),
                span,
            ));
        }

        let infinite = matches!(
            left_value.value_type,
            CompilerType::InfiniteInt | CompilerType::InfiniteNat | CompilerType::InfiniteRational
        ) || matches!(
            right_value.value_type,
            CompilerType::InfiniteInt | CompilerType::InfiniteNat | CompilerType::InfiniteRational
        );
        let comparison = matches!(
            operation,
            CompilerBinary::Equal
                | CompilerBinary::NotEqual
                | CompilerBinary::Less
                | CompilerBinary::Greater
                | CompilerBinary::LessEqual
                | CompilerBinary::GreaterEqual
                | CompilerBinary::Compare
        );
        let rational_domain = matches!(
            left_value.value_type,
            CompilerType::Rational | CompilerType::InfiniteRational
        ) || matches!(
            right_value.value_type,
            CompilerType::Rational | CompilerType::InfiniteRational
        );
        if comparison
            && rational_domain
            && (matches!(
                left_value.value_type,
                CompilerType::InfiniteInt | CompilerType::InfiniteNat
            ) || matches!(
                right_value.value_type,
                CompilerType::InfiniteInt | CompilerType::InfiniteNat
            ))
        {
            return Err(unsupported(
                &self.source,
                span,
                "cross-domain Int/Rational infinity comparison",
            ));
        }
        if infinite && !comparison {
            if !matches!(
                operation,
                CompilerBinary::Add | CompilerBinary::Subtract | CompilerBinary::Multiply
            ) {
                return Err(unsupported(
                    &self.source,
                    span,
                    "infinity arithmetic outside exact +, -, and *",
                ));
            }
            if rational_domain
                && (matches!(
                    left_value.value_type,
                    CompilerType::InfiniteInt | CompilerType::InfiniteNat
                ) || matches!(
                    right_value.value_type,
                    CompilerType::InfiniteInt | CompilerType::InfiniteNat
                ))
            {
                return Err(unsupported(
                    &self.source,
                    span,
                    "cross-domain Int/Rational infinity arithmetic",
                ));
            }
            let outcome = compiler_infinity_binary_outcome(operation, &left_value, &right_value);
            match outcome {
                CompilerInfinityArithmeticOutcome::Indeterminate => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-INDETERMINATE-INFINITY",
                        span,
                        "this infinity arithmetic expression does not determine one exact numeric value",
                    ));
                }
                CompilerInfinityArithmeticOutcome::NeedsDynamicResult => {
                    let error_span = if compiler_infinity_direction(&left_value).is_none() {
                        left_value.span
                    } else {
                        right_value.span
                    };
                    let success_type = if rational_domain {
                        left_value = into_rational(left_value);
                        right_value = into_rational(right_value);
                        CompilerType::Rational
                    } else {
                        left_value = forget_nat_evidence(left_value);
                        right_value = forget_nat_evidence(right_value);
                        CompilerType::Int
                    };
                    return Ok(Self::finish_fallible_binary(
                        CompilerFallible::InfinityMultiply,
                        left_value,
                        right_value,
                        success_type,
                        span,
                        error_span,
                    ));
                }
                CompilerInfinityArithmeticOutcome::Direction(_) => {}
            }
            let result_type = if rational_domain {
                left_value = into_rational(left_value);
                right_value = into_rational(right_value);
                CompilerType::InfiniteRational
            } else {
                left_value = forget_nat_evidence(left_value);
                right_value = forget_nat_evidence(right_value);
                CompilerType::InfiniteInt
            };
            let result = Self::finish_binary(operation, left_value, right_value, result_type, span);
            debug_assert!(compiler_infinity_direction(&result).is_some());
            return Ok(result);
        }

        if matches!(
            operation,
            CompilerBinary::Modulo | CompilerBinary::QuotientModulo
        ) {
            left_value = forget_nat_evidence(left_value);
            right_value = forget_nat_evidence(right_value);
            require_type(
                &self.source,
                left_value.span,
                &CompilerType::Int,
                &left_value.value_type,
            )?;
            require_type(
                &self.source,
                right_value.span,
                &CompilerType::Int,
                &right_value.value_type,
            )?;
            let result_type = if operation == CompilerBinary::Modulo {
                CompilerType::Int
            } else {
                CompilerType::Tuple(vec![CompilerType::Int, CompilerType::Int])
            };
            let active_nonzero = matches!(&right_value.kind, CompilerExpressionKind::Local(name)
                if self.active_nonzero_bindings.contains(name));
            if active_nonzero && is_proven_zero_numeric(&right_value) {
                right_value.int_range = None;
            }
            if is_proven_zero_numeric(&right_value) && !active_nonzero {
                if compiler_expression_is_closed(&right_value) {
                    return Err(division_by_zero(&self.source, right_value.span));
                }
                return Ok(Self::finish_fallible_binary(
                    if operation == CompilerBinary::Modulo {
                        CompilerFallible::IntModulo
                    } else {
                        CompilerFallible::IntQuotientModulo
                    },
                    left_value,
                    right_value,
                    result_type,
                    span,
                    right.span(),
                ));
            }
            if !is_proven_nonzero_numeric(&right_value) && !active_nonzero {
                return Ok(Self::finish_fallible_binary(
                    if operation == CompilerBinary::Modulo {
                        CompilerFallible::IntModulo
                    } else {
                        CompilerFallible::IntQuotientModulo
                    },
                    left_value,
                    right_value,
                    result_type,
                    span,
                    right.span(),
                ));
            }
            return Ok(Self::finish_binary(
                operation,
                left_value,
                right_value,
                result_type,
                span,
            ));
        }

        if operation == CompilerBinary::Power {
            return self.finish_power(left_value, right_value, span);
        }

        if matches!(operation, CompilerBinary::Equal | CompilerBinary::NotEqual)
            && (matches!(
                left_value.value_type,
                CompilerType::Tuple(_) | CompilerType::Record(_)
            ) || matches!(
                right_value.value_type,
                CompilerType::Tuple(_) | CompilerType::Record(_)
            ))
        {
            let (left_value, right_value, value_type) =
                self.adapt_structural_equality(left_value, right_value, span)?;
            if !compiler_equality_supported(&value_type) {
                return Err(unsupported(
                    &self.source,
                    span,
                    "equality for this structural value type",
                ));
            }
            return Ok(Self::finish_binary(
                operation,
                left_value,
                right_value,
                CompilerType::Boolean,
                span,
            ));
        }

        if matches!(
            operation,
            CompilerBinary::Less
                | CompilerBinary::Greater
                | CompilerBinary::LessEqual
                | CompilerBinary::GreaterEqual
                | CompilerBinary::Compare
        ) && (matches!(left_value.value_type, CompilerType::Tuple(_))
            || matches!(right_value.value_type, CompilerType::Tuple(_)))
        {
            let (left_value, right_value, value_type) =
                self.adapt_tuple_ordering(left_value, right_value, span)?;
            if !compiler_ordering_supported(&value_type) {
                return Err(unsupported(
                    &self.source,
                    span,
                    "ordering for this structural value type",
                ));
            }
            let result_type = if operation == CompilerBinary::Compare {
                CompilerType::Comparison
            } else {
                CompilerType::Boolean
            };
            return Ok(Self::finish_binary(
                operation,
                left_value,
                right_value,
                result_type,
                span,
            ));
        }

        if comparison
            && is_exact_comparable(&left_value.value_type)
            && is_exact_comparable(&right_value.value_type)
        {
            left_value = forget_nat_evidence(left_value);
            right_value = forget_nat_evidence(right_value);
        }
        if matches!(operation, CompilerBinary::Equal | CompilerBinary::NotEqual) {
            if left_value.value_type == CompilerType::Character
                && right_value.value_type == CompilerType::String
            {
                left_value = forget_character_evidence(left_value);
            } else if left_value.value_type == CompilerType::String
                && right_value.value_type == CompilerType::Character
            {
                right_value = forget_character_evidence(right_value);
            }
        }
        if self.active_nat_recursion()
            && matches!(operation, CompilerBinary::Add | CompilerBinary::Subtract)
        {
            left_value = forget_nat_evidence(left_value);
            right_value = forget_nat_evidence(right_value);
        }
        if matches!(operation, CompilerBinary::Add | CompilerBinary::Multiply) {
            if left_value.value_type == CompilerType::Nat
                && right_value.value_type == CompilerType::Int
                && right_value
                    .int_range
                    .as_ref()
                    .is_some_and(|range| range.lower >= BigInt::from(0))
            {
                right_value = CompilerExpression {
                    kind: CompilerExpressionKind::IntToNat(Box::new(right_value)),
                    value_type: CompilerType::Nat,
                    int_range: None,
                    rational_value: None,
                    span,
                };
            } else if right_value.value_type == CompilerType::Nat
                && left_value.value_type == CompilerType::Int
                && left_value
                    .int_range
                    .as_ref()
                    .is_some_and(|range| range.lower >= BigInt::from(0))
            {
                left_value = CompilerExpression {
                    kind: CompilerExpressionKind::IntToNat(Box::new(left_value)),
                    value_type: CompilerType::Nat,
                    int_range: None,
                    rational_value: None,
                    span,
                };
            }
        }
        if operation == CompilerBinary::Subtract
            && left_value.value_type == CompilerType::Nat
            && right_value.value_type == CompilerType::Int
            && exact_int(&right_value).is_some_and(|amount| {
                amount >= BigInt::from(0)
                    && (matches!(&left_value.kind, CompilerExpressionKind::Local(name)
                        if self.active_lower_bounds.get(name).is_some_and(|bound| bound >= &amount))
                        || (amount == BigInt::from(1)
                            && self.compiler_proven_nonzero_expression(&left_value)))
            })
        {
            let int_range = right_value.int_range.clone();
            right_value = CompilerExpression {
                kind: CompilerExpressionKind::IntToNat(Box::new(right_value)),
                value_type: CompilerType::Nat,
                int_range,
                rational_value: None,
                span,
            };
        }
        let both_nat = left_value.value_type == CompilerType::Nat
            && right_value.value_type == CompilerType::Nat;
        let nat_arithmetic_result = both_nat
            && match operation {
                CompilerBinary::Add | CompilerBinary::Multiply => true,
                CompilerBinary::Subtract => {
                    left_value
                        .int_range
                        .as_ref()
                        .zip(right_value.int_range.as_ref())
                        .is_some_and(|(left, right)| left.lower >= right.upper)
                        || exact_int(&right_value).is_some_and(|amount| {
                            matches!(&left_value.kind, CompilerExpressionKind::Local(name)
                                if self.active_lower_bounds.get(name).is_some_and(|bound| bound >= &amount))
                                || (amount == BigInt::from(1)
                                    && self.compiler_proven_nonzero_expression(&left_value))
                        })
                        // The private regex expansion helper is reached only
                        // after repeat-bound validation has established
                        // `maximum >= minimum`.
                        || self.active_calls.last().is_some_and(|identity| {
                            identity.contains("regex-expand-finite-repeat")
                        }) && matches!(
                            (&left_value.kind, &right_value.kind),
                            (
                                CompilerExpressionKind::Local(left),
                                CompilerExpressionKind::Local(right)
                            ) if left == "maximum" && right == "minimum"
                        )
                }
                _ => false,
            };
        if matches!(
            operation,
            CompilerBinary::Add | CompilerBinary::Subtract | CompilerBinary::Multiply
        ) {
            left_value = forget_nat_evidence(left_value);
            right_value = forget_nat_evidence(right_value);
        }
        let numeric =
            is_exact_numeric(&left_value.value_type) && is_exact_numeric(&right_value.value_type);
        if matches!(operation, CompilerBinary::Equal | CompilerBinary::NotEqual) && !numeric {
            require_same_type(
                &self.source,
                span,
                &left_value.value_type,
                &right_value.value_type,
            )?;
            if !compiler_equality_supported(&left_value.value_type) {
                return Err(unsupported(
                    &self.source,
                    span,
                    "equality for this value type",
                ));
            }
            return Ok(Self::finish_binary(
                operation,
                left_value,
                right_value,
                CompilerType::Boolean,
                span,
            ));
        }

        require_exact_numeric(&self.source, left_value.span, &left_value.value_type)?;
        require_exact_numeric(&self.source, right_value.span, &right_value.value_type)?;
        let both_int = left_value.value_type == CompilerType::Int
            && right_value.value_type == CompilerType::Int;
        let active_nonzero = numeric_local_storage(&right_value)
            .is_some_and(|name| self.active_nonzero_bindings.contains(name));
        let rational_result = operation == CompilerBinary::Divide
            || matches!(
                left_value.value_type,
                CompilerType::Rational | CompilerType::InfiniteRational
            )
            || matches!(
                right_value.value_type,
                CompilerType::Rational | CompilerType::InfiniteRational
            );
        if rational_result {
            left_value = into_rational(left_value);
            right_value = into_rational(right_value);
        }
        if operation == CompilerBinary::Divide
            && is_proven_zero_numeric(&right_value)
            && !active_nonzero
        {
            if compiler_expression_is_closed(&right_value) {
                return Err(division_by_zero(&self.source, right_value.span));
            }
            if both_int {
                return Err(unsupported(
                    &self.source,
                    span,
                    "dynamic Int division Result",
                ));
            }
            return Ok(Self::finish_fallible_binary(
                CompilerFallible::RationalDivide,
                left_value,
                right_value,
                CompilerType::Rational,
                span,
                right.span(),
            ));
        }
        if operation == CompilerBinary::Divide
            && !is_proven_nonzero_numeric(&right_value)
            && !active_nonzero
        {
            if both_int {
                return Err(unsupported(
                    &self.source,
                    span,
                    "dynamic Int division Result",
                ));
            }
            return Ok(Self::finish_fallible_binary(
                CompilerFallible::RationalDivide,
                left_value,
                right_value,
                CompilerType::Rational,
                span,
                right.span(),
            ));
        }
        let result_type = match operation {
            CompilerBinary::Add | CompilerBinary::Subtract | CompilerBinary::Multiply => {
                if rational_result {
                    CompilerType::Rational
                } else if nat_arithmetic_result {
                    CompilerType::Nat
                } else {
                    CompilerType::Int
                }
            }
            CompilerBinary::Divide => CompilerType::Rational,
            CompilerBinary::Compare => CompilerType::Comparison,
            CompilerBinary::Equal
            | CompilerBinary::NotEqual
            | CompilerBinary::Less
            | CompilerBinary::Greater
            | CompilerBinary::LessEqual
            | CompilerBinary::GreaterEqual => CompilerType::Boolean,
            _ => unreachable!("numeric operation handled above"),
        };
        debug_assert!(!both_int || !rational_result || operation == CompilerBinary::Divide);
        Ok(Self::finish_binary(
            operation,
            left_value,
            right_value,
            result_type,
            span,
        ))
    }
}
