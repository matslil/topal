//! Shared normalized source text, byte ranges, and position mapping.

use std::fmt;

/// Unicode data version fixed by the initial Topal language context.
pub const UNICODE_VERSION: (u8, u8, u8) = (17, 0, 0);

const _: () = {
    assert!(unicode_ident::UNICODE_VERSION.0 == UNICODE_VERSION.0);
    assert!(unicode_ident::UNICODE_VERSION.1 == UNICODE_VERSION.1);
    assert!(unicode_ident::UNICODE_VERSION.2 == UNICODE_VERSION.2);
    assert!(unicode_normalization::UNICODE_VERSION.0 == UNICODE_VERSION.0);
    assert!(unicode_normalization::UNICODE_VERSION.1 == UNICODE_VERSION.1);
    assert!(unicode_normalization::UNICODE_VERSION.2 == UNICODE_VERSION.2);
};

#[must_use]
pub fn is_nfc(text: &str) -> bool {
    unicode_normalization::is_nfc(text)
}

#[must_use]
pub fn is_identifier_start(character: char) -> bool {
    unicode_ident::is_xid_start(character)
}

#[must_use]
pub fn is_identifier_continue(character: char) -> bool {
    unicode_ident::is_xid_continue(character)
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceError {
    pub code: &'static str,
    pub span: Span,
    pub message: &'static str,
}

impl fmt::Display for SourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for SourceError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceText {
    text: String,
    line_starts: Vec<usize>,
}

impl SourceText {
    /// Validate and normalize decoded UTF-8 source.
    ///
    /// # Errors
    ///
    /// Returns a stable diagnostic for forbidden scalars or line endings.
    pub fn new(text: &str) -> Result<Self, SourceError> {
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        for (offset, character) in text.char_indices() {
            if character == '\0' || is_noncharacter(character) {
                return Err(SourceError {
                    code: "E-SOURCE-DECODE",
                    span: Span::new(offset, offset + character.len_utf8()),
                    message: "source contains a forbidden Unicode scalar",
                });
            }
            if character == '\r' && !text[offset..].starts_with("\r\n") {
                return Err(SourceError {
                    code: "E-SOURCE-LINE-END",
                    span: Span::new(offset, offset + 1),
                    message: "bare carriage return is not a valid line ending",
                });
            }
        }
        let text = text.replace("\r\n", "\n");
        let mut line_starts = vec![0];
        line_starts.extend(text.match_indices('\n').map(|(offset, _)| offset + 1));
        Ok(Self { text, line_starts })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.text
    }

    #[must_use]
    pub fn slice(&self, span: Span) -> &str {
        &self.text[span.start..span.end]
    }

    #[must_use]
    pub fn position(&self, offset: usize) -> Position {
        let line_index = self.line_starts.partition_point(|start| *start <= offset) - 1;
        Position {
            line: line_index + 1,
            column: self.text[self.line_starts[line_index]..offset]
                .chars()
                .count()
                + 1,
        }
    }
}

const fn is_noncharacter(character: char) -> bool {
    let scalar = character as u32;
    (scalar >= 0xfdd0 && scalar <= 0xfdef) || scalar & 0xffff >= 0xfffe
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_and_maps_unicode_positions() {
        let source = SourceText::new("\u{feff}å\r\nβ").unwrap();
        assert_eq!(source.as_str(), "å\nβ");
        assert_eq!(source.position(3), Position { line: 2, column: 1 });
    }

    #[test]
    fn rejects_bare_carriage_return() {
        assert_eq!(
            SourceText::new("a\rb").unwrap_err().code,
            "E-SOURCE-LINE-END"
        );
    }

    #[test]
    fn exposes_pinned_unicode_17_tables() {
        assert_eq!(UNICODE_VERSION, (17, 0, 0));
        assert!(is_nfc("é"));
        assert!(!is_nfc("e\u{301}"));
        assert!(is_identifier_start('\u{1c89}'));
    }
}
