use super::*;
use crate::{DigestEntry, SystemsArtifactProvenance};
use object::{Object, ObjectSymbol};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};

fn files(rebuilt: &Rebuilt, work: &Path) -> Result<BTreeMap<String, Vec<u8>>, CompileError> {
    let mut files = BTreeMap::from([
        (
            "tail-root.t".into(),
            rebuilt.record.root_source.as_bytes().to_vec(),
        ),
        (
            "tail-source.t".into(),
            rebuilt.record.library.text.as_bytes().to_vec(),
        ),
        (
            "tail-source.modules.json".into(),
            crate::systems_library::modules::snapshot(&rebuilt.record.library.input())?,
        ),
        (
            "tail-root.binding.json".into(),
            serde_json::to_vec_pretty(&rebuilt.record).map_err(invalid)?,
        ),
    ]);
    for (index, object) in rebuilt.record.objects.iter().enumerate() {
        files.insert(
            format!("tail-{}.o", object.role),
            rebuilt.objects[index].clone(),
        );
    }
    for name in [
        "source.ll",
        "source.native.ll",
        "source.retained.ll",
        "runtime.ll",
        "runtime.native.ll",
        "owner.ll",
        "owner.native.ll",
    ] {
        files.insert(
            format!("tail-{name}"),
            fs::read(work.join(name)).map_err(invalid)?,
        );
    }
    Ok(files)
}
fn inputs(binding: &Binding) -> Vec<DigestEntry> {
    let mut inputs = vec![
        DigestEntry {
            identity: "tail-root-source".into(),
            sha256: crate::artifact::sha256(binding.root_source.as_bytes()),
        },
        DigestEntry {
            identity: format!("tail-source:{}", binding.library.source),
            sha256: crate::artifact::sha256(binding.library.text.as_bytes()),
        },
    ];
    inputs.extend(binding.library.modules.iter().map(|m| DigestEntry {
        identity: format!("tail-source-module:{}", m.source),
        sha256: crate::artifact::sha256(m.text.as_bytes()),
    }));
    inputs
}
pub(crate) fn publish_outputs(
    rebuilt: Rebuilt,
    work: &Path,
    stage: &Path,
    artifact: &mut SystemsArtifactProvenance,
) -> Result<(), CompileError> {
    if rebuilt.record.linked.is_empty() {
        return Err(invalid("linked reconstruction is required"));
    }
    for (name, bytes) in files(&rebuilt, work)? {
        fs::write(stage.join(&name), &bytes).map_err(invalid)?;
        artifact.outputs.push(DigestEntry {
            identity: name,
            sha256: crate::artifact::sha256(&bytes),
        });
    }
    artifact.inputs.extend(inputs(&rebuilt.record));
    artifact.tail_root = Some(rebuilt.record);
    artifact.inputs.sort_by(|a, b| a.identity.cmp(&b.identity));
    artifact.outputs.sort_by(|a, b| a.identity.cmp(&b.identity));
    Ok(())
}

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(std::path::PathBuf);
impl Scratch {
    fn new() -> Result<Self, CompileError> {
        for _ in 0..128 {
            let path = std::env::temp_dir().join(format!(
                "topal-tail-boot-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(e) => return Err(invalid(e)),
            }
        }
        Err(invalid("cannot reserve reconstruction workspace"))
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub(crate) fn validate_boot(
    kernel: &[u8],
    artifact: &SystemsArtifactProvenance,
) -> Result<(), CompileError> {
    let binding = artifact
        .tail_root
        .as_ref()
        .ok_or_else(|| invalid("missing tail binding"))?;
    if binding.schema != "topal.tail-root-reconstruction.x86_64/1"
        || artifact.region_library.is_some()
        || artifact.region_runtime.is_some()
    {
        return Err(invalid("mixed or incompatible root source binding"));
    }
    let file = object::File::parse(kernel).map_err(invalid)?;
    if file.entry()
        != file
            .symbol_by_name(crate::X86_SYSTEMS_KERNEL_ENTRY)
            .ok_or_else(|| invalid("missing actual bootstrap entry"))?
            .address()
    {
        return Err(invalid("ELF entry does not select the reconstructed root"));
    }
    use crate::systems_provider_object::root_region as old;
    use crate::systems_region_library as legacy;
    if artifact
        .inputs
        .iter()
        .any(|i| i.identity.starts_with("kernel-region-source"))
        || artifact
            .outputs
            .iter()
            .any(|o| o.identity.starts_with("region-library."))
        || file.symbols().filter_map(|s| s.name().ok()).any(|name| {
            name.starts_with("topal.kernel.region.")
                || name.starts_with(legacy::ROOT_FRAME_PREFIX)
                || name.starts_with(legacy::PREFIX) && !name.starts_with(legacy::ACTION_PREFIX)
                || name.starts_with("topal_x86_systems_region_")
                    && ![old::APPLY, old::FINALIZE].contains(&name)
        })
    {
        return Err(invalid("orphan legacy root source/native provenance"));
    }
    for name in [
        old::ARENA,
        old::CONTROL,
        old::HISTORY,
        legacy::EXPORT,
        legacy::ALLOCATE,
        legacy::ABORT,
        legacy::ROOT_FRAME_EXPORT,
        legacy::ROOT_FRAME_ALLOCATE,
        legacy::ROOT_FRAME_ABORT,
    ] {
        if file.symbols().any(|s| s.name().ok() == Some(name)) {
            return Err(invalid("legacy root code/storage mixed with tail input"));
        }
    }
    let tools = LlvmTools::discover(None)?;
    if binding.llvm_version != tools.version || artifact.llvm_version != tools.version {
        return Err(invalid(
            "tail packaging requires the exact qualified LLVM toolchain",
        ));
    }
    let work = Scratch::new()?;
    let mut fresh = rebuild(&binding.root_source, &binding.library, &tools, &work.0)?;
    fresh.bind(kernel)?;
    if fresh.record != *binding
        || fresh.program.bootstrap_storage.capacity_bytes != artifact.bootstrap_storage_capacity
        || fresh.program.bootstrap_storage.alignment_bytes != artifact.bootstrap_storage_alignment
        || crate::systems_artifact::semantic_trace(&fresh.program)? != artifact.semantic_trace
    {
        return Err(invalid(
            "reconstructed source/native/caller binding differs",
        ));
    }
    let plan = crate::plan_x86_64_systems_provider(&fresh.program)?;
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
            return Err(invalid("caller/target/provider metadata differs"));
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
        return Err(invalid("caller placement/entry metadata differs"));
    }
    for (identity, index) in [
        (crate::X86_SYSTEMS_ROOT_OBJECT_REVISION, 0),
        (crate::X86_SYSTEMS_PROVIDER_OBJECT_REVISION, 1),
    ] {
        let found = artifact
            .inputs
            .iter()
            .filter(|i| i.identity == identity)
            .collect::<Vec<_>>();
        if found.len() != 1 || found[0].sha256 != fresh.record.objects[index].sha256 {
            return Err(invalid("root/provider object provenance differs"));
        }
    }
    check_other_slots(&fresh.program, artifact)?;
    // Every source/artifact identity must be unique and exactly regenerated.
    let want_inputs = inputs(&fresh.record);
    let want_outputs = files(&fresh, &work.0)?;
    for (actual, expected) in [
        (
            &artifact.inputs,
            want_inputs
                .iter()
                .map(|d| (d.identity.clone(), d.sha256.clone()))
                .collect::<BTreeMap<_, _>>(),
        ),
        (
            &artifact.outputs,
            want_outputs
                .iter()
                .map(|(name, bytes)| (name.clone(), crate::artifact::sha256(bytes)))
                .collect(),
        ),
    ] {
        let found = actual
            .iter()
            .filter(|d| d.identity.starts_with("tail-"))
            .collect::<Vec<_>>();
        if found.len() != expected.len()
            || expected.iter().any(|(name, digest)| {
                found
                    .iter()
                    .filter(|d| &d.identity == name && &d.sha256 == digest)
                    .count()
                    != 1
            })
        {
            return Err(invalid("tail source/object output provenance differs"));
        }
    }
    let symbols = file
        .symbols()
        .filter_map(|s| s.name().ok())
        .filter(|n| {
            n.starts_with("topal_tail_")
                || n.starts_with("topal_x86_tail_")
                || n.starts_with("topal.tail.component.")
        })
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let expected = fresh
        .objects
        .iter()
        .map(|b| object::File::parse(b.as_slice()).map_err(invalid))
        .collect::<Result<Vec<_>, _>>()?;
    let expected = expected
        .iter()
        .flat_map(|f| f.symbols())
        .filter(|s| s.section_index().is_some())
        .filter_map(|s| s.name().ok())
        .filter(|n| {
            n.starts_with("topal_tail_")
                || n.starts_with("topal_x86_tail_")
                || n.starts_with("topal.tail.component.")
        })
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if symbols != expected {
        return Err(invalid("extra or missing tail native namespace"));
    }
    Ok(())
}

fn check_other_slots(
    program: &CompilerSystemsProgram,
    artifact: &SystemsArtifactProvenance,
) -> Result<(), CompileError> {
    for (requirement, record) in [
        (&program.time_library, &artifact.time_library),
        (&program.image_library, &artifact.image_library),
        (&program.dispatch_library, &artifact.dispatch_library),
        (
            &program.process_identity_library,
            &artifact.process_identity_library,
        ),
    ] {
        match (requirement, record) {
            (None, None) => {}
            (Some(r), Some(a))
                if r.source == a.source
                    && r.export == a.source_export
                    && r.interface.as_str() == a.interface
                    && r.stack_limit_bytes == a.stack_limit_bytes
                    && r.runtime_limit_bytes == Some(a.allocation.limit_bytes) => {}
            _ => return Err(invalid("numeric source slot differs from checked caller")),
        }
    }
    match (&program.kernel_library, &artifact.kernel_library) {
        (None, None) => {}
        (Some(r), Some(a))
            if r.source == a.source
                && r.export == a.source_export
                && r.interface.as_str() == a.interface
                && r.stack_limit_bytes == a.stack_limit_bytes => {}
        _ => return Err(invalid("policy source slot differs from checked caller")),
    }
    for (requirement, record) in [
        (
            &program.file_position_library,
            &artifact.file_position_library,
        ),
        (
            &program.ram_page_pool_library,
            &artifact.ram_page_pool_library,
        ),
        (&program.inode_mode_library, &artifact.inode_mode_library),
        (&program.namespace_library, &artifact.namespace_library),
    ] {
        match (requirement, record) {
            (None, None) => {}
            (Some(r), Some(a))
                if r.source == a.source
                    && r.export == a.source_export
                    && r.stack_limit_bytes == a.stack_limit_bytes
                    && r.runtime_limit_bytes == Some(a.allocation.numeric.limit_bytes) => {}
            _ => return Err(invalid("owned source slot differs from checked caller")),
        }
    }
    Ok(())
}
