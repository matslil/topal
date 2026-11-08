//! Actual source/checkpoint closure and sequential private-stack evidence.
use crate::CompileError;
use object::{Object, ObjectSymbol, SymbolKind};
use std::collections::{BTreeMap, BTreeSet};

fn invalid(reason: impl std::fmt::Display) -> CompileError {
    CompileError::Tool(format!("E-SYSTEMS-TAIL-NATIVE: {reason}"))
}
pub(crate) fn inspect(source: &[u8], runtime: &[u8]) -> Result<serde_json::Value, CompileError> {
    let source = object::File::parse(source).map_err(invalid)?;
    let runtime = object::File::parse(runtime).map_err(invalid)?;
    let functions = |file: &object::File<'_>| {
        file.symbols()
            .filter(|s| s.is_definition() && s.kind() == SymbolKind::Text)
            .map(|s| s.name().map(str::to_owned).map_err(invalid))
            .collect::<Result<BTreeSet<_>, _>>()
    };
    let source_names = functions(&source)?;
    let runtime_names = functions(&runtime)?;
    let runtime_imports = runtime
        .symbols()
        .filter(|s| s.is_undefined())
        .filter_map(|s| s.name().ok())
        .filter(|s| !s.is_empty())
        .collect::<BTreeSet<_>>();
    if runtime_imports
        != BTreeSet::from([
            "topal_tail_step_component_v1",
            "topal_tail_readonly_bytes_v1",
            "topal_x86_tail_boundary_abort_v1",
            "topal_x86_tail_boundary_active_v1",
        ])
    {
        return Err(invalid(format!(
            "runtime import closure: {runtime_imports:?}"
        )));
    }
    let source_imports = source
        .symbols()
        .filter(|s| s.is_undefined())
        .filter_map(|s| s.name().ok())
        .filter(|s| !s.is_empty())
        .collect::<BTreeSet<_>>();
    if source_imports
        .iter()
        .any(|name| !runtime_names.contains(*name))
    {
        return Err(invalid(format!("unbound source hook: {source_imports:?}")));
    }
    let mut frames = crate::systems_library::stack_sizes(&source)?;
    let runtime_frames = crate::systems_library::stack_sizes(&runtime)?;
    if frames.keys().cloned().collect::<BTreeSet<_>>() != source_names
        || runtime_frames.keys().cloned().collect::<BTreeSet<_>>() != runtime_names
    {
        return Err(invalid("incomplete source/runtime stack metadata"));
    }
    if frames.keys().any(|name| runtime_frames.contains_key(name)) {
        return Err(invalid("source/runtime symbol collision"));
    }
    frames.extend(runtime_frames);
    let source_allowed = runtime_names.iter().map(String::as_str).collect::<Vec<_>>();
    let mut calls = crate::systems_numeric_library::library_object_calls(
        &source,
        &source_names,
        &source_allowed,
    )?;
    let runtime_allowed = [
        "topal_tail_readonly_bytes_v1",
        "topal_tail_step_component_v1",
        "topal_x86_tail_boundary_abort_v1",
    ];
    calls.extend(crate::systems_numeric_library::library_object_calls(
        &runtime,
        &runtime_names,
        &runtime_allowed,
    )?);
    fn visit(
        name: &str,
        frames: &BTreeMap<String, u64>,
        calls: &BTreeMap<String, BTreeSet<String>>,
        active: &mut BTreeSet<String>,
        cache: &mut BTreeMap<String, u64>,
    ) -> Result<u64, CompileError> {
        if let Some(value) = cache.get(name) {
            return Ok(*value);
        }
        if !active.insert(name.into()) {
            return Err(invalid("cyclic native source/hook call path"));
        }
        let mut child = 0;
        for callee in calls
            .get(name)
            .ok_or_else(|| invalid(format!("missing native callee {name}")))?
        {
            child = child.max(visit(callee, frames, calls, active, cache)?);
        }
        let total = frames[name]
            .checked_add(8)
            .and_then(|n| n.checked_add(child))
            .ok_or_else(|| invalid("stack bound overflow"))?;
        active.remove(name);
        cache.insert(name.into(), total);
        Ok(total)
    }
    // The target abort entry adds its call return word, then pivots without
    // pushing on the source stack. Recovery/controller stack remains separate.
    let mut paths = BTreeMap::from([("topal_x86_tail_boundary_abort_v1".into(), 8)]);
    let maximum = visit(
        "topal_tail_step_component_v1",
        &frames,
        &calls,
        &mut BTreeSet::new(),
        &mut paths,
    )?;
    let checkpoint = visit(
        "topal_tail_checkpoint_body_v1",
        &frames,
        &calls,
        &mut BTreeSet::new(),
        &mut paths,
    )?;
    let completion = visit(
        "topal_tail_complete_body_v1",
        &frames,
        &calls,
        &mut BTreeSet::new(),
        &mut paths,
    )?;
    if maximum.max(checkpoint).max(completion) > 4096 {
        return Err(invalid(format!(
            "source/checkpoint/completion stack {maximum}/{checkpoint}/{completion} exceeds reserve"
        )));
    }
    Ok(
        serde_json::json!({"scope":"source/checkpoint/completion native closure and sequential private-stack paths; kernel admission/caller/context unqualified",
        "source_imports":source_imports,"runtime_imports":runtime_imports,"frames":frames,"calls":calls,
        "reached_call_paths":paths,"maximum_source_stack_bytes":maximum,"maximum_checkpoint_stack_bytes":checkpoint,"maximum_completion_stack_bytes":completion,"source_stack_limit_bytes":4096}),
    )
}
