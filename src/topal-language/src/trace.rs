use std::io::{self, Write};

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
            "{{\"schema\":\"topal.test-trace/1\",\"event\":\"{}\",\"rule\":\"{}\",\"detail\":\"{}\"}}",
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
