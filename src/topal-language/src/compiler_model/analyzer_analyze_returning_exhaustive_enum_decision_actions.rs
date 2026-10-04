impl Analyzer {
    #[allow(clippy::too_many_lines)] // Exhaustiveness and return-aware action analysis stay adjacent.
    fn analyze_returning_exhaustive_enum_decision_actions(
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
        if !is_supported_returning_exhaustive_enum_action_shape(&self.source, rules) {
            return Ok(None);
        }
        let Some(function_result) = function_result else {
            return Ok(None);
        };
        let subject = self.analyze_expression(subject, environment)?;
        match subject.value_type.clone() {
            CompilerType::Comparison => {
                let alternatives = rules
                    .iter()
                    .filter_map(|rule| match rule.matcher {
                        DecisionMatcher::Identifier(matcher) => Some(self.source.slice(matcher)),
                        _ => None,
                    })
                    .collect::<BTreeSet<_>>();
                if alternatives != BTreeSet::from(["Less", "Equal", "Greater"]) {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-INCOMPLETE-DECISION",
                        *span,
                        "decision does not cover every `Comparison` alternative",
                    ));
                }
                let mut when_less = None;
                let mut when_equal = None;
                let mut when_greater = None;
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
                    let DecisionMatcher::Identifier(matcher) = rule.matcher else {
                        unreachable!("preselected exhaustive Enum decision shape")
                    };
                    match self.source.slice(matcher) {
                        "Less" => when_less = Some(action),
                        "Equal" => when_equal = Some(action),
                        "Greater" => when_greater = Some(action),
                        _ => unreachable!("validated exhaustive Comparison alternative"),
                    }
                }
                let when_less = when_less.expect("complete Less action");
                let when_equal = when_equal.expect("complete Equal action");
                let when_greater = when_greater.expect("complete Greater action");
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
                let positions = rules
                    .iter()
                    .map(|rule| {
                        let DecisionMatcher::Identifier(matcher) = rule.matcher else {
                            unreachable!("preselected exhaustive Enum decision shape")
                        };
                        let label = self.source.slice(matcher);
                        enumeration
                            .alternatives
                            .iter()
                            .position(|alternative| alternative == label)
                            .ok_or_else(|| {
                                source_diagnostic(
                                    &self.source,
                                    "E-UNKNOWN-ENUM-ALTERNATIVE",
                                    matcher,
                                    format!(
                                        "`{label}` is not an alternative of `{}`",
                                        enumeration.name
                                    ),
                                )
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if positions.len() != enumeration.alternatives.len() {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-INCOMPLETE-DECISION",
                        *span,
                        format!(
                            "decision does not cover every `{}` alternative",
                            enumeration.name
                        ),
                    ));
                }
                let mut lowered = Vec::with_capacity(rules.len());
                for (rule, value) in rules.iter().zip(positions) {
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
                    lowered.push(CompilerEnumRule {
                        value: u32::try_from(value)
                            .expect("enum declaration already fits the native tag"),
                        action,
                        span: rule.span,
                    });
                }
                let actions = lowered.iter().map(|rule| &rule.action).collect::<Vec<_>>();
                let (value_type, int_range, rational_value) =
                    self.decision_facts(&actions, *span)?;
                Ok(Some(CompilerExpression {
                    kind: CompilerExpressionKind::EnumDecision {
                        subject: Box::new(subject),
                        rules: lowered,
                        otherwise: None,
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
                "all-returning exhaustive Enum actions for a non-Enum subject",
            )),
        }
    }

    #[allow(clippy::too_many_lines)] // Pattern facts and return-aware action analysis stay adjacent.
    fn analyze_returning_optional_decision_actions(
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
        if !is_supported_returning_optional_action_shape(rules) {
            return Ok(None);
        }
        let Some(function_result) = function_result else {
            return Ok(None);
        };
        let subject = self.analyze_expression(subject, environment)?;
        let CompilerType::Optional(payload_type) = subject.value_type.clone() else {
            return Err(unsupported(
                &self.source,
                subject.span,
                "all-returning Optional actions for a non-Optional subject",
            ));
        };
        let subject_facts = self.known_structural_value_facts(&subject, environment)?;
        let payload_facts = present_optional_payload(subject_facts)
            .or_else(|| self.list_first_string_payload_facts(&subject, environment));
        let mut some = None;
        let mut none = None;
        let mut otherwise = None;
        for rule in rules {
            match rule.matcher {
                DecisionMatcher::Optional {
                    some: true,
                    binding: Some(binding),
                    ..
                } => {
                    let name = self.source.slice(binding).to_owned();
                    let mut branch = decision_binding_environment(
                        environment,
                        &name,
                        payload_type.as_ref().clone(),
                        binding.start,
                    );
                    if let Some(payload_facts) = &payload_facts {
                        let binding_facts = branch
                            .get_mut(&name)
                            .expect("Optional payload binding was inserted");
                        retain_static_value_facts(binding_facts, payload_facts);
                    }
                    let (action, returned) = self.analyze_direct_statement_expression(
                        &rule.action,
                        &branch,
                        Some(function_result),
                        Some(function_result),
                        true,
                    )?;
                    assert!(
                        returned,
                        "a checked returning Optional action exits its function"
                    );
                    some = Some((name, binding, action));
                }
                DecisionMatcher::Optional {
                    some: false,
                    binding: None,
                    ..
                } => {
                    let (action, returned) = self.analyze_direct_statement_expression(
                        &rule.action,
                        environment,
                        Some(function_result),
                        Some(function_result),
                        true,
                    )?;
                    assert!(
                        returned,
                        "a checked returning Optional action exits its function"
                    );
                    none = Some(action);
                }
                DecisionMatcher::Otherwise(_) => {
                    let (action, returned) = self.analyze_direct_statement_expression(
                        &rule.action,
                        environment,
                        Some(function_result),
                        Some(function_result),
                        true,
                    )?;
                    assert!(
                        returned,
                        "a checked returning Optional action exits its function"
                    );
                    otherwise = Some(action);
                }
                _ => unreachable!("preselected complete Optional decision shape"),
            }
        }
        let (some_binding, some_action) = if let Some((name, binding, action)) = some {
            (Some((name, binding)), action)
        } else {
            (None, otherwise.clone().expect("complete Some action"))
        };
        let none_action = none
            .or_else(|| otherwise.clone())
            .expect("complete None action");
        let (value_type, int_range, rational_value) =
            self.decision_facts(&[&some_action, &none_action], *span)?;
        Ok(Some(CompilerExpression {
            kind: CompilerExpressionKind::OptionalDecision {
                subject: Box::new(subject),
                some_binding,
                some_action: Box::new(some_action),
                none_action: Box::new(none_action),
            },
            value_type,
            int_range,
            rational_value,
            span: *span,
        }))
    }

    #[allow(clippy::too_many_lines)] // Both Result payload environments stay adjacent.
    fn analyze_returning_result_decision_actions(
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
        if !is_supported_returning_result_action_shape(rules) {
            return Ok(None);
        }
        let Some(function_result) = function_result else {
            return Ok(None);
        };
        let subject = self.analyze_expression(subject, environment)?;
        let CompilerType::Result(success_type) = subject.value_type.clone() else {
            return Err(unsupported(
                &self.source,
                subject.span,
                "all-returning Result actions for a non-Result subject",
            ));
        };
        let subject_facts = (success_type.as_ref() == &CompilerType::Function)
            .then(|| self.known_structural_value_facts(&subject, environment))
            .transpose()?;
        let mut ok = None;
        let mut error = None;
        for rule in rules {
            let DecisionMatcher::Result {
                error: is_error,
                binding,
                ..
            } = rule.matcher
            else {
                unreachable!("preselected complete Result decision shape")
            };
            let name = self.source.slice(binding).to_owned();
            let binding_type = if is_error {
                CompilerType::Error
            } else {
                success_type.as_ref().clone()
            };
            let mut branch =
                decision_binding_environment(environment, &name, binding_type, binding.start);
            if !is_error
                && let Some(success_facts) = subject_facts
                    .as_ref()
                    .and_then(|facts| facts.result.as_ref())
            {
                let binding_facts = branch
                    .get_mut(&name)
                    .expect("Result success decision binding was inserted");
                retain_static_value_facts(binding_facts, &success_facts.success);
            }
            let (action, returned) = self.analyze_direct_statement_expression(
                &rule.action,
                &branch,
                Some(function_result),
                Some(function_result),
                true,
            )?;
            assert!(
                returned,
                "a checked returning Result action exits its function"
            );
            if is_error {
                error = Some((name, binding, Box::new(action)));
            } else {
                ok = Some((name, binding, action));
            }
        }
        let (ok_binding, ok_binding_span, ok_action) = ok.expect("complete Ok action");
        let error_fallback = error.expect("complete Error action");
        let (value_type, int_range, rational_value) =
            self.decision_facts(&[&ok_action, error_fallback.2.as_ref()], *span)?;
        Ok(Some(CompilerExpression {
            kind: CompilerExpressionKind::ResultDecision {
                subject: Box::new(subject),
                ok_binding,
                ok_binding_span,
                ok_action: Box::new(ok_action),
                error_codes: Vec::new(),
                error_fallback: Some(error_fallback),
            },
            value_type,
            int_range,
            rational_value,
            span: *span,
        }))
    }

    #[allow(clippy::too_many_lines)] // Ordered code rules and both Result payload paths stay adjacent.
    fn analyze_returning_error_code_decision_actions(
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
        if !is_supported_returning_error_code_action_shape(rules) {
            return Ok(None);
        }
        let Some(function_result) = function_result else {
            return Ok(None);
        };
        let subject = self.analyze_expression(subject, environment)?;
        let CompilerType::Result(success_type) = subject.value_type.clone() else {
            return Err(unsupported(
                &self.source,
                subject.span,
                "all-returning qualified Error-code actions for a non-Result subject",
            ));
        };
        let subject_facts = (success_type.as_ref() == &CompilerType::Function)
            .then(|| self.known_structural_value_facts(&subject, environment))
            .transpose()?;
        let mut ok = None;
        let mut error_codes = Vec::new();
        let mut seen_codes = BTreeSet::new();
        let mut error_fallback = None;
        for rule in rules {
            match rule.matcher {
                DecisionMatcher::Result {
                    error: false,
                    binding,
                    ..
                } if ok.is_none() => {
                    let name = self.source.slice(binding).to_owned();
                    let mut branch = decision_binding_environment(
                        environment,
                        &name,
                        success_type.as_ref().clone(),
                        binding.start,
                    );
                    if let Some(success_facts) = subject_facts
                        .as_ref()
                        .and_then(|facts| facts.result.as_ref())
                    {
                        let binding_facts = branch
                            .get_mut(&name)
                            .expect("Result success decision binding was inserted");
                        retain_static_value_facts(binding_facts, &success_facts.success);
                    }
                    let (action, returned) = self.analyze_direct_statement_expression(
                        &rule.action,
                        &branch,
                        Some(function_result),
                        Some(function_result),
                        true,
                    )?;
                    assert!(returned, "a checked Ok action exits its function");
                    ok = Some((name, binding, action));
                }
                DecisionMatcher::Result {
                    error: true,
                    binding,
                    ..
                } if error_fallback.is_none() => {
                    let name = self.source.slice(binding).to_owned();
                    let branch = decision_binding_environment(
                        environment,
                        &name,
                        CompilerType::Error,
                        binding.start,
                    );
                    let (action, returned) = self.analyze_direct_statement_expression(
                        &rule.action,
                        &branch,
                        Some(function_result),
                        Some(function_result),
                        true,
                    )?;
                    assert!(returned, "a checked Error fallback exits its function");
                    error_fallback = Some((name, binding, Box::new(action)));
                }
                DecisionMatcher::ErrorCode {
                    namespace,
                    vocabulary,
                    code,
                    ..
                } => {
                    if error_fallback.is_some() {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-UNREACHABLE-ERROR-CODE-PATTERN",
                            rule.span,
                            "qualified Error-code pattern is unreachable after Error fallback",
                        ));
                    }
                    let code_value = arithmetic_error_code(
                        self.source.slice(namespace),
                        self.source.slice(vocabulary),
                        self.source.slice(code),
                    )
                    .ok_or_else(|| {
                        source_diagnostic(
                            &self.source,
                            "E-UNKNOWN-ERROR-CODE",
                            code,
                            "the compiler subset requires a qualified arithmetic Error code",
                        )
                    })?;
                    if !seen_codes.insert(code_value) {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-DUPLICATE-ERROR-CODE-PATTERN",
                            rule.span,
                            "an arithmetic Error code is matched more than once",
                        ));
                    }
                    let (action, returned) = self.analyze_direct_statement_expression(
                        &rule.action,
                        environment,
                        Some(function_result),
                        Some(function_result),
                        true,
                    )?;
                    assert!(returned, "a checked Error-code action exits its function");
                    error_codes.push(CompilerErrorCodeRule {
                        code: code_value,
                        action,
                        span: rule.span,
                    });
                }
                _ => {
                    return Err(unsupported(
                        &self.source,
                        rule.span,
                        "all-returning qualified Error-code decision matcher",
                    ));
                }
            }
        }
        let (ok_binding, ok_binding_span, ok_action) = ok.expect("complete Ok action");
        if error_fallback.is_none() && seen_codes != BTreeSet::from([0, 1, 2, 3]) {
            return Err(source_diagnostic(
                &self.source,
                "E-INCOMPLETE-ERROR-CODE-DECISION",
                *span,
                "Result decision requires Error fallback or every arithmetic Error code",
            ));
        }
        let mut actions = vec![&ok_action];
        actions.extend(error_codes.iter().map(|rule| &rule.action));
        if let Some((_, _, action)) = &error_fallback {
            actions.push(action);
        }
        let (value_type, int_range, rational_value) = self.decision_facts(&actions, *span)?;
        Ok(Some(CompilerExpression {
            kind: CompilerExpressionKind::ResultDecision {
                subject: Box::new(subject),
                ok_binding,
                ok_binding_span,
                ok_action: Box::new(ok_action),
                error_codes,
                error_fallback,
            },
            value_type,
            int_range,
            rational_value,
            span: *span,
        }))
    }

    #[allow(clippy::too_many_lines)] // Entry bindings and exact structural facts stay adjacent.
    fn analyze_returning_list_decision_actions(
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
        if !is_supported_returning_list_action_shape(&self.source, rules) {
            return Ok(None);
        }
        let Some(function_result) = function_result else {
            return Ok(None);
        };
        let subject = self.analyze_expression(subject, environment)?;
        let CompilerType::List(element_type) = subject.value_type.clone() else {
            return Err(unsupported(
                &self.source,
                subject.span,
                "all-returning List actions for a non-List subject",
            ));
        };
        if !compiler_list_observation_element_supported(element_type.as_ref())
            && element_type.as_ref() != &CompilerType::Function
        {
            return Err(unsupported(
                &self.source,
                subject.span,
                "all-returning decision actions for this List element type",
            ));
        }
        let subject_entries = (element_type.as_ref() == &CompilerType::Function)
            .then(|| self.known_structural_value_facts(&subject, environment))
            .transpose()?
            .and_then(|facts| facts.list_entries);
        let mut entry = None;
        let mut empty = None;
        for rule in rules {
            match rule.matcher {
                DecisionMatcher::ListEmpty(_) => {
                    let (action, returned) = self.analyze_direct_statement_expression(
                        &rule.action,
                        environment,
                        Some(function_result),
                        Some(function_result),
                        true,
                    )?;
                    assert!(
                        returned,
                        "a checked returning Empty action exits its function"
                    );
                    empty = Some(action);
                }
                DecisionMatcher::ListEntry { first, rest, .. } => {
                    let first_name = self.source.slice(first).to_owned();
                    let rest_name = self.source.slice(rest).to_owned();
                    let mut branch = decision_binding_environment(
                        environment,
                        &first_name,
                        element_type.as_ref().clone(),
                        first.start,
                    );
                    branch = decision_binding_environment(
                        &branch,
                        &rest_name,
                        CompilerType::List(element_type.clone()),
                        rest.start,
                    );
                    if let Some((first_facts, remaining_facts)) = subject_entries
                        .as_ref()
                        .and_then(|entries| entries.split_first())
                    {
                        retain_static_value_facts(
                            branch
                                .get_mut(&first_name)
                                .expect("List entry binding was inserted"),
                            first_facts,
                        );
                        branch
                            .get_mut(&rest_name)
                            .expect("remaining List binding was inserted")
                            .list_entries = Some(remaining_facts.to_vec());
                    }
                    let (action, returned) = self.analyze_direct_statement_expression(
                        &rule.action,
                        &branch,
                        Some(function_result),
                        Some(function_result),
                        true,
                    )?;
                    assert!(
                        returned,
                        "a checked returning Entry action exits its function"
                    );
                    entry = Some((((first_name, first), (rest_name, rest)), action));
                }
                _ => unreachable!("preselected complete List decision shape"),
            }
        }
        let (entry_bindings, entry_action) = entry.expect("complete Entry action");
        let empty_action = empty.expect("complete Empty action");
        let (value_type, int_range, rational_value) =
            self.decision_facts(&[&entry_action, &empty_action], *span)?;
        Ok(Some(CompilerExpression {
            kind: CompilerExpressionKind::ListDecision {
                subject: Box::new(subject),
                entry_bindings: Some(entry_bindings),
                entry_action: Box::new(entry_action),
                empty_action: Box::new(empty_action),
            },
            value_type,
            int_range,
            rational_value,
            span: *span,
        }))
    }

    #[allow(clippy::too_many_lines)] // Matcher typing and return-aware action analysis stay adjacent.
    fn analyze_returning_ordered_comparison_decision_actions(
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
        if !is_supported_returning_ordered_comparison_action_shape(rules) {
            return Ok(None);
        }
        let Some(function_result) = function_result else {
            return Ok(None);
        };
        let mut subject = self.analyze_expression(subject, environment)?;
        match &subject.value_type {
            CompilerType::Int | CompilerType::Rational => {}
            CompilerType::Nat if self.active_nat_recursion() => {
                subject = forget_nat_evidence(subject);
            }
            _ => {
                return Err(unsupported(
                    &self.source,
                    subject.span,
                    "all-returning ordered comparison actions for a non-numeric subject",
                ));
            }
        }
        require_exact_numeric(&self.source, subject.span, &subject.value_type)?;
        let mut lowered = Vec::new();
        let mut otherwise = None;
        for rule in rules {
            match &rule.matcher {
                DecisionMatcher::Comparison {
                    kind,
                    operand,
                    span: matcher_span,
                } => {
                    let Some(operation) = comparison_binary(*kind) else {
                        return Err(unsupported(
                            &self.source,
                            *matcher_span,
                            "comparison decision callable",
                        ));
                    };
                    let mut operand = self.analyze_expression(operand, environment)?;
                    if operand.value_type == CompilerType::Nat {
                        operand = forget_nat_evidence(operand);
                    }
                    require_exact_numeric(&self.source, operand.span, &operand.value_type)?;
                    let subject_to_rational = subject.value_type == CompilerType::Int
                        && operand.value_type == CompilerType::Rational;
                    if subject.value_type == CompilerType::Rational
                        && operand.value_type == CompilerType::Int
                    {
                        operand = into_rational(operand);
                    }
                    if !subject_to_rational {
                        require_same_type(
                            &self.source,
                            operand.span,
                            &subject.value_type,
                            &operand.value_type,
                        )?;
                    }
                    let (action, returned) = self.analyze_direct_statement_expression(
                        &rule.action,
                        environment,
                        Some(function_result),
                        Some(function_result),
                        true,
                    )?;
                    assert!(
                        returned,
                        "a checked returning comparison action exits its function"
                    );
                    lowered.push(CompilerComparisonRule {
                        operation,
                        operand,
                        action,
                        subject_to_rational,
                        span: rule.span,
                    });
                }
                DecisionMatcher::Otherwise(_) => {
                    let (action, returned) = self.analyze_direct_statement_expression(
                        &rule.action,
                        environment,
                        Some(function_result),
                        Some(function_result),
                        true,
                    )?;
                    assert!(
                        returned,
                        "a checked returning comparison fallback exits its function"
                    );
                    otherwise = Some(action);
                }
                _ => unreachable!("preselected complete ordered comparison decision shape"),
            }
        }
        let otherwise = otherwise.expect("complete ordered comparison fallback");
        let mut branches = lowered.iter().map(|rule| &rule.action).collect::<Vec<_>>();
        branches.push(&otherwise);
        let (value_type, int_range, rational_value) = self.decision_facts(&branches, *span)?;
        Ok(Some(CompilerExpression {
            kind: CompilerExpressionKind::OrderedComparisonDecision {
                subject: Box::new(subject),
                rules: lowered,
                otherwise: Box::new(otherwise),
            },
            value_type,
            int_range,
            rational_value,
            span: *span,
        }))
    }

    fn analyze_returning_product_field(
        &mut self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
        function_result: Option<&CompilerType>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        let Expression::Product { fields, .. } = expression else {
            return Ok(None);
        };
        let Some(returning_index) = fields
            .iter()
            .position(|field| direct_expression_returns_from_function(&field.value))
        else {
            return Ok(None);
        };
        let mut labels = BTreeSet::new();
        let mut preceding = Vec::with_capacity(returning_index);
        for (index, field) in fields[..=returning_index].iter().enumerate() {
            if let Some(label_span) = field.label {
                let label = self.source.slice(label_span).to_owned();
                if !labels.insert(label) {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-DUPLICATE-RECORD-FIELD",
                        label_span,
                        "record field label occurs more than once",
                    ));
                }
            }
            if index == returning_index {
                break;
            }
            preceding.push(self.analyze_expression(&field.value, environment)?);
        }
        let (result, returned) = self.analyze_direct_statement_expression(
            &fields[returning_index].value,
            environment,
            function_result,
            function_result,
            true,
        )?;
        assert!(
            returned,
            "a checked returning product field exits its function"
        );
        if preceding.is_empty() {
            return Ok(Some(result));
        }
        Ok(Some(CompilerExpression {
            value_type: result.value_type.clone(),
            int_range: result.int_range.clone(),
            rational_value: result.rational_value.clone(),
            kind: CompilerExpressionKind::ExitSequence {
                preceding,
                result: Box::new(result),
            },
            span: expression.span(),
        }))
    }

    fn analyze_returning_collection_source(
        &mut self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
        function_result: Option<&CompilerType>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        let Expression::Application { items, .. } = expression else {
            return Ok(None);
        };
        let (source, map_policy) = match items.as_slice() {
            [Expression::Identifier(operation), source]
                if matches!(
                    self.source.slice(*operation),
                    "collect" | "collect-set" | "collect-bag"
                ) =>
            {
                (source, None)
            }
            [
                source,
                Expression::Identifier(operation),
                Expression::Identifier(target),
            ] if self.source.slice(*operation) == "collect"
                && matches!(self.source.slice(*target), "Array" | "String") =>
            {
                (source, None)
            }
            [
                Expression::Identifier(operation),
                source,
                Expression::Identifier(resolving),
                Expression::Identifier(policy),
            ] if self.source.slice(*operation) == "collect-map"
                && self.source.slice(*resolving) == "resolving" =>
            {
                (source, Some(*policy))
            }
            _ => return Ok(None),
        };
        if !direct_expression_returns_from_function(source) {
            return Ok(None);
        }
        if let Some(policy) = map_policy
            && !matches!(
                self.source.slice(policy),
                "reject" | "keep-first" | "keep-last"
            )
        {
            return Err(source_diagnostic(
                &self.source,
                "E-MAP-COLLISION-POLICY",
                policy,
                "collect-map policy must be reject, keep-first, or keep-last",
            ));
        }
        let (result, returned) = self.analyze_direct_statement_expression(
            source,
            environment,
            function_result,
            function_result,
            true,
        )?;
        assert!(
            returned,
            "a checked returning collection source exits its function"
        );
        Ok(Some(result))
    }

    fn analyze_returning_modular_reduction_operand(
        &mut self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
        function_result: Option<&CompilerType>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
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
        if self.source.slice(*operation) != "modulo"
            || !direct_expression_returns_from_function(operand)
            || self
                .modulars
                .get(self.source.slice(*type_name))
                .is_none_or(|(_, declaration)| declaration.end > type_name.start)
        {
            return Ok(None);
        }
        let (result, returned) = self.analyze_direct_statement_expression(
            operand,
            environment,
            function_result,
            function_result,
            true,
        )?;
        assert!(
            returned,
            "a checked returning modular reduction operand exits its function"
        );
        Ok(Some(result))
    }

    fn analyze_returning_operator_operand(
        &mut self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
        function_result: Option<&CompilerType>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        let Expression::Application { items, .. } = expression else {
            return Ok(None);
        };
        if items.len() >= 2
            && direct_expression_returns_from_function(&items[0])
            && matches!(items[1], Expression::Callable { .. })
        {
            let (value, returned) = self.analyze_direct_statement_expression(
                &items[0],
                environment,
                function_result,
                function_result,
                true,
            )?;
            assert!(returned, "a checked returning operand exits its function");
            return Ok(Some(value));
        }
        let Some(returning_index) = (2..items.len()).find(|index| {
            matches!(items[index - 1], Expression::Callable { .. })
                && direct_expression_returns_from_function(&items[*index])
        }) else {
            return Ok(None);
        };
        let preceding = &items[..returning_index - 1];
        let right = &items[returning_index];
        let preceding_span = Span::new(
            preceding[0].span().start,
            preceding.last().expect("nonempty prefix").span().end,
        );
        let preceding = if let [preceding] = preceding {
            self.analyze_expression(preceding, environment)?
        } else {
            self.analyze_expression(
                &Expression::Application {
                    items: preceding.to_vec(),
                    span: preceding_span,
                },
                environment,
            )?
        };
        let (result, returned) = self.analyze_direct_statement_expression(
            right,
            environment,
            function_result,
            function_result,
            true,
        )?;
        assert!(returned, "a checked returning operand exits its function");
        Ok(Some(CompilerExpression {
            value_type: result.value_type.clone(),
            int_range: result.int_range.clone(),
            rational_value: result.rational_value.clone(),
            kind: CompilerExpressionKind::ExitSequence {
                preceding: vec![preceding],
                result: Box::new(result),
            },
            span: expression.span(),
        }))
    }

    fn analyze_returning_named_call_argument(
        &mut self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
        function_result: Option<&CompilerType>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        let Expression::Application { items, .. } = expression else {
            return Ok(None);
        };
        let Some((function_index, function_name)) =
            items.iter().enumerate().rev().find_map(|(index, item)| {
                let Expression::Identifier(name) = item else {
                    return None;
                };
                let name = self.source.slice(*name);
                (!environment.contains_key(name) && self.functions.contains_key(name))
                    .then_some((index, name))
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
                    && (!self.static_context || declaration.is_static)
                    && declaration.parameters.iter().all(|parameter| {
                        parameter.fields.is_empty()
                            && parameter.default.is_none()
                            && parameter.qualifier.is_none()
                    }))
            });
        if !admitted_declaration {
            return Ok(None);
        }
        let mut preceding = Vec::with_capacity(usize::from(preceding_source.is_some()));
        if let Some(source) = preceding_source {
            let value = if let [value] = source {
                self.analyze_expression(value, environment)?
            } else {
                self.analyze_expression(
                    &Expression::Application {
                        items: source.to_vec(),
                        span: Span::new(
                            source[0].span().start,
                            source.last().expect("nonempty call prefix").span().end,
                        ),
                    },
                    environment,
                )?
            };
            preceding.push(value);
        }
        let (result, returned) = self.analyze_direct_statement_expression(
            &items[returning_index],
            environment,
            function_result,
            function_result,
            true,
        )?;
        assert!(
            returned,
            "a checked returning named-call argument exits its function"
        );
        if preceding.is_empty() {
            return Ok(Some(result));
        }
        Ok(Some(CompilerExpression {
            value_type: result.value_type.clone(),
            int_range: result.int_range.clone(),
            rational_value: result.rational_value.clone(),
            kind: CompilerExpressionKind::ExitSequence {
                preceding,
                result: Box::new(result),
            },
            span: expression.span(),
        }))
    }

    fn analyze_returning_unary_constructor_argument(
        &mut self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
        function_result: Option<&CompilerType>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        let Expression::Application { items, .. } = expression else {
            return Ok(None);
        };
        let [Expression::Identifier(constructor), argument] = items.as_slice() else {
            return Ok(None);
        };
        let constructor_name = self.source.slice(*constructor);
        let built_in = matches!(
            constructor_name,
            "Some" | "String" | "Character" | "Int" | "Nat" | "Rational"
        );
        let declared_union =
            self.sum_alternatives
                .get(constructor_name)
                .is_some_and(|(sum, value, declaration)| {
                    declaration.end <= constructor.start
                        && sum.alternatives
                            [usize::try_from(*value).expect("u32 sum tag fits usize")]
                        .payload
                        .is_some()
                });
        let declared_constraint =
            self.constraint_bindings
                .get(constructor_name)
                .is_some_and(|_| {
                    self.constraint_binding_declarations
                        .get(constructor_name)
                        .is_some_and(|end| *end <= constructor.start)
                });
        let declared_modular = self
            .modulars
            .get(constructor_name)
            .is_some_and(|(_, declaration)| declaration.end <= constructor.start);
        if (!built_in && !declared_union && !declared_constraint && !declared_modular)
            || !direct_expression_returns_from_function(argument)
        {
            return Ok(None);
        }
        let (result, returned) = self.analyze_direct_statement_expression(
            argument,
            environment,
            function_result,
            function_result,
            true,
        )?;
        assert!(
            returned,
            "a checked returning unary constructor argument exits its function"
        );
        Ok(Some(result))
    }

    fn analyze_returning_variant_constructor_argument(
        &mut self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
        function_result: Option<&CompilerType>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
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
        let admitted = self.source.slice(*at) == "at"
            && self
                .sums
                .get(self.source.slice(*type_name))
                .is_some_and(|(sum, declaration)| {
                    sum.positional
                        && declaration.end <= type_name.start
                        && parse_integer(self.source.slice(*index))
                            .and_then(|value| value.to_string().parse::<usize>().ok())
                            .is_some_and(|value| value < sum.alternatives.len())
                });
        if !admitted || !direct_expression_returns_from_function(argument) {
            return Ok(None);
        }
        let (result, returned) = self.analyze_direct_statement_expression(
            argument,
            environment,
            function_result,
            function_result,
            true,
        )?;
        assert!(
            returned,
            "a checked returning Variant constructor argument exits its function"
        );
        Ok(Some(result))
    }

    #[allow(clippy::too_many_lines)] // Exhaustive statement admission keeps the subset boundary visible.
    fn analyze_block(
        &mut self,
        statements: &[Statement],
        environment: &mut BTreeMap<String, BindingFacts>,
        kind: BlockKind,
        enclosing_result: Option<&CompilerType>,
    ) -> Result<CompilerBlock, Diagnostic> {
        self.analyze_block_control(
            statements,
            environment,
            kind,
            enclosing_result,
            enclosing_result,
            kind == BlockKind::Function,
        )
        .map(|analyzed| analyzed.block)
    }
}
