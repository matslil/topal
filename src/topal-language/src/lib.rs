//! Shared frontend and evaluator for Topal tools.

mod compiler_model;
mod concurrency;
mod documentation;
mod execution;
mod modules;
mod source;
mod trace;

pub use compiler_model::{
    CompilerBinary, CompilerBinding, CompilerBlock, CompilerComparisonRule, CompilerExpression,
    CompilerExpressionKind, CompilerFallible, CompilerFunction, CompilerParameter, CompilerProgram,
    CompilerStatement, CompilerType, CompilerValidation, IntRange, analyze_for_compiler,
};
pub use concurrency::{
    Admission, DependencyGraph, DependencyKind, Interaction, InteractionForm, Protocol,
    ProtocolTransition, TaskScope, validate_schedule_equivalence,
};
pub use documentation::lang_documentation;
pub use execution::{
    ExecutionHistory, ExecutionSnapshot, ExecutionState, ExecutionTransition, SourceRange,
};
pub use modules::{declares_library, declares_string_solver, load_module_tree};
pub use source::{Execution, ExecutionStep, Session, Value, display_string_literal};
pub use topal_semantics::LanguageVersion;
pub use topal_source::Diagnostic;
pub use topal_source::UNICODE_VERSION;
pub use trace::{
    DEBUGGING_PROFILE, JsonLines, TEST_TRACE_SCHEMA, TESTING_PROFILE, TraceEvent, TraceSink,
};
