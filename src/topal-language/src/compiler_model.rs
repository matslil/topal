//! Shared checked model consumed by the native compiler.
//!
//! This first model intentionally covers a bounded, explicit language slice.
//! Extending it is how later compiler increments acquire semantics; the LLVM
//! backend never reinterprets the source syntax itself.

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

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
    AnonymousPattern, CallableKind, DecisionMatcher, Expression, FunctionClauses,
    FunctionParameter, InterfaceFunction, ProductField, Statement, lex, parse,
};

use crate::modules::SourceModule;
use crate::source::{
    body_mentions_name, direct_expression_returns_from_function, explicit_absolute_measure,
    explicit_single_measure, expression_mentions_name, is_supported_returning_boolean_action_shape,
    is_supported_returning_comparison_value_action_shape,
    is_supported_returning_enum_fallback_action_shape,
    is_supported_returning_error_code_action_shape,
    is_supported_returning_exhaustive_enum_action_shape, is_supported_returning_list_action_shape,
    is_supported_returning_optional_action_shape,
    is_supported_returning_ordered_comparison_action_shape,
    is_supported_returning_result_action_shape, parse_integer, parse_rational, parse_string,
    prove_euclidean_recursion, prove_explicit_parameter_recursion, prove_int_recursion,
    prove_mutual_bounded_recursion_edge,
};

include!("compiler_model/model.rs");
include!("compiler_model/analysis_setup.rs");
include!("compiler_model/collect_interface_sources.rs");
include!("compiler_model/exact_boolean_local_function_generator_body.rs");
include!("compiler_model/exact_recursive_nominal_generator_body.rs");
include!("compiler_model/collect_character_generators.rs");
include!("compiler_model/analyzer_install_modular_types.rs");
include!("compiler_model/analyzer_analyze_returning_exhaustive_enum_decision_actions.rs");
include!("compiler_model/analyzer_analyze_block_control.rs");
include!("compiler_model/analyzer_analyze_expression.rs");
include!("compiler_model/analyzer_native_serialization_bytes.rs");
include!("compiler_model/analyzer_analyze_application.rs");
include!("compiler_model/analyzer_analyze_collection_application.rs");
include!("compiler_model/analyzer_analyze_function_value_chain.rs");
include!("compiler_model/analyzer_forward_function_aggregate_captures.rs");
include!("compiler_model/analyzer_analyze_range_observation.rs");
include!("compiler_model/analyzer_adapt_structural_equality.rs");
include!("compiler_model/analyzer_analyze_resolved_call_from.rs");
include!("compiler_model/analyzer_capture_forwarding_overload.rs");
include!("compiler_model/analyzer_closes_proven_mutual_bounded_cycle.rs");
include!("compiler_model/analyzer_analyze_comparison_value_decision.rs");
include!("compiler_model/function_body_context_member_span.rs");
include!("compiler_model/external_record_fields.rs");
include!("compiler_model/known_constraint_predicate.rs");
include!("compiler_model/adapt_custom_generator_initial.rs");
#[cfg(test)]
mod tests {
    include!(
        "compiler_model/tests/analyzes_functions_and_boolean_decisions_for_the_native_backend.rs"
    );
    include!("compiler_model/tests/models_custom_generator_local_character_alias.rs");
    include!("compiler_model/tests/models_custom_generator_explicit_string_return_before_yield.rs");
    include!("compiler_model/tests/models_exact_rationals_across_custom_generator_directions.rs");
    include!("compiler_model/tests/models_rational_lists_across_private_boundaries.rs");
    include!(
        "compiler_model/tests/models_named_constraint_validation_and_refined_base_operations.rs"
    );
    include!("compiler_model/tests/retains_and_erases_exact_function_interface_evidence.rs");
    include!("compiler_model/tests/models_map_function_environments_as_exact_private_paths.rs");
    include!("compiler_model/tests/models_non_escaping_nested_functions_with_private_captures.rs");
    include!(
        "compiler_model/tests/models_fundamental_layout_policy_values_as_distinct_nominal_enums.rs"
    );
    include!(
        "compiler_model/tests/models_contextual_exact_infinities_and_explicit_int_range_endpoints.rs"
    );
}
