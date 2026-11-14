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

pub(super) fn byte_snapshot(code: &X86FunctionEncoder) -> Option<serde_json::Value> {
    if !code.owned_user_extents {
        return None;
    }
    let mut locations = Vec::new();
    let mut indices = std::collections::BTreeMap::new();
    let mut spans = Vec::new();
    let mut next = 0;
    for &(start, end, location) in &code.byte_origins {
        assert_eq!(start, next, "unrecorded emitter bytes");
        assert!(end > start && end <= code.bytes.len());
        let key = (location.file(), location.line(), location.column());
        let index = *indices.entry(key).or_insert_with(|| {
            let index = locations.len();
            locations.push(serde_json::json!({
                "file": location.file(), "line": location.line(), "column": location.column(),
            }));
            index
        });
        spans.push([start, end, index]);
        next = end;
    }
    assert_eq!(next, code.bytes.len(), "unrecorded emitter suffix");
    Some(serde_json::json!({
        "schema": "topal.owned-process-byte-origins/1",
        "function": super::X86_SYSTEMS_STATIC_IMAGE_PROCESS_ENTER_SYMBOL,
        "locations": locations, "spans": spans, "labels": code.labels,
        "fixups": &code.fixups,
        "scope": "Test-only emission locations covering selected function bytes, including data and fixup fields. Locations describe the writer; they do not prove pointer authority, admission dominance, source-to-native conformance or access lifetime",
    }))
}
