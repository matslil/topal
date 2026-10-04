impl Parser<'_> {
    fn duplicate_arithmetic_code(&self, rules: &[DecisionRule]) -> Option<(Span, &str)> {
        let mut seen = std::collections::BTreeSet::new();
        rules.iter().find_map(|rule| match rule.matcher {
            DecisionMatcher::ErrorCode {
                namespace,
                vocabulary,
                code,
                ..
            } if self.source.slice(namespace) == "lang"
                && self.source.slice(vocabulary) == "arithmetic" =>
            {
                let name = self.source.slice(code);
                (!seen.insert(name)).then_some((code, name))
            }
            _ => None,
        })
    }

    fn error_code_after_fallback(rules: &[DecisionRule]) -> Option<Span> {
        let fallback = rules
            .iter()
            .position(|rule| matches!(rule.matcher, DecisionMatcher::Result { error: true, .. }))?;
        rules
            .iter()
            .skip(fallback + 1)
            .find_map(|rule| match rule.matcher {
                DecisionMatcher::ErrorCode { span, .. } => Some(span),
                _ => None,
            })
    }

    fn rule_after_otherwise(rules: &[DecisionRule]) -> Option<Span> {
        let fallback = rules
            .iter()
            .position(|rule| matches!(rule.matcher, DecisionMatcher::Otherwise(_)))?;
        rules
            .get(fallback + 1)
            .map(|rule| matcher_span(&rule.matcher))
    }

    #[allow(clippy::too_many_lines)] // Matcher-specific diagnostics remain explicit and source-located.
    fn decision_rule(&mut self) -> Option<DecisionRule> {
        let matcher_token = self.take_nontrivia()?;
        let (matcher, action) = match matcher_token.kind {
            TokenKind::Boolean => {
                let separator = self.take_nontrivia()?;
                if separator.kind != TokenKind::Identifier
                    || self.source.slice(separator.span) != "then"
                {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-EXPECTED-THEN",
                        span: separator.span,
                        message: "expected `then` between the matcher and delayed action".into(),
                    });
                    return None;
                }
                (
                    DecisionMatcher::Boolean {
                        value: self.source.slice(matcher_token.span) == "true",
                        span: matcher_token.span,
                    },
                    self.expression()?,
                )
            }
            TokenKind::Identifier if self.source.slice(matcher_token.span) == "otherwise" => (
                DecisionMatcher::Otherwise(matcher_token.span),
                self.expression()?,
            ),
            TokenKind::Identifier if self.begins_error_code_pattern(matcher_token.span) => (
                self.error_code_matcher(matcher_token.span)?,
                self.expression()?,
            ),
            TokenKind::Identifier
                if matches!(self.source.slice(matcher_token.span), "Ok" | "Error") =>
            {
                let binding = self.take_nontrivia()?;
                let separator = self.take_nontrivia()?;
                if binding.kind != TokenKind::Identifier
                    || separator.kind != TokenKind::Identifier
                    || self.source.slice(separator.span) != "then"
                {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-EXPECTED-RESULT-PATTERN",
                        span: Span::new(matcher_token.span.start, separator.span.end),
                        message: "expected `Ok name then` or `Error name then`".into(),
                    });
                    return None;
                }
                (
                    DecisionMatcher::Result {
                        error: self.source.slice(matcher_token.span) == "Error",
                        binding: binding.span,
                        span: Span::new(matcher_token.span.start, binding.span.end),
                    },
                    self.expression()?,
                )
            }
            TokenKind::Identifier
                if matches!(self.source.slice(matcher_token.span), "Some" | "None") =>
            {
                (self.optional_matcher(matcher_token)?, self.expression()?)
            }
            TokenKind::Identifier if self.source.slice(matcher_token.span) == "Empty" => {
                (self.list_empty_matcher(matcher_token)?, self.expression()?)
            }
            TokenKind::Identifier if self.source.slice(matcher_token.span) == "Entry" => {
                (self.list_entry_matcher(matcher_token)?, self.expression()?)
            }
            TokenKind::Identifier
                if self.peek_nontrivia().is_some_and(|token| {
                    token.kind == TokenKind::Identifier && self.source.slice(token.span) == "at"
                }) =>
            {
                (self.variant_matcher(matcher_token)?, self.expression()?)
            }
            TokenKind::Identifier => (self.identifier_matcher(matcher_token)?, self.expression()?),
            token_kind if comparison_callable(token_kind).is_some() => {
                let kind = comparison_callable(token_kind).expect("checked comparison token");
                let operand = self.expression_before_then(matcher_token.span)?;
                let span = Span::new(matcher_token.span.start, operand.span().end);
                (
                    DecisionMatcher::Comparison {
                        kind,
                        operand,
                        span,
                    },
                    self.expression()?,
                )
            }
            _ => {
                self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-UNSUPPORTED-DECISION-MATCHER",
                        span: matcher_token.span,
                        message: "the implemented decision subset accepts Boolean literals, comparisons, or `otherwise`".into(),
                    });
                return None;
            }
        };
        let span = Span::new(matcher_span(&matcher).start, action.span().end);
        Some(DecisionRule {
            matcher,
            action,
            span,
        })
    }

    fn optional_matcher(&mut self, constructor: Token) -> Option<DecisionMatcher> {
        let some = self.source.slice(constructor.span) == "Some";
        let binding = some.then(|| self.take_nontrivia()).flatten();
        let separator = self.take_nontrivia()?;
        let valid = separator.kind == TokenKind::Identifier
            && self.source.slice(separator.span) == "then"
            && binding.is_none_or(|binding| binding.kind == TokenKind::Identifier);
        if !valid {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-OPTIONAL-PATTERN",
                span: Span::new(constructor.span.start, separator.span.end),
                message: if some {
                    "expected `Some name then`"
                } else {
                    "expected `None then`"
                }
                .into(),
            });
            return None;
        }
        Some(DecisionMatcher::Optional {
            some,
            binding: binding.map(|binding| binding.span),
            span: Span::new(
                constructor.span.start,
                binding.map_or(constructor.span.end, |binding| binding.span.end),
            ),
        })
    }

    fn list_entry_matcher(&mut self, constructor: Token) -> Option<DecisionMatcher> {
        let opening = self.take_nontrivia()?;
        let first = self.take_nontrivia()?;
        let comma = self.take_nontrivia()?;
        let rest = self.take_nontrivia()?;
        let closing = self.take_nontrivia()?;
        let separator = self.take_nontrivia()?;
        if opening.kind != TokenKind::LeftParen
            || first.kind != TokenKind::Identifier
            || comma.kind != TokenKind::Comma
            || rest.kind != TokenKind::Identifier
            || closing.kind != TokenKind::RightParen
            || separator.kind != TokenKind::Identifier
            || self.source.slice(separator.span) != "then"
        {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-LIST-PATTERN",
                span: Span::new(constructor.span.start, separator.span.end),
                message: "expected `Entry ( first, rest ) then`".into(),
            });
            return None;
        }
        Some(DecisionMatcher::ListEntry {
            first: first.span,
            rest: rest.span,
            span: Span::new(constructor.span.start, closing.span.end),
        })
    }

    fn list_empty_matcher(&mut self, constructor: Token) -> Option<DecisionMatcher> {
        let separator = self.take_nontrivia()?;
        if separator.kind != TokenKind::Identifier || self.source.slice(separator.span) != "then" {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-LIST-PATTERN",
                span: Span::new(constructor.span.start, separator.span.end),
                message: "expected `Empty then`".into(),
            });
            return None;
        }
        Some(DecisionMatcher::ListEmpty(constructor.span))
    }

    fn identifier_matcher(&mut self, identifier: Token) -> Option<DecisionMatcher> {
        let separator = self.take_nontrivia()?;
        if separator.kind == TokenKind::Identifier && self.source.slice(separator.span) != "then" {
            let then = self.take_nontrivia()?;
            if then.kind == TokenKind::Identifier && self.source.slice(then.span) == "then" {
                return Some(DecisionMatcher::Union {
                    alternative: identifier.span,
                    binding: separator.span,
                    span: Span::new(identifier.span.start, separator.span.end),
                });
            }
        }
        if separator.kind != TokenKind::Identifier || self.source.slice(separator.span) != "then" {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-THEN",
                span: separator.span,
                message: "expected `then` between the matcher and delayed action".into(),
            });
            return None;
        }
        Some(DecisionMatcher::Identifier(identifier.span))
    }

    fn variant_matcher(&mut self, type_name: Token) -> Option<DecisionMatcher> {
        self.take_nontrivia();
        let index = self.take_nontrivia()?;
        let binding = self.take_nontrivia()?;
        let then = self.take_nontrivia()?;
        if index.kind != TokenKind::Integer
            || binding.kind != TokenKind::Identifier
            || then.kind != TokenKind::Identifier
            || self.source.slice(then.span) != "then"
        {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-VARIANT-PATTERN",
                span: Span::new(type_name.span.start, then.span.end),
                message: "expected `Type at index binding then`".into(),
            });
            return None;
        }
        Some(DecisionMatcher::Variant {
            type_name: type_name.span,
            index: index.span,
            binding: binding.span,
            span: Span::new(type_name.span.start, binding.span.end),
        })
    }

    fn error_code_matcher(&mut self, error: Span) -> Option<DecisionMatcher> {
        let opening = self.take_nontrivia()?;
        let field = self.take_nontrivia()?;
        let is = self.take_nontrivia()?;
        let namespace = self.take_nontrivia()?;
        let vocabulary = self.take_nontrivia()?;
        let code = self.take_nontrivia()?;
        let closing = self.take_nontrivia()?;
        let separator = self.take_nontrivia()?;
        if opening.kind != TokenKind::LeftParen
            || field.kind != TokenKind::Identifier
            || self.source.slice(field.span) != "code"
            || is.kind != TokenKind::Identifier
            || self.source.slice(is.span) != "is"
            || namespace.kind != TokenKind::Identifier
            || vocabulary.kind != TokenKind::Identifier
            || code.kind != TokenKind::Identifier
            || closing.kind != TokenKind::RightParen
            || separator.kind != TokenKind::Identifier
            || self.source.slice(separator.span) != "then"
        {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-ERROR-CODE-PATTERN",
                span: Span::new(error.start, separator.span.end),
                message: "expected `Error ( code is namespace vocabulary code ) then`".into(),
            });
            return None;
        }
        Some(DecisionMatcher::ErrorCode {
            namespace: namespace.span,
            vocabulary: vocabulary.span,
            code: code.span,
            span: Span::new(error.start, closing.span.end),
        })
    }

    fn begins_error_code_pattern(&self, matcher: Span) -> bool {
        self.source.slice(matcher) == "Error"
            && self
                .peek_nontrivia()
                .is_some_and(|token| token.kind == TokenKind::LeftParen)
    }

    fn expression_before_then(&mut self, matcher_span: Span) -> Option<Expression> {
        let Some(separator_index) = self.tokens[self.cursor..]
            .iter()
            .take_while(|token| token.kind != TokenKind::Newline)
            .position(|token| {
                token.kind == TokenKind::Identifier && self.source.slice(token.span) == "then"
            })
            .map(|offset| self.cursor + offset)
        else {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-THEN",
                span: Span::new(matcher_span.end, matcher_span.end),
                message: "expected `then` between the matcher and delayed action".into(),
            });
            return None;
        };
        let mut parser = Self {
            source: self.source,
            tokens: &self.tokens[self.cursor..separator_index],
            cursor: 0,
            delimiter_depth: 0,
            current_indent: self.current_indent,
            diagnostics: Vec::new(),
        };
        let operand = parser.expression();
        self.diagnostics.extend(parser.diagnostics);
        self.cursor = separator_index + 1;
        operand
    }

    fn static_function_parameters(
        &mut self,
        opening: Token,
    ) -> Option<(Vec<FunctionParameter>, Token)> {
        let delimited = opening.kind == TokenKind::LeftParen;
        self.delimiter_depth += usize::from(delimited);
        let parsed = self.static_function_parameters_inner(opening);
        self.delimiter_depth -= usize::from(delimited);
        parsed
    }

    #[allow(clippy::too_many_lines)] // Classifier forms are parsed explicitly for precise spans.
    fn static_function_parameters_inner(
        &mut self,
        opening: Token,
    ) -> Option<(Vec<FunctionParameter>, Token)> {
        let mut parameters = Vec::new();
        let mut input = self.take_nontrivia()?;
        let closing = loop {
            if input.kind == TokenKind::RightParen {
                break input;
            }
            if input.kind == TokenKind::LeftParen {
                let (fields, package_closing) = self.parameter_package(input)?;
                let separator = self.take_nontrivia()?;
                if !matches!(separator.kind, TokenKind::Comma | TokenKind::RightParen) {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-FUNCTION-PARAMETER-PACKAGE",
                        span: separator.span,
                        message: "expected `,` or `)` after packaged function operand".into(),
                    });
                    return None;
                }
                parameters.push(FunctionParameter {
                    name: input.span,
                    classifier: Span::new(input.span.start, package_closing.span.end),
                    qualifier: None,
                    fields,
                    default: None,
                });
                if separator.kind == TokenKind::RightParen {
                    break separator;
                }
                input = self.take_nontrivia()?;
                continue;
            }
            let colon = self.take_nontrivia()?;
            let classifier_start = self.take_nontrivia()?;
            let classifier = self.classifier_from_first(classifier_start)?;
            let mut separator = self.take_nontrivia()?;
            let qualifier = if separator.kind == TokenKind::Colon {
                let qualifier = self.take_nontrivia()?;
                if qualifier.kind != TokenKind::Identifier
                    || !matches!(self.source.slice(qualifier.span), "Exclusive" | "Consumes")
                {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-FUNCTION-PARAMETER-QUALIFIER",
                        span: qualifier.span,
                        message:
                            "the implemented parameter qualifiers are `Exclusive` and `Consumes`"
                                .into(),
                    });
                    return None;
                }
                separator = self.take_nontrivia()?;
                Some(qualifier.span)
            } else {
                None
            };
            if !matches!(input.kind, TokenKind::Identifier | TokenKind::Discard)
                || colon.kind != TokenKind::Colon
                || !matches!(
                    classifier_start.kind,
                    TokenKind::Identifier | TokenKind::LeftParen
                )
                || !matches!(separator.kind, TokenKind::Comma | TokenKind::RightParen)
            {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-UNSUPPORTED-FUNCTION-HEADER",
                    span: Span::new(opening.span.start, separator.span.end),
                    message:
                        "function parameters must have the form `pattern : Type`, separated by commas"
                            .into(),
                });
                self.skip_to_newline();
                return None;
            }
            parameters.push(FunctionParameter {
                name: input.span,
                classifier,
                qualifier,
                fields: Vec::new(),
                default: None,
            });
            if separator.kind == TokenKind::RightParen {
                break separator;
            }
            input = self.take_nontrivia()?;
        };
        Some((parameters, closing))
    }

    fn parameter_package(&mut self, opening: Token) -> Option<(Vec<FunctionParameter>, Token)> {
        self.delimiter_depth += 1;
        let mut fields = Vec::new();
        let closing = loop {
            let name = self.take_nontrivia()?;
            if name.kind == TokenKind::RightParen {
                break name;
            }
            let colon = self.take_nontrivia()?;
            let classifier_start = self.take_nontrivia()?;
            let classifier = self.classifier_from_first(classifier_start)?;
            if !matches!(name.kind, TokenKind::Identifier | TokenKind::Discard)
                || colon.kind != TokenKind::Colon
            {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-FUNCTION-PARAMETER-PACKAGE",
                    span: Span::new(opening.span.start, classifier.end),
                    message: "packaged parameters require `name : Type` fields".into(),
                });
                self.delimiter_depth -= 1;
                return None;
            }
            let default = if self.peek_nontrivia().is_some_and(|token| {
                token.kind == TokenKind::Identifier && self.source.slice(token.span) == "default"
            }) {
                self.take_nontrivia();
                self.expression()
            } else {
                None
            };
            let separator = self.take_nontrivia()?;
            fields.push(FunctionParameter {
                name: name.span,
                classifier,
                qualifier: None,
                fields: Vec::new(),
                default,
            });
            if separator.kind == TokenKind::RightParen {
                break separator;
            }
            if separator.kind != TokenKind::Comma {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-FUNCTION-PARAMETER-PACKAGE",
                    span: separator.span,
                    message: "expected `,` or `)` after packaged parameter field".into(),
                });
                self.delimiter_depth -= 1;
                return None;
            }
        };
        self.delimiter_depth -= 1;
        Some((fields, closing))
    }

    fn expression(&mut self) -> Option<Expression> {
        let first = self.primary()?;
        let mut items = vec![first];
        loop {
            if self
                .peek_nontrivia()
                .is_none_or(|token| matches!(token.kind, TokenKind::RightParen | TokenKind::Comma))
            {
                break;
            }
            let Some(item) = self.primary() else {
                break;
            };
            items.push(item);
        }
        if items.len() == 1 {
            return items.pop();
        }
        let span = Span::new(
            items[0].span().start,
            items.last().expect("nonempty").span().end,
        );
        Some(Expression::Application { items, span })
    }

    #[allow(clippy::too_many_lines)] // Literal suffix validation stays beside the primary token dispatch.
    fn primary(&mut self) -> Option<Expression> {
        let token = self.take_nontrivia()?;
        match token.kind {
            TokenKind::Boolean => Some(Expression::Boolean(token.span)),
            TokenKind::Integer => {
                if self
                    .peek_nontrivia()
                    .is_some_and(|next| next.kind == TokenKind::LeftBracket)
                {
                    self.take_nontrivia();
                    let unit = self.take_nontrivia()?;
                    let closing = self.take_nontrivia()?;
                    if unit.kind != TokenKind::Identifier || closing.kind != TokenKind::RightBracket
                    {
                        self.diagnostics.push(SyntaxDiagnostic {
                            code: "E-MEASUREMENT-SUFFIX",
                            span: Span::new(token.span.start, closing.span.end),
                            message: "a size suffix has the form `[b]` or `[B]`".into(),
                        });
                        return None;
                    }
                    Some(Expression::Measured {
                        value: token.span,
                        unit: unit.span,
                        span: Span::new(token.span.start, closing.span.end),
                    })
                } else {
                    Some(Expression::Integer(token.span))
                }
            }
            TokenKind::Rational => Some(Expression::Rational(token.span)),
            TokenKind::Infinity => Some(Expression::Infinity(token.span)),
            TokenKind::String => Some(Expression::String(token.span)),
            TokenKind::Identifier | TokenKind::Version => Some(Expression::Identifier(token.span)),
            TokenKind::At => {
                let selected = self.take_nontrivia()?;
                if selected.kind != TokenKind::Identifier {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-CONTEXT-SELECTION",
                        span: selected.span,
                        message: "`@` requires a member of the defining context".into(),
                    });
                    return None;
                }
                Some(Expression::ContextIdentifier(selected.span))
            }
            TokenKind::Discard => Some(Expression::Discard(token.span)),
            TokenKind::Equals => Some(Expression::Callable {
                kind: CallableKind::Equal,
                span: token.span,
            }),
            TokenKind::NotEquals => Some(Expression::Callable {
                kind: CallableKind::NotEqual,
                span: token.span,
            }),
            TokenKind::Less => Some(Expression::Callable {
                kind: CallableKind::Less,
                span: token.span,
            }),
            TokenKind::Greater => Some(Expression::Callable {
                kind: CallableKind::Greater,
                span: token.span,
            }),
            TokenKind::LessEqual => Some(Expression::Callable {
                kind: CallableKind::LessEqual,
                span: token.span,
            }),
            TokenKind::Compare => Some(Expression::Callable {
                kind: CallableKind::Compare,
                span: token.span,
            }),
            TokenKind::Range => Some(Expression::Callable {
                kind: CallableKind::Range,
                span: token.span,
            }),
            TokenKind::RangeOpen => Some(Expression::Callable {
                kind: CallableKind::RangeOpen,
                span: token.span,
            }),
            TokenKind::RangeInclusive => Some(Expression::Callable {
                kind: CallableKind::RangeInclusive,
                span: token.span,
            }),
            TokenKind::RangeOpenInclusive => Some(Expression::Callable {
                kind: CallableKind::RangeOpenInclusive,
                span: token.span,
            }),
            TokenKind::GreaterEqual => Some(Expression::Callable {
                kind: CallableKind::GreaterEqual,
                span: token.span,
            }),
            TokenKind::Plus => Some(Expression::Callable {
                kind: CallableKind::Plus,
                span: token.span,
            }),
            TokenKind::Minus => Some(Expression::Callable {
                kind: CallableKind::Minus,
                span: token.span,
            }),
            TokenKind::Star => Some(Expression::Callable {
                kind: CallableKind::Multiply,
                span: token.span,
            }),
            TokenKind::Slash => Some(Expression::Callable {
                kind: CallableKind::Divide,
                span: token.span,
            }),
            TokenKind::SlashPercent => Some(Expression::Callable {
                kind: CallableKind::QuotientModulo,
                span: token.span,
            }),
            TokenKind::Percent => Some(Expression::Callable {
                kind: CallableKind::Modulo,
                span: token.span,
            }),
            TokenKind::Caret => Some(Expression::Callable {
                kind: CallableKind::Power,
                span: token.span,
            }),
            TokenKind::LeftParen => {
                self.delimiter_depth += 1;
                let expression = self.parenthesized(token);
                self.delimiter_depth -= 1;
                expression
            }
            TokenKind::LeftBrace => self.braced_expression(token),
            _ => {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-EXPECTED-EXPRESSION",
                    span: token.span,
                    message: "expected a literal, name, callable, or parenthesized expression"
                        .into(),
                });
                None
            }
        }
    }

    fn braced_expression(&mut self, opening: Token) -> Option<Expression> {
        let start = self.cursor;
        let mut depth = 1_usize;
        let mut closing_index = None;
        for (offset, token) in self.tokens[start..].iter().enumerate() {
            match token.kind {
                TokenKind::LeftBrace => depth += 1,
                TokenKind::RightBrace => {
                    depth -= 1;
                    if depth == 0 {
                        closing_index = Some(start + offset);
                        break;
                    }
                }
                _ => {}
            }
        }
        let Some(closing_index) = closing_index else {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-RBRACE",
                span: opening.span,
                message: "expected a closing brace".into(),
            });
            return None;
        };
        if self.braces_introduce_anonymous_function(start, closing_index) {
            return self.anonymous_function(opening);
        }

        let closing = self.tokens[closing_index];
        let block_indent = self.tokens[start..closing_index]
            .windows(2)
            .filter_map(|tokens| {
                (tokens[0].kind == TokenKind::Newline && tokens[1].kind == TokenKind::Whitespace)
                    .then_some(tokens[1].span.end - tokens[1].span.start)
            })
            .min()
            .unwrap_or(self.current_indent);
        let mut block_parser = Self {
            source: self.source,
            tokens: &self.tokens[start..closing_index],
            cursor: 0,
            delimiter_depth: 0,
            current_indent: block_indent,
            diagnostics: Vec::new(),
        };
        let mut statements = Vec::new();
        while block_parser.skip_separators() {
            if let Some(statement) = block_parser.statement() {
                statements.push(statement);
            }
            if block_parser.peek_nontrivia().is_some() {
                block_parser.error_current(
                    "E-UNEXPECTED-TOKEN",
                    "unexpected token after block statement",
                );
                block_parser.skip_to_newline();
            }
        }
        validate_diagnostic_controls(self.source, &statements, &mut block_parser.diagnostics);
        self.diagnostics.extend(block_parser.diagnostics);
        self.cursor = closing_index + 1;
        Some(Expression::Block {
            statements,
            span: Span::new(opening.span.start, closing.span.end),
        })
    }

    fn braces_introduce_anonymous_function(&self, start: usize, closing: usize) -> bool {
        let content = self.tokens[start..closing]
            .iter()
            .filter(|token| !token.kind.is_trivia())
            .collect::<Vec<_>>();
        let mut index = 0;
        let mut parameters = !content.is_empty();
        while parameters && index < content.len() {
            if !anonymous_pattern_tokens(&content, &mut index) {
                parameters = false;
                break;
            }
            if index < content.len() {
                if content[index].kind != TokenKind::Comma {
                    parameters = false;
                    break;
                }
                index += 1;
            }
        }
        if !parameters {
            return false;
        }
        self.tokens[closing + 1..]
            .iter()
            .take_while(|token| token.kind != TokenKind::Newline)
            .any(|token| !token.kind.is_trivia())
    }

    fn anonymous_function(&mut self, opening: Token) -> Option<Expression> {
        let mut parameters = Vec::new();
        loop {
            let token = self.peek_nontrivia()?;
            if token.kind == TokenKind::RightBrace {
                self.take_nontrivia();
                if parameters.is_empty() {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-EMPTY-ANONYMOUS-FUNCTION-PATTERN",
                        span: Span::new(opening.span.start, token.span.end),
                        message:
                            "an inferred anonymous function requires at least one parameter pattern"
                                .into(),
                    });
                    return None;
                }
                let Some(body) = self.expression() else {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-EXPECTED-ANONYMOUS-FUNCTION-BODY",
                        span: token.span,
                        message: "expected an anonymous-function body after the parameter pattern"
                            .into(),
                    });
                    return None;
                };
                let span = Span::new(opening.span.start, body.span().end);
                return Some(Expression::AnonymousFunction {
                    parameters,
                    body: Box::new(body),
                    span,
                });
            }
            let parameter = self.anonymous_pattern()?;
            let parameter_span = parameter.span();
            parameters.push(parameter);
            let Some(separator) = self.peek_nontrivia() else {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-EXPECTED-RBRACE",
                    span: parameter_span,
                    message: "expected `}` after anonymous-function parameters".into(),
                });
                return None;
            };
            if separator.kind == TokenKind::Comma {
                self.take_nontrivia();
            } else if separator.kind != TokenKind::RightBrace {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-EXPECTED-ANONYMOUS-FUNCTION-SEPARATOR",
                    span: separator.span,
                    message: "expected `,` or `}` after anonymous-function parameter".into(),
                });
                return None;
            }
        }
    }

    fn anonymous_pattern(&mut self) -> Option<AnonymousPattern> {
        let token = self.take_nontrivia()?;
        if matches!(token.kind, TokenKind::Identifier | TokenKind::Discard) {
            return Some(AnonymousPattern::Binding(token.span));
        }
        if token.kind != TokenKind::LeftParen {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-ANONYMOUS-FUNCTION-PARAMETER",
                span: token.span,
                message: "expected a parameter pattern or closing brace".into(),
            });
            return None;
        }

        let mut fields = Vec::new();
        let closing = loop {
            let Some(next) = self.peek_nontrivia() else {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-EXPECTED-ANONYMOUS-FUNCTION-PARAMETER",
                    span: token.span,
                    message: "expected a binding or product in the product pattern".into(),
                });
                return None;
            };
            if !matches!(
                next.kind,
                TokenKind::Identifier | TokenKind::Discard | TokenKind::LeftParen
            ) {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-EXPECTED-ANONYMOUS-FUNCTION-PARAMETER",
                    span: next.span,
                    message: "expected a binding or product in the product pattern".into(),
                });
                return None;
            }
            fields.push(self.anonymous_pattern()?);
            let separator = self.take_nontrivia()?;
            if separator.kind == TokenKind::RightParen {
                break separator;
            }
            if separator.kind != TokenKind::Comma {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-EXPECTED-ANONYMOUS-FUNCTION-SEPARATOR",
                    span: separator.span,
                    message: "expected `,` or `)` in the product pattern".into(),
                });
                return None;
            }
        };
        Some(AnonymousPattern::Product {
            fields,
            span: Span::new(token.span.start, closing.span.end),
        })
    }

    fn parenthesized(&mut self, opening: Token) -> Option<Expression> {
        if self
            .peek_nontrivia()
            .is_some_and(|value| value.kind == TokenKind::RightParen)
        {
            let closing = self
                .take_nontrivia()
                .expect("peeked closing parenthesis remains available");
            return Some(Expression::Unit(Span::new(
                opening.span.start,
                closing.span.end,
            )));
        }
        let Some(first) = self.product_field() else {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-RPAREN",
                span: Span::new(opening.span.end, opening.span.end),
                message: "expected expression and closing parenthesis".into(),
            });
            return None;
        };
        let product = first.label.is_some()
            || self
                .peek_nontrivia()
                .is_some_and(|value| value.kind == TokenKind::Comma);
        let mut fields = vec![first.clone()];
        if product {
            loop {
                if !self
                    .peek_nontrivia()
                    .is_some_and(|value| value.kind == TokenKind::Comma)
                {
                    break;
                }
                self.take_nontrivia();
                if self
                    .peek_nontrivia()
                    .is_some_and(|value| value.kind == TokenKind::RightParen)
                {
                    break;
                }
                let Some(field) = self.product_field() else {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-EXPECTED-RPAREN",
                        span: Span::new(opening.span.end, opening.span.end),
                        message: "expected product field or closing parenthesis".into(),
                    });
                    return None;
                };
                fields.push(field);
            }
        }
        let closing = self.take_nontrivia();
        if !closing.is_some_and(|value| value.kind == TokenKind::RightParen) {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-RPAREN",
                span: Span::new(opening.span.end, opening.span.end),
                message: "expected closing parenthesis".into(),
            });
        }
        if product {
            let first_labeled = fields[0].label.is_some();
            if let Some(mixed) = fields
                .iter()
                .find(|field| field.label.is_some() != first_labeled)
            {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-MIXED-PRODUCT-FIELDS",
                    span: mixed.label.unwrap_or_else(|| mixed.value.span()),
                    message: "a product cannot mix positional and labeled fields".into(),
                });
            }
            Some(Expression::Product {
                fields,
                span: Span::new(
                    opening.span.start,
                    closing.map_or(first.value.span().end, |value| value.span.end),
                ),
            })
        } else {
            Some(first.value)
        }
    }

    fn product_field(&mut self) -> Option<ProductField> {
        let checkpoint = self.cursor;
        if let Some(label) = self.take_nontrivia()
            && label.kind == TokenKind::Identifier
            && self.peek_nontrivia().is_some_and(|separator| {
                separator.kind == TokenKind::Identifier && self.source.slice(separator.span) == "is"
            })
        {
            self.take_nontrivia();
            return self.expression().map(|value| ProductField {
                label: Some(label.span),
                value,
            });
        }
        self.cursor = checkpoint;
        self.expression()
            .map(|value| ProductField { label: None, value })
    }

    fn skip_separators(&mut self) -> bool {
        while let Some(token) = self.tokens.get(self.cursor) {
            if token.kind.is_trivia() {
                self.cursor += 1;
            } else {
                break;
            }
        }
        self.cursor < self.tokens.len()
    }

    fn take_nontrivia(&mut self) -> Option<Token> {
        while let Some(token) = self.tokens.get(self.cursor).copied() {
            if token.kind == TokenKind::Newline && self.delimiter_depth == 0 {
                return None;
            }
            self.cursor += 1;
            if !token.kind.is_trivia() {
                return Some(token);
            }
        }
        None
    }

    fn peek_nontrivia(&self) -> Option<Token> {
        for token in self.tokens[self.cursor..].iter().copied() {
            if token.kind == TokenKind::Newline && self.delimiter_depth == 0 {
                return None;
            }
            if !token.kind.is_trivia() {
                return Some(token);
            }
        }
        None
    }

    fn peek(&self) -> Option<Token> {
        self.tokens.get(self.cursor).copied()
    }

    fn error_current(&mut self, code: &'static str, message: &'static str) {
        let span = self.peek().map_or(Span::default(), |token| token.span);
        self.diagnostics.push(SyntaxDiagnostic {
            code,
            span,
            message: message.to_owned(),
        });
    }

    fn skip_to_newline(&mut self) {
        while self
            .peek()
            .is_some_and(|token| token.kind != TokenKind::Newline)
        {
            self.cursor += 1;
        }
    }
}
