impl Generator<'_> {
    #[allow(clippy::if_not_else, clippy::too_many_lines)] // Captured calls precede exhaustive ordinary calls.
    fn emit_expression(
        &mut self,
        expression: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
    ) -> LlValue {
        match &expression.kind {
            CompilerExpressionKind::Unit
            | CompilerExpressionKind::Identity(_)
            | CompilerExpressionKind::TypeView(_)
            | CompilerExpressionKind::FunctionView(_)
            | CompilerExpressionKind::LanguageContext(_)
            | CompilerExpressionKind::NativeSerializer(_)
            | CompilerExpressionKind::ExternalMetadata(_) => LlValue::Unit,
            CompilerExpressionKind::Infinity { negative } => {
                let direction = if *negative { "negative" } else { "positive" };
                if expression.value_type == CompilerType::InfiniteRational {
                    LlValue::Rational(body.instruction(
                        &format!(
                            "call ptr @topal.runtime.rational.from.int(ptr @topal.runtime.int.{direction}.infinity)"
                        ),
                        expression.span,
                        &mut self.debug,
                    ))
                } else {
                    LlValue::Int(format!("@topal.runtime.int.{direction}.infinity"))
                }
            }
            CompilerExpressionKind::ExternalLocationConstruct(location) => {
                let range_start = self.emit_int_literal(&location.offset.offset_type.range.lower);
                let offset = self.emit_int_literal(&location.offset.offset);
                LlValue::ExternalLocation {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.location.make(ptr {}, ptr {})",
                            range_start.integer(),
                            offset.integer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    location: location.location_type.clone(),
                }
            }
            CompilerExpressionKind::ExternalLocationWrite {
                location, value, ..
            } => {
                let location = self.emit_expression(location, body, environment);
                let value = self.emit_expression(value, body, environment);
                body.effect(
                    &format!(
                        "call void @topal.runtime.location.write(ptr {}, ptr {})",
                        location.location_pointer(),
                        value.integer()
                    ),
                    expression.span,
                    &mut self.debug,
                );
                LlValue::Unit
            }
            CompilerExpressionKind::ExternalLocationRead { location, .. } => {
                let location = self.emit_expression(location, body, environment);
                LlValue::Int(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.location.read(ptr {})",
                        location.location_pointer()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::Capability(capability) => {
                LlValue::StaticDisplay(capability.display())
            }
            CompilerExpressionKind::Serialize { bytes, value } => {
                let value = self.emit_expression(value, body, environment);
                let (expected, byte_count) = self.emit_raw_bytes_global(bytes);
                let stream = body.instruction(
                    &format!(
                        "call ptr @topal.runtime.serialization.make(ptr {expected}, i64 {byte_count})"
                    ),
                    expression.span,
                    &mut self.debug,
                );
                LlValue::SerializationStream {
                    stream,
                    expected,
                    byte_count: byte_count.to_string(),
                    value: Box::new(value),
                }
            }
            CompilerExpressionKind::Deserialize(stream) => {
                let stream = self.emit_expression(stream, body, environment);
                let LlValue::SerializationStream {
                    stream,
                    expected,
                    byte_count,
                    value,
                } = stream
                else {
                    unreachable!("checked deserialize operand retains its native stream")
                };
                body.effect(
                    &format!(
                        "call void @topal.runtime.serialization.verify(ptr {stream}, ptr {expected}, i64 {byte_count})"
                    ),
                    expression.span,
                    &mut self.debug,
                );
                *value
            }
            CompilerExpressionKind::TaskConstruct { task, initial } => {
                let initial = self.emit_expression(initial, body, environment);
                LlValue::Task {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.task.make(ptr {})",
                            initial.integer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    task: task.clone(),
                }
            }
            CompilerExpressionKind::TaskStateLoad { task, .. } => {
                let task = self.emit_expression(task, body, environment);
                let state = body.instruction(
                    &format!(
                        "call ptr @topal.runtime.task.state.load(ptr {})",
                        task.task_pointer()
                    ),
                    expression.span,
                    &mut self.debug,
                );
                if let CompilerType::TaskResponse(success) = &expression.value_type {
                    debug_assert_eq!(success.as_ref(), &CompilerType::Nat);
                    LlValue::Result {
                        value: body.instruction(
                            &format!("call ptr @topal.runtime.result.success(ptr {state})"),
                            expression.span,
                            &mut self.debug,
                        ),
                        success: success.as_ref().clone(),
                        function_captures: Vec::new(),
                    }
                } else {
                    LlValue::Int(state)
                }
            }
            CompilerExpressionKind::TaskStateReplace { task, value, .. } => {
                let task = self.emit_expression(task, body, environment);
                let value = self.emit_expression(value, body, environment);
                body.effect(
                    &format!(
                        "call void @topal.runtime.task.state.replace(ptr {}, ptr {})",
                        task.task_pointer(),
                        value.integer()
                    ),
                    expression.span,
                    &mut self.debug,
                );
                LlValue::Unit
            }
            CompilerExpressionKind::Completed => LlValue::Completed("0".into()),
            CompilerExpressionKind::Effect => LlValue::Effect("0".into()),
            CompilerExpressionKind::TypeValue(value) => LlValue::Enum {
                value: value.to_string(),
                enumeration: fundamental_type_enumeration(),
            },
            CompilerExpressionKind::Root => LlValue::Enum {
                value: "0".into(),
                enumeration: scope_enumeration(),
            },
            CompilerExpressionKind::LintNamespace => LlValue::Enum {
                value: "1".into(),
                enumeration: scope_enumeration(),
            },
            CompilerExpressionKind::FunctionValue(value) => LlValue::Enum {
                value: value.to_string(),
                enumeration: function_value_enumeration(self.program),
            },
            CompilerExpressionKind::ConstraintValue(value) => LlValue::Enum {
                value: value.to_string(),
                enumeration: constraint_value_enumeration(self.program),
            },
            CompilerExpressionKind::Boolean(value) => LlValue::Boolean(value.to_string()),
            CompilerExpressionKind::Version(value) => self.emit_version_literal(
                [value.major, value.minor, value.patch, value.build],
                value.to_string(),
                body,
                expression.span,
            ),
            CompilerExpressionKind::Int(value) => self.emit_int_literal(value),
            CompilerExpressionKind::IntToModular { value, modular } => {
                let value = self.emit_expression(value, body, environment);
                LlValue::Modular {
                    value: value.integer().to_owned(),
                    modular: modular.clone(),
                }
            }
            CompilerExpressionKind::ModularReduce { value, modular } => {
                let value = self.emit_expression(value, body, environment);
                self.emit_modular_reduce(value.integer(), modular, body, expression.span)
            }
            CompilerExpressionKind::ModularValidate {
                value,
                modular,
                error_span,
            } => self.emit_modular_validation(
                value,
                modular,
                *error_span,
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::Rational(value) => {
                self.emit_rational_literal(value, body, expression.span)
            }
            CompilerExpressionKind::String(value) => {
                self.emit_string_literal(value, body, expression.span)
            }
            CompilerExpressionKind::StringEmpty => {
                LlValue::String(self.emit_string_value("", body, expression.span))
            }
            CompilerExpressionKind::StringConcat { left, right } => {
                let left = self.emit_expression(left, body, environment);
                let right = self.emit_expression(right, body, environment);
                LlValue::String(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.string.concat(ptr {}, ptr {})",
                        left.string(),
                        right.string()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::StringRangeSelect {
                text,
                characters,
                range,
            } => self.emit_string_range_select(
                text,
                characters,
                range,
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::StringEmptyPredicate(value) => {
                let value = self.emit_expression(value, body, environment);
                LlValue::Boolean(body.instruction(
                    &format!(
                        "call i1 @topal.runtime.string.is.empty(ptr {})",
                        value.string()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::StringUtf8ByteCount(value) => {
                let value = self.emit_expression(value, body, environment);
                LlValue::Int(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.string.utf8.byte.count(ptr {})",
                        value.string()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::StringCharacterCount(value) => {
                let value = self.emit_expression(value, body, environment);
                LlValue::Int(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.string.character.count(ptr {})",
                        value.string()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::StringUnicodeScalarValue(value) => {
                let value = self.emit_expression(value, body, environment);
                LlValue::Int(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.string.unicode.scalar.value(ptr {})",
                        value.string()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::CharacterAsciiDecimalDigit(value) => {
                let value = self.emit_expression(value, body, environment);
                LlValue::Optional {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.character.ascii.decimal.digit(ptr {})",
                            value.string()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    payload: CompilerType::Nat,
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::CharacterUnicodeWhitespace(value) => {
                let value = self.emit_expression(value, body, environment);
                LlValue::Boolean(body.instruction(
                    &format!(
                        "call i1 @topal.runtime.character.unicode.whitespace(ptr {})",
                        value.string()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::StringCharactersGenerator { text, .. } => {
                let _ = self.emit_expression(text, body, environment);
                let CompilerType::Generator(generator) = &expression.value_type else {
                    unreachable!("checked characters construction retains its Generator type")
                };
                LlValue::Generator {
                    value: "0".into(),
                    generator: generator.clone(),
                    captured_initial: None,
                    captured_additional_initials: Vec::new(),
                }
            }
            CompilerExpressionKind::StringRangeCharactersGenerator { text, range, .. } => {
                let _ = self.emit_expression(text, body, environment);
                let _ = self.emit_expression(range, body, environment);
                let CompilerType::Generator(generator) = &expression.value_type else {
                    unreachable!("checked range characters construction retains its Generator type")
                };
                LlValue::Generator {
                    value: "0".into(),
                    generator: generator.clone(),
                    captured_initial: None,
                    captured_additional_initials: Vec::new(),
                }
            }
            CompilerExpressionKind::StringRangeCharactersCollect {
                text,
                characters,
                range,
            } => self.emit_string_range_characters_collect(
                text,
                characters,
                range,
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::StringProvenanceCharactersGenerator { text, .. } => {
                let _ = self.emit_expression(text, body, environment);
                let CompilerType::Generator(generator) = &expression.value_type else {
                    unreachable!(
                        "checked provenance characters construction retains its Generator type"
                    )
                };
                LlValue::Generator {
                    value: "0".into(),
                    generator: generator.clone(),
                    captured_initial: None,
                    captured_additional_initials: Vec::new(),
                }
            }
            CompilerExpressionKind::StringProvenanceCharactersCollect { text, characters } => self
                .emit_string_provenance_characters_collect(
                    text,
                    characters,
                    body,
                    environment,
                    expression.span,
                ),
            CompilerExpressionKind::StringDynamicScalarCharactersCollect(text) => {
                let text = self.emit_expression(text, body, environment);
                self.scalar_list_runtime_fragments
                    .insert(ScalarListRuntimeFragment::String);
                LlValue::List {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.string.unicode.scalar.characters(ptr {})",
                            text.string()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    element: CompilerType::Character,
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::StringCharactersCollect { text, .. } => {
                self.emit_expression(text, body, environment)
            }
            CompilerExpressionKind::StringCharactersClose(generator)
            | CompilerExpressionKind::CustomCharacterClose { generator, .. } => {
                let _ = self.emit_expression(generator, body, environment);
                LlValue::Unit
            }
            CompilerExpressionKind::CustomCharacterHandledClose { generator, handler } => self
                .emit_custom_character_handled_close(
                    generator,
                    handler,
                    body,
                    environment,
                    expression.span,
                ),
            CompilerExpressionKind::StringCharactersForeach { .. } => {
                self.emit_string_characters_foreach(expression, body, environment)
            }
            CompilerExpressionKind::CustomCharacterGenerator {
                declaration_span,
                initial_parameter,
                initial,
                prefix,
                ..
            } => {
                let initial = self.emit_expression(initial, body, environment);
                if !prefix.statements.is_empty() {
                    let parent_scope = body.subprogram;
                    body.subprogram = self.debug.lexical_block(*declaration_span, parent_scope);
                    let variable = self.debug.local(
                        &initial_parameter.name,
                        initial_parameter.span,
                        &initial_parameter.value_type,
                        body.subprogram,
                    );
                    let location = self.debug.location(initial_parameter.span, body.subprogram);
                    body.debug_value(&initial, variable, location);
                    let mut prefix_environment = environment.clone();
                    prefix_environment.insert(initial_parameter.name.clone(), initial);
                    let prefix_result = self.emit_block(prefix, body, &mut prefix_environment);
                    debug_assert!(matches!(prefix_result, LlValue::Unit));
                    body.subprogram = parent_scope;
                }
                let CompilerType::Generator(generator) = &expression.value_type else {
                    unreachable!("checked custom construction retains its Generator type")
                };
                LlValue::Generator {
                    value: "0".into(),
                    generator: generator.clone(),
                    captured_initial: None,
                    captured_additional_initials: Vec::new(),
                }
            }
            CompilerExpressionKind::CustomValueGenerator {
                initial,
                additional_initials,
                ..
            } => {
                let initial = self.emit_expression(initial, body, environment);
                let additional_initials = additional_initials
                    .iter()
                    .map(|initial| self.emit_expression(initial, body, environment))
                    .collect();
                let CompilerType::Generator(generator) = &expression.value_type else {
                    unreachable!("checked custom construction retains its Generator type")
                };
                LlValue::Generator {
                    value: "0".into(),
                    generator: generator.clone(),
                    captured_initial: Some(Box::new(initial)),
                    captured_additional_initials: additional_initials,
                }
            }
            CompilerExpressionKind::CustomCharacterForeach {
                source,
                declaration_span,
                characters,
                locals,
                parameter,
                body: action,
                result,
            } => {
                let parent_scope = body.subprogram;
                if !locals.is_empty() {
                    body.subprogram = self.debug.lexical_block(*declaration_span, parent_scope);
                }
                let _ = self.emit_character_sequence_foreach(
                    source,
                    characters,
                    locals.first(),
                    parameter,
                    action,
                    body,
                    environment,
                    expression.span,
                );
                let value = self.emit_expression(result, body, environment);
                body.subprogram = parent_scope;
                value
            }
            CompilerExpressionKind::CustomValueForeach { .. } => {
                self.emit_custom_value_foreach(expression, body, environment)
            }
            CompilerExpressionKind::ErrorCode(value) => LlValue::ErrorCode(value.to_string()),
            CompilerExpressionKind::Enum(value) => {
                let CompilerType::Enum(enumeration) = &expression.value_type else {
                    unreachable!("checked Enum value retains its nominal type")
                };
                LlValue::Enum {
                    value: value.to_string(),
                    enumeration: enumeration.clone(),
                }
            }
            CompilerExpressionKind::IterateGenerator { initial, .. } => {
                let _ = self.emit_expression(initial, body, environment);
                let CompilerType::Generator(generator) = &expression.value_type else {
                    unreachable!("checked iterate construction retains its Generator type")
                };
                LlValue::Generator {
                    value: "0".into(),
                    generator: generator.clone(),
                    captured_initial: None,
                    captured_additional_initials: Vec::new(),
                }
            }
            CompilerExpressionKind::GeneratorTakeWhile { generator, .. } => {
                let _ = self.emit_expression(generator, body, environment);
                let CompilerType::Generator(generator) = &expression.value_type else {
                    unreachable!("checked take-while retains its Generator type")
                };
                LlValue::Generator {
                    value: "0".into(),
                    generator: generator.clone(),
                    captured_initial: None,
                    captured_additional_initials: Vec::new(),
                }
            }
            CompilerExpressionKind::UnfoldGenerator { seed, .. } => {
                let _ = self.emit_expression(seed, body, environment);
                let CompilerType::Generator(generator) = &expression.value_type else {
                    unreachable!("checked unfold construction retains its Generator type")
                };
                LlValue::Generator {
                    value: "0".into(),
                    generator: generator.clone(),
                    captured_initial: None,
                    captured_additional_initials: Vec::new(),
                }
            }
            CompilerExpressionKind::IterateGeneratorForeach {
                generator,
                parameter,
                body: action,
            } => self.emit_iterate_foreach(
                generator,
                parameter,
                action,
                body,
                environment,
                expression.span,
            ),
            CompilerExpressionKind::GeneratorCollect(generator) => {
                if matches!(
                    generator.kind,
                    CompilerExpressionKind::UnfoldGenerator { .. }
                ) {
                    self.emit_unfold_collect(generator, body, environment, expression.span)
                } else {
                    self.emit_iterate_collect(generator, body, environment, expression.span)
                }
            }
            CompilerExpressionKind::Sum { value, payload } => {
                let CompilerType::Sum(sum) = &expression.value_type else {
                    unreachable!("checked sum value retains its nominal type")
                };
                let mut payloads = sum
                    .alternatives
                    .iter()
                    .map(|alternative| {
                        alternative
                            .payload
                            .as_ref()
                            .map(|payload_type| Box::new(zero_machine_value(payload_type)))
                    })
                    .collect::<Vec<_>>();
                if let Some(payload) = payload {
                    payloads[usize::try_from(*value).expect("u32 sum tag fits usize")] =
                        Some(Box::new(self.emit_expression(payload, body, environment)));
                }
                LlValue::Sum {
                    tag: value.to_string(),
                    payloads,
                    sum: sum.clone(),
                }
            }
            CompilerExpressionKind::Tuple(values) => LlValue::Tuple(
                values
                    .iter()
                    .map(|value| self.emit_expression(value, body, environment))
                    .collect(),
            ),
            CompilerExpressionKind::TupleField { tuple, index } => {
                let LlValue::Tuple(values) = self.emit_expression(tuple, body, environment) else {
                    unreachable!("checked tuple field has a Tuple receiver")
                };
                values
                    .into_iter()
                    .nth(*index)
                    .expect("checked tuple field index remains in bounds")
            }
            CompilerExpressionKind::Record(fields) => {
                let mut fields = fields
                    .iter()
                    .map(|(label, value)| {
                        (
                            label.clone(),
                            self.emit_expression(value, body, environment),
                        )
                    })
                    .collect::<Vec<_>>();
                let canonical_labels = match &expression.value_type {
                    CompilerType::Record(field_types) => field_types
                        .iter()
                        .map(|(label, _)| label.as_str())
                        .collect::<Vec<_>>(),
                    _ => unreachable!("checked Record expression retains its Record type"),
                };
                let order = fields
                    .iter()
                    .map(|(label, _)| {
                        canonical_labels
                            .iter()
                            .position(|candidate| candidate == label)
                            .expect("checked Record type retains every value label")
                            .to_string()
                    })
                    .collect();
                fields.sort_by(|left, right| left.0.cmp(&right.0));
                LlValue::Record { fields, order }
            }
            CompilerExpressionKind::RecordReconstruct { base, replacements } => {
                let LlValue::Record { mut fields, order } =
                    self.emit_expression(base, body, environment)
                else {
                    unreachable!("checked reconstruction base has a Record type")
                };
                for (label, replacement) in replacements {
                    let replacement = self.emit_expression(replacement, body, environment);
                    let (_, value) = fields
                        .iter_mut()
                        .find(|(name, _)| name == label)
                        .unwrap_or_else(|| panic!("checked Record retains field `{label}`"));
                    *value = replacement;
                }
                LlValue::Record { fields, order }
            }
            CompilerExpressionKind::RecordField { record, label } => {
                let LlValue::Record { fields, .. } =
                    self.emit_expression(record, body, environment)
                else {
                    unreachable!("checked field selection has a Record receiver")
                };
                fields
                    .into_iter()
                    .find_map(|(name, value)| (name == *label).then_some(value))
                    .unwrap_or_else(|| panic!("checked Record retains field `{label}`"))
            }
            CompilerExpressionKind::Block(block) => {
                let parent_scope = body.subprogram;
                body.subprogram = self.debug.lexical_block(expression.span, parent_scope);
                let mut nested = environment.clone();
                let value = self.emit_block(block, body, &mut nested);
                body.subprogram = parent_scope;
                value
            }
            CompilerExpressionKind::ExitSequence { preceding, result } => {
                for value in preceding {
                    let _ = self.emit_expression(value, body, environment);
                }
                self.emit_expression(result, body, environment)
            }
            CompilerExpressionKind::PrivateBinding {
                storage_name,
                value,
                body: result,
            } => {
                let value = self.emit_expression(value, body, environment);
                let mut nested = environment.clone();
                extend_function_capture_environment(&mut nested, &value);
                nested.insert(storage_name.clone(), value);
                self.emit_expression(result, body, &nested)
            }
            CompilerExpressionKind::Local(name)
            | CompilerExpressionKind::InfinityLocal {
                storage_name: name, ..
            } => environment
                .get(name)
                .unwrap_or_else(|| panic!("checked local `{name}` remains available"))
                .clone(),
            CompilerExpressionKind::Negate(operand) => {
                let emitted = self.emit_expression(operand, body, environment);
                match emitted {
                    LlValue::Int(operand) => LlValue::Int(body.instruction(
                        &format!("call ptr @topal.runtime.int.negate(ptr {operand})"),
                        expression.span,
                        &mut self.debug,
                    )),
                    LlValue::Rational(operand) => LlValue::Rational(body.instruction(
                        &format!("call ptr @topal.runtime.rational.negate(ptr {operand})"),
                        expression.span,
                        &mut self.debug,
                    )),
                    LlValue::Modular { value, modular } => {
                        let value = body.instruction(
                            &format!("call ptr @topal.runtime.int.negate(ptr {value})"),
                            expression.span,
                            &mut self.debug,
                        );
                        self.emit_modular_reduce(&value, &modular, body, expression.span)
                    }
                    _ => unreachable!("checked negate operand is exact numeric"),
                }
            }
            CompilerExpressionKind::Absolute(operand) => {
                let emitted = self.emit_expression(operand, body, environment);
                match emitted {
                    LlValue::Int(operand) => LlValue::Int(body.instruction(
                        &format!("call ptr @topal.runtime.int.absolute(ptr {operand})"),
                        expression.span,
                        &mut self.debug,
                    )),
                    LlValue::Rational(operand) => LlValue::Rational(body.instruction(
                        &format!("call ptr @topal.runtime.rational.absolute(ptr {operand})"),
                        expression.span,
                        &mut self.debug,
                    )),
                    _ => unreachable!("checked absolute operand is exact numeric"),
                }
            }
            CompilerExpressionKind::IntToRational(operand) => {
                let emitted = self.emit_expression(operand, body, environment);
                LlValue::Rational(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.rational.from.int(ptr {})",
                        emitted.integer()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::RationalConstruct {
                numerator,
                denominator,
            } => {
                let numerator = self.emit_expression(numerator, body, environment);
                let denominator = self.emit_expression(denominator, body, environment);
                LlValue::Rational(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.rational.make(ptr {}, ptr {})",
                        numerator.integer(),
                        denominator.integer()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::RationalToInt(value) => {
                let value = self.emit_expression(value, body, environment);
                LlValue::Int(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.rational.numerator(ptr {})",
                        value.rational()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::IntToNat(value)
            | CompilerExpressionKind::ExternalLayoutCoerce { value, .. } => {
                self.emit_expression(value, body, environment)
            }
            CompilerExpressionKind::IntToNatBoundary(value) => {
                self.emit_required_nat(value, body, environment, expression.span)
            }
            CompilerExpressionKind::ResultSuccess(value) => {
                let success = self.emit_expression(value, body, environment);
                let function_captures = match &success {
                    LlValue::Function { captures, .. } => captures.clone(),
                    _ => Vec::new(),
                };
                let payload =
                    self.emit_result_payload(&success, &value.value_type, body, value.span);
                LlValue::Result {
                    value: body.instruction(
                        &format!("call ptr @topal.runtime.result.success(ptr {payload})"),
                        expression.span,
                        &mut self.debug,
                    ),
                    success: value.value_type.clone(),
                    function_captures,
                }
            }
            CompilerExpressionKind::ResultError(value) => {
                let error = self.emit_expression(value, body, environment);
                LlValue::Result {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.result.error(ptr {})",
                            error.error()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    success: match &expression.value_type {
                        CompilerType::Result(success) => success.as_ref().clone(),
                        _ => unreachable!("checked Result Error expression has Result type"),
                    },
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ResultProject(value) => {
                self.emit_result_project(value, body, environment, expression.span, true)
            }
            CompilerExpressionKind::ResultProjectBoundary(value) => {
                self.emit_result_project(value, body, environment, expression.span, false)
            }
            CompilerExpressionKind::OptionalSome(value) => {
                let payload_value = self.emit_expression(value, body, environment);
                let function_captures = match &payload_value {
                    LlValue::Function { captures, .. } => captures.clone(),
                    _ => Vec::new(),
                };
                let payload = self.emit_optional_payload_pointer(
                    &payload_value,
                    &value.value_type,
                    body,
                    value.span,
                );
                LlValue::Optional {
                    value: body.instruction(
                        &format!("call ptr @topal.runtime.optional.some(ptr {payload})"),
                        expression.span,
                        &mut self.debug,
                    ),
                    payload: value.value_type.clone(),
                    function_captures,
                }
            }
            CompilerExpressionKind::OptionalNone => {
                let CompilerType::Optional(payload) = &expression.value_type else {
                    unreachable!("checked None retains its Optional classifier")
                };
                LlValue::Optional {
                    value: body.instruction(
                        "call ptr @topal.runtime.optional.none()",
                        expression.span,
                        &mut self.debug,
                    ),
                    payload: payload.as_ref().clone(),
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::TraversalControl { finish, value } => {
                let payload = self.emit_expression(value, body, environment);
                let control = body.instruction(
                    "call ptr @topal.platform.allocate(i64 16)",
                    expression.span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("store i64 {}, ptr {control}, align 8", u8::from(*finish)),
                    expression.span,
                    &mut self.debug,
                );
                let payload_address = body.instruction(
                    &format!("getelementptr i8, ptr {control}, i64 8"),
                    expression.span,
                    &mut self.debug,
                );
                body.effect(
                    &format!(
                        "store ptr {}, ptr {payload_address}, align 8",
                        payload.integer()
                    ),
                    expression.span,
                    &mut self.debug,
                );
                LlValue::TraversalControl {
                    value: control,
                    payload: CompilerType::Int,
                }
            }
            CompilerExpressionKind::ListEmpty => {
                let CompilerType::List(element) = &expression.value_type else {
                    unreachable!("checked Empty retains its List classifier")
                };
                LlValue::List {
                    value: "null".into(),
                    element: element.as_ref().clone(),
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListEntry { value, remaining } => {
                let value = self.emit_expression(value, body, environment);
                let remaining = self.emit_expression(remaining, body, environment);
                let LlValue::List {
                    value: remaining,
                    element,
                    mut function_captures,
                } = remaining
                else {
                    unreachable!("checked Entry tail retains its List classifier")
                };
                let (allocation_size, next_offset) = match &element {
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
                    | CompilerType::Function => (16, 8),
                    CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::Int => {
                        (16, 8)
                    }
                    CompilerType::Optional(payload)
                        if payload.as_ref() == &CompilerType::Rational =>
                    {
                        (16, 8)
                    }
                    CompilerType::Optional(payload)
                        if payload.as_ref() == &CompilerType::String =>
                    {
                        (16, 8)
                    }
                    CompilerType::Tuple(fields)
                        if matches!(
                            fields.as_slice(),
                            [
                                CompilerType::Int
                                    | CompilerType::Nat
                                    | CompilerType::Character
                                    | CompilerType::String,
                                CompilerType::Int
                                    | CompilerType::Nat
                                    | CompilerType::Character
                                    | CompilerType::String
                            ] | [CompilerType::String, CompilerType::Function]
                        ) =>
                    {
                        (24, 16)
                    }
                    element
                        if compiler_list_integer_pair(element)
                            || compiler_list_string_rational_pair(element) =>
                    {
                        (24, 16)
                    }
                    element if compiler_int_list_pair(element) => (24, 16),
                    element if compiler_boolean_int_pair(element) => (24, 16),
                    element if compiler_boolean_string_pair(element) => (24, 16),
                    element if compiler_three_pointer_tuple(element) => (32, 24),
                    element if compiler_int_int_string_int_tuple(element) => (40, 32),
                    CompilerType::Tuple(fields)
                        if fields.as_slice()
                            == [
                                CompilerType::Int,
                                CompilerType::Int,
                                CompilerType::Boolean,
                                CompilerType::Boolean,
                            ] =>
                    {
                        (32, 24)
                    }
                    CompilerType::Range(endpoint) if endpoint.as_ref() == &CompilerType::Int => {
                        (16, 8)
                    }
                    CompilerType::List(inner)
                        if inner.as_ref() == &CompilerType::Int
                            || inner.as_ref() == &CompilerType::String
                            || compiler_int_string_pair(inner)
                            || compiler_integer_pair(inner) =>
                    {
                        (16, 8)
                    }
                    element if compiler_nested_int_list_element(element) => (16, 8),
                    _ => unreachable!("checked List element has an admitted node layout"),
                };
                let node = body.instruction(
                    &format!("call ptr @topal.platform.allocate(i64 {allocation_size})"),
                    expression.span,
                    &mut self.debug,
                );
                match (&element, &value) {
                    (CompilerType::Unit, LlValue::Unit) => body.effect(
                        &format!("store i8 0, ptr {node}, align 1"),
                        expression.span,
                        &mut self.debug,
                    ),
                    (CompilerType::Completed, LlValue::Completed(value))
                    | (CompilerType::Effect, LlValue::Effect(value)) => body.effect(
                        &format!("store i8 {value}, ptr {node}, align 1"),
                        expression.span,
                        &mut self.debug,
                    ),
                    (CompilerType::Boolean, LlValue::Boolean(value)) => body.effect(
                        &format!("store i1 {value}, ptr {node}, align 1"),
                        expression.span,
                        &mut self.debug,
                    ),
                    (CompilerType::Comparison, LlValue::Comparison(value))
                    | (CompilerType::ErrorCode, LlValue::ErrorCode(value))
                    | (CompilerType::Type, LlValue::Enum { value, .. })
                    | (CompilerType::Enum(_), LlValue::Enum { value, .. })
                    | (
                        CompilerType::Function,
                        LlValue::Function { value, .. } | LlValue::Enum { value, .. },
                    ) => body.effect(
                        &format!("store i32 {value}, ptr {node}, align 4"),
                        expression.span,
                        &mut self.debug,
                    ),
                    (CompilerType::Int | CompilerType::Nat, LlValue::Int(value))
                    | (CompilerType::Rational, LlValue::Rational(value))
                    | (CompilerType::Character | CompilerType::String, LlValue::String(value)) => {
                        body.effect(
                            &format!("store ptr {value}, ptr {node}, align 8"),
                            expression.span,
                            &mut self.debug,
                        );
                    }
                    (
                        CompilerType::Range(endpoint),
                        LlValue::Range {
                            value,
                            endpoint: value_endpoint,
                        },
                    ) if endpoint.as_ref() == value_endpoint => {
                        body.effect(
                            &format!("store ptr {value}, ptr {node}, align 8"),
                            expression.span,
                            &mut self.debug,
                        );
                    }
                    (
                        CompilerType::Modular(modular),
                        LlValue::Modular {
                            value,
                            modular: value_modular,
                        },
                    ) if modular == value_modular => {
                        body.effect(
                            &format!("store ptr {value}, ptr {node}, align 8"),
                            expression.span,
                            &mut self.debug,
                        );
                    }
                    (
                        CompilerType::Optional(payload),
                        LlValue::Optional {
                            value,
                            payload: value_payload,
                            ..
                        },
                    ) if payload.as_ref() == value_payload
                        && matches!(
                            value_payload,
                            CompilerType::Int | CompilerType::Rational | CompilerType::String
                        ) =>
                    {
                        body.effect(
                            &format!("store ptr {value}, ptr {node}, align 8"),
                            expression.span,
                            &mut self.debug,
                        );
                    }
                    (CompilerType::Tuple(field_types), LlValue::Tuple(values))
                        if matches!(
                            field_types.as_slice(),
                            [
                                CompilerType::Int
                                    | CompilerType::Nat
                                    | CompilerType::Character
                                    | CompilerType::String,
                                CompilerType::Int
                                    | CompilerType::Nat
                                    | CompilerType::Character
                                    | CompilerType::String
                            ] | [CompilerType::String, CompilerType::Function]
                        ) =>
                    {
                        let [left, right] = values.as_slice() else {
                            unreachable!("checked List pair value retains two fields")
                        };
                        body.effect(
                            &format!(
                                "store ptr {}, ptr {node}, align 8",
                                left.int_or_string_pointer()
                            ),
                            expression.span,
                            &mut self.debug,
                        );
                        let right_address = body.instruction(
                            &format!("getelementptr i8, ptr {node}, i64 8"),
                            expression.span,
                            &mut self.debug,
                        );
                        match right {
                            LlValue::Function { value, .. } | LlValue::Enum { value, .. } => {
                                body.effect(
                                    &format!("store i32 {value}, ptr {right_address}, align 4"),
                                    expression.span,
                                    &mut self.debug,
                                );
                            }
                            _ => body.effect(
                                &format!(
                                    "store ptr {}, ptr {right_address}, align 8",
                                    right.int_or_string_pointer()
                                ),
                                expression.span,
                                &mut self.debug,
                            ),
                        }
                    }
                    (element, LlValue::Tuple(values))
                        if compiler_list_integer_pair(element)
                            || compiler_list_string_rational_pair(element) =>
                    {
                        let [list, integer] = values.as_slice() else {
                            unreachable!("checked List/list-integer pair retains two fields")
                        };
                        body.effect(
                            &format!("store ptr {}, ptr {node}, align 8", list.list_pointer()),
                            expression.span,
                            &mut self.debug,
                        );
                        let integer_address = body.instruction(
                            &format!("getelementptr i8, ptr {node}, i64 8"),
                            expression.span,
                            &mut self.debug,
                        );
                        body.effect(
                            &format!(
                                "store ptr {}, ptr {integer_address}, align 8",
                                integer.aggregate_pointer()
                            ),
                            expression.span,
                            &mut self.debug,
                        );
                    }
                    (element, LlValue::Tuple(values)) if compiler_int_list_pair(element) => {
                        let [integer, list] = values.as_slice() else {
                            unreachable!("checked integer/List pair retains two fields")
                        };
                        body.effect(
                            &format!("store ptr {}, ptr {node}, align 8", integer.integer()),
                            expression.span,
                            &mut self.debug,
                        );
                        let list_address = body.instruction(
                            &format!("getelementptr i8, ptr {node}, i64 8"),
                            expression.span,
                            &mut self.debug,
                        );
                        body.effect(
                            &format!(
                                "store ptr {}, ptr {list_address}, align 8",
                                list.list_pointer()
                            ),
                            expression.span,
                            &mut self.debug,
                        );
                    }
                    (element, LlValue::Tuple(values)) if compiler_boolean_int_pair(element) => {
                        let [boolean, integer] = values.as_slice() else {
                            unreachable!("checked Boolean/Int pair retains two fields")
                        };
                        body.effect(
                            &format!("store i1 {}, ptr {node}, align 1", boolean.boolean()),
                            expression.span,
                            &mut self.debug,
                        );
                        let integer_address = body.instruction(
                            &format!("getelementptr i8, ptr {node}, i64 8"),
                            expression.span,
                            &mut self.debug,
                        );
                        body.effect(
                            &format!(
                                "store ptr {}, ptr {integer_address}, align 8",
                                integer.integer()
                            ),
                            expression.span,
                            &mut self.debug,
                        );
                    }
                    (element, LlValue::Tuple(values)) if compiler_boolean_string_pair(element) => {
                        let [boolean, string] = values.as_slice() else {
                            unreachable!("checked Boolean/String pair retains two fields")
                        };
                        body.effect(
                            &format!("store i1 {}, ptr {node}, align 1", boolean.boolean()),
                            expression.span,
                            &mut self.debug,
                        );
                        let string_address = body.instruction(
                            &format!("getelementptr i8, ptr {node}, i64 8"),
                            expression.span,
                            &mut self.debug,
                        );
                        body.effect(
                            &format!(
                                "store ptr {}, ptr {string_address}, align 8",
                                string.string()
                            ),
                            expression.span,
                            &mut self.debug,
                        );
                    }
                    (element, LlValue::Tuple(values)) if compiler_three_pointer_tuple(element) => {
                        let [first, second, third] = values.as_slice() else {
                            unreachable!("checked List triple value retains three fields")
                        };
                        for (offset, field) in [(0, first), (8, second), (16, third)] {
                            let address = if offset == 0 {
                                node.clone()
                            } else {
                                body.instruction(
                                    &format!("getelementptr i8, ptr {node}, i64 {offset}"),
                                    expression.span,
                                    &mut self.debug,
                                )
                            };
                            body.effect(
                                &format!(
                                    "store ptr {}, ptr {address}, align 8",
                                    field.aggregate_pointer()
                                ),
                                expression.span,
                                &mut self.debug,
                            );
                        }
                    }
                    (element, LlValue::Tuple(values))
                        if compiler_int_int_string_int_tuple(element) =>
                    {
                        let [first, second, third, fourth] = values.as_slice() else {
                            unreachable!("checked pointer four-tuple retains four fields")
                        };
                        for (offset, field) in [(0, first), (8, second), (16, third), (24, fourth)]
                        {
                            let address = if offset == 0 {
                                node.clone()
                            } else {
                                body.instruction(
                                    &format!("getelementptr i8, ptr {node}, i64 {offset}"),
                                    expression.span,
                                    &mut self.debug,
                                )
                            };
                            body.effect(
                                &format!(
                                    "store ptr {}, ptr {address}, align 8",
                                    field.int_or_string_pointer()
                                ),
                                expression.span,
                                &mut self.debug,
                            );
                        }
                    }
                    (CompilerType::Tuple(field_types), LlValue::Tuple(values))
                        if field_types.as_slice()
                            == [
                                CompilerType::Int,
                                CompilerType::Int,
                                CompilerType::Boolean,
                                CompilerType::Boolean,
                            ] =>
                    {
                        let [first, second, third, fourth] = values.as_slice() else {
                            unreachable!("checked List four-tuple retains four fields")
                        };
                        body.effect(
                            &format!("store ptr {}, ptr {node}, align 8", first.integer()),
                            expression.span,
                            &mut self.debug,
                        );
                        let second_address = body.instruction(
                            &format!("getelementptr i8, ptr {node}, i64 8"),
                            expression.span,
                            &mut self.debug,
                        );
                        body.effect(
                            &format!(
                                "store ptr {}, ptr {second_address}, align 8",
                                second.integer()
                            ),
                            expression.span,
                            &mut self.debug,
                        );
                        let third_address = body.instruction(
                            &format!("getelementptr i8, ptr {node}, i64 16"),
                            expression.span,
                            &mut self.debug,
                        );
                        body.effect(
                            &format!("store i1 {}, ptr {third_address}, align 1", third.boolean()),
                            expression.span,
                            &mut self.debug,
                        );
                        let fourth_address = body.instruction(
                            &format!("getelementptr i8, ptr {node}, i64 17"),
                            expression.span,
                            &mut self.debug,
                        );
                        body.effect(
                            &format!(
                                "store i1 {}, ptr {fourth_address}, align 1",
                                fourth.boolean()
                            ),
                            expression.span,
                            &mut self.debug,
                        );
                    }
                    (
                        CompilerType::List(inner),
                        LlValue::List {
                            value,
                            element: value_element,
                            ..
                        },
                    ) if (matches!(inner.as_ref(), CompilerType::Int | CompilerType::Nat)
                        || inner.as_ref() == &CompilerType::String
                        || compiler_integer_pair(inner)
                        || compiler_int_string_pair(inner)
                        || compiler_nested_int_list_element(inner))
                        && value_element == inner.as_ref() =>
                    {
                        body.effect(
                            &format!("store ptr {value}, ptr {node}, align 8"),
                            expression.span,
                            &mut self.debug,
                        );
                    }
                    _ => unreachable!("checked List element has an admitted node layout"),
                }
                let next = body.instruction(
                    &format!("getelementptr i8, ptr {node}, i64 {next_offset}"),
                    expression.span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("store ptr {remaining}, ptr {next}, align 8"),
                    expression.span,
                    &mut self.debug,
                );
                if element == CompilerType::Function || compiler_string_function_pair(&element) {
                    let captures = match value {
                        LlValue::Function { captures, .. } => captures,
                        LlValue::Enum { .. } => Vec::new(),
                        LlValue::Tuple(mut values) if compiler_string_function_pair(&element) => {
                            match values.pop().expect("checked Map pair has a Function value") {
                                LlValue::Function { captures, .. } => captures,
                                LlValue::Enum { .. } => Vec::new(),
                                _ => unreachable!(
                                    "checked Map pair retains callable captures in its value"
                                ),
                            }
                        }
                        _ => unreachable!("checked List entry retains callable captures"),
                    };
                    function_captures.insert(0, captures);
                }
                LlValue::List {
                    value: node,
                    element,
                    function_captures,
                }
            }
            CompilerExpressionKind::ListContainsEntry { list, value } => {
                let string = matches!(&list.value_type, CompilerType::List(element)
                    if matches!(element.as_ref(), CompilerType::Character | CompilerType::String));
                let rational = matches!(&list.value_type, CompilerType::List(element)
                    if element.as_ref() == &CompilerType::Rational);
                let int_pair = matches!(&list.value_type, CompilerType::List(element)
                    if compiler_integer_pair(element.as_ref()));
                if string {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::String);
                } else if int_pair {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntPair);
                } else {
                    self.list_int_runtime_fragments
                        .insert(ListIntRuntimeFragment::Containment);
                }
                let list = self.emit_expression(list, body, environment);
                let value = self.emit_expression(value, body, environment);
                if int_pair {
                    let LlValue::Tuple(fields) = value else {
                        unreachable!("checked pair containment retains its pair value")
                    };
                    return LlValue::Boolean(body.instruction(
                        &format!(
                            "call i1 @topal.runtime.list.int.pair.contains.entry(ptr {}, ptr {}, ptr {})",
                            list.list_pointer(),
                            fields[0].integer(),
                            fields[1].integer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ));
                }
                LlValue::Boolean(body.instruction(
                    &format!(
                        "call i1 @topal.runtime.list.{}.contains.entry(ptr {}, ptr {})",
                        if string {
                            "string"
                        } else if rational {
                            "rational"
                        } else {
                            "int"
                        },
                        list.list_pointer(),
                        if string {
                            value.string()
                        } else if rational {
                            value.rational()
                        } else {
                            value.integer()
                        }
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::ListContainsSequence { list, pattern }
            | CompilerExpressionKind::ListContainsSubsequence { list, pattern } => {
                let int_pair = matches!(&list.value_type, CompilerType::List(element)
                    if compiler_integer_pair(element.as_ref()));
                let string = matches!(&list.value_type, CompilerType::List(element)
                    if matches!(element.as_ref(), CompilerType::Character | CompilerType::String));
                if int_pair {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntPair);
                } else if string {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::String);
                } else {
                    self.list_int_runtime_fragments
                        .insert(ListIntRuntimeFragment::Containment);
                }
                let operation = if matches!(
                    &expression.kind,
                    CompilerExpressionKind::ListContainsSequence { .. }
                ) {
                    "sequence"
                } else {
                    "subsequence"
                };
                let list = self.emit_expression(list, body, environment);
                let pattern = self.emit_expression(pattern, body, environment);
                LlValue::Boolean(body.instruction(
                    &format!(
                        "call i1 @topal.runtime.list.{}.contains.{operation}(ptr {}, ptr {})",
                        if int_pair {
                            "int.pair"
                        } else if string {
                            "string"
                        } else {
                            "int"
                        },
                        list.list_pointer(),
                        pattern.list_pointer()
                    ),
                    expression.span,
                    &mut self.debug,
                ))
            }
            CompilerExpressionKind::ListRemoveFirst { list, value }
            | CompilerExpressionKind::ListRemoveAll { list, value } => {
                self.list_int_runtime_fragments
                    .insert(ListIntRuntimeFragment::Removal);
                let operation = if matches!(
                    &expression.kind,
                    CompilerExpressionKind::ListRemoveFirst { .. }
                ) {
                    "first"
                } else {
                    "all"
                };
                let list = self.emit_expression(list, body, environment);
                let LlValue::List {
                    value: list,
                    element,
                    ..
                } = list
                else {
                    unreachable!("checked removal subject is a List")
                };
                let value = self.emit_expression(value, body, environment);
                LlValue::List {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.list.int.remove.{operation}(ptr {}, ptr {})",
                            list,
                            value.integer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    element,
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListPrepend { list, value } => {
                let CompilerType::List(element) = &expression.value_type else {
                    unreachable!("checked List prepend retains its List result")
                };
                let list = self.emit_expression(list, body, environment);
                let value = self.emit_expression(value, body, environment);
                let node = body.instruction(
                    "call ptr @topal.platform.allocate(i64 16)",
                    expression.span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("store ptr {}, ptr {node}, align 8", value.integer()),
                    expression.span,
                    &mut self.debug,
                );
                let next = body.instruction(
                    &format!("getelementptr i8, ptr {node}, i64 8"),
                    expression.span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("store ptr {}, ptr {next}, align 8", list.list_pointer()),
                    expression.span,
                    &mut self.debug,
                );
                LlValue::List {
                    value: node,
                    element: element.as_ref().clone(),
                    function_captures: Vec::new(),
                }
            }
            CompilerExpressionKind::ListAppend { list, value } => {
                let CompilerType::List(element) = &expression.value_type else {
                    unreachable!("checked List append retains its List result")
                };
                let string = matches!(&list.value_type, CompilerType::List(element)
                    if element.as_ref() == &CompilerType::String);
                let boolean = element.as_ref() == &CompilerType::Boolean;
                let rational = element.as_ref() == &CompilerType::Rational;
                let rational_nat_pair = compiler_rational_nat_pair(element.as_ref());
                let list_integer_pair = compiler_list_integer_pair(element.as_ref());
                let int_pair = compiler_integer_pair(element.as_ref()) || list_integer_pair;
                let pair_layout = int_pair || rational_nat_pair;
                let int_string_pair = compiler_int_string_pair(element.as_ref());
                let string_pair = compiler_string_pair(element.as_ref());
                let int_list_pair = compiler_int_list_pair(element.as_ref());
                let boolean_string_pair = compiler_boolean_string_pair(element.as_ref());
                let int_int_boolean_pair = compiler_int_int_boolean_pair(element.as_ref());
                let three_pointer = compiler_three_pointer_tuple(element.as_ref());
                let four_pointer = compiler_int_int_string_int_tuple(element.as_ref());
                let nested_int = compiler_nested_int_list_element(element.as_ref());
                let nested_pointer = nested_int
                    || matches!(element.as_ref(), CompilerType::List(inner)
                        if inner.as_ref() == &CompilerType::String
                            || compiler_integer_pair(inner.as_ref()));
                let int_range = matches!(element.as_ref(), CompilerType::Range(endpoint)
                    if endpoint.as_ref() == &CompilerType::Int);
                if pair_layout {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntPair);
                } else if int_string_pair || int_list_pair {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntStringPair);
                } else if string_pair {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::StringPair);
                } else if int_int_boolean_pair {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntIntBooleanPair);
                } else if boolean_string_pair {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::BooleanStringPair);
                } else if three_pointer {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntTriple);
                } else if four_pointer {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::IntIntStringIntTuple);
                } else if string {
                    self.scalar_list_runtime_fragments
                        .insert(ScalarListRuntimeFragment::String);
                } else {
                    self.list_int_runtime_fragments
                        .insert(ListIntRuntimeFragment::Core);
                }
                let list = self.emit_expression(list, body, environment);
                let value = self.emit_expression(value, body, environment);
                let singleton = body.instruction(
                    &format!(
                        "call ptr @topal.platform.allocate(i64 {})",
                        if four_pointer {
                            40
                        } else if int_int_boolean_pair || three_pointer {
                            32
                        } else if pair_layout
                            || int_string_pair
                            || string_pair
                            || int_list_pair
                            || boolean_string_pair
                        {
                            24
                        } else {
                            16
                        }
                    ),
                    expression.span,
                    &mut self.debug,
                );
                let (stored_value, second_value) = if four_pointer {
                    let LlValue::Tuple(fields) = value else {
                        unreachable!("checked pointer four-tuple append retains its tuple value")
                    };
                    let [first, second, third, fourth] = fields.as_slice() else {
                        unreachable!("checked pointer four-tuple append retains four fields")
                    };
                    for (offset, field) in [(16, third), (24, fourth)] {
                        let address = body.instruction(
                            &format!("getelementptr i8, ptr {singleton}, i64 {offset}"),
                            expression.span,
                            &mut self.debug,
                        );
                        body.effect(
                            &format!(
                                "store ptr {}, ptr {address}, align 8",
                                field.int_or_string_pointer()
                            ),
                            expression.span,
                            &mut self.debug,
                        );
                    }
                    (
                        first.int_or_string_pointer().to_owned(),
                        Some(second.int_or_string_pointer().to_owned()),
                    )
                } else if int_int_boolean_pair {
                    let LlValue::Tuple(fields) = value else {
                        unreachable!("checked four-tuple append retains its tuple value")
                    };
                    let [first, second, third, fourth] = fields.as_slice() else {
                        unreachable!("checked four-tuple append retains four fields")
                    };
                    for (offset, boolean) in [(16, third.boolean()), (17, fourth.boolean())] {
                        let address = body.instruction(
                            &format!("getelementptr i8, ptr {singleton}, i64 {offset}"),
                            expression.span,
                            &mut self.debug,
                        );
                        body.effect(
                            &format!("store i1 {boolean}, ptr {address}, align 1"),
                            expression.span,
                            &mut self.debug,
                        );
                    }
                    (
                        first.integer().to_owned(),
                        Some(second.integer().to_owned()),
                    )
                } else if three_pointer {
                    let LlValue::Tuple(fields) = value else {
                        unreachable!("checked triple append retains its tuple value")
                    };
                    let [first, second, third] = fields.as_slice() else {
                        unreachable!("checked triple append retains three fields")
                    };
                    let third_address = body.instruction(
                        &format!("getelementptr i8, ptr {singleton}, i64 16"),
                        expression.span,
                        &mut self.debug,
                    );
                    body.effect(
                        &format!(
                            "store ptr {}, ptr {third_address}, align 8",
                            third.aggregate_pointer()
                        ),
                        expression.span,
                        &mut self.debug,
                    );
                    (
                        first.aggregate_pointer().to_owned(),
                        Some(second.aggregate_pointer().to_owned()),
                    )
                } else if list_integer_pair {
                    let LlValue::Tuple(fields) = value else {
                        unreachable!("checked list/integer pair append retains its tuple value")
                    };
                    (
                        fields[0].list_pointer().to_owned(),
                        Some(fields[1].integer().to_owned()),
                    )
                } else if string_pair {
                    let LlValue::Tuple(fields) = value else {
                        unreachable!("checked String pair append retains its tuple value")
                    };
                    (
                        fields[0].string().to_owned(),
                        Some(fields[1].string().to_owned()),
                    )
                } else if int_pair || int_string_pair || int_list_pair {
                    let LlValue::Tuple(fields) = value else {
                        unreachable!("checked pair append retains its tuple value")
                    };
                    let first = fields[0].integer().to_owned();
                    let second = if int_pair {
                        fields[1].integer()
                    } else if int_list_pair {
                        fields[1].list_pointer()
                    } else {
                        fields[1].string()
                    }
                    .to_owned();
                    (first, Some(second))
                } else if rational_nat_pair {
                    let LlValue::Tuple(fields) = value else {
                        unreachable!("checked Rational/Nat pair append retains its tuple value")
                    };
                    (
                        fields[0].rational().to_owned(),
                        Some(fields[1].integer().to_owned()),
                    )
                } else if boolean_string_pair {
                    let LlValue::Tuple(fields) = value else {
                        unreachable!("checked Boolean/String pair append retains its tuple value")
                    };
                    (
                        fields[0].boolean().to_owned(),
                        Some(fields[1].string().to_owned()),
                    )
                } else if string {
                    (value.string().to_owned(), None)
                } else if boolean {
                    (value.boolean().to_owned(), None)
                } else if rational {
                    (value.rational().to_owned(), None)
                } else if nested_pointer {
                    (value.list_pointer().to_owned(), None)
                } else if int_range {
                    (value.range().0.to_owned(), None)
                } else {
                    (value.integer().to_owned(), None)
                };
                body.effect(
                    &if boolean || boolean_string_pair {
                        format!("store i1 {stored_value}, ptr {singleton}, align 1")
                    } else {
                        format!("store ptr {stored_value}, ptr {singleton}, align 8")
                    },
                    expression.span,
                    &mut self.debug,
                );
                if let Some(second_value) = second_value {
                    let second = body.instruction(
                        &format!("getelementptr i8, ptr {singleton}, i64 8"),
                        expression.span,
                        &mut self.debug,
                    );
                    body.effect(
                        &format!("store ptr {second_value}, ptr {second}, align 8"),
                        expression.span,
                        &mut self.debug,
                    );
                }
                let next = body.instruction(
                    &format!(
                        "getelementptr i8, ptr {singleton}, i64 {}",
                        if four_pointer {
                            32
                        } else if int_int_boolean_pair || three_pointer {
                            24
                        } else if pair_layout
                            || int_string_pair
                            || string_pair
                            || int_list_pair
                            || boolean_string_pair
                        {
                            16
                        } else {
                            8
                        }
                    ),
                    expression.span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("store ptr null, ptr {next}, align 8"),
                    expression.span,
                    &mut self.debug,
                );
                LlValue::List {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.list.{}.concat(ptr {}, ptr {singleton})",
                            if four_pointer {
                                "int-int-string-int"
                            } else if pair_layout {
                                "int.pair"
                            } else if int_string_pair || int_list_pair {
                                "int-string"
                            } else if string_pair {
                                "string-pair"
                            } else if int_int_boolean_pair {
                                "int-int-boolean-pair"
                            } else if boolean_string_pair {
                                "boolean-string"
                            } else if three_pointer {
                                "int.triple"
                            } else if string {
                                "string"
                            } else {
                                "int"
                            },
                            list.list_pointer()
                        ),
                        expression.span,
                        &mut self.debug,
                    ),
                    element: element.as_ref().clone(),
                    function_captures: Vec::new(),
                }
            }
            _ => self.emit_expression_remaining(expression, body, environment),
        }
    }
}
