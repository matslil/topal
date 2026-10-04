impl Analyzer {
    fn analyze_function_value_chain(
        &mut self,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        let Some(first) = items.first() else {
            return Ok(None);
        };
        let eligible = match first {
            Expression::Identifier(name) => {
                let source_arity = items.len().saturating_sub(1);
                items.len() > 2
                    && if let Some(callable) = environment
                        .get(self.source.slice(*name))
                        .and_then(|facts| facts.callable.as_ref())
                    {
                        match callable {
                            CompilerCallableFacts::Named { declarations, .. } => !declarations
                                .iter()
                                .any(|declaration| declaration.parameters.len() == source_arity),
                            CompilerCallableFacts::Symbolic(_)
                            | CompilerCallableFacts::Anonymous { .. } => true,
                        }
                    } else {
                        self.functions
                            .get(self.source.slice(*name))
                            .is_some_and(|declarations| {
                                !declarations
                                    .iter()
                                    .any(|declaration| declaration.parameters.len() == source_arity)
                            })
                    }
            }
            Expression::Application { .. } => items.len() == 2,
            _ => false,
        };
        if !eligible {
            return Ok(None);
        }

        let mut value = self.analyze_expression(first, environment)?;
        if value.value_type != CompilerType::Function {
            return Ok(None);
        }
        for operand in &items[1..] {
            if value.value_type != CompilerType::Function {
                return Err(source_diagnostic(
                    &self.source,
                    "E-NO-APPLICABLE-OVERLOAD",
                    operand.span(),
                    format!(
                        "application chain produced `{}` before its next operand",
                        value.value_type.name()
                    ),
                ));
            }
            let callable = self
                .known_callable(&value, environment, value.span.start)?
                .expect("checked Function chain value retains callable facts");
            let call_span = Span::new(value.span.start, operand.span().end);
            let application = [first.clone(), operand.clone()];
            let applied =
                self.analyze_callable_application(callable, &application, call_span, environment)?;
            let result_type = applied.value_type.clone();
            let int_range = applied.int_range.clone();
            let rational_value = applied.rational_value.clone();
            value = CompilerExpression {
                kind: CompilerExpressionKind::PrivateBinding {
                    storage_name: format!(
                        "topal.function.chain.{}.{}",
                        call_span.start, call_span.end
                    ),
                    value: Box::new(value),
                    body: Box::new(applied),
                },
                value_type: result_type,
                int_range,
                rational_value,
                span: call_span,
            };
        }
        value.span = span;
        Ok(Some(value))
    }

    fn record_selection_candidate(
        &self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> bool {
        match expression {
            Expression::Product { fields, .. } => {
                !fields.is_empty() && fields.iter().all(|field| field.label.is_some())
            }
            Expression::Identifier(name) => environment
                .get(self.source.slice(*name))
                .is_some_and(|facts| matches!(facts.value_type, CompilerType::Record(_))),
            _ => false,
        }
    }

    fn analyze_record_field(
        &mut self,
        record: &Expression,
        field: Span,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let record = self.analyze_expression(record, environment)?;
        let CompilerType::Record(fields) = &record.value_type else {
            unreachable!("record selection candidate retains a Record type")
        };
        let label = self.source.slice(field).to_owned();
        let value_type = fields
            .iter()
            .find_map(|(name, value_type)| (name == &label).then(|| value_type.clone()))
            .ok_or_else(|| {
                source_diagnostic(
                    &self.source,
                    "E-NO-SUCH-RECORD-FIELD",
                    field,
                    format!("record has no field named `{label}`"),
                )
            })?;
        let facts =
            Self::known_record_field_facts(&record, &label, environment).unwrap_or_default();
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::RecordField {
                record: Box::new(record),
                label,
            },
            value_type,
            int_range: facts.int_range,
            rational_value: facts.rational_value,
            span,
        })
    }

    fn analyze_record_reconstruction(
        &mut self,
        base: &Expression,
        replacements: &[ProductField],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let base_value = self.analyze_expression(base, environment)?;
        let CompilerType::Record(fields) = &base_value.value_type else {
            return Err(source_diagnostic(
                &self.source,
                "E-RECONSTRUCT-NON-RECORD",
                base.span(),
                "`with` reconstruction requires a labeled product",
            ));
        };
        let record_type = base_value.value_type.clone();
        let mut replaced = BTreeSet::new();
        let mut values = Vec::with_capacity(replacements.len());
        for replacement in replacements {
            let label_span = replacement.label.expect("preselected labeled replacement");
            let label = self.source.slice(label_span).to_owned();
            if !replaced.insert(label.clone()) {
                return Err(source_diagnostic(
                    &self.source,
                    "E-DUPLICATE-RECONSTRUCTION-FIELD",
                    label_span,
                    format!("field `{label}` is replaced more than once"),
                ));
            }
            let expected = fields
                .iter()
                .find_map(|(name, value_type)| (name == &label).then_some(value_type))
                .ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-NO-SUCH-RECORD-FIELD",
                        label_span,
                        format!("record has no field named `{label}`"),
                    )
                })?;
            let value = self.analyze_expression(&replacement.value, environment)?;
            let value = adapt_call_argument(expected, &value).ok_or_else(|| {
                source_diagnostic(
                    &self.source,
                    "E-TYPE-MISMATCH",
                    value.span,
                    format!(
                        "record field `{label}` expects {}, found {}",
                        expected.name(),
                        value.value_type.name()
                    ),
                )
            })?;
            values.push((label, value));
        }
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::RecordReconstruct {
                base: Box::new(base_value),
                replacements: values,
            },
            value_type: record_type,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn finish_optional_none(
        &self,
        payload: CompilerType,
        span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        require_optional_payload(&self.source, span, &payload)?;
        Ok(CompilerExpression {
            value_type: CompilerType::Optional(Box::new(payload)),
            kind: CompilerExpressionKind::OptionalNone,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_error_field(
        &mut self,
        error: &Expression,
        field: Span,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let error = self.analyze_expression(error, environment)?;
        if !matches!(
            &error.value_type,
            CompilerType::Error | CompilerType::Result(_)
        ) {
            return Err(source_diagnostic(
                &self.source,
                "E-TYPE-MISMATCH",
                error.span,
                format!(
                    "expected Error or Result, found {}",
                    error.value_type.name()
                ),
            ));
        }
        let (field, value_type) = match self.source.slice(field) {
            "code" => (CompilerErrorField::Code, CompilerType::ErrorCode),
            "domain" => (CompilerErrorField::Domain, CompilerType::ErrorDomain),
            "detail" => (
                CompilerErrorField::Detail,
                CompilerType::Optional(Box::new(CompilerType::String)),
            ),
            "cause" => (
                CompilerErrorField::Cause,
                CompilerType::Optional(Box::new(CompilerType::Error)),
            ),
            "source" => (
                CompilerErrorField::Source,
                CompilerType::Optional(Box::new(CompilerType::SourceLocation)),
            ),
            _ => unreachable!("implemented Error field spelling selected above"),
        };
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::ErrorField {
                error: Box::new(error),
                field,
            },
            value_type,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_int_constructor(
        &mut self,
        argument: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let value = self.analyze_expression(argument, environment)?;
        self.finish_int_conversion(value, span, argument.span())
    }

    fn finish_int_conversion(
        &self,
        value: CompilerExpression,
        span: Span,
        error_span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        if value.value_type == CompilerType::Int {
            return Ok(value);
        }
        require_type(
            &self.source,
            value.span,
            &CompilerType::Rational,
            &value.value_type,
        )?;
        if let Some(rational) = &value.rational_value {
            if rational.denom() != &BigInt::from(1) && compiler_expression_is_closed(&value) {
                return Err(source_diagnostic(
                    &self.source,
                    "E-RATIONAL-NOT-EXACT-INT",
                    error_span,
                    format!(
                        "exact Rational operand has denominator {}, so Int cannot represent it",
                        rational.denom()
                    ),
                ));
            }
            if rational.denom() == &BigInt::from(1) {
                let numerator = rational.numer().clone();
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::RationalToInt(Box::new(value)),
                    value_type: CompilerType::Int,
                    int_range: Some(IntRange::exact(numerator)),
                    rational_value: None,
                    span,
                });
            }
        }
        Ok(Self::finish_validation(
            CompilerValidation::RationalToInt,
            value,
            CompilerType::Int,
            span,
            error_span,
        ))
    }

    fn analyze_nat_constructor(
        &mut self,
        argument: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let mut value = self.analyze_expression(argument, environment)?;
        if value.value_type == CompilerType::Nat {
            value.span = span;
            return Ok(value);
        }
        self.finish_nat_conversion(value, span, argument.span())
    }

    fn finish_character_conversion(
        &self,
        mut value: CompilerExpression,
        error_span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        if value.value_type == CompilerType::Character {
            return Ok(value);
        }
        require_type(
            &self.source,
            value.span,
            &CompilerType::String,
            &value.value_type,
        )?;
        let Some(text) = exact_string(&value) else {
            return Err(unsupported(
                &self.source,
                error_span,
                "dynamic Character constraint validation",
            ));
        };
        let count = character_count(&text);
        if count != 1 {
            return Err(source_diagnostic(
                &self.source,
                "E-CHARACTER-CLASSIFIER",
                error_span,
                format!(
                    "Character requires exactly one user-perceived character, but this String contains {count}"
                ),
            ));
        }
        value.value_type = CompilerType::Character;
        Ok(value)
    }

    fn finish_proven_character_conversion(
        &self,
        mut value: CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
        error_span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        if value.value_type != CompilerType::String {
            return self.finish_character_conversion(value, error_span);
        }
        let candidates = self
            .known_structural_value_facts(&value, environment)?
            .string_characters;
        if candidates.is_some_and(|candidates| {
            !candidates.is_empty()
                && candidates
                    .iter()
                    .all(|candidate| character_count(candidate) == 1)
        }) {
            value.value_type = CompilerType::Character;
            return Ok(value);
        }
        self.finish_character_conversion(value, error_span)
    }

    fn analyze_static_character_count(
        &mut self,
        operation: &str,
        operand: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let operand_value = self.analyze_expression(operand, environment)?;
        if operation == "entry-count"
            && let CompilerType::List(element) = &operand_value.value_type
        {
            if !compiler_list_observation_element_supported(element.as_ref())
                && !compiler_nested_int_list_element(element.as_ref())
                && !compiler_nested_int_string_list_element(element.as_ref())
            {
                return Err(unsupported(
                    &self.source,
                    operand_value.span,
                    "entry-count for this List element type",
                ));
            }
            let int_range = Self::known_list_count(&operand_value, environment)
                .map(|count| IntRange::exact(BigInt::from(count)));
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListEntryCount(Box::new(operand_value)),
                value_type: CompilerType::Nat,
                int_range,
                rational_value: None,
                span,
            });
        }
        if operation == "entry-count"
            && matches!(
                operand_value.value_type,
                CompilerType::Array { .. }
                    | CompilerType::Set(_)
                    | CompilerType::Bag(_)
                    | CompilerType::Map { .. }
            )
        {
            let int_range = match &operand_value.value_type {
                CompilerType::Array { count, .. } => Some(IntRange::exact(BigInt::from(*count))),
                _ => None,
            };
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ContainerEntryCount(Box::new(operand_value)),
                value_type: CompilerType::Nat,
                int_range,
                rational_value: None,
                span,
            });
        }
        require_type(
            &self.source,
            operand_value.span,
            &CompilerType::String,
            &operand_value.value_type,
        )?;
        let Some(text) = Self::known_string_value(&operand_value, environment) else {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::StringCharacterCount(Box::new(operand_value)),
                value_type: CompilerType::Int,
                int_range: Some(IntRange {
                    lower: BigInt::from(0_u8),
                    upper: BigInt::from(u64::MAX),
                }),
                rational_value: None,
                span,
            });
        };
        let count = BigInt::from(character_count(&text));
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::Int(count.clone()),
            value_type: CompilerType::Int,
            int_range: Some(IntRange::exact(count)),
            rational_value: None,
            span,
        })
    }

    fn flatten_collection_pattern(
        &self,
        pattern: &AnonymousPattern,
        value_type: &CompilerType,
        flattened: &mut Vec<(Span, CompilerType)>,
    ) -> Result<(), Diagnostic> {
        match pattern {
            AnonymousPattern::Binding(name) => flattened.push((*name, value_type.clone())),
            AnonymousPattern::Product {
                fields,
                span: pattern_span,
            } => {
                let CompilerType::Tuple(field_types) = value_type else {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-ANONYMOUS-PRODUCT-PATTERN",
                        *pattern_span,
                        "anonymous product pattern requires a positional product",
                    ));
                };
                if fields.len() != field_types.len() {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-ANONYMOUS-FUNCTION-ARITY",
                        *pattern_span,
                        format!(
                            "collection product pattern expects {} fields, found {}",
                            fields.len(),
                            field_types.len()
                        ),
                    ));
                }
                for (field, field_type) in fields.iter().zip(field_types) {
                    self.flatten_collection_pattern(field, field_type, flattened)?;
                }
            }
        }
        Ok(())
    }

    #[allow(clippy::too_many_lines)] // Pattern binding and capture checks stay beside body analysis.
    fn analyze_collection_function(
        &mut self,
        parameters: &[AnonymousPattern],
        body: &Expression,
        parameter_types: &[CompilerType],
        outer_environment: &BTreeMap<String, BindingFacts>,
        static_context: bool,
        span: Span,
    ) -> Result<(Vec<CompilerParameter>, CompilerBlock), Diagnostic> {
        if parameters.len() != parameter_types.len() {
            return Err(source_diagnostic(
                &self.source,
                "E-ANONYMOUS-FUNCTION-ARITY",
                span,
                format!(
                    "collection function expects {} parameters, found {}",
                    parameter_types.len(),
                    parameters.len()
                ),
            ));
        }
        let mut flattened = Vec::new();
        for (parameter, value_type) in parameters.iter().zip(parameter_types) {
            self.flatten_collection_pattern(parameter, value_type, &mut flattened)?;
        }
        let mut environment = outer_environment.clone();
        let mut lowered = Vec::with_capacity(flattened.len());
        let mut declared = BTreeSet::new();
        let parameter_facts = std::mem::take(&mut self.collection_parameter_facts);
        let active_nonnegative_before_body = self.active_nonnegative_bindings.clone();
        for (parameter_index, (name_span, value_type)) in flattened.into_iter().enumerate() {
            let name = self.source.slice(name_span).to_owned();
            let discarded = name == "_";
            if !discarded && !declared.insert(name.clone()) {
                return Err(source_diagnostic(
                    &self.source,
                    "E-DUPLICATE-BINDING",
                    name_span,
                    format!("`{name}` is already declared in this parameter pattern"),
                ));
            }
            if !discarded {
                environment = decision_binding_environment(
                    &environment,
                    &name,
                    value_type.clone(),
                    name_span.start,
                );
                if let Some(facts) = parameter_facts.get(parameter_index)
                    && let Some(binding) = environment.get_mut(&name)
                {
                    retain_static_value_facts(binding, facts);
                }
                if value_type == CompilerType::Nat {
                    self.active_nonnegative_bindings.insert(name.clone());
                }
            }
            lowered.push(CompilerParameter {
                name,
                discarded,
                source_visible: !discarded,
                value_type,
                int_range: None,
                span: name_span,
            });
        }

        let previous_static_context = self.static_context;
        let previous_in_function = self.in_function;
        self.static_context = static_context;
        self.in_function = true;
        let analyzed = match body {
            Expression::Block { statements, .. } => {
                self.analyze_block(statements, &mut environment, BlockKind::Function, None)
            }
            expression => self
                .analyze_expression(expression, &environment)
                .map(|result| CompilerBlock {
                    statements: Vec::new(),
                    result,
                }),
        };
        self.static_context = previous_static_context;
        self.in_function = previous_in_function;
        self.active_nonnegative_bindings = active_nonnegative_before_body;
        Ok((lowered, analyzed?))
    }

    fn analyze_int_iterate_generator(
        &mut self,
        initial: &Expression,
        parameters: &[AnonymousPattern],
        next_body: &Expression,
        function_span: Span,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let initial = self.analyze_expression(initial, environment)?;
        require_type(
            &self.source,
            initial.span,
            &CompilerType::Int,
            &initial.value_type,
        )?;
        let consumed_before_body = self.consumed_generators.clone();
        let (parameters, next) = self.analyze_collection_function(
            parameters,
            next_body,
            &[CompilerType::Int],
            environment,
            self.static_context,
            function_span,
        )?;
        if self.consumed_generators != consumed_before_body {
            return Err(unsupported(
                &self.source,
                function_span,
                "generator capture in an iterate operation",
            ));
        }
        require_type(
            &self.source,
            next.result.span,
            &CompilerType::Int,
            &next.result.value_type,
        )?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::IterateGenerator {
                initial: Box::new(initial),
                parameters,
                next: Box::new(next),
            },
            value_type: int_unit_generator_type(),
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_int_generator_take_while(
        &mut self,
        mut generator: CompilerExpression,
        parameters: &[AnonymousPattern],
        predicate_body: &Expression,
        function_span: Span,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        require_type(
            &self.source,
            generator.span,
            &int_unit_generator_type(),
            &generator.value_type,
        )?;
        let retained = match &generator.kind {
            CompilerExpressionKind::Local(storage_name) => self.generator_values.get(storage_name),
            CompilerExpressionKind::Call { symbol, .. } => {
                self.returned_generator_values.get(symbol)
            }
            CompilerExpressionKind::PrivateBinding { body, .. } => match &body.kind {
                CompilerExpressionKind::Call { symbol, .. } => {
                    self.returned_generator_values.get(symbol)
                }
                _ => None,
            },
            _ => None,
        };
        if let Some(retained) = retained {
            generator = retained.clone();
        }
        let consumed_before_body = self.consumed_generators.clone();
        let (parameters, predicate) = self.analyze_collection_function(
            parameters,
            predicate_body,
            &[CompilerType::Int],
            environment,
            self.static_context,
            function_span,
        )?;
        if self.consumed_generators != consumed_before_body {
            return Err(unsupported(
                &self.source,
                function_span,
                "generator capture in a take-while predicate",
            ));
        }
        require_type(
            &self.source,
            predicate.result.span,
            &CompilerType::Boolean,
            &predicate.result.value_type,
        )?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::GeneratorTakeWhile {
                generator: Box::new(generator),
                parameters,
                predicate: Box::new(predicate),
            },
            value_type: int_unit_generator_type(),
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_int_list_unfold_generator(
        &mut self,
        seed: &Expression,
        parameters: &[AnonymousPattern],
        step_body: &Expression,
        function_span: Span,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let seed = self.analyze_expression(seed, environment)?;
        let seed_type = CompilerType::List(Box::new(CompilerType::Int));
        require_type(&self.source, seed.span, &seed_type, &seed.value_type)?;
        let consumed_before_body = self.consumed_generators.clone();
        let (parameters, step) = self.analyze_collection_function(
            parameters,
            step_body,
            std::slice::from_ref(&seed_type),
            environment,
            self.static_context,
            function_span,
        )?;
        if self.consumed_generators != consumed_before_body {
            return Err(unsupported(
                &self.source,
                function_span,
                "generator capture in an unfold operation",
            ));
        }
        let expected = CompilerType::Optional(Box::new(CompilerType::Tuple(vec![
            CompilerType::Int,
            seed_type,
        ])));
        require_type(
            &self.source,
            step.result.span,
            &expected,
            &step.result.value_type,
        )?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::UnfoldGenerator {
                seed: Box::new(seed),
                parameters,
                step: Box::new(step),
            },
            value_type: int_unit_generator_type(),
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_static_character_at(
        &mut self,
        text: &Expression,
        index: &Expression,
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
        let index_value = self.analyze_expression(index, environment)?;
        require_type(
            &self.source,
            index_value.span,
            &CompilerType::Int,
            &index_value.value_type,
        )?;
        let text = Self::known_string_value(&text_value, environment)
            .ok_or_else(|| unsupported(&self.source, text.span(), "dynamic Character indexing"))?;
        let exact_index = exact_int(&index_value)
            .ok_or_else(|| unsupported(&self.source, index.span(), "dynamic Character index"))?;
        let payload = usize::try_from(&exact_index)
            .ok()
            .and_then(|index| character_at(&text, index))
            .map(|character| CompilerExpression {
                kind: CompilerExpressionKind::String(character.to_owned()),
                value_type: CompilerType::Character,
                int_range: None,
                rational_value: None,
                span,
            });
        Ok(CompilerExpression {
            kind: payload.map_or(CompilerExpressionKind::OptionalNone, |character| {
                CompilerExpressionKind::OptionalSome(Box::new(character))
            }),
            value_type: CompilerType::Optional(Box::new(CompilerType::Character)),
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_static_unicode_transform(
        &mut self,
        operation: &str,
        operand: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let operand_value = self.analyze_expression(operand, environment)?;
        require_type(
            &self.source,
            operand_value.span,
            &CompilerType::String,
            &operand_value.value_type,
        )?;
        let text = Self::known_string_value(&operand_value, environment).ok_or_else(|| {
            unsupported(
                &self.source,
                operand.span(),
                "dynamic Unicode transformation",
            )
        })?;
        let transformed = match operation {
            "upper" => uppercase(&text),
            "lower" => lowercase(&text),
            "case-fold" => case_fold(&text),
            _ => unreachable!("Unicode transform spelling selected above"),
        };
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::String(transformed),
            value_type: CompilerType::String,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_static_normalization(
        &mut self,
        operand: &Expression,
        form: &str,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let operand_value = self.analyze_expression(operand, environment)?;
        require_type(
            &self.source,
            operand_value.span,
            &CompilerType::String,
            &operand_value.value_type,
        )?;
        let text = Self::known_string_value(&operand_value, environment).ok_or_else(|| {
            unsupported(
                &self.source,
                operand.span(),
                "dynamic Unicode normalization",
            )
        })?;
        let normalized = match form {
            "NFC" => normalize_nfc(&text),
            "NFD" => normalize_nfd(&text),
            _ => {
                return Err(source_diagnostic(
                    &self.source,
                    "E-NO-APPLICABLE-OVERLOAD",
                    span,
                    "the compiler normalization subset requires NFC or NFD",
                ));
            }
        };
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::String(normalized),
            value_type: CompilerType::String,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_static_canonical_equality(
        &mut self,
        left: &Expression,
        right: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let left_value = self.analyze_expression(left, environment)?;
        let right_value = self.analyze_expression(right, environment)?;
        require_type(
            &self.source,
            left_value.span,
            &CompilerType::String,
            &left_value.value_type,
        )?;
        require_type(
            &self.source,
            right_value.span,
            &CompilerType::String,
            &right_value.value_type,
        )?;
        let left_text = Self::known_string_value(&left_value, environment).ok_or_else(|| {
            unsupported(
                &self.source,
                left.span(),
                "dynamic canonical String equality",
            )
        })?;
        let right_text = Self::known_string_value(&right_value, environment).ok_or_else(|| {
            unsupported(
                &self.source,
                right.span(),
                "dynamic canonical String equality",
            )
        })?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::Boolean(canonically_equal(&left_text, &right_text)),
            value_type: CompilerType::Boolean,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn known_string_value(
        value: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Option<String> {
        Self::known_string_expression(value, environment)
    }

    fn known_closed_int_range(
        value: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Option<ClosedIntRange> {
        match &value.kind {
            CompilerExpressionKind::Binary {
                operation,
                left,
                right,
            } if matches!(
                operation,
                CompilerBinary::Range
                    | CompilerBinary::RangeOpen
                    | CompilerBinary::RangeInclusive
                    | CompilerBinary::RangeOpenInclusive
            ) =>
            {
                let left = left
                    .int_range
                    .as_ref()
                    .filter(|range| range.lower == range.upper)?;
                let right = right
                    .int_range
                    .as_ref()
                    .filter(|range| range.lower == range.upper)?;
                let (lower_inclusive, upper_inclusive) = match operation {
                    CompilerBinary::Range => (true, false),
                    CompilerBinary::RangeOpen => (false, false),
                    CompilerBinary::RangeInclusive => (true, true),
                    CompilerBinary::RangeOpenInclusive => (false, true),
                    _ => unreachable!("guard selected a Range constructor"),
                };
                Some(ClosedIntRange {
                    lower: left.lower.clone(),
                    upper: right.lower.clone(),
                    lower_inclusive,
                    upper_inclusive,
                })
            }
            CompilerExpressionKind::Local(name) => binding_facts_by_storage(environment, name)
                .and_then(|facts| facts.closed_int_range.clone()),
            _ => None,
        }
    }

    fn known_list_count(
        value: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Option<usize> {
        match &value.kind {
            CompilerExpressionKind::ListEmpty => Some(0),
            CompilerExpressionKind::ListEntry { remaining, .. } => {
                Self::known_list_count(remaining, environment)?.checked_add(1)
            }
            CompilerExpressionKind::Local(name) => {
                binding_facts_by_storage(environment, name).and_then(|facts| facts.list_count)
            }
            CompilerExpressionKind::ListInsertAt {
                list,
                inserted,
                inserts_list,
                ..
            } => Self::known_list_count(list, environment)?.checked_add(if *inserts_list {
                Self::known_list_count(inserted, environment)?
            } else {
                1
            }),
            CompilerExpressionKind::ListIndexOperation {
                list,
                index,
                operation,
            } => {
                let count = Self::known_list_count(list, environment)?;
                match operation {
                    CompilerListIndexOperation::Take => Some(*index),
                    CompilerListIndexOperation::Drop => count.checked_sub(*index),
                    CompilerListIndexOperation::Remove => count.checked_sub(1),
                    CompilerListIndexOperation::Split => None,
                }
            }
            CompilerExpressionKind::ListRemoveIndexRange { list, start, end } => {
                Self::known_list_count(list, environment)?.checked_sub(end.checked_sub(*start)?)
            }
            CompilerExpressionKind::ListZip {
                left,
                right,
                operation,
                ..
            } => {
                let left = Self::known_list_count(left, environment)?;
                let right = Self::known_list_count(right, environment)?;
                match operation {
                    CompilerListZipOperation::Exact => (left == right).then_some(left),
                    CompilerListZipOperation::Shortest => Some(left.min(right)),
                    CompilerListZipOperation::Longest => Some(left.max(right)),
                }
            }
            CompilerExpressionKind::ListEntries(list) => Self::known_list_count(list, environment),
            _ => None,
        }
    }

    fn known_list_string_keys(
        value: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Option<Vec<String>> {
        match &value.kind {
            CompilerExpressionKind::ListEmpty => Some(Vec::new()),
            CompilerExpressionKind::ListEntry { value, remaining } => {
                let CompilerExpressionKind::Tuple(fields) = &value.kind else {
                    return None;
                };
                let [key, _] = fields.as_slice() else {
                    return None;
                };
                let key = exact_string(key)?;
                let mut keys = vec![key];
                keys.extend(Self::known_list_string_keys(remaining, environment)?);
                Some(keys)
            }
            CompilerExpressionKind::Local(name) => binding_facts_by_storage(environment, name)
                .and_then(|facts| facts.list_string_keys.clone()),
            _ => None,
        }
    }

    fn exact_usize(value: &CompilerExpression) -> Option<usize> {
        let range = value
            .int_range
            .as_ref()
            .filter(|range| range.lower == range.upper)?;
        usize::try_from(&range.lower).ok()
    }

    fn specialize_exact_ordered_call(
        &self,
        function_name: &str,
        arguments: &[CompilerExpression],
        environment: &BTreeMap<String, BindingFacts>,
        span: Span,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        let Some(values) = arguments.first() else {
            return Ok(None);
        };
        let facts = self.known_structural_value_facts(values, environment)?;
        if values.value_type == int_list_type()
            && let Some(entries) = exact_int_entries(&facts)
        {
            let sought = arguments.get(1).and_then(exact_int);
            return Ok(exact_ordered_int_call(
                function_name,
                &entries,
                sought.as_ref(),
                arguments.get(1).and_then(Self::exact_usize),
                arguments.get(1).and_then(|right| {
                    self.known_structural_value_facts(right, environment)
                        .ok()
                        .and_then(|facts| exact_int_entries(&facts))
                }),
                span,
            ));
        }
        if values.value_type == CompilerType::List(Box::new(CompilerType::Rational))
            && let Some(entries) = exact_rational_entries(&facts)
        {
            let sought = arguments
                .get(1)
                .and_then(|value| value.rational_value.as_ref());
            return Ok(exact_ordered_rational_call(
                function_name,
                &entries,
                sought,
                arguments.get(1).and_then(Self::exact_usize),
                arguments.get(1).and_then(|right| {
                    self.known_structural_value_facts(right, environment)
                        .ok()
                        .and_then(|facts| exact_rational_entries(&facts))
                }),
                span,
            ));
        }
        Ok(None)
    }

    fn known_string_expression(
        value: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Option<String> {
        match &value.kind {
            CompilerExpressionKind::String(value) => Some(value.clone()),
            CompilerExpressionKind::StringEmpty => Some(String::new()),
            CompilerExpressionKind::StringConcat { left, right } => {
                let mut value = Self::known_string_expression(left, environment)?;
                value.push_str(&Self::known_string_expression(right, environment)?);
                Some(value)
            }
            CompilerExpressionKind::StringCharactersCollect { text, .. } => {
                Self::known_string_expression(text, environment)
            }
            CompilerExpressionKind::Local(name) => binding_facts_by_storage(environment, name)
                .and_then(|facts| facts.string_value.clone()),
            CompilerExpressionKind::RecordField { record, label } => {
                Self::known_record_string(record, label, environment)
            }
            CompilerExpressionKind::TupleField { tuple, index } => {
                Self::known_tuple_field_facts(tuple, *index, environment)?.string_value
            }
            _ => None,
        }
    }

    fn known_tuple_field_facts(
        value: &CompilerExpression,
        index: usize,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Option<StaticValueFacts> {
        match &value.kind {
            CompilerExpressionKind::Tuple(fields) => fields
                .get(index)
                .map(|value| Self::known_scalar_facts(value, environment)),
            CompilerExpressionKind::Local(name) => binding_facts_by_storage(environment, name)
                .and_then(|facts| facts.tuple_fields.get(index).cloned()),
            CompilerExpressionKind::TupleField {
                tuple,
                index: outer_index,
            } => Self::known_tuple_field_facts(tuple, *outer_index, environment)?
                .tuple_fields
                .get(index)
                .cloned(),
            _ => None,
        }
    }

    fn known_record_string(
        value: &CompilerExpression,
        label: &str,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Option<String> {
        Self::known_record_field_facts(value, label, environment)?.string_value
    }

    fn known_record_field_facts(
        value: &CompilerExpression,
        label: &str,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Option<StaticValueFacts> {
        match &value.kind {
            CompilerExpressionKind::Record(fields) => fields
                .iter()
                .find_map(|(name, value)| (name == label).then_some(value))
                .map(|value| Self::known_scalar_facts(value, environment)),
            CompilerExpressionKind::Local(name) => binding_facts_by_storage(environment, name)
                .and_then(|facts| facts.record_fields.get(label).cloned()),
            CompilerExpressionKind::RecordField {
                record,
                label: outer_label,
            } => Self::known_record_field_facts(record, outer_label, environment)?
                .record_fields
                .get(label)
                .cloned(),
            _ => None,
        }
    }

    fn known_record_fields(
        value: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> BTreeMap<String, StaticValueFacts> {
        match &value.kind {
            CompilerExpressionKind::Record(fields) => fields
                .iter()
                .map(|(name, value)| (name.clone(), Self::known_scalar_facts(value, environment)))
                .collect(),
            CompilerExpressionKind::RecordReconstruct { base, replacements } => {
                let mut fields = Self::known_record_fields(base, environment);
                for (name, value) in replacements {
                    fields.insert(name.clone(), Self::known_scalar_facts(value, environment));
                }
                fields
            }
            CompilerExpressionKind::Local(name) => binding_facts_by_storage(environment, name)
                .map_or_else(BTreeMap::new, |facts| facts.record_fields.clone()),
            CompilerExpressionKind::RecordField { record, label } => {
                Self::known_record_field_facts(record, label, environment)
                    .map_or_else(BTreeMap::new, |facts| facts.record_fields)
            }
            _ => BTreeMap::new(),
        }
    }

    fn known_namespace(
        &self,
        value: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
        capture_position: usize,
        kind: BlockKind,
    ) -> Result<Option<CompilerNamespaceFacts>, Diagnostic> {
        if value.value_type != CompilerType::Scope {
            return Ok(None);
        }
        if kind != BlockKind::TopLevel {
            return Err(unsupported(
                &self.source,
                value.span,
                "non-root namespace alias binding",
            ));
        }
        self.resolve_namespace(value, environment, capture_position)
    }

    fn resolve_namespace(
        &self,
        value: &CompilerExpression,
        environment: &BTreeMap<String, BindingFacts>,
        capture_position: usize,
    ) -> Result<Option<CompilerNamespaceFacts>, Diagnostic> {
        match &value.kind {
            CompilerExpressionKind::Root => Ok(Some(CompilerNamespaceFacts {
                name: "root".into(),
                bindings: self.root_bindings.clone(),
                functions: self
                    .functions
                    .iter()
                    .filter_map(|(name, declarations)| {
                        let visible = declarations
                            .iter()
                            .filter(|declaration| declaration.span.end <= capture_position)
                            .cloned()
                            .collect::<Vec<_>>();
                        (!visible.is_empty()).then(|| (name.clone(), visible))
                    })
                    .collect(),
                generators: self
                    .generators
                    .iter()
                    .filter_map(|(name, declarations)| {
                        let visible = declarations
                            .iter()
                            .filter(|declaration| declaration.span.end <= capture_position)
                            .cloned()
                            .collect::<Vec<_>>();
                        (!visible.is_empty()).then(|| (name.clone(), visible))
                    })
                    .collect(),
            })),
            CompilerExpressionKind::LintNamespace => Ok(Some(CompilerNamespaceFacts {
                name: "lang lint".into(),
                bindings: BTreeMap::new(),
                functions: BTreeMap::new(),
                generators: BTreeMap::new(),
            })),
            CompilerExpressionKind::Local(name) => binding_facts_by_storage(environment, name)
                .and_then(|facts| facts.namespace.clone())
                .map(Some)
                .ok_or_else(|| unsupported(&self.source, value.span, "opaque Scope alias")),
            _ => Err(unsupported(
                &self.source,
                value.span,
                "computed Scope alias",
            )),
        }
    }

    fn scope_parameter_arguments(
        &self,
        declaration: &FunctionSource,
        arguments: &[CompilerExpression],
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<
        (
            Vec<Option<CompilerNamespaceFacts>>,
            Vec<CompilerContextCapture>,
        ),
        Diagnostic,
    > {
        let mut namespaces = Vec::with_capacity(declaration.parameters.len());
        let mut captures = Vec::new();
        for (parameter, argument) in declaration.parameters.iter().zip(arguments) {
            if self.parse_classifier(parameter.classifier)? != CompilerType::Scope {
                namespaces.push(None);
                continue;
            }
            if self.in_function && matches!(argument.kind, CompilerExpressionKind::Root) {
                return Err(unsupported(
                    &self.source,
                    argument.span,
                    "function-body live root Scope argument",
                ));
            }
            let mut namespace = self
                .resolve_namespace(argument, environment, argument.span.start)?
                .ok_or_else(|| unsupported(&self.source, argument.span, "opaque Scope argument"))?;
            if namespace.name != "root" {
                return Err(unsupported(
                    &self.source,
                    argument.span,
                    "non-root Scope argument",
                ));
            }
            let parameter_name = self.source.slice(parameter.name);
            if parameter_name == "_" {
                namespaces.push(Some(namespace));
                continue;
            }
            let represented_members = namespace
                .bindings
                .iter()
                .map(|(member_name, facts)| (member_name.clone(), facts.clone()))
                .collect::<Vec<_>>();
            for (member_name, facts) in represented_members {
                if !compiler_function_result_supported(&facts.value_type) {
                    namespace.bindings.remove(&member_name);
                    continue;
                }
                let span = parameter.name;
                let hidden_name = format!("{parameter_name} {member_name}");
                namespace
                    .bindings
                    .get_mut(&member_name)
                    .expect("captured namespace data member exists")
                    .storage_name
                    .clone_from(&hidden_name);
                captures.push(CompilerContextCapture {
                    parameter_name: hidden_name,
                    value_type: facts.value_type.clone(),
                    int_range: facts.int_range.clone(),
                    rational_value: facts.rational_value.clone(),
                    argument: data_member_expression(&facts, span),
                    span,
                });
            }
            namespaces.push(Some(namespace));
        }
        Ok((namespaces, captures))
    }

    fn callable_parameter_arguments(
        &self,
        declaration: &FunctionSource,
        mut callables: Vec<Option<CompilerCallableFacts>>,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<
        (
            Vec<Option<CompilerCallableFacts>>,
            Vec<CompilerContextCapture>,
        ),
        Diagnostic,
    > {
        let mut forwarded = Vec::new();
        for (parameter, callable) in declaration.parameters.iter().zip(&mut callables) {
            let parameter_name = self.source.slice(parameter.name);
            if parameter_name == "_" {
                continue;
            }
            let Some(callable) = callable else {
                continue;
            };
            self.forward_callable_captures(
                parameter_name,
                parameter.name,
                callable,
                environment,
                &mut forwarded,
            )?;
        }
        Ok((callables, forwarded))
    }

    fn aggregate_parameter_arguments(
        &self,
        declaration: &FunctionSource,
        arguments: &[CompilerExpression],
        mut facts: Vec<StaticValueFacts>,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<(Vec<StaticValueFacts>, Vec<CompilerContextCapture>), Diagnostic> {
        let mut forwarded = Vec::new();
        for ((parameter, argument), facts) in
            declaration.parameters.iter().zip(arguments).zip(&mut facts)
        {
            let parameter_name = self.source.slice(parameter.name);
            if parameter_name == "_" || !compiler_type_is_function_aggregate(&argument.value_type) {
                continue;
            }
            self.forward_function_aggregate_captures(
                &argument.value_type,
                facts,
                parameter_name,
                parameter.name,
                environment,
                &mut forwarded,
            )?;
        }
        Ok((facts, forwarded))
    }

    fn named_callable_environment_captures(
        &self,
        declarations: &[FunctionSource],
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<Vec<CompilerContextCapture>, Diagnostic> {
        let mut captures = Vec::new();
        let mut retained_names = BTreeSet::new();
        for declaration in declarations {
            for capture in self.defining_context_captures(declaration, environment)? {
                if retained_names.insert(capture.parameter_name.clone()) {
                    captures.push(capture);
                }
            }
        }
        captures.sort_by_key(|capture| {
            let (group, member_name) =
                if let Some(member_name) = capture.parameter_name.strip_prefix("@ ") {
                    (0_u8, member_name)
                } else if let Some(member_name) = capture.parameter_name.strip_prefix("root ") {
                    (1_u8, member_name)
                } else {
                    (2_u8, capture.parameter_name.as_str())
                };
            let declaration_end = self
                .root_bindings
                .get(member_name)
                .map_or(usize::MAX, |facts| facts.declaration_end);
            (group, declaration_end, capture.parameter_name.clone())
        });
        Ok(captures)
    }

    fn extend_named_callable_environment_captures(
        &self,
        callable: &mut CompilerCallableFacts,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<(), Diagnostic> {
        let CompilerCallableFacts::Named {
            declarations,
            captures,
            ..
        } = callable
        else {
            return Ok(());
        };
        let mut capture_environment = environment.clone();
        for capture in captures.iter().filter(|capture| {
            capture.parameter_name.starts_with("@ ") || capture.parameter_name.starts_with("root ")
        }) {
            let (CompilerExpressionKind::Local(storage_name)
            | CompilerExpressionKind::InfinityLocal { storage_name, .. }) = &capture.argument.kind
            else {
                continue;
            };
            if let Some(facts) = binding_facts_by_storage(environment, storage_name)
                .cloned()
                .or_else(|| named_callable_capture_binding(capture))
            {
                capture_environment.insert(capture.parameter_name.clone(), facts);
            }
        }
        let retained_names = captures
            .iter()
            .map(|capture| capture.parameter_name.clone())
            .collect::<BTreeSet<_>>();
        captures.extend(
            self.named_callable_environment_captures(declarations, &capture_environment)?
                .into_iter()
                .filter(|capture| !retained_names.contains(&capture.parameter_name)),
        );
        Ok(())
    }
}
