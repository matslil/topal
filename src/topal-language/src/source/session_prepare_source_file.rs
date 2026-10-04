impl Session {
    /// Prepare a complete source file and require its initial language selection.
    ///
    /// # Errors
    ///
    /// Returns a diagnostic when the header is absent or the source is invalid.
    pub fn prepare_source_file(
        &mut self,
        input: &str,
        trace: &mut impl TraceSink,
    ) -> Result<Execution, Diagnostic> {
        let mut execution = self.prepare(input, trace)?;
        let Some(Statement::LanguageSelection {
            version, features, ..
        }) = execution.statements.first()
        else {
            return Err(diagnostic(
                &execution.source,
                "E-MISSING-LANGUAGE-VERSION",
                Span::new(0, 0),
                "source files begin with a language-version selection",
            )
            .with_help("add `use language (\n  version is v0.1\n)` at the start of the file"));
        };
        let features = features.clone();
        let requested = execution
            .source
            .slice(*version)
            .parse::<LanguageVersion>()
            .map_err(|message| {
                diagnostic(&execution.source, "E-LANGUAGE-VERSION", *version, message)
            })?;
        if !requested.is_supported_core() {
            return Err(diagnostic(
                &execution.source,
                "E-UNSUPPORTED-LANGUAGE-VERSION",
                *version,
                format!(
                    "language version `{requested}` is not supported; highest supported version is `{}`",
                    Self::highest_supported_language_version()
                ),
            ));
        }
        self.language_version = requested;
        self.declared_libraries.clear();
        let mut libraries = BTreeSet::new();
        let mut declarations_closed = false;
        for statement in execution.statements.iter().skip(1) {
            match statement {
                Statement::LibrarySelection { name, span, .. } if !declarations_closed => {
                    let identity = execution.source.slice(*name);
                    if !libraries.insert(identity.to_owned()) {
                        return Err(diagnostic(
                            &execution.source,
                            "E-DUPLICATE-LIBRARY",
                            *span,
                            format!("library `{identity}` is declared more than once"),
                        ));
                    }
                }
                Statement::LibrarySelection { span, .. } => {
                    return Err(diagnostic(
                        &execution.source,
                        "E-LIBRARY-DECLARATION-ORDER",
                        *span,
                        "library dependencies immediately follow the initial language selection",
                    ));
                }
                _ => declarations_closed = true,
            }
        }
        let documentation_lexed = lex(&execution.source);
        let documentation_parsed = parse(&execution.source, &documentation_lexed);
        for declaration in extract_documentation(
            &execution.source,
            &documentation_lexed,
            &documentation_parsed,
        ) {
            if let Some(documentation) = declaration.documentation {
                self.documentation.insert(declaration.name, documentation);
            }
        }
        self.language_features = features
            .iter()
            .map(|feature| execution.source.slice(*feature).to_owned())
            .collect();
        execution.statements.remove(0);
        if execution.statements.is_empty() {
            return Err(expected_statement(input));
        }
        let feature_names = self
            .language_features
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(",");
        let detail = if feature_names.is_empty() {
            requested.to_string()
        } else {
            format!("{requested};features={feature_names}")
        };
        trace.record(TraceEvent {
            event: "language.context.selected",
            rule: "TOPAL-SYN-CONTEXT-001",
            detail: &detail,
        });
        Ok(execution)
    }

    /// Evaluate one expression against an immutable binding snapshot.
    ///
    /// # Errors
    ///
    /// Returns a diagnostic when the input is not exactly one expression or
    /// when that expression cannot be evaluated.
    pub fn inspect(
        bindings: &BTreeMap<String, Value>,
        input: &str,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let mut session = Self {
            bindings: bindings.clone(),
            functions: Box::new(BTreeMap::new()),
            generators: Box::new(BTreeMap::new()),
            root_namespace: None,
            defining_context: None,
            declared_names: bindings.keys().cloned().collect(),
            published_names: BTreeSet::new(),
            documentation: Box::new(BTreeMap::new()),
            language_version: LanguageVersion::DESIGN_0,
            language_features: BTreeSet::new(),
            declared_libraries: BTreeSet::new(),
            consumed_names: BTreeSet::new(),
            local_function_names: BTreeSet::new(),
            enum_types: BTreeMap::new(),
            union_types: Box::new(BTreeMap::new()),
            generic_types: BTreeMap::new(),
            call_stack: Vec::new(),
            static_context: false,
            task_state: None,
            next_task_identity: Cell::new(0),
            next_transaction_identity: Cell::new(0),
        };
        let mut execution = session.prepare(input, trace)?;
        if !matches!(execution.statements.as_slice(), [Statement::Expression(_)]) {
            let span = execution
                .statements
                .first()
                .map_or_else(|| Span::new(0, 0), statement_span);
            return Err(diagnostic(
                &execution.source,
                "D-EXPECTED-EXPRESSION",
                span,
                "debugger inspection requires exactly one expression",
            ));
        }
        match execution.step(&mut session, trace)? {
            ExecutionStep::Complete(value) => Ok(value),
            ExecutionStep::Advanced { .. } => unreachable!("one expression completes execution"),
            ExecutionStep::Returned { .. } => unreachable!("inspection rejects return statements"),
        }
    }

    fn checkpoint(&self, trace: &mut impl TraceSink, value: Option<&Value>, span: Option<Span>) {
        trace.checkpoint(ExecutionSnapshot {
            bindings: &self.bindings,
            value,
            span,
        });
    }

    fn evaluate_returning_operator_operand_step(
        &self,
        source: &SourceText,
        expression: &Expression,
        return_classifier: Option<&str>,
        trace: &mut impl TraceSink,
    ) -> Result<Option<ExecutionStep>, Diagnostic> {
        let Expression::Application { items, .. } = expression else {
            return Ok(None);
        };
        let (returning, preceding) = if items.len() >= 2
            && direct_expression_returns_from_function(&items[0])
            && matches!(items[1], Expression::Callable { .. })
        {
            (&items[0], None)
        } else {
            let Some(returning_index) = (2..items.len()).find(|index| {
                matches!(items[index - 1], Expression::Callable { .. })
                    && direct_expression_returns_from_function(&items[*index])
            }) else {
                return Ok(None);
            };
            (&items[returning_index], Some(&items[..returning_index - 1]))
        };
        if let Some(preceding) = preceding {
            if let [preceding] = preceding {
                let _ = self.evaluate_expression(source, preceding, trace)?;
            } else {
                let preceding = Expression::Application {
                    span: Span::new(
                        preceding[0].span().start,
                        preceding.last().expect("nonempty prefix").span().end,
                    ),
                    items: preceding.to_vec(),
                };
                let _ = self.evaluate_expression(source, &preceding, trace)?;
            }
        }
        let Expression::Block { statements, .. } = returning else {
            unreachable!("a direct returning operator operand is a lexical block")
        };
        let step = self.evaluate_block_step(
            source,
            statements,
            return_classifier,
            return_classifier,
            trace,
        )?;
        assert!(
            matches!(step, ExecutionStep::Returned { .. }),
            "a direct returning operator operand exits its function"
        );
        Ok(Some(step))
    }

    fn evaluate_returning_modular_reduction_operand_step(
        &self,
        source: &SourceText,
        expression: &Expression,
        return_classifier: Option<&str>,
        trace: &mut impl TraceSink,
    ) -> Result<Option<ExecutionStep>, Diagnostic> {
        let Expression::Application { items, .. } = expression else {
            return Ok(None);
        };
        let [
            operand,
            Expression::Identifier(operation),
            Expression::Identifier(type_name),
        ] = items.as_slice()
        else {
            return Ok(None);
        };
        if source.slice(*operation) != "modulo"
            || !direct_expression_returns_from_function(operand)
            || !matches!(
                self.bindings.get(source.slice(*type_name)),
                Some(Value::ModularType(_))
            )
        {
            return Ok(None);
        }
        let Expression::Block { statements, .. } = operand else {
            unreachable!("a direct returning modular reduction operand is a lexical block")
        };
        let step = self.evaluate_block_step(
            source,
            statements,
            return_classifier,
            return_classifier,
            trace,
        )?;
        assert!(
            matches!(step, ExecutionStep::Returned { .. }),
            "a direct returning modular reduction operand exits its function"
        );
        Ok(Some(step))
    }

    fn evaluate_returning_collection_source_step(
        &self,
        source: &SourceText,
        expression: &Expression,
        return_classifier: Option<&str>,
        trace: &mut impl TraceSink,
    ) -> Result<Option<ExecutionStep>, Diagnostic> {
        let Expression::Application { items, .. } = expression else {
            return Ok(None);
        };
        let (collection, map_policy) = match items.as_slice() {
            [Expression::Identifier(operation), collection]
                if matches!(
                    source.slice(*operation),
                    "collect" | "collect-set" | "collect-bag"
                ) =>
            {
                (collection, None)
            }
            [
                collection,
                Expression::Identifier(operation),
                Expression::Identifier(target),
            ] if source.slice(*operation) == "collect"
                && matches!(source.slice(*target), "Array" | "String") =>
            {
                (collection, None)
            }
            [
                Expression::Identifier(operation),
                collection,
                Expression::Identifier(resolving),
                Expression::Identifier(policy),
            ] if source.slice(*operation) == "collect-map"
                && source.slice(*resolving) == "resolving" =>
            {
                (collection, Some(*policy))
            }
            _ => return Ok(None),
        };
        if !direct_expression_returns_from_function(collection) {
            return Ok(None);
        }
        if let Some(policy) = map_policy
            && !matches!(source.slice(policy), "reject" | "keep-first" | "keep-last")
        {
            return Err(diagnostic(
                source,
                "E-MAP-COLLISION-POLICY",
                policy,
                "collect-map policy must be reject, keep-first, or keep-last",
            ));
        }
        let Expression::Block { statements, .. } = collection else {
            unreachable!("a direct returning collection source is a lexical block")
        };
        let step = self.evaluate_block_step(
            source,
            statements,
            return_classifier,
            return_classifier,
            trace,
        )?;
        assert!(
            matches!(step, ExecutionStep::Returned { .. }),
            "a direct returning collection source exits its function"
        );
        Ok(Some(step))
    }

    #[allow(clippy::too_many_lines)] // Exit-shape routing stays explicit and source-ordered.
    fn evaluate_returning_embedded_expression_step(
        &self,
        source: &SourceText,
        expression: &Expression,
        return_classifier: Option<&str>,
        trace: &mut impl TraceSink,
    ) -> Result<Option<ExecutionStep>, Diagnostic> {
        if let Some(step) = self.evaluate_returning_decision_subject_step(
            source,
            expression,
            return_classifier,
            trace,
        )? {
            return Ok(Some(step));
        }
        if let Some(step) = self.evaluate_returning_boolean_decision_action_step(
            source,
            expression,
            return_classifier,
            trace,
        )? {
            return Ok(Some(step));
        }
        if let Some(step) = self.evaluate_returning_enum_fallback_decision_action_step(
            source,
            expression,
            return_classifier,
            trace,
        )? {
            return Ok(Some(step));
        }
        if let Some(step) = self.evaluate_returning_exhaustive_enum_decision_action_step(
            source,
            expression,
            return_classifier,
            trace,
        )? {
            return Ok(Some(step));
        }
        if let Some(step) = self.evaluate_returning_optional_decision_action_step(
            source,
            expression,
            return_classifier,
            trace,
        )? {
            return Ok(Some(step));
        }
        if let Some(step) = self.evaluate_returning_result_decision_action_step(
            source,
            expression,
            return_classifier,
            trace,
        )? {
            return Ok(Some(step));
        }
        if let Some(step) = self.evaluate_returning_error_code_decision_action_step(
            source,
            expression,
            return_classifier,
            trace,
        )? {
            return Ok(Some(step));
        }
        if let Some(step) = self.evaluate_returning_list_decision_action_step(
            source,
            expression,
            return_classifier,
            trace,
        )? {
            return Ok(Some(step));
        }
        if let Some(step) = self.evaluate_returning_comparison_value_decision_action_step(
            source,
            expression,
            return_classifier,
            trace,
        )? {
            return Ok(Some(step));
        }
        if let Some(step) = self.evaluate_returning_ordered_comparison_decision_action_step(
            source,
            expression,
            return_classifier,
            trace,
        )? {
            return Ok(Some(step));
        }
        if let Some(step) = self.evaluate_returning_product_field_step(
            source,
            expression,
            return_classifier,
            trace,
        )? {
            return Ok(Some(step));
        }
        if let Some(step) = self.evaluate_returning_collection_source_step(
            source,
            expression,
            return_classifier,
            trace,
        )? {
            return Ok(Some(step));
        }
        if let Some(step) = self.evaluate_returning_modular_reduction_operand_step(
            source,
            expression,
            return_classifier,
            trace,
        )? {
            return Ok(Some(step));
        }
        if let Some(step) = self.evaluate_returning_operator_operand_step(
            source,
            expression,
            return_classifier,
            trace,
        )? {
            return Ok(Some(step));
        }
        if let Some(step) = self.evaluate_returning_unary_constructor_argument_step(
            source,
            expression,
            return_classifier,
            trace,
        )? {
            return Ok(Some(step));
        }
        if let Some(step) = self.evaluate_returning_variant_constructor_argument_step(
            source,
            expression,
            return_classifier,
            trace,
        )? {
            return Ok(Some(step));
        }
        self.evaluate_returning_named_call_argument_step(
            source,
            expression,
            return_classifier,
            trace,
        )
    }

    fn evaluate_returning_decision_subject_step(
        &self,
        source: &SourceText,
        expression: &Expression,
        return_classifier: Option<&str>,
        trace: &mut impl TraceSink,
    ) -> Result<Option<ExecutionStep>, Diagnostic> {
        let Expression::DecisionTable { subject, .. } = expression else {
            return Ok(None);
        };
        if !direct_expression_returns_from_function(subject) {
            return Ok(None);
        }
        let Expression::Block { statements, .. } = subject.as_ref() else {
            unreachable!("a direct returning decision subject is a lexical block")
        };
        let step = self.evaluate_block_step(
            source,
            statements,
            return_classifier,
            return_classifier,
            trace,
        )?;
        assert!(
            matches!(step, ExecutionStep::Returned { .. }),
            "a direct returning decision subject exits its function"
        );
        Ok(Some(step))
    }

    fn evaluate_returning_boolean_decision_action_step(
        &self,
        source: &SourceText,
        expression: &Expression,
        return_classifier: Option<&str>,
        trace: &mut impl TraceSink,
    ) -> Result<Option<ExecutionStep>, Diagnostic> {
        let Expression::DecisionTable { subject, rules, .. } = expression else {
            return Ok(None);
        };
        if !is_supported_returning_boolean_action_shape(rules) {
            return Ok(None);
        }
        let subject_span = subject.span();
        let subject = self.evaluate_expression(source, subject, trace)?;
        let (subject, decision_rule) = match subject {
            Value::Boolean(subject) => (subject, "TOPAL-DECISION-BOOLEAN-001"),
            Value::Optional { .. } if matches!(rules.as_slice(), [rule] if matches!(rule.matcher, DecisionMatcher::Otherwise(_))) => {
                (false, "TOPAL-DECISION-OPTIONAL-001")
            }
            _ => {
                return Err(diagnostic(
                    source,
                    "E-DECISION-SUBJECT-TYPE",
                    subject_span,
                    "Boolean literal matchers require a Boolean subject",
                ));
            }
        };
        let mut selected = None;
        for (index, rule) in rules.iter().enumerate() {
            let matches = match rule.matcher {
                DecisionMatcher::Boolean { value, .. } => value == subject,
                DecisionMatcher::Otherwise(_) => true,
                _ => unreachable!("preselected complete Boolean decision shape"),
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
        let (index, selected) = selected.expect("a complete Boolean decision selects an action");
        let detail = format!("rule={index}");
        trace.record(TraceEvent {
            event: "decision.rule.selected",
            rule: decision_rule,
            detail: &detail,
        });
        let Expression::Block { statements, .. } = &selected.action else {
            unreachable!("a direct returning Boolean action is a lexical block")
        };
        let step = self.evaluate_block_step(
            source,
            statements,
            return_classifier,
            return_classifier,
            trace,
        )?;
        assert!(
            matches!(step, ExecutionStep::Returned { .. }),
            "a selected returning Boolean action exits its function"
        );
        Ok(Some(step))
    }

    fn evaluate_returning_comparison_value_decision_action_step(
        &self,
        source: &SourceText,
        expression: &Expression,
        return_classifier: Option<&str>,
        trace: &mut impl TraceSink,
    ) -> Result<Option<ExecutionStep>, Diagnostic> {
        let Expression::DecisionTable { subject, rules, .. } = expression else {
            return Ok(None);
        };
        if !is_supported_returning_comparison_value_action_shape(source, rules) {
            return Ok(None);
        }
        let subject_span = subject.span();
        let Value::Enum {
            type_name,
            alternative,
        } = self.evaluate_expression(source, subject, trace)?
        else {
            return Err(diagnostic(
                source,
                "E-DECISION-SUBJECT-TYPE",
                subject_span,
                "Comparison alternative matchers require a Comparison subject",
            ));
        };
        if type_name != "Comparison" {
            return Err(diagnostic(
                source,
                "E-DECISION-SUBJECT-TYPE",
                subject_span,
                "Comparison alternative matchers require a Comparison subject",
            ));
        }
        let mut selected = None;
        for (index, rule) in rules.iter().enumerate() {
            let matches = match rule.matcher {
                DecisionMatcher::Identifier(matcher) => source.slice(matcher) == alternative,
                DecisionMatcher::Otherwise(_) => true,
                _ => unreachable!("preselected complete Comparison decision shape"),
            };
            let detail = format!("rule={index};matched={matches}");
            trace.record(TraceEvent {
                event: "decision.rule.considered",
                rule: "TOPAL-DECISION-ENUM-001",
                detail: &detail,
            });
            if matches {
                selected = Some((index, rule));
                break;
            }
        }
        let (index, selected) = selected.expect("a complete Comparison decision selects an action");
        let detail = format!("rule={index}");
        trace.record(TraceEvent {
            event: "decision.rule.selected",
            rule: "TOPAL-DECISION-ENUM-001",
            detail: &detail,
        });
        let Expression::Block { statements, .. } = &selected.action else {
            unreachable!("a direct returning Comparison action is a lexical block")
        };
        let step = self.evaluate_block_step(
            source,
            statements,
            return_classifier,
            return_classifier,
            trace,
        )?;
        assert!(
            matches!(step, ExecutionStep::Returned { .. }),
            "a selected returning Comparison action exits its function"
        );
        Ok(Some(step))
    }

    fn evaluate_returning_enum_fallback_decision_action_step(
        &self,
        source: &SourceText,
        expression: &Expression,
        return_classifier: Option<&str>,
        trace: &mut impl TraceSink,
    ) -> Result<Option<ExecutionStep>, Diagnostic> {
        let Expression::DecisionTable { subject, rules, .. } = expression else {
            return Ok(None);
        };
        if !is_supported_returning_enum_fallback_action_shape(source, rules) {
            return Ok(None);
        }
        let subject_span = subject.span();
        let subject = self.evaluate_expression(source, subject, trace)?;
        let Value::Enum {
            type_name,
            alternative,
        } = &subject
        else {
            return Err(diagnostic(
                source,
                "E-DECISION-SUBJECT-TYPE",
                subject_span,
                "enum alternative matchers require an Enum subject",
            ));
        };
        let mut selected = None;
        for (index, rule) in rules.iter().enumerate() {
            let matches = match rule.matcher {
                DecisionMatcher::Identifier(matcher) => {
                    let name = source.slice(matcher);
                    if type_name == "Comparison" {
                        if !matches!(name, "Less" | "Equal" | "Greater") {
                            return Err(diagnostic(
                                source,
                                "E-UNBOUND-NAME",
                                matcher,
                                format!("enum matcher `{name}` is not declared"),
                            ));
                        }
                        alternative == name
                    } else {
                        let Some(candidate) = self.bindings.get(name).cloned() else {
                            return Err(diagnostic(
                                source,
                                "E-UNBOUND-NAME",
                                matcher,
                                format!("enum matcher `{name}` is not declared"),
                            ));
                        };
                        values_equal(subject.clone(), candidate, trace).unwrap_or(false)
                    }
                }
                DecisionMatcher::Otherwise(_) => true,
                _ => unreachable!("preselected final-fallback Enum decision shape"),
            };
            let detail = format!("rule={index};matched={matches}");
            trace.record(TraceEvent {
                event: "decision.rule.considered",
                rule: "TOPAL-DECISION-ENUM-001",
                detail: &detail,
            });
            if matches {
                selected = Some((index, rule));
                break;
            }
        }
        let (index, selected) = selected.expect("a final-fallback Enum decision selects an action");
        let detail = format!("rule={index}");
        trace.record(TraceEvent {
            event: "decision.rule.selected",
            rule: "TOPAL-DECISION-ENUM-001",
            detail: &detail,
        });
        let Expression::Block { statements, .. } = &selected.action else {
            unreachable!("a direct returning Enum action is a lexical block")
        };
        let step = self.evaluate_block_step(
            source,
            statements,
            return_classifier,
            return_classifier,
            trace,
        )?;
        assert!(
            matches!(step, ExecutionStep::Returned { .. }),
            "a selected returning Enum action exits its function"
        );
        Ok(Some(step))
    }

    fn evaluate_returning_exhaustive_enum_decision_action_step(
        &self,
        source: &SourceText,
        expression: &Expression,
        return_classifier: Option<&str>,
        trace: &mut impl TraceSink,
    ) -> Result<Option<ExecutionStep>, Diagnostic> {
        let Expression::DecisionTable {
            subject,
            rules,
            span,
        } = expression
        else {
            return Ok(None);
        };
        if !is_supported_returning_exhaustive_enum_action_shape(source, rules) {
            return Ok(None);
        }
        let subject_span = subject.span();
        let subject = self.evaluate_expression(source, subject, trace)?;
        let Value::Enum {
            type_name,
            alternative,
        } = &subject
        else {
            return Err(diagnostic(
                source,
                "E-DECISION-SUBJECT-TYPE",
                subject_span,
                "enum alternative matchers require an Enum subject",
            ));
        };
        let alternatives = rules
            .iter()
            .filter_map(|rule| match rule.matcher {
                DecisionMatcher::Identifier(matcher) => Some(source.slice(matcher).to_owned()),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        if known_enum_alternatives(self, type_name).as_ref() != Some(&alternatives) {
            return Err(diagnostic(
                source,
                "E-INCOMPLETE-DECISION",
                *span,
                format!("decision does not cover every `{type_name}` alternative"),
            ));
        }
        let (index, selected) = rules
            .iter()
            .enumerate()
            .find(|(_, rule)| {
                matches!(rule.matcher, DecisionMatcher::Identifier(matcher) if source.slice(matcher) == alternative)
            })
            .expect("an exhaustive Enum decision selects an action");
        for considered in 0..=index {
            let detail = format!("rule={considered};matched={}", considered == index);
            trace.record(TraceEvent {
                event: "decision.rule.considered",
                rule: "TOPAL-DECISION-ENUM-001",
                detail: &detail,
            });
        }
        let detail = format!("rule={index}");
        trace.record(TraceEvent {
            event: "decision.rule.selected",
            rule: "TOPAL-DECISION-ENUM-001",
            detail: &detail,
        });
        let Expression::Block { statements, .. } = &selected.action else {
            unreachable!("a direct returning Enum action is a lexical block")
        };
        let step = self.evaluate_block_step(
            source,
            statements,
            return_classifier,
            return_classifier,
            trace,
        )?;
        assert!(
            matches!(step, ExecutionStep::Returned { .. }),
            "a selected returning Enum action exits its function"
        );
        Ok(Some(step))
    }

    fn evaluate_returning_optional_decision_action_step(
        &self,
        source: &SourceText,
        expression: &Expression,
        return_classifier: Option<&str>,
        trace: &mut impl TraceSink,
    ) -> Result<Option<ExecutionStep>, Diagnostic> {
        let Expression::DecisionTable { subject, rules, .. } = expression else {
            return Ok(None);
        };
        if !is_supported_returning_optional_action_shape(rules) {
            return Ok(None);
        }
        let subject_span = subject.span();
        let Value::Optional { payload, .. } = self.evaluate_expression(source, subject, trace)?
        else {
            return Err(diagnostic(
                source,
                "E-DECISION-SUBJECT-TYPE",
                subject_span,
                "Optional matchers require an Optional subject",
            ));
        };
        let is_some = payload.is_some();
        let mut selected = None;
        for (index, rule) in rules.iter().enumerate() {
            let matches = match rule.matcher {
                DecisionMatcher::Optional { some, .. } => some == is_some,
                DecisionMatcher::Otherwise(_) => true,
                _ => unreachable!("preselected complete Optional decision shape"),
            };
            let detail = format!("rule={index};matched={matches}");
            trace.record(TraceEvent {
                event: "decision.rule.considered",
                rule: "TOPAL-DECISION-OPTIONAL-001",
                detail: &detail,
            });
            if matches {
                selected = Some((index, rule));
                break;
            }
        }
        let (index, selected) = selected.expect("a complete Optional decision selects an action");
        let detail = format!("rule={index}");
        trace.record(TraceEvent {
            event: "decision.rule.selected",
            rule: "TOPAL-DECISION-OPTIONAL-001",
            detail: &detail,
        });
        let Expression::Block { statements, .. } = &selected.action else {
            unreachable!("a direct returning Optional action is a lexical block")
        };
        let step = if let DecisionMatcher::Optional {
            binding: Some(binding),
            ..
        } = selected.matcher
        {
            let name = source.slice(binding);
            let mut branch = self.clone();
            branch.bindings.insert(
                name.to_owned(),
                *payload.expect("a Some matcher selects a present Optional payload"),
            );
            trace.record(TraceEvent {
                event: "optional.payload.bound",
                rule: "TOPAL-DECISION-OPTIONAL-001",
                detail: name,
            });
            branch.evaluate_block_step(
                source,
                statements,
                return_classifier,
                return_classifier,
                trace,
            )?
        } else {
            self.evaluate_block_step(
                source,
                statements,
                return_classifier,
                return_classifier,
                trace,
            )?
        };
        assert!(
            matches!(step, ExecutionStep::Returned { .. }),
            "a selected returning Optional action exits its function"
        );
        Ok(Some(step))
    }

    fn evaluate_returning_result_decision_action_step(
        &self,
        source: &SourceText,
        expression: &Expression,
        return_classifier: Option<&str>,
        trace: &mut impl TraceSink,
    ) -> Result<Option<ExecutionStep>, Diagnostic> {
        let Expression::DecisionTable { subject, rules, .. } = expression else {
            return Ok(None);
        };
        if !is_supported_returning_result_action_shape(rules) {
            return Ok(None);
        }
        let subject = self.evaluate_expression(source, subject, trace)?;
        let is_error = matches!(subject, Value::Error { .. });
        let (index, selected) = rules
            .iter()
            .enumerate()
            .find(|(_, rule)| {
                matches!(rule.matcher, DecisionMatcher::Result { error, .. } if error == is_error)
            })
            .expect("a complete Result decision selects an action");
        for considered in 0..=index {
            let detail = format!("rule={considered};matched={}", considered == index);
            trace.record(TraceEvent {
                event: "decision.rule.considered",
                rule: "TOPAL-DECISION-RESULT-001",
                detail: &detail,
            });
        }
        let detail = format!("rule={index}");
        trace.record(TraceEvent {
            event: "decision.rule.selected",
            rule: "TOPAL-DECISION-RESULT-001",
            detail: &detail,
        });
        let DecisionMatcher::Result { binding, .. } = selected.matcher else {
            unreachable!("preselected complete Result decision shape")
        };
        let Expression::Block { statements, .. } = &selected.action else {
            unreachable!("a direct returning Result action is a lexical block")
        };
        let name = source.slice(binding);
        let mut branch = self.clone();
        branch.bindings.insert(name.to_owned(), subject);
        trace.record(TraceEvent {
            event: "result.payload.bound",
            rule: "TOPAL-DECISION-RESULT-001",
            detail: name,
        });
        let step = branch.evaluate_block_step(
            source,
            statements,
            return_classifier,
            return_classifier,
            trace,
        )?;
        assert!(
            matches!(step, ExecutionStep::Returned { .. }),
            "a selected returning Result action exits its function"
        );
        Ok(Some(step))
    }

    fn evaluate_returning_error_code_decision_action_step(
        &self,
        source: &SourceText,
        expression: &Expression,
        return_classifier: Option<&str>,
        trace: &mut impl TraceSink,
    ) -> Result<Option<ExecutionStep>, Diagnostic> {
        let Expression::DecisionTable { subject, rules, .. } = expression else {
            return Ok(None);
        };
        if !is_supported_returning_error_code_action_shape(rules) {
            return Ok(None);
        }
        let subject = self.evaluate_expression(source, subject, trace)?;
        let is_error = matches!(subject, Value::Error { .. });
        let mut selected = None;
        for (index, rule) in rules.iter().enumerate() {
            let matches = match rule.matcher {
                DecisionMatcher::Result { error, .. } => error == is_error,
                DecisionMatcher::ErrorCode {
                    namespace,
                    vocabulary,
                    code,
                    ..
                } => {
                    let namespace = source.slice(namespace);
                    let vocabulary = source.slice(vocabulary);
                    let code_span = code;
                    let code = source.slice(code_span);
                    let known = namespace == "lang"
                        && vocabulary == "arithmetic"
                        && is_arithmetic_error_code(code);
                    if !known {
                        return Err(diagnostic(
                            source,
                            "E-UNKNOWN-ERROR-CODE",
                            code_span,
                            "the Error-code pattern requires a code published by the qualified arithmetic namespace",
                        ));
                    }
                    matches!(&subject, Value::Error { code: subject_code, .. } if subject_code == code)
                }
                _ => unreachable!("preselected complete qualified Error-code decision shape"),
            };
            let detail = format!("rule={index};matched={matches}");
            trace.record(TraceEvent {
                event: "decision.rule.considered",
                rule: "TOPAL-DECISION-RESULT-001",
                detail: &detail,
            });
            if matches {
                selected = Some((index, rule));
                break;
            }
        }
        let (index, selected) = selected.expect("a complete Error-code decision selects an action");
        let detail = format!("rule={index}");
        trace.record(TraceEvent {
            event: "decision.rule.selected",
            rule: "TOPAL-DECISION-RESULT-001",
            detail: &detail,
        });
        if let DecisionMatcher::ErrorCode { code, .. } = selected.matcher {
            trace.record(TraceEvent {
                event: "error.code.matched",
                rule: "TOPAL-DECISION-ERROR-CODE-001",
                detail: source.slice(code),
            });
        }
        let Expression::Block { statements, .. } = &selected.action else {
            unreachable!("a direct returning Error-code action is a lexical block")
        };
        let mut branch = self.clone();
        if let DecisionMatcher::Result { binding, .. } = selected.matcher {
            let name = source.slice(binding);
            branch.bindings.insert(name.to_owned(), subject);
            trace.record(TraceEvent {
                event: "result.payload.bound",
                rule: "TOPAL-DECISION-RESULT-001",
                detail: name,
            });
        }
        let step = branch.evaluate_block_step(
            source,
            statements,
            return_classifier,
            return_classifier,
            trace,
        )?;
        assert!(
            matches!(step, ExecutionStep::Returned { .. }),
            "a selected returning Error-code action exits its function"
        );
        Ok(Some(step))
    }

    fn evaluate_returning_list_decision_action_step(
        &self,
        source: &SourceText,
        expression: &Expression,
        return_classifier: Option<&str>,
        trace: &mut impl TraceSink,
    ) -> Result<Option<ExecutionStep>, Diagnostic> {
        let Expression::DecisionTable { subject, rules, .. } = expression else {
            return Ok(None);
        };
        if !is_supported_returning_list_action_shape(source, rules) {
            return Ok(None);
        }
        let subject_span = subject.span();
        let Value::List {
            element_classifier,
            mut entries,
        } = self.evaluate_expression(source, subject, trace)?
        else {
            return Err(diagnostic(
                source,
                "E-DECISION-SUBJECT-TYPE",
                subject_span,
                "list matchers require a List subject",
            ));
        };
        let is_empty = entries.is_empty();
        let (index, selected) = rules
            .iter()
            .enumerate()
            .find(|(_, rule)| {
                matches!(rule.matcher, DecisionMatcher::ListEmpty(_) if is_empty)
                    || matches!(rule.matcher, DecisionMatcher::ListEntry { .. } if !is_empty)
            })
            .expect("a complete List decision selects an action");
        for considered in 0..=index {
            let detail = format!("rule={considered};matched={}", considered == index);
            trace.record(TraceEvent {
                event: "decision.rule.considered",
                rule: "TOPAL-DECISION-LIST-001",
                detail: &detail,
            });
        }
        let detail = format!("rule={index}");
        trace.record(TraceEvent {
            event: "decision.rule.selected",
            rule: "TOPAL-DECISION-LIST-001",
            detail: &detail,
        });
        let Expression::Block { statements, .. } = &selected.action else {
            unreachable!("a direct returning List action is a lexical block")
        };
        let step = if let DecisionMatcher::ListEntry { first, rest, .. } = selected.matcher {
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
            branch.evaluate_block_step(
                source,
                statements,
                return_classifier,
                return_classifier,
                trace,
            )?
        } else {
            self.evaluate_block_step(
                source,
                statements,
                return_classifier,
                return_classifier,
                trace,
            )?
        };
        assert!(
            matches!(step, ExecutionStep::Returned { .. }),
            "a selected returning List action exits its function"
        );
        Ok(Some(step))
    }

    fn evaluate_returning_ordered_comparison_decision_action_step(
        &self,
        source: &SourceText,
        expression: &Expression,
        return_classifier: Option<&str>,
        trace: &mut impl TraceSink,
    ) -> Result<Option<ExecutionStep>, Diagnostic> {
        let Expression::DecisionTable { subject, rules, .. } = expression else {
            return Ok(None);
        };
        if !is_supported_returning_ordered_comparison_action_shape(rules) {
            return Ok(None);
        }
        let subject_span = subject.span();
        let subject = self.evaluate_expression(source, subject, trace)?;
        let mut selected = None;
        for (index, rule) in rules.iter().enumerate() {
            let matches = match &rule.matcher {
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
                _ => unreachable!("preselected complete ordered comparison decision shape"),
            };
            let detail = format!("rule={index};matched={matches}");
            trace.record(TraceEvent {
                event: "decision.rule.considered",
                rule: "TOPAL-DECISION-COMPARISON-001",
                detail: &detail,
            });
            if matches {
                selected = Some((index, rule));
                break;
            }
        }
        let (index, selected) =
            selected.expect("a complete ordered comparison decision selects an action");
        let detail = format!("rule={index}");
        trace.record(TraceEvent {
            event: "decision.rule.selected",
            rule: "TOPAL-DECISION-COMPARISON-001",
            detail: &detail,
        });
        let Expression::Block { statements, .. } = &selected.action else {
            unreachable!("a direct returning comparison action is a lexical block")
        };
        let step = self.evaluate_block_step(
            source,
            statements,
            return_classifier,
            return_classifier,
            trace,
        )?;
        assert!(
            matches!(step, ExecutionStep::Returned { .. }),
            "a selected returning comparison action exits its function"
        );
        Ok(Some(step))
    }

    fn evaluate_returning_product_field_step(
        &self,
        source: &SourceText,
        expression: &Expression,
        return_classifier: Option<&str>,
        trace: &mut impl TraceSink,
    ) -> Result<Option<ExecutionStep>, Diagnostic> {
        let Expression::Product { fields, span } = expression else {
            return Ok(None);
        };
        let Some(returning_index) = fields
            .iter()
            .position(|field| direct_expression_returns_from_function(&field.value))
        else {
            return Ok(None);
        };
        let labeled = fields.iter().filter(|field| field.label.is_some()).count();
        if labeled != 0 && labeled != fields.len() {
            return Err(diagnostic(
                source,
                "E-MIXED-PRODUCT-FIELDS",
                *span,
                "a product cannot mix positional and labeled fields",
            ));
        }
        let mut labels = BTreeSet::new();
        for (index, field) in fields[..=returning_index].iter().enumerate() {
            if let Some(label_span) = field.label {
                let label = source.slice(label_span);
                if !labels.insert(label) {
                    return Err(diagnostic(
                        source,
                        "E-DUPLICATE-RECORD-FIELD",
                        label_span,
                        "record field label occurs more than once",
                    ));
                }
            }
            if index == returning_index {
                break;
            }
            let _ = self.evaluate_expression(source, &field.value, trace)?;
        }
        let Expression::Block { statements, .. } = &fields[returning_index].value else {
            unreachable!("a direct returning product field is a lexical block")
        };
        let step = self.evaluate_block_step(
            source,
            statements,
            return_classifier,
            return_classifier,
            trace,
        )?;
        assert!(
            matches!(step, ExecutionStep::Returned { .. }),
            "a direct returning product field exits its function"
        );
        Ok(Some(step))
    }

    fn evaluate_returning_named_call_argument_step(
        &self,
        source: &SourceText,
        expression: &Expression,
        return_classifier: Option<&str>,
        trace: &mut impl TraceSink,
    ) -> Result<Option<ExecutionStep>, Diagnostic> {
        let Expression::Application { items, .. } = expression else {
            return Ok(None);
        };
        let Some((function_index, function_name)) =
            items.iter().enumerate().rev().find_map(|(index, item)| {
                let Expression::Identifier(name) = item else {
                    return None;
                };
                let name = source.slice(*name);
                let unshadowed = !self.bindings.contains_key(name)
                    || matches!(
                        self.bindings.get(name),
                        Some(Value::NamedFunction(function)) if function.name == name
                    );
                (unshadowed && self.functions.contains_key(name)).then_some((index, name))
            })
        else {
            return Ok(None);
        };
        let Some((argument_count, returning_index, preceding_source)) =
            (if function_index == 0 && items.len() == 2 {
                direct_expression_returns_from_function(&items[1]).then_some((1, 1, None))
            } else if function_index >= 1 && function_index + 2 == items.len() {
                if function_index == 1 && direct_expression_returns_from_function(&items[0]) {
                    Some((2, 0, None))
                } else if direct_expression_returns_from_function(&items[function_index + 1]) {
                    Some((2, function_index + 1, Some(&items[..function_index])))
                } else {
                    None
                }
            } else {
                None
            })
        else {
            return Ok(None);
        };
        let admitted_declaration = self
            .functions
            .get(function_name)
            .is_some_and(|declarations| {
                matches!(declarations.as_slice(), [declaration]
                if declaration.parameters.len() == argument_count
                    && declaration.parameter_packages.is_empty()
                    && (!self.static_context || declaration.is_static))
            });
        if !admitted_declaration {
            return Ok(None);
        }
        if let Some(preceding) = preceding_source {
            if let [preceding] = preceding {
                let _ = self.evaluate_expression(source, preceding, trace)?;
            } else {
                let preceding = Expression::Application {
                    span: Span::new(
                        preceding[0].span().start,
                        preceding.last().expect("nonempty call prefix").span().end,
                    ),
                    items: preceding.to_vec(),
                };
                let _ = self.evaluate_expression(source, &preceding, trace)?;
            }
        }
        let Expression::Block { statements, .. } = &items[returning_index] else {
            unreachable!("a direct returning named-call argument is a lexical block")
        };
        let step = self.evaluate_block_step(
            source,
            statements,
            return_classifier,
            return_classifier,
            trace,
        )?;
        assert!(
            matches!(step, ExecutionStep::Returned { .. }),
            "a direct returning named-call argument exits its function"
        );
        Ok(Some(step))
    }

    fn evaluate_returning_unary_constructor_argument_step(
        &self,
        source: &SourceText,
        expression: &Expression,
        return_classifier: Option<&str>,
        trace: &mut impl TraceSink,
    ) -> Result<Option<ExecutionStep>, Diagnostic> {
        let Expression::Application { items, .. } = expression else {
            return Ok(None);
        };
        let [Expression::Identifier(constructor), argument] = items.as_slice() else {
            return Ok(None);
        };
        let constructor_name = source.slice(*constructor);
        let built_in = matches!(
            constructor_name,
            "Some" | "String" | "Character" | "Int" | "Nat" | "Rational"
        );
        let declared_union = self.union_constructor(constructor_name).is_some();
        let declared_constraint = matches!(
            self.bindings.get(constructor_name),
            Some(Value::Constraint(constraint))
                if constraint.base_classifier == "Int"
        );
        let declared_modular = matches!(
            self.bindings.get(constructor_name),
            Some(Value::ModularType(_))
        );
        if (!built_in && !declared_union && !declared_constraint && !declared_modular)
            || !direct_expression_returns_from_function(argument)
        {
            return Ok(None);
        }
        let Expression::Block { statements, .. } = argument else {
            unreachable!("a direct returning unary constructor argument is a lexical block")
        };
        let step = self.evaluate_block_step(
            source,
            statements,
            return_classifier,
            return_classifier,
            trace,
        )?;
        assert!(
            matches!(step, ExecutionStep::Returned { .. }),
            "a direct returning unary constructor argument exits its function"
        );
        Ok(Some(step))
    }

    fn evaluate_returning_variant_constructor_argument_step(
        &self,
        source: &SourceText,
        expression: &Expression,
        return_classifier: Option<&str>,
        trace: &mut impl TraceSink,
    ) -> Result<Option<ExecutionStep>, Diagnostic> {
        let Expression::Application { items, .. } = expression else {
            return Ok(None);
        };
        let [
            Expression::Identifier(type_name),
            Expression::Identifier(at),
            Expression::Integer(index),
            argument,
        ] = items.as_slice()
        else {
            return Ok(None);
        };
        let key = format!("at {}", source.slice(*index));
        let admitted = source.slice(*at) == "at"
            && self
                .union_types
                .get(source.slice(*type_name))
                .and_then(|alternatives| alternatives.get(&key))
                .and_then(Option::as_deref)
                .is_some();
        if !admitted || !direct_expression_returns_from_function(argument) {
            return Ok(None);
        }
        let Expression::Block { statements, .. } = argument else {
            unreachable!("a direct returning Variant constructor argument is a lexical block")
        };
        let step = self.evaluate_block_step(
            source,
            statements,
            return_classifier,
            return_classifier,
            trace,
        )?;
        assert!(
            matches!(step, ExecutionStep::Returned { .. }),
            "a direct returning Variant constructor argument exits its function"
        );
        Ok(Some(step))
    }
}
