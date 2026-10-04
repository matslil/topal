impl Session {
    #[allow(clippy::too_many_lines)] // Keep recursive expression cases together and auditable.
    fn evaluate_expression(
        &self,
        source: &SourceText,
        expression: &Expression,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let value = match expression {
            Expression::Block { statements, .. } => self.evaluate_block(source, statements, trace),
            Expression::Boolean(span) => Ok(evaluate_boolean_literal(source, *span, trace)),
            Expression::Measured { value, unit, span } => {
                let amount = parse_integer(source.slice(*value)).ok_or_else(|| {
                    diagnostic(
                        source,
                        "E-SIZE-LITERAL",
                        *span,
                        "size must use a Nat literal",
                    )
                })?;
                if amount < BigInt::from(0) || !matches!(source.slice(*unit), "b" | "B") {
                    return Err(diagnostic(
                        source,
                        "E-SIZE-LITERAL",
                        *span,
                        "size must be nonnegative and use `[b]` or `[B]`",
                    ));
                }
                let bits = if source.slice(*unit) == "B" {
                    amount * 8
                } else {
                    amount
                };
                trace.record(TraceEvent {
                    event: "layout.size.constructed",
                    rule: "TOPAL-LAYOUT-SIZE-001",
                    detail: source.slice(*span),
                });
                Ok(Value::SizeBits(bits))
            }
            Expression::Unit(_) => {
                trace.record(TraceEvent {
                    event: "product.unit",
                    rule: "TOPAL-TYPE-PRODUCT-001",
                    detail: "Tuple()",
                });
                Ok(Value::Unit)
            }
            Expression::Product { fields, span } => {
                self.evaluate_product(source, fields, *span, trace)
            }
            Expression::DecisionTable {
                subject,
                rules,
                span,
            } => {
                let subject_span = subject.span();
                let subject = self.evaluate_expression(source, subject, trace)?;
                let enum_matchers = rules
                    .iter()
                    .filter_map(|rule| match rule.matcher {
                        DecisionMatcher::Identifier(span) => Some(source.slice(span).to_owned()),
                        _ => None,
                    })
                    .collect::<BTreeSet<_>>();
                let has_result_matchers = rules.iter().any(|rule| {
                    matches!(
                        rule.matcher,
                        DecisionMatcher::Result { .. } | DecisionMatcher::ErrorCode { .. }
                    )
                });
                let has_optional_matchers = rules
                    .iter()
                    .any(|rule| matches!(rule.matcher, DecisionMatcher::Optional { .. }));
                let has_list_matchers = rules.iter().any(|rule| {
                    matches!(
                        rule.matcher,
                        DecisionMatcher::ListEmpty(_) | DecisionMatcher::ListEntry { .. }
                    )
                });
                let has_union_matchers = rules.iter().any(|rule| {
                    matches!(
                        rule.matcher,
                        DecisionMatcher::Union { .. } | DecisionMatcher::Variant { .. }
                    )
                });
                if !enum_matchers.is_empty()
                    && !has_union_matchers
                    && !rules
                        .iter()
                        .any(|rule| matches!(rule.matcher, DecisionMatcher::Otherwise(_)))
                {
                    let Value::Enum { type_name, .. } = &subject else {
                        return Err(diagnostic(
                            source,
                            "E-DECISION-SUBJECT-TYPE",
                            subject_span,
                            "enum alternative matchers require an Enum subject",
                        ));
                    };
                    if known_enum_alternatives(self, type_name).as_ref() != Some(&enum_matchers) {
                        return Err(diagnostic(
                            source,
                            "E-INCOMPLETE-DECISION",
                            *span,
                            format!("decision does not cover every `{type_name}` alternative"),
                        ));
                    }
                }
                let decision_rule = if has_union_matchers {
                    "TOPAL-DECISION-UNION-001"
                } else if has_list_matchers {
                    "TOPAL-DECISION-LIST-001"
                } else if has_optional_matchers {
                    "TOPAL-DECISION-OPTIONAL-001"
                } else if has_result_matchers {
                    "TOPAL-DECISION-RESULT-001"
                } else if !enum_matchers.is_empty() {
                    "TOPAL-DECISION-ENUM-001"
                } else if rules
                    .iter()
                    .any(|rule| matches!(&rule.matcher, DecisionMatcher::Comparison { .. }))
                {
                    "TOPAL-DECISION-COMPARISON-001"
                } else {
                    "TOPAL-DECISION-BOOLEAN-001"
                };
                let mut selected = None;
                for (index, rule) in rules.iter().enumerate() {
                    let matches = match &rule.matcher {
                        DecisionMatcher::Boolean { value, .. } => {
                            let Value::Boolean(subject) = &subject else {
                                return Err(diagnostic(
                                    source,
                                    "E-DECISION-SUBJECT-TYPE",
                                    subject_span,
                                    "Boolean literal matchers require a Boolean subject",
                                ));
                            };
                            *value == *subject
                        }
                        DecisionMatcher::Identifier(matcher) => {
                            let name = source.slice(*matcher);
                            if let Value::Enum {
                                type_name,
                                alternative,
                            } = &subject
                                && type_name == "Comparison"
                                && matches!(name, "Less" | "Equal" | "Greater")
                            {
                                alternative == name
                            } else {
                                let Some(candidate) = self.bindings.get(name).cloned() else {
                                    return Err(diagnostic(
                                        source,
                                        "E-UNBOUND-NAME",
                                        *matcher,
                                        format!("enum matcher `{name}` is not declared"),
                                    ));
                                };
                                values_equal(subject.clone(), candidate, trace).unwrap_or(false)
                            }
                        }
                        DecisionMatcher::Union { alternative, .. } => {
                            matches!(
                                &subject,
                                Value::Union(union)
                                    if union.payload.is_some()
                                        && union.alternative == source.slice(*alternative)
                            )
                        }
                        DecisionMatcher::Variant {
                            type_name, index, ..
                        } => {
                            let alternative = format!("at {}", source.slice(*index));
                            matches!(
                                &subject,
                                Value::Union(union)
                                    if union.payload.is_some()
                                        && union.type_name == source.slice(*type_name)
                                        && union.alternative == alternative
                            )
                        }
                        DecisionMatcher::Result { error, .. } => {
                            *error == matches!(subject, Value::Error { .. })
                        }
                        DecisionMatcher::Optional { some, .. } => {
                            let Value::Optional { payload, .. } = &subject else {
                                return Err(diagnostic(
                                    source,
                                    "E-DECISION-SUBJECT-TYPE",
                                    subject_span,
                                    "Optional matchers require an Optional subject",
                                ));
                            };
                            *some == payload.is_some()
                        }
                        DecisionMatcher::ListEmpty(_) => {
                            let Value::List { entries, .. } = &subject else {
                                return Err(diagnostic(
                                    source,
                                    "E-DECISION-SUBJECT-TYPE",
                                    subject_span,
                                    "list matchers require a List subject",
                                ));
                            };
                            entries.is_empty()
                        }
                        DecisionMatcher::ListEntry { .. } => {
                            let Value::List { entries, .. } = &subject else {
                                return Err(diagnostic(
                                    source,
                                    "E-DECISION-SUBJECT-TYPE",
                                    subject_span,
                                    "list matchers require a List subject",
                                ));
                            };
                            !entries.is_empty()
                        }
                        DecisionMatcher::ErrorCode {
                            namespace,
                            vocabulary,
                            code,
                            ..
                        } => {
                            let namespace = source.slice(*namespace);
                            let vocabulary = source.slice(*vocabulary);
                            let code_span = *code;
                            let code = source.slice(code_span);
                            let known = namespace == "lang"
                                && ((vocabulary == "arithmetic" && is_arithmetic_error_code(code))
                                    || (vocabulary == "generator" && code == "generator-closed"));
                            if !known {
                                return Err(diagnostic(
                                    source,
                                    "E-UNKNOWN-ERROR-CODE",
                                    code_span,
                                    "the error-code pattern requires a code published by the qualified language namespace",
                                ));
                            }
                            let matched = matches!(&subject, Value::Error { code: subject_code, .. } if subject_code == code);
                            if vocabulary == "generator" && matched {
                                trace.record(TraceEvent {
                                    event: "generator.error.code.matched",
                                    rule: "TOPAL-GENERATOR-CLOSE-CODE-PATTERN-001",
                                    detail: code,
                                });
                            }
                            matched
                        }
                        DecisionMatcher::Comparison {
                            kind,
                            operand,
                            span: matcher_span,
                        } => {
                            let right_span = operand.span();
                            let right = self.evaluate_expression(source, operand, trace)?;
                            matches!(
                                apply_binary(
                                    source,
                                    *kind,
                                    subject.clone(),
                                    right,
                                    (*matcher_span, subject_span, right_span),
                                    trace,
                                )?,
                                Value::Boolean(true)
                            )
                        }
                        DecisionMatcher::Otherwise(_) => true,
                    };
                    let detail = format!("rule={index};matched={matches}");
                    trace.record(TraceEvent {
                        event: "decision.rule.considered",
                        rule: decision_rule,
                        detail: &detail,
                    });
                    if matches {
                        selected = Some((index, rule));
                        break;
                    }
                }
                let Some((index, selected_rule)) = selected else {
                    return Err(diagnostic(
                        source,
                        "E-INCOMPLETE-DECISION",
                        *span,
                        "no decision rule matched the subject",
                    ));
                };
                let detail = format!("rule={index}");
                trace.record(TraceEvent {
                    event: "decision.rule.selected",
                    rule: decision_rule,
                    detail: &detail,
                });
                if let DecisionMatcher::ErrorCode { code, .. } = selected_rule.matcher {
                    trace.record(TraceEvent {
                        event: "error.code.matched",
                        rule: "TOPAL-DECISION-ERROR-CODE-001",
                        detail: source.slice(code),
                    });
                }
                if let DecisionMatcher::Result { binding, .. } = selected_rule.matcher {
                    let name = source.slice(binding);
                    let mut branch = Self {
                        bindings: self.bindings.clone(),
                        functions: self.functions.clone(),
                        generators: self.generators.clone(),
                        root_namespace: Some(self.effective_root_namespace()),
                        defining_context: self.defining_context.clone(),
                        declared_names: self.declared_names.clone(),
                        published_names: self.published_names.clone(),
                        documentation: self.documentation.clone(),
                        language_version: self.language_version,
                        language_features: self.language_features.clone(),
                        declared_libraries: self.declared_libraries.clone(),
                        consumed_names: self.consumed_names.clone(),
                        local_function_names: self.local_function_names.clone(),
                        enum_types: self.enum_types.clone(),
                        union_types: self.union_types.clone(),
                        generic_types: self.generic_types.clone(),
                        call_stack: self.call_stack.clone(),
                        static_context: self.static_context,
                        task_state: self.task_state.clone(),
                        next_task_identity: Cell::new(self.next_task_identity.get()),
                        next_transaction_identity: Cell::new(self.next_transaction_identity.get()),
                    };
                    branch.bindings.insert(name.to_owned(), subject);
                    trace.record(TraceEvent {
                        event: "result.payload.bound",
                        rule: "TOPAL-DECISION-RESULT-001",
                        detail: name,
                    });
                    branch.evaluate_expression(source, &selected_rule.action, trace)
                } else if let DecisionMatcher::Optional {
                    binding: Some(binding),
                    ..
                } = selected_rule.matcher
                {
                    let Value::Optional {
                        payload: Some(payload),
                        ..
                    } = subject
                    else {
                        unreachable!("Some matcher selected only for a present payload")
                    };
                    let name = source.slice(binding);
                    let mut branch = self.clone();
                    branch.bindings.insert(name.to_owned(), *payload);
                    trace.record(TraceEvent {
                        event: "optional.payload.bound",
                        rule: "TOPAL-DECISION-OPTIONAL-001",
                        detail: name,
                    });
                    branch.evaluate_expression(source, &selected_rule.action, trace)
                } else if let DecisionMatcher::ListEntry { first, rest, .. } = selected_rule.matcher
                {
                    let Value::List {
                        element_classifier,
                        mut entries,
                    } = subject
                    else {
                        unreachable!("Entry matcher selected only for a nonempty List")
                    };
                    let first_value = entries.remove(0);
                    let first = source.slice(first);
                    let rest = source.slice(rest);
                    let mut branch = self.clone();
                    branch.bindings.insert(first.to_owned(), first_value);
                    branch.bindings.insert(
                        rest.to_owned(),
                        Value::List {
                            element_classifier,
                            entries,
                        },
                    );
                    let detail = format!("first={first};rest={rest}");
                    trace.record(TraceEvent {
                        event: "list.entry.decomposed",
                        rule: "TOPAL-DECISION-LIST-001",
                        detail: &detail,
                    });
                    branch.evaluate_expression(source, &selected_rule.action, trace)
                } else if let DecisionMatcher::Union { binding, .. } = selected_rule.matcher {
                    self.evaluate_union_decision_action(
                        source,
                        subject,
                        binding,
                        &selected_rule.action,
                        trace,
                    )
                } else if let DecisionMatcher::Variant { binding, .. } = selected_rule.matcher {
                    self.evaluate_union_decision_action(
                        source,
                        subject,
                        binding,
                        &selected_rule.action,
                        trace,
                    )
                } else {
                    self.evaluate_expression(source, &selected_rule.action, trace)
                }
            }
            Expression::Integer(span) => evaluate_integer_literal(source, *span, trace),
            Expression::Infinity(span) => Err(diagnostic(
                source,
                "E-INFINITY-CONTEXT",
                *span,
                "an infinity constant requires an explicit supported numeric classifier",
            )),
            Expression::Rational(span) => evaluate_rational_literal(source, *span, trace),
            Expression::String(span) => evaluate_string_literal(source, *span, trace),
            Expression::Identifier(span) => self.resolve_identifier(source, *span, trace),
            Expression::ContextIdentifier(span) => {
                if self.call_stack.is_empty()
                    && self.task_state.is_none()
                    && self.defining_context.is_none()
                {
                    return Err(diagnostic(
                        source,
                        "E-CONTEXT-SELECTION",
                        *span,
                        "`@` selects the defining context from inside a function",
                    ));
                }
                let value = self
                    .task_state
                    .as_ref()
                    .and_then(|state| state.get(source.slice(*span)))
                    .or_else(|| {
                        self.defining_context
                            .as_ref()
                            .and_then(|context| context.get(source.slice(*span)))
                    })
                    .cloned()
                    .ok_or_else(|| {
                        diagnostic(
                            source,
                            "E-CONTEXT-SELECTION",
                            *span,
                            format!("defining context has no member `{}`", source.slice(*span)),
                        )
                    })?;
                trace.record(TraceEvent {
                    event: "context.member.selected",
                    rule: "TOPAL-CONTEXT-SELECT-001",
                    detail: source.slice(*span),
                });
                Ok(value)
            }
            Expression::Discard(span) => Err(diagnostic(
                source,
                "E-DISCARD-VALUE",
                *span,
                "discard is valid only in a declaration or pattern",
            )),
            Expression::AnonymousFunction {
                parameters,
                body,
                span: _,
            } => Ok(self.capture_anonymous_function(source, parameters, body, trace)),
            Expression::Callable { kind, .. } => {
                trace.record(TraceEvent {
                    event: "function.callable.captured",
                    rule: "TOPAL-FUNCTION-CALLABLE-VALUE-001",
                    detail: callable_name(*kind),
                });
                Ok(Value::Callable(*kind))
            }
            Expression::Application { items, span } => {
                if let Some(value) = self.evaluate_layout_application(source, items, *span, trace) {
                    return value;
                }
                if let [Expression::Identifier(empty), element] = items.as_slice()
                    && source.slice(*empty) == "Empty"
                {
                    let element = self.evaluate_expression(source, element, trace)?;
                    let Value::Type(element_classifier) = element else {
                        return Err(diagnostic(
                            source,
                            "E-LIST-EMPTY-CLASSIFIER",
                            items[1].span(),
                            "Empty requires an element type",
                        ));
                    };
                    return Ok(construct_empty_list(element_classifier, trace));
                }
                if let [Expression::Identifier(task), options] = items.as_slice()
                    && source.slice(*task) == "Task"
                {
                    let Expression::Product { fields, .. } = options else {
                        return Err(diagnostic(
                            source,
                            "E-TASK-OPTIONS",
                            options.span(),
                            "Task requires one labeled option record",
                        ));
                    };
                    let mut resolved_options = Vec::with_capacity(fields.len());
                    for field in fields {
                        let Some(label) = field.label else {
                            return Err(diagnostic(
                                source,
                                "E-TASK-OPTIONS",
                                field.value.span(),
                                "Task options must be labeled",
                            ));
                        };
                        let value = if source.slice(label) == "identity"
                            && let Expression::Identifier(identity) = field.value
                        {
                            Value::String(source.slice(identity).to_owned())
                        } else {
                            self.evaluate_expression(source, &field.value, trace)?
                        };
                        resolved_options.push((source.slice(label).to_owned(), value));
                    }
                    trace.record(TraceEvent {
                        event: "task.type.specialized",
                        rule: "TOPAL-TASK-DEFINITION-001",
                        detail: "Task",
                    });
                    return Ok(Value::TaskType(Box::new(TaskTypeValue {
                        name: None,
                        options: resolved_options,
                    })));
                }
                if self.is_native_serialization(source, items) {
                    return self.evaluate_native_serialization(source, items, *span, trace);
                }
                if let [Expression::Identifier(definition), argument] = items.as_slice()
                    && matches!(
                        self.bindings.get(source.slice(*definition)),
                        Some(Value::TaskDefinition(_))
                    )
                {
                    return self.construct_task_instance(
                        source,
                        *definition,
                        argument,
                        *span,
                        trace,
                    );
                }
                if matches!(items.first(), Some(Expression::Identifier(instance))
                    if matches!(self.bindings.get(source.slice(*instance)), Some(Value::TaskInstance(_))))
                {
                    return self.evaluate_task_message(source, items, *span, trace);
                }
                if Self::is_lang_introspection(source, items) {
                    return self.evaluate_lang_introspection(source, items, *span, trace);
                }
                if Self::is_empty_effects(source, items) {
                    trace.record(TraceEvent {
                        event: "effects.empty.constructed",
                        rule: "TOPAL-EFFECT-EMPTY-001",
                        detail: "Effects ()",
                    });
                    return Ok(Value::Effects(Vec::new()));
                }
                if Self::is_use_application(source, items) {
                    return self.evaluate_use_application(source, items, *span, trace);
                }
                if self.is_bound_namespace_application(source, items) {
                    return self.evaluate_bound_namespace_application(source, items, *span, trace);
                }
                if Self::is_root_qualified_application(source, items) {
                    return self.evaluate_root_qualified_application(source, items, *span, trace);
                }
                if Self::is_unfold_construction(source, items) {
                    return self.construct_unfold_generator(source, items, *span, trace);
                }
                if Self::is_iterate_take_while_construction(source, items) {
                    return self.construct_iterate_take_while(source, items, *span, trace);
                }
                if Self::is_iterate_construction(source, items) {
                    return self.construct_iterate_generator(source, items, *span, trace);
                }
                if Self::is_generator_take_while_application(source, items) {
                    return self.apply_generator_take_while(source, items, *span, trace);
                }
                if self.is_bound_named_function_call(source, items) {
                    return self
                        .evaluate_bound_named_function_call(source, expression, items, trace);
                }
                if self.is_bound_callable_call(source, items) {
                    return self.evaluate_bound_callable_call(source, items, *span, trace);
                }
                if Self::is_traversal_control_constructor(source, items) {
                    return self.construct_traversal_control(source, items, *span, trace);
                }
                if self.is_bound_anonymous_call(source, items) {
                    return self.evaluate_bound_anonymous_call(source, items, *span, trace);
                }
                if Self::is_record_reconstruction(source, items) {
                    return self.evaluate_record_reconstruction(source, items, *span, trace);
                }
                if self.is_bound_list_higher_order_application(source, items) {
                    return self.evaluate_list_higher_order(source, items, *span, trace);
                }
                if Self::is_range_selection(source, items) {
                    return self.evaluate_range_selection(source, items, *span, trace);
                }
                if self.is_explicit_modulo(source, items) {
                    return self.apply_explicit_modulo(source, items, trace);
                }
                if Self::is_modular_type_definition(source, items) {
                    return self.construct_modular_type(source, items, trace);
                }
                if self.is_modular_construction(source, items) {
                    return self.construct_modular_value(source, items, *span, trace);
                }
                if Self::is_constraint_definition(source, items) {
                    return self.construct_constraint(source, items, trace);
                }
                if self.is_constraint_application(source, items) {
                    return self.apply_constraint(source, items, *span, trace);
                }
                if self.application_is_union_constructor(source, items) {
                    return self.construct_union_application(source, items, *span, trace);
                }
                if matches!(
                    items.as_slice(),
                    [Expression::Identifier(operation), ..]
                        if matches!(source.slice(*operation), "unzip" | "collect" | "collect-set" | "collect-bag" | "collect-map")
                ) || matches!(
                    items.as_slice(),
                    [_, Expression::Identifier(operation), ..]
                        if matches!(source.slice(*operation), "zip-longest" | "collect")
                ) {
                    return self.evaluate_list_materialization(source, items, *span, trace);
                }
                if matches!(
                    items.as_slice(),
                    [_, Expression::Identifier(operation), Expression::AnonymousFunction { .. }]
                        if matches!(source.slice(*operation), "map" | "select" | "remove-indexes" | "remove-values")
                ) || matches!(
                    items.as_slice(),
                    [_, Expression::Identifier(operation), _, Expression::AnonymousFunction { .. }]
                        if source.slice(*operation) == "fold"
                ) {
                    return self.evaluate_list_higher_order(source, items, *span, trace);
                }
                if Self::is_characters_application(source, items) {
                    return self.evaluate_characters_application(source, items, *span, trace);
                }
                if let [left, Expression::Identifier(callable), right] = items.as_slice()
                    && source.slice(*callable) == "canonically-equals"
                {
                    let left_span = left.span();
                    let right_span = right.span();
                    let left = self.evaluate_expression(source, left, trace)?;
                    let right = self.evaluate_expression(source, right, trace)?;
                    let (Value::String(left), Value::String(right)) = (left, right) else {
                        return Err(diagnostic(
                            source,
                            "E-CANONICAL-EQUALITY-OPERANDS",
                            cover(left_span, right_span),
                            "canonically-equals requires two String operands",
                        ));
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.canonically-equals(String,String)",
                    });
                    let value = Value::Boolean(canonically_equal(&left, &right));
                    trace.record(TraceEvent {
                        event: "string.canonical-equality.compared",
                        rule: "TOPAL-STRING-CANONICAL-EQUALITY-001",
                        detail: if matches!(value, Value::Boolean(true)) {
                            "equal"
                        } else {
                            "unequal"
                        },
                    });
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if let Some(value) = evaluate_arithmetic_error_code(source, items, trace) {
                    return Ok(value);
                }
                if let Some(value) = evaluate_generator_error_code(source, items, trace) {
                    return Ok(value);
                }
                if let [Expression::Identifier(constructor), payload] = items.as_slice()
                    && source.slice(*constructor) == "Some"
                {
                    let value = self.evaluate_expression(source, payload, trace)?;
                    let payload_classifier = structural_value_classifier(&value);
                    trace.record(TraceEvent {
                        event: "optional.some.constructed",
                        rule: "TOPAL-TYPE-OPTIONAL-CONSTRUCT-001",
                        detail: &payload_classifier,
                    });
                    return Ok(Value::Optional {
                        payload_classifier,
                        payload: Some(Box::new(value)),
                    });
                }
                if let [Expression::Identifier(constructor), domain] = items.as_slice()
                    && source.slice(*constructor) == "None"
                    && let Some(mut payload_classifier) = classifier_expression(source, domain)
                {
                    payload_classifier =
                        substitute_classifier(&payload_classifier, &self.generic_types);
                    trace.record(TraceEvent {
                        event: "optional.none.constructed",
                        rule: "TOPAL-TYPE-OPTIONAL-CONSTRUCT-001",
                        detail: &payload_classifier,
                    });
                    return Ok(Value::Optional {
                        payload_classifier,
                        payload: None,
                    });
                }
                if let [Expression::Identifier(constructor), character] = items.as_slice()
                    && source.slice(*constructor) == "String"
                {
                    let value = self.evaluate_expression(source, character, trace)?;
                    let Value::String(text) = value else {
                        return Err(diagnostic(
                            source,
                            "E-STRING-CONSTRUCTOR-CHARACTER",
                            character.span(),
                            "String construction requires a Character value",
                        ));
                    };
                    let count = character_count(&text);
                    if count != 1 {
                        return Err(diagnostic(
                            source,
                            "E-STRING-CONSTRUCTOR-CHARACTER",
                            character.span(),
                            format!(
                                "String construction requires one Character, but the operand contains {count}"
                            ),
                        ));
                    }
                    trace.record(TraceEvent {
                        event: "string.from-character",
                        rule: "TOPAL-STRING-FROM-CHARACTER-001",
                        detail: "preserved",
                    });
                    return Ok(Value::String(text));
                }
                if let [Expression::Identifier(constructor), operand] = items.as_slice()
                    && source.slice(*constructor) == "Int"
                {
                    let value = self.evaluate_expression(source, operand, trace)?;
                    return construct_int(source, operand, value, trace);
                }
                if let [Expression::Identifier(constructor), operand] = items.as_slice()
                    && source.slice(*constructor) == "Nat"
                {
                    let value = self.evaluate_expression(source, operand, trace)?;
                    return construct_nat(source, operand, value, trace);
                }
                if let [Expression::Identifier(constructor), operand] = items.as_slice()
                    && source.slice(*constructor) == "Rational"
                {
                    let value = self.evaluate_expression(source, operand, trace)?;
                    return construct_rational(source, operand, value, trace);
                }
                if let [Expression::Identifier(callable), operand] = items.as_slice()
                    && source.slice(*callable) == "absolute"
                {
                    let operand_span = operand.span();
                    let value = self.evaluate_expression(source, operand, trace)?;
                    let (value, selection, classifier) = match value {
                        Value::Infinity { classifier, .. } => {
                            let selection = if classifier == "Rational" {
                                "root.absolute(Rational)"
                            } else {
                                "root.absolute(Int)"
                            };
                            let result_classifier = if classifier == "Nat" {
                                "Nat".to_owned()
                            } else {
                                classifier
                            };
                            (
                                Value::Infinity {
                                    negative: false,
                                    classifier: result_classifier.clone(),
                                },
                                selection,
                                if result_classifier == "Rational" {
                                    "Rational"
                                } else {
                                    "Int"
                                },
                            )
                        }
                        Value::Int(value) => (
                            Value::Int(if value < BigInt::from(0) {
                                -value
                            } else {
                                value
                            }),
                            "root.absolute(Int)",
                            "Int",
                        ),
                        Value::Rational(value) => (
                            Value::Rational(
                                if value < BigRational::from_integer(BigInt::from(0)) {
                                    -value
                                } else {
                                    value
                                },
                            ),
                            "root.absolute(Rational)",
                            "Rational",
                        ),
                        _ => {
                            return Err(diagnostic(
                                source,
                                "E-NO-APPLICABLE-OVERLOAD",
                                operand_span,
                                "absolute requires an exact numeric operand",
                            ));
                        }
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: selection,
                    });
                    trace.record(TraceEvent {
                        event: "evaluation.absolute",
                        rule: if matches!(&value, Value::Infinity { .. }) {
                            "TOPAL-NUM-INFINITY-ARITHMETIC-001"
                        } else {
                            "TOPAL-NUM-ABS-001"
                        },
                        detail: classifier,
                    });
                    return Ok(value);
                }
                if let [
                    Expression::Identifier(callable),
                    Expression::Identifier(domain),
                ] = items.as_slice()
                    && source.slice(*callable) == "zero"
                {
                    let (value, selection, classifier) = match source.slice(*domain) {
                        "Int" => (Value::Int(BigInt::from(0)), "root.zero(Int)", "Int"),
                        "Nat" => (Value::Int(BigInt::from(0)), "root.zero(Nat)", "Nat"),
                        "Rational" => (
                            Value::Rational(BigRational::from_integer(BigInt::from(0))),
                            "root.zero(Rational)",
                            "Rational",
                        ),
                        _ => {
                            return Err(diagnostic(
                                source,
                                "E-NO-APPLICABLE-OVERLOAD",
                                *domain,
                                "zero requires a supported numeric type",
                            ));
                        }
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: selection,
                    });
                    trace.record(TraceEvent {
                        event: "numeric.zero.constructed",
                        rule: "TOPAL-NUM-ZERO-001",
                        detail: classifier,
                    });
                    return Ok(value);
                }
                if let [
                    Expression::Identifier(callable),
                    Expression::Identifier(domain),
                ] = items.as_slice()
                    && source.slice(*callable) == "one"
                    && matches!(source.slice(*domain), "Int" | "Nat" | "Rational")
                {
                    let (value, selection, classifier) = match source.slice(*domain) {
                        "Int" => (Value::Int(BigInt::from(1)), "root.one(Int)", "Int"),
                        "Nat" => (Value::Int(BigInt::from(1)), "root.one(Nat)", "Nat"),
                        "Rational" => (
                            Value::Rational(BigRational::from_integer(BigInt::from(1))),
                            "root.one(Rational)",
                            "Rational",
                        ),
                        _ => unreachable!("guarded numeric one domain"),
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: selection,
                    });
                    trace.record(TraceEvent {
                        event: "numeric.one.constructed",
                        rule: "TOPAL-NUM-ONE-001",
                        detail: classifier,
                    });
                    return Ok(value);
                }
                if is_singleton_list_construction(source, items) {
                    return evaluate_singleton_list(source, self, items, trace);
                }
                if is_explicit_empty_list_construction(source, items) {
                    return evaluate_empty_list(source, items, trace);
                }
                if let [Expression::Identifier(callable), operand] = items.as_slice()
                    && source.slice(*callable) == "negate"
                {
                    let operand_span = operand.span();
                    let value = self.evaluate_expression(source, operand, trace)?;
                    let (value, selection, classifier, rule) = match value {
                        Value::Infinity {
                            negative,
                            classifier,
                        } => {
                            let rational = classifier == "Rational";
                            (
                                Value::Infinity {
                                    negative: !negative,
                                    classifier: if classifier == "Nat" {
                                        "Int".to_owned()
                                    } else {
                                        classifier
                                    },
                                },
                                if rational {
                                    "root.negate(Rational)"
                                } else {
                                    "root.negate(Int)"
                                },
                                if rational { "Rational" } else { "Int" },
                                "TOPAL-NUM-INFINITY-ARITHMETIC-001",
                            )
                        }
                        Value::Int(value) => (
                            Value::Int(-value),
                            "root.negate(Int)",
                            "Int",
                            "TOPAL-NUM-NEG-001",
                        ),
                        Value::Rational(value) => (
                            Value::Rational(-value),
                            "root.negate(Rational)",
                            "Rational",
                            "TOPAL-NUM-RAT-NEG-001",
                        ),
                        _ => {
                            return Err(diagnostic(
                                source,
                                "E-NO-APPLICABLE-OVERLOAD",
                                operand_span,
                                "negate requires an exact numeric operand",
                            ));
                        }
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: selection,
                    });
                    trace.record(TraceEvent {
                        event: "evaluation.negate",
                        rule,
                        detail: classifier,
                    });
                    return Ok(value);
                }
                if let [Expression::Identifier(callable), operand] = items.as_slice()
                    && source.slice(*callable) == "not"
                {
                    let value = self.evaluate_expression(source, operand, trace)?;
                    let Value::Boolean(value) = value else {
                        return Err(diagnostic(
                            source,
                            "E-BOOLEAN-NOT-OPERAND",
                            operand.span(),
                            "not requires a Boolean operand",
                        ));
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.not(Boolean)",
                    });
                    trace.record(TraceEvent {
                        event: "evaluation.logical",
                        rule: "TOPAL-TYPE-BOOLEAN-LOGIC-001",
                        detail: "not",
                    });
                    return Ok(Value::Boolean(!value));
                }
                if let [left, Expression::Identifier(callable), right] = items.as_slice()
                    && matches!(source.slice(*callable), "in" | "contains")
                {
                    let left = self.evaluate_expression(source, left, trace)?;
                    let right = self.evaluate_expression(source, right, trace)?;
                    return apply_range_membership(
                        source,
                        source.slice(*callable),
                        left,
                        right,
                        *span,
                        trace,
                    );
                }
                if let [text, Expression::Identifier(callable), index] = items.as_slice()
                    && source.slice(*callable) == "character-at"
                {
                    let text_span = text.span();
                    let index_span = index.span();
                    let text = self.evaluate_expression(source, text, trace)?;
                    let index = self.evaluate_expression(source, index, trace)?;
                    let (Value::String(text), Value::Int(index)) = (text, index) else {
                        return Err(diagnostic(
                            source,
                            "E-CHARACTER-AT-OPERANDS",
                            cover(text_span, index_span),
                            "character-at requires a String and an Int index",
                        ));
                    };
                    let payload = usize::try_from(index)
                        .ok()
                        .and_then(|index| character_at(&text, index))
                        .map(|character| Box::new(Value::String(character.to_owned())));
                    trace.record(TraceEvent {
                        event: "string.character-at",
                        rule: "TOPAL-STRING-CHARACTER-AT-001",
                        detail: if payload.is_some() { "Some" } else { "None" },
                    });
                    return Ok(Value::Optional {
                        payload_classifier: "Character".to_owned(),
                        payload,
                    });
                }
                if let [left, Expression::Identifier(callable), right] = items.as_slice()
                    && source.slice(*callable) == "and"
                {
                    let left = self.evaluate_expression(source, left, trace)?;
                    let right = self.evaluate_expression(source, right, trace)?;
                    return apply_and(source, left, right, *span, trace);
                }
                if let [left, Expression::Identifier(callable), right] = items.as_slice()
                    && source.slice(*callable) == "or"
                {
                    let left = self.evaluate_expression(source, left, trace)?;
                    let right = self.evaluate_expression(source, right, trace)?;
                    if let (Value::Capability(mut left), Value::Capability(right)) =
                        (left.clone(), right.clone())
                    {
                        left.extend(right);
                        left.sort();
                        left.dedup();
                        trace.record(TraceEvent {
                            event: "capability.composed",
                            rule: "TOPAL-CAPABILITY-EVIDENCE-001",
                            detail: "or",
                        });
                        return Ok(Value::Capability(left));
                    }
                    let (Value::Boolean(left), Value::Boolean(right)) = (left, right) else {
                        return Err(diagnostic(
                            source,
                            "E-BOOLEAN-OR-OPERANDS",
                            *span,
                            "or requires two Boolean operands; range union is a Predicate",
                        ));
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.or(Boolean,Boolean)",
                    });
                    trace.record(TraceEvent {
                        event: "evaluation.logical",
                        rule: "TOPAL-TYPE-BOOLEAN-LOGIC-001",
                        detail: "or:eager",
                    });
                    return Ok(Value::Boolean(left || right));
                }
                if let [left, Expression::Identifier(callable), right] = items.as_slice()
                    && source.slice(*callable) == "xor"
                {
                    let left = self.evaluate_expression(source, left, trace)?;
                    let right = self.evaluate_expression(source, right, trace)?;
                    let (Value::Boolean(left), Value::Boolean(right)) = (left, right) else {
                        return Err(diagnostic(
                            source,
                            "E-BOOLEAN-XOR-OPERANDS",
                            *span,
                            "xor requires two Boolean operands",
                        ));
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.xor(Boolean,Boolean)",
                    });
                    trace.record(TraceEvent {
                        event: "evaluation.logical",
                        rule: "TOPAL-TYPE-BOOLEAN-LOGIC-001",
                        detail: "xor:eager",
                    });
                    return Ok(Value::Boolean(left != right));
                }
                if items.len() == 3
                    && let Expression::Identifier(name_span) = items[1]
                    && (!self.bindings.contains_key(source.slice(name_span))
                        || matches!(
                            self.bindings.get(source.slice(name_span)),
                            Some(Value::NamedFunction(function))
                                if function.name == source.slice(name_span)
                        ))
                    && self.functions.contains_key(source.slice(name_span))
                {
                    let argument_span = cover(items[0].span(), items[2].span());
                    let call = Expression::Application {
                        items: vec![
                            Expression::Identifier(name_span),
                            Expression::Product {
                                fields: vec![
                                    topal_syntax::ProductField {
                                        label: None,
                                        value: items[0].clone(),
                                    },
                                    topal_syntax::ProductField {
                                        label: None,
                                        value: items[2].clone(),
                                    },
                                ],
                                span: argument_span,
                            },
                        ],
                        span: *span,
                    };
                    return self.evaluate_expression(source, &call, trace);
                }
                if items.len() == 2
                    && let Expression::Identifier(name_span) = &items[0]
                    && !self.bindings.contains_key(source.slice(*name_span))
                    && let Some(candidates) = self.generators.get(source.slice(*name_span)).cloned()
                {
                    let name = source.slice(*name_span);
                    let argument_span = items[1].span();
                    let argument = self.evaluate_expression(source, &items[1], trace)?;
                    let Some(generator) = candidates
                        .iter()
                        .find(|candidate| function_accepts(&candidate.parameters, &argument))
                        .cloned()
                    else {
                        return Err(no_applicable_generator(
                            source,
                            name,
                            argument_span,
                            &argument,
                            &candidates,
                        ));
                    };
                    let mut generator_scope = Self {
                        bindings: generator.bindings,
                        functions: self.functions.clone(),
                        generators: self.generators.clone(),
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
                        call_stack: self.call_stack.clone(),
                        static_context: false,
                        task_state: None,
                        next_task_identity: Cell::new(self.next_task_identity.get()),
                        next_transaction_identity: Cell::new(self.next_transaction_identity.get()),
                    };
                    bind_generator_arguments(
                        &mut generator_scope,
                        &generator.parameters,
                        argument,
                        trace,
                    );
                    let signature = generator
                        .parameters
                        .iter()
                        .map(|(_, classifier)| classifier.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    trace.record(TraceEvent {
                        event: "generator.selected",
                        rule: "TOPAL-GENERATOR-OVERLOAD-001",
                        detail: &signature,
                    });
                    trace.record(TraceEvent {
                        event: "generator.started",
                        rule: "TOPAL-GENERATOR-DECLARATION-001",
                        detail: name,
                    });
                    let mut cursor = 0;
                    let mut pending_yield = None;
                    let mut resume_binding = None;
                    let mut returned = None;
                    advance_custom_generator(
                        &generator.source,
                        &generator.body,
                        &mut cursor,
                        &mut generator_scope,
                        &mut pending_yield,
                        &mut resume_binding,
                        &mut returned,
                        &generator.yielded,
                        &generator.result,
                        name,
                        trace,
                    )?;
                    let origin = format!("root.{name}");
                    let value = Value::SuspendedGenerator {
                        source: Box::new(generator.source),
                        body: Box::new(generator.body),
                        cursor,
                        bindings: Box::new(generator_scope.bindings),
                        scope_state: Box::new(GeneratorScopeState {
                            functions: *generator_scope.functions,
                            declared_names: generator_scope.declared_names,
                            local_function_names: generator_scope.local_function_names,
                            enum_types: generator_scope.enum_types,
                            union_types: generator_scope.union_types,
                        }),
                        pending_yield,
                        resume_binding,
                        returned: returned.map(Box::new),
                        yield_classifier: generator.yielded,
                        return_classifier: generator.result,
                        origin,
                        task_state: None,
                        task_owner: None,
                    };
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && let Expression::Identifier(name) = &items[0]
                    && source.slice(*name) == "ascii-decimal-text?"
                {
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let Value::String(text) = operand else {
                        return Err(diagnostic(
                            source,
                            "E-ASCII-DIGIT-OPERAND",
                            operand_span,
                            "ASCII decimal text classification requires String",
                        ));
                    };
                    let value = Value::Boolean(text.bytes().all(|byte| byte.is_ascii_digit()));
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && let Expression::Identifier(name) = &items[0]
                    && source.slice(*name) == "ascii-decimal-digit"
                {
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let Value::String(character) = operand else {
                        return Err(diagnostic(
                            source,
                            "E-ASCII-DIGIT-OPERAND",
                            operand_span,
                            "ASCII decimal digit classification requires Character",
                        ));
                    };
                    let payload = character
                        .as_bytes()
                        .first()
                        .copied()
                        .filter(|_| character.len() == 1)
                        .filter(u8::is_ascii_digit)
                        .map(|digit| Box::new(Value::Int(BigInt::from(digit - b'0'))));
                    let value = Value::Optional {
                        payload_classifier: "Nat".into(),
                        payload,
                    };
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && let Expression::Identifier(name) = &items[0]
                    && source.slice(*name) == "unicode-scalar-characters"
                {
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let Value::String(text) = operand else {
                        return Err(diagnostic(
                            source,
                            "E-UNICODE-SCALAR-OPERAND",
                            operand_span,
                            "Unicode scalar decomposition requires String",
                        ));
                    };
                    let value = Value::List {
                        element_classifier: "Character".into(),
                        entries: scalar_characters(&text)
                            .into_iter()
                            .map(Value::String)
                            .collect(),
                    };
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && let Expression::Identifier(name) = &items[0]
                    && source.slice(*name) == "unicode-scalar-value"
                {
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let Value::String(character) = operand else {
                        return Err(diagnostic(
                            source,
                            "E-UNICODE-SCALAR-OPERAND",
                            operand_span,
                            "Unicode scalar value requires Character",
                        ));
                    };
                    let mut scalars = character.chars();
                    let first = scalars.next();
                    let Some(first) = first.filter(|_| scalars.next().is_none()) else {
                        return Err(diagnostic(
                            source,
                            "E-UNICODE-SCALAR-OPERAND",
                            operand_span,
                            "Unicode scalar value requires exactly one scalar value",
                        ));
                    };
                    let value = Value::Int(BigInt::from(u32::from(first)));
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && let Expression::Identifier(name) = &items[0]
                    && matches!(
                        source.slice(*name),
                        "unicode-whitespace-character"
                            | "unicode-line-feed-character"
                            | "unicode-carriage-return-character"
                            | "unicode-decimal-digit-character"
                            | "unicode-word-character"
                    )
                {
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let Value::String(character) = operand else {
                        return Err(diagnostic(
                            source,
                            "E-UNICODE-WHITESPACE-OPERAND",
                            operand_span,
                            "Unicode whitespace classification requires Character",
                        ));
                    };
                    let operation = source.slice(*name);
                    let mut scalars = character.chars();
                    let first = scalars.next();
                    let matches = scalars.next().is_none()
                        && match operation {
                            "unicode-whitespace-character" => {
                                first.is_some_and(is_unicode_white_space)
                            }
                            "unicode-line-feed-character" => first == Some('\n'),
                            "unicode-carriage-return-character" => first == Some('\r'),
                            "unicode-decimal-digit-character" => {
                                first.is_some_and(is_decimal_digit)
                            }
                            "unicode-word-character" => first.is_some_and(is_regex_word),
                            _ => unreachable!("Unicode Character primitive is dispatched"),
                        };
                    let value = Value::Boolean(matches);
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && let Expression::Identifier(name_span) = &items[0]
                    && (!self.bindings.contains_key(source.slice(*name_span))
                        || matches!(
                            self.bindings.get(source.slice(*name_span)),
                            Some(Value::NamedFunction(function))
                                if function.name == source.slice(*name_span)
                        ))
                    && self.functions.contains_key(source.slice(*name_span))
                {
                    return self
                        .evaluate_user_function_call(source, *name_span, &items[1], *span, trace);
                }
                if items.len() == 2
                    && matches!(&items[0], Expression::Identifier(name) if source.slice(*name) == "empty?")
                {
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let value = apply_empty_predicate(source, operand, operand_span, trace)?;
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && let Expression::Identifier(name) = &items[0]
                    && matches!(
                        source.slice(*name),
                        "array-at?" | "map-lookup" | "set-contains?" | "bag-multiplicity"
                    )
                {
                    let operation = source.slice(*name);
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let value =
                        apply_collection_query(source, operation, operand, operand_span, trace)?;
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && let Expression::Identifier(name) = &items[0]
                    && matches!(
                        source.slice(*name),
                        "range-lower"
                            | "range-upper"
                            | "range-lower-inclusive?"
                            | "range-upper-inclusive?"
                    )
                {
                    let operation = source.slice(*name);
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let value = apply_range_bound(source, operation, operand, operand_span, trace)?;
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if is_list_uncons(source, items) {
                    return evaluate_list_uncons(source, self, items, *span, trace);
                }
                if is_list_projection(source, items) {
                    return evaluate_list_projection(source, self, items, *span, trace);
                }
                if items.len() == 2
                    && let Expression::Identifier(name) = &items[0]
                    && matches!(source.slice(*name), "character-count" | "entry-count")
                {
                    let operation = source.slice(*name);
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let value = apply_count(source, operation, operand, operand_span, trace)?;
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && matches!(&items[0], Expression::Identifier(name) if source.slice(*name) == "upper")
                {
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let Value::String(text) = operand else {
                        return Err(diagnostic(
                            source,
                            "E-NO-APPLICABLE-OVERLOAD",
                            operand_span,
                            "upper requires a String operand in the implemented subset",
                        ));
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.upper(String)",
                    });
                    let value = Value::String(uppercase(&text));
                    trace.record(TraceEvent {
                        event: "string.uppercased",
                        rule: "TOPAL-STRING-UPPER-001",
                        detail: "unicode-default",
                    });
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && matches!(&items[0], Expression::Identifier(name) if source.slice(*name) == "lower")
                {
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let Value::String(text) = operand else {
                        return Err(diagnostic(
                            source,
                            "E-NO-APPLICABLE-OVERLOAD",
                            operand_span,
                            "lower requires a String operand in the implemented subset",
                        ));
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.lower(String)",
                    });
                    let value = Value::String(lowercase(&text));
                    trace.record(TraceEvent {
                        event: "string.lowercased",
                        rule: "TOPAL-STRING-LOWER-001",
                        detail: "unicode-default",
                    });
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && matches!(&items[0], Expression::Identifier(name) if source.slice(*name) == "case-fold")
                {
                    let operand_span = items[1].span();
                    let operand = self.evaluate_expression(source, &items[1], trace)?;
                    let Value::String(text) = operand else {
                        return Err(diagnostic(
                            source,
                            "E-NO-APPLICABLE-OVERLOAD",
                            operand_span,
                            "case-fold requires a String operand in the implemented subset",
                        ));
                    };
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.case-fold(String)",
                    });
                    let value = Value::String(case_fold(&text));
                    trace.record(TraceEvent {
                        event: "string.case-folded",
                        rule: "TOPAL-STRING-CASE-FOLD-001",
                        detail: "unicode-default-full",
                    });
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                if items.len() == 2
                    && matches!(&items[0], Expression::Identifier(name) if source.slice(*name) == "empty")
                    && matches!(&items[1], Expression::Identifier(name) if source.slice(*name) == "String")
                {
                    trace.record(TraceEvent {
                        event: "operator.selected",
                        rule: "TOPAL-TYPE-CALL-001",
                        detail: "root.empty(String)",
                    });
                    trace.record(TraceEvent {
                        event: "string.empty",
                        rule: "TOPAL-STRING-EMPTY-001",
                        detail: "String",
                    });
                    let value = Value::String(String::new());
                    self.checkpoint(trace, Some(&value), Some(*span));
                    return Ok(value);
                }
                let (mut result, mut index) = if matches!(
                    items.first(),
                    Some(Expression::Callable {
                        kind: CallableKind::Minus,
                        ..
                    })
                ) {
                    let Some(operand) = items.get(1) else {
                        return Err(diagnostic(
                            source,
                            "E-EXPECTED-OPERAND",
                            *span,
                            "expected an operand after prefix -",
                        ));
                    };
                    let operand = self.evaluate_expression(source, operand, trace)?;
                    (apply_negate(source, operand, *span, trace)?, 2)
                } else {
                    (self.evaluate_expression(source, &items[0], trace)?, 1)
                };
                let mut composing_literals = matches!(items.first(), Some(Expression::String(_)));
                while index < items.len() {
                    if matches!(
                        &result,
                        Value::NamedFunction(_) | Value::Callable(_) | Value::AnonymousFunction(_)
                    ) {
                        let argument_expression = &items[index];
                        let argument_span = argument_expression.span();
                        let argument =
                            self.evaluate_expression(source, argument_expression, trace)?;
                        let call_span = cover(items[0].span(), argument_span);
                        result = self.invoke_function_value(
                            source,
                            result,
                            argument,
                            argument_span,
                            call_span,
                            trace,
                        )?;
                        self.checkpoint(trace, Some(&result), Some(call_span));
                        index += 1;
                        composing_literals = false;
                        continue;
                    }
                    if composing_literals
                        && let Expression::String(right_span) = &items[index]
                        && let Value::String(left) = &result
                    {
                        let Value::String(right) =
                            self.evaluate_expression(source, &items[index], trace)?
                        else {
                            unreachable!("string literal constructs String");
                        };
                        trace.record(TraceEvent {
                            event: "string.literals.composed",
                            rule: "TOPAL-STRING-LITERAL-COMPOSE-001",
                            detail: "String",
                        });
                        result = Value::String(format!("{left}{right}"));
                        self.checkpoint(
                            trace,
                            Some(&result),
                            Some(cover(items[0].span(), *right_span)),
                        );
                        index += 1;
                        continue;
                    }
                    composing_literals = false;
                    if let Expression::Identifier(callable_span) = &items[index]
                        && source.slice(*callable_span) == "normalize"
                        && let Value::String(text) = &result
                    {
                        let Some(form) = items.get(index + 1) else {
                            return Err(diagnostic(
                                source,
                                "E-EXPECTED-OPERAND",
                                Span::new(callable_span.end, callable_span.end),
                                "expected a normalization form after normalize",
                            ));
                        };
                        let form_span = form.span();
                        let Expression::Identifier(form_name) = form else {
                            return Err(diagnostic(
                                source,
                                "E-NO-APPLICABLE-OVERLOAD",
                                form_span,
                                "the implemented String normalize operation requires NFC or NFD",
                            ));
                        };
                        let form_name = source.slice(*form_name);
                        let (normalized, selection, rule) = match form_name {
                            "NFC" => (
                                normalize_nfc(text),
                                "root.normalize(String,NFC)",
                                "TOPAL-STRING-NORMALIZE-NFC-001",
                            ),
                            "NFD" => (
                                normalize_nfd(text),
                                "root.normalize(String,NFD)",
                                "TOPAL-STRING-NORMALIZE-NFD-001",
                            ),
                            _ => {
                                return Err(diagnostic(
                                    source,
                                    "E-NO-APPLICABLE-OVERLOAD",
                                    form_span,
                                    "the implemented String normalize operation requires NFC or NFD",
                                ));
                            }
                        };
                        trace.record(TraceEvent {
                            event: "operator.selected",
                            rule: "TOPAL-TYPE-CALL-001",
                            detail: selection,
                        });
                        let changed = normalized != *text;
                        trace.record(TraceEvent {
                            event: "string.normalized",
                            rule,
                            detail: if changed {
                                "changed=true"
                            } else {
                                "changed=false"
                            },
                        });
                        result = Value::String(normalized);
                        self.checkpoint(
                            trace,
                            Some(&result),
                            Some(cover(items[0].span(), form_span)),
                        );
                        index += 2;
                        continue;
                    }
                    if let Expression::Identifier(callable_span) = &items[index]
                        && source.slice(*callable_span) == "byte-count"
                        && let Value::String(text) = &result
                    {
                        let Some(encoding) = items.get(index + 1) else {
                            return Err(diagnostic(
                                source,
                                "E-EXPECTED-OPERAND",
                                Span::new(callable_span.end, callable_span.end),
                                "expected an Encoding after byte-count",
                            ));
                        };
                        let encoding_span = encoding.span();
                        if !matches!(encoding, Expression::Identifier(name) if source.slice(*name) == "Utf8")
                        {
                            return Err(diagnostic(
                                source,
                                "E-NO-APPLICABLE-OVERLOAD",
                                encoding_span,
                                "the implemented String byte-count operation requires Utf8",
                            ));
                        }
                        trace.record(TraceEvent {
                            event: "operator.selected",
                            rule: "TOPAL-TYPE-CALL-001",
                            detail: "root.byte-count(String,Utf8)",
                        });
                        let byte_count = text.len();
                        let detail = format!("bytes={byte_count}");
                        trace.record(TraceEvent {
                            event: "string.utf8-byte-count",
                            rule: "TOPAL-STRING-UTF8-BYTE-COUNT-001",
                            detail: &detail,
                        });
                        result = Value::Int(BigInt::from(byte_count));
                        self.checkpoint(
                            trace,
                            Some(&result),
                            Some(cover(items[0].span(), encoding_span)),
                        );
                        index += 2;
                        continue;
                    }
                    if let Expression::Identifier(callable_span) = &items[index]
                        && source.slice(*callable_span) == "reverse"
                        && matches!(result, Value::List { .. })
                    {
                        apply_list_reverse(&mut result, trace);
                        index += 1;
                        continue;
                    }
                    if let Expression::Identifier(callable_span) = &items[index]
                        && matches!(
                            source.slice(*callable_span),
                            "stable-sort" | "stable-sort-descending"
                        )
                        && matches!(result, Value::List { .. })
                    {
                        let descending = source.slice(*callable_span) == "stable-sort-descending";
                        apply_list_stable_sort(
                            source,
                            &mut result,
                            descending,
                            *callable_span,
                            trace,
                        )?;
                        index += 1;
                        continue;
                    }
                    if let Expression::Identifier(callable_span) = &items[index]
                        && source.slice(*callable_span) == "entries"
                        && matches!(result, Value::List { .. })
                    {
                        result = apply_list_entries_view(result, trace);
                        index += 1;
                        continue;
                    }
                    if let Expression::Identifier(callable_span) = &items[index]
                        && source.slice(*callable_span) == "insert-at"
                        && let Value::List { .. } = result
                    {
                        result = self.evaluate_list_insert_at(
                            source,
                            result,
                            items.get(index + 1),
                            items.get(index + 2),
                            *callable_span,
                            trace,
                        )?;
                        index += 3;
                        continue;
                    }
                    if let Expression::Identifier(callable_span) = &items[index]
                        && matches!(
                            source.slice(*callable_span),
                            "prepend"
                                | "append"
                                | "concat"
                                | "contains-entry"
                                | "contains-sequence"
                                | "contains-subsequence"
                                | "split-at"
                                | "take"
                                | "drop"
                                | "remove"
                                | "remove-indexes"
                                | "zip-exact"
                                | "zip-shortest"
                                | "remove-first"
                                | "remove-all"
                        )
                        && matches!(result, Value::List { .. })
                    {
                        let operation = source.slice(*callable_span);
                        let Some(right) = items.get(index + 1) else {
                            return Err(diagnostic(
                                source,
                                "E-EXPECTED-OPERAND",
                                Span::new(callable_span.end, callable_span.end),
                                format!("expected an operand after {operation}"),
                            ));
                        };
                        let right_span = right.span();
                        let right_is_closed = expression_is_closed(right);
                        let right = self.evaluate_expression(source, right, trace)?;
                        result = apply_list_operation(
                            source,
                            operation,
                            result,
                            right,
                            right_span,
                            right_is_closed,
                            trace,
                        )?;
                        self.checkpoint(
                            trace,
                            Some(&result),
                            Some(cover(items[0].span(), right_span)),
                        );
                        index += 2;
                        continue;
                    }
                    if let Expression::Identifier(callable_span) = &items[index]
                        && source.slice(*callable_span) == "concat"
                        && let Value::String(left) = &result
                    {
                        let Some(right) = items.get(index + 1) else {
                            return Err(diagnostic(
                                source,
                                "E-EXPECTED-OPERAND",
                                Span::new(callable_span.end, callable_span.end),
                                "expected a String after concat",
                            ));
                        };
                        let right_span = right.span();
                        let right = self.evaluate_expression(source, right, trace)?;
                        let Value::String(right) = right else {
                            return Err(diagnostic(
                                source,
                                "E-NO-APPLICABLE-OVERLOAD",
                                right_span,
                                "concat requires two String operands",
                            ));
                        };
                        trace.record(TraceEvent {
                            event: "operator.selected",
                            rule: "TOPAL-TYPE-CALL-001",
                            detail: "root.concat(String,String)",
                        });
                        trace.record(TraceEvent {
                            event: "evaluation.concat",
                            rule: "TOPAL-STRING-CONCAT-001",
                            detail: "String",
                        });
                        result = Value::String(format!("{left}{right}"));
                        self.checkpoint(
                            trace,
                            Some(&result),
                            Some(cover(items[0].span(), right_span)),
                        );
                        index += 2;
                        continue;
                    }
                    if let Expression::Identifier(label_span) = &items[index]
                        && let Value::Error {
                            domain,
                            code,
                            line,
                            column,
                        } = &result
                    {
                        let label = source.slice(*label_span);
                        let selected = match label {
                            "code" => Value::Enum {
                                type_name: "lang arithmetic ArithmeticErrorCode".into(),
                                alternative: code.clone(),
                            },
                            "domain" => Value::ErrorDomain(domain.clone()),
                            "detail" => Value::Optional {
                                payload_classifier: "String".into(),
                                payload: None,
                            },
                            "cause" => Value::Optional {
                                payload_classifier: "Error".into(),
                                payload: None,
                            },
                            "source" => Value::Optional {
                                payload_classifier: "SourceLocation".into(),
                                payload: Some(Box::new(Value::Record(vec![
                                    ("line".into(), Value::Int(BigInt::from(*line))),
                                    ("column".into(), Value::Int(BigInt::from(*column))),
                                ]))),
                            },
                            _ => {
                                return Err(diagnostic(
                                    source,
                                    "E-NO-SUCH-ERROR-FIELD",
                                    *label_span,
                                    format!("Error has no implemented field named `{label}`"),
                                ));
                            }
                        };
                        trace.record(TraceEvent {
                            event: "error.field.selected",
                            rule: "TOPAL-ERROR-FIELD-001",
                            detail: label,
                        });
                        result = selected;
                        index += 1;
                        continue;
                    }
                    if let Expression::Identifier(label_span) = &items[index]
                        && let Value::Record(fields) = &result
                    {
                        let label = source.slice(*label_span);
                        let selected = fields
                            .iter()
                            .find(|(field, _)| field == label)
                            .map(|(_, value)| value.clone())
                            .ok_or_else(|| {
                                diagnostic(
                                    source,
                                    "E-NO-SUCH-RECORD-FIELD",
                                    *label_span,
                                    format!("record has no field named `{label}`"),
                                )
                            })?;
                        trace.record(TraceEvent {
                            event: "record.field.selected",
                            rule: "TOPAL-TYPE-PRODUCT-001",
                            detail: label,
                        });
                        result = selected;
                        index += 1;
                        continue;
                    }
                    let Expression::Callable {
                        kind,
                        span: operator_span,
                    } = &items[index]
                    else {
                        let mut error = diagnostic(
                            source,
                            "E-UNSUPPORTED-APPLICATION",
                            items[index].span(),
                            "the implemented subset requires a symbolic callable",
                        );
                        if let Expression::Identifier(name_span) = &items[index]
                            && let Some(candidate) =
                                closest_root_operation(source.slice(*name_span))
                        {
                            error = error.with_help(format!("did you mean `{candidate}`?"));
                        }
                        return Err(error);
                    };
                    let Some(right) = items.get(index + 1) else {
                        return Err(diagnostic(
                            source,
                            "E-EXPECTED-OPERAND",
                            Span::new(operator_span.end, operator_span.end),
                            "expected an operand after callable",
                        ));
                    };
                    let right_span = right.span();
                    let right = self.evaluate_expression(source, right, trace)?;
                    result = apply_binary(
                        source,
                        *kind,
                        result,
                        right,
                        (*span, items[0].span(), right_span),
                        trace,
                    )?;
                    self.checkpoint(
                        trace,
                        Some(&result),
                        Some(cover(items[0].span(), right_span)),
                    );
                    index += 2;
                }
                Ok(result)
            }
        }?;
        self.checkpoint(trace, Some(&value), Some(expression.span()));
        Ok(value)
    }
}
