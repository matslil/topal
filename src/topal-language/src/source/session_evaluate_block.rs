impl Session {
    #[inline(never)]
    fn evaluate_block(
        &self,
        source: &SourceText,
        statements: &[Statement],
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        match self.evaluate_block_step(source, statements, None, None, trace)? {
            ExecutionStep::Complete(value) => Ok(value),
            ExecutionStep::Advanced { .. } => unreachable!("a block runs to completion"),
            ExecutionStep::Returned { .. } => {
                unreachable!("a standalone block rejects return without a function context")
            }
        }
    }

    #[inline(never)]
    fn evaluate_block_step(
        &self,
        source: &SourceText,
        statements: &[Statement],
        result_classifier: Option<&str>,
        return_classifier: Option<&str>,
        trace: &mut impl TraceSink,
    ) -> Result<ExecutionStep, Diagnostic> {
        if statements.is_empty() {
            trace.record(TraceEvent {
                event: "block.empty.evaluated",
                rule: "TOPAL-SYN-GRAMMAR-001",
                detail: "Unit",
            });
            return Ok(ExecutionStep::Complete(Value::Unit));
        }
        let mut branch = self.clone();
        let mut execution = Execution {
            source: source.clone(),
            statements: statements.to_vec(),
            cursor: 0,
            result_classifier: result_classifier.map(str::to_owned),
            return_classifier: return_classifier.map(str::to_owned),
        };
        loop {
            match execution.step(&mut branch, trace)? {
                ExecutionStep::Complete(value) => {
                    trace.record(TraceEvent {
                        event: "block.evaluated",
                        rule: "TOPAL-SYN-GRAMMAR-001",
                        detail: &structural_value_classifier(&value),
                    });
                    return Ok(ExecutionStep::Complete(value));
                }
                ExecutionStep::Advanced { .. } => {}
                returned @ ExecutionStep::Returned { .. } => {
                    return Ok(returned);
                }
            }
        }
    }

    fn evaluate_record(
        &self,
        source: &SourceText,
        fields: &[topal_syntax::ProductField],
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let mut values = Vec::with_capacity(fields.len());
        for field in fields {
            let label_span = field.label.expect("record fields are labeled");
            let label = source.slice(label_span);
            if values.iter().any(|(existing, _)| existing == label) {
                return Err(diagnostic(
                    source,
                    "E-DUPLICATE-RECORD-FIELD",
                    label_span,
                    "record field label occurs more than once",
                ));
            }
            let value = self.evaluate_expression(source, &field.value, trace)?;
            values.push((label.to_owned(), value));
        }
        let detail = format!("fields={}", values.len());
        trace.record(TraceEvent {
            event: "product.record",
            rule: "TOPAL-TYPE-PRODUCT-001",
            detail: &detail,
        });
        Ok(Value::Record(values))
    }

    fn evaluate_product(
        &self,
        source: &SourceText,
        fields: &[topal_syntax::ProductField],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let labeled = fields.iter().filter(|field| field.label.is_some()).count();
        if labeled != 0 && labeled != fields.len() {
            return Err(diagnostic(
                source,
                "E-MIXED-PRODUCT-FIELDS",
                span,
                "a product cannot mix positional and labeled fields",
            ));
        }
        if labeled == 0 {
            let values = fields
                .iter()
                .map(|field| self.evaluate_expression(source, &field.value, trace))
                .collect::<Result<Vec<_>, _>>()?;
            let detail = format!("fields={}", values.len());
            trace.record(TraceEvent {
                event: "product.tuple",
                rule: "TOPAL-TYPE-PRODUCT-001",
                detail: &detail,
            });
            Ok(Value::Tuple(values))
        } else {
            self.evaluate_record(source, fields, trace)
        }
    }

    #[allow(clippy::too_many_lines)] // Built-in static policy identities stay explicit and auditable.
    fn resolve_identifier(
        &self,
        source: &SourceText,
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let name = source.slice(span);
        if let Ok(version) = name.parse::<LanguageVersion>() {
            return Ok(Value::Version(version));
        }
        if let Some(classifier) = self.generic_types.get(name) {
            trace.record(TraceEvent {
                event: "type.resolved",
                rule: "TOPAL-FUNCTION-GENERIC-HEADER-001",
                detail: classifier,
            });
            return Ok(Value::Type(classifier.clone()));
        }
        if matches!(name, "Little" | "Big") {
            trace.record(TraceEvent {
                event: "layout.policy.resolved",
                rule: "TOPAL-LAYOUT-ENDIAN-001",
                detail: name,
            });
            return Ok(Value::Enum {
                type_name: "Endian".into(),
                alternative: name.into(),
            });
        }
        if matches!(name, "ReadWrite" | "ReadOnly" | "WriteOnly" | "Reserved") {
            trace.record(TraceEvent {
                event: "layout.policy.resolved",
                rule: "TOPAL-LAYOUT-ACCESS-001",
                detail: name,
            });
            return Ok(Value::Enum {
                type_name: "Access".into(),
                alternative: name.into(),
            });
        }
        if matches!(name, "MostSignificantFirst" | "LeastSignificantFirst") {
            trace.record(TraceEvent {
                event: "layout.policy.resolved",
                rule: "TOPAL-LAYOUT-BIT-ORDER-001",
                detail: name,
            });
            return Ok(Value::Enum {
                type_name: "BitOrder".into(),
                alternative: name.into(),
            });
        }
        if matches!(name, "Natural" | "Packed") {
            trace.record(TraceEvent {
                event: "layout.policy.resolved",
                rule: "TOPAL-LAYOUT-PACKING-001",
                detail: name,
            });
            return Ok(Value::Enum {
                type_name: "Packing".into(),
                alternative: name.into(),
            });
        }
        if name == "Declared" {
            trace.record(TraceEvent {
                event: "layout.policy.resolved",
                rule: "TOPAL-LAYOUT-FIELD-ORDER-001",
                detail: name,
            });
            return Ok(Value::Enum {
                type_name: "FieldOrder".into(),
                alternative: name.into(),
            });
        }
        if matches!(name, "AfterTag" | "Overlay") {
            trace.record(TraceEvent {
                event: "layout.policy.resolved",
                rule: "TOPAL-LAYOUT-PAYLOAD-PLACEMENT-001",
                detail: name,
            });
            return Ok(Value::Enum {
                type_name: "PayloadPlacement".into(),
                alternative: name.into(),
            });
        }
        if matches!(name, "NoLength" | "NoTerminator") {
            trace.record(TraceEvent {
                event: "layout.policy.resolved",
                rule: "TOPAL-LAYOUT-ABSENCE-POLICY-001",
                detail: name,
            });
            return Ok(Value::Enum {
                type_name: "LayoutPolicy".into(),
                alternative: name.into(),
            });
        }
        if matches!(
            name,
            "Empty"
                | "BooleanBits"
                | "RawBits"
                | "UnsignedBinary"
                | "TwosComplement"
                | "OnesComplement"
                | "SignMagnitude"
                | "BiasedBinary"
                | "Ratio"
                | "Utf8"
                | "Utf16"
                | "Utf32"
                | "Ascii"
                | "Tagged"
                | "NoPadding"
                | "Cached"
                | "Uncached"
                | "Memory"
                | "MMIO"
        ) {
            return Ok(Value::Enum {
                type_name: "LayoutEncoding".into(),
                alternative: name.into(),
            });
        }
        if matches!(
            name,
            "Boolean"
                | "Completed"
                | "Int"
                | "MessageContext"
                | "Nat"
                | "Rational"
                | "Scope"
                | "String"
                | "Unit"
        ) && name != "Completed"
        {
            trace.record(TraceEvent {
                event: "type.resolved",
                rule: "TOPAL-ABSTRACTION-TYPE-VALUE-001",
                detail: name,
            });
            return Ok(Value::Type(name.into()));
        }
        if matches!(
            name,
            "Equality" | "Ordering" | "Foldable" | "Membership" | "Indexed" | "Keyed"
        ) {
            trace.record(TraceEvent {
                event: "capability.resolved",
                rule: "TOPAL-CAPABILITY-EVIDENCE-001",
                detail: name,
            });
            return Ok(Value::Capability(vec![BTreeSet::from([name.to_owned()])]));
        }
        if matches!(name, "std" | "advent-of-code") && !self.declared_libraries.contains(name) {
            return Err(diagnostic(
                source,
                "E-UNDECLARED-LIBRARY",
                span,
                format!("the `{name}` namespace requires `use library {name} ( version is v0.1 )`"),
            ));
        }
        if name == "root" {
            trace.record(TraceEvent {
                event: "namespace.resolved",
                rule: "TOPAL-NAMESPACE-ROOT-001",
                detail: "root",
            });
            return Ok(Value::Namespace(self.effective_root_namespace()));
        }
        if name == "Completed" {
            trace.record(TraceEvent {
                event: "completion.evidence",
                rule: "TOPAL-EXEC-COMPLETED-001",
                detail: "Completed",
            });
            return Ok(Value::Completed);
        }
        if self.consumed_names.contains(name) {
            return Err(consumed_generator_diagnostic(source, span, name));
        }
        let value = if let Some(value) = self.bindings.get(name) {
            value.clone()
        } else if let Some(candidates) = self.functions.get(name) {
            Value::NamedFunction(Rc::new(NamedFunction {
                name: name.to_owned(),
                candidates: candidates.clone(),
            }))
        } else {
            let error = diagnostic(source, "E-UNBOUND-NAME", span, "name is not bound");
            return Err(closest_name(name, self.bindings.keys())
                .or_else(|| closest_name(name, self.functions.keys()))
                .or_else(|| closest_root_operation(name))
                .map_or(error.clone(), |candidate| {
                    error.with_help(format!("did you mean `{candidate}`?"))
                }));
        };
        trace.record(TraceEvent {
            event: "binding.resolved",
            rule: "TOPAL-SYN-BIND-001",
            detail: name,
        });
        Ok(value)
    }

    fn evaluate_union_decision_action(
        &self,
        source: &SourceText,
        subject: Value,
        binding: Span,
        action: &Expression,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let Value::Union(mut union) = subject else {
            unreachable!("payload Union matcher selected only for a payload alternative")
        };
        let payload = union
            .payload
            .take()
            .expect("payload matcher selected a present payload");
        let name = source.slice(binding);
        let mut branch = self.clone();
        branch.bindings.insert(name.to_owned(), *payload);
        trace.record(TraceEvent {
            event: "union.payload.bound",
            rule: "TOPAL-DECISION-UNION-001",
            detail: name,
        });
        branch.evaluate_expression(source, action, trace)
    }

    fn union_constructor(&self, name: &str) -> Option<(&str, &str)> {
        self.union_types
            .iter()
            .find_map(|(type_name, alternatives)| {
                alternatives
                    .get(name)
                    .and_then(|classifier| classifier.as_deref())
                    .map(|classifier| (type_name.as_str(), classifier))
            })
    }

    fn classifier_supports_equality(&self, classifier: &str) -> bool {
        self.classifier_supports_equality_inner(classifier.trim(), &mut BTreeSet::new())
    }

    fn classifier_supports_equality_inner(
        &self,
        classifier: &str,
        visiting_sums: &mut BTreeSet<String>,
    ) -> bool {
        if matches!(
            classifier,
            "Unit"
                | "Completed"
                | "Effect"
                | "Type"
                | "Boolean"
                | "Character"
                | "Comparison"
                | "ErrorCode"
                | "Int"
                | "Nat"
                | "Rational"
                | "String"
        ) || self.enum_types.contains_key(classifier)
        {
            return true;
        }
        if let Some(payload) = optional_payload_classifier(classifier) {
            return self.classifier_supports_equality_inner(payload, visiting_sums);
        }
        if let Some(element) = list_element_classifier(classifier) {
            return self.classifier_supports_equality_inner(element, visiting_sums);
        }
        if let Some(fields) = tuple_classifiers(classifier) {
            return fields
                .into_iter()
                .all(|field| self.classifier_supports_equality_inner(field, visiting_sums));
        }
        if let Some(fields) = record_classifiers(classifier) {
            return fields
                .into_iter()
                .all(|(_, field)| self.classifier_supports_equality_inner(field, visiting_sums));
        }
        if let Some(alternatives) = self.union_types.get(classifier) {
            if !visiting_sums.insert(classifier.to_owned()) {
                return false;
            }
            let supports_equality = alternatives.values().all(|payload| {
                payload.as_deref().is_none_or(|payload| {
                    self.classifier_supports_equality_inner(payload, visiting_sums)
                })
            });
            visiting_sums.remove(classifier);
            return supports_equality;
        }
        match self.bindings.get(classifier) {
            Some(Value::ModularType(_)) => true,
            Some(Value::Constraint(constraint)) => {
                self.classifier_supports_equality_inner(&constraint.base_classifier, visiting_sums)
            }
            _ => false,
        }
    }

    fn application_is_union_constructor(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(constructor), _] if self.union_constructor(source.slice(*constructor)).is_some())
            || matches!(
                items,
                [Expression::Identifier(type_name), Expression::Identifier(at), Expression::Integer(_), _]
                    if source.slice(*at) == "at" && self.union_types.contains_key(source.slice(*type_name))
            )
    }

    fn is_constraint_definition(source: &SourceText, items: &[Expression]) -> bool {
        matches!(
            items,
            [Expression::Identifier(_), Expression::Identifier(operation), Expression::AnonymousFunction { .. }]
                if source.slice(*operation) == "constraint"
        )
    }

    fn is_modular_type_definition(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(kind), _] if matches!(source.slice(*kind), "ModNat" | "ModInt"))
    }

    fn is_explicit_modulo(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(
            items,
            [_, Expression::Identifier(operation), Expression::Identifier(type_name)]
                if source.slice(*operation) == "modulo"
                    && matches!(self.bindings.get(source.slice(*type_name)), Some(Value::ModularType(_)))
        )
    }

    fn is_range_selection(source: &SourceText, items: &[Expression]) -> bool {
        matches!(
            items,
            [_, Expression::Identifier(operation), selector]
                if matches!(source.slice(*operation), "select" | "select-index")
                    && !matches!(selector, Expression::AnonymousFunction { .. })
        )
    }

    fn is_bound_list_higher_order_application(
        &self,
        source: &SourceText,
        items: &[Expression],
    ) -> bool {
        let bound_function = |expression: &Expression| {
            matches!(expression, Expression::Identifier(name)
                if matches!(self.bindings.get(source.slice(*name)), Some(Value::AnonymousFunction(_))))
        };
        matches!(
            items,
            [_, Expression::Identifier(operation), function]
                if matches!(source.slice(*operation), "map" | "select" | "remove-indexes" | "remove-values")
                    && bound_function(function)
        ) || matches!(
            items,
            [_, Expression::Identifier(operation), _, function]
                if source.slice(*operation) == "fold" && bound_function(function)
        )
    }

    fn is_bound_anonymous_call(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(name), _]
            if matches!(self.bindings.get(source.slice(*name)), Some(Value::AnonymousFunction(_))))
    }

    fn is_bound_callable_call(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(name), _]
            if matches!(self.bindings.get(source.slice(*name)), Some(Value::Callable(_))))
    }

    fn is_bound_named_function_call(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(name), _]
            if matches!(self.bindings.get(source.slice(*name)), Some(Value::NamedFunction(_))))
    }

    fn is_root_qualified_application(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(root), Expression::Identifier(_), ..]
            if source.slice(*root) == "root")
    }

    fn is_empty_effects(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(name), Expression::Unit(_)] if source.slice(*name) == "Effects")
    }

    fn is_use_application(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(keyword), _]
            if source.slice(*keyword) == "use")
    }

    fn evaluate_use_application(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [_, selected] = items else {
            unreachable!("preselected use application")
        };
        let value = self.evaluate_expression(source, selected, trace)?;
        if !matches!(value, Value::Namespace(_)) {
            return Err(diagnostic(
                source,
                "E-USE-NON-NAMESPACE",
                selected.span(),
                "use requires a published namespace path",
            ));
        }
        trace.record(TraceEvent {
            event: "namespace.made-available",
            rule: "TOPAL-NAMESPACE-USE-001",
            detail: &value.to_string(),
        });
        self.checkpoint(trace, Some(&value), Some(span));
        Ok(value)
    }

    fn is_bound_namespace_application(&self, source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(alias), Expression::Identifier(_), ..]
            if matches!(self.bindings.get(source.slice(*alias)), Some(Value::Namespace(_))))
    }

    fn evaluate_bound_namespace_application(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [
            Expression::Identifier(alias),
            Expression::Identifier(member),
            remainder @ ..,
        ] = items
        else {
            unreachable!("preselected namespace alias application")
        };
        let Some(Value::Namespace(namespace)) = self.bindings.get(source.slice(*alias)) else {
            unreachable!("preselected namespace alias")
        };
        let alias_name = source.slice(*alias);
        if matches!(alias_name, "std" | "advent-of-code")
            && !self.declared_libraries.contains(alias_name)
        {
            return Err(diagnostic(
                source,
                "E-UNDECLARED-LIBRARY",
                *alias,
                format!(
                    "the `{alias_name}` namespace requires `use library {alias_name} ( version is v0.1 )`"
                ),
            ));
        }
        let member_name = source.slice(*member);
        if !namespace.bindings.contains_key(member_name)
            && !namespace.functions.contains_key(member_name)
            && !namespace.generators.contains_key(member_name)
        {
            let names = namespace
                .bindings
                .keys()
                .chain(namespace.functions.keys())
                .chain(namespace.generators.keys());
            let error = diagnostic(
                source,
                "E-NAMESPACE-MEMBER-NOT-FOUND",
                *member,
                format!(
                    "namespace `{}` has no member `{member_name}`",
                    namespace.name
                ),
            );
            return Err(
                closest_name(member_name, names).map_or(error.clone(), |candidate| {
                    error.with_help(format!("did you mean `{candidate}`?"))
                }),
            );
        }
        trace.record(TraceEvent {
            event: "namespace.alias.member.resolved",
            rule: "TOPAL-NAMESPACE-ALIAS-001",
            detail: member_name,
        });
        let mut qualified = self.clone();
        qualified.bindings = namespace.bindings.clone();
        *qualified.functions = namespace.functions.clone();
        *qualified.generators = namespace.generators.clone();
        if remainder.is_empty() {
            return qualified.resolve_identifier(source, *member, trace);
        }
        let expression = Expression::Application {
            items: std::iter::once(Expression::Identifier(*member))
                .chain(remainder.iter().cloned())
                .collect(),
            span,
        };
        qualified.evaluate_expression(source, &expression, trace)
    }

    fn evaluate_root_qualified_application(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [
            Expression::Identifier(_),
            Expression::Identifier(member),
            remainder @ ..,
        ] = items
        else {
            unreachable!("preselected root-qualified application")
        };
        let member_name = source.slice(*member);
        let namespace = self.effective_root_namespace();
        if !namespace.bindings.contains_key(member_name)
            && !namespace.functions.contains_key(member_name)
            && !namespace.generators.contains_key(member_name)
        {
            let names = namespace
                .bindings
                .keys()
                .chain(namespace.functions.keys())
                .chain(namespace.generators.keys());
            let error = diagnostic(
                source,
                "E-NAMESPACE-MEMBER-NOT-FOUND",
                *member,
                format!("namespace `root` has no member `{member_name}`"),
            );
            return Err(
                closest_name(member_name, names).map_or(error.clone(), |candidate| {
                    error.with_help(format!("did you mean `{candidate}`?"))
                }),
            );
        }
        trace.record(TraceEvent {
            event: "namespace.member.resolved",
            rule: "TOPAL-NAMESPACE-ROOT-001",
            detail: member_name,
        });
        let mut qualified = self.clone();
        qualified.bindings = namespace.bindings.clone();
        *qualified.functions = namespace.functions.clone();
        *qualified.generators = namespace.generators.clone();
        if remainder.is_empty() {
            return qualified.resolve_identifier(source, *member, trace);
        }
        let expression = Expression::Application {
            items: std::iter::once(Expression::Identifier(*member))
                .chain(remainder.iter().cloned())
                .collect(),
            span,
        };
        qualified.evaluate_expression(source, &expression, trace)
    }

    fn evaluate_bound_named_function_call(
        &self,
        source: &SourceText,
        expression: &Expression,
        items: &[Expression],
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(alias), _] = items else {
            unreachable!("preselected bound named function call")
        };
        let alias = source.slice(*alias);
        let Some(Value::NamedFunction(function)) = self.bindings.get(alias) else {
            unreachable!("preselected named function binding")
        };
        let mut invocation = Box::new(self.clone());
        invocation.root_namespace = Some(self.effective_root_namespace());
        invocation.bindings.remove(alias);
        invocation
            .functions
            .insert(alias.to_owned(), function.candidates.clone());
        trace.record(TraceEvent {
            event: "function.value.called",
            rule: "TOPAL-FUNCTION-VALUE-001",
            detail: &function.name,
        });
        invocation.evaluate_expression(source, expression, trace)
    }

    #[inline(never)]
    fn evaluate_user_function_call(
        &self,
        source: &SourceText,
        name_span: Span,
        argument_expression: &Expression,
        call_span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let argument_span = argument_expression.span();
        let argument = self.evaluate_expression(source, argument_expression, trace)?;
        self.invoke_user_function(source, name_span, argument_span, argument, call_span, trace)
    }

    #[inline(never)]
    #[allow(clippy::too_many_lines)] // Call admission, scope setup, and cleanup remain ordered.
    fn invoke_user_function(
        &self,
        source: &SourceText,
        name_span: Span,
        argument_span: Span,
        argument: Value,
        call_span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let name = source.slice(name_span);
        let candidates = self
            .functions
            .get(name)
            .expect("preselected user function")
            .clone();
        self.invoke_user_function_candidates(
            source,
            name,
            name_span,
            argument_span,
            argument,
            call_span,
            &candidates,
            trace,
        )
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments, clippy::too_many_lines)] // Retained callable identity shares ordinary invocation semantics.
    fn invoke_user_function_candidates(
        &self,
        source: &SourceText,
        name: &str,
        name_span: Span,
        argument_span: Span,
        argument: Value,
        call_span: Span,
        candidates: &[UserFunction],
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let function = candidates
            .iter()
            .find(|function| {
                (!self.static_context || function.is_static)
                    && user_function_accepts(function, &argument)
            })
            .cloned();
        let Some(function) = function else {
            if self.static_context && candidates.iter().all(|function| !function.is_static) {
                return Err(diagnostic(
                    source,
                    "E-STATIC-CALLS-RUNTIME-FUNCTION",
                    name_span,
                    format!("static execution cannot call ordinary function `{name}`"),
                ));
            }
            return Err(no_applicable_overload(
                source,
                name,
                argument_span,
                &argument,
                candidates,
                self.static_context,
            ));
        };
        if matches!(
            argument,
            Value::CharacterGenerator { .. }
                | Value::CharacterReturningGenerator { .. }
                | Value::SuspendedGenerator { .. }
        ) {
            let classifier = structural_value_classifier(&argument);
            trace.record(TraceEvent {
                event: "generator.parameter.transferred",
                rule: if matches!(argument, Value::SuspendedGenerator { .. }) {
                    "TOPAL-GENERATOR-FUNCTION-PARAMETER-001"
                } else {
                    "TOPAL-STRING-CHARACTERS-PARAMETER-001"
                },
                detail: &classifier,
            });
        }
        let signature = function_signature(name, &function);
        let recursion_rule = recursion_rule_for_call(&self.call_stack, name, &signature, &function);
        if self
            .call_stack
            .iter()
            .any(|active| active.signature == signature)
            && recursion_rule.is_none()
        {
            return Err(diagnostic(
                source,
                "E-UNPROVEN-RECURSION",
                name_span,
                format!(
                    "recursive cycle returning to `{name}` requires termination proof on every call edge"
                ),
            ));
        }
        let rule = function_rule(function.is_static, function.parameters.len());
        if let Some(recursion_rule) = recursion_rule {
            if is_mutual_recursion_rule(recursion_rule) {
                trace.record(TraceEvent {
                    event: "function.recursion.cycle.proven",
                    rule: recursion_rule,
                    detail: name,
                });
            }
            trace.record(TraceEvent {
                event: "function.recursion.descended",
                rule: recursion_rule,
                detail: name,
            });
        }
        if candidates.len() > 1 {
            trace.record(TraceEvent {
                event: "function.overload.selected",
                rule: "TOPAL-FUNCTION-OVERLOAD-001",
                detail: &signature,
            });
        }
        let mut function_scope = Box::new(Self {
            bindings: function.bindings.clone(),
            functions: self.functions.clone(),
            generators: self.generators.clone(),
            root_namespace: Some(self.effective_root_namespace()),
            defining_context: function_defining_context(&function),
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
            generic_types: BTreeMap::new(),
            call_stack: self.call_stack.clone(),
            static_context: function.is_static,
            task_state: None,
            next_task_identity: Cell::new(self.next_task_identity.get()),
            next_transaction_identity: Cell::new(self.next_transaction_identity.get()),
        });
        function_scope.call_stack.push(ActiveCall {
            name: name.to_owned(),
            signature: signature.clone(),
            termination_rule: function.termination_rule,
            recursion_target: function.recursion_target.clone(),
        });
        bind_function_arguments(&mut function_scope, &function, argument, trace, rule)?;
        function_scope.enforce_function_precondition(&function, name, trace)?;
        let mut invocation_generics = BTreeMap::new();
        populate_function_generics(&function, &function_scope, &mut invocation_generics);
        function_scope.generic_types = invocation_generics;
        trace.record(TraceEvent {
            event: "function.selected",
            rule: "TOPAL-TYPE-CALL-001",
            detail: &signature,
        });
        trace.record(TraceEvent {
            event: "function.entry",
            rule,
            detail: name,
        });
        let mut body_execution = Execution {
            source: function.source.clone(),
            statements: (*function.body).clone(),
            cursor: 0,
            result_classifier: Some(function.result.clone()),
            return_classifier: Some(function.result.clone()),
        };
        let (value, result_span) = loop {
            match body_execution.step(&mut function_scope, trace)? {
                ExecutionStep::Advanced { .. } => {}
                ExecutionStep::Complete(value) => {
                    break (
                        value,
                        statement_span(function.body.last().expect("function body is nonempty")),
                    );
                }
                ExecutionStep::Returned { value, span } => break (value, span),
            }
        };
        if !function.result.starts_with("Generator ") {
            close_remaining_character_generators(&mut function_scope, trace)?;
        }
        if !generic_result_accepts(&function, &function_scope, &value) {
            return Err(diagnostic(
                &function.source,
                "E-FUNCTION-RESULT-TYPE",
                result_span,
                format!(
                    "function `{name}` returned `{}`, outside `{}`",
                    structural_value_classifier(&value),
                    function.result,
                ),
            ));
        }
        function_scope.enforce_function_postcondition(&function, name, &value, trace)?;
        if let Value::Error { domain, code, .. } = &value
            && result_success_classifier(&function.result).is_some()
        {
            let detail = format!("domain={domain};code={code}");
            trace.record(TraceEvent {
                event: "result.error.propagated",
                rule: "TOPAL-TYPE-RESULT-001",
                detail: &detail,
            });
        }
        if matches!(
            value,
            Value::CharacterGenerator { .. }
                | Value::CharacterReturningGenerator { .. }
                | Value::SuspendedGenerator { .. }
        ) {
            let classifier = structural_value_classifier(&value);
            trace.record(TraceEvent {
                event: "generator.result.transferred",
                rule: if matches!(value, Value::SuspendedGenerator { .. }) {
                    "TOPAL-GENERATOR-FUNCTION-RESULT-001"
                } else {
                    "TOPAL-STRING-CHARACTERS-RESULT-001"
                },
                detail: &classifier,
            });
        }
        trace.record(TraceEvent {
            event: "function.exit",
            rule,
            detail: name,
        });
        self.checkpoint(trace, Some(&value), Some(call_span));
        Ok(value)
    }

    fn invoke_function_value(
        &self,
        source: &SourceText,
        function: Value,
        argument: Value,
        argument_span: Span,
        call_span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        match function {
            Value::NamedFunction(function) => {
                trace.record(TraceEvent {
                    event: "function.value.called",
                    rule: "TOPAL-FUNCTION-VALUE-001",
                    detail: &function.name,
                });
                self.invoke_user_function_candidates(
                    source,
                    &function.name,
                    call_span,
                    argument_span,
                    argument,
                    call_span,
                    &function.candidates,
                    trace,
                )
            }
            Value::Callable(kind) => {
                Self::invoke_callable_value(source, kind, argument, argument_span, call_span, trace)
            }
            function @ Value::AnonymousFunction(_) => {
                let Value::AnonymousFunction(anonymous) = &function else {
                    unreachable!("matched anonymous function")
                };
                let arity = anonymous.parameters.len();
                let arguments = match (arity, argument) {
                    (1, value) => vec![value],
                    (_, Value::Tuple(values)) => values,
                    (_, value) => {
                        return Err(diagnostic(
                            source,
                            "E-ANONYMOUS-ARGUMENT-PACKAGE",
                            argument_span,
                            format!(
                                "anonymous function expects {arity} arguments packaged as a tuple, found `{}`",
                                structural_value_classifier(&value)
                            ),
                        ));
                    }
                };
                self.invoke_anonymous_function(&function, arguments, call_span, trace)
            }
            value => Err(diagnostic(
                source,
                "E-NO-APPLICABLE-OVERLOAD",
                argument_span,
                format!(
                    "application chain requires Function before its next operand, found `{}`",
                    structural_value_classifier(&value)
                ),
            )),
        }
    }

    fn invoke_callable_value(
        source: &SourceText,
        kind: CallableKind,
        argument: Value,
        argument_span: Span,
        call_span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        trace.record(TraceEvent {
            event: "function.callable.called",
            rule: "TOPAL-FUNCTION-CALLABLE-VALUE-001",
            detail: callable_name(kind),
        });
        match argument {
            Value::Tuple(mut operands) if operands.len() == 2 => {
                let right = operands.pop().expect("two operands");
                let left = operands.pop().expect("two operands");
                apply_binary(
                    source,
                    kind,
                    left,
                    right,
                    (call_span, argument_span, argument_span),
                    trace,
                )
            }
            operand if kind == CallableKind::Minus => {
                apply_negate(source, operand, call_span, trace)
            }
            value => Err(diagnostic(
                source,
                "E-CALLABLE-ARGUMENT-PACKAGE",
                argument_span,
                format!(
                    "callable `{}` requires a two-field positional product, found `{}`",
                    callable_name(kind),
                    structural_value_classifier(&value)
                ),
            )),
        }
    }

    fn evaluate_bound_callable_call(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(name), argument] = items else {
            unreachable!("preselected bound callable call")
        };
        let Value::Callable(kind) = self.resolve_identifier(source, *name, trace)? else {
            unreachable!("preselected callable binding")
        };
        let argument_span = argument.span();
        let argument = self.evaluate_expression(source, argument, trace)?;
        Self::invoke_callable_value(source, kind, argument, argument_span, span, trace)
    }

    fn is_traversal_control_constructor(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [Expression::Identifier(name), _]
            if matches!(source.slice(*name), "Continue" | "Finish"))
    }

    fn is_iterate_construction(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [_, Expression::Identifier(operation), Expression::AnonymousFunction { .. }]
            if source.slice(*operation) == "iterate")
    }

    fn is_unfold_construction(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items, [_, Expression::Identifier(operation), Expression::AnonymousFunction { .. }]
            if source.slice(*operation) == "unfold")
    }

    fn construct_unfold_generator(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [seed, _, step_expression] = items else {
            unreachable!("preselected unfold construction")
        };
        let seed = self.evaluate_expression(source, seed, trace)?;
        let step = self.evaluate_expression(source, step_expression, trace)?;
        if !matches!(&step, Value::AnonymousFunction(function) if function.parameters.len() == 1) {
            return Err(diagnostic(
                source,
                "E-UNFOLD-FUNCTION-ARITY",
                step_expression.span(),
                "unfold step function requires exactly one seed parameter",
            ));
        }
        trace.record(TraceEvent {
            event: "generator.unfold.constructed",
            rule: "TOPAL-GENERATOR-UNFOLD-001",
            detail: &structural_value_classifier(&seed),
        });
        let yield_classifier = infer_unfold_yield_classifier(&seed, &step);
        let value = Value::UnfoldGenerator {
            seed: Box::new(seed),
            step: Box::new(step),
            yield_classifier,
        };
        self.checkpoint(trace, Some(&value), Some(span));
        Ok(value)
    }

    fn is_iterate_take_while_construction(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items,
            [_, Expression::Identifier(iterate), Expression::AnonymousFunction { .. }, Expression::Identifier(take_while), Expression::AnonymousFunction { .. }]
                if source.slice(*iterate) == "iterate" && source.slice(*take_while) == "take-while")
    }

    fn construct_iterate_take_while(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [initial, _, next_expression, _, predicate_expression] = items else {
            unreachable!("preselected iterate take-while construction")
        };
        let current = self.evaluate_expression(source, initial, trace)?;
        let classifier = structural_value_classifier(&current);
        let next = self.evaluate_expression(source, next_expression, trace)?;
        let predicate = self.evaluate_expression(source, predicate_expression, trace)?;
        for (value, expression, role) in [
            (&next, next_expression, "next"),
            (&predicate, predicate_expression, "predicate"),
        ] {
            if !matches!(value, Value::AnonymousFunction(function) if function.parameters.len() == 1)
            {
                return Err(diagnostic(
                    source,
                    "E-GENERATED-TRAVERSAL-FUNCTION-ARITY",
                    expression.span(),
                    format!("iterate {role} function requires exactly one parameter"),
                ));
            }
        }
        trace.record(TraceEvent {
            event: "generator.take-while.constructed",
            rule: "TOPAL-GENERATOR-TAKE-WHILE-001",
            detail: &classifier,
        });
        let value = Value::IterateGenerator {
            current: Box::new(current),
            next: Box::new(next),
            take_while: Some(Box::new(predicate)),
            classifier,
        };
        self.checkpoint(trace, Some(&value), Some(span));
        Ok(value)
    }

    fn is_generator_take_while_application(source: &SourceText, items: &[Expression]) -> bool {
        matches!(items,
            [_, Expression::Identifier(take_while), Expression::AnonymousFunction { .. }]
                if source.slice(*take_while) == "take-while")
    }

    fn apply_generator_take_while(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [generator, _, predicate] = items else {
            unreachable!("preselected generator take-while application")
        };
        let generator_span = generator.span();
        let generator = self.evaluate_expression(source, generator, trace)?;
        let Value::IterateGenerator {
            current,
            next,
            classifier,
            ..
        } = generator
        else {
            return Err(diagnostic(
                source,
                "E-TAKE-WHILE-SOURCE",
                generator_span,
                "take-while requires a lazy iterate generator",
            ));
        };
        let predicate_value = self.evaluate_expression(source, predicate, trace)?;
        if !matches!(&predicate_value, Value::AnonymousFunction(function) if function.parameters.len() == 1)
        {
            return Err(diagnostic(
                source,
                "E-GENERATED-TRAVERSAL-FUNCTION-ARITY",
                predicate.span(),
                "take-while predicate requires exactly one parameter",
            ));
        }
        trace.record(TraceEvent {
            event: "generator.take-while.constructed",
            rule: "TOPAL-GENERATOR-TAKE-WHILE-001",
            detail: &classifier,
        });
        let value = Value::IterateGenerator {
            current,
            next,
            take_while: Some(Box::new(predicate_value)),
            classifier,
        };
        self.checkpoint(trace, Some(&value), Some(span));
        Ok(value)
    }

    fn construct_iterate_generator(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [initial, _, next_expression] = items else {
            unreachable!("preselected iterate construction")
        };
        let current = self.evaluate_expression(source, initial, trace)?;
        let classifier = structural_value_classifier(&current);
        let next = self.evaluate_expression(source, next_expression, trace)?;
        let Value::AnonymousFunction(function) = &next else {
            unreachable!("iterate syntax requires an anonymous function")
        };
        if function.parameters.len() != 1 {
            return Err(diagnostic(
                source,
                "E-ITERATE-FUNCTION-ARITY",
                next_expression.span(),
                "iterate next function requires exactly one parameter",
            ));
        }
        trace.record(TraceEvent {
            event: "generator.iterate.constructed",
            rule: "TOPAL-GENERATOR-ITERATE-001",
            detail: &classifier,
        });
        let value = Value::IterateGenerator {
            current: Box::new(current),
            next: Box::new(next),
            take_while: None,
            classifier,
        };
        self.checkpoint(trace, Some(&value), Some(span));
        Ok(value)
    }

    fn construct_traversal_control(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(name), payload] = items else {
            unreachable!("preselected traversal-control constructor")
        };
        let payload = self.evaluate_expression(source, payload, trace)?;
        let constructor = source.slice(*name);
        trace.record(TraceEvent {
            event: "traversal.control.constructed",
            rule: "TOPAL-EXEC-TRAVERSAL-CONTROL-001",
            detail: constructor,
        });
        let value = if constructor == "Continue" {
            Value::Continue(Box::new(payload))
        } else {
            Value::Finish(Box::new(payload))
        };
        self.checkpoint(trace, Some(&value), Some(span));
        Ok(value)
    }

    fn evaluate_bound_anonymous_call(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(name), argument_expression] = items else {
            unreachable!("preselected bound anonymous call")
        };
        let function = self.resolve_identifier(source, *name, trace)?;
        let arity = match &function {
            Value::AnonymousFunction(function) => function.parameters.len(),
            _ => unreachable!("preselected anonymous binding"),
        };
        let argument = self.evaluate_expression(source, argument_expression, trace)?;
        let arguments = match (arity, argument) {
            (1, value) => vec![value],
            (_, Value::Tuple(values)) => values,
            (_, value) => {
                return Err(diagnostic(
                    source,
                    "E-ANONYMOUS-ARGUMENT-PACKAGE",
                    argument_expression.span(),
                    format!(
                        "anonymous function expects {arity} arguments packaged as a tuple, found `{}`",
                        structural_value_classifier(&value)
                    ),
                ));
            }
        };
        self.invoke_anonymous_function(&function, arguments, span, trace)
    }

    fn is_record_reconstruction(source: &SourceText, items: &[Expression]) -> bool {
        matches!(
            items,
            [_, Expression::Identifier(operation), Expression::Product { fields, .. }]
                if source.slice(*operation) == "with"
                    && !fields.is_empty()
                    && fields.iter().all(|field| field.label.is_some())
        )
    }

    fn evaluate_record_reconstruction(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [
            base,
            _,
            Expression::Product {
                fields: replacements,
                ..
            },
        ] = items
        else {
            unreachable!("preselected record reconstruction")
        };
        let Value::Record(mut fields) = self.evaluate_expression(source, base, trace)? else {
            return Err(diagnostic(
                source,
                "E-RECONSTRUCT-NON-RECORD",
                base.span(),
                "`with` reconstruction requires a labeled product",
            ));
        };
        let mut replaced = BTreeSet::new();
        for replacement in replacements {
            let label_span = replacement.label.expect("preselected labeled replacement");
            let label = source.slice(label_span);
            if !replaced.insert(label) {
                return Err(diagnostic(
                    source,
                    "E-DUPLICATE-RECONSTRUCTION-FIELD",
                    label_span,
                    format!("field `{label}` is replaced more than once"),
                ));
            }
            let Some((_, value)) = fields.iter_mut().find(|(name, _)| name == label) else {
                return Err(diagnostic(
                    source,
                    "E-NO-SUCH-RECORD-FIELD",
                    label_span,
                    format!("record has no field named `{label}`"),
                ));
            };
            *value = self.evaluate_expression(source, &replacement.value, trace)?;
            trace.record(TraceEvent {
                event: "record.field.replaced",
                rule: "TOPAL-TYPE-RECONSTRUCT-001",
                detail: label,
            });
        }
        let result = Value::Record(fields);
        self.checkpoint(trace, Some(&result), Some(span));
        Ok(result)
    }

    fn evaluate_range_selection(
        &self,
        source: &SourceText,
        items: &[Expression],
        span: Span,
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [collection, Expression::Identifier(operation), selector] = items else {
            unreachable!("preselected range selection")
        };
        let collection_value = self.evaluate_expression(source, collection, trace)?;
        let selector_value = self.evaluate_expression(source, selector, trace)?;
        let Value::IntRange {
            lower,
            upper,
            lower_inclusive,
            upper_inclusive,
        } = selector_value
        else {
            return Err(diagnostic(
                source,
                "E-SELECTION-RANGE",
                selector.span(),
                "range selection requires Range Int",
            ));
        };
        let operation = source.slice(*operation);
        let result = match collection_value {
            Value::List {
                element_classifier,
                entries,
            } if operation == "select-index" => {
                let entries = entries
                    .into_iter()
                    .enumerate()
                    .filter(|(index, _)| {
                        let index = BigInt::from(*index);
                        bound_contains(&index, &lower, &upper, lower_inclusive, upper_inclusive)
                    })
                    .map(|(_, value)| value)
                    .collect();
                Value::List {
                    element_classifier,
                    entries,
                }
            }
            Value::List {
                element_classifier,
                entries,
            } if operation == "select" => {
                let entries = entries
                    .into_iter()
                    .filter(|value| {
                        matches!(value, Value::Int(candidate) if bound_contains(candidate, &lower, &upper, lower_inclusive, upper_inclusive))
                    })
                    .collect();
                Value::List {
                    element_classifier,
                    entries,
                }
            }
            Value::String(text) if operation == "select-index" => {
                let selected = characters(&text)
                    .enumerate()
                    .filter(|(index, _)| {
                        let index = BigInt::from(*index);
                        bound_contains(&index, &lower, &upper, lower_inclusive, upper_inclusive)
                    })
                    .map(|(_, character)| character)
                    .collect::<String>();
                Value::String(selected)
            }
            value => {
                return Err(diagnostic(
                    source,
                    "E-SELECTION-SOURCE",
                    collection.span(),
                    format!(
                        "{operation} range has no overload for `{}`",
                        structural_value_classifier(&value)
                    ),
                ));
            }
        };
        trace.record(TraceEvent {
            event: "collection.range.selected",
            rule: if operation == "select-index" {
                "TOPAL-RANGE-INDEX-SELECTION-001"
            } else {
                "TOPAL-RANGE-VALUE-SELECTION-001"
            },
            detail: operation,
        });
        self.checkpoint(trace, Some(&result), Some(span));
        Ok(result)
    }

    fn apply_explicit_modulo(
        &self,
        source: &SourceText,
        items: &[Expression],
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [operand, _, Expression::Identifier(type_name)] = items else {
            unreachable!("preselected explicit modular reduction")
        };
        let operand_value = self.evaluate_expression(source, operand, trace)?;
        let Value::Int(value) = operand_value else {
            return Err(diagnostic(
                source,
                "E-MODULO-OPERAND",
                operand.span(),
                "explicit modulo construction requires Int",
            ));
        };
        let name = source.slice(*type_name);
        let Value::ModularType(kind) = self.bindings.get(name).expect("known modular type") else {
            unreachable!("preselected modular type")
        };
        let value = reduce_modular(value, &kind.lower, &kind.upper);
        trace.record(TraceEvent {
            event: "numeric.modular.reduced",
            rule: "TOPAL-NUM-MODULAR-REDUCE-001",
            detail: name,
        });
        Ok(Value::Modular {
            type_name: name.into(),
            lower: kind.lower.clone(),
            upper: kind.upper.clone(),
            value,
        })
    }

    fn construct_modular_type(
        &self,
        source: &SourceText,
        items: &[Expression],
        trace: &mut impl TraceSink,
    ) -> Result<Value, Diagnostic> {
        let [Expression::Identifier(kind), range] = items else {
            unreachable!("preselected modular type definition")
        };
        let signed = source.slice(*kind) == "ModInt";
        let range_value = self.evaluate_expression(source, range, trace)?;
        let Value::IntRange {
            lower,
            upper,
            lower_inclusive: true,
            upper_inclusive: true,
        } = range_value
        else {
            return Err(diagnostic(
                source,
                "E-MODULAR-RANGE",
                range.span(),
                "ModNat and ModInt require a finite Int range",
            ));
        };
        if lower > BigInt::from(0)
            || upper < BigInt::from(0)
            || (!signed && lower != BigInt::from(0))
        {
            return Err(diagnostic(
                source,
                "E-MODULAR-RANGE",
                range.span(),
                "modular range must contain zero and ModNat must begin at zero",
            ));
        }
        trace.record(TraceEvent {
            event: "numeric.modular.type.constructed",
            rule: "TOPAL-NUM-MODULAR-TYPE-001",
            detail: source.slice(*kind),
        });
        Ok(Value::ModularType(Box::new(ModularType {
            name: None,
            signed,
            lower,
            upper,
        })))
    }
}
