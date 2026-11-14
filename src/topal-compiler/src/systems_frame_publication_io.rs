use super::*;
use crate::{DigestEntry, SystemsArtifactProvenance};
use std::collections::BTreeSet;

fn files(fresh: &Rebuilt, work: &Path) -> Result<BTreeMap<String, Vec<u8>>, CompileError> {
    let mut files = BTreeMap::from([
        (
            "frame-root.t".into(),
            fresh.record.root_source.as_bytes().to_vec(),
        ),
        (
            "frame-policy.t".into(),
            fresh.record.library.text.as_bytes().to_vec(),
        ),
        (
            "frame-policy.modules.json".into(),
            crate::systems_library::modules::snapshot(&fresh.record.library.input())?,
        ),
        (
            "frame-root.binding.json".into(),
            serde_json::to_vec_pretty(&fresh.record).map_err(invalid)?,
        ),
    ]);
    for (record, bytes) in fresh.record.objects.iter().zip(&fresh.objects) {
        files.insert(format!("frame-{}.o", record.role), bytes.clone());
    }
    for (role, source) in &fresh.record.other_sources {
        files.insert(format!("frame-{role}.t"), source.text.as_bytes().to_vec());
        files.insert(
            format!("frame-{role}.modules.json"),
            crate::systems_library::modules::snapshot(&source.input())?,
        );
    }
    for role in [
        "source",
        "runtime",
        "owner",
        "frame-controller",
        "frame-target",
        "frame-dispatch",
    ] {
        for suffix in ["ll", "native.ll"] {
            let name = format!("{role}.{suffix}");
            files.insert(
                format!("frame-{name}"),
                fs::read(work.join(name)).map_err(invalid)?,
            );
        }
    }
    files.insert(
        "frame-source.retained.ll".into(),
        fs::read(work.join("source.retained.ll")).map_err(invalid)?,
    );
    Ok(files)
}
fn inputs(binding: &Binding) -> Vec<DigestEntry> {
    let mut result = vec![DigestEntry {
        identity: "frame-root-source".into(),
        sha256: crate::artifact::sha256(binding.root_source.as_bytes()),
    }];
    for (role, source) in std::iter::once(("policy", &binding.library)).chain(
        binding
            .other_sources
            .iter()
            .map(|(role, source)| (role.as_str(), source)),
    ) {
        result.push(DigestEntry {
            identity: format!("frame-source:{role}:{}", source.source),
            sha256: crate::artifact::sha256(source.text.as_bytes()),
        });
        result.extend(source.modules.iter().map(|m| DigestEntry {
            identity: format!("frame-module:{role}:{}", m.source),
            sha256: crate::artifact::sha256(m.text.as_bytes()),
        }));
    }
    result
}
pub(crate) fn publish_outputs(
    fresh: Rebuilt,
    work: &Path,
    stage: &Path,
    artifact: &mut SystemsArtifactProvenance,
) -> Result<(), CompileError> {
    if fresh.record.linked.is_empty()
        || artifact.tail_root.is_some()
        || artifact.frame_root.is_some()
    {
        return Err(invalid("complete unmixed frame reconstruction required"));
    }
    for (name, bytes) in files(&fresh, work)? {
        fs::write(stage.join(&name), &bytes).map_err(invalid)?;
        artifact.outputs.push(DigestEntry {
            identity: name,
            sha256: crate::artifact::sha256(&bytes),
        });
    }
    artifact.inputs.extend(inputs(&fresh.record));
    artifact.frame_root = Some(fresh.record);
    artifact.inputs.sort_by(|a, b| a.identity.cmp(&b.identity));
    artifact.outputs.sort_by(|a, b| a.identity.cmp(&b.identity));
    Ok(())
}

pub(crate) fn validate_boot(
    kernel: &[u8],
    artifact: &SystemsArtifactProvenance,
) -> Result<(), CompileError> {
    let binding = artifact
        .frame_root
        .as_ref()
        .ok_or_else(|| invalid("missing frame binding"))?;
    let tools = LlvmTools::discover(None)?;
    if artifact.tail_root.is_some()
        || artifact
            .inputs
            .iter()
            .chain(&artifact.outputs)
            .any(|d| d.identity.starts_with("tail-"))
        || binding.schema != "topal.frame-root-reconstruction.x86_64/1"
        || binding.llvm_version != tools.version
        || artifact.llvm_version != tools.version
    {
        return Err(invalid("mixed source binding or different toolchain"));
    }
    let scratch = crate::systems_tail_publication::Scratch::new()?;
    let mut fresh = rebuild(&binding.root_source, &binding.library, &tools, &scratch.0)?;
    // This is the complete kernel route: absent other snapshots are not a
    // request to validate only the standalone frame component.
    fresh.complete_libraries(&binding.other_sources, &tools, &scratch.0)?;
    fresh.bind(kernel)?;
    if fresh.record != *binding
        || crate::systems_artifact::semantic_trace(&fresh.program)? != artifact.semantic_trace
        || artifact.bootstrap_storage_capacity != fresh.program.bootstrap_storage.capacity_bytes
        || artifact.bootstrap_storage_alignment != fresh.program.bootstrap_storage.alignment_bytes
    {
        return Err(invalid(
            "fresh complete source/native/load/caller binding differs",
        ));
    }
    let checked =
        crate::systems_frame_policy_binding::check(&binding.root_source, &binding.library.input())?;
    let plan = crate::systems_provider::plan_frame_systems_provider(&checked)?;
    for (actual, expected) in [
        (artifact.target.as_str(), plan.target),
        (artifact.board.as_str(), plan.board),
        (artifact.profile.as_str(), plan.profile),
        (artifact.machine_cpu.as_str(), plan.machine_cpu_model),
        (artifact.codegen_cpu.as_str(), plan.codegen_cpu),
        (artifact.data_layout.as_str(), plan.data_layout),
        (artifact.object_format.as_str(), plan.object_format),
        (artifact.relocation_model.as_str(), plan.relocation_model),
        (artifact.code_model.as_str(), plan.code_model),
        (
            artifact.platform_abi.as_str(),
            crate::X86_SYSTEMS_PLATFORM_ABI,
        ),
        (
            artifact.root_object.as_str(),
            crate::X86_SYSTEMS_ROOT_OBJECT_REVISION,
        ),
        (
            artifact.provider_object.as_str(),
            crate::X86_SYSTEMS_PROVIDER_OBJECT_REVISION,
        ),
        (
            artifact.entry_symbol.as_str(),
            crate::X86_SYSTEMS_KERNEL_ENTRY,
        ),
    ] {
        if actual != expected {
            return Err(invalid("target/provider/caller metadata differs"));
        }
    }
    if artifact.placements != crate::systems_artifact::linked_placements(kernel)?
        || artifact.exception_entries
            != [
                crate::X86_SYSTEMS_DEBUG_BREAK_ENTRY,
                crate::X86_SYSTEMS_LOCAL_NOTIFICATION_ENTRY,
                crate::X86_SYSTEMS_DEADLINE_ENTRY,
                crate::X86_SYSTEMS_USER_COPY_FAULT_SYMBOL,
            ]
    {
        return Err(invalid("entry/placement metadata differs"));
    }
    for (identity, index) in [
        (crate::X86_SYSTEMS_ROOT_OBJECT_REVISION, 0),
        (crate::X86_SYSTEMS_PROVIDER_OBJECT_REVISION, 1),
    ] {
        let found = artifact
            .inputs
            .iter()
            .filter(|d| d.identity == identity)
            .collect::<Vec<_>>();
        if found.len() != 1 || found[0].sha256 != fresh.record.objects[index].sha256 {
            return Err(invalid("actual root/provider input differs"));
        }
    }
    crate::systems_tail_publication::check_other_slots(&fresh.program, artifact)?;
    super::sources::validate(&fresh.record, artifact)?;
    match (&fresh.program.region_library, &artifact.region_library) {
        (None, None) => {}
        (Some(r), Some(a))
            if r.source == a.source
                && r.export == a.source_export
                && r.stack_limit_bytes == a.stack_limit_bytes => {}
        _ => return Err(invalid("root-region source slot differs")),
    }
    for (role, digest) in [
        (
            "library-kernel",
            artifact.kernel_library.as_ref().map(|a| &a.object_sha256),
        ),
        (
            "library-time",
            artifact.time_library.as_ref().map(|a| &a.object_sha256),
        ),
        (
            "library-image",
            artifact.image_library.as_ref().map(|a| &a.object_sha256),
        ),
        (
            "library-dispatch",
            artifact.dispatch_library.as_ref().map(|a| &a.object_sha256),
        ),
        (
            "library-process-identity",
            artifact
                .process_identity_library
                .as_ref()
                .map(|a| &a.object_sha256),
        ),
        (
            "library-region",
            artifact.region_library.as_ref().map(|a| &a.object_sha256),
        ),
        (
            "library-file-position",
            artifact
                .file_position_library
                .as_ref()
                .map(|a| &a.object_sha256),
        ),
        (
            "library-ram-page-pool",
            artifact
                .ram_page_pool_library
                .as_ref()
                .map(|a| &a.object_sha256),
        ),
        (
            "library-inode-mode",
            artifact
                .inode_mode_library
                .as_ref()
                .map(|a| &a.object_sha256),
        ),
        (
            "library-namespace",
            artifact
                .namespace_library
                .as_ref()
                .map(|a| &a.object_sha256),
        ),
    ] {
        if digest != binding.other_original_objects.get(role) {
            return Err(invalid("other library object provenance differs"));
        }
    }
    let expected_outputs = files(&fresh, &scratch.0)?;
    let expected_inputs = inputs(&fresh.record);
    for (actual, wanted) in [
        (
            &artifact.inputs,
            expected_inputs
                .into_iter()
                .map(|d| (d.identity, d.sha256))
                .collect::<BTreeMap<_, _>>(),
        ),
        (
            &artifact.outputs,
            expected_outputs
                .into_iter()
                .map(|(n, b)| (n, crate::artifact::sha256(&b)))
                .collect(),
        ),
    ] {
        let mut seen = BTreeSet::new();
        if actual.iter().any(|d| !seen.insert(d.identity.as_str())) {
            return Err(invalid("duplicate artifact identity"));
        }
        let found = actual
            .iter()
            .filter(|d| d.identity.starts_with("frame-"))
            .map(|d| (d.identity.clone(), d.sha256.clone()))
            .collect::<BTreeMap<_, _>>();
        if found != wanted {
            return Err(invalid("frame source/object output association differs"));
        }
    }
    let file = object::File::parse(kernel).map_err(invalid)?;
    let relevant = |name: &str| {
        name.starts_with("topal_frame_")
            || name.starts_with("topal_x86_frame_")
            || name.starts_with("topal_tail_")
            || name.starts_with("topal_x86_tail_")
            || name.starts_with("topal.tail-inputs.")
    };
    let found = file
        .symbols()
        .filter_map(|s| s.name().ok())
        .filter(|n| relevant(n))
        .collect::<BTreeSet<_>>();
    let parsed = fresh
        .objects
        .iter()
        .map(|b| object::File::parse(b.as_slice()).map_err(invalid))
        .collect::<Result<Vec<_>, _>>()?;
    let expected = parsed
        .iter()
        .flat_map(|f| f.symbols())
        .filter(|s| s.section_index().is_some())
        .filter_map(|s| s.name().ok())
        .filter(|n| relevant(n))
        .collect::<BTreeSet<_>>();
    if found != expected {
        return Err(invalid("extra or missing frame native namespace"));
    }
    crate::systems_root_region_boot::validate_other_root(kernel, artifact)
}
