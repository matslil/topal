//! Complete action/source/runtime/data binding before boot packaging.
use crate::{CompileError, SystemsArtifactProvenance};
use object::{Object, ObjectSymbol};

pub(crate) fn validate(
    kernel: &[u8],
    artifact: &SystemsArtifactProvenance,
) -> Result<(), CompileError> {
    use crate::systems_provider_object::file_region as native;
    use crate::systems_region_library as source;
    let invalid = |message: &str| CompileError::Tool(format!("E-SYSTEMS-REGION-ACTION: {message}"));
    let file = object::File::parse(kernel).map_err(|e| invalid(&e.to_string()))?;
    let legacy = file.symbol_by_name(source::ACTION_EXPORT).is_some();
    let framed = file.symbol_by_name(source::FRAME_EXPORT).is_some();
    if legacy && framed {
        return Err(invalid("mixed persistent action transport profiles"));
    }
    let selected = legacy || framed;
    if selected != artifact.file_position_library.is_some()
        || selected != artifact.file_position_runtime.is_some()
    {
        return Err(invalid(
            "code and complete persistent action binding disagree",
        ));
    }
    let source_names = if framed {
        [
            source::FRAME_ALLOCATE,
            source::FRAME_ABORT,
            source::FRAME_EXPORT,
        ]
    } else {
        [
            source::ACTION_ALLOCATE,
            source::ACTION_ABORT,
            source::ACTION_EXPORT,
        ]
    };
    let unselected_names = if framed {
        [
            source::ACTION_ALLOCATE,
            source::ACTION_ABORT,
            source::ACTION_EXPORT,
        ]
    } else {
        [
            source::FRAME_ALLOCATE,
            source::FRAME_ABORT,
            source::FRAME_EXPORT,
        ]
    };
    let native_names = [
        native::APPLY,
        native::FINALIZE,
        native::ARENA,
        native::CONTROL,
        native::HISTORY,
        source_names[0],
        source_names[1],
        source_names[2],
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
        if unselected_names.contains(&name)
            || ((!framed || !selected) && name.starts_with(source::FRAME_PREFIX))
            || ((!legacy || !selected) && name.starts_with(source::ACTION_PREFIX))
        {
            return Err(invalid("unselected action transport namespace"));
        }
        if name.starts_with("topal_x86_systems_file_position_") && !native_names.contains(&name) {
            return Err(invalid("extra native action runtime symbol"));
        }
    }
    let Some(action) = &artifact.file_position_library else {
        if artifact
            .outputs
            .iter()
            .any(|o| o.identity.starts_with("file-position-library."))
            || artifact
                .inputs
                .iter()
                .any(|i| i.identity.starts_with("kernel-file-position-source"))
        {
            return Err(invalid(
                "extra action source snapshot without a declared binding",
            ));
        }
        return Ok(());
    };
    crate::systems_library::modules::validate_binding(
        &action.modules,
        &action.module_snapshot_sha256,
        &artifact.inputs,
        &artifact.outputs,
        "kernel-file-position-source-module:",
        "file-position-library.modules.json",
    )?;
    if artifact
        .inputs
        .iter()
        .filter(|i| {
            i.identity == format!("kernel-file-position-source:{}", action.source)
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
        ("file-position-library.t", &action.source_sha256),
        ("file-position-library.ll", &action.llvm_sha256),
        (
            "file-position-library.retained.ll",
            &action.retained_llvm_sha256,
        ),
        ("file-position-library.o", &action.object_sha256),
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
    if artifact.file_position_runtime.as_ref() != Some(&expected) {
        return Err(invalid(
            "persistent action runtime/caller/source/data or bounds mismatch",
        ));
    }
    Ok(())
}
