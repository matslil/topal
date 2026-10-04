impl Parser<'_> {
    fn statement(&mut self) -> Option<Statement> {
        if self.foreach_separator_index().is_some() {
            return self.foreach_statement();
        }
        self.ordinary_statement()
    }

    #[allow(clippy::too_many_lines)] // Declaration forms retain localized diagnostics.
    fn ordinary_statement(&mut self) -> Option<Statement> {
        let checkpoint = self.cursor;
        let first = self.take_nontrivia()?;
        if first.kind == TokenKind::Identifier && self.interface_implementation_ahead() {
            return self.interface_implementation(first);
        }
        if first.kind == TokenKind::Identifier
            && self.source.slice(first.span) == "use"
            && self.peek_nontrivia().is_some_and(|next| {
                next.kind == TokenKind::Identifier && self.source.slice(next.span) == "language"
            })
        {
            return self.language_selection(first);
        }
        if first.kind == TokenKind::Identifier
            && self.source.slice(first.span) == "use"
            && self.peek_nontrivia().is_some_and(|next| {
                next.kind == TokenKind::Identifier && self.source.slice(next.span) == "library"
            })
        {
            return self.library_selection(first);
        }
        if first.kind == TokenKind::Identifier && self.source.slice(first.span) == "pub" {
            let declaration = self.ordinary_statement()?;
            if !matches!(
                declaration,
                Statement::Binding { .. }
                    | Statement::Function { .. }
                    | Statement::Generator { .. }
                    | Statement::Union { .. }
            ) {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-PUBLICATION-TARGET",
                    span: statement_span(&declaration),
                    message: "`pub` requires a declaration".into(),
                });
                return None;
            }
            let span = Span::new(first.span.start, statement_span(&declaration).end);
            return Some(Statement::Published {
                declaration: Box::new(declaration),
                span,
            });
        }
        if first.kind == TokenKind::Identifier
            && self.source.slice(first.span) == "lang"
            && self.peek_nontrivia().is_some_and(|operation| {
                matches!(
                    self.source.slice(operation.span),
                    "disable-warning"
                        | "push-disable-warning"
                        | "pop-disable-warning"
                        | "disable-diagnostic"
                        | "push-disable-diagnostic"
                        | "pop-disable-diagnostic"
                )
            })
        {
            return self.diagnostic_control(first);
        }
        if first.kind == TokenKind::Identifier && self.source.slice(first.span) == "return" {
            let Some(value) = self.expression() else {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-EXPECTED-RETURN-VALUE",
                    span: Span::new(first.span.end, first.span.end),
                    message: "expected an expression after `return`".into(),
                });
                return None;
            };
            return Some(Statement::Return {
                keyword: first.span,
                value,
            });
        }
        if first.kind == TokenKind::At {
            let name = self.take_nontrivia();
            let separator = self.peek_nontrivia();
            if let (Some(name), Some(separator)) = (name, separator)
                && name.kind == TokenKind::Identifier
                && separator.kind == TokenKind::Identifier
                && self.source.slice(separator.span) == "is"
            {
                self.take_nontrivia();
                let value = self.expression()?;
                return Some(Statement::ContextAssignment {
                    name: name.span,
                    span: Span::new(first.span.start, value.span().end),
                    value,
                });
            }
            self.cursor = checkpoint;
        }
        if first.kind == TokenKind::Boolean
            && self.peek_nontrivia().is_some_and(|second| {
                second.kind == TokenKind::Identifier && self.source.slice(second.span) == "is"
            })
        {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-RESERVED-BOOLEAN-LITERAL",
                span: first.span,
                message: "a Boolean literal cannot introduce a binding".into(),
            });
            self.skip_to_newline();
            return None;
        }
        if matches!(first.kind, TokenKind::Identifier | TokenKind::Discard)
            && let Some(second) = self.peek_nontrivia()
            && second.kind == TokenKind::Identifier
            && self.source.slice(second.span) == "is"
        {
            let separator = self
                .take_nontrivia()
                .expect("peeked token remains available");
            if first.kind == TokenKind::Identifier
                && let Some(declaration) = self.peek_nontrivia()
                && declaration.kind == TokenKind::Identifier
            {
                match self.source.slice(declaration.span) {
                    "fn" => return self.function(first),
                    "generator" => return self.generator(first),
                    "Union" => return self.union(first),
                    "Interface" => return self.interface(first),
                    _ => {}
                }
            }
            if let Some(value) = self.expression() {
                if first.kind == TokenKind::Identifier
                    && let Some((indent, body_start)) = self.indented_body_after_current_line()
                {
                    self.cursor = body_start;
                    let declarations = self.indented_function_body(indent)?;
                    let end = statement_span(declarations.last()?).end;
                    return Some(Statement::Implementation {
                        name: first.span,
                        classifier: value,
                        declarations,
                        span: Span::new(first.span.start, end),
                    });
                }
                return Some(if first.kind == TokenKind::Discard {
                    Statement::Discard {
                        span: first.span,
                        value,
                    }
                } else {
                    Statement::Binding {
                        name: first.span,
                        classifier: None,
                        value,
                    }
                });
            }
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-EXPRESSION",
                span: Span::new(separator.span.end, separator.span.end),
                message: "expected a binding initializer".into(),
            });
            return None;
        }
        if first.kind == TokenKind::Identifier
            && self
                .peek_nontrivia()
                .is_some_and(|token| token.kind == TokenKind::Colon)
        {
            self.take_nontrivia();
            let classifier_start = self.take_nontrivia()?;
            let classifier = self.classifier_from_first(classifier_start)?;
            if self.peek_nontrivia().is_none() {
                return Some(Statement::StateField {
                    name: first.span,
                    classifier,
                });
            }
            let separator = self.take_nontrivia()?;
            if !matches!(
                classifier_start.kind,
                TokenKind::Identifier | TokenKind::LeftParen
            ) || separator.kind != TokenKind::Identifier
                || self.source.slice(separator.span) != "is"
            {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-EXPECTED-CLASSIFIED-BINDING",
                    span: Span::new(first.span.start, separator.span.end),
                    message: "expected `name : Classifier is expression`".into(),
                });
                return None;
            }
            return self.expression().map(|value| Statement::Binding {
                name: first.span,
                classifier: Some(classifier),
                value,
            });
        }
        self.cursor = checkpoint;
        self.expression().map(Statement::Expression)
    }

    fn interface_implementation_ahead(&self) -> bool {
        let remaining = &self.tokens[self.cursor..];
        if !remaining
            .iter()
            .find(|token| {
                !matches!(
                    token.kind,
                    TokenKind::Whitespace | TokenKind::Comment | TokenKind::Documentation
                )
            })
            .is_some_and(|token| token.kind == TokenKind::Newline)
        {
            return false;
        }
        let Some(newline) = remaining
            .iter()
            .position(|token| token.kind == TokenKind::Newline)
        else {
            return false;
        };
        let Some(indent) = remaining
            .get(newline + 1)
            .filter(|token| token.kind == TokenKind::Whitespace)
        else {
            return false;
        };
        if indent.span.end - indent.span.start <= self.current_indent {
            return false;
        }
        let significant = remaining[newline + 1..]
            .iter()
            .filter(|token| {
                !matches!(
                    token.kind,
                    TokenKind::Whitespace | TokenKind::Comment | TokenKind::Documentation
                )
            })
            .take(3)
            .collect::<Vec<_>>();
        matches!(significant.as_slice(), [name, separator, function]
            if name.kind == TokenKind::Identifier
                && separator.kind == TokenKind::Identifier
                && self.source.slice(separator.span) == "is"
                && function.kind == TokenKind::Identifier
                && self.source.slice(function.span) == "fn")
    }

    fn interface_implementation(&mut self, interface: Token) -> Option<Statement> {
        if !self
            .peek()
            .is_some_and(|token| token.kind == TokenKind::Newline)
        {
            return None;
        }
        self.cursor += 1;
        let indent = self.peek()?;
        if indent.kind != TokenKind::Whitespace {
            return None;
        }
        let declarations = self.indented_function_body(indent.span.end - indent.span.start)?;
        let end = statement_span(declarations.last()?).end;
        Some(Statement::InterfaceImplementation {
            interface: interface.span,
            declarations,
            span: Span::new(interface.span.start, end),
        })
    }

    fn language_selection(&mut self, use_keyword: Token) -> Option<Statement> {
        let language = self.take_nontrivia()?;
        let opening = self.take_nontrivia()?;
        if opening.kind != TokenKind::LeftParen {
            self.error_current("E-LANGUAGE-SELECTION", "expected `(` after `use language`");
            return None;
        }
        self.delimiter_depth += 1;
        let mut version = None;
        let mut features = Vec::new();
        let closing = loop {
            let field = self.take_nontrivia()?;
            if field.kind == TokenKind::RightParen {
                break field;
            }
            let separator = self.take_nontrivia()?;
            let value = self.take_nontrivia()?;
            if field.kind != TokenKind::Identifier
                || separator.kind != TokenKind::Identifier
                || self.source.slice(separator.span) != "is"
            {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-LANGUAGE-SELECTION",
                    span: Span::new(field.span.start, value.span.end),
                    message: "expected `field is value` in the language record".into(),
                });
                self.delimiter_depth -= 1;
                return None;
            }
            if self.source.slice(field.span) == "version"
                && (value.kind != TokenKind::Version || version.replace(value.span).is_some())
            {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-LANGUAGE-VERSION",
                    span: value.span,
                    message: "language selection requires one version value".into(),
                });
            }
            if self.source.slice(field.span) == "features" {
                if value.kind == TokenKind::LeftParen {
                    let mut depth = 1_usize;
                    while depth > 0 {
                        let Some(item) = self.take_nontrivia() else {
                            self.diagnostics.push(SyntaxDiagnostic {
                                code: "E-LANGUAGE-FEATURES",
                                span: value.span,
                                message: "language feature collection is not closed".into(),
                            });
                            self.delimiter_depth -= 1;
                            return None;
                        };
                        match item.kind {
                            TokenKind::LeftParen => depth += 1,
                            TokenKind::RightParen => depth -= 1,
                            TokenKind::Identifier if depth == 1 => features.push(item.span),
                            TokenKind::Comma => {}
                            _ => self.diagnostics.push(SyntaxDiagnostic {
                                code: "E-LANGUAGE-FEATURES",
                                span: item.span,
                                message: "language features must be identifiers".into(),
                            }),
                        }
                    }
                } else {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-LANGUAGE-FEATURES",
                        span: value.span,
                        message: "language features require a parenthesized collection".into(),
                    });
                }
            } else {
                while self.peek_nontrivia().is_some_and(|token| {
                    !matches!(token.kind, TokenKind::Comma | TokenKind::RightParen)
                }) {
                    self.take_nontrivia();
                }
            }
            if self
                .peek_nontrivia()
                .is_some_and(|token| token.kind == TokenKind::Comma)
            {
                self.take_nontrivia();
            }
        };
        self.delimiter_depth -= 1;
        let Some(version) = version else {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-LANGUAGE-VERSION",
                span: language.span,
                message: "language selection requires a `version` association".into(),
            });
            return None;
        };
        Some(Statement::LanguageSelection {
            version,
            features,
            span: Span::new(use_keyword.span.start, closing.span.end),
        })
    }

    fn library_selection(&mut self, use_keyword: Token) -> Option<Statement> {
        let library = self.take_nontrivia()?;
        let name = self.take_nontrivia()?;
        let opening = self.take_nontrivia()?;
        if opening.kind != TokenKind::LeftParen {
            self.error_current(
                "E-LIBRARY-SELECTION",
                "expected `(` after the library identity",
            );
            return None;
        }
        self.delimiter_depth += 1;
        let field = self.take_nontrivia()?;
        let separator = self.take_nontrivia()?;
        let version = self.take_nontrivia()?;
        let closing = self.take_nontrivia()?;
        self.delimiter_depth -= 1;
        let valid = name.kind == TokenKind::Identifier
            && field.kind == TokenKind::Identifier
            && self.source.slice(field.span) == "version"
            && separator.kind == TokenKind::Identifier
            && self.source.slice(separator.span) == "is"
            && version.kind == TokenKind::Version
            && closing.kind == TokenKind::RightParen;
        if !valid {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-LIBRARY-SELECTION",
                span: Span::new(library.span.start, closing.span.end),
                message: "expected `use library name ( version is V )`".into(),
            });
            return None;
        }
        Some(Statement::LibrarySelection {
            name: name.span,
            version: version.span,
            span: Span::new(use_keyword.span.start, closing.span.end),
        })
    }

    fn diagnostic_control(&mut self, lang: Token) -> Option<Statement> {
        let operation = self.take_nontrivia()?;
        let operation_text = self.source.slice(operation.span);
        let kind = match operation_text {
            "disable-warning" | "disable-diagnostic" => DiagnosticControlKind::DisableNext,
            "push-disable-warning" | "push-disable-diagnostic" => DiagnosticControlKind::Push,
            "pop-disable-warning" | "pop-disable-diagnostic" => DiagnosticControlKind::Pop,
            _ => {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-DIAGNOSTIC-CONTROL",
                    span: operation.span,
                    message: "expected a diagnostic-control operation after `lang`".into(),
                });
                self.skip_to_newline();
                return None;
            }
        };
        let structured = operation_text.ends_with("-diagnostic");
        let mut identity = Vec::new();
        let end = if structured {
            let opening = self.take_nontrivia()?;
            if opening.kind != TokenKind::LeftParen {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-DIAGNOSTIC-CONTROL",
                    span: opening.span,
                    message: "expected a parenthesized diagnostic identity".into(),
                });
                return None;
            }
            loop {
                let Some(token) = self.take_nontrivia() else {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-DIAGNOSTIC-CONTROL",
                        span: opening.span,
                        message: "expected `)` after diagnostic identity".into(),
                    });
                    return None;
                };
                if token.kind == TokenKind::RightParen {
                    if identity.len() < 2 {
                        self.diagnostics.push(SyntaxDiagnostic {
                            code: "E-DIAGNOSTIC-CONTROL",
                            span: Span::new(opening.span.start, token.span.end),
                            message: "diagnostic identity requires a namespace and name".into(),
                        });
                        return None;
                    }
                    break token.span.end;
                }
                if token.kind != TokenKind::Identifier {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-DIAGNOSTIC-CONTROL",
                        span: token.span,
                        message: "diagnostic identity contains only identifier components".into(),
                    });
                    return None;
                }
                identity.push(token.span);
            }
        } else {
            let warning = self.take_nontrivia()?;
            if warning.kind != TokenKind::Identifier {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-DIAGNOSTIC-CONTROL",
                    span: Span::new(lang.span.start, warning.span.end),
                    message: "expected `lang diagnostic-operation warning-name`".into(),
                });
                return None;
            }
            identity.push(warning.span);
            warning.span.end
        };
        if operation.kind != TokenKind::Identifier {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-DIAGNOSTIC-CONTROL",
                span: operation.span,
                message: "expected a diagnostic-control operation after `lang`".into(),
            });
            return None;
        }
        Some(Statement::DiagnosticControl {
            operation: kind,
            identity,
            span: Span::new(lang.span.start, end),
        })
    }

    fn foreach_separator_index(&self) -> Option<usize> {
        self.tokens[self.cursor..]
            .iter()
            .position(|token| {
                token.kind == TokenKind::Identifier && self.source.slice(token.span) == "foreach"
            })
            .map(|offset| self.cursor + offset)
            .filter(|index| {
                !self.tokens[self.cursor..*index]
                    .iter()
                    .any(|token| token.kind == TokenKind::Newline)
            })
    }

    fn indented_body_after_current_line(&self) -> Option<(usize, usize)> {
        let newline = self.tokens.get(self.cursor)?;
        if newline.kind != TokenKind::Newline {
            return None;
        }
        let body_start = self.cursor + 1;
        let indent = self.tokens.get(body_start)?;
        let width = indent.span.end - indent.span.start;
        (indent.kind == TokenKind::Whitespace && width > self.current_indent)
            .then_some((width, body_start))
    }

    fn foreach_statement(&mut self) -> Option<Statement> {
        let separator_index = self.foreach_separator_index()?;
        let mut source_start = self.cursor;
        let mut result = None;
        let nontrivia = self.tokens[self.cursor..separator_index]
            .iter()
            .enumerate()
            .filter(|(_, token)| !token.kind.is_trivia())
            .collect::<Vec<_>>();
        if let Some((_, first)) = nontrivia.first()
            && first.kind == TokenKind::Identifier
            && let Some((is_position, (is_offset, _))) =
                nontrivia.iter().enumerate().find(|(_, (_, token))| {
                    token.kind == TokenKind::Identifier && self.source.slice(token.span) == "is"
                })
        {
            let classifier = if is_position == 1 {
                None
            } else if is_position >= 3 && nontrivia[1].1.kind == TokenKind::Colon {
                Some(Span::new(
                    nontrivia[2].1.span.start,
                    nontrivia[is_position - 1].1.span.end,
                ))
            } else {
                return None;
            };
            result = Some((first.span, classifier));
            source_start = self.cursor + is_offset + 1;
        }
        let mut source_parser = Self {
            source: self.source,
            tokens: &self.tokens[source_start..separator_index],
            cursor: 0,
            delimiter_depth: 0,
            current_indent: self.current_indent,
            diagnostics: Vec::new(),
        };
        let source_expression = source_parser.expression()?;
        self.diagnostics.extend(source_parser.diagnostics);
        self.cursor = separator_index + 1;
        let opening = self.take_nontrivia()?;
        let binding = self.take_nontrivia()?;
        let closing = self.take_nontrivia()?;
        if opening.kind != TokenKind::LeftBrace
            || binding.kind != TokenKind::Identifier
            || closing.kind != TokenKind::RightBrace
        {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-FOREACH-BINDING",
                span: Span::new(opening.span.start, closing.span.end),
                message: "expected `source foreach { value }`".into(),
            });
            return None;
        }
        let newline = self.peek()?;
        if newline.kind != TokenKind::Newline {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-FOREACH-BODY",
                span: closing.span,
                message: "expected an indented foreach body on the next line".into(),
            });
            return None;
        }
        self.cursor += 1;
        let Some(indent) = self.peek() else {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-FOREACH-BODY",
                span: closing.span,
                message: "expected an indented foreach body on the next line".into(),
            });
            return None;
        };
        if indent.kind != TokenKind::Whitespace {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-INDENTED-BODY",
                span: indent.span,
                message: "foreach body must be indented".into(),
            });
            return None;
        }
        let body = self.indented_function_body(indent.span.end - indent.span.start)?;
        let end = statement_span(body.last().expect("foreach body is nonempty")).end;
        let start = result.map_or(source_expression.span().start, |(name, _)| name.start);
        Some(Statement::Foreach {
            span: Span::new(start, end),
            result,
            source: source_expression,
            binding: binding.span,
            body,
        })
    }

    #[allow(clippy::too_many_lines)] // Header continuations and body diagnostics remain localized.
    fn function(&mut self, name: Token) -> Option<Statement> {
        let function = self.take_nontrivia()?;
        let next = self.take_nontrivia()?;
        let is_static =
            next.kind == TokenKind::Identifier && self.source.slice(next.span) == "static";
        let opening = if is_static {
            self.take_nontrivia()?
        } else {
            next
        };
        let (parameters, closing) = self.static_function_parameters(opening)?;
        if parameters.len() > 2 {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-FUNCTION-OPERAND-COUNT",
                span: Span::new(opening.span.start, closing.span.end),
                message: "a function has at most two syntactic operands; package additional values explicitly".into(),
            });
            self.skip_to_newline();
            return None;
        }
        let mut clauses = FunctionClauses::default();
        let mut clause_order = 0_u8;
        let arrow = loop {
            let token = self.take_function_header_token()?;
            if token.kind == TokenKind::Arrow {
                break token;
            }
            let clause = self.source.slice(token.span);
            let (order, slot) = match clause {
                "requires" => (1, &mut clauses.requires),
                "effects" => (2, &mut clauses.effects),
                "guarantees" => (3, &mut clauses.guarantees),
                _ => {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-FUNCTION-CLAUSE-PLACEMENT",
                        span: token.span,
                        message: "expected `requires`, `effects`, `guarantees`, or `->` after the parameter list".into(),
                    });
                    self.skip_to_newline();
                    return None;
                }
            };
            if order <= clause_order || slot.is_some() {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-FUNCTION-CLAUSE-ORDER",
                    span: token.span,
                    message:
                        "function clauses occur once in `requires`, `effects`, `guarantees` order"
                            .into(),
                });
                self.skip_to_newline();
                return None;
            }
            clause_order = order;
            let Some(expression) =
                self.function_clause_expression(&["requires", "effects", "guarantees"])
            else {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-FUNCTION-CLAUSE-EXPRESSION",
                    span: token.span,
                    message: "function clause requires an expression on the same logical line"
                        .into(),
                });
                return None;
            };
            *slot = Some(Box::new(expression));
        };
        let result_token = self.take_nontrivia()?;
        let first_result = self.function_result(result_token)?;
        let (result, effect_bound) = if self
            .peek_nontrivia()
            .is_some_and(|token| token.kind == TokenKind::Colon)
            && self
                .source
                .slice(result_token.span)
                .chars()
                .next()
                .is_some_and(char::is_lowercase)
        {
            self.take_nontrivia();
            let classifier_start = self.take_nontrivia()?;
            let result = self.function_result(classifier_start)?;
            clauses.result_binding = Some(result_token.span);
            (result, None)
        } else {
            let effect_bound = if self
                .peek_nontrivia()
                .is_some_and(|token| token.kind == TokenKind::Colon)
            {
                self.function_effect_bound()
            } else if self.effect_bound_on_following_line() {
                self.cursor += 1;
                self.function_effect_bound()
            } else {
                None
            };
            (first_result, effect_bound)
        };
        let valid = self.source.slice(function.span) == "fn"
            && opening.kind == TokenKind::LeftParen
            && closing.kind == TokenKind::RightParen
            && arrow.kind == TokenKind::Arrow
            && matches!(
                result_token.kind,
                TokenKind::Identifier | TokenKind::LeftParen
            );
        if !valid {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-UNSUPPORTED-FUNCTION-HEADER",
                span: Span::new(function.span.start, result.end),
                message: "the implemented function subset requires `fn static ( name : Type, ... ) -> ResultType`".into(),
            });
            self.skip_to_newline();
            return None;
        }
        let checkpoint = self.cursor;
        if let Some(token) = self.take_function_header_token() {
            if token.kind == TokenKind::Identifier && self.source.slice(token.span) == "ensures" {
                if clauses.result_binding.is_none() {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-FUNCTION-RESULT-BINDING",
                        span: token.span,
                        message: "`ensures` requires `-> result : ResultClassifier`".into(),
                    });
                    self.skip_to_newline();
                    return None;
                }
                clauses.ensures = self.function_clause_expression(&[]).map(Box::new);
                if clauses.ensures.is_none() {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-FUNCTION-CLAUSE-EXPRESSION",
                        span: token.span,
                        message: "`ensures` requires a relation expression".into(),
                    });
                    return None;
                }
            } else {
                self.cursor = checkpoint;
            }
        }
        if !self
            .peek()
            .is_some_and(|token| token.kind == TokenKind::Newline)
        {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-FUNCTION-BODY",
                span: Span::new(result.end, result.end),
                message: "expected an indented function body on the next line".into(),
            });
            return None;
        }
        self.cursor += 1;
        let Some(indent) = self.peek() else {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-FUNCTION-BODY",
                span: Span::new(result.end, result.end),
                message: "expected an indented function body on the next line".into(),
            });
            return None;
        };
        if indent.kind != TokenKind::Whitespace {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-INDENTED-BODY",
                span: indent.span,
                message: "function body must be indented".into(),
            });
            return None;
        }
        let body = self.indented_function_body(indent.span.end - indent.span.start)?;
        let body_end = statement_span(body.last().expect("function body is nonempty")).end;
        if self.source.slice(result) != "Unit"
            && matches!(
                body.last(),
                Some(Statement::Binding { .. } | Statement::Discard { .. })
            )
            && self.tokens[self.cursor..]
                .iter()
                .all(|token| token.kind.is_trivia())
        {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-FUNCTION-BODY",
                span: Span::new(body_end, body_end),
                message: "expected a final expression producing the function result".into(),
            });
            return None;
        }
        Some(Statement::Function {
            name: name.span,
            is_static,
            parameters,
            result,
            effect_bound,
            clauses: Box::new(clauses),
            span: Span::new(name.span.start, body_end),
            body,
        })
    }

    fn function_clause_expression(&mut self, following_clauses: &[&str]) -> Option<Expression> {
        let start = self.cursor;
        let mut depth = 0_usize;
        let end = self.tokens[start..]
            .iter()
            .position(|token| {
                match token.kind {
                    TokenKind::LeftParen | TokenKind::LeftBrace | TokenKind::LeftBracket => {
                        depth += 1;
                    }
                    TokenKind::RightParen | TokenKind::RightBrace | TokenKind::RightBracket => {
                        depth = depth.saturating_sub(1);
                    }
                    _ => {}
                }
                depth == 0
                    && (token.kind == TokenKind::Newline
                        || token.kind == TokenKind::Arrow
                        || (token.kind == TokenKind::Identifier
                            && following_clauses.contains(&self.source.slice(token.span))))
            })
            .map_or(self.tokens.len(), |offset| start + offset);
        let mut parser = Self {
            source: self.source,
            tokens: &self.tokens[start..end],
            cursor: 0,
            delimiter_depth: 0,
            current_indent: self.current_indent,
            diagnostics: Vec::new(),
        };
        let expression = parser.expression();
        self.diagnostics.extend(parser.diagnostics);
        self.cursor = end;
        expression
    }

    fn take_function_header_token(&mut self) -> Option<Token> {
        if let Some(token) = self.take_nontrivia() {
            return Some(token);
        }
        let checkpoint = self.cursor;
        if !self
            .peek()
            .is_some_and(|token| token.kind == TokenKind::Newline)
        {
            return None;
        }
        self.cursor += 1;
        let token = self.take_nontrivia();
        if token.is_none() {
            self.cursor = checkpoint;
        }
        token
    }

    fn effect_bound_on_following_line(&self) -> bool {
        if !self
            .tokens
            .get(self.cursor)
            .is_some_and(|token| token.kind == TokenKind::Newline)
        {
            return false;
        }
        self.tokens[self.cursor + 1..]
            .iter()
            .take_while(|token| token.kind != TokenKind::Newline)
            .find(|token| !token.kind.is_trivia())
            .is_some_and(|token| token.kind == TokenKind::Colon)
    }

    fn function_effect_bound(&mut self) -> Option<Span> {
        let colon = self.take_nontrivia()?;
        debug_assert_eq!(colon.kind, TokenKind::Colon);
        let first = self.take_nontrivia();
        let Some(first) = first else {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-EFFECT-BOUND",
                span: colon.span,
                message: "expected an effect or resource bound after `:`".into(),
            });
            return None;
        };
        let mut end = first.span.end;
        while let Some(token) = self.take_nontrivia() {
            end = token.span.end;
        }
        Some(Span::new(first.span.start, end))
    }

    fn union(&mut self, name: Token) -> Option<Statement> {
        let union = self.take_nontrivia()?;
        let newline = self.peek()?;
        if newline.kind != TokenKind::Newline {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-UNION-ALTERNATIVES",
                span: union.span,
                message: "Union alternatives must begin on following indented lines".into(),
            });
            return None;
        }
        self.cursor += 1;
        let mut alternatives = Vec::new();
        let mut end = union.span.end;
        while let Some(indent) = self.peek() {
            if indent.kind != TokenKind::Whitespace {
                break;
            }
            self.cursor += 1;
            let alternative = self.take_nontrivia()?;
            if alternative.kind != TokenKind::Identifier {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-EXPECTED-UNION-ALTERNATIVE",
                    span: alternative.span,
                    message: "expected a Union alternative name".into(),
                });
                return None;
            }
            let classifier = if self
                .peek_nontrivia()
                .is_some_and(|token| token.kind == TokenKind::Colon)
            {
                self.take_nontrivia();
                let first = self.take_nontrivia()?;
                Some(self.classifier_from_first(first)?)
            } else {
                None
            };
            end = classifier.map_or(alternative.span.end, |span| span.end);
            alternatives.push(UnionAlternative {
                name: alternative.span,
                classifier,
            });
            if self
                .peek()
                .is_some_and(|token| token.kind == TokenKind::Newline)
            {
                self.cursor += 1;
            } else {
                break;
            }
        }
        if alternatives.is_empty() {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EMPTY-UNION",
                span: union.span,
                message: "a Union requires at least one alternative".into(),
            });
            return None;
        }
        Some(Statement::Union {
            name: name.span,
            alternatives,
            span: Span::new(name.span.start, end),
        })
    }

    #[allow(clippy::too_many_lines)] // Interface clauses mirror complete function headers.
    fn interface(&mut self, name: Token) -> Option<Statement> {
        let interface = self.take_nontrivia()?;
        let newline = self.peek()?;
        if self.source.slice(interface.span) != "Interface" || newline.kind != TokenKind::Newline {
            self.error_current(
                "E-EXPECTED-INTERFACE-OPERATIONS",
                "interface operations begin on following indented lines",
            );
            return None;
        }
        self.cursor += 1;
        let Some(indent) = self.peek() else {
            self.error_current("E-EMPTY-INTERFACE", "an interface requires an operation");
            return None;
        };
        if indent.kind != TokenKind::Whitespace {
            self.error_current(
                "E-EXPECTED-INTERFACE-OPERATIONS",
                "interface operations must be indented",
            );
            return None;
        }
        let indentation = indent.span.end - indent.span.start;
        let mut functions = Vec::new();
        while let Some(current) = self.peek() {
            if current.kind != TokenKind::Whitespace
                || current.span.end - current.span.start != indentation
            {
                break;
            }
            self.cursor += 1;
            let operation = self.take_nontrivia()?;
            let separator = self.take_nontrivia()?;
            let function = self.take_nontrivia()?;
            let opening = self.take_nontrivia()?;
            let (parameters, closing) = self.static_function_parameters(opening)?;
            let mut clauses = FunctionClauses::default();
            let mut clause_order = 0_u8;
            let arrow = loop {
                let token = self.take_function_header_token()?;
                if token.kind == TokenKind::Arrow {
                    break token;
                }
                let clause = self.source.slice(token.span);
                let (order, slot) = match clause {
                    "requires" => (1, &mut clauses.requires),
                    "effects" => (2, &mut clauses.effects),
                    "guarantees" => (3, &mut clauses.guarantees),
                    _ => {
                        self.diagnostics.push(SyntaxDiagnostic {
                            code: "E-FUNCTION-CLAUSE-PLACEMENT",
                            span: token.span,
                            message: "expected `requires`, `effects`, `guarantees`, or `->` after the interface-operation parameter list".into(),
                        });
                        self.skip_to_newline();
                        return None;
                    }
                };
                if order <= clause_order || slot.is_some() {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-FUNCTION-CLAUSE-ORDER",
                        span: token.span,
                        message:
                            "function clauses occur once in `requires`, `effects`, `guarantees` order"
                                .into(),
                    });
                    self.skip_to_newline();
                    return None;
                }
                clause_order = order;
                let Some(expression) =
                    self.function_clause_expression(&["requires", "effects", "guarantees"])
                else {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-FUNCTION-CLAUSE-EXPRESSION",
                        span: token.span,
                        message: "function clause requires an expression on the same logical line"
                            .into(),
                    });
                    return None;
                };
                *slot = Some(Box::new(expression));
            };
            let result_token = self.take_nontrivia()?;
            let first_result = self.function_result(result_token)?;
            let result = if self
                .peek_nontrivia()
                .is_some_and(|token| token.kind == TokenKind::Colon)
                && self
                    .source
                    .slice(result_token.span)
                    .chars()
                    .next()
                    .is_some_and(char::is_lowercase)
            {
                self.take_nontrivia();
                let classifier_start = self.take_nontrivia()?;
                clauses.result_binding = Some(result_token.span);
                self.function_result(classifier_start)?
            } else {
                first_result
            };
            let checkpoint = self.cursor;
            if let Some(token) = self.take_function_header_token() {
                if token.kind == TokenKind::Identifier && self.source.slice(token.span) == "ensures"
                {
                    if clauses.result_binding.is_none() {
                        self.diagnostics.push(SyntaxDiagnostic {
                            code: "E-FUNCTION-RESULT-BINDING",
                            span: token.span,
                            message: "`ensures` requires `-> result : ResultClassifier`".into(),
                        });
                        self.skip_to_newline();
                        return None;
                    }
                    clauses.ensures = self.function_clause_expression(&[]).map(Box::new);
                    if clauses.ensures.is_none() {
                        self.diagnostics.push(SyntaxDiagnostic {
                            code: "E-FUNCTION-CLAUSE-EXPRESSION",
                            span: token.span,
                            message: "`ensures` requires a relation expression".into(),
                        });
                        return None;
                    }
                } else {
                    self.cursor = checkpoint;
                }
            }
            let end = clauses
                .ensures
                .as_deref()
                .map_or(result.end, |relation| relation.span().end);
            if operation.kind != TokenKind::Identifier
                || self.source.slice(separator.span) != "is"
                || self.source.slice(function.span) != "fn"
                || closing.kind != TokenKind::RightParen
                || arrow.kind != TokenKind::Arrow
            {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-INTERFACE-OPERATION",
                    span: Span::new(operation.span.start, end),
                    message: "expected `name is fn (inputs) -> Result`".into(),
                });
                return None;
            }
            functions.push(InterfaceFunction {
                name: operation.span,
                parameters,
                result,
                clauses,
                span: Span::new(operation.span.start, end),
            });
            let checkpoint = self.cursor;
            if self
                .peek()
                .is_some_and(|token| token.kind == TokenKind::Newline)
            {
                self.cursor += 1;
                if !self.peek().is_some_and(|token| {
                    token.kind == TokenKind::Whitespace
                        && token.span.end - token.span.start == indentation
                }) {
                    self.cursor = checkpoint;
                    break;
                }
            }
        }
        let Some(last) = functions.last() else {
            self.error_current("E-EMPTY-INTERFACE", "an interface requires an operation");
            return None;
        };
        Some(Statement::Interface {
            name: name.span,
            span: Span::new(name.span.start, last.span.end),
            functions,
        })
    }

    fn generator(&mut self, name: Token) -> Option<Statement> {
        let keyword = self.take_nontrivia()?;
        let opening = self.take_nontrivia()?;
        let (parameters, closing) = self.static_function_parameters(opening)?;
        if parameters.is_empty() {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-GENERATOR-OPERAND-COUNT",
                span: Span::new(opening.span.start, closing.span.end),
                message: "a generator requires at least one initial operand".into(),
            });
            return None;
        }
        if self.source.slice(keyword.span) != "generator" || opening.kind != TokenKind::LeftParen {
            return None;
        }
        let yielded = self.generator_header_clause("yields")?;
        let resumed = self.generator_header_clause("resumes")?;
        let result = self.generator_result_clause()?;
        while self
            .peek()
            .is_some_and(|token| token.kind == TokenKind::Newline)
        {
            self.cursor += 1;
        }
        let Some(indent) = self.peek() else {
            self.error_current(
                "E-EXPECTED-GENERATOR-BODY",
                "expected an indented generator body",
            );
            return None;
        };
        if indent.kind != TokenKind::Whitespace {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-INDENTED-BODY",
                span: indent.span,
                message: "generator body must be indented".into(),
            });
            return None;
        }
        let body = self.indented_function_body(indent.span.end - indent.span.start)?;
        let body_end = statement_span(body.last().expect("generator body is nonempty")).end;
        Some(Statement::Generator {
            name: name.span,
            parameters,
            yielded,
            resumed,
            result,
            body,
            span: Span::new(name.span.start, body_end),
        })
    }

    fn generator_header_clause(&mut self, expected: &'static str) -> Option<Span> {
        self.expect_generator_header_newline()?;
        let keyword = self.take_nontrivia()?;
        let classifier = self.generator_classifier()?;
        if keyword.kind != TokenKind::Identifier || self.source.slice(keyword.span) != expected {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-UNSUPPORTED-GENERATOR-HEADER",
                span: Span::new(keyword.span.start, classifier.end),
                message: format!("expected `{expected} Type` in the generator header"),
            });
            return None;
        }
        Some(classifier)
    }

    fn generator_result_clause(&mut self) -> Option<Span> {
        self.expect_generator_header_newline()?;
        let arrow = self.take_nontrivia()?;
        let classifier = self.generator_classifier()?;
        if arrow.kind != TokenKind::Arrow {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-UNSUPPORTED-GENERATOR-HEADER",
                span: Span::new(arrow.span.start, classifier.end),
                message: "expected `-> Type` in the generator header".into(),
            });
            return None;
        }
        Some(classifier)
    }

    fn generator_classifier(&mut self) -> Option<Span> {
        let first = self.take_nontrivia()?;
        self.classifier_from_first(first)
    }

    fn classifier_from_first(&mut self, first: Token) -> Option<Span> {
        if first.kind == TokenKind::LeftParen {
            let mut depth = 1_usize;
            let mut end = first.span.end;
            while depth > 0 {
                let token = self.take_nontrivia()?;
                match token.kind {
                    TokenKind::LeftParen => depth += 1,
                    TokenKind::RightParen => depth -= 1,
                    _ => {}
                }
                end = token.span.end;
            }
            return Some(Span::new(first.span.start, end));
        }
        if first.kind != TokenKind::Identifier {
            return None;
        }
        if self.source.slice(first.span) == "fn" {
            if self.peek_nontrivia().is_some_and(|token| {
                token.kind == TokenKind::Identifier && self.source.slice(token.span) == "static"
            }) {
                self.take_nontrivia();
            }
            let opening = self.take_nontrivia()?;
            if opening.kind != TokenKind::LeftParen {
                return None;
            }
            let mut depth = 1_usize;
            let mut closing = opening;
            while depth > 0 {
                let token = self.take_nontrivia()?;
                match token.kind {
                    TokenKind::LeftParen => depth += 1,
                    TokenKind::RightParen => depth -= 1,
                    _ => {}
                }
                closing = token;
            }
            let arrow = self.take_nontrivia()?;
            if arrow.kind != TokenKind::Arrow {
                return None;
            }
            let result_start = self.take_nontrivia()?;
            let result = self.classifier_from_first(result_start)?;
            return Some(Span::new(
                first.span.start,
                result.end.max(closing.span.end),
            ));
        }
        if matches!(
            self.source.slice(first.span),
            "Array" | "Map" | "Result" | "Record"
        ) {
            if !self
                .peek_nontrivia()
                .is_some_and(|token| token.kind == TokenKind::LeftParen)
            {
                return Some(first.span);
            }
            let opening = self.take_nontrivia()?;
            if opening.kind != TokenKind::LeftParen {
                return None;
            }
            let mut depth = 1_usize;
            let mut end = opening.span.end;
            while depth > 0 {
                let token = self.take_nontrivia()?;
                match token.kind {
                    TokenKind::LeftParen => depth += 1,
                    TokenKind::RightParen => depth -= 1,
                    _ => {}
                }
                end = token.span.end;
            }
            return Some(Span::new(first.span.start, end));
        }
        if matches!(
            self.source.slice(first.span),
            "Bag" | "List" | "Optional" | "Range" | "Set"
        ) {
            let payload = self.generator_classifier()?;
            return Some(Span::new(first.span.start, payload.end));
        }
        if self.source.slice(first.span) == "Generator" {
            let _yielded = self.generator_classifier()?;
            let _resumed = self.generator_classifier()?;
            let returned = self.generator_classifier()?;
            return Some(Span::new(first.span.start, returned.end));
        }
        Some(first.span)
    }

    fn expect_generator_header_newline(&mut self) -> Option<()> {
        if !self
            .peek()
            .is_some_and(|token| token.kind == TokenKind::Newline)
        {
            self.error_current(
                "E-UNSUPPORTED-GENERATOR-HEADER",
                "generator header clauses must start on separate indented lines",
            );
            return None;
        }
        self.cursor += 1;
        if !self
            .peek()
            .is_some_and(|token| token.kind == TokenKind::Whitespace)
        {
            self.error_current(
                "E-EXPECTED-INDENTED-GENERATOR-HEADER",
                "generator header clauses must be indented",
            );
            return None;
        }
        self.cursor += 1;
        Some(())
    }

    fn function_result(&mut self, first: Token) -> Option<Span> {
        self.classifier_from_first(first)
    }

    fn indented_function_body(&mut self, body_indent: usize) -> Option<Vec<Statement>> {
        self.cursor += 1;
        let outer_indent = self.current_indent;
        self.current_indent = body_indent;
        let first = self.statement();
        self.current_indent = outer_indent;
        let mut body = vec![first?];
        loop {
            if !self
                .peek()
                .is_some_and(|token| token.kind == TokenKind::Newline)
            {
                break;
            }
            let checkpoint = self.cursor;
            self.cursor += 1;
            let Some(indent) = self
                .peek()
                .filter(|token| token.kind == TokenKind::Whitespace)
            else {
                self.cursor = checkpoint;
                break;
            };
            let indent_width = indent.span.end - indent.span.start;
            if indent_width > body_indent {
                self.cursor = checkpoint;
                let Some(Statement::Expression(subject)) = body.pop() else {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-DECISION-SUBJECT",
                        span: indent.span,
                        message: "a decision table must follow a subject expression".into(),
                    });
                    return None;
                };
                body.push(Statement::Expression(
                    self.decision_table(subject, indent_width)?,
                ));
                continue;
            }
            if indent_width < body_indent {
                self.cursor = checkpoint;
                break;
            }
            self.cursor += 1;
            self.current_indent = body_indent;
            let statement = self.statement();
            self.current_indent = outer_indent;
            body.push(statement?);
        }
        Some(body)
    }

    #[allow(clippy::too_many_lines)] // Completeness and reachability checks stay adjacent to parsing.
    fn decision_table(&mut self, subject: Expression, rule_indent: usize) -> Option<Expression> {
        let mut rules = Vec::new();
        while self
            .tokens
            .get(self.cursor)
            .is_some_and(|token| token.kind == TokenKind::Newline)
            && self.tokens.get(self.cursor + 1).is_some_and(|token| {
                token.kind == TokenKind::Whitespace
                    && token.span.end - token.span.start == rule_indent
            })
        {
            self.cursor += 2;
            rules.push(self.decision_rule()?);
        }
        if let Some(span) = Self::rule_after_otherwise(&rules) {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-UNREACHABLE-DECISION-RULE",
                span,
                message: "decision rule is unreachable after `otherwise`".into(),
            });
            return None;
        }
        if let Some((span, code)) = self.duplicate_arithmetic_code(&rules) {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-DUPLICATE-ERROR-CODE-PATTERN",
                span,
                message: format!("arithmetic error code `{code}` is matched more than once"),
            });
            return None;
        }
        if let Some(span) = Self::error_code_after_fallback(&rules) {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-UNREACHABLE-ERROR-CODE-PATTERN",
                span,
                message: "qualified Error-code pattern is unreachable after `Error problem`".into(),
            });
            return None;
        }
        let complete = rules
            .iter()
            .any(|rule| matches!(&rule.matcher, DecisionMatcher::Otherwise(_)))
            || [false, true].into_iter().all(|value| {
                rules.iter().any(|rule| {
                    matches!(&rule.matcher, DecisionMatcher::Boolean { value: found, .. } if *found == value)
                })
            })
            || rules
                .iter()
                .all(|rule| {
                    matches!(
                        rule.matcher,
                        DecisionMatcher::Identifier(_)
                            | DecisionMatcher::Union { .. }
                            | DecisionMatcher::Variant { .. }
                    )
                })
            || [false, true].into_iter().all(|error| {
                rules.iter().any(|rule| {
                    matches!(rule.matcher, DecisionMatcher::Result { error: found, .. } if found == error)
                })
            })
            || [false, true].into_iter().all(|some| {
                rules.iter().any(|rule| {
                    matches!(rule.matcher, DecisionMatcher::Optional { some: found, .. } if found == some)
                })
            })
            || [false, true].into_iter().all(|entry| {
                rules.iter().any(|rule| {
                    matches!(rule.matcher, DecisionMatcher::ListEntry { .. } if entry)
                        || matches!(rule.matcher, DecisionMatcher::ListEmpty(_) if !entry)
                })
            })
            || self.complete_arithmetic_result(&rules);
        if !complete {
            let end = rules
                .last()
                .map_or(subject.span().end, |rule| rule.span.end);
            let missing_codes = self.missing_arithmetic_codes(&rules);
            let (code, message) = if missing_codes.len() < 4 {
                (
                    "E-INCOMPLETE-ERROR-CODE-DECISION",
                    format!(
                        "decision is missing arithmetic error code alternatives: {}",
                        missing_codes.join(", ")
                    ),
                )
            } else {
                (
                    "E-UNSUPPORTED-INCOMPLETE-DECISION",
                    "the implemented decision subset requires complete Boolean cases or `otherwise`"
                        .to_owned(),
                )
            };
            self.diagnostics.push(SyntaxDiagnostic {
                code,
                span: Span::new(subject.span().start, end),
                message,
            });
            return None;
        }
        let end = rules.last().expect("complete decision has rules").span.end;
        Some(Expression::DecisionTable {
            span: Span::new(subject.span().start, end),
            subject: Box::new(subject),
            rules,
        })
    }

    fn complete_arithmetic_result(&self, rules: &[DecisionRule]) -> bool {
        let has_ok = rules
            .iter()
            .any(|rule| matches!(rule.matcher, DecisionMatcher::Result { error: false, .. }));
        let codes = rules
            .iter()
            .filter_map(|rule| match rule.matcher {
                DecisionMatcher::ErrorCode {
                    namespace,
                    vocabulary,
                    code,
                    ..
                } if self.source.slice(namespace) == "lang"
                    && self.source.slice(vocabulary) == "arithmetic" =>
                {
                    Some(self.source.slice(code))
                }
                _ => None,
            })
            .collect::<std::collections::BTreeSet<_>>();
        has_ok
            && codes
                == [
                    "out-of-range",
                    "not-representable",
                    "division-by-zero",
                    "indeterminate",
                ]
                .into_iter()
                .collect()
    }

    fn missing_arithmetic_codes(&self, rules: &[DecisionRule]) -> Vec<&'static str> {
        let present = rules
            .iter()
            .filter_map(|rule| match rule.matcher {
                DecisionMatcher::ErrorCode {
                    namespace,
                    vocabulary,
                    code,
                    ..
                } if self.source.slice(namespace) == "lang"
                    && self.source.slice(vocabulary) == "arithmetic" =>
                {
                    Some(self.source.slice(code))
                }
                _ => None,
            })
            .collect::<std::collections::BTreeSet<_>>();
        [
            "out-of-range",
            "not-representable",
            "division-by-zero",
            "indeterminate",
        ]
        .into_iter()
        .filter(|code| !present.contains(code))
        .collect()
    }
}
