//! Component call closure; no actual owning-gateway or fatal recovery proof.
use crate::CompileError;
use object::{Object, ObjectSymbol, SymbolKind};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn inspect(
    controller: &[u8],
    target: &[u8],
    owner: &serde_json::Value,
) -> Result<serde_json::Value, CompileError> {
    inspect_objects(controller, target, None, None, owner)
}
pub(crate) fn inspect_gateway(
    controller: &[u8],
    target: &[u8],
    dispatch: &[u8],
    owner: &serde_json::Value,
) -> Result<serde_json::Value, CompileError> {
    inspect_objects(controller, target, Some(dispatch), None, owner)
}
pub(crate) fn inspect_user_extents(
    controller: &[u8],
    target: &[u8],
    dispatch: &[u8],
    extent: &[u8],
    owner: &serde_json::Value,
) -> Result<serde_json::Value, CompileError> {
    inspect_objects(controller, target, Some(dispatch), Some(extent), owner)
}
fn inspect_objects(
    controller: &[u8],
    target: &[u8],
    dispatch: Option<&[u8]>,
    extent: Option<&[u8]>,
    owner: &serde_json::Value,
) -> Result<serde_json::Value, CompileError> {
    let invalid = |s: &str| CompileError::Tool(format!("E-SYSTEMS-FRAME-POLICY-BINDING: {s}"));
    let controller = object::File::parse(controller).map_err(|e| invalid(&e.to_string()))?;
    let target = object::File::parse(target).map_err(|e| invalid(&e.to_string()))?;
    let dispatch = dispatch
        .map(object::File::parse)
        .transpose()
        .map_err(|e| invalid(&e.to_string()))?;
    let extent = extent
        .map(object::File::parse)
        .transpose()
        .map_err(|e| invalid(&e.to_string()))?;
    let mut objects = vec![&controller, &target];
    if let Some(file) = dispatch.as_ref() {
        objects.push(file);
    }
    if let Some(file) = extent.as_ref() {
        objects.push(file);
    }
    let mut frames = BTreeMap::new();
    let mut calls = BTreeMap::new();
    let mut sets = Vec::new();
    for object in &objects {
        let names = object
            .symbols()
            .filter(|s| s.is_definition() && s.kind() == SymbolKind::Text)
            .map(|s| s.name().unwrap().to_owned())
            .collect::<BTreeSet<_>>();
        let found = crate::systems_library::stack_sizes(object)?;
        if found.keys().cloned().collect::<BTreeSet<_>>() != names {
            return Err(invalid("incomplete frame stack metadata"));
        }
        for (name, bytes) in found {
            if frames.insert(name, bytes).is_some() {
                return Err(invalid("duplicate frame function"));
            }
        }
        sets.push(names);
    }
    let mut names = frames
        .keys()
        .map(String::as_str)
        .chain(["topal_tail_owner_invoke_v1"])
        .collect::<Vec<_>>();
    if dispatch.is_some() {
        names.push("topal_x86_frame_gateway_abort_v1");
    }
    for (index, object) in objects.into_iter().enumerate() {
        calls.extend(crate::systems_numeric_library::library_object_calls(
            object,
            &sets[index],
            &names,
        )?);
    }
    let raw_frames = frames.clone();
    // LLVM's inline-asm stack delta is invisible to .stack_sizes. The target
    // register observation contains one balanced pushfq/popq pair, adding 8.
    let bootstrap = frames
        .get_mut("t.bootstrap")
        .ok_or_else(|| invalid("missing architecture observation frame"))?;
    *bootstrap = bootstrap
        .checked_add(8)
        .ok_or_else(|| invalid("bootstrap stack overflow"))?;
    let inner = owner["combined_normal_stack_bytes"]
        .as_u64()
        .ok_or_else(|| invalid("missing owner bound"))?;
    let mut cache = BTreeMap::from([("topal_tail_owner_invoke_v1".to_owned(), inner)]);
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
        let error = || {
            CompileError::Tool(
                "E-SYSTEMS-FRAME-POLICY-BINDING: cyclic, missing or overflowing native path".into(),
            )
        };
        if !active.insert(name.into()) {
            return Err(error());
        }
        let children = calls
            .get(name)
            .ok_or_else(error)?
            .iter()
            .map(|n| visit(n, frames, calls, active, cache))
            .collect::<Result<Vec<_>, _>>()?;
        let child = children.into_iter().max().unwrap_or(0);
        let value = frames
            .get(name)
            .and_then(|n| n.checked_add(8))
            .and_then(|n| n.checked_add(child))
            .ok_or_else(error)?;
        active.remove(name);
        cache.insert(name.into(), value);
        Ok(value)
    }
    let callback = if dispatch.is_some() {
        cache.insert("topal_x86_frame_gateway_abort_v1".into(), 0);
        let bytes = visit(
            "topal_tail_boundary_fatal_v1",
            &frames,
            &calls,
            &mut BTreeSet::new(),
            &mut cache,
        )?;
        cache.insert(
            "topal_tail_owner_invoke_v1".into(),
            inner
                .checked_add(bytes)
                .ok_or_else(|| invalid("callback stack overflow"))?,
        );
        bytes
    } else {
        0
    };
    let mut roots = BTreeMap::new();
    for name in [
        "topal_frame_allocator_allocate_v1",
        "topal_frame_allocator_release_v1",
        "topal_frame_allocator_pin_v1",
        "topal_frame_allocator_resolve_v1",
        "topal_x86_frame_allocator_create_v1",
    ] {
        let bytes = visit(name, &frames, &calls, &mut BTreeSet::new(), &mut cache)?;
        if bytes > 4096 {
            return Err(invalid(
                "component path exceeds 4096 bytes before the actual owning gateway",
            ));
        }
        roots.insert(name, bytes);
    }
    if dispatch.is_some() {
        let bytes = visit(
            "topal_frame_kernel_dispatch_v1",
            &frames,
            &calls,
            &mut BTreeSet::new(),
            &mut cache,
        )?;
        roots.insert("topal_frame_kernel_dispatch_v1", bytes);
    }
    let mut result = serde_json::json!({"scope":"native controller and bootstrap target normal paths with standalone source owner contribution; owning gateway, entry fixture, complete fatal recovery/caller retirement and publication excluded","raw_frames":raw_frames,"frames_with_inline_push":frames,"calls":calls,"normal_owner_contribution":inner,"paths":cache,"roots":roots});
    if dispatch.is_some() {
        result["scope"] = serde_json::json!(
            "native dispatch/controller/target paths including a conservative owner fatal-callback contribution; machine gateway retirement, entry fixture and publication require separate proof"
        );
        result["fatal_callback_contribution"] = serde_json::json!(callback);
    }
    if extent.is_some() {
        result["user_extent_component"] = serde_json::json!({"included_in_complete_dispatch_path":true,
            "scope":"native walk/target/caller path costs; actual owning entry, physical retirement and runtime qualification remain separate"});
    }
    Ok(result)
}
