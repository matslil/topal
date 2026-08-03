use topal_source::{SourceText, Span};

use crate::{Lexed, SyntaxDiagnostic, Token, TokenKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expression {
    Integer(Span),
    Identifier(Span),
    Add {
        left: Box<Self>,
        right: Box<Self>,
        span: Span,
    },
}

impl Expression {
    #[must_use]
    pub const fn span(&self) -> Span {
        match self {
            Self::Integer(span) | Self::Identifier(span) | Self::Add { span, .. } => *span,
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
        let mut left = self.primary()?;
        loop {
            let checkpoint = self.cursor;
            let Some(operator) = self.take_nontrivia() else {
                break;
            };
            if operator.kind != TokenKind::Plus {
                self.cursor = checkpoint;
                break;
            }
            let Some(right) = self.primary() else {
                self.diagnostics.push(SyntaxDiagnostic {
                    code: "E-EXPECTED-OPERAND",
                    span: Span::new(operator.span.end, operator.span.end),
                    message: "expected an operand after +",
                });
                break;
            };
            let span = Span::new(left.span().start, right.span().end);
            left = Expression::Add {
                left: Box::new(left),
                right: Box::new(right),
                span,
            };
        }
        Some(left)
    }

    fn primary(&mut self) -> Option<Expression> {
        let token = self.take_nontrivia()?;
        match token.kind {
            TokenKind::Integer => Some(Expression::Integer(token.span)),
            TokenKind::Identifier => Some(Expression::Identifier(token.span)),
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
    fn addition_associates_left() {
        let source = SourceText::new("1 + 2 + 3").unwrap();
        let parsed = parse(&source, &lex(&source));
        let Statement::Expression(Expression::Add { left, .. }) = &parsed.statements[0] else {
            panic!("expected addition");
        };
        assert!(matches!(left.as_ref(), Expression::Add { .. }));
        assert!(parsed.diagnostics.is_empty());
    }

    #[test]
    fn recovers_incomplete_addition() {
        let source = SourceText::new("1 +").unwrap();
        let parsed = parse(&source, &lex(&source));
        assert_eq!(parsed.diagnostics[0].code, "E-EXPECTED-OPERAND");
    }

    #[test]
    fn recovers_incomplete_binding() {
        let source = SourceText::new("answer is").unwrap();
        let parsed = parse(&source, &lex(&source));
        assert_eq!(parsed.diagnostics[0].code, "E-EXPECTED-EXPRESSION");
    }
}
