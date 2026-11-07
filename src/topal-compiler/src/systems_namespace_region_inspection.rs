//! Exact namespace source/controller/private storage and complete stack binding.
use super::*;
use crate::SystemsFilePositionRegionBinding;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsNamespaceRegionBinding {
    pub schema: String,
    pub invocation: SystemsFilePositionRegionBinding,
    pub metadata_offset: u64,
    pub metadata_bytes: u64,
    pub staged_input_offset: u64,
    pub staged_input_bytes: u64,
    pub comparison_bytes: u64,
    pub recovery_bytes: u64,
    pub helper_stack_bytes: u64,
    pub private_bss_minimum_address: u64,
}

pub(crate) fn inspect_linked(
    bytes: &[u8],
    library: &crate::SystemsRegionLibraryArtifact,
) -> Result<SystemsNamespaceRegionBinding, CompileError> {
    use super::super::file_region::{InvocationTemplate, inspect_owned};
    if library.schema != "topal.kernel-region-frame-source-object.x86_64/4"
        || library.instance.as_deref() != Some(crate::systems_region_library::NAMESPACE_INSTANCE)
        || library.scalar_count != Some(2)
        || oracle::HELPER_STACK_BYTES > library.maximum_stack_bytes
    {
        return Err(CompileError::Tool(
            "E-SYSTEMS-NAMESPACE: incompatible source instance/context".into(),
        ));
    }
    // The 64-function source qualification budget is checked by the source
    // compiler. Its separate native runtime functions do not consume that cap.
    library.native_symbols()?;
    let invocation = inspect_owned(
        bytes,
        library,
        InvocationTemplate {
            framed: true,
            schema: "topal.namespace-region-invocation.x86_64/1",
            interface: "borrowed-byte-region-instance-action-v1",
            export: NAMESPACE_FRAME_EXPORT,
            allocate: NAMESPACE_FRAME_ALLOCATE,
            abort_hook: NAMESPACE_FRAME_ABORT,
            arena: ARENA,
            control: CONTROL,
            history: HISTORY,
            extra_storage: vec![
                (oracle::EXPECTED, oracle::EXPECTED_BYTES, 4096),
                (RECOVERY, RECOVERY_BYTES, 8),
            ],
            live_frame_bytes: LIVE_FRAME_BYTES,
            call_alignment_bytes: CALL_ALIGNMENT_BYTES,
            code: templates()?,
        },
    )?;
    if invocation
        .storage
        .iter()
        .any(|s| s.address < PRIVATE_BSS_MINIMUM)
        || invocation.controller_owner.address < PRIVATE_BSS_MINIMUM
    {
        return Err(CompileError::Tool(
            "E-SYSTEMS-NAMESPACE: private BSS crosses the reserved user window".into(),
        ));
    }
    Ok(SystemsNamespaceRegionBinding {
        schema: "topal.namespace-owner-binding.x86_64/1".into(),
        invocation,
        metadata_offset: METADATA as u64,
        metadata_bytes: 4096,
        staged_input_offset: 3072,
        staged_input_bytes: 512,
        comparison_bytes: oracle::EXPECTED_BYTES,
        recovery_bytes: RECOVERY_BYTES,
        helper_stack_bytes: oracle::HELPER_STACK_BYTES,
        private_bss_minimum_address: PRIVATE_BSS_MINIMUM,
    })
}
