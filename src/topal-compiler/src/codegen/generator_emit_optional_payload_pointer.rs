impl Generator<'_> {
    #[allow(clippy::too_many_lines)] // Keeps every supported Optional payload ABI in one dispatcher.
    fn emit_optional_payload_pointer(
        &mut self,
        value: &LlValue,
        value_type: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        if value_type == &CompilerType::Boolean {
            let LlValue::Boolean(value) = value else {
                unreachable!("checked Optional Boolean payload")
            };
            return self.emit_boxed_boolean(value, body, span);
        }
        let boxed_i32 = match (value, value_type) {
            (LlValue::Enum { value, enumeration }, CompilerType::Enum(expected_enumeration))
                if enumeration == expected_enumeration =>
            {
                Some(value)
            }
            (
                LlValue::Function { value, .. } | LlValue::Enum { value, .. },
                CompilerType::Function,
            ) => Some(value),
            _ => None,
        };
        if let Some(value) = boxed_i32 {
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
            return storage;
        }
        if let (LlValue::Tuple(fields), CompilerType::Tuple(types)) = (value, value_type)
            && (compiler_integer_pair(value_type)
                || matches!(types.as_slice(), [CompilerType::Int, CompilerType::String])
                || compiler_int_list_pair(value_type))
        {
            let [integer, second] = fields.as_slice() else {
                unreachable!("checked Optional product payload has two fields")
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
            let second_address = body.instruction(
                &format!("getelementptr i8, ptr {storage}, i64 8"),
                span,
                &mut self.debug,
            );
            let second = match second {
                LlValue::Int(second) if compiler_integer_pair(value_type) => second,
                LlValue::String(second) | LlValue::List { value: second, .. } => second,
                _ => unreachable!("checked Optional product payload has pointer fields"),
            };
            body.effect(
                &format!("store ptr {second}, ptr {second_address}, align 8"),
                span,
                &mut self.debug,
            );
            return storage;
        }
        if let (LlValue::Tuple(fields), CompilerType::Tuple(_)) = (value, value_type)
            && compiler_list_string_rational_pair(value_type)
        {
            let [list, rational] = fields.as_slice() else {
                unreachable!("checked Optional list/Rational pair has two fields")
            };
            let storage = body.instruction(
                "call ptr @topal.platform.allocate(i64 16)",
                span,
                &mut self.debug,
            );
            body.effect(
                &format!("store ptr {}, ptr {storage}, align 8", list.list_pointer()),
                span,
                &mut self.debug,
            );
            let rational_address = body.instruction(
                &format!("getelementptr i8, ptr {storage}, i64 8"),
                span,
                &mut self.debug,
            );
            body.effect(
                &format!(
                    "store ptr {}, ptr {rational_address}, align 8",
                    rational.rational()
                ),
                span,
                &mut self.debug,
            );
            return storage;
        }
        if let (LlValue::Tuple(fields), CompilerType::Tuple(_)) = (value, value_type)
            && compiler_three_pointer_tuple(value_type)
        {
            let storage = body.instruction(
                "call ptr @topal.platform.allocate(i64 24)",
                span,
                &mut self.debug,
            );
            for (index, field) in fields.iter().enumerate() {
                let offset = index * 8;
                let address = if offset == 0 {
                    storage.clone()
                } else {
                    body.instruction(
                        &format!("getelementptr i8, ptr {storage}, i64 {offset}"),
                        span,
                        &mut self.debug,
                    )
                };
                body.effect(
                    &format!(
                        "store ptr {}, ptr {address}, align 8",
                        field.aggregate_pointer()
                    ),
                    span,
                    &mut self.debug,
                );
            }
            return storage;
        }
        if let (LlValue::Tuple(fields), CompilerType::Tuple(_)) = (value, value_type)
            && compiler_int_int_string_int_tuple(value_type)
        {
            let storage = body.instruction(
                "call ptr @topal.platform.allocate(i64 32)",
                span,
                &mut self.debug,
            );
            for (index, field) in fields.iter().enumerate() {
                let address = if index == 0 {
                    storage.clone()
                } else {
                    body.instruction(
                        &format!("getelementptr i8, ptr {storage}, i64 {}", index * 8),
                        span,
                        &mut self.debug,
                    )
                };
                let (LlValue::Int(field) | LlValue::String(field)) = field else {
                    unreachable!("checked four-pointer Optional field")
                };
                body.effect(
                    &format!("store ptr {field}, ptr {address}, align 8"),
                    span,
                    &mut self.debug,
                );
            }
            return storage;
        }
        optional_payload_pointer(value).to_owned()
    }

    fn result_success_value(
        &mut self,
        payload: &str,
        success: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) -> LlValue {
        match success {
            CompilerType::Unit => LlValue::Unit,
            CompilerType::Boolean => LlValue::Boolean(body.instruction(
                &format!("load i1, ptr {payload}, align 1"),
                span,
                &mut self.debug,
            )),
            CompilerType::Int | CompilerType::Nat => LlValue::Int(payload.into()),
            CompilerType::Modular(modular) => LlValue::Modular {
                value: payload.into(),
                modular: modular.clone(),
            },
            CompilerType::Rational => LlValue::Rational(payload.into()),
            CompilerType::String => LlValue::String(payload.into()),
            CompilerType::Range(endpoint) => LlValue::Range {
                value: payload.into(),
                endpoint: endpoint.as_ref().clone(),
            },
            CompilerType::Result(nested) => LlValue::Result {
                value: payload.into(),
                success: nested.as_ref().clone(),
                function_captures: Vec::new(),
            },
            CompilerType::List(element) => LlValue::List {
                value: payload.into(),
                element: element.as_ref().clone(),
                function_captures: Vec::new(),
            },
            CompilerType::Enum(enumeration) => LlValue::Enum {
                value: body.instruction(
                    &format!("load i32, ptr {payload}, align 4"),
                    span,
                    &mut self.debug,
                ),
                enumeration: enumeration.clone(),
            },
            CompilerType::Function => LlValue::Function {
                value: body.instruction(
                    &format!("load i32, ptr {payload}, align 4"),
                    span,
                    &mut self.debug,
                ),
                enumeration: function_value_enumeration(self.program),
                captures: Vec::new(),
            },
            CompilerType::Tuple(types)
                if types.as_slice() == [CompilerType::Int, CompilerType::String] =>
            {
                let integer = body.instruction(
                    &format!("load ptr, ptr {payload}, align 8"),
                    span,
                    &mut self.debug,
                );
                let text_address = body.instruction(
                    &format!("getelementptr i8, ptr {payload}, i64 8"),
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
            CompilerType::Tuple(types)
                if matches!(types.as_slice(), [CompilerType::Int, CompilerType::Int]) =>
            {
                let quotient = body.instruction(
                    &format!("call ptr @topal.runtime.int.divmod.quotient(ptr {payload})"),
                    span,
                    &mut self.debug,
                );
                let remainder = body.instruction(
                    &format!("call ptr @topal.runtime.int.divmod.remainder(ptr {payload})"),
                    span,
                    &mut self.debug,
                );
                LlValue::Tuple(vec![LlValue::Int(quotient), LlValue::Int(remainder)])
            }
            _ => unreachable!("checked Result success has a pointer payload representation"),
        }
    }

    #[allow(clippy::too_many_lines)] // Exact domains and result representations are selected together.
    fn emit_binary(
        &mut self,
        operation: CompilerBinary,
        left: &LlValue,
        right: &LlValue,
        body: &mut FunctionBody,
        span: Span,
    ) -> LlValue {
        match operation {
            CompilerBinary::Range
            | CompilerBinary::RangeOpen
            | CompilerBinary::RangeInclusive
            | CompilerBinary::RangeOpenInclusive => {
                let (lower_inclusive, upper_inclusive) = match operation {
                    CompilerBinary::Range => (1, 0),
                    CompilerBinary::RangeOpen => (0, 0),
                    CompilerBinary::RangeInclusive => (1, 1),
                    CompilerBinary::RangeOpenInclusive => (0, 1),
                    _ => unreachable!(),
                };
                let (left, endpoint) = numeric_pointer(left);
                let (right, right_endpoint) = numeric_pointer(right);
                debug_assert_eq!(endpoint, right_endpoint);
                LlValue::Range {
                    value: body.instruction(
                        &format!(
                            "call ptr @topal.runtime.range.make(ptr {left}, ptr {right}, i64 {lower_inclusive}, i64 {upper_inclusive})"
                        ),
                        span,
                        &mut self.debug,
                    ),
                    endpoint: endpoint.clone(),
                }
            }
            CompilerBinary::In | CompilerBinary::Contains => {
                let (range, value) = if operation == CompilerBinary::In {
                    (right, left)
                } else {
                    (left, right)
                };
                let (range, endpoint) = range.range();
                let (value, value_type) = numeric_pointer(value);
                debug_assert_eq!(endpoint, &value_type);
                LlValue::Boolean(body.instruction(
                    &format!(
                        "call i1 @topal.runtime.range.{}.contains(ptr {range}, ptr {value})",
                        numeric_domain(endpoint)
                    ),
                    span,
                    &mut self.debug,
                ))
            }
            CompilerBinary::Add | CompilerBinary::Subtract | CompilerBinary::Multiply => {
                let name = match operation {
                    CompilerBinary::Add => "add",
                    CompilerBinary::Subtract => "subtract",
                    CompilerBinary::Multiply => "multiply",
                    _ => unreachable!(),
                };
                if let (
                    LlValue::Modular {
                        value: left,
                        modular,
                    },
                    LlValue::Modular {
                        value: right,
                        modular: right_modular,
                    },
                ) = (left, right)
                {
                    debug_assert_eq!(modular, right_modular);
                    let value = body.instruction(
                        &format!("call ptr @topal.runtime.int.{name}(ptr {left}, ptr {right})"),
                        span,
                        &mut self.debug,
                    );
                    return self.emit_modular_reduce(&value, modular, body, span);
                }
                let (domain, left, right) = match (left, right) {
                    (LlValue::Int(left), LlValue::Int(right)) => ("int", left, right),
                    (LlValue::Rational(left), LlValue::Rational(right)) => {
                        ("rational", left, right)
                    }
                    _ => unreachable!("checked arithmetic operands agree"),
                };
                let value = body.instruction(
                    &format!("call ptr @topal.runtime.{domain}.{name}(ptr {left}, ptr {right})"),
                    span,
                    &mut self.debug,
                );
                if domain == "int" {
                    LlValue::Int(value)
                } else {
                    LlValue::Rational(value)
                }
            }
            CompilerBinary::Divide => {
                let value = body.instruction(
                    &format!(
                        "call ptr @topal.runtime.rational.divide(ptr {}, ptr {})",
                        left.rational(),
                        right.rational()
                    ),
                    span,
                    &mut self.debug,
                );
                LlValue::Rational(value)
            }
            CompilerBinary::Modulo => LlValue::Int(body.instruction(
                &format!(
                    "call ptr @topal.runtime.int.modulo(ptr {}, ptr {})",
                    left.integer(),
                    right.integer()
                ),
                span,
                &mut self.debug,
            )),
            CompilerBinary::QuotientModulo => {
                let pair = body.instruction(
                    &format!(
                        "call ptr @topal.runtime.int.quotient.modulo(ptr {}, ptr {})",
                        left.integer(),
                        right.integer()
                    ),
                    span,
                    &mut self.debug,
                );
                let quotient = body.instruction(
                    &format!("call ptr @topal.runtime.int.divmod.quotient(ptr {pair})"),
                    span,
                    &mut self.debug,
                );
                let remainder = body.instruction(
                    &format!("call ptr @topal.runtime.int.divmod.remainder(ptr {pair})"),
                    span,
                    &mut self.debug,
                );
                LlValue::Tuple(vec![LlValue::Int(quotient), LlValue::Int(remainder)])
            }
            CompilerBinary::Power => match left {
                LlValue::Int(left) => LlValue::Int(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.int.power(ptr {left}, ptr {})",
                        right.integer()
                    ),
                    span,
                    &mut self.debug,
                )),
                LlValue::Rational(left) => LlValue::Rational(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.rational.power(ptr {left}, ptr {})",
                        right.integer()
                    ),
                    span,
                    &mut self.debug,
                )),
                _ => unreachable!("checked power base is exact numeric"),
            },
            CompilerBinary::Compare => {
                LlValue::Comparison(self.emit_total_compare(left, right, body, span))
            }
            CompilerBinary::And => match (left, right) {
                (LlValue::Boolean(left), LlValue::Boolean(right)) => LlValue::Boolean(
                    body.instruction(&format!("and i1 {left}, {right}"), span, &mut self.debug),
                ),
                (
                    LlValue::Range {
                        value: left,
                        endpoint,
                    },
                    LlValue::Range {
                        value: right,
                        endpoint: right_endpoint,
                    },
                ) => {
                    debug_assert_eq!(endpoint, right_endpoint);
                    LlValue::Range {
                        value: body.instruction(
                            &format!(
                                "call ptr @topal.runtime.range.{}.intersection(ptr {left}, ptr {right})",
                                numeric_domain(endpoint)
                            ),
                            span,
                            &mut self.debug,
                        ),
                        endpoint: endpoint.clone(),
                    }
                }
                _ => unreachable!("checked conjunction operands agree"),
            },
            CompilerBinary::Or => LlValue::Boolean(body.instruction(
                &format!("or i1 {}, {}", left.boolean(), right.boolean()),
                span,
                &mut self.debug,
            )),
            CompilerBinary::Xor => LlValue::Boolean(body.instruction(
                &format!("xor i1 {}, {}", left.boolean(), right.boolean()),
                span,
                &mut self.debug,
            )),
            CompilerBinary::Equal | CompilerBinary::NotEqual => {
                let predicate = if operation == CompilerBinary::Equal {
                    "eq"
                } else {
                    "ne"
                };
                let equal = self.emit_equal(left, right, body, span);
                LlValue::Boolean(body.instruction(
                    &format!("icmp {predicate} i1 {equal}, true"),
                    span,
                    &mut self.debug,
                ))
            }
            CompilerBinary::Less
            | CompilerBinary::Greater
            | CompilerBinary::LessEqual
            | CompilerBinary::GreaterEqual => {
                let predicate = match operation {
                    CompilerBinary::Less => "slt",
                    CompilerBinary::Greater => "sgt",
                    CompilerBinary::LessEqual => "sle",
                    CompilerBinary::GreaterEqual => "sge",
                    _ => unreachable!(),
                };
                let comparison = self.emit_total_compare(left, right, body, span);
                LlValue::Boolean(body.instruction(
                    &format!("icmp {predicate} i32 {comparison}, 0"),
                    span,
                    &mut self.debug,
                ))
            }
        }
    }

    #[allow(clippy::too_many_lines)] // Exhaustive structural value lowering keeps representation pairs visible.
    fn emit_equal(
        &mut self,
        left: &LlValue,
        right: &LlValue,
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        match (left, right) {
            (LlValue::Unit, LlValue::Unit) => {
                body.instruction("icmp eq i8 0, 0", span, &mut self.debug)
            }
            (LlValue::Completed(left), LlValue::Completed(right))
            | (LlValue::Effect(left), LlValue::Effect(right)) => body.instruction(
                &format!("icmp eq i8 {left}, {right}"),
                span,
                &mut self.debug,
            ),
            (LlValue::Boolean(left), LlValue::Boolean(right)) => body.instruction(
                &format!("icmp eq i1 {left}, {right}"),
                span,
                &mut self.debug,
            ),
            (LlValue::Comparison(left), LlValue::Comparison(right))
            | (LlValue::ErrorCode(left), LlValue::ErrorCode(right)) => body.instruction(
                &format!("icmp eq i32 {left}, {right}"),
                span,
                &mut self.debug,
            ),
            (
                LlValue::Enum {
                    value: left,
                    enumeration,
                },
                LlValue::Enum {
                    value: right,
                    enumeration: right_enumeration,
                },
            )
            | (
                LlValue::Function {
                    value: left,
                    enumeration,
                    ..
                },
                LlValue::Function {
                    value: right,
                    enumeration: right_enumeration,
                    ..
                },
            ) => {
                debug_assert_eq!(enumeration, right_enumeration);
                body.instruction(
                    &format!("icmp eq i32 {left}, {right}"),
                    span,
                    &mut self.debug,
                )
            }
            (LlValue::String(left), LlValue::String(right)) => body.instruction(
                &format!("call i1 @topal.runtime.string.equal(ptr {left}, ptr {right})"),
                span,
                &mut self.debug,
            ),
            (
                LlValue::Range {
                    value: left,
                    endpoint,
                },
                LlValue::Range {
                    value: right,
                    endpoint: right_endpoint,
                },
            ) => {
                debug_assert_eq!(endpoint, right_endpoint);
                body.instruction(
                    &format!(
                        "call i1 @topal.runtime.range.{}.equal(ptr {left}, ptr {right})",
                        numeric_domain(endpoint)
                    ),
                    span,
                    &mut self.debug,
                )
            }
            (
                LlValue::List {
                    value: left,
                    element,
                    ..
                },
                LlValue::List {
                    value: right,
                    element: right_element,
                    ..
                },
            ) => self.emit_list_equal(left, right, element, right_element, body, span),
            (
                LlValue::Optional {
                    value: left,
                    payload,
                    ..
                },
                LlValue::Optional {
                    value: right,
                    payload: right_payload,
                    ..
                },
            ) => self.emit_optional_equal(left, right, payload, right_payload, body, span),
            (
                LlValue::Result {
                    value: left,
                    success,
                    ..
                },
                LlValue::Result {
                    value: right,
                    success: right_success,
                    ..
                },
            ) => self.emit_result_equal(left, right, success, right_success, body, span),
            (LlValue::Int(_), LlValue::Int(_))
            | (LlValue::Modular { .. }, LlValue::Modular { .. })
            | (LlValue::Rational(_), LlValue::Rational(_)) => {
                let comparison = self.emit_numeric_compare(left, right, body, span);
                body.instruction(
                    &format!("icmp eq i32 {comparison}, 0"),
                    span,
                    &mut self.debug,
                )
            }
            (LlValue::Tuple(left), LlValue::Tuple(right)) => {
                debug_assert_eq!(left.len(), right.len());
                let mut fields = left.iter().zip(right);
                let Some((left, right)) = fields.next() else {
                    return "true".into();
                };
                let mut equal = self.emit_equal(left, right, body, span);
                for (left, right) in fields {
                    let field_equal = self.emit_equal(left, right, body, span);
                    equal = body.instruction(
                        &format!("and i1 {equal}, {field_equal}"),
                        span,
                        &mut self.debug,
                    );
                }
                equal
            }
            (LlValue::Record { fields: left, .. }, LlValue::Record { fields: right, .. }) => {
                self.emit_record_equal(left, right, body, span)
            }
            (
                LlValue::Sum {
                    tag: left_tag,
                    payloads: left_payloads,
                    sum,
                },
                LlValue::Sum {
                    tag: right_tag,
                    payloads: right_payloads,
                    sum: right_sum,
                },
            ) => {
                debug_assert_eq!(sum, right_sum);
                self.emit_sum_equal(
                    left_tag,
                    left_payloads,
                    right_tag,
                    right_payloads,
                    sum,
                    body,
                    span,
                )
            }
            _ => unreachable!("checked equality values agree"),
        }
    }

    #[allow(clippy::too_many_arguments)] // Both structural operands remain explicit at the lowering boundary.
    fn emit_sum_equal(
        &mut self,
        left_tag: &str,
        left_payloads: &[Option<Box<LlValue>>],
        right_tag: &str,
        right_payloads: &[Option<Box<LlValue>>],
        sum: &CompilerSumType,
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        debug_assert_eq!(left_payloads.len(), sum.alternatives.len());
        debug_assert_eq!(right_payloads.len(), sum.alternatives.len());
        let tags_equal = body.instruction(
            &format!("icmp eq i32 {left_tag}, {right_tag}"),
            span,
            &mut self.debug,
        );
        let compare_tag = body.label("sum.equal.tag");
        let unequal = body.label("sum.equal.unequal");
        let merge = body.label("sum.equal.merge");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {tags_equal}, label %{compare_tag}, label %{unequal}"),
            location,
        );
        body.start_block(&compare_tag);
        let alternatives = sum
            .alternatives
            .iter()
            .map(|_| body.label("sum.equal.alternative"))
            .collect::<Vec<_>>();
        let cases = alternatives
            .iter()
            .enumerate()
            .map(|(tag, label)| format!("i32 {tag}, label %{label}"))
            .collect::<Vec<_>>()
            .join(" ");
        body.terminator(
            &format!("switch i32 {left_tag}, label %{unequal} [ {cases} ]"),
            location,
        );

        let mut branches = Vec::with_capacity(sum.alternatives.len() + 1);
        for (index, ((alternative, left), right)) in sum
            .alternatives
            .iter()
            .zip(left_payloads)
            .zip(right_payloads)
            .enumerate()
        {
            body.start_block(&alternatives[index]);
            let equal = match (&alternative.payload, left, right) {
                (None, None, None) => "true".to_owned(),
                (Some(_), Some(left), Some(right)) => self.emit_equal(left, right, body, span),
                _ => unreachable!("checked sum equality retains every declared payload"),
            };
            let predecessor = body.current_block.clone();
            body.terminator(&format!("br label %{merge}"), location);
            branches.push((equal, predecessor));
        }
        body.start_block(&unequal);
        let unequal_predecessor = body.current_block.clone();
        body.terminator(&format!("br label %{merge}"), location);
        branches.push(("false".to_owned(), unequal_predecessor));

        body.start_block(&merge);
        body.instruction(
            &format!(
                "phi i1 {}",
                branches
                    .iter()
                    .map(|(equal, predecessor)| format!("[ {equal}, %{predecessor} ]"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            span,
            &mut self.debug,
        )
    }

    #[allow(clippy::too_many_lines)] // Each admitted private List layout selects one exact runtime.
    fn emit_list_equal(
        &mut self,
        left: &str,
        right: &str,
        element: &CompilerType,
        right_element: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        debug_assert_eq!(element, right_element);
        let runtime = if element == &CompilerType::Unit {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::Unit);
            "unit"
        } else if element == &CompilerType::Completed {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::Completed);
            "completed"
        } else if element == &CompilerType::Effect {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::Effect);
            "effect"
        } else if element == &CompilerType::Type {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::Type);
            "type"
        } else if matches!(element, CompilerType::Enum(_)) {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::Enum);
            "enum"
        } else if matches!(element, CompilerType::Modular(_)) {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::Modular);
            "modular"
        } else if matches!(element, CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::Int)
        {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::OptionalInt);
            "optional.int"
        } else if matches!(element, CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::Rational)
        {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::OptionalRational);
            "optional.rational"
        } else if matches!(element, CompilerType::Optional(payload) if payload.as_ref() == &CompilerType::String)
        {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::OptionalString);
            "optional.string"
        } else if compiler_integer_pair(element) {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::IntPair);
            "int.pair"
        } else if compiler_three_pointer_tuple(element) {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::IntTriple);
            "int.triple"
        } else if compiler_int_int_boolean_pair(element) {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::IntIntBooleanPair);
            "int-int-boolean-pair"
        } else if compiler_int_int_string_int_tuple(element) {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::IntIntStringIntTuple);
            "int-int-string-int"
        } else if matches!(element, CompilerType::Range(endpoint)
            if endpoint.as_ref() == &CompilerType::Int)
        {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::IntRange);
            "int.range"
        } else if compiler_int_string_pair(element) || compiler_int_list_pair(element) {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::IntStringPair);
            "int-string"
        } else if compiler_string_int_pair(element) {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::StringIntPair);
            "string-int"
        } else if compiler_string_pair(element) {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::StringPair);
            "string-pair"
        } else if compiler_boolean_string_pair(element) {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::BooleanStringPair);
            "boolean-string"
        } else if element == &CompilerType::Boolean {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::Boolean);
            "boolean"
        } else if element == &CompilerType::Comparison {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::Comparison);
            "comparison"
        } else if element == &CompilerType::ErrorCode {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::ErrorCode);
            "error.code"
        } else if matches!(element, CompilerType::Character | CompilerType::String) {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::String);
            "string"
        } else if element == &CompilerType::Rational {
            self.scalar_list_runtime_fragments
                .insert(ScalarListRuntimeFragment::Rational);
            "rational"
        } else if compiler_nested_int_string_list_element(element) {
            self.list_int_runtime_fragments
                .insert(ListIntRuntimeFragment::NestedIntStringCore);
            "nested.int-string"
        } else if compiler_nested_int_list_element(element) {
            self.list_int_runtime_fragments
                .insert(ListIntRuntimeFragment::NestedIntCore);
            "nested.int"
        } else {
            debug_assert!(matches!(element, CompilerType::Int | CompilerType::Nat));
            self.list_int_runtime_fragments
                .insert(ListIntRuntimeFragment::Core);
            "int"
        };
        body.instruction(
            &format!("call i1 @topal.runtime.list.{runtime}.equal(ptr {left}, ptr {right})"),
            span,
            &mut self.debug,
        )
    }

    fn emit_result_equal(
        &mut self,
        left: &str,
        right: &str,
        success: &CompilerType,
        right_success: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        debug_assert_eq!(success, right_success);
        assert!(
            matches!(success, CompilerType::Enum(_))
                || matches!(success, CompilerType::Tuple(fields)
                    if fields.as_slice() == [CompilerType::Int, CompilerType::String])
        );
        let left_error = body.instruction(
            &format!("call i1 @topal.runtime.result.is.error(ptr {left})"),
            span,
            &mut self.debug,
        );
        let right_error = body.instruction(
            &format!("call i1 @topal.runtime.result.is.error(ptr {right})"),
            span,
            &mut self.debug,
        );
        let any_error = body.instruction(
            &format!("or i1 {left_error}, {right_error}"),
            span,
            &mut self.debug,
        );
        let failure = body.label("result.product.equal.error");
        let success_label = body.label("result.product.equal.payload");
        let merge = body.label("result.product.equal.merge");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {any_error}, label %{failure}, label %{success_label}"),
            location,
        );
        body.start_block(&success_label);
        let left_payload = body.instruction(
            &format!("call ptr @topal.runtime.result.payload(ptr {left})"),
            span,
            &mut self.debug,
        );
        let right_payload = body.instruction(
            &format!("call ptr @topal.runtime.result.payload(ptr {right})"),
            span,
            &mut self.debug,
        );
        let left_value = self.result_success_value(&left_payload, success, body, span);
        let right_value = self.result_success_value(&right_payload, success, body, span);
        let payload_equal = self.emit_equal(&left_value, &right_value, body, span);
        let success_predecessor = body.current_block.clone();
        body.terminator(&format!("br label %{merge}"), location);
        body.start_block(&failure);
        let failure_predecessor = body.current_block.clone();
        body.terminator(&format!("br label %{merge}"), location);
        body.start_block(&merge);
        body.instruction(
            &format!(
                "phi i1 [ {payload_equal}, %{success_predecessor} ], [ false, %{failure_predecessor} ]"
            ),
            span,
            &mut self.debug,
        )
    }

    fn emit_optional_equal(
        &mut self,
        left: &str,
        right: &str,
        payload: &CompilerType,
        right_payload: &CompilerType,
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        debug_assert_eq!(payload, right_payload);
        let runtime = match payload {
            CompilerType::Int => "optional.int.equal",
            CompilerType::Rational => "optional.rational.equal",
            CompilerType::String => "optional.string.equal",
            payload
                if matches!(payload, CompilerType::Enum(_) | CompilerType::Function)
                    || compiler_nested_int_list_element(payload)
                    || matches!(payload, CompilerType::Tuple(fields)
                        if fields.as_slice() == [CompilerType::Int, CompilerType::String]) =>
            {
                let left_some = body.instruction(
                    &format!("call i1 @topal.runtime.optional.is.some(ptr {left})"),
                    span,
                    &mut self.debug,
                );
                let right_some = body.instruction(
                    &format!("call i1 @topal.runtime.optional.is.some(ptr {right})"),
                    span,
                    &mut self.debug,
                );
                let both_some = body.instruction(
                    &format!("and i1 {left_some}, {right_some}"),
                    span,
                    &mut self.debug,
                );
                let payload_label = body.label("optional.product.equal.payload");
                let tag_label = body.label("optional.product.equal.tag");
                let merge_label = body.label("optional.product.equal.merge");
                let location = self.debug.location(span, body.subprogram);
                body.terminator(
                    &format!("br i1 {both_some}, label %{payload_label}, label %{tag_label}"),
                    location,
                );
                body.start_block(&payload_label);
                let left_payload = body.instruction(
                    &format!("call ptr @topal.runtime.optional.payload(ptr {left})"),
                    span,
                    &mut self.debug,
                );
                let right_payload = body.instruction(
                    &format!("call ptr @topal.runtime.optional.payload(ptr {right})"),
                    span,
                    &mut self.debug,
                );
                let left_value =
                    self.emit_optional_payload_value(left_payload, payload, body, span);
                let right_value =
                    self.emit_optional_payload_value(right_payload, payload, body, span);
                let payload_equal = self.emit_equal(&left_value, &right_value, body, span);
                let payload_predecessor = body.current_block.clone();
                body.terminator(&format!("br label %{merge_label}"), location);
                body.start_block(&tag_label);
                let tags_equal = body.instruction(
                    &format!("icmp eq i1 {left_some}, {right_some}"),
                    span,
                    &mut self.debug,
                );
                let tag_predecessor = body.current_block.clone();
                body.terminator(&format!("br label %{merge_label}"), location);
                body.start_block(&merge_label);
                return body.instruction(
                    &format!(
                        "phi i1 [ {payload_equal}, %{payload_predecessor} ], [ {tags_equal}, %{tag_predecessor} ]"
                    ),
                    span,
                    &mut self.debug,
                );
            }
            _ => unreachable!("checked Optional equality has canonical evidence"),
        };
        body.instruction(
            &format!("call i1 @topal.runtime.{runtime}(ptr {left}, ptr {right})"),
            span,
            &mut self.debug,
        )
    }

    fn emit_record_equal(
        &mut self,
        left: &[(String, LlValue)],
        right: &[(String, LlValue)],
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        debug_assert_eq!(left.len(), right.len());
        let mut equal = None;
        for (label, left) in left {
            let right = right
                .iter()
                .find_map(|(name, value)| (name == label).then_some(value))
                .unwrap_or_else(|| panic!("checked Record equality retains `{label}`"));
            let field_equal = self.emit_equal(left, right, body, span);
            equal = Some(match equal {
                None => field_equal,
                Some(equal) => body.instruction(
                    &format!("and i1 {equal}, {field_equal}"),
                    span,
                    &mut self.debug,
                ),
            });
        }
        equal.unwrap_or_else(|| "true".into())
    }

    fn emit_total_compare(
        &mut self,
        left: &LlValue,
        right: &LlValue,
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        match (left, right) {
            (LlValue::Int(_), LlValue::Int(_))
            | (LlValue::Modular { .. }, LlValue::Modular { .. })
            | (LlValue::Rational(_), LlValue::Rational(_)) => {
                self.emit_numeric_compare(left, right, body, span)
            }
            (LlValue::Tuple(left), LlValue::Tuple(right)) => {
                debug_assert_eq!(left.len(), right.len());
                let mut fields = left.iter().zip(right);
                let Some((left, right)) = fields.next() else {
                    return "0".into();
                };
                let mut comparison = self.emit_total_compare(left, right, body, span);
                for (left, right) in fields {
                    let comparison_block = body.current_block.clone();
                    let compare_next = body.label("tuple.compare.next");
                    let compare_done = body.label("tuple.compare.done");
                    let equal = body.instruction(
                        &format!("icmp eq i32 {comparison}, 0"),
                        span,
                        &mut self.debug,
                    );
                    let location = self.debug.location(span, body.subprogram);
                    body.terminator(
                        &format!("br i1 {equal}, label %{compare_next}, label %{compare_done}"),
                        location,
                    );
                    body.start_block(&compare_next);
                    let next = self.emit_total_compare(left, right, body, span);
                    let next_block = body.current_block.clone();
                    body.terminator(&format!("br label %{compare_done}"), location);
                    body.start_block(&compare_done);
                    comparison = body.instruction(
                        &format!(
                            "phi i32 [ {comparison}, %{comparison_block} ], [ {next}, %{next_block} ]"
                        ),
                        span,
                        &mut self.debug,
                    );
                }
                comparison
            }
            _ => unreachable!("checked total-order values agree"),
        }
    }

    fn emit_numeric_compare(
        &mut self,
        left: &LlValue,
        right: &LlValue,
        body: &mut FunctionBody,
        span: Span,
    ) -> String {
        let (domain, left, right) = match (left, right) {
            (LlValue::Int(left), LlValue::Int(right)) => ("int", left, right),
            (
                LlValue::Modular {
                    value: left,
                    modular,
                },
                LlValue::Modular {
                    value: right,
                    modular: right_modular,
                },
            ) => {
                debug_assert_eq!(modular, right_modular);
                ("int", left, right)
            }
            (LlValue::Rational(left), LlValue::Rational(right)) => ("rational", left, right),
            _ => unreachable!("checked numeric comparison operands agree"),
        };
        body.instruction(
            &format!("call i32 @topal.runtime.{domain}.compare(ptr {left}, ptr {right})"),
            span,
            &mut self.debug,
        )
    }

    fn emit_boolean_decision(
        &mut self,
        subject: &CompilerExpression,
        when_true: &CompilerExpression,
        when_false: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let emitted_subject = self.emit_expression(subject, body, environment);
        let subject = emitted_subject.boolean();
        let true_label = body.label("decision.true");
        let false_label = body.label("decision.false");
        let merge_label = body.label("decision.merge");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("br i1 {subject}, label %{true_label}, label %{false_label}"),
            location,
        );

        body.start_block(&true_label);
        let true_value = self.emit_expression(when_true, body, environment);
        let true_predecessor = body.current_block.clone();
        let true_location = self.debug.location(when_true.span, body.subprogram);
        body.terminator(&format!("br label %{merge_label}"), true_location);

        body.start_block(&false_label);
        let false_value = self.emit_expression(when_false, body, environment);
        let false_predecessor = body.current_block.clone();
        let false_location = self.debug.location(when_false.span, body.subprogram);
        body.terminator(&format!("br label %{merge_label}"), false_location);

        body.start_block(&merge_label);
        self.emit_decision_phi(
            &[
                (true_value, true_predecessor),
                (false_value, false_predecessor),
            ],
            body,
            span,
        )
    }

    #[allow(clippy::too_many_arguments)] // Mirrors the checked decision node without hiding evaluation order.
    fn emit_ordered_comparison_decision(
        &mut self,
        subject: &CompilerExpression,
        rules: &[CompilerComparisonRule],
        otherwise: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let subject = self.emit_expression(subject, body, environment);
        let merge = body.label("comparison.decision.merge");
        let mut branches = Vec::with_capacity(rules.len() + 1);
        for rule in rules {
            let action_label = body.label("comparison.decision.action");
            let next_label = body.label("comparison.decision.next");
            let operand = self.emit_expression(&rule.operand, body, environment);
            let comparable_subject = if rule.subject_to_rational {
                LlValue::Rational(body.instruction(
                    &format!(
                        "call ptr @topal.runtime.rational.from.int(ptr {})",
                        subject.integer()
                    ),
                    rule.span,
                    &mut self.debug,
                ))
            } else {
                subject.clone()
            };
            let predicate = self
                .emit_binary(
                    rule.operation,
                    &comparable_subject,
                    &operand,
                    body,
                    rule.span,
                )
                .boolean()
                .to_owned();
            let location = self.debug.location(rule.span, body.subprogram);
            body.terminator(
                &format!("br i1 {predicate}, label %{action_label}, label %{next_label}"),
                location,
            );
            body.start_block(&action_label);
            let action = self.emit_expression(&rule.action, body, environment);
            let predecessor = body.current_block.clone();
            body.terminator(&format!("br label %{merge}"), location);
            branches.push((action, predecessor));
            body.start_block(&next_label);
        }
        let fallback = self.emit_expression(otherwise, body, environment);
        let fallback_predecessor = body.current_block.clone();
        let fallback_location = self.debug.location(otherwise.span, body.subprogram);
        body.terminator(&format!("br label %{merge}"), fallback_location);
        branches.push((fallback, fallback_predecessor));
        body.start_block(&merge);
        self.emit_decision_phi(&branches, body, span)
    }

    #[allow(clippy::too_many_arguments)] // Mirrors the three closed Comparison alternatives.
    fn emit_comparison_value_decision(
        &mut self,
        subject: &CompilerExpression,
        when_less: &CompilerExpression,
        when_equal: &CompilerExpression,
        when_greater: &CompilerExpression,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let emitted_subject = self.emit_expression(subject, body, environment);
        let LlValue::Comparison(subject) = emitted_subject else {
            unreachable!("checked decision subject is Comparison")
        };
        let less = body.label("comparison.value.less");
        let equal = body.label("comparison.value.equal");
        let greater = body.label("comparison.value.greater");
        let merge = body.label("comparison.value.merge");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!(
                "switch i32 {subject}, label %{greater} [ i32 -1, label %{less} i32 0, label %{equal} ]"
            ),
            location,
        );
        let mut branches = Vec::with_capacity(3);
        for (label, action) in [
            (&less, when_less),
            (&equal, when_equal),
            (&greater, when_greater),
        ] {
            body.start_block(label);
            let value = self.emit_expression(action, body, environment);
            let predecessor = body.current_block.clone();
            let action_location = self.debug.location(action.span, body.subprogram);
            body.terminator(&format!("br label %{merge}"), action_location);
            branches.push((value, predecessor));
        }
        body.start_block(&merge);
        self.emit_decision_phi(&branches, body, span)
    }

    fn emit_enum_decision(
        &mut self,
        subject: &CompilerExpression,
        rules: &[CompilerEnumRule],
        otherwise: Option<&CompilerExpression>,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let emitted_subject = self.emit_expression(subject, body, environment);
        let LlValue::Enum { value: subject, .. } = emitted_subject else {
            unreachable!("checked decision subject is a nominal Enum")
        };
        let labels = rules
            .iter()
            .map(|_| body.label("enum.decision.alternative"))
            .collect::<Vec<_>>();
        let default = body.label("enum.decision.default");
        let merge = body.label("enum.decision.merge");
        let cases = rules
            .iter()
            .zip(&labels)
            .map(|(rule, label)| format!("i32 {}, label %{label}", rule.value))
            .collect::<Vec<_>>()
            .join(" ");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("switch i32 {subject}, label %{default} [ {cases} ]"),
            location,
        );
        let mut branches = Vec::with_capacity(rules.len() + usize::from(otherwise.is_some()));
        for (rule, label) in rules.iter().zip(labels) {
            body.start_block(&label);
            let value = self.emit_expression(&rule.action, body, environment);
            let predecessor = body.current_block.clone();
            let action_location = self.debug.location(rule.action.span, body.subprogram);
            body.terminator(&format!("br label %{merge}"), action_location);
            branches.push((value, predecessor));
        }
        body.start_block(&default);
        if let Some(otherwise) = otherwise {
            let value = self.emit_expression(otherwise, body, environment);
            let predecessor = body.current_block.clone();
            let action_location = self.debug.location(otherwise.span, body.subprogram);
            body.terminator(&format!("br label %{merge}"), action_location);
            branches.push((value, predecessor));
        } else {
            body.effect(
                "call void @topal.platform.exit(i64 70)",
                span,
                &mut self.debug,
            );
            body.terminator("unreachable", location);
        }
        body.start_block(&merge);
        self.emit_decision_phi(&branches, body, span)
    }

    fn emit_sum_decision(
        &mut self,
        subject: &CompilerExpression,
        rules: &[CompilerSumRule],
        otherwise: Option<&CompilerExpression>,
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
        span: Span,
    ) -> LlValue {
        let emitted_subject = self.emit_expression(subject, body, environment);
        let LlValue::Sum { tag, payloads, sum } = emitted_subject else {
            unreachable!("checked decision subject is a nominal sum")
        };
        let labels = rules
            .iter()
            .map(|_| body.label("sum.decision.alternative"))
            .collect::<Vec<_>>();
        let default = body.label("sum.decision.default");
        let merge = body.label("sum.decision.merge");
        let cases = rules
            .iter()
            .zip(&labels)
            .map(|(rule, label)| format!("i32 {}, label %{label}", rule.value))
            .collect::<Vec<_>>()
            .join(" ");
        let location = self.debug.location(span, body.subprogram);
        body.terminator(
            &format!("switch i32 {tag}, label %{default} [ {cases} ]"),
            location,
        );
        let mut branches = Vec::with_capacity(rules.len() + usize::from(otherwise.is_some()));
        for (rule, label) in rules.iter().zip(labels) {
            body.start_block(&label);
            let mut branch_environment = environment.clone();
            if let Some((name, binding_span)) = &rule.binding {
                let index = usize::try_from(rule.value).expect("u32 sum tag fits usize");
                let payload_type = sum.alternatives[index]
                    .payload
                    .as_ref()
                    .expect("checked payload matcher retains its payload type");
                let payload = payloads[index]
                    .as_deref()
                    .expect("checked payload matcher retains its payload value")
                    .clone();
                let variable = self
                    .debug
                    .local(name, *binding_span, payload_type, body.subprogram);
                let binding_location = self.debug.location(*binding_span, body.subprogram);
                if payload_type.machine_scalar() {
                    body.debug_value(&payload, variable, binding_location);
                } else if private_aggregate_value_supported(payload_type) {
                    let aggregate = match (&payload, payload_type) {
                        (LlValue::Tuple(fields), CompilerType::Tuple(field_types)) => {
                            self.emit_tuple_aggregate(fields, field_types, body, *binding_span)
                        }
                        (LlValue::Record { fields, order }, CompilerType::Record(field_types)) => {
                            self.emit_record_aggregate(
                                fields,
                                order,
                                field_types,
                                body,
                                *binding_span,
                            )
                        }
                        (LlValue::Sum { .. }, CompilerType::Sum(_)) => {
                            self.emit_sum_aggregate(&payload, body, *binding_span)
                        }
                        _ => unreachable!("checked sum payload retains its aggregate type"),
                    };
                    self.emit_aggregate_debug_shadow(
                        &aggregate,
                        payload_type,
                        variable,
                        binding_location,
                        body,
                        *binding_span,
                    );
                }
                branch_environment.insert(name.clone(), payload);
            }
            let value = self.emit_expression(&rule.action, body, &branch_environment);
            let predecessor = body.current_block.clone();
            let action_location = self.debug.location(rule.action.span, body.subprogram);
            body.terminator(&format!("br label %{merge}"), action_location);
            branches.push((value, predecessor));
        }
        body.start_block(&default);
        if let Some(otherwise) = otherwise {
            let value = self.emit_expression(otherwise, body, environment);
            let predecessor = body.current_block.clone();
            let action_location = self.debug.location(otherwise.span, body.subprogram);
            body.terminator(&format!("br label %{merge}"), action_location);
            branches.push((value, predecessor));
        } else {
            body.effect(
                "call void @topal.platform.exit(i64 70)",
                span,
                &mut self.debug,
            );
            body.terminator("unreachable", location);
        }
        body.start_block(&merge);
        self.emit_decision_phi(&branches, body, span)
    }
}
