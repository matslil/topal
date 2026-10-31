//! Inspect the actual numeric object, direct calls, frames and qualified hooks.
use super::{EXPORT_SYMBOL, SystemsNumericAllocationBound, check_numeric_library, invalid};
use crate::{CompileError, LlvmTools, SystemsLibraryFunctionStack, SystemsSourceLibraryInput};
use object::{
    Object, ObjectKind, ObjectSection, ObjectSymbol, RelocationTarget, SectionFlags, SymbolKind,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;
use topal_language::compiler::CompilerSystemsSourceLibraryRequirement;

pub(crate) const ALLOCATE: &str = "topal_kernel_numeric_allocate_v1";
pub(crate) const ABORT: &str = "topal_kernel_numeric_abort_v1";
const ALIGNMENT_RESERVE: u64 = 32;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsNumericLibraryArtifact {
    pub schema: String,
    pub interface: String,
    pub source: String,
    pub source_name: String,
    pub source_export: String,
    pub native_export: String,
    pub source_sha256: String,
    pub checked_client_sha256: String,
    pub llvm_sha256: String,
    pub retained_llvm_sha256: String,
    pub platform_hooks_sha256: String,
    pub object_sha256: String,
    pub llvm_version: String,
    pub machine_profile: String,
    pub disassembly_sha256: String,
    pub native_alignment_reserve_bytes: u64,
    pub native_leaf_return_bytes: u64,
    pub maximum_stack_bytes: u64,
    pub stack_limit_bytes: u64,
    pub functions: Vec<SystemsLibraryFunctionStack>,
    pub allocation: SystemsNumericAllocationBound,
}
pub(crate) struct CompiledNumericLibrary {
    pub(crate) source: String,
    pub(crate) llvm: String,
    pub(crate) retained_llvm: String,
    pub(crate) object: Vec<u8>,
    pub(crate) record: SystemsNumericLibraryArtifact,
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn run(command: &mut Command) -> Result<String, CompileError> {
    let output = command.output().map_err(|e| invalid(e.to_string()))?;
    if !output.status.success() {
        return Err(invalid(format!(
            "{command:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    String::from_utf8(output.stdout).map_err(|e| invalid(e.to_string()))
}
pub(crate) fn build_numeric_library(
    requirement: &CompilerSystemsSourceLibraryRequirement,
    input: &SystemsSourceLibraryInput,
    tools: &LlvmTools,
    work: &Path,
) -> Result<CompiledNumericLibrary, CompileError> {
    let checked = check_numeric_library(requirement, input)?;
    let llvm = crate::codegen::emit_numeric_kernel_library(&checked)?;
    let source = work.join("time-library.ll");
    let bc = work.join("time-library.bc");
    let retained = work.join("time-library.retained.ll");
    let object = work.join("time-library.o");
    fs::write(&source, &llvm).map_err(|e| invalid(e.to_string()))?;
    run(Command::new(tools.tool_path("llvm-as"))
        .arg(&source)
        .arg("-o")
        .arg(&bc))?;
    run(Command::new(tools.tool_path("opt"))
        .arg("-passes=globaldce,verify")
        .arg("-S")
        .arg(&bc)
        .arg("-o")
        .arg(&retained))?;
    run(Command::new(tools.tool_path("llvm-as"))
        .arg(&retained)
        .arg("-o")
        .arg(&bc))?;
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
        .arg(&bc)
        .arg("-o")
        .arg(&object))?;
    let bytes = fs::read(&object).map_err(|e| invalid(e.to_string()))?;
    let file = object::File::parse(bytes.as_slice()).map_err(|e| invalid(e.to_string()))?;
    if file.kind() != ObjectKind::Relocatable || file.architecture() != object::Architecture::X86_64
    {
        return Err(invalid("wrong numeric object target/kind"));
    }
    for section in file.sections() {
        if let SectionFlags::Elf { sh_flags, .. } = section.flags() {
            if (section.size() > 0 && sh_flags.0 & 3 == 3) || sh_flags.0 & 0x400 != 0 {
                return Err(invalid(
                    "numeric object owns writable allocated or TLS data",
                ));
            }
        }
    }
    let expected_source: BTreeSet<_> = checked
        .program
        .functions
        .iter()
        .map(|f| format!("{}{}", super::FUNCTION_PREFIX, f.symbol))
        .collect();
    let mut names = BTreeSet::new();
    let mut undefined = BTreeSet::new();
    for symbol in file.symbols() {
        let name = symbol.name().map_err(|e| invalid(e.to_string()))?;
        if symbol.is_undefined() && !name.is_empty() {
            undefined.insert(name.to_owned());
        }
        if symbol.kind() == SymbolKind::Text && symbol.is_definition() {
            if symbol.is_global() && name != EXPORT_SYMBOL {
                return Err(invalid("unexpected exported numeric symbol"));
            }
            if name != EXPORT_SYMBOL
                && !expected_source.contains(name)
                && !name.starts_with("topal.runtime.int.")
                && ![
                    "topal.platform.allocate",
                    "topal.platform.exit",
                    "topal.platform.write",
                    "memset",
                ]
                .contains(&name)
            {
                return Err(invalid(format!("unexpected numeric code {name}")));
            }
            names.insert(name.to_owned());
        }
    }
    if undefined != BTreeSet::from([ALLOCATE.to_owned(), ABORT.to_owned()])
        || !names.contains(EXPORT_SYMBOL)
        || !expected_source.iter().all(|s| names.contains(s))
    {
        return Err(invalid(format!(
            "incomplete numeric closure/dependencies: {undefined:?}"
        )));
    }
    let frames = crate::systems_library::stack_sizes(&file)?;
    if frames.keys().cloned().collect::<BTreeSet<_>>() != names {
        return Err(invalid("incomplete numeric frame metadata"));
    }
    let calls = object_calls(&file, &names)?;
    let mut costs = BTreeMap::from([(ALLOCATE.to_owned(), 8), (ABORT.to_owned(), 8)]);
    let mut records = Vec::new();
    let mut visiting = BTreeSet::new();
    fn visit(
        name: &str,
        frames: &BTreeMap<String, u64>,
        calls: &BTreeMap<String, BTreeSet<String>>,
        costs: &mut BTreeMap<String, u64>,
        visiting: &mut BTreeSet<String>,
        records: &mut Vec<SystemsLibraryFunctionStack>,
    ) -> Result<u64, CompileError> {
        if let Some(cost) = costs.get(name) {
            return Ok(*cost);
        }
        if !visiting.insert(name.to_owned()) {
            return Err(invalid("cyclic numeric native call graph"));
        }
        let mut child = 0;
        for callee in &calls[name] {
            child = child.max(visit(callee, frames, calls, costs, visiting, records)?);
        }
        let total = frames[name]
            .checked_add(8)
            .and_then(|n| n.checked_add(child))
            .ok_or_else(|| invalid("numeric frame overflow"))?;
        costs.insert(name.to_owned(), total);
        visiting.remove(name);
        records.push(SystemsLibraryFunctionStack {
            symbol: name.into(),
            frame_bytes: frames[name],
            maximum_call_path_bytes: total,
            callees: calls[name].iter().cloned().collect(),
        });
        Ok(total)
    }
    for name in &names {
        visit(
            name,
            &frames,
            &calls,
            &mut costs,
            &mut visiting,
            &mut records,
        )?;
    }
    let maximum_stack_bytes = costs[EXPORT_SYMBOL]
        .checked_add(ALIGNMENT_RESERVE)
        .ok_or_else(|| invalid("numeric shim frame overflow"))?;
    if maximum_stack_bytes > requirement.stack_limit_bytes {
        return Err(invalid(format!(
            "numeric stack requires {maximum_stack_bytes} bytes"
        )));
    }
    let disassembly = run(Command::new(tools.tool_path("llvm-objdump"))
        .args(["-d", "--no-show-raw-insn"])
        .arg(&object))?;
    crate::systems_library::inspect_instructions(&disassembly)?;
    let retained_llvm = fs::read_to_string(&retained).map_err(|e| invalid(e.to_string()))?;
    let record = SystemsNumericLibraryArtifact {
        schema: "topal.kernel-numeric-source-library/1".into(),
        interface: requirement.interface.as_str().into(),
        source: input.source.clone(),
        source_name: input.source_name.clone(),
        source_export: requirement.export.clone(),
        native_export: EXPORT_SYMBOL.into(),
        source_sha256: digest(input.text.as_bytes()),
        checked_client_sha256: digest(checked.program.source.as_str().as_bytes()),
        llvm_sha256: digest(llvm.as_bytes()),
        retained_llvm_sha256: digest(retained_llvm.as_bytes()),
        platform_hooks_sha256: digest(include_bytes!("runtime/kernel_numeric_platform.ll")),
        object_sha256: digest(&bytes),
        llvm_version: tools.version.clone(),
        machine_profile: "x86_64-owned-numeric-gpr-only/1".into(),
        disassembly_sha256: digest(disassembly.as_bytes()),
        native_alignment_reserve_bytes: ALIGNMENT_RESERVE,
        native_leaf_return_bytes: 8,
        maximum_stack_bytes,
        stack_limit_bytes: requirement.stack_limit_bytes,
        functions: records,
        allocation: checked.allocation,
    };
    Ok(CompiledNumericLibrary {
        source: input.text.clone(),
        llvm,
        retained_llvm,
        object: bytes,
        record,
    })
}

fn object_calls(
    file: &object::File<'_>,
    names: &BTreeSet<String>,
) -> Result<BTreeMap<String, BTreeSet<String>>, CompileError> {
    let mut result = BTreeMap::new();
    for name in names {
        let function = file
            .symbol_by_name(name)
            .ok_or_else(|| invalid("numeric function missing"))?;
        let section = file
            .section_by_index(
                function
                    .section_index()
                    .ok_or_else(|| invalid("numeric function section missing"))?,
            )
            .map_err(|e| invalid(e.to_string()))?;
        let data = section.data().map_err(|e| invalid(e.to_string()))?;
        // Function sections ensure every inter-function call has a relocation.
        if function.address() != 0 || function.size() != section.size() {
            return Err(invalid(
                "numeric function does not own its whole text section",
            ));
        }
        let mut edges = BTreeSet::new();
        for (offset, relocation) in section.relocations() {
            if offset == 0 || data.get(offset as usize - 1) != Some(&0xe8) {
                continue;
            }
            if relocation.size() != 32
                || !matches!(
                    relocation.kind(),
                    object::RelocationKind::Relative | object::RelocationKind::PltRelative
                )
            {
                return Err(invalid("unqualified numeric call relocation"));
            }
            let target = match relocation.target() {
                RelocationTarget::Symbol(index) => {
                    let symbol = file
                        .symbol_by_index(index)
                        .map_err(|e| invalid(e.to_string()))?;
                    if symbol.kind() == SymbolKind::Section {
                        let section = symbol
                            .section_index()
                            .ok_or_else(|| invalid("call target section missing"))?;
                        let address = symbol
                            .address()
                            .checked_add_signed(
                                relocation
                                    .addend()
                                    .checked_add(4)
                                    .ok_or_else(|| invalid("call relocation addend overflow"))?,
                            )
                            .ok_or_else(|| invalid("call target overflow"))?;
                        function_in_section(file, section, address)?
                    } else {
                        if relocation.addend() != -4 {
                            return Err(invalid("call is not to a function entry"));
                        }
                        symbol
                            .name()
                            .map_err(|e| invalid(e.to_string()))?
                            .to_owned()
                    }
                }
                RelocationTarget::Section(section) => function_in_section(
                    file,
                    section,
                    u64::try_from(
                        relocation
                            .addend()
                            .checked_add(4)
                            .ok_or_else(|| invalid("call relocation addend overflow"))?,
                    )
                    .map_err(|_| invalid("negative call target"))?,
                )?,
                _ => return Err(invalid("unknown numeric call target")),
            };
            if !names.contains(&target) && ![ALLOCATE, ABORT].contains(&target.as_str()) {
                return Err(invalid(format!("unqualified numeric callee {target}")));
            }
            edges.insert(target);
        }
        result.insert(name.clone(), edges);
    }
    Ok(result)
}
fn function_in_section(
    file: &object::File<'_>,
    section: object::SectionIndex,
    address: u64,
) -> Result<String, CompileError> {
    let found = file
        .symbols()
        .filter(|s| {
            s.is_definition()
                && s.kind() == SymbolKind::Text
                && s.section_index() == Some(section)
                && s.address() == address
        })
        .collect::<Vec<_>>();
    if found.len() != 1 {
        return Err(invalid("ambiguous numeric call entry"));
    }
    Ok(found[0].name().map_err(|e| invalid(e.to_string()))?.into())
}

pub(crate) fn inspect_linked_numeric_library(
    bytes: &[u8],
    record: &SystemsNumericLibraryArtifact,
) -> Result<(), CompileError> {
    let file = object::File::parse(bytes).map_err(|e| invalid(e.to_string()))?;
    if record.schema != "topal.kernel-numeric-source-library/1"
        || record.interface != "relative-sleep-deadline-v1"
        || record.native_export != EXPORT_SYMBOL
        || record.stack_limit_bytes != 4096
        || record.maximum_stack_bytes > 4096
        || record.native_alignment_reserve_bytes != 32
        || record.native_leaf_return_bytes != 8
        || record.allocation.limit_bytes != 65536
        || record.allocation.maximum_bytes > 65536
        || record.allocation.header_bytes != 32
        || record.allocation.runtime_sha256 != super::CERTIFIED_RUNTIME_SHA256
        || record.machine_profile != "x86_64-owned-numeric-gpr-only/1"
        || !record.functions.iter().any(|f| f.symbol == EXPORT_SYMBOL)
    {
        return Err(invalid(
            "incomplete numeric binding or resource certificate",
        ));
    }
    let mut seen = BTreeSet::new();
    for function in &record.functions {
        if !seen.insert(&function.symbol) {
            return Err(invalid("duplicate numeric function binding"));
        }
        let found = file
            .symbols()
            .filter(|s| s.is_definition() && s.name().ok() == Some(function.symbol.as_str()))
            .collect::<Vec<_>>();
        if found.len() != 1 {
            return Err(invalid("missing or ambiguous numeric function binding"));
        }
        let symbol = &found[0];
        let end = symbol
            .address()
            .checked_add(symbol.size())
            .ok_or_else(|| invalid("numeric code extent overflow"))?;
        if symbol.kind() != SymbolKind::Text
            || symbol.size() == 0
            || symbol.address() < 0x200000
            || end > 0x40000000
            || (symbol.address() < 0x800000 && end > 0x400000)
        {
            return Err(invalid("numeric code escapes supervisor identity mappings"));
        }
        let section = file
            .section_by_index(
                symbol
                    .section_index()
                    .ok_or_else(|| invalid("numeric code has no section"))?,
            )
            .map_err(|e| invalid(e.to_string()))?;
        if !matches!(section.flags(),SectionFlags::Elf {sh_flags,..} if sh_flags.0&4!=0 && sh_flags.0&1==0)
        {
            return Err(invalid("numeric code is not immutable executable text"));
        }
    }
    let arena = file
        .symbol_by_name("topal_x86_systems_numeric_arena")
        .ok_or_else(|| invalid("numeric owner storage missing"))?;
    let end = arena
        .address()
        .checked_add(arena.size())
        .ok_or_else(|| invalid("numeric arena extent overflow"))?;
    if !arena.is_definition()
        || arena.kind() != SymbolKind::Data
        || arena.size() != 65536
        || arena.address() % 4096 != 0
        || arena.address() < 0x200000
        || end > 0x40000000
        || (arena.address() < 0x800000 && end > 0x400000)
    {
        return Err(invalid(format!(
            "unqualified numeric owner storage: address={:#x} size={} kind={:?}",
            arena.address(),
            arena.size(),
            arena.kind()
        )));
    }
    let section = file
        .section_by_index(
            arena
                .section_index()
                .ok_or_else(|| invalid("numeric arena has no section"))?,
        )
        .map_err(|e| invalid(e.to_string()))?;
    if !matches!(section.flags(),SectionFlags::Elf {sh_flags,..} if sh_flags.0&3==3 && sh_flags.0&4==0)
    {
        return Err(invalid(
            "numeric owner storage is not writable nonexecutable data",
        ));
    }
    Ok(())
}

/// Discharge the external-leaf assumption used by the numeric frame bound.
pub(crate) fn inspect_numeric_provider(tools: &LlvmTools, path: &Path) -> Result<(), CompileError> {
    let disassembly = run(Command::new(tools.tool_path("llvm-objdump"))
        .arg("-d")
        .arg("--no-show-raw-insn")
        .arg(format!("--disassemble-symbols={ALLOCATE},{ABORT}"))
        .arg(path))?;
    let bytes = fs::read(path).map_err(|e| invalid(e.to_string()))?;
    let file = object::File::parse(bytes.as_slice()).map_err(|e| invalid(e.to_string()))?;
    let mut ranges = Vec::new();
    for name in [ALLOCATE, ABORT] {
        let symbol = file
            .symbol_by_name(name)
            .ok_or_else(|| invalid("numeric runtime hook missing"))?;
        let end = symbol
            .address()
            .checked_add(symbol.size())
            .ok_or_else(|| invalid("numeric hook extent overflow"))?;
        if !symbol.is_definition() || symbol.kind() != SymbolKind::Text || symbol.size() == 0 {
            return Err(invalid("invalid numeric runtime hook"));
        }
        ranges.push((symbol.address(), end));
    }
    let mut observed = BTreeSet::new();
    let mut count = 0;
    for line in disassembly.lines() {
        let Some((address, body)) = line.split_once(':') else {
            continue;
        };
        if address.trim().is_empty() || !address.trim().chars().all(|c| c.is_ascii_hexdigit()) {
            continue;
        }
        let address =
            u64::from_str_radix(address.trim(), 16).map_err(|e| invalid(e.to_string()))?;
        let Some(index) = ranges
            .iter()
            .position(|(start, end)| *start <= address && address < *end)
        else {
            continue;
        };
        observed.insert(index);
        let Some(mnemonic) = body.split_whitespace().next() else {
            continue;
        };
        count += 1;
        if ["push", "pop", "call", "enter", "leave"]
            .iter()
            .any(|prefix| mnemonic.starts_with(prefix))
            || [
                "%rsp", "%esp", "%sp", "%spl", "%rbp", "%ebp", "%bp", "%bpl", "%rbx", "%ebx",
                "%bx", "%bl", "%bh", "%r12", "%r13", "%r14", "%r15", "%xmm", "%ymm", "%zmm", "%mm",
                "%st",
            ]
            .iter()
            .any(|reg| body.contains(reg))
            || mnemonic.starts_with('f')
            || mnemonic.starts_with('v')
            || [
                "syscall", "sysenter", "swapgs", "wrmsr", "rdmsr", "iret", "iretq", "ldmxcsr",
                "stmxcsr",
            ]
            .contains(&mnemonic)
        {
            return Err(invalid(format!(
                "numeric runtime hook violates leaf ABI: {}",
                line.trim()
            )));
        }
    }
    if count == 0 || observed.len() != 2 {
        return Err(invalid(
            "numeric runtime hook inspection found incomplete code",
        ));
    }
    Ok(())
}
