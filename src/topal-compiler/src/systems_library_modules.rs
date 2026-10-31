//! Frozen ordinary source dependencies for the two pure systems interfaces.
use super::{SystemsSourceLibraryInput, invalid};
use crate::CompileError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};
use topal_language::modules::{
    SourceModule, declared_libraries, published_function_names, references_module,
    select_source_modules,
};
use topal_source::SourceText;
use topal_syntax::{Statement, lex, parse};

const MAX_MODULES: usize = 64;
const MAX_BYTES: usize = 1048576;
const MAX_ENTRIES: usize = 4096;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsSourceModuleInput {
    pub source: String,
    pub identity: Vec<String>,
    pub text: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsSourceModuleArtifact {
    pub source: String,
    pub identity: Vec<String>,
    pub sha256: String,
}

fn identity(path: &str) -> Result<Vec<String>, CompileError> {
    let path = Path::new(path);
    if path.extension().and_then(|p| p.to_str()) != Some("t")
        || path
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(invalid(
            "module source must be a contained relative .t path",
        ));
    }
    let file = path
        .file_name()
        .and_then(|p| p.to_str())
        .ok_or_else(|| invalid("invalid module filename"))?;
    if matches!(file, "application.t" | "package.t" | "library.t") {
        return Err(invalid("context facade is not an ordinary source module"));
    }
    let mut result = path
        .parent()
        .into_iter()
        .flat_map(Path::components)
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    if result.is_empty() {
        return Err(invalid("module file is outside a library directory"));
    }
    let normalized = path
        .components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    if normalized != path.to_string_lossy() {
        return Err(invalid("module path is not normalized"));
    }
    if file != "module.t" {
        result.push(path.file_stem().unwrap().to_string_lossy().into_owned());
    }
    if result.is_empty() {
        return Err(invalid("module has no library identity"));
    }
    Ok(result)
}

fn referenced(text: &str, module: &SystemsSourceModuleInput) -> bool {
    if Path::new(&module.source)
        .file_name()
        .is_some_and(|p| p == "module.t")
    {
        published_function_names(&module.text)
            .into_iter()
            .any(|name| {
                let mut qualified = module.identity.clone();
                qualified.push(name);
                references_module(text, &qualified)
            })
    } else {
        references_module(text, &module.identity)
    }
}

fn selected(text: &str, module: &SystemsSourceModuleInput) -> bool {
    declared_libraries(text)
        .iter()
        .any(|library| library == &module.identity[0])
        && referenced(text, module)
}

fn declarations(text: &str, name: &str) -> Result<(), CompileError> {
    let source = SourceText::new(text).map_err(|e| invalid(e.to_string()))?;
    let parsed = parse(&source, &lex(&source));
    if !parsed.diagnostics.is_empty() {
        return Err(invalid(format!("invalid module syntax: {name}")));
    }
    fn admitted(statement: &Statement) -> bool {
        match statement {
            Statement::LanguageSelection { .. }
            | Statement::LibrarySelection { .. }
            | Statement::Function { .. } => true,
            Statement::Published { declaration, .. } => admitted(declaration),
            _ => false,
        }
    }
    if parsed.statements.iter().any(|s| !admitted(s)) {
        return Err(invalid(format!(
            "module runtime initializers or unqualified declarations: {name}"
        )));
    }
    Ok(())
}

/// Check the exact reachable source closure without filesystem authority.
pub(crate) fn validate(
    input: &SystemsSourceLibraryInput,
) -> Result<Vec<SourceModule>, CompileError> {
    if input.modules.len() > MAX_MODULES {
        return Err(invalid("module count exceeds 64"));
    }
    let total = input
        .modules
        .iter()
        .try_fold(input.text.len(), |n, m| n.checked_add(m.text.len()))
        .ok_or_else(|| invalid("source size overflow"))?;
    if total > MAX_BYTES {
        return Err(invalid("source snapshot exceeds 1 MiB"));
    }
    let mut paths = BTreeSet::new();
    let mut identities = BTreeSet::new();
    for m in &input.modules {
        if identity(&m.source)? != m.identity
            || m.source == input.source
            || !paths.insert(&m.source)
            || !identities.insert(&m.identity)
        {
            return Err(invalid(
                "duplicate, conflicting or mismatched module identity/path",
            ));
        }
        declarations(&m.text, &m.source)?;
    }
    for text in std::iter::once(&input.text).chain(input.modules.iter().map(|m| &m.text)) {
        for module in &input.modules {
            if referenced(text, module) && !selected(text, module) {
                return Err(invalid(
                    "module reference without an explicit library selection",
                ));
            }
        }
    }
    fn visit(
        index: usize,
        input: &SystemsSourceLibraryInput,
        active: &mut BTreeSet<usize>,
        seen: &mut BTreeSet<usize>,
    ) -> Result<(), CompileError> {
        if active.contains(&index) {
            return Err(invalid("cyclic source module dependencies"));
        }
        if seen.contains(&index) {
            return Ok(());
        }
        active.insert(index);
        for (next, module) in input.modules.iter().enumerate() {
            if selected(&input.modules[index].text, module) {
                visit(next, input, active, seen)?;
            }
        }
        active.remove(&index);
        seen.insert(index);
        Ok(())
    }
    let mut seen = BTreeSet::new();
    for (index, module) in input.modules.iter().enumerate() {
        if selected(&input.text, module) {
            visit(index, input, &mut BTreeSet::new(), &mut seen)?;
        }
    }
    if seen.len() != input.modules.len() {
        return Err(invalid("unselected module snapshot input"));
    }
    let mut modules = input
        .modules
        .iter()
        .map(|m| SourceModule {
            identity: m.identity.clone(),
            source_name: m.source.clone(),
            source: m.text.clone(),
        })
        .collect::<Vec<_>>();
    modules.sort_by(|a, b| a.identity.cmp(&b.identity));
    Ok(modules)
}

/// Bound traversal before invoking shared ordinary selection. No dependency
/// symlinks or special files can make discovery leave the selected tree.
fn inspect_tree(path: &Path, entries: &mut usize, bytes: &mut usize) -> Result<(), CompileError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|e| invalid(e.to_string()))?;
    if metadata.file_type().is_symlink() {
        return Err(invalid("symlinked dependency trees are unqualified"));
    }
    if metadata.is_dir() {
        for child in std::fs::read_dir(path).map_err(|e| invalid(e.to_string()))? {
            *entries += 1;
            if *entries > MAX_ENTRIES {
                return Err(invalid("module discovery exceeds 4096 entries"));
            }
            inspect_tree(
                &child.map_err(|e| invalid(e.to_string()))?.path(),
                entries,
                bytes,
            )?;
        }
    } else if !metadata.is_file() {
        return Err(invalid("special dependency files are unqualified"));
    } else if path.extension().is_some_and(|p| p == "t") {
        *bytes = bytes
            .checked_add(
                usize::try_from(metadata.len()).map_err(|_| invalid("source size overflow"))?,
            )
            .filter(|n| *n <= MAX_BYTES)
            .ok_or_else(|| invalid("source discovery exceeds 1 MiB"))?;
    }
    Ok(())
}

pub(crate) fn load(input: &mut SystemsSourceLibraryInput, root: &Path) -> Result<(), CompileError> {
    let mut frontier = vec![input.text.clone()];
    let mut inspected = BTreeSet::new();
    let mut entries = 0;
    let mut bytes = input.text.len();
    let mut selected_modules: BTreeMap<Vec<String>, SystemsSourceModuleInput> = BTreeMap::new();
    while let Some(text) = frontier.pop() {
        let libraries = declared_libraries(&text);
        for library in &libraries {
            if !inspected.insert(library.clone()) {
                continue;
            }
            // Library identities are source names, never path syntax.
            let path = Path::new(library);
            if path.components().count() != 1
                || !matches!(path.components().next(), Some(Component::Normal(_)))
            {
                return Err(invalid("invalid library identity"));
            }
            inspect_tree(&root.join(library), &mut entries, &mut bytes)?;
        }
        let names = libraries.iter().map(String::as_str).collect::<Vec<_>>();
        let modules =
            select_source_modules(&text, root, &names).map_err(|e| invalid(e.to_string()))?;
        for module in modules {
            let path = Path::new(&module.source_name)
                .canonicalize()
                .map_err(|e| invalid(e.to_string()))?;
            let relative = path
                .strip_prefix(root)
                .map_err(|_| invalid("module escapes the selected root"))?
                .to_str()
                .ok_or_else(|| invalid("non-Unicode module path"))?
                .to_owned();
            let frozen = SystemsSourceModuleInput {
                source: relative,
                identity: module.identity,
                text: module.source,
            };
            if let Some(previous) = selected_modules.get(&frozen.identity) {
                if previous != &frozen {
                    return Err(invalid("conflicting module inputs during discovery"));
                }
            } else {
                if selected_modules.len() == MAX_MODULES {
                    return Err(invalid("module count exceeds 64"));
                }
                frontier.push(frozen.text.clone());
                selected_modules.insert(frozen.identity.clone(), frozen);
            }
        }
    }
    input.modules = selected_modules.into_values().collect();
    validate(input)?;
    Ok(())
}

pub(crate) fn records(input: &SystemsSourceLibraryInput) -> Vec<SystemsSourceModuleArtifact> {
    let mut records = input
        .modules
        .iter()
        .map(|m| SystemsSourceModuleArtifact {
            source: m.source.clone(),
            identity: m.identity.clone(),
            sha256: format!("{:x}", Sha256::digest(m.text.as_bytes())),
        })
        .collect::<Vec<_>>();
    records.sort_by(|a, b| a.identity.cmp(&b.identity));
    records
}

/// Check imported public closures without the policy root's private bindings.
/// The ordinary compiler currently combines source texts; an independent
/// unprivileged client prevents accidental caller-scope name capture, even if
/// constant folding would remove that reference from the final typed graph.
pub(crate) fn check_isolation(
    input: &SystemsSourceLibraryInput,
    program: &topal_language::CompilerProgram,
) -> Result<(), CompileError> {
    use topal_language::CompilerType as T;
    if input.modules.is_empty() {
        return Ok(());
    }
    let modules = validate(input)?;
    let mut client = "use language (version is v0.1)\n".to_owned();
    let libraries = modules
        .iter()
        .map(|m| m.identity[0].as_str())
        .collect::<BTreeSet<_>>();
    for library in libraries {
        client.push_str(&format!("use library {library} (version is v0.1)\n"));
    }
    let mut exports = Vec::new();
    let mut serial = 0;
    fn argument(kind: &T, client: &mut String, serial: &mut usize) -> Result<String, CompileError> {
        if let T::Tuple(fields) = kind {
            let fields = fields
                .iter()
                .map(|f| argument(f, client, serial))
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(format!("({})", fields.join(", ")));
        }
        let (classifier, value) = match kind {
            T::Boolean => ("Boolean", "false"),
            T::Nat => ("Nat", "0"),
            T::Int => ("Int", "0"),
            _ => {
                return Err(invalid(
                    "unqualified imported value type for isolated module checking",
                ));
            }
        };
        let name = format!("module-probe-arg{}", *serial);
        *serial += 1;
        client.push_str(&format!("{name} : {classifier} is {value}\n"));
        Ok(name)
    }
    for function in &program.functions {
        let Some(identity) = &function.module_identity else {
            continue;
        };
        let module = input
            .modules
            .iter()
            .find(|m| &m.identity == identity)
            .ok_or_else(|| invalid("function without a frozen module owner"))?;
        let name = function
            .source_name
            .rsplit('.')
            .next()
            .ok_or_else(|| invalid("missing source function name"))?;
        if !published_function_names(&module.text)
            .iter()
            .any(|n| topal_source::canonically_equal(n, name))
        {
            continue;
        }
        let arguments = function
            .parameters
            .iter()
            .map(|p| argument(&p.value_type, &mut client, &mut serial))
            .collect::<Result<Vec<_>, _>>()?;
        let operands = if arguments.len() == 1 {
            arguments[0].clone()
        } else {
            format!("({})", arguments.join(", "))
        };
        client.push_str(&format!(
            "module-probe-result{serial} is {} {name} {operands}\n",
            identity.join(" ")
        ));
        serial += 1;
        exports.push(function.source_name.clone());
    }
    client.push_str("()\n");
    topal_language::analyze_for_compiler_with_modules_and_declared_exports(
        &client, &modules, &exports,
    )
    .map_err(|e| {
        invalid(format!(
            "module closure depends on an unqualified or private caller scope: {}",
            e.render("module-isolation.t")
        ))
    })?;
    Ok(())
}

pub(crate) fn snapshot(input: &SystemsSourceLibraryInput) -> Result<Vec<u8>, CompileError> {
    let mut modules = input.modules.clone();
    modules.sort_by(|a, b| a.identity.cmp(&b.identity));
    serde_json::to_vec_pretty(&modules).map_err(|e| invalid(e.to_string()))
}

pub(crate) fn validate_binding(
    records: &[SystemsSourceModuleArtifact],
    snapshot_digest: &str,
    inputs: &[crate::DigestEntry],
    outputs: &[crate::DigestEntry],
    prefix: &str,
    output: &str,
) -> Result<(), CompileError> {
    let mut paths = BTreeSet::new();
    let mut identities = BTreeSet::new();
    for record in records {
        if identity(&record.source)? != record.identity
            || !paths.insert(&record.source)
            || !identities.insert(&record.identity)
            || record.sha256.len() != 64
            || !record.sha256.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(invalid("invalid module provenance identity"));
        }
    }
    let actual = inputs
        .iter()
        .filter(|i| i.identity.starts_with(prefix))
        .collect::<Vec<_>>();
    if records.len() > MAX_MODULES
        || actual.len() != records.len()
        || records.iter().any(|m| {
            actual
                .iter()
                .filter(|i| i.identity == format!("{prefix}{}", m.source) && i.sha256 == m.sha256)
                .count()
                != 1
        })
        || outputs.iter().filter(|o| o.identity == output).count() != 1
        || !outputs
            .iter()
            .any(|o| o.identity == output && o.sha256 == snapshot_digest)
        || snapshot_digest.len() != 64
        || !snapshot_digest.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(invalid("module snapshot and artifact provenance disagree"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn finite() -> (
        topal_language::compiler::CompilerSystemsSourceLibraryRequirement,
        SystemsSourceLibraryInput,
    ) {
        let requirement = topal_language::compiler::CompilerSystemsSourceLibraryRequirement {
            source: "policy/open.t".into(),
            export: "boot-open-decision".into(),
            interface:
                topal_language::compiler::CompilerSystemsSourceLibraryInterface::BootOpenDecisionV1,
            stack_limit_bytes: 1024,
            runtime_limit_bytes: None,
        };
        let text = include_str!("../../../linux-kernel/kernel/policy/boot-open-access.t").into();
        let input=SystemsSourceLibraryInput {source:requirement.source.clone(),source_name:requirement.source.clone(),text,
            modules:vec![SystemsSourceModuleInput {source:"support/boot.t".into(),identity:vec!["support".into(),"boot".into()],text:"use language (version is v0.1)\nuse library support (version is v0.1)\npub missing? is fn (present : Boolean) -> Boolean\n  support logic absent? present\n".into()},
            SystemsSourceModuleInput {source:"support/logic.t".into(),identity:vec!["support".into(),"logic".into()],text:"use language (version is v0.1)\npub absent? is fn (present : Boolean) -> Boolean\n  not present\n".into()}]};
        (requirement, input)
    }
    #[test]
    fn imported_helpers_use_shared_selection_lowering_and_real_source_locations() {
        // TOPAL-SYSTEMS-SOURCE-MODULES-001
        let (requirement, input) = finite();
        let checked = super::super::check_source_library(&requirement, &input).unwrap();
        assert_eq!(checked.program().dependencies.len(), 2);
        let llvm = crate::codegen::emit_finite_kernel_library(&checked).unwrap();
        assert!(llvm.contains("filename: \"logic.t\", directory: \"support\""));
        assert!(llvm.contains("support_2elogic_2eabsent_3f"));
        let root = std::env::temp_dir().join(format!("topal-module-policy-{}", std::process::id()));
        std::fs::create_dir_all(root.join("policy")).unwrap();
        std::fs::create_dir_all(root.join("support")).unwrap();
        std::fs::write(root.join(&input.source), &input.text).unwrap();
        for m in &input.modules {
            std::fs::write(root.join(&m.source), &m.text).unwrap();
        }
        let loaded = super::super::load_source_library(&requirement, &root).unwrap();
        assert_eq!(records(&loaded), records(&input));
        let tools = crate::LlvmTools::discover(None).unwrap();
        let library =
            super::super::build_source_library(&requirement, &loaded, &tools, &root).unwrap();
        assert_eq!(library.record.modules.len(), 2);
        assert!(library.record.maximum_stack_bytes <= 1024);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn closed_snapshots_reject_missing_extra_conflicting_cyclic_and_effectful_inputs() {
        // TOPAL-SYSTEMS-SOURCE-MODULES-001
        let (requirement, input) = finite();
        for mode in 0..13 {
            let mut changed = input.clone();
            match mode {
                0 => {
                    changed.modules.pop();
                }
                1 => {
                    changed.modules.push(changed.modules[0].clone());
                }
                2 => changed.modules[0].source = "../outside.t".into(),
                3 => changed.modules[0].identity = vec!["other".into()],
                4 => changed.modules[1].text.push_str("\nside-effect is 4\n"),
                5 => {
                    changed.modules[1].text = changed.modules[1]
                        .text
                        .replace("  not present", "  support boot missing? present")
                        .replace(
                            "use language (version is v0.1)",
                            "use language (version is v0.1)\nuse library support (version is v0.1)",
                        )
                }
                6 => {
                    changed.modules[0].text =
                        changed.modules[0].text.replace("pub missing?", "missing?")
                }
                7 => {
                    changed.text = changed
                        .text
                        .replace("  support boot missing? directory", "  directory")
                }
                8 => {
                    changed.modules[0].text = changed.modules[0]
                        .text
                        .replace("use library support (version is v0.1)\n", "")
                }
                9 => changed.modules[1].source = "support/./logic.t".into(),
                10 => {
                    changed.modules[1].text = changed.modules[1].text.replace(
                        "  not present",
                        "  private-copy is ReadDescription\n  private-copy = ReadDescription",
                    )
                }
                11 => changed.modules[1]
                    .text
                    .push_str("\npub Hidden is Enum (One, Two)\n"),
                12 => {
                    changed.text.push_str(
                        "\nprivate-root is fn (present : Boolean) -> Boolean\n  not present\n",
                    );
                    changed.modules[1].text = changed.modules[1]
                        .text
                        .replace("  not present", "  private-root present");
                }
                _ => unreachable!(),
            }
            assert!(
                super::super::check_source_library(&requirement, &changed).is_err(),
                "mode {mode}"
            );
        }
    }
    #[test]
    fn numeric_imports_keep_declared_types_and_complete_allocation_certificate() {
        // TOPAL-SYSTEMS-SOURCE-MODULES-001, TOPAL-SYSTEMS-NUMERIC-RUNTIME-001
        let requirement=topal_language::compiler::CompilerSystemsSourceLibraryRequirement {
            source:"policy/time.t".into(),export:"sleep-deadline".into(),
            interface:topal_language::compiler::CompilerSystemsSourceLibraryInterface::RelativeSleepDeadlineV1,
            stack_limit_bytes:4096,runtime_limit_bytes:Some(65536),
        };
        let text =
            include_str!("../../../linux-kernel/kernel/policy/relative-sleep-deadline.t").into();
        let input=SystemsSourceLibraryInput {source:requirement.source.clone(),source_name:requirement.source.clone(),text,modules:vec![SystemsSourceModuleInput {
            source:"support/clock.t".into(),identity:vec!["support".into(),"clock".into()],text:"use language (version is v0.1)\nfirst-quotient is fn (quotient : Int, remainder : Int) -> Int\n  quotient\npub rounded is fn (target : Nat) -> Int\n  ten : Nat is 10\n  nine : Nat is 9\n  first-quotient ((target + nine) /% ten)\n".into()
        }]};
        let checked =
            crate::systems_numeric_library::check_numeric_library(&requirement, &input).unwrap();
        assert_eq!(checked.program.dependencies.len(), 1);
        assert!(checked.allocation.maximum_bytes <= 65536);
        let export = checked
            .program
            .functions
            .iter()
            .find(|f| f.source_name == "sleep-deadline")
            .unwrap();
        assert!(export.parameters.iter().all(|p| p.int_range.is_none()));
        let llvm = crate::codegen::emit_numeric_kernel_library(&checked).unwrap();
        assert!(llvm.contains("filename: \"clock.t\", directory: \"support\""));
    }
    #[test]
    fn discovery_budgets_symlinks_and_provenance_mismatches_fail_closed() {
        // TOPAL-SYSTEMS-SOURCE-MODULES-001
        let (requirement, input) = finite();
        let mut oversized = input.clone();
        oversized.text.push_str(&" ".repeat(MAX_BYTES));
        assert!(
            validate(&oversized)
                .unwrap_err()
                .to_string()
                .contains("1 MiB")
        );
        oversized = input.clone();
        oversized.modules = vec![input.modules[0].clone(); MAX_MODULES + 1];
        assert!(validate(&oversized).unwrap_err().to_string().contains("64"));
        let records = records(&input);
        let digest = format!("{:x}", Sha256::digest(snapshot(&input).unwrap()));
        let inputs = records
            .iter()
            .map(|m| crate::DigestEntry {
                identity: format!("kernel-source-module:{}", m.source),
                sha256: m.sha256.clone(),
            })
            .collect::<Vec<_>>();
        let outputs = vec![crate::DigestEntry {
            identity: "kernel-library.modules.json".into(),
            sha256: digest.clone(),
        }];
        let check = |records: &[SystemsSourceModuleArtifact],
                     inputs: &[crate::DigestEntry],
                     outputs: &[crate::DigestEntry]| {
            validate_binding(
                records,
                &digest,
                inputs,
                outputs,
                "kernel-source-module:",
                "kernel-library.modules.json",
            )
        };
        check(&records, &inputs, &outputs).unwrap();
        assert!(check(&records, &inputs[..1], &outputs).is_err());
        assert!(check(&records, &inputs, &[]).is_err());
        let mut duplicate = outputs.clone();
        duplicate.extend(outputs.clone());
        assert!(check(&records, &inputs, &duplicate).is_err());
        let mut wrong = records.clone();
        wrong[0].sha256 = "0".repeat(64);
        assert!(check(&wrong, &inputs, &outputs).is_err());
        let root = std::env::temp_dir().join(format!("topal-module-budget-{}", std::process::id()));
        std::fs::create_dir_all(root.join("policy")).unwrap();
        std::fs::create_dir_all(root.join("support")).unwrap();
        std::fs::write(root.join(&input.source), &input.text).unwrap();
        for m in &input.modules {
            std::fs::write(root.join(&m.source), &m.text).unwrap();
        }
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(root.join("support/logic.t"), root.join("support/alias.t"))
                .unwrap();
            assert!(
                super::super::load_source_library(&requirement, &root)
                    .unwrap_err()
                    .to_string()
                    .contains("symlink")
            );
            std::fs::remove_file(root.join("support/alias.t")).unwrap();
        }
        for i in 0..MAX_ENTRIES {
            std::fs::write(root.join(format!("support/unused-{i}")), []).unwrap();
        }
        assert!(
            super::super::load_source_library(&requirement, &root)
                .unwrap_err()
                .to_string()
                .contains("4096")
        );
        std::fs::remove_dir_all(root).unwrap();
    }
}
