impl Analyzer {
    #[allow(clippy::too_many_lines)] // Exhaustive expression admission keeps the subset boundary visible.
    fn analyze_expression(
        &mut self,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let span = expression.span();
        match expression {
            Expression::Block { statements, .. } => {
                let mut nested = environment.clone();
                let mut block =
                    self.analyze_block(statements, &mut nested, BlockKind::Lexical, None)?;
                if statements.is_empty() {
                    block.result = unit_expression(span);
                }
                Ok(CompilerExpression {
                    value_type: block.result.value_type.clone(),
                    int_range: block.result.int_range.clone(),
                    rational_value: block.result.rational_value.clone(),
                    kind: CompilerExpressionKind::Block(Box::new(block)),
                    span,
                })
            }
            Expression::Unit(_) => Ok(unit_expression(span)),
            Expression::Identifier(name) if self.source.slice(*name) == "Completed" => {
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Completed,
                    value_type: CompilerType::Completed,
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Identifier(name) if self.source.slice(*name) == "root" => {
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Root,
                    value_type: CompilerType::Scope,
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::ContextIdentifier(member) => {
                if !self.in_function {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-CONTEXT-SELECTION",
                        *member,
                        "defining-context selection is available only inside a function body",
                    ));
                }
                let member_name = self.source.slice(*member);
                let parameter_name = format!("@ {member_name}");
                let facts = environment.get(&parameter_name).ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-COMPILER-UNSUPPORTED",
                        *member,
                        format!(
                            "compiler increment cannot capture defining-context member `{member_name}`"
                        ),
                    )
                })?;
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Local(facts.storage_name.clone()),
                    value_type: facts.value_type.clone(),
                    int_range: facts.int_range.clone(),
                    rational_value: facts.rational_value.clone(),
                    span,
                })
            }
            Expression::AnonymousFunction {
                parameters,
                body,
                span,
            } => {
                self.function_values_used = true;
                let mut parameter_names = BTreeSet::new();
                for parameter in parameters {
                    collect_anonymous_pattern_names(&self.source, parameter, &mut parameter_names);
                }
                let captures = environment
                    .iter()
                    .filter(|(name, _)| {
                        !parameter_names.contains(name.as_str())
                            && expression_mentions_environment_binding(&self.source, body, name)
                    })
                    .map(|(name, facts)| (name.clone(), facts.clone()))
                    .collect();
                let tag = if let Some(tag) =
                    self.anonymous_function_value_tags.get(&span.start).copied()
                {
                    tag
                } else {
                    let tag_index = self
                        .functions
                        .len()
                        .checked_add(COMPILER_SYMBOLIC_CALLABLES.len())
                        .and_then(|value| {
                            value.checked_add(self.anonymous_function_value_names.len())
                        })
                        .ok_or_else(|| {
                            unsupported(&self.source, *span, "native Function value tag")
                        })?;
                    let tag = u32::try_from(tag_index).map_err(|_| {
                        unsupported(&self.source, *span, "native Function value tag")
                    })?;
                    let display = format!("<anonymous fn/{}>", parameters.len());
                    self.anonymous_function_value_names.push(display);
                    self.anonymous_function_value_tags.insert(span.start, tag);
                    tag
                };
                self.anonymous_callables.insert(
                    tag,
                    CompilerCallableFacts::Anonymous {
                        parameters: parameters.clone(),
                        body: body.as_ref().clone(),
                        captures,
                        static_context: self.static_context,
                        span: *span,
                    },
                );
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::FunctionValue(tag),
                    value_type: CompilerType::Function,
                    int_range: None,
                    rational_value: None,
                    span: *span,
                })
            }
            Expression::Callable { kind, span } => {
                self.function_values_used = true;
                let offset = COMPILER_SYMBOLIC_CALLABLES
                    .iter()
                    .position(|(candidate, _)| candidate == kind)
                    .expect("every source symbolic callable has a compiler observation tag");
                let value = self
                    .functions
                    .len()
                    .checked_add(offset)
                    .and_then(|value| u32::try_from(value).ok())
                    .ok_or_else(|| unsupported(&self.source, *span, "native Function value tag"))?;
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::FunctionValue(value),
                    value_type: CompilerType::Function,
                    int_range: None,
                    rational_value: None,
                    span: *span,
                })
            }
            Expression::Identifier(name)
                if !environment.contains_key(self.source.slice(*name))
                    && self.functions.contains_key(self.source.slice(*name)) =>
            {
                self.function_values_used = true;
                let function_name = self.source.slice(*name);
                let declarations = self
                    .functions
                    .get(function_name)
                    .expect("guard established a function declaration");
                if !declarations
                    .iter()
                    .any(|declaration| declaration.span.end <= name.start)
                {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-UNBOUND-NAME",
                        *name,
                        format!("function `{function_name}` is not yet a value"),
                    ));
                }
                let value = self
                    .functions
                    .keys()
                    .position(|candidate| candidate == function_name)
                    .and_then(|value| u32::try_from(value).ok())
                    .ok_or_else(|| unsupported(&self.source, *name, "native Function value tag"))?;
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::FunctionValue(value),
                    value_type: CompilerType::Function,
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Identifier(name)
                if environment
                    .get(self.source.slice(*name))
                    .and_then(|facts| facts.static_capability.as_ref())
                    .is_some() =>
            {
                let capability = environment
                    .get(self.source.slice(*name))
                    .and_then(|facts| facts.static_capability.clone())
                    .expect("guard established retained Capability metadata");
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Capability(capability),
                    value_type: CompilerType::Capability,
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Identifier(name) if fundamental_capability(self.source.slice(*name)) => {
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Capability(CompilerCapability::atomic(
                        self.source.slice(*name),
                    )),
                    value_type: CompilerType::Capability,
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Identifier(name)
                if fundamental_type_value(self.source.slice(*name)).is_some() =>
            {
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::TypeValue(
                        fundamental_type_value(self.source.slice(*name))
                            .expect("guard established a fundamental Type value"),
                    ),
                    value_type: CompilerType::Type,
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Identifier(name)
                if compiler_layout_policy(self.source.slice(*name)).is_some() =>
            {
                let (enumeration, value) = compiler_layout_policy(self.source.slice(*name))
                    .expect("guard established a fundamental layout policy");
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Enum(value),
                    value_type: CompilerType::Enum(enumeration),
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Boolean(value) => Ok(CompilerExpression {
                kind: CompilerExpressionKind::Boolean(self.source.slice(*value) == "true"),
                value_type: CompilerType::Boolean,
                int_range: None,
                rational_value: None,
                span,
            }),
            Expression::Integer(value) => {
                let integer = parse_integer(self.source.slice(*value)).ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-NUMERIC-LITERAL",
                        *value,
                        "invalid integer literal",
                    )
                })?;
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Int(integer.clone()),
                    value_type: CompilerType::Int,
                    int_range: Some(IntRange::exact(integer)),
                    rational_value: None,
                    span,
                })
            }
            Expression::Infinity(span) => Err(source_diagnostic(
                &self.source,
                "E-INFINITY-CONTEXT",
                *span,
                "an infinity constant requires an explicit supported numeric classifier",
            )),
            Expression::Rational(value) => {
                let rational = parse_rational(self.source.slice(*value)).ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-NUMERIC-LITERAL",
                        *value,
                        "invalid rational literal",
                    )
                })?;
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Rational(rational.clone()),
                    value_type: CompilerType::Rational,
                    int_range: None,
                    rational_value: Some(rational),
                    span,
                })
            }
            Expression::String(value) => {
                let value = parse_string(self.source.slice(*value)).ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-STRING-LITERAL",
                        *value,
                        "invalid string literal delimiter",
                    )
                })?;
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::String(value.to_owned()),
                    value_type: CompilerType::String,
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Identifier(name)
                if !environment.contains_key(self.source.slice(*name))
                    && self
                        .enum_alternatives
                        .get(self.source.slice(*name))
                        .is_some_and(|(_, _, declaration)| declaration.end <= name.start) =>
            {
                let (enumeration, value, _) = self
                    .enum_alternatives
                    .get(self.source.slice(*name))
                    .expect("checked enum alternative exists")
                    .clone();
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Enum(value),
                    value_type: CompilerType::Enum(enumeration),
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Identifier(name)
                if !environment.contains_key(self.source.slice(*name))
                    && self
                        .sum_alternatives
                        .get(self.source.slice(*name))
                        .is_some_and(|(sum, value, declaration)| {
                            declaration.end <= name.start
                                && sum.alternatives
                                    [usize::try_from(*value).expect("u32 sum tag fits usize")]
                                .payload
                                .is_none()
                        }) =>
            {
                let (sum, value, _) = self
                    .sum_alternatives
                    .get(self.source.slice(*name))
                    .expect("checked payload-free Union alternative exists")
                    .clone();
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Sum {
                        value,
                        payload: None,
                    },
                    value_type: CompilerType::Sum(sum),
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Product { fields, .. } => {
                if !fields.is_empty() && fields.iter().all(|field| field.label.is_some()) {
                    let mut values = Vec::with_capacity(fields.len());
                    let mut value_types = Vec::with_capacity(fields.len());
                    for field in fields {
                        let label_span = field.label.expect("record fields are labeled");
                        let label = self.source.slice(label_span).to_owned();
                        if values
                            .iter()
                            .any(|(existing, _): &(String, CompilerExpression)| existing == &label)
                        {
                            return Err(source_diagnostic(
                                &self.source,
                                "E-DUPLICATE-RECORD-FIELD",
                                label_span,
                                "record field label occurs more than once",
                            ));
                        }
                        let value = self.analyze_expression(&field.value, environment)?;
                        value_types.push((label.clone(), value.value_type.clone()));
                        values.push((label, value));
                    }
                    if value_types
                        .iter()
                        .any(|(_, value_type)| compiler_type_contains_generator(value_type))
                    {
                        return Err(unsupported(
                            &self.source,
                            span,
                            "generator containment in a product",
                        ));
                    }
                    value_types.sort_by(|left, right| left.0.cmp(&right.0));
                    return Ok(CompilerExpression {
                        value_type: CompilerType::Record(value_types),
                        kind: CompilerExpressionKind::Record(values),
                        int_range: None,
                        rational_value: None,
                        span,
                    });
                }
                let values = fields
                    .iter()
                    .map(|field| self.analyze_expression(&field.value, environment))
                    .collect::<Result<Vec<_>, _>>()?;
                if values
                    .iter()
                    .any(|value| compiler_type_contains_generator(&value.value_type))
                {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "generator containment in a product",
                    ));
                }
                Ok(CompilerExpression {
                    value_type: CompilerType::Tuple(
                        values
                            .iter()
                            .map(|value| value.value_type.clone())
                            .collect(),
                    ),
                    kind: CompilerExpressionKind::Tuple(values),
                    int_range: None,
                    rational_value: None,
                    span,
                })
            }
            Expression::Identifier(name) => {
                let name_text = self.source.slice(*name);
                let facts = environment.get(name_text).ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-UNBOUND-NAME",
                        *name,
                        format!("name `{name_text}` is not bound"),
                    )
                })?;
                if !facts.runtime_bound {
                    if facts.value_type == CompilerType::Function
                        && let Some(mut callable @ CompilerCallableFacts::Named { .. }) =
                            facts.callable.clone()
                    {
                        self.extend_named_callable_environment_captures(
                            &mut callable,
                            environment,
                        )?;
                        self.function_values_used = true;
                        let tag = if let Some(tag) = self
                            .nested_function_value_tags
                            .get(&facts.storage_name)
                            .copied()
                        {
                            self.anonymous_callables.insert(tag, callable);
                            tag
                        } else {
                            let tag_index = self
                                .functions
                                .len()
                                .checked_add(COMPILER_SYMBOLIC_CALLABLES.len())
                                .and_then(|value| {
                                    value.checked_add(self.anonymous_function_value_names.len())
                                })
                                .ok_or_else(|| {
                                    unsupported(&self.source, *name, "native Function value tag")
                                })?;
                            let tag = u32::try_from(tag_index).map_err(|_| {
                                unsupported(&self.source, *name, "native Function value tag")
                            })?;
                            self.anonymous_function_value_names
                                .push(format!("<fn {name_text}>"));
                            self.anonymous_callables.insert(tag, callable);
                            self.nested_function_value_tags
                                .insert(facts.storage_name.clone(), tag);
                            tag
                        };
                        return Ok(CompilerExpression {
                            kind: CompilerExpressionKind::FunctionValue(tag),
                            value_type: CompilerType::Function,
                            int_range: None,
                            rational_value: None,
                            span: *name,
                        });
                    }
                    let feature = if compiler_type_contains_static_only(&facts.value_type) {
                        "runtime use of a static compiler value"
                    } else {
                        "nested Function value outside direct application"
                    };
                    return Err(unsupported(&self.source, *name, feature));
                }
                if matches!(facts.value_type, CompilerType::Generator(_))
                    && !self.consumed_generators.insert(facts.storage_name.clone())
                {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-GENERATOR-CONSUMED",
                        *name,
                        format!("generator `{name_text}` was already consumed"),
                    ));
                }
                Ok(CompilerExpression {
                    kind: facts.infinity_negative.map_or_else(
                        || CompilerExpressionKind::Local(facts.storage_name.clone()),
                        |negative| CompilerExpressionKind::InfinityLocal {
                            storage_name: facts.storage_name.clone(),
                            negative,
                        },
                    ),
                    value_type: facts.value_type.clone(),
                    int_range: facts.int_range.clone(),
                    rational_value: facts.rational_value.clone(),
                    span,
                })
            }
            Expression::Application { items, .. } => {
                self.analyze_application(items, span, environment)
            }
            Expression::DecisionTable { subject, rules, .. } => {
                self.analyze_decision(subject, rules, span, environment)
            }
            _ => Err(unsupported(&self.source, span, "expression form")),
        }
    }

    #[allow(clippy::too_many_lines)] // Predicate lowering and nominal registration remain one checked transaction.
    fn analyze_constraint_definition(
        &mut self,
        name: &str,
        expression: &Expression,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let (base, parameters, predicate, span) =
            constraint_definition(&self.source, expression).expect("preselected constraint");
        let base_type = self.parse_classifier(base)?;
        if !matches!(
            base_type,
            CompilerType::Boolean
                | CompilerType::Int
                | CompilerType::Nat
                | CompilerType::Rational
                | CompilerType::String
        ) {
            return Err(unsupported(
                &self.source,
                base,
                "native constraint base classifier",
            ));
        }
        let [AnonymousPattern::Binding(parameter)] = parameters else {
            return Err(unsupported(
                &self.source,
                span,
                "native constraint predicate pattern",
            ));
        };
        let parameter_name = self.source.slice(*parameter).to_owned();
        if let Some(capture) = environment.keys().find(|candidate| {
            candidate.as_str() != parameter_name
                && expression_mentions_name(&self.source, predicate, candidate)
        }) {
            return Err(unsupported(
                &self.source,
                predicate.span(),
                &format!("captured constraint predicate value `{capture}`"),
            ));
        }
        let storage_name = format!("topal.constraint.{}.{}", span.start, parameter_name);
        let mut predicate_environment = BTreeMap::new();
        predicate_environment.insert(
            parameter_name.clone(),
            BindingFacts {
                storage_name: storage_name.clone(),
                origin: span.start,
                runtime_bound: true,
                value_type: base_type.clone(),
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
        let previous_in_function = self.in_function;
        self.in_function = true;
        let predicate_result =
            if nonempty_string_constraint_predicate(&self.source, predicate, &parameter_name) {
                let parameter = CompilerExpression {
                    kind: CompilerExpressionKind::Local(storage_name.clone()),
                    value_type: CompilerType::String,
                    int_range: None,
                    rational_value: None,
                    span: predicate.span(),
                };
                Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Not(Box::new(CompilerExpression {
                        kind: CompilerExpressionKind::StringEmptyPredicate(Box::new(parameter)),
                        value_type: CompilerType::Boolean,
                        int_range: None,
                        rational_value: None,
                        span: predicate.span(),
                    })),
                    value_type: CompilerType::Boolean,
                    int_range: None,
                    rational_value: None,
                    span: predicate.span(),
                })
            } else {
                self.analyze_expression(predicate, &predicate_environment)
            };
        self.in_function = previous_in_function;
        let predicate = predicate_result?;
        require_type(
            &self.source,
            predicate.span,
            &CompilerType::Boolean,
            &predicate.value_type,
        )?;
        let tag = u32::try_from(self.constraints.len())
            .map_err(|_| unsupported(&self.source, span, "native Constraint value tag"))?;
        self.constraints.push(CompilerConstraint {
            name: name.to_owned(),
            base_type,
            parameter: parameter_name,
            parameter_storage: storage_name,
            predicate,
            span,
        });
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::ConstraintValue(tag),
            value_type: CompilerType::Constraint,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_static_introspection(
        &self,
        items: &[Expression],
        span: Span,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        if let Some(value) = self.analyze_static_introspection_prefix(items, span)? {
            return Ok(Some(value));
        }
        self.analyze_static_introspection_relation(items, span)
    }

    fn analyze_static_introspection_prefix(
        &self,
        items: &[Expression],
        span: Span,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        if let [
            Expression::Identifier(namespace),
            Expression::Identifier(operation),
        ] = items
            && self.source.slice(*namespace) == "lang"
        {
            let value = match self.source.slice(*operation) {
                "context" => CompilerExpression {
                    kind: CompilerExpressionKind::LanguageContext(CompilerLanguageContext {
                        language: "topal".into(),
                        version: self.language_version,
                        features: self.language_features.clone(),
                    }),
                    value_type: CompilerType::LanguageContext,
                    int_range: None,
                    rational_value: None,
                    span,
                },
                "version" => CompilerExpression {
                    kind: CompilerExpressionKind::Version(self.language_version),
                    value_type: CompilerType::Version,
                    int_range: None,
                    rational_value: None,
                    span,
                },
                "lint" => self.analyze_lint_namespace(*operation, span)?,
                _ => return Ok(None),
            };
            return Ok(Some(value));
        }

        if let [
            Expression::Identifier(namespace),
            Expression::Identifier(operation),
            subject,
        ] = items
            && self.source.slice(*namespace) == "lang"
            && matches!(self.source.slice(*operation), "identity" | "view")
        {
            let Expression::Identifier(subject_span) = subject else {
                return Err(unsupported(
                    &self.source,
                    subject.span(),
                    "static introspection of a runtime value",
                ));
            };
            let identity = self.source.slice(*subject_span);
            if fundamental_type_value(identity).is_none() {
                if self.source.slice(*operation) == "view" {
                    return Ok(None);
                }
                return Err(unsupported(
                    &self.source,
                    *subject_span,
                    "lang identity for this static object",
                ));
            }
            let (kind, value_type) = if self.source.slice(*operation) == "identity" {
                (
                    CompilerExpressionKind::Identity(CompilerIdentity {
                        kind: ObjectKind::Type,
                        canonical: format!("type:{identity}"),
                    }),
                    CompilerType::Identity,
                )
            } else {
                (
                    CompilerExpressionKind::TypeView(CompilerTypeView {
                        form: CompilerTypeViewForm::Primitive,
                        identity: identity.into(),
                    }),
                    CompilerType::TypeView,
                )
            };
            return Ok(Some(CompilerExpression {
                kind,
                value_type,
                int_range: None,
                rational_value: None,
                span,
            }));
        }

        Ok(None)
    }

    fn analyze_lint_namespace(
        &self,
        operation: Span,
        span: Span,
    ) -> Result<CompilerExpression, Diagnostic> {
        if !self
            .language_features
            .iter()
            .any(|feature| feature == "lint")
        {
            return Err(source_diagnostic(
                &self.source,
                "E-LINT-VARIANT",
                operation,
                "the `lang lint` namespace requires the `lint` language feature",
            ));
        }
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::LintNamespace,
            value_type: CompilerType::Scope,
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_static_introspection_relation(
        &self,
        items: &[Expression],
        span: Span,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        if let [
            left,
            Expression::Identifier(namespace),
            Expression::Identifier(operation),
            right,
        ] = items
            && self.source.slice(*namespace) == "lang"
            && matches!(
                self.source.slice(*operation),
                "same-object" | "equivalent-type"
            )
        {
            let (Expression::Identifier(left), Expression::Identifier(right)) = (left, right)
            else {
                return Err(unsupported(
                    &self.source,
                    span,
                    "static introspection relation over runtime values",
                ));
            };
            let left = self.source.slice(*left);
            let right = self.source.slice(*right);
            if fundamental_type_value(left).is_none() || fundamental_type_value(right).is_none() {
                return Err(unsupported(
                    &self.source,
                    span,
                    "static introspection relation for these object kinds",
                ));
            }
            return Ok(Some(CompilerExpression {
                kind: CompilerExpressionKind::Boolean(left == right),
                value_type: CompilerType::Boolean,
                int_range: None,
                rational_value: None,
                span,
            }));
        }

        Ok(None)
    }

    fn analyze_function_view(
        &self,
        items: &[Expression],
        span: Span,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        let [
            Expression::Identifier(namespace),
            Expression::Identifier(operation),
            subject,
        ] = items
        else {
            return Ok(None);
        };
        if self.source.slice(*namespace) != "lang" || self.source.slice(*operation) != "view" {
            return Ok(None);
        }
        let Expression::Identifier(name_span) = subject else {
            return Err(unsupported(
                &self.source,
                subject.span(),
                "lang view for this static object",
            ));
        };
        let name = self.source.slice(*name_span);
        let Some(declarations) = self.functions.get(name) else {
            return Err(unsupported(
                &self.source,
                *name_span,
                "lang view for this static object",
            ));
        };
        let visible = declarations
            .iter()
            .filter(|declaration| declaration.span.end <= name_span.start)
            .collect::<Vec<_>>();
        let [declaration] = visible.as_slice() else {
            return Err(unsupported(
                &self.source,
                *name_span,
                "zero- or multi-overload Function view",
            ));
        };
        if declaration.declared_effects.is_none() {
            return Err(unsupported(
                &self.source,
                declaration.effect_bound.unwrap_or(declaration.span),
                "Function view without an admitted explicit empty effect bound",
            ));
        }
        Ok(Some(CompilerExpression {
            kind: CompilerExpressionKind::FunctionView(CompilerFunctionView {
                identity: format!("root.{name}"),
                inputs: declaration
                    .parameters
                    .iter()
                    .map(|parameter| compact_classifier(self.source.slice(parameter.classifier)))
                    .collect(),
                output: compact_classifier(self.source.slice(declaration.result)),
                is_static: declaration.is_static,
                declared_effects: declaration.declared_effects.clone(),
            }),
            value_type: CompilerType::FunctionView,
            int_range: None,
            rational_value: None,
            span,
        }))
    }

    #[allow(clippy::too_many_lines)] // Keeps constraint proof and rejection paths together.
    fn analyze_constraint_application(
        &mut self,
        tag: u32,
        operand: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let constraint =
            self.constraints[usize::try_from(tag).expect("u32 tag fits usize")].clone();
        let mut value = self.analyze_expression(operand, environment)?;
        if matches!(value.value_type, CompilerType::Refined { .. }) {
            value = forget_refined_evidence(value);
        }
        require_same_type(
            &self.source,
            operand.span(),
            &constraint.base_type,
            &value.value_type,
        )?;
        let exact_numeric_value = (constraint.name == "PositiveSize")
            .then_some(())
            .and(value.int_range.as_ref())
            .and_then(|range| {
                (range.lower == range.upper).then(|| CompilerExpression {
                    kind: CompilerExpressionKind::Int(range.lower.clone()),
                    value_type: value.value_type.clone(),
                    int_range: Some(range.clone()),
                    rational_value: None,
                    span: value.span,
                })
            })
            .or_else(|| {
                (constraint.name == "Probability")
                    .then(|| value.rational_value.clone())
                    .flatten()
                    .map(|rational| CompilerExpression {
                        kind: CompilerExpressionKind::Rational(rational.clone()),
                        value_type: CompilerType::Rational,
                        int_range: None,
                        rational_value: Some(rational),
                        span: value.span,
                    })
            })
            .or_else(|| {
                let CompilerExpressionKind::Binary {
                    operation: CompilerBinary::Equal,
                    left,
                    right,
                } = &value.kind
                else {
                    return None;
                };
                (constraint.name == "EqualLengths")
                    .then(|| exact_int(left).zip(exact_int(right)))
                    .flatten()
                    .map(|(left, right)| CompilerExpression {
                        kind: CompilerExpressionKind::Boolean(left == right),
                        value_type: CompilerType::Boolean,
                        int_range: None,
                        rational_value: None,
                        span: value.span,
                    })
            })
            .or_else(|| {
                (constraint.name == "NonemptyPattern")
                    .then(|| Self::known_string_value(&value, environment))
                    .flatten()
                    .map(|text| CompilerExpression {
                        kind: CompilerExpressionKind::String(text),
                        value_type: CompilerType::String,
                        int_range: None,
                        rational_value: None,
                        span: value.span,
                    })
            });
        if compiler_expression_is_closed(&value) || exact_numeric_value.is_some() {
            let accepted = known_constraint_predicate(
                &constraint.predicate,
                &constraint.parameter_storage,
                exact_numeric_value.as_ref().unwrap_or(&value),
            );
            if accepted.is_none() && constraint.name == "Pass" {
                return Ok(Self::finish_validation(
                    CompilerValidation::Constraint(tag),
                    value,
                    constraint.base_type,
                    span,
                    operand.span(),
                ));
            }
            let accepted = accepted.ok_or_else(|| {
                unsupported(
                    &self.source,
                    constraint.predicate.span,
                    "closed constraint predicate evaluation",
                )
            })?;
            if !accepted {
                return Err(source_diagnostic(
                    &self.source,
                    "E-CONSTRAINT-REJECTED",
                    operand.span(),
                    format!("value does not satisfy constraint `{}`", constraint.name),
                ));
            }
            value.value_type = CompilerType::Refined {
                constraint: constraint.name,
                base: Box::new(constraint.base_type),
            };
            value.span = span;
            return Ok(value);
        }
        Ok(Self::finish_validation(
            CompilerValidation::Constraint(tag),
            value,
            constraint.base_type,
            span,
            operand.span(),
        ))
    }

    fn analyze_sum_construction(
        &mut self,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        let selected = if let [Expression::Identifier(constructor), payload] = items {
            let Some((sum, value, declaration)) = self
                .sum_alternatives
                .get(self.source.slice(*constructor))
                .cloned()
            else {
                return Ok(None);
            };
            if declaration.end > constructor.start {
                return Ok(None);
            }
            let expected = sum.alternatives
                [usize::try_from(value).expect("u32 sum tag fits usize")]
            .payload
            .clone();
            let Some(expected) = expected else {
                return Ok(None);
            };
            (sum, value, expected, payload)
        } else if let [
            Expression::Identifier(type_name),
            Expression::Identifier(at),
            Expression::Integer(index),
            payload,
        ] = items
            && self.source.slice(*at) == "at"
            && let Some((sum, declaration)) = self.sums.get(self.source.slice(*type_name)).cloned()
            && sum.positional
            && declaration.end <= type_name.start
        {
            let value = parse_integer(self.source.slice(*index))
                .and_then(|value| value.to_string().parse::<usize>().ok())
                .filter(|value| *value < sum.alternatives.len())
                .ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-VARIANT-INDEX",
                        *index,
                        "Variant alternative index is outside its declared bounds",
                    )
                })?;
            let expected = sum.alternatives[value]
                .payload
                .clone()
                .expect("positional Variant alternatives carry payloads");
            (
                sum,
                u32::try_from(value).expect("validated native sum tag"),
                expected,
                payload,
            )
        } else {
            return Ok(None);
        };
        let (sum, value, expected, payload) = selected;
        let payload_value =
            self.analyze_expression_with_expected(payload, environment, Some(&expected))?;
        let payload_value = adapt_call_argument(&expected, &payload_value).ok_or_else(|| {
            source_diagnostic(
                &self.source,
                if sum.positional {
                    "E-VARIANT-PAYLOAD-CLASSIFIER"
                } else {
                    "E-UNION-PAYLOAD-CLASSIFIER"
                },
                payload.span(),
                format!(
                    "sum alternative `{}` requires {}, found {}",
                    sum.alternatives[usize::try_from(value).expect("u32 sum tag fits usize")].name,
                    expected.name(),
                    payload_value.value_type.name()
                ),
            )
        })?;
        Ok(Some(CompilerExpression {
            kind: CompilerExpressionKind::Sum {
                value,
                payload: Some(Box::new(payload_value)),
            },
            value_type: CompilerType::Sum(sum),
            int_range: None,
            rational_value: None,
            span,
        }))
    }

    fn analyze_modular_construction(
        &mut self,
        modular: CompilerModularType,
        operand: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let operand = self.analyze_expression(operand, environment)?;
        require_type(
            &self.source,
            operand.span,
            &CompilerType::Int,
            &operand.value_type,
        )?;
        if operand
            .int_range
            .as_ref()
            .is_some_and(|range| range.lower >= modular.lower && range.upper <= modular.upper)
        {
            let int_range = operand.int_range.clone();
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::IntToModular {
                    value: Box::new(operand),
                    modular: modular.clone(),
                },
                value_type: CompilerType::Modular(modular),
                int_range,
                rational_value: None,
                span,
            });
        }
        if operand
            .int_range
            .as_ref()
            .is_some_and(|range| range.upper < modular.lower || range.lower > modular.upper)
            && compiler_expression_is_closed(&operand)
        {
            return Err(source_diagnostic(
                &self.source,
                "E-MODULAR-OUT-OF-RANGE",
                operand.span,
                format!("value is outside `{}` canonical range", modular.name),
            ));
        }
        let error_span = operand.span;
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::ModularValidate {
                value: Box::new(operand),
                modular: modular.clone(),
                error_span,
            },
            value_type: CompilerType::Result(Box::new(CompilerType::Modular(modular))),
            int_range: None,
            rational_value: None,
            span,
        })
    }

    fn analyze_modular_reduction(
        &mut self,
        modular: CompilerModularType,
        operand: &Expression,
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        let operand = self.analyze_expression(operand, environment)?;
        require_type(
            &self.source,
            operand.span,
            &CompilerType::Int,
            &operand.value_type,
        )?;
        let int_range = exact_int(&operand)
            .map(|value| IntRange::exact(reduce_modular(value, &modular)))
            .or_else(|| Some(modular_range(&modular)));
        Ok(CompilerExpression {
            kind: CompilerExpressionKind::ModularReduce {
                value: Box::new(operand),
                modular: modular.clone(),
            },
            value_type: CompilerType::Modular(modular),
            int_range,
            rational_value: None,
            span,
        })
    }

    fn is_lang_operation(&self, expression: &Expression, expected: &str) -> bool {
        matches!(
            expression,
            Expression::Application { items, .. }
                if matches!(items.as_slice(),
                    [Expression::Identifier(lang), Expression::Identifier(operation)]
                        if self.source.slice(*lang) == "lang"
                            && self.source.slice(*operation) == expected)
        )
    }

    fn analyze_native_serialization(
        &mut self,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        if let [
            Expression::Identifier(lang),
            Expression::Identifier(version),
            operation,
        ] = items
            && self.source.slice(*lang) == "lang"
            && self.source.slice(*version) == "version"
            && self.is_lang_operation(operation, "serialize")
        {
            return Ok(Some(CompilerExpression {
                kind: CompilerExpressionKind::NativeSerializer(self.language_version),
                value_type: CompilerType::NativeSerializer(self.language_version),
                int_range: None,
                rational_value: None,
                span,
            }));
        }

        if let [
            Expression::Identifier(lang),
            Expression::Identifier(operation),
            stream,
        ] = items
            && self.source.slice(*lang) == "lang"
            && self.source.slice(*operation) == "deserialize"
        {
            let stream = self.analyze_expression(stream, environment)?;
            let CompilerType::SerializationStream(payload) = &stream.value_type else {
                return Err(source_diagnostic(
                    &self.source,
                    "E-DESERIALIZE-OPERAND",
                    stream.span,
                    "lang deserialize requires a native SerializationStream",
                ));
            };
            let payload = payload.as_ref().clone();
            return Ok(Some(CompilerExpression {
                kind: CompilerExpressionKind::Deserialize(Box::new(stream)),
                value_type: payload,
                int_range: None,
                rational_value: None,
                span,
            }));
        }

        let (version, subject) = match items {
            [version, operation, subject] if self.is_lang_operation(operation, "serialize") => {
                let parsed_version = match version {
                    Expression::Identifier(identifier) => self
                        .source
                        .slice(*identifier)
                        .parse::<LanguageVersion>()
                        .ok(),
                    _ => None,
                };
                let version = if let Some(version) = parsed_version {
                    version
                } else {
                    let version = self.analyze_expression(version, environment)?;
                    let CompilerExpressionKind::Version(version) = version.kind else {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-SERIALIZATION-VERSION",
                            version.span,
                            "the left operand of lang serialize must be a statically known Version",
                        ));
                    };
                    version
                };
                (version, subject)
            }
            [Expression::Identifier(name), subject] => {
                let Some(CompilerType::NativeSerializer(version)) = environment
                    .get(self.source.slice(*name))
                    .map(|facts| &facts.value_type)
                else {
                    return Ok(None);
                };
                (*version, subject)
            }
            _ => return Ok(None),
        };
        let value = self.analyze_expression(subject, environment)?;
        let bytes = self.native_serialization_bytes(version, &value, environment)?;
        let value_type = value.value_type.clone();
        Ok(Some(CompilerExpression {
            kind: CompilerExpressionKind::Serialize {
                bytes,
                value: Box::new(value),
            },
            value_type: CompilerType::SerializationStream(Box::new(value_type)),
            int_range: None,
            rational_value: None,
            span,
        }))
    }

    #[allow(clippy::too_many_lines)] // Transaction admission and lowering stay visibly coupled.
    fn analyze_task_application(
        &mut self,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<Option<CompilerExpression>, Diagnostic> {
        if let [Expression::Identifier(definition_name), initial] = items
            && let Some(definition) = self
                .task_definitions
                .get(self.source.slice(*definition_name))
                .filter(|definition| definition.span.end <= definition_name.start)
                .cloned()
        {
            let mut initial = self.analyze_expression(initial, environment)?;
            if initial.value_type == CompilerType::Int {
                let initial_span = initial.span;
                initial = self.finish_nat_conversion(initial, span, initial_span)?;
            }
            require_type(
                &self.source,
                initial.span,
                definition.task.state_type.as_ref(),
                &initial.value_type,
            )?;
            return Ok(Some(CompilerExpression {
                kind: CompilerExpressionKind::TaskConstruct {
                    task: definition.task.clone(),
                    initial: Box::new(initial),
                },
                value_type: CompilerType::Task(Box::new(definition.task)),
                int_range: None,
                rational_value: None,
                span,
            }));
        }

        let [instance, Expression::Identifier(operation), payload] = items else {
            return Ok(None);
        };
        let Expression::Identifier(instance_name) = instance else {
            return Ok(None);
        };
        let Some(CompilerType::Task(task)) = environment
            .get(self.source.slice(*instance_name))
            .map(|facts| &facts.value_type)
        else {
            return Ok(None);
        };
        let task = task.as_ref().clone();
        let operation_name = self.source.slice(*operation).to_owned();
        if operation_name == "start" {
            return Err(source_diagnostic(
                &self.source,
                "E-TASK-START-PRIVATE",
                *operation,
                "start is a lifecycle handler and cannot receive a message",
            ));
        }
        let handler = task
            .handlers
            .iter()
            .find(|handler| handler.name == operation_name)
            .ok_or_else(|| {
                source_diagnostic(
                    &self.source,
                    "E-TASK-HANDLER",
                    *operation,
                    "task capability exposes no such message handler",
                )
            })?
            .clone();
        let message = CompilerTaskMessage {
            operation: operation_name.clone(),
            transaction_identity: self.next_task_transaction,
        };
        self.next_task_transaction = self
            .next_task_transaction
            .checked_add(1)
            .ok_or_else(|| unsupported(&self.source, span, "task transaction identity"))?;
        let task_value = self.analyze_expression(instance, environment)?;
        match handler.kind {
            CompilerTaskHandlerKind::Event => {
                let mut payload = self.analyze_expression(payload, environment)?;
                if payload.value_type == CompilerType::Int {
                    let payload_span = payload.span;
                    payload = self.finish_nat_conversion(payload, span, payload_span)?;
                }
                require_type(
                    &self.source,
                    payload.span,
                    &handler.payload_type,
                    &payload.value_type,
                )?;
                let state = CompilerExpression {
                    kind: CompilerExpressionKind::TaskStateLoad {
                        task: Box::new(task_value.clone()),
                        message: message.clone(),
                    },
                    value_type: CompilerType::Nat,
                    int_range: None,
                    rational_value: None,
                    span: *operation,
                };
                let next = CompilerExpression {
                    kind: CompilerExpressionKind::Binary {
                        operation: CompilerBinary::Add,
                        left: Box::new(state),
                        right: Box::new(payload),
                    },
                    value_type: CompilerType::Nat,
                    int_range: None,
                    rational_value: None,
                    span,
                };
                Ok(Some(CompilerExpression {
                    kind: CompilerExpressionKind::TaskStateReplace {
                        task: Box::new(task_value),
                        value: Box::new(next),
                        message,
                    },
                    value_type: CompilerType::Unit,
                    int_range: None,
                    rational_value: None,
                    span,
                }))
            }
            CompilerTaskHandlerKind::Request => {
                let payload = self.analyze_expression(payload, environment)?;
                require_type(
                    &self.source,
                    payload.span,
                    &handler.payload_type,
                    &payload.value_type,
                )?;
                if !matches!(payload.kind, CompilerExpressionKind::Unit) {
                    return Err(unsupported(
                        &self.source,
                        payload.span,
                        "effectful direct task request payload",
                    ));
                }
                Ok(Some(CompilerExpression {
                    kind: CompilerExpressionKind::TaskStateLoad {
                        task: Box::new(task_value),
                        message,
                    },
                    value_type: CompilerType::TaskResponse(Box::new(handler.response_type)),
                    int_range: None,
                    rational_value: None,
                    span,
                }))
            }
            CompilerTaskHandlerKind::Stream => {
                let payload = self.analyze_expression(payload, environment)?;
                require_type(
                    &self.source,
                    payload.span,
                    &handler.payload_type,
                    &payload.value_type,
                )?;
                if !matches!(payload.kind, CompilerExpressionKind::Unit) {
                    return Err(unsupported(
                        &self.source,
                        payload.span,
                        "effectful direct task stream payload",
                    ));
                }
                let stream = self
                    .task_definitions
                    .get(&task.definition)
                    .and_then(|definition| definition.handlers.get(&operation_name))
                    .cloned()
                    .expect("checked task metadata retains its stream source");
                let [
                    Statement::Expression(Expression::Application { items, .. }),
                    Statement::Expression(Expression::Unit(result_span)),
                ] = stream.body.as_slice()
                else {
                    unreachable!("checked task stream has one yield and one final Unit")
                };
                let [_, Expression::ContextIdentifier(state_span)] = items.as_slice() else {
                    unreachable!("checked task stream yields its private state")
                };
                let owner_name = self.source.slice(*instance_name).to_owned();
                let owner_type = CompilerType::Task(Box::new(task.clone()));
                let initial_parameter = CompilerParameter {
                    name: owner_name.clone(),
                    discarded: false,
                    source_visible: true,
                    value_type: owner_type.clone(),
                    int_range: None,
                    span: *instance_name,
                };
                let owner = CompilerExpression {
                    kind: CompilerExpressionKind::Local(owner_name),
                    value_type: owner_type,
                    int_range: None,
                    rational_value: None,
                    span: *state_span,
                };
                let yielded = CompilerExpression {
                    kind: CompilerExpressionKind::TaskStateLoad {
                        task: Box::new(owner),
                        message,
                    },
                    value_type: CompilerType::Nat,
                    int_range: None,
                    rational_value: None,
                    span: *state_span,
                };
                let result = CompilerExpression {
                    kind: CompilerExpressionKind::ResultSuccess(Box::new(unit_expression(
                        *result_span,
                    ))),
                    value_type: CompilerType::TaskResponse(Box::new(CompilerType::Unit)),
                    int_range: None,
                    rational_value: None,
                    span: *result_span,
                };
                let stream_type = handler
                    .stream_type
                    .expect("checked stream handler retains its directions");
                Ok(Some(CompilerExpression {
                    kind: CompilerExpressionKind::CustomValueGenerator {
                        declaration: operation_name,
                        declaration_namespace: task.definition.clone(),
                        declaration_span: stream.span,
                        initial_parameter: Box::new(initial_parameter),
                        initial: Box::new(task_value),
                        additional_initial_parameters: Vec::new(),
                        additional_initials: Vec::new(),
                        prefix: Box::new(CompilerBlock {
                            statements: Vec::new(),
                            result: unit_expression(stream.span),
                        }),
                        yields: vec![CompilerGeneratorYield::Value(Box::new(yielded))],
                        continuations: Vec::new(),
                        explicit_return: None,
                        result: Box::new(result),
                    },
                    value_type: CompilerType::Generator(stream_type),
                    int_range: None,
                    rational_value: None,
                    span,
                }))
            }
            CompilerTaskHandlerKind::Terminate => {
                Err(unsupported(&self.source, span, "task termination delivery"))
            }
            CompilerTaskHandlerKind::Start => unreachable!("start was rejected before dispatch"),
        }
    }
}
