//! Complete action/source/runtime/data binding before boot packaging.
use crate::{CompileError, SystemsArtifactProvenance};
use object::{Object, ObjectSymbol};

pub(crate) fn validate(
    kernel: &[u8],
    artifact: &SystemsArtifactProvenance,
) -> Result<(), CompileError> {
    use crate::systems_provider_object::namespace_region as native;
    use crate::systems_region_library as source;
    let invalid = |message: &str| CompileError::Tool(format!("E-SYSTEMS-NAMESPACE: {message}"));
    let file = object::File::parse(kernel).map_err(|e| invalid(&e.to_string()))?;
    let choices = [(
        source::NAMESPACE_FRAME_ALLOCATE,
        source::NAMESPACE_FRAME_ABORT,
        source::NAMESPACE_FRAME_EXPORT,
        source::NAMESPACE_FRAME_PREFIX,
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
    if selected != artifact.namespace_library.is_some()
        || selected != artifact.namespace_runtime.is_some()
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
        native::RECOVERY,
        native::HALT,
        crate::systems_provider_object::namespace_oracle::EXPECTED,
        crate::systems_provider_object::namespace_oracle::PREPARE,
        crate::systems_provider_object::namespace_oracle::CHECK,
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
        if name.starts_with("topal_x86_systems_namespace_") && !native_names.contains(&name) {
            return Err(invalid("extra native action runtime symbol"));
        }
    }
    let Some(action) = &artifact.namespace_library else {
        if artifact
            .outputs
            .iter()
            .any(|o| o.identity.starts_with("namespace-library."))
            || artifact
                .inputs
                .iter()
                .any(|i| i.identity.starts_with("kernel-namespace-source"))
        {
            return Err(invalid(
                "extra action source snapshot without a declared binding",
            ));
        }
        return Ok(());
    };
    let source_outputs = [
        "namespace-library.t",
        "namespace-library.ll",
        "namespace-library.retained.ll",
        "namespace-library.o",
        "namespace-library.modules.json",
    ];
    let output_names = artifact
        .outputs
        .iter()
        .filter(|entry| entry.identity.starts_with("namespace-library."))
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
            .filter(|entry| entry.identity.starts_with("kernel-namespace-source"))
            .count()
            != action.modules.len() + 1
    {
        return Err(invalid(
            "extra, missing or duplicate namespace source snapshot identity",
        ));
    }
    crate::systems_library::modules::validate_binding(
        &action.modules,
        &action.module_snapshot_sha256,
        &artifact.inputs,
        &artifact.outputs,
        "kernel-namespace-source-module:",
        "namespace-library.modules.json",
    )?;
    if artifact
        .inputs
        .iter()
        .filter(|i| {
            i.identity == format!("kernel-namespace-source:{}", action.source)
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
        ("namespace-library.t", &action.source_sha256),
        ("namespace-library.ll", &action.llvm_sha256),
        (
            "namespace-library.retained.ll",
            &action.retained_llvm_sha256,
        ),
        ("namespace-library.o", &action.object_sha256),
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
    if artifact.inode_mode_runtime.is_none()
        || artifact.ram_page_pool_runtime.is_none()
        || artifact
            .file_position_library
            .as_ref()
            .and_then(|r| r.instance.as_deref())
            != Some(source::FILE_INSTANCE)
    {
        return Err(invalid(
            "namespace action lacks the required exclusive filesystem controllers",
        ));
    }
    let namespace_entries = artifact.volatile_files.namespace_entries.is_some();
    if namespace_entries
        && !artifact
            .kernel_library
            .as_ref()
            .is_some_and(|library| library.interface == "file-open-decision-v1")
    {
        return Err(invalid(
            "V6 requires its actual named-file open-plan binding",
        ));
    }
    let expected = native::inspect_linked(kernel, action, namespace_entries)?;
    if let Some(entries) = &artifact.volatile_files.namespace_entries {
        if action
            .maximum_stack_bytes
            .checked_add(entries.namespace_caller_stack_bytes)
            .is_none_or(|bound| bound > 4096)
        {
            return Err(invalid(
                "complete namespace source/entry caller exceeds its stack reserve",
            ));
        }
    }
    if artifact.namespace_runtime.as_ref() != Some(&expected) {
        return Err(invalid(
            "persistent action runtime/caller/source/data or bounds mismatch",
        ));
    }
    Ok(())
}
