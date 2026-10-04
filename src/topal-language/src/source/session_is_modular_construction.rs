impl Session {
    fn is_modular_construction(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(name), _] if matches!(self.bindings.get(source.slice(*name)), Some(Value::ModularType(_))))
    }

    fn construct_modular_value(
        &self,
        source: &SourceText,
        items: &[Expression],
        _span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(name_span), operand] = items else {
            unreachable!("preselected modular construction")
        };
        let name = source.slice(*name_span);
        let Value::ModularType(kind) = self.bindings.get(name).expect("known modular type") else {
            unreachable!("preselected modular type")
        };
        let operand_value = self.evaluate_expression(source, operand, trace)?;
        let Value::Int(value) = operand_value else {
            return Err(diagnostic(
                source,
                "E-MODULAR-CONSTRUCTION-OPERAND",
                operand.span(),
                "modular construction requires Int",
            ));
        };
        if value < kind.lower || value > kind.upper {
            if expression_is_closed(operand) {
                return Err(diagnostic(
                    source,
                    "E-MODULAR-OUT-OF-RANGE",
                    operand.span(),
                    format!("value is outside `{name}` canonical range"),
                ));
            }
            let position = source.position(operand.span().start);
            return Ok(Value::Error {
                domain: format!("root.{name}(Int)"),
                code: "out-of-range".into(),
                line: position.line,
                column: position.column,
            });
        }
        trace.record(TraceEvent {
            event: "numeric.modular.constructed",
            rule: "TOPAL-NUM-MODULAR-CONSTRUCT-001",
            detail: name,
        });
        Ok(Value::Modular {
            type_name: name.into(),
            lower: kind.lower.clone(),
            upper: kind.upper.clone(),
            value,
        })
    }

    fn construct_constraint(
        &self,
        source: &SourceText,
        items: &[Expression],
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(base), _, predicate] = items else {
            unreachable!("preselected constraint definition")
        };
        let base_classifier = source.slice(*base);
        if !matches!(
            base_classifier,
            "Boolean" | "Int" | "Nat" | "Rational" | "String"
        ) {
            return Err(diagnostic(
                source,
                "E-CONSTRAINT-BASE",
                *base,
                "constraint base must be a supported value classifier",
            ));
        }
        let predicate = self.evaluate_expression(source, predicate, trace)?;
        trace.record(TraceEvent {
            event: "constraint.constructed",
            rule: "TOPAL-TYPE-CONSTRAINT-001",
            detail: base_classifier,
        });
        Ok(Value::Constraint(Box::new(ConstraintValue {
            name: None,
            base_classifier: base_classifier.into(),
            predicate,
        })))
    }

    fn is_constraint_application(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(name), _] if matches!(self.bindings.get(source.slice(*name)), Some(Value::Constraint(_))))
    }

    fn apply_constraint(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(name_span), operand] = items else {
            unreachable!("preselected constraint application")
        };
        let name = source.slice(*name_span);
        let Value::Constraint(constraint) = self.bindings.get(name).expect("known constraint")
        else {
            unreachable!("preselected constraint value")
        };
        let value = self.evaluate_expression(source, operand, trace)?;
        if !value_has_classifier(&value, &constraint.base_classifier) {
            return Err(diagnostic(
                source,
                "E-CONSTRAINT-OPERAND",
                operand.span(),
                format!(
                    "constraint `{name}` requires `{}`",
                    constraint.base_classifier
                ),
            ));
        }
        let decision = self.invoke_anonymous_function(
            &constraint.predicate,
            vec![value.clone()],
            span,
            trace,
        )?;
        let Value::Boolean(accepted) = decision else {
            return Err(diagnostic(
                source,
                "E-CONSTRAINT-PREDICATE-RESULT",
                operand.span(),
                "constraint predicate must return Boolean",
            ));
        };
        trace.record(TraceEvent {
            event: "constraint.validated",
            rule: "TOPAL-TYPE-CONSTRAINT-VALIDATE-001",
            detail: if accepted { "accepted" } else { "rejected" },
        });
        if accepted {
            return Ok(Value::Refined {
                constraint: name.into(),
                base_classifier: constraint.base_classifier.clone(),
                value: Box::new(value),
            });
        }
        if expression_is_closed(operand) {
            return Err(diagnostic(
                source,
                "E-CONSTRAINT-REJECTED",
                operand.span(),
                format!("value does not satisfy constraint `{name}`"),
            ));
        }
        let position = source.position(operand.span().start);
        Ok(Value::Error {
            domain: format!("root.{name}({})", constraint.base_classifier),
            code: "out-of-range".into(),
            line: position.line,
            column: position.column,
        })
    }

    fn is_characters_application(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items.first(), Some(Expression::Identifier(operation)) if source.slice(*operation) == "characters")
    }

    fn evaluate_characters_application(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let text = items.get(1).expect("characters application has text");
        let text_span = text.span();
        let text = self.evaluate_expression(source, text, trace)?;
        let Value::String(text) = text else {
            return Err(diagnostic(
                source,
                "E-CHARACTERS-OPERAND",
                text_span,
                "characters requires a String operand",
            ));
        };
        let value = if items.len() == 4 {
            trace.record(TraceEvent {
                event: "operator.selected",
                rule: "TOPAL-TYPE-CALL-001",
                detail: "root.characters(String)",
            });
            let mut collected = String::new();
            for character in characters(&text) {
                trace.record(TraceEvent {
                    event: "generator.yielded",
                    rule: "TOPAL-STRING-CHARACTERS-COLLECT-001",
                    detail: character,
                });
                collected.push_str(character);
            }
            trace.record(TraceEvent {
                event: "string.characters.collected",
                rule: "TOPAL-STRING-CHARACTERS-COLLECT-001",
                detail: "String",
            });
            Value::String(collected)
        } else {
            trace.record(TraceEvent {
                event: "generator.started",
                rule: "TOPAL-STRING-CHARACTERS-GENERATOR-001",
                detail: "Generator Character Unit Unit",
            });
            Value::CharacterGenerator {
                generated: characters(&text).map(str::to_owned).collect(),
                origin: "root.characters".to_owned(),
            }
        };
        self.checkpoint(trace, Some(&value), Some(span));
        Ok(value)
    }

    fn construct_union_application(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        if let [Expression::Identifier(constructor), payload] = items {
            return self.construct_union(source, *constructor, payload, span, trace);
        }
        let [
            Expression::Identifier(type_name),
            _,
            Expression::Integer(index),
            payload,
        ] = items
        else {
            unreachable!("preselected positional Variant constructor application")
        };
        let index_text = source.slice(*index);
        let key = format!("at {index_text}");
        let type_text = source.slice(*type_name);
        let classifier = self
            .union_types
            .get(type_text)
            .and_then(|alternatives| alternatives.get(&key))
            .and_then(Option::as_deref)
            .ok_or_else(|| {
                diagnostic(
                    source,
                    "E-VARIANT-INDEX",
                    *index,
                    "Variant alternative index is outside its declared bounds",
                )
            })?;
        let value = self.evaluate_expression(source, payload, trace)?;
        if !value_has_classifier(&value, classifier) {
            return Err(diagnostic(
                source,
                "E-VARIANT-PAYLOAD-CLASSIFIER",
                payload.span(),
                format!("Variant alternative {index_text} requires `{classifier}`"),
            ));
        }
        trace.record(TraceEvent {
            event: "variant.constructed",
            rule: "TOPAL-TYPE-VARIANT-001",
            detail: index_text,
        });
        Ok(Value::Union(Box::new(UnionValue {
            type_name: type_text.into(),
            alternative: key,
            payload_classifier: Some(classifier.into()),
            payload: Some(Box::new(value)),
            supports_equality: self.classifier_supports_equality(type_text),
        })))
    }

    fn construct_union(
        &self,
        source: &SourceText,
        constructor: Span,
        payload: &Expression,
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let name = source.slice(constructor);
        let (type_name, classifier) = self
            .union_constructor(name)
            .expect("preselected payload Union constructor");
        let value = self.evaluate_expression(source, payload, trace)?;
        if !value_has_classifier(&value, classifier) {
            return Err(diagnostic(
                source,
                "E-UNION-PAYLOAD-CLASSIFIER",
                payload.span(),
                format!(
                    "Union constructor `{name}` requires `{classifier}`, found `{}`",
                    structural_value_classifier(&value)
                ),
            ));
        }
        trace.record(TraceEvent {
            event: "union.constructed",
            rule: "TOPAL-TYPE-UNION-001",
            detail: name,
        });
        let result = Value::Union(Box::new(UnionValue {
            type_name: type_name.to_owned(),
            alternative: name.to_owned(),
            payload_classifier: Some(classifier.to_owned()),
            payload: Some(Box::new(value)),
            supports_equality: self.classifier_supports_equality(type_name),
        }));
        self.checkpoint(trace, Some(&result), Some(span));
        Ok(result)
    }

    fn invoke_anonymous_function(
        &self,
        function: &Value,
        arguments: Vec<Value>,
        call_span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let Value::AnonymousFunction(function) = function else {
            unreachable!("anonymous invocation is dispatched only for an anonymous function")
        };
        let AnonymousFunction {
            source,
            parameters,
            body,
            bindings,
            defining_context,
        } = function.as_ref();
        if parameters.len() != arguments.len() {
            return Err(diagnostic(
                source,
                "E-ANONYMOUS-FUNCTION-ARITY",
                call_span,
                format!(
                    "anonymous function expects {} arguments, found {}",
                    parameters.len(),
                    arguments.len()
                ),
            ));
        }
        let mut invocation = Box::new(self.clone());
        invocation.root_namespace = Some(self.effective_root_namespace());
        invocation.bindings = bindings.clone();
        invocation.defining_context.clone_from(defining_context);
        let mut matched_bindings = BTreeMap::new();
        for (parameter, argument) in parameters.iter().zip(arguments) {
            bind_anonymous_pattern(
                source,
                &mut invocation,
                &mut matched_bindings,
                parameter,
                argument,
                call_span,
                trace,
            )?;
        }
        let detail = format!("arguments={}", parameters.len());
        trace.record(TraceEvent {
            event: "function.anonymous.called",
            rule: "TOPAL-FUNCTION-ANONYMOUS-001",
            detail: &detail,
        });
        invocation.evaluate_expression(source, body, trace)
    }

    fn capture_anonymous_function(
        &self,
        source: &SourceText,
        parameters: &[AnonymousPattern],
        body: &Expression,
        trace: &mut impl TraceSink,
    ) -> Value {
        let parameters = parameters
            .iter()
            .map(|parameter| capture_anonymous_pattern(source, parameter))
            .collect::<Vec<_>>();
        let detail = format!("parameters={}", parameters.len());
        trace.record(TraceEvent {
            event: "function.anonymous.captured",
            rule: "TOPAL-FUNCTION-ANONYMOUS-001",
            detail: &detail,
        });
        let bindings = self
            .bindings
            .iter()
            .filter(|(name, _)| expression_mentions_name(source, body, name))
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect();
        Value::AnonymousFunction(Rc::new(AnonymousFunction {
            source: source.clone(),
            parameters,
            body: Box::new(body.clone()),
            bindings,
            defining_context: self
                .defining_context
                .clone()
                .or_else(|| Some(self.bindings.clone())),
        }))
    }

    fn evaluate_list_insert_at(
        &self,
        source: &SourceText,
        list: Value,
        boundary: Option<&Expression>,
        inserted: Option<&Expression>,
        operation_span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let Some(boundary) = boundary else {
            return Err(diagnostic(
                source,
                "E-EXPECTED-OPERAND",
                operation_span,
                "expected a boundary after insert-at",
            ));
        };
        let Some(inserted) = inserted else {
            return Err(diagnostic(
                source,
                "E-EXPECTED-OPERAND",
                boundary.span(),
                "expected a value or List after the insertion boundary",
            ));
        };
        let boundary_value = self.evaluate_expression(source, boundary, trace)?;
        let inserted_value = self.evaluate_expression(source, inserted, trace)?;
        apply_list_insert_at(
            source,
            list,
            boundary_value,
            boundary.span(),
            inserted_value,
            inserted.span(),
            trace,
        )
    }

    #[allow(clippy::too_many_lines)] // Collection laws remain explicit in one isolated frame.
    fn evaluate_list_higher_order(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        if let [collection, Expression::Identifier(operation_span), function] = items
            && matches!(
                source.slice(*operation_span),
                "map" | "select" | "remove-indexes" | "remove-values"
            )
        {
            return (|| {
                let collection_span = collection.span();
                let collection = self.evaluate_expression(source, collection, trace)?;
                let Value::List {
                    element_classifier,
                    entries,
                } = collection
                else {
                    return Err(diagnostic(
                        source,
                        "E-COLLECTION-OPERATION-SOURCE",
                        collection_span,
                        format!(
                            "{} requires a homogeneous collection",
                            source.slice(*operation_span)
                        ),
                    ));
                };
                let function_span = function.span();
                let function = self.evaluate_expression(source, function, trace)?;
                let operation = source.slice(*operation_span);
                let mut output = Vec::new();
                for (index, entry) in entries.into_iter().enumerate() {
                    let input = entry.clone();
                    let argument = if operation == "remove-indexes" {
                        Value::Int(BigInt::from(index))
                    } else {
                        entry
                    };
                    let transformed =
                        self.invoke_anonymous_function(&function, vec![argument], span, trace)?;
                    if matches!(operation, "select" | "remove-indexes" | "remove-values") {
                        let Value::Boolean(retain) = transformed else {
                            return Err(diagnostic(
                                source,
                                "E-SELECT-PREDICATE-RESULT",
                                function_span,
                                format!("{operation} predicate must return Boolean"),
                            ));
                        };
                        if retain == (operation == "select") {
                            output.push(input);
                        }
                    } else {
                        output.push(transformed);
                    }
                }
                let output_classifier = if operation != "map" || output.is_empty() {
                    element_classifier
                } else {
                    let classifier = structural_value_classifier(&output[0]);
                    if output
                        .iter()
                        .any(|value| structural_value_classifier(value) != classifier)
                    {
                        return Err(diagnostic(
                            source,
                            "E-MAP-RESULT-CLASSIFIER",
                            function_span,
                            "map transformation returned values with different classifiers",
                        ));
                    }
                    classifier
                };
                let selection = format!("root.{operation}(List {output_classifier})");
                trace.record(TraceEvent {
                    event: "operator.selected",
                    rule: "TOPAL-TYPE-CALL-001",
                    detail: &selection,
                });
                trace.record(TraceEvent {
                    event: match operation {
                        "map" => "list.mapped",
                        "select" => "list.selected",
                        _ => "list.entries.removed",
                    },
                    rule: match operation {
                        "map" => "TOPAL-COLLECTION-MAP-001",
                        "select" => "TOPAL-COLLECTION-SELECT-001",
                        "remove-indexes" => "TOPAL-LIST-REMOVE-INDEXES-001",
                        "remove-values" => "TOPAL-LIST-REMOVE-VALUES-001",
                        _ => unreachable!("known higher-order List operation"),
                    },
                    detail: &output_classifier,
                });
                let result = Value::List {
                    element_classifier: output_classifier,
                    entries: output,
                };
                self.checkpoint(trace, Some(&result), Some(span));
                Ok(result)
            })();
        }
        if let [
            collection,
            Expression::Identifier(operation),
            initial,
            function,
        ] = items
            && source.slice(*operation) == "fold"
        {
            return (|| {
                let collection_span = collection.span();
                let collection = self.evaluate_expression(source, collection, trace)?;
                let Value::List { entries, .. } = collection else {
                    return Err(diagnostic(
                        source,
                        "E-COLLECTION-OPERATION-SOURCE",
                        collection_span,
                        "fold requires an ordered homogeneous collection",
                    ));
                };
                let mut state = self.evaluate_expression(source, initial, trace)?;
                let expected = structural_value_classifier(&state);
                let function_span = function.span();
                let function = self.evaluate_expression(source, function, trace)?;
                for entry in entries {
                    let transformed = self.invoke_anonymous_function(
                        &function,
                        vec![state.clone(), entry],
                        span,
                        trace,
                    )?;
                    state = match transformed {
                        Value::Continue(next) => *next,
                        Value::Finish(result) => {
                            let result = *result;
                            if !value_has_classifier(&result, &expected) {
                                return Err(diagnostic(
                                    source,
                                    "E-FOLD-FINISH-CLASSIFIER",
                                    function_span,
                                    format!("Finish result must satisfy `{expected}`"),
                                ));
                            }
                            trace.record(TraceEvent {
                                event: "traversal.finished",
                                rule: "TOPAL-EXEC-TRAVERSAL-CONTROL-001",
                                detail: "fold",
                            });
                            return Ok(result);
                        }
                        value => value,
                    };
                    if !value_has_classifier(&state, &expected) {
                        return Err(diagnostic(
                            source,
                            "E-FOLD-STATE-CLASSIFIER",
                            function_span,
                            format!("fold step must preserve state classifier `{expected}`"),
                        ));
                    }
                }
                trace.record(TraceEvent {
                    event: "list.folded",
                    rule: "TOPAL-COLLECTION-FOLD-001",
                    detail: &expected,
                });
                self.checkpoint(trace, Some(&state), Some(span));
                Ok(state)
            })();
        }
        unreachable!("higher-order List operation is preselected by its application shape")
    }

    #[allow(clippy::too_many_lines)] // Collector spellings and their distinct laws remain auditable together.
    fn evaluate_list_materialization(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        if let [Expression::Identifier(operation), pairs] = items
            && source.slice(*operation) == "unzip"
        {
            let pairs_span = pairs.span();
            let pairs = self.evaluate_expression(source, pairs, trace)?;
            return apply_list_unzip(source, pairs, pairs_span, trace);
        }
        if let [Expression::Identifier(operation), collection] = items
            && source.slice(*operation) == "collect"
        {
            let value = self.evaluate_expression(source, collection, trace)?;
            if let Value::IterateGenerator { .. } = value {
                return self.collect_iterate_generator(
                    source,
                    value,
                    collection.span(),
                    span,
                    trace,
                );
            }
            if let Value::UnfoldGenerator { .. } = value {
                return self.collect_unfold_generator(
                    source,
                    value,
                    collection.span(),
                    span,
                    trace,
                );
            }
            if let Value::CharacterGenerator { generated, origin } = value {
                trace.record(TraceEvent {
                    event: "generator.consumed",
                    rule: "TOPAL-STRING-CHARACTERS-GENERATOR-001",
                    detail: &origin,
                });
                let result = Value::List {
                    element_classifier: "Character".into(),
                    entries: generated.into_iter().map(Value::String).collect(),
                };
                trace.record(TraceEvent {
                    event: "list.collected",
                    rule: "TOPAL-COLLECTION-COLLECT-LIST-001",
                    detail: "List Character",
                });
                return Ok(result);
            }
            if matches!(value, Value::List { .. }) {
                trace.record(TraceEvent {
                    event: "list.collected",
                    rule: "TOPAL-COLLECTION-COLLECT-LIST-001",
                    detail: "List",
                });
                return Ok(value);
            }
            return Err(diagnostic(
                source,
                "E-COLLECT-SOURCE",
                collection.span(),
                "unary collect requires a finite homogeneous traversal",
            ));
        }
        if let [
            collection,
            Expression::Identifier(operation),
            Expression::Identifier(target),
        ] = items
            && source.slice(*operation) == "collect"
        {
            let value = self.evaluate_expression(source, collection, trace)?;
            if source.slice(*target) == "Array" {
                let Value::List {
                    element_classifier,
                    entries,
                } = value
                else {
                    return Err(diagnostic(
                        source,
                        "E-COLLECT-ARRAY-SOURCE",
                        collection.span(),
                        "Array collection requires a finite List",
                    ));
                };
                trace.record(TraceEvent {
                    event: "array.collected",
                    rule: "TOPAL-ARRAY-COLLECT-001",
                    detail: &format!("count={}", entries.len()),
                });
                return Ok(Value::Array {
                    element_classifier,
                    entries,
                });
            }
            if source.slice(*target) != "String" {
                return Err(diagnostic(
                    source,
                    "E-COLLECT-TARGET",
                    *target,
                    "implemented collectors are Array and String",
                ));
            }
            let Value::List { entries, .. } = value else {
                return Err(diagnostic(
                    source,
                    "E-COLLECT-SOURCE",
                    collection.span(),
                    "String collection requires a finite List of Character or String entries",
                ));
            };
            let mut text = String::new();
            for entry in entries {
                let Value::String(fragment) = entry else {
                    return Err(diagnostic(
                        source,
                        "E-COLLECT-STRING-ENTRY",
                        collection.span(),
                        "String collection requires Character or String entries",
                    ));
                };
                text.push_str(&fragment);
            }
            trace.record(TraceEvent {
                event: "string.collected",
                rule: "TOPAL-COLLECTION-COLLECT-STRING-001",
                detail: "String",
            });
            return Ok(Value::String(text));
        }
        if let [Expression::Identifier(operation), collection] = items
            && matches!(source.slice(*operation), "collect-set" | "collect-bag")
        {
            let value = self.evaluate_expression(source, collection, trace)?;
            return collect_unordered(
                source,
                source.slice(*operation),
                value,
                collection.span(),
                trace,
            );
        }
        if let [
            Expression::Identifier(operation),
            collection,
            Expression::Identifier(resolving),
            Expression::Identifier(policy),
        ] = items
            && source.slice(*operation) == "collect-map"
            && source.slice(*resolving) == "resolving"
        {
            let value = self.evaluate_expression(source, collection, trace)?;
            return collect_map(
                source,
                value,
                source.slice(*policy),
                collection.span(),
                trace,
            );
        }
        if let [
            left_with_default,
            Expression::Identifier(operation),
            right_with_default,
        ] = items
            && source.slice(*operation) == "zip-longest"
        {
            let left = self.evaluate_expression(source, left_with_default, trace)?;
            let right = self.evaluate_expression(source, right_with_default, trace)?;
            return apply_list_zip_longest(source, left, right, span, trace);
        }
        Err(diagnostic(
            source,
            "E-COLLECTION-APPLICATION",
            span,
            "collection materialization does not match a declared operation form",
        ))
    }

    fn collect_iterate_generator(
        &self,
        source: &SourceText,
        generator: Value,
        source_span: Span,
        result_span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let Value::IterateGenerator {
            mut current,
            next,
            take_while,
            classifier,
        } = generator
        else {
            unreachable!("generated collection requires iterate generator")
        };
        let Some(predicate) = take_while else {
            return Err(diagnostic(
                source,
                "E-UNBOUNDED-GENERATOR-COLLECT",
                source_span,
                "collect requires a statically finite generated traversal",
            ));
        };
        let mut entries = Vec::new();
        loop {
            let accepted = self.invoke_anonymous_function(
                &predicate,
                vec![(*current).clone()],
                result_span,
                trace,
            )?;
            let Value::Boolean(accepted) = accepted else {
                return Err(diagnostic(
                    source,
                    "E-TAKE-WHILE-PREDICATE-RESULT",
                    source_span,
                    "take-while predicate must return Boolean",
                ));
            };
            if !accepted {
                break;
            }
            entries.push((*current).clone());
            let next_value =
                self.invoke_anonymous_function(&next, vec![*current], result_span, trace)?;
            if !value_has_classifier(&next_value, &classifier) {
                return Err(diagnostic(
                    source,
                    "E-ITERATE-NEXT-CLASSIFIER",
                    source_span,
                    format!("iterate next function must return `{classifier}`"),
                ));
            }
            *current = next_value;
        }
        trace.record(TraceEvent {
            event: "generator.collected",
            rule: "TOPAL-GENERATOR-COLLECT-001",
            detail: &classifier,
        });
        let value = Value::List {
            element_classifier: classifier,
            entries,
        };
        self.checkpoint(trace, Some(&value), Some(result_span));
        Ok(value)
    }

    fn collect_unfold_generator(
        &self,
        source: &SourceText,
        generator: Value,
        source_span: Span,
        result_span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let Value::UnfoldGenerator {
            mut seed,
            step,
            yield_classifier,
        } = generator
        else {
            unreachable!("unfold collection requires unfold generator")
        };
        let seed_classifier = structural_value_classifier(&seed);
        let mut element_classifier = (yield_classifier != "Value").then_some(yield_classifier);
        let mut entries = Vec::new();
        loop {
            let result = self.invoke_anonymous_function(&step, vec![*seed], result_span, trace)?;
            let Value::Optional { payload, .. } = result else {
                return Err(diagnostic(
                    source,
                    "E-UNFOLD-STEP-RESULT",
                    source_span,
                    "unfold step must return Optional (Yield, Seed)",
                ));
            };
            let Some(payload) = payload else {
                break;
            };
            let Value::Tuple(mut pair) = *payload else {
                return Err(diagnostic(
                    source,
                    "E-UNFOLD-STEP-RESULT",
                    source_span,
                    "unfold Some payload must be a two-field positional product",
                ));
            };
            if pair.len() != 2 {
                return Err(diagnostic(
                    source,
                    "E-UNFOLD-STEP-RESULT",
                    source_span,
                    "unfold Some payload must contain yielded value and next seed",
                ));
            }
            let next_seed = pair.pop().expect("two-field unfold payload");
            let yielded = pair.pop().expect("two-field unfold payload");
            if !value_has_classifier(&next_seed, &seed_classifier) {
                return Err(diagnostic(
                    source,
                    "E-UNFOLD-SEED-CLASSIFIER",
                    source_span,
                    format!("unfold next seed must satisfy `{seed_classifier}`"),
                ));
            }
            let yielded_classifier = structural_value_classifier(&yielded);
            if element_classifier
                .as_ref()
                .is_some_and(|expected| expected != &yielded_classifier)
            {
                return Err(diagnostic(
                    source,
                    "E-UNFOLD-YIELD-CLASSIFIER",
                    source_span,
                    "unfold step yielded inconsistent value classifiers",
                ));
            }
            element_classifier.get_or_insert(yielded_classifier);
            trace.record(TraceEvent {
                event: "generator.yielded",
                rule: "TOPAL-GENERATOR-UNFOLD-COLLECT-001",
                detail: &yielded.to_string(),
            });
            entries.push(yielded);
            *seed = next_seed;
        }
        let element_classifier = element_classifier.unwrap_or_else(|| "Value".into());
        trace.record(TraceEvent {
            event: "generator.collected",
            rule: "TOPAL-GENERATOR-UNFOLD-COLLECT-001",
            detail: &element_classifier,
        });
        let value = Value::List {
            element_classifier,
            entries,
        };
        self.checkpoint(trace, Some(&value), Some(result_span));
        Ok(value)
    }
}
