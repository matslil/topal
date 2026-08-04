//! Shared frontend and evaluator for Topal tools.

mod execution;
mod source;
mod trace;

pub use execution::{ExecutionHistory, ExecutionTransition};
pub use source::{Diagnostic, Session, Value};
pub use topal_source::UNICODE_VERSION;
pub use trace::{JsonLines, TraceEvent, TraceSink};
