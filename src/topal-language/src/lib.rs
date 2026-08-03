//! Shared frontend and evaluator for Topal tools.

mod source;
mod trace;

pub use source::{Diagnostic, Session, Value};
pub use trace::{JsonLines, TraceEvent, TraceSink};
