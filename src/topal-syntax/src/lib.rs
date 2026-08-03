//! Lossless, recovery-friendly Topal tokenization shared by language tools.

use topal_source::{SourceText, Span};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenKind {
    Whitespace,
    Newline,
    Comment,
    Hashbang,
    Identifier,
    Integer,
    LeftParen,
    RightParen,
    Comma,
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    Unknown,
}

impl TokenKind {
    #[must_use]
    pub const fn is_trivia(self) -> bool {
        matches!(
            self,
            Self::Whitespace | Self::Newline | Self::Comment | Self::Hashbang
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyntaxDiagnostic {
    pub code: &'static str,
    pub span: Span,
    pub message: &'static str,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Lexed {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<SyntaxDiagnostic>,
}

#[must_use]
pub fn lex(source: &SourceText) -> Lexed {
    let text = source.as_str();
    let mut result = Lexed::default();
    let mut offset = 0;
    while offset < text.len() {
        let rest = &text[offset..];
        let (kind, length) = if offset == 0 && rest.starts_with("#!") {
            (TokenKind::Hashbang, rest.find('\n').unwrap_or(rest.len()))
        } else {
            next_token(rest)
        };
        let span = Span::new(offset, offset + length);
        result.tokens.push(Token { kind, span });
        if kind == TokenKind::Unknown {
            result.diagnostics.push(SyntaxDiagnostic {
                code: "E-UNKNOWN-TOKEN",
                span,
                message: "character does not begin a token in design-0",
            });
        }
        offset += length;
    }
    result
}

fn next_token(rest: &str) -> (TokenKind, usize) {
    let first = rest.chars().next().expect("nonempty source");
    match first {
        ' ' | '\t' => (
            TokenKind::Whitespace,
            take_while(rest, |c| matches!(c, ' ' | '\t')),
        ),
        '\n' => (TokenKind::Newline, 1),
        '#' => (TokenKind::Comment, rest.find('\n').unwrap_or(rest.len())),
        '(' => (TokenKind::LeftParen, 1),
        ')' => (TokenKind::RightParen, 1),
        ',' => (TokenKind::Comma, 1),
        '+' => (TokenKind::Plus, 1),
        '-' if rest[1..].starts_with(|c: char| c.is_ascii_digit()) => {
            (TokenKind::Integer, 1 + take_number(&rest[1..]))
        }
        '-' => (TokenKind::Minus, 1),
        '*' => (TokenKind::Star, 1),
        '/' => (TokenKind::Slash, 1),
        '^' => (TokenKind::Caret, 1),
        c if c.is_ascii_digit() => (TokenKind::Integer, take_number(rest)),
        c if unicode_ident::is_xid_start(c) => (
            TokenKind::Identifier,
            take_while(rest, |value| {
                unicode_ident::is_xid_continue(value) || value == '-'
            }),
        ),
        c => (TokenKind::Unknown, c.len_utf8()),
    }
}

fn take_number(text: &str) -> usize {
    take_while(text, |character| {
        character.is_ascii_alphanumeric() || character == '_'
    })
}

fn take_while(text: &str, predicate: impl Fn(char) -> bool) -> usize {
    text.char_indices()
        .find_map(|(offset, character)| (!predicate(character)).then_some(offset))
        .unwrap_or(text.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(text: &str) -> Vec<TokenKind> {
        lex(&SourceText::new(text).unwrap())
            .tokens
            .into_iter()
            .map(|token| token.kind)
            .collect()
    }

    #[test]
    fn preserves_trivia_and_symbolic_application() {
        assert_eq!(
            kinds("value + 2 # sum\n"),
            vec![
                TokenKind::Identifier,
                TokenKind::Whitespace,
                TokenKind::Plus,
                TokenKind::Whitespace,
                TokenKind::Integer,
                TokenKind::Whitespace,
                TokenKind::Comment,
                TokenKind::Newline,
            ]
        );
    }

    #[test]
    fn distinguishes_signed_literal_from_prefix_minus() {
        assert_eq!(
            kinds("-42 - 42"),
            vec![
                TokenKind::Integer,
                TokenKind::Whitespace,
                TokenKind::Minus,
                TokenKind::Whitespace,
                TokenKind::Integer,
            ]
        );
    }

    #[test]
    fn covers_every_source_byte() {
        let source = SourceText::new("#!/usr/bin/env topal\nα + ?").unwrap();
        let lexed = lex(&source);
        let reconstructed = lexed
            .tokens
            .iter()
            .map(|token| source.slice(token.span))
            .collect::<String>();
        assert_eq!(reconstructed, source.as_str());
        assert_eq!(lexed.diagnostics.len(), 1);
    }
}
