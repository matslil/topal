impl Generator<'_> {
    #[allow(clippy::too_many_lines)] // Exhaustive joins preserve checked representation identity.
    fn emit_decision_phi(
        &mut self,
        branches: &[(LlValue, String)],
        body: &mut FunctionBody,
        span: Span,
    ) -> LlValue {
        let first = &branches.first().expect("complete decision has a branch").0;
        let incoming = |value: fn(&LlValue) -> &str| {
            branches
                .iter()
                .map(|(branch, predecessor)| format!("[{}, %{predecessor}]", value(branch)))
                .collect::<Vec<_>>()
                .join(", ")
        };
        match first {
            LlValue::Unit => LlValue::Unit,
            LlValue::StaticDisplay(_) => {
                unreachable!("static Capability decision results are not admitted")
            }
            LlValue::Completed(_) | LlValue::Effect(_) => {
                let value = body.instruction(
                    &format!("phi i8 {}", incoming(LlValue::singleton)),
                    span,
                    &mut self.debug,
                );
                if matches!(first, LlValue::Completed(_)) {
                    LlValue::Completed(value)
                } else {
                    LlValue::Effect(value)
                }
            }
            LlValue::Boolean(_) => LlValue::Boolean(body.instruction(
                &format!("phi i1 {}", incoming(LlValue::boolean)),
                span,
                &mut self.debug,
            )),
            LlValue::Version { .. } => {
                unreachable!("Version decision results are not admitted")
            }
            LlValue::Function { .. } => {
                unreachable!("captured Function decision results are not admitted")
            }
            LlValue::SerializationStream { .. } => {
                let payload_branches = branches
                    .iter()
                    .map(|(branch, predecessor)| {
                        let LlValue::SerializationStream { value, .. } = branch else {
                            unreachable!(
                                "checked decision branches share a SerializationStream type"
                            )
                        };
                        (value.as_ref().clone(), predecessor.clone())
                    })
                    .collect::<Vec<_>>();
                LlValue::SerializationStream {
                    stream: body.instruction(
                        &format!(
                            "phi ptr {}",
                            incoming(LlValue::serialization_stream_pointer)
                        ),
                        span,
                        &mut self.debug,
                    ),
                    expected: body.instruction(
                        &format!(
                            "phi ptr {}",
                            incoming(LlValue::serialization_expected_pointer)
                        ),
                        span,
                        &mut self.debug,
                    ),
                    byte_count: body.instruction(
                        &format!("phi i64 {}", incoming(LlValue::serialization_byte_count)),
                        span,
                        &mut self.debug,
                    ),
                    value: Box::new(self.emit_decision_phi(&payload_branches, body, span)),
                }
            }
            LlValue::Task { task, .. } => LlValue::Task {
                value: body.instruction(
                    &format!("phi ptr {}", incoming(LlValue::task_pointer)),
                    span,
                    &mut self.debug,
                ),
                task: task.clone(),
            },
            LlValue::ExternalLocation { location, .. } => LlValue::ExternalLocation {
                value: body.instruction(
                    &format!("phi ptr {}", incoming(LlValue::location_pointer)),
                    span,
                    &mut self.debug,
                ),
                location: location.clone(),
            },
            LlValue::Int(_) => LlValue::Int(body.instruction(
                &format!("phi ptr {}", incoming(LlValue::integer)),
                span,
                &mut self.debug,
            )),
            LlValue::Modular { modular, .. } => LlValue::Modular {
                value: body.instruction(
                    &format!("phi ptr {}", incoming(LlValue::modular_pointer)),
                    span,
                    &mut self.debug,
                ),
                modular: modular.clone(),
            },
            LlValue::Rational(_) => LlValue::Rational(body.instruction(
                &format!("phi ptr {}", incoming(LlValue::rational)),
                span,
                &mut self.debug,
            )),
            LlValue::String(_) => LlValue::String(body.instruction(
                &format!("phi ptr {}", incoming(LlValue::string)),
                span,
                &mut self.debug,
            )),
            LlValue::Error(_) => LlValue::Error(body.instruction(
                &format!("phi ptr {}", incoming(LlValue::error)),
                span,
                &mut self.debug,
            )),
            LlValue::ErrorDomain(_) => LlValue::ErrorDomain(body.instruction(
                &format!("phi ptr {}", incoming(LlValue::error_domain)),
                span,
                &mut self.debug,
            )),
            LlValue::SourceLocation(_) => LlValue::SourceLocation(body.instruction(
                &format!("phi ptr {}", incoming(LlValue::source_location)),
                span,
                &mut self.debug,
            )),
            LlValue::Comparison(_) => LlValue::Comparison(body.instruction(
                &format!("phi i32 {}", incoming(LlValue::comparison)),
                span,
                &mut self.debug,
            )),
            LlValue::ErrorCode(_) => LlValue::ErrorCode(body.instruction(
                &format!("phi i32 {}", incoming(LlValue::error_code)),
                span,
                &mut self.debug,
            )),
            LlValue::Enum { enumeration, .. } => LlValue::Enum {
                value: body.instruction(
                    &format!("phi i32 {}", incoming(LlValue::enumeration)),
                    span,
                    &mut self.debug,
                ),
                enumeration: enumeration.clone(),
            },
            LlValue::Generator { generator, .. } => LlValue::Generator {
                value: body.instruction(
                    &format!("phi i32 {}", incoming(LlValue::generator_token)),
                    span,
                    &mut self.debug,
                ),
                generator: generator.clone(),
                captured_initial: None,
                captured_additional_initials: Vec::new(),
            },
            LlValue::Range { endpoint, .. } => {
                let endpoint = endpoint.clone();
                LlValue::Range {
                    value: body.instruction(
                        &format!("phi ptr {}", incoming(LlValue::range_pointer)),
                        span,
                        &mut self.debug,
                    ),
                    endpoint,
                }
            }
            LlValue::Result { success, .. } => {
                let success = success.clone();
                LlValue::Result {
                    value: body.instruction(
                        &format!("phi ptr {}", incoming(LlValue::result_pointer)),
                        span,
                        &mut self.debug,
                    ),
                    success,
                    function_captures: Vec::new(),
                }
            }
            LlValue::Optional { payload, .. } => {
                let payload = payload.clone();
                LlValue::Optional {
                    value: body.instruction(
                        &format!("phi ptr {}", incoming(LlValue::optional_pointer)),
                        span,
                        &mut self.debug,
                    ),
                    payload,
                    function_captures: Vec::new(),
                }
            }
            LlValue::TraversalControl { payload, .. } => {
                let payload = payload.clone();
                LlValue::TraversalControl {
                    value: body.instruction(
                        &format!("phi ptr {}", incoming(LlValue::traversal_control_pointer)),
                        span,
                        &mut self.debug,
                    ),
                    payload,
                }
            }
            LlValue::List { element, .. } => {
                let element = element.clone();
                LlValue::List {
                    value: body.instruction(
                        &format!("phi ptr {}", incoming(LlValue::list_pointer)),
                        span,
                        &mut self.debug,
                    ),
                    element,
                    function_captures: Vec::new(),
                }
            }
            LlValue::Container { value_type, .. } => {
                let value_type = value_type.clone();
                LlValue::Container {
                    value: body.instruction(
                        &format!("phi ptr {}", incoming(LlValue::container_pointer)),
                        span,
                        &mut self.debug,
                    ),
                    value_type,
                    function_captures: Vec::new(),
                    function_capture_keys: Vec::new(),
                }
            }
            LlValue::Sum { sum, .. } => {
                let sum = sum.clone();
                let tags = branches
                    .iter()
                    .map(|(branch, predecessor)| {
                        let LlValue::Sum { tag, .. } = branch else {
                            unreachable!("checked decision branches share a sum type")
                        };
                        format!("[{tag}, %{predecessor}]")
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let tag = body.instruction(&format!("phi i32 {tags}"), span, &mut self.debug);
                let payloads = sum
                    .alternatives
                    .iter()
                    .enumerate()
                    .map(|(index, alternative)| {
                        alternative.payload.as_ref().map(|_| {
                            let payload_branches = branches
                                .iter()
                                .map(|(branch, predecessor)| {
                                    let LlValue::Sum { payloads, .. } = branch else {
                                        unreachable!("checked decision branches share a sum type")
                                    };
                                    (
                                        payloads[index]
                                            .as_deref()
                                            .expect("sum payload slot retains a machine value")
                                            .clone(),
                                        predecessor.clone(),
                                    )
                                })
                                .collect::<Vec<_>>();
                            Box::new(self.emit_decision_phi(&payload_branches, body, span))
                        })
                    })
                    .collect();
                LlValue::Sum { tag, payloads, sum }
            }
            LlValue::Tuple(fields) => LlValue::Tuple(
                (0..fields.len())
                    .map(|index| {
                        let field_branches = branches
                            .iter()
                            .map(|(branch, predecessor)| {
                                let LlValue::Tuple(fields) = branch else {
                                    unreachable!("checked decision branches share a Tuple type")
                                };
                                (fields[index].clone(), predecessor.clone())
                            })
                            .collect::<Vec<_>>();
                        self.emit_decision_phi(&field_branches, body, span)
                    })
                    .collect(),
            ),
            LlValue::Record { fields, order } => {
                let fields = (0..fields.len())
                    .map(|index| {
                        let field_branches = branches
                            .iter()
                            .map(|(branch, predecessor)| {
                                let LlValue::Record { fields, .. } = branch else {
                                    unreachable!("checked decision branches share a Record type")
                                };
                                (fields[index].1.clone(), predecessor.clone())
                            })
                            .collect::<Vec<_>>();
                        (
                            fields[index].0.clone(),
                            self.emit_decision_phi(&field_branches, body, span),
                        )
                    })
                    .collect();
                let order = (0..order.len())
                    .map(|index| {
                        let incoming = branches
                            .iter()
                            .map(|(branch, predecessor)| {
                                let LlValue::Record { order, .. } = branch else {
                                    unreachable!("checked decision branches share a Record type")
                                };
                                format!("[{}, %{predecessor}]", order[index])
                            })
                            .collect::<Vec<_>>()
                            .join(", ");
                        body.instruction(&format!("phi i32 {incoming}"), span, &mut self.debug)
                    })
                    .collect();
                LlValue::Record { fields, order }
            }
        }
    }

    #[allow(clippy::too_many_lines)] // Source value rendering keeps every admitted representation explicit.
    fn emit_print(&mut self, value: &LlValue, body: &mut FunctionBody, span: Span) {
        match value {
            LlValue::Unit => self.emit_write_literal("()", body, span),
            LlValue::Completed(_) => self.emit_write_literal("Completed", body, span),
            LlValue::Effect(_) => self.emit_write_literal("Effects ()", body, span),
            LlValue::Boolean(value) => {
                let true_label = body.label("print.true");
                let false_label = body.label("print.false");
                let done_label = body.label("print.done");
                let location = self.debug.location(span, body.subprogram);
                body.terminator(
                    &format!("br i1 {value}, label %{true_label}, label %{false_label}"),
                    location,
                );
                body.start_block(&true_label);
                self.emit_write_literal("true", body, span);
                body.terminator(&format!("br label %{done_label}"), location);
                body.start_block(&false_label);
                self.emit_write_literal("false", body, span);
                body.terminator(&format!("br label %{done_label}"), location);
                body.start_block(&done_label);
            }
            LlValue::Version { display, .. } | LlValue::StaticDisplay(display) => {
                self.emit_write_literal(display, body, span);
            }
            LlValue::SerializationStream { byte_count, .. } => {
                self.emit_write_literal("SerializationStream ( ", body, span);
                body.effect(
                    &format!("call void @topal.runtime.u64.print(i64 {byte_count})"),
                    span,
                    &mut self.debug,
                );
                self.emit_write_literal(" bytes )", body, span);
            }
            LlValue::Task { task, .. } => {
                self.emit_write_literal(
                    &format!("<{} {}>", task.classifier, task.identity),
                    body,
                    span,
                );
            }
            LlValue::ExternalLocation { location, .. } => {
                self.emit_write_literal(&format!("<{}>", location.identity), body, span);
            }
            LlValue::Int(value) => body.effect(
                &format!("call void @topal.runtime.int.print(ptr {value})"),
                span,
                &mut self.debug,
            ),
            LlValue::Modular { value, modular } => {
                self.emit_write_literal(&modular.name, body, span);
                self.emit_write_literal(" ", body, span);
                body.effect(
                    &format!("call void @topal.runtime.int.print(ptr {value})"),
                    span,
                    &mut self.debug,
                );
            }
            LlValue::Rational(value) => body.effect(
                &format!("call void @topal.runtime.rational.print(ptr {value})"),
                span,
                &mut self.debug,
            ),
            LlValue::Comparison(value) => {
                let less = body.label("print.less");
                let equal = body.label("print.equal");
                let greater = body.label("print.greater");
                let done = body.label("print.comparison.done");
                let location = self.debug.location(span, body.subprogram);
                body.terminator(
                    &format!(
                        "switch i32 {value}, label %{greater} [ i32 -1, label %{less} i32 0, label %{equal} ]"
                    ),
                    location,
                );
                for (label, text) in [(&less, "Less"), (&equal, "Equal"), (&greater, "Greater")] {
                    body.start_block(label);
                    self.emit_write_literal(text, body, span);
                    body.terminator(&format!("br label %{done}"), location);
                }
                body.start_block(&done);
            }
            LlValue::Enum { value, enumeration }
            | LlValue::Function {
                value, enumeration, ..
            } => {
                self.emit_print_enum(value, enumeration, body, span);
            }
            LlValue::Generator { generator, .. } => {
                self.emit_write_literal(
                    &format!("<{}>", CompilerType::Generator(generator.clone()).name()),
                    body,
                    span,
                );
            }
            LlValue::Sum { tag, payloads, sum } => {
                let labels = sum
                    .alternatives
                    .iter()
                    .map(|_| body.label("print.sum.alternative"))
                    .collect::<Vec<_>>();
                let invalid = body.label("print.sum.invalid");
                let done = body.label("print.sum.done");
                let cases = labels
                    .iter()
                    .enumerate()
                    .map(|(index, label)| format!("i32 {index}, label %{label}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                let location = self.debug.location(span, body.subprogram);
                body.terminator(
                    &format!("switch i32 {tag}, label %{invalid} [ {cases} ]"),
                    location,
                );
                for (index, (label, alternative)) in
                    labels.iter().zip(&sum.alternatives).enumerate()
                {
                    body.start_block(label);
                    self.emit_write_literal(&alternative.name, body, span);
                    if let Some(payload) = payloads[index].as_deref() {
                        self.emit_write_literal(" ", body, span);
                        self.emit_print(payload, body, span);
                    }
                    body.terminator(&format!("br label %{done}"), location);
                }
                body.start_block(&invalid);
                body.effect(
                    "call void @topal.platform.exit(i64 70)",
                    span,
                    &mut self.debug,
                );
                body.terminator("unreachable", location);
                body.start_block(&done);
            }
            LlValue::Range { value, endpoint } => body.effect(
                &format!(
                    "call void @topal.runtime.range.{}.print(ptr {value})",
                    numeric_domain(endpoint)
                ),
                span,
                &mut self.debug,
            ),
            LlValue::Result { value, success, .. } => {
                self.emit_print_result(value, success, body, span);
            }
            LlValue::Optional { value, payload, .. } => {
                self.emit_print_optional(value, payload, body, span);
            }
            LlValue::TraversalControl { value, payload } => {
                self.emit_print_traversal_control(value, payload, body, span);
            }
            LlValue::List { value, element, .. } => {
                self.emit_print_list(value, element, body, span);
            }
            LlValue::Container {
                value, value_type, ..
            } => {
                self.emit_print_container(value, value_type, body, span);
            }
            LlValue::String(value) => {
                body.effect(
                    &format!("call void @topal.runtime.string.print(ptr {value})"),
                    span,
                    &mut self.debug,
                );
            }
            LlValue::Error(value) => {
                body.effect(
                    &format!("call void @topal.runtime.error.print(ptr {value})"),
                    span,
                    &mut self.debug,
                );
            }
            LlValue::ErrorCode(value) => {
                body.effect(
                    &format!("call void @topal.runtime.error.code.print(i32 {value})"),
                    span,
                    &mut self.debug,
                );
            }
            LlValue::ErrorDomain(value) => {
                body.effect(
                    &format!("call void @topal.runtime.string.raw.print(ptr {value})"),
                    span,
                    &mut self.debug,
                );
            }
            LlValue::SourceLocation(value) => {
                self.emit_write_literal("(line is ", body, span);
                let line = body.instruction(
                    &format!("call ptr @topal.runtime.source.location.line(ptr {value})"),
                    span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("call void @topal.runtime.int.print(ptr {line})"),
                    span,
                    &mut self.debug,
                );
                self.emit_write_literal(", column is ", body, span);
                let column = body.instruction(
                    &format!("call ptr @topal.runtime.source.location.column(ptr {value})"),
                    span,
                    &mut self.debug,
                );
                body.effect(
                    &format!("call void @topal.runtime.int.print(ptr {column})"),
                    span,
                    &mut self.debug,
                );
                self.emit_write_literal(")", body, span);
            }
            LlValue::Tuple(fields) => {
                self.emit_write_literal("(", body, span);
                for (index, field) in fields.iter().enumerate() {
                    if index != 0 {
                        self.emit_write_literal(", ", body, span);
                    }
                    self.emit_print(field, body, span);
                }
                if fields.len() == 1 {
                    self.emit_write_literal(",", body, span);
                }
                self.emit_write_literal(")", body, span);
            }
            LlValue::Record { fields, order } => {
                self.emit_print_record(fields, order, body, span);
            }
        }
    }

    fn emit_print_container(
        &mut self,
        container: &str,
        value_type: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) {
        match value_type {
            CompilerType::Array { element, .. } => {
                let entries = self.emit_container_entries(container, 8, body, span);
                self.emit_print_sequence_container("Array", &entries, element, body, span);
            }
            CompilerType::Set(element) => {
                let entries = self.emit_container_entries(container, 8, body, span);
                self.emit_print_sequence_container("Set", &entries, element, body, span);
            }
            CompilerType::Bag(element) if element.as_ref() == &CompilerType::Int => {
                let entries = self.emit_container_entries(container, 16, body, span);
                self.emit_print_bag(&entries, body, span);
            }
            CompilerType::Map {
                key,
                value: map_value,
            } if key.as_ref() == &CompilerType::String
                && matches!(
                    map_value.as_ref(),
                    CompilerType::Int | CompilerType::Function
                ) =>
            {
                let entries = self.emit_container_entries(container, 8, body, span);
                self.emit_print_map(&entries, map_value, body, span);
            }
            _ => unreachable!("checked fundamental container has an admitted printer"),
        }
    }

    fn emit_container_entries(
        &mut self,
        container: &str,
        offset: usize,
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        let address = body.instruction(
            &format!("getelementptr i8, ptr {container}, i64 {offset}"),
            span,
            &mut self.debug,
        );
        body.instruction(
            &format!("load ptr, ptr {address}, align 8"),
            span,
            &mut self.debug,
        )
    }

    fn emit_print_sequence_container(
        &mut self,
        name: &str,
        entries: &str,
        element: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) {
        debug_assert!(matches!(
            element,
            CompilerType::Int | CompilerType::String | CompilerType::Function
        ));
        self.emit_write_literal(&format!("{name} ("), body, span);
        let initial = body.current_block.clone();
        let loop_label = body.label("print.container.loop");
        let entry = body.label("print.container.entry");
        let first = body.label("print.container.first");
        let separator = body.label("print.container.separator");
        let render = body.label("print.container.render");
        let advance = body.label("print.container.advance");
        let done = body.label("print.container.done");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let next_name = format!("%{render}.next");
        let current = body.instruction(
            &format!("phi ptr [{entries}, %{initial}], [{next_name}, %{advance}]"),
            span,
            &mut self.debug,
        );
        let is_first = body.instruction(
            &format!("phi i1 [true, %{initial}], [false, %{advance}]"),
            span,
            &mut self.debug,
        );
        let empty = body.instruction(
            &format!("icmp eq ptr {current}, null"),
            span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {empty}, label %{done}, label %{entry}"),
            location,
        );

        body.start_block(&entry);
        body.terminator(
            &format!("br i1 {is_first}, label %{first}, label %{separator}"),
            location,
        );
        body.start_block(&first);
        body.terminator(&format!("br label %{render}"), location);
        body.start_block(&separator);
        self.emit_write_literal(", ", body, span);
        body.terminator(&format!("br label %{render}"), location);

        body.start_block(&render);
        let value = match element {
            CompilerType::Int => LlValue::Int(body.instruction(
                &format!("load ptr, ptr {current}, align 8"),
                span,
                &mut self.debug,
            )),
            CompilerType::String => LlValue::String(body.instruction(
                &format!("load ptr, ptr {current}, align 8"),
                span,
                &mut self.debug,
            )),
            CompilerType::Function => LlValue::Function {
                value: body.instruction(
                    &format!("load i32, ptr {current}, align 4"),
                    span,
                    &mut self.debug,
                ),
                enumeration: function_value_enumeration(self.program),
                captures: Vec::new(),
            },
            _ => unreachable!("checked sequence container element is supported"),
        };
        self.emit_print(&value, body, span);
        let next_address = body.instruction(
            &format!("getelementptr i8, ptr {current}, i64 8"),
            span,
            &mut self.debug,
        );
        body.named_instruction(
            &next_name,
            &format!("load ptr, ptr {next_address}, align 8"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{advance}"), location);
        body.start_block(&advance);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&done);
        self.emit_write_literal(")", body, span);
    }

    fn emit_print_bag(&mut self, entries: &str, body: &mut FunctionBody, span: Span) {
        self.emit_write_literal("Bag (", body, span);
        let initial = body.current_block.clone();
        let loop_label = body.label("print.bag.loop");
        let entry = body.label("print.bag.entry");
        let first = body.label("print.bag.first");
        let separator = body.label("print.bag.separator");
        let render = body.label("print.bag.render");
        let advance = body.label("print.bag.advance");
        let done = body.label("print.bag.done");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let next_name = format!("%{render}.next");
        let current = body.instruction(
            &format!("phi ptr [{entries}, %{initial}], [{next_name}, %{advance}]"),
            span,
            &mut self.debug,
        );
        let is_first = body.instruction(
            &format!("phi i1 [true, %{initial}], [false, %{advance}]"),
            span,
            &mut self.debug,
        );
        let empty = body.instruction(
            &format!("icmp eq ptr {current}, null"),
            span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {empty}, label %{done}, label %{entry}"),
            location,
        );
        body.start_block(&entry);
        body.terminator(
            &format!("br i1 {is_first}, label %{first}, label %{separator}"),
            location,
        );
        body.start_block(&first);
        body.terminator(&format!("br label %{render}"), location);
        body.start_block(&separator);
        self.emit_write_literal(", ", body, span);
        body.terminator(&format!("br label %{render}"), location);

        body.start_block(&render);
        let value = body.instruction(
            &format!("load ptr, ptr {current}, align 8"),
            span,
            &mut self.debug,
        );
        let count_address = body.instruction(
            &format!("getelementptr i8, ptr {current}, i64 8"),
            span,
            &mut self.debug,
        );
        let count = body.instruction(
            &format!("load i64, ptr {count_address}, align 8"),
            span,
            &mut self.debug,
        );
        let count = body.instruction(
            &format!("call ptr @topal.runtime.int.from.u64(i64 {count})"),
            span,
            &mut self.debug,
        );
        self.emit_print(
            &LlValue::Tuple(vec![LlValue::Int(value), LlValue::Int(count)]),
            body,
            span,
        );
        let next_address = body.instruction(
            &format!("getelementptr i8, ptr {current}, i64 16"),
            span,
            &mut self.debug,
        );
        body.named_instruction(
            &next_name,
            &format!("load ptr, ptr {next_address}, align 8"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{advance}"), location);
        body.start_block(&advance);
        body.terminator(&format!("br label %{loop_label}"), location);
        body.start_block(&done);
        self.emit_write_literal(")", body, span);
    }

    fn emit_print_map(
        &mut self,
        entries: &str,
        value_type: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) {
        self.emit_write_literal("Map (", body, span);
        let initial = body.current_block.clone();
        let loop_label = body.label("print.map.loop");
        let entry = body.label("print.map.entry");
        let first = body.label("print.map.first");
        let separator = body.label("print.map.separator");
        let render = body.label("print.map.render");
        let advance = body.label("print.map.advance");
        let done = body.label("print.map.done");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let next_name = format!("%{render}.next");
        let current = body.instruction(
            &format!("phi ptr [{entries}, %{initial}], [{next_name}, %{advance}]"),
            span,
            &mut self.debug,
        );
        let is_first = body.instruction(
            &format!("phi i1 [true, %{initial}], [false, %{advance}]"),
            span,
            &mut self.debug,
        );
        let empty = body.instruction(
            &format!("icmp eq ptr {current}, null"),
            span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {empty}, label %{done}, label %{entry}"),
            location,
        );
        body.start_block(&entry);
        body.terminator(
            &format!("br i1 {is_first}, label %{first}, label %{separator}"),
            location,
        );
        body.start_block(&first);
        body.terminator(&format!("br label %{render}"), location);
        body.start_block(&separator);
        self.emit_write_literal(", ", body, span);
        body.terminator(&format!("br label %{render}"), location);

        body.start_block(&render);
        let key = body.instruction(
            &format!("load ptr, ptr {current}, align 8"),
            span,
            &mut self.debug,
        );
        let value_address = body.instruction(
            &format!("getelementptr i8, ptr {current}, i64 8"),
            span,
            &mut self.debug,
        );
        let value = match value_type {
            CompilerType::Int => LlValue::Int(body.instruction(
                &format!("load ptr, ptr {value_address}, align 8"),
                span,
                &mut self.debug,
            )),
            CompilerType::Function => LlValue::Function {
                value: body.instruction(
                    &format!("load i32, ptr {value_address}, align 4"),
                    span,
                    &mut self.debug,
                ),
                enumeration: function_value_enumeration(self.program),
                captures: Vec::new(),
            },
            _ => unreachable!("checked Map value has an admitted printer"),
        };
        self.emit_print(
            &LlValue::Tuple(vec![LlValue::String(key), value]),
            body,
            span,
        );
        let next_address = body.instruction(
            &format!("getelementptr i8, ptr {current}, i64 16"),
            span,
            &mut self.debug,
        );
        body.named_instruction(
            &next_name,
            &format!("load ptr, ptr {next_address}, align 8"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{advance}"), location);
        body.start_block(&advance);
        body.terminator(&format!("br label %{loop_label}"), location);
        body.start_block(&done);
        self.emit_write_literal(")", body, span);
    }

    fn emit_print_record(
        &mut self,
        fields: &[(String, LlValue)],
        order: &[String],
        body: &mut FunctionBody,
        span: Span,
    ) {
        debug_assert_eq!(fields.len(), order.len());
        self.emit_write_literal("(", body, span);
        for (position, selector) in order.iter().enumerate() {
            if position != 0 {
                self.emit_write_literal(", ", body, span);
            }
            if let Ok(index) = selector.parse::<usize>() {
                let (label, field) = &fields[index];
                self.emit_write_literal(label, body, span);
                self.emit_write_literal(" is ", body, span);
                self.emit_print(field, body, span);
            } else {
                self.emit_print_dynamic_record_field(selector, fields, body, span);
            }
        }
        self.emit_write_literal(")", body, span);
    }

    fn emit_print_dynamic_record_field(
        &mut self,
        selector: &str,
        fields: &[(String, LlValue)],
        body: &mut FunctionBody,
        span: Span,
    ) {
        let labels = fields
            .iter()
            .map(|_| body.label("print.record.field"))
            .collect::<Vec<_>>();
        let invalid = body.label("print.record.invalid");
        let done = body.label("print.record.done");
        let cases = labels
            .iter()
            .enumerate()
            .map(|(index, label)| format!("i32 {index}, label %{label}"))
            .collect::<Vec<_>>()
            .join(" ");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("switch i32 {selector}, label %{invalid} [ {cases} ]"),
            location,
        );
        for (branch, (label, field)) in labels.iter().zip(fields) {
            body.start_block(branch);
            self.emit_write_literal(label, body, span);
            self.emit_write_literal(" is ", body, span);
            self.emit_print(field, body, span);
            body.terminator(&format!("br label %{done}"), location);
        }
        body.start_block(&invalid);
        body.effect(
            "call void @topal.platform.exit(i64 70)",
            span,
            &mut self.debug,
        );
        body.terminator("unreachable", location);
        body.start_block(&done);
    }

    fn emit_print_enum(
        &mut self,
        value: &str,
        enumeration: &CompilerEnumType,
        body: &mut FunctionBody,
        span: Span,
    ) {
        let labels = enumeration
            .alternatives
            .iter()
            .map(|_| body.label("print.enum.alternative"))
            .collect::<Vec<_>>();
        let invalid = body.label("print.enum.invalid");
        let done = body.label("print.enum.done");
        let cases = labels
            .iter()
            .enumerate()
            .map(|(index, label)| format!("i32 {index}, label %{label}"))
            .collect::<Vec<_>>()
            .join(" ");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("switch i32 {value}, label %{invalid} [ {cases} ]"),
            location,
        );
        for (label, alternative) in labels.iter().zip(&enumeration.alternatives) {
            body.start_block(label);
            self.emit_write_literal(alternative, body, span);
            body.terminator(&format!("br label %{done}"), location);
        }
        body.start_block(&invalid);
        body.effect(
            "call void @topal.platform.exit(i64 70)",
            span,
            &mut self.debug,
        );
        body.terminator("unreachable", location);
        body.start_block(&done);
    }

    fn emit_print_result(
        &mut self,
        value: &str,
        success: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) {
        let is_error = body.instruction(
            &format!("call i1 @topal.runtime.result.is.error(ptr {value})"),
            span,
            &mut self.debug,
        );
        let payload = body.instruction(
            &format!("call ptr @topal.runtime.result.payload(ptr {value})"),
            span,
            &mut self.debug,
        );
        let error = body.label("print.result.error");
        let ok = body.label("print.result.ok");
        let done = body.label("print.result.done");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {is_error}, label %{error}, label %{ok}"),
            location,
        );
        body.start_block(&error);
        body.effect(
            &format!("call void @topal.runtime.error.print(ptr {payload})"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{done}"), location);
        body.start_block(&ok);
        let success = self.result_success_value(&payload, success, body, span);
        self.emit_print(&success, body, span);
        body.terminator(&format!("br label %{done}"), location);
        body.start_block(&done);
    }

    fn emit_print_optional(
        &mut self,
        value: &str,
        payload_type: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) {
        let is_some = body.instruction(
            &format!("call i1 @topal.runtime.optional.is.some(ptr {value})"),
            span,
            &mut self.debug,
        );
        let some = body.label("print.optional.some");
        let none = body.label("print.optional.none");
        let done = body.label("print.optional.done");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {is_some}, label %{some}, label %{none}"),
            location,
        );
        body.start_block(&some);
        self.emit_write_literal("Some ", body, span);
        let payload = body.instruction(
            &format!("call ptr @topal.runtime.optional.payload(ptr {value})"),
            span,
            &mut self.debug,
        );
        let payload = self.emit_optional_payload_value(payload, payload_type, body, span);
        self.emit_print(&payload, body, span);
        body.terminator(&format!("br label %{done}"), location);
        body.start_block(&none);
        self.emit_write_literal("None", body, span);
        body.terminator(&format!("br label %{done}"), location);
        body.start_block(&done);
    }

    fn emit_print_traversal_control(
        &mut self,
        value: &str,
        payload_type: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) {
        let tag = body.instruction(
            &format!("load i64, ptr {value}, align 8"),
            span,
            &mut self.debug,
        );
        let is_finish = body.instruction(&format!("icmp eq i64 {tag}, 1"), span, &mut self.debug);
        let payload_address = body.instruction(
            &format!("getelementptr i8, ptr {value}, i64 8"),
            span,
            &mut self.debug,
        );
        let payload = body.instruction(
            &format!("load ptr, ptr {payload_address}, align 8"),
            span,
            &mut self.debug,
        );
        let continue_label = body.label("print.traversal.continue");
        let finish_label = body.label("print.traversal.finish");
        let done = body.label("print.traversal.done");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {is_finish}, label %{finish_label}, label %{continue_label}"),
            location,
        );
        body.start_block(&continue_label);
        self.emit_write_literal("Continue ", body, span);
        let payload = match payload_type {
            CompilerType::Int => LlValue::Int(payload.clone()),
            _ => unreachable!("checked TraversalControl payload has an admitted printer"),
        };
        self.emit_print(&payload, body, span);
        body.terminator(&format!("br label %{done}"), location);
        body.start_block(&finish_label);
        self.emit_write_literal("Finish ", body, span);
        self.emit_print(&payload, body, span);
        body.terminator(&format!("br label %{done}"), location);
        body.start_block(&done);
    }

    #[allow(clippy::too_many_lines)] // Admitted element layouts retain explicit loads and traversal.
    fn emit_print_list(
        &mut self,
        value: &str,
        element: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) {
        let initial = body.current_block.clone();
        let loop_label = body.label("print.list.loop");
        let entry = body.label("print.list.entry");
        let advance = body.label("print.list.advance");
        let empty = body.label("print.list.empty");
        let close_loop = body.label("print.list.close.loop");
        let close_one = body.label("print.list.close.one");
        let done = body.label("print.list.done");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&loop_label);
        let next_name = format!("%{entry}.next");
        let next_depth_name = format!("%{entry}.depth.next");
        let current = body.instruction(
            &format!("phi ptr [{value}, %{initial}], [{next_name}, %{advance}]"),
            span,
            &mut self.debug,
        );
        let depth = body.instruction(
            &format!("phi i64 [0, %{initial}], [{next_depth_name}, %{advance}]"),
            span,
            &mut self.debug,
        );
        let is_empty = body.instruction(
            &format!("icmp eq ptr {current}, null"),
            span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {is_empty}, label %{empty}, label %{entry}"),
            location,
        );

        body.start_block(&entry);
        self.emit_write_literal("Entry ( ", body, span);
        let (payload, next_offset) = match element {
            CompilerType::Unit => {
                let _ = body.instruction(
                    &format!("load i8, ptr {current}, align 1"),
                    span,
                    &mut self.debug,
                );
                (LlValue::Unit, 8)
            }
            CompilerType::Completed => (
                LlValue::Completed(body.instruction(
                    &format!("load i8, ptr {current}, align 1"),
                    span,
                    &mut self.debug,
                )),
                8,
            ),
            CompilerType::Effect => (
                LlValue::Effect(body.instruction(
                    &format!("load i8, ptr {current}, align 1"),
                    span,
                    &mut self.debug,
                )),
                8,
            ),
            CompilerType::Type => (
                LlValue::Enum {
                    value: body.instruction(
                        &format!("load i32, ptr {current}, align 4"),
                        span,
                        &mut self.debug,
                    ),
                    enumeration: fundamental_type_enumeration(),
                },
                8,
            ),
            CompilerType::Enum(enumeration) => (
                LlValue::Enum {
                    value: body.instruction(
                        &format!("load i32, ptr {current}, align 4"),
                        span,
                        &mut self.debug,
                    ),
                    enumeration: enumeration.clone(),
                },
                8,
            ),
            CompilerType::Modular(modular) => (
                LlValue::Modular {
                    value: body.instruction(
                        &format!("load ptr, ptr {current}, align 8"),
                        span,
                        &mut self.debug,
                    ),
                    modular: modular.clone(),
                },
                8,
            ),
            CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::Int => (
                LlValue::Optional {
                    value: body.instruction(
                        &format!("load ptr, ptr {current}, align 8"),
                        span,
                        &mut self.debug,
                    ),
                    payload: CompilerType::Int,
                    function_captures: Vec::new(),
                },
                8,
            ),
            CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::Rational => (
                LlValue::Optional {
                    value: body.instruction(
                        &format!("load ptr, ptr {current}, align 8"),
                        span,
                        &mut self.debug,
                    ),
                    payload: CompilerType::Rational,
                    function_captures: Vec::new(),
                },
                8,
            ),
            CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::String => (
                LlValue::Optional {
                    value: body.instruction(
                        &format!("load ptr, ptr {current}, align 8"),
                        span,
                        &mut self.debug,
                    ),
                    payload: CompilerType::String,
                    function_captures: Vec::new(),
                },
                8,
            ),
            CompilerType::Boolean => (
                LlValue::Boolean(body.instruction(
                    &format!("load i1, ptr {current}, align 1"),
                    span,
                    &mut self.debug,
                )),
                8,
            ),
            CompilerType::Comparison => (
                LlValue::Comparison(body.instruction(
                    &format!("load i32, ptr {current}, align 4"),
                    span,
                    &mut self.debug,
                )),
                8,
            ),
            CompilerType::ErrorCode => (
                LlValue::ErrorCode(body.instruction(
                    &format!("load i32, ptr {current}, align 4"),
                    span,
                    &mut self.debug,
                )),
                8,
            ),
            CompilerType::Int | CompilerType::Nat => (
                LlValue::Int(body.instruction(
                    &format!("load ptr, ptr {current}, align 8"),
                    span,
                    &mut self.debug,
                )),
                8,
            ),
            CompilerType::Rational => (
                LlValue::Rational(body.instruction(
                    &format!("load ptr, ptr {current}, align 8"),
                    span,
                    &mut self.debug,
                )),
                8,
            ),
            CompilerType::Character | CompilerType::String => (
                LlValue::String(body.instruction(
                    &format!("load ptr, ptr {current}, align 8"),
                    span,
                    &mut self.debug,
                )),
                8,
            ),
            CompilerType::Function => (
                LlValue::Function {
                    value: body.instruction(
                        &format!("load i32, ptr {current}, align 4"),
                        span,
                        &mut self.debug,
                    ),
                    enumeration: function_value_enumeration(self.program),
                    captures: Vec::new(),
                },
                8,
            ),
            CompilerType::Tuple(fields)
                if matches!(
                    fields.as_slice(),
                    [
                        CompilerType::Int | CompilerType::String,
                        CompilerType::Int | CompilerType::String
                    ]
                ) =>
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
                (
                    LlValue::Tuple(vec![
                        match &fields[0] {
                            CompilerType::Int => LlValue::Int(left),
                            CompilerType::String => LlValue::String(left),
                            _ => unreachable!(),
                        },
                        match &fields[1] {
                            CompilerType::Int => LlValue::Int(right),
                            CompilerType::String => LlValue::String(right),
                            _ => unreachable!(),
                        },
                    ]),
                    16,
                )
            }
            CompilerType::List(inner)
                if inner.as_ref() == &CompilerType::Int || compiler_int_string_pair(inner) =>
            {
                (
                    LlValue::List {
                        value: body.instruction(
                            &format!("load ptr, ptr {current}, align 8"),
                            span,
                            &mut self.debug,
                        ),
                        element: inner.as_ref().clone(),
                        function_captures: Vec::new(),
                    },
                    8,
                )
            }
            CompilerType::Record(fields)
                if fields.as_slice()
                    == [
                        ("index".into(), CompilerType::Int),
                        ("value".into(), CompilerType::Int),
                    ] =>
            {
                let index = body.instruction(
                    &format!("load ptr, ptr {current}, align 8"),
                    span,
                    &mut self.debug,
                );
                let value_address = body.instruction(
                    &format!("getelementptr i8, ptr {current}, i64 8"),
                    span,
                    &mut self.debug,
                );
                let value = body.instruction(
                    &format!("load ptr, ptr {value_address}, align 8"),
                    span,
                    &mut self.debug,
                );
                (
                    LlValue::Record {
                        fields: vec![
                            ("index".into(), LlValue::Int(index)),
                            ("value".into(), LlValue::Int(value)),
                        ],
                        order: vec!["0".into(), "1".into()],
                    },
                    24,
                )
            }
            _ => unreachable!("checked List element has an admitted printer"),
        };
        self.emit_print(&payload, body, span);
        self.emit_write_literal(", ", body, span);
        body.terminator(&format!("br label %{advance}"), location);
        body.start_block(&advance);
        let next_address = body.instruction(
            &format!("getelementptr i8, ptr {current}, i64 {next_offset}"),
            span,
            &mut self.debug,
        );
        body.named_instruction(
            &next_name,
            &format!("load ptr, ptr {next_address}, align 8"),
            span,
            &mut self.debug,
        );
        body.named_instruction(
            &next_depth_name,
            &format!("add i64 {depth}, 1"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{loop_label}"), location);

        body.start_block(&empty);
        self.emit_write_literal("Empty", body, span);
        body.terminator(&format!("br label %{close_loop}"), location);

        body.start_block(&close_loop);
        let close_next_name = format!("%{close_one}.next");
        let remaining = body.instruction(
            &format!("phi i64 [{depth}, %{empty}], [{close_next_name}, %{close_one}]"),
            span,
            &mut self.debug,
        );
        let closed = body.instruction(
            &format!("icmp eq i64 {remaining}, 0"),
            span,
            &mut self.debug,
        );
        body.terminator(
            &format!("br i1 {closed}, label %{done}, label %{close_one}"),
            location,
        );

        body.start_block(&close_one);
        self.emit_write_literal(" )", body, span);
        body.named_instruction(
            &close_next_name,
            &format!("sub i64 {remaining}, 1"),
            span,
            &mut self.debug,
        );
        body.terminator(&format!("br label %{close_loop}"), location);
        body.start_block(&done);
    }

    fn emit_write_literal(&mut self, text: &str, body: &mut FunctionBody, span: Span) {
        if text.is_empty() {
            return;
        }
        let (name, length) = self.emit_bytes_global(text);
        body.effect(
            &format!("call void @topal.platform.write_all(ptr {name}, i64 {length})"),
            span,
            &mut self.debug,
        );
    }

    fn emit_bytes_global(&mut self, text: &str) -> (String, usize) {
        self.emit_raw_bytes_global(text.as_bytes())
    }

    fn emit_raw_bytes_global(&mut self, bytes: &[u8]) -> (String, usize) {
        let name = format!(".topal.bytes.{}", self.next_global);
        self.next_global += 1;
        self.globals.push(format!(
            "@{name} = private unnamed_addr constant [{} x i8] c\"{}\", align 1",
            bytes.len(),
            llvm_bytes(bytes)
        ));
        (format!("@{name}"), bytes.len())
    }

    fn emit_string_literal(&mut self, text: &str, body: &mut FunctionBody, span: Span) -> LlValue {
        LlValue::String(self.emit_string_value(text, body, span))
    }

    fn emit_string_value(&mut self, text: &str, body: &mut FunctionBody, span: Span) -> String {
        let (bytes, length) = self.emit_bytes_global(text);
        let display = display_string_literal(text);
        let (display_bytes, display_length) = self.emit_bytes_global(&display);
        body.instruction(
            &format!(
                "call ptr @topal.runtime.string.make(ptr {bytes}, i64 {length}, ptr {display_bytes}, i64 {display_length})"
            ),
            span,
            &mut self.debug,
        )
    }

    fn emit_string_range_select(
        &mut self,
        text: &CompilerExpression,
        characters: &[String],
        range: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let text = self.emit_expression(text, body, environment);
        let range = self.emit_expression(range, body, environment);
        let range = range.range().0.to_owned();
        if characters.is_empty() {
            return LlValue::String(body.instruction(
                &format!(
                    "call ptr @topal.runtime.string.select.index.range(ptr {}, ptr {range})",
                    text.string()
                ),
                span,
                &mut self.debug,
            ));
        }
        let mut selected = self.emit_string_value("", body, span);
        for (index, character) in characters.iter().enumerate() {
            let present = body.label("string.range.present");
            let absent = body.label("string.range.absent");
            let merge = body.label("string.range.merge");
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
            let location = self.debug.location(span, body.subprogram);
            body.terminator(
                &format!("br i1 {contains}, label %{present}, label %{absent}"),
                location,
            );

            body.start_block(&present);
            let character = self.emit_string_value(character, body, span);
            let appended = body.instruction(
                &format!("call ptr @topal.runtime.string.concat(ptr {selected}, ptr {character})"),
                span,
                &mut self.debug,
            );
            body.terminator(&format!("br label %{merge}"), location);

            body.start_block(&absent);
            body.terminator(&format!("br label %{merge}"), location);

            body.start_block(&merge);
            selected = body.instruction(
                &format!("phi ptr [{appended}, %{present}], [{selected}, %{absent}]"),
                span,
                &mut self.debug,
            );
        }
        LlValue::String(selected)
    }
}
