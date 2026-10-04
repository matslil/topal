impl Generator<'_> {
    fn emit_custom_character_handled_close(
        &mut self,
        generator: &CompilerExpression,
        handler: &CompilerGeneratorCloseHandler,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let _ = self.emit_expression(generator, body, environment);
        let domain = self.emit_string_value("root", body, span);
        let source = self.emit_string_value(self.source_name, body, span);
        let position = self.program.source.position(span.start);
        let result_pointer = body.instruction(
            &format!(
                "call ptr @topal.runtime.result.failure(i32 0, ptr {domain}, ptr {source}, i64 {}, i64 {})",
                position.line, position.column
            ),
            span,
            &mut self.debug,
        );
        let result_type = self.debug.result_unit_generator_error_type;
        let result_variable = self.debug.local_with_type_id(
            &handler.result_binding,
            handler.result_binding_span,
            result_type,
            body.subprogram,
        );
        let result_location = self
            .debug
            .location(handler.result_binding_span, body.subprogram);
        let result_debug_address = body.instruction(
            "alloca ptr, align 8",
            handler.result_binding_span,
            &mut self.debug,
        );
        body.effect(
            &format!("store ptr {result_pointer}, ptr {result_debug_address}, align 8"),
            handler.result_binding_span,
            &mut self.debug,
        );
        body.debug_declare(&result_debug_address, result_variable, result_location);

        let error_pointer = body.instruction(
            &format!("call ptr @topal.runtime.result.payload(ptr {result_pointer})"),
            handler.result_binding_span,
            &mut self.debug,
        );
        if let Some(rule) = handler.error_codes.iter().find(|rule| rule.code == 0) {
            let _ = body.instruction(
                &format!("call i32 @topal.runtime.error.code(ptr {error_pointer})"),
                rule.action.span,
                &mut self.debug,
            );
            let result = self.emit_expression(&rule.action, body, environment);
            debug_assert!(matches!(result, LlValue::Unit));
            return result;
        }
        let error = LlValue::Error(error_pointer);
        let error_type = self.debug.generator_error_type;
        let error_variable = self.debug.local_with_type_id(
            &handler.error_binding,
            handler.error_binding_span,
            error_type,
            body.subprogram,
        );
        let error_location = self
            .debug
            .location(handler.error_binding_span, body.subprogram);
        body.debug_value(&error, error_variable, error_location);
        let _ = body.instruction(
            &format!("call i32 @topal.runtime.error.code(ptr {})", error.error()),
            handler.error_action.span,
            &mut self.debug,
        );
        let mut handler_environment = environment.clone();
        handler_environment.insert(handler.error_binding.clone(), error);
        let result = self.emit_expression(&handler.error_action, body, &handler_environment);
        debug_assert!(matches!(result, LlValue::Unit));
        result
    }

    fn emit_result_project(
        &mut self,
        value: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
        propagate_error: bool,
    ) -> LlValue {
        let result = self.emit_expression(value, body, environment);
        let LlValue::Result {
            value,
            success,
            function_captures,
        } = result
        else {
            unreachable!("checked projection operand is Result")
        };
        let is_error = body.instruction(
            &format!("call i1 @topal.runtime.result.is.error(ptr {value})"),
            span,
            &mut self.debug,
        );
        let failure = body.label("result.project.error");
        let success_label = body.label("result.project.ok");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {is_error}, label %{failure}, label %{success_label}"),
            location,
        );
        body.start_block(&failure);
        if !propagate_error || self.current_function_return_type.is_none() {
            self.emit_print(
                &LlValue::Result {
                    value: value.clone(),
                    success: success.clone(),
                    function_captures: function_captures.clone(),
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
        } else if self.current_result_captures.is_empty() {
            body.terminator(&format!("ret ptr {value}"), location);
        } else {
            let return_type = self
                .current_function_return_type
                .clone()
                .expect("function emission retains its LLVM return type");
            let mut aggregate = body.instruction(
                &format!("insertvalue {return_type} poison, ptr {value}, 0"),
                span,
                &mut self.debug,
            );
            for (index, capture) in self.current_result_captures.clone().iter().enumerate() {
                let zero = zero_machine_value(&capture.value_type);
                let operand = self.emit_machine_operand(&zero, &capture.value_type, body, span);
                aggregate = body.instruction(
                    &format!(
                        "insertvalue {return_type} {aggregate}, {operand}, {}",
                        index + 1
                    ),
                    span,
                    &mut self.debug,
                );
            }
            body.terminator(&format!("ret {return_type} {aggregate}"), location);
        }
        body.start_block(&success_label);
        let payload = body.instruction(
            &format!("call ptr @topal.runtime.result.payload(ptr {value})"),
            span,
            &mut self.debug,
        );
        let mut success_value = self.result_success_value(&payload, &success, body, span);
        if let LlValue::Function { captures, .. } = &mut success_value {
            *captures = function_captures;
        }
        success_value
    }

    #[allow(clippy::too_many_arguments)] // Optional binding and delayed alternatives stay explicit.
    fn emit_optional_decision(
        &mut self,
        subject: &CompilerExpression,
        some_binding: Option<&(String, Span)>,
        some_action: &CompilerExpression,
        none_action: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let optional = self.emit_expression(subject, body, environment);
        let LlValue::Optional {
            value,
            payload,
            function_captures,
        } = optional
        else {
            unreachable!("checked Optional decision subject is Optional")
        };
        let is_some = body.instruction(
            &format!("call i1 @topal.runtime.optional.is.some(ptr {value})"),
            subject.span,
            &mut self.debug,
        );
        let some_label = body.label("optional.decision.some");
        let none_label = body.label("optional.decision.none");
        let merge = body.label("optional.decision.merge");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {is_some}, label %{some_label}, label %{none_label}"),
            location,
        );

        body.start_block(&some_label);
        let mut some_environment = environment.clone();
        if let Some((some_binding, some_binding_span)) = some_binding {
            let payload_pointer = body.instruction(
                &format!("call ptr @topal.runtime.optional.payload(ptr {value})"),
                *some_binding_span,
                &mut self.debug,
            );
            let payload_value = self.emit_optional_payload_value(
                payload_pointer,
                &payload,
                body,
                *some_binding_span,
            );
            let payload_value = if let LlValue::Function {
                value, enumeration, ..
            } = payload_value
            {
                LlValue::Function {
                    value,
                    enumeration,
                    captures: function_captures.clone(),
                }
            } else {
                payload_value
            };
            let variable =
                self.debug
                    .local(some_binding, *some_binding_span, &payload, body.subprogram);
            let binding_location = self.debug.location(*some_binding_span, body.subprogram);
            body.debug_value(&payload_value, variable, binding_location);
            some_environment.insert(some_binding.into(), payload_value);
        }
        let some_value = self.emit_expression(some_action, body, &some_environment);
        let some_predecessor = body.current_block.clone();
        let some_location = self.debug.location(some_action.span, body.subprogram);
        body.terminator(&format!("br label %{merge}"), some_location);

        body.start_block(&none_label);
        let none_value = self.emit_expression(none_action, body, environment);
        let none_predecessor = body.current_block.clone();
        let none_location = self.debug.location(none_action.span, body.subprogram);
        body.terminator(&format!("br label %{merge}"), none_location);

        body.start_block(&merge);
        self.emit_decision_phi(
            &[
                (some_value, some_predecessor),
                (none_value, none_predecessor),
            ],
            body,
            span,
        )
    }

    #[allow(clippy::too_many_lines)] // Each admitted Optional payload layout remains explicit.
    fn emit_optional_payload_value(
        &mut self,
        value: String,
        value_type: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) -> LlValue {
        match value_type {
            CompilerType::Boolean => LlValue::Boolean(body.instruction(
                &format!("load i1, ptr {value}, align 1"),
                span,
                &mut self.debug,
            )),
            CompilerType::List(element) => LlValue::List {
                value,
                element: element.as_ref().clone(),
                function_captures: Vec::new(),
            },
            CompilerType::Enum(enumeration) => LlValue::Enum {
                value: body.instruction(
                    &format!("load i32, ptr {value}, align 4"),
                    span,
                    &mut self.debug,
                ),
                enumeration: enumeration.clone(),
            },
            CompilerType::Function => LlValue::Function {
                value: body.instruction(
                    &format!("load i32, ptr {value}, align 4"),
                    span,
                    &mut self.debug,
                ),
                enumeration: function_value_enumeration(self.program),
                captures: Vec::new(),
            },
            CompilerType::Tuple(fields)
                if fields.as_slice() == [CompilerType::Int, CompilerType::String] =>
            {
                let integer = body.instruction(
                    &format!("load ptr, ptr {value}, align 8"),
                    span,
                    &mut self.debug,
                );
                let text_address = body.instruction(
                    &format!("getelementptr i8, ptr {value}, i64 8"),
                    span,
                    &mut self.debug,
                );
                let text = body.instruction(
                    &format!("load ptr, ptr {text_address}, align 8"),
                    span,
                    &mut self.debug,
                );
                LlValue::Tuple(vec![LlValue::Int(integer), LlValue::String(text)])
            }
            CompilerType::Tuple(fields) if compiler_integer_pair(value_type) => {
                debug_assert_eq!(fields.len(), 2);
                self.emit_boxed_int_tuple(&value, 2, body, span)
            }
            CompilerType::Tuple(fields)
                if compiler_list_integer_pair(value_type)
                    || compiler_list_string_rational_pair(value_type) =>
            {
                let [CompilerType::List(element), scalar_type] = fields.as_slice() else {
                    unreachable!("checked list/integer pair retains its classifiers")
                };
                let list = body.instruction(
                    &format!("load ptr, ptr {value}, align 8"),
                    span,
                    &mut self.debug,
                );
                let integer_address = body.instruction(
                    &format!("getelementptr i8, ptr {value}, i64 8"),
                    span,
                    &mut self.debug,
                );
                let integer = body.instruction(
                    &format!("load ptr, ptr {integer_address}, align 8"),
                    span,
                    &mut self.debug,
                );
                LlValue::Tuple(vec![
                    LlValue::List {
                        value: list,
                        element: element.as_ref().clone(),
                        function_captures: Vec::new(),
                    },
                    if scalar_type == &CompilerType::Rational {
                        LlValue::Rational(integer)
                    } else {
                        LlValue::Int(integer)
                    },
                ])
            }
            CompilerType::Tuple(fields) if compiler_three_pointer_tuple(value_type) => {
                let values = fields
                    .iter()
                    .enumerate()
                    .map(|(index, field_type)| {
                        let address = if index == 0 {
                            value.clone()
                        } else {
                            body.instruction(
                                &format!("getelementptr i8, ptr {value}, i64 {}", index * 8),
                                span,
                                &mut self.debug,
                            )
                        };
                        let field = body.instruction(
                            &format!("load ptr, ptr {address}, align 8"),
                            span,
                            &mut self.debug,
                        );
                        match field_type {
                            CompilerType::Int | CompilerType::Nat => LlValue::Int(field),
                            CompilerType::Rational => LlValue::Rational(field),
                            CompilerType::String => LlValue::String(field),
                            CompilerType::List(element) => LlValue::List {
                                value: field,
                                element: element.as_ref().clone(),
                                function_captures: Vec::new(),
                            },
                            _ => unreachable!("checked three-pointer Optional field"),
                        }
                    })
                    .collect();
                LlValue::Tuple(values)
            }
            CompilerType::Tuple(fields) if compiler_int_int_string_int_tuple(value_type) => {
                let values = fields
                    .iter()
                    .enumerate()
                    .map(|(index, field_type)| {
                        let address = if index == 0 {
                            value.clone()
                        } else {
                            body.instruction(
                                &format!("getelementptr i8, ptr {value}, i64 {}", index * 8),
                                span,
                                &mut self.debug,
                            )
                        };
                        let field = body.instruction(
                            &format!("load ptr, ptr {address}, align 8"),
                            span,
                            &mut self.debug,
                        );
                        match field_type {
                            CompilerType::String => LlValue::String(field),
                            CompilerType::Int | CompilerType::Nat => LlValue::Int(field),
                            _ => unreachable!("checked four-pointer Optional field"),
                        }
                    })
                    .collect();
                LlValue::Tuple(values)
            }
            CompilerType::Tuple(fields) if compiler_int_list_pair(value_type) => {
                let [_, CompilerType::List(element)] = fields.as_slice() else {
                    unreachable!("checked integer/list pair retains its classifiers")
                };
                let first = body.instruction(
                    &format!("load ptr, ptr {value}, align 8"),
                    span,
                    &mut self.debug,
                );
                let rest_address = body.instruction(
                    &format!("getelementptr i8, ptr {value}, i64 8"),
                    span,
                    &mut self.debug,
                );
                let rest = body.instruction(
                    &format!("load ptr, ptr {rest_address}, align 8"),
                    span,
                    &mut self.debug,
                );
                LlValue::Tuple(vec![
                    LlValue::Int(first),
                    LlValue::List {
                        value: rest,
                        element: element.as_ref().clone(),
                        function_captures: Vec::new(),
                    },
                ])
            }
            _ => optional_payload_value(value, value_type),
        }
    }

    fn emit_boxed_int_tuple(
        &mut self,
        value: &str,
        field_count: usize,
        body: &mut FunctionBody,
        span: Span,
    ) -> LlValue {
        let values = (0..field_count)
            .map(|index| {
                let offset = index * 8;
                let address = if offset == 0 {
                    value.to_owned()
                } else {
                    body.instruction(
                        &format!("getelementptr i8, ptr {value}, i64 {offset}"),
                        span,
                        &mut self.debug,
                    )
                };
                LlValue::Int(body.instruction(
                    &format!("load ptr, ptr {address}, align 8"),
                    span,
                    &mut self.debug,
                ))
            })
            .collect();
        LlValue::Tuple(values)
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)] // Keeps typed alternatives and scoped bindings together.
    fn emit_list_decision(
        &mut self,
        subject: &CompilerExpression,
        entry_bindings: Option<&ListEntryBindings>,
        entry_action: &CompilerExpression,
        empty_action: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let list = self.emit_expression(subject, body, environment);
        let LlValue::List {
            value: list,
            element,
            function_captures,
        } = list
        else {
            unreachable!("checked List decision subject is a List")
        };
        let is_empty = body.instruction(
            &format!("icmp eq ptr {list}, null"),
            subject.span,
            &mut self.debug,
        );
        let entry_label = body.label("list.decision.entry");
        let empty_label = body.label("list.decision.empty");
        let merge = body.label("list.decision.merge");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {is_empty}, label %{empty_label}, label %{entry_label}"),
            location,
        );

        body.start_block(&entry_label);
        let mut entry_environment = environment.clone();
        if let Some(((first_name, first_span), (rest_name, rest_span))) = entry_bindings {
            let (first_value, rest_function_captures) = match &element {
                CompilerType::Unit => {
                    let _ = body.instruction(
                        &format!("load i8, ptr {list}, align 1"),
                        *first_span,
                        &mut self.debug,
                    );
                    (LlValue::Unit, Vec::new())
                }
                CompilerType::Completed => (
                    LlValue::Completed(body.instruction(
                        &format!("load i8, ptr {list}, align 1"),
                        *first_span,
                        &mut self.debug,
                    )),
                    Vec::new(),
                ),
                CompilerType::Effect => (
                    LlValue::Effect(body.instruction(
                        &format!("load i8, ptr {list}, align 1"),
                        *first_span,
                        &mut self.debug,
                    )),
                    Vec::new(),
                ),
                CompilerType::Type => (
                    LlValue::Enum {
                        value: body.instruction(
                            &format!("load i32, ptr {list}, align 4"),
                            *first_span,
                            &mut self.debug,
                        ),
                        enumeration: fundamental_type_enumeration(),
                    },
                    Vec::new(),
                ),
                CompilerType::Enum(enumeration) => (
                    LlValue::Enum {
                        value: body.instruction(
                            &format!("load i32, ptr {list}, align 4"),
                            *first_span,
                            &mut self.debug,
                        ),
                        enumeration: enumeration.clone(),
                    },
                    Vec::new(),
                ),
                CompilerType::Modular(modular) => (
                    LlValue::Modular {
                        value: body.instruction(
                            &format!("load ptr, ptr {list}, align 8"),
                            *first_span,
                            &mut self.debug,
                        ),
                        modular: modular.clone(),
                    },
                    Vec::new(),
                ),
                CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::Int => (
                    LlValue::Optional {
                        value: body.instruction(
                            &format!("load ptr, ptr {list}, align 8"),
                            *first_span,
                            &mut self.debug,
                        ),
                        payload: CompilerType::Int,
                        function_captures: Vec::new(),
                    },
                    Vec::new(),
                ),
                CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::Rational => (
                    LlValue::Optional {
                        value: body.instruction(
                            &format!("load ptr, ptr {list}, align 8"),
                            *first_span,
                            &mut self.debug,
                        ),
                        payload: CompilerType::Rational,
                        function_captures: Vec::new(),
                    },
                    Vec::new(),
                ),
                CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::String => (
                    LlValue::Optional {
                        value: body.instruction(
                            &format!("load ptr, ptr {list}, align 8"),
                            *first_span,
                            &mut self.debug,
                        ),
                        payload: CompilerType::String,
                        function_captures: Vec::new(),
                    },
                    Vec::new(),
                ),
                CompilerType::Boolean => (
                    LlValue::Boolean(body.instruction(
                        &format!("load i1, ptr {list}, align 1"),
                        *first_span,
                        &mut self.debug,
                    )),
                    Vec::new(),
                ),
                CompilerType::Comparison => (
                    LlValue::Comparison(body.instruction(
                        &format!("load i32, ptr {list}, align 4"),
                        *first_span,
                        &mut self.debug,
                    )),
                    Vec::new(),
                ),
                CompilerType::ErrorCode => (
                    LlValue::ErrorCode(body.instruction(
                        &format!("load i32, ptr {list}, align 4"),
                        *first_span,
                        &mut self.debug,
                    )),
                    Vec::new(),
                ),
                CompilerType::Character | CompilerType::String => (
                    LlValue::String(body.instruction(
                        &format!("load ptr, ptr {list}, align 8"),
                        *first_span,
                        &mut self.debug,
                    )),
                    Vec::new(),
                ),
                CompilerType::Int | CompilerType::Nat => (
                    LlValue::Int(body.instruction(
                        &format!("load ptr, ptr {list}, align 8"),
                        *first_span,
                        &mut self.debug,
                    )),
                    Vec::new(),
                ),
                CompilerType::Rational => (
                    LlValue::Rational(body.instruction(
                        &format!("load ptr, ptr {list}, align 8"),
                        *first_span,
                        &mut self.debug,
                    )),
                    Vec::new(),
                ),
                CompilerType::Function => {
                    let first_captures = function_captures.first().cloned().unwrap_or_default();
                    let remaining_captures =
                        function_captures.get(1..).unwrap_or_default().to_vec();
                    (
                        LlValue::Function {
                            value: body.instruction(
                                &format!("load i32, ptr {list}, align 4"),
                                *first_span,
                                &mut self.debug,
                            ),
                            enumeration: function_value_enumeration(self.program),
                            captures: first_captures,
                        },
                        remaining_captures,
                    )
                }
                CompilerType::Tuple(fields)
                    if fields.as_slice() == [CompilerType::Int, CompilerType::Int] =>
                {
                    let left = body.instruction(
                        &format!("load ptr, ptr {list}, align 8"),
                        *first_span,
                        &mut self.debug,
                    );
                    let right_address = body.instruction(
                        &format!("getelementptr i8, ptr {list}, i64 8"),
                        *first_span,
                        &mut self.debug,
                    );
                    let right = body.instruction(
                        &format!("load ptr, ptr {right_address}, align 8"),
                        *first_span,
                        &mut self.debug,
                    );
                    (
                        LlValue::Tuple(vec![LlValue::Int(left), LlValue::Int(right)]),
                        Vec::new(),
                    )
                }
                CompilerType::Tuple(fields)
                    if fields.as_slice() == [CompilerType::Int, CompilerType::String] =>
                {
                    let left = body.instruction(
                        &format!("load ptr, ptr {list}, align 8"),
                        *first_span,
                        &mut self.debug,
                    );
                    let right_address = body.instruction(
                        &format!("getelementptr i8, ptr {list}, i64 8"),
                        *first_span,
                        &mut self.debug,
                    );
                    let right = body.instruction(
                        &format!("load ptr, ptr {right_address}, align 8"),
                        *first_span,
                        &mut self.debug,
                    );
                    (
                        LlValue::Tuple(vec![LlValue::Int(left), LlValue::String(right)]),
                        Vec::new(),
                    )
                }
                CompilerType::Tuple(fields)
                    if fields.as_slice() == [CompilerType::String, CompilerType::Int] =>
                {
                    let left = body.instruction(
                        &format!("load ptr, ptr {list}, align 8"),
                        *first_span,
                        &mut self.debug,
                    );
                    let right_address = body.instruction(
                        &format!("getelementptr i8, ptr {list}, i64 8"),
                        *first_span,
                        &mut self.debug,
                    );
                    let right = body.instruction(
                        &format!("load ptr, ptr {right_address}, align 8"),
                        *first_span,
                        &mut self.debug,
                    );
                    (
                        LlValue::Tuple(vec![LlValue::String(left), LlValue::Int(right)]),
                        Vec::new(),
                    )
                }
                CompilerType::Tuple(fields)
                    if fields.as_slice() == [CompilerType::String, CompilerType::String] =>
                {
                    let left = body.instruction(
                        &format!("load ptr, ptr {list}, align 8"),
                        *first_span,
                        &mut self.debug,
                    );
                    let right_address = body.instruction(
                        &format!("getelementptr i8, ptr {list}, i64 8"),
                        *first_span,
                        &mut self.debug,
                    );
                    let right = body.instruction(
                        &format!("load ptr, ptr {right_address}, align 8"),
                        *first_span,
                        &mut self.debug,
                    );
                    (
                        LlValue::Tuple(vec![LlValue::String(left), LlValue::String(right)]),
                        Vec::new(),
                    )
                }
                _ => unreachable!("checked List decision has an admitted element type"),
            };
            let rest_offset = if matches!(
                &element,
                CompilerType::Tuple(fields)
                    if matches!(fields.as_slice(),
                        [
                            CompilerType::Int | CompilerType::String,
                            CompilerType::Int | CompilerType::String
                        ])
            ) {
                16
            } else {
                8
            };
            let rest_address = body.instruction(
                &format!("getelementptr i8, ptr {list}, i64 {rest_offset}"),
                *rest_span,
                &mut self.debug,
            );
            let rest = body.instruction(
                &format!("load ptr, ptr {rest_address}, align 8"),
                *rest_span,
                &mut self.debug,
            );
            let rest_value = LlValue::List {
                value: rest,
                element: element.clone(),
                function_captures: rest_function_captures,
            };
            let first_variable =
                self.debug
                    .local(first_name, *first_span, &element, body.subprogram);
            let rest_type = CompilerType::List(Box::new(element.clone()));
            let rest_variable =
                self.debug
                    .local(rest_name, *rest_span, &rest_type, body.subprogram);
            let first_location = self.debug.location(*first_span, body.subprogram);
            let rest_location = self.debug.location(*rest_span, body.subprogram);
            body.debug_value(&first_value, first_variable, first_location);
            body.debug_value(&rest_value, rest_variable, rest_location);
            entry_environment.insert(first_name.clone(), first_value);
            entry_environment.insert(rest_name.clone(), rest_value);
        }
        let entry_value = self.emit_expression(entry_action, body, &entry_environment);
        let entry_predecessor = body.current_block.clone();
        let entry_location = self.debug.location(entry_action.span, body.subprogram);
        body.terminator(&format!("br label %{merge}"), entry_location);

        body.start_block(&empty_label);
        let empty_value = self.emit_expression(empty_action, body, environment);
        let empty_predecessor = body.current_block.clone();
        let empty_location = self.debug.location(empty_action.span, body.subprogram);
        body.terminator(&format!("br label %{merge}"), empty_location);

        body.start_block(&merge);
        self.emit_decision_phi(
            &[
                (entry_value, entry_predecessor),
                (empty_value, empty_predecessor),
            ],
            body,
            span,
        )
    }

    fn emit_collection_environment(
        &mut self,
        parameters: &[CompilerParameter],
        values: &[LlValue],
        body: &mut FunctionBody,
        outer: &BTreeMap<String, LlValue>,
    ) -> BTreeMap<String, LlValue> {
        debug_assert_eq!(parameters.len(), values.len());
        let mut environment = outer.clone();
        for (parameter, value) in parameters.iter().zip(values) {
            if parameter.discarded {
                continue;
            }
            let variable = self.debug.local(
                &parameter.name,
                parameter.span,
                &parameter.value_type,
                body.subprogram,
            );
            let location = self.debug.location(parameter.span, body.subprogram);
            body.debug_value(value, variable, location);
            environment.insert(parameter.name.clone(), value.clone());
        }
        environment
    }

    #[allow(clippy::too_many_lines)] // The loop keeps predicate, publication, and next ordering explicit.
    fn emit_iterate_collect(
        &mut self,
        generator: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let CompilerExpressionKind::GeneratorTakeWhile {
            generator,
            parameters: predicate_parameters,
            predicate,
        } = &generator.kind
        else {
            unreachable!("checked collect retains a bounded iterate generator")
        };
        let CompilerExpressionKind::IterateGenerator {
            initial,
            parameters: next_parameters,
            next,
        } = &generator.kind
        else {
            unreachable!("checked collect retains its direct iterate construction")
        };
        let initial = self
            .emit_expression(initial, body, environment)
            .integer()
            .to_owned();
        let preheader = body.current_block.clone();
        let loop_label = body.label("generator.collect.loop");
        let accepted = body.label("generator.collect.accepted");
        let first = body.label("generator.collect.first");
        let link = body.label("generator.collect.link");
        let linked = body.label("generator.collect.linked");
        let done = body.label("generator.collect.done");
        let next_current = body.reserve_value();
        let node = body.reserve_value();
        let next_head = body.reserve_value();
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let current = body.instruction(
            &format!("phi ptr [{initial}, %{preheader}], [{next_current}, %{linked}]"),
            span,
            &mut self.debug,
        );
        let head = body.instruction(
            &format!("phi ptr [null, %{preheader}], [{next_head}, %{linked}]"),
            span,
            &mut self.debug,
        );
        let previous = body.instruction(
            &format!("phi ptr [null, %{preheader}], [{node}, %{linked}]"),
            span,
            &mut self.debug,
        );
        let mut predicate_environment = self.emit_collection_environment(
            predicate_parameters,
            &[LlValue::Int(current.clone())],
            body,
            environment,
        );
        let accepted_value = self
            .emit_block(predicate, body, &mut predicate_environment)
            .boolean()
            .to_owned();
        body.terminator(
            &format!("br i1 {accepted_value}, label %{accepted}, label %{done}"),
            location,
        );

        body.start_block(&accepted);
        body.define_reserved(
            &node,
            "call ptr @topal.platform.allocate(i64 16)",
            span,
            &mut self.debug,
        );
        body.effect(
            &format!("store ptr {current}, ptr {node}, align 8"),
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
        let mut next_environment = self.emit_collection_environment(
            next_parameters,
            &[LlValue::Int(current)],
            body,
            environment,
        );
        let next_value = self
            .emit_block(next, body, &mut next_environment)
            .integer()
            .to_owned();
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
        body.terminator(&format!("br label %{linked}"), location);
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
        body.terminator(&format!("br label %{linked}"), location);
        body.start_block(&linked);
        body.define_reserved(
            &next_head,
            &format!("phi ptr [{node}, %{first}], [{head}, %{link}]"),
            span,
            &mut self.debug,
        );
        body.define_reserved(
            &next_current,
            &format!("phi ptr [{next_value}, %{first}], [{next_value}, %{link}]"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&done);
        LlValue::List {
            value: head,
            element: CompilerType::Int,
            function_captures: Vec::new(),
        }
    }

    #[allow(clippy::too_many_lines)] // The loop keeps seed elimination and node publication explicit.
    fn emit_unfold_collect(
        &mut self,
        generator: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let CompilerExpressionKind::UnfoldGenerator {
            seed,
            parameters,
            step,
        } = &generator.kind
        else {
            unreachable!("checked collect retains its unfold construction")
        };
        debug_assert!(step.statements.is_empty());
        debug_assert!(matches!(
            step.result.kind,
            CompilerExpressionKind::ListUncons(_)
        ));
        let initial = self
            .emit_expression(seed, body, environment)
            .list_pointer()
            .to_owned();
        let preheader = body.current_block.clone();
        let loop_label = body.label("generator.unfold.collect.loop");
        let some = body.label("generator.unfold.collect.some");
        let first = body.label("generator.unfold.collect.first");
        let link = body.label("generator.unfold.collect.link");
        let linked = body.label("generator.unfold.collect.linked");
        let done = body.label("generator.unfold.collect.done");
        let next_seed = body.reserve_value();
        let node = body.reserve_value();
        let next_head = body.reserve_value();
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let current = body.instruction(
            &format!("phi ptr [{initial}, %{preheader}], [{next_seed}, %{linked}]"),
            span,
            &mut self.debug,
        );
        let head = body.instruction(
            &format!("phi ptr [null, %{preheader}], [{next_head}, %{linked}]"),
            span,
            &mut self.debug,
        );
        let previous = body.instruction(
            &format!("phi ptr [null, %{preheader}], [{node}, %{linked}]"),
            span,
            &mut self.debug,
        );
        let _step_environment = self.emit_collection_environment(
            parameters,
            &[LlValue::List {
                value: current.clone(),
                element: CompilerType::Int,
                function_captures: Vec::new(),
            }],
            body,
            environment,
        );
        let has_value = body.instruction(
            &format!("icmp ne ptr {current}, null"),
            step.result.span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {has_value}, label %{some}, label %{done}"),
            location,
        );

        body.start_block(&some);
        let yielded = body.instruction(
            &format!("load ptr, ptr {current}, align 8"),
            step.result.span,
            &mut self.debug,
        );
        let seed_next_address = body.instruction(
            &format!("getelementptr i8, ptr {current}, i64 8"),
            step.result.span,
            &mut self.debug,
        );
        body.define_reserved(
            &next_seed,
            &format!("load ptr, ptr {seed_next_address}, align 8"),
            step.result.span,
            &mut self.debug,
        );
        body.define_reserved(
            &node,
            "call ptr @topal.platform.allocate(i64 16)",
            span,
            &mut self.debug,
        );
        body.effect(
            &format!("store ptr {yielded}, ptr {node}, align 8"),
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
        body.terminator(&format!("br label %{linked}"), location);
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
        body.terminator(&format!("br label %{linked}"), location);
        body.start_block(&linked);
        body.define_reserved(
            &next_head,
            &format!("phi ptr [{node}, %{first}], [{head}, %{link}]"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&done);
        LlValue::List {
            value: head,
            element: CompilerType::Int,
            function_captures: Vec::new(),
        }
    }

    fn emit_iterate_foreach(
        &mut self,
        generator: &CompilerExpression,
        parameter: &CompilerParameter,
        action: &CompilerBlock,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let CompilerExpressionKind::GeneratorTakeWhile {
            generator,
            parameters: predicate_parameters,
            predicate,
        } = &generator.kind
        else {
            unreachable!("checked foreach retains a bounded iterate generator")
        };
        let CompilerExpressionKind::IterateGenerator {
            initial,
            parameters: next_parameters,
            next,
        } = &generator.kind
        else {
            unreachable!("checked foreach retains its iterate construction")
        };
        let initial = self
            .emit_expression(initial, body, environment)
            .integer()
            .to_owned();
        let preheader = body.current_block.clone();
        let loop_label = body.label("generator.foreach.loop");
        let accepted = body.label("generator.foreach.accepted");
        let resumed = body.label("generator.foreach.resumed");
        let done = body.label("generator.foreach.done");
        let next_current = body.reserve_value();
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let current = body.instruction(
            &format!("phi ptr [{initial}, %{preheader}], [{next_current}, %{resumed}]"),
            span,
            &mut self.debug,
        );
        let mut predicate_environment = self.emit_collection_environment(
            predicate_parameters,
            &[LlValue::Int(current.clone())],
            body,
            environment,
        );
        let accepted_value = self
            .emit_block(predicate, body, &mut predicate_environment)
            .boolean()
            .to_owned();
        body.terminator(
            &format!("br i1 {accepted_value}, label %{accepted}, label %{done}"),
            location,
        );

        body.start_block(&accepted);
        let mut action_environment = environment.clone();
        if !parameter.discarded {
            let variable = self.debug.local(
                &parameter.name,
                parameter.span,
                &CompilerType::Int,
                body.subprogram,
            );
            let binding_location = self.debug.location(parameter.span, body.subprogram);
            let value = LlValue::Int(current.clone());
            body.debug_value(&value, variable, binding_location);
            action_environment.insert(parameter.name.clone(), value);
        }
        let action_value = self.emit_block(action, body, &mut action_environment);
        debug_assert!(matches!(action_value, LlValue::Unit));
        let mut next_environment = self.emit_collection_environment(
            next_parameters,
            &[LlValue::Int(current)],
            body,
            environment,
        );
        let next_value = self
            .emit_block(next, body, &mut next_environment)
            .integer()
            .to_owned();
        let next_predecessor = body.current_block.clone();
        body.terminator(&format!("br label %{resumed}"), location);

        body.start_block(&resumed);
        body.define_reserved(
            &next_current,
            &format!("phi ptr [{next_value}, %{next_predecessor}]"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&done);
        LlValue::Unit
    }

    fn emit_list_foreach(
        &mut self,
        list: &CompilerExpression,
        parameter: &CompilerParameter,
        action: &CompilerBlock,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let source = self
            .emit_expression(list, body, environment)
            .list_pointer()
            .to_owned();
        let preheader = body.current_block.clone();
        let loop_label = body.label("list.foreach.loop");
        let visit = body.label("list.foreach.visit");
        let resumed = body.label("list.foreach.resumed");
        let done = body.label("list.foreach.done");
        let next_current = body.reserve_value();
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let current = body.instruction(
            &format!("phi ptr [{source}, %{preheader}], [{next_current}, %{resumed}]"),
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
            parameter.span,
            &mut self.debug,
        );
        let next_address = body.instruction(
            &format!("getelementptr i8, ptr {current}, i64 8"),
            span,
            &mut self.debug,
        );
        let next = body.instruction(
            &format!("load ptr, ptr {next_address}, align 8"),
            span,
            &mut self.debug,
        );
        let mut action_environment = environment.clone();
        if !parameter.discarded {
            let value = LlValue::Int(value);
            let variable = self.debug.local(
                &parameter.name,
                parameter.span,
                &CompilerType::Int,
                body.subprogram,
            );
            let binding_location = self.debug.location(parameter.span, body.subprogram);
            body.debug_value(&value, variable, binding_location);
            action_environment.insert(parameter.name.clone(), value);
        }
        let action_value = self.emit_block(action, body, &mut action_environment);
        debug_assert!(matches!(action_value, LlValue::Unit));
        let predecessor = body.current_block.clone();
        body.terminator(&format!("br label %{resumed}"), location);

        body.start_block(&resumed);
        body.define_reserved(
            &next_current,
            &format!("phi ptr [{next}, %{predecessor}]"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&done);
        LlValue::Unit
    }

    #[allow(clippy::too_many_arguments, clippy::too_many_lines)] // The loop keeps node publication explicit.
    fn emit_list_map(
        &mut self,
        list: &CompilerExpression,
        parameters: &[CompilerParameter],
        action: &CompilerBlock,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let LlValue::List {
            value: source,
            element,
            ..
        } = self.emit_expression(list, body, environment)
        else {
            unreachable!("checked map subject retains its List classifier")
        };
        let preheader = body.current_block.clone();
        let loop_label = body.label("list.map.loop");
        let visit = body.label("list.map.visit");
        let first = body.label("list.map.first");
        let link = body.label("list.map.link");
        let linked = body.label("list.map.linked");
        let done = body.label("list.map.done");
        let next = body.reserve_value();
        let node = body.reserve_value();
        let next_head = body.reserve_value();
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let current = body.instruction(
            &format!("phi ptr [{source}, %{preheader}], [{next}, %{linked}]"),
            span,
            &mut self.debug,
        );
        let head = body.instruction(
            &format!("phi ptr [null, %{preheader}], [{next_head}, %{linked}]"),
            span,
            &mut self.debug,
        );
        let previous = body.instruction(
            &format!("phi ptr [null, %{preheader}], [{node}, %{linked}]"),
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
        let (values, next_offset) = match &element {
            CompilerType::Int => {
                let value = body.instruction(
                    &format!("load ptr, ptr {current}, align 8"),
                    span,
                    &mut self.debug,
                );
                (vec![LlValue::Int(value)], 8)
            }
            CompilerType::Tuple(fields)
                if fields.as_slice() == [CompilerType::Int, CompilerType::Int] =>
            {
                let left = body.instruction(
                    &format!("load ptr, ptr {current}, align 8"),
                    span,
                    &mut self.debug,
                );
                let right_address = body.instruction(
                    &format!("getelementptr i8, ptr {current}, i64 8"),
                    span,
                    &mut self.debug,
                );
                let right = body.instruction(
                    &format!("load ptr, ptr {right_address}, align 8"),
                    span,
                    &mut self.debug,
                );
                (vec![LlValue::Int(left), LlValue::Int(right)], 16)
            }
            CompilerType::List(inner) if inner.as_ref() == &CompilerType::Int => {
                let value = body.instruction(
                    &format!("load ptr, ptr {current}, align 8"),
                    span,
                    &mut self.debug,
                );
                (
                    vec![LlValue::List {
                        value,
                        element: CompilerType::Int,
                        function_captures: Vec::new(),
                    }],
                    8,
                )
            }
            _ => unreachable!("checked map subject has an admitted List element layout"),
        };
        let next_address = body.instruction(
            &format!("getelementptr i8, ptr {current}, i64 {next_offset}"),
            span,
            &mut self.debug,
        );
        body.define_reserved(
            &next,
            &format!("load ptr, ptr {next_address}, align 8"),
            span,
            &mut self.debug,
        );
        let mut action_environment =
            self.emit_collection_environment(parameters, &values, body, environment);
        let mapped = self.emit_block(action, body, &mut action_environment);
        body.define_reserved(
            &node,
            "call ptr @topal.platform.allocate(i64 16)",
            span,
            &mut self.debug,
        );
        let mapped = if action.result.value_type == CompilerType::List(Box::new(CompilerType::Int))
        {
            mapped.list_pointer()
        } else {
            mapped.integer()
        };
        body.effect(
            &format!("store ptr {mapped}, ptr {node}, align 8"),
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
        body.terminator(&format!("br label %{linked}"), location);
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
        body.terminator(&format!("br label %{linked}"), location);
        body.start_block(&linked);
        body.define_reserved(
            &next_head,
            &format!("phi ptr [{node}, %{first}], [{head}, %{link}]"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&done);
        LlValue::List {
            value: head,
            element: action.result.value_type.clone(),
            function_captures: Vec::new(),
        }
    }
}
