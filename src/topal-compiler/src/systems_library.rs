//! Checked pure source units and their private kernel-library boundary.
use crate::CompileError;
#[path = "systems_library_object.rs"]
mod native_object;
pub(crate) use native_object::{build_source_library, inspect_linked_library};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use topal_language::compiler::{
    CompilerBinary, CompilerBlock, CompilerExpression, CompilerExpressionKind as K,
    CompilerProgram, CompilerStatement, CompilerSystemsSourceLibraryRequirement, CompilerType,
    analyze_for_compiler,
};
use topal_language::{modules::declared_libraries, published_function_names};
use topal_source::{Diagnostic, Span, canonically_equal};

pub(crate) const EXPORT_SYMBOL: &str = "topal_kernel_boot_open_policy_v1";
pub(crate) const CHOICES: [&str; 4] = [
    "ReadDescription",
    "NotDirectory",
    "DirectoryWrite",
    "ReadOnlyView",
];

/// Explicit source snapshot for the import declared by a checked systems root.
#[derive(Clone, Debug)]
pub struct SystemsSourceLibraryInput {
    pub source: String,
    pub source_name: String,
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsLibraryFunctionStack {
    pub symbol: String,
    pub frame_bytes: u64,
    pub maximum_call_path_bytes: u64,
    pub callees: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsSourceLibraryArtifact {
    pub schema: String,
    pub interface: String,
    pub source: String,
    pub source_name: String,
    pub source_sha256: String,
    pub checked_client_sha256: String,
    pub llvm_sha256: String,
    pub object_sha256: String,
    pub native_export: String,
    pub source_export: String,
    pub source_tags: [u32; 4],
    pub llvm_version: String,
    pub machine_profile: String,
    pub disassembly_sha256: String,
    pub native_alignment_reserve_bytes: u64,
    pub maximum_stack_bytes: u64,
    pub stack_limit_bytes: u64,
    pub functions: Vec<SystemsLibraryFunctionStack>,
}

pub(crate) struct CheckedFiniteLibrary {
    program: CompilerProgram,
    source_name: String,
    entry_symbol: String,
    source_tags: [u32; 4],
    calls: BTreeMap<String, BTreeSet<String>>,
}
impl CheckedFiniteLibrary {
    pub(crate) fn program(&self) -> &CompilerProgram {
        &self.program
    }
    pub(crate) fn source_name(&self) -> &str {
        &self.source_name
    }
    pub(crate) fn entry_symbol(&self) -> &str {
        &self.entry_symbol
    }
    pub(crate) const fn source_tags(&self) -> &[u32; 4] {
        &self.source_tags
    }
    pub(crate) fn calls(&self) -> &BTreeMap<String, BTreeSet<String>> {
        &self.calls
    }
}

fn invalid(message: impl Into<String>) -> CompileError {
    CompileError::Tool(format!("E-SYSTEMS-LIBRARY: {}", message.into()))
}
fn finite(value: &CompilerType) -> bool {
    matches!(value, CompilerType::Boolean | CompilerType::Enum(_))
}
fn check_block(block: &CompilerBlock, calls: &mut BTreeSet<String>) -> Result<(), String> {
    for statement in &block.statements {
        let expression = match statement {
            CompilerStatement::Binding(binding) => &binding.value,
            CompilerStatement::Discard(expression) => expression,
        };
        check_expression(expression, calls)?;
    }
    check_expression(&block.result, calls)
}
fn check_expression(
    value: &CompilerExpression,
    calls: &mut BTreeSet<String>,
) -> Result<(), String> {
    if !finite(&value.value_type) {
        return Err("library values must be Boolean or nominal Enum".into());
    }
    match &value.kind {
        K::Boolean(_) | K::Local(_) => Ok(()),
        K::Enum(tag) => match &value.value_type {
            CompilerType::Enum(kind) if (*tag as usize)<kind.alternatives.len() => Ok(()),
            _ => Err("invalid checked Enum literal".into()),
        },
        K::Not(input) => check_expression(input,calls),
        K::Binary { operation: CompilerBinary::Equal | CompilerBinary::NotEqual, left,right } => {
            check_expression(left,calls)?; check_expression(right,calls)
        }
        K::Block(block) => check_block(block,calls),
        K::Call { symbol,arguments } => {
            for argument in arguments { check_expression(argument,calls)?; }
            calls.insert(symbol.clone()); Ok(())
        }
        K::BooleanDecision { subject,when_true,when_false } => {
            check_expression(subject,calls)?; check_expression(when_true,calls)?;
            check_expression(when_false,calls)
        }
        K::EnumDecision { subject,rules,otherwise } => {
            check_expression(subject,calls)?;
            for rule in rules { check_expression(&rule.action,calls)?; }
            if let Some(fallback)=otherwise { check_expression(fallback,calls)?; }
            Ok(())
        }
        _ => Err("unqualified freestanding form: only finite pure values, bindings, decisions and direct calls are admitted".into()),
    }
}

pub(crate) fn check_source_library(
    requirement: &CompilerSystemsSourceLibraryRequirement,
    input: &SystemsSourceLibraryInput,
) -> Result<CheckedFiniteLibrary, CompileError> {
    requirement.validate().map_err(|e| invalid(e.to_string()))?;
    if !declared_libraries(&input.text).is_empty() {
        return Err(invalid(
            "source-library imports are not qualified in this initial unit profile",
        ));
    }
    if input.source != requirement.source {
        return Err(invalid(
            "source snapshot does not match the declared import",
        ));
    }
    if !published_function_names(&input.text)
        .iter()
        .any(|name| canonically_equal(name, &requirement.export))
    {
        return Err(invalid(format!(
            "{}: selected export is not a public function",
            input.source_name
        )));
    }
    // Ordinary instantiation retains unconstrained Boolean parameters. The
    // client is only a type/overload-selection root and is never emitted.
    let client = format!(
        "{}\n{} (false, false, false)\n",
        input.text, requirement.export
    );
    let program = analyze_for_compiler(&client)
        .map_err(|diagnostic| CompileError::Tool(diagnostic.render(&input.source_name)))?;
    if !program.main.statements.is_empty() {
        return Err(invalid(
            "source-library unit must contain declarations, not runtime initializers",
        ));
    }
    let K::Call { symbol, arguments } = &program.main.result.kind else {
        return Err(invalid(
            "public export did not instantiate as an ordinary function",
        ));
    };
    if arguments.len() != 3
        || arguments
            .iter()
            .any(|a| a.value_type != CompilerType::Boolean)
    {
        return Err(invalid("boot-open interface requires three Boolean facts"));
    }
    let entry = program
        .functions
        .iter()
        .find(|f| &f.symbol == symbol)
        .ok_or_else(|| invalid("missing instantiated export"))?;
    if entry.parameters.len() != 3
        || entry
            .parameters
            .iter()
            .any(|p| p.value_type != CompilerType::Boolean)
    {
        return Err(invalid("export has hidden or non-Boolean inputs"));
    }
    let CompilerType::Enum(result) = &entry.result_type else {
        return Err(invalid(
            "boot-open interface requires the nominal BootOpenDecision result",
        ));
    };
    if !canonically_equal(&result.name, "BootOpenDecision") || result.alternatives.len() != 4 {
        return Err(invalid("wrong boot-open result classifier"));
    }
    let mut tags = [0; 4];
    for (index, name) in CHOICES.iter().enumerate() {
        tags[index] = result
            .alternatives
            .iter()
            .position(|candidate| canonically_equal(candidate, name))
            .ok_or_else(|| invalid(format!("missing semantic result {name}")))?
            as u32;
    }
    let entry_symbol = entry.symbol.clone();
    let mut seen = BTreeSet::new();
    let mut all_calls = BTreeMap::new();
    for function in &program.functions {
        if function.foreign.is_some()
            || !function.result_captures.is_empty()
            || !function.pattern_identities.is_empty()
            || function
                .declared_effects
                .as_ref()
                .is_some_and(|row| !row.identities.is_empty())
            || !finite(&function.result_type)
            || function
                .parameters
                .iter()
                .any(|p| !finite(&p.value_type) || p.int_range.is_some())
        {
            return Err(invalid(format!(
                "{}: unqualified function interface or effect/capture",
                function.source_name
            )));
        }
        let mut edges = BTreeSet::new();
        check_block(&function.body, &mut edges).map_err(|message| {
            let position = program.source.position(function.span.start);
            CompileError::Tool(
                Diagnostic::error(
                    "E-SYSTEMS-LIBRARY-PROFILE",
                    position.line,
                    position.column,
                    message,
                )
                .with_source_span(Span::new(function.span.start, function.span.end))
                .render(&input.source_name),
            )
        })?;
        // Shared model promises callee-before-caller order. A recursive or
        // unresolved edge cannot obtain a kernel binding under this profile.
        if edges.iter().any(|target| !seen.contains(target)) {
            return Err(invalid("recursive or unresolved library call graph"));
        }
        seen.insert(function.symbol.clone());
        all_calls.insert(function.symbol.clone(), edges);
    }
    let mut reachable = BTreeSet::new();
    let mut pending = vec![entry_symbol.clone()];
    while let Some(symbol) = pending.pop() {
        if reachable.insert(symbol.clone()) {
            pending.extend(all_calls[&symbol].iter().cloned());
        }
    }
    if reachable != seen {
        return Err(invalid(
            "source unit retained functions outside the selected export closure",
        ));
    }
    Ok(CheckedFiniteLibrary {
        program,
        source_name: input.source_name.clone(),
        entry_symbol,
        source_tags: tags,
        calls: all_calls,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn requirement() -> CompilerSystemsSourceLibraryRequirement {
        CompilerSystemsSourceLibraryRequirement {
            source: "policy/boot-open-access.t".into(),
            export: "boot-open-decision".into(),
            interface:
                topal_language::compiler::CompilerSystemsSourceLibraryInterface::BootOpenDecisionV1,
            stack_limit_bytes: 1024,
        }
    }
    fn input() -> SystemsSourceLibraryInput {
        SystemsSourceLibraryInput {
            source: requirement().source,
            source_name: "policy/boot-open-access.t".into(),
            text: include_str!("../../../linux-kernel/kernel/policy/boot-open-access.t").into(),
        }
    }
    #[test]
    fn finite_library_uses_ordinary_frontend_and_codegen() {
        let checked = check_source_library(&requirement(), &input()).unwrap();
        assert_eq!(*checked.source_tags(), [0, 1, 2, 3]);
        assert!(checked.program().functions.len() > 1);
        let llvm = crate::codegen::emit_finite_kernel_library(&checked).unwrap();
        assert!(llvm.contains("call fastcc i32"));
        assert!(llvm.contains("noredzone noimplicitfloat"));
        assert!(!llvm.contains("@_start"));
        assert!(!llvm.contains("@malloc"));
        let mut reordered = input();
        reordered.text = reordered.text.replace(
            "ReadDescription, NotDirectory, DirectoryWrite, ReadOnlyView",
            "ReadOnlyView, DirectoryWrite, NotDirectory, ReadDescription",
        );
        assert_eq!(
            *check_source_library(&requirement(), &reordered)
                .unwrap()
                .source_tags(),
            [3, 2, 1, 0]
        );
    }
    #[test]
    fn native_finite_library_has_bounded_stack_and_preserves_enum_meaning() {
        let tools = crate::LlvmTools::discover(None).unwrap();
        let parent =
            std::env::temp_dir().join(format!("topal-finite-policy-{}", std::process::id()));
        std::fs::create_dir_all(&parent).unwrap();
        for reordered in [false, true] {
            let directory = parent.join(if reordered { "reordered" } else { "normal" });
            std::fs::create_dir_all(&directory).unwrap();
            let mut source = input();
            if reordered {
                source.text = source.text.replace(
                    "ReadDescription, NotDirectory, DirectoryWrite, ReadOnlyView",
                    "ReadOnlyView, DirectoryWrite, NotDirectory, ReadDescription",
                );
            }
            let library =
                build_source_library(&requirement(), &source, &tools, &directory).unwrap();
            assert!(library.record.maximum_stack_bytes <= 1024);
            assert!(library.record.maximum_stack_bytes > 32);
            assert_eq!(library.record.functions.len(), 7);
            let driver = directory.join("driver.c");
            std::fs::write(&driver, r#"
#include <stdint.h>
extern uint64_t topal_kernel_boot_open_policy_v1(uint64_t);
int main(void) {
  const uint64_t expected[8]={0,3,0,2,1,1,0,2};
  for(uint64_t i=0;i<8;i++) if(topal_kernel_boot_open_policy_v1(i)!=expected[i]) return 1;
  return topal_kernel_boot_open_policy_v1(8)!=UINT64_MAX || topal_kernel_boot_open_policy_v1(UINT64_MAX)!=UINT64_MAX;
}
"#).unwrap();
            let executable = directory.join("policy");
            let output = std::process::Command::new("cc")
                .arg("-no-pie")
                .arg(&driver)
                .arg(directory.join("kernel-library.o"))
                .arg("-o")
                .arg(&executable)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(
                std::process::Command::new(&executable)
                    .status()
                    .unwrap()
                    .success()
            );
        }
        std::fs::remove_dir_all(parent).unwrap();
    }
    #[test]
    fn compiled_stack_metadata_rejects_an_excessive_acyclic_call_path() {
        let tools = crate::LlvmTools::discover(None).unwrap();
        let directory =
            std::env::temp_dir().join(format!("topal-finite-policy-stack-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let mut source = input();
        source.text="use language (version is v0.1)\npub BootOpenDecision is Enum (ReadDescription, NotDirectory, DirectoryWrite, ReadOnlyView)\n".into();
        let parameters = (0..160)
            .map(|i| format!("p{i} : Boolean"))
            .collect::<Vec<_>>()
            .join(", ");
        source.text.push_str(&format!("wide is fn (({parameters})) -> BootOpenDecision\n  p159\n    true then ReadDescription\n    false then ReadOnlyView\n"));
        let arguments = (0..160)
            .map(|i| ["a", "b", "c"][i % 3])
            .collect::<Vec<_>>()
            .join(", ");
        source.text.push_str(&format!("pub boot-open-decision is fn ((a : Boolean, b : Boolean, c : Boolean)) -> BootOpenDecision\n  wide ({arguments})\n"));
        let error = match build_source_library(&requirement(), &source, &tools, &directory) {
            Ok(_) => panic!("excessive stack was admitted"),
            Err(error) => error,
        };
        assert!(
            error.to_string().contains("library stack requires"),
            "{error}"
        );
        std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn finite_profile_rejects_nonfinite_values_and_cycles() {
        let mut source = input();
        source.text = source
            .text
            .replace("  directory\n", "  text is \"unqualified\"\n  directory\n");
        let error = match check_source_library(&requirement(), &source) {
            Ok(_) => panic!("nonfinite local admitted"),
            Err(error) => error,
        };
        assert!(
            error.to_string().contains("E-SYSTEMS-LIBRARY-PROFILE"),
            "{error}"
        );
        let mut source = input();
        source.text = source
            .text
            .replace("false then ReadOnlyView", "false then deny-write directory");
        assert!(check_source_library(&requirement(), &source).is_err());
    }
    #[test]
    fn source_loading_rejects_symlink_escape_and_missing_units() {
        let root = std::env::temp_dir().join(format!("topal-policy-path-{}", std::process::id()));
        std::fs::create_dir_all(root.join("policy")).unwrap();
        assert!(load_source_library(&requirement(), &root).is_err());
        let outside = root.with_extension("t");
        std::fs::write(&outside, &input().text).unwrap();
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&outside, root.join(&requirement().source)).unwrap();
            assert!(load_source_library(&requirement(), &root).is_err());
        }
        std::fs::remove_dir_all(root).unwrap();
        std::fs::remove_file(outside).unwrap();
    }
    #[test]
    fn declared_library_never_uses_missing_or_private_source() {
        let mut imported = input();
        imported.text = imported.text.replacen(
            "use language (version is v0.1)",
            "use language (version is v0.1)\nuse library std (version is v0.1)",
            1,
        );
        assert!(check_source_library(&requirement(), &imported).is_err());
        let mut source = input();
        source.source = "other.t".into();
        assert!(check_source_library(&requirement(), &source).is_err());
        let mut source = input();
        source.text = source
            .text
            .replace("pub boot-open-decision", "boot-open-decision");
        assert!(check_source_library(&requirement(), &source).is_err());
        let mut source = input();
        source.text = source.text.replace("-> BootOpenDecision", "-> Boolean");
        assert!(check_source_library(&requirement(), &source).is_err());
    }
}

/// Resolve a declared unit under the explicitly selected library root.
pub(crate) fn load_source_library(
    requirement: &CompilerSystemsSourceLibraryRequirement,
    root: &std::path::Path,
) -> Result<SystemsSourceLibraryInput, CompileError> {
    requirement.validate().map_err(|e| invalid(e.to_string()))?;
    let root = root
        .canonicalize()
        .map_err(|e| invalid(format!("library root: {e}")))?;
    let path = root
        .join(&requirement.source)
        .canonicalize()
        .map_err(|e| invalid(format!("{}: {e}", requirement.source)))?;
    if !path.starts_with(&root) || !path.is_file() {
        return Err(invalid("source unit escapes the selected library root"));
    }
    let text = std::fs::read_to_string(&path).map_err(|e| invalid(e.to_string()))?;
    Ok(SystemsSourceLibraryInput {
        source: requirement.source.clone(),
        source_name: path.display().to_string(),
        text,
    })
}
