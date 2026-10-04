impl Analyzer {
    fn analyze_comparison_value_decision(
        &mut self,
        subject: CompilerExpression,
        rules: &[topal_syntax::DecisionRule],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let mut when_less = None;
        let mut when_equal = None;
        let mut when_greater = None;
        let mut otherwise = None;
        for (index, rule) in rules.iter().enumerate() {
            let destination = match &rule.matcher {
                DecisionMatcher::Identifier(name) => match self.source.slice(*name) {
                    "Less" => &mut when_less,
                    "Equal" => &mut when_equal,
                    "Greater" => &mut when_greater,
                    _ => return Err(unsupported(&self.source, *name, "Comparison alternative")),
                },
                DecisionMatcher::Otherwise(_) if index + 1 == rules.len() => &mut otherwise,
                DecisionMatcher::Otherwise(_) => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-UNREACHABLE-DECISION-RULE",
                        rule.span,
                        "otherwise must be the final Comparison rule",
                    ));
                }
                _ => return Err(unsupported(&self.source, rule.span, "decision matcher")),
            };
            if destination.is_some() {
                return Err(source_diagnostic(
                    &self.source,
                    "E-DUPLICATE-DECISION-RULE",
                    rule.span,
                    "a Comparison alternative appears more than once",
                ));
            }
            *destination = Some(self.analyze_expression(&rule.action, environment)?);
        }
        let missing = |name: &str| {
            source_diagnostic(
                &self.source,
                "E-INCOMPLETE-DECISION",
                span,
                format!("Comparison decision does not cover {name}"),
            )
        };
        let when_less = when_less
            .or_else(|| otherwise.clone())
            .ok_or_else(|| missing("Less"))?;
        let when_equal = when_equal
            .or_else(|| otherwise.clone())
            .ok_or_else(|| missing("Equal"))?;
        let when_greater = when_greater
            .or(otherwise)
            .ok_or_else(|| missing("Greater"))?;
        let (value_type, int_range, rational_value) =
            self.decision_facts(&[&when_less, &when_equal, &when_greater], span)?;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::ComparisonValueDecision {
                subject: Box::new(subject),
                when_less: Box::new(when_less),
                when_equal: Box::new(when_equal),
                when_greater: Box::new(when_greater),
            },
            value_type,
            int_range,
            rational_value,
            span,
        })
    }

    fn decision_facts(
        &self,
        branches: &[&CompilerExpression],
        span: Span,
    ) -> Result<(CompilerType, Option<IntRange>, Option<BigRational>), Diagnostic> {
        let first = branches.first().expect("a complete decision has a branch");
        for branch in &branches[1..] {
            require_same_type(&self.source, span, &first.value_type, &branch.value_type)?;
        }
        if !compiler_function_result_supported(&first.value_type) {
            return Err(unsupported(
                &self.source,
                span,
                "decision actions with unsupported machine results",
            ));
        }
        let int_range = branches
            .iter()
            .try_fold(None, |range: Option<IntRange>, branch| {
                match (range, &branch.int_range) {
                    (None, Some(next)) => Some(Some(next.clone())),
                    (Some(current), Some(next)) => Some(Some(IntRange::union(&current, next))),
                    (_, None) => None,
                }
            })
            .flatten();
        let rational_value = first.rational_value.clone().filter(|value| {
            branches
                .iter()
                .all(|branch| branch.rational_value.as_ref() == Some(value))
        });
        Ok((first.value_type.clone(), int_range, rational_value))
    }
}
