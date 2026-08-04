use topal_source::{SourceText, Span};

use crate::{Lexed, SyntaxDiagnostic, Token, TokenKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    Unit(Span),
    Boolean(Span),
    Tuple { items: Vec<Self>, span: Span },
    Integer(Span),
    Rational(Span),
    String(Span),
    Identifier(Span),
    Callable { kind: CallableKind, span: Span },
    Application { items: Vec<Self>, span: Span },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallableKind {
    Equal,
    NotEqual,
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
            | Self::Callable { span, .. }
            | Self::Tuple { span, .. }
            | Self::Application { span, .. } => *span,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Statement {
    Binding { name: Span, value: Expression },
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
        if first.kind == TokenKind::Identifier
            && let Some(second) = self.peek_nontrivia()
            && second.kind == TokenKind::Identifier
            && self.source.slice(second.span) == "is"
        {
            let separator = self
                .take_nontrivia()
                .expect("peeked token remains available");
            if let Some(value) = self.expression() {
                return Some(Statement::Binding {
                    name: first.span,
                    value,
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
            TokenKind::Equals => Some(Expression::Callable {
                kind: CallableKind::Equal,
                span: token.span,
            }),
            TokenKind::NotEquals => Some(Expression::Callable {
                kind: CallableKind::NotEqual,
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
        let Some(first) = self.expression() else {
            self.diagnostics.push(SyntaxDiagnostic {
                code: "E-EXPECTED-RPAREN",
                span: Span::new(opening.span.end, opening.span.end),
                message: "expected expression and closing parenthesis",
            });
            return None;
        };
        let mut items = self
            .peek_nontrivia()
            .is_some_and(|value| value.kind == TokenKind::Comma)
            .then(|| vec![first.clone()]);
        if let Some(product_items) = &mut items {
            loop {
                self.take_nontrivia();
                if self
                    .peek_nontrivia()
                    .is_some_and(|value| value.kind == TokenKind::RightParen)
                {
                    break;
                }
                let Some(item) = self.expression() else {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-EXPECTED-RPAREN",
                        span: Span::new(opening.span.end, opening.span.end),
                        message: "expected product field or closing parenthesis",
                    });
                    return None;
                };
                product_items.push(item);
                if !self
                    .peek_nontrivia()
                    .is_some_and(|value| value.kind == TokenKind::Comma)
                {
                    break;
                }
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
        if let Some(items) = items {
            Some(Expression::Tuple {
                items,
                span: Span::new(
                    opening.span.start,
                    closing.map_or(first.span().end, |value| value.span.end),
                ),
            })
        } else {
            Some(first)
        }
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
        let Statement::Expression(Expression::Tuple { items, .. }) = &tuple.statements[0] else {
            panic!("expected tuple");
        };
        assert_eq!(items.len(), 2);
        assert!(tuple.diagnostics.is_empty());
    }

    #[test]
    fn ignores_newlines_inside_parentheses() {
        let source = SourceText::new("(\n1,\n2\n)").unwrap();
        let parsed = parse(&source, &lex(&source));
        let Statement::Expression(Expression::Tuple { items, .. }) = &parsed.statements[0] else {
            panic!("expected tuple");
        };
        assert_eq!(items.len(), 2);
        assert!(parsed.diagnostics.is_empty());
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
}
