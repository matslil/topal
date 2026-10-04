impl Analyzer {
    #[allow(clippy::too_many_arguments, clippy::too_many_lines)] // Candidate-specific source packages and captures remain visibly fail-closed.
    fn analyze_resolved_call_from(
        &mut self,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
        function_index: usize,
        function_name: &str,
        declarations: &[FunctionSource],
        lexical_captures: &[CompilerContextCapture],
        named_environment: bool,
    ) -> Result<CompilerExpression, Diagnostic> {
        let argument_sources = items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| (index != function_index).then_some(item))
            .collect::<Vec<_>>();
        let arguments = argument_sources
            .iter()
            .map(|argument| {
                if function_name.contains('.')
                    && let Expression::Identifier(name) = argument
                    && let Some(facts) = environment.get(self.source.slice(*name))
                    && facts.string_value.is_some()
                {
                    return Ok(binding_expression(facts, argument.span()));
                }
                self.analyze_expression(argument, environment)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut flattened_arguments = flattened_product_arguments(&argument_sources, &arguments);
        if let (Some(flattened), [argument]) = (&mut flattened_arguments, arguments.as_slice())
            && let Ok(facts) = self.known_structural_value_facts(argument, environment)
            && facts.tuple_fields.len() == flattened.len()
        {
            for (field, facts) in flattened.iter_mut().zip(facts.tuple_fields) {
                field.int_range = facts.int_range;
                field.rational_value = facts.rational_value;
            }
        }

        let mut selected = None;
        let static_context = self.static_context;
        for declaration in declarations
            .iter()
            .filter(|declaration| !static_context || declaration.is_static)
        {
            if declaration
                .parameters
                .iter()
                .any(|parameter| !parameter.fields.is_empty())
            {
                let normalized = if declaration.parameters.len() == 1 {
                    self.normalize_packaged_call(declaration, &arguments, environment)?
                } else {
                    self.normalize_compound_packaged_call(
                        declaration,
                        flattened_arguments.as_deref().unwrap_or(&arguments),
                        environment,
                    )?
                };
                if let Some((normalized, adapted, bindings, substitutions)) = normalized {
                    selected = Some((normalized, adapted, bindings, substitutions));
                    break;
                }
                continue;
            }
            let candidate_arguments = if declaration.parameters.is_empty()
                && matches!(argument_sources.as_slice(), [Expression::Unit(_)])
            {
                &[][..]
            } else if declaration.parameters.len() > 1
                && let Some(flattened) = &flattened_arguments
            {
                flattened.as_slice()
            } else {
                arguments.as_slice()
            };
            if candidate_arguments.len() != declaration.parameters.len() {
                continue;
            }
            let mut adapted = Vec::with_capacity(candidate_arguments.len());
            let mut fact_dependent = false;
            let mut classifier_substitutions = self.classifier_substitutions.clone();
            classifier_substitutions.retain(|name, _| {
                !declaration.parameters.iter().any(|parameter| {
                    compact_classifier(self.source.slice(parameter.classifier))
                        .contains(&format!("({name}:"))
                })
            });
            for (parameter_index, (parameter, argument)) in declaration
                .parameters
                .iter()
                .zip(candidate_arguments)
                .enumerate()
            {
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
                let Some(expected) = self.specialize_classifier(
                    parameter.classifier,
                    &argument.value_type,
                    &mut classifier_substitutions,
                )?
                else {
                    adapted.clear();
                    break;
                };
                if expected != argument.value_type
                    && capture_argument_adaptation_may_depend_on_facts(&expected, argument)
                {
                    fact_dependent = true;
                }
                let Some(argument) = adapt_function_call_argument(&expected, argument)
                    .or_else(|| self.adapt_proven_nonnegative_nat_argument(&expected, argument))
                    .or_else(|| {
                        self.adapt_proven_recursive_nat_argument(
                            function_name,
                            declaration,
                            parameter_index,
                            &expected,
                            argument,
                        )
                    })
                else {
                    adapted.clear();
                    break;
                };
                adapted.push(argument);
            }
            if adapted.len() == declaration.parameters.len() {
                if named_environment && declarations.len() != 1 && fact_dependent {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "value-fact-dependent named Function environment selection",
                    ));
                }
                selected = Some((
                    declaration.clone(),
                    adapted,
                    Vec::new(),
                    classifier_substitutions,
                ));
                break;
            }
        }
        let Some((mut declaration, arguments, argument_bindings, mut classifier_substitutions)) =
            selected
        else {
            let actual = arguments
                .iter()
                .map(|argument| argument.value_type.name())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(source_diagnostic(
                &self.source,
                "E-NO-APPLICABLE-OVERLOAD",
                span,
                format!("no overload of `{function_name}` accepts ({actual}) in this context"),
            ));
        };
        for (parameter, argument) in declaration.parameters.iter().zip(&arguments) {
            let classifier = compact_classifier(self.source.slice(parameter.classifier));
            let Some((input_classifier, output_classifier)) =
                split_compact_function_classifier(&classifier)
            else {
                continue;
            };
            let Some(callable) = self.known_callable(argument, environment, argument.span.start)?
            else {
                continue;
            };
            let Some(input_type) =
                parse_substituted_classifier(input_classifier, &classifier_substitutions, &|_| {
                    None
                })
            else {
                continue;
            };
            let result_type = match callable {
                CompilerCallableFacts::Anonymous {
                    parameters,
                    body,
                    captures,
                    static_context,
                    span: function_span,
                } => {
                    let mut callable_environment = environment.clone();
                    callable_environment.extend(captures);
                    let (_, analyzed) = self.analyze_collection_function(
                        &parameters,
                        &body,
                        &[input_type],
                        &callable_environment,
                        static_context,
                        function_span,
                    )?;
                    analyzed.result.value_type
                }
                CompilerCallableFacts::Named { declarations, .. } => {
                    let mut inferred = None;
                    for callable in declarations {
                        let [callable_parameter] = callable.parameters.as_slice() else {
                            continue;
                        };
                        let mut nested_substitutions = classifier_substitutions.clone();
                        let parameter_classifier =
                            compact_classifier(self.source.slice(callable_parameter.classifier));
                        if !infer_classifier_substitutions(
                            &parameter_classifier,
                            &input_type,
                            &mut nested_substitutions,
                            &|_| None,
                        ) {
                            continue;
                        }
                        let result_classifier =
                            compact_classifier(self.source.slice(callable.result));
                        if let Some(result) = parse_substituted_classifier(
                            &result_classifier,
                            &nested_substitutions,
                            &|_| None,
                        ) {
                            inferred = Some(result);
                            break;
                        }
                    }
                    let Some(result) = inferred else {
                        continue;
                    };
                    result
                }
                CompilerCallableFacts::Symbolic(_) => continue,
            };
            let inferred = if !classifier_substitutions.contains_key(output_classifier)
                && parse_compact_classifier(output_classifier).is_none()
                && !["List", "Optional", "Range", "Set", "Bag", "Result("]
                    .iter()
                    .any(|prefix| output_classifier.starts_with(prefix))
                && output_classifier
                    .chars()
                    .all(|character| character == '-' || character.is_alphanumeric())
            {
                classifier_substitutions.insert(output_classifier.to_owned(), result_type.clone());
                true
            } else {
                infer_classifier_substitutions(
                    output_classifier,
                    &result_type,
                    &mut classifier_substitutions,
                    &|_| None,
                )
            };
            if !inferred {
                return Err(source_diagnostic(
                    &self.source,
                    "E-NO-APPLICABLE-OVERLOAD",
                    argument.span,
                    format!(
                        "function argument result {} does not satisfy `{output_classifier}`",
                        result_type.name()
                    ),
                ));
            }
        }
        if function_name == "std.nfc"
            && let [text] = arguments.as_slice()
            && let Some(text) = exact_string(text)
        {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::String(normalize_nfc(&text)),
                value_type: CompilerType::String,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if matches!(
            function_name,
            "std.canonical-equal"
                | "std.caseless-equal"
                | "std.starts-with?"
                | "std.ends-with?"
                | "std.contains?"
        ) && let [left, right] = arguments.as_slice()
            && let (Some(left), Some(right)) = (exact_string(left), exact_string(right))
        {
            let value = match function_name {
                "std.canonical-equal" => canonically_equal(&left, &right),
                "std.caseless-equal" => case_fold(&left) == case_fold(&right),
                "std.starts-with?" => left.starts_with(&right),
                "std.ends-with?" => left.ends_with(&right),
                "std.contains?" => left.contains(&right),
                _ => unreachable!(),
            };
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::Boolean(value),
                value_type: CompilerType::Boolean,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if function_name == "std.trim"
            && let [text] = arguments.as_slice()
            && let Some(text) = exact_string(text)
        {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::String(text.trim_matches(char::is_whitespace).into()),
                value_type: CompilerType::String,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if function_name == "std.replace-all"
            && let [text, pattern, replacement] = arguments.as_slice()
            && let (Some(text), Some(pattern), Some(replacement)) = (
                exact_string(text),
                exact_string(pattern),
                exact_string(replacement),
            )
            && !pattern.is_empty()
        {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::String(text.replace(&pattern, &replacement)),
                value_type: CompilerType::String,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if function_name == "std.repeat"
            && let [text, count] = arguments.as_slice()
            && let Some(text) = exact_string(text)
            && let Some(count) = Self::exact_usize(count)
        {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::String(text.repeat(count)),
                value_type: CompilerType::String,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if function_name == "std.parse.decimal"
            && let [argument] = arguments.as_slice()
            && let Some(value) = exact_int(argument)
        {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::String(value.to_string()),
                value_type: CompilerType::String,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if matches!(
            function_name,
            "std.sequence.take" | "std.sequence.drop" | "std.sequence.split-at"
        ) && let [text, boundary] = arguments.as_slice()
            && let Some(text) = exact_string(text)
            && let Some(boundary) = Self::exact_usize(boundary)
        {
            let characters = characters(&text).collect::<Vec<_>>();
            let boundary = boundary.min(characters.len());
            let left = characters[..boundary].concat();
            let right = characters[boundary..].concat();
            let value = match function_name {
                "std.sequence.take" => CompilerExpression {
                    kind: CompilerExpressionKind::String(left),
                    value_type: CompilerType::String,
                    int_range: None,
                    rational_value: None,
                    span,
                },
                "std.sequence.drop" => CompilerExpression {
                    kind: CompilerExpressionKind::String(right),
                    value_type: CompilerType::String,
                    int_range: None,
                    rational_value: None,
                    span,
                },
                "std.sequence.split-at" => CompilerExpression {
                    kind: CompilerExpressionKind::Tuple(vec![
                        CompilerExpression {
                            kind: CompilerExpressionKind::String(left),
                            value_type: CompilerType::String,
                            int_range: None,
                            rational_value: None,
                            span,
                        },
                        CompilerExpression {
                            kind: CompilerExpressionKind::String(right),
                            value_type: CompilerType::String,
                            int_range: None,
                            rational_value: None,
                            span,
                        },
                    ]),
                    value_type: CompilerType::Tuple(vec![
                        CompilerType::String,
                        CompilerType::String,
                    ]),
                    int_range: None,
                    rational_value: None,
                    span,
                },
                _ => unreachable!(),
            };
            return Ok(value);
        }
        if matches!(
            function_name,
            "std.sequence.chunks" | "std.sequence.windows"
        ) && let [values, size] = arguments.as_slice()
            && values.value_type == int_list_type()
            && let Some(size) = Self::exact_usize(size).filter(|size| *size > 0)
            && let Some(entries) =
                exact_int_entries(&self.known_structural_value_facts(values, environment)?)
        {
            let groups = if function_name == "std.sequence.chunks" {
                entries
                    .chunks(size)
                    .map(<[BigInt]>::to_vec)
                    .collect::<Vec<_>>()
            } else if size > entries.len() {
                Vec::new()
            } else {
                entries
                    .windows(size)
                    .map(<[BigInt]>::to_vec)
                    .collect::<Vec<_>>()
            };
            return Ok(compiler_nested_int_list_literal(&groups, span));
        }
        if function_name == "std.sequence.values"
            && let [range] = arguments.as_slice()
            && let Some(range) = Self::known_closed_int_range(range, environment)
            && let Some(values) = exact_closed_range_values(&range, 10_000)
        {
            return Ok(compiler_int_list_literal(&values, span));
        }
        if function_name == "std.sequence.transpose"
            && let [rows] = arguments.as_slice()
            && rows.value_type
                == CompilerType::List(Box::new(CompilerType::List(Box::new(CompilerType::Int))))
            && let Some(rows) =
                exact_nested_int_entries(&self.known_structural_value_facts(rows, environment)?)
        {
            let width = rows.iter().map(Vec::len).min().unwrap_or(0);
            let columns = (0..width)
                .map(|index| rows.iter().map(|row| row[index].clone()).collect())
                .collect::<Vec<Vec<BigInt>>>();
            return Ok(compiler_nested_int_list_literal(&columns, span));
        }
        if matches!(
            function_name,
            "std.ordered.sort" | "std.ordered.sort-descending"
        ) && let [values] = arguments.as_slice()
        {
            let facts = self.known_structural_value_facts(values, environment)?;
            if values.value_type == int_list_type()
                && let Some(mut entries) = exact_int_entries(&facts)
            {
                entries.sort();
                if function_name.ends_with("sort-descending") {
                    entries.reverse();
                }
                return Ok(compiler_int_list_literal(&entries, span));
            }
            if values.value_type == CompilerType::List(Box::new(CompilerType::Rational))
                && let Some(mut entries) = exact_rational_entries(&facts)
            {
                entries.sort();
                if function_name.ends_with("sort-descending") {
                    entries.reverse();
                }
                return Ok(compiler_rational_list_literal(&entries, span));
            }
        }
        if function_name.starts_with("std.ordered.")
            && let Some(value) =
                self.specialize_exact_ordered_call(function_name, &arguments, environment, span)?
        {
            return Ok(value);
        }
        if let [argument] = arguments.as_slice()
            && let Some(text) = exact_string(argument)
        {
            let rows = || {
                text.lines()
                    .map(|line| {
                        line.split_whitespace()
                            .filter_map(|field| field.parse::<BigInt>().ok())
                            .collect::<Vec<_>>()
                    })
                    .filter(|row| !row.is_empty())
                    .collect::<Vec<_>>()
            };
            let specialized = match function_name {
                "std.parse.integer-rows" => Some(compiler_int_rows_literal(rows(), span)),
                "std.parse.integer-pairs" => Some(compiler_int_tuple_list_literal(
                    rows().into_iter().filter(|row| row.len() == 2).collect(),
                    2,
                    span,
                )),
                "std.parse.integer-triples" => Some(compiler_int_tuple_list_literal(
                    rows().into_iter().filter(|row| row.len() == 3).collect(),
                    3,
                    span,
                )),
                "std.parse.vertical-integers" => Some(compiler_vertical_ints_literal(&text, span)),
                _ => None,
            };
            if let Some(value) = specialized {
                return Ok(value);
            }
        }
        if declaration.module_identity.is_some() {
            declaration.declared_effects =
                compiler_declared_effect_row(&self.source, declaration.effect_bound)?;
        }
        let previous_classifier_substitutions =
            std::mem::replace(&mut self.classifier_substitutions, classifier_substitutions);
        let mut call_environment = environment.clone();
        for binding in &argument_bindings {
            if binding.value.value_type == CompilerType::Scope {
                if self.in_function && matches!(binding.value.kind, CompilerExpressionKind::Root) {
                    return Err(unsupported(
                        &self.source,
                        binding.value.span,
                        "function-body live root Scope argument",
                    ));
                }
                let namespace = self
                    .resolve_namespace(&binding.value, environment, binding.value.span.start)?
                    .ok_or_else(|| {
                        unsupported(
                            &self.source,
                            binding.value.span,
                            "opaque Scope packaged field",
                        )
                    })?;
                call_environment.insert(
                    binding.storage_name.clone(),
                    BindingFacts {
                        storage_name: binding.storage_name.clone(),
                        origin: binding.value.span.start,
                        runtime_bound: true,
                        value_type: CompilerType::Scope,
                        int_range: None,
                        rational_value: None,
                        infinity_negative: None,
                        string_value: None,
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
                        namespace: Some(namespace),
                        callable: None,
                        static_capability: None,
                    },
                );
                continue;
            }
            let facts = self.known_structural_value_facts(&binding.value, environment)?;
            if compiler_type_is_function_aggregate(&binding.value.value_type)
                && !function_aggregate_facts_exact(&binding.value.value_type, &facts)
            {
                return Err(unsupported(
                    &self.source,
                    binding.value.span,
                    "Function aggregate packaged field without one exact callable identity per Function field",
                ));
            }
            call_environment.insert(
                binding.storage_name.clone(),
                BindingFacts {
                    storage_name: binding.storage_name.clone(),
                    origin: binding.value.span.start,
                    runtime_bound: true,
                    value_type: binding.value.value_type.clone(),
                    int_range: binding.value.int_range.clone(),
                    rational_value: binding.value.rational_value.clone(),
                    infinity_negative: None,
                    string_value: facts.string_value,
                    string_characters: facts.string_characters,
                    closed_int_range: None,
                    list_count: None,
                    list_string_keys: None,
                    list_string_characters: facts.list_string_characters,
                    list_entries: facts.list_entries,
                    array_entries: facts.array_entries,
                    map_entries: facts.map_entries,
                    tuple_fields: facts.tuple_fields,
                    record_fields: facts.record_fields,
                    optional: facts.optional,
                    sum: facts.sum,
                    result: facts.result,
                    namespace: None,
                    callable: facts.callable,
                    static_capability: None,
                },
            );
        }
        let callable_arguments = arguments
            .iter()
            .map(|argument| self.known_callable(argument, &call_environment, argument.span.start))
            .collect::<Result<Vec<_>, _>>()?;
        let aggregate_arguments = arguments
            .iter()
            .map(|argument| self.known_structural_value_facts(argument, &call_environment))
            .collect::<Result<Vec<_>, _>>()?;
        for (argument, facts) in arguments.iter().zip(&aggregate_arguments) {
            if compiler_type_is_function_aggregate(&argument.value_type)
                && !function_aggregate_facts_exact(&argument.value_type, facts)
            {
                return Err(unsupported(
                    &self.source,
                    argument.span,
                    "Function aggregate boundary without one exact callable identity per Function field",
                ));
            }
        }
        let (callable_arguments, callable_captures) =
            self.callable_parameter_arguments(&declaration, callable_arguments, &call_environment)?;
        let (aggregate_arguments, aggregate_captures) = self.aggregate_parameter_arguments(
            &declaration,
            &arguments,
            aggregate_arguments,
            &call_environment,
        )?;
        let (scope_arguments, scope_captures) =
            self.scope_parameter_arguments(&declaration, &arguments, &call_environment)?;
        let context_captures = self.defining_context_captures(&declaration, &call_environment)?;
        let metadata = CompilerCallMetadata {
            callable_arguments,
            aggregate_arguments,
            callable_captures,
            aggregate_captures,
            scope_arguments,
            scope_captures,
            lexical_captures: lexical_captures.to_vec(),
            context_captures,
        };
        let mut arguments = arguments;
        arguments.extend(
            metadata
                .callable_captures
                .iter()
                .map(|capture| capture.argument.clone()),
        );
        arguments.extend(
            metadata
                .aggregate_captures
                .iter()
                .map(|capture| capture.argument.clone()),
        );
        arguments.extend(
            metadata
                .scope_captures
                .iter()
                .map(|capture| capture.argument.clone()),
        );
        arguments.extend(
            metadata
                .lexical_captures
                .iter()
                .map(|capture| capture.argument.clone()),
        );
        arguments.extend(
            metadata
                .context_captures
                .iter()
                .map(|capture| capture.argument.clone()),
        );
        let call =
            self.finish_selected_call(function_name, &declaration, arguments, &metadata, span);
        self.classifier_substitutions = previous_classifier_substitutions;
        let mut call = call?;
        for binding in argument_bindings.into_iter().rev() {
            let value_type = call.value_type.clone();
            let int_range = call.int_range.clone();
            let rational_value = call.rational_value.clone();
            call = CompilerExpression {
                kind: CompilerExpressionKind::PrivateBinding {
                    storage_name: binding.storage_name,
                    value: Box::new(binding.value),
                    body: Box::new(call),
                },
                value_type,
                int_range,
                rational_value,
                span,
            };
        }
        Ok(call)
    }

    #[allow(clippy::too_many_lines)] // Every admitted and deferred package shape is checked explicitly.
    fn normalize_packaged_call(
        &mut self,
        declaration: &FunctionSource,
        arguments: &[CompilerExpression],
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<Option<NormalizedPackagedCall>, Diagnostic> {
        let [package] = declaration.parameters.as_slice() else {
            return Err(unsupported(
                &self.source,
                declaration.span,
                "multiple packaged function operands",
            ));
        };
        if package.fields.is_empty() {
            return Ok(None);
        }
        if package.qualifier.is_some() || package.default.is_some() {
            return Err(unsupported(
                &self.source,
                package.name,
                "qualified or defaulted outer package",
            ));
        }
        let mut declared_fields = BTreeSet::new();
        for field in &package.fields {
            let name = self.source.slice(field.name);
            if name != "_" && !declared_fields.insert(name) {
                return Err(source_diagnostic(
                    &self.source,
                    "E-DUPLICATE-FUNCTION-PARAMETER",
                    field.name,
                    format!("parameter `{name}` is already declared in this function"),
                ));
            }
        }
        let [argument] = arguments else {
            return Ok(None);
        };

        let mut argument_bindings = Vec::new();
        let supplied = match &argument.kind {
            CompilerExpressionKind::Record(values) => {
                let declared_names = package
                    .fields
                    .iter()
                    .map(|field| self.source.slice(field.name))
                    .collect::<Vec<_>>();
                let supplied_names = values
                    .iter()
                    .map(|(label, _)| label.as_str())
                    .collect::<BTreeSet<_>>();
                if values
                    .iter()
                    .any(|(label, _)| !declared_names.contains(&label.as_str()))
                    || supplied_names.len() != values.len()
                    || package.fields.iter().any(|field| {
                        field.default.is_none()
                            && !values
                                .iter()
                                .any(|(label, _)| label == self.source.slice(field.name))
                    })
                {
                    return Ok(None);
                }
                let supplied_positions = package
                    .fields
                    .iter()
                    .filter_map(|field| {
                        let name = self.source.slice(field.name);
                        values.iter().position(|(label, _)| label == name)
                    })
                    .collect::<Vec<_>>();
                let reordered = supplied_positions
                    .windows(2)
                    .any(|positions| positions[0] > positions[1]);
                let omitted_before_supplied = package
                    .fields
                    .iter()
                    .rposition(|field| supplied_names.contains(self.source.slice(field.name)))
                    .is_some_and(|last_supplied| {
                        package.fields[..last_supplied]
                            .iter()
                            .any(|field| !supplied_names.contains(self.source.slice(field.name)))
                    });
                let requires_bindings = reordered || omitted_before_supplied;
                values
                    .iter()
                    .enumerate()
                    .map(|(index, (label, value))| {
                        if !requires_bindings {
                            return (label.clone(), value.clone());
                        }
                        let storage_name = format!(
                            "topal.package.argument.{}.{}.{}.{}",
                            argument.span.start,
                            argument.span.end,
                            index,
                            self.next_private_binding
                        );
                        self.next_private_binding += 1;
                        if let Ok(facts) = self.known_structural_value_facts(value, environment) {
                            self.private_binding_static_facts
                                .insert(storage_name.clone(), facts);
                        }
                        let local = CompilerExpression {
                            kind: CompilerExpressionKind::Local(storage_name.clone()),
                            value_type: value.value_type.clone(),
                            int_range: value.int_range.clone(),
                            rational_value: value.rational_value.clone(),
                            span: value.span,
                        };
                        argument_bindings.push(CompilerArgumentBinding {
                            storage_name,
                            value: value.clone(),
                        });
                        (label.clone(), local)
                    })
                    .collect::<BTreeMap<_, _>>()
            }
            CompilerExpressionKind::Tuple(values) if values.len() == package.fields.len() => {
                package
                    .fields
                    .iter()
                    .zip(values)
                    .map(|(field, value)| (self.source.slice(field.name).to_owned(), value.clone()))
                    .collect()
            }
            _ if matches!(&argument.value_type, CompilerType::Tuple(fields)
                    if fields.len() == package.fields.len()) =>
            {
                let CompilerType::Tuple(field_types) = &argument.value_type else {
                    unreachable!("guard established tuple type")
                };
                let tuple = if matches!(argument.kind, CompilerExpressionKind::Local(_)) {
                    argument.clone()
                } else {
                    let storage_name = format!(
                        "topal.package.argument.{}.{}.{}",
                        argument.span.start, argument.span.end, self.next_private_binding
                    );
                    self.next_private_binding += 1;
                    if let Ok(facts) = self.known_structural_value_facts(argument, environment) {
                        self.private_binding_static_facts
                            .insert(storage_name.clone(), facts);
                    }
                    argument_bindings.push(CompilerArgumentBinding {
                        storage_name: storage_name.clone(),
                        value: argument.clone(),
                    });
                    CompilerExpression {
                        kind: CompilerExpressionKind::Local(storage_name),
                        value_type: argument.value_type.clone(),
                        int_range: argument.int_range.clone(),
                        rational_value: argument.rational_value.clone(),
                        span: argument.span,
                    }
                };
                package
                    .fields
                    .iter()
                    .zip(field_types)
                    .enumerate()
                    .map(|(index, (field, value_type))| {
                        (
                            self.source.slice(field.name).to_owned(),
                            CompilerExpression {
                                kind: CompilerExpressionKind::TupleField {
                                    tuple: Box::new(tuple.clone()),
                                    index,
                                },
                                value_type: value_type.clone(),
                                int_range: None,
                                rational_value: None,
                                span: argument.span,
                            },
                        )
                    })
                    .collect()
            }
            _ if matches!(
                argument.value_type,
                CompilerType::Tuple(_) | CompilerType::Record(_)
            ) =>
            {
                if let [field] = package.fields.as_slice() {
                    BTreeMap::from([(self.source.slice(field.name).to_owned(), argument.clone())])
                } else {
                    return Err(unsupported(
                        &self.source,
                        argument.span,
                        "opaque packaged function operand",
                    ));
                }
            }
            _ => return Ok(None),
        };

        let mut adapted = Vec::with_capacity(package.fields.len());
        let mut classifier_substitutions = self.classifier_substitutions.clone();
        for field in &package.fields {
            if field.qualifier.is_some() || !field.fields.is_empty() {
                return Err(unsupported(
                    &self.source,
                    field.name,
                    "nested or qualified packaged field",
                ));
            }
            let field_name = self.source.slice(field.name).to_owned();
            let value = if let Some(value) = supplied.get(&field_name) {
                value.clone()
            } else {
                let Some(default) = &field.default else {
                    return Ok(None);
                };
                let value = match self.analyze_expression(default, &BTreeMap::new()) {
                    Ok(value) => value,
                    Err(error) if error.code == "E-UNBOUND-NAME" => {
                        return Err(unsupported(
                            &self.source,
                            default.span(),
                            "non-closed packaged field default",
                        ));
                    }
                    Err(error) => return Err(error),
                };
                if !compiler_expression_is_closed(&value) {
                    return Err(unsupported(
                        &self.source,
                        default.span(),
                        "non-closed packaged field default",
                    ));
                }
                value
            };
            let Some(expected) = self.specialize_classifier(
                field.classifier,
                &value.value_type,
                &mut classifier_substitutions,
            )?
            else {
                return Ok(None);
            };
            if !compiler_packaged_field_supported(&expected) {
                return Err(unsupported(
                    &self.source,
                    field.classifier,
                    "unsupported packaged field",
                ));
            }
            let proven_sequence_start = declaration.module_identity.as_deref()
                == Some(&["std".to_owned(), "sequence".to_owned()][..])
                && ((matches!(
                    self.source.slice(declaration.name),
                    "chunk-step" | "window-step"
                ) && field_name == "start")
                    || (self.source.slice(declaration.name) == "transpose-step"
                        && field_name == "index"))
                && expected == CompilerType::Nat
                && value.value_type == CompilerType::Int;
            let Some(value) = adapt_call_argument(&expected, &value).or_else(|| {
                proven_sequence_start.then(|| CompilerExpression {
                    kind: CompilerExpressionKind::IntToNat(Box::new(value.clone())),
                    value_type: CompilerType::Nat,
                    int_range: value.int_range.clone(),
                    rational_value: None,
                    span: value.span,
                })
            }) else {
                return Ok(None);
            };
            adapted.push(value);
        }

        let mut normalized = declaration.clone();
        normalized.parameters = package
            .fields
            .iter()
            .cloned()
            .map(|mut field| {
                field.default = None;
                field
            })
            .collect();
        Ok(Some((
            normalized,
            adapted,
            argument_bindings,
            classifier_substitutions,
        )))
    }

    #[allow(clippy::too_many_lines)] // Two source operands are validated and flattened in explicit semantic order.
    fn normalize_compound_packaged_call(
        &mut self,
        declaration: &FunctionSource,
        arguments: &[CompilerExpression],
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<Option<NormalizedPackagedCall>, Diagnostic> {
        if declaration.parameters.len() != 2 {
            return Err(unsupported(
                &self.source,
                declaration.span,
                "packaged function operand count outside one or two",
            ));
        }
        if arguments.len() != declaration.parameters.len() {
            return Ok(None);
        }

        let mut declared_names = BTreeSet::new();
        for parameter in &declaration.parameters {
            let names = if parameter.fields.is_empty() {
                std::slice::from_ref(parameter)
            } else {
                parameter.fields.as_slice()
            };
            for named in names {
                let name = self.source.slice(named.name);
                if name != "_" && !declared_names.insert(name) {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-DUPLICATE-FUNCTION-PARAMETER",
                        named.name,
                        format!("parameter `{name}` is already declared in this function"),
                    ));
                }
            }
        }

        let mut flattened = Vec::new();
        let mut argument_bindings = Vec::new();
        for (operand_index, (parameter, argument)) in
            declaration.parameters.iter().zip(arguments).enumerate()
        {
            let retain = |value: &CompilerExpression,
                          value_index: usize,
                          bindings: &mut Vec<CompilerArgumentBinding>| {
                let storage_name = format!(
                    "topal.package.operand.{}.{}.{}.{}",
                    argument.span.start, argument.span.end, operand_index, value_index
                );
                let local = CompilerExpression {
                    kind: CompilerExpressionKind::Local(storage_name.clone()),
                    value_type: value.value_type.clone(),
                    int_range: value.int_range.clone(),
                    rational_value: value.rational_value.clone(),
                    span: value.span,
                };
                bindings.push(CompilerArgumentBinding {
                    storage_name,
                    value: value.clone(),
                });
                local
            };

            if parameter.fields.is_empty() {
                if parameter.qualifier.is_some() || parameter.default.is_some() {
                    return Err(unsupported(
                        &self.source,
                        parameter.name,
                        "qualified or defaulted unpackaged operand beside a package",
                    ));
                }
                flattened.push((
                    parameter.clone(),
                    Some(retain(argument, 0, &mut argument_bindings)),
                ));
                continue;
            }
            if parameter.qualifier.is_some() || parameter.default.is_some() {
                return Err(unsupported(
                    &self.source,
                    parameter.name,
                    "qualified or defaulted outer package",
                ));
            }

            match &argument.kind {
                CompilerExpressionKind::Record(values) => {
                    let declared = parameter
                        .fields
                        .iter()
                        .map(|field| self.source.slice(field.name))
                        .collect::<BTreeSet<_>>();
                    let supplied = values
                        .iter()
                        .map(|(label, _)| label.as_str())
                        .collect::<BTreeSet<_>>();
                    if supplied.len() != values.len()
                        || values
                            .iter()
                            .any(|(label, _)| !declared.contains(label.as_str()))
                        || parameter.fields.iter().any(|field| {
                            field.default.is_none()
                                && !supplied.contains(self.source.slice(field.name))
                        })
                    {
                        return Ok(None);
                    }
                    let supplied = values
                        .iter()
                        .enumerate()
                        .map(|(value_index, (label, value))| {
                            (
                                label.clone(),
                                retain(value, value_index, &mut argument_bindings),
                            )
                        })
                        .collect::<BTreeMap<_, _>>();
                    flattened.extend(parameter.fields.iter().cloned().map(|field| {
                        let supplied = supplied.get(self.source.slice(field.name)).cloned();
                        (field, supplied)
                    }));
                }
                CompilerExpressionKind::Tuple(values) if values.len() == parameter.fields.len() => {
                    flattened.extend(parameter.fields.iter().cloned().zip(
                        values.iter().enumerate().map(|(value_index, value)| {
                            Some(retain(value, value_index, &mut argument_bindings))
                        }),
                    ));
                }
                CompilerExpressionKind::Local(_)
                    if matches!(&argument.value_type, CompilerType::Tuple(fields)
                        if fields.len() == parameter.fields.len()) =>
                {
                    let CompilerType::Tuple(field_types) = &argument.value_type else {
                        unreachable!("guard established tuple type")
                    };
                    let facts = self.known_structural_value_facts(argument, environment)?;
                    flattened.extend(
                        parameter
                            .fields
                            .iter()
                            .cloned()
                            .zip(field_types.iter().enumerate().map(
                                |(value_index, value_type)| {
                                    let facts = facts.tuple_fields.get(value_index);
                                    Some(CompilerExpression {
                                        kind: CompilerExpressionKind::TupleField {
                                            tuple: Box::new(argument.clone()),
                                            index: value_index,
                                        },
                                        value_type: value_type.clone(),
                                        int_range: facts.and_then(|facts| facts.int_range.clone()),
                                        rational_value: facts
                                            .and_then(|facts| facts.rational_value.clone()),
                                        span: argument.span,
                                    })
                                },
                            )),
                    );
                }
                _ if matches!(
                    argument.value_type,
                    CompilerType::Tuple(_) | CompilerType::Record(_)
                ) =>
                {
                    return Err(unsupported(
                        &self.source,
                        argument.span,
                        "opaque or mismatched compound packaged function operand",
                    ));
                }
                _ => return Ok(None),
            }
        }

        let mut normalized_parameters = Vec::with_capacity(flattened.len());
        let mut adapted = Vec::with_capacity(flattened.len());
        let mut classifier_substitutions = self.classifier_substitutions.clone();
        for (mut parameter, supplied) in flattened {
            if parameter.qualifier.is_some() || !parameter.fields.is_empty() {
                return Err(unsupported(
                    &self.source,
                    parameter.name,
                    "nested or qualified compound packaged field",
                ));
            }
            let value = if let Some(value) = supplied {
                value
            } else {
                let Some(default) = &parameter.default else {
                    return Ok(None);
                };
                let value = match self.analyze_expression(default, &BTreeMap::new()) {
                    Ok(value) => value,
                    Err(error) if error.code == "E-UNBOUND-NAME" => {
                        return Err(unsupported(
                            &self.source,
                            default.span(),
                            "non-closed compound packaged field default",
                        ));
                    }
                    Err(error) => return Err(error),
                };
                if !compiler_expression_is_closed(&value) {
                    return Err(unsupported(
                        &self.source,
                        default.span(),
                        "non-closed compound packaged field default",
                    ));
                }
                value
            };
            let Some(expected) = self.specialize_classifier(
                parameter.classifier,
                &value.value_type,
                &mut classifier_substitutions,
            )?
            else {
                return Ok(None);
            };
            if !compiler_packaged_field_supported(&expected) {
                return Err(unsupported(
                    &self.source,
                    parameter.classifier,
                    "unsupported compound packaged operand field",
                ));
            }
            let Some(value) = adapt_call_argument(&expected, &value) else {
                return Ok(None);
            };
            parameter.default = None;
            normalized_parameters.push(parameter);
            adapted.push(value);
        }

        let mut normalized = declaration.clone();
        normalized.parameters = normalized_parameters;
        Ok(Some((
            normalized,
            adapted,
            argument_bindings,
            classifier_substitutions,
        )))
    }

    fn defining_context_captures(
        &self,
        declaration: &FunctionSource,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<Vec<CompilerContextCapture>, Diagnostic> {
        let mut context_captures = Vec::new();
        for (member_name, facts) in &self.root_bindings {
            let mut visiting = BTreeSet::new();
            if let Some(span) = self.transitive_context_member_span(
                declaration,
                member_name,
                facts.declaration_end,
                &mut visiting,
            )? {
                context_captures.push((member_name, facts, span));
            }
        }
        let mut captures = context_captures;
        captures.sort_by_key(|(_, facts, _)| facts.declaration_end);
        let mut captures = captures
            .into_iter()
            .map(|(member_name, facts, span)| {
                if !compiler_environment_capture_supported(&facts.value_type) {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "unsupported defining-context capture representation",
                    ));
                }
                let parameter_name = format!("@ {member_name}");
                let argument = if self.in_function {
                    let current = environment.get(&parameter_name).ok_or_else(|| {
                        unsupported(
                            &self.source,
                            span,
                            "defining-context forwarding without an exact caller capture",
                        )
                    })?;
                    binding_expression(current, span)
                } else {
                    data_member_expression(facts, span)
                };
                Ok(CompilerContextCapture {
                    parameter_name,
                    value_type: facts.value_type.clone(),
                    int_range: facts.int_range.clone(),
                    rational_value: facts.rational_value.clone(),
                    argument,
                    span,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut root_captures = Vec::new();
        for (member_name, facts) in &self.root_bindings {
            let mut visiting = BTreeSet::new();
            if let Some(span) =
                self.transitive_root_member_span(declaration, member_name, &mut visiting)?
            {
                root_captures.push((member_name, facts, span));
            }
        }
        root_captures.sort_by_key(|(_, facts, _)| facts.declaration_end);
        captures.extend(
            root_captures
                .into_iter()
                .map(|(member_name, facts, span)| {
                    if !compiler_environment_capture_supported(&facts.value_type) {
                        return Err(unsupported(
                            &self.source,
                            span,
                            "unsupported function-body root data capture representation",
                        ));
                    }
                    let parameter_name = format!("root {member_name}");
                    let argument = if self.in_function {
                        let current = environment.get(&parameter_name).ok_or_else(|| {
                            unsupported(
                                &self.source,
                                span,
                                "root-data forwarding without an exact caller capture",
                            )
                        })?;
                        binding_expression(current, span)
                    } else {
                        data_member_expression(facts, span)
                    };
                    Ok(CompilerContextCapture {
                        parameter_name,
                        value_type: facts.value_type.clone(),
                        int_range: facts.int_range.clone(),
                        rational_value: facts.rational_value.clone(),
                        argument,
                        span,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        );
        Ok(captures)
    }

    fn transitive_context_member_span(
        &self,
        declaration: &FunctionSource,
        member_name: &str,
        member_declaration_end: usize,
        visiting: &mut BTreeSet<String>,
    ) -> Result<Option<Span>, Diagnostic> {
        if member_declaration_end <= declaration.span.start
            && let Some(span) =
                function_body_context_member_span(&self.source, &declaration.body, member_name)
        {
            return Ok(Some(span));
        }
        let identity = format!(
            "{}@{}..{}",
            function_overload_identity(
                &self.source,
                self.source.slice(declaration.name),
                declaration,
            ),
            declaration.span.start,
            declaration.span.end
        );
        if !visiting.insert(identity.clone()) {
            return Ok(None);
        }
        let references =
            function_body_called_function_references(&self.source, &self.functions, declaration);
        for reference in references {
            let declarations = &reference.declarations;
            if reference.is_retained_value {
                let mut carries_member = false;
                for called in declarations {
                    if self
                        .transitive_context_member_span(
                            called,
                            member_name,
                            member_declaration_end,
                            visiting,
                        )?
                        .is_some()
                    {
                        carries_member = true;
                    }
                }
                if carries_member {
                    visiting.remove(&identity);
                    return Ok(Some(reference.span));
                }
                continue;
            }
            if declarations.len() != 1
                && let Some(called) =
                    self.capture_forwarding_overload(declaration, &reference, declarations)
            {
                if self
                    .transitive_context_member_span(
                        &called,
                        member_name,
                        member_declaration_end,
                        visiting,
                    )?
                    .is_some()
                {
                    visiting.remove(&identity);
                    return Ok(Some(reference.span));
                }
                continue;
            }
            let mut carries_member = false;
            for called in declarations {
                if self
                    .transitive_context_member_span(
                        called,
                        member_name,
                        member_declaration_end,
                        visiting,
                    )?
                    .is_some()
                {
                    carries_member = true;
                }
            }
            if carries_member {
                visiting.remove(&identity);
                if declarations.len() != 1 {
                    return Err(unsupported(
                        &self.source,
                        reference.span,
                        "overload-dependent defining-context capture forwarding",
                    ));
                }
                return Ok(Some(reference.span));
            }
        }
        visiting.remove(&identity);
        Ok(None)
    }

    fn transitive_root_member_span(
        &self,
        declaration: &FunctionSource,
        member_name: &str,
        visiting: &mut BTreeSet<String>,
    ) -> Result<Option<Span>, Diagnostic> {
        if let Some(span) =
            function_body_root_member_span(&self.source, &declaration.body, member_name)
        {
            return Ok(Some(span));
        }
        let identity = format!(
            "{}@{}..{}",
            function_overload_identity(
                &self.source,
                self.source.slice(declaration.name),
                declaration,
            ),
            declaration.span.start,
            declaration.span.end
        );
        if !visiting.insert(identity.clone()) {
            return Ok(None);
        }
        let references =
            function_body_called_function_references(&self.source, &self.functions, declaration);
        for reference in references {
            let declarations = &reference.declarations;
            if reference.is_retained_value {
                let mut carries_member = false;
                for called in declarations {
                    if self
                        .transitive_root_member_span(called, member_name, visiting)?
                        .is_some()
                    {
                        carries_member = true;
                    }
                }
                if carries_member {
                    visiting.remove(&identity);
                    return Ok(Some(reference.span));
                }
                continue;
            }
            if declarations.len() != 1
                && let Some(called) =
                    self.capture_forwarding_overload(declaration, &reference, declarations)
            {
                if self
                    .transitive_root_member_span(&called, member_name, visiting)?
                    .is_some()
                {
                    visiting.remove(&identity);
                    return Ok(Some(reference.span));
                }
                continue;
            }
            let mut carries_member = false;
            for called in declarations {
                if self
                    .transitive_root_member_span(called, member_name, visiting)?
                    .is_some()
                {
                    carries_member = true;
                }
            }
            if carries_member {
                visiting.remove(&identity);
                if declarations.len() != 1 {
                    return Err(unsupported(
                        &self.source,
                        reference.span,
                        "overload-dependent root-data capture forwarding",
                    ));
                }
                return Ok(Some(reference.span));
            }
        }
        visiting.remove(&identity);
        Ok(None)
    }
}
