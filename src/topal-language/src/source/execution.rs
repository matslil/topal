impl Execution {
    #[allow(clippy::too_many_lines)] // Traversal keeps consumption, action, resume, and return auditable together.
    fn execute_foreach(
        &self,
        session: &mut Session,
        trace: &mut impl TraceSink,
        source: &Expression,
        binding: Span,
        body: &[Statement],
        span: Span,
    ) -> Result<(Value, Span), Diagnostic> {
        let (generated, origin, returned, returned_classifier) = match source {
            Expression::Identifier(name) => {
                let name_text = self.source.slice(*name);
                let value = session.bindings.remove(name_text).ok_or_else(|| {
                    if session.consumed_names.contains(name_text) {
                        consumed_generator_diagnostic(&self.source, *name, name_text)
                    } else {
                        diagnostic(&self.source, "E-UNBOUND-NAME", *name, "name is not bound")
                    }
                })?;
                if let Value::SuspendedGenerator { .. } = value {
                    session.declared_names.remove(name_text);
                    session.consumed_names.insert(name_text.to_owned());
                    trace.record(TraceEvent {
                        event: "generator.consumed",
                        rule: "TOPAL-GENERATOR-DECLARATION-001",
                        detail: name_text,
                    });
                    return self
                        .execute_suspended_foreach(session, trace, value, binding, body, span);
                }
                if let Value::IterateGenerator { .. } = value {
                    session.declared_names.remove(name_text);
                    session.consumed_names.insert(name_text.to_owned());
                    trace.record(TraceEvent {
                        event: "generator.consumed",
                        rule: "TOPAL-GENERATOR-ITERATE-FOREACH-001",
                        detail: name_text,
                    });
                    return self
                        .execute_iterate_foreach(session, trace, value, binding, body, span);
                }
                let (generated, origin, returned, returned_classifier) = match value {
                    Value::CharacterGenerator { generated, origin } => (
                        generated.into_iter().map(Value::String).collect(),
                        origin,
                        Value::Unit,
                        "Unit",
                    ),
                    Value::CharacterReturningGenerator {
                        generated,
                        returned,
                        origin,
                    } => (
                        generated.into_iter().map(Value::String).collect(),
                        origin,
                        Value::String(returned),
                        "Character",
                    ),
                    Value::List {
                        element_classifier,
                        entries,
                    } => {
                        session.bindings.insert(
                            name_text.to_owned(),
                            Value::List {
                                element_classifier,
                                entries: entries.clone(),
                            },
                        );
                        session.declared_names.insert(name_text.to_owned());
                        (entries, "root.List".into(), Value::Unit, "Unit")
                    }
                    _ => return Err(foreach_source_diagnostic(&self.source, source.span())),
                };
                if origin != "root.List" {
                    session.declared_names.remove(name_text);
                    session.consumed_names.insert(name_text.to_owned());
                    trace.record(TraceEvent {
                        event: "generator.consumed",
                        rule: "TOPAL-STRING-CHARACTERS-GENERATOR-001",
                        detail: name_text,
                    });
                }
                (generated, origin, returned, returned_classifier)
            }
            Expression::Application { items, .. } => {
                let [Expression::Identifier(operation), text] = items.as_slice() else {
                    return Err(foreach_source_diagnostic(&self.source, source.span()));
                };
                if self.source.slice(*operation) != "characters" {
                    return Err(foreach_source_diagnostic(&self.source, source.span()));
                }
                let text_value = session.evaluate_expression(&self.source, text, trace)?;
                let Value::String(text_value) = text_value else {
                    return Err(diagnostic(
                        &self.source,
                        "E-CHARACTERS-OPERAND",
                        text.span(),
                        "characters requires a String operand",
                    ));
                };
                (
                    characters(&text_value)
                        .map(|character| Value::String(character.to_owned()))
                        .collect(),
                    "root.characters".to_owned(),
                    Value::Unit,
                    "Unit",
                )
            }
            _ => return Err(foreach_source_diagnostic(&self.source, source.span())),
        };
        let traversal_rule = if origin == "root.characters" {
            "TOPAL-STRING-CHARACTERS-FOREACH-001"
        } else if origin == "root.List" {
            "TOPAL-COLLECTION-FOREACH-001"
        } else {
            "TOPAL-GENERATOR-FOREACH-001"
        };
        let binding_name = self.source.slice(binding).to_owned();
        for entry in &generated {
            let mut iteration = session.clone();
            iteration
                .bindings
                .insert(binding_name.clone(), entry.clone());
            iteration.declared_names.insert(binding_name.clone());
            trace.record(TraceEvent {
                event: "generator.yielded",
                rule: traversal_rule,
                detail: &entry.to_string(),
            });
            let mut body_execution = Self {
                source: self.source.clone(),
                statements: body.to_vec(),
                cursor: 0,
                result_classifier: None,
                return_classifier: None,
            };
            loop {
                match body_execution.step(&mut iteration, trace)? {
                    ExecutionStep::Advanced { .. } => {}
                    ExecutionStep::Complete(Value::Unit) => break,
                    ExecutionStep::Complete(_) => {
                        return Err(diagnostic(
                            &self.source,
                            "E-FOREACH-ACTION-RESULT",
                            statement_span(body.last().expect("foreach body is nonempty")),
                            "foreach action must return Unit",
                        ));
                    }
                    ExecutionStep::Returned { .. } => {
                        unreachable!("foreach body has no function return context")
                    }
                }
            }
            trace.record(TraceEvent {
                event: "generator.resumed",
                rule: traversal_rule,
                detail: "Unit",
            });
        }
        trace.record(TraceEvent {
            event: "generator.returned",
            rule: generator_return_rule(
                &origin,
                generated.is_empty(),
                returned_classifier,
                traversal_rule,
            ),
            detail: returned_classifier,
        });
        Ok((returned, span))
    }

    fn execute_iterate_foreach(
        &self,
        session: &Session,
        trace: &mut impl TraceSink,
        generator: Value,
        binding: Span,
        body: &[Statement],
        span: Span,
    ) -> Result<(Value, Span), Diagnostic> {
        let Value::IterateGenerator {
            mut current,
            next,
            take_while,
            classifier,
        } = generator
        else {
            unreachable!("iterate traversal requires iterate generator")
        };
        let Some(predicate) = take_while else {
            return Err(diagnostic(
                &self.source,
                "E-UNBOUNDED-GENERATOR-TRAVERSAL",
                span,
                "complete foreach traversal of unbounded iterate requires a stopping transformation",
            ));
        };
        let binding_name = self.source.slice(binding).to_owned();
        loop {
            let accepted = session.invoke_anonymous_function(
                &predicate,
                vec![(*current).clone()],
                span,
                trace,
            )?;
            let Value::Boolean(accepted) = accepted else {
                return Err(diagnostic(
                    &self.source,
                    "E-TAKE-WHILE-PREDICATE-RESULT",
                    span,
                    "take-while predicate must return Boolean",
                ));
            };
            if !accepted {
                trace.record(TraceEvent {
                    event: "generator.returned",
                    rule: "TOPAL-GENERATOR-TAKE-WHILE-001",
                    detail: "Unit",
                });
                return Ok((Value::Unit, span));
            }
            trace.record(TraceEvent {
                event: "generator.yielded",
                rule: "TOPAL-GENERATOR-ITERATE-FOREACH-001",
                detail: &current.to_string(),
            });
            let mut iteration = session.clone();
            iteration
                .bindings
                .insert(binding_name.clone(), (*current).clone());
            iteration.declared_names.insert(binding_name.clone());
            let mut body_execution = Self {
                source: self.source.clone(),
                statements: body.to_vec(),
                cursor: 0,
                result_classifier: None,
                return_classifier: None,
            };
            loop {
                match body_execution.step(&mut iteration, trace)? {
                    ExecutionStep::Advanced { .. } => {}
                    ExecutionStep::Complete(Value::Unit) => break,
                    ExecutionStep::Complete(_) => {
                        return Err(diagnostic(
                            &self.source,
                            "E-FOREACH-ACTION-RESULT",
                            statement_span(body.last().expect("foreach body is nonempty")),
                            "foreach action must return Unit",
                        ));
                    }
                    ExecutionStep::Returned { .. } => {
                        unreachable!("foreach body has no function return context")
                    }
                }
            }
            let next_value =
                session.invoke_anonymous_function(&next, vec![*current], span, trace)?;
            if !value_has_classifier(&next_value, &classifier) {
                return Err(diagnostic(
                    &self.source,
                    "E-ITERATE-NEXT-CLASSIFIER",
                    span,
                    format!("iterate next function must return `{classifier}`"),
                ));
            }
            *current = next_value;
            trace.record(TraceEvent {
                event: "generator.resumed",
                rule: "TOPAL-GENERATOR-ITERATE-FOREACH-001",
                detail: "Unit",
            });
        }
    }

    #[allow(clippy::too_many_lines)] // State restoration and suspension order remain explicit and auditable.
    fn execute_suspended_foreach(
        &self,
        session: &Session,
        trace: &mut impl TraceSink,
        mut generator: Value,
        binding: Span,
        body: &[Statement],
        span: Span,
    ) -> Result<(Value, Span), Diagnostic> {
        let Value::SuspendedGenerator {
            source,
            body: generator_body,
            ref mut cursor,
            ref mut bindings,
            ref mut scope_state,
            ref mut pending_yield,
            ref mut resume_binding,
            ref mut returned,
            yield_classifier,
            return_classifier,
            origin,
            ref mut task_state,
            ref task_owner,
        } = generator
        else {
            unreachable!("caller selects a suspended generator")
        };
        let binding_name = self.source.slice(binding).to_owned();
        let mut yielded_any = pending_yield.is_some();
        loop {
            if let Some(yielded) = pending_yield.take() {
                yielded_any = true;
                let detail = yielded.to_string();
                trace.record(TraceEvent {
                    event: "generator.yielded",
                    rule: "TOPAL-GENERATOR-FOREACH-001",
                    detail: &detail,
                });
                let mut iteration = session.clone();
                iteration.bindings.insert(binding_name.clone(), *yielded);
                iteration.declared_names.insert(binding_name.clone());
                let mut action = Self {
                    source: self.source.clone(),
                    statements: body.to_vec(),
                    cursor: 0,
                    result_classifier: None,
                    return_classifier: None,
                };
                loop {
                    match action.step(&mut iteration, trace)? {
                        ExecutionStep::Advanced { .. } => {}
                        ExecutionStep::Complete(Value::Unit) => break,
                        ExecutionStep::Complete(_) => {
                            return Err(diagnostic(
                                &self.source,
                                "E-FOREACH-ACTION-RESULT",
                                statement_span(body.last().expect("foreach body is nonempty")),
                                "foreach action must return Unit",
                            ));
                        }
                        ExecutionStep::Returned { .. } => unreachable!("foreach cannot return"),
                    }
                }
                trace.record(TraceEvent {
                    event: "generator.resumed",
                    rule: "TOPAL-GENERATOR-FOREACH-001",
                    detail: "Unit",
                });
                let mut scope = session.clone();
                scope.task_state = task_state.take();
                scope.bindings = std::mem::take(bindings);
                scope.functions = Box::new(std::mem::take(&mut scope_state.functions));
                scope.declared_names = std::mem::take(&mut scope_state.declared_names);
                scope.local_function_names = std::mem::take(&mut scope_state.local_function_names);
                scope.enum_types = std::mem::take(&mut scope_state.enum_types);
                if let Some(name) = resume_binding.take() {
                    scope.bindings.insert(name.clone(), Value::Unit);
                    scope.declared_names.insert(name.clone());
                    trace.record(TraceEvent {
                        event: "generator.resume.bound",
                        rule: "TOPAL-GENERATOR-RESUME-BINDING-001",
                        detail: &name,
                    });
                }
                let mut next_returned = returned.take().map(|value| *value);
                advance_custom_generator(
                    &source,
                    &generator_body,
                    cursor,
                    &mut scope,
                    pending_yield,
                    resume_binding,
                    &mut next_returned,
                    &yield_classifier,
                    &return_classifier,
                    origin.rsplit('.').next().unwrap_or(&origin),
                    trace,
                )?;
                **bindings = scope.bindings;
                scope_state.functions = *scope.functions;
                scope_state.declared_names = scope.declared_names;
                scope_state.local_function_names = scope.local_function_names;
                scope_state.enum_types = scope.enum_types;
                *task_state = scope.task_state;
                sync_stream_task_state(session, task_owner.as_deref(), task_state.as_ref());
                *returned = next_returned.map(Box::new);
                continue;
            }
            let value = returned.take().map_or(Value::Unit, |value| *value);
            trace.record(TraceEvent {
                event: "generator.returned",
                rule: generator_return_rule(
                    &origin,
                    !yielded_any,
                    &return_classifier,
                    "TOPAL-GENERATOR-FOREACH-001",
                ),
                detail: &return_classifier,
            });
            sync_stream_task_state(session, task_owner.as_deref(), task_state.as_ref());
            return Ok((value, span));
        }
    }

    fn execute_discard(
        &self,
        session: &mut Session,
        trace: &mut impl TraceSink,
        span: Span,
        value: &Expression,
    ) -> Result<(Value, Span), Diagnostic> {
        session.evaluate_expression(&self.source, value, trace)?;
        trace.record(TraceEvent {
            event: "binding.discarded",
            rule: "TOPAL-SYN-BIND-001",
            detail: "_",
        });
        Ok((Value::Unit, cover(span, value.span())))
    }

    #[allow(clippy::too_many_lines)] // Declaration validation and trace setup stay auditable together.
    fn declare_task_implementation(
        &self,
        session: &mut Session,
        trace: &mut impl TraceSink,
        name: Span,
        classifier: &Expression,
        declarations: &[Statement],
        span: Span,
    ) -> Result<(Value, Span), Diagnostic> {
        let Value::TaskType(task_type) =
            session.evaluate_expression(&self.source, classifier, trace)?
        else {
            return Err(diagnostic(
                &self.source,
                "E-TASK-IMPLEMENTATION-TYPE",
                classifier.span(),
                "an indented implementation requires a specialized Task type",
            ));
        };
        let mut state_fields = Vec::new();
        let mut handler_session = session.clone();
        for declaration in declarations {
            match declaration {
                Statement::StateField {
                    name: field,
                    classifier,
                } => state_fields.push((
                    self.source.slice(*field).to_owned(),
                    self.source.slice(*classifier).to_owned(),
                )),
                Statement::Function {
                    name,
                    is_static,
                    parameters,
                    result,
                    effect_bound,
                    clauses,
                    body,
                    span,
                } => {
                    self.declare_function(
                        &mut handler_session,
                        trace,
                        FunctionDeclaration {
                            name: *name,
                            is_static: *is_static,
                            parameters,
                            result: *result,
                            effect_bound: *effect_bound,
                            clauses,
                            body,
                            span: *span,
                        },
                    )?;
                }
                Statement::Generator {
                    name,
                    parameters,
                    yielded,
                    resumed,
                    result,
                    body,
                    span,
                } => {
                    self.declare_generator(
                        &mut handler_session,
                        trace,
                        GeneratorDeclaration {
                            name: *name,
                            parameters,
                            yielded: *yielded,
                            resumed: *resumed,
                            result: *result,
                            body,
                            span: *span,
                        },
                    )?;
                }
                _ => {
                    return Err(diagnostic(
                        &self.source,
                        "E-TASK-IMPLEMENTATION-MEMBER",
                        statement_span(declaration),
                        "task implementations initially contain state fields and handlers",
                    ));
                }
            }
        }
        let handlers: BTreeMap<String, Vec<UserFunction>> = declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Statement::Function { name, .. } => {
                    let name = self.source.slice(*name).to_owned();
                    Some((
                        name.clone(),
                        handler_session.functions.get(&name).cloned().unwrap(),
                    ))
                }
                _ => None,
            })
            .collect();
        let streams: BTreeMap<String, Vec<UserGenerator>> = declarations
            .iter()
            .filter_map(|declaration| match declaration {
                Statement::Generator { name, .. } => {
                    let name = self.source.slice(*name).to_owned();
                    Some((
                        name.clone(),
                        handler_session.generators.get(&name).cloned().unwrap(),
                    ))
                }
                _ => None,
            })
            .collect();
        if !handlers.contains_key("start") {
            return Err(diagnostic(
                &self.source,
                "E-TASK-START-REQUIRED",
                name,
                "every task implementation requires a start handler",
            ));
        }
        for (handler_name, candidates) in &handlers {
            for handler in candidates {
                if matches!(handler_name.as_str(), "start" | "terminate") {
                    if result_success_classifier(&handler.result) == Some("Unit") {
                        return Err(diagnostic(
                            &self.source,
                            "E-TASK-START-RESULT",
                            name,
                            "start cannot return Result with Unit success",
                        ));
                    }
                    continue;
                }
                if handler.parameters.is_empty()
                    || handler.parameters.len() > 2
                    || handler.parameters[0].1 != "MessageContext"
                {
                    return Err(diagnostic(
                        &self.source,
                        "E-TASK-HANDLER-SHAPE",
                        name,
                        format!(
                            "message handler `{handler_name}` requires MessageContext plus zero or one ordinary operand"
                        ),
                    ));
                }
                if handler.result != "Unit"
                    && result_success_classifier(&handler.result)
                        .is_none_or(|success| success == "Unit")
                {
                    return Err(diagnostic(
                        &self.source,
                        "E-TASK-HANDLER-RESULT",
                        name,
                        format!(
                            "message handler `{handler_name}` must return Unit or Result with a non-Unit success value"
                        ),
                    ));
                }
            }
        }
        for (stream_name, candidates) in &streams {
            for stream in candidates {
                if stream.parameters.is_empty()
                    || stream.parameters.len() > 2
                    || stream.parameters[0].1 != "MessageContext"
                    || result_success_classifier(&stream.result).is_none()
                {
                    return Err(diagnostic(
                        &self.source,
                        "E-TASK-STREAM-SHAPE",
                        name,
                        format!(
                            "stream handler `{stream_name}` requires MessageContext, at most one payload, and a Result final return"
                        ),
                    ));
                }
            }
        }
        let task_type = TaskTypeValue {
            name: classifier_name(&self.source, classifier),
            ..*task_type
        };
        let value = Value::TaskDefinition(Box::new(TaskDefinitionValue {
            name: self.source.slice(name).to_owned(),
            task_type,
            source: self.source.clone(),
            state_fields,
            handlers,
            streams,
        }));
        session
            .bindings
            .insert(self.source.slice(name).to_owned(), value.clone());
        session
            .declared_names
            .insert(self.source.slice(name).to_owned());
        trace.record(TraceEvent {
            event: "task.definition.declared",
            rule: "TOPAL-TASK-DEFINITION-001",
            detail: self.source.slice(name),
        });
        Ok((value, span))
    }

    #[allow(clippy::too_many_lines)] // Declaration validation and trace setup stay auditable together.
    fn declare_function(
        &self,
        session: &mut Session,
        trace: &mut impl TraceSink,
        declaration: FunctionDeclaration<'_>,
    ) -> Result<(Value, Span), Diagnostic> {
        let FunctionDeclaration {
            name,
            is_static,
            parameters,
            result,
            effect_bound,
            clauses,
            body,
            span,
        } = declaration;
        let name_text = self.source.slice(name);
        if session.declared_names.contains(name_text)
            && !session.local_function_names.contains(name_text)
        {
            return Err(diagnostic(
                &self.source,
                "E-DUPLICATE-BINDING",
                name,
                "name is already bound in this scope",
            ));
        }
        let result_text = self.source.slice(result);
        if session.language_version == LanguageVersion::DESIGN_0
            && (clauses.requires.is_some()
                || clauses.effects.is_some()
                || clauses.guarantees.is_some()
                || clauses.result_binding.is_some()
                || clauses.ensures.is_some()
                || parameters
                    .iter()
                    .any(|parameter| parameter.qualifier.is_some()))
        {
            return Err(diagnostic(
                &self.source,
                "E-UNSUPPORTED-LANGUAGE-CONSTRUCT",
                span,
                "function contracts and parameter qualifiers require language version v0.2",
            ));
        }
        if session.language_version == LanguageVersion::DESIGN_1
            && let Some(effect_bound) = effect_bound
        {
            return Err(diagnostic(
                &self.source,
                "E-FUNCTION-CLAUSE-PLACEMENT",
                effect_bound,
                "v0.2 uses `effects` or `guarantees` before the function arrow",
            ));
        }
        if session.language_version == LanguageVersion::DESIGN_1
            && let Some(qualifier) = parameters.iter().find_map(|parameter| parameter.qualifier)
        {
            return Err(diagnostic(
                &self.source,
                "E-PARAMETER-EVIDENCE-UNAVAILABLE",
                qualifier,
                "this interpreter cannot prove invocation-local exclusivity or enforce caller consumption",
            ));
        }
        let effect_bound_text = clauses.effects.as_deref().map_or_else(
            || effect_bound.map(|bound| self.source.slice(bound).to_owned()),
            |bound| Some(self.source.slice(bound.span()).to_owned()),
        );
        let implementation_guarantee = clauses
            .guarantees
            .as_deref()
            .map(|guarantees| self.source.slice(guarantees.span()));
        if let Some(guarantee) = implementation_guarantee
            && !guarantee.trim_start().starts_with("Prefer")
        {
            return Err(diagnostic(
                &self.source,
                "E-IMPLEMENTATION-EVIDENCE-UNAVAILABLE",
                clauses
                    .guarantees
                    .as_deref()
                    .expect("known implementation guarantee")
                    .span(),
                "the interpreter cannot verify this hard implementation guarantee",
            ));
        }
        let modular_names = session
            .bindings
            .iter()
            .filter_map(|(name, value)| {
                matches!(value, Value::ModularType(_)).then_some(name.clone())
            })
            .collect::<BTreeSet<_>>();
        let mut generic_names = BTreeSet::new();
        for parameter in parameters {
            collect_generic_names(
                self.source.slice(parameter.classifier),
                &session.enum_types,
                &modular_names,
                &mut generic_names,
            );
            for field in &parameter.fields {
                collect_generic_names(
                    self.source.slice(field.classifier),
                    &session.enum_types,
                    &modular_names,
                    &mut generic_names,
                );
            }
        }
        if !supported_generic_classifier(
            result_text,
            &generic_names,
            &session.enum_types,
            &modular_names,
        ) && !session.union_types.contains_key(result_text)
        {
            return Err(diagnostic(
                &self.source,
                "E-UNSUPPORTED-RESULT-CLASSIFIER",
                result,
                "the result classifier is not supported by this interpreter subset",
            ));
        }
        validate_parameter_names(&self.source, parameters)?;
        let mut parameter_packages = BTreeMap::new();
        let parameters = parameters
            .iter()
            .enumerate()
            .map(|(index, parameter)| {
                if !parameter.fields.is_empty() {
                    let fields = parameter
                        .fields
                        .iter()
                        .map(|field| {
                            let classifier = self.source.slice(field.classifier);
                            if !supported_value_classifier(classifier, &session.enum_types)
                                && !supported_generic_classifier(
                                    classifier,
                                    &generic_names,
                                    &session.enum_types,
                                    &modular_names,
                                )
                                && !session.union_types.contains_key(classifier)
                            {
                                return Err(diagnostic(
                                    &self.source,
                                    "E-UNSUPPORTED-PARAMETER-CLASSIFIER",
                                    field.classifier,
                                    "the packaged parameter classifier is not supported by this interpreter subset",
                                ));
                            }
                            Ok(UserParameterField {
                                name: self.source.slice(field.name).to_owned(),
                                classifier: classifier.to_owned(),
                                default: field.default.clone(),
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let classifier = format!(
                        "({})",
                        fields
                            .iter()
                            .map(|field| format!("{} is {}", field.name, field.classifier))
                            .collect::<Vec<_>>()
                            .join(", ")
                    );
                    parameter_packages.insert(index, fields);
                    return Ok((format!("$package{index}"), classifier));
                }
                let classifier = self.source.slice(parameter.classifier);
                if !supported_value_classifier(classifier, &session.enum_types)
                    && generic_capability_classifier(classifier).is_none()
                    && !generic_names.contains(classifier)
                    && !supported_generic_classifier(
                        classifier,
                        &generic_names,
                        &session.enum_types,
                        &modular_names,
                    )
                    && !session.union_types.contains_key(classifier)
                {
                    return Err(diagnostic(
                        &self.source,
                        "E-UNSUPPORTED-PARAMETER-CLASSIFIER",
                        parameter.classifier,
                        "the parameter classifier is not supported by this interpreter subset",
                    ));
                }
                Ok((
                    self.source.slice(parameter.name).to_owned(),
                    classifier.to_owned(),
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if session.local_function_names.contains(name_text)
            && session.functions[name_text].iter().any(|function| {
                function.is_static == is_static
                    && function
                        .parameters
                        .iter()
                        .map(|(_, classifier)| classifier)
                        .eq(parameters.iter().map(|(_, classifier)| classifier))
            })
        {
            return Err(diagnostic(
                &self.source,
                "E-DUPLICATE-FUNCTION-OVERLOAD",
                name,
                "an overload with the same input classifiers and staticness already exists",
            ));
        }
        let direct_termination_rule = prove_euclidean_recursion(
            &self.source,
            name_text,
            &parameters,
            effect_bound_text.as_deref(),
            body,
        )
        .or_else(|| {
            prove_explicit_parameter_recursion(
                &self.source,
                name_text,
                &parameters,
                effect_bound_text.as_deref(),
                body,
            )
        })
        .or_else(|| prove_int_recursion(&self.source, name_text, &parameters, body));
        let mutual_edge = direct_termination_rule
            .is_none()
            .then(|| {
                prove_mutual_bounded_recursion_edge(&self.source, name_text, &parameters, body)
            })
            .flatten();
        let recursion_target = mutual_edge.as_ref().map(|(target, _)| target.clone());
        let termination_rule =
            direct_termination_rule.or_else(|| mutual_edge.as_ref().map(|(_, rule)| *rule));
        let rule = function_rule(is_static, parameters.len());
        let mut bindings = session
            .bindings
            .iter()
            .filter(|(captured_name, _)| body_mentions_name(&self.source, body, captured_name))
            .map(|(captured_name, value)| (captured_name.clone(), value.clone()))
            .collect::<BTreeMap<_, _>>();
        for (captured_name, candidates) in session
            .functions
            .iter()
            .filter(|(captured_name, _)| body_mentions_name(&self.source, body, captured_name))
        {
            bindings.entry(captured_name.clone()).or_insert_with(|| {
                Value::NamedFunction(Rc::new(NamedFunction {
                    name: captured_name.clone(),
                    candidates: candidates.clone(),
                }))
            });
        }
        let context_source = session
            .defining_context
            .as_ref()
            .unwrap_or(&session.bindings);
        let context_bindings = body_context_names(&self.source, body)
            .into_iter()
            .filter_map(|captured_name| {
                context_source
                    .get(&captured_name)
                    .cloned()
                    .map(|value| (captured_name, value))
            })
            .collect();
        let function = UserFunction {
            source: self.source.clone(),
            is_static,
            parameters,
            parameter_packages,
            result: result_text.to_owned(),
            generic_names,
            metadata: Box::new(UserFunctionMetadata {
                effect_bound: effect_bound_text.clone(),
                contracts: (clauses.requires.is_some()
                    || clauses.guarantees.is_some()
                    || clauses.result_binding.is_some()
                    || clauses.ensures.is_some())
                .then(|| UserFunctionContracts {
                    requires: clauses.requires.as_deref().cloned(),
                    guarantees: clauses.guarantees.as_deref().cloned(),
                    result_binding: clauses
                        .result_binding
                        .map(|binding| self.source.slice(binding).to_owned()),
                    ensures: clauses.ensures.as_deref().cloned(),
                }),
            }),
            body: Box::new(body.to_vec()),
            bindings,
            context_bindings,
            termination_rule,
            recursion_target: recursion_target.clone(),
        };
        if session.local_function_names.contains(name_text) {
            session.functions.get_mut(name_text).unwrap().push(function);
        } else {
            session
                .functions
                .insert(name_text.to_owned(), vec![function]);
        }
        session.bindings.remove(name_text);
        session.declared_names.insert(name_text.to_owned());
        session.local_function_names.insert(name_text.to_owned());
        trace.record(TraceEvent {
            event: "function.declared",
            rule,
            detail: name_text,
        });
        if let Some(effect_bound) = &effect_bound_text {
            trace.record(TraceEvent {
                event: "function.effect-bound.declared",
                rule: "TOPAL-FUNCTION-EFFECT-BOUND-001",
                detail: effect_bound,
            });
        }
        if let Some(requirement) = &clauses.requires {
            trace.record(TraceEvent {
                event: "function.precondition.declared",
                rule: "TOPAL-CONTRACT-REQUIRES-001",
                detail: self.source.slice(requirement.span()),
            });
        }
        if let Some(guarantees) = &clauses.guarantees {
            let guarantee = self.source.slice(guarantees.span());
            trace.record(TraceEvent {
                event: "function.guarantee.declared",
                rule: "TOPAL-IMPL-SELECTION-001",
                detail: guarantee,
            });
        }
        if let Some(relation) = &clauses.ensures {
            trace.record(TraceEvent {
                event: "function.postcondition.declared",
                rule: "TOPAL-CONTRACT-ENSURES-001",
                detail: self.source.slice(relation.span()),
            });
        }
        if result_success_classifier(result_text).is_some() {
            trace.record(TraceEvent {
                event: "function.result.contract",
                rule: "TOPAL-TYPE-RESULT-001",
                detail: result_text,
            });
        }
        if let Some(termination_rule) = direct_termination_rule {
            trace.record(TraceEvent {
                event: "function.recursion.proven",
                rule: termination_rule,
                detail: name_text,
            });
        } else if let Some(target) = recursion_target {
            let detail = format!("{name_text}->{target}");
            trace.record(TraceEvent {
                event: "function.recursion.edge.candidate",
                rule: termination_rule.expect("a mutual edge has a termination rule"),
                detail: &detail,
            });
        }
        Ok((Value::Unit, span))
    }

    fn declare_generator(
        &self,
        session: &mut Session,
        trace: &mut impl TraceSink,
        declaration: GeneratorDeclaration<'_>,
    ) -> Result<(Value, Span), Diagnostic> {
        let GeneratorDeclaration {
            name,
            parameters,
            yielded,
            resumed,
            result,
            body,
            span,
        } = declaration;
        if !session.call_stack.is_empty() {
            return Err(diagnostic(
                &self.source,
                "E-UNSUPPORTED-GENERATOR-SCOPE",
                name,
                "the implemented generator subset requires a root-namespace declaration",
            ));
        }
        let name_text = self.source.slice(name);
        if session.declared_names.contains(name_text) && !session.generators.contains_key(name_text)
        {
            return Err(diagnostic(
                &self.source,
                "E-DUPLICATE-BINDING",
                name,
                "name is already bound in this scope",
            ));
        }
        validate_parameter_names(&self.source, parameters)?;
        let parameters = parameters
            .iter()
            .map(|parameter| {
                (
                    self.source.slice(parameter.name).to_owned(),
                    self.source.slice(parameter.classifier).to_owned(),
                )
            })
            .collect::<Vec<_>>();
        let yield_classifier = self.source.slice(yielded);
        let result_classifier = self.source.slice(result);
        if !parameters.iter().all(|(_, classifier)| {
            supported_generator_value_classifier(classifier, &session.enum_types)
        }) || !supported_generator_value_classifier(yield_classifier, &session.enum_types)
            || self.source.slice(resumed) != "Unit"
            || !supported_generator_value_classifier(result_classifier, &session.enum_types)
        {
            return Err(diagnostic(
                &self.source,
                "E-UNSUPPORTED-GENERATOR-SIGNATURE",
                span,
                "the implemented generator subset requires supported scalar, Optional, Range, or declared enum input/yield/return classifiers and Unit resume",
            ));
        }
        if !supported_generator_body(&self.source, body) {
            return Err(diagnostic(
                &self.source,
                "E-UNSUPPORTED-GENERATOR-BODY",
                span,
                "the implemented generator subset requires bindings, discarded computations, or yield statements followed by a final expression",
            ));
        }
        let overloads = session.generators.entry(name_text.to_owned()).or_default();
        if overloads.iter().any(|candidate| {
            candidate.parameters.len() == parameters.len()
                && candidate
                    .parameters
                    .iter()
                    .zip(&parameters)
                    .all(|((_, left), (_, right))| left == right)
        }) {
            return Err(diagnostic(
                &self.source,
                "E-DUPLICATE-GENERATOR-OVERLOAD",
                name,
                format!("generator overload `{name_text}` has the same input classifiers"),
            ));
        }
        overloads.push(UserGenerator {
            source: self.source.clone(),
            parameters,
            yielded: yield_classifier.to_owned(),
            result: result_classifier.to_owned(),
            body: body.to_vec(),
            bindings: session.bindings.clone(),
        });
        session.declared_names.insert(name_text.to_owned());
        trace.record(TraceEvent {
            event: "generator.declared",
            rule: "TOPAL-GENERATOR-DECLARATION-001",
            detail: name_text,
        });
        let classifier = format!("Generator {yield_classifier} Unit {result_classifier}");
        trace.record(TraceEvent {
            event: "generator.classified",
            rule: "TOPAL-GENERATOR-DECLARATION-001",
            detail: &classifier,
        });
        Ok((Value::Unit, span))
    }

    /// Execute one source statement.
    ///
    /// # Errors
    ///
    /// Returns a name-resolution or evaluation diagnostic at the failing step.
    #[allow(clippy::too_many_lines)] // Statement dispatch remains explicit and exhaustively typed.
    pub fn step(
        &mut self,
        session: &mut Session,
        trace: &mut impl TraceSink,
    ) -> Result<ExecutionStep, Diagnostic> {
        let statement = &self.statements[self.cursor];
        if let Some(name) = declaration_name_span(statement) {
            reject_v02_language_object_shadowing(&self.source, session, name)?;
        }
        let (value, span) = match statement {
            Statement::LanguageSelection {
                version,
                features,
                span,
            } => {
                let requested = self
                    .source
                    .slice(*version)
                    .parse::<LanguageVersion>()
                    .map_err(|message| {
                        diagnostic(&self.source, "E-LANGUAGE-VERSION", *version, message)
                    })?;
                if !requested.is_supported_core() {
                    return Err(diagnostic(
                        &self.source,
                        "E-UNSUPPORTED-LANGUAGE-VERSION",
                        *version,
                        format!(
                            "language version `{requested}` is not supported; highest supported version is `{}`",
                            LanguageVersion::DESIGN_1
                        ),
                    ));
                }
                session.language_version = requested;
                session.language_features = features
                    .iter()
                    .map(|feature| self.source.slice(*feature).to_owned())
                    .collect();
                let feature_names = features
                    .iter()
                    .map(|feature| self.source.slice(*feature))
                    .collect::<Vec<_>>()
                    .join(",");
                let detail = if feature_names.is_empty() {
                    requested.to_string()
                } else {
                    format!("{requested};features={feature_names}")
                };
                trace.record(TraceEvent {
                    event: "language.context.selected",
                    rule: "TOPAL-SYN-GRAMMAR-001",
                    detail: &detail,
                });
                (Value::Unit, *span)
            }
            Statement::LibrarySelection {
                name,
                version,
                span,
            } => {
                let library_name = self.source.slice(*name);
                let requested = self.source.slice(*version);
                if !matches!(library_name, "std" | "advent-of-code") {
                    return Err(diagnostic(
                        &self.source,
                        "E-UNSUPPORTED-LIBRARY",
                        *name,
                        format!("library `{library_name}` is not available"),
                    ));
                }
                if requested != "v0.1" {
                    return Err(diagnostic(
                        &self.source,
                        "E-UNSUPPORTED-LIBRARY-VERSION",
                        *version,
                        format!(
                            "library `{library_name}` version `{requested}` is not supported; available version is `v0.1`"
                        ),
                    ));
                }
                session.declared_libraries.insert(library_name.to_owned());
                trace.record(TraceEvent {
                    event: "library.dependency.selected",
                    rule: "TOPAL-SYN-LIBRARY-001",
                    detail: if library_name == "std" {
                        "std@v0.1"
                    } else {
                        "advent-of-code@v0.1"
                    },
                });
                (Value::Unit, *span)
            }
            Statement::Published {
                declaration, span, ..
            } => {
                let published_name = declaration_name(&self.source, declaration);
                trace.record(TraceEvent {
                    event: "namespace.member.published",
                    rule: "TOPAL-NAMESPACE-ROOT-001",
                    detail: self.source.slice(*span),
                });
                let outcome = self.execute_published(session, trace, declaration)?;
                if let Some(name) = published_name {
                    session.published_names.insert(name.to_owned());
                }
                match outcome {
                    ExecutionStep::Complete(value) | ExecutionStep::Advanced { value, .. } => {
                        (value, *span)
                    }
                    ExecutionStep::Returned { value, span } => {
                        return Ok(ExecutionStep::Returned { value, span });
                    }
                }
            }
            Statement::DiagnosticControl { span, .. } => (Value::Unit, *span),
            Statement::Binding {
                name,
                classifier,
                value,
            } => match self.execute_binding(session, trace, *name, *classifier, value)? {
                BindingOutcome::Bound(value, span) => (value, span),
                BindingOutcome::Returned(value, span) => {
                    return Ok(ExecutionStep::Returned { value, span });
                }
            },
            Statement::Implementation {
                name,
                classifier,
                declarations,
                span,
            } => self.declare_task_implementation(
                session,
                trace,
                *name,
                classifier,
                declarations,
                *span,
            )?,
            Statement::StateField { name, .. } => {
                return Err(diagnostic(
                    &self.source,
                    "E-STATE-FIELD-CONTEXT",
                    *name,
                    "a task state field is valid only inside a task implementation",
                ));
            }
            Statement::ContextAssignment { name, value, span } => {
                let value = session.evaluate_expression(&self.source, value, trace)?;
                let Some(state) = session.task_state.as_mut() else {
                    return Err(diagnostic(
                        &self.source,
                        "E-TASK-STATE-CONTEXT",
                        *name,
                        "task state replacement requires an executing task handler",
                    ));
                };
                if !state.contains_key(self.source.slice(*name)) {
                    return Err(diagnostic(
                        &self.source,
                        "E-UNKNOWN-TASK-STATE",
                        *name,
                        "task implementation declares no such state field",
                    ));
                }
                state.insert(self.source.slice(*name).to_owned(), value);
                trace.record(TraceEvent {
                    event: "task.state.replaced",
                    rule: "TOPAL-TASK-STATE-001",
                    detail: self.source.slice(*name),
                });
                (Value::Unit, *span)
            }
            Statement::Function {
                name,
                is_static,
                parameters,
                result,
                effect_bound,
                clauses,
                body,
                span,
            } => self.declare_function(
                session,
                trace,
                FunctionDeclaration {
                    name: *name,
                    is_static: *is_static,
                    parameters,
                    result: *result,
                    effect_bound: *effect_bound,
                    clauses,
                    body,
                    span: *span,
                },
            )?,
            Statement::Generator {
                name,
                parameters,
                yielded,
                resumed,
                result,
                body,
                span,
            } => self.declare_generator(
                session,
                trace,
                GeneratorDeclaration {
                    name: *name,
                    parameters,
                    yielded: *yielded,
                    resumed: *resumed,
                    result: *result,
                    body,
                    span: *span,
                },
            )?,
            Statement::Union {
                name,
                alternatives,
                span,
            } => declare_union(&self.source, session, *name, alternatives, *span, trace)?,
            Statement::Interface {
                name,
                functions,
                span,
            } => {
                let name_text = self.source.slice(*name);
                if session.declared_names.contains(name_text) {
                    return Err(diagnostic(
                        &self.source,
                        "E-DUPLICATE-DECLARATION",
                        *name,
                        format!("`{name_text}` is already declared"),
                    ));
                }
                let mut operations = BTreeMap::new();
                for function in functions {
                    if session.language_version == LanguageVersion::DESIGN_0
                        && (function.clauses.requires.is_some()
                            || function.clauses.effects.is_some()
                            || function.clauses.guarantees.is_some()
                            || function.clauses.result_binding.is_some()
                            || function.clauses.ensures.is_some()
                            || function
                                .parameters
                                .iter()
                                .any(|parameter| parameter.qualifier.is_some()))
                    {
                        return Err(diagnostic(
                            &self.source,
                            "E-UNSUPPORTED-LANGUAGE-CONSTRUCT",
                            function.span,
                            "interface contracts and parameter qualifiers require language version v0.2",
                        ));
                    }
                    let operation = self.source.slice(function.name).to_owned();
                    if operations
                        .insert(
                            operation.clone(),
                            InterfaceFunctionShape {
                                parameters: function
                                    .parameters
                                    .iter()
                                    .map(|parameter| {
                                        (
                                            self.source.slice(parameter.classifier).to_owned(),
                                            parameter.qualifier.map(|qualifier| {
                                                self.source.slice(qualifier).to_owned()
                                            }),
                                        )
                                    })
                                    .collect(),
                                result: self.source.slice(function.result).to_owned(),
                                clauses: interface_clause_shape(&self.source, &function.clauses),
                            },
                        )
                        .is_some()
                    {
                        return Err(diagnostic(
                            &self.source,
                            "E-DUPLICATE-INTERFACE-OPERATION",
                            function.name,
                            format!("interface operation `{operation}` is declared twice"),
                        ));
                    }
                }
                let value = Value::Interface(Box::new(InterfaceValue {
                    name: name_text.to_owned(),
                    functions: operations,
                }));
                session.bindings.insert(name_text.to_owned(), value.clone());
                session.declared_names.insert(name_text.to_owned());
                trace.record(TraceEvent {
                    event: "interface.declared",
                    rule: "TOPAL-INTERFACE-SHAPE-001",
                    detail: name_text,
                });
                (value, *span)
            }
            Statement::InterfaceImplementation {
                interface,
                declarations,
                span,
            } => {
                let interface_name = self.source.slice(*interface);
                let Some(Value::Interface(shape)) = session.bindings.get(interface_name) else {
                    return Err(diagnostic(
                        &self.source,
                        "E-UNKNOWN-INTERFACE",
                        *interface,
                        format!("`{interface_name}` is not a declared interface"),
                    ));
                };
                let supplied = declarations
                    .iter()
                    .map(|declaration| match declaration {
                        Statement::Function {
                            name,
                            parameters,
                            result,
                            clauses,
                            ..
                        } => Some((
                            self.source.slice(*name).to_owned(),
                            InterfaceFunctionShape {
                                parameters: parameters
                                    .iter()
                                    .map(|parameter| {
                                        (
                                            self.source.slice(parameter.classifier).to_owned(),
                                            parameter.qualifier.map(|qualifier| {
                                                self.source.slice(qualifier).to_owned()
                                            }),
                                        )
                                    })
                                    .collect::<Vec<_>>(),
                                result: self.source.slice(*result).to_owned(),
                                clauses: interface_clause_shape(&self.source, clauses),
                            },
                        )),
                        _ => None,
                    })
                    .collect::<Option<BTreeMap<_, _>>>()
                    .ok_or_else(|| {
                        diagnostic(
                            &self.source,
                            "E-INTERFACE-IMPLEMENTATION",
                            *span,
                            "an interface implementation contains function declarations only",
                        )
                    })?;
                if supplied != shape.functions {
                    return Err(diagnostic(
                        &self.source,
                        "E-INTERFACE-IMPLEMENTATION",
                        *span,
                        "implementation operations must exactly match the interface shapes",
                    ));
                }
                for declaration in declarations {
                    let mut nested = Execution {
                        source: self.source.clone(),
                        statements: vec![declaration.clone()],
                        cursor: 0,
                        result_classifier: None,
                        return_classifier: None,
                    };
                    let _ = nested.step(session, trace)?;
                }
                trace.record(TraceEvent {
                    event: "interface.implemented",
                    rule: "TOPAL-INTERFACE-IMPLEMENTATION-001",
                    detail: interface_name,
                });
                (Value::Unit, *span)
            }
            Statement::Foreach {
                result,
                source,
                binding,
                body,
                span,
            } => {
                let (value, span) =
                    self.execute_foreach(session, trace, source, *binding, body, *span)?;
                if let Some((result, classifier)) = result {
                    let name = self.source.slice(*result);
                    if session.declared_names.contains(name) {
                        return Err(diagnostic(
                            &self.source,
                            "E-DUPLICATE-BINDING",
                            *result,
                            "name is already bound in this scope",
                        ));
                    }
                    if let Some(classifier) = classifier {
                        let expected = self.source.slice(*classifier);
                        if !value_has_classifier(&value, expected) {
                            let found = structural_value_classifier(&value);
                            return Err(diagnostic(
                                &self.source,
                                "E-FOREACH-RESULT-CLASSIFIER",
                                *classifier,
                                format!(
                                    "foreach returned `{found}`, but binding `{name}` requires `{expected}`"
                                ),
                            )
                            .with_help(format!(
                                "use classifier `{found}` here or traverse a generator returning `{expected}`"
                            )));
                        }
                    }
                    session.bindings.insert(name.to_owned(), value.clone());
                    session.declared_names.insert(name.to_owned());
                    trace.record(TraceEvent {
                        event: "generator.foreach.result.bound",
                        rule: "TOPAL-GENERATOR-FOREACH-RESULT-001",
                        detail: name,
                    });
                }
                (value, span)
            }
            Statement::Discard {
                span,
                value:
                    Expression::Block {
                        statements,
                        span: block_span,
                    },
            } => match session.evaluate_block_step(
                &self.source,
                statements,
                None,
                self.return_classifier.as_deref(),
                trace,
            )? {
                ExecutionStep::Complete(_) => {
                    trace.record(TraceEvent {
                        event: "binding.discarded",
                        rule: "TOPAL-SYN-BIND-001",
                        detail: "_",
                    });
                    (Value::Unit, cover(*span, *block_span))
                }
                ExecutionStep::Returned { value, span } => {
                    return Ok(ExecutionStep::Returned { value, span });
                }
                ExecutionStep::Advanced { .. } => unreachable!("a block runs to completion"),
            },
            Statement::Discard { span, value } => {
                if let Some(step) = session.evaluate_returning_embedded_expression_step(
                    &self.source,
                    value,
                    self.return_classifier.as_deref(),
                    trace,
                )? {
                    return Ok(step);
                }
                self.execute_discard(session, trace, *span, value)?
            }
            Statement::Return { keyword, value } => {
                if self.return_classifier.is_none() {
                    return Err(diagnostic(
                        &self.source,
                        "E-RETURN-OUTSIDE-FUNCTION",
                        *keyword,
                        "`return` is valid only inside a function body",
                    ));
                }
                let span = cover(*keyword, value.span());
                if let Some(step) = session.evaluate_returning_embedded_expression_step(
                    &self.source,
                    value,
                    self.return_classifier.as_deref(),
                    trace,
                )? {
                    return Ok(step);
                }
                let value = if let Expression::Block { statements, .. } = value {
                    match session.evaluate_block_step(
                        &self.source,
                        statements,
                        self.return_classifier.as_deref(),
                        self.return_classifier.as_deref(),
                        trace,
                    )? {
                        ExecutionStep::Complete(value) => value,
                        returned @ ExecutionStep::Returned { .. } => return Ok(returned),
                        ExecutionStep::Advanced { .. } => {
                            unreachable!("a block runs to completion")
                        }
                    }
                } else {
                    evaluate_expression_with_optional_context(
                        &self.source,
                        session,
                        value,
                        self.return_classifier.as_deref(),
                        trace,
                    )?
                };
                let classifier = structural_value_classifier(&value);
                trace.record(TraceEvent {
                    event: "function.return.explicit",
                    rule: "TOPAL-FUNCTION-RETURN-001",
                    detail: &classifier,
                });
                session.checkpoint(trace, Some(&value), Some(span));
                self.cursor = self.statements.len();
                return Ok(ExecutionStep::Returned { value, span });
            }
            Statement::Expression(expression) => {
                let final_expression = self.cursor + 1 == self.statements.len();
                let expected = final_expression
                    .then_some(self.result_classifier.as_deref())
                    .flatten();
                if let Some(step) = session.evaluate_returning_embedded_expression_step(
                    &self.source,
                    expression,
                    self.return_classifier.as_deref(),
                    trace,
                )? {
                    return Ok(step);
                }
                let value = if let Expression::Block { statements, .. } = expression {
                    match session.evaluate_block_step(
                        &self.source,
                        statements,
                        expected,
                        self.return_classifier.as_deref(),
                        trace,
                    )? {
                        ExecutionStep::Complete(value) => value,
                        ExecutionStep::Returned { value, span } => {
                            return Ok(ExecutionStep::Returned { value, span });
                        }
                        ExecutionStep::Advanced { .. } => {
                            unreachable!("a block runs to completion")
                        }
                    }
                } else {
                    evaluate_expression_with_optional_context(
                        &self.source,
                        session,
                        expression,
                        expected,
                        trace,
                    )?
                };
                if self.cursor + 1 != self.statements.len() && value != Value::Unit {
                    return Err(diagnostic(
                        &self.source,
                        "E-DISCARDED-VALUE",
                        expression.span(),
                        "a non-final expression value cannot be discarded",
                    ));
                }
                consume_generator_argument(&self.source, session, expression);
                (value, expression.span())
            }
        };
        self.cursor += 1;
        if self.cursor == self.statements.len() {
            record_result(trace, &value);
            session.checkpoint(trace, Some(&value), Some(span));
            Ok(ExecutionStep::Complete(value))
        } else {
            session.checkpoint(trace, Some(&value), Some(span));
            Ok(ExecutionStep::Advanced { value, span })
        }
    }

    #[inline(never)]
    fn execute_published(
        &self,
        session: &mut Session,
        trace: &mut impl TraceSink,
        declaration: &Statement,
    ) -> Result<ExecutionStep, Diagnostic> {
        let mut published = Self {
            source: self.source.clone(),
            statements: vec![declaration.clone()],
            cursor: 0,
            result_classifier: self.result_classifier.clone(),
            return_classifier: self.return_classifier.clone(),
        };
        published.step(session, trace)
    }

    #[allow(clippy::too_many_lines)] // Declaration specializations remain ordered before ordinary projection.
    fn execute_binding(
        &self,
        session: &mut Session,
        trace: &mut impl TraceSink,
        name: Span,
        classifier: Option<Span>,
        initializer: &Expression,
    ) -> Result<BindingOutcome, Diagnostic> {
        let name_text = self.source.slice(name);
        if session.declared_names.contains(name_text) {
            return Err(diagnostic(
                &self.source,
                "E-DUPLICATE-BINDING",
                name,
                "name is already bound in this scope",
            ));
        }
        if let Some((value, span)) = declare_enum(&self.source, name, initializer, session, trace)?
        {
            return Ok(BindingOutcome::Bound(value, span));
        }
        if let Some((value, span)) =
            declare_variant(&self.source, name, initializer, session, trace)
        {
            return Ok(BindingOutcome::Bound(value, span));
        }
        if let Some(step) = session.evaluate_returning_embedded_expression_step(
            &self.source,
            initializer,
            self.return_classifier.as_deref(),
            trace,
        )? {
            let ExecutionStep::Returned { value, span } = step else {
                unreachable!("a returning operator operand exits its function")
            };
            return Ok(BindingOutcome::Returned(value, span));
        }
        let mut evaluated = if let Expression::Block { statements, .. } = initializer {
            match session.evaluate_block_step(
                &self.source,
                statements,
                classifier.map(|classifier| self.source.slice(classifier)),
                self.return_classifier.as_deref(),
                trace,
            )? {
                ExecutionStep::Complete(value) => value,
                ExecutionStep::Returned { value, span } => {
                    return Ok(BindingOutcome::Returned(value, span));
                }
                ExecutionStep::Advanced { .. } => unreachable!("a block runs to completion"),
            }
        } else {
            evaluate_binding_initializer(&self.source, session, initializer, classifier, trace)?
        };
        consume_generator_argument(&self.source, session, initializer);
        if let Some(classifier) = classifier {
            let classifier_text =
                substitute_classifier(self.source.slice(classifier), &session.generic_types);
            evaluated = narrow_rational_to_int(
                &self.source,
                initializer,
                evaluated,
                &classifier_text,
                self.return_classifier.as_deref(),
                trace,
            )?;
            if matches!(evaluated, Value::Error { .. }) {
                let Some(return_classifier) = &self.return_classifier else {
                    return Err(diagnostic(
                        &self.source,
                        "E-RESULT-PROJECTION-OUTSIDE-FUNCTION",
                        initializer.span(),
                        "a failed Result cannot propagate from top-level execution",
                    ));
                };
                if result_success_classifier(return_classifier).is_none() {
                    return Err(diagnostic(
                        &self.source,
                        "E-RESULT-PROJECTION-INFALLIBLE",
                        initializer.span(),
                        format!(
                            "cannot propagate a failed Result from a function returning `{return_classifier}`"
                        ),
                    ));
                }
                trace.record(TraceEvent {
                    event: "result.error.projected",
                    rule: "TOPAL-TYPE-RESULT-PROJECT-001",
                    detail: name_text,
                });
                return Ok(BindingOutcome::Returned(evaluated, initializer.span()));
            }
            if !value_has_classifier(&evaluated, &classifier_text) {
                if classifier_text == "Character"
                    && let Value::String(text) = &evaluated
                {
                    let count = character_count(text);
                    return Err(diagnostic(
                        &self.source,
                        "E-CHARACTER-CLASSIFIER",
                        initializer.span(),
                        format!(
                            "Character requires exactly one user-perceived character, but this String contains {count}"
                        ),
                    ));
                }
                return Err(diagnostic(
                    &self.source,
                    "E-BINDING-CLASSIFIER",
                    initializer.span(),
                    format!(
                        "initializer has `{}`, which does not satisfy `{classifier_text}`",
                        structural_value_classifier(&evaluated)
                    ),
                ));
            }
            trace.record(TraceEvent {
                event: "result.success.projected",
                rule: "TOPAL-TYPE-RESULT-PROJECT-001",
                detail: name_text,
            });
        }
        if let Value::Constraint(constraint) = &mut evaluated {
            constraint.name = Some(name_text.to_owned());
        }
        if let Value::ModularType(kind) = &mut evaluated {
            kind.name = Some(name_text.to_owned());
        }
        session.bindings.insert(name_text.to_owned(), evaluated);
        session.functions.remove(name_text);
        session.declared_names.insert(name_text.to_owned());
        trace.record(TraceEvent {
            event: "binding.bind",
            rule: "TOPAL-SYN-BIND-001",
            detail: name_text,
        });
        Ok(BindingOutcome::Bound(
            Value::Unit,
            cover(name, initializer.span()),
        ))
    }
}
