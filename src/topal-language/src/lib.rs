//! Shared frontend and evaluator for Topal tools.

mod concurrency;
mod execution;
mod source;
mod trace;

pub use concurrency::{
    Admission, DependencyGraph, DependencyKind, Interaction, InteractionForm, Protocol,
    ProtocolTransition, TaskScope, validate_schedule_equivalence,
};
pub use execution::{
    ExecutionHistory, ExecutionSnapshot, ExecutionState, ExecutionTransition, SourceRange,
};
pub use source::{Execution, ExecutionStep, Session, Value};
pub use topal_semantics::LanguageVersion;
pub use topal_source::Diagnostic;
pub use topal_source::UNICODE_VERSION;
pub use trace::{
    DEBUGGING_PROFILE, JsonLines, TEST_TRACE_SCHEMA, TESTING_PROFILE, TraceEvent, TraceSink,
};
