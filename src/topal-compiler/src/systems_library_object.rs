//! LLVM production and inspection for the closed finite source-library profile.
use super::{
    EXPORT_SYMBOL, SystemsLibraryFunctionStack, SystemsSourceLibraryArtifact,
    SystemsSourceLibraryInput, check_source_library, invalid,
};
use crate::{CompileError, LlvmTools};
use ::object::{
    Object, ObjectKind, ObjectSection, ObjectSymbol, RelocationKind, RelocationTarget,
    SectionFlags, SymbolKind,
};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;
use topal_language::compiler::CompilerSystemsSourceLibraryRequirement;

// Worst-case downward alignment plus two private shim words. The shim erases
// its saved caller-SP slot before returning to the original kernel stack.
const NATIVE_SHIM_BYTES: u64 = 32;

pub(crate) struct CompiledSourceLibrary {
    pub(crate) llvm: String,
    pub(crate) object: Vec<u8>,
    pub(crate) source: String,
    pub(crate) record: SystemsSourceLibraryArtifact,
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn run(command: &mut Command) -> Result<String, CompileError> {
    let output = command
        .output()
        .map_err(|e| invalid(format!("cannot run {command:?}: {e}")))?;
    if !output.status.success() {
        return Err(invalid(format!(
            "{command:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    String::from_utf8(output.stdout).map_err(|e| invalid(e.to_string()))
}

pub(crate) fn build_source_library(
    requirement: &CompilerSystemsSourceLibraryRequirement,
    input: &SystemsSourceLibraryInput,
    tools: &LlvmTools,
    work: &Path,
) -> Result<CompiledSourceLibrary, CompileError> {
    let unit = check_source_library(requirement, input)?;
    let llvm = crate::codegen::emit_finite_kernel_library(&unit)?;
    let source = work.join("kernel-library.ll");
    let bitcode = work.join("kernel-library.bc");
    let object_path = work.join("kernel-library.o");
    fs::write(&source, &llvm).map_err(|e| invalid(e.to_string()))?;
    run(Command::new(tools.tool_path("llvm-as"))
        .arg(&source)
        .arg("-o")
        .arg(&bitcode))?;
    run(Command::new(tools.tool_path("opt"))
        .arg("-passes=verify")
        .arg("-disable-output")
        .arg(&bitcode))?;
    run(Command::new(tools.tool_path("llc"))
        .arg("-O0")
        .arg("-filetype=obj")
        .arg("-mtriple=x86_64-unknown-none")
        .arg("-mcpu=x86-64")
        .arg("-mattr=-mmx,-sse,-sse2,-avx,-avx2")
        .arg("-relocation-model=static")
        .arg("--frame-pointer=all")
        .arg("-stack-size-section")
        .arg(&bitcode)
        .arg("-o")
        .arg(&object_path))?;
    let bytes = fs::read(&object_path).map_err(|e| invalid(e.to_string()))?;
    let file = ::object::File::parse(bytes.as_slice()).map_err(|e| invalid(e.to_string()))?;
    if file.kind() != ObjectKind::Relocatable
        || file.architecture() != ::object::Architecture::X86_64
    {
        return Err(invalid(
            "source library must be an x86-64 relocatable object",
        ));
    }
    for section in file.sections() {
        if let SectionFlags::Elf { sh_flags, .. } = section.flags() {
            if section.size() > 0 && sh_flags.0 & 3 == 3 {
                return Err(invalid("writable allocated library data is forbidden"));
            }
            if sh_flags.0 & 0x400 != 0 {
                return Err(invalid("TLS library data is forbidden"));
            }
        }
    }
    let mut expected: BTreeSet<_> = unit.calls().keys().cloned().collect();
    expected.insert(EXPORT_SYMBOL.into());
    let mut found = BTreeSet::new();
    for symbol in file.symbols() {
        let name = symbol.name().map_err(|e| invalid(e.to_string()))?;
        if symbol.is_undefined() && !name.is_empty() {
            return Err(invalid(format!("undefined library symbol {name}")));
        }
        if symbol.kind() == SymbolKind::Text && symbol.is_definition() {
            if symbol.is_global() && name != EXPORT_SYMBOL {
                return Err(invalid("unexpected public library code symbol"));
            }
            found.insert(name.to_owned());
        }
    }
    if found != expected {
        return Err(invalid(
            "generated library code escaped its checked function closure",
        ));
    }
    let frames = stack_sizes(&file)?;
    if frames.keys().cloned().collect::<BTreeSet<_>>() != expected {
        return Err(invalid("missing or unexpected stack-size evidence"));
    }
    let mut costs = BTreeMap::new();
    let mut records = Vec::new();
    for function in &unit.program().functions {
        let callees = &unit.calls()[&function.symbol];
        let child = callees
            .iter()
            .map(|name| costs[name])
            .max()
            .unwrap_or(0_u64);
        let cost = frames[&function.symbol]
            .checked_add(8)
            .and_then(|v| v.checked_add(child))
            .ok_or_else(|| invalid("library stack bound overflow"))?;
        costs.insert(function.symbol.clone(), cost);
        records.push(SystemsLibraryFunctionStack {
            symbol: function.symbol.clone(),
            frame_bytes: frames[&function.symbol],
            maximum_call_path_bytes: cost,
            callees: callees.iter().cloned().collect(),
        });
    }
    let wrapper = frames[EXPORT_SYMBOL]
        .checked_add(8)
        .and_then(|v| v.checked_add(costs[unit.entry_symbol()]))
        .ok_or_else(|| invalid("adapter stack bound overflow"))?;
    let maximum = wrapper
        .checked_add(NATIVE_SHIM_BYTES)
        .ok_or_else(|| invalid("native shim stack bound overflow"))?;
    if maximum > requirement.stack_limit_bytes {
        return Err(invalid(format!(
            "library stack requires {maximum} bytes, limit {}",
            requirement.stack_limit_bytes
        )));
    }
    records.push(SystemsLibraryFunctionStack {
        symbol: EXPORT_SYMBOL.into(),
        frame_bytes: frames[EXPORT_SYMBOL],
        maximum_call_path_bytes: wrapper,
        callees: vec![unit.entry_symbol().into()],
    });
    let disassembly = run(Command::new(tools.tool_path("llvm-objdump"))
        .arg("-d")
        .arg("--no-show-raw-insn")
        .arg(&object_path))?;
    inspect_instructions(&disassembly)?;
    let record = SystemsSourceLibraryArtifact {
        schema: "topal.kernel-source-library/1".into(),
        interface: requirement.interface.as_str().into(),
        source: input.source.clone(),
        source_name: input.source_name.clone(),
        source_sha256: digest(input.text.as_bytes()),
        checked_client_sha256: digest(unit.program().source.as_str().as_bytes()),
        llvm_sha256: digest(llvm.as_bytes()),
        object_sha256: digest(&bytes),
        native_export: EXPORT_SYMBOL.into(),
        source_export: requirement.export.clone(),
        source_tags: *unit.source_tags(),
        llvm_version: tools.version.clone(),
        machine_profile: "x86_64-gpr-only-no-red-zone-acyclic/1".into(),
        disassembly_sha256: digest(disassembly.as_bytes()),
        native_alignment_reserve_bytes: NATIVE_SHIM_BYTES,
        maximum_stack_bytes: maximum,
        stack_limit_bytes: requirement.stack_limit_bytes,
        functions: records,
    };
    Ok(CompiledSourceLibrary {
        llvm,
        object: bytes,
        source: input.text.clone(),
        record,
    })
}

fn uleb(data: &[u8], cursor: &mut usize) -> Result<u64, CompileError> {
    let mut result = 0_u64;
    for shift in (0..=63).step_by(7) {
        let byte = *data
            .get(*cursor)
            .ok_or_else(|| invalid("truncated stack-size value"))?;
        *cursor += 1;
        let part = u64::from(byte & 127);
        if shift == 63 && part > 1 {
            return Err(invalid("overflowing stack-size value"));
        }
        result |= part << shift;
        if byte & 128 == 0 {
            return Ok(result);
        }
    }
    Err(invalid("overflowing stack-size encoding"))
}
fn stack_sizes(file: &::object::File<'_>) -> Result<BTreeMap<String, u64>, CompileError> {
    let mut result = BTreeMap::new();
    for section in file
        .sections()
        .filter(|section| section.name().ok() == Some(".stack_sizes"))
    {
        let data = section.data().map_err(|e| invalid(e.to_string()))?;
        let relocations: BTreeMap<_, _> = section.relocations().collect();
        let mut cursor = 0;
        while cursor < data.len() {
            let relocation = relocations
                .get(&(cursor as u64))
                .ok_or_else(|| invalid("stack-size record lacks a function relocation"))?;
            if relocation.kind() != RelocationKind::Absolute || relocation.size() != 64 {
                return Err(invalid("unqualified stack-size relocation"));
            }
            let (target_section, target_address) = match relocation.target() {
                RelocationTarget::Symbol(index) => {
                    let symbol = file
                        .symbol_by_index(index)
                        .map_err(|e| invalid(e.to_string()))?;
                    (
                        symbol
                            .section_index()
                            .ok_or_else(|| invalid("stack relocation has no owned section"))?,
                        symbol.address(),
                    )
                }
                RelocationTarget::Section(index) => (index, 0),
                _ => return Err(invalid("invalid stack-size relocation target")),
            };
            let address = target_address
                .checked_add_signed(relocation.addend())
                .ok_or_else(|| invalid("stack-size relocation overflow"))?;
            let candidates = file
                .symbols()
                .filter(|s| {
                    s.kind() == SymbolKind::Text
                        && s.is_definition()
                        && s.section_index() == Some(target_section)
                        && s.address() == address
                })
                .collect::<Vec<_>>();
            if candidates.len() != 1 {
                return Err(invalid("ambiguous stack-size function identity"));
            }
            let name = candidates[0]
                .name()
                .map_err(|e| invalid(e.to_string()))?
                .to_owned();
            cursor = cursor
                .checked_add(8)
                .filter(|end| *end <= data.len())
                .ok_or_else(|| invalid("truncated stack-size address"))?;
            let size = uleb(data, &mut cursor)?;
            if result.insert(name, size).is_some() {
                return Err(invalid("duplicate stack-size function evidence"));
            }
        }
    }
    Ok(result)
}

fn inspect_instructions(text: &str) -> Result<(), CompileError> {
    let forbidden = [
        "syscall", "sysenter", "int", "int3", "in", "inb", "inw", "inl", "out", "outb", "outw",
        "outl", "cli", "sti", "hlt", "swapgs", "wrmsr", "rdmsr", "rdtsc", "rdtscp", "rdpmc",
        "cpuid", "rdrand", "rdseed", "lgdt", "lidt", "iret", "iretq", "lret", "lretq", "ldmxcsr",
        "stmxcsr", "xsave", "xrstor", "fxsave", "fxrstor",
    ];
    let mut count = 0;
    for line in text.lines() {
        let Some((address, body)) = line.split_once(':') else {
            continue;
        };
        if address.trim().is_empty() || !address.trim().chars().all(|c| c.is_ascii_hexdigit()) {
            continue;
        }
        let Some(mnemonic) = body.split_whitespace().next() else {
            continue;
        };
        count += 1;
        if forbidden.contains(&mnemonic)
            || mnemonic.starts_with('f')
            || mnemonic.starts_with('v')
            || body.contains("%xmm")
            || body.contains("%ymm")
            || body.contains("%zmm")
            || body.contains("%mm")
            || body.contains("%st")
            || body.contains("%k0")
            || body.contains("%cr")
            || body.contains("%dr")
            || body.contains("%fs:")
            || body.contains("%gs:")
            || (mnemonic.starts_with("call") && body.contains('*'))
        {
            return Err(invalid(format!(
                "unqualified library instruction: {}",
                line.trim()
            )));
        }
    }
    if count == 0 {
        return Err(invalid("library disassembly contains no instructions"));
    }
    Ok(())
}

pub(crate) fn inspect_linked_library(
    bytes: &[u8],
    record: &SystemsSourceLibraryArtifact,
) -> Result<(), CompileError> {
    let file = ::object::File::parse(bytes).map_err(|e| invalid(e.to_string()))?;
    if record.schema != "topal.kernel-source-library/1"
        || record.interface != "boot-open-decision-v1"
        || record.native_export != EXPORT_SYMBOL
        || record.machine_profile != "x86_64-gpr-only-no-red-zone-acyclic/1"
        || record.stack_limit_bytes != 1024
        || record.maximum_stack_bytes > record.stack_limit_bytes
        || record.native_alignment_reserve_bytes != NATIVE_SHIM_BYTES
        || record.source_tags.iter().copied().collect::<BTreeSet<_>>()
            != BTreeSet::from([0, 1, 2, 3])
        || !record.functions.iter().any(|f| f.symbol == EXPORT_SYMBOL)
    {
        return Err(invalid("incomplete or unqualified linked library binding"));
    }
    for function in &record.functions {
        let symbol = file
            .symbol_by_name(&function.symbol)
            .ok_or_else(|| invalid("linked source-library function is missing"))?;
        let end = symbol
            .address()
            .checked_add(symbol.size())
            .ok_or_else(|| invalid("library code extent overflow"))?;
        if symbol.kind() != SymbolKind::Text
            || !symbol.is_definition()
            || symbol.size() == 0
            || symbol.address() < 0x200000
            || end > 0x40000000
            || (symbol.address() < 0x800000 && end > 0x400000)
        {
            return Err(invalid(
                "library code is not contained in the active supervisor identity mappings",
            ));
        }
        let section = file
            .section_by_index(
                symbol
                    .section_index()
                    .ok_or_else(|| invalid("library code has no section"))?,
            )
            .map_err(|e| invalid(e.to_string()))?;
        if let SectionFlags::Elf { sh_flags, .. } = section.flags() {
            if sh_flags.0 & 4 == 0 || sh_flags.0 & 1 != 0 {
                return Err(invalid(
                    "linked library code is not immutable executable text",
                ));
            }
        }
    }
    Ok(())
}
