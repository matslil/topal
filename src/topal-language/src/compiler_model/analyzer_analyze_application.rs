impl Analyzer {
    #[allow(clippy::too_many_lines)] // Root operations are admitted explicitly and in source-selection order.
    fn analyze_application(
        &mut self,
        items: &[Expression],
        span: Span,
        environment: &BTreeMap<String, BindingFacts>,
    ) -> Result<CompilerExpression, Diagnostic> {
        if items.len() >= 5
            && items.len() % 2 == 1
            && items[1..].chunks_exact(2).all(|pair| {
                matches!(&pair[0], Expression::Identifier(operation)
                    if matches!(self.source.slice(*operation), "append" | "concat"))
            })
        {
            let mut value = self.analyze_expression(&items[0], environment)?;
            for pair in items[1..].chunks_exact(2) {
                let Expression::Identifier(operation_span) = &pair[0] else {
                    unreachable!("checked chained List operation")
                };
                let operation = self.source.slice(*operation_span).to_owned();
                if matches!(
                    value.value_type,
                    CompilerType::String | CompilerType::Character
                ) {
                    if operation != "concat" {
                        return Err(unsupported(
                            &self.source,
                            *operation_span,
                            "String append operation",
                        ));
                    }
                    value.value_type = CompilerType::String;
                    let mut right = self.analyze_expression(&pair[1], environment)?;
                    if !matches!(
                        right.value_type,
                        CompilerType::String | CompilerType::Character
                    ) {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-TYPE-MISMATCH",
                            right.span,
                            format!(
                                "expected String or Character, found {}",
                                right.value_type.name()
                            ),
                        ));
                    }
                    right.value_type = CompilerType::String;
                    value = CompilerExpression {
                        kind: CompilerExpressionKind::StringConcat {
                            left: Box::new(value),
                            right: Box::new(right),
                        },
                        value_type: CompilerType::String,
                        int_range: None,
                        rational_value: None,
                        span,
                    };
                    continue;
                }
                let CompilerType::List(element) = &value.value_type else {
                    return Err(unsupported(
                        &self.source,
                        value.span,
                        "List concatenation subject",
                    ));
                };
                let supported = matches!(
                    element.as_ref(),
                    CompilerType::Int | CompilerType::Nat | CompilerType::Rational
                ) || element.as_ref() == &CompilerType::String
                    || compiler_integer_pair(element.as_ref())
                    || compiler_list_integer_pair(element.as_ref())
                    || compiler_int_string_pair(element.as_ref())
                    || compiler_int_list_pair(element.as_ref())
                    || compiler_boolean_string_pair(element.as_ref())
                    || compiler_int_int_boolean_pair(element.as_ref())
                    || compiler_int_int_string_int_tuple(element.as_ref())
                    || compiler_int_triple(element.as_ref())
                    || compiler_nested_int_list_element(element.as_ref())
                    || compiler_nested_string_list_element(element.as_ref());
                if !supported {
                    return Err(unsupported(
                        &self.source,
                        value.span,
                        "concatenation for this List element type",
                    ));
                }
                let value_type = value.value_type.clone();
                let right = if operation == "concat" {
                    self.analyze_expression_with_expected(&pair[1], environment, Some(&value_type))?
                } else {
                    self.analyze_expression_with_expected(
                        &pair[1],
                        environment,
                        Some(element.as_ref()),
                    )?
                };
                let kind = if operation == "concat" {
                    require_same_type(&self.source, right.span, &value_type, &right.value_type)?;
                    CompilerExpressionKind::ListConcat {
                        left: Box::new(value),
                        right: Box::new(right),
                    }
                } else {
                    let right = self.finish_required_list_entry_value(right, element)?;
                    CompilerExpressionKind::ListAppend {
                        list: Box::new(value),
                        value: Box::new(right),
                    }
                };
                value = CompilerExpression {
                    kind,
                    value_type,
                    int_range: None,
                    rational_value: None,
                    span,
                };
            }
            return Ok(value);
        }
        if let [Expression::Identifier(operation), operand] = items
            && matches!(
                self.source.slice(*operation),
                "character-count" | "entry-count"
            )
        {
            let operation = self.source.slice(*operation).to_owned();
            return self.analyze_static_character_count(&operation, operand, span, environment);
        }
        if let Some(value) = self.analyze_external_storage(items, span, environment)? {
            return Ok(value);
        }
        if let Some(value) = self.analyze_task_application(items, span, environment)? {
            return Ok(value);
        }
        if let Some(value) = self.analyze_native_serialization(items, span, environment)? {
            return Ok(value);
        }
        if let Some(value) = self.analyze_static_introspection(items, span)? {
            return Ok(value);
        }
        if let Some(view) = self.analyze_function_view(items, span)? {
            return Ok(view);
        }
        if let [Expression::Identifier(operation), character] = items
            && self.source.slice(*operation) == "ascii-decimal-digit"
        {
            let character = self.analyze_expression(character, environment)?;
            let character =
                self.finish_proven_character_conversion(character, environment, span)?;
            require_type(
                &self.source,
                character.span,
                &CompilerType::Character,
                &character.value_type,
            )?;
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::CharacterAsciiDecimalDigit(Box::new(character)),
                value_type: CompilerType::Optional(Box::new(CompilerType::Nat)),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(operation), text] = items
            && self.source.slice(*operation) == "unicode-scalar-characters"
        {
            let text = self.analyze_expression(text, environment)?;
            require_type(
                &self.source,
                text.span,
                &CompilerType::String,
                &text.value_type,
            )?;
            let list_type = CompilerType::List(Box::new(CompilerType::Character));
            if let Some(value) = Self::known_string_value(&text, environment) {
                let mut list = CompilerExpression {
                    kind: CompilerExpressionKind::ListEmpty,
                    value_type: list_type.clone(),
                    int_range: None,
                    rational_value: None,
                    span,
                };
                for character in scalar_characters(&value).into_iter().rev() {
                    let value = CompilerExpression {
                        kind: CompilerExpressionKind::String(character),
                        value_type: CompilerType::Character,
                        int_range: None,
                        rational_value: None,
                        span,
                    };
                    list = CompilerExpression {
                        kind: CompilerExpressionKind::ListEntry {
                            value: Box::new(value),
                            remaining: Box::new(list),
                        },
                        value_type: list_type.clone(),
                        int_range: None,
                        rational_value: None,
                        span,
                    };
                }
                return Ok(list);
            }
            let Some(candidates) = self
                .known_structural_value_facts(&text, environment)?
                .string_characters
            else {
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::StringDynamicScalarCharactersCollect(Box::new(
                        text,
                    )),
                    value_type: list_type,
                    int_range: None,
                    rational_value: None,
                    span,
                });
            };
            let candidates = candidates
                .into_iter()
                .flat_map(|candidate| scalar_characters(&candidate))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::StringProvenanceCharactersCollect {
                    text: Box::new(text),
                    characters: candidates,
                },
                value_type: list_type,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(operation), character] = items
            && self.source.slice(*operation) == "unicode-scalar-value"
        {
            let character = self.analyze_expression(character, environment)?;
            if !matches!(
                character.value_type,
                CompilerType::String | CompilerType::Character
            ) {
                require_type(
                    &self.source,
                    character.span,
                    &CompilerType::String,
                    &character.value_type,
                )?;
            }
            let scalar_value = |candidate: &str| {
                let mut scalars = candidate.chars();
                let first = scalars.next()?;
                scalars
                    .next()
                    .is_none()
                    .then_some(BigInt::from(u32::from(first)))
            };
            if let Some(value) = Self::known_string_value(&character, environment) {
                let value = scalar_value(&value).ok_or_else(|| {
                    unsupported(
                        &self.source,
                        character.span,
                        "Unicode scalar value for a non-scalar Character",
                    )
                })?;
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Int(value.clone()),
                    value_type: CompilerType::Int,
                    int_range: Some(IntRange::exact(value)),
                    rational_value: None,
                    span,
                });
            }
            let Some(candidates) = self
                .known_structural_value_facts(&character, environment)?
                .string_characters
                .filter(|candidates| !candidates.is_empty())
            else {
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::StringUnicodeScalarValue(Box::new(character)),
                    value_type: CompilerType::Int,
                    int_range: Some(IntRange {
                        lower: BigInt::from(0),
                        upper: BigInt::from(0x0010_ffff_u32),
                    }),
                    rational_value: None,
                    span,
                });
            };
            let candidates = candidates
                .into_iter()
                .map(|candidate| {
                    scalar_value(&candidate)
                        .map(|value| (candidate, value))
                        .ok_or_else(|| {
                            unsupported(
                                &self.source,
                                character.span,
                                "Unicode scalar value for a non-scalar Character provenance",
                            )
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let Some((fallback_character, fallback_value)) = candidates.last() else {
                return Err(unsupported(
                    &self.source,
                    character.span,
                    "Unicode scalar value with empty Character provenance",
                ));
            };
            let mut result = CompilerExpression {
                kind: CompilerExpressionKind::Int(fallback_value.clone()),
                value_type: CompilerType::Int,
                int_range: Some(IntRange::exact(fallback_value.clone())),
                rational_value: None,
                span,
            };
            for (candidate, value) in candidates
                .iter()
                .filter(|(candidate, _)| candidate != fallback_character)
                .rev()
            {
                let predicate = Self::finish_binary(
                    CompilerBinary::Equal,
                    character.clone(),
                    CompilerExpression {
                        kind: CompilerExpressionKind::String(candidate.clone()),
                        value_type: CompilerType::Character,
                        int_range: None,
                        rational_value: None,
                        span,
                    },
                    CompilerType::Boolean,
                    span,
                );
                let when_true = CompilerExpression {
                    kind: CompilerExpressionKind::Int(value.clone()),
                    value_type: CompilerType::Int,
                    int_range: Some(IntRange::exact(value.clone())),
                    rational_value: None,
                    span,
                };
                let int_range = IntRange::union(
                    when_true.int_range.as_ref().expect("exact scalar value"),
                    result.int_range.as_ref().expect("scalar decision range"),
                );
                result = CompilerExpression {
                    kind: CompilerExpressionKind::BooleanDecision {
                        subject: Box::new(predicate),
                        when_true: Box::new(when_true),
                        when_false: Box::new(result),
                    },
                    value_type: CompilerType::Int,
                    int_range: Some(int_range),
                    rational_value: None,
                    span,
                };
            }
            return Ok(result);
        }
        if let [Expression::Identifier(operation), character] = items
            && matches!(
                self.source.slice(*operation),
                "unicode-carriage-return-character"
                    | "unicode-line-feed-character"
                    | "unicode-whitespace-character"
                    | "unicode-decimal-digit-character"
                    | "unicode-word-character"
            )
        {
            let character = self.analyze_expression(character, environment)?;
            require_type(
                &self.source,
                character.span,
                &CompilerType::Character,
                &character.value_type,
            )?;
            let operation_name = self.source.slice(*operation);
            if matches!(
                operation_name,
                "unicode-carriage-return-character" | "unicode-line-feed-character"
            ) {
                let expected = if operation_name == "unicode-carriage-return-character" {
                    "\r"
                } else {
                    "\n"
                };
                return Ok(Self::finish_binary(
                    CompilerBinary::Equal,
                    character,
                    CompilerExpression {
                        kind: CompilerExpressionKind::String(expected.into()),
                        value_type: CompilerType::Character,
                        int_range: None,
                        rational_value: None,
                        span: *operation,
                    },
                    CompilerType::Boolean,
                    span,
                ));
            }
            let Some(candidates) = self
                .known_structural_value_facts(&character, environment)?
                .string_characters
            else {
                if operation_name == "unicode-whitespace-character" {
                    return Ok(CompilerExpression {
                        kind: CompilerExpressionKind::CharacterUnicodeWhitespace(Box::new(
                            character,
                        )),
                        value_type: CompilerType::Boolean,
                        int_range: None,
                        rational_value: None,
                        span,
                    });
                }
                return Err(unsupported(
                    &self.source,
                    character.span,
                    "Unicode Character predicate without finite Character provenance",
                ));
            };
            let mut matching = candidates
                .into_iter()
                .filter(|candidate| unicode_character_predicate(operation_name, candidate))
                .collect::<BTreeSet<_>>()
                .into_iter();
            let Some(first) = matching.next() else {
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::Boolean(false),
                    value_type: CompilerType::Boolean,
                    int_range: None,
                    rational_value: None,
                    span,
                });
            };
            let comparison = |expected: String| {
                Self::finish_binary(
                    CompilerBinary::Equal,
                    character.clone(),
                    CompilerExpression {
                        kind: CompilerExpressionKind::String(expected),
                        value_type: CompilerType::Character,
                        int_range: None,
                        rational_value: None,
                        span: *operation,
                    },
                    CompilerType::Boolean,
                    span,
                )
            };
            let mut result = comparison(first);
            for expected in matching {
                result = Self::finish_binary(
                    CompilerBinary::Or,
                    result,
                    comparison(expected),
                    CompilerType::Boolean,
                    span,
                );
            }
            return Ok(result);
        }
        if let Some(value) = self.analyze_library_application(items, span, environment)? {
            return Ok(value);
        }
        if let Some(sum) = self.analyze_sum_construction(items, span, environment)? {
            return Ok(sum);
        }
        if let [Expression::Identifier(operation), text] = items
            && self.source.slice(*operation) == "characters"
        {
            return self.analyze_closed_string_characters_generator(text, span, environment);
        }
        if let [
            Expression::Identifier(characters),
            text,
            Expression::Identifier(operation),
            Expression::Identifier(target),
        ] = items
            && self.source.slice(*characters) == "characters"
            && self.source.slice(*operation) == "collect"
            && self.source.slice(*target) == "String"
        {
            let generator = self.analyze_closed_string_characters_generator(
                text,
                Span::new(characters.start, text.span().end),
                environment,
            )?;
            let CompilerExpressionKind::StringCharactersGenerator { text, characters } =
                generator.kind
            else {
                return Err(unsupported(
                    &self.source,
                    span,
                    "direct dynamic String Character collection",
                ));
            };
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::StringCharactersCollect { text, characters },
                value_type: CompilerType::String,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(name), operand] = items
            && let Some((modular, declaration)) = self.modulars.get(self.source.slice(*name))
            && declaration.end <= name.start
        {
            return self.analyze_modular_construction(modular.clone(), operand, span, environment);
        }
        if let [
            operand,
            Expression::Identifier(operation),
            Expression::Identifier(name),
        ] = items
            && self.source.slice(*operation) == "modulo"
            && let Some((modular, declaration)) = self.modulars.get(self.source.slice(*name))
            && declaration.end <= name.start
        {
            return self.analyze_modular_reduction(modular.clone(), operand, span, environment);
        }
        if let [Expression::Identifier(keyword), selected] = items
            && self.source.slice(*keyword) == "use"
        {
            if self.in_function {
                return Err(unsupported(
                    &self.source,
                    span,
                    "function-body namespace use",
                ));
            }
            let selected = self.analyze_expression(selected, environment)?;
            if selected.value_type != CompilerType::Scope {
                return Err(source_diagnostic(
                    &self.source,
                    "E-USE-NON-NAMESPACE",
                    selected.span,
                    "use requires a published namespace path",
                ));
            }
            return Ok(selected);
        }
        if let Some((Expression::Identifier(namespace), remaining)) = items.split_first()
            && self.source.slice(*namespace) == "root"
            && let Some(Expression::Identifier(member)) = remaining.first()
        {
            let member_name = self.source.slice(*member).to_owned();
            if self.functions.contains_key(&member_name) {
                return self.analyze_resolved_call(remaining, span, environment, 0, &member_name);
            }
            if let Some(declarations) = self.generators.get(&member_name).cloned() {
                let visible = declarations
                    .into_iter()
                    .filter(|declaration| declaration.span.end <= span.start)
                    .collect::<Vec<_>>();
                if !visible.is_empty() {
                    return self.analyze_custom_generator_call_from(
                        remaining,
                        span,
                        environment,
                        0,
                        &member_name,
                        &visible,
                        "root",
                    );
                }
            }
            if remaining.len() == 1
                && let Some(facts) = self.root_bindings.get(&member_name)
            {
                if compiler_type_contains_generator(&facts.value_type) {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "qualified generator access",
                    ));
                }
                if self.in_function {
                    let parameter_name = format!("root {member_name}");
                    let facts = environment.get(&parameter_name).ok_or_else(|| {
                        unsupported(
                            &self.source,
                            *member,
                            "uncaptured function-body root data member",
                        )
                    })?;
                    return Ok(CompilerExpression {
                        kind: CompilerExpressionKind::Local(facts.storage_name.clone()),
                        value_type: facts.value_type.clone(),
                        int_range: facts.int_range.clone(),
                        rational_value: facts.rational_value.clone(),
                        span,
                    });
                }
                return Ok(data_member_expression(facts, span));
            }
            return Err(unsupported(&self.source, *member, "qualified root member"));
        }
        if let Some((Expression::Identifier(alias), remaining)) = items.split_first()
            && let Some(namespace) = environment
                .get(self.source.slice(*alias))
                .and_then(|facts| facts.namespace.as_ref())
            && let Some(Expression::Identifier(member)) = remaining.first()
        {
            let member_name = self.source.slice(*member).to_owned();
            if let Some(declarations) = namespace.functions.get(&member_name) {
                return self.analyze_resolved_call_from(
                    remaining,
                    span,
                    environment,
                    0,
                    &member_name,
                    declarations,
                    &[],
                    false,
                );
            }
            if let Some(declarations) = namespace.generators.get(&member_name) {
                let declarations = declarations.clone();
                let namespace_name = namespace.name.clone();
                return self.analyze_custom_generator_call_from(
                    remaining,
                    span,
                    environment,
                    0,
                    &member_name,
                    &declarations,
                    &namespace_name,
                );
            }
            if remaining.len() == 1
                && let Some(facts) = namespace.bindings.get(&member_name)
            {
                if compiler_type_contains_generator(&facts.value_type) {
                    return Err(unsupported(
                        &self.source,
                        span,
                        "qualified generator access",
                    ));
                }
                return Ok(data_member_expression(facts, span));
            }
            return Err(source_diagnostic(
                &self.source,
                "E-COMPILER-UNSUPPORTED",
                *member,
                format!(
                    "compiler increment cannot resolve member `{member_name}` from namespace `{}`",
                    namespace.name
                ),
            ));
        }
        if let Some(Expression::Identifier(alias)) = items.first()
            && let Some(callable) = environment
                .get(self.source.slice(*alias))
                .and_then(|facts| facts.callable.as_ref())
        {
            return self.analyze_callable_application(callable.clone(), items, span, environment);
        }
        if let [Expression::Identifier(name), operand] = items
            && let Some(tag) = self
                .constraint_bindings
                .get(self.source.slice(*name))
                .copied()
            && self
                .constraint_binding_declarations
                .get(self.source.slice(*name))
                .is_some_and(|end| *end <= name.start)
        {
            return self.analyze_constraint_application(tag, operand, span, environment);
        }
        if let [Expression::Identifier(name), operand] = items
            && self.source.slice(*name) == "DecimalText"
            && self
                .source
                .slice(Span::new(0, name.start))
                .contains("DecimalText is String constraint")
        {
            let mut value = self.analyze_expression(operand, environment)?;
            require_type(
                &self.source,
                value.span,
                &CompilerType::String,
                &value.value_type,
            )?;
            let Some(text) = Self::known_string_value(&value, environment) else {
                return Err(unsupported(
                    &self.source,
                    operand.span(),
                    "dynamic DecimalText constraint validation",
                ));
            };
            if !text.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(source_diagnostic(
                    &self.source,
                    "E-CONSTRAINT-PREDICATE",
                    operand.span(),
                    "DecimalText requires only ASCII decimal digits",
                ));
            }
            value.value_type = CompilerType::Refined {
                constraint: "DecimalText".into(),
                base: Box::new(CompilerType::String),
            };
            value.span = span;
            return Ok(value);
        }
        if items.len() > 1
            && items
                .iter()
                .all(|item| matches!(item, Expression::String(_)))
        {
            let mut value = String::new();
            for item in items {
                let Expression::String(literal) = item else {
                    unreachable!("checked adjacent String literals")
                };
                value.push_str(parse_string(self.source.slice(*literal)).ok_or_else(|| {
                    source_diagnostic(
                        &self.source,
                        "E-STRING-LITERAL",
                        *literal,
                        "invalid string literal delimiter",
                    )
                })?);
            }
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::String(value),
                value_type: CompilerType::String,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [
            Expression::Identifier(operation),
            Expression::Identifier(domain),
        ] = items
            && self.source.slice(*operation) == "empty"
            && self.source.slice(*domain) == "String"
        {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::StringEmpty,
                value_type: CompilerType::String,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(constructor), Expression::Unit(_)] = items
            && self.source.slice(*constructor) == "Effects"
        {
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::Effect,
                value_type: CompilerType::Effect,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(constructor), value] = items
            && matches!(self.source.slice(*constructor), "Continue" | "Finish")
        {
            let finish = self.source.slice(*constructor) == "Finish";
            let value = self.analyze_expression(value, environment)?;
            require_type(
                &self.source,
                value.span,
                &CompilerType::Int,
                &value.value_type,
            )?;
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::TraversalControl {
                    finish,
                    value: Box::new(value),
                },
                value_type: CompilerType::TraversalControl(Box::new(CompilerType::Int)),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if items.len() >= 3
            && items.len() % 2 == 1
            && items.iter().skip(1).step_by(2).all(
                |item| matches!(item, Expression::Identifier(operation) if self.source.slice(*operation) == "concat"),
            )
        {
            let mut result = self.analyze_expression(&items[0], environment)?;
            if matches!(result.value_type, CompilerType::String | CompilerType::Character) {
                result.value_type = CompilerType::String;
                for operand in items.iter().skip(2).step_by(2) {
                    let mut right = self.analyze_expression(operand, environment)?;
                    if !matches!(right.value_type, CompilerType::String | CompilerType::Character)
                    {
                        return Err(source_diagnostic(
                            &self.source,
                            "E-TYPE-MISMATCH",
                            right.span,
                            format!("expected String or Character, found {}", right.value_type.name()),
                        ));
                    }
                    right.value_type = CompilerType::String;
                    let expression_span = Span::new(result.span.start, right.span.end);
                    result = CompilerExpression {
                        kind: CompilerExpressionKind::StringConcat {
                            left: Box::new(result),
                            right: Box::new(right),
                        },
                        value_type: CompilerType::String,
                        int_range: None,
                        rational_value: None,
                        span: expression_span,
                    };
                }
                result.span = span;
                return Ok(result);
            }
        }
        if let [
            Expression::Identifier(empty),
            Expression::Identifier(list),
            element,
        ] = items
            && self.source.slice(*empty) == "empty"
            && self.source.slice(*list) == "List"
        {
            let element_type = self.parse_classifier(element.span())?;
            if element_type != CompilerType::Int {
                return Err(unsupported(
                    &self.source,
                    element.span(),
                    "explicit empty List element classifier",
                ));
            }
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListEmpty,
                value_type: CompilerType::List(Box::new(element_type)),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(empty), classifier] = items
            && self.source.slice(*empty) == "Empty"
        {
            let element_type = self.parse_classifier(classifier.span())?;
            if !compiler_list_node_element_supported(&element_type) {
                return Err(unsupported(
                    &self.source,
                    classifier.span(),
                    "explicit Empty element classifier",
                ));
            }
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListEmpty,
                value_type: CompilerType::List(Box::new(element_type)),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(constructor), value] = items
            && self.source.slice(*constructor) == "one"
            && !matches!(value, Expression::Identifier(domain) if matches!(self.source.slice(*domain), "Int" | "Nat" | "Rational"))
        {
            let value = self.analyze_expression(value, environment)?;
            if !compiler_list_node_element_supported(&value.value_type) {
                return Err(unsupported(
                    &self.source,
                    value.span,
                    "singleton List element classifier",
                ));
            }
            let element_type = value.value_type.clone();
            let list_type = CompilerType::List(Box::new(element_type));
            let empty = CompilerExpression {
                kind: CompilerExpressionKind::ListEmpty,
                value_type: list_type.clone(),
                int_range: None,
                rational_value: None,
                span,
            };
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListEntry {
                    value: Box::new(value),
                    remaining: Box::new(empty),
                },
                value_type: list_type,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(operation), pairs] = items
            && self.source.slice(*operation) == "unzip"
        {
            let pairs = self.analyze_expression(pairs, environment)?;
            let pair_type = CompilerType::Tuple(vec![CompilerType::Int, CompilerType::Int]);
            require_type(
                &self.source,
                pairs.span,
                &CompilerType::List(Box::new(pair_type)),
                &pairs.value_type,
            )?;
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListUnzip(Box::new(pairs)),
                value_type: CompilerType::Tuple(vec![int_list_type(), int_list_type()]),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(operation), list] = items
            && self.source.slice(*operation) == "collect"
            && matches!(list, Expression::Identifier(name)
                if environment.get(self.source.slice(*name)).is_some_and(|facts|
                    matches!(facts.value_type, CompilerType::List(_))))
        {
            let mut list = self.analyze_expression(list, environment)?;
            debug_assert!(matches!(list.value_type, CompilerType::List(_)));
            list.span = span;
            return Ok(list);
        }
        if let [
            list,
            Expression::Identifier(operation),
            Expression::Identifier(classifier),
        ] = items
            && self.source.slice(*operation) == "collect"
            && self.source.slice(*classifier) == "Array"
        {
            let list = self.analyze_expression(list, environment)?;
            let CompilerType::List(element) = &list.value_type else {
                return Err(unsupported(
                    &self.source,
                    list.span,
                    "Array collection source outside List",
                ));
            };
            let element = element.as_ref().clone();
            if !matches!(&element, CompilerType::Int | CompilerType::Function) {
                return Err(unsupported(
                    &self.source,
                    list.span,
                    "Array collection source element classifier",
                ));
            }
            let count = Self::known_list_count(&list, environment).ok_or_else(|| {
                unsupported(&self.source, list.span, "Array with a dynamic entry count")
            })?;
            if element == CompilerType::Function {
                let entries = self
                    .known_structural_value_facts(&list, environment)?
                    .list_entries
                    .ok_or_else(|| {
                        unsupported(
                            &self.source,
                            list.span,
                            "Array Function source without exact finite entry facts",
                        )
                    })?;
                if entries.len() != count {
                    return Err(unsupported(
                        &self.source,
                        list.span,
                        "Array Function source with inconsistent exact entry facts",
                    ));
                }
            }
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ContainerCollect {
                    source: Box::new(list),
                    kind: CompilerContainerKind::Array,
                    map_policy: None,
                    map_keys: None,
                },
                value_type: CompilerType::Array {
                    count,
                    element: Box::new(element),
                },
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(operation), list] = items
            && matches!(self.source.slice(*operation), "collect-set" | "collect-bag")
        {
            let operation = self.source.slice(*operation).to_owned();
            let list = self.analyze_expression(list, environment)?;
            require_int_list(&self.source, &list, "unordered collection source")?;
            let (kind, value_type) = if operation == "collect-set" {
                (
                    CompilerContainerKind::Set,
                    CompilerType::Set(Box::new(CompilerType::Int)),
                )
            } else {
                (
                    CompilerContainerKind::Bag,
                    CompilerType::Bag(Box::new(CompilerType::Int)),
                )
            };
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ContainerCollect {
                    source: Box::new(list),
                    kind,
                    map_policy: None,
                    map_keys: None,
                },
                value_type,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [
            Expression::Identifier(operation),
            pairs,
            Expression::Identifier(resolving),
            Expression::Identifier(policy),
        ] = items
            && self.source.slice(*operation) == "collect-map"
            && self.source.slice(*resolving) == "resolving"
        {
            let policy = match self.source.slice(*policy) {
                "reject" => CompilerMapCollisionPolicy::Reject,
                "keep-first" => CompilerMapCollisionPolicy::KeepFirst,
                "keep-last" => CompilerMapCollisionPolicy::KeepLast,
                _ => {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-MAP-COLLISION-POLICY",
                        *policy,
                        "collect-map policy must be reject, keep-first, or keep-last",
                    ));
                }
            };
            let pairs = self.analyze_expression(pairs, environment)?;
            let CompilerType::List(element) = &pairs.value_type else {
                return Err(unsupported(
                    &self.source,
                    pairs.span,
                    "Map collection source outside List",
                ));
            };
            let CompilerType::Tuple(fields) = element.as_ref() else {
                return Err(unsupported(
                    &self.source,
                    pairs.span,
                    "Map collection source outside key/value products",
                ));
            };
            let [
                CompilerType::String,
                map_value @ (CompilerType::Int | CompilerType::Function),
            ] = fields.as_slice()
            else {
                return Err(unsupported(
                    &self.source,
                    pairs.span,
                    "Map collection key/value classifiers",
                ));
            };
            let map_value = map_value.clone();
            let map_keys = Self::known_list_string_keys(&pairs, environment);
            if map_value == CompilerType::Function && map_keys.is_none() {
                return Err(unsupported(
                    &self.source,
                    pairs.span,
                    "Map Function source without exact String keys",
                ));
            }
            if map_value == CompilerType::Function && map_keys.as_ref().is_some_and(Vec::is_empty) {
                return Err(unsupported(
                    &self.source,
                    pairs.span,
                    "empty Map Function collection before typed empty Map parity",
                ));
            }
            if policy == CompilerMapCollisionPolicy::Reject {
                let keys = map_keys.as_ref().ok_or_else(|| {
                    unsupported(&self.source, pairs.span, "dynamic reject-policy Map keys")
                })?;
                let mut distinct = BTreeSet::new();
                if keys.iter().any(|key| !distinct.insert(key.clone())) {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-MAP-KEY-COLLISION",
                        pairs.span,
                        "collect-map encountered a duplicate key under reject policy",
                    ));
                }
            }
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ContainerCollect {
                    source: Box::new(pairs),
                    kind: CompilerContainerKind::Map,
                    map_policy: Some(policy),
                    map_keys,
                },
                value_type: CompilerType::Map {
                    key: Box::new(CompilerType::String),
                    value: Box::new(map_value),
                },
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [list, Expression::Identifier(operation)] = items
            && self.source.slice(*operation) == "entries"
        {
            let list = self.analyze_expression(list, environment)?;
            require_int_list(&self.source, &list, "entries source")?;
            let entry = CompilerType::Record(vec![
                ("index".into(), CompilerType::Int),
                ("value".into(), CompilerType::Int),
            ]);
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListEntries(Box::new(list)),
                value_type: CompilerType::List(Box::new(entry)),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [
            list,
            Expression::Identifier(operation),
            Expression::Identifier(classifier),
        ] = items
            && self.source.slice(*operation) == "collect"
            && self.source.slice(*classifier) == "String"
        {
            let list = self.analyze_expression(list, environment)?;
            require_type(
                &self.source,
                list.span,
                &CompilerType::List(Box::new(CompilerType::String)),
                &list.value_type,
            )?;
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListCollectString(Box::new(list)),
                value_type: CompilerType::String,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [list, Expression::Identifier(operation), boundary, inserted] = items
            && self.source.slice(*operation) == "insert-at"
        {
            let list = self.analyze_expression(list, environment)?;
            require_int_list(&self.source, &list, "insert-at source")?;
            let count = Self::known_list_count(&list, environment).ok_or_else(|| {
                unsupported(&self.source, list.span, "dynamic List insert-at boundary")
            })?;
            let boundary = self.analyze_expression(boundary, environment)?;
            require_type(
                &self.source,
                boundary.span,
                &CompilerType::Int,
                &boundary.value_type,
            )?;
            let boundary_value = Self::exact_usize(&boundary).ok_or_else(|| {
                unsupported(
                    &self.source,
                    boundary.span,
                    "dynamic or negative List insert-at boundary",
                )
            })?;
            if boundary_value > count {
                return Err(source_diagnostic(
                    &self.source,
                    "E-LIST-BOUNDARY-OUT-OF-RANGE",
                    boundary.span,
                    "insert-at operand is outside the List's valid bounds",
                ));
            }
            let inserted = self.analyze_expression(inserted, environment)?;
            let inserts_list = inserted.value_type == int_list_type();
            if !inserts_list {
                require_type(
                    &self.source,
                    inserted.span,
                    &CompilerType::Int,
                    &inserted.value_type,
                )?;
            }
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListInsertAt {
                    list: Box::new(list),
                    boundary: boundary_value,
                    inserted: Box::new(inserted),
                    inserts_list,
                },
                value_type: int_list_type(),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [
            list,
            Expression::Identifier(operation),
            Expression::AnonymousFunction {
                parameters,
                body,
                span: function_span,
            },
        ] = items
            && matches!(
                self.source.slice(*operation),
                "remove-indexes" | "remove-values"
            )
        {
            let indexes = self.source.slice(*operation) == "remove-indexes";
            let list = self.analyze_expression(list, environment)?;
            require_int_list(&self.source, &list, "predicate removal source")?;
            let (parameters, predicate) = self.analyze_collection_function(
                parameters,
                body,
                &[CompilerType::Int],
                environment,
                self.static_context,
                *function_span,
            )?;
            require_type(
                &self.source,
                predicate.result.span,
                &CompilerType::Boolean,
                &predicate.result.value_type,
            )?;
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListReject {
                    list: Box::new(list),
                    parameters,
                    predicate: Box::new(predicate),
                    indexes,
                },
                value_type: int_list_type(),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [list, Expression::Identifier(operation), operand] = items
            && matches!(
                self.source.slice(*operation),
                "split-at" | "take" | "drop" | "remove" | "remove-indexes"
            )
        {
            let operation_name = self.source.slice(*operation).to_owned();
            let list = self.analyze_expression(list, environment)?;
            require_int_list(&self.source, &list, "indexed List operation source")?;
            let count = Self::known_list_count(&list, environment).ok_or_else(|| {
                unsupported(
                    &self.source,
                    list.span,
                    "dynamic checked List index or boundary",
                )
            })?;
            let operand = self.analyze_expression(operand, environment)?;
            if operation_name == "remove-indexes"
                && operand.value_type == CompilerType::Range(Box::new(CompilerType::Int))
            {
                let range =
                    Self::known_closed_int_range(&operand, environment).ok_or_else(|| {
                        unsupported(&self.source, operand.span, "dynamic List index range")
                    })?;
                let lower = usize::try_from(&range.lower).map_err(|_| {
                    source_diagnostic(
                        &self.source,
                        "E-LIST-BOUNDARY-OUT-OF-RANGE",
                        operand.span,
                        "remove-indexes operand is outside the List's valid bounds",
                    )
                })?;
                let upper = usize::try_from(&range.upper).map_err(|_| {
                    source_diagnostic(
                        &self.source,
                        "E-LIST-BOUNDARY-OUT-OF-RANGE",
                        operand.span,
                        "remove-indexes operand is outside the List's valid bounds",
                    )
                })?;
                let start = lower
                    .checked_add(usize::from(!range.lower_inclusive))
                    .ok_or_else(|| {
                        source_diagnostic(
                            &self.source,
                            "E-LIST-BOUNDARY-OUT-OF-RANGE",
                            operand.span,
                            "remove-indexes operand is outside the List's valid bounds",
                        )
                    })?;
                let end = upper
                    .checked_add(usize::from(range.upper_inclusive))
                    .ok_or_else(|| {
                        source_diagnostic(
                            &self.source,
                            "E-LIST-BOUNDARY-OUT-OF-RANGE",
                            operand.span,
                            "remove-indexes operand is outside the List's valid bounds",
                        )
                    })?;
                if start > end || end > count {
                    return Err(source_diagnostic(
                        &self.source,
                        "E-LIST-BOUNDARY-OUT-OF-RANGE",
                        operand.span,
                        "remove-indexes operand is outside the List's valid bounds",
                    ));
                }
                return Ok(CompilerExpression {
                    kind: CompilerExpressionKind::ListRemoveIndexRange {
                        list: Box::new(list),
                        start,
                        end,
                    },
                    value_type: int_list_type(),
                    int_range: None,
                    rational_value: None,
                    span,
                });
            }
            require_type(
                &self.source,
                operand.span,
                &CompilerType::Int,
                &operand.value_type,
            )?;
            let index = Self::exact_usize(&operand).ok_or_else(|| {
                unsupported(
                    &self.source,
                    operand.span,
                    "dynamic or negative List index or boundary",
                )
            })?;
            let operation = match operation_name.as_str() {
                "split-at" => CompilerListIndexOperation::Split,
                "take" => CompilerListIndexOperation::Take,
                "drop" => CompilerListIndexOperation::Drop,
                "remove" => CompilerListIndexOperation::Remove,
                _ => unreachable!("remove-indexes Range handled above"),
            };
            let valid = if operation == CompilerListIndexOperation::Remove {
                index < count
            } else {
                index <= count
            };
            if !valid {
                return Err(source_diagnostic(
                    &self.source,
                    "E-LIST-BOUNDARY-OUT-OF-RANGE",
                    operand.span,
                    format!("{operation_name} operand is outside the List's valid bounds"),
                ));
            }
            let value_type = if operation == CompilerListIndexOperation::Split {
                CompilerType::Tuple(vec![int_list_type(), int_list_type()])
            } else {
                int_list_type()
            };
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListIndexOperation {
                    list: Box::new(list),
                    index,
                    operation,
                },
                value_type,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [left, Expression::Identifier(operation), right] = items
            && matches!(self.source.slice(*operation), "zip-exact" | "zip-shortest")
        {
            let exact = self.source.slice(*operation) == "zip-exact";
            let left = self.analyze_expression(left, environment)?;
            let right = self.analyze_expression(right, environment)?;
            let CompilerType::List(left_element) = &left.value_type else {
                return Err(unsupported(&self.source, left.span, "zip left source"));
            };
            let CompilerType::List(right_element) = &right.value_type else {
                return Err(unsupported(&self.source, right.span, "zip right source"));
            };
            require_same_type(
                &self.source,
                right.span,
                left_element.as_ref(),
                right_element.as_ref(),
            )?;
            if !matches!(
                left_element.as_ref(),
                CompilerType::Int | CompilerType::Rational
            ) {
                return Err(unsupported(
                    &self.source,
                    left.span,
                    "zip source for this List element type",
                ));
            }
            let pair = CompilerType::Tuple(vec![
                left_element.as_ref().clone(),
                right_element.as_ref().clone(),
            ]);
            let list_type = CompilerType::List(Box::new(pair));
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListZip {
                    left: Box::new(left),
                    right: Box::new(right),
                    operation: if exact {
                        CompilerListZipOperation::Exact
                    } else {
                        CompilerListZipOperation::Shortest
                    },
                    left_default: None,
                    right_default: None,
                },
                value_type: if exact {
                    CompilerType::Result(Box::new(list_type))
                } else {
                    list_type
                },
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [left, Expression::Identifier(operation), right] = items
            && self.source.slice(*operation) == "zip-longest"
        {
            let left = self.analyze_expression(left, environment)?;
            let right = self.analyze_expression(right, environment)?;
            let CompilerExpressionKind::Tuple(mut left_fields) = left.kind else {
                return Err(unsupported(
                    &self.source,
                    left.span,
                    "zip-longest left operand",
                ));
            };
            let CompilerExpressionKind::Tuple(mut right_fields) = right.kind else {
                return Err(unsupported(
                    &self.source,
                    right.span,
                    "zip-longest right operand",
                ));
            };
            if left_fields.len() != 2 || right_fields.len() != 2 {
                return Err(unsupported(&self.source, span, "zip-longest operands"));
            }
            let left_default = left_fields.pop().expect("checked two fields");
            let left = left_fields.pop().expect("checked two fields");
            let right_default = right_fields.pop().expect("checked two fields");
            let right = right_fields.pop().expect("checked two fields");
            require_int_list(&self.source, &left, "zip-longest left source")?;
            require_int_list(&self.source, &right, "zip-longest right source")?;
            require_type(
                &self.source,
                left_default.span,
                &CompilerType::Int,
                &left_default.value_type,
            )?;
            require_type(
                &self.source,
                right_default.span,
                &CompilerType::Int,
                &right_default.value_type,
            )?;
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListZip {
                    left: Box::new(left),
                    right: Box::new(right),
                    operation: CompilerListZipOperation::Longest,
                    left_default: Some(Box::new(left_default)),
                    right_default: Some(Box::new(right_default)),
                },
                value_type: CompilerType::List(Box::new(CompilerType::Tuple(vec![
                    CompilerType::Int,
                    CompilerType::Int,
                ]))),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [
            operand,
            Expression::Identifier(reverse),
            Expression::Callable { kind, .. },
            right,
        ] = items
            && self.source.slice(*reverse) == "reverse"
            && matches!(kind, CallableKind::Equal | CallableKind::NotEqual)
        {
            let operand = self.analyze_expression(operand, environment)?;
            let CompilerType::List(element) = &operand.value_type else {
                return Err(unsupported(
                    &self.source,
                    operand.span,
                    "reverse for this value type",
                ));
            };
            if !matches!(
                element.as_ref(),
                CompilerType::Int | CompilerType::Character
            ) && !compiler_nested_int_list_element(element.as_ref())
                && !compiler_int_string_pair(element.as_ref())
                && !compiler_int_list_pair(element.as_ref())
            {
                return Err(unsupported(
                    &self.source,
                    operand.span,
                    "reverse for this List element type",
                ));
            }
            let list_type = operand.value_type.clone();
            let reversed = CompilerExpression {
                kind: CompilerExpressionKind::ListReverse(Box::new(operand)),
                value_type: list_type.clone(),
                int_range: None,
                rational_value: None,
                span: Span::new(items[0].span().start, items[1].span().end),
            };
            let right = self.analyze_expression(right, environment)?;
            require_same_type(&self.source, right.span, &list_type, &right.value_type)?;
            return Ok(Self::finish_binary(
                if kind == &CallableKind::Equal {
                    CompilerBinary::Equal
                } else {
                    CompilerBinary::NotEqual
                },
                reversed,
                right,
                CompilerType::Boolean,
                span,
            ));
        }
        if let [operand, Expression::Identifier(operation)] = items
            && self.source.slice(*operation) == "reverse"
        {
            let operand = self.analyze_expression(operand, environment)?;
            let CompilerType::List(element) = &operand.value_type else {
                return Err(unsupported(
                    &self.source,
                    operand.span,
                    "reverse for this value type",
                ));
            };
            if !matches!(
                element.as_ref(),
                CompilerType::Int | CompilerType::Character
            ) && !compiler_nested_int_list_element(element.as_ref())
                && !compiler_int_string_pair(element.as_ref())
                && !compiler_int_list_pair(element.as_ref())
            {
                return Err(unsupported(
                    &self.source,
                    operand.span,
                    "reverse for this List element type",
                ));
            }
            let value_type = operand.value_type.clone();
            return Ok(CompilerExpression {
                kind: CompilerExpressionKind::ListReverse(Box::new(operand)),
                value_type,
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [Expression::Identifier(operation), operand] = items
            && matches!(self.source.slice(*operation), "first" | "rest" | "uncons")
        {
            let operation = self.source.slice(*operation).to_owned();
            let operand = self.analyze_expression(operand, environment)?;
            let CompilerType::List(element) = &operand.value_type else {
                return Err(unsupported(
                    &self.source,
                    operand.span,
                    "List projection operand",
                ));
            };
            if element.as_ref() != &CompilerType::Int
                && !(operation == "first"
                    && (compiler_list_observation_element_supported(element.as_ref())
                        || compiler_nested_int_list_element(element.as_ref())
                        || compiler_nested_int_string_list_element(element.as_ref())))
                && !(operation == "rest" && element.as_ref() == &CompilerType::String)
            {
                return Err(unsupported(
                    &self.source,
                    operand.span,
                    "projection for this List element type",
                ));
            }
            let element_type = element.as_ref().clone();
            let list_type = operand.value_type.clone();
            let (kind, payload_type) = match operation.as_str() {
                "first" => (
                    CompilerExpressionKind::ListFirst(Box::new(operand)),
                    element_type,
                ),
                "rest" => (
                    CompilerExpressionKind::ListRest(Box::new(operand)),
                    list_type.clone(),
                ),
                "uncons" => (
                    CompilerExpressionKind::ListUncons(Box::new(operand)),
                    CompilerType::Tuple(vec![CompilerType::Int, list_type]),
                ),
                _ => unreachable!(),
            };
            return Ok(CompilerExpression {
                kind,
                value_type: CompilerType::Optional(Box::new(payload_type)),
                int_range: None,
                rational_value: None,
                span,
            });
        }
        if let [list, Expression::Identifier(operation), right] = items
            && matches!(
                self.source.slice(*operation),
                "prepend" | "append" | "concat"
            )
        {
            let operation = self.source.slice(*operation).to_owned();
            let list = self.analyze_expression(list, environment)?;
            if let CompilerType::List(element) = &list.value_type {
                let supported = matches!(
                    element.as_ref(),
                    CompilerType::Int | CompilerType::Nat | CompilerType::Rational
                ) || element.as_ref() == &CompilerType::String
                    && matches!(operation.as_str(), "append" | "concat")
                    || operation == "append" && element.as_ref() == &CompilerType::Boolean
                    || operation == "append"
                        && (compiler_integer_pair(element.as_ref())
                            || compiler_rational_nat_pair(element.as_ref())
                            || compiler_list_integer_pair(element.as_ref())
                            || compiler_int_string_pair(element.as_ref())
                            || compiler_string_pair(element.as_ref())
                            || compiler_int_list_pair(element.as_ref())
                            || compiler_boolean_string_pair(element.as_ref())
                            || compiler_int_int_boolean_pair(element.as_ref())
                            || compiler_int_int_string_int_tuple(element.as_ref())
                            || compiler_int_triple(element.as_ref())
                            || compiler_string_string_rational_triple(element.as_ref())
                            || compiler_string_rational_string_list_triple(element.as_ref())
                            || compiler_string_string_list_int_triple(element.as_ref())
                            || compiler_int_int_list_triple(element.as_ref())
                            || compiler_nested_int_list_element(element.as_ref())
                            || compiler_nested_string_list_element(element.as_ref())
                            || compiler_nested_integer_pair_list_element(element.as_ref())
                            || matches!(element.as_ref(), CompilerType::Range(endpoint)
                                if endpoint.as_ref() == &CompilerType::Int))
                    || (compiler_nested_int_list_element(element.as_ref())
                        || compiler_nested_string_list_element(element.as_ref())
                        || compiler_integer_pair(element.as_ref())
                        || compiler_int_string_pair(element.as_ref())
                        || compiler_int_list_pair(element.as_ref())
                        || compiler_boolean_string_pair(element.as_ref())
                        || compiler_int_int_string_int_tuple(element.as_ref()))
                        && operation == "concat";
                if !supported {
                    return Err(unsupported(
                        &self.source,
                        list.span,
                        "operation for this List element type",
                    ));
                }
                let list_type = list.value_type.clone();
                let right = if operation == "concat" {
                    self.analyze_expression_with_expected(right, environment, Some(&list_type))?
                } else if element.as_ref() == &CompilerType::Nat {
                    self.analyze_expression(right, environment)?
                } else {
                    self.analyze_expression_with_expected(
                        right,
                        environment,
                        Some(element.as_ref()),
                    )?
                };
                let kind = match operation.as_str() {
                    "prepend" => {
                        let right = self.finish_required_list_entry_value(right, element)?;
                        CompilerExpressionKind::ListPrepend {
                            list: Box::new(list),
                            value: Box::new(right),
                        }
                    }
                    "append" => {
                        let right = self.finish_required_list_entry_value(right, element)?;
                        CompilerExpressionKind::ListAppend {
                            list: Box::new(list),
                            value: Box::new(right),
                        }
                    }
                    "concat" => {
                        require_same_type(&self.source, right.span, &list_type, &right.value_type)?;
                        CompilerExpressionKind::ListConcat {
                            left: Box::new(list),
                            right: Box::new(right),
                        }
                    }
                    _ => unreachable!(),
                };
                return Ok(CompilerExpression {
                    kind,
                    value_type: list_type,
                    int_range: None,
                    rational_value: None,
                    span,
                });
            }
            if operation != "concat" {
                return Err(unsupported(
                    &self.source,
                    list.span,
                    "List insertion subject",
                ));
            }
        }
        self.analyze_collection_application(items, span, environment)
    }
}
