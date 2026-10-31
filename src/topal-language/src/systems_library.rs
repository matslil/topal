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
