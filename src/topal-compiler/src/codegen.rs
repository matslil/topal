use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use num_bigint::{BigInt, Sign};
use num_rational::BigRational;
use topal_language::compiler::{
    CompilerAggregatePathElement, CompilerBinary, CompilerBlock, CompilerCValue,
    CompilerComparisonRule, CompilerContainerKind, CompilerEnumRule, CompilerEnumType,
    CompilerErrorCodeRule, CompilerErrorField, CompilerExpression, CompilerExpressionKind,
    CompilerFallible, CompilerFunction, CompilerFunctionResultCapture,
    CompilerGeneratorCloseHandler, CompilerGeneratorLocal, CompilerGeneratorType,
    CompilerGeneratorYield, CompilerListIndexOperation, CompilerListZipOperation,
    CompilerLocationType, CompilerMapCollisionPolicy, CompilerModularType, CompilerParameter,
    CompilerProgram, CompilerStatement, CompilerSumRule, CompilerSumType, CompilerTaskType,
    CompilerType, CompilerValidation, compiler_function_result_capture_storage,
    display_string_literal,
};
use topal_source::Span;

use crate::{DATA_LAYOUT, TARGET_TRIPLE};

include!("codegen/generator_support.rs");
include!("codegen/generator_new.rs");
include!("codegen/generator_emit_expression.rs");
include!("codegen/generator_emit_expression_remaining.rs");
include!("codegen/generator_emit_custom_character_handled_close.rs");
include!("codegen/generator_emit_list_select.rs");
include!("codegen/generator_emit_optional_payload_pointer.rs");
include!("codegen/generator_emit_decision_phi.rs");
include!("codegen/generator_emit_string_range_characters_collect.rs");
include!("codegen/ll_value.rs");
include!("codegen/debug_info.rs");
include!("codegen/llvm_types_and_runtime.rs");
#[cfg(test)]
mod tests {
    include!("codegen/tests/emits_target_platform_runtime_and_debug_metadata.rs");
    include!("codegen/tests/embeds_and_validates_canonical_native_serialization.rs");
    include!("codegen/tests/emits_required_nat_validation_for_dynamic_list_insertion.rs");
    include!("codegen/tests/emits_recursive_nominal_values_across_custom_generator_directions.rs");
}
