//! Test-only census of emitted helper calls, TOPAL-SYSTEMS-USER-EXTENT-001.
//! Labels and producer locations describe emitted bytes; they grant no access
//! authority and do not prove dominance of every user load/store by admission.
use super::X86FunctionEncoder;

pub(super) fn snapshot(code: &X86FunctionEncoder) -> Option<serde_json::Value> {
    if !code.owned_user_extents {
        return None;
    }
    let mut calls = code.extent_call_sites.clone();
    for call in &mut calls {
        let offset = call["offset"].as_u64().unwrap() as usize;
        let nearest = code
            .labels
            .values()
            .filter(|&&at| at <= offset)
            .max()
            .unwrap();
        call["nearest_labels"] = serde_json::json!(
            code.labels
                .iter()
                .filter(|(_, at)| *at == nearest)
                .map(|(label, _)| label)
                .collect::<Vec<_>>()
        );
    }
    let helpers = ["runtime-readable-extent", "runtime-writable-extent"]
        .into_iter()
        .map(|name| (name.to_owned(), code.labels[name]))
        .collect::<std::collections::BTreeMap<_, _>>();
    Some(serde_json::json!({
        "schema": "topal.owned-extent-call-inventory/1",
        "function": super::X86_SYSTEMS_STATIC_IMAGE_PROCESS_ENTER_SYMBOL,
        "helpers": helpers,
        "calls": calls,
        "scope": "Selected emitted calls to the two owned extent stubs; not a complete direct-user-access inventory, proof of load/store dominance, runtime coverage or production qualification",
    }))
}
