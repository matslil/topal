//! Exact first-schema storage/projection obligations; not a source ABI.
use serde::{Deserialize, Serialize};

pub(crate) const STATE_SYMBOL: &str = "topal_x86_systems_identity_value_store";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsRetainedStateBound {
    pub schema: String,
    pub semantic_state: String,
    pub store_symbol: String,
    pub limit_bytes: u64,
    pub alignment_bytes: u64,
    pub header_bytes: u64,
    pub bank_count: u64,
    pub bank_bytes: u64,
    pub record_bytes: u64,
    pub numeric_node_bytes: u64,
    pub maximum_retained_graph_bytes: u64,
    pub maximum_value_bits: u64,
    pub maximum_bridge_validation_words: u64,
    pub maximum_prepared_validation_words: u64,
    pub maximum_commit_validation_words: u64,
    pub maximum_release_validation_words: u64,
    pub maximum_initialization_validation_words: u64,
    pub maximum_initialization_binding_words: u64,
    pub maximum_input_record_words: u64,
    pub maximum_projection_words: u64,
    pub maximum_preparation_metadata_words: u64,
    pub maximum_commit_metadata_words: u64,
    pub maximum_retirement_words: u64,
    pub maximum_terminal_words: u64,
    pub publication_protocol: String,
    pub value_layout: Option<crate::systems_retained_layout::RetainedValueLayout>,
    pub maximum_source_numeric_read_operations: u64,
    pub maximum_source_result_record_words: u64,
    pub source_result_order_values: u64,
    pub maximum_source_result_order_checks: u64,
    pub source_result_boolean_values: u64,
}
impl SystemsRetainedStateBound {
    pub(crate) fn qualified() -> Self {
        Self {
            schema: "topal.retained-numeric-state/2".into(),
            semantic_state: "Record(next:Nat); zero-sentinel or 2..2147483647".into(),
            store_symbol: STATE_SYMBOL.into(),
            limit_bytes: 4096,
            alignment_bytes: 4096,
            header_bytes: 64,
            bank_count: 2,
            bank_bytes: 2016,
            record_bytes: 16,
            numeric_node_bytes: 24,
            maximum_retained_graph_bytes: 40,
            maximum_value_bits: 31,
            // Eight header reads plus either 504 zero words, or 252 inactive
            // words + five graph words + 247 tail words. Record transport is
            // additional. Projection excludes its three preparation fields.
            maximum_bridge_validation_words: 512,
            // Native owner validation adds fourteen graph/header reads and
            // 247 clean-tail reads. Commit/release check six/two header words.
            maximum_prepared_validation_words: 261,
            maximum_commit_validation_words: 6,
            maximum_release_validation_words: 2,
            maximum_initialization_validation_words: 512,
            maximum_initialization_binding_words: 2,
            maximum_input_record_words: 2,
            maximum_projection_words: 5,
            maximum_preparation_metadata_words: 3,
            maximum_commit_metadata_words: 6,
            // Erasure and readback are both charged.
            maximum_retirement_words: 504,
            maximum_terminal_words: 1024,
            publication_protocol: "prepare-clean-publish-retire-v1".into(),
            value_layout: None,
            maximum_source_numeric_read_operations: 6,
            maximum_source_result_record_words: 3,
            source_result_order_values: 2,
            maximum_source_result_order_checks: 3,
            source_result_boolean_values: 0,
        }
    }
    pub(crate) fn structured() -> Result<Self, crate::CompileError> {
        let layout = crate::systems_retained_layout::RetainedValueLayout::identity(true)?;
        let mut result = Self::qualified();
        result.schema = "topal.retained-structured-state/1".into();
        result.semantic_state="Record(next:Nat,history:Record(creations:Nat,wrapped:Boolean)); initial (0,0,false), next 2..2147483647, creations 1..18446744073709551615".into();
        result.record_bytes = layout.record_bytes;
        result.maximum_retained_graph_bytes = layout.graph_bytes;
        result.maximum_value_bits = 64;
        result.maximum_prepared_validation_words = layout.prepared_validation_words();
        result.maximum_input_record_words = layout.record_bytes / 8;
        result.maximum_projection_words = layout.projection_words();
        result.maximum_source_numeric_read_operations = 10;
        result.maximum_source_result_record_words = crate::codegen::target_value_layout(
            &crate::systems_retained_layout::structured_result_type(),
        )
        .size
            / 64;
        result.source_result_order_values = 6;
        result.maximum_source_result_order_checks = 9;
        result.source_result_boolean_values = 1;
        result.value_layout = Some(layout);
        Ok(result)
    }
}
