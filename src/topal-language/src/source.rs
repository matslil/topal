use std::cell::{Cell, RefCell};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::rc::Rc;

use num_bigint::BigInt;
use num_rational::BigRational;
use topal_semantics::{LanguageVersion, ObjectKind};
use topal_serialization::{
    Event as SerializedEvent, Header as SerializationHeader, Limits as SerializationLimits,
    SerializedValue, Stream as SerializationStream, StreamByteOrder, TypeDefinition,
    deserialize as deserialize_native, serialize as serialize_native,
};
use topal_source::{
    Diagnostic, SourceText, Span, canonically_equal, case_fold, character_at, character_count,
    characters, is_decimal_digit, is_regex_word, lowercase, normalize_nfc, normalize_nfd,
    scalar_characters, uppercase,
};
use topal_syntax::{
    AnonymousPattern, CallableKind, DecisionMatcher, DecisionRule, Expression, FunctionClauses,
    FunctionParameter, Statement, extract_documentation, lex, parse,
};

use crate::{ExecutionSnapshot, TraceEvent, TraceSink};

include!("source/value.rs");
include!("source/session_effective_root_namespace.rs");
include!("source/session_prepare_source_file.rs");
include!("source/session_evaluate_expression.rs");
include!("source/session_evaluate_block.rs");
include!("source/session_is_modular_construction.rs");
include!("source/infer_unfold_yield_classifier.rs");
include!("source/execution.rs");
include!("source/reject_v02_language_object_shadowing.rs");
include!("source/supported_generator_value_classifier.rs");
include!("source/prove_mutual_bounded_recursion_edge.rs");
include!("source/evaluate_rational_literal.rs");
include!("source/is_unicode_white_space.rs");
include!("source/apply_quotient_modulo.rs");
#[cfg(test)]
mod tests {
    include!("source/tests/evaluate.rs");
}
include!("source/declares_and_calls_static_nullary_functions.rs");
include!("source/mutual_increasing_int_recursion_requires_one_direction_for_the_complete_cycle.rs");
include!("source/custom_generator_transfers_boolean_values.rs");
