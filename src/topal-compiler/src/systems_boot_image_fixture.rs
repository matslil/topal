use super::*;

/// Test-only unpublished component boot. It produces no artifact provenance or
/// publication authority; the public boot/publisher guards remain mandatory.
pub(crate) fn unpublished_component_fixture(kernel: &[u8]) -> Result<Vec<u8>, CompileError> {
    let file = object::File::parse(kernel).map_err(|e| CompileError::Tool(e.to_string()))?;
    let segments = collect_load_segments(&file)?;
    let entry = required_executable_symbol(&file, &segments, X86_SYSTEMS_KERNEL_ENTRY)?;
    let debug = required_executable_symbol(&file, &segments, X86_SYSTEMS_DEBUG_BREAK_ENTRY)?;
    let local = required_executable_symbol(&file, &segments, X86_SYSTEMS_LOCAL_NOTIFICATION_ENTRY)?;
    let deadline = required_executable_symbol(&file, &segments, X86_SYSTEMS_DEADLINE_ENTRY)?;
    let fault = required_executable_symbol(&file, &segments, X86_SYSTEMS_USER_COPY_FAULT_SYMBOL)?;
    let mut protected = materialize_protected_payload(&segments)?;
    install_transition_support(&mut protected, entry, debug, local, deadline, fault)?;
    pad_to(&mut protected, 16);
    let size = u32::try_from(protected.len()).map_err(|e| CompileError::Tool(e.to_string()))?;
    let mut bytes = build_setup_header(size.div_ceil(16), size)?;
    bytes.extend_from_slice(&protected);
    Ok(bytes)
}
