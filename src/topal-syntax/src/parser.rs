use topal_source::{SourceText, Span};

use crate::{Lexed, SyntaxDiagnostic, Token, TokenKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    Integer(Span),
    Rational(Span),
    Identifier(Span),
    Callable { kind: CallableKind, span: Span },
    Application { items: Vec<Self>, span: Span },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallableKind {
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
            Self::Integer(span)
            | Self::Rational(span)
            | Self::Identifier(span)
            | Self::Callable { span, .. }
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
    diagnostics: Vec<SyntaxDiagnostic>,
}

impl Parser<'_> {
    fn statement(&mut self) -> Option<Statement> {
        let checkpoint = self.cursor;
        let first = self.take_nontrivia()?;
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
            TokenKind::Integer => Some(Expression::Integer(token.span)),
            TokenKind::Rational => Some(Expression::Rational(token.span)),
            TokenKind::Identifier => Some(Expression::Identifier(token.span)),
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
                let expression = self.expression();
                let closing = self.take_nontrivia();
                if !closing.is_some_and(|value| value.kind == TokenKind::RightParen) {
                    self.diagnostics.push(SyntaxDiagnostic {
                        code: "E-EXPECTED-RPAREN",
                        span: Span::new(token.span.end, token.span.end),
                        message: "expected closing parenthesis",
                    });
                }
                expression
            }
            _ => {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-EXPECTED-EXPRESSION",
                    span: token.span,
                    message: "expected an integer, name, or parenthesized expression",
                });
                None
            }
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
            if token.kind == TokenKind::Newline {
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
            if token.kind == TokenKind::Newline {
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
}
