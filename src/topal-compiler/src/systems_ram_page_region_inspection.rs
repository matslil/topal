//! Exact native templates, owned storage and combined source/caller bounds.
use super::*;
use serde::{Deserialize, Serialize};

use crate::{SystemsRegionReadOnlyBinding, SystemsRootRegionCode, SystemsRootRegionStorage};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsRamPageRegionBinding {
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
    pub map_offset: u64,
    pub map_bytes: u64,
    pub association_offset: u64,
    pub association_bytes: u64,
    pub payload_offset: u64,
    pub payload_bytes: u64,
    pub slots: u8,
    pub controller_owner: SystemsRootRegionStorage,
    pub controller_code: SystemsRootRegionCode,
}
pub(crate) fn inspect_linked(
    bytes: &[u8],
    library: &crate::SystemsRegionLibraryArtifact,
) -> Result<SystemsRamPageRegionBinding, CompileError> {
    use super::super::file_region::{InvocationTemplate, fatal_template, inspect_owned};
    use crate::systems_region_library::{
        PAGE_FRAME_ABORT, PAGE_FRAME_ALLOCATE, PAGE_FRAME_EXPORT, PAGE_INSTANCE,
    };
    if library.schema != "topal.kernel-region-frame-source-object.x86_64/2"
        || library.instance.as_deref() != Some(PAGE_INSTANCE)
        || library.functions.len() > 64
    {
        return Err(CompileError::Tool(
            "E-SYSTEMS-REGION-PAGE: incompatible source instance/context/function bound".into(),
        ));
    }
    let binding = inspect_owned(
        bytes,
        library,
        InvocationTemplate {
            framed: true,
            schema: "topal.ram-page-region-invocation.x86_64/1",
            interface: "borrowed-byte-region-instance-action-v1",
            export: PAGE_FRAME_EXPORT,
            allocate: PAGE_FRAME_ALLOCATE,
            abort_hook: PAGE_FRAME_ABORT,
            arena: ARENA,
            control: CONTROL,
            history: HISTORY,
            live_frame_bytes: LIVE_FRAME_BYTES,
            call_alignment_bytes: CALL_ALIGNMENT_BYTES,
            code: vec![
                (APPLY, apply()?),
                (PAGE_FRAME_ALLOCATE, allocator()?),
                (PAGE_FRAME_ABORT, fatal_template()),
                (FINALIZE, finalize()?),
            ],
        },
    )?;
    Ok(SystemsRamPageRegionBinding {
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
        map_offset: MAP as u64,
        map_bytes: 8,
        association_offset: PAGES as u64,
        association_bytes: 32,
        payload_offset: BUFFER as u64,
        payload_bytes: 4096,
        slots: 8,
    })
}
