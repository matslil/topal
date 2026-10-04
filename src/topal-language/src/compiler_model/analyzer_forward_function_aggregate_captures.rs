impl Analyzer {
    #[allow(clippy::too_many_lines)] // Every admitted aggregate path keeps its capture checks adjacent.
    fn forward_function_aggregate_captures(
        &self,
        value_type: &CompilerType,
        facts: &mut StaticValueFacts,
        boundary_name: &str,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
        forwarded: &mut Vec<CompilerContextCapture>,
    ) -> Result<(), Diagnostic> {
        match value_type {
            CompilerType::Function => {
                let callable = facts.callable.as_mut().ok_or_else(|| {
                    unsupported(
                        &self.source,
                        span,
                        "Function aggregate boundary without one exact callable identity per Function field",
                    )
                })?;
                self.forward_callable_captures(
                    boundary_name,
                    span,
                    callable,
                    environment,
                    forwarded,
                )
            }
            CompilerType::Tuple(fields) => {
                if fields.len() != facts.tuple_fields.len() {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "Function aggregate boundary without exact Tuple field facts",
                    ));
                }
                for (index, (field, facts)) in
                    fields.iter().zip(&mut facts.tuple_fields).enumerate()
                {
                    self.forward_function_aggregate_captures(
                        field,
                        facts,
                        &format!("{boundary_name} tuple {index}"),
                        span,
                        environment,
                        forwarded,
                    )?;
                }
                Ok(())
            }
            CompilerType::Record(fields) => {
                if fields.len() != facts.record_fields.len() {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "Function aggregate boundary without exact Record field facts",
                    ));
                }
                for (name, field) in fields {
                    let field_facts = facts.record_fields.get_mut(name).ok_or_else(|| {
                        unsupported(
                            &self.source,
                            span,
                            "Function aggregate boundary without exact Record field facts",
                        )
                    })?;
                    self.forward_function_aggregate_captures(
                        field,
                        field_facts,
                        &format!("{boundary_name} field {name}"),
                        span,
                        environment,
                        forwarded,
                    )?;
                }
                Ok(())
            }
            CompilerType::List(element) if element.as_ref() == &CompilerType::Function => {
                let entries = facts.list_entries.as_mut().ok_or_else(|| {
                    unsupported(
                        &self.source,
                        span,
                        "List Function boundary without exact finite entry facts",
                    )
                })?;
                for (index, entry_facts) in entries.iter_mut().enumerate() {
                    self.forward_function_aggregate_captures(
                        element,
                        entry_facts,
                        &format!("{boundary_name} List entry {index}"),
                        span,
                        environment,
                        forwarded,
                    )?;
                }
                Ok(())
            }
            CompilerType::Array { count, element }
                if element.as_ref() == &CompilerType::Function =>
            {
                let entries = facts.array_entries.as_mut().ok_or_else(|| {
                    unsupported(
                        &self.source,
                        span,
                        "Array Function boundary without exact entry facts",
                    )
                })?;
                if entries.len() != *count {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "Array Function boundary with inconsistent exact entry facts",
                    ));
                }
                for (index, entry_facts) in entries.iter_mut().enumerate() {
                    self.forward_function_aggregate_captures(
                        element,
                        entry_facts,
                        &format!("{boundary_name} Array entry {index}"),
                        span,
                        environment,
                        forwarded,
                    )?;
                }
                Ok(())
            }
            CompilerType::Map { key, value }
                if key.as_ref() == &CompilerType::String
                    && value.as_ref() == &CompilerType::Function =>
            {
                let entries = facts.map_entries.as_mut().ok_or_else(|| {
                    unsupported(
                        &self.source,
                        span,
                        "Map Function boundary without exact entry facts",
                    )
                })?;
                for (key, value_facts) in entries {
                    self.forward_function_aggregate_captures(
                        value,
                        value_facts,
                        &format!("{boundary_name} Map value {key:?}"),
                        span,
                        environment,
                        forwarded,
                    )?;
                }
                Ok(())
            }
            CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::Function => {
                match facts.optional.as_mut() {
                    Some(CompilerOptionalFacts::Some(payload_facts)) => self
                        .forward_function_aggregate_captures(
                            payload,
                            payload_facts,
                            &format!("{boundary_name} optional payload"),
                            span,
                            environment,
                            forwarded,
                        ),
                    Some(CompilerOptionalFacts::None) => Ok(()),
                    None => Err(unsupported(
                        &self.source,
                        span,
                        "Optional Function boundary without exact presence facts",
                    )),
                }
            }
            CompilerType::Sum(sum) => self.forward_function_sum_captures(
                sum,
                facts,
                boundary_name,
                span,
                environment,
                forwarded,
            ),
            CompilerType::Result(success) if success.as_ref() == &CompilerType::Function => {
                let success_facts = facts.result.as_mut().ok_or_else(|| {
                    unsupported(
                        &self.source,
                        span,
                        "Result Function boundary without exact success facts",
                    )
                })?;
                self.forward_function_aggregate_captures(
                    success,
                    &mut success_facts.success,
                    &format!("{boundary_name} Result success"),
                    span,
                    environment,
                    forwarded,
                )
            }
            _ => Ok(()),
        }
    }

    fn forward_function_sum_captures(
        &self,
        sum: &CompilerSumType,
        facts: &mut StaticValueFacts,
        boundary_name: &str,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
        forwarded: &mut Vec<CompilerContextCapture>,
    ) -> Result<(), Diagnostic> {
        let Some(sum_facts) = facts.sum.as_mut() else {
            return Err(unsupported(
                &self.source,
                span,
                "Function Sum boundary without exact active-alternative facts",
            ));
        };
        let index = usize::try_from(sum_facts.alternative).expect("u32 sum tag fits usize");
        let alternative = sum.alternatives.get(index).ok_or_else(|| {
            unsupported(
                &self.source,
                span,
                "Function Sum boundary with invalid active-alternative facts",
            )
        })?;
        match (&alternative.payload, sum_facts.payload.as_mut()) {
            (Some(payload), Some(payload_facts)) => self.forward_function_aggregate_captures(
                payload,
                payload_facts,
                &format!("{boundary_name} alternative {}", alternative.name),
                span,
                environment,
                forwarded,
            ),
            (None, None) => Ok(()),
            _ => Err(unsupported(
                &self.source,
                span,
                "Function Sum boundary without exact active-payload facts",
            )),
        }
    }

    fn forward_callable_captures(
        &self,
        boundary_name: &str,
        span: Span,
        callable: &mut CompilerCallableFacts,
        environment: &BTreeMap<String, BindingFacts>,
        forwarded: &mut Vec<CompilerContextCapture>,
    ) -> Result<(), Diagnostic> {
        self.extend_named_callable_environment_captures(callable, environment)?;
        match callable {
            CompilerCallableFacts::Anonymous { captures, .. } => {
                for (capture_name, capture) in captures {
                    let current = binding_facts_by_storage(environment, &capture.storage_name)
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
                                span,
                                &format!(
                                    "capturing Function boundary `{boundary_name}` outside capture `{capture_name}` lifetime or private representation"
                                ),
                            )
                        })?;
                    let hidden_name = format!("{boundary_name} capture {capture_name}");
                    forwarded.push(CompilerContextCapture {
                        parameter_name: hidden_name.clone(),
                        value_type: current.value_type.clone(),
                        int_range: current.int_range.clone(),
                        rational_value: current.rational_value.clone(),
                        argument: binding_expression(&current, span),
                        span,
                    });
                    capture.storage_name = hidden_name;
                    capture.origin = span.start;
                }
            }
            CompilerCallableFacts::Named { captures, .. } => {
                for capture in captures {
                    let (CompilerExpressionKind::Local(storage_name)
                    | CompilerExpressionKind::InfinityLocal { storage_name, .. }) =
                        &capture.argument.kind
                    else {
                        return Err(unsupported(
                            &self.source,
                            span,
                            &format!(
                                "capturing named Function boundary `{boundary_name}` without a retained private value"
                            ),
                        ));
                    };
                    let current = binding_facts_by_storage(environment, storage_name)
                        .cloned()
                        .or_else(|| named_callable_capture_binding(capture))
                        .filter(|current| {
                            current.runtime_bound
                                && current.value_type == capture.value_type
                                && compiler_function_result_supported(&current.value_type)
                                && !compiler_type_contains_generator(&current.value_type)
                                && !compiler_type_is_function_aggregate(&current.value_type)
                        })
                        .ok_or_else(|| {
                            unsupported(
                                &self.source,
                                span,
                                &format!(
                                    "capturing Function boundary `{boundary_name}` outside capture `{}` lifetime or private representation",
                                    capture.parameter_name
                                ),
                            )
                        })?;
                    let hidden_name = format!("{boundary_name} capture {}", capture.parameter_name);
                    forwarded.push(CompilerContextCapture {
                        parameter_name: hidden_name.clone(),
                        value_type: current.value_type.clone(),
                        int_range: current.int_range.clone(),
                        rational_value: current.rational_value.clone(),
                        argument: binding_expression(&current, span),
                        span,
                    });
                    capture.argument = CompilerExpression {
                        kind: CompilerExpressionKind::Local(hidden_name),
                        value_type: current.value_type.clone(),
                        int_range: current.int_range.clone(),
                        rational_value: current.rational_value.clone(),
                        span,
                    };
                }
            }
            CompilerCallableFacts::Symbolic(_) => {}
        }
        Ok(())
    }

    fn known_callable(
        &self,
        value: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
        capture_position: usize,
    ) -> Result<Option<CompilerCallableFacts>, Diagnostic> {
        if value.value_type != CompilerType::Function {
            return Ok(None);
        }
        match &value.kind {
            CompilerExpressionKind::FunctionValue(tag) => {
                let index = usize::try_from(*tag).expect("u32 tag fits usize");
                if index >= self.functions.len() {
                    let offset = index - self.functions.len();
                    if let Some((kind, _)) = COMPILER_SYMBOLIC_CALLABLES.get(offset) {
                        return Ok(Some(CompilerCallableFacts::Symbolic(*kind)));
                    }
                    return self
                        .anonymous_callables
                        .get(tag)
                        .cloned()
                        .map(Some)
                        .ok_or_else(|| {
                            unsupported(&self.source, value.span, "unknown Function value tag")
                        });
                }
                let name = self
                    .functions
                    .keys()
                    .nth(index)
                    .expect("checked Function value tag names a declaration")
                    .clone();
                let declarations = self
                    .functions
                    .get(&name)
                    .expect("checked Function value retains its declarations")
                    .iter()
                    .filter(|declaration| declaration.span.end <= capture_position)
                    .cloned()
                    .collect::<Vec<_>>();
                let captures =
                    self.named_callable_environment_captures(&declarations, environment)?;
                Ok(Some(CompilerCallableFacts::Named {
                    name,
                    declarations,
                    captures,
                }))
            }
            CompilerExpressionKind::Local(name) => binding_facts_by_storage(environment, name)
                .and_then(|facts| facts.callable.clone())
                .map(Some)
                .ok_or_else(|| unsupported(&self.source, value.span, "opaque Function value")),
            CompilerExpressionKind::Call { symbol, .. } => {
                let mut callable = self
                    .returned_function_values
                    .get(symbol)
                    .cloned()
                    .ok_or_else(|| {
                        unsupported(&self.source, value.span, "opaque Function result")
                    })?;
                let result_captures = &self
                    .instances
                    .iter()
                    .find(|function| function.symbol == *symbol)
                    .expect("checked Function result has a generated specialization")
                    .result_captures;
                remap_returned_callable_captures(
                    &mut callable,
                    result_captures,
                    symbol,
                    value.span,
                );
                Ok(Some(callable))
            }
            CompilerExpressionKind::TupleField { .. }
            | CompilerExpressionKind::RecordField { .. } => self
                .known_structural_value_facts(value, environment)?
                .callable
                .map(Some)
                .ok_or_else(|| {
                    unsupported(
                        &self.source,
                        value.span,
                        "Function aggregate field without retained callable identity",
                    )
                }),
            CompilerExpressionKind::PrivateBinding { body, .. } => {
                self.known_callable(body, environment, capture_position)
            }
            _ => Err(unsupported(
                &self.source,
                value.span,
                "computed Function value",
            )),
        }
    }

    fn known_scalar_facts(
        value: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> StaticValueFacts {
        let string_characters = match &value.kind {
            CompilerExpressionKind::StringRangeSelect { characters, .. } => {
                Some(characters.clone())
            }
            CompilerExpressionKind::Local(name) => binding_facts_by_storage(environment, name)
                .and_then(|facts| facts.string_characters.clone()),
            _ => Self::known_string_expression(value, environment)
                .map(|text| characters(&text).map(str::to_owned).collect()),
        };
        StaticValueFacts {
            int_range: value.int_range.clone(),
            rational_value: value.rational_value.clone(),
            string_value: Self::known_string_expression(value, environment),
            string_characters,
            callable: None,
            tuple_fields: Vec::new(),
            record_fields: Self::known_record_fields(value, environment),
            list_entries: None,
            list_string_characters: None,
            array_entries: None,
            map_entries: None,
            optional: None,
            sum: None,
            result: None,
        }
    }

    fn list_first_string_payload_facts(
        &self,
        subject: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Option<StaticValueFacts> {
        let CompilerExpressionKind::ListFirst(list) = &subject.kind else {
            return None;
        };
        let CompilerType::Optional(payload) = &subject.value_type else {
            return None;
        };
        if !matches!(
            payload.as_ref(),
            CompilerType::Character | CompilerType::String
        ) {
            return None;
        }
        let characters = self
            .known_structural_value_facts(list, environment)
            .ok()?
            .list_string_characters?;
        Some(StaticValueFacts {
            string_characters: Some(characters),
            ..StaticValueFacts::default()
        })
    }

    #[allow(clippy::too_many_lines)] // Structural fact retention stays exhaustive in one dispatcher.
    fn known_structural_value_facts(
        &self,
        value: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<StaticValueFacts, Diagnostic> {
        let mut facts = Self::known_scalar_facts(value, environment);
        match &value.kind {
            CompilerExpressionKind::Tuple(fields) => {
                facts.tuple_fields = fields
                    .iter()
                    .map(|field| self.known_structural_value_facts(field, environment))
                    .collect::<Result<Vec<_>, _>>()?;
            }
            CompilerExpressionKind::Record(fields) => {
                facts.record_fields = fields
                    .iter()
                    .map(|(name, field)| {
                        self.known_structural_value_facts(field, environment)
                            .map(|facts| (name.clone(), facts))
                    })
                    .collect::<Result<BTreeMap<_, _>, _>>()?;
            }
            CompilerExpressionKind::Binary {
                operation: CompilerBinary::QuotientModulo,
                left,
                right,
            } => {
                if let (Some(left), Some(right)) = (exact_int(left), exact_int(right))
                    && right != BigInt::from(0)
                {
                    facts.tuple_fields = vec![
                        StaticValueFacts {
                            int_range: Some(IntRange::exact(&left / &right)),
                            ..StaticValueFacts::default()
                        },
                        StaticValueFacts {
                            int_range: Some(IntRange::exact(&left % &right)),
                            ..StaticValueFacts::default()
                        },
                    ];
                }
            }
            CompilerExpressionKind::ListEmpty
                if matches!(&value.value_type, CompilerType::List(_)) =>
            {
                facts.list_entries = Some(Vec::new());
                if matches!(&value.value_type, CompilerType::List(element)
                    if compiler_type_contains_string(element.as_ref()))
                {
                    facts.list_string_characters = Some(Vec::new());
                }
            }
            CompilerExpressionKind::ListEntry {
                value: entry,
                remaining,
            } if matches!(&value.value_type, CompilerType::List(_)) => {
                let entry_facts = self.known_structural_value_facts(entry, environment)?;
                let remaining_facts = self.known_structural_value_facts(remaining, environment)?;
                if let Some(mut remaining_entries) = remaining_facts.list_entries.clone() {
                    let mut entries = vec![entry_facts.clone()];
                    entries.append(&mut remaining_entries);
                    facts.list_entries = Some(entries);
                }
                if matches!(&value.value_type, CompilerType::List(element)
                    if compiler_type_contains_string(element.as_ref()))
                {
                    facts.list_string_characters = merge_string_characters(
                        static_value_string_characters(&entry_facts),
                        remaining_facts.list_string_characters,
                    );
                }
            }
            CompilerExpressionKind::ListAppend {
                list, value: entry, ..
            } if matches!(&value.value_type, CompilerType::List(element)
                    if compiler_type_contains_string(element.as_ref())) =>
            {
                let list_facts = self.known_structural_value_facts(list, environment)?;
                let entry_facts = self.known_structural_value_facts(entry, environment)?;
                facts.list_string_characters = merge_string_characters(
                    list_facts.list_string_characters,
                    static_value_string_characters(&entry_facts),
                );
            }
            CompilerExpressionKind::ListConcat { left, right }
                if matches!(&value.value_type, CompilerType::List(element)
                    if compiler_type_contains_string(element.as_ref())) =>
            {
                facts.list_string_characters = merge_string_characters(
                    self.known_structural_value_facts(left, environment)?
                        .list_string_characters,
                    self.known_structural_value_facts(right, environment)?
                        .list_string_characters,
                );
            }
            CompilerExpressionKind::ListRangeSelect { list, .. }
            | CompilerExpressionKind::ListReverse(list) => {
                facts.list_string_characters = self
                    .known_structural_value_facts(list, environment)?
                    .list_string_characters;
            }
            CompilerExpressionKind::StringCharactersCollect { characters, .. }
            | CompilerExpressionKind::StringProvenanceCharactersCollect { characters, .. } => {
                facts.list_string_characters = Some(characters.clone());
            }
            CompilerExpressionKind::ListFold { list, body, .. } => {
                facts = self.known_structural_value_facts(&body.result, environment)?;
                let candidates = self
                    .known_structural_value_facts(list, environment)?
                    .list_string_characters;
                retain_string_character_provenance_for_type(
                    &mut facts,
                    &value.value_type,
                    candidates,
                );
            }
            CompilerExpressionKind::ContainerCollect {
                source,
                kind: CompilerContainerKind::Array,
                ..
            } if matches!(
                &value.value_type,
                CompilerType::Array { element, .. }
                    if element.as_ref() == &CompilerType::Function
            ) =>
            {
                facts.array_entries = self
                    .known_structural_value_facts(source, environment)?
                    .list_entries;
            }
            CompilerExpressionKind::ContainerCollect {
                source,
                kind: CompilerContainerKind::Map,
                map_policy,
                map_keys: Some(keys),
            } if matches!(
                &value.value_type,
                CompilerType::Map { key, value }
                    if key.as_ref() == &CompilerType::String
                        && value.as_ref() == &CompilerType::Function
            ) =>
            {
                let source_entries = self
                    .known_structural_value_facts(source, environment)?
                    .list_entries
                    .ok_or_else(|| {
                        unsupported(
                            &self.source,
                            source.span,
                            "Map Function source without exact finite entry facts",
                        )
                    })?;
                if source_entries.len() != keys.len() {
                    return Err(unsupported(
                        &self.source,
                        source.span,
                        "Map Function source with inconsistent exact key/value facts",
                    ));
                }
                let mut positions = BTreeMap::new();
                let mut entries = Vec::new();
                for (key, pair_facts) in keys.iter().zip(source_entries) {
                    let value_facts = pair_facts.tuple_fields.get(1).cloned().ok_or_else(|| {
                        unsupported(
                            &self.source,
                            source.span,
                            "Map Function source without exact pair value facts",
                        )
                    })?;
                    if let Some(index) = positions.get(key).copied() {
                        if matches!(map_policy, Some(CompilerMapCollisionPolicy::KeepLast)) {
                            entries[index] = (key.clone(), value_facts);
                        }
                    } else {
                        positions.insert(key.clone(), entries.len());
                        entries.push((key.clone(), value_facts));
                    }
                }
                facts.map_entries = Some(entries);
            }
            CompilerExpressionKind::ListFirst(list) => {
                if let Some(entries) = self
                    .known_structural_value_facts(list, environment)?
                    .list_entries
                {
                    facts.optional = Some(
                        entries
                            .first()
                            .cloned()
                            .map_or(CompilerOptionalFacts::None, |entry| {
                                CompilerOptionalFacts::Some(Box::new(entry))
                            }),
                    );
                }
            }
            CompilerExpressionKind::ArrayAt { array, index }
                if value.value_type == CompilerType::Optional(Box::new(CompilerType::Function)) =>
            {
                let CompilerType::Array { count, element } = &array.value_type else {
                    unreachable!("checked Array access retains its Array classifier")
                };
                debug_assert_eq!(element.as_ref(), &CompilerType::Function);
                let entries = self
                    .known_structural_value_facts(array, environment)?
                    .array_entries
                    .ok_or_else(|| {
                        unsupported(
                            &self.source,
                            array.span,
                            "Array Function access without exact entry facts",
                        )
                    })?;
                if entries.len() != *count {
                    return Err(unsupported(
                        &self.source,
                        array.span,
                        "Array Function access with inconsistent exact entry facts",
                    ));
                }
                facts.optional = Some(if *index < *count {
                    CompilerOptionalFacts::Some(Box::new(entries[*index].clone()))
                } else {
                    CompilerOptionalFacts::None
                });
            }
            CompilerExpressionKind::MapLookup {
                mapping,
                exact_key: Some(key),
                ..
            } if value.value_type == CompilerType::Optional(Box::new(CompilerType::Function)) => {
                let entries = self
                    .known_structural_value_facts(mapping, environment)?
                    .map_entries
                    .ok_or_else(|| {
                        unsupported(
                            &self.source,
                            mapping.span,
                            "Map Function lookup without exact entry facts",
                        )
                    })?;
                facts.optional = Some(
                    entries
                        .into_iter()
                        .find_map(|(candidate, facts)| {
                            (candidate == *key)
                                .then_some(CompilerOptionalFacts::Some(Box::new(facts)))
                        })
                        .unwrap_or(CompilerOptionalFacts::None),
                );
            }
            CompilerExpressionKind::OptionalSome(payload) => {
                facts.optional = Some(CompilerOptionalFacts::Some(Box::new(
                    self.known_structural_value_facts(payload, environment)?,
                )));
            }
            CompilerExpressionKind::OptionalNone => {
                facts.optional = Some(CompilerOptionalFacts::None);
            }
            CompilerExpressionKind::Sum {
                value: alternative,
                payload,
            } => {
                let CompilerType::Sum(sum) = &value.value_type else {
                    unreachable!("checked Sum expression retains its nominal classifier")
                };
                facts.sum = Some(self.known_sum_value_facts(
                    sum,
                    *alternative,
                    payload.as_deref(),
                    environment,
                )?);
            }
            CompilerExpressionKind::ResultSuccess(payload) => {
                facts.result = Some(CompilerResultFacts {
                    success: Box::new(self.known_structural_value_facts(payload, environment)?),
                });
            }
            CompilerExpressionKind::ResultProject(result)
            | CompilerExpressionKind::ResultProjectBoundary(result) => {
                facts = self
                    .known_structural_value_facts(result, environment)?
                    .result
                    .map(|result| *result.success)
                    .unwrap_or_default();
            }
            CompilerExpressionKind::RecordReconstruct { base, replacements } => {
                facts = self.known_structural_value_facts(base, environment)?;
                for (name, replacement) in replacements {
                    facts.record_fields.insert(
                        name.clone(),
                        self.known_structural_value_facts(replacement, environment)?,
                    );
                }
            }
            CompilerExpressionKind::BooleanDecision {
                when_true,
                when_false,
                ..
            } => {
                facts = merge_static_provenance(
                    self.known_structural_value_facts(when_true, environment)?,
                    self.known_structural_value_facts(when_false, environment)?,
                );
            }
            CompilerExpressionKind::OptionalDecision {
                subject,
                some_action,
                none_action,
                ..
            } => {
                let mut some_facts = self.known_structural_value_facts(some_action, environment)?;
                if some_facts.string_characters.is_none() {
                    let payload_facts = self
                        .known_structural_value_facts(subject, environment)
                        .ok()
                        .and_then(present_optional_payload)
                        .or_else(|| self.list_first_string_payload_facts(subject, environment));
                    if let Some(payload_facts) = payload_facts {
                        some_facts = payload_facts;
                    }
                }
                facts = merge_static_provenance(
                    some_facts,
                    self.known_structural_value_facts(none_action, environment)?,
                );
            }
            CompilerExpressionKind::ListDecision {
                entry_action,
                empty_action,
                ..
            } => {
                facts = merge_static_provenance(
                    self.known_structural_value_facts(entry_action, environment)?,
                    self.known_structural_value_facts(empty_action, environment)?,
                );
            }
            CompilerExpressionKind::Local(name) => {
                if let Some(binding) = binding_facts_by_storage(environment, name) {
                    facts.callable.clone_from(&binding.callable);
                    facts
                        .string_characters
                        .clone_from(&binding.string_characters);
                    facts.tuple_fields.clone_from(&binding.tuple_fields);
                    facts.record_fields.clone_from(&binding.record_fields);
                    facts.list_entries.clone_from(&binding.list_entries);
                    facts
                        .list_string_characters
                        .clone_from(&binding.list_string_characters);
                    facts.array_entries.clone_from(&binding.array_entries);
                    facts.map_entries.clone_from(&binding.map_entries);
                    facts.optional.clone_from(&binding.optional);
                    facts.sum.clone_from(&binding.sum);
                    facts.result.clone_from(&binding.result);
                } else if let Some(binding) = self.private_binding_static_facts.get(name) {
                    facts = binding.clone();
                }
            }
            CompilerExpressionKind::TupleField { tuple, index } => {
                facts = self
                    .known_structural_value_facts(tuple, environment)?
                    .tuple_fields
                    .get(*index)
                    .cloned()
                    .unwrap_or_default();
            }
            CompilerExpressionKind::RecordField { record, label } => {
                facts = self
                    .known_structural_value_facts(record, environment)?
                    .record_fields
                    .get(label)
                    .cloned()
                    .unwrap_or_default();
            }
            CompilerExpressionKind::Call { symbol, .. } => {
                if let Some(returned) = self.returned_aggregate_value_facts.get(symbol) {
                    facts = returned.clone();
                    let result_captures = &self
                        .instances
                        .iter()
                        .find(|function| function.symbol == *symbol)
                        .expect("checked Function aggregate result has a generated specialization")
                        .result_captures;
                    remap_returned_aggregate_captures(
                        &mut facts,
                        result_captures,
                        symbol,
                        value.span,
                    );
                }
            }
            CompilerExpressionKind::Block(block) => {
                facts = self.known_structural_value_facts(&block.result, environment)?;
            }
            CompilerExpressionKind::PrivateBinding { body, .. } => {
                facts = self.known_structural_value_facts(body, environment)?;
            }
            _ => {}
        }
        self.complete_structural_callable_facts(value, environment, &mut facts)?;
        Ok(facts)
    }

    fn complete_structural_callable_facts(
        &self,
        value: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
        facts: &mut StaticValueFacts,
    ) -> Result<(), Diagnostic> {
        if value.value_type == CompilerType::Function
            && facts.callable.is_none()
            && !matches!(
                value.kind,
                CompilerExpressionKind::TupleField { .. }
                    | CompilerExpressionKind::RecordField { .. }
            )
        {
            facts.callable = self.known_callable(value, environment, value.span.start)?;
        }
        Ok(())
    }

    fn known_sum_value_facts(
        &self,
        sum: &CompilerSumType,
        alternative: u32,
        payload: Option<&CompilerExpression>,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerSumFacts, Diagnostic> {
        let index = usize::try_from(alternative).expect("u32 sum tag fits usize");
        Ok(CompilerSumFacts {
            alternative,
            alternative_name: sum.alternatives[index].name.clone(),
            payload: payload
                .map(|payload| self.known_structural_value_facts(payload, environment))
                .transpose()?
                .map(Box::new),
        })
    }

    fn function_aggregate_result_captures(
        &self,
        value_type: &CompilerType,
        facts: &StaticValueFacts,
        environment: &BTreeMap<String, BindingFacts>,
        span: Span,
    ) -> Result<Vec<CompilerFunctionResultCapture>, Diagnostic> {
        let mut captures = Vec::new();
        self.collect_function_aggregate_result_captures(
            value_type,
            facts,
            environment,
            span,
            &mut Vec::new(),
            &mut captures,
        )?;
        Ok(captures)
    }

    #[allow(clippy::too_many_lines)] // Every admitted result path keeps its capture checks adjacent.
    fn collect_function_aggregate_result_captures(
        &self,
        value_type: &CompilerType,
        facts: &StaticValueFacts,
        environment: &BTreeMap<String, BindingFacts>,
        span: Span,
        path: &mut Vec<CompilerAggregatePathElement>,
        result: &mut Vec<CompilerFunctionResultCapture>,
    ) -> Result<(), Diagnostic> {
        match value_type {
            CompilerType::Function => {
                self.collect_function_leaf_result_captures(facts, environment, span, path, result)
            }
            CompilerType::Tuple(fields) => {
                if fields.len() != facts.tuple_fields.len() {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "Function aggregate result without exact Tuple field facts",
                    ));
                }
                for (index, (field, facts)) in fields.iter().zip(&facts.tuple_fields).enumerate() {
                    path.push(CompilerAggregatePathElement::Tuple(index));
                    self.collect_function_aggregate_result_captures(
                        field,
                        facts,
                        environment,
                        span,
                        path,
                        result,
                    )?;
                    path.pop();
                }
                Ok(())
            }
            CompilerType::Record(fields) => {
                if fields.len() != facts.record_fields.len() {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "Function aggregate result without exact Record field facts",
                    ));
                }
                for (name, field) in fields {
                    let field_facts = facts.record_fields.get(name).ok_or_else(|| {
                        unsupported(
                            &self.source,
                            span,
                            "Function aggregate result without exact Record field facts",
                        )
                    })?;
                    path.push(CompilerAggregatePathElement::Record(name.clone()));
                    self.collect_function_aggregate_result_captures(
                        field,
                        field_facts,
                        environment,
                        span,
                        path,
                        result,
                    )?;
                    path.pop();
                }
                Ok(())
            }
            CompilerType::List(element) if element.as_ref() == &CompilerType::Function => {
                let entries = facts.list_entries.as_ref().ok_or_else(|| {
                    unsupported(
                        &self.source,
                        span,
                        "List Function result without exact finite entry facts",
                    )
                })?;
                for (index, entry_facts) in entries.iter().enumerate() {
                    path.push(CompilerAggregatePathElement::ListEntry(index));
                    self.collect_function_aggregate_result_captures(
                        element,
                        entry_facts,
                        environment,
                        span,
                        path,
                        result,
                    )?;
                    path.pop();
                }
                Ok(())
            }
            CompilerType::Array { count, element }
                if element.as_ref() == &CompilerType::Function =>
            {
                let entries = facts.array_entries.as_ref().ok_or_else(|| {
                    unsupported(
                        &self.source,
                        span,
                        "Array Function result without exact entry facts",
                    )
                })?;
                if entries.len() != *count {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "Array Function result with inconsistent exact entry facts",
                    ));
                }
                for (index, entry_facts) in entries.iter().enumerate() {
                    path.push(CompilerAggregatePathElement::ArrayEntry(index));
                    self.collect_function_aggregate_result_captures(
                        element,
                        entry_facts,
                        environment,
                        span,
                        path,
                        result,
                    )?;
                    path.pop();
                }
                Ok(())
            }
            CompilerType::Map { key, value }
                if key.as_ref() == &CompilerType::String
                    && value.as_ref() == &CompilerType::Function =>
            {
                let entries = facts.map_entries.as_ref().ok_or_else(|| {
                    unsupported(
                        &self.source,
                        span,
                        "Map Function result without exact entry facts",
                    )
                })?;
                for (key, value_facts) in entries {
                    path.push(CompilerAggregatePathElement::MapValue(key.clone()));
                    self.collect_function_aggregate_result_captures(
                        value,
                        value_facts,
                        environment,
                        span,
                        path,
                        result,
                    )?;
                    path.pop();
                }
                Ok(())
            }
            CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::Function => {
                match facts.optional.as_ref() {
                    Some(CompilerOptionalFacts::Some(payload_facts)) => {
                        path.push(CompilerAggregatePathElement::OptionalPayload);
                        let result = self.collect_function_aggregate_result_captures(
                            payload,
                            payload_facts,
                            environment,
                            span,
                            path,
                            result,
                        );
                        path.pop();
                        result
                    }
                    Some(CompilerOptionalFacts::None) => Ok(()),
                    None => Err(unsupported(
                        &self.source,
                        span,
                        "Optional Function result without exact presence facts",
                    )),
                }
            }
            CompilerType::Sum(sum) => self.collect_function_sum_result_captures(
                sum,
                facts,
                environment,
                span,
                path,
                result,
            ),
            CompilerType::Result(success) if success.as_ref() == &CompilerType::Function => {
                let success_facts = facts.result.as_ref().ok_or_else(|| {
                    unsupported(
                        &self.source,
                        span,
                        "Result Function result without exact success facts",
                    )
                })?;
                path.push(CompilerAggregatePathElement::ResultSuccess);
                let collected = self.collect_function_aggregate_result_captures(
                    success,
                    &success_facts.success,
                    environment,
                    span,
                    path,
                    result,
                );
                path.pop();
                collected
            }
            _ => Ok(()),
        }
    }

    fn collect_function_sum_result_captures(
        &self,
        sum: &CompilerSumType,
        facts: &StaticValueFacts,
        environment: &BTreeMap<String, BindingFacts>,
        span: Span,
        path: &mut Vec<CompilerAggregatePathElement>,
        result: &mut Vec<CompilerFunctionResultCapture>,
    ) -> Result<(), Diagnostic> {
        let Some(sum_facts) = facts.sum.as_ref() else {
            return Err(unsupported(
                &self.source,
                span,
                "Function Sum result without exact active-alternative facts",
            ));
        };
        let index = usize::try_from(sum_facts.alternative).expect("u32 sum tag fits usize");
        let alternative = sum.alternatives.get(index).ok_or_else(|| {
            unsupported(
                &self.source,
                span,
                "Function Sum result with invalid active-alternative facts",
            )
        })?;
        match (&alternative.payload, sum_facts.payload.as_deref()) {
            (Some(payload), Some(payload_facts)) => {
                path.push(CompilerAggregatePathElement::SumPayload(
                    alternative.name.clone(),
                ));
                let collected = self.collect_function_aggregate_result_captures(
                    payload,
                    payload_facts,
                    environment,
                    span,
                    path,
                    result,
                );
                path.pop();
                collected
            }
            (None, None) => Ok(()),
            _ => Err(unsupported(
                &self.source,
                span,
                "Function Sum result without exact active-payload facts",
            )),
        }
    }

    fn collect_function_leaf_result_captures(
        &self,
        facts: &StaticValueFacts,
        environment: &BTreeMap<String, BindingFacts>,
        span: Span,
        path: &[CompilerAggregatePathElement],
        result: &mut Vec<CompilerFunctionResultCapture>,
    ) -> Result<(), Diagnostic> {
        let callable = facts.callable.as_ref().ok_or_else(|| {
            unsupported(
                &self.source,
                span,
                "Function aggregate result without one exact callable identity per Function field",
            )
        })?;
        match callable {
            CompilerCallableFacts::Anonymous { captures, .. } => {
                for (name, capture) in captures {
                    let current = binding_facts_by_storage(environment, &capture.storage_name)
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
                                span,
                                &format!(
                                    "capturing Function aggregate result outside capture `{name}` lifetime or private representation"
                                ),
                            )
                        })?;
                    result.push(CompilerFunctionResultCapture {
                        name: name.clone(),
                        path: path.to_vec(),
                        value_type: current.value_type.clone(),
                        value: binding_expression(&current, span),
                    });
                }
            }
            CompilerCallableFacts::Named { captures, .. } => {
                self.collect_named_callable_result_captures(
                    captures,
                    environment,
                    span,
                    path,
                    result,
                )?;
            }
            CompilerCallableFacts::Symbolic(_) => {}
        }
        Ok(())
    }

    fn collect_named_callable_result_captures(
        &self,
        captures: &[CompilerContextCapture],
        environment: &BTreeMap<String, BindingFacts>,
        span: Span,
        path: &[CompilerAggregatePathElement],
        result: &mut Vec<CompilerFunctionResultCapture>,
    ) -> Result<(), Diagnostic> {
        for capture in captures {
            let (CompilerExpressionKind::Local(storage_name)
            | CompilerExpressionKind::InfinityLocal { storage_name, .. }) = &capture.argument.kind
            else {
                return Err(unsupported(
                    &self.source,
                    span,
                    "named Function result without a retained private environment value",
                ));
            };
            let current = binding_facts_by_storage(environment, storage_name)
                .cloned()
                .or_else(|| named_callable_capture_binding(capture))
                .filter(|current| {
                    current.runtime_bound
                        && current.value_type == capture.value_type
                        && compiler_function_result_supported(&current.value_type)
                        && !compiler_type_contains_generator(&current.value_type)
                        && !compiler_type_is_function_aggregate(&current.value_type)
                })
                .ok_or_else(|| {
                    unsupported(
                        &self.source,
                        span,
                        &format!(
                            "named Function result outside environment `{}` lifetime or private representation",
                            capture.parameter_name
                        ),
                    )
                })?;
            result.push(CompilerFunctionResultCapture {
                name: capture.parameter_name.clone(),
                path: path.to_vec(),
                value_type: current.value_type.clone(),
                value: binding_expression(&current, span),
            });
        }
        Ok(())
    }

    fn finish_nat_conversion(
        &self,
        value: CompilerExpression,
        span: Span,
        error_span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        require_type(
            &self.source,
            value.span,
            &CompilerType::Int,
            &value.value_type,
        )?;
        let proven_nonnegative_power = matches!(
            &value.kind,
            CompilerExpressionKind::Binary {
                operation: CompilerBinary::Power,
                left,
                right,
            } if right.value_type == CompilerType::Nat
                && left
                    .int_range
                    .as_ref()
                    .is_some_and(|range| range.lower >= BigInt::from(0))
        );
        if proven_nonnegative_power {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::IntToNat(Box::new(value)),
                value_type: CompilerType::Nat,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if value
            .int_range
            .as_ref()
            .is_some_and(|range| range.lower >= BigInt::from(0))
        {
            let int_range = value.int_range.clone();
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::IntToNat(Box::new(value)),
                value_type: CompilerType::Nat,
                int_range,
                rational_value: None,
                span,
            });
        }
        if self.compiler_proven_nonnegative_expression(&value) {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::IntToNat(Box::new(value)),
                value_type: CompilerType::Nat,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        // The private quantile helper is reached only for a nonempty List and a
        // checked probability, so its accepted boundary is at least one.
        let proven_floor_position = self
            .active_calls
            .last()
            .is_some_and(|identity| identity.contains("floor-position"))
            && matches!(&value.kind, CompilerExpressionKind::Binary {
                operation: CompilerBinary::Subtract,
                right,
                ..
            } if exact_int(right).is_some_and(|value| value == BigInt::from(1)));
        if proven_floor_position {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::IntToNat(Box::new(value)),
                value_type: CompilerType::Nat,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if value
            .int_range
            .as_ref()
            .is_some_and(|range| range.upper < BigInt::from(0))
            && compiler_expression_is_closed(&value)
        {
            return Err(source_diagnostic(
                &self.source,
                "E-NAT-OUT-OF-RANGE",
                error_span,
                "a negative Int is outside the Nat constraint",
            ));
        }
        Ok(Self::finish_validation(
            CompilerValidation::IntToNat,
            value,
            CompilerType::Nat,
            span,
            error_span,
        ))
    }

    fn analyze_rational_constructor(
        &mut self,
        argument: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        if let Expression::Product { fields, .. } = argument
            && fields.len() == 2
            && fields.iter().all(|field| field.label.is_none())
        {
            let numerator = self.analyze_expression(&fields[0].value, environment)?;
            let denominator = self.analyze_expression(&fields[1].value, environment)?;
            require_type(
                &self.source,
                numerator.span,
                &CompilerType::Int,
                &numerator.value_type,
            )?;
            require_type(
                &self.source,
                denominator.span,
                &CompilerType::Int,
                &denominator.value_type,
            )?;
            if is_proven_zero_numeric(&denominator) {
                if compiler_expression_is_closed(&denominator) {
                    if is_proven_zero_numeric(&numerator) {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-INDETERMINATE-RATIONAL",
                            argument.span(),
                            "Rational (0, 0) does not determine one numeric value",
                        ));
                    }
                    return Err(division_by_zero(&self.source, argument.span()));
                }
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Fallible {
                        operation: CompilerFallible::RationalConstruct,
                        left: Box::new(numerator),
                        right: Box::new(denominator),
                        error_span: argument.span(),
                    },
                    value_type: CompilerType::Result(Box::new(CompilerType::Rational)),
                    int_range: None,
                    rational_value: None,
                    span,
                });
            }
            if !is_proven_nonzero_numeric(&denominator) {
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Fallible {
                        operation: CompilerFallible::RationalConstruct,
                        left: Box::new(numerator),
                        right: Box::new(denominator),
                        error_span: argument.span(),
                    },
                    value_type: CompilerType::Result(Box::new(CompilerType::Rational)),
                    int_range: None,
                    rational_value: None,
                    span,
                });
            }
            let rational_value = exact_int(&numerator)
                .zip(exact_int(&denominator))
                .map(|(numerator, denominator)| BigRational::new(numerator, denominator));
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::RationalConstruct {
                    numerator: Box::new(numerator),
                    denominator: Box::new(denominator),
                },
                value_type: CompilerType::Rational,
                int_range: None,
                rational_value,
                span,
            });
        }
        let mut value = self.analyze_expression(argument, environment)?;
        if value.value_type == CompilerType::Nat {
            value.value_type = CompilerType::Int;
        }
        require_type(
            &self.source,
            value.span,
            &CompilerType::Int,
            &value.value_type,
        )?;
        let rational_value = exact_int(&value).map(BigRational::from_integer);
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::IntToRational(Box::new(value)),
            value_type: CompilerType::Rational,
            int_range: None,
            rational_value,
            span,
        })
    }
}
