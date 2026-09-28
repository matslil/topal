//! Shared frontend and evaluator for Topal tools.

mod compiler_model;
mod concurrency;
mod documentation;
mod execution;
pub mod modules;
mod source;
mod trace;

/// Checked semantic model consumed by native compiler backends.
pub mod compiler {
    pub use crate::compiler_model::*;
}

/// Deterministic source execution model consumed by interpreters and debuggers.
pub mod interpreter {
    pub use crate::execution::{
        ExecutionHistory, ExecutionSnapshot, ExecutionState, ExecutionTransition, SourceRange,
    };
    pub use crate::source::{Execution, ExecutionStep, Session, Value, display_string_literal};
}

/// Stable semantic tracing interface shared by execution adapters.
pub mod tracing {
    pub use crate::trace::{
        DEBUGGING_PROFILE, JsonLines, TEST_TRACE_SCHEMA, TESTING_PROFILE, TraceEvent, TraceSink,
    };
}

pub use compiler_model::{
    CompilerAddressOffset, CompilerAddressOffsetType, CompilerAddressRange,
    CompilerAddressRangeType, CompilerAggregatePathElement, CompilerBinary, CompilerBinding,
    CompilerBlock, CompilerCapability, CompilerComparisonRule, CompilerConstraint,
    CompilerContainerKind, CompilerDependency, CompilerEffectRow, CompilerEnumRule,
    CompilerEnumType, CompilerErrorCodeRule, CompilerErrorField, CompilerExpression,
    CompilerExpressionKind, CompilerExternalLayout, CompilerExternalLayoutFamily,
    CompilerExternalMetadata, CompilerFallible, CompilerFunction, CompilerFunctionResultCapture,
    CompilerFunctionView, CompilerGeneratorCloseHandler, CompilerGeneratorLocal,
    CompilerGeneratorType, CompilerGeneratorYield, CompilerIdentity, CompilerInterface,
    CompilerInterfaceImplementation, CompilerInterfaceOperation,
    CompilerInterfaceOperationEvidence, CompilerLanguageContext, CompilerListIndexOperation,
    CompilerListZipOperation, CompilerLocation, CompilerLocationType, CompilerMapCollisionPolicy,
    CompilerModularType, CompilerParameter, CompilerPatternIdentity, CompilerProgram,
    CompilerSourceModule, CompilerStatement, CompilerSumAlternative, CompilerSumRule,
    CompilerSumType, CompilerTaskHandler, CompilerTaskHandlerKind, CompilerTaskMessage,
    CompilerTaskScheduler, CompilerTaskType, CompilerType, CompilerTypeView, CompilerTypeViewForm,
    CompilerValidation, IntRange, analyze_for_compiler, analyze_for_compiler_with_modules,
    compiler_function_result_capture_storage,
};
pub use concurrency::{
    Admission, DependencyGraph, DependencyKind, Interaction, InteractionForm, Protocol,
    ProtocolTransition, TaskScope, validate_schedule_equivalence,
};
pub use documentation::lang_documentation;
pub use execution::{
    ExecutionHistory, ExecutionSnapshot, ExecutionState, ExecutionTransition, SourceRange,
};
pub use modules::{
    ModuleSelectionError, SourceModule, declares_library, declares_string_solver, load_module_tree,
    published_function_names, references_module, select_source_modules,
};
pub use source::{Execution, ExecutionStep, Session, Value, display_string_literal};
pub use topal_semantics::LanguageVersion;
pub use topal_source::Diagnostic;
pub use topal_source::UNICODE_VERSION;
pub use trace::{
    DEBUGGING_PROFILE, JsonLines, TEST_TRACE_SCHEMA, TESTING_PROFILE, TraceEvent, TraceSink,
};
