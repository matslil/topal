//! Private component closure and conservative normal boot-stack bound.
use crate::CompileError;
use object::{Object, ObjectSymbol, SymbolKind};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn inspect(bytes: &[u8]) -> Result<serde_json::Value, CompileError> {
    inspect_selected(bytes, false)
}

pub(crate) fn inspect_selected(
    bytes: &[u8],
    tables: bool,
) -> Result<serde_json::Value, CompileError> {
    let invalid =
        || CompileError::Tool("incomplete image-backing component closure or stack bound".into());
    let file = object::File::parse(bytes).map_err(|_| invalid())?;
    let names = file
        .symbols()
        .filter(|s| s.is_definition() && s.kind() == SymbolKind::Text)
        .map(|s| s.name().map(str::to_owned).map_err(|_| invalid()))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let imports = file
        .symbols()
        .filter(|s| s.is_undefined())
        .filter_map(|s| s.name().ok())
        .filter(|s| !s.is_empty())
        .collect::<BTreeSet<_>>();
    let mut expected = BTreeSet::from([
        "topal_x86_frame_context_v1",
        "topal_x86_frame_registry_v1",
        "topal_x86_frame_table_v1",
        "topal_x86_frame_root_tokens_v1",
        "topal_x86_frame_details_v1",
        "topal_x86_kernel_reservation_end_v1",
        "topal_x86_frame_gateway_v1",
        "topal_x86_systems_fatal",
    ]);
    if tables {
        expected.extend([
            "topal_x86_frame_handoff_v1",
            crate::X86_SYSTEMS_KERNEL_ENTRY,
            crate::X86_SYSTEMS_DEBUG_BREAK_ENTRY,
            crate::X86_SYSTEMS_LOCAL_NOTIFICATION_ENTRY,
            crate::X86_SYSTEMS_DEADLINE_ENTRY,
            crate::X86_SYSTEMS_USER_COPY_FAULT_SYMBOL,
        ]);
    }
    let frames = crate::systems_library::stack_sizes(&file)?;
    if imports != expected || frames.keys().cloned().collect::<BTreeSet<_>>() != names {
        return Err(invalid());
    }
    let calls = crate::systems_numeric_library::library_object_calls(
        &file,
        &names,
        &["topal_x86_frame_gateway_v1", "topal_x86_systems_fatal"],
    )?;
    fn visit(
        name: &str,
        calls: &BTreeMap<String, BTreeSet<String>>,
        active: &mut BTreeSet<String>,
        done: &mut BTreeSet<String>,
    ) -> bool {
        if done.contains(name) {
            return true;
        }
        if !active.insert(name.into()) {
            return false;
        }
        if let Some(children) = calls.get(name) {
            for child in children {
                if calls.contains_key(child) && !visit(child, calls, active, done) {
                    return false;
                }
            }
        }
        active.remove(name);
        done.insert(name.into());
        true
    }
    let mut done = BTreeSet::new();
    for name in &names {
        if !visit(name, &calls, &mut BTreeSet::new(), &mut done) {
            return Err(invalid());
        }
    }
    // Sum all acyclic helper frames and their return words, deliberately
    // overcounting mutually exclusive paths. The unchanged gateway accounts
    // for its complete 4032-byte scratch plus saved registers/return/alignment.
    let helper = frames
        .values()
        .try_fold(0u64, |total, n| total.checked_add(*n)?.checked_add(8))
        .and_then(|n| n.checked_add(if tables { 8 } else { 0 }))
        .ok_or_else(invalid)?;
    let combined = helper
        .checked_add(4096)
        .filter(|n| *n <= 8192)
        .ok_or_else(invalid)?;
    let roots = if tables {
        [
            super::TABLE_PREPARE,
            super::TABLE_DISCARD,
            super::TABLE_RELEASED,
        ]
    } else {
        [super::PREPARE, super::DISCARD, super::RELEASED]
    };
    if !roots.iter().all(|s| names.contains(*s)) {
        return Err(invalid());
    }
    Ok(serde_json::json!({
        "scope": "Private inactive-preparation direct-call closure and conservative normal boot-stack bound; fatal recovery and production publication excluded",
        "object_sha256": crate::artifact::sha256(bytes), "imports": imports,
        "frames": frames, "calls": calls, "acyclic": true,
        "conservative_helper_stack_bytes": helper, "gateway_window_bytes": 4096,
        "combined_boot_stack_bytes": combined, "boot_stack_allowance_bytes": 8192,
        "protected_context": false, "production_qualified": false,
        "inactive_tables": tables,
        "inline_flags_stack_bytes": if tables {8} else {0},
    }))
}
