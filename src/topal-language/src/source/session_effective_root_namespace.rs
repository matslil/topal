impl Session {
    fn effective_root_namespace(&self) -> Rc<NamespaceValue> {
        self.root_namespace.clone().unwrap_or_else(|| {
            Rc::new(NamespaceValue {
                name: "root".into(),
                bindings: self.bindings.clone(),
                functions: (*self.functions).clone(),
                generators: (*self.generators).clone(),
            })
        })
    }

    fn layout_attributes(
        &self,
        source: &SourceText,
        expression: &Expression,
        trace: &mut impl TraceSink,
    ) -> Result<Vec<(String, Value)>, Diagnostic> {
        let Value::Record(attributes) = self.evaluate_expression(source, expression, trace)? else {
            return Err(diagnostic(
                source,
                "E-LAYOUT-ATTRIBUTES",
                expression.span(),
                "layout attributes require a labeled record",
            ));
        };
        Ok(attributes)
    }

    #[allow(clippy::too_many_lines)] // Closed layout forms and their diagnostics remain auditable together.
    fn evaluate_layout_application(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Option<Result<Value, Diagnostic>> {
        let identifier = |expression: &Expression| match expression {
            Expression::Identifier(name) => Some(source.slice(*name)),
            _ => None,
        };
        if let [operation, location] = items
            && identifier(operation) == Some("read")
            && let Some(name) = identifier(location)
            && let Some(Value::Location {
                layout, storage, ..
            }) = self.bindings.get(name)
        {
            if matches!(layout_access(layout), "WriteOnly" | "Reserved") {
                return Some(Err(diagnostic(
                    source,
                    "E-LAYOUT-NOT-READABLE",
                    span,
                    "location layout does not permit reads",
                )));
            }
            let result = storage.borrow().clone().ok_or_else(|| {
                diagnostic(
                    source,
                    "E-LAYOUT-UNINITIALIZED",
                    span,
                    "location has no stored layout value",
                )
            });
            if result.is_ok() {
                trace.record(TraceEvent {
                    event: "location.read",
                    rule: "TOPAL-LOCATION-READ-001",
                    detail: name,
                });
            }
            return Some(result);
        }
        if let [location, operation, value] = items
            && identifier(operation) == Some("write")
            && let Some(name) = identifier(location)
            && let Some(Value::Location {
                layout, storage, ..
            }) = self.bindings.get(name)
        {
            if matches!(layout_access(layout), "ReadOnly" | "Reserved") {
                return Some(Err(diagnostic(
                    source,
                    "E-LAYOUT-NOT-WRITABLE",
                    span,
                    "location layout does not permit writes",
                )));
            }
            let value_span = value.span();
            let result = self
                .evaluate_expression(source, value, trace)
                .and_then(|value| {
                    let stored = coerce_layout_value(source, value_span, layout, value)?;
                    *storage.borrow_mut() = Some(stored);
                    trace.record(TraceEvent {
                        event: "location.written",
                        rule: "TOPAL-LOCATION-WRITE-001",
                        detail: name,
                    });
                    Ok(Value::Unit)
                });
            return Some(result);
        }
        if let [attributes, constructor, semantic @ ..] = items
            && identifier(constructor) == Some("Layout")
            && !semantic.is_empty()
        {
            let result = self
                .layout_attributes(source, attributes, trace)
                .and_then(|attributes| {
                    let semantic_span = Span::new(
                        semantic.first().expect("checked nonempty").span().start,
                        semantic.last().expect("checked nonempty").span().end,
                    );
                    let semantic = source.slice(semantic_span).trim();
                    validate_layout_attributes(source, span, semantic, &attributes)?;
                    trace.record(TraceEvent {
                        event: "layout.constructed",
                        rule: "TOPAL-LAYOUT-CONSTRUCT-001",
                        detail: semantic,
                    });
                    Ok(Value::LayoutType(Box::new(LayoutValue {
                        semantic: semantic.into(),
                        attributes,
                    })))
                });
            return Some(result);
        }
        if let [constructor, semantic] = items
            && identifier(constructor) == Some("Layout")
        {
            if matches!(semantic, Expression::Product { .. }) {
                return Some(
                    self.layout_attributes(source, semantic, trace)
                        .map(Value::LayoutFactory),
                );
            }
            let semantic = source.slice(semantic.span()).trim();
            let attributes = if semantic == "Unit" {
                vec![
                    ("storage-size".into(), Value::SizeBits(BigInt::from(0))),
                    (
                        "encoding".into(),
                        Value::Enum {
                            type_name: "LayoutEncoding".into(),
                            alternative: "Empty".into(),
                        },
                    ),
                ]
            } else {
                return Some(Err(diagnostic(
                    source,
                    "E-LAYOUT-ATTRIBUTES",
                    span,
                    "this semantic type requires explicit layout attributes",
                )));
            };
            return Some(Ok(Value::LayoutType(Box::new(LayoutValue {
                semantic: semantic.into(),
                attributes,
            }))));
        }
        if let [name, semantic @ ..] = items
            && !semantic.is_empty()
            && let Some(name) = identifier(name)
            && let Some(Value::LayoutFactory(attributes)) = self.bindings.get(name)
        {
            let semantic_span = Span::new(
                semantic.first().unwrap().span().start,
                semantic.last().unwrap().span().end,
            );
            let semantic = source.slice(semantic_span).trim();
            return Some(
                validate_layout_attributes(source, span, semantic, attributes).map(|()| {
                    Value::LayoutType(Box::new(LayoutValue {
                        semantic: semantic.into(),
                        attributes: attributes.clone(),
                    }))
                }),
            );
        }
        if let [constructor, attributes] = items
            && matches!(
                identifier(constructor),
                Some("AddressRange" | "AddressOffset")
            )
        {
            let kind = identifier(constructor).unwrap();
            return Some(
                self.layout_attributes(source, attributes, trace)
                    .map(|attributes| {
                        if kind == "AddressRange" {
                            Value::AddressRangeType(attributes)
                        } else {
                            Value::AddressOffsetType(attributes)
                        }
                    }),
            );
        }
        if let [attributes, constructor, argument] = items
            && matches!(
                identifier(constructor),
                Some("AddressRange" | "AddressOffset")
            )
        {
            let kind = identifier(constructor).unwrap();
            return Some(self.layout_attributes(source, attributes, trace).and_then(
                |attributes| {
                    let value = self.evaluate_expression(source, argument, trace)?;
                    if kind == "AddressRange" {
                        match value {
                            Value::IntRange { lower, upper, .. } if lower >= BigInt::from(0) => {
                                Ok(Value::AddressRange {
                                    attributes,
                                    lower,
                                    upper,
                                })
                            }
                            _ => Err(diagnostic(
                                source,
                                "E-ADDRESS-RANGE",
                                argument.span(),
                                "AddressRange requires a nonnegative Nat range",
                            )),
                        }
                    } else {
                        match value {
                            Value::Int(offset) if offset >= BigInt::from(0) => {
                                validate_address_offset(
                                    source,
                                    argument.span(),
                                    &attributes,
                                    &offset,
                                )?;
                                Ok(Value::AddressOffset { attributes, offset })
                            }
                            _ => Err(diagnostic(
                                source,
                                "E-ADDRESS-OFFSET",
                                argument.span(),
                                "AddressOffset requires a Nat byte offset",
                            )),
                        }
                    }
                },
            ));
        }
        if let [constructor, layout] = items
            && identifier(constructor) == Some("Location")
        {
            return Some(self.evaluate_expression(source, layout, trace).and_then(
                |value| match value {
                    Value::LayoutType(layout) => Ok(Value::LocationType(layout)),
                    _ => Err(diagnostic(
                        source,
                        "E-LOCATION-LAYOUT",
                        layout.span(),
                        "Location requires an explicit Layout value",
                    )),
                },
            ));
        }
        if let [name, argument] = items
            && let Some(name) = identifier(name)
            && let Some(constructor) = self.bindings.get(name)
        {
            let result = match constructor {
                Value::AddressRangeType(attributes) => Some(
                    self.evaluate_expression(source, argument, trace).and_then(
                        |value| match value {
                            Value::IntRange { lower, upper, .. } if lower >= BigInt::from(0) => {
                                Ok(Value::AddressRange {
                                    attributes: attributes.clone(),
                                    lower,
                                    upper,
                                })
                            }
                            _ => Err(diagnostic(
                                source,
                                "E-ADDRESS-RANGE",
                                argument.span(),
                                "AddressRange requires a nonnegative Nat range",
                            )),
                        },
                    ),
                ),
                Value::AddressOffsetType(attributes) => Some(
                    self.evaluate_expression(source, argument, trace).and_then(
                        |value| match value {
                            Value::Int(offset) if offset >= BigInt::from(0) => {
                                validate_address_offset(
                                    source,
                                    argument.span(),
                                    attributes,
                                    &offset,
                                )?;
                                Ok(Value::AddressOffset {
                                    attributes: attributes.clone(),
                                    offset,
                                })
                            }
                            _ => Err(diagnostic(
                                source,
                                "E-ADDRESS-OFFSET",
                                argument.span(),
                                "AddressOffset requires a Nat byte offset",
                            )),
                        },
                    ),
                ),
                Value::LocationType(layout) => Some(
                    self.evaluate_expression(source, argument, trace).and_then(
                        |offset| match offset {
                            Value::AddressOffset { attributes, offset } => {
                                validate_location_fit(
                                    source,
                                    argument.span(),
                                    layout,
                                    &attributes,
                                    &offset,
                                )?;
                                Ok(Value::Location {
                                    layout: layout.clone(),
                                    offset: Box::new(Value::AddressOffset { attributes, offset }),
                                    storage: Box::new(RefCell::new(None)),
                                })
                            }
                            _ => Err(diagnostic(
                                source,
                                "E-LOCATION-OFFSET",
                                argument.span(),
                                "a location requires an AddressOffset value",
                            )),
                        },
                    ),
                ),
                Value::LayoutType(layout) => Some(
                    self.evaluate_expression(source, argument, trace)
                        .and_then(|value| {
                            coerce_layout_value(source, argument.span(), layout, value)
                        }),
                ),
                _ => None,
            };
            if result.is_some() {
                return result;
            }
        }
        None
    }

    fn construct_task_instance(
        &self,
        source: &SourceText,
        definition_name: Span,
        argument: &Expression,
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let Some(Value::TaskDefinition(definition)) =
            self.bindings.get(source.slice(definition_name)).cloned()
        else {
            unreachable!("task definition was checked before construction");
        };
        let start = definition
            .handlers
            .get("start")
            .and_then(|handlers| handlers.first())
            .cloned()
            .expect("task definitions require start");
        let argument = self.evaluate_expression(source, argument, trace)?;
        if !function_accepts(&start.parameters, &argument) {
            return Err(diagnostic(
                source,
                "E-TASK-START-ARGUMENT",
                span,
                "task construction arguments do not match its start handler",
            ));
        }
        let state = definition
            .state_fields
            .iter()
            .map(|(name, _)| (name.clone(), Value::Unit))
            .collect();
        let (start_result, state) =
            self.invoke_task_handler(&definition, &start, argument, state, trace)?;
        if matches!(start_result, Value::Error { .. }) {
            trace.record(TraceEvent {
                event: "task.start.failed",
                rule: "TOPAL-TASK-LIFECYCLE-001",
                detail: &definition.name,
            });
            return Ok(start_result);
        }
        for (name, classifier) in &definition.state_fields {
            if !state
                .get(name)
                .is_some_and(|value| value_has_classifier(value, classifier))
            {
                return Err(diagnostic(
                    source,
                    "E-TASK-STATE-INITIALIZATION",
                    span,
                    format!("start did not initialize `{name}` as `{classifier}`"),
                ));
            }
        }
        let identity = self.next_task_identity.get();
        self.next_task_identity.set(identity + 1);
        let value = Value::TaskInstance(Box::new(RefCell::new(TaskInstanceValue {
            identity,
            definition: *definition,
            state,
            terminated: false,
        })));
        trace.record(TraceEvent {
            event: "task.started",
            rule: "TOPAL-TASK-LIFECYCLE-001",
            detail: &identity.to_string(),
        });
        Ok(value)
    }

    #[allow(clippy::too_many_lines)] // One message transaction keeps its stable trace and state transition together.
    fn evaluate_task_message(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [
            Expression::Identifier(instance_name),
            Expression::Identifier(operation),
            payload,
        ] = items
        else {
            return Err(diagnostic(
                source,
                "E-TASK-MESSAGE-SHAPE",
                span,
                "task messages require `task operation payload`",
            ));
        };
        let instance_key = source.slice(*instance_name).to_owned();
        let Some(Value::TaskInstance(instance)) = self.bindings.get(&instance_key) else {
            unreachable!("task instance was checked before message execution");
        };
        let instance_snapshot = instance.borrow().clone();
        let operation_name = source.slice(*operation);
        if operation_name == "start" {
            return Err(diagnostic(
                source,
                "E-TASK-START-PRIVATE",
                *operation,
                "start is a lifecycle handler and cannot receive a message",
            ));
        }
        if instance_snapshot
            .definition
            .streams
            .contains_key(operation_name)
        {
            return self.construct_task_stream(
                source,
                &instance_key,
                operation_name,
                payload,
                span,
                trace,
            );
        }
        let handler = instance_snapshot
            .definition
            .handlers
            .get(operation_name)
            .and_then(|handlers| handlers.first())
            .cloned()
            .ok_or_else(|| {
                diagnostic(
                    source,
                    "E-TASK-HANDLER",
                    *operation,
                    "task capability exposes no such message handler",
                )
            })?;
        if instance_snapshot.terminated {
            if handler.result == "Unit" {
                trace.record(TraceEvent {
                    event: "message.discarded.terminated",
                    rule: "TOPAL-TASK-LIFECYCLE-001",
                    detail: operation_name,
                });
                return Ok(Value::Unit);
            }
            let position = source.position(operation.start);
            return Ok(Value::Error {
                domain: "lang task".into(),
                code: "task-terminated".into(),
                line: position.line,
                column: position.column,
            });
        }
        let payload = self.evaluate_expression(source, payload, trace)?;
        if operation_name == "terminate" {
            if !function_accepts(&handler.parameters, &payload) {
                return Err(diagnostic(
                    source,
                    "E-TASK-TERMINATE-ARGUMENT",
                    span,
                    "termination reason does not match the lifecycle handler",
                ));
            }
            let (result, state) = self.invoke_task_handler(
                &instance_snapshot.definition,
                &handler,
                payload,
                instance_snapshot.state.clone(),
                trace,
            )?;
            let mut updated = instance_snapshot;
            updated.state = state;
            updated.terminated = true;
            *instance.borrow_mut() = updated;
            trace.record(TraceEvent {
                event: "task.terminated",
                rule: "TOPAL-TASK-LIFECYCLE-001",
                detail: &instance_key,
            });
            return Ok(result);
        }
        let context = Value::Record(vec![
            (
                "session-id".into(),
                Value::Int(BigInt::from(self.next_transaction_identity.get())),
            ),
            ("sender".into(), Value::String("root".into())),
        ]);
        let argument = match handler.parameters.len() {
            1 => context,
            2 => Value::Tuple(vec![context, payload]),
            _ => {
                return Err(diagnostic(
                    source,
                    "E-TASK-HANDLER-SHAPE",
                    *operation,
                    "message handler requires MessageContext plus zero or one ordinary operand",
                ));
            }
        };
        let transaction = self.next_transaction_identity.get();
        self.next_transaction_identity.set(transaction + 1);
        let detail = format!(
            "transaction={transaction};task={};operation={operation_name}",
            instance_snapshot.identity
        );
        trace.record(TraceEvent {
            event: "message.sent",
            rule: "TOPAL-CONC-INTERACT-001",
            detail: &detail,
        });
        trace.record(TraceEvent {
            event: "message.received",
            rule: "TOPAL-DEBUG-MESSAGE-001",
            detail: &detail,
        });
        let (result, state) = self.invoke_task_handler(
            &instance_snapshot.definition,
            &handler,
            argument,
            instance_snapshot.state.clone(),
            trace,
        )?;
        for (name, classifier) in &instance_snapshot.definition.state_fields {
            if !state
                .get(name)
                .is_some_and(|value| value_has_classifier(value, classifier))
            {
                return Err(diagnostic(
                    source,
                    "E-TASK-STATE-REPLACEMENT",
                    span,
                    format!("message left `{name}` outside `{classifier}`"),
                ));
            }
        }
        let mut updated = instance_snapshot;
        updated.state = state;
        *instance.borrow_mut() = updated;
        trace.record(TraceEvent {
            event: "message.completed",
            rule: "TOPAL-CONC-ORDER-001",
            detail: &detail,
        });
        Ok(result)
    }

    #[allow(clippy::too_many_lines)] // Initial delivery, suspension state, and transaction evidence remain together.
    fn construct_task_stream(
        &self,
        source: &SourceText,
        instance_name: &str,
        operation: &str,
        payload: &Expression,
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let Some(Value::TaskInstance(instance)) = self.bindings.get(instance_name) else {
            unreachable!("task stream owner was resolved before dispatch")
        };
        let snapshot = instance.borrow().clone();
        if snapshot.terminated {
            let position = source.position(span.start);
            return Ok(Value::Error {
                domain: "lang task".into(),
                code: "task-terminated".into(),
                line: position.line,
                column: position.column,
            });
        }
        let payload = self.evaluate_expression(source, payload, trace)?;
        let context = Value::Record(vec![
            (
                "session-id".into(),
                Value::Int(BigInt::from(self.next_transaction_identity.get())),
            ),
            ("sender".into(), Value::String("root".into())),
        ]);
        let candidates = &snapshot.definition.streams[operation];
        let argument = if candidates
            .iter()
            .any(|candidate| candidate.parameters.len() == 2)
        {
            Value::Tuple(vec![context, payload])
        } else {
            context
        };
        let generator = candidates
            .iter()
            .find(|candidate| function_accepts(&candidate.parameters, &argument))
            .cloned()
            .ok_or_else(|| {
                diagnostic(
                    source,
                    "E-TASK-STREAM-ARGUMENT",
                    span,
                    "no stream handler accepts this payload",
                )
            })?;
        let transaction = self.next_transaction_identity.get();
        self.next_transaction_identity.set(transaction + 1);
        let detail = format!(
            "transaction={transaction};task={};operation={operation}",
            snapshot.identity
        );
        trace.record(TraceEvent {
            event: "message.sent",
            rule: "TOPAL-CONC-INTERACT-001",
            detail: &detail,
        });
        trace.record(TraceEvent {
            event: "message.received",
            rule: "TOPAL-DEBUG-MESSAGE-001",
            detail: &detail,
        });
        let mut scope = Self {
            bindings: generator.bindings.clone(),
            functions: Box::new(snapshot.definition.handlers.clone()),
            generators: Box::new(snapshot.definition.streams.clone()),
            root_namespace: Some(self.effective_root_namespace()),
            defining_context: self.defining_context.clone(),
            declared_names: BTreeSet::new(),
            published_names: BTreeSet::new(),
            documentation: self.documentation.clone(),
            language_version: self.language_version,
            language_features: self.language_features.clone(),
            declared_libraries: self.declared_libraries.clone(),
            consumed_names: BTreeSet::new(),
            local_function_names: BTreeSet::new(),
            enum_types: self.enum_types.clone(),
            union_types: self.union_types.clone(),
            generic_types: self.generic_types.clone(),
            call_stack: Vec::new(),
            static_context: false,
            task_state: Some(snapshot.state),
            next_task_identity: Cell::new(self.next_task_identity.get()),
            next_transaction_identity: Cell::new(self.next_transaction_identity.get()),
        };
        bind_generator_arguments(&mut scope, &generator.parameters, argument, trace);
        let mut cursor = 0;
        let mut pending_yield = None;
        let mut resume_binding = None;
        let mut returned = None;
        advance_custom_generator(
            &generator.source,
            &generator.body,
            &mut cursor,
            &mut scope,
            &mut pending_yield,
            &mut resume_binding,
            &mut returned,
            &generator.yielded,
            &generator.result,
            operation,
            trace,
        )?;
        sync_stream_task_state(self, Some(instance_name), scope.task_state.as_ref());
        trace.record(TraceEvent {
            event: "message.stream.started",
            rule: "TOPAL-TASK-MESSAGE-001",
            detail: &detail,
        });
        Ok(Value::SuspendedGenerator {
            source: Box::new(generator.source),
            body: Box::new(generator.body),
            cursor,
            bindings: Box::new(scope.bindings),
            scope_state: Box::new(GeneratorScopeState {
                functions: *scope.functions,
                declared_names: scope.declared_names,
                local_function_names: scope.local_function_names,
                enum_types: scope.enum_types,
                union_types: scope.union_types,
            }),
            pending_yield,
            resume_binding,
            returned: returned.map(Box::new),
            yield_classifier: generator.yielded,
            return_classifier: generator.result,
            origin: format!("task.{operation}.transaction-{transaction}"),
            task_state: scope.task_state,
            task_owner: Some(instance_name.into()),
        })
    }

    #[allow(clippy::too_many_lines)] // Handler contracts and state remain one auditable boundary.
    fn invoke_task_handler(
        &self,
        definition: &TaskDefinitionValue,
        function: &UserFunction,
        argument: Value,
        state: BTreeMap<String, Value>,
        trace: &mut impl TraceSink,
    ) -> Result<(Value, BTreeMap<String, Value>), Diagnostic> {
        let mut scope = Self {
            bindings: function.bindings.clone(),
            functions: Box::new(definition.handlers.clone()),
            generators: self.generators.clone(),
            root_namespace: Some(self.effective_root_namespace()),
            defining_context: function_defining_context(function),
            declared_names: BTreeSet::new(),
            published_names: BTreeSet::new(),
            documentation: self.documentation.clone(),
            language_version: self.language_version,
            language_features: self.language_features.clone(),
            declared_libraries: self.declared_libraries.clone(),
            consumed_names: BTreeSet::new(),
            local_function_names: BTreeSet::new(),
            enum_types: self.enum_types.clone(),
            union_types: self.union_types.clone(),
            generic_types: self.generic_types.clone(),
            call_stack: vec![ActiveCall {
                name: definition.name.clone(),
                signature: function_signature(&definition.name, function),
                termination_rule: None,
                recursion_target: None,
            }],
            static_context: false,
            task_state: Some(state),
            next_task_identity: Cell::new(self.next_task_identity.get()),
            next_transaction_identity: Cell::new(self.next_transaction_identity.get()),
        };
        bind_function_arguments(
            &mut scope,
            function,
            argument,
            trace,
            "TOPAL-FUNCTION-ORDINARY-001",
        )?;
        if let Some(requirement) = function
            .metadata
            .contracts
            .as_ref()
            .and_then(|contracts| contracts.requires.as_ref())
        {
            let proof = scope.evaluate_expression(&function.source, requirement, trace)?;
            if proof != Value::Boolean(true) {
                return Err(diagnostic(
                    &function.source,
                    "E-CONTRACT-REQUIRES",
                    requirement.span(),
                    "task handler precondition is not proven for this invocation",
                ));
            }
        }
        let mut execution = Execution {
            source: definition.source.clone(),
            statements: (*function.body).clone(),
            cursor: 0,
            result_classifier: Some(function.result.clone()),
            return_classifier: Some(function.result.clone()),
        };
        let value = loop {
            match execution.step(&mut scope, trace)? {
                ExecutionStep::Advanced { .. } => {}
                ExecutionStep::Complete(value) | ExecutionStep::Returned { value, .. } => {
                    break value;
                }
            }
        };
        if !value_has_classifier(&value, &function.result) {
            return Err(diagnostic(
                &definition.source,
                "E-TASK-HANDLER-RESULT",
                statement_span(function.body.last().expect("handler body is nonempty")),
                format!(
                    "task handler returned a value outside `{}`",
                    function.result
                ),
            ));
        }
        if let Some(relation) = function
            .metadata
            .contracts
            .as_ref()
            .and_then(|contracts| contracts.ensures.as_ref())
        {
            let binding = function
                .metadata
                .contracts
                .as_ref()
                .and_then(|contracts| contracts.result_binding.as_ref())
                .expect("parser requires a result binding for ensures");
            scope.bindings.insert(binding.clone(), value.clone());
            let proof = scope.evaluate_expression(&function.source, relation, trace)?;
            if proof != Value::Boolean(true) {
                return Err(diagnostic(
                    &function.source,
                    "E-CONTRACT-ENSURES",
                    relation.span(),
                    "task handler result does not satisfy its postcondition",
                ));
            }
        }
        Ok((value, scope.task_state.unwrap_or_default()))
    }

    #[inline(never)]
    fn enforce_function_precondition(
        &mut self,
        function: &UserFunction,
        name: &str,
        trace: &mut impl TraceSink,
    ) -> Result<(), Diagnostic> {
        let Some(requirement) = function
            .metadata
            .contracts
            .as_ref()
            .and_then(|contracts| contracts.requires.as_ref())
        else {
            return Ok(());
        };
        self.enforce_required_precondition(function, name, requirement, trace)
    }

    #[inline(never)]
    fn enforce_required_precondition(
        &mut self,
        function: &UserFunction,
        name: &str,
        requirement: &Expression,
        trace: &mut impl TraceSink,
    ) -> Result<(), Diagnostic> {
        let proof = self.evaluate_expression(&function.source, requirement, trace)?;
        if proof != Value::Boolean(true) {
            return Err(diagnostic(
                &function.source,
                "E-CONTRACT-REQUIRES",
                requirement.span(),
                "function precondition is not proven for this invocation",
            ));
        }
        trace.record(TraceEvent {
            event: "function.precondition.proven",
            rule: "TOPAL-CONTRACT-REQUIRES-001",
            detail: name,
        });
        Ok(())
    }

    #[inline(never)]
    fn enforce_function_postcondition(
        &mut self,
        function: &UserFunction,
        name: &str,
        value: &Value,
        trace: &mut impl TraceSink,
    ) -> Result<(), Diagnostic> {
        let Some(contracts) = function.metadata.contracts.as_ref() else {
            return Ok(());
        };
        let Some(relation) = contracts.ensures.as_ref() else {
            return Ok(());
        };
        self.enforce_required_postcondition(function, name, value, contracts, relation, trace)
    }

    #[inline(never)]
    fn enforce_required_postcondition(
        &mut self,
        function: &UserFunction,
        name: &str,
        value: &Value,
        contracts: &UserFunctionContracts,
        relation: &Expression,
        trace: &mut impl TraceSink,
    ) -> Result<(), Diagnostic> {
        let binding = contracts
            .result_binding
            .as_ref()
            .expect("parser requires a result binding for ensures");
        self.bindings.insert(binding.clone(), value.clone());
        let proof = self.evaluate_expression(&function.source, relation, trace)?;
        if proof != Value::Boolean(true) {
            return Err(diagnostic(
                &function.source,
                "E-CONTRACT-ENSURES",
                relation.span(),
                "function result does not satisfy its postcondition",
            ));
        }
        trace.record(TraceEvent {
            event: "function.postcondition.proven",
            rule: "TOPAL-CONTRACT-ENSURES-001",
            detail: name,
        });
        Ok(())
    }

    fn is_lang_operation(source: &SourceText, expression: &Expression, expected: &str) -> bool {
        matches!(
            expression,
            Expression::Application { items, .. }
                if matches!(items.as_slice(),
                    [Expression::Identifier(lang), Expression::Identifier(operation)]
                        if source.slice(*lang) == "lang" && source.slice(*operation) == expected)
        )
    }

    fn is_native_serialization(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items,
            [Expression::Identifier(lang), Expression::Identifier(operation), _]
                if source.slice(*lang) == "lang" && source.slice(*operation) == "deserialize")
            || matches!(items,
                [Expression::Identifier(lang), Expression::Identifier(version), operation]
                    if source.slice(*lang) == "lang" && source.slice(*version) == "version"
                        && Self::is_lang_operation(source, operation, "serialize"))
            || matches!(items, [_, operation, _] if Self::is_lang_operation(source, operation, "serialize"))
            || matches!(items,
                [Expression::Identifier(name), _]
                    if matches!(self.bindings.get(source.slice(*name)), Some(Value::NativeSerializer(_))))
    }

    #[allow(clippy::too_many_lines)] // Protocol boundary diagnostics stay beside each source form.
    fn evaluate_native_serialization(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        if let [
            Expression::Identifier(lang),
            Expression::Identifier(operation),
            stream,
        ] = items
            && source.slice(*lang) == "lang"
            && source.slice(*operation) == "deserialize"
        {
            let Value::SerializationStream(bytes) =
                self.evaluate_expression(source, stream, trace)?
            else {
                return Err(diagnostic(
                    source,
                    "E-DESERIALIZE-OPERAND",
                    stream.span(),
                    "lang deserialize requires a native SerializationStream",
                ));
            };
            let decoded =
                deserialize_native(&bytes, SerializationLimits::default()).map_err(|error| {
                    diagnostic(
                        source,
                        "E-DESERIALIZATION",
                        span,
                        format!(
                            "native stream rejected at {} byte {}: {}",
                            error.stage, error.offset, error.message
                        ),
                    )
                })?;
            let event = decoded.events.first().ok_or_else(|| {
                diagnostic(
                    source,
                    "E-DESERIALIZATION",
                    span,
                    "native stream contains no value event",
                )
            })?;
            let value = value_from_serialized(event, &decoded.types).ok_or_else(|| diagnostic(source, "E-DESERIALIZATION-OBJECT", span, "native value is understood but cannot be reconstructed by this interpreter revision"))?;
            trace.record(TraceEvent {
                event: "serialization.deserialized",
                rule: "TOPAL-SER-DESER-001",
                detail: "validated native event",
            });
            return Ok(value);
        }

        let (version, subject) = match items {
            [
                Expression::Identifier(lang),
                Expression::Identifier(version),
                operation,
            ] if source.slice(*lang) == "lang"
                && source.slice(*version) == "version"
                && Self::is_lang_operation(source, operation, "serialize") =>
            {
                return Ok(Value::NativeSerializer(self.language_version));
            }
            [version, operation, subject]
                if Self::is_lang_operation(source, operation, "serialize") =>
            {
                let Value::Version(version) = self.evaluate_expression(source, version, trace)?
                else {
                    return Err(diagnostic(
                        source,
                        "E-SERIALIZATION-VERSION",
                        version.span(),
                        "the left operand of lang serialize must be a Version",
                    ));
                };
                (version, subject)
            }
            [Expression::Identifier(name), subject] => {
                let Some(Value::NativeSerializer(version)) = self.bindings.get(source.slice(*name))
                else {
                    return Err(diagnostic(
                        source,
                        "E-SERIALIZATION-OPERATION",
                        span,
                        "expected a native serialization operation",
                    ));
                };
                (*version, subject)
            }
            _ => {
                return Err(diagnostic(
                    source,
                    "E-SERIALIZATION-OPERATION",
                    span,
                    "expected `version (lang serialize) value`",
                ));
            }
        };
        let value = self.evaluate_expression(source, subject, trace)?;
        let stream = stream_for_value(version, &value).map_err(|message| {
            diagnostic(source, "E-SERIALIZATION-VALUE", subject.span(), message)
        })?;
        let bytes = serialize_native(&stream)
            .map_err(|error| diagnostic(source, "E-SERIALIZATION", span, error.message))?;
        trace.record(TraceEvent {
            event: "serialization.serialized",
            rule: "TOPAL-SER-CANON-001",
            detail: &format!("{} bytes", bytes.len()),
        });
        Ok(Value::SerializationStream(bytes))
    }

    fn is_lang_introspection(source: &SourceText, items: &[Expression]) -> bool {
        let qualified_prefix = matches!(
            (items.first(), items.get(1)),
            (Some(Expression::Identifier(lang)), Some(Expression::Identifier(operation)))
                if source.slice(*lang) == "lang"
                    && matches!(source.slice(*operation),
                        "context" | "version" | "lint" | "identity" | "view" | "declaration" | "public-members")
        );
        let qualified_infix = matches!(
            (items.get(1), items.get(2)),
            (Some(Expression::Identifier(lang)), Some(Expression::Identifier(operation)))
                if source.slice(*lang) == "lang"
                    && matches!(source.slice(*operation),
                        "same-object" | "equivalent-type" | "compatible-with" | "same-layout")
        );
        qualified_prefix || qualified_infix
    }

    #[allow(clippy::too_many_lines)] // Qualified static operations remain explicit and auditable.
    fn evaluate_lang_introspection(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        if let [
            Expression::Identifier(lang),
            Expression::Identifier(operation),
        ] = items
            && source.slice(*lang) == "lang"
        {
            return match source.slice(*operation) {
                "context" => {
                    trace.record(TraceEvent {
                        event: "introspection.context.viewed",
                        rule: "TOPAL-SYN-CONTEXT-001",
                        detail: &self.language_version.to_string(),
                    });
                    Ok(Value::Introspection(Box::new(
                        IntrospectionValue::LanguageContext {
                            language: "topal".into(),
                            version: self.language_version,
                            features: self.language_features.iter().cloned().collect(),
                        },
                    )))
                }
                "version" => Ok(Value::Version(self.language_version)),
                "lint" => {
                    if !self.language_features.contains("lint") {
                        return Err(diagnostic(
                            source,
                            "E-LINT-VARIANT",
                            *operation,
                            "the `lang lint` namespace requires the `lint` language feature",
                        ));
                    }
                    trace.record(TraceEvent {
                        event: "lint.context.viewed",
                        rule: "TOPAL-SYN-CONTEXT-001",
                        detail: "lang lint",
                    });
                    Ok(Value::Namespace(Rc::new(NamespaceValue {
                        name: "lang lint".into(),
                        bindings: BTreeMap::new(),
                        functions: BTreeMap::new(),
                        generators: BTreeMap::new(),
                    })))
                }
                _ => Err(diagnostic(
                    source,
                    "E-INTROSPECTION-OPERATION",
                    *operation,
                    "unknown qualified static introspection operation",
                )),
            };
        }
        if let [
            Expression::Identifier(lang),
            Expression::Identifier(operation),
            subject,
        ] = items
            && source.slice(*lang) == "lang"
        {
            let subject_span = subject.span();
            let value = self.evaluate_expression(source, subject, trace)?;
            let result = match source.slice(*operation) {
                "identity" => Value::Introspection(Box::new(IntrospectionValue::Identity {
                    kind: value.object_kind(),
                    canonical: introspection_identity(&value).ok_or_else(|| {
                        diagnostic(
                            source,
                            "E-STATIC-INTROSPECTION-SUBJECT",
                            subject_span,
                            "lang identity requires a statically known language object",
                        )
                    })?,
                })),
                "view" => introspection_view(source, value, subject_span)?,
                "declaration" => {
                    let name = match subject {
                        Expression::Identifier(name) => Some(source.slice(*name).to_owned()),
                        _ => None,
                    };
                    Value::Introspection(Box::new(IntrospectionValue::DeclarationView {
                        canonical_path: name.as_ref().map(|name| format!("root.{name}")),
                        documentation: name
                            .as_ref()
                            .and_then(|name| self.documentation.get(name).cloned()),
                        name,
                        language_version: self.language_version,
                    }))
                }
                "public-members" => {
                    let Value::Namespace(namespace) = value else {
                        return Err(diagnostic(
                            source,
                            "E-INTROSPECTION-KIND",
                            subject_span,
                            "lang public-members requires a visible Scope",
                        ));
                    };
                    let mut members = if namespace.name == "root" {
                        self.published_names.iter().cloned().collect::<Vec<_>>()
                    } else {
                        namespace
                            .bindings
                            .keys()
                            .chain(namespace.functions.keys())
                            .chain(namespace.generators.keys())
                            .cloned()
                            .collect::<Vec<_>>()
                    };
                    members.sort();
                    members.dedup();
                    Value::Introspection(Box::new(IntrospectionValue::ScopeView {
                        identity: namespace.name.clone(),
                        members,
                    }))
                }
                _ => {
                    return Err(diagnostic(
                        source,
                        "E-INTROSPECTION-OPERATION",
                        *operation,
                        "unknown qualified static introspection operation",
                    ));
                }
            };
            trace.record(TraceEvent {
                event: "introspection.object.viewed",
                rule: "TOPAL-TYPE-KIND-001",
                detail: source.slice(*operation),
            });
            return Ok(result);
        }
        if let [
            left,
            Expression::Identifier(lang),
            Expression::Identifier(operation),
            right,
        ] = items
            && source.slice(*lang) == "lang"
        {
            let left = self.evaluate_expression(source, left, trace)?;
            let right = self.evaluate_expression(source, right, trace)?;
            let relation = source.slice(*operation);
            let result = match relation {
                "same-object" => {
                    let left = introspection_identity(&left);
                    let right = introspection_identity(&right);
                    match (left, right) {
                        (Some(left), Some(right)) => left == right,
                        _ => {
                            return Err(diagnostic(
                                source,
                                "E-STATIC-INTROSPECTION-SUBJECT",
                                span,
                                "lang same-object requires statically known language objects",
                            ));
                        }
                    }
                }
                "equivalent-type" | "compatible-with"
                    if matches!(left, Value::Type(_) | Value::ModularType(_))
                        && matches!(right, Value::Type(_) | Value::ModularType(_)) =>
                {
                    introspection_identity(&left) == introspection_identity(&right)
                }
                "equivalent-type" | "compatible-with" => {
                    return Err(diagnostic(
                        source,
                        "E-INTROSPECTION-KIND",
                        span,
                        "this relation requires two statically known Type objects",
                    ));
                }
                "same-layout" => {
                    return Err(diagnostic(
                        source,
                        "E-INTROSPECTION-KIND",
                        span,
                        "lang same-layout requires two explicit Layout values",
                    ));
                }
                _ => {
                    return Err(diagnostic(
                        source,
                        "E-INTROSPECTION-RELATION",
                        *operation,
                        "unknown qualified introspection relation",
                    ));
                }
            };
            trace.record(TraceEvent {
                event: "introspection.relation.compared",
                rule: "TOPAL-TYPE-ID-001",
                detail: relation,
            });
            return Ok(Value::Boolean(result));
        }
        Err(diagnostic(
            source,
            "E-INTROSPECTION-SYNTAX",
            span,
            "qualified introspection requires `lang operation subject`",
        ))
    }

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an interactive evaluation context for a supported language version.
    ///
    /// # Errors
    ///
    /// Returns an error when this tool does not implement `language_version`.
    pub fn for_language_version(language_version: LanguageVersion) -> Result<Self, &'static str> {
        if !language_version.is_supported_core() {
            return Err("the highest language version supported by this tool is v0.2");
        }
        Ok(Self {
            language_version,
            ..Self::default()
        })
    }

    /// The highest source language version implemented by this evaluator.
    #[must_use]
    pub const fn highest_supported_language_version() -> LanguageVersion {
        LanguageVersion::DESIGN_1
    }

    /// Evaluate one source file as an isolated module and bind its published
    /// interface under `name` in this session.
    ///
    /// # Errors
    ///
    /// Returns a source, semantic, or duplicate-module diagnostic.
    ///
    /// # Panics
    ///
    /// Panics only if an already accepted Rust string cannot be represented by
    /// the shared source layer while rendering a duplicate-name diagnostic.
    pub fn load_module(
        &mut self,
        name: &str,
        input: &str,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        if self.declared_names.contains(name) {
            let source = SourceText::new(input).expect("module input was accepted as UTF-8");
            return Err(diagnostic(
                &source,
                "E-DUPLICATE-MODULE",
                Span::new(0, 0),
                format!("module `{name}` is already declared"),
            ));
        }
        let mut module = Self::new();
        module.evaluate_source_file(input, trace)?;
        self.attach_module(name, module, trace)
    }

    /// Attach an already evaluated child scope under one canonical component.
    ///
    /// # Errors
    ///
    /// Returns a duplicate-module diagnostic when `name` is already declared.
    pub fn attach_module(
        &mut self,
        name: &str,
        module: Self,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        if self.declared_names.contains(name) {
            return Err(Diagnostic::error(
                "E-DUPLICATE-MODULE",
                1,
                1,
                format!("module `{name}` is already declared"),
            ));
        }
        let namespace = module.into_published_namespace(name);
        self.bindings.insert(name.to_owned(), namespace.clone());
        self.declared_names.insert(name.to_owned());
        self.published_names.insert(name.to_owned());
        trace.record(TraceEvent {
            event: "module.loaded",
            rule: "TOPAL-NAMESPACE-USE-001",
            detail: name,
        });
        Ok(namespace)
    }

    fn into_published_namespace(self, name: &str) -> Value {
        let bindings = self
            .bindings
            .into_iter()
            .filter(|(member, _)| self.published_names.contains(member))
            .collect();
        let functions = self
            .functions
            .into_iter()
            .filter(|(member, _)| self.published_names.contains(member))
            .collect();
        let generators = self
            .generators
            .into_iter()
            .filter(|(member, _)| self.published_names.contains(member))
            .collect();
        Value::Namespace(Rc::new(NamespaceValue {
            name: name.to_owned(),
            bindings,
            functions,
            generators,
        }))
    }

    /// Report whether a complete block statement should await a dedented line
    /// before an interactive session submits it.
    #[must_use]
    pub fn awaits_dedent(input: &str) -> bool {
        let Ok(source) = SourceText::new(input) else {
            return false;
        };
        let parsed = parse(&source, &lex(&source));
        parsed.diagnostics.is_empty()
            && matches!(
                parsed.statements.as_slice(),
                [Statement::Function { .. }
                    | Statement::Generator { .. }
                    | Statement::Union { .. }
                    | Statement::Interface { .. }
                    | Statement::Foreach { .. }]
            )
    }

    /// Evaluate one source unit and return its final value.
    ///
    /// # Errors
    ///
    /// Returns a source, syntax, name-resolution, or evaluation diagnostic.
    pub fn evaluate(
        &mut self,
        input: &str,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let mut execution = self.prepare(input, trace)?;
        loop {
            match execution.step(self, trace)? {
                ExecutionStep::Complete(value) => return Ok(value),
                ExecutionStep::Advanced { .. } => {}
                ExecutionStep::Returned { .. } => {
                    unreachable!("top-level return is rejected before completing a step")
                }
            }
        }
    }

    /// Evaluate a complete source file, including its mandatory language header.
    ///
    /// # Errors
    ///
    /// Returns a diagnostic when the header is absent or the source is invalid.
    pub fn evaluate_source_file(
        &mut self,
        input: &str,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let mut execution = self.prepare_source_file(input, trace)?;
        loop {
            match execution.step(self, trace)? {
                ExecutionStep::Complete(value) => return Ok(value),
                ExecutionStep::Advanced { .. } => {}
                ExecutionStep::Returned { .. } => unreachable!("top-level return is rejected"),
            }
        }
    }

    /// Prepare a source unit for resumable execution.
    ///
    /// # Errors
    ///
    /// Returns a source or syntax diagnostic before any statement executes.
    pub fn prepare(
        &self,
        input: &str,
        trace: &mut impl TraceSink,
    ) -> Result<Execution, Diagnostic> {
        self.checkpoint(trace, None, None);
        trace.record(TraceEvent {
            event: "context.selected",
            rule: "TOPAL-SYN-UNICODE-001",
            detail: "design-0;Unicode=17.0.0",
        });
        let source = accepted_source(input, trace)?;
        let lexed = lex(&source);
        let parsed = parse(&source, &lexed);
        if let Some(error) = parsed.diagnostics.first() {
            return Err(diagnostic(&source, error.code, error.span, &error.message));
        }
        if parsed.statements.is_empty() {
            return Err(expected_statement(input));
        }
        Ok(Execution {
            source,
            statements: parsed.statements,
            cursor: 0,
            result_classifier: None,
            return_classifier: None,
        })
    }
}
