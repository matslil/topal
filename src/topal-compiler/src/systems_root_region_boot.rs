//! Complete root source/instance/caller ownership before boot packaging.
use crate::{CompileError, SystemsArtifactProvenance};
use object::{Object, ObjectSymbol};

pub(crate) fn validate(
    kernel: &[u8],
    artifact: &SystemsArtifactProvenance,
) -> Result<(), CompileError> {
    if artifact.tail_root.is_some() {
        return crate::systems_tail_publication::validate_boot(kernel, artifact);
    }
    use crate::systems_provider_object::root_region as native;
    use crate::systems_region_library as source;
    let invalid = |message: &str| CompileError::Tool(format!("E-SYSTEMS-REGION-ROOT: {message}"));
    let file = object::File::parse(kernel).map_err(|e| invalid(&e.to_string()))?;
    if file.symbols().filter_map(|s| s.name().ok()).any(|name| {
        name.starts_with("topal.tail-inputs.")
            || name.starts_with("topal_tail_")
            || name.starts_with("topal_x86_tail_")
            || name.starts_with("topal.tail.component.")
    }) || artifact
        .inputs
        .iter()
        .chain(&artifact.outputs)
        .any(|i| i.identity.starts_with("tail-"))
    {
        return Err(invalid("orphan tail source/native binding"));
    }
    let choices = [
        (
            source::ALLOCATE,
            source::ABORT,
            source::EXPORT,
            source::PREFIX,
        ),
        (
            source::ROOT_FRAME_ALLOCATE,
            source::ROOT_FRAME_ABORT,
            source::ROOT_FRAME_EXPORT,
            source::ROOT_FRAME_PREFIX,
        ),
    ];
    let active = choices
        .iter()
        .enumerate()
        .filter(|(_, row)| file.symbol_by_name(row.2).is_some())
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let choice = match active.as_slice() {
        [] => None,
        [index] => Some(*index),
        _ => return Err(invalid("mixed root source contexts")),
    };
    let selected = choice.is_some();
    if selected != artifact.region_library.is_some()
        || selected != artifact.region_runtime.is_some()
    {
        return Err(invalid("code and complete root binding disagree"));
    }
    let row = choices[choice.unwrap_or(0)];
    let names = [
        native::APPLY,
        native::FINALIZE,
        native::ARENA,
        native::CONTROL,
        native::HISTORY,
        row.0,
        row.1,
        row.2,
    ];
    for name in names {
        if file
            .symbols()
            .filter(|s| s.is_definition() && s.name().ok() == Some(name))
            .count()
            != usize::from(selected)
        {
            return Err(invalid("incomplete or extra native root runtime ownership"));
        }
    }
    for symbol in file.symbols() {
        let name = symbol.name().map_err(|e| invalid(&e.to_string()))?;
        for (index, (allocate, abort, export, prefix)) in choices.iter().enumerate() {
            let owned_prefix = name.starts_with(prefix)
                && !(index == 0 && name.starts_with(source::ACTION_PREFIX));
            if choice != Some(index)
                && ([*allocate, *abort, *export].contains(&name)
                    || owned_prefix
                    || index == 0 && name.starts_with("topal.kernel.region."))
            {
                return Err(invalid("unselected root source namespace"));
            }
        }
        if name.starts_with("topal_x86_systems_region_") && !names.contains(&name) {
            return Err(invalid("extra native root invocation symbol"));
        }
    }
    let Some(region) = &artifact.region_library else {
        if artifact
            .inputs
            .iter()
            .any(|i| i.identity.starts_with("kernel-region-source"))
            || artifact
                .outputs
                .iter()
                .any(|o| o.identity.starts_with("region-library."))
        {
            return Err(invalid("orphan root source binding"));
        }
        return Ok(());
    };
    crate::systems_library::modules::validate_binding(
        &region.modules,
        &region.module_snapshot_sha256,
        &artifact.inputs,
        &artifact.outputs,
        "kernel-region-source-module:",
        "region-library.modules.json",
    )?;
    if artifact
        .inputs
        .iter()
        .filter(|i| {
            i.identity == format!("kernel-region-source:{}", region.source)
                && i.sha256 == region.source_sha256
        })
        .count()
        != 1
    {
        return Err(invalid("source identity does not match its owned entry"));
    }
    for (name, digest) in [
        ("region-library.t", &region.source_sha256),
        ("region-library.ll", &region.llvm_sha256),
        ("region-library.retained.ll", &region.retained_llvm_sha256),
        ("region-library.o", &region.object_sha256),
    ] {
        if artifact
            .outputs
            .iter()
            .filter(|o| o.identity == name && &o.sha256 == digest)
            .count()
            != 1
        {
            return Err(invalid("source/object output binding mismatch"));
        }
    }
    let expected = native::inspect_linked(kernel, region, artifact.bootstrap_storage_capacity)?;
    if artifact.region_runtime.as_ref() != Some(&expected) {
        return Err(invalid("root runtime or combined bounds mismatch"));
    }
    Ok(())
}
