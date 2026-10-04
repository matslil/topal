impl Generator<'_> {
    fn emit_string_range_characters_collect(
        &mut self,
        text: &CompilerExpression,
        characters: &[String],
        range: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let _ = self.emit_expression(text, body, environment);
        let range = self.emit_expression(range, body, environment);
        let range = range.range().0.to_owned();
        self.scalar_list_runtime_fragments
            .insert(ScalarListRuntimeFragment::String);
        let mut selected = "null".to_owned();
        for (index, character) in characters.iter().enumerate().rev() {
            let character = self.emit_string_value(character, body, span);
            let node = body.instruction(
                "call ptr @topal.platform.allocate(i64 16)",
                span,
                &mut self.debug,
            );
            body.effect(
                &format!("store ptr {character}, ptr {node}, align 8"),
                span,
                &mut self.debug,
            );
            let next = body.instruction(
                &format!("getelementptr i8, ptr {node}, i64 8"),
                span,
                &mut self.debug,
            );
            body.effect(
                &format!("store ptr {selected}, ptr {next}, align 8"),
                span,
                &mut self.debug,
            );
            let index_value = body.instruction(
                &format!("call ptr @topal.runtime.int.from.u64(i64 {index})"),
                span,
                &mut self.debug,
            );
            let contains = body.instruction(
                &format!(
                    "call i1 @topal.runtime.range.int.contains(ptr {range}, ptr {index_value})"
                ),
                span,
                &mut self.debug,
            );
            selected = body.instruction(
                &format!("select i1 {contains}, ptr {node}, ptr {selected}"),
                span,
                &mut self.debug,
            );
        }
        LlValue::List {
            value: selected,
            element: CompilerType::Character,
            function_captures: Vec::new(),
        }
    }

    #[allow(clippy::too_many_lines)] // Candidate matching and list construction share one backwards scan.
    fn emit_string_provenance_characters_collect(
        &mut self,
        text: &CompilerExpression,
        characters: &[String],
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let text = self.emit_expression(text, body, environment);
        self.scalar_list_runtime_fragments
            .insert(ScalarListRuntimeFragment::String);
        if characters.is_empty() {
            return LlValue::List {
                value: "null".into(),
                element: CompilerType::Character,
                function_captures: Vec::new(),
            };
        }
        let data = body.instruction(
            &format!("load ptr, ptr {}, align 8", text.string()),
            span,
            &mut self.debug,
        );
        let length_address = body.instruction(
            &format!("getelementptr i8, ptr {}, i64 8", text.string()),
            span,
            &mut self.debug,
        );
        let length = body.instruction(
            &format!("load i64, ptr {length_address}, align 8"),
            span,
            &mut self.debug,
        );
        let mut candidates = characters.to_vec();
        candidates.sort_by_key(|character| std::cmp::Reverse(character.len()));
        candidates.dedup();

        let preheader = body.current_block.clone();
        let loop_label = body.label("string.characters.collect.loop");
        let done = body.label("string.characters.collect.done");
        let invalid = body.label("string.characters.collect.invalid");
        let checks = candidates
            .iter()
            .map(|_| body.label("string.characters.collect.check"))
            .collect::<Vec<_>>();
        let tests = candidates
            .iter()
            .map(|_| body.label("string.characters.collect.test"))
            .collect::<Vec<_>>();
        let matched = candidates
            .iter()
            .map(|_| body.label("string.characters.collect.matched"))
            .collect::<Vec<_>>();
        let next_offsets = candidates
            .iter()
            .map(|_| body.reserve_value())
            .collect::<Vec<_>>();
        let nodes = candidates
            .iter()
            .map(|_| body.reserve_value())
            .collect::<Vec<_>>();
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let offset_incoming = matched
            .iter()
            .zip(&next_offsets)
            .map(|(block, value)| format!("[{value}, %{block}]"))
            .collect::<Vec<_>>()
            .join(", ");
        let head_incoming = matched
            .iter()
            .zip(&nodes)
            .map(|(block, value)| format!("[{value}, %{block}]"))
            .collect::<Vec<_>>()
            .join(", ");
        let offset = body.instruction(
            &format!("phi i64 [{length}, %{preheader}], {offset_incoming}"),
            span,
            &mut self.debug,
        );
        let head = body.instruction(
            &format!("phi ptr [null, %{preheader}], {head_incoming}"),
            span,
            &mut self.debug,
        );
        let empty = body.instruction(&format!("icmp eq i64 {offset}, 0"), span, &mut self.debug);
        body.terminator(
            &format!("br i1 {empty}, label %{done}, label %{}", checks[0]),
            location,
        );

        for (index, character) in candidates.iter().enumerate() {
            let next_check = checks.get(index + 1).unwrap_or(&invalid);
            body.start_block(&checks[index]);
            let enough = body.instruction(
                &format!("icmp uge i64 {offset}, {}", character.len()),
                span,
                &mut self.debug,
            );
            body.terminator(
                &format!(
                    "br i1 {enough}, label %{}, label %{next_check}",
                    tests[index]
                ),
                location,
            );

            body.start_block(&tests[index]);
            let start = body.instruction(
                &format!("sub i64 {offset}, {}", character.len()),
                span,
                &mut self.debug,
            );
            let mut equal = None;
            for (byte_index, byte) in character.as_bytes().iter().enumerate() {
                let address = body.instruction(
                    &format!("getelementptr i8, ptr {data}, i64 {start}"),
                    span,
                    &mut self.debug,
                );
                let address = if byte_index == 0 {
                    address
                } else {
                    body.instruction(
                        &format!("getelementptr i8, ptr {address}, i64 {byte_index}"),
                        span,
                        &mut self.debug,
                    )
                };
                let actual = body.instruction(
                    &format!("load i8, ptr {address}, align 1"),
                    span,
                    &mut self.debug,
                );
                let byte_equal = body.instruction(
                    &format!("icmp eq i8 {actual}, {byte}"),
                    span,
                    &mut self.debug,
                );
                equal = Some(match equal {
                    None => byte_equal,
                    Some(previous) => body.instruction(
                        &format!("and i1 {previous}, {byte_equal}"),
                        span,
                        &mut self.debug,
                    ),
                });
            }
            body.terminator(
                &format!(
                    "br i1 {}, label %{}, label %{next_check}",
                    equal.expect("grapheme candidates are nonempty"),
                    matched[index]
                ),
                location,
            );

            body.start_block(&matched[index]);
            let character_value = self.emit_string_value(character, body, span);
            body.define_reserved(
                &nodes[index],
                "call ptr @topal.platform.allocate(i64 16)",
                span,
                &mut self.debug,
            );
            body.effect(
                &format!("store ptr {character_value}, ptr {}, align 8", nodes[index]),
                span,
                &mut self.debug,
            );
            let next = body.instruction(
                &format!("getelementptr i8, ptr {}, i64 8", nodes[index]),
                span,
                &mut self.debug,
            );
            body.effect(
                &format!("store ptr {head}, ptr {next}, align 8"),
                span,
                &mut self.debug,
            );
            body.define_reserved(
                &next_offsets[index],
                &format!("sub i64 {offset}, {}", character.len()),
                span,
                &mut self.debug,
            );
            body.terminator(&format!("br label %{loop_label}"), location);
        }

        body.start_block(&invalid);
        body.terminator("unreachable", location);
        body.start_block(&done);
        LlValue::List {
            value: head,
            element: CompilerType::Character,
            function_captures: Vec::new(),
        }
    }

    fn emit_string_characters_foreach(
        &mut self,
        traversal: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
    ) -> LlValue {
        let CompilerExpressionKind::StringCharactersForeach {
            source,
            characters,
            parameter,
            body: action,
        } = &traversal.kind
        else {
            unreachable!("checked Character foreach retains its traversal")
        };
        self.emit_character_sequence_foreach(
            source,
            characters,
            None,
            parameter,
            action,
            body,
            environment,
            traversal.span,
        )
    }

    #[allow(clippy::too_many_arguments)] // The checked traversal pieces remain explicit at lowering.
    fn emit_character_sequence_foreach(
        &mut self,
        source: &CompilerExpression,
        characters: &[String],
        generator_local: Option<&CompilerGeneratorLocal>,
        parameter: &CompilerParameter,
        action: &CompilerBlock,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let _ = self.emit_expression(source, body, environment);
        let mut generator_local_address = None;
        let debug_address = (!characters.is_empty() && !parameter.discarded).then(|| {
            let variable = self.debug.local(
                &parameter.name,
                parameter.span,
                &parameter.value_type,
                body.subprogram,
            );
            let address = body.instruction("alloca ptr, align 8", span, &mut self.debug);
            let location = self.debug.location(parameter.span, body.subprogram);
            body.debug_declare(&address, variable, location);
            address
        });
        for (index, character) in characters.iter().enumerate() {
            if let Some(local) = generator_local
                && local.parameter.value_type == CompilerType::Character
                && local.activation_after_resumptions == index
            {
                let parameter = &local.parameter;
                let variable = self.debug.local(
                    &parameter.name,
                    parameter.span,
                    &parameter.value_type,
                    body.subprogram,
                );
                let address =
                    body.instruction("alloca ptr, align 8", parameter.span, &mut self.debug);
                let location = self.debug.location(parameter.span, body.subprogram);
                body.debug_declare(&address, variable, location);
                generator_local_address = Some(address);
            }
            let value = LlValue::String(self.emit_string_value(character, body, span));
            let mut action_environment = environment.clone();
            if let Some(local) = generator_local
                && local.parameter.value_type == CompilerType::Character
                && local.activation_after_resumptions == index
                && let Some(address) = &generator_local_address
            {
                body.effect(
                    &format!("store ptr {}, ptr {address}, align 8", value.string()),
                    local.parameter.span,
                    &mut self.debug,
                );
            }
            if let Some(address) = &debug_address {
                body.effect(
                    &format!("store ptr {}, ptr {address}, align 8", value.string()),
                    parameter.span,
                    &mut self.debug,
                );
            }
            if !parameter.discarded {
                action_environment.insert(parameter.name.clone(), value);
            }
            let action_value = self.emit_block(action, body, &mut action_environment);
            debug_assert!(matches!(action_value, LlValue::Unit));
            if let Some(local) = generator_local
                && local.parameter.value_type == CompilerType::Unit
                && local.activation_after_resumptions == index + 1
            {
                let parameter = &local.parameter;
                let variable = self.debug.local(
                    &parameter.name,
                    parameter.span,
                    &parameter.value_type,
                    body.subprogram,
                );
                let address =
                    body.instruction("alloca i8, align 1", parameter.span, &mut self.debug);
                let location = self.debug.location(parameter.span, body.subprogram);
                body.debug_declare(&address, variable, location);
                body.effect(
                    &format!("store i8 0, ptr {address}, align 1"),
                    parameter.span,
                    &mut self.debug,
                );
            }
        }
        LlValue::Unit
    }

    fn emit_custom_value_action_debug_address(
        &mut self,
        parameter: &CompilerParameter,
        span: Span,
        body: &mut FunctionBody,
    ) -> (String, u64) {
        let variable = self.debug.local(
            &parameter.name,
            parameter.span,
            &parameter.value_type,
            body.subprogram,
        );
        let llvm_type = llvm_value_type(&parameter.value_type);
        let alignment = target_value_layout(&parameter.value_type).alignment / 8;
        let address = body.instruction(
            &format!("alloca {llvm_type}, align {alignment}"),
            span,
            &mut self.debug,
        );
        let location = self.debug.location(parameter.span, body.subprogram);
        body.debug_declare(&address, variable, location);
        (address, alignment)
    }

    #[allow(clippy::too_many_arguments)] // Final source scope keeps every checked value explicit.
    fn emit_custom_value_final_debug(
        &mut self,
        declaration_span: Span,
        initial_parameter: &CompilerParameter,
        initial: &LlValue,
        explicit_return: Option<Span>,
        result_span: Span,
        traversal_span: Span,
        body: &mut FunctionBody,
    ) {
        if initial_parameter.value_type == CompilerType::Unit {
            let address = body.instruction("alloca i8, align 1", traversal_span, &mut self.debug);
            body.effect(
                &format!("store i8 0, ptr {address}, align 1"),
                traversal_span,
                &mut self.debug,
            );
            body.subprogram = self.debug.lexical_block(declaration_span, body.subprogram);
            let variable = self.debug.local(
                &initial_parameter.name,
                initial_parameter.span,
                &initial_parameter.value_type,
                body.subprogram,
            );
            let location = self.debug.location(result_span, body.subprogram);
            body.debug_declare(&address, variable, location);
            body.effect(
                &format!("store i8 0, ptr {address}, align 1"),
                result_span,
                &mut self.debug,
            );
        } else if explicit_return.is_some()
            || matches!(
                initial_parameter.value_type,
                CompilerType::Int
                    | CompilerType::Nat
                    | CompilerType::Comparison
                    | CompilerType::Enum(_)
                    | CompilerType::Rational
                    | CompilerType::Optional(_)
                    | CompilerType::Range(_)
                    | CompilerType::Result(_)
                    | CompilerType::List(_)
                    | CompilerType::Tuple(_)
            )
        {
            let llvm_type = llvm_value_type(&initial_parameter.value_type);
            let alignment = target_value_layout(&initial_parameter.value_type).alignment / 8;
            let address = body.instruction(
                &format!("alloca {llvm_type}, align {alignment}"),
                traversal_span,
                &mut self.debug,
            );
            let initial_operand = self.emit_machine_operand(
                initial,
                &initial_parameter.value_type,
                body,
                traversal_span,
            );
            body.effect(
                &format!("store {initial_operand}, ptr {address}, align {alignment}"),
                traversal_span,
                &mut self.debug,
            );
            body.subprogram = self.debug.lexical_block(declaration_span, body.subprogram);
            let variable = self.debug.local(
                &initial_parameter.name,
                initial_parameter.span,
                &initial_parameter.value_type,
                body.subprogram,
            );
            let location = self
                .debug
                .location(explicit_return.unwrap_or(result_span), body.subprogram);
            body.debug_declare(&address, variable, location);
            if matches!(
                initial_parameter.value_type,
                CompilerType::Comparison
                    | CompilerType::Enum(_)
                    | CompilerType::Result(_)
                    | CompilerType::List(_)
                    | CompilerType::Tuple(_)
            ) {
                body.effect(
                    &format!("store {initial_operand}, ptr {address}, align {alignment}"),
                    result_span,
                    &mut self.debug,
                );
            }
        } else if initial_parameter.value_type == CompilerType::Boolean {
            body.subprogram = self.debug.lexical_block(declaration_span, body.subprogram);
            let variable = self.debug.local(
                &initial_parameter.name,
                initial_parameter.span,
                &initial_parameter.value_type,
                body.subprogram,
            );
            let location = self.debug.location(result_span, body.subprogram);
            body.debug_value(initial, variable, location);
        }
    }

    #[allow(clippy::too_many_lines)] // Ordered captures, suspensions, action scopes, and final-value debug lifetimes remain adjacent.
    fn emit_custom_value_foreach(
        &mut self,
        traversal: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
    ) -> LlValue {
        let CompilerExpressionKind::CustomValueForeach {
            source,
            transferred_initial,
            declaration_span,
            initial_parameter,
            additional_initial_parameters,
            prefix,
            yields,
            continuations,
            explicit_return,
            parameter,
            body: action,
            result,
        } = &traversal.kind
        else {
            unreachable!("checked value foreach retains its traversal")
        };
        let source = self.emit_expression(source, body, environment);
        let initial = if let Some(transferred_initial) = transferred_initial {
            self.emit_expression(transferred_initial, body, environment)
        } else {
            source.generator_initial().clone()
        };
        let additional_initials = source.generator_additional_initials().to_vec();
        let traversal_parent_scope = body.subprogram;
        let mut generator_environment = environment.clone();
        generator_environment.insert(initial_parameter.name.clone(), initial.clone());
        for (parameter, value) in additional_initial_parameters
            .iter()
            .zip(&additional_initials)
        {
            generator_environment.insert(parameter.name.clone(), value.clone());
        }
        if !additional_initial_parameters.is_empty() || !prefix.statements.is_empty() {
            body.subprogram = self
                .debug
                .lexical_block(*declaration_span, traversal_parent_scope);
            for (parameter, value) in std::iter::once(initial_parameter.as_ref())
                .chain(additional_initial_parameters)
                .zip(std::iter::once(&initial).chain(&additional_initials))
            {
                let variable = self.debug.local(
                    &parameter.name,
                    parameter.span,
                    &parameter.value_type,
                    body.subprogram,
                );
                let llvm_type = llvm_value_type(&parameter.value_type);
                let alignment = target_value_layout(&parameter.value_type).alignment / 8;
                let address = body.instruction(
                    &format!("alloca {llvm_type}, align {alignment}"),
                    parameter.span,
                    &mut self.debug,
                );
                let location = self.debug.location(parameter.span, body.subprogram);
                body.debug_declare(&address, variable, location);
                let operand =
                    self.emit_machine_operand(value, &parameter.value_type, body, parameter.span);
                body.effect(
                    &format!("store {operand}, ptr {address}, align {alignment}"),
                    parameter.span,
                    &mut self.debug,
                );
            }
            let prefix_value = self.emit_block(prefix, body, &mut generator_environment);
            debug_assert!(matches!(prefix_value, LlValue::Unit));
        }
        let continuation_debug = (!continuations.is_empty()).then(|| {
            let scope = self.debug.lexical_block(*declaration_span, body.subprogram);
            let variable = self.debug.local(
                &initial_parameter.name,
                initial_parameter.span,
                &initial_parameter.value_type,
                scope,
            );
            (scope, variable)
        });
        let generator_scope = body.subprogram;
        let action_scope = if additional_initial_parameters.is_empty() {
            generator_scope
        } else {
            self.debug
                .lexical_block(parameter.span, traversal_parent_scope)
        };
        let debug_address = (!yields.is_empty() && !parameter.discarded).then(|| {
            body.subprogram = action_scope;
            let address =
                self.emit_custom_value_action_debug_address(parameter, traversal.span, body);
            body.subprogram = generator_scope;
            address
        });
        for (index, yielded) in yields.iter().enumerate() {
            let (value, span) = match yielded {
                CompilerGeneratorYield::Initial(span) => (initial.clone(), *span),
                CompilerGeneratorYield::Value(value) => (
                    self.emit_expression(value, body, &generator_environment),
                    value.span,
                ),
            };
            if let Some((address, alignment)) = &debug_address {
                let operand = self.emit_machine_operand(&value, &parameter.value_type, body, span);
                let store = format!("store {operand}, ptr {address}, align {alignment}");
                body.effect(&store, span, &mut self.debug);
                if parameter.value_type == CompilerType::Boolean {
                    body.effect(&store, parameter.span, &mut self.debug);
                } else if matches!(
                    parameter.value_type,
                    CompilerType::Comparison
                        | CompilerType::Enum(_)
                        | CompilerType::Result(_)
                        | CompilerType::List(_)
                        | CompilerType::Tuple(_)
                ) {
                    let action_span =
                        action
                            .statements
                            .first()
                            .map_or(action.result.span, |statement| match statement {
                                CompilerStatement::Binding(binding) => binding.span,
                                CompilerStatement::Discard(value) => value.span,
                            });
                    body.effect(&store, action_span, &mut self.debug);
                } else if parameter.value_type == CompilerType::Unit {
                    body.effect(&store, action.result.span, &mut self.debug);
                }
            }
            let mut action_environment = environment.clone();
            if !parameter.discarded {
                action_environment.insert(parameter.name.clone(), value);
            }
            body.subprogram = action_scope;
            let action_value = self.emit_block(action, body, &mut action_environment);
            debug_assert!(matches!(action_value, LlValue::Unit));
            body.subprogram = generator_scope;
            if let Some(continuation) = continuations
                .iter()
                .find(|continuation| continuation.after_resumptions == index + 1)
            {
                let parent_scope = body.subprogram;
                if let Some((scope, variable)) = continuation_debug {
                    body.subprogram = scope;
                    let location = self.debug.location(initial_parameter.span, body.subprogram);
                    body.debug_value(&initial, variable, location);
                }
                let mut continuation_environment = generator_environment.clone();
                let continuation_value =
                    self.emit_block(&continuation.body, body, &mut continuation_environment);
                debug_assert!(matches!(continuation_value, LlValue::Unit));
                body.subprogram = parent_scope;
            }
        }
        if additional_initial_parameters.is_empty() {
            self.emit_custom_value_final_debug(
                *declaration_span,
                initial_parameter,
                &initial,
                *explicit_return,
                result.span,
                traversal.span,
                body,
            );
        }
        let result_environment = generator_environment;
        let result = self.emit_expression(result, body, &result_environment);
        body.subprogram = traversal_parent_scope;
        result
    }

    fn emit_int_literal(&mut self, value: &BigInt) -> LlValue {
        LlValue::Int(self.emit_int_global(value))
    }

    fn emit_version_literal(
        &mut self,
        components: [u64; 4],
        display: String,
        body: &mut FunctionBody,
        span: Span,
    ) -> LlValue {
        let fields = components
            .into_iter()
            .map(|component| self.emit_int_global(&BigInt::from(component)))
            .collect::<Vec<_>>();
        let llvm_type = "{ ptr, ptr, ptr, ptr }";
        let value = body.instruction(
            &format!("alloca {llvm_type}, align 8"),
            span,
            &mut self.debug,
        );
        for (index, field) in fields.iter().enumerate() {
            let address = body.instruction(
                &format!("getelementptr {llvm_type}, ptr {value}, i32 0, i32 {index}"),
                span,
                &mut self.debug,
            );
            body.effect(
                &format!("store ptr {field}, ptr {address}, align 8"),
                span,
                &mut self.debug,
            );
        }
        LlValue::Version { value, display }
    }

    fn emit_int_global(&mut self, value: &BigInt) -> String {
        let (sign, limbs) = value.to_u32_digits();
        let negative = usize::from(sign == Sign::Minus);
        let name = format!(".topal.int.{}", self.next_global);
        self.next_global += 1;
        let values = limbs
            .iter()
            .map(|limb| format!("i32 {limb}"))
            .collect::<Vec<_>>()
            .join(", ");
        self.globals.push(format!(
            "@{name} = private unnamed_addr constant {{ i64, i64, [{} x i32] }} {{ i64 {negative}, i64 {}, [{} x i32] [{values}] }}, align 8",
            limbs.len(),
            limbs.len(),
            limbs.len()
        ));
        format!("@{name}")
    }

    fn emit_modular_reduce(
        &mut self,
        value: &str,
        modular: &CompilerModularType,
        body: &mut FunctionBody,
        span: Span,
    ) -> LlValue {
        let lower = self.emit_int_global(&modular.lower);
        let modulus = self.emit_int_global(&(&modular.upper - &modular.lower + BigInt::from(1)));
        let shifted = body.instruction(
            &format!("call ptr @topal.runtime.int.subtract(ptr {value}, ptr {lower})"),
            span,
            &mut self.debug,
        );
        let residue = body.instruction(
            &format!("call ptr @topal.runtime.int.modulo(ptr {shifted}, ptr {modulus})"),
            span,
            &mut self.debug,
        );
        let canonical = body.instruction(
            &format!("call ptr @topal.runtime.int.add(ptr {residue}, ptr {lower})"),
            span,
            &mut self.debug,
        );
        LlValue::Modular {
            value: canonical,
            modular: modular.clone(),
        }
    }

    fn emit_rational_literal(
        &mut self,
        value: &BigRational,
        body: &mut FunctionBody,
        span: Span,
    ) -> LlValue {
        let numerator = self.emit_int_global(value.numer());
        let denominator = self.emit_int_global(value.denom());
        LlValue::Rational(body.instruction(
            &format!("call ptr @topal.runtime.rational.raw(ptr {numerator}, ptr {denominator})"),
            span,
            &mut self.debug,
        ))
    }
}
