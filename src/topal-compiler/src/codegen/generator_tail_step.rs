impl Generator<'_> {
    fn emit_call_argument_values(
        &mut self,
        arguments: &[CompilerExpression],
        body: &mut FunctionBody,
        environment: &BTreeMap<String, LlValue>,
    ) -> Vec<LlValue> {
        let mut argument_environment = environment.clone();
        arguments
            .iter()
            .map(|argument| {
                let value = self.emit_expression(argument, body, &argument_environment);
                extend_function_capture_environment(&mut argument_environment, &value);
                value
            })
            .collect()
    }
    fn emit_tail_step(&mut self, function: &CompilerFunction) {
        let types = function
            .parameters
            .iter()
            .map(|p| p.value_type.clone())
            .collect::<Vec<_>>();
        let subprogram = self.debug.subprogram(
            &format!("{} [tail step]", function.source_name),
            "topal_tail_step_component_v1",
            function.span,
            &CompilerType::Unit,
            &types,
        );
        let mut body = FunctionBody::new(subprogram);
        body.effect(
            "call void @topal_tail_step_begin_v1(ptr %tail.context)",
            function.span,
            &mut self.debug,
        );
        let mut env = BTreeMap::new();
        for (index, parameter) in function.parameters.iter().enumerate() {
            let span = parameter.span;
            let value = match parameter.value_type {
                CompilerType::KernelByteRegion => LlValue::KernelRegion(body.instruction(
                    "call ptr @topal_tail_input_view_v1(ptr %tail.context)",
                    span,
                    &mut self.debug,
                )),
                CompilerType::Nat | CompilerType::Int => LlValue::Int(body.instruction(
                    &format!(
                        "call ptr @topal_tail_input_number_v1(ptr %tail.context, i64 {index})"
                    ),
                    span,
                    &mut self.debug,
                )),
                CompilerType::Boolean => {
                    let raw = body.instruction(
                        &format!(
                            "call i64 @topal_tail_input_boolean_v1(ptr %tail.context, i64 {index})"
                        ),
                        span,
                        &mut self.debug,
                    );
                    LlValue::Boolean(body.instruction(
                        &format!("icmp ne i64 {raw}, 0"),
                        span,
                        &mut self.debug,
                    ))
                }
                _ => unreachable!("rebuilt tail scalar interface"),
            };
            if parameter.source_visible && !parameter.discarded {
                let variable = self.debug.parameter(
                    &parameter.name,
                    index + 1,
                    span,
                    &parameter.value_type,
                    subprogram,
                );
                let location = self.debug.location(span, subprogram);
                body.debug_value(&value, variable, location);
            }
            env.insert(parameter.name.clone(), value);
        }
        self.emit_statements(&function.body.statements, &mut body, &mut env);
        self.emit_tail_position(&function.body.result, &function.symbol, &mut body, &env);
        self.functions.push(format!("define i64 @topal_tail_step_component_v1(ptr %tail.context) nounwind noinline noredzone noimplicitfloat \"disable-tail-calls\"=\"true\" !dbg !{subprogram} {{\n{}\n}}\n", body.render()));
    }

    fn emit_tail_position(
        &mut self,
        expression: &CompilerExpression,
        entry: &str,
        body: &mut FunctionBody,
        env: &BTreeMap<String, LlValue>,
    ) {
        let span = expression.span;
        match &expression.kind {
            CompilerExpressionKind::Call { symbol, arguments } if symbol == entry => {
                // All source argument effects finish, in order, before any
                // private copy can fail. No recursive native call is emitted.
                let values = self.emit_call_argument_values(arguments, body, env);
                for (index, value) in values.iter().enumerate() {
                    match value {
                        LlValue::KernelRegion(view) => body.effect(&format!("call void @topal_tail_validate_view_v1(ptr %tail.context, ptr {view})"), span, &mut self.debug),
                        LlValue::Int(number) => body.effect(&format!("call void @topal_tail_stage_number_v1(ptr %tail.context, i64 {index}, ptr {number})"), span, &mut self.debug),
                        LlValue::Boolean(boolean) => {
                            let value = body.instruction(&format!("zext i1 {boolean} to i64"), span, &mut self.debug);
                            body.effect(&format!("call void @topal_tail_stage_boolean_v1(ptr %tail.context, i64 {index}, i64 {value})"), span, &mut self.debug);
                        }
                        _ => unreachable!("rebuilt tail argument"),
                    }
                }
                let location = self.debug.location(span, body.subprogram);
                body.terminator("ret i64 1", location);
            }
            CompilerExpressionKind::Block(block) => {
                let mut local = env.clone();
                self.emit_statements(&block.statements, body, &mut local);
                self.emit_tail_position(&block.result, entry, body, &local);
            }
            CompilerExpressionKind::BooleanDecision {
                subject,
                when_true,
                when_false,
            } => {
                let predicate = self
                    .emit_expression(subject, body, env)
                    .boolean()
                    .to_owned();
                let yes = body.label("tail.true");
                let no = body.label("tail.false");
                let location = self.debug.location(span, body.subprogram);
                body.terminator(
                    &format!("br i1 {predicate}, label %{yes}, label %{no}"),
                    location,
                );
                body.start_block(&yes);
                self.emit_tail_position(when_true, entry, body, env);
                body.start_block(&no);
                self.emit_tail_position(when_false, entry, body, env);
            }
            CompilerExpressionKind::OrderedComparisonDecision {
                subject,
                rules,
                otherwise,
            } => {
                let subject = self.emit_expression(subject, body, env);
                for rule in rules {
                    let operand = self.emit_expression(&rule.operand, body, env);
                    let predicate = self
                        .emit_binary(rule.operation, &subject, &operand, body, rule.span)
                        .boolean()
                        .to_owned();
                    let yes = body.label("tail.compare.action");
                    let no = body.label("tail.compare.next");
                    let location = self.debug.location(rule.span, body.subprogram);
                    body.terminator(
                        &format!("br i1 {predicate}, label %{yes}, label %{no}"),
                        location,
                    );
                    body.start_block(&yes);
                    self.emit_tail_position(&rule.action, entry, body, env);
                    body.start_block(&no);
                }
                self.emit_tail_position(otherwise, entry, body, env);
            }
            CompilerExpressionKind::OptionalDecision {
                subject,
                some_binding,
                some_action,
                none_action,
            } => {
                let optional = self.emit_expression(subject, body, env);
                let LlValue::Optional { value, payload, .. } = optional else {
                    unreachable!("checked Optional")
                };
                let predicate = body.instruction(
                    &format!("call i1 @topal.runtime.optional.is.some(ptr {value})"),
                    span,
                    &mut self.debug,
                );
                let yes = body.label("tail.some");
                let no = body.label("tail.none");
                let location = self.debug.location(span, body.subprogram);
                body.terminator(
                    &format!("br i1 {predicate}, label %{yes}, label %{no}"),
                    location,
                );
                body.start_block(&yes);
                let mut local = env.clone();
                if let Some((name, binding_span)) = some_binding {
                    let pointer = body.instruction(
                        &format!("call ptr @topal.runtime.optional.payload(ptr {value})"),
                        *binding_span,
                        &mut self.debug,
                    );
                    let value =
                        self.emit_optional_payload_value(pointer, &payload, body, *binding_span);
                    let variable = self
                        .debug
                        .local(name, *binding_span, &payload, body.subprogram);
                    let location = self.debug.location(*binding_span, body.subprogram);
                    body.debug_value(&value, variable, location);
                    local.insert(name.clone(), value);
                }
                self.emit_tail_position(some_action, entry, body, &local);
                body.start_block(&no);
                self.emit_tail_position(none_action, entry, body, env);
            }
            _ => {
                let value = self.emit_expression(expression, body, env);
                match value {
                    LlValue::Unit => body.effect("call void @topal_tail_result_unit_v1(ptr %tail.context)", span, &mut self.debug),
                    LlValue::Int(number) => body.effect(&format!("call void @topal_tail_result_number_v1(ptr %tail.context, ptr {number})"), span, &mut self.debug),
                    LlValue::Boolean(boolean) => {
                        let value = body.instruction(&format!("zext i1 {boolean} to i64"), span, &mut self.debug);
                        body.effect(&format!("call void @topal_tail_result_boolean_v1(ptr %tail.context, i64 {value})"), span, &mut self.debug);
                    }
                    _ => unreachable!("rebuilt tail result"),
                }
                let location = self.debug.location(span, body.subprogram);
                body.terminator("ret i64 0", location);
            }
        }
    }
}
