use std::io::{self, Write};

use crate::ExecutionSnapshot;

/// Stable interpreter/compiler comparison envelope.
pub const TEST_TRACE_SCHEMA: &str = "topal.test-trace/1";

/// One stable, machine-readable interpreter decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceEvent<'a> {
    pub event: &'static str,
    pub rule: &'static str,
    pub detail: &'a str,
}

/// Destination for semantic decision events.
pub trait TraceSink {
    fn record(&mut self, event: TraceEvent<'_>);

    fn checkpoint(&mut self, _snapshot: ExecutionSnapshot<'_>) {}
}

impl TraceSink for Vec<String> {
    fn record(&mut self, event: TraceEvent<'_>) {
        self.push(event.to_json_line());
    }
}

impl TraceEvent<'_> {
    /// Serialize the stable trace envelope as one JSON Lines record.
    #[must_use]
    pub fn to_json_line(&self) -> String {
        format!(
            "{{\"schema\":\"{TEST_TRACE_SCHEMA}\",\"event\":\"{}\",\"rule\":\"{}\",\"detail\":\"{}\"}}",
            escape(self.event),
            escape(self.rule),
            escape(self.detail)
        )
    }
}

/// A trace sink backed by a writer.
pub struct JsonLines<W> {
    writer: W,
}

impl<W> JsonLines<W> {
    #[must_use]
    pub const fn new(writer: W) -> Self {
        Self { writer }
    }
}

impl<W: Write> TraceSink for JsonLines<W> {
    fn record(&mut self, event: TraceEvent<'_>) {
        // Trace I/O cannot alter language execution. The CLI checks stderr itself.
        let _ = writeln!(self.writer, "{}", event.to_json_line());
    }
}

impl TraceSink for io::Sink {
    fn record(&mut self, _event: TraceEvent<'_>) {}
}

fn escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if c.is_control() => {
                use std::fmt::Write as _;
                let _ = write!(escaped, "\\u{:04x}", c as u32);
            }
            c => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_schema_is_shared_by_every_event() {
        let line = TraceEvent {
            event: "event",
            rule: "RULE",
            detail: "detail",
        }
        .to_json_line();
        assert!(line.contains(TEST_TRACE_SCHEMA));
    }

    #[test]
    fn trace_serialization_is_deterministic() {
        let event = TraceEvent {
            event: "chosen",
            rule: "RULE-001",
            detail: "same",
        };
        assert_eq!(event.to_json_line(), event.to_json_line());
    }

    #[test]
    fn trace_strings_are_validly_escaped() {
        let line = TraceEvent {
            event: "quote\"",
            rule: "slash\\",
            detail: "line\nnext",
        }
        .to_json_line();
        assert!(line.contains("quote\\\""));
        assert!(line.contains("slash\\\\"));
        assert!(line.contains("line\\nnext"));
    }

    #[test]
    fn one_event_always_occupies_one_json_line() {
        let line = TraceEvent {
            event: "a\nb",
            rule: "r\nr",
            detail: "d\nd",
        }
        .to_json_line();
        assert!(!line.contains('\n'));
    }
}
