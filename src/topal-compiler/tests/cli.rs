use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use sha2::{Digest, Sha256};
use topal_c_abi::{
    AccessLibrary, CEffect, CValue, Function, InputArtifact, PLATFORM_ABI, Parameter, SCHEMA,
    SHARED_SCHEMA, SharedAccessLibrary, SharedObjectArtifact, TARGET,
};
use topal_compiler::{LlvmTools, NATIVE_ABI, NativeArtifactMetadata, metadata_path};
use topal_language::interpreter::Session;
use topal_language::modules::{declared_libraries, load_module_tree, select_source_modules};

include!("cli/next_test.rs");
include!("cli/lexical_block_return_is_freestanding_and_debuggable.rs");
include!("cli/infix_collect_source_block_exits_are_freestanding_and_debuggable.rs");
include!("cli/ordered_comparison_action_exits_are_joined_freestanding_and_debuggable.rs");
include!("cli/returned_custom_generator_is_freestanding_and_debuggable.rs");
include!("cli/custom_generator_result_values_are_freestanding_structured_and_debuggable.rs");
include!("cli/gdb_renders_values_projected_from_reconstructed_records.rs");
include!("cli/nat_lists_are_private_freestanding_and_debuggable.rs");
include!("cli/int_pair_lists_are_complete_private_freestanding_and_debuggable.rs");
include!("cli/gdb_retains_fundamental_type_identity_across_a_function_boundary.rs");
include!("cli/empty_function_effect_bound_is_static_freestanding_and_debuggable.rs");
include!("cli/anonymous_product_patterns_are_private_freestanding_and_debuggable.rs");
include!("cli/packaged_function_operand_is_flat_private_freestanding_and_debuggable.rs");
include!("cli/function_root_data_forwarding_is_private_freestanding_and_debuggable.rs");
include!("cli/function_environment_boundaries_are_exact_private_freestanding_and_debuggable.rs");
include!("cli/modular_values_are_private_freestanding_and_debuggable.rs");
