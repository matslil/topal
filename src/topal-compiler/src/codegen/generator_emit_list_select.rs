impl Generator<'_> {
    #[allow(clippy::too_many_arguments, clippy::too_many_lines)] // Selection publishes only fresh retained nodes.
    fn emit_list_select(
        &mut self,
        list: &CompilerExpression,
        parameters: &[CompilerParameter],
        predicate: &CompilerBlock,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
        reject: bool,
        indexes: bool,
    ) -> LlValue {
        let CompilerType::List(element) = &list.value_type else {
            unreachable!("checked selection retains its List source")
        };
        let element = element.as_ref().clone();
        let source = self
            .emit_expression(list, body, environment)
            .list_pointer()
            .to_owned();
        let preheader = body.current_block.clone();
        let loop_label = body.label("list.select.loop");
        let visit = body.label("list.select.visit");
        let selected = body.label("list.select.selected");
        let skipped = body.label("list.select.skipped");
        let first = body.label("list.select.first");
        let link = body.label("list.select.link");
        let selected_merge = body.label("list.select.selected.merge");
        let advance = body.label("list.select.advance");
        let done = body.label("list.select.done");
        let next = body.reserve_value();
        let node = body.reserve_value();
        let selected_head = body.reserve_value();
        let next_head = body.reserve_value();
        let next_previous = body.reserve_value();
        let next_index = body.reserve_value();
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let current = body.instruction(
            &format!("phi ptr [{source}, %{preheader}], [{next}, %{advance}]"),
            span,
            &mut self.debug,
        );
        let head = body.instruction(
            &format!("phi ptr [null, %{preheader}], [{next_head}, %{advance}]"),
            span,
            &mut self.debug,
        );
        let previous = body.instruction(
            &format!("phi ptr [null, %{preheader}], [{next_previous}, %{advance}]"),
            span,
            &mut self.debug,
        );
        let index = body.instruction(
            &format!("phi i64 [0, %{preheader}], [{next_index}, %{advance}]"),
            span,
            &mut self.debug,
        );
        let empty = body.instruction(
            &format!("icmp eq ptr {current}, null"),
            span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {empty}, label %{done}, label %{visit}"),
            location,
        );

        body.start_block(&visit);
        let value = body.instruction(
            &format!("load ptr, ptr {current}, align 8"),
            span,
            &mut self.debug,
        );
        let next_address = body.instruction(
            &format!("getelementptr i8, ptr {current}, i64 8"),
            span,
            &mut self.debug,
        );
        body.define_reserved(
            &next,
            &format!("load ptr, ptr {next_address}, align 8"),
            span,
            &mut self.debug,
        );
        let predicate_value = if indexes {
            LlValue::Int(body.instruction(
                &format!("call ptr @topal.runtime.int.from.u64(i64 {index})"),
                span,
                &mut self.debug,
            ))
        } else if compiler_nested_int_list_element(&element) {
            LlValue::List {
                value: value.clone(),
                element: CompilerType::Int,
                function_captures: Vec::new(),
            }
        } else {
            LlValue::Int(value.clone())
        };
        let mut predicate_environment =
            self.emit_collection_environment(parameters, &[predicate_value], body, environment);
        let keep = self.emit_block(predicate, body, &mut predicate_environment);
        let keep = if reject {
            body.instruction(
                &format!("xor i1 {}, true", keep.boolean()),
                span,
                &mut self.debug,
            )
        } else {
            keep.boolean().to_owned()
        };
        body.terminator(
            &format!("br i1 {keep}, label %{selected}, label %{skipped}"),
            location,
        );

        body.start_block(&skipped);
        body.terminator(&format!("br label %{advance}"), location);
        body.start_block(&selected);
        body.define_reserved(
            &node,
            "call ptr @topal.platform.allocate(i64 16)",
            span,
            &mut self.debug,
        );
        body.effect(
            &format!("store ptr {value}, ptr {node}, align 8"),
            span,
            &mut self.debug,
        );
        let node_next = body.instruction(
            &format!("getelementptr i8, ptr {node}, i64 8"),
            span,
            &mut self.debug,
        );
        body.effect(
            &format!("store ptr null, ptr {node_next}, align 8"),
            span,
            &mut self.debug,
        );
        let has_previous = body.instruction(
            &format!("icmp ne ptr {previous}, null"),
            span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {has_previous}, label %{link}, label %{first}"),
            location,
        );

        body.start_block(&first);
        body.terminator(&format!("br label %{selected_merge}"), location);
        body.start_block(&link);
        let previous_next = body.instruction(
            &format!("getelementptr i8, ptr {previous}, i64 8"),
            span,
            &mut self.debug,
        );
        body.effect(
            &format!("store ptr {node}, ptr {previous_next}, align 8"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{selected_merge}"), location);
        body.start_block(&selected_merge);
        body.define_reserved(
            &selected_head,
            &format!("phi ptr [{node}, %{first}], [{head}, %{link}]"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{advance}"), location);

        body.start_block(&advance);
        body.define_reserved(
            &next_head,
            &format!("phi ptr [{head}, %{skipped}], [{selected_head}, %{selected_merge}]"),
            span,
            &mut self.debug,
        );
        body.define_reserved(
            &next_previous,
            &format!("phi ptr [{previous}, %{skipped}], [{node}, %{selected_merge}]"),
            span,
            &mut self.debug,
        );
        body.define_reserved(
            &next_index,
            &format!("add i64 {index}, 1"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&done);
        LlValue::List {
            value: head,
            element,
            function_captures: Vec::new(),
        }
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)] // Fold control flow and enclosing LLVM state remain explicit.
    fn emit_list_fold(
        &mut self,
        list: &CompilerExpression,
        initial: &CompilerExpression,
        parameters: &[CompilerParameter],
        action: &CompilerBlock,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        // The private String-pair node layout is two pointer fields followed by
        // the next-node pointer. Keep this specialization explicit until the
        // general aggregate-list layout machinery is available.
        let string_pair_optional_fold = matches!(
            (&list.value_type, &initial.value_type),
            (
                CompilerType::List(element),
                CompilerType::Optional(payload)
            ) if element.as_ref()
                == &CompilerType::Tuple(vec![CompilerType::String, CompilerType::String])
                && payload.as_ref() == &CompilerType::String
        );
        let integer_pair_optional_nat_fold = matches!(
            (&list.value_type, &initial.value_type),
            (CompilerType::List(element), CompilerType::Optional(payload))
                if compiler_integer_pair(element.as_ref())
                    && payload.as_ref() == &CompilerType::Nat
        );
        let nested_int_optional_nat_fold = matches!(
            (&list.value_type, &initial.value_type),
            (CompilerType::List(element), CompilerType::Optional(payload))
                if compiler_nested_int_list_element(element.as_ref())
                    && payload.as_ref() == &CompilerType::Nat
        );
        let generic_optional_state_fold = matches!(
            (&list.value_type, &initial.value_type),
            (CompilerType::List(element), CompilerType::Optional(payload))
                if element.as_ref() == payload.as_ref()
        );
        let string_list_fold = matches!(
            (&list.value_type, &initial.value_type),
            (CompilerType::List(element), CompilerType::List(state_element))
                if state_element.as_ref() == &CompilerType::String
                    && (matches!(element.as_ref(), CompilerType::Character | CompilerType::String)
                        || compiler_string_pair(element.as_ref()))
        );
        let nested_int_list_state_fold = matches!(
            (&list.value_type, &initial.value_type),
            (CompilerType::List(element), CompilerType::List(state_element))
                if (compiler_nested_int_list_element(state_element.as_ref())
                    || matches!(state_element.as_ref(), CompilerType::List(inner)
                        if compiler_integer_pair(inner.as_ref())))
                    && (element.as_ref() == &CompilerType::Int
                        || compiler_nested_int_list_element(element.as_ref())
                        || matches!(element.as_ref(), CompilerType::List(inner)
                            if compiler_integer_pair(inner.as_ref()))
                        || compiler_boolean_int_pair(element.as_ref()))
        );
        let character_nat_list_fold = matches!(
            (&list.value_type, &initial.value_type),
            (CompilerType::List(element), CompilerType::List(state_element))
                if matches!(element.as_ref(), CompilerType::Character | CompilerType::Int)
                    && state_element.as_ref() == &CompilerType::Nat
        );
        let int_list_state_fold = matches!(
            (&list.value_type, &initial.value_type),
            (CompilerType::List(element), CompilerType::List(state_element))
                if (matches!(element.as_ref(), CompilerType::Int | CompilerType::Nat | CompilerType::Rational)
                    || compiler_integer_pair(element.as_ref())
                    || compiler_rational_nat_pair(element.as_ref()))
                    && matches!(state_element.as_ref(), CompilerType::Int | CompilerType::Nat | CompilerType::Rational)
        );
        let nested_int_source_int_list_state_fold = matches!(
            (&list.value_type, &initial.value_type),
            (CompilerType::List(element), CompilerType::List(state_element))
                if compiler_nested_int_list_element(element.as_ref())
                    && matches!(state_element.as_ref(), CompilerType::Int | CompilerType::Nat)
        );
        let int_triple_source_int_list_state_fold = matches!(
            (&list.value_type, &initial.value_type),
            (CompilerType::List(element), CompilerType::List(state_element))
                if compiler_three_pointer_tuple(element.as_ref())
                    && state_element.as_ref() == &CompilerType::Int
        );
        let tuple_list_state_fold = matches!(
            (&list.value_type, &initial.value_type),
            (CompilerType::List(_), CompilerType::List(state_element))
                if matches!(state_element.as_ref(), CompilerType::Tuple(_))
        );
        let range_list_state_fold = matches!(
            &initial.value_type,
            CompilerType::List(state_element)
                if matches!(state_element.as_ref(), CompilerType::Range(endpoint)
                    if endpoint.as_ref() == &CompilerType::Int)
        );
        let range_source_fold = matches!(
            &list.value_type,
            CompilerType::List(element)
                if matches!(element.as_ref(), CompilerType::Range(endpoint)
                    if endpoint.as_ref() == &CompilerType::Int)
        );
        let character_string_fold = matches!(
            (&list.value_type, &initial.value_type),
            (CompilerType::List(element), CompilerType::String)
                if element.as_ref() == &CompilerType::Character
        );
        let rational_state_fold = initial.value_type == CompilerType::Rational;
        let rational_source_fold = matches!(&list.value_type, CompilerType::List(element)
            if element.as_ref() == &CompilerType::Rational);
        let tuple_state_fold = matches!(initial.value_type, CompilerType::Tuple(_));
        let boolean_state_fold = initial.value_type == CompilerType::Boolean;
        let generic_list_state_fold = matches!(initial.value_type, CompilerType::List(_));
        let nested_int_source_boolean_fold = boolean_state_fold
            && matches!(&list.value_type, CompilerType::List(element)
                if compiler_nested_int_list_element(element.as_ref())
                    || matches!(element.as_ref(), CompilerType::List(inner)
                        if compiler_integer_pair(inner.as_ref())));
        let boolean_int_pair_fold = matches!(&list.value_type, CompilerType::List(element)
            if compiler_boolean_int_pair(element.as_ref()));
        let pair_fold = matches!(&list.value_type, CompilerType::List(element)
            if compiler_integer_pair(element.as_ref())
                || compiler_rational_nat_pair(element.as_ref())
                || compiler_rational_pair(element.as_ref())
                || compiler_int_string_pair(element.as_ref())
                || compiler_int_list_pair(element.as_ref())
                || compiler_string_pair(element.as_ref())
                || compiler_boolean_int_pair(element.as_ref()));
        let three_pointer_fold = matches!(&list.value_type, CompilerType::List(element)
            if compiler_three_pointer_tuple(element.as_ref()));
        let four_pointer_fold = matches!(&list.value_type, CompilerType::List(element)
            if compiler_int_int_string_int_tuple(element.as_ref()));
        let source = self
            .emit_expression(list, body, environment)
            .list_pointer()
            .to_owned();
        let initial_type = initial.value_type.clone();
        let initial_value = self.emit_expression(initial, body, environment);
        let initial = if tuple_state_fold {
            let LlValue::Tuple(fields) = &initial_value else {
                unreachable!("checked tuple fold state retains its aggregate value")
            };
            let CompilerType::Tuple(field_types) = &initial.value_type else {
                unreachable!("checked tuple fold state retains its classifier")
            };
            self.emit_tuple_aggregate(fields, field_types, body, span)
        } else if boolean_state_fold {
            initial_value.boolean().to_owned()
        } else if string_pair_optional_fold
            || integer_pair_optional_nat_fold
            || nested_int_optional_nat_fold
            || generic_optional_state_fold
        {
            initial_value.optional_pointer().to_owned()
        } else if character_string_fold {
            initial_value.string().to_owned()
        } else if rational_state_fold {
            initial_value.rational().to_owned()
        } else if generic_list_state_fold
            || string_list_fold
            || nested_int_list_state_fold
            || character_nat_list_fold
            || int_list_state_fold
            || nested_int_source_int_list_state_fold
            || int_triple_source_int_list_state_fold
            || tuple_list_state_fold
            || range_list_state_fold
        {
            initial_value.list_pointer().to_owned()
        } else {
            initial_value.integer().to_owned()
        };
        let preheader = body.current_block.clone();
        let loop_label = body.label("list.fold.loop");
        let visit = body.label("list.fold.visit");
        let advance = body.label("list.fold.advance");
        let done = body.label("list.fold.done");
        let next = body.reserve_value();
        let next_state = body.reserve_value();
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let current = body.instruction(
            &format!("phi ptr [{source}, %{preheader}], [{next}, %{advance}]"),
            span,
            &mut self.debug,
        );
        let state_llvm_type =
            (tuple_state_fold || boolean_state_fold).then(|| llvm_value_type(&initial_type));
        let state = body.instruction(
            &format!(
                "phi {} [{initial}, %{preheader}], [{next_state}, %{advance}]",
                state_llvm_type.as_deref().unwrap_or("ptr")
            ),
            span,
            &mut self.debug,
        );
        let empty = body.instruction(
            &format!("icmp eq ptr {current}, null"),
            span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {empty}, label %{done}, label %{visit}"),
            location,
        );

        body.start_block(&visit);
        let value = body.instruction(
            &format!(
                "load {}, ptr {current}, align {}",
                if boolean_int_pair_fold { "i1" } else { "ptr" },
                if boolean_int_pair_fold { 1 } else { 8 }
            ),
            span,
            &mut self.debug,
        );
        let second = pair_fold.then(|| {
            let address = body.instruction(
                &format!("getelementptr i8, ptr {current}, i64 8"),
                span,
                &mut self.debug,
            );
            body.instruction(
                &format!("load ptr, ptr {address}, align 8"),
                span,
                &mut self.debug,
            )
        });
        let triple_tail = three_pointer_fold.then(|| {
            [8_i64, 16_i64].map(|offset| {
                let address = body.instruction(
                    &format!("getelementptr i8, ptr {current}, i64 {offset}"),
                    span,
                    &mut self.debug,
                );
                body.instruction(
                    &format!("load ptr, ptr {address}, align 8"),
                    span,
                    &mut self.debug,
                )
            })
        });
        let four_tail = four_pointer_fold.then(|| {
            [8_i64, 16_i64, 24_i64].map(|offset| {
                let address = body.instruction(
                    &format!("getelementptr i8, ptr {current}, i64 {offset}"),
                    span,
                    &mut self.debug,
                );
                body.instruction(
                    &format!("load ptr, ptr {address}, align 8"),
                    span,
                    &mut self.debug,
                )
            })
        });
        let next_address = body.instruction(
            &format!(
                "getelementptr i8, ptr {current}, i64 {}",
                if four_pointer_fold {
                    32
                } else if three_pointer_fold {
                    24
                } else if pair_fold {
                    16
                } else {
                    8
                }
            ),
            span,
            &mut self.debug,
        );
        body.define_reserved(
            &next,
            &format!("load ptr, ptr {next_address}, align 8"),
            span,
            &mut self.debug,
        );
        let state_value = if tuple_state_fold {
            let CompilerType::Tuple(fields) = &initial_type else {
                unreachable!("checked tuple fold state retains its classifier")
            };
            self.emit_tuple_extract(&state, fields, body, span)
        } else if boolean_state_fold {
            LlValue::Boolean(state.clone())
        } else if string_pair_optional_fold
            || integer_pair_optional_nat_fold
            || nested_int_optional_nat_fold
        {
            LlValue::Optional {
                value: state.clone(),
                payload: if generic_optional_state_fold {
                    let CompilerType::Optional(payload) = &initial_type else {
                        unreachable!("checked Optional fold state retains its classifier")
                    };
                    payload.as_ref().clone()
                } else if string_pair_optional_fold {
                    CompilerType::String
                } else {
                    CompilerType::Nat
                },
                function_captures: Vec::new(),
            }
        } else if generic_list_state_fold {
            let CompilerType::List(element) = &initial_type else {
                unreachable!("checked list fold state retains its classifier")
            };
            LlValue::List {
                value: state.clone(),
                element: element.as_ref().clone(),
                function_captures: Vec::new(),
            }
        } else if nested_int_list_state_fold {
            let CompilerType::List(element) = &initial_type else {
                unreachable!("checked nested-list fold state retains its List classifier")
            };
            LlValue::List {
                value: state.clone(),
                element: element.as_ref().clone(),
                function_captures: Vec::new(),
            }
        } else if character_nat_list_fold {
            LlValue::List {
                value: state.clone(),
                element: CompilerType::Nat,
                function_captures: Vec::new(),
            }
        } else if int_list_state_fold
            || nested_int_source_int_list_state_fold
            || int_triple_source_int_list_state_fold
        {
            LlValue::List {
                value: state.clone(),
                element: if int_list_state_fold || nested_int_source_int_list_state_fold {
                    let CompilerType::List(element) = &initial_type else {
                        unreachable!("checked numeric-list fold state retains its List classifier")
                    };
                    element.as_ref().clone()
                } else {
                    CompilerType::Int
                },
                function_captures: Vec::new(),
            }
        } else if tuple_list_state_fold {
            let CompilerType::List(element) = &initial_type else {
                unreachable!("checked tuple-list fold state retains its classifier")
            };
            LlValue::List {
                value: state.clone(),
                element: element.as_ref().clone(),
                function_captures: Vec::new(),
            }
        } else if range_list_state_fold {
            LlValue::List {
                value: state.clone(),
                element: CompilerType::Range(Box::new(CompilerType::Int)),
                function_captures: Vec::new(),
            }
        } else if character_string_fold {
            LlValue::String(state.clone())
        } else if rational_state_fold {
            LlValue::Rational(state.clone())
        } else {
            LlValue::Int(state.clone())
        };
        let entry_value = if let Some([second, third, fourth]) = four_tail {
            let CompilerType::List(element) = &list.value_type else {
                unreachable!("checked four-pointer fold subject retains its List classifier")
            };
            let CompilerType::Tuple(fields) = element.as_ref() else {
                unreachable!("checked four-pointer fold entry retains its tuple classifier")
            };
            LlValue::Tuple(
                [value, second, third, fourth]
                    .into_iter()
                    .zip(fields)
                    .map(|(value, field)| match field {
                        CompilerType::String => LlValue::String(value),
                        CompilerType::Int | CompilerType::Nat => LlValue::Int(value),
                        _ => unreachable!("checked four-pointer fold field"),
                    })
                    .collect(),
            )
        } else if let Some([second, third]) = triple_tail {
            let CompilerType::List(element) = &list.value_type else {
                unreachable!("checked three-pointer fold subject retains its List classifier")
            };
            let CompilerType::Tuple(fields) = element.as_ref() else {
                unreachable!("checked three-pointer fold entry retains its tuple classifier")
            };
            LlValue::Tuple(
                [value, second, third]
                    .into_iter()
                    .zip(fields)
                    .map(|(value, field)| match field {
                        CompilerType::Int | CompilerType::Nat => LlValue::Int(value),
                        CompilerType::Rational => LlValue::Rational(value),
                        CompilerType::String => LlValue::String(value),
                        CompilerType::List(element) => LlValue::List {
                            value,
                            element: element.as_ref().clone(),
                            function_captures: Vec::new(),
                        },
                        _ => unreachable!("checked three-pointer fold field"),
                    })
                    .collect(),
            )
        } else if let Some(second) = second {
            let CompilerType::List(element) = &list.value_type else {
                unreachable!("checked fold subject retains its List classifier")
            };
            if compiler_string_pair(element.as_ref()) {
                LlValue::Tuple(vec![LlValue::String(value), LlValue::String(second)])
            } else if compiler_int_string_pair(element.as_ref()) {
                LlValue::Tuple(vec![LlValue::Int(value), LlValue::String(second)])
            } else if compiler_int_list_pair(element.as_ref()) {
                let CompilerType::Tuple(fields) = element.as_ref() else {
                    unreachable!("checked integer/list pair retains its tuple classifier")
                };
                let [_, CompilerType::List(inner)] = fields.as_slice() else {
                    unreachable!("checked integer/list pair retains its field classifiers")
                };
                LlValue::Tuple(vec![
                    LlValue::Int(value),
                    LlValue::List {
                        value: second,
                        element: inner.as_ref().clone(),
                        function_captures: Vec::new(),
                    },
                ])
            } else if compiler_boolean_int_pair(element.as_ref()) {
                LlValue::Tuple(vec![LlValue::Boolean(value), LlValue::Int(second)])
            } else if compiler_rational_nat_pair(element.as_ref()) {
                LlValue::Tuple(vec![LlValue::Rational(value), LlValue::Int(second)])
            } else if compiler_rational_pair(element.as_ref()) {
                LlValue::Tuple(vec![LlValue::Rational(value), LlValue::Rational(second)])
            } else {
                LlValue::Tuple(vec![LlValue::Int(value), LlValue::Int(second)])
            }
        } else if range_source_fold {
            LlValue::Range {
                value,
                endpoint: CompilerType::Int,
            }
        } else if string_list_fold
            || matches!(&list.value_type, CompilerType::List(element)
                if matches!(element.as_ref(), CompilerType::Character | CompilerType::String))
        {
            LlValue::String(value)
        } else if (nested_int_list_state_fold
            || nested_int_optional_nat_fold
            || nested_int_source_int_list_state_fold
            || nested_int_source_boolean_fold)
            && matches!(&list.value_type, CompilerType::List(element)
                if compiler_nested_int_list_element(element.as_ref())
                    || matches!(element.as_ref(), CompilerType::List(inner)
                        if compiler_integer_pair(inner.as_ref())))
        {
            let CompilerType::List(source_element) = &list.value_type else {
                unreachable!("checked nested-list fold source retains its List classifier")
            };
            let CompilerType::List(entry_element) = source_element.as_ref() else {
                unreachable!("checked nested-list fold entry retains its List classifier")
            };
            LlValue::List {
                value,
                element: entry_element.as_ref().clone(),
                function_captures: Vec::new(),
            }
        } else if rational_source_fold {
            LlValue::Rational(value)
        } else {
            LlValue::Int(value)
        };
        let mut parameter_values = vec![state_value];
        match entry_value {
            LlValue::Tuple(fields) if parameters.len() == fields.len() + 1 => {
                parameter_values.extend(fields);
            }
            value => parameter_values.push(value),
        }
        let mut action_environment =
            self.emit_collection_environment(parameters, &parameter_values, body, environment);
        let value = self.emit_block(action, body, &mut action_environment);
        let finished_state = if action.result.value_type
            == CompilerType::TraversalControl(Box::new(CompilerType::Int))
        {
            let control = value.traversal_control().0;
            let tag = body.instruction(
                &format!("load i64, ptr {control}, align 8"),
                action.result.span,
                &mut self.debug,
            );
            let payload_address = body.instruction(
                &format!("getelementptr i8, ptr {control}, i64 8"),
                action.result.span,
                &mut self.debug,
            );
            let payload = body.instruction(
                &format!("load ptr, ptr {payload_address}, align 8"),
                action.result.span,
                &mut self.debug,
            );
            let finish = body.instruction(
                &format!("icmp eq i64 {tag}, 1"),
                action.result.span,
                &mut self.debug,
            );
            let continue_label = body.label("list.fold.continue");
            let finish_label = body.label("list.fold.finish");
            body.terminator(
                &format!("br i1 {finish}, label %{finish_label}, label %{continue_label}"),
                location,
            );
            body.start_block(&continue_label);
            body.define_reserved(
                &next_state,
                &format!("freeze ptr {payload}"),
                span,
                &mut self.debug,
            );
            body.terminator(&format!("br label %{advance}"), location);
            body.start_block(&finish_label);
            body.terminator(&format!("br label %{done}"), location);
            Some((payload, finish_label))
        } else {
            let (value, llvm_type) = if tuple_state_fold {
                let LlValue::Tuple(fields) = &value else {
                    unreachable!("checked tuple fold action retains its aggregate result")
                };
                let CompilerType::Tuple(field_types) = &initial_type else {
                    unreachable!("checked tuple fold state retains its classifier")
                };
                (
                    self.emit_tuple_aggregate(fields, field_types, body, span),
                    llvm_value_type(&initial_type),
                )
            } else if boolean_state_fold {
                (value.boolean().to_owned(), "i1".to_owned())
            } else if string_pair_optional_fold
                || integer_pair_optional_nat_fold
                || nested_int_optional_nat_fold
                || generic_optional_state_fold
            {
                (value.optional_pointer().to_owned(), "ptr".to_owned())
            } else if character_string_fold {
                (value.string().to_owned(), "ptr".to_owned())
            } else if rational_state_fold {
                (value.rational().to_owned(), "ptr".to_owned())
            } else if generic_list_state_fold
                || string_list_fold
                || nested_int_list_state_fold
                || character_nat_list_fold
                || int_list_state_fold
                || nested_int_source_int_list_state_fold
                || int_triple_source_int_list_state_fold
                || tuple_list_state_fold
                || range_list_state_fold
            {
                (value.list_pointer().to_owned(), "ptr".to_owned())
            } else {
                (value.integer().to_owned(), "ptr".to_owned())
            };
            body.define_reserved(
                &next_state,
                &format!("freeze {llvm_type} {value}"),
                span,
                &mut self.debug,
            );
            body.terminator(&format!("br label %{advance}"), location);
            None
        };
        body.start_block(&advance);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&done);
        if let Some((finished, predecessor)) = finished_state {
            LlValue::Int(body.instruction(
                &format!("phi ptr [{state}, %{loop_label}], [{finished}, %{predecessor}]"),
                span,
                &mut self.debug,
            ))
        } else if tuple_state_fold {
            let CompilerType::Tuple(fields) = &initial_type else {
                unreachable!("checked tuple fold state retains its classifier")
            };
            self.emit_tuple_extract(&state, fields, body, span)
        } else if boolean_state_fold {
            LlValue::Boolean(state)
        } else if string_pair_optional_fold
            || integer_pair_optional_nat_fold
            || nested_int_optional_nat_fold
            || generic_optional_state_fold
        {
            LlValue::Optional {
                value: state,
                payload: if generic_optional_state_fold {
                    let CompilerType::Optional(payload) = &initial_type else {
                        unreachable!("checked Optional fold state retains its classifier")
                    };
                    payload.as_ref().clone()
                } else if string_pair_optional_fold {
                    CompilerType::String
                } else {
                    CompilerType::Nat
                },
                function_captures: Vec::new(),
            }
        } else if generic_list_state_fold {
            let CompilerType::List(element) = &initial_type else {
                unreachable!("checked list fold state retains its classifier")
            };
            LlValue::List {
                value: state,
                element: element.as_ref().clone(),
                function_captures: Vec::new(),
            }
        } else if nested_int_list_state_fold {
            LlValue::List {
                value: state,
                element: CompilerType::List(Box::new(CompilerType::Int)),
                function_captures: Vec::new(),
            }
        } else if character_nat_list_fold {
            LlValue::List {
                value: state,
                element: CompilerType::Nat,
                function_captures: Vec::new(),
            }
        } else if int_list_state_fold
            || nested_int_source_int_list_state_fold
            || int_triple_source_int_list_state_fold
        {
            LlValue::List {
                value: state,
                element: if int_list_state_fold || nested_int_source_int_list_state_fold {
                    let CompilerType::List(element) = &initial_type else {
                        unreachable!("checked numeric-list fold state retains its List classifier")
                    };
                    element.as_ref().clone()
                } else {
                    CompilerType::Int
                },
                function_captures: Vec::new(),
            }
        } else if tuple_list_state_fold {
            let CompilerType::List(element) = &initial_type else {
                unreachable!("checked tuple-list fold state retains its classifier")
            };
            LlValue::List {
                value: state,
                element: element.as_ref().clone(),
                function_captures: Vec::new(),
            }
        } else if range_list_state_fold {
            LlValue::List {
                value: state,
                element: CompilerType::Range(Box::new(CompilerType::Int)),
                function_captures: Vec::new(),
            }
        } else if character_string_fold {
            LlValue::String(state)
        } else if rational_state_fold {
            LlValue::Rational(state)
        } else {
            LlValue::Int(state)
        }
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)] // Result alternatives retain bindings and delayed actions explicitly.
    fn emit_result_decision(
        &mut self,
        subject: &CompilerExpression,
        ok_binding: &str,
        ok_binding_span: Span,
        ok_action: &CompilerExpression,
        error_codes: &[CompilerErrorCodeRule],
        error_fallback: Option<&(String, Span, Box<CompilerExpression>)>,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let result = self.emit_expression(subject, body, environment);
        let LlValue::Result {
            value,
            success,
            function_captures,
        } = result
        else {
            unreachable!("checked Result decision subject is Result")
        };
        let is_error = body.instruction(
            &format!("call i1 @topal.runtime.result.is.error(ptr {value})"),
            subject.span,
            &mut self.debug,
        );
        let ok_label = body.label("result.decision.ok");
        let error_label = body.label("result.decision.error");
        let merge = body.label("result.decision.merge");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {is_error}, label %{error_label}, label %{ok_label}"),
            location,
        );

        let mut branches = Vec::with_capacity(error_codes.len() + 2);
        body.start_block(&ok_label);
        let payload = body.instruction(
            &format!("call ptr @topal.runtime.result.payload(ptr {value})"),
            ok_binding_span,
            &mut self.debug,
        );
        let mut success_value =
            self.result_success_value(&payload, &success, body, ok_binding_span);
        if let LlValue::Function { captures, .. } = &mut success_value {
            *captures = function_captures;
        }
        let variable = self
            .debug
            .local(ok_binding, ok_binding_span, &success, body.subprogram);
        let binding_location = self.debug.location(ok_binding_span, body.subprogram);
        body.debug_value(&success_value, variable, binding_location);
        let mut ok_environment = environment.clone();
        ok_environment.insert(ok_binding.into(), success_value);
        let ok_value = self.emit_expression(ok_action, body, &ok_environment);
        let predecessor = body.current_block.clone();
        let ok_location = self.debug.location(ok_action.span, body.subprogram);
        body.terminator(&format!("br label %{merge}"), ok_location);
        branches.push((ok_value, predecessor));

        body.start_block(&error_label);
        let error = body.instruction(
            &format!("call ptr @topal.runtime.result.payload(ptr {value})"),
            subject.span,
            &mut self.debug,
        );
        if error_codes.is_empty() {
            self.emit_result_error_fallback(
                error_fallback.expect("complete Result decision has Error fallback"),
                &error,
                &merge,
                body,
                environment,
                &mut branches,
            );
        } else {
            let code = body.instruction(
                &format!("call i32 @topal.runtime.error.code(ptr {error})"),
                subject.span,
                &mut self.debug,
            );
            let labels = error_codes
                .iter()
                .map(|_| body.label("result.decision.code"))
                .collect::<Vec<_>>();
            let default = body.label("result.decision.error.fallback");
            let cases = error_codes
                .iter()
                .zip(&labels)
                .map(|(rule, label)| format!("i32 {}, label %{label}", rule.code))
                .collect::<Vec<_>>()
                .join(" ");
            body.terminator(
                &format!("switch i32 {code}, label %{default} [ {cases} ]"),
                location,
            );
            for (rule, label) in error_codes.iter().zip(labels) {
                body.start_block(&label);
                let action = self.emit_expression(&rule.action, body, environment);
                let predecessor = body.current_block.clone();
                let action_location = self.debug.location(rule.action.span, body.subprogram);
                body.terminator(&format!("br label %{merge}"), action_location);
                branches.push((action, predecessor));
            }
            body.start_block(&default);
            if let Some(fallback) = error_fallback {
                self.emit_result_error_fallback(
                    fallback,
                    &error,
                    &merge,
                    body,
                    environment,
                    &mut branches,
                );
            } else {
                body.effect(
                    "call void @topal.platform.exit(i64 70)",
                    span,
                    &mut self.debug,
                );
                body.terminator("unreachable", location);
            }
        }
        body.start_block(&merge);
        self.emit_decision_phi(&branches, body, span)
    }

    #[allow(clippy::too_many_arguments)] // Helper keeps the Error binding scoped to its action block.
    fn emit_result_error_fallback(
        &mut self,
        fallback: &(String, Span, Box<CompilerExpression>),
        error: &str,
        merge: &str,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        branches: &mut Vec<(LlValue, String)>,
    ) {
        let (binding, binding_span, action) = fallback;
        let error_value = LlValue::Error(error.into());
        let variable = self.debug.local(
            binding,
            *binding_span,
            &CompilerType::Error,
            body.subprogram,
        );
        let location = self.debug.location(*binding_span, body.subprogram);
        body.debug_value(&error_value, variable, location);
        let mut branch_environment = environment.clone();
        branch_environment.insert(binding.clone(), error_value);
        let value = self.emit_expression(action, body, &branch_environment);
        let predecessor = body.current_block.clone();
        let action_location = self.debug.location(action.span, body.subprogram);
        body.terminator(&format!("br label %{merge}"), action_location);
        branches.push((value, predecessor));
    }

    fn emit_required_nat(
        &mut self,
        value: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let result = self.emit_validation(
            CompilerValidation::IntToNat,
            value,
            value.span,
            body,
            environment,
            span,
        );
        let LlValue::Result {
            value: result_value,
            success,
            function_captures,
        } = result
        else {
            unreachable!("Nat boundary validation returns Result")
        };
        let is_error = body.instruction(
            &format!("call i1 @topal.runtime.result.is.error(ptr {result_value})"),
            span,
            &mut self.debug,
        );
        let failure = body.label("nat.boundary.error");
        let success_label = body.label("nat.boundary.ok");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {is_error}, label %{failure}, label %{success_label}"),
            location,
        );
        body.start_block(&failure);
        self.emit_print(
            &LlValue::Result {
                value: result_value.clone(),
                success: success.clone(),
                function_captures,
            },
            body,
            span,
        );
        self.emit_write_literal("\n", body, span);
        body.effect(
            "call void @topal.platform.exit(i64 1)",
            span,
            &mut self.debug,
        );
        body.terminator("unreachable", location);
        body.start_block(&success_label);
        let payload = body.instruction(
            &format!("call ptr @topal.runtime.result.payload(ptr {result_value})"),
            span,
            &mut self.debug,
        );
        self.result_success_value(&payload, &success, body, span)
    }

    #[allow(clippy::too_many_arguments)] // Validation retains explicit Error provenance at the ABI call.
    fn emit_validation(
        &mut self,
        operation: CompilerValidation,
        value: &CompilerExpression,
        error_span: Span,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        if let CompilerValidation::Constraint(tag) = operation {
            return self.emit_constraint_validation(
                tag,
                value,
                error_span,
                body,
                environment,
                span,
            );
        }
        let value = self.emit_expression(value, body, environment);
        let (runtime, domain, success, value) = match operation {
            CompilerValidation::RationalToInt => (
                "rational.try.to.int",
                "root.Int(Rational)",
                CompilerType::Int,
                value.rational(),
            ),
            CompilerValidation::IntToNat => (
                "int.try.to.nat",
                "root.Nat(Int)",
                CompilerType::Nat,
                value.integer(),
            ),
            CompilerValidation::Constraint(_) => {
                unreachable!("Constraint validation was lowered above")
            }
        };
        let domain_global = self.emit_string_value(domain, body, span);
        let source_global = self.emit_string_value(self.source_name, body, span);
        let position = self.program.source.position(error_span.start);
        LlValue::Result {
            value: body.instruction(
                &format!(
                    "call ptr @topal.runtime.{runtime}(ptr {value}, ptr {domain_global}, ptr {source_global}, i64 {}, i64 {})",
                    position.line, position.column
                ),
                span,
                &mut self.debug,
            ),
            success,
            function_captures: Vec::new(),
        }
    }

    #[allow(clippy::too_many_arguments)] // Predicate environment and Error provenance stay explicit.
    fn emit_constraint_validation(
        &mut self,
        tag: u32,
        value: &CompilerExpression,
        error_span: Span,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let constraint =
            self.program.constraints[usize::try_from(tag).expect("u32 tag fits usize")].clone();
        let value = self.emit_expression(value, body, environment);
        let mut predicate_environment = environment.clone();
        predicate_environment.insert(constraint.parameter_storage, value.clone());
        let accepted = self.emit_expression(&constraint.predicate, body, &predicate_environment);
        let accepted_label = body.label("constraint.accepted");
        let rejected_label = body.label("constraint.rejected");
        let merge = body.label("constraint.merge");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!(
                "br i1 {}, label %{accepted_label}, label %{rejected_label}",
                accepted.boolean()
            ),
            location,
        );

        body.start_block(&accepted_label);
        let payload = self.emit_result_payload(&value, &constraint.base_type, body, error_span);
        let success = body.instruction(
            &format!("call ptr @topal.runtime.result.success(ptr {payload})"),
            span,
            &mut self.debug,
        );
        let success_predecessor = body.current_block.clone();
        body.terminator(&format!("br label %{merge}"), location);

        body.start_block(&rejected_label);
        let domain = format!("root.{}({})", constraint.name, constraint.base_type.name());
        let domain = self.emit_string_value(&domain, body, span);
        let source = self.emit_string_value(self.source_name, body, span);
        let position = self.program.source.position(error_span.start);
        let failure = body.instruction(
            &format!(
                "call ptr @topal.runtime.result.failure(i32 0, ptr {domain}, ptr {source}, i64 {}, i64 {})",
                position.line, position.column
            ),
            span,
            &mut self.debug,
        );
        let failure_predecessor = body.current_block.clone();
        body.terminator(&format!("br label %{merge}"), location);

        body.start_block(&merge);
        let value = body.instruction(
            &format!(
                "phi ptr [{success}, %{success_predecessor}], [{failure}, %{failure_predecessor}]"
            ),
            span,
            &mut self.debug,
        );
        LlValue::Result {
            value,
            success: constraint.base_type,
            function_captures: Vec::new(),
        }
    }

    fn emit_modular_validation(
        &mut self,
        value: &CompilerExpression,
        modular: &CompilerModularType,
        error_span: Span,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let value = self.emit_expression(value, body, environment);
        let value = value.integer();
        let lower = self.emit_int_global(&modular.lower);
        let upper = self.emit_int_global(&modular.upper);
        let lower_order = body.instruction(
            &format!("call i32 @topal.runtime.int.compare(ptr {value}, ptr {lower})"),
            span,
            &mut self.debug,
        );
        let upper_order = body.instruction(
            &format!("call i32 @topal.runtime.int.compare(ptr {value}, ptr {upper})"),
            span,
            &mut self.debug,
        );
        let at_or_above = body.instruction(
            &format!("icmp sge i32 {lower_order}, 0"),
            span,
            &mut self.debug,
        );
        let at_or_below = body.instruction(
            &format!("icmp sle i32 {upper_order}, 0"),
            span,
            &mut self.debug,
        );
        let accepted = body.instruction(
            &format!("and i1 {at_or_above}, {at_or_below}"),
            span,
            &mut self.debug,
        );
        let accepted_label = body.label("modular.accepted");
        let rejected_label = body.label("modular.rejected");
        let merge = body.label("modular.merge");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {accepted}, label %{accepted_label}, label %{rejected_label}"),
            location,
        );

        body.start_block(&accepted_label);
        let success = body.instruction(
            &format!("call ptr @topal.runtime.result.success(ptr {value})"),
            span,
            &mut self.debug,
        );
        let success_predecessor = body.current_block.clone();
        body.terminator(&format!("br label %{merge}"), location);

        body.start_block(&rejected_label);
        let domain = self.emit_string_value(&format!("root.{}(Int)", modular.name), body, span);
        let source = self.emit_string_value(self.source_name, body, span);
        let position = self.program.source.position(error_span.start);
        let failure = body.instruction(
            &format!(
                "call ptr @topal.runtime.result.failure(i32 0, ptr {domain}, ptr {source}, i64 {}, i64 {})",
                position.line, position.column
            ),
            span,
            &mut self.debug,
        );
        let failure_predecessor = body.current_block.clone();
        body.terminator(&format!("br label %{merge}"), location);

        body.start_block(&merge);
        let value = body.instruction(
            &format!(
                "phi ptr [{success}, %{success_predecessor}], [{failure}, %{failure_predecessor}]"
            ),
            span,
            &mut self.debug,
        );
        LlValue::Result {
            value,
            success: CompilerType::Modular(modular.clone()),
            function_captures: Vec::new(),
        }
    }

    #[allow(clippy::too_many_arguments)] // Fallible ABI arguments keep structured provenance explicit.
    fn emit_fallible(
        &mut self,
        operation: CompilerFallible,
        left: &CompilerExpression,
        right: &CompilerExpression,
        error_span: Span,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let left = self.emit_expression(left, body, environment);
        let right = self.emit_expression(right, body, environment);
        let (runtime, domain, success, left, right) = match operation {
            CompilerFallible::RationalConstruct => (
                "rational.try.make",
                "root.Rational(Int,Int)",
                CompilerType::Rational,
                left.integer(),
                right.integer(),
            ),
            CompilerFallible::RationalDivide => (
                "rational.try.divide",
                "root./(Rational,Rational)",
                CompilerType::Rational,
                left.rational(),
                right.rational(),
            ),
            CompilerFallible::RationalPower => (
                "rational.try.power",
                "root.^(Rational,Int)",
                CompilerType::Rational,
                left.rational(),
                right.integer(),
            ),
            CompilerFallible::IntModulo => (
                "int.try.modulo",
                "root.%(Int,Int)",
                CompilerType::Int,
                left.integer(),
                right.integer(),
            ),
            CompilerFallible::IntQuotientModulo => (
                "int.try.quotient.modulo",
                "root./%(Int,Int)",
                CompilerType::Tuple(vec![CompilerType::Int, CompilerType::Int]),
                left.integer(),
                right.integer(),
            ),
            CompilerFallible::InfinityMultiply => {
                self.needs_infinity_result_runtime = true;
                match (&left, &right) {
                    (LlValue::Int(left), LlValue::Int(right)) => (
                        "int.try.multiply.infinity",
                        "root.*(Int,Int)",
                        CompilerType::Int,
                        left.as_str(),
                        right.as_str(),
                    ),
                    (LlValue::Rational(left), LlValue::Rational(right)) => (
                        "rational.try.multiply.infinity",
                        "root.*(Rational,Rational)",
                        CompilerType::Rational,
                        left.as_str(),
                        right.as_str(),
                    ),
                    _ => {
                        unreachable!("checked dynamic infinity multiplication has one exact domain")
                    }
                }
            }
        };
        let domain_global = self.emit_string_value(domain, body, span);
        let source_global = self.emit_string_value(self.source_name, body, span);
        let position = self.program.source.position(error_span.start);
        LlValue::Result {
            value: body.instruction(
                &format!(
                    "call ptr @topal.runtime.{runtime}(ptr {left}, ptr {right}, ptr {domain_global}, ptr {source_global}, i64 {}, i64 {})",
                    position.line, position.column
                ),
                span,
                &mut self.debug,
            ),
            success,
            function_captures: Vec::new(),
        }
    }

    fn emit_result_payload(
        &mut self,
        value: &LlValue,
        value_type: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        match (value, value_type) {
            (LlValue::Unit, CompilerType::Unit) => "null".into(),
            (LlValue::Boolean(value), CompilerType::Boolean) => {
                self.emit_boxed_boolean(value, body, span)
            }
            (LlValue::Modular { value, modular }, CompilerType::Modular(modular_type))
                if modular == modular_type =>
            {
                value.clone()
            }
            (LlValue::Int(value), CompilerType::Int | CompilerType::Nat)
            | (LlValue::Rational(value), CompilerType::Rational)
            | (LlValue::String(value), CompilerType::String)
            | (LlValue::Range { value, .. }, CompilerType::Range(_))
            | (LlValue::Result { value, .. }, CompilerType::Result(_)) => value.clone(),
            (LlValue::Enum { value, enumeration }, CompilerType::Enum(expected_enumeration))
                if enumeration == expected_enumeration =>
            {
                let storage = body.instruction(
                    "call ptr @topal.platform.allocate(i64 4)",
                    span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("store i32 {value}, ptr {storage}, align 4"),
                    span,
                    &mut self.debug,
                );
                storage
            }
            (
                LlValue::Function { value, .. } | LlValue::Enum { value, .. },
                CompilerType::Function,
            ) => {
                let storage = body.instruction(
                    "call ptr @topal.platform.allocate(i64 4)",
                    span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("store i32 {value}, ptr {storage}, align 4"),
                    span,
                    &mut self.debug,
                );
                storage
            }
            (LlValue::Tuple(fields), CompilerType::Tuple(types))
                if types.as_slice() == [CompilerType::Int, CompilerType::String] =>
            {
                let [integer, text] = fields.as_slice() else {
                    unreachable!("checked Result product payload has two fields")
                };
                let storage = body.instruction(
                    "call ptr @topal.platform.allocate(i64 16)",
                    span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("store ptr {}, ptr {storage}, align 8", integer.integer()),
                    span,
                    &mut self.debug,
                );
                let text_address = body.instruction(
                    &format!("getelementptr i8, ptr {storage}, i64 8"),
                    span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("store ptr {}, ptr {text_address}, align 8", text.string()),
                    span,
                    &mut self.debug,
                );
                storage
            }
            (LlValue::Tuple(fields), CompilerType::Tuple(types))
                if matches!(types.as_slice(), [CompilerType::Int, CompilerType::Int]) =>
            {
                let [quotient, remainder] = fields.as_slice() else {
                    unreachable!("checked quotient/modulo payload has two fields")
                };
                body.instruction(
                    &format!(
                        "call ptr @topal.runtime.int.divmod.pair(ptr {}, ptr {})",
                        quotient.integer(),
                        remainder.integer()
                    ),
                    span,
                    &mut self.debug,
                )
            }
            _ => unreachable!("checked Result success has a pointer payload representation"),
        }
    }

    fn emit_boxed_boolean(&mut self, value: &str, body: &mut FunctionBody, span: Span) -> String {
        let storage = body.instruction(
            "call ptr @topal.platform.allocate(i64 1)",
            span,
            &mut self.debug,
        );
        body.effect(
            &format!("store i1 {value}, ptr {storage}, align 1"),
            span,
            &mut self.debug,
        );
        storage
    }
}
