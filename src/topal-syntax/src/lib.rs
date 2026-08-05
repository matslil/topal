//! Lossless, recovery-friendly Topal tokenization shared by language tools.

use topal_source::{SourceText, Span, is_identifier_continue, is_identifier_start, is_nfc};

mod parser;
pub use parser::{CallableKind, Expression, ParsedSource, ProductField, Statement, parse};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenKind {
    Whitespace,
    Newline,
    Comment,
    Hashbang,
    Identifier,
    Discard,
    Boolean,
    Integer,
    Rational,
    String,
    LeftParen,
    RightParen,
    Comma,
    Equals,
    NotEquals,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
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
        if kind == TokenKind::String && !string_is_terminated(source.slice(span)) {
            result.diagnostics.push(SyntaxDiagnostic {
                code: "E-UNTERMINATED-STRING",
                span,
                message: "string literal has no matching closing delimiter",
            });
        }
        if kind == TokenKind::Identifier && !is_nfc(source.slice(span)) {
            result.diagnostics.push(SyntaxDiagnostic {
                code: "E-NON-NFC-TOKEN",
                span,
                message: "source token is not Unicode Normalization Form C",
            });
        }
        if kind == TokenKind::String
            && let Some(tag_span) = non_nfc_string_tag(source.slice(span), span.start)
        {
            result.diagnostics.push(SyntaxDiagnostic {
                code: "E-NON-NFC-TOKEN",
                span: tag_span,
                message: "string literal tag is not Unicode Normalization Form C",
            });
        }
        offset += length;
    }
    result
}

fn next_token(rest: &str) -> (TokenKind, usize) {
    if let Some(length) = take_string(rest) {
        return (TokenKind::String, length);
    }
    if rest.starts_with("!=") {
        return (TokenKind::NotEquals, 2);
    }
    if rest.starts_with("<=") {
        return (TokenKind::LessEqual, 2);
    }
    if rest.starts_with(">=") {
        return (TokenKind::GreaterEqual, 2);
    }
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
        '=' => (TokenKind::Equals, 1),
        '<' => (TokenKind::Less, 1),
        '>' => (TokenKind::Greater, 1),
        '+' => (TokenKind::Plus, 1),
        '-' if rest[1..].starts_with(|c: char| c.is_ascii_digit()) => {
            let (kind, length) = take_number(&rest[1..]);
            (kind, length + 1)
        }
        '-' => (TokenKind::Minus, 1),
        '*' => (TokenKind::Star, 1),
        '/' => (TokenKind::Slash, 1),
        '^' => (TokenKind::Caret, 1),
        c if c.is_ascii_digit() => take_number(rest),
        '_' if rest.len() == 1
            || rest[1..]
                .chars()
                .next()
                .is_none_or(|next| !is_identifier_continue(next) && !matches!(next, '-' | '?')) =>
        {
            (TokenKind::Discard, 1)
        }
        '_' => {
            let length = take_identifier(rest);
            (TokenKind::Identifier, length)
        }
        c if is_identifier_start(c) => {
            let length = take_identifier(rest);
            let kind = if matches!(&rest[..length], "true" | "false") {
                TokenKind::Boolean
            } else {
                TokenKind::Identifier
            };
            (kind, length)
        }
        c => (TokenKind::Unknown, c.len_utf8()),
    }
}

fn take_identifier(text: &str) -> usize {
    let mut length = take_while(text, |value| is_identifier_continue(value) || value == '-');
    if text[length..].starts_with('?') {
        length += 1;
    }
    length
}

fn take_string(text: &str) -> Option<usize> {
    let opening = text.find('"')?;
    let tag = &text[..opening];
    if !tag.chars().all(valid_tag_character) {
        return None;
    }
    let after_opening = &text[opening + 1..];
    let closing = format!("\"{tag}");
    Some(
        after_opening
            .find(&closing)
            .map_or(text.len(), |offset| opening + 1 + offset + closing.len()),
    )
}

fn valid_tag_character(character: char) -> bool {
    character != '"'
        && !character.is_whitespace()
        && !matches!(character, '(' | ')' | '{' | '}' | '[' | ']')
}

fn string_is_terminated(text: &str) -> bool {
    let Some(opening) = text.find('"') else {
        return false;
    };
    let tag = &text[..opening];
    text[opening + 1..].ends_with(&format!("\"{tag}"))
}

fn non_nfc_string_tag(text: &str, token_start: usize) -> Option<Span> {
    let opening = text.find('"')?;
    let tag = &text[..opening];
    (!tag.is_empty() && !is_nfc(tag)).then(|| Span::new(token_start, token_start + opening))
}

fn take_number(text: &str) -> (TokenKind, usize) {
    if text.starts_with("0b") || text.starts_with("0o") || text.starts_with("0x") {
        return (
            TokenKind::Integer,
            take_while(text, |character| {
                character.is_ascii_alphanumeric() || character == '_'
            }),
        );
    }
    let mut length = take_while(text, |character| {
        character.is_ascii_digit() || character == '_'
    });
    let mut rational = false;
    if text[length..].starts_with('.')
        && text[length + 1..]
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_digit())
    {
        rational = true;
        length += 1;
        length += take_while(&text[length..], |character| {
            character.is_ascii_digit() || character == '_'
        });
    }
    if text[length..].starts_with(['e', 'E']) {
        rational = true;
        length += 1;
        if text[length..].starts_with(['+', '-']) {
            length += 1;
        }
        length += take_while(&text[length..], |character| {
            character.is_ascii_digit() || character == '_'
        });
    }
    (
        if rational {
            TokenKind::Rational
        } else {
            TokenKind::Integer
        },
        length,
    )
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
    fn tokenizes_fundamental_equality_as_a_callable_symbol() {
        assert_eq!(
            kinds("true = false"),
            vec![
                TokenKind::Boolean,
                TokenKind::Whitespace,
                TokenKind::Equals,
                TokenKind::Whitespace,
                TokenKind::Boolean,
            ]
        );
    }

    #[test]
    fn selects_the_declared_inequality_symbol_as_one_token() {
        assert_eq!(
            kinds("1 != 2"),
            vec![
                TokenKind::Integer,
                TokenKind::Whitespace,
                TokenKind::NotEquals,
                TokenKind::Whitespace,
                TokenKind::Integer,
            ]
        );
    }

    #[test]
    fn longest_matches_exact_ordering_symbols() {
        assert_eq!(
            kinds("1 < 2 <= 3 > 2 >= 1"),
            vec![
                TokenKind::Integer,
                TokenKind::Whitespace,
                TokenKind::Less,
                TokenKind::Whitespace,
                TokenKind::Integer,
                TokenKind::Whitespace,
                TokenKind::LessEqual,
                TokenKind::Whitespace,
                TokenKind::Integer,
                TokenKind::Whitespace,
                TokenKind::Greater,
                TokenKind::Whitespace,
                TokenKind::Integer,
                TokenKind::Whitespace,
                TokenKind::GreaterEqual,
                TokenKind::Whitespace,
                TokenKind::Integer,
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
    fn reserves_complete_boolean_literal_spellings() {
        assert_eq!(
            kinds("true false true-value"),
            vec![
                TokenKind::Boolean,
                TokenKind::Whitespace,
                TokenKind::Boolean,
                TokenKind::Whitespace,
                TokenKind::Identifier,
            ]
        );
    }

    #[test]
    fn reserves_the_complete_discard_spelling() {
        let source = SourceText::new("_ _value").unwrap();
        let kinds = lex(&source)
            .tokens
            .into_iter()
            .filter(|token| !token.kind.is_trivia())
            .map(|token| token.kind)
            .collect::<Vec<_>>();
        assert_eq!(kinds, [TokenKind::Discard, TokenKind::Identifier]);
    }

    #[test]
    fn distinguishes_rational_literals_from_operators_and_based_digits() {
        assert_eq!(
            kinds("-1.25e+3+0xCAFE"),
            vec![TokenKind::Rational, TokenKind::Plus, TokenKind::Integer]
        );
    }

    #[test]
    fn tokenizes_ordinary_tagged_and_multiline_strings_losslessly() {
        assert_eq!(kinds("\"plain\\n{value}\""), vec![TokenKind::String]);
        assert_eq!(
            kinds("text\"a \"quote\" and\nnewline\"text"),
            vec![TokenKind::String]
        );
    }

    #[test]
    fn retains_unterminated_string_for_recovery() {
        let source = SourceText::new("tag\"unfinished").unwrap();
        let lexed = lex(&source);
        assert_eq!(lexed.tokens[0].kind, TokenKind::String);
        assert_eq!(lexed.diagnostics[0].code, "E-UNTERMINATED-STRING");
    }

    #[test]
    fn rejects_non_nfc_identifier_and_literal_tag_but_preserves_contents() {
        let source =
            SourceText::new("e\u{301} tag\u{301}\"e\u{301}\"tag\u{301} \"e\u{301}\"").unwrap();
        let lexed = lex(&source);
        assert_eq!(
            lexed
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code)
                .collect::<Vec<_>>(),
            vec!["E-NON-NFC-TOKEN", "E-NON-NFC-TOKEN"]
        );
    }

    #[test]
    fn predicate_question_mark_is_a_terminal_identifier_suffix() {
        let source = SourceText::new("empty? value?? _?").unwrap();
        let lexed = lex(&source);
        assert_eq!(source.slice(lexed.tokens[0].span), "empty?");
        assert_eq!(lexed.tokens[0].kind, TokenKind::Identifier);
        assert_eq!(source.slice(lexed.tokens[2].span), "value?");
        assert_eq!(lexed.tokens[3].kind, TokenKind::Unknown);
        assert_eq!(source.slice(lexed.tokens[5].span), "_?");
        assert_eq!(lexed.tokens[5].kind, TokenKind::Identifier);
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
