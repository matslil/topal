//! Structural library import syntax; ordinary functions use the shared frontend.
use super::{
    CompilerSystemsSourceLibraryInterface, CompilerSystemsSourceLibraryRequirement, Diagnostic,
    Expression, ProductField, SourceText, Span, artifact_field_label, identifier_is,
    parse_storage_natural, parse_string, source_diagnostic,
};
use std::collections::BTreeSet;

fn error(source: &SourceText, span: Span, message: impl Into<String>) -> Diagnostic {
    source_diagnostic(source, "E-SYSTEMS-LIBRARY", span, message)
}

pub(super) fn parse_requirement(
    source: &SourceText,
    field: &ProductField,
) -> Result<CompilerSystemsSourceLibraryRequirement, Diagnostic> {
    let Expression::Application { items, span } = &field.value else {
        return Err(error(
            source,
            field.value.span(),
            "kernel-library requires a source-library declaration",
        ));
    };
    let [lang, systems, library, Expression::Product { fields, .. }] = items.as_slice() else {
        return Err(error(
            source,
            *span,
            "kernel-library requires lang systems source-library (...)",
        ));
    };
    if !identifier_is(source, lang, "lang")
        || !identifier_is(source, systems, "systems")
        || !identifier_is(source, library, "source-library")
    {
        return Err(error(source, *span, "unknown source-library constructor"));
    }
    let (mut path, mut export, mut interface, mut stack) = (None, None, None, None);
    let mut runtime = None;
    let mut seen = BTreeSet::new();
    for field in fields {
        let label = artifact_field_label(source, field)?;
        let name = source.slice(label);
        if !seen.insert(name) {
            return Err(error(
                source,
                label,
                format!("duplicate source-library field {name}"),
            ));
        }
        match name {
            "source" => {
                let Expression::String(value) = &field.value else {
                    return Err(error(
                        source,
                        field.value.span(),
                        "source-library source requires a static string",
                    ));
                };
                path = Some(
                    parse_string(source.slice(*value))
                        .ok_or_else(|| error(source, *value, "invalid source path"))?
                        .to_owned(),
                );
            }
            "export" => {
                let Expression::Identifier(value) = &field.value else {
                    return Err(error(
                        source,
                        field.value.span(),
                        "source-library export requires one structural identifier",
                    ));
                };
                export = Some(source.slice(*value).to_owned());
            }
            "interface" if identifier_is(source, &field.value, "boot-open-decision-v1") => {
                interface = Some(CompilerSystemsSourceLibraryInterface::BootOpenDecisionV1);
            }
            "interface" if identifier_is(source, &field.value, "relative-sleep-deadline-v1") => {
                interface = Some(CompilerSystemsSourceLibraryInterface::RelativeSleepDeadlineV1);
            }
            "interface" if identifier_is(source, &field.value, "static-elf64-header-v1") => {
                interface = Some(CompilerSystemsSourceLibraryInterface::StaticElf64HeaderV1);
            }
            "interface" if identifier_is(source, &field.value, "static-linux-dispatch-v1") => {
                interface = Some(CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV1);
            }
            "runtime-limit-bytes" => {
                runtime = Some(parse_storage_natural(source, &field.value)?);
            }
            "stack-limit-bytes" => {
                stack = Some(parse_storage_natural(source, &field.value)?);
            }
            _ => {
                return Err(error(
                    source,
                    label,
                    format!("unqualified source-library field/interface {name}"),
                ));
            }
        }
    }
    let (Some(source_path), Some(export), Some(interface), Some(stack_limit_bytes)) =
        (path, export, interface, stack)
    else {
        return Err(error(
            source,
            *span,
            "source library requires source, export, interface and stack-limit-bytes",
        ));
    };
    let result = CompilerSystemsSourceLibraryRequirement {
        source: source_path,
        export,
        interface,
        stack_limit_bytes,
        runtime_limit_bytes: runtime,
    };
    result
        .validate()
        .map_err(|invalid| error(source, *span, invalid.message))?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::super::*;
    #[test]
    fn dispatch_declarations_preserve_independent_policy_selection_and_closed_shape() {
        // TOPAL-SYSTEMS-LINUX-DISPATCH-LIBRARY-001
        let full = include_str!("../../../linux-kernel/kernel/arch/x86_64/toolchain-gate.t");
        let remove = |source: &str, slot: &str| {
            let start = source.find(&format!("  {slot} is")).unwrap();
            let end = start + source[start..].find("\n  ),\n").unwrap() + 6;
            format!("{}{}", &source[..start], &source[end..])
        };
        let base = remove(full, "dispatch-library");
        let declaration = "  dispatch-library is lang systems source-library (\n    source is \"arch/x86_64/policy/static-linux-dispatch.t\",\n    export is select-operation,\n    interface is static-linux-dispatch-v1,\n    stack-limit-bytes is 4096,\n    runtime-limit-bytes is 65536\n  ),\n";
        for without_time in [false, true] {
            for without_image in [false, true] {
                let mut source = base.clone();
                if without_time {
                    source = remove(&source, "time-library");
                }
                if without_image {
                    source = remove(&source, "image-library");
                }
                let insert = |value: &str| {
                    source.replacen(
                        "  bootstrap-storage is",
                        &format!("{value}  bootstrap-storage is"),
                        1,
                    )
                };
                let program = analyze_systems_for_compiler(
                    &insert(declaration),
                    &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
                )
                .unwrap();
                assert_eq!(program.time_library.is_none(), without_time);
                assert_eq!(program.image_library.is_none(), without_image);
                assert_eq!(
                    program.dispatch_library.unwrap().interface,
                    CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV1
                );
                for (before, after) in [
                    ("static-linux-dispatch-v1", "static-elf64-header-v1"),
                    (
                        "arch/x86_64/policy/static-linux-dispatch.t",
                        "../dispatch.t",
                    ),
                    ("select-operation", "0"),
                    ("stack-limit-bytes is 4096", "stack-limit-bytes is 4095"),
                    (
                        "runtime-limit-bytes is 65536",
                        "runtime-limit-bytes is 65535",
                    ),
                ] {
                    assert!(
                        analyze_systems_for_compiler(
                            &insert(&declaration.replace(before, after)),
                            &CompilerSystemsTargetSelection::initial_x86_64_qemu()
                        )
                        .is_err(),
                        "{after}"
                    );
                }
                assert!(
                    analyze_systems_for_compiler(
                        &insert(&declaration.repeat(2)),
                        &CompilerSystemsTargetSelection::initial_x86_64_qemu()
                    )
                    .is_err()
                );
            }
        }
    }

    #[test]
    fn image_policy_declarations_are_independent_and_structurally_checked() {
        // TOPAL-SYSTEMS-ELF64-HEADER-LIBRARY-001
        let full = include_str!("../../../linux-kernel/kernel/arch/x86_64/toolchain-gate.t");
        let image_start = full.find("  image-library is").unwrap();
        let image_end = full[image_start..].find("  bootstrap-storage is").unwrap() + image_start;
        let original = format!("{}{}", &full[..image_start], &full[image_end..]);
        let start = original.find("  time-library is").unwrap();
        let end = original[start..].find("  bootstrap-storage is").unwrap() + start;
        let no_time = format!("{}{}", &original[..start], &original[end..]);
        let declaration = "  image-library is lang systems source-library (\n    source is \"policy/static-elf64-header.t\",\n    export is header-admitted?,\n    interface is static-elf64-header-v1,\n    stack-limit-bytes is 4096,\n    runtime-limit-bytes is 65536\n  ),\n";
        let target = CompilerSystemsTargetSelection::initial_x86_64_qemu();
        for (base, has_time) in [(original.as_str(), true), (no_time.as_str(), false)] {
            let insert = |declaration: &str| {
                base.replacen(
                    "  bootstrap-storage is",
                    &format!("{declaration}  bootstrap-storage is"),
                    1,
                )
            };
            let source = insert(declaration);
            let program = analyze_systems_for_compiler(&source, &target).unwrap();
            assert_eq!(program.time_library.is_some(), has_time);
            let image = program.image_library.unwrap();
            assert_eq!(
                image.interface,
                CompilerSystemsSourceLibraryInterface::StaticElf64HeaderV1
            );
            assert_eq!(image.source, "policy/static-elf64-header.t");
            for (before, after) in [
                ("static-elf64-header-v1", "relative-sleep-deadline-v1"),
                ("policy/static-elf64-header.t", "../header.t"),
                ("header-admitted?", "7"),
                ("stack-limit-bytes is 4096", "stack-limit-bytes is 4095"),
                (
                    "runtime-limit-bytes is 65536",
                    "runtime-limit-bytes is 65535",
                ),
            ] {
                assert!(
                    analyze_systems_for_compiler(
                        &insert(&declaration.replace(before, after)),
                        &target
                    )
                    .is_err(),
                    "accepted {after}"
                );
            }
            assert!(
                analyze_systems_for_compiler(&insert(&declaration.repeat(2)), &target).is_err()
            );
        }
    }
    #[test]
    fn numeric_time_library_retains_exact_declared_resource_obligations() {
        // TOPAL-SYSTEMS-NUMERIC-RUNTIME-001
        let source = include_str!("../../../linux-kernel/kernel/arch/x86_64/toolchain-gate.t");
        let source = source.to_owned();
        let target = CompilerSystemsTargetSelection::initial_x86_64_qemu();
        let program = analyze_systems_for_compiler(&source, &target).unwrap();
        let time = program.time_library.unwrap();
        assert_eq!(time.runtime_limit_bytes, Some(65536));
        assert_eq!(time.stack_limit_bytes, 4096);
        for changed in [
            source.replace(
                "runtime-limit-bytes is 65536",
                "runtime-limit-bytes is 65535",
            ),
            source.replace("runtime-limit-bytes is 65536", "runtime-limit-bytes is 0"),
            source.replace("stack-limit-bytes is 4096", "stack-limit-bytes is 1024"),
        ] {
            assert!(analyze_systems_for_compiler(&changed, &target).is_err());
        }
    }
    #[test]
    fn source_library_import_is_optional_and_structurally_checked() {
        // TOPAL-SYSTEMS-SOURCE-LIBRARY-001
        let source = include_str!("../../../linux-kernel/kernel/arch/x86_64/toolchain-gate.t");
        let target = CompilerSystemsTargetSelection::initial_x86_64_qemu();
        let program = analyze_systems_for_compiler(source, &target).unwrap();
        assert_eq!(
            program.kernel_library.unwrap().source,
            "policy/boot-open-access.t"
        );
        for (before, after) in [
            ("policy/boot-open-access.t", "../policy.t"),
            ("stack-limit-bytes is 1024", "stack-limit-bytes is 1025"),
            ("interface is boot-open-decision-v1", "interface is other"),
            ("export is boot-open-decision", "export is 7"),
            (
                "stack-limit-bytes is 1024",
                "source is \"other.t\", stack-limit-bytes is 1024",
            ),
        ] {
            assert!(
                analyze_systems_for_compiler(&source.replacen(before, after, 1), &target).is_err(),
                "{after}"
            );
        }
        let start = source.find("  kernel-library is").unwrap();
        let end = source[start..].find("  bootstrap-storage is").unwrap() + start;
        let legacy = format!("{}{}", &source[..start], &source[end..]);
        assert!(
            analyze_systems_for_compiler(&legacy, &target)
                .unwrap()
                .kernel_library
                .is_none()
        );
    }
}
