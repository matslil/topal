//! Complete action/source/runtime/data binding before boot packaging.
use crate::{CompileError, SystemsArtifactProvenance};
use object::{Object, ObjectSymbol};

pub(crate) fn validate(
    kernel: &[u8],
    artifact: &SystemsArtifactProvenance,
) -> Result<(), CompileError> {
    use crate::systems_provider_object::inode_region as native;
    use crate::systems_region_library as source;
    let invalid = |message: &str| CompileError::Tool(format!("E-SYSTEMS-INODE-MODE: {message}"));
    let file = object::File::parse(kernel).map_err(|e| invalid(&e.to_string()))?;
    let choices = [(
        source::INODE_FRAME_ALLOCATE,
        source::INODE_FRAME_ABORT,
        source::INODE_FRAME_EXPORT,
        source::INODE_FRAME_PREFIX,
    )];
    let active = choices
        .iter()
        .enumerate()
        .filter(|(_, choice)| file.symbol_by_name(choice.2).is_some())
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let choice = match active.as_slice() {
        [] => None,
        [index] => Some(*index),
        _ => return Err(invalid("mixed persistent action transport profiles")),
    };
    let selected = choice.is_some();
    if selected != artifact.inode_mode_library.is_some()
        || selected != artifact.inode_mode_runtime.is_some()
    {
        return Err(invalid(
            "code and complete persistent action binding disagree",
        ));
    }
    let names = choices[choice.unwrap_or(0)];
    let native_names = [
        native::APPLY,
        native::FINALIZE,
        native::ARENA,
        native::CONTROL,
        native::HISTORY,
        names.0,
        names.1,
        names.2,
    ];
    for name in native_names {
        if file
            .symbols()
            .filter(|s| s.is_definition() && s.name().ok() == Some(name))
            .count()
            != usize::from(selected)
        {
            return Err(invalid(
                "incomplete or extra native action runtime ownership",
            ));
        }
    }
    for symbol in file.symbols() {
        let name = symbol.name().map_err(|e| invalid(&e.to_string()))?;
        for (index, (allocate, abort, export, prefix)) in choices.iter().enumerate() {
            if choice != Some(index)
                && ([*allocate, *abort, *export].contains(&name) || name.starts_with(prefix))
            {
                return Err(invalid("unselected action transport namespace"));
            }
        }
        if name.starts_with("topal_x86_systems_inode_mode_") && !native_names.contains(&name) {
            return Err(invalid("extra native action runtime symbol"));
        }
    }
    let Some(action) = &artifact.inode_mode_library else {
        if artifact
            .outputs
            .iter()
            .any(|o| o.identity.starts_with("inode-mode-library."))
            || artifact
                .inputs
                .iter()
                .any(|i| i.identity.starts_with("kernel-inode-mode-source"))
        {
            return Err(invalid(
                "extra action source snapshot without a declared binding",
            ));
        }
        return Ok(());
    };
    let source_outputs = [
        "inode-mode-library.t",
        "inode-mode-library.ll",
        "inode-mode-library.retained.ll",
        "inode-mode-library.o",
        "inode-mode-library.modules.json",
    ];
    let output_names = artifact
        .outputs
        .iter()
        .filter(|entry| entry.identity.starts_with("inode-mode-library."))
        .map(|entry| entry.identity.as_str())
        .collect::<Vec<_>>();
    if output_names.len() != source_outputs.len()
        || source_outputs.iter().any(|expected| {
            output_names
                .iter()
                .filter(|actual| *actual == expected)
                .count()
                != 1
        })
        || artifact
            .inputs
            .iter()
            .filter(|entry| entry.identity.starts_with("kernel-inode-mode-source"))
            .count()
            != action.modules.len() + 1
    {
        return Err(invalid(
            "extra, missing or duplicate inode source snapshot identity",
        ));
    }
    crate::systems_library::modules::validate_binding(
        &action.modules,
        &action.module_snapshot_sha256,
        &artifact.inputs,
        &artifact.outputs,
        "kernel-inode-mode-source-module:",
        "inode-mode-library.modules.json",
    )?;
    if artifact
        .inputs
        .iter()
        .filter(|i| {
            i.identity == format!("kernel-inode-mode-source:{}", action.source)
                && i.sha256 == action.source_sha256
        })
        .count()
        != 1
    {
        return Err(invalid(
            "source identity does not match its persistent action entry",
        ));
    }
    for (name, digest) in [
        ("inode-mode-library.t", &action.source_sha256),
        ("inode-mode-library.ll", &action.llvm_sha256),
        (
            "inode-mode-library.retained.ll",
            &action.retained_llvm_sha256,
        ),
        ("inode-mode-library.o", &action.object_sha256),
    ] {
        if artifact
            .outputs
            .iter()
            .filter(|o| o.identity == name && &o.sha256 == digest)
            .count()
            != 1
        {
            return Err(invalid("action source/object output binding mismatch"));
        }
    }
    let expected = native::inspect_linked(kernel, action)?;
    if artifact.inode_mode_runtime.as_ref() != Some(&expected) {
        return Err(invalid(
            "persistent action runtime/caller/source/data or bounds mismatch",
        ));
    }
    Ok(())
}
