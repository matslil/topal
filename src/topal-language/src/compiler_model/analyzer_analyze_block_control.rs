impl Analyzer {
    #[allow(clippy::too_many_lines)] // Exhaustive statement admission keeps the subset boundary visible.
    fn analyze_block_control(
        &mut self,
        statements: &[Statement],
        environment: &mut BTreeMap<String, BindingFacts>,
        kind: BlockKind,
        block_result: Option<&CompilerType>,
        function_result: Option<&CompilerType>,
        allow_function_return: bool,
    ) -> Result<AnalyzedBlock, Diagnostic> {
        let mut lowered = Vec::new();
        let mut generator_bindings = Vec::new();
        let mut result = None;
        let mut explicit_return = false;
        let mut returns_from_function = false;
        let mut declared = if kind == BlockKind::Lexical {
            BTreeSet::new()
        } else {
            environment.keys().cloned().collect()
        };
        if kind != BlockKind::TopLevel
            && let Some(statement) = statements.iter().find(|statement| {
                interface_declaration(statement).is_some()
                    || matches!(statement, Statement::InterfaceImplementation { .. })
            })
        {
            return Err(unsupported(
                &self.source,
                statement_span(statement),
                "non-root Interface declaration or implementation",
            ));
        }
        if kind == BlockKind::Lexical
            && let Some(declaration) = statements
                .iter()
                .find(|statement| self.is_declaration(statement))
        {
            return Err(unsupported(
                &self.source,
                statement_span(declaration),
                "nested declaration",
            ));
        }
        let executable = statements
            .iter()
            .filter(|statement| {
                !self.is_declaration(statement)
                    || (kind == BlockKind::Function
                        && (matches!(statement, Statement::Function { .. })
                            || matches!(
                                statement,
                                Statement::Published { declaration, .. }
                                    if matches!(declaration.as_ref(), Statement::Function { .. })
                            )))
            })
            .collect::<Vec<_>>();

        for (index, statement) in executable.iter().enumerate() {
            let last = index + 1 == executable.len();
            let published_binding = matches!(
                *statement,
                Statement::Published { declaration, .. }
                    if matches!(declaration.as_ref(), Statement::Binding { .. })
            );
            let statement = match *statement {
                Statement::Published { declaration, .. }
                    if matches!(declaration.as_ref(), Statement::Binding { .. }) =>
                {
                    declaration.as_ref()
                }
                statement => statement,
            };
            match statement {
                Statement::DiagnosticControl { .. } => {
                    if last {
                        result = Some(unit_expression(statement_span(statement)));
                    }
                }
                Statement::Function { .. } if kind == BlockKind::Function => {
                    self.bind_nested_function(statement, environment, &mut declared)?;
                    if last {
                        result = Some(unit_expression(statement_span(statement)));
                    }
                }
                Statement::Published { declaration, .. }
                    if kind == BlockKind::Function
                        && matches!(declaration.as_ref(), Statement::Function { .. }) =>
                {
                    self.bind_nested_function(statement, environment, &mut declared)?;
                    if last {
                        result = Some(unit_expression(statement_span(statement)));
                    }
                }
                Statement::Binding {
                    name,
                    classifier,
                    value: initializer,
                } => {
                    let name_text = self.source.slice(*name).to_owned();
                    if declared.contains(&name_text)
                        || (kind == BlockKind::TopLevel
                            && (name_text == "root"
                                || self.functions.contains_key(&name_text)
                                || self.generators.contains_key(&name_text)
                                || self.enums.contains_key(&name_text)
                                || self.enum_alternatives.contains_key(&name_text)
                                || self.sums.contains_key(&name_text)
                                || self.sum_alternatives.contains_key(&name_text)
                                || self.modulars.contains_key(&name_text)
                                || self.interfaces.contains_key(&name_text)
                                || self.task_types.contains_key(&name_text)
                                || self.task_definitions.contains_key(&name_text)))
                    {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-DUPLICATE-BINDING",
                            *name,
                            format!("`{name_text}` is already declared in this scope"),
                        ));
                    }
                    let expected = classifier
                        .map(|classifier| self.parse_classifier(classifier))
                        .transpose()?;
                    let (mut value, returned) =
                        if constraint_definition(&self.source, initializer).is_some() {
                            (
                                self.analyze_constraint_definition(
                                    &name_text,
                                    initializer,
                                    environment,
                                )?,
                                false,
                            )
                        } else {
                            self.analyze_direct_statement_expression(
                                initializer,
                                environment,
                                expected.as_ref(),
                                function_result,
                                allow_function_return,
                            )?
                        };
                    if returned {
                        explicit_return = true;
                        returns_from_function = true;
                        result = Some(value);
                        break;
                    }
                    if let (Some(classifier), Some(expected)) = (classifier, expected) {
                        if expected == CompilerType::Int
                            && value.value_type == CompilerType::Rational
                        {
                            let span = value.span;
                            value = self.finish_int_conversion(value, span, span)?;
                        } else if expected == CompilerType::Nat
                            && value.value_type == CompilerType::Int
                        {
                            let span = value.span;
                            value = self.finish_nat_conversion(value, span, span)?;
                        } else if expected != value.value_type
                            && let Some(adapted) = adapt_function_call_argument(&expected, &value)
                        {
                            value = adapted;
                        }
                        if let CompilerType::Result(success) = &value.value_type
                            && (success.as_ref() == &expected
                                || matches!(&expected, CompilerType::Refined { base, .. }
                                    if success.as_ref() == base.as_ref()))
                        {
                            let boundary_projection = kind != BlockKind::TopLevel
                                && !matches!(function_result, Some(CompilerType::Result(_)))
                                && matches!(&expected,
                                    CompilerType::Refined { constraint, .. }
                                        if matches!(constraint.as_str(),
                                            "NonemptyPattern" | "RegexValid" | "NonnegativeWeight"));
                            if kind != BlockKind::TopLevel
                                && !matches!(function_result, Some(CompilerType::Result(_)))
                                && !boundary_projection
                            {
                                return Err(source_diagnostic(
                                    &self.source,
                                    "E-RESULT-PROJECTION-CONTEXT",
                                    value.span,
                                    "Result success projection requires an enclosing compatible Result function",
                                ));
                            }
                            let span = value.span;
                            value = CompilerExpression {
                                kind: if boundary_projection {
                                    CompilerExpressionKind::ResultProjectBoundary(Box::new(value))
                                } else {
                                    CompilerExpressionKind::ResultProject(Box::new(value))
                                },
                                value_type: expected.clone(),
                                int_range: None,
                                rational_value: None,
                                span,
                            };
                        }
                        if !matches!(
                            (&expected, &value.value_type),
                            (CompilerType::Int, CompilerType::InfiniteInt)
                                | (CompilerType::Nat, CompilerType::InfiniteNat)
                                | (CompilerType::Rational, CompilerType::InfiniteRational)
                        ) {
                            require_same_type(
                                &self.source,
                                *classifier,
                                &expected,
                                &value.value_type,
                            )?;
                        }
                    }
                    reject_static_value_containment(&self.source, &value)?;
                    if published_binding && compiler_type_contains_infinity(&value.value_type) {
                        return Err(unsupported(
                            &self.source,
                            value.span,
                            "public infinity boundary",
                        ));
                    }
                    let constraint_tag = if value.value_type == CompilerType::Constraint {
                        if kind != BlockKind::TopLevel {
                            return Err(unsupported(
                                &self.source,
                                *name,
                                "non-root Constraint binding",
                            ));
                        }
                        let tag = if constraint_definition(&self.source, initializer).is_some() {
                            let CompilerExpressionKind::ConstraintValue(tag) = &value.kind else {
                                unreachable!("checked constraint construction has an identity tag")
                            };
                            *tag
                        } else {
                            let Expression::Identifier(source_name_span) = initializer else {
                                return Err(unsupported(
                                    &self.source,
                                    initializer.span(),
                                    "Constraint value expression",
                                ));
                            };
                            let source_name = self.source.slice(*source_name_span);
                            let source_tag =
                                *self.constraint_bindings.get(source_name).ok_or_else(|| {
                                    unsupported(
                                        &self.source,
                                        *source_name_span,
                                        "Constraint value without retained static metadata",
                                    )
                                })?;
                            let mut constraint = self.constraints
                                [usize::try_from(source_tag).expect("u32 tag fits usize")]
                            .clone();
                            constraint.name.clone_from(&name_text);
                            constraint.span = Span::new(name.start, initializer.span().end);
                            let tag = u32::try_from(self.constraints.len()).map_err(|_| {
                                unsupported(&self.source, *name, "native Constraint value tag")
                            })?;
                            self.constraints.push(constraint);
                            value.kind = CompilerExpressionKind::ConstraintValue(tag);
                            tag
                        };
                        Some(tag)
                    } else {
                        None
                    };
                    let external_binding = matches!(
                        value.kind,
                        CompilerExpressionKind::ExternalMetadata(_)
                            | CompilerExpressionKind::ExternalLocationConstruct(_)
                    );
                    if external_binding && kind != BlockKind::TopLevel {
                        return Err(unsupported(
                            &self.source,
                            *name,
                            "non-root external-storage binding",
                        ));
                    }
                    if matches!(value.value_type, CompilerType::ExternalLocation(_))
                        && !matches!(
                            value.kind,
                            CompilerExpressionKind::ExternalLocationConstruct(_)
                        )
                    {
                        return Err(unsupported(
                            &self.source,
                            value.span,
                            "copied or selected external Location value",
                        ));
                    }
                    if kind == BlockKind::TopLevel {
                        self.external_metadata.remove(&name_text);
                        self.external_locations.remove(&name_text);
                    }
                    let external_static =
                        if let CompilerExpressionKind::ExternalMetadata(metadata) = &mut value.kind
                        {
                            assign_external_metadata_identity(metadata, &name_text);
                            self.external_metadata
                                .insert(name_text.clone(), metadata.clone());
                            true
                        } else {
                            false
                        };
                    if let CompilerExpressionKind::ExternalLocationConstruct(location) = &value.kind
                    {
                        self.external_locations
                            .insert(name_text.clone(), location.clone());
                    }
                    let string_value = Self::known_string_value(&value, environment);
                    let closed_int_range = Self::known_closed_int_range(&value, environment);
                    let list_count = Self::known_list_count(&value, environment);
                    let list_string_keys = Self::known_list_string_keys(&value, environment);
                    let aggregate_facts = self.known_structural_value_facts(&value, environment)?;
                    let returned_callable_captures =
                        returned_function_capture_bindings(&aggregate_facts);
                    let tuple_fields = aggregate_facts.tuple_fields;
                    let record_fields = aggregate_facts.record_fields;
                    let list_entries = aggregate_facts.list_entries;
                    let string_characters = aggregate_facts.string_characters;
                    let list_string_characters = aggregate_facts.list_string_characters;
                    let array_entries = aggregate_facts.array_entries;
                    let map_entries = aggregate_facts.map_entries;
                    let optional = aggregate_facts.optional;
                    let sum = aggregate_facts.sum;
                    let result_facts = aggregate_facts.result;
                    let namespace =
                        self.known_namespace(&value, environment, initializer.span().start, kind)?;
                    let callable = aggregate_facts.callable;
                    let static_capability = match &value.kind {
                        CompilerExpressionKind::Capability(capability) => Some(capability.clone()),
                        _ => None,
                    };
                    if static_capability.is_some() && kind != BlockKind::TopLevel {
                        return Err(unsupported(
                            &self.source,
                            *name,
                            "non-root Capability binding",
                        ));
                    }
                    let storage_name = if kind == BlockKind::TopLevel {
                        format!("topal.root.{}.{}", name.start, name_text)
                    } else if matches!(value.value_type, CompilerType::Generator(_)) {
                        format!("topal.generator.{}.{}", name.start, name_text)
                    } else {
                        name_text.clone()
                    };
                    let generator_value = if matches!(value.value_type, CompilerType::Generator(_))
                    {
                        match &value.kind {
                            CompilerExpressionKind::Local(storage_name) => {
                                self.generator_values.get(storage_name).cloned()
                            }
                            CompilerExpressionKind::Call { symbol, .. } => {
                                self.returned_generator_values.get(symbol).cloned()
                            }
                            CompilerExpressionKind::PrivateBinding { body, .. }
                                if matches!(&body.kind, CompilerExpressionKind::Call { .. }) =>
                            {
                                let CompilerExpressionKind::Call { symbol, .. } = &body.kind else {
                                    unreachable!("guard retains a call")
                                };
                                self.returned_generator_values.get(symbol).cloned()
                            }
                            CompilerExpressionKind::IterateGenerator { .. }
                            | CompilerExpressionKind::GeneratorTakeWhile { .. }
                            | CompilerExpressionKind::UnfoldGenerator { .. }
                            | CompilerExpressionKind::StringCharactersGenerator { .. }
                            | CompilerExpressionKind::StringRangeCharactersGenerator { .. }
                            | CompilerExpressionKind::StringProvenanceCharactersGenerator {
                                ..
                            }
                            | CompilerExpressionKind::CustomCharacterGenerator { .. }
                            | CompilerExpressionKind::CustomValueGenerator { .. } => {
                                Some(value.clone())
                            }
                            _ => None,
                        }
                    } else {
                        None
                    };
                    let infinity_negative = compiler_infinity_direction(&value);
                    let facts = BindingFacts {
                        storage_name: storage_name.clone(),
                        origin: name.start,
                        runtime_bound: !external_static
                            && !compiler_type_contains_static_only(&value.value_type),
                        value_type: value.value_type.clone(),
                        int_range: value.int_range.clone(),
                        rational_value: value.rational_value.clone(),
                        infinity_negative,
                        string_value,
                        string_characters,
                        closed_int_range,
                        list_count,
                        list_string_keys,
                        list_string_characters,
                        list_entries,
                        array_entries,
                        map_entries,
                        tuple_fields,
                        record_fields,
                        optional,
                        sum,
                        result: result_facts,
                        namespace,
                        callable,
                        static_capability,
                    };
                    if matches!(facts.value_type, CompilerType::Generator(_)) {
                        if let Some(generator_value) = generator_value {
                            self.generator_values
                                .insert(storage_name.clone(), generator_value);
                        }
                        generator_bindings.push((storage_name.clone(), name_text.clone(), *name));
                    }
                    if self.compiler_proven_nonnegative_list(&value) {
                        self.nonnegative_lists.insert(storage_name.clone());
                    }
                    environment.insert(name_text.clone(), facts.clone());
                    if facts.value_type == CompilerType::Nat
                        || self.compiler_proven_nonnegative_expression(&value)
                    {
                        self.active_nonnegative_bindings
                            .insert(storage_name.clone());
                    }
                    if kind == BlockKind::TopLevel
                        && let Some(callable) = facts.callable.clone()
                    {
                        self.root_callable_bindings
                            .insert(name_text.clone(), callable);
                    }
                    for capture in returned_callable_captures {
                        environment.insert(capture.storage_name.clone(), capture);
                    }
                    if let Some(tag) = constraint_tag {
                        self.constraint_bindings.insert(name_text.clone(), tag);
                        self.constraint_binding_declarations
                            .insert(name_text.clone(), statement_span(statement).end);
                    }
                    if kind == BlockKind::TopLevel && facts.runtime_bound {
                        self.root_bindings.insert(
                            name_text.clone(),
                            CompilerDataMemberFacts::from_binding(
                                &facts,
                                statement_span(statement).end,
                            ),
                        );
                    }
                    declared.insert(name_text.clone());
                    lowered.push(CompilerStatement::Binding(CompilerBinding {
                        name: name_text,
                        storage_name,
                        span: *name,
                        value,
                    }));
                    if last {
                        result = Some(unit_expression(statement_span(statement)));
                    }
                }
                Statement::Foreach {
                    result: foreach_result,
                    source,
                    binding,
                    body,
                    span,
                } if kind == BlockKind::TopLevel
                    || (kind == BlockKind::Function
                        && matches!(source, Expression::Identifier(name)
                        if environment.get(self.source.slice(*name)).is_some_and(|facts|
                            facts.storage_name == self.source.slice(*name)
                                && ((foreach_result.is_none()
                                    && is_admitted_character_generator_type(&facts.value_type))
                                    || (foreach_result.is_some()
                                        && is_admitted_value_boundary_generator_type(
                                            &facts.value_type
                                        )))))) =>
                {
                    let value =
                        self.analyze_root_foreach(source, *binding, body, *span, environment)?;
                    if let Some((name, classifier)) = foreach_result {
                        let admitted_typed_result = matches!(
                            &value.kind,
                            CompilerExpressionKind::CustomValueForeach { result, .. }
                                if matches!(exact_string(result).as_deref(), Some("unary" | "binary"))
                                    || (kind == BlockKind::Function
                                        && (exact_string(result).as_deref() == Some("done")
                                            || exact_int_string_product(result, 8, "done")
                                            || exact_nested_result_product_value(
                                                result, 8, "done"
                                            )
                                            || exact_int_list_append(result, "initial", 9)))
                        );
                        if value.value_type != CompilerType::Unit && !admitted_typed_result {
                            return Err(unsupported(
                                &self.source,
                                *span,
                                "binding a non-Unit custom generator final result outside admitted overload and function-boundary paths",
                            ));
                        }
                        let name_text = self.source.slice(*name).to_owned();
                        if declared.contains(&name_text)
                            || name_text == "root"
                            || self.functions.contains_key(&name_text)
                            || self.enums.contains_key(&name_text)
                            || self.enum_alternatives.contains_key(&name_text)
                            || self.sums.contains_key(&name_text)
                            || self.sum_alternatives.contains_key(&name_text)
                            || self.modulars.contains_key(&name_text)
                            || self.interfaces.contains_key(&name_text)
                        {
                            return Err(source_diagnostic(
                                &self.source,
                                "E-DUPLICATE-BINDING",
                                *name,
                                format!("`{name_text}` is already declared in this scope"),
                            ));
                        }
                        if let Some(classifier) = classifier {
                            let expected = self.parse_classifier(*classifier)?;
                            require_same_type(
                                &self.source,
                                *classifier,
                                &expected,
                                &value.value_type,
                            )?;
                        }
                        let storage_name = if kind == BlockKind::TopLevel {
                            format!("topal.root.{}.{}", name.start, name_text)
                        } else {
                            name_text.clone()
                        };
                        let facts = BindingFacts {
                            storage_name: storage_name.clone(),
                            origin: name.start,
                            runtime_bound: true,
                            value_type: value.value_type.clone(),
                            int_range: value.int_range.clone(),
                            rational_value: value.rational_value.clone(),
                            infinity_negative: compiler_infinity_direction(&value),
                            string_value: None,
                            string_characters: None,
                            closed_int_range: None,
                            list_count: Self::known_list_count(&value, environment),
                            list_string_keys: Self::known_list_string_keys(&value, environment),
                            list_string_characters: None,
                            list_entries: None,
                            array_entries: None,
                            map_entries: None,
                            tuple_fields: Vec::new(),
                            record_fields: BTreeMap::new(),
                            optional: None,
                            sum: None,
                            result: None,
                            namespace: None,
                            callable: None,
                            static_capability: None,
                        };
                        environment.insert(name_text.clone(), facts.clone());
                        if kind == BlockKind::TopLevel {
                            self.root_bindings.insert(
                                name_text.clone(),
                                CompilerDataMemberFacts::from_binding(&facts, span.end),
                            );
                        }
                        declared.insert(name_text.clone());
                        lowered.push(CompilerStatement::Binding(CompilerBinding {
                            name: name_text,
                            storage_name,
                            value,
                            span: *name,
                        }));
                    } else if last && value.value_type != CompilerType::Unit {
                        result = Some(value);
                    } else {
                        lowered.push(CompilerStatement::Discard(value));
                    }
                    if last && result.is_none() {
                        result = Some(unit_expression(*span));
                    }
                }
                Statement::Discard { value, .. } => {
                    let (value, returned) = self.analyze_direct_statement_expression(
                        value,
                        environment,
                        None,
                        function_result,
                        allow_function_return,
                    )?;
                    if returned {
                        explicit_return = true;
                        returns_from_function = true;
                        result = Some(value);
                        break;
                    }
                    if matches!(value.value_type, CompilerType::Generator(_)) {
                        return Err(unsupported(
                            &self.source,
                            value.span,
                            "generator abandonment and close delivery",
                        ));
                    }
                    reject_static_value_containment(&self.source, &value)?;
                    lowered.push(CompilerStatement::Discard(value));
                    if last {
                        result = Some(unit_expression(statement_span(statement)));
                    }
                }
                Statement::Expression(expression) if last => {
                    let (value, returned) = self.analyze_direct_statement_expression(
                        expression,
                        environment,
                        block_result,
                        function_result,
                        allow_function_return,
                    )?;
                    explicit_return |= returned;
                    returns_from_function |= returned;
                    result = Some(value);
                }
                Statement::Expression(expression) => {
                    let (value, returned) = self.analyze_direct_statement_expression(
                        expression,
                        environment,
                        None,
                        function_result,
                        allow_function_return,
                    )?;
                    if returned {
                        explicit_return = true;
                        returns_from_function = true;
                        result = Some(value);
                        break;
                    }
                    if value.value_type == CompilerType::Unit
                        && matches!(
                            value.kind,
                            CompilerExpressionKind::TaskStateReplace { .. }
                                | CompilerExpressionKind::ExternalLocationWrite { .. }
                        )
                    {
                        lowered.push(CompilerStatement::Discard(value));
                    } else {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-NONFINAL-VALUE",
                            statement_span(statement),
                            "a non-final value expression must be explicitly discarded or bound",
                        ));
                    }
                }
                Statement::Return { value, .. }
                    if kind == BlockKind::Function
                        || (kind == BlockKind::Lexical && allow_function_return) =>
                {
                    explicit_return = true;
                    returns_from_function = true;
                    let (value, _) = self.analyze_direct_statement_expression(
                        value,
                        environment,
                        function_result,
                        function_result,
                        true,
                    )?;
                    result = Some(value);
                    break;
                }
                Statement::Return { .. } if kind == BlockKind::TopLevel || !self.in_function => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-RETURN-OUTSIDE-FUNCTION",
                        statement_span(statement),
                        "`return` is available only inside a function",
                    ));
                }
                Statement::Return { .. } if kind == BlockKind::Lexical => {
                    return Err(unsupported(
                        &self.source,
                        statement_span(statement),
                        "return through a nested lexical block outside an admitted direct statement position",
                    ));
                }
                Statement::LibrarySelection { .. } if kind == BlockKind::TopLevel => {}
                Statement::LibrarySelection { .. } => {
                    return Err(unsupported(
                        &self.source,
                        statement_span(statement),
                        "nested source library dependency",
                    ));
                }
                _ => {
                    return Err(unsupported(
                        &self.source,
                        statement_span(statement),
                        "statement form",
                    ));
                }
            }
        }
        let unconsumed_generators = generator_bindings
            .iter()
            .filter(|(storage, _, _)| !self.consumed_generators.contains(storage))
            .collect::<Vec<_>>();
        if !unconsumed_generators.is_empty() {
            let [(storage, name, binding_span)] = unconsumed_generators.as_slice() else {
                let (_, name, span) = unconsumed_generators[0];
                let feature = format!("unconsumed generator `{name}` and close delivery");
                return Err(unsupported(&self.source, *span, &feature));
            };
            let close_context = kind == BlockKind::Function
                && !self.static_context
                && function_result == Some(&CompilerType::Unit)
                && !explicit_return
                && result
                    .as_ref()
                    .is_some_and(|result| result.value_type == CompilerType::Unit);
            let close_handler = close_context
                .then(|| self.generator_values.get(storage))
                .flatten()
                .and_then(|generator| match &generator.kind {
                    CompilerExpressionKind::CustomCharacterGenerator {
                        prefix,
                        characters,
                        locals,
                        close_handler,
                        result,
                        ..
                    } if prefix.statements.is_empty()
                        && characters.len() == 1
                        && locals.is_empty()
                        && result.value_type == CompilerType::Unit =>
                    {
                        Some(close_handler.clone())
                    }
                    _ => None,
                });
            let Some(close_handler) = close_handler else {
                let feature = format!("unconsumed generator `{name}` and close delivery");
                return Err(unsupported(&self.source, *binding_span, &feature));
            };
            let provenance = self
                .generator_values
                .remove(storage)
                .expect("checked close-only generator provenance exists");
            let close_span = result.as_ref().map_or(*binding_span, |value| value.span);
            let generator = CompilerExpression {
                kind: CompilerExpressionKind::Local((*storage).clone()),
                value_type: provenance.value_type.clone(),
                int_range: None,
                rational_value: None,
                span: *binding_span,
            };
            let close = match close_handler {
                Some(handler) => CompilerExpressionKind::CustomCharacterHandledClose {
                    generator: Box::new(generator),
                    handler,
                },
                None => CompilerExpressionKind::CustomCharacterClose {
                    generator: Box::new(generator),
                    provenance: Box::new(provenance),
                    close_domain: "root".into(),
                },
            };
            lowered.push(CompilerStatement::Discard(CompilerExpression {
                kind: close,
                value_type: CompilerType::Unit,
                int_range: None,
                rational_value: None,
                span: close_span,
            }));
        }
        let result = result.unwrap_or_else(|| unit_expression(Span::new(0, 0)));
        if kind == BlockKind::TopLevel
            && matches!(result.value_type, CompilerType::Generator(_))
            && matches!(result.kind, CompilerExpressionKind::Call { .. })
        {
            return Err(unsupported(
                &self.source,
                result.span,
                "unbound returned Generator and close delivery",
            ));
        }
        Ok(AnalyzedBlock {
            block: CompilerBlock {
                statements: lowered,
                result,
            },
            returns_from_function,
        })
    }

    #[allow(clippy::too_many_lines)] // Every admitted closed traversal source stays fail-closed here.
    fn analyze_root_foreach(
        &mut self,
        source: &Expression,
        binding: Span,
        statements: &[Statement],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        if let Expression::Identifier(name) = source
            && environment
                .get(self.source.slice(*name))
                .is_some_and(|facts| facts.value_type == int_list_type())
        {
            let list = self.analyze_expression(source, environment)?;
            let (parameter, body) =
                self.analyze_unit_foreach_body(binding, statements, CompilerType::Int)?;
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListForeach {
                    list: Box::new(list),
                    parameter,
                    body: Box::new(body),
                },
                value_type: CompilerType::Unit,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let Expression::Application { items, .. } = source
            && let [Expression::Identifier(operation), text] = items.as_slice()
            && self.source.slice(*operation) == "characters"
        {
            let generator =
                self.analyze_closed_string_characters_generator(text, source.span(), environment)?;
            let CompilerExpressionKind::StringCharactersGenerator { characters, .. } =
                &generator.kind
            else {
                return Err(unsupported(
                    &self.source,
                    source.span(),
                    "direct dynamic String Character foreach",
                ));
            };
            return self.finish_string_characters_foreach(
                generator.clone(),
                characters.clone(),
                binding,
                statements,
                span,
            );
        }
        let retained_generator = if let Expression::Identifier(name) = source {
            environment
                .get(self.source.slice(*name))
                .and_then(|facts| self.generator_values.get(&facts.storage_name))
                .cloned()
        } else {
            None
        };
        if let Some(CompilerExpression {
            kind: CompilerExpressionKind::StringCharactersGenerator { characters, .. },
            ..
        }) = retained_generator.clone()
        {
            let source_value = self.analyze_expression(source, environment)?;
            require_type(
                &self.source,
                source_value.span,
                &character_unit_generator_type(),
                &source_value.value_type,
            )?;
            return self.finish_string_characters_foreach(
                source_value,
                characters,
                binding,
                statements,
                span,
            );
        }
        if let Some(CompilerExpression {
            kind:
                CompilerExpressionKind::CustomValueGenerator {
                    declaration_span,
                    initial_parameter,
                    initial,
                    additional_initial_parameters,
                    prefix,
                    yields,
                    continuations,
                    explicit_return,
                    result,
                    ..
                },
            value_type: CompilerType::Generator(generator_type),
            ..
        }) = retained_generator.clone()
        {
            let source_value = self.analyze_expression(source, environment)?;
            require_type(
                &self.source,
                source_value.span,
                &CompilerType::Generator(generator_type.clone()),
                &source_value.value_type,
            )?;
            let result_type = generator_type.result_type.as_ref().clone();
            let (parameter, body) = if matches!(initial_parameter.value_type, CompilerType::Task(_))
            {
                self.analyze_task_stream_foreach_body(
                    binding,
                    statements,
                    generator_type.yield_type.as_ref().clone(),
                    span,
                )?
            } else {
                self.analyze_unit_foreach_body(
                    binding,
                    statements,
                    generator_type.yield_type.as_ref().clone(),
                )?
            };
            if additional_initial_parameters.is_empty()
                && initial_parameter.value_type == CompilerType::Int
                && !exact_int_value_generator_action(&parameter, &body)
            {
                return Err(unsupported(
                    &self.source,
                    span,
                    "Int-value custom generator foreach action outside discarded value + 1",
                ));
            }
            if !additional_initial_parameters.is_empty()
                && !exact_string_overload_generator_action(&parameter, &body)
            {
                return Err(unsupported(
                    &self.source,
                    span,
                    "multi-input String-yield custom generator foreach action outside empty? value",
                ));
            }
            if initial_parameter.value_type == CompilerType::Nat
                && !exact_nat_value_generator_action(&parameter, &body)
            {
                return Err(unsupported(
                    &self.source,
                    span,
                    "Nat-value custom generator foreach action outside discarded value + 1",
                ));
            }
            if initial_parameter.value_type == int_list_type()
                && !exact_list_value_generator_action(&parameter, &body)
            {
                return Err(unsupported(
                    &self.source,
                    span,
                    "List-Int-value custom generator foreach action outside discarded entry-count values",
                ));
            }
            if matches!(initial_parameter.value_type, CompilerType::Enum(_))
                && !exact_enum_value_generator_action(&parameter, &body)
            {
                return Err(unsupported(
                    &self.source,
                    span,
                    "Choice-value custom generator foreach action outside discarded choice = First",
                ));
            }
            if initial_parameter.value_type == CompilerType::Comparison
                && !exact_comparison_value_generator_action(&parameter, &body)
            {
                return Err(unsupported(
                    &self.source,
                    span,
                    "Comparison-value custom generator foreach action outside discarded comparison = (1 <=> 2)",
                ));
            }
            if initial_parameter.value_type
                == CompilerType::Tuple(vec![CompilerType::Int, CompilerType::String])
                && !exact_int_string_product_value_generator_action(&parameter, &body)
            {
                return Err(unsupported(
                    &self.source,
                    span,
                    "(Int, String)-value custom generator foreach action outside discarded value = (7, \"item\")",
                ));
            }
            if initial_parameter.value_type
                == CompilerType::Result(Box::new(CompilerType::Rational))
                && !exact_result_rational_value_generator_action(&parameter, &body)
            {
                return Err(unsupported(
                    &self.source,
                    span,
                    "Result-Rational-value custom generator foreach action outside discarded candidate = candidate",
                ));
            }
            if initial_parameter.value_type == CompilerType::Rational
                && !exact_rational_value_generator_action(&parameter, &body)
            {
                return Err(unsupported(
                    &self.source,
                    span,
                    "Rational-value custom generator foreach action outside discarded value + Rational (1, 3)",
                ));
            }
            if initial_parameter.value_type == CompilerType::Unit
                && !exact_unit_value_generator_action(&parameter, &body)
            {
                return Err(unsupported(
                    &self.source,
                    span,
                    "Unit-value custom generator foreach action outside its named identity value",
                ));
            }
            if initial_parameter.value_type == CompilerType::Optional(Box::new(CompilerType::Int))
                && !exact_optional_int_value_generator_action(&parameter, &body)
            {
                return Err(unsupported(
                    &self.source,
                    span,
                    "Optional-Int-value custom generator foreach action outside discarded candidate = Some 7",
                ));
            }
            if initial_parameter.value_type
                == CompilerType::Optional(Box::new(CompilerType::Tuple(vec![
                    CompilerType::Int,
                    CompilerType::String,
                ])))
            {
                let some_graph = exact_optional_int_string_value(&initial, 7, "item")
                    && exact_optional_int_string_value(&result, 8, "done")
                    && exact_nested_optional_value_generator_action(&parameter, &body);
                let none_graph = exact_optional_int_string_none_value(&initial)
                    && exact_optional_int_string_none_value(&result)
                    && exact_nested_none_value_generator_action(&parameter, &body);
                let mixed_graph = exact_optional_int_string_value(&initial, 7, "item")
                    && exact_nested_result_product_value(&result, 8, "done")
                    && exact_nested_optional_value_generator_action(&parameter, &body);
                if !some_graph && !none_graph && !mixed_graph {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "nested Optional-value custom generator outside an exact all-Some, all-None, or admitted Result-final input/yield/action/final graph",
                    ));
                }
            }
            if initial_parameter.value_type == nested_result_product_type()
                && !exact_nested_result_value_generator_action(&parameter, &body)
            {
                return Err(unsupported(
                    &self.source,
                    span,
                    "nested Result-value custom generator foreach action outside discarded candidate = (7, \"item\")",
                ));
            }
            if recursive_nominal_enumeration(&initial_parameter.value_type).is_some()
                && !exact_recursive_nominal_generator_action(&parameter, &body)
            {
                return Err(unsupported(
                    &self.source,
                    span,
                    "recursive nominal custom generator foreach action outside discarded candidate = (Some First, First)",
                ));
            }
            if initial_parameter.value_type == CompilerType::Range(Box::new(CompilerType::Int))
                && !exact_int_range_value_generator_action(&parameter, &body)
            {
                return Err(unsupported(
                    &self.source,
                    span,
                    "Range-Int-value custom generator foreach action outside discarded 5 in interval",
                ));
            }
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::CustomValueForeach {
                    source: Box::new(source_value),
                    transferred_initial: self.in_function.then_some(initial),
                    declaration_span,
                    initial_parameter,
                    additional_initial_parameters,
                    prefix,
                    yields,
                    continuations,
                    explicit_return,
                    parameter,
                    body: Box::new(body),
                    result,
                },
                value_type: result_type,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let Some(CompilerExpression {
            kind:
                CompilerExpressionKind::CustomCharacterGenerator {
                    declaration_span,
                    characters,
                    locals,
                    close_handler,
                    result,
                    ..
                },
            ..
        }) = retained_generator
        {
            if close_handler.is_some() {
                return Err(unsupported(
                    &self.source,
                    source.span(),
                    "custom Generator close handler on a successful traversal path",
                ));
            }
            let source_value = self.analyze_expression(source, environment)?;
            let result_type = result.value_type.clone();
            require_type(
                &self.source,
                source_value.span,
                &character_generator_type(result_type.clone()),
                &source_value.value_type,
            )?;
            let (parameter, body) =
                self.analyze_unit_foreach_body(binding, statements, CompilerType::Character)?;
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::CustomCharacterForeach {
                    source: Box::new(source_value),
                    declaration_span,
                    characters,
                    locals,
                    parameter,
                    body: Box::new(body),
                    result,
                },
                value_type: result_type,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        self.analyze_bounded_int_iterate_foreach(source, binding, statements, span, environment)
    }

    fn analyze_closed_string_characters_generator(
        &mut self,
        text: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let text_value = self.analyze_expression(text, environment)?;
        require_type(
            &self.source,
            text_value.span,
            &CompilerType::String,
            &text_value.value_type,
        )?;
        if let CompilerExpressionKind::StringRangeSelect {
            text,
            characters,
            range,
        } = &text_value.kind
        {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::StringRangeCharactersGenerator {
                    text: text.clone(),
                    characters: characters.clone(),
                    range: range.clone(),
                },
                value_type: character_unit_generator_type(),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if matches!(&text_value.kind, CompilerExpressionKind::Local(storage_name)
            if binding_facts_by_storage(environment, storage_name)
                .is_none_or(|facts| facts.string_value.is_none()))
        {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::StringProvenanceCharactersGenerator {
                    text: Box::new(text_value),
                    characters: Vec::new(),
                },
                value_type: character_unit_generator_type(),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let Some(text) = Self::known_string_value(&text_value, environment) {
            let characters = characters(&text).map(str::to_owned).collect();
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::StringCharactersGenerator {
                    text: Box::new(text_value),
                    characters,
                },
                value_type: character_unit_generator_type(),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        let provenance = self.known_structural_value_facts(&text_value, environment)?;
        provenance.string_characters.map_or_else(
            || {
                Err(unsupported(
                    &self.source,
                    text.span(),
                    "dynamic String Character generator",
                ))
            },
            |characters| {
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::StringProvenanceCharactersGenerator {
                        text: Box::new(text_value),
                        characters,
                    },
                    value_type: character_unit_generator_type(),
                    int_range: None,
                    rational_value: None,
                    span,
                })
            },
        )
    }

    fn finish_string_characters_foreach(
        &mut self,
        source: CompilerExpression,
        characters: Vec<String>,
        binding: Span,
        statements: &[Statement],
        span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        let (parameter, body) =
            self.analyze_unit_foreach_body(binding, statements, CompilerType::Character)?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::StringCharactersForeach {
                source: Box::new(source),
                characters,
                parameter,
                body: Box::new(body),
            },
            value_type: CompilerType::Unit,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_bounded_int_iterate_foreach(
        &mut self,
        source: &Expression,
        binding: Span,
        statements: &[Statement],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let retained_generator = if let Expression::Identifier(name) = source {
            environment
                .get(self.source.slice(*name))
                .and_then(|facts| self.generator_values.get(&facts.storage_name))
                .cloned()
        } else {
            None
        };
        let source_value = self.analyze_expression(source, environment)?;
        require_type(
            &self.source,
            source_value.span,
            &int_unit_generator_type(),
            &source_value.value_type,
        )?;
        let generator = retained_generator.as_ref().unwrap_or(&source_value);
        let CompilerExpressionKind::GeneratorTakeWhile {
            generator: iterate,
            parameters: predicate_parameters,
            predicate,
        } = &generator.kind
        else {
            return Err(source_diagnostic(
                &self.source,
                "E-UNBOUNDED-GENERATOR-TRAVERSAL",
                source.span(),
                "foreach requires a statically finite generated traversal",
            ));
        };
        let CompilerExpressionKind::IterateGenerator {
            initial,
            parameters: next_parameters,
            next,
        } = &iterate.kind
        else {
            return Err(unsupported(
                &self.source,
                source.span(),
                "foreach through this Generator construction",
            ));
        };
        if !matches!(initial.kind, CompilerExpressionKind::Int(_))
            || !compiler_block_is_closed_over_parameters(next_parameters, next)
            || !compiler_block_is_closed_over_parameters(predicate_parameters, predicate)
        {
            return Err(unsupported(
                &self.source,
                source.span(),
                "captured or dynamically initialized iterate foreach",
            ));
        }

        let (parameter, body) =
            self.analyze_unit_foreach_body(binding, statements, CompilerType::Int)?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::IterateGeneratorForeach {
                generator: Box::new(generator.clone()),
                parameter,
                body: Box::new(body),
            },
            value_type: CompilerType::Unit,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_unit_foreach_body(
        &mut self,
        binding: Span,
        statements: &[Statement],
        value_type: CompilerType,
    ) -> Result<(CompilerParameter, CompilerBlock), Diagnostic> {
        let parameter_name = self.source.slice(binding).to_owned();
        let parameter = CompilerParameter {
            name: parameter_name.clone(),
            discarded: parameter_name == "_",
            source_visible: parameter_name != "_",
            value_type: value_type.clone(),
            int_range: None,
            span: binding,
        };
        if value_type == CompilerType::Nat {
            let body =
                exact_nat_value_generator_foreach_body(&self.source, &parameter, statements)?;
            return Ok((parameter, body));
        }
        if value_type
            == CompilerType::Optional(Box::new(CompilerType::Tuple(vec![
                CompilerType::Int,
                CompilerType::String,
            ])))
        {
            let body = exact_nested_optional_value_generator_foreach_body(
                &self.source,
                &parameter,
                statements,
            )?;
            return Ok((parameter, body));
        }
        if value_type == CompilerType::Result(Box::new(CompilerType::Rational)) {
            let body = exact_result_rational_value_generator_foreach_body(
                &self.source,
                &parameter,
                statements,
            )?;
            return Ok((parameter, body));
        }
        if value_type == nested_result_product_type() {
            let body = exact_nested_result_value_generator_foreach_body(
                &self.source,
                &parameter,
                statements,
            )?;
            return Ok((parameter, body));
        }
        if let Some(enumeration) = recursive_nominal_enumeration(&value_type) {
            let body = exact_recursive_nominal_generator_foreach_body(
                &self.source,
                &parameter,
                enumeration,
                statements,
            )?;
            return Ok((parameter, body));
        }
        let mut body_environment = BTreeMap::new();
        if !parameter.discarded {
            body_environment.insert(
                parameter_name,
                BindingFacts {
                    storage_name: parameter.name.clone(),
                    origin: parameter.span.start,
                    runtime_bound: true,
                    value_type,
                    int_range: None,
                    rational_value: None,
                    infinity_negative: None,
                    string_value: None,
                    string_characters: None,
                    closed_int_range: None,
                    list_count: None,
                    list_string_keys: None,
                    list_string_characters: None,
                    list_entries: None,
                    array_entries: None,
                    map_entries: None,
                    tuple_fields: Vec::new(),
                    record_fields: BTreeMap::new(),
                    optional: None,
                    sum: None,
                    result: None,
                    namespace: None,
                    callable: None,
                    static_capability: None,
                },
            );
        }
        let body = self.analyze_block(
            statements,
            &mut body_environment,
            BlockKind::Lexical,
            Some(&CompilerType::Unit),
        )?;
        require_type(
            &self.source,
            body.result.span,
            &CompilerType::Unit,
            &body.result.value_type,
        )?;
        Ok((parameter, body))
    }

    fn analyze_task_stream_foreach_body(
        &mut self,
        binding: Span,
        statements: &[Statement],
        value_type: CompilerType,
        span: Span,
    ) -> Result<(CompilerParameter, CompilerBlock), Diagnostic> {
        let parameter_name = self.source.slice(binding).to_owned();
        let parameter = CompilerParameter {
            name: parameter_name.clone(),
            discarded: parameter_name == "_",
            source_visible: parameter_name != "_",
            value_type: value_type.clone(),
            int_range: None,
            span: binding,
        };
        let mut environment = BTreeMap::new();
        if !parameter.discarded {
            environment.insert(
                parameter_name,
                BindingFacts {
                    storage_name: parameter.name.clone(),
                    origin: binding.start,
                    runtime_bound: true,
                    value_type,
                    int_range: None,
                    rational_value: None,
                    infinity_negative: None,
                    string_value: None,
                    string_characters: None,
                    closed_int_range: None,
                    list_count: None,
                    list_string_keys: None,
                    list_string_characters: None,
                    list_entries: None,
                    array_entries: None,
                    map_entries: None,
                    tuple_fields: Vec::new(),
                    record_fields: BTreeMap::new(),
                    optional: None,
                    sum: None,
                    result: None,
                    namespace: None,
                    callable: None,
                    static_capability: None,
                },
            );
        }
        let body = self.analyze_block(
            statements,
            &mut environment,
            BlockKind::Lexical,
            Some(&CompilerType::Unit),
        )?;
        if !body.statements.is_empty() || !matches!(body.result.kind, CompilerExpressionKind::Unit)
        {
            return Err(unsupported(
                &self.source,
                span,
                "direct task stream action outside one inert Unit resumption",
            ));
        }
        Ok((parameter, body))
    }
}
