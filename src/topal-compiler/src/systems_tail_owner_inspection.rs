//! Controller frames plus explicit native-boundary stack costs, not kernel proof.
use crate::CompileError;
use object::{Object, ObjectSymbol, SymbolKind};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn inspect(
    bytes: &[u8],
    inner: &serde_json::Value,
) -> Result<serde_json::Value, CompileError> {
    let invalid = |text: String| CompileError::Tool(format!("E-SYSTEMS-TAIL-OWNER: {text}"));
    let file = object::File::parse(bytes).map_err(|e| invalid(e.to_string()))?;
    let names = file
        .symbols()
        .filter(|s| s.is_definition() && s.kind() == SymbolKind::Text)
        .map(|s| s.name().unwrap().to_owned())
        .collect::<BTreeSet<_>>();
    let imports = file
        .symbols()
        .filter(|s| s.is_undefined())
        .filter_map(|s| s.name().ok())
        .filter(|s| !s.is_empty())
        .collect::<BTreeSet<_>>();
    let expected = BTreeSet::from([
        "topal_tail_step_component_v1",
        "topal_x86_tail_boundary_active_v1",
        "topal_x86_tail_boundary_call_v1",
        "topal_x86_tail_checkpoint_call_v1",
        "topal_x86_tail_complete_call_v1",
        "topal_x86_tail_boundary_fail_v1",
    ]);
    if imports != expected {
        return Err(invalid(format!(
            "unexpected controller imports {imports:?}"
        )));
    }
    let frames = crate::systems_library::stack_sizes(&file)?;
    if frames.keys().cloned().collect::<BTreeSet<_>>() != names {
        return Err(invalid("incomplete controller stack metadata".into()));
    }
    let calls = crate::systems_numeric_library::library_object_calls(
        &file,
        &names,
        &expected.into_iter().collect::<Vec<_>>(),
    )?;
    fn visit(
        name: &str,
        frames: &BTreeMap<String, u64>,
        calls: &BTreeMap<String, BTreeSet<String>>,
        active: &mut BTreeSet<String>,
        cache: &mut BTreeMap<String, u64>,
    ) -> Result<u64, CompileError> {
        if let Some(n) = cache.get(name) {
            return Ok(*n);
        }
        if !active.insert(name.into()) {
            return Err(CompileError::Tool("cyclic owner controller calls".into()));
        }
        let child = calls
            .get(name)
            .ok_or_else(|| CompileError::Tool(format!("missing owner callee {name}")))?
            .iter()
            .map(|s| visit(s, frames, calls, active, cache))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .max()
            .unwrap_or(0);
        let total = frames
            .get(name)
            .and_then(|n| n.checked_add(8))
            .and_then(|n| n.checked_add(child))
            .ok_or_else(|| {
                CompileError::Tool(format!("missing or overflowing owner frame {name}"))
            })?;
        active.remove(name);
        cache.insert(name.into(), total);
        Ok(total)
    }
    // The return word + six saved GPRs occupy 56 bytes on the owner stack.
    // Source/checkpoint/completion bodies use the separate sequential stack.
    let mut native: BTreeMap<String, u64> = BTreeMap::from([
        ("topal_x86_tail_boundary_call_v1".into(), 56),
        ("topal_x86_tail_checkpoint_call_v1".into(), 56),
        ("topal_x86_tail_complete_call_v1".into(), 56),
        ("topal_x86_tail_boundary_fail_v1".into(), 56),
    ]);
    let mut combined = native.clone();
    for (symbol, key) in [
        (
            "topal_x86_tail_boundary_call_v1",
            "maximum_source_stack_bytes",
        ),
        (
            "topal_x86_tail_checkpoint_call_v1",
            "maximum_checkpoint_stack_bytes",
        ),
        (
            "topal_x86_tail_complete_call_v1",
            "maximum_completion_stack_bytes",
        ),
    ] {
        let prior = combined[symbol];
        let bound = inner[key]
            .as_u64()
            .and_then(|n| prior.checked_add(n))
            .ok_or_else(|| invalid("invalid inner stack bound".into()))?;
        combined.insert(symbol.into(), bound);
    }
    let own = visit(
        "topal_tail_owner_invoke_v1",
        &frames,
        &calls,
        &mut BTreeSet::new(),
        &mut native,
    )?;
    let joined = visit(
        "topal_tail_owner_invoke_v1",
        &frames,
        &calls,
        &mut BTreeSet::new(),
        &mut combined,
    )?;
    if joined > 4096 {
        return Err(invalid(format!(
            "controller/inner normal path {joined} exceeds declared stack reserve"
        )));
    }
    Ok(
        serde_json::json!({"scope":"controller and normal boundary/body call paths; fatal handler, actual kernel caller/context and complete caller retirement unqualified","imports":imports,"frames":frames,"calls":calls,"owner_stack_paths":native,"combined_paths":combined,"owner_stack_bytes":own,"combined_normal_stack_bytes":joined}),
    )
}
