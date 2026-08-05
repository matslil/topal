use topal_source::{SourceText, Span};

use crate::{Lexed, SyntaxDiagnostic, Token, TokenKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    Unit(Span),
    Boolean(Span),
    Product {
        fields: Vec<ProductField>,
        span: Span,
    },
    Integer(Span),
    Rational(Span),
    String(Span),
    Identifier(Span),
    Discard(Span),
    Callable {
        kind: CallableKind,
        span: Span,
    },
    Application {
        items: Vec<Self>,
        span: Span,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallableKind {
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    Plus,
    Minus,
    Multiply,
    Divide,
    Power,
}

impl Expression {
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::Unit(span)
            | Self::Boolean(span)
            | Self::Integer(span)
            | Self::Rational(span)
            | Self::String(span)
            | Self::Identifier(span)
            | Self::Discard(span)
            | Self::Callable { span, .. }
            | Self::Product { span, .. }
            | Self::Application { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductField {
    pub label: Option<Span>,
    pub value: Expression,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FunctionParameter {
    pub name: Span,
    pub classifier: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Statement {
    Binding {
        name: Span,
        value: Expression,
    },
    StaticFunction {
        name: Span,
        parameter: Option<FunctionParameter>,
        result: Span,
        body: Expression,
        span: Span,
    },
    Discard {
        span: Span,
        value: Expression,
    },
    Expression(Expression),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ParsedSource {
    pub statements: Vec<Statement>,
    pub diagnostics: Vec<SyntaxDiagnostic>,
}

#[must_use]
pub fn parse(source: &SourceText, lexed: &Lexed) -> ParsedSource {
    let mut parser = Parser {
        source,
        tokens: &lexed.tokens,
        cursor: 0,
        delimiter_depth: 0,
        diagnostics: lexed.diagnostics.clone(),
    };
    let mut statements = Vec::new();
    while parser.skip_separators() {
        if let Some(statement) = parser.statement() {
            statements.push(statement);
        }
        if parser
            .peek()
            .is_some_and(|token| token.kind != TokenKind::Newline)
        {
            parser.error_current("E-UNSUPPORTED-SYNTAX", "unsupported token after expression");
            parser.skip_to_newline();
        }
    }
    ParsedSource {
        statements,
        diagnostics: parser.diagnostics,
    }
}

struct Parser<'a> {
    source: &'a SourceText,
    tokens: &'a [Token],
    cursor: usize,
    delimiter_depth: usize,
    diagnostics: Vec<SyntaxDiagnostic>,
}

impl Parser<'_> {
    fn statement(&mut self) -> Option<Statement> {
        let checkpoint = self.cursor;
        let first = self.take_nontrivia()?;
        if first.kind == TokenKind::Boolean
            && self.peek_nontrivia().is_some_and(|second| {
                second.kind == TokenKind::Identifier && self.source.slice(second.span) == "is"
            })
        {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-RESERVED-BOOLEAN-LITERAL",
                span: first.span,
                message: "a Boolean literal cannot introduce a binding",
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
                && self.peek_nontrivia().is_some_and(|token| {
                    token.kind == TokenKind::Identifier && self.source.slice(token.span) == "fn"
                })
            {
                return self.static_function(first);
            }
            if let Some(value) = self.expression() {
                return Some(if first.kind == TokenKind::Discard {
                    Statement::Discard {
                        span: first.span,
                        value,
                    }
                } else {
                    Statement::Binding {
                        name: first.span,
                        value,
                    }
                });
            }
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-EXPRESSION",
                span: Span::new(separator.span.end, separator.span.end),
                message: "expected a binding initializer",
            });
            return None;
        }
        self.cursor = checkpoint;
        self.expression().map(Statement::Expression)
    }

    fn static_function(&mut self, name: Token) -> Option<Statement> {
        let function = self.take_nontrivia()?;
        let static_token = self.take_nontrivia()?;
        let opening = self.take_nontrivia()?;
        let first_input = self.take_nontrivia()?;
        let (parameter, closing) = if first_input.kind == TokenKind::RightParen {
            (None, first_input)
        } else {
            let colon = self.take_nontrivia()?;
            let classifier = self.take_nontrivia()?;
            let closing = self.take_nontrivia()?;
            if first_input.kind != TokenKind::Identifier
                || colon.kind != TokenKind::Colon
                || classifier.kind != TokenKind::Identifier
                || closing.kind != TokenKind::RightParen
            {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-UNSUPPORTED-FUNCTION-HEADER",
                    span: Span::new(opening.span.start, closing.span.end),
                    message: "the implemented function subset accepts `()` or one `name : Type` parameter",
                });
                self.skip_to_newline();
                return None;
            }
            (
                Some(FunctionParameter {
                    name: first_input.span,
                    classifier: classifier.span,
                }),
                closing,
            )
        };
        let arrow = self.take_nontrivia()?;
        let result = self.take_nontrivia()?;
        let valid = self.source.slice(function.span) == "fn"
            && static_token.kind == TokenKind::Identifier
            && self.source.slice(static_token.span) == "static"
            && opening.kind == TokenKind::LeftParen
            && closing.kind == TokenKind::RightParen
            && arrow.kind == TokenKind::Arrow
            && result.kind == TokenKind::Identifier;
        if !valid {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-UNSUPPORTED-FUNCTION-HEADER",
                span: Span::new(function.span.start, result.span.end),
                message: "the implemented function subset requires `fn static ( [name : Type] ) -> ResultType`",
            });
            self.skip_to_newline();
            return None;
        }
        if !self
            .peek()
            .is_some_and(|token| token.kind == TokenKind::Newline)
        {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-FUNCTION-BODY",
                span: Span::new(result.span.end, result.span.end),
                message: "expected an indented function body on the next line",
            });
            return None;
        }
        self.cursor += 1;
        let Some(indent) = self.peek() else {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-FUNCTION-BODY",
                span: Span::new(result.span.end, result.span.end),
                message: "expected an indented function body on the next line",
            });
            return None;
        };
        if indent.kind != TokenKind::Whitespace {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-INDENTED-BODY",
                span: indent.span,
                message: "function body must be indented",
            });
            return None;
        }
        self.cursor += 1;
        let body = self.expression()?;
        Some(Statement::StaticFunction {
            name: name.span,
            parameter,
            result: result.span,
            span: Span::new(name.span.start, body.span().end),
            body,
        })
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

    fn primary(&mut self) -> Option<Expression> {
        let token = self.take_nontrivia()?;
        match token.kind {
            TokenKind::Boolean => Some(Expression::Boolean(token.span)),
            TokenKind::Integer => Some(Expression::Integer(token.span)),
            TokenKind::Rational => Some(Expression::Rational(token.span)),
            TokenKind::String => Some(Expression::String(token.span)),
            TokenKind::Identifier => Some(Expression::Identifier(token.span)),
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
            _ => {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-EXPECTED-EXPRESSION",
                    span: token.span,
                    message: "expected a literal, name, callable, or parenthesized expression",
                });
                None
            }
        }
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
                message: "expected expression and closing parenthesis",
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
                        message: "expected product field or closing parenthesis",
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
                message: "expected closing parenthesis",
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
                    message: "a product cannot mix positional and labeled fields",
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
            message,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lex;

    #[test]
    fn retains_generic_application_order() {
        let source = SourceText::new("1 + 2 * 3").unwrap();
        let parsed = parse(&source, &lex(&source));
        let Statement::Expression(Expression::Application { items, .. }) = &parsed.statements[0]
        else {
            panic!("expected application");
        };
        assert_eq!(items.len(), 5);
        assert!(matches!(
            items[1],
            Expression::Callable {
                kind: CallableKind::Plus,
                ..
            }
        ));
        assert!(matches!(
            items[3],
            Expression::Callable {
                kind: CallableKind::Multiply,
                ..
            }
        ));
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    fn parses_the_zero_field_product_as_unit() {
        let source = SourceText::new("()").unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty());
        assert_eq!(
            parsed.statements,
            vec![Statement::Expression(Expression::Unit(Span::new(0, 2)))]
        );
    }

    #[test]
    fn distinguishes_grouping_and_positional_products() {
        let grouped_source = SourceText::new("(1)").unwrap();
        let grouped = parse(&grouped_source, &lex(&grouped_source));
        assert!(matches!(
            grouped.statements[0],
            Statement::Expression(Expression::Integer(_))
        ));

        let tuple_source = SourceText::new("(1, 2,)").unwrap();
        let tuple = parse(&tuple_source, &lex(&tuple_source));
        let Statement::Expression(Expression::Product { fields, .. }) = &tuple.statements[0] else {
            panic!("expected tuple");
        };
        assert_eq!(fields.len(), 2);
        assert!(tuple.diagnostics.is_empty());
    }

    #[test]
    fn ignores_newlines_inside_parentheses() {
        let source = SourceText::new("(\n1,\n2\n)").unwrap();
        let parsed = parse(&source, &lex(&source));
        let Statement::Expression(Expression::Product { fields, .. }) = &parsed.statements[0]
        else {
            panic!("expected tuple");
        };
        assert_eq!(fields.len(), 2);
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    fn retains_labels_on_record_product_fields() {
        let source = SourceText::new("(name is \"Ada\", active is true)").unwrap();
        let parsed = parse(&source, &lex(&source));
        let Statement::Expression(Expression::Product { fields, .. }) = &parsed.statements[0]
        else {
            panic!("expected product");
        };
        assert_eq!(fields.len(), 2);
        assert_eq!(source.slice(fields[0].label.unwrap()), "name");
        assert_eq!(source.slice(fields[1].label.unwrap()), "active");
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    fn rejects_products_that_mix_positional_and_labeled_fields() {
        let source = SourceText::new("(1, name is \"Ada\")").unwrap();
        let parsed = parse(&source, &lex(&source));
        assert_eq!(parsed.diagnostics[0].code, "E-MIXED-PRODUCT-FIELDS");
        assert_eq!(source.slice(parsed.diagnostics[0].span), "name");
    }

    #[test]
    fn retains_incomplete_parentheses_for_recovery() {
        let source = SourceText::new("(\n1,").unwrap();
        let parsed = parse(&source, &lex(&source));
        assert_eq!(parsed.diagnostics[0].code, "E-EXPECTED-RPAREN");
    }

    #[test]
    fn retains_named_and_prefix_application_terms() {
        let named_source = SourceText::new("print value").unwrap();
        let named = parse(&named_source, &lex(&named_source));
        let Statement::Expression(Expression::Application { items, .. }) = &named.statements[0]
        else {
            panic!("expected named application");
        };
        assert_eq!(items.len(), 2);

        let prefix_source = SourceText::new("- 42").unwrap();
        let prefix = parse(&prefix_source, &lex(&prefix_source));
        let Statement::Expression(Expression::Application { items, .. }) = &prefix.statements[0]
        else {
            panic!("expected prefix application");
        };
        assert!(matches!(
            items[0],
            Expression::Callable {
                kind: CallableKind::Minus,
                ..
            }
        ));
    }

    #[test]
    fn retains_incomplete_application_for_semantic_recovery() {
        let source = SourceText::new("1 +").unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(matches!(
            parsed.statements[0],
            Statement::Expression(Expression::Application { .. })
        ));
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    fn recovers_incomplete_binding() {
        let source = SourceText::new("answer is").unwrap();
        let parsed = parse(&source, &lex(&source));
        assert_eq!(parsed.diagnostics[0].code, "E-EXPECTED-EXPRESSION");
    }

    #[test]
    fn rejects_boolean_literal_as_a_binding_name() {
        let source = SourceText::new("true is 1").unwrap();
        let parsed = parse(&source, &lex(&source));
        assert_eq!(parsed.diagnostics[0].code, "E-RESERVED-BOOLEAN-LITERAL");
    }

    #[test]
    fn parses_static_nullary_function_with_indented_body() {
        let source = SourceText::new("answer is fn static () -> Int\n  40 + 2\nanswer ()").unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty());
        let Statement::StaticFunction {
            name, result, body, ..
        } = &parsed.statements[0]
        else {
            panic!("expected static function declaration");
        };
        assert_eq!(source.slice(*name), "answer");
        assert_eq!(source.slice(*result), "Int");
        assert!(matches!(body, Expression::Application { .. }));
        assert!(matches!(parsed.statements[1], Statement::Expression(_)));
    }

    #[test]
    fn parses_one_typed_static_function_parameter() {
        let source = SourceText::new(
            "increment is fn static (input : Int) -> Int\n  input + 1\nincrement 41",
        )
        .unwrap();
        let parsed = parse(&source, &lex(&source));
        assert!(parsed.diagnostics.is_empty());
        let Statement::StaticFunction {
            parameter: Some(parameter),
            ..
        } = parsed.statements[0]
        else {
            panic!("expected one static parameter");
        };
        assert_eq!(source.slice(parameter.name), "input");
        assert_eq!(source.slice(parameter.classifier), "Int");
    }
}
