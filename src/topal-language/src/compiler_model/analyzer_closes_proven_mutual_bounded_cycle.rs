impl Analyzer {
    fn closes_proven_mutual_bounded_cycle(&self, target_identity: &str, target_name: &str) -> bool {
        let Some(cycle_start) = self
            .active_calls
            .iter()
            .position(|identity| identity == target_identity)
        else {
            return false;
        };
        let cycle = &self.active_calls[cycle_start..];
        if cycle.len() < 2 {
            return false;
        }
        let Some(target) = self.active_recursive_functions.get(target_identity) else {
            return false;
        };
        let rule = target.proof.rule;
        if target.source_name != target_name
            || !matches!(
                rule,
                "TOPAL-FUNCTION-RECURSION-INT-MUTUAL-001"
                    | "TOPAL-FUNCTION-RECURSION-INT-MUTUAL-INCREASING-001"
                    | "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001"
                    | "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001"
            )
            || !cycle.iter().all(|identity| {
                self.active_recursive_functions
                    .get(identity)
                    .is_some_and(|active| active.proof.rule == rule)
            })
        {
            return false;
        }
        let internal_edges_close = cycle.windows(2).all(|pair| {
            let current = self
                .active_recursive_functions
                .get(&pair[0])
                .expect("active cycle member retains proof metadata");
            let next = self
                .active_recursive_functions
                .get(&pair[1])
                .expect("active cycle member retains proof metadata");
            current.proof.mutual_target.as_deref() == Some(next.source_name.as_str())
        });
        let last = self
            .active_recursive_functions
            .get(cycle.last().expect("cycle has at least two members"))
            .expect("active cycle member retains proof metadata");
        internal_edges_close && last.proof.mutual_target.as_deref() == Some(target_name)
    }

    fn adapt_proven_recursive_nat_argument(
        &self,
        function_name: &str,
        declaration: &FunctionSource,
        parameter_index: usize,
        expected: &CompilerType,
        argument: &CompilerExpression,
    ) -> Option<CompilerExpression> {
        if expected != &CompilerType::Nat || argument.value_type != CompilerType::Int {
            return None;
        }
        let target_identity = function_overload_identity(&self.source, function_name, declaration);
        let active_identity = self.active_calls.last()?;
        let active = self.active_recursive_functions.get(active_identity)?;
        let direct_edge =
            active_identity == &target_identity && active.proof.mutual_target.is_none();
        let declaration_name = self.source.slice(declaration.name);
        let mutual_nat_edge = active.proof.mutual_target.as_deref() == Some(declaration_name)
            && matches!(
                active.proof.rule,
                "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001"
                    | "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001"
            );
        if !(direct_edge || mutual_nat_edge)
            || !active.proof.nat_step_parameters.contains(&parameter_index)
        {
            return None;
        }
        Some(CompilerExpression {
            kind: CompilerExpressionKind::IntToNat(Box::new(argument.clone())),
            value_type: CompilerType::Nat,
            int_range: argument.int_range.clone(),
            rational_value: None,
            span: argument.span,
        })
    }

    fn adapt_proven_nonnegative_nat_argument(
        &self,
        expected: &CompilerType,
        argument: &CompilerExpression,
    ) -> Option<CompilerExpression> {
        if expected != &CompilerType::Nat || argument.value_type != CompilerType::Int {
            return None;
        }
        if self.compiler_proven_nonnegative_expression(argument) {
            return Some(CompilerExpression {
                kind: CompilerExpressionKind::IntToNat(Box::new(argument.clone())),
                value_type: CompilerType::Nat,
                int_range: argument.int_range.clone(),
                rational_value: None,
                span: argument.span,
            });
        }
        let CompilerExpressionKind::Binary {
            operation: CompilerBinary::Subtract,
            left,
            right,
        } = &argument.kind
        else {
            return None;
        };
        let CompilerExpressionKind::Local(length) = &left.kind else {
            return None;
        };
        let CompilerExpressionKind::Binary {
            operation: CompilerBinary::Modulo,
            right: divisor,
            ..
        } = &right.kind
        else {
            return None;
        };
        let CompilerExpressionKind::Local(divisor) = &divisor.kind else {
            return None;
        };
        if length != divisor || !self.active_nonzero_bindings.contains(length) {
            return None;
        }
        Some(CompilerExpression {
            kind: CompilerExpressionKind::IntToNat(Box::new(argument.clone())),
            value_type: CompilerType::Nat,
            int_range: None,
            rational_value: None,
            span: argument.span,
        })
    }

    fn active_nat_recursion(&self) -> bool {
        self.active_calls
            .last()
            .and_then(|identity| self.active_recursive_functions.get(identity))
            .is_some_and(|active| {
                !active.proof.nat_step_parameters.is_empty()
                    && matches!(
                        active.proof.rule,
                        "TOPAL-FUNCTION-RECURSION-NAT-001"
                            | "TOPAL-FUNCTION-RECURSION-NAT-INCREASING-001"
                            | "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-001"
                            | "TOPAL-FUNCTION-RECURSION-NAT-MUTUAL-INCREASING-001"
                            | "TOPAL-FUNCTION-DECREASES-001"
                    )
            })
    }

    fn analyze_decision(
        &mut self,
        subject: &Expression,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        if let Some(decision) =
            self.analyze_cascading_boolean_optional_decision(subject, rules, span, environment)?
        {
            return Ok(decision);
        }
        if let Some(decision) =
            self.analyze_cascading_boolean_boolean_decision(subject, rules, span, environment)?
        {
            return Ok(decision);
        }
        if let Some(decision) =
            self.analyze_cascading_boolean_comparison_decision(subject, rules, span, environment)?
        {
            return Ok(decision);
        }
        let subject = self.analyze_expression(subject, environment)?;
        match &subject.value_type {
            CompilerType::Boolean => {
                self.analyze_boolean_decision(subject, rules, span, environment)
            }
            CompilerType::Comparison => {
                self.analyze_comparison_value_decision(subject, rules, span, environment)
            }
            CompilerType::Enum(enumeration) => {
                let enumeration = enumeration.clone();
                self.analyze_enum_decision(subject, &enumeration, rules, span, environment)
            }
            CompilerType::Sum(sum) => {
                let sum = sum.clone();
                self.analyze_sum_decision(subject, &sum, rules, span, environment)
            }
            CompilerType::Result(success) => {
                let success = success.as_ref().clone();
                self.analyze_result_decision(subject, &success, rules, span, environment, None)
            }
            CompilerType::Optional(payload) => {
                let payload = payload.as_ref().clone();
                self.analyze_optional_decision(subject, &payload, rules, span, environment, None)
            }
            CompilerType::List(element)
                if compiler_list_observation_element_supported(element.as_ref())
                    || element.as_ref() == &CompilerType::Function =>
            {
                let element = element.as_ref().clone();
                self.analyze_list_decision(subject, &element, rules, span, environment)
            }
            CompilerType::List(_) => Err(unsupported(
                &self.source,
                subject.span,
                "decision for this List element type",
            )),
            CompilerType::Character => self.analyze_ordered_comparison_decision(
                forget_character_evidence(subject),
                rules,
                span,
                environment,
                None,
            ),
            CompilerType::Int
            | CompilerType::Rational
            | CompilerType::String
            | CompilerType::Tuple(_) => {
                self.analyze_ordered_comparison_decision(subject, rules, span, environment, None)
            }
            CompilerType::Nat => self.analyze_ordered_comparison_decision(
                forget_nat_evidence(subject),
                rules,
                span,
                environment,
                None,
            ),
            _ => Err(unsupported(
                &self.source,
                subject.span,
                "decision subject type",
            )),
        }
    }

    fn analyze_cascading_boolean_optional_decision(
        &mut self,
        subject: &Expression,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        let Expression::DecisionTable {
            subject: inner_subject,
            rules: inner_rules,
            ..
        } = subject
        else {
            return Ok(None);
        };
        if !rules.iter().all(|rule| {
            matches!(
                rule.matcher,
                DecisionMatcher::Optional { .. } | DecisionMatcher::Otherwise(_)
            )
        }) || !inner_rules.iter().all(|rule| {
            matches!(
                rule.matcher,
                DecisionMatcher::Boolean { .. } | DecisionMatcher::Otherwise(_)
            )
        }) {
            return Ok(None);
        }
        let inner_subject = self.analyze_expression(inner_subject, environment)?;
        if inner_subject.value_type != CompilerType::Boolean {
            return Ok(None);
        }
        let mut when_true = None;
        let mut when_false = None;
        let mut otherwise = None;
        for rule in inner_rules {
            let action = self.analyze_expression(&rule.action, environment)?;
            match rule.matcher {
                DecisionMatcher::Boolean { value: true, .. } => when_true = Some(action),
                DecisionMatcher::Boolean { value: false, .. } => when_false = Some(action),
                DecisionMatcher::Otherwise(_) => otherwise = Some(action),
                _ => unreachable!("preselected Boolean cascading decision"),
            }
        }
        let mut when_true = when_true.or_else(|| otherwise.clone()).ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "Boolean decision does not cover `true`",
            )
        })?;
        let mut when_false = when_false.or(otherwise).ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "Boolean decision does not cover `false`",
            )
        })?;
        if let CompilerType::Optional(payload) = &when_true.value_type {
            let payload = payload.as_ref().clone();
            when_true = self.analyze_optional_decision(
                when_true,
                &payload,
                rules,
                span,
                environment,
                None,
            )?;
        }
        if let CompilerType::Optional(payload) = &when_false.value_type {
            let payload = payload.as_ref().clone();
            when_false = self.analyze_optional_decision(
                when_false,
                &payload,
                rules,
                span,
                environment,
                None,
            )?;
        }
        adapt_decision_pair(&mut when_true, &mut when_false);
        let (value_type, int_range, rational_value) =
            self.decision_facts(&[&when_true, &when_false], span)?;
        Ok(Some(CompilerExpression {
            kind: CompilerExpressionKind::BooleanDecision {
                subject: Box::new(inner_subject),
                when_true: Box::new(when_true),
                when_false: Box::new(when_false),
            },
            value_type,
            int_range,
            rational_value,
            span,
        }))
    }

    fn analyze_cascading_boolean_boolean_decision(
        &mut self,
        subject: &Expression,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        let Expression::DecisionTable {
            subject: inner_subject,
            rules: inner_rules,
            ..
        } = subject
        else {
            return Ok(None);
        };
        if !rules.iter().all(|rule| {
            matches!(
                rule.matcher,
                DecisionMatcher::Boolean { .. } | DecisionMatcher::Otherwise(_)
            )
        }) || !inner_rules.iter().all(|rule| {
            matches!(
                rule.matcher,
                DecisionMatcher::Boolean { .. } | DecisionMatcher::Otherwise(_)
            )
        }) {
            return Ok(None);
        }
        let inner_subject = self.analyze_expression(inner_subject, environment)?;
        if inner_subject.value_type != CompilerType::Boolean {
            return Ok(None);
        }
        let mut when_true = None;
        let mut when_false = None;
        let mut otherwise = None;
        for rule in inner_rules {
            let action = self.analyze_expression(&rule.action, environment)?;
            match rule.matcher {
                DecisionMatcher::Boolean { value: true, .. } => when_true = Some(action),
                DecisionMatcher::Boolean { value: false, .. } => when_false = Some(action),
                DecisionMatcher::Otherwise(_) => otherwise = Some(action),
                _ => unreachable!("preselected Boolean cascading decision"),
            }
        }
        let mut when_true = when_true.or_else(|| otherwise.clone()).ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "Boolean decision does not cover `true`",
            )
        })?;
        let mut when_false = when_false.or(otherwise).ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "Boolean decision does not cover `false`",
            )
        })?;
        if when_true.value_type == CompilerType::Boolean {
            when_true = self.analyze_boolean_decision(when_true, rules, span, environment)?;
        }
        if when_false.value_type == CompilerType::Boolean {
            when_false = self.analyze_boolean_decision(when_false, rules, span, environment)?;
        }
        adapt_decision_pair(&mut when_true, &mut when_false);
        let (value_type, int_range, rational_value) =
            self.decision_facts(&[&when_true, &when_false], span)?;
        Ok(Some(CompilerExpression {
            kind: CompilerExpressionKind::BooleanDecision {
                subject: Box::new(inner_subject),
                when_true: Box::new(when_true),
                when_false: Box::new(when_false),
            },
            value_type,
            int_range,
            rational_value,
            span,
        }))
    }

    #[allow(clippy::too_many_lines)] // Keeps the nested decision proof environment coherent.
    fn analyze_cascading_boolean_comparison_decision(
        &mut self,
        subject: &Expression,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        let Expression::DecisionTable {
            subject: inner_subject,
            rules: inner_rules,
            ..
        } = subject
        else {
            return Ok(None);
        };
        let Some(operand_source) = rules.iter().find_map(|rule| match &rule.matcher {
            DecisionMatcher::Comparison { operand, .. } => Some(operand),
            _ => None,
        }) else {
            return Ok(None);
        };
        if !inner_rules.iter().all(|rule| {
            matches!(
                rule.matcher,
                DecisionMatcher::Boolean { .. } | DecisionMatcher::Otherwise(_)
            )
        }) {
            return Ok(None);
        }
        let inner_subject = self.analyze_expression(inner_subject, environment)?;
        if inner_subject.value_type != CompilerType::Boolean {
            return Ok(None);
        }
        let operand_type = self
            .analyze_expression(operand_source, environment)?
            .value_type;
        let mut when_true = None;
        let mut when_false = None;
        let mut otherwise = None;
        for rule in inner_rules {
            let action = self.analyze_expression(&rule.action, environment)?;
            match rule.matcher {
                DecisionMatcher::Boolean { value: true, .. } => when_true = Some(action),
                DecisionMatcher::Boolean { value: false, .. } => when_false = Some(action),
                DecisionMatcher::Otherwise(_) => otherwise = Some(action),
                _ => unreachable!("preselected Boolean cascading decision"),
            }
        }
        let mut when_true = when_true.or_else(|| otherwise.clone()).ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "Boolean decision does not cover `true`",
            )
        })?;
        let mut when_false = when_false.or(otherwise).ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "Boolean decision does not cover `false`",
            )
        })?;
        let continuation = |value_type: &CompilerType| {
            matches!(
                (value_type, &operand_type),
                (
                    CompilerType::Int | CompilerType::Nat | CompilerType::Rational,
                    CompilerType::Int | CompilerType::Nat | CompilerType::Rational
                )
            ) || value_type == &operand_type
        };
        if continuation(&when_true.value_type) {
            if when_true.value_type == CompilerType::Nat {
                when_true = forget_nat_evidence(when_true);
            }
            when_true = self.analyze_ordered_comparison_decision(
                when_true,
                rules,
                span,
                environment,
                None,
            )?;
        }
        if continuation(&when_false.value_type) {
            if when_false.value_type == CompilerType::Nat {
                when_false = forget_nat_evidence(when_false);
            }
            when_false = self.analyze_ordered_comparison_decision(
                when_false,
                rules,
                span,
                environment,
                None,
            )?;
        }
        adapt_decision_pair(&mut when_true, &mut when_false);
        let (value_type, int_range, rational_value) =
            self.decision_facts(&[&when_true, &when_false], span)?;
        Ok(Some(CompilerExpression {
            kind: CompilerExpressionKind::BooleanDecision {
                subject: Box::new(inner_subject),
                when_true: Box::new(when_true),
                when_false: Box::new(when_false),
            },
            value_type,
            int_range,
            rational_value,
            span,
        }))
    }

    #[allow(clippy::too_many_lines)] // Exhaustive Optional coverage and contextual actions stay adjacent.
    fn analyze_optional_decision(
        &mut self,
        subject: CompilerExpression,
        payload_type: &CompilerType,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
        expected: Option<&CompilerType>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let subject_facts = self.known_structural_value_facts(&subject, environment)?;
        let payload_facts = present_optional_payload(subject_facts)
            .or_else(|| self.list_first_string_payload_facts(&subject, environment));
        let mut some = None;
        let mut none = None;
        let mut otherwise = None;
        for rule in rules {
            if otherwise.is_some() {
                return Err(source_diagnostic(
                    &self.source,
                    "E-UNREACHABLE-DECISION-RULE",
                    rule.span,
                    "an Optional rule cannot follow otherwise",
                ));
            }
            match rule.matcher {
                DecisionMatcher::Optional {
                    some: true,
                    binding: Some(binding),
                    ..
                } if some.is_none() => {
                    let name = self.source.slice(binding).to_owned();
                    let mut branch = decision_binding_environment(
                        environment,
                        &name,
                        payload_type.clone(),
                        binding.start,
                    );
                    if let Some(payload_facts) = &payload_facts {
                        let binding_facts = branch
                            .get_mut(&name)
                            .expect("Optional payload binding was inserted");
                        retain_static_value_facts(binding_facts, payload_facts);
                    }
                    some = Some((
                        name,
                        binding,
                        self.analyze_expression_with_expected(&rule.action, &branch, expected)?,
                    ));
                }
                DecisionMatcher::Optional {
                    some: false,
                    binding: None,
                    ..
                } if none.is_none() => {
                    none = Some(self.analyze_expression_with_expected(
                        &rule.action,
                        environment,
                        expected,
                    )?);
                }
                DecisionMatcher::Otherwise(_) => {
                    otherwise = Some(self.analyze_expression_with_expected(
                        &rule.action,
                        environment,
                        expected,
                    )?);
                }
                DecisionMatcher::Optional { .. } => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-DUPLICATE-DECISION-RULE",
                        rule.span,
                        "an Optional alternative appears more than once",
                    ));
                }
                _ => {
                    return Err(unsupported(
                        &self.source,
                        rule.span,
                        "Optional decision matcher",
                    ));
                }
            }
        }
        let (some_binding, mut some_action) = if let Some((name, binding, action)) = some {
            (Some((name, binding)), action)
        } else if let Some(action) = otherwise.clone() {
            (None, action)
        } else {
            return Err(source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "Optional decision does not cover Some",
            ));
        };
        let mut none_action = none.or(otherwise).ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "Optional decision does not cover None",
            )
        })?;
        let character_adapted = if some_action.value_type == CompilerType::Character
            && none_action.value_type == CompilerType::String
            && Self::known_string_value(&none_action, environment)
                .is_some_and(|value| characters(&value).count() == 1)
        {
            none_action.value_type = CompilerType::Character;
            true
        } else if none_action.value_type == CompilerType::Character
            && some_action.value_type == CompilerType::String
            && Self::known_string_value(&some_action, environment)
                .is_some_and(|value| characters(&value).count() == 1)
        {
            some_action.value_type = CompilerType::Character;
            true
        } else {
            false
        };
        if !character_adapted {
            adapt_decision_pair(&mut some_action, &mut none_action);
        }
        let (value_type, int_range, rational_value) =
            self.decision_facts(&[&some_action, &none_action], span)?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::OptionalDecision {
                subject: Box::new(subject),
                some_binding,
                some_action: Box::new(some_action),
                none_action: Box::new(none_action),
            },
            value_type,
            int_range,
            rational_value,
            span,
        })
    }

    #[allow(clippy::too_many_lines)] // Complete alternatives and exact entry-fact transfer stay adjacent.
    fn analyze_list_decision(
        &mut self,
        subject: CompilerExpression,
        element_type: &CompilerType,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let subject_entries = (element_type == &CompilerType::Function)
            .then(|| self.known_structural_value_facts(&subject, environment))
            .transpose()?
            .and_then(|facts| facts.list_entries);
        let mut entry = None;
        let mut empty = None;
        let mut otherwise = None;
        for rule in rules {
            if otherwise.is_some() {
                return Err(source_diagnostic(
                    &self.source,
                    "E-UNREACHABLE-DECISION-RULE",
                    rule.span,
                    "a List rule cannot follow otherwise",
                ));
            }
            match rule.matcher {
                DecisionMatcher::ListEmpty(_) if empty.is_none() => {
                    empty = Some(self.analyze_expression(&rule.action, environment)?);
                }
                DecisionMatcher::ListEntry { first, rest, .. } if entry.is_none() => {
                    let first_name = self.source.slice(first).to_owned();
                    let rest_name = self.source.slice(rest).to_owned();
                    if first_name == rest_name {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-DUPLICATE-BINDING",
                            rest,
                            "the List entry and remaining List bindings must be distinct",
                        ));
                    }
                    let mut branch = decision_binding_environment(
                        environment,
                        &first_name,
                        element_type.clone(),
                        first.start,
                    );
                    branch = decision_binding_environment(
                        &branch,
                        &rest_name,
                        CompilerType::List(Box::new(element_type.clone())),
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
                    entry = Some((
                        ((first_name, first), (rest_name, rest)),
                        self.analyze_expression(&rule.action, &branch)?,
                    ));
                }
                DecisionMatcher::Otherwise(_) => {
                    otherwise = Some(self.analyze_expression(&rule.action, environment)?);
                }
                DecisionMatcher::ListEmpty(_) | DecisionMatcher::ListEntry { .. } => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-DUPLICATE-DECISION-RULE",
                        rule.span,
                        "a List alternative appears more than once",
                    ));
                }
                _ => {
                    return Err(unsupported(
                        &self.source,
                        rule.span,
                        "List decision matcher",
                    ));
                }
            }
        }
        let (entry_bindings, entry_action) = if let Some((bindings, action)) = entry {
            (Some(bindings), action)
        } else if let Some(action) = otherwise.clone() {
            (None, action)
        } else {
            return Err(source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "List decision does not cover Entry",
            ));
        };
        let empty_action = empty.or(otherwise).ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "List decision does not cover Empty",
            )
        })?;
        let (value_type, int_range, rational_value) =
            self.decision_facts(&[&entry_action, &empty_action], span)?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::ListDecision {
                subject: Box::new(subject),
                entry_bindings,
                entry_action: Box::new(entry_action),
                empty_action: Box::new(empty_action),
            },
            value_type,
            int_range,
            rational_value,
            span,
        })
    }

    fn analyze_enum_decision(
        &mut self,
        subject: CompilerExpression,
        enumeration: &CompilerEnumType,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let mut lowered = Vec::new();
        let mut seen = BTreeSet::new();
        let mut otherwise = None;
        for rule in rules {
            match rule.matcher {
                DecisionMatcher::Identifier(matcher) if otherwise.is_none() => {
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
                            format!("`{label}` is not an alternative of `{}`", enumeration.name),
                        ));
                    };
                    let value =
                        u32::try_from(value).expect("enum declaration already fits the native tag");
                    if !seen.insert(value) {
                        // The earlier source-ordered matcher always selects this
                        // alternative, so the repeated action is unreachable.
                        continue;
                    }
                    lowered.push(CompilerEnumRule {
                        value,
                        action: self.analyze_expression(&rule.action, environment)?,
                        span: rule.span,
                    });
                }
                DecisionMatcher::Otherwise(_) if otherwise.is_none() => {
                    otherwise = Some(Box::new(
                        self.analyze_expression(&rule.action, environment)?,
                    ));
                }
                _ if otherwise.is_some() => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-UNREACHABLE-DECISION-RULE",
                        rule.span,
                        "an Enum rule cannot follow otherwise",
                    ));
                }
                _ => {
                    return Err(unsupported(
                        &self.source,
                        rule.span,
                        "Enum decision matcher",
                    ));
                }
            }
        }
        if otherwise.is_none() && seen.len() != enumeration.alternatives.len() {
            return Err(source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                format!(
                    "decision does not cover every `{}` alternative",
                    enumeration.name
                ),
            ));
        }
        let mut actions = lowered.iter().map(|rule| &rule.action).collect::<Vec<_>>();
        if let Some(action) = &otherwise {
            actions.push(action);
        }
        let (value_type, int_range, rational_value) = self.decision_facts(&actions, span)?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::EnumDecision {
                subject: Box::new(subject),
                rules: lowered,
                otherwise,
            },
            value_type,
            int_range,
            rational_value,
            span,
        })
    }

    #[allow(clippy::too_many_lines)] // Nominal and positional matcher validation stays adjacent to completeness checks.
    fn analyze_sum_decision(
        &mut self,
        subject: CompilerExpression,
        sum: &CompilerSumType,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let mut lowered = Vec::new();
        let mut seen = BTreeSet::new();
        let mut otherwise = None;
        let subject_facts = compiler_type_is_function_aggregate(&subject.value_type)
            .then(|| self.known_structural_value_facts(&subject, environment))
            .transpose()?;
        for rule in rules {
            if otherwise.is_some() {
                return Err(source_diagnostic(
                    &self.source,
                    "E-UNREACHABLE-DECISION-RULE",
                    rule.span,
                    "a sum rule cannot follow otherwise",
                ));
            }
            if matches!(rule.matcher, DecisionMatcher::Otherwise(_)) {
                otherwise = Some(Box::new(
                    self.analyze_expression(&rule.action, environment)?,
                ));
                continue;
            }
            let (value, binding) = match rule.matcher {
                DecisionMatcher::Identifier(matcher) if !sum.positional => {
                    let label = self.source.slice(matcher);
                    let value = sum
                        .alternatives
                        .iter()
                        .position(|alternative| alternative.name == label)
                        .ok_or_else(|| {
                            source_diagnostic(
                                &self.source,
                                "E-UNKNOWN-UNION-ALTERNATIVE",
                                matcher,
                                format!("`{label}` is not an alternative of `{}`", sum.name),
                            )
                        })?;
                    if sum.alternatives[value].payload.is_some() {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-UNION-PAYLOAD-BINDING",
                            matcher,
                            format!("Union alternative `{label}` requires a payload binding"),
                        ));
                    }
                    (value, None)
                }
                DecisionMatcher::Union {
                    alternative,
                    binding,
                    ..
                } if !sum.positional => {
                    let label = self.source.slice(alternative);
                    let value = sum
                        .alternatives
                        .iter()
                        .position(|candidate| candidate.name == label)
                        .ok_or_else(|| {
                            source_diagnostic(
                                &self.source,
                                "E-UNKNOWN-UNION-ALTERNATIVE",
                                alternative,
                                format!("`{label}` is not an alternative of `{}`", sum.name),
                            )
                        })?;
                    if sum.alternatives[value].payload.is_none() {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-UNION-PAYLOAD-BINDING",
                            binding,
                            format!("Union alternative `{label}` has no payload"),
                        ));
                    }
                    (value, Some(binding))
                }
                DecisionMatcher::Variant {
                    type_name,
                    index,
                    binding,
                    ..
                } if sum.positional => {
                    let matcher_type = self.source.slice(type_name);
                    if matcher_type != sum.name {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-VARIANT-TYPE",
                            type_name,
                            format!(
                                "Variant matcher `{matcher_type}` does not match `{}`",
                                sum.name
                            ),
                        ));
                    }
                    let value = parse_integer(self.source.slice(index))
                        .and_then(|value| value.to_string().parse::<usize>().ok())
                        .filter(|value| *value < sum.alternatives.len())
                        .ok_or_else(|| {
                            source_diagnostic(
                                &self.source,
                                "E-VARIANT-INDEX",
                                index,
                                "Variant alternative index is outside its declared bounds",
                            )
                        })?;
                    (value, Some(binding))
                }
                _ => {
                    return Err(unsupported(&self.source, rule.span, "sum decision matcher"));
                }
            };
            let value = u32::try_from(value).expect("sum alternative fits native tag");
            if !seen.insert(value) {
                continue;
            }
            let binding = binding.map(|binding| (self.source.slice(binding).to_owned(), binding));
            let mut branch = if let Some((name, binding_span)) = &binding {
                decision_binding_environment(
                    environment,
                    name,
                    sum.alternatives[usize::try_from(value).expect("u32 sum tag fits usize")]
                        .payload
                        .clone()
                        .expect("payload matcher selected a payload alternative"),
                    binding_span.start,
                )
            } else {
                environment.clone()
            };
            if let (Some((name, _)), Some(subject_facts)) = (&binding, &subject_facts)
                && let Some(sum_facts) = &subject_facts.sum
                && sum_facts.alternative == value
                && let Some(payload_facts) = sum_facts.payload.as_deref()
            {
                let binding = branch
                    .get_mut(name)
                    .expect("payload decision binding was inserted");
                retain_static_value_facts(binding, payload_facts);
            }
            lowered.push(CompilerSumRule {
                value,
                binding,
                action: self.analyze_expression(&rule.action, &branch)?,
                span: rule.span,
            });
        }
        if otherwise.is_none() && seen.len() != sum.alternatives.len() {
            return Err(source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                format!("decision does not cover every `{}` alternative", sum.name),
            ));
        }
        let mut actions = lowered.iter().map(|rule| &rule.action).collect::<Vec<_>>();
        if let Some(action) = &otherwise {
            actions.push(action);
        }
        let (value_type, int_range, rational_value) = self.decision_facts(&actions, span)?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::SumDecision {
                subject: Box::new(subject),
                rules: lowered,
                otherwise,
            },
            value_type,
            int_range,
            rational_value,
            span,
        })
    }

    #[allow(clippy::too_many_lines)] // Result binding, reachability, and completeness checks stay adjacent.
    fn analyze_result_decision(
        &mut self,
        subject: CompilerExpression,
        success_type: &CompilerType,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
        expected: Option<&CompilerType>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let subject_facts = (success_type == &CompilerType::Function)
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
                        success_type.clone(),
                        binding.start,
                    );
                    if let Some(success_facts) = subject_facts
                        .as_ref()
                        .and_then(|facts| facts.result.as_ref())
                    {
                        let binding = branch
                            .get_mut(&name)
                            .expect("Result success decision binding was inserted");
                        retain_static_value_facts(binding, &success_facts.success);
                    }
                    ok = Some((
                        name,
                        binding,
                        self.analyze_expression_with_expected(&rule.action, &branch, expected)?,
                    ));
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
                    error_fallback = Some((
                        name,
                        binding,
                        Box::new(self.analyze_expression_with_expected(
                            &rule.action,
                            &branch,
                            expected,
                        )?),
                    ));
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
                    error_codes.push(CompilerErrorCodeRule {
                        code: code_value,
                        action: self.analyze_expression_with_expected(
                            &rule.action,
                            environment,
                            expected,
                        )?,
                        span: rule.span,
                    });
                }
                _ => {
                    return Err(unsupported(
                        &self.source,
                        rule.span,
                        "Result decision matcher",
                    ));
                }
            }
        }
        let (ok_binding, ok_binding_span, ok_action) = ok.ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "Result decision does not cover Ok",
            )
        })?;
        if error_fallback.is_none() && seen_codes != BTreeSet::from([0, 1, 2, 3]) {
            return Err(source_diagnostic(
                &self.source,
                "E-INCOMPLETE-ERROR-CODE-DECISION",
                span,
                "Result decision requires Error fallback or every arithmetic Error code",
            ));
        }
        let mut actions = vec![&ok_action];
        actions.extend(error_codes.iter().map(|rule| &rule.action));
        if let Some((_, _, action)) = &error_fallback {
            actions.push(action);
        }
        let (value_type, int_range, rational_value) = self.decision_facts(&actions, span)?;
        Ok(CompilerExpression {
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
            span,
        })
    }

    #[allow(clippy::too_many_lines)] // Keeps branch bindings and proof propagation together.
    fn analyze_boolean_decision(
        &mut self,
        subject: CompilerExpression,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        require_type(
            &self.source,
            subject.span,
            &CompilerType::Boolean,
            &subject.value_type,
        )?;
        let mut when_true = None;
        let mut when_false = None;
        let mut otherwise = None;
        let nonzero_when_false = equality_zero_local(&subject)
            .or_else(|| less_equal_zero_nat_local(&subject, environment))
            .map(ToOwned::to_owned);
        let nonzero_when_true = greater_nat_local(&subject, environment)
            .or_else(|| call_true_nonzero_local(&subject, &self.true_nonzero_parameters))
            .map(ToOwned::to_owned);
        let lower_bound_when_true = greater_nat_local_bound(&subject, environment);
        let lower_bound_when_false = less_equal_nat_local_false_bound(&subject, environment)
            .or_else(|| {
                nonzero_when_false.as_ref().and_then(|name| {
                    (binding_facts_by_storage(environment, name)
                        .is_some_and(|facts| facts.value_type == CompilerType::Nat)
                        || self.active_nonnegative_bindings.contains(name))
                    .then(|| (name.clone(), BigInt::from(1)))
                })
            });
        let nonzero_call_when_false =
            equality_zero_call(&subject).and_then(|call| self.compiler_call_proof_key(call));
        let mut nonnegative_when_false = false_branch_nonnegative_locals(&subject);
        extend_false_branch_nonnegative_locals(&subject, &mut nonnegative_when_false);
        for rule in rules {
            match rule.matcher {
                DecisionMatcher::Boolean { value: true, .. } if when_true.is_none() => {
                    let inserted = nonzero_when_true
                        .as_ref()
                        .is_some_and(|name| self.active_nonzero_bindings.insert(name.clone()));
                    let previous_bounds = self.active_lower_bounds.clone();
                    if let Some((name, bound)) = &lower_bound_when_true {
                        self.active_lower_bounds
                            .entry(name.clone())
                            .and_modify(|current| *current = current.clone().max(bound.clone()))
                            .or_insert_with(|| bound.clone());
                    }
                    let action = self.analyze_expression(&rule.action, environment);
                    self.active_lower_bounds = previous_bounds;
                    if inserted {
                        self.active_nonzero_bindings
                            .remove(nonzero_when_true.as_ref().expect("inserted binding"));
                    }
                    when_true = Some(action?);
                }
                DecisionMatcher::Boolean { value: false, .. } if when_false.is_none() => {
                    let inserted = nonzero_when_false
                        .as_ref()
                        .is_some_and(|name| self.active_nonzero_bindings.insert(name.clone()));
                    let previous_bounds = self.active_lower_bounds.clone();
                    if let Some((name, bound)) = &lower_bound_when_false {
                        self.active_lower_bounds
                            .entry(name.clone())
                            .and_modify(|current| *current = current.clone().max(bound.clone()))
                            .or_insert_with(|| bound.clone());
                    }
                    let inserted_call = nonzero_call_when_false
                        .as_ref()
                        .is_some_and(|key| self.active_nonzero_calls.insert(key.clone()));
                    let inserted_nonnegative = nonnegative_when_false
                        .iter()
                        .filter(|name| self.active_nonnegative_bindings.insert((*name).clone()))
                        .cloned()
                        .collect::<Vec<_>>();
                    let action = self.analyze_expression(&rule.action, environment);
                    for name in inserted_nonnegative {
                        self.active_nonnegative_bindings.remove(&name);
                    }
                    if inserted_call {
                        self.active_nonzero_calls.remove(
                            nonzero_call_when_false
                                .as_ref()
                                .expect("inserted nonzero call"),
                        );
                    }
                    self.active_lower_bounds = previous_bounds;
                    if inserted {
                        self.active_nonzero_bindings
                            .remove(nonzero_when_false.as_ref().expect("inserted binding"));
                    }
                    when_false = Some(action?);
                }
                DecisionMatcher::Otherwise(_) if otherwise.is_none() => {
                    let inserted = nonzero_when_false
                        .as_ref()
                        .is_some_and(|name| self.active_nonzero_bindings.insert(name.clone()));
                    let previous_bounds = self.active_lower_bounds.clone();
                    if let Some((name, bound)) = &lower_bound_when_false {
                        self.active_lower_bounds
                            .entry(name.clone())
                            .and_modify(|current| *current = current.clone().max(bound.clone()))
                            .or_insert_with(|| bound.clone());
                    }
                    let inserted_call = nonzero_call_when_false
                        .as_ref()
                        .is_some_and(|key| self.active_nonzero_calls.insert(key.clone()));
                    let inserted_nonnegative = nonnegative_when_false
                        .iter()
                        .filter(|name| self.active_nonnegative_bindings.insert((*name).clone()))
                        .cloned()
                        .collect::<Vec<_>>();
                    let action = self.analyze_expression(&rule.action, environment);
                    for name in inserted_nonnegative {
                        self.active_nonnegative_bindings.remove(&name);
                    }
                    if inserted_call {
                        self.active_nonzero_calls.remove(
                            nonzero_call_when_false
                                .as_ref()
                                .expect("inserted nonzero call"),
                        );
                    }
                    self.active_lower_bounds = previous_bounds;
                    if inserted {
                        self.active_nonzero_bindings
                            .remove(nonzero_when_false.as_ref().expect("inserted binding"));
                    }
                    otherwise = Some(action?);
                }
                _ => return Err(unsupported(&self.source, rule.span, "decision matcher")),
            }
        }
        let mut when_true = when_true.or_else(|| otherwise.clone()).ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "Boolean decision does not cover `true`",
            )
        })?;
        let mut when_false = when_false.or(otherwise).ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "Boolean decision does not cover `false`",
            )
        })?;
        adapt_decision_pair(&mut when_true, &mut when_false);
        let (value_type, int_range, rational_value) =
            self.decision_facts(&[&when_true, &when_false], span)?;
        Ok(CompilerExpression {
            value_type,
            kind: CompilerExpressionKind::BooleanDecision {
                subject: Box::new(subject),
                when_true: Box::new(when_true),
                when_false: Box::new(when_false),
            },
            int_range,
            rational_value,
            span,
        })
    }

    #[allow(clippy::too_many_lines)] // Numeric ordering and structural equality share one ordered rule-selection lowering.
    fn analyze_ordered_comparison_decision(
        &mut self,
        subject: CompilerExpression,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
        expected: Option<&CompilerType>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let numeric = matches!(
            subject.value_type,
            CompilerType::Int | CompilerType::Rational
        );
        let mut lowered = Vec::new();
        let mut otherwise = None;
        let mut nonzero_otherwise = None;
        for (index, rule) in rules.iter().enumerate() {
            match &rule.matcher {
                DecisionMatcher::Comparison {
                    kind,
                    operand,
                    span: matcher_span,
                } => {
                    if otherwise.is_some() {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-UNREACHABLE-DECISION-RULE",
                            rule.span,
                            "a comparison rule cannot follow otherwise",
                        ));
                    }
                    let Some(operation) = comparison_binary(*kind) else {
                        return Err(unsupported(
                            &self.source,
                            *matcher_span,
                            "comparison decision callable",
                        ));
                    };
                    if !numeric
                        && !matches!(operation, CompilerBinary::Equal | CompilerBinary::NotEqual)
                        && !compiler_ordering_supported(&subject.value_type)
                    {
                        return Err(unsupported(
                            &self.source,
                            *matcher_span,
                            "ordered comparison for this decision subject type",
                        ));
                    }
                    let mut operand = self.analyze_expression(operand, environment)?;
                    if numeric && operand.value_type == CompilerType::Nat {
                        operand = forget_nat_evidence(operand);
                    }
                    if numeric {
                        require_exact_numeric(&self.source, operand.span, &operand.value_type)?;
                    }
                    let subject_to_rational = numeric
                        && subject.value_type == CompilerType::Int
                        && operand.value_type == CompilerType::Rational;
                    if numeric
                        && subject.value_type == CompilerType::Rational
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
                    if (operation == CompilerBinary::LessEqual
                        || operation == CompilerBinary::Equal)
                        && exact_int(&operand).is_some_and(|value| value == BigInt::from(0))
                        && let CompilerExpressionKind::Local(name) = &subject.kind
                        && binding_facts_by_storage(environment, name).is_some_and(|facts| {
                            (operation == CompilerBinary::LessEqual
                                && facts.value_type == CompilerType::Nat)
                                || (operation == CompilerBinary::Equal
                                    && facts.value_type == CompilerType::Int)
                        })
                    {
                        nonzero_otherwise = Some(name.clone());
                    }
                    lowered.push(CompilerComparisonRule {
                        operation,
                        operand,
                        action: self.analyze_expression_with_expected(
                            &rule.action,
                            environment,
                            expected,
                        )?,
                        subject_to_rational,
                        span: rule.span,
                    });
                }
                DecisionMatcher::Otherwise(_) if index + 1 == rules.len() => {
                    let inserted = nonzero_otherwise
                        .as_ref()
                        .is_some_and(|name| self.active_nonzero_bindings.insert(name.clone()));
                    let action =
                        self.analyze_expression_with_expected(&rule.action, environment, expected);
                    if inserted {
                        self.active_nonzero_bindings
                            .remove(nonzero_otherwise.as_ref().expect("inserted binding"));
                    }
                    otherwise = Some(action?);
                }
                DecisionMatcher::Otherwise(_) => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-UNREACHABLE-DECISION-RULE",
                        rule.span,
                        "otherwise must be the final comparison rule",
                    ));
                }
                _ => return Err(unsupported(&self.source, rule.span, "decision matcher")),
            }
        }
        let mut otherwise = otherwise.ok_or_else(|| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                "a comparison decision requires otherwise",
            )
        })?;
        for rule in &mut lowered {
            adapt_decision_pair(&mut rule.action, &mut otherwise);
        }
        let mut branches = lowered.iter().map(|rule| &rule.action).collect::<Vec<_>>();
        branches.push(&otherwise);
        let (value_type, int_range, rational_value) = self.decision_facts(&branches, span)?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::OrderedComparisonDecision {
                subject: Box::new(subject),
                rules: lowered,
                otherwise: Box::new(otherwise),
            },
            value_type,
            int_range,
            rational_value,
            span,
        })
    }
}
