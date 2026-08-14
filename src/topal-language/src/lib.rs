//! Shared frontend and evaluator for Topal tools.

mod execution;
mod source;
mod trace;

pub use execution::{
    ExecutionHistory, ExecutionSnapshot, ExecutionState, ExecutionTransition, SourceRange,
};
pub use source::{Diagnostic, Execution, ExecutionStep, Session, Value};
pub use topal_geir::{
    BoundaryError as CompilerBoundaryError, COMPILER_ONLY_ERROR_CODE, CompilerOnlyOperation,
    ToolRole, require_compiler,
};
pub use topal_semantics::LanguageVersion;
pub use topal_source::UNICODE_VERSION;
pub use trace::{JsonLines, TEST_TRACE_SCHEMA, TraceEvent, TraceSink};
