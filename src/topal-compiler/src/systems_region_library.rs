//! Checked standalone source objects; kernel-root integration remains separate.
use crate::{CompileError, LlvmTools, SystemsLibraryFunctionStack, SystemsSourceLibraryInput};
use object::{Object, ObjectSection, ObjectSymbol, SectionFlags, SymbolKind};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;
use topal_language::compiler::{
    CompilerKernelRegionEffects, CompilerKernelRegionProfile, CompilerProgram,
};

#[path = "systems_region_data.rs"]
mod data;
pub use data::{
    SystemsRegionReadOnlyBinding, SystemsRegionReadOnlyData, inspect_systems_region_action_data,
    inspect_systems_region_read_only_data,
};

pub(crate) const EXPORT: &str = "topal_kernel_region_apply_v1";
pub(crate) const PREFIX: &str = "topal.region.";
pub(crate) const ALLOCATE: &str = "topal_kernel_region_allocate_v1";
pub(crate) const ABORT: &str = "topal_kernel_region_abort_v1";
pub(crate) const ACTION_EXPORT: &str = "topal_kernel_region_action_apply_v1";
pub(crate) const ACTION_PREFIX: &str = "topal.region.action.";
pub(crate) const ACTION_ALLOCATE: &str = "topal_kernel_region_action_allocate_v1";
pub(crate) const ACTION_ABORT: &str = "topal_kernel_region_action_abort_v1";
pub(crate) const FRAME_EXPORT: &str = "topal_kernel_region_frame_apply_v1";
pub(crate) const FRAME_PREFIX: &str = "topal.framed-region.";
pub(crate) const FRAME_ALLOCATE: &str = "topal_kernel_region_frame_allocate_v1";
pub(crate) const FRAME_ABORT: &str = "topal_kernel_region_frame_abort_v1";
// Reviewed <=8-byte loops: no allocation in span/store; load at most 24 bytes
// for int.from.u64 plus 16 for Some (or 16 for None). Shared callees retain
// their existing independent runtime certificates; see TK-DEC-279.
pub(crate) const UNSIGNED_RUNTIME_SHA256: &str =
    "d4671003ce1b40ec4b6e732815267d193ab1851980c8405ffdc852d69784a5c4";

pub(crate) fn source_object_schema(
    profile: CompilerKernelRegionProfile,
    instance: bool,
    unsigned: bool,
) -> &'static str {
    match (profile, instance, unsigned) {
        (CompilerKernelRegionProfile::Unit, _, false) => {
            "topal.kernel-region-source-object.x86_64/3"
        }
        (CompilerKernelRegionProfile::Unit, _, true) => {
            "topal.kernel-region-source-object.x86_64/4"
        }
        (CompilerKernelRegionProfile::Action, _, false) => {
            "topal.kernel-region-action-source-object.x86_64/1"
        }
        (CompilerKernelRegionProfile::Action, _, true) => {
            "topal.kernel-region-action-source-object.x86_64/2"
        }
        (CompilerKernelRegionProfile::FramedAction { .. }, false, false) => {
            "topal.kernel-region-frame-source-object.x86_64/1"
        }
        (CompilerKernelRegionProfile::FramedAction { .. }, true, false) => {
            "topal.kernel-region-frame-source-object.x86_64/2"
        }
        (CompilerKernelRegionProfile::FramedAction { .. }, false, true) => {
            "topal.kernel-region-frame-source-object.x86_64/3"
        }
        (CompilerKernelRegionProfile::FramedAction { .. }, true, true) => {
            "topal.kernel-region-frame-source-object.x86_64/4"
        }
    }
}

#[path = "systems_region_instance.rs"]
mod instance;
pub(crate) use instance::*;

/// Missing optional profile fields remain absent; a present JSON null cannot
/// erase a framed-only key before the legacy context checks see it.
pub(crate) fn deserialize_present_region_field<'de, T, D>(
    deserializer: D,
) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    T::deserialize(deserializer).map(Some)
}

impl CheckedRegionLibrary {
    pub(crate) fn native_export(&self) -> &str {
        self.native_names().0
    }
    pub(crate) fn native_names(&self) -> (&str, &str, &str) {
        (
            &self.symbols.export,
            &self.symbols.allocate,
            &self.symbols.abort,
        )
    }
    pub(crate) fn prefix(&self) -> &str {
        &self.symbols.prefix
    }
}

fn invalid(message: impl Into<String>) -> CompileError {
    CompileError::Tool(format!("E-SYSTEMS-REGION-LIBRARY: {}", message.into()))
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn run(command: &mut Command) -> Result<String, CompileError> {
    let output = command
        .output()
        .map_err(|error| invalid(error.to_string()))?;
    if !output.status.success() {
        return Err(invalid(format!(
            "{command:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    String::from_utf8(output.stdout).map_err(|error| invalid(error.to_string()))
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsRegionEffect {
    pub symbol: String,
    pub read_owner: bool,
    pub write_owner: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsRegionLibraryArtifact {
    pub schema: String,
    pub source: String,
    pub source_name: String,
    pub source_export: String,
    pub native_export: String,
    pub source_sha256: String,
    pub modules: Vec<crate::SystemsSourceModuleArtifact>,
    pub module_snapshot_sha256: String,
    pub declared_entry: String,
    /// Only framed objects carry a count; legacy serialization remains intact.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present_region_field"
    )]
    pub scalar_count: Option<u8>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_present_region_field"
    )]
    pub instance: Option<String>,
    pub effects: Vec<SystemsRegionEffect>,
    pub runtime_sha256: String,
    pub platform_hooks_sha256: String,
    pub llvm_sha256: String,
    pub retained_llvm_sha256: String,
    pub object_sha256: String,
    pub disassembly_sha256: String,
    pub llvm_version: String,
    pub machine_profile: String,
    pub target: String,
    pub data_layout: String,
    pub native_abi: String,
    pub allocation: crate::systems_numeric_library::SystemsRegionAllocationBound,
    pub maximum_stack_bytes: u64,
    pub stack_limit_bytes: u64,
    pub imported_hook_stack_bytes: u64,
    pub imported_hooks: Vec<String>,
    pub functions: Vec<SystemsLibraryFunctionStack>,
    /// New action-only evidence; the Unit schema/serialization stays unchanged.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub read_only_data: Vec<SystemsRegionReadOnlyData>,
}

#[derive(Clone)]
pub struct CompiledSystemsRegionLibrary {
    pub source: String,
    pub module_snapshot: Vec<u8>,
    pub llvm: String,
    pub retained_llvm: String,
    pub object: Vec<u8>,
    pub record: SystemsRegionLibraryArtifact,
}

pub(crate) struct CheckedRegionLibrary {
    pub(crate) program: CompilerProgram,
    pub(crate) source_name: String,
    pub(crate) entry_symbol: String,
    pub(crate) profile: CompilerKernelRegionProfile,
    pub(crate) symbols: RegionNativeSymbols,
}

pub(crate) fn check_region_isolation(
    input: &SystemsSourceLibraryInput,
    program: &CompilerProgram,
    modules: &[topal_language::modules::SourceModule],
) -> Result<(), CompileError> {
    let mut exports = Vec::new();
    for function in &program.functions {
        let Some(identity) = &function.module_identity else {
            continue;
        };
        let module = input
            .modules
            .iter()
            .find(|module| &module.identity == identity)
            .ok_or_else(|| invalid("function without a frozen module owner"))?;
        let name = function
            .source_name
            .rsplit('.')
            .next()
            .ok_or_else(|| invalid("missing module function name"))?;
        if !topal_language::modules::published_function_names(&module.text)
            .iter()
            .any(|candidate| topal_source::canonically_equal(candidate, name))
        {
            continue;
        }
        exports.push(topal_language::compiler::CompilerKernelRegionModuleExport {
            module: identity.clone(),
            name: name.into(),
            parameters: function
                .parameters
                .iter()
                .map(|parameter| parameter.value_type.clone())
                .collect(),
            result: function.result_type.clone(),
        });
    }
    if !exports.is_empty() {
        topal_language::compiler::analyze_kernel_region_module_exports(modules, &exports).map_err(
            |error| {
                invalid(format!(
                    "module closure depends on an unqualified or private caller scope: {}",
                    error.render("module-isolation.t")
                ))
            },
        )?;
    }
    Ok(())
}

/// Compile and inspect one source object in caller-owned temporary work space.
/// The result supplies no live region, kernel-root binding or cleanup authority.
pub fn build_systems_region_library(
    input: &SystemsSourceLibraryInput,
    export: &str,
    tools: &LlvmTools,
    work: &Path,
) -> Result<CompiledSystemsRegionLibrary, CompileError> {
    build_region_profile(
        input,
        export,
        tools,
        work,
        CompilerKernelRegionProfile::Unit,
        None,
    )
}

/// Compile a separate action context; this grants no persistent field authority.
pub fn build_systems_region_action_library(
    input: &SystemsSourceLibraryInput,
    export: &str,
    tools: &LlvmTools,
    work: &Path,
) -> Result<CompiledSystemsRegionLibrary, CompileError> {
    build_region_profile(
        input,
        export,
        tools,
        work,
        CompilerKernelRegionProfile::Action,
        None,
    )
}

pub fn build_systems_region_framed_action_library(
    input: &SystemsSourceLibraryInput,
    export: &str,
    scalar_count: u8,
    tools: &LlvmTools,
    work: &Path,
) -> Result<CompiledSystemsRegionLibrary, CompileError> {
    build_region_profile(
        input,
        export,
        tools,
        work,
        CompilerKernelRegionProfile::FramedAction { scalar_count },
        None,
    )
}

/// Build an independently named object; the label confers no live caller grant.
pub fn build_systems_region_instance_library(
    input: &SystemsSourceLibraryInput,
    export: &str,
    scalar_count: u8,
    instance: &str,
    tools: &LlvmTools,
    work: &Path,
) -> Result<CompiledSystemsRegionLibrary, CompileError> {
    build_region_profile(
        input,
        export,
        tools,
        work,
        CompilerKernelRegionProfile::FramedAction { scalar_count },
        Some(instance),
    )
}

fn build_region_profile(
    input: &SystemsSourceLibraryInput,
    export: &str,
    tools: &LlvmTools,
    work: &Path,
    profile: CompilerKernelRegionProfile,
    instance: Option<&str>,
) -> Result<CompiledSystemsRegionLibrary, CompileError> {
    let symbols = RegionNativeSymbols::for_profile(profile, instance)?;
    let modules = crate::systems_library::modules::validate(input)?;
    let program = match profile {
        CompilerKernelRegionProfile::Unit => {
            topal_language::compiler::analyze_kernel_region_library(&input.text, &modules, export)
        }
        CompilerKernelRegionProfile::Action => {
            topal_language::compiler::analyze_kernel_region_action_library(
                &input.text,
                &modules,
                export,
            )
        }
        CompilerKernelRegionProfile::FramedAction { scalar_count } => {
            topal_language::compiler::analyze_kernel_region_framed_action_library(
                &input.text,
                &modules,
                export,
                scalar_count,
            )
        }
    }
    .map_err(|error| CompileError::Tool(error.render(&input.source_name)))?;
    check_region_isolation(input, &program, &modules)?;
    let effects = topal_language::compiler::kernel_region_function_effects(&program)
        .map_err(|error| CompileError::Tool(error.render(&input.source_name)))?;
    let entry = program
        .functions
        .iter()
        .filter(|f| {
            f.module_identity.is_none() && topal_source::canonically_equal(&f.source_name, export)
        })
        .collect::<Vec<_>>();
    let [entry] = entry.as_slice() else {
        return Err(invalid("missing or duplicate region entry specialization"));
    };
    let entry_symbol = entry.symbol.clone();
    let allocation = crate::systems_numeric_library::bound_region_library_profile(
        &program,
        &entry_symbol,
        profile,
    )?;
    let checked = CheckedRegionLibrary {
        program,
        source_name: input.source_name.clone(),
        entry_symbol,
        profile,
        symbols,
    };
    let (native_export, prefix) = (checked.native_export(), checked.prefix());
    let (_, allocate, abort) = checked.native_names();
    let llvm = crate::codegen::emit_region_kernel_library(&checked)?;
    let source = work.join("kernel-region.ll");
    let bitcode = work.join("kernel-region.bc");
    let retained = work.join("kernel-region.retained.ll");
    let object = work.join("kernel-region.o");
    std::fs::write(&source, &llvm).map_err(|error| invalid(error.to_string()))?;
    run(Command::new(tools.tool_path("llvm-as"))
        .arg(&source)
        .arg("-o")
        .arg(&bitcode))?;
    run(Command::new(tools.tool_path("opt"))
        .args(["-passes=globaldce,verify", "-S"])
        .arg(&bitcode)
        .arg("-o")
        .arg(&retained))?;
    // Tool work paths are not source provenance and must not affect replay.
    let retained_text =
        std::fs::read_to_string(&retained).map_err(|error| invalid(error.to_string()))?;
    let retained_llvm = retained_text
        .lines()
        .map(|line| {
            if line.starts_with("; ModuleID = ") {
                "; ModuleID = 'kernel-region'"
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    std::fs::write(&retained, &retained_llvm).map_err(|error| invalid(error.to_string()))?;
    run(Command::new(tools.tool_path("llvm-as"))
        .arg(&retained)
        .arg("-o")
        .arg(&bitcode))?;
    run(Command::new(tools.tool_path("llc"))
        .args([
            "-O0",
            "-filetype=obj",
            "-mtriple=x86_64-unknown-none",
            "-mcpu=x86-64",
            "-mattr=-mmx,-sse,-sse2,-avx,-avx2",
            "-relocation-model=static",
            "--frame-pointer=all",
            "-stack-size-section",
            "-function-sections",
            "--disable-tail-calls",
        ])
        .arg(&bitcode)
        .arg("-o")
        .arg(&object))?;
    let bytes = std::fs::read(&object).map_err(|error| invalid(error.to_string()))?;
    let file = object::File::parse(bytes.as_slice()).map_err(|error| invalid(error.to_string()))?;
    if file.kind() != object::ObjectKind::Relocatable
        || file.architecture() != object::Architecture::X86_64
    {
        return Err(invalid("incorrect source object architecture/kind"));
    }
    for section in file.sections() {
        if let SectionFlags::Elf { sh_flags, .. } = section.flags() {
            if (section.size() > 0 && sh_flags.0 & 3 == 3) || sh_flags.0 & 0x400 != 0 {
                return Err(invalid(
                    "region library owns writable allocated or TLS storage",
                ));
            }
        }
    }
    let expected_source = checked
        .program
        .functions
        .iter()
        .map(|f| format!("{prefix}{}", f.symbol))
        .collect::<BTreeSet<_>>();
    let mut names = BTreeSet::new();
    let mut undefined = BTreeSet::new();
    for symbol in file.symbols() {
        let name = symbol.name().map_err(|error| invalid(error.to_string()))?;
        if symbol.is_undefined() && !name.is_empty() {
            undefined.insert(name.to_owned());
        }
        if symbol.kind() == SymbolKind::Text && symbol.is_definition() {
            if symbol.is_global() && name != native_export {
                return Err(invalid(format!("unexpected region export {name}")));
            }
            if name != native_export
                && !expected_source.contains(name)
                && !name.starts_with(&format!("{prefix}runtime.int."))
                && !(profile == CompilerKernelRegionProfile::Unit
                    && name.starts_with("topal.kernel.region."))
                && !(profile != CompilerKernelRegionProfile::Unit
                    && name.starts_with(&format!("{prefix}primitive.")))
                && !name.starts_with(&format!("{prefix}runtime.optional."))
                && !name.starts_with(&format!("{prefix}runtime.list."))
                && !matches!(
                    name.strip_prefix(prefix).unwrap_or(""),
                    "runtime.container.list.count"
                        | "runtime.container.array.int.collect"
                        | "runtime.container.array.int.at"
                        | "runtime.container.entry.count"
                        | "runtime.container.empty"
                )
                && !name.starts_with(&format!("{prefix}platform."))
                && name != format!("{prefix}memset")
            {
                return Err(invalid(format!("unqualified region code {name}")));
            }
            names.insert(name.into());
        }
    }
    if undefined != BTreeSet::from([allocate.into(), abort.into()])
        || !names.contains(native_export)
        || !expected_source.iter().all(|name| names.contains(name))
    {
        return Err(invalid(format!(
            "incomplete region object closure: {undefined:?}"
        )));
    }
    let frames = crate::systems_library::stack_sizes(&file)?;
    if frames.keys().cloned().collect::<BTreeSet<_>>() != names {
        return Err(invalid("incomplete region stack metadata"));
    }
    let calls =
        crate::systems_numeric_library::library_object_calls(&file, &names, &[allocate, abort])?;
    let (maximum_stack_bytes, functions) =
        bound_stack(&names, &frames, &calls, native_export, allocate, abort)?;
    let disassembly = run(Command::new(tools.tool_path("llvm-objdump"))
        .args(["-d", "--no-show-raw-insn"])
        .arg(&object))?
    .replace(object.to_string_lossy().as_ref(), "kernel-region.o");
    crate::systems_library::inspect_instructions(&disassembly)?;
    let snapshot = crate::systems_library::modules::snapshot(input)?;
    let record = SystemsRegionLibraryArtifact {
        schema: source_object_schema(
            profile,
            checked.symbols.instance.is_some(),
            allocation.unsigned_runtime_sha256.is_some(),
        )
        .into(),
        source: input.source.clone(),
        source_name: input.source_name.clone(),
        source_export: export.into(),
        native_export: native_export.into(),
        source_sha256: digest(input.text.as_bytes()),
        modules: crate::systems_library::modules::records(input),
        module_snapshot_sha256: digest(&snapshot),
        declared_entry: match profile {
            CompilerKernelRegionProfile::Unit => "(KernelByteRegion) -> Unit".into(),
            CompilerKernelRegionProfile::Action => "(KernelByteRegion, Nat, Nat) -> Boolean".into(),
            CompilerKernelRegionProfile::FramedAction { scalar_count } => format!(
                "(KernelByteRegion{}) -> Boolean",
                ", Nat".repeat(usize::from(scalar_count))
            ),
        },
        scalar_count: match profile {
            CompilerKernelRegionProfile::FramedAction { scalar_count } => Some(scalar_count),
            _ => None,
        },
        instance: checked.symbols.instance.clone(),
        effects: effects
            .into_iter()
            .map(
                |CompilerKernelRegionEffects {
                     symbol,
                     read,
                     write,
                 }| SystemsRegionEffect {
                    symbol,
                    read_owner: read,
                    write_owner: write,
                },
            )
            .collect(),
        runtime_sha256: digest(include_bytes!("runtime/kernel_regions.ll")),
        platform_hooks_sha256: digest(include_bytes!("runtime/kernel_region_platform.ll")),
        llvm_sha256: digest(llvm.as_bytes()),
        retained_llvm_sha256: digest(retained_llvm.as_bytes()),
        object_sha256: digest(&bytes),
        disassembly_sha256: digest(disassembly.as_bytes()),
        llvm_version: tools.version.clone(),
        machine_profile: "x86_64-owned-region-gpr-only/1".into(),
        allocation,
        target: "x86_64-unknown-none".into(),
        data_layout: crate::X86_SYSTEMS_DATA_LAYOUT.into(),
        native_abi: match profile {
            CompilerKernelRegionProfile::Unit => "topal.kernel-region-control-view64.x86_64-sysv/1",
            CompilerKernelRegionProfile::Action => {
                "topal.kernel-region-action-control-view64-u64x2-bool.x86_64-sysv/1"
            }
            CompilerKernelRegionProfile::FramedAction { .. } => {
                "topal.kernel-region-frame-control-view64-u64frame-bool.x86_64-sysv/1"
            }
        }
        .into(),
        maximum_stack_bytes,
        stack_limit_bytes: 4096,
        imported_hook_stack_bytes: 8,
        imported_hooks: vec![allocate.into(), abort.into()],
        functions,
        read_only_data: data::object_data(
            &bytes,
            prefix,
            profile != CompilerKernelRegionProfile::Action,
        )?,
    };
    Ok(CompiledSystemsRegionLibrary {
        source: input.text.clone(),
        module_snapshot: snapshot,
        llvm,
        retained_llvm,
        object: bytes,
        record,
    })
}

/// Reproduce the source proof and complete inspected object in fresh caller-owned
/// work space. A deserialized record alone never qualifies a supplied object.
/// This does not bind the result to a live owner or a kernel root.
pub fn verify_systems_region_library(
    input: &SystemsSourceLibraryInput,
    export: &str,
    tools: &LlvmTools,
    work: &Path,
    candidate: &CompiledSystemsRegionLibrary,
) -> Result<(), CompileError> {
    verify_region_profile(
        input,
        export,
        tools,
        work,
        candidate,
        CompilerKernelRegionProfile::Unit,
        None,
    )
}

pub fn verify_systems_region_action_library(
    input: &SystemsSourceLibraryInput,
    export: &str,
    tools: &LlvmTools,
    work: &Path,
    candidate: &CompiledSystemsRegionLibrary,
) -> Result<(), CompileError> {
    verify_region_profile(
        input,
        export,
        tools,
        work,
        candidate,
        CompilerKernelRegionProfile::Action,
        None,
    )
}

pub fn verify_systems_region_framed_action_library(
    input: &SystemsSourceLibraryInput,
    export: &str,
    scalar_count: u8,
    tools: &LlvmTools,
    work: &Path,
    candidate: &CompiledSystemsRegionLibrary,
) -> Result<(), CompileError> {
    verify_region_profile(
        input,
        export,
        tools,
        work,
        candidate,
        CompilerKernelRegionProfile::FramedAction { scalar_count },
        None,
    )
}

pub fn verify_systems_region_instance_library(
    input: &SystemsSourceLibraryInput,
    export: &str,
    scalar_count: u8,
    instance: &str,
    tools: &LlvmTools,
    work: &Path,
    candidate: &CompiledSystemsRegionLibrary,
) -> Result<(), CompileError> {
    verify_region_profile(
        input,
        export,
        tools,
        work,
        candidate,
        CompilerKernelRegionProfile::FramedAction { scalar_count },
        Some(instance),
    )
}

fn verify_region_profile(
    input: &SystemsSourceLibraryInput,
    export: &str,
    tools: &LlvmTools,
    work: &Path,
    candidate: &CompiledSystemsRegionLibrary,
    profile: CompilerKernelRegionProfile,
    instance: Option<&str>,
) -> Result<(), CompileError> {
    let expected = build_region_profile(input, export, tools, work, profile, instance)?;
    if candidate.source != expected.source
        || candidate.module_snapshot != expected.module_snapshot
        || candidate.llvm != expected.llvm
        || candidate.retained_llvm != expected.retained_llvm
        || candidate.object != expected.object
        || candidate.record != expected.record
    {
        return Err(invalid(
            "source proof, runtime, object or complete certificate mismatch",
        ));
    }
    Ok(())
}

fn bound_stack(
    names: &BTreeSet<String>,
    frames: &BTreeMap<String, u64>,
    calls: &BTreeMap<String, BTreeSet<String>>,
    export: &str,
    allocate: &str,
    abort: &str,
) -> Result<(u64, Vec<SystemsLibraryFunctionStack>), CompileError> {
    fn visit(
        name: &str,
        frames: &BTreeMap<String, u64>,
        calls: &BTreeMap<String, BTreeSet<String>>,
        cache: &mut BTreeMap<String, u64>,
        active: &mut BTreeSet<String>,
        records: &mut Vec<SystemsLibraryFunctionStack>,
    ) -> Result<u64, CompileError> {
        if let Some(bound) = cache.get(name) {
            return Ok(*bound);
        }
        if !active.insert(name.into()) {
            return Err(invalid("cyclic region native call graph"));
        }
        let mut nested = 0;
        for callee in &calls[name] {
            nested = nested.max(visit(callee, frames, calls, cache, active, records)?);
        }
        let total = frames[name]
            .checked_add(8)
            .and_then(|value| value.checked_add(nested))
            .ok_or_else(|| invalid("region stack overflow"))?;
        cache.insert(name.into(), total);
        active.remove(name);
        records.push(SystemsLibraryFunctionStack {
            symbol: name.into(),
            frame_bytes: frames[name],
            maximum_call_path_bytes: total,
            callees: calls[name].iter().cloned().collect(),
        });
        Ok(total)
    }
    let mut cache = BTreeMap::from([(allocate.into(), 8), (abort.into(), 8)]);
    let mut records = Vec::new();
    let mut active = BTreeSet::new();
    for name in names {
        visit(name, frames, calls, &mut cache, &mut active, &mut records)?;
    }
    let maximum = cache[export]
        .checked_add(8)
        .ok_or_else(|| invalid("region caller alignment overflow"))?;
    if maximum > 4096 {
        return Err(invalid(format!(
            "region source needs {maximum} stack bytes"
        )));
    }
    Ok((maximum, records))
}

#[cfg(test)]
#[path = "systems_region_library_tests.rs"]
mod tests;
