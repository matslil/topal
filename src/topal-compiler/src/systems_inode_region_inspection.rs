//! Exact controller/source/layout ownership and complete combined stack proof.
use super::*;
use crate::{SystemsRegionReadOnlyBinding, SystemsRootRegionCode, SystemsRootRegionStorage};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsInodeRegionBinding {
    pub schema: String,
    pub interface: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::systems_region_library::deserialize_present_region_field"
    )]
    pub scalar_count: Option<u8>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::systems_region_library::deserialize_present_region_field"
    )]
    pub instance: Option<String>,
    pub maximum_source_stack_bytes: u64,
    pub caller_alignment_bytes: u64,
    pub live_frame_bytes: u64,
    pub combined_stack_bytes: u64,
    pub retired_stack_reserve_bytes: u64,
    pub imported_hook_stack_bytes: u64,
    pub storage: Vec<SystemsRootRegionStorage>,
    pub code: Vec<SystemsRootRegionCode>,
    pub source_code: Vec<SystemsRootRegionCode>,
    pub source_data: Vec<SystemsRegionReadOnlyBinding>,
    pub controller_owner: SystemsRootRegionStorage,
    pub controller_code: SystemsRootRegionCode,
    pub directory_metadata_offset: u64,
    pub file_object_offset: u64,
    pub file_object_bytes: u64,
    pub file_objects: u8,
    pub file_metadata_offset: u64,
    pub pipe_pool_offset: u64,
    pub pipe_record_bytes: u64,
    pub pipe_objects: u8,
    pub pipe_metadata_offset: u64,
    pub pipe_creation_time_offset: u64,
    pub metadata_bytes: u64,
}
pub(crate) fn inspect_linked(
    bytes: &[u8],
    library: &crate::SystemsRegionLibraryArtifact,
) -> Result<SystemsInodeRegionBinding, CompileError> {
    use super::super::file_region::{InvocationTemplate, fatal_template, inspect_owned};
    use crate::systems_region_library::INODE_INSTANCE;
    if library.schema != "topal.kernel-region-frame-source-object.x86_64/4"
        || library.instance.as_deref() != Some(INODE_INSTANCE)
        || library.functions.len() > 64
    {
        return Err(CompileError::Tool(
            "E-SYSTEMS-INODE-MODE: incompatible source instance/context".into(),
        ));
    }
    library.native_symbols()?;
    let binding = inspect_owned(
        bytes,
        library,
        InvocationTemplate {
            framed: true,
            schema: "topal.inode-region-invocation.x86_64/1",
            interface: "borrowed-byte-region-instance-action-v1",
            export: INODE_FRAME_EXPORT,
            allocate: INODE_FRAME_ALLOCATE,
            abort_hook: INODE_FRAME_ABORT,
            arena: ARENA,
            control: CONTROL,
            history: HISTORY,
            extra_storage: Vec::new(),
            live_frame_bytes: LIVE_FRAME_BYTES,
            call_alignment_bytes: CALL_ALIGNMENT_BYTES,
            code: vec![
                (APPLY, apply()?),
                (INODE_FRAME_ALLOCATE, allocator()?),
                (INODE_FRAME_ABORT, fatal_template()),
                (FINALIZE, finalize()?),
            ],
        },
    )?;
    Ok(SystemsInodeRegionBinding {
        schema: binding.schema,
        interface: binding.interface,
        scalar_count: binding.scalar_count,
        instance: binding.instance,
        maximum_source_stack_bytes: binding.maximum_source_stack_bytes,
        caller_alignment_bytes: binding.caller_alignment_bytes,
        live_frame_bytes: binding.live_frame_bytes,
        combined_stack_bytes: binding.combined_stack_bytes,
        retired_stack_reserve_bytes: binding.retired_stack_reserve_bytes,
        imported_hook_stack_bytes: binding.imported_hook_stack_bytes,
        storage: binding.storage,
        code: binding.code,
        source_code: binding.source_code,
        source_data: binding.source_data,
        controller_owner: binding.controller_owner,
        controller_code: binding.controller_code,
        directory_metadata_offset: DIRECTORY as u64,
        file_object_offset: OBJECTS as u64,
        file_object_bytes: 128,
        file_objects: 4,
        file_metadata_offset: 24,
        pipe_pool_offset: PIPES as u64,
        pipe_record_bytes: 128,
        pipe_objects: 4,
        pipe_metadata_offset: 64,
        pipe_creation_time_offset: 96,
        metadata_bytes: 32,
    })
}
