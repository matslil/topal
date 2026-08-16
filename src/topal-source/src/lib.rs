//! Shared normalized source text, byte ranges, and position mapping.

use std::fmt;

use icu_properties::props::{DefaultIgnorableCodePoint, GeneralCategory};
use icu_properties::{CodePointMapData, CodePointSetData};

/// Unicode data version fixed by the initial Topal language context.
pub const UNICODE_VERSION: (u8, u8, u8) = (17, 0, 0);

const _: () = {
    assert!(unicode_normalization::UNICODE_VERSION.0 == UNICODE_VERSION.0);
    assert!(unicode_normalization::UNICODE_VERSION.1 == UNICODE_VERSION.1);
    assert!(unicode_normalization::UNICODE_VERSION.2 == UNICODE_VERSION.2);
    assert!(unicode_segmentation::UNICODE_VERSION.0 == UNICODE_VERSION.0 as u64);
    assert!(unicode_segmentation::UNICODE_VERSION.1 == UNICODE_VERSION.1 as u64);
    assert!(unicode_segmentation::UNICODE_VERSION.2 == UNICODE_VERSION.2 as u64);
};

#[must_use]
pub fn is_nfc(text: &str) -> bool {
    unicode_normalization::is_nfc(text)
}

/// Returns the canonical NFC transformation under the language context.
#[must_use]
pub fn normalize_nfc(text: &str) -> String {
    use unicode_normalization::UnicodeNormalization as _;

    text.nfc().collect()
}

/// Returns the canonical NFD transformation under the language context.
#[must_use]
pub fn normalize_nfd(text: &str) -> String {
    use unicode_normalization::UnicodeNormalization as _;

    text.nfd().collect()
}

#[must_use]
pub fn is_identifier_start(character: char) -> bool {
    is_identifier_character(character) && !is_decimal_digit(character)
}

#[must_use]
pub fn is_decimal_digit(character: char) -> bool {
    CodePointMapData::<GeneralCategory>::new().get(character) == GeneralCategory::DecimalNumber
}

#[must_use]
pub fn is_identifier_character(character: char) -> bool {
    if matches!(
        character,
        '"' | '#' | '(' | ')' | '{' | '}' | '[' | ']' | ','
    ) || CodePointSetData::new::<DefaultIgnorableCodePoint>().contains(character)
    {
        return false;
    }

    !matches!(
        CodePointMapData::<GeneralCategory>::new().get(character),
        GeneralCategory::Control
            | GeneralCategory::Format
            | GeneralCategory::PrivateUse
            | GeneralCategory::Unassigned
            | GeneralCategory::SpaceSeparator
            | GeneralCategory::LineSeparator
            | GeneralCategory::ParagraphSeparator
    )
}

/// Counts extended grapheme clusters under the language context's Unicode data.
#[must_use]
pub fn character_count(text: &str) -> usize {
    use unicode_segmentation::UnicodeSegmentation as _;

    text.graphemes(true).count()
}

/// Selects one extended grapheme cluster under the language context.
#[must_use]
pub fn character_at(text: &str, index: usize) -> Option<&str> {
    use unicode_segmentation::UnicodeSegmentation as _;

    text.graphemes(true).nth(index)
}

/// Applies Unicode's locale-independent default uppercase mapping.
#[must_use]
pub fn uppercase(text: &str) -> String {
    text.to_uppercase()
}

/// Applies Unicode's locale-independent default lowercase mapping.
#[must_use]
pub fn lowercase(text: &str) -> String {
    text.to_lowercase()
}

/// Applies Unicode's full, locale-independent default case folding.
#[must_use]
pub fn case_fold(text: &str) -> String {
    use unicode_casefold::UnicodeCaseFold as _;

    text.case_fold().collect()
}

/// Tests Unicode canonical equivalence without changing either input.
#[must_use]
pub fn canonically_equal(left: &str, right: &str) -> bool {
    use unicode_normalization::UnicodeNormalization as _;

    left.nfd().eq(right.nfd())
}

/// Traverses the preserved text as extended grapheme clusters.
pub fn characters(text: &str) -> impl Iterator<Item = &str> {
    use unicode_segmentation::UnicodeSegmentation as _;

    text.graphemes(true)
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

/// Tool-independent severity used by every Topal diagnostic adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Severity {
    Warning,
    Error,
}

impl Severity {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

/// Best-practice provenance attached to a linter diagnostic.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BestPracticeDiagnostic {
    pub identity: String,
    pub version: String,
    pub rule_version: String,
}

/// Shared diagnostic data rendered by interpreter, compiler, linter, and LSP
/// adapters. Locations are one-based human source coordinates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub source_line: Option<Box<str>>,
    pub marker_width: usize,
    pub help: Option<Box<str>>,
    pub best_practice: Option<Box<BestPracticeDiagnostic>>,
}

impl Diagnostic {
    #[must_use]
    pub fn error(
        code: impl Into<String>,
        line: usize,
        column: usize,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity: Severity::Error,
            code: code.into(),
            line,
            column,
            message: message.into(),
            source_line: None,
            marker_width: 1,
            help: None,
            best_practice: None,
        }
    }

    #[must_use]
    pub fn warning(
        code: impl Into<String>,
        line: usize,
        column: usize,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity: Severity::Warning,
            ..Self::error(code, line, column, message)
        }
    }

    #[must_use]
    pub fn with_source_excerpt(mut self, source_line: Option<String>, marker_width: usize) -> Self {
        self.source_line = source_line.map(String::into_boxed_str);
        self.marker_width = marker_width.max(1);
        self
    }

    #[must_use]
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into().into_boxed_str());
        self
    }

    #[must_use]
    pub fn with_best_practice(
        mut self,
        identity: impl Into<String>,
        version: impl Into<String>,
        rule_version: impl Into<String>,
    ) -> Self {
        self.best_practice = Some(Box::new(BestPracticeDiagnostic {
            identity: identity.into(),
            version: version.into(),
            rule_version: rule_version.into(),
        }));
        self
    }

    #[must_use]
    pub fn render(&self, source_name: &str) -> String {
        let mut rendered = format!(
            "{}[{}]: {}\n --> {source_name}:{}:{}",
            self.severity.label(),
            self.code,
            self.message,
            self.line,
            self.column
        );
        if let Some(source_line) = &self.source_line {
            let gutter_width = self.line.to_string().len();
            let _ = fmt::Write::write_fmt(
                &mut rendered,
                format_args!(
                    "\n{empty:>gutter_width$} |\n{line:>gutter_width$} | {source_line}\n{empty:>gutter_width$} | {padding}{markers}",
                    empty = "",
                    line = self.line,
                    padding = " ".repeat(self.column.saturating_sub(1)),
                    markers = "^".repeat(self.marker_width),
                ),
            );
        }
        if let Some(help) = &self.help {
            let _ = fmt::Write::write_fmt(
                &mut rendered,
                format_args!(
                    "\n{empty:>width$} |\n{empty:>width$} = help: {help}",
                    empty = "",
                    width = self.line.to_string().len()
                ),
            );
        }
        rendered
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.render("<input>"))
    }
}

impl std::error::Error for Diagnostic {}

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
        assert!(is_identifier_start('🙂'));
        assert!(is_identifier_character('+'));
        assert!(!is_identifier_start('7'));
        assert!(!is_identifier_start('٧'));
        assert!(is_decimal_digit('٧'));
        assert!(!is_identifier_character('#'));
        assert!(!is_identifier_character('\u{200b}'));
        assert!(!is_identifier_character('\u{e000}'));
        assert!(!is_identifier_character('\u{0378}'));
    }

    #[test]
    fn selects_extended_grapheme_clusters() {
        let text = "a\u{301}👩‍🔬🇸🇪";
        assert_eq!(character_at(text, 0), Some("a\u{301}"));
        assert_eq!(character_at(text, 1), Some("👩‍🔬"));
        assert_eq!(character_at(text, 2), Some("🇸🇪"));
        assert_eq!(character_at(text, 3), None);
    }

    #[test]
    fn applies_default_unicode_uppercase_mapping() {
        assert_eq!(uppercase("Straße σς"), "STRASSE ΣΣ");
    }

    #[test]
    fn applies_default_unicode_lowercase_mapping() {
        assert_eq!(lowercase("İΣ"), "i̇ς");
    }

    #[test]
    fn applies_full_default_unicode_case_folding() {
        assert_eq!(case_fold("Straße Σς"), "strasse σσ");
    }

    #[test]
    fn compares_canonical_unicode_equivalence() {
        assert!(canonically_equal("é", "e\u{301}"));
        assert!(!canonically_equal("é", "e"));
    }

    #[test]
    fn traverses_extended_grapheme_clusters_in_order() {
        assert_eq!(
            characters("a\u{301}👩‍🔬🇸🇪").collect::<Vec<_>>(),
            ["a\u{301}", "👩‍🔬", "🇸🇪"]
        );
    }

    #[test]
    fn normalizes_preserved_text_explicitly() {
        assert_eq!(normalize_nfc("e\u{301}"), "é");
        assert_eq!(normalize_nfc("é"), "é");
    }

    #[test]
    fn decomposes_preserved_text_explicitly() {
        assert_eq!(normalize_nfd("é"), "e\u{301}");
        assert_eq!(normalize_nfd("e\u{301}"), "e\u{301}");
    }

    #[test]
    fn counts_user_perceived_characters() {
        assert_eq!(character_count(""), 0);
        assert_eq!(character_count("a\u{301}"), 1);
        assert_eq!(character_count("👩‍🔬"), 1);
        assert_eq!(character_count("🇸🇪!"), 2);
    }

    #[test]
    fn shared_diagnostic_renders_rust_style_source_context() {
        let diagnostic = Diagnostic::error("E-EXAMPLE", 2, 3, "example failed")
            .with_source_excerpt(Some("  value".into()), 5)
            .with_help("replace the value");
        assert_eq!(
            diagnostic.render("example.t"),
            "error[E-EXAMPLE]: example failed\n --> example.t:2:3\n  |\n2 |   value\n  |   ^^^^^\n  |\n  = help: replace the value"
        );
    }

    #[test]
    fn shared_diagnostic_retains_best_practice_provenance() {
        let diagnostic = Diagnostic::warning("L-EXAMPLE", 1, 1, "consider this")
            .with_best_practice("lang best-practice example", "v0.2", "v0.3");
        assert_eq!(diagnostic.severity, Severity::Warning);
        assert_eq!(
            diagnostic.best_practice,
            Some(Box::new(BestPracticeDiagnostic {
                identity: "lang best-practice example".into(),
                version: "v0.2".into(),
                rule_version: "v0.3".into(),
            }))
        );
    }
}
