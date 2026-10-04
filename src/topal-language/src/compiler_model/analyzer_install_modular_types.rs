impl Analyzer {
    fn install_modular_types(&mut self, declarations: &[ModularSource]) -> Result<(), Diagnostic> {
        let environment = BTreeMap::new();
        for declaration in declarations {
            let range = self.analyze_expression(&declaration.range, &environment)?;
            let CompilerExpressionKind::Binary {
                operation: CompilerBinary::RangeInclusive,
                left,
                right,
            } = &range.kind
            else {
                return Err(source_diagnostic(
                    &self.source,
                    "E-MODULAR-RANGE",
                    declaration.range.span(),
                    "ModNat and ModInt require a finite inclusive Int range",
                ));
            };
            if left.value_type != CompilerType::Int || right.value_type != CompilerType::Int {
                return Err(source_diagnostic(
                    &self.source,
                    "E-MODULAR-RANGE",
                    declaration.range.span(),
                    "ModNat and ModInt require a finite inclusive Int range",
                ));
            }
            let Some(lower) = exact_int(left) else {
                return Err(unsupported(
                    &self.source,
                    left.span,
                    "dynamic modular lower bound",
                ));
            };
            let Some(upper) = exact_int(right) else {
                return Err(unsupported(
                    &self.source,
                    right.span,
                    "dynamic modular upper bound",
                ));
            };
            if lower > BigInt::from(0)
                || upper < BigInt::from(0)
                || (!declaration.signed && lower != BigInt::from(0))
            {
                return Err(source_diagnostic(
                    &self.source,
                    "E-MODULAR-RANGE",
                    declaration.range.span(),
                    "modular range must contain zero and ModNat must begin at zero",
                ));
            }
            let name = self.source.slice(declaration.name).to_owned();
            self.modulars.insert(
                name.clone(),
                (
                    CompilerModularType {
                        name,
                        signed: declaration.signed,
                        lower,
                        upper,
                    },
                    declaration.span,
                ),
            );
        }
        Ok(())
    }

    fn interface_operation(
        &self,
        name: Span,
        parameters: &[FunctionParameter],
        result: Span,
        clauses: &FunctionClauses,
        span: Span,
    ) -> Result<CompilerInterfaceOperation, Diagnostic> {
        if clauses != &FunctionClauses::default() {
            return Err(unsupported(
                &self.source,
                span,
                "v0.2 interface contract clauses in a v0.1 compilation",
            ));
        }
        let parameters = parameters
            .iter()
            .map(|parameter| {
                if parameter.qualifier.is_some()
                    || parameter.default.is_some()
                    || !parameter.fields.is_empty()
                {
                    return Err(unsupported(
                        &self.source,
                        parameter.name,
                        "packaged, defaulted, or qualified interface parameter",
                    ));
                }
                let parameter_type = self.parse_classifier(parameter.classifier)?;
                if !compiler_function_parameter_supported(&parameter_type)
                    || compiler_type_contains_static_only(&parameter_type)
                {
                    return Err(unsupported(
                        &self.source,
                        parameter.classifier,
                        "interface parameter classifier without an admitted native function ABI",
                    ));
                }
                Ok(parameter_type)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let result_type = self.parse_classifier(result)?;
        if !compiler_function_result_supported(&result_type)
            || compiler_type_contains_static_only(&result_type)
        {
            return Err(unsupported(
                &self.source,
                result,
                "interface result classifier without an admitted native function ABI",
            ));
        }
        Ok(CompilerInterfaceOperation {
            name: self.source.slice(name).to_owned(),
            parameters,
            result: result_type,
        })
    }

    fn install_interfaces(
        &mut self,
        declarations: &BTreeMap<String, InterfaceSource>,
    ) -> Result<(), Diagnostic> {
        for (name, declaration) in declarations {
            let mut operations = declaration
                .functions
                .iter()
                .map(|function| {
                    self.interface_operation(
                        function.name,
                        &function.parameters,
                        function.result,
                        &function.clauses,
                        function.span,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            operations.sort_by(|left, right| left.name.cmp(&right.name));
            self.interfaces.insert(
                name.clone(),
                (
                    CompilerInterface {
                        identity: format!("root.{name}"),
                        operations,
                    },
                    declaration.span,
                ),
            );
        }
        Ok(())
    }

    fn validate_interface_implementations(
        &mut self,
        statements: &[Statement],
    ) -> Result<(), Diagnostic> {
        for statement in statements {
            let Statement::InterfaceImplementation {
                interface,
                declarations,
                span,
            } = statement
            else {
                continue;
            };
            let implementation =
                self.validate_interface_implementation(*interface, declarations, *span)?;
            self.interface_implementations.push(implementation);
        }
        Ok(())
    }

    fn validate_interface_implementation(
        &self,
        interface: Span,
        declarations: &[Statement],
        span: Span,
    ) -> Result<CompilerInterfaceImplementation, Diagnostic> {
        let interface_name = self.source.slice(interface);
        let (shape, _) = self
            .interfaces
            .get(interface_name)
            .filter(|(_, declaration_span)| declaration_span.end <= interface.start)
            .ok_or_else(|| {
                source_diagnostic(
                    &self.source,
                    "E-UNKNOWN-INTERFACE",
                    interface,
                    format!("`{interface_name}` is not a declared interface"),
                )
            })?;
        let mut supplied = BTreeMap::new();
        for declaration in declarations {
            let Statement::Function {
                name,
                is_static,
                parameters,
                result,
                effect_bound,
                clauses,
                span: function_span,
                ..
            } = declaration
            else {
                return Err(source_diagnostic(
                    &self.source,
                    "E-INTERFACE-IMPLEMENTATION",
                    span,
                    "an interface implementation contains function declarations only",
                ));
            };
            if *is_static {
                return Err(unsupported(
                    &self.source,
                    *function_span,
                    "static interface implementation function",
                ));
            }
            let declared_effects = compiler_declared_effect_row(&self.source, *effect_bound)?;
            let actual =
                self.interface_operation(*name, parameters, *result, clauses, *function_span)?;
            let operation_name = actual.name.clone();
            if supplied.contains_key(&operation_name) {
                return Err(source_diagnostic(
                    &self.source,
                    "E-INTERFACE-IMPLEMENTATION",
                    *name,
                    format!("interface operation `{operation_name}` is implemented more than once"),
                ));
            }
            let inputs = parameters
                .iter()
                .map(|parameter| compact_classifier(self.source.slice(parameter.classifier)))
                .collect::<Vec<_>>()
                .join(",");
            supplied.insert(
                operation_name,
                (
                    actual,
                    CompilerInterfaceOperationEvidence {
                        role: self.source.slice(*name).to_owned(),
                        declaration_identity: format!(
                            "root.{}:ordinary({inputs})",
                            self.source.slice(*name)
                        ),
                        declared_effects,
                    },
                ),
            );
        }
        if supplied.len() != shape.operations.len()
            || shape.operations.iter().any(|expected| {
                supplied
                    .get(&expected.name)
                    .is_none_or(|(actual, _)| actual != expected)
            })
        {
            return Err(source_diagnostic(
                &self.source,
                "E-INTERFACE-IMPLEMENTATION",
                span,
                "implementation operations must exactly match the interface shapes",
            ));
        }
        Ok(CompilerInterfaceImplementation {
            interface_identity: shape.identity.clone(),
            operations: supplied
                .into_values()
                .map(|(_, evidence)| evidence)
                .collect(),
        })
    }

    fn analyze_contextual_tuple(
        &mut self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
        expected: Option<&CompilerType>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        let Some(CompilerType::Tuple(expected_fields)) = expected else {
            return Ok(None);
        };
        let Expression::Product { fields, span } = expression else {
            return Ok(None);
        };
        if fields.len() != expected_fields.len() || fields.iter().any(|field| field.label.is_some())
        {
            return Ok(None);
        }
        let values = fields
            .iter()
            .zip(expected_fields)
            .map(|(field, expected)| {
                self.analyze_expression_with_expected(&field.value, environment, Some(expected))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Some(CompilerExpression {
            kind: CompilerExpressionKind::Tuple(values),
            value_type: CompilerType::Tuple(expected_fields.clone()),
            int_range: None,
            rational_value: None,
            span: *span,
        }))
    }

    fn finish_contextual_scalar(
        &mut self,
        value: CompilerExpression,
        expected: Option<&CompilerType>,
        span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        match expected {
            Some(expected @ CompilerType::Result(success)) if value.value_type == **success => {
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::ResultSuccess(Box::new(value)),
                    value_type: expected.clone(),
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Some(expected @ CompilerType::Result(_)) if value.value_type == CompilerType::Error => {
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::ResultError(Box::new(value)),
                    value_type: expected.clone(),
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Some(CompilerType::Nat) if value.value_type == CompilerType::Int => {
                self.finish_nat_conversion(value, span, span)
            }
            Some(CompilerType::Character) if value.value_type == CompilerType::String => {
                self.finish_character_conversion(value, span)
            }
            Some(CompilerType::String) if value.value_type == CompilerType::Character => {
                Ok(forget_character_evidence(value))
            }
            _ => Ok(value),
        }
    }

    fn analyze_contextual_list(
        &mut self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
        expected: Option<&CompilerType>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        let Some(CompilerType::List(element)) = expected else {
            return Ok(None);
        };
        if let Expression::Identifier(name) = expression
            && self.source.slice(*name) == "Empty"
        {
            return Ok(Some(CompilerExpression {
                kind: CompilerExpressionKind::ListEmpty,
                value_type: CompilerType::List(element.clone()),
                int_range: None,
                rational_value: None,
                span: expression.span(),
            }));
        }
        let Expression::Application { items, span } = expression else {
            return Ok(None);
        };
        if let [Expression::Identifier(constructor), value] = items.as_slice()
            && self.source.slice(*constructor) == "one"
        {
            let value = self.analyze_expression_with_expected(value, environment, Some(element))?;
            let value = self.finish_list_entry_value(value, element)?;
            let list_type = CompilerType::List(element.clone());
            return Ok(Some(CompilerExpression {
                kind: CompilerExpressionKind::ListEntry {
                    value: Box::new(value),
                    remaining: Box::new(CompilerExpression {
                        kind: CompilerExpressionKind::ListEmpty,
                        value_type: list_type.clone(),
                        int_range: None,
                        rational_value: None,
                        span: *span,
                    }),
                },
                value_type: list_type,
                int_range: None,
                rational_value: None,
                span: *span,
            }));
        }
        let [
            Expression::Identifier(constructor),
            Expression::Product { fields, .. },
        ] = items.as_slice()
        else {
            return Ok(None);
        };
        if self.source.slice(*constructor) != "Entry" {
            return Ok(None);
        }
        let [value, remaining] = fields.as_slice() else {
            return Ok(None);
        };
        if value.label.is_some() || remaining.label.is_some() {
            return Ok(None);
        }
        let value =
            self.analyze_expression_with_expected(&value.value, environment, Some(element))?;
        let value = self.finish_list_entry_value(value, element)?;
        let list_type = CompilerType::List(element.clone());
        let remaining =
            self.analyze_expression_with_expected(&remaining.value, environment, Some(&list_type))?;
        require_same_type(
            &self.source,
            remaining.span,
            &list_type,
            &remaining.value_type,
        )?;
        Ok(Some(CompilerExpression {
            kind: CompilerExpressionKind::ListEntry {
                value: Box::new(value),
                remaining: Box::new(remaining),
            },
            value_type: list_type,
            int_range: None,
            rational_value: None,
            span: *span,
        }))
    }

    #[allow(clippy::too_many_lines)] // Contextual Result decisions stay beside scalar and collection adaptation.
    fn analyze_expression_with_expected(
        &mut self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
        expected: Option<&CompilerType>,
    ) -> Result<CompilerExpression, Diagnostic> {
        if let Expression::Infinity(span) = expression {
            let negative = self.source.slice(*span).starts_with('-');
            let value_type = match expected {
                Some(CompilerType::Int) => CompilerType::InfiniteInt,
                Some(CompilerType::Nat) if !negative => CompilerType::InfiniteNat,
                Some(CompilerType::Nat) => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-INFINITY-CLASSIFIER",
                        *span,
                        "-Infinity does not satisfy Nat",
                    ));
                }
                Some(CompilerType::Rational) => CompilerType::InfiniteRational,
                _ => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-INFINITY-CONTEXT",
                        *span,
                        "this compiler increment requires an explicit Int, Nat, or Rational infinity classifier",
                    ));
                }
            };
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::Infinity { negative },
                value_type,
                int_range: None,
                rational_value: None,
                span: *span,
            });
        }
        if let Expression::Identifier(name) = expression
            && self.source.slice(*name) == "None"
            && let Some(CompilerType::Optional(payload)) = expected
        {
            return self.finish_optional_none(payload.as_ref().clone(), expression.span());
        }
        if let Some(value) = self.analyze_contextual_tuple(expression, environment, expected)? {
            return Ok(value);
        }
        if let Some(value) = self.analyze_contextual_list(expression, environment, expected)? {
            return Ok(value);
        }
        if expected == Some(&CompilerType::Nat)
            && let Expression::DecisionTable {
                subject,
                rules,
                span,
            } = expression
        {
            let subject = self.analyze_expression(subject, environment)?;
            let subject_type = subject.value_type.clone();
            return match subject_type {
                CompilerType::Int | CompilerType::Rational => self
                    .analyze_ordered_comparison_decision(
                        subject,
                        rules,
                        *span,
                        environment,
                        expected,
                    ),
                CompilerType::Nat => self.analyze_ordered_comparison_decision(
                    forget_nat_evidence(subject),
                    rules,
                    *span,
                    environment,
                    expected,
                ),
                CompilerType::Optional(payload) => self.analyze_optional_decision(
                    subject,
                    payload.as_ref(),
                    rules,
                    *span,
                    environment,
                    expected,
                ),
                _ => self
                    .analyze_expression(expression, environment)
                    .and_then(|value| {
                        self.finish_contextual_scalar(value, expected, expression.span())
                    }),
            };
        }
        if let Some(expected @ CompilerType::Result(_)) = expected
            && let Expression::DecisionTable {
                subject,
                rules,
                span,
            } = expression
        {
            let subject = self.analyze_expression(subject, environment)?;
            if let CompilerType::Result(success) = subject.value_type.clone() {
                return self.analyze_result_decision(
                    subject,
                    success.as_ref(),
                    rules,
                    *span,
                    environment,
                    Some(expected),
                );
            }
        }
        let value = self.analyze_expression(expression, environment)?;
        self.finish_contextual_scalar(value, expected, expression.span())
    }

    fn finish_list_entry_value(
        &mut self,
        mut value: CompilerExpression,
        element: &CompilerType,
    ) -> Result<CompilerExpression, Diagnostic> {
        if element == &CompilerType::Nat && value.value_type == CompilerType::Int {
            let span = value.span;
            value = self.finish_nat_conversion(value, span, span)?;
        }
        if !matches!(
            (element, &value.value_type),
            (CompilerType::Nat, CompilerType::InfiniteNat)
                | (CompilerType::Rational, CompilerType::InfiniteRational)
        ) {
            require_same_type(&self.source, value.span, element, &value.value_type)?;
        }
        Ok(value)
    }

    fn finish_required_list_entry_value(
        &self,
        value: CompilerExpression,
        element: &CompilerType,
    ) -> Result<CompilerExpression, Diagnostic> {
        if element == &CompilerType::Nat && value.value_type == CompilerType::Int {
            let span = value.span;
            if value
                .int_range
                .as_ref()
                .is_some_and(|range| range.lower >= BigInt::from(0))
            {
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::IntToNat(Box::new(value)),
                    value_type: CompilerType::Nat,
                    int_range: None,
                    rational_value: None,
                    span,
                });
            }
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::IntToNatBoundary(Box::new(value)),
                value_type: CompilerType::Nat,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        require_same_type(&self.source, value.span, element, &value.value_type)?;
        Ok(value)
    }

    fn parse_classifier(&self, span: Span) -> Result<CompilerType, Diagnostic> {
        let classifier = compact_classifier(self.source.slice(span));
        if classifier == "DecimalText"
            && self
                .source
                .slice(Span::new(0, span.start))
                .contains("DecimalText is String constraint")
        {
            return Ok(CompilerType::Refined {
                constraint: classifier,
                base: Box::new(CompilerType::String),
            });
        }
        if !self.classifier_substitutions.is_empty()
            && let Some(value_type) =
                parse_substituted_classifier(&classifier, &self.classifier_substitutions, &|name| {
                    self.enums
                        .get(name)
                        .filter(|(_, declaration)| declaration.end <= span.start)
                        .map(|(enumeration, _)| CompilerType::Enum(enumeration.clone()))
                        .or_else(|| {
                            self.modulars
                                .get(name)
                                .filter(|(_, declaration)| declaration.end <= span.start)
                                .map(|(modular, _)| CompilerType::Modular(modular.clone()))
                        })
                        .or_else(|| {
                            self.sums
                                .get(name)
                                .filter(|(_, declaration)| declaration.end <= span.start)
                                .map(|(sum, _)| CompilerType::Sum(sum.clone()))
                        })
                })
        {
            return Ok(value_type);
        }
        if let Some((enumeration, declaration)) = self.enums.get(&classifier)
            && declaration.end <= span.start
        {
            return Ok(CompilerType::Enum(enumeration.clone()));
        }
        if let Some((sum, declaration)) = self.sums.get(&classifier)
            && declaration.end <= span.start
        {
            return Ok(CompilerType::Sum(sum.clone()));
        }
        if let Some((modular, declaration)) = self.modulars.get(&classifier)
            && declaration.end <= span.start
        {
            return Ok(CompilerType::Modular(modular.clone()));
        }
        if let Some((success, codes)) = classifier
            .strip_prefix("Result(")
            .and_then(|value| value.strip_suffix(')'))
            .and_then(split_classifier_once)
            && codes == "langarithmeticArithmeticErrorCode"
            && let Some((modular, declaration)) = self.modulars.get(success)
            && declaration.end <= span.start
        {
            return Ok(CompilerType::Result(Box::new(CompilerType::Modular(
                modular.clone(),
            ))));
        }
        if let Some(tag) = self.constraint_bindings.get(&classifier)
            && let Some(constraint) = self
                .constraints
                .get(usize::try_from(*tag).expect("u32 tag fits usize"))
            && constraint.span.end <= span.start
        {
            return Ok(CompilerType::Refined {
                constraint: classifier,
                base: Box::new(constraint.base_type.clone()),
            });
        }
        parse_compact_classifier_with(&classifier, &|name| {
            self.enums
                .get(name)
                .filter(|(_, declaration)| declaration.end <= span.start)
                .map(|(enumeration, _)| CompilerType::Enum(enumeration.clone()))
                .or_else(|| {
                    self.modulars
                        .get(name)
                        .filter(|(_, declaration)| declaration.end <= span.start)
                        .map(|(modular, _)| CompilerType::Modular(modular.clone()))
                })
                .or_else(|| {
                    self.sums
                        .get(name)
                        .filter(|(_, declaration)| declaration.end <= span.start)
                        .map(|(sum, _)| CompilerType::Sum(sum.clone()))
                })
        })
        .ok_or_else(|| unsupported(&self.source, span, "classifier"))
    }

    fn specialize_classifier(
        &self,
        span: Span,
        actual: &CompilerType,
        substitutions: &mut BTreeMap<String, CompilerType>,
    ) -> Result<Option<CompilerType>, Diagnostic> {
        let classifier = compact_classifier(self.source.slice(span));
        let generic_pattern = classifier_uses_substitution(&classifier, substitutions);
        if infer_classifier_substitutions(&classifier, actual, substitutions, &|name| {
            self.enums
                .get(name)
                .map(|(enumeration, _)| CompilerType::Enum(enumeration.clone()))
                .or_else(|| {
                    self.modulars
                        .get(name)
                        .map(|(modular, _)| CompilerType::Modular(modular.clone()))
                })
                .or_else(|| {
                    self.sums
                        .get(name)
                        .map(|(sum, _)| CompilerType::Sum(sum.clone()))
                })
        }) {
            if generic_pattern {
                Ok(Some(actual.clone()))
            } else {
                self.parse_classifier(span).map(Some)
            }
        } else if generic_pattern {
            Ok(None)
        } else {
            self.parse_classifier(span).map(Some)
        }
    }

    fn is_declaration(&self, statement: &Statement) -> bool {
        if matches!(
            statement,
            Statement::LanguageSelection { .. }
                | Statement::Function { .. }
                | Statement::Generator { .. }
                | Statement::Interface { .. }
                | Statement::InterfaceImplementation { .. }
        ) || matches!(statement, Statement::Published { declaration, .. } if matches!(declaration.as_ref(), Statement::Function { .. } | Statement::Interface { .. }))
        {
            return true;
        }
        let span = statement_span(statement);
        if self
            .task_types
            .values()
            .any(|declaration| declaration.span == span)
            || self
                .task_definitions
                .values()
                .any(|declaration| declaration.span == span)
        {
            return true;
        }
        if let Some(declaration) = enum_declaration(&self.source, statement) {
            let name = self.source.slice(declaration.name);
            return self
                .enums
                .get(name)
                .is_some_and(|(_, span)| *span == declaration.span);
        }
        if let Some(declaration) = sum_declaration(&self.source, statement) {
            let name = self.source.slice(declaration.name);
            return self
                .sums
                .get(name)
                .is_some_and(|(_, span)| *span == declaration.span);
        }
        if let Some(declaration) = modular_declaration(&self.source, statement) {
            let name = self.source.slice(declaration.name);
            return self
                .modulars
                .get(name)
                .is_some_and(|(_, span)| *span == declaration.span);
        }
        false
    }

    #[allow(clippy::too_many_lines)] // Capture validation keeps every nested-function boundary explicit.
    fn bind_nested_function(
        &mut self,
        statement: &Statement,
        environment: &mut BTreeMap<String, BindingFacts>,
        declared: &mut BTreeSet<String>,
    ) -> Result<(), Diagnostic> {
        let Statement::Function {
            name,
            is_static,
            parameters,
            result,
            effect_bound,
            clauses,
            body,
            span,
        } = statement
        else {
            return Err(unsupported(
                &self.source,
                statement_span(statement),
                "published nested function declaration",
            ));
        };
        if self.static_context
            || *is_static
            || effect_bound.is_some()
            || **clauses != FunctionClauses::default()
        {
            return Err(unsupported(
                &self.source,
                *span,
                "static, measured, constrained, or effectful nested function",
            ));
        }
        let name_text = self.source.slice(*name).to_owned();
        if declared.contains(&name_text)
            || self.functions.contains_key(&name_text)
            || self.active_calls.iter().any(|identity| {
                identity
                    .split_once(':')
                    .is_some_and(|(name, _)| name == name_text)
            })
        {
            return Err(source_diagnostic(
                &self.source,
                "E-DUPLICATE-BINDING",
                *name,
                format!("`{name_text}` is already declared in this invocation scope"),
            ));
        }
        let parameter_names = parameters
            .iter()
            .map(|parameter| self.source.slice(parameter.name))
            .collect::<BTreeSet<_>>();
        let static_callable_environment = environment
            .iter()
            .filter_map(|(candidate, facts)| {
                let CompilerCallableFacts::Named {
                    declarations,
                    captures,
                    ..
                } = facts.callable.as_ref()?
                else {
                    return None;
                };
                if parameter_names.contains(candidate.as_str())
                    || !body_mentions_name(&self.source, body, candidate)
                    || facts.runtime_bound
                    || !captures.iter().all(|capture| {
                        declarations.iter().all(|declaration| {
                            !body_mentions_name(
                                &self.source,
                                &declaration.body,
                                &capture.parameter_name,
                            )
                        })
                    })
                {
                    return None;
                }
                let mut facts = facts.clone();
                let Some(CompilerCallableFacts::Named { captures, .. }) = facts.callable.as_mut()
                else {
                    unreachable!("checked named callable retains its identity")
                };
                captures.clear();
                Some((candidate.clone(), facts))
            })
            .collect::<BTreeMap<_, _>>();
        if let Some((candidate, _)) = environment.iter().find(|(candidate, facts)| {
            !parameter_names.contains(candidate.as_str())
                && body_mentions_name(&self.source, body, candidate)
                && (!static_callable_environment.contains_key(candidate.as_str())
                    && (!facts.runtime_bound
                        || !compiler_environment_capture_supported(&facts.value_type)))
        }) {
            return Err(unsupported(
                &self.source,
                *name,
                &format!(
                    "nested Function capture `{candidate}` without an admitted private representation"
                ),
            ));
        }
        let captures = environment
            .iter()
            .filter(|(candidate, facts)| {
                facts.runtime_bound
                    && !candidate.starts_with("@ ")
                    && !candidate.starts_with("root ")
                    && !parameter_names.contains(candidate.as_str())
                    && compiler_environment_capture_supported(&facts.value_type)
            })
            .map(|(candidate, facts)| CompilerContextCapture {
                parameter_name: candidate.clone(),
                value_type: facts.value_type.clone(),
                int_range: facts.int_range.clone(),
                rational_value: facts.rational_value.clone(),
                argument: CompilerExpression {
                    kind: CompilerExpressionKind::Local(facts.storage_name.clone()),
                    value_type: facts.value_type.clone(),
                    int_range: facts.int_range.clone(),
                    rational_value: facts.rational_value.clone(),
                    span: *name,
                },
                span: *name,
            })
            .collect();
        let declaration = FunctionSource {
            name: *name,
            parameters: parameters.clone(),
            result: *result,
            effect_bound: None,
            declared_effects: None,
            body: body.clone(),
            span: *span,
            is_static: false,
            published: false,
            module_identity: None,
        };
        if !static_callable_environment.is_empty() {
            self.nested_static_environments
                .insert(span.start, static_callable_environment);
        }
        environment.insert(
            name_text.clone(),
            BindingFacts {
                storage_name: format!("topal.nested.function.{}.{}", name.start, name_text),
                origin: name.start,
                runtime_bound: false,
                value_type: CompilerType::Function,
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
                callable: Some(CompilerCallableFacts::Named {
                    name: name_text.clone(),
                    declarations: vec![declaration],
                    captures,
                }),
                static_capability: None,
            },
        );
        declared.insert(name_text);
        Ok(())
    }

    fn analyze_direct_statement_expression(
        &mut self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
        expected: Option<&CompilerType>,
        function_result: Option<&CompilerType>,
        allow_function_return: bool,
    ) -> Result<(CompilerExpression, bool), Diagnostic> {
        if allow_function_return
            && let Some(value) = self.analyze_returning_embedded_expression(
                expression,
                environment,
                function_result,
            )?
        {
            return Ok((value, true));
        }
        let Expression::Block { statements, .. } = expression else {
            return self
                .analyze_expression_with_expected(expression, environment, expected)
                .map(|value| (value, false));
        };
        let span = expression.span();
        let mut nested = environment.clone();
        let mut analyzed = self.analyze_block_control(
            statements,
            &mut nested,
            BlockKind::Lexical,
            expected,
            function_result,
            allow_function_return,
        )?;
        if statements.is_empty() {
            analyzed.block.result = unit_expression(span);
        }
        let returns_from_function = analyzed.returns_from_function;
        let mut value = CompilerExpression {
            value_type: analyzed.block.result.value_type.clone(),
            int_range: analyzed.block.result.int_range.clone(),
            rational_value: analyzed.block.result.rational_value.clone(),
            kind: CompilerExpressionKind::Block(Box::new(analyzed.block)),
            span,
        };
        if !returns_from_function {
            value = match expected {
                Some(CompilerType::Character) if value.value_type == CompilerType::String => {
                    self.finish_character_conversion(value, span)?
                }
                Some(CompilerType::String) if value.value_type == CompilerType::Character => {
                    forget_character_evidence(value)
                }
                _ => value,
            };
        }
        Ok((value, returns_from_function))
    }

    #[allow(clippy::too_many_lines)] // Admission order stays explicit and fail-closed.
    fn analyze_returning_embedded_expression(
        &mut self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
        function_result: Option<&CompilerType>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        if let Some(value) =
            self.analyze_returning_decision_subject(expression, environment, function_result)?
        {
            return Ok(Some(value));
        }
        if let Some(value) = self.analyze_returning_boolean_decision_actions(
            expression,
            environment,
            function_result,
        )? {
            return Ok(Some(value));
        }
        if let Some(value) = self.analyze_returning_enum_fallback_decision_actions(
            expression,
            environment,
            function_result,
        )? {
            return Ok(Some(value));
        }
        if let Some(value) = self.analyze_returning_exhaustive_enum_decision_actions(
            expression,
            environment,
            function_result,
        )? {
            return Ok(Some(value));
        }
        if let Some(value) = self.analyze_returning_optional_decision_actions(
            expression,
            environment,
            function_result,
        )? {
            return Ok(Some(value));
        }
        if let Some(value) = self.analyze_returning_result_decision_actions(
            expression,
            environment,
            function_result,
        )? {
            return Ok(Some(value));
        }
        if let Some(value) = self.analyze_returning_error_code_decision_actions(
            expression,
            environment,
            function_result,
        )? {
            return Ok(Some(value));
        }
        if let Some(value) =
            self.analyze_returning_list_decision_actions(expression, environment, function_result)?
        {
            return Ok(Some(value));
        }
        if let Some(value) = self.analyze_returning_comparison_value_decision_actions(
            expression,
            environment,
            function_result,
        )? {
            return Ok(Some(value));
        }
        if let Some(value) = self.analyze_returning_ordered_comparison_decision_actions(
            expression,
            environment,
            function_result,
        )? {
            return Ok(Some(value));
        }
        if let Some(value) =
            self.analyze_returning_product_field(expression, environment, function_result)?
        {
            return Ok(Some(value));
        }
        if let Some(value) =
            self.analyze_returning_collection_source(expression, environment, function_result)?
        {
            return Ok(Some(value));
        }
        if let Some(value) = self.analyze_returning_modular_reduction_operand(
            expression,
            environment,
            function_result,
        )? {
            return Ok(Some(value));
        }
        if let Some(value) =
            self.analyze_returning_operator_operand(expression, environment, function_result)?
        {
            return Ok(Some(value));
        }
        if let Some(value) = self.analyze_returning_unary_constructor_argument(
            expression,
            environment,
            function_result,
        )? {
            return Ok(Some(value));
        }
        if let Some(value) = self.analyze_returning_variant_constructor_argument(
            expression,
            environment,
            function_result,
        )? {
            return Ok(Some(value));
        }
        self.analyze_returning_named_call_argument(expression, environment, function_result)
    }

    fn analyze_returning_decision_subject(
        &mut self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
        function_result: Option<&CompilerType>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        let Expression::DecisionTable { subject, .. } = expression else {
            return Ok(None);
        };
        if !direct_expression_returns_from_function(subject) {
            return Ok(None);
        }
        let (result, returned) = self.analyze_direct_statement_expression(
            subject,
            environment,
            function_result,
            function_result,
            true,
        )?;
        assert!(
            returned,
            "a checked returning decision subject exits its function"
        );
        Ok(Some(result))
    }

    fn analyze_returning_boolean_decision_actions(
        &mut self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
        function_result: Option<&CompilerType>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        let Expression::DecisionTable {
            subject,
            rules,
            span,
        } = expression
        else {
            return Ok(None);
        };
        if !is_supported_returning_boolean_action_shape(rules) {
            return Ok(None);
        }
        let Some(function_result) = function_result else {
            return Ok(None);
        };
        let subject = self.analyze_expression(subject, environment)?;
        if subject.value_type != CompilerType::Boolean {
            if matches!(subject.value_type, CompilerType::Optional(_))
                && matches!(rules.as_slice(), [rule] if matches!(rule.matcher, DecisionMatcher::Otherwise(_)))
            {
                return Ok(None);
            }
            return Err(unsupported(
                &self.source,
                subject.span,
                "all-returning decision actions for a non-Boolean subject",
            ));
        }
        let mut when_true = None;
        let mut when_false = None;
        let mut otherwise = None;
        for rule in rules {
            let (action, returned) = self.analyze_direct_statement_expression(
                &rule.action,
                environment,
                Some(function_result),
                Some(function_result),
                true,
            )?;
            assert!(
                returned,
                "a checked returning decision action exits its function"
            );
            match rule.matcher {
                DecisionMatcher::Boolean { value: true, .. } => when_true = Some(action),
                DecisionMatcher::Boolean { value: false, .. } => when_false = Some(action),
                DecisionMatcher::Otherwise(_) => otherwise = Some(action),
                _ => unreachable!("preselected complete Boolean decision shape"),
            }
        }
        let when_true = when_true
            .or_else(|| otherwise.clone())
            .expect("complete true action");
        let when_false = when_false.or(otherwise).expect("complete false action");
        let (value_type, int_range, rational_value) =
            self.decision_facts(&[&when_true, &when_false], *span)?;
        Ok(Some(CompilerExpression {
            value_type,
            kind: CompilerExpressionKind::BooleanDecision {
                subject: Box::new(subject),
                when_true: Box::new(when_true),
                when_false: Box::new(when_false),
            },
            int_range,
            rational_value,
            span: *span,
        }))
    }

    fn analyze_returning_comparison_value_decision_actions(
        &mut self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
        function_result: Option<&CompilerType>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        let Expression::DecisionTable {
            subject,
            rules,
            span,
        } = expression
        else {
            return Ok(None);
        };
        if !is_supported_returning_comparison_value_action_shape(&self.source, rules) {
            return Ok(None);
        }
        let Some(function_result) = function_result else {
            return Ok(None);
        };
        let subject = self.analyze_expression(subject, environment)?;
        if subject.value_type != CompilerType::Comparison {
            return Err(unsupported(
                &self.source,
                subject.span,
                "all-returning Comparison actions for a non-Comparison subject",
            ));
        }
        let mut when_less = None;
        let mut when_equal = None;
        let mut when_greater = None;
        let mut otherwise = None;
        for rule in rules {
            let (action, returned) = self.analyze_direct_statement_expression(
                &rule.action,
                environment,
                Some(function_result),
                Some(function_result),
                true,
            )?;
            assert!(
                returned,
                "a checked returning Comparison action exits its function"
            );
            match rule.matcher {
                DecisionMatcher::Identifier(matcher) => match self.source.slice(matcher) {
                    "Less" => when_less = Some(action),
                    "Equal" => when_equal = Some(action),
                    "Greater" => when_greater = Some(action),
                    _ => unreachable!("preselected Comparison alternative"),
                },
                DecisionMatcher::Otherwise(_) => otherwise = Some(action),
                _ => unreachable!("preselected complete Comparison decision shape"),
            }
        }
        let when_less = when_less
            .or_else(|| otherwise.clone())
            .expect("complete Less action");
        let when_equal = when_equal
            .or_else(|| otherwise.clone())
            .expect("complete Equal action");
        let when_greater = when_greater.or(otherwise).expect("complete Greater action");
        let (value_type, int_range, rational_value) =
            self.decision_facts(&[&when_less, &when_equal, &when_greater], *span)?;
        Ok(Some(CompilerExpression {
            value_type,
            kind: CompilerExpressionKind::ComparisonValueDecision {
                subject: Box::new(subject),
                when_less: Box::new(when_less),
                when_equal: Box::new(when_equal),
                when_greater: Box::new(when_greater),
            },
            int_range,
            rational_value,
            span: *span,
        }))
    }

    #[allow(clippy::too_many_lines)] // Enum validation and return-aware action analysis stay adjacent.
    fn analyze_returning_enum_fallback_decision_actions(
        &mut self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
        function_result: Option<&CompilerType>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        let Expression::DecisionTable {
            subject,
            rules,
            span,
        } = expression
        else {
            return Ok(None);
        };
        if !is_supported_returning_enum_fallback_action_shape(&self.source, rules) {
            return Ok(None);
        }
        let Some(function_result) = function_result else {
            return Ok(None);
        };
        let subject = self.analyze_expression(subject, environment)?;
        match subject.value_type.clone() {
            CompilerType::Comparison => {
                let mut when_less = None;
                let mut when_equal = None;
                let mut when_greater = None;
                let mut otherwise = None;
                for rule in rules {
                    let (action, returned) = self.analyze_direct_statement_expression(
                        &rule.action,
                        environment,
                        Some(function_result),
                        Some(function_result),
                        true,
                    )?;
                    assert!(
                        returned,
                        "a checked returning Enum action exits its function"
                    );
                    match rule.matcher {
                        DecisionMatcher::Identifier(matcher) => match self.source.slice(matcher) {
                            "Less" => when_less = Some(action),
                            "Equal" => when_equal = Some(action),
                            "Greater" => when_greater = Some(action),
                            _ => {
                                return Err(unsupported(
                                    &self.source,
                                    matcher,
                                    "Comparison alternative",
                                ));
                            }
                        },
                        DecisionMatcher::Otherwise(_) => otherwise = Some(action),
                        _ => unreachable!("preselected final-fallback Enum decision shape"),
                    }
                }
                let otherwise = otherwise.expect("final Comparison fallback");
                let when_less = when_less.unwrap_or_else(|| otherwise.clone());
                let when_equal = when_equal.unwrap_or_else(|| otherwise.clone());
                let when_greater = when_greater.unwrap_or(otherwise);
                let (value_type, int_range, rational_value) =
                    self.decision_facts(&[&when_less, &when_equal, &when_greater], *span)?;
                Ok(Some(CompilerExpression {
                    value_type,
                    kind: CompilerExpressionKind::ComparisonValueDecision {
                        subject: Box::new(subject),
                        when_less: Box::new(when_less),
                        when_equal: Box::new(when_equal),
                        when_greater: Box::new(when_greater),
                    },
                    int_range,
                    rational_value,
                    span: *span,
                }))
            }
            CompilerType::Enum(enumeration) => {
                let mut lowered = Vec::with_capacity(rules.len().saturating_sub(1));
                let mut otherwise = None;
                for rule in rules {
                    let (action, returned) = self.analyze_direct_statement_expression(
                        &rule.action,
                        environment,
                        Some(function_result),
                        Some(function_result),
                        true,
                    )?;
                    assert!(
                        returned,
                        "a checked returning Enum action exits its function"
                    );
                    match rule.matcher {
                        DecisionMatcher::Identifier(matcher) => {
                            let label = self.source.slice(matcher);
                            let Some(value) = enumeration
                                .alternatives
                                .iter()
                                .position(|alternative| alternative == label)
                            else {
                                return Err(source_diagnostic(
                                    &self.source,
                                    "E-UNKNOWN-ENUM-ALTERNATIVE",
                                    matcher,
                                    format!(
                                        "`{label}` is not an alternative of `{}`",
                                        enumeration.name
                                    ),
                                ));
                            };
                            lowered.push(CompilerEnumRule {
                                value: u32::try_from(value)
                                    .expect("enum declaration already fits the native tag"),
                                action,
                                span: rule.span,
                            });
                        }
                        DecisionMatcher::Otherwise(_) => otherwise = Some(action),
                        _ => unreachable!("preselected final-fallback Enum decision shape"),
                    }
                }
                let otherwise = otherwise.expect("complete Enum fallback");
                let mut actions = lowered.iter().map(|rule| &rule.action).collect::<Vec<_>>();
                actions.push(&otherwise);
                let (value_type, int_range, rational_value) =
                    self.decision_facts(&actions, *span)?;
                Ok(Some(CompilerExpression {
                    kind: CompilerExpressionKind::EnumDecision {
                        subject: Box::new(subject),
                        rules: lowered,
                        otherwise: Some(Box::new(otherwise)),
                    },
                    value_type,
                    int_range,
                    rational_value,
                    span: *span,
                }))
            }
            _ => Err(unsupported(
                &self.source,
                subject.span,
                "all-returning final-fallback Enum actions for a non-Enum subject",
            )),
        }
    }
}
