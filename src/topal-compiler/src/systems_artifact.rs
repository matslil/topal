//! Linked and structurally inspected freestanding systems artifacts.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use object::write::{Object, Relocation, SectionId, Symbol, SymbolId, SymbolSection};
use object::{
    Architecture, BinaryFormat, Endianness, Object as _, ObjectSection as _, ObjectSegment as _,
    ObjectSymbol as _, RelocationEncoding, RelocationFlags, RelocationKind, SectionKind,
    SymbolFlags, SymbolKind, SymbolScope,
};
use serde::{Deserialize, Serialize};
use topal_language::compiler::{
    CompilerSystemsDisposition, CompilerSystemsOperation, CompilerSystemsProgram,
    CompilerSystemsTransition, SYSTEMS_BOOT_MEMORY_DESCRIBE, SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
    SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE, SYSTEMS_BOOTSTRAP_STORAGE_PROVISION,
    SYSTEMS_CONSOLE_WRITE, SYSTEMS_DEBUG_BREAK, SYSTEMS_FATAL, SYSTEMS_RESUME_DEBUG_BREAK,
    model_systems_transitions,
};

use crate::artifact::sha256;
use crate::{
    CompileError, DigestEntry, LlvmTools, X86_SYSTEMS_ALLOCATABLE_FLOOR,
    X86_SYSTEMS_BOOT_MEMORY_SYMBOL, X86_SYSTEMS_BOOTSTRAP_STORAGE_SECTION,
    X86_SYSTEMS_PLATFORM_ABI, X86_SYSTEMS_PROVIDER_OBJECT_REVISION, X86_SYSTEMS_PROVIDER_REVISION,
    X86_SYSTEMS_PROVIDER_TEXT_SECTION, generate_x86_64_systems_provider_object,
};

pub const X86_SYSTEMS_ARTIFACT_REVISION: &str = "topal.systems-artifact.x86_64-qemu-pc-q35/2";
pub const X86_SYSTEMS_ROOT_OBJECT_REVISION: &str = "topal.systems-root-object.x86_64/2";
pub const X86_SYSTEMS_ROOT_TEXT_SECTION: &str = ".text.topal.systems.root";
pub const X86_SYSTEMS_KERNEL_ENTRY: &str = "_topal_kernel_entry";
pub const X86_SYSTEMS_DEBUG_BREAK_ENTRY: &str = "topal_x86_systems_debug_break_entry";
pub const SYSTEMS_KERNEL_FILE: &str = "kernel.elf";
pub const SYSTEMS_DEBUG_FILE: &str = "kernel.debug";
pub const SYSTEMS_MAP_FILE: &str = "kernel.map";
pub const SYSTEMS_PROVENANCE_FILE: &str = "provenance.json";

static NEXT_STAGE: AtomicU64 = AtomicU64::new(0);

struct LinkedOutputs {
    kernel: Vec<u8>,
    debug: Vec<u8>,
    map: Vec<u8>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsArtifactProvenance {
    pub schema: String,
    pub provider: String,
    pub provider_object: String,
    pub root_object: String,
    pub target: String,
    pub board: String,
    pub profile: String,
    pub machine_cpu: String,
    pub codegen_cpu: String,
    pub platform_abi: String,
    pub data_layout: String,
    pub object_format: String,
    pub relocation_model: String,
    pub code_model: String,
    pub llvm_version: String,
    pub entry_symbol: String,
    pub exception_entries: Vec<String>,
    pub bootstrap_storage_capacity: u64,
    pub bootstrap_storage_alignment: u64,
    pub placements: Vec<SystemsArtifactPlacement>,
    pub semantic_trace: Vec<String>,
    pub inputs: Vec<DigestEntry>,
    pub outputs: Vec<DigestEntry>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SystemsArtifactPlacement {
    pub semantic_identity: String,
    pub symbol: String,
    pub section: String,
    pub address: u64,
    pub size: u64,
    pub alignment: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishedSystemsArtifact {
    pub directory: PathBuf,
    pub kernel: PathBuf,
    pub debug: PathBuf,
    pub map: PathBuf,
    pub provenance: PathBuf,
    pub record: SystemsArtifactProvenance,
}

/// Link, inspect, and atomically publish the initial freestanding systems ELF set.
///
/// The destination is a directory so the kernel, debug symbols, link map, and
/// canonical provenance become visible in one rename operation.
///
/// # Errors
///
/// Returns an error before publication when the checked program cannot be
/// lowered, the LLVM 22 linker/tool fails, structural inspection fails, or the
/// destination already exists.
pub fn publish_x86_64_systems_artifact(
    program: &CompilerSystemsProgram,
    tools: &LlvmTools,
    destination: &Path,
) -> Result<PublishedSystemsArtifact, CompileError> {
    let generated_provider = generate_x86_64_systems_provider_object(program)?;
    let root_object = generate_root_object(program)?;
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(io_error("create artifact parent", parent))?;
    if destination.exists() {
        return Err(CompileError::Io(format!(
            "systems artifact destination already exists: {}",
            destination.display()
        )));
    }
    let stage = allocate_stage(parent)?;
    let result = publish_in_stage(
        program,
        tools,
        &generated_provider.plan,
        &generated_provider.bytes,
        &root_object,
        &stage,
    );
    let record = match result {
        Ok(record) => record,
        Err(error) => {
            let _ = fs::remove_dir_all(&stage);
            return Err(error);
        }
    };
    if let Err(error) = fs::rename(&stage, destination) {
        let _ = fs::remove_dir_all(&stage);
        return Err(CompileError::Io(format!(
            "cannot atomically publish systems artifact {}: {error}",
            destination.display()
        )));
    }
    Ok(PublishedSystemsArtifact {
        directory: destination.to_owned(),
        kernel: destination.join(SYSTEMS_KERNEL_FILE),
        debug: destination.join(SYSTEMS_DEBUG_FILE),
        map: destination.join(SYSTEMS_MAP_FILE),
        provenance: destination.join(SYSTEMS_PROVENANCE_FILE),
        record,
    })
}

fn publish_in_stage(
    program: &CompilerSystemsProgram,
    tools: &LlvmTools,
    plan: &crate::X86SystemsProviderPlan,
    provider_object: &[u8],
    root_object: &[u8],
    stage: &Path,
) -> Result<SystemsArtifactProvenance, CompileError> {
    let work = stage.join("work");
    fs::create_dir(&work).map_err(io_error("create artifact work directory", &work))?;
    let provider_path = work.join("provider.o");
    let root_path = work.join("root.o");
    fs::write(&provider_path, provider_object)
        .map_err(io_error("write provider object", &provider_path))?;
    fs::write(&root_path, root_object).map_err(io_error("write root object", &root_path))?;

    let linked = build_linked_outputs(tools, &work, stage)?;
    inspect_linked_kernel(
        &linked.kernel,
        &linked.debug,
        &linked.map,
        plan.bootstrap_placement.capacity_bytes,
        program
            .bootstrap
            .handler
            .operations
            .iter()
            .any(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::BootstrapStoreByte { .. }
                        | CompilerSystemsOperation::BootstrapLoadByteEquals { .. }
                )
            }),
    )?;
    let placements = linked_placements(&linked.kernel)?;
    let record = artifact_provenance(
        program,
        tools,
        plan,
        provider_object,
        root_object,
        &linked.kernel,
        &linked.debug,
        &linked.map,
        placements,
    )?;
    let mut encoded = serde_json::to_vec_pretty(&record).map_err(|error| {
        CompileError::Tool(format!(
            "cannot encode systems artifact provenance: {error}"
        ))
    })?;
    encoded.push(b'\n');
    let provenance_path = stage.join(SYSTEMS_PROVENANCE_FILE);
    fs::write(&provenance_path, encoded).map_err(io_error(
        "write systems artifact provenance",
        &provenance_path,
    ))?;
    fs::remove_dir_all(&work).map_err(io_error("remove artifact work directory", &work))?;
    Ok(record)
}

fn build_linked_outputs(
    tools: &LlvmTools,
    work: &Path,
    stage: &Path,
) -> Result<LinkedOutputs, CompileError> {
    let unstripped_path = work.join("kernel.unstripped");
    let kernel_path = stage.join(SYSTEMS_KERNEL_FILE);
    let debug_path = stage.join(SYSTEMS_DEBUG_FILE);
    let map_path = stage.join(SYSTEMS_MAP_FILE);

    run_tool(
        Command::new(tools.tool_path("rust-lld"))
            .current_dir(work)
            .arg("-flavor")
            .arg("gnu")
            .arg("-m")
            .arg("elf_x86_64")
            .arg("-static")
            .arg("--no-dynamic-linker")
            .arg("--build-id=none")
            .arg("--entry")
            .arg(X86_SYSTEMS_KERNEL_ENTRY)
            .arg("-z")
            .arg("max-page-size=0x1000")
            .arg(format!("-Map=../{SYSTEMS_MAP_FILE}"))
            .arg("root.o")
            .arg("provider.o")
            .arg("-o")
            .arg("kernel.unstripped"),
    )?;

    let objcopy = tools.tool_path("llvm-objcopy");
    if !objcopy.is_file() {
        return Err(CompileError::Tool(format!(
            "LLVM tool `{}` is missing; systems artifact publication requires llvm-objcopy",
            objcopy.display()
        )));
    }
    run_tool(
        Command::new(&objcopy)
            .arg("--only-keep-debug")
            .arg(&unstripped_path)
            .arg(&debug_path),
    )?;
    run_tool(
        Command::new(&objcopy)
            .current_dir(stage)
            .arg("--strip-debug")
            .arg(format!("--add-gnu-debuglink={SYSTEMS_DEBUG_FILE}"))
            .arg(&unstripped_path)
            .arg(&kernel_path),
    )?;

    let kernel = fs::read(&kernel_path).map_err(io_error("read linked kernel", &kernel_path))?;
    let debug = fs::read(&debug_path).map_err(io_error("read debug artifact", &debug_path))?;
    let map = fs::read(&map_path).map_err(io_error("read link map", &map_path))?;
    Ok(LinkedOutputs { kernel, debug, map })
}

#[allow(clippy::too_many_arguments)] // Every independently hashed artifact input stays explicit.
fn artifact_provenance(
    program: &CompilerSystemsProgram,
    tools: &LlvmTools,
    plan: &crate::X86SystemsProviderPlan,
    provider_object: &[u8],
    root_object: &[u8],
    kernel: &[u8],
    debug: &[u8],
    map: &[u8],
    placements: Vec<SystemsArtifactPlacement>,
) -> Result<SystemsArtifactProvenance, CompileError> {
    let mut inputs = vec![
        DigestEntry {
            identity: X86_SYSTEMS_PROVIDER_OBJECT_REVISION.into(),
            sha256: sha256(provider_object),
        },
        DigestEntry {
            identity: X86_SYSTEMS_ROOT_OBJECT_REVISION.into(),
            sha256: sha256(root_object),
        },
    ];
    inputs.sort_by(|left, right| left.identity.cmp(&right.identity));
    let mut outputs = vec![
        DigestEntry {
            identity: SYSTEMS_DEBUG_FILE.into(),
            sha256: sha256(debug),
        },
        DigestEntry {
            identity: SYSTEMS_KERNEL_FILE.into(),
            sha256: sha256(kernel),
        },
        DigestEntry {
            identity: SYSTEMS_MAP_FILE.into(),
            sha256: sha256(map),
        },
    ];
    outputs.sort_by(|left, right| left.identity.cmp(&right.identity));
    Ok(SystemsArtifactProvenance {
        schema: X86_SYSTEMS_ARTIFACT_REVISION.into(),
        provider: X86_SYSTEMS_PROVIDER_REVISION.into(),
        provider_object: X86_SYSTEMS_PROVIDER_OBJECT_REVISION.into(),
        root_object: X86_SYSTEMS_ROOT_OBJECT_REVISION.into(),
        target: plan.target.into(),
        board: plan.board.into(),
        profile: plan.profile.into(),
        machine_cpu: plan.machine_cpu_model.into(),
        codegen_cpu: plan.codegen_cpu.into(),
        platform_abi: X86_SYSTEMS_PLATFORM_ABI.into(),
        data_layout: plan.data_layout.into(),
        object_format: plan.object_format.into(),
        relocation_model: plan.relocation_model.into(),
        code_model: plan.code_model.into(),
        llvm_version: tools.version.clone(),
        entry_symbol: X86_SYSTEMS_KERNEL_ENTRY.into(),
        exception_entries: vec![X86_SYSTEMS_DEBUG_BREAK_ENTRY.into()],
        bootstrap_storage_capacity: plan.bootstrap_placement.capacity_bytes,
        bootstrap_storage_alignment: plan.bootstrap_placement.alignment_bytes,
        placements,
        semantic_trace: semantic_trace(program)?,
        inputs,
        outputs,
    })
}

fn inspect_linked_kernel(
    kernel: &[u8],
    debug: &[u8],
    map: &[u8],
    storage_capacity: u64,
    requires_bootstrap_memory: bool,
) -> Result<(), CompileError> {
    let file = object::File::parse(kernel)
        .map_err(|error| CompileError::Tool(format!("cannot parse linked systems ELF: {error}")))?;
    inspect_kernel_elf(&file, storage_capacity, requires_bootstrap_memory)?;
    inspect_debug_and_map(debug, map)
}

fn inspect_kernel_elf(
    file: &object::File<'_>,
    storage_capacity: u64,
    requires_bootstrap_memory: bool,
) -> Result<(), CompileError> {
    if file.format() != BinaryFormat::Elf
        || file.architecture() != Architecture::X86_64
        || !file.is_little_endian()
        || file.kind() != object::ObjectKind::Executable
    {
        return Err(CompileError::Tool(
            "linked systems artifact is not a little-endian executable ELF64 x86-64 image".into(),
        ));
    }
    let entry = file
        .symbol_by_name(X86_SYSTEMS_KERNEL_ENTRY)
        .ok_or_else(|| {
            CompileError::Tool(format!(
                "linked systems artifact omits `{X86_SYSTEMS_KERNEL_ENTRY}`"
            ))
        })?;
    if entry.address() == 0 || entry.address() != file.entry() {
        return Err(CompileError::Tool(
            "linked systems ELF entry does not select its generated bootstrap root".into(),
        ));
    }
    inspect_bootstrap_reservation_floor(file)?;
    if requires_bootstrap_memory {
        inspect_bootstrap_memory_instructions(file)?;
    }
    if file
        .symbols()
        .filter(object::ObjectSymbol::is_undefined)
        .count()
        != 0
        || file
            .sections()
            .flat_map(|section| section.relocations())
            .count()
            != 0
    {
        return Err(CompileError::Tool(
            "linked systems ELF retains an undefined dependency or relocation".into(),
        ));
    }
    for forbidden in [
        ".interp",
        ".dynamic",
        ".dynsym",
        ".plt",
        ".got.plt",
        ".eh_frame",
        ".eh_frame_hdr",
        ".gcc_except_table",
    ] {
        if file.section_by_name(forbidden).is_some() {
            return Err(CompileError::Tool(format!(
                "linked systems ELF contains forbidden hosted section `{forbidden}`"
            )));
        }
    }
    for required in [
        X86_SYSTEMS_KERNEL_ENTRY,
        X86_SYSTEMS_DEBUG_BREAK_ENTRY,
        X86_SYSTEMS_BOOT_MEMORY_SYMBOL,
        "topal_x86_systems_uart16550_write",
        "topal_x86_systems_interrupt_return",
        "topal_x86_systems_fatal",
    ] {
        let symbol = file.symbol_by_name(required).ok_or_else(|| {
            CompileError::Tool(format!(
                "linked systems ELF omits generated symbol `{required}`"
            ))
        })?;
        if symbol.kind() != SymbolKind::Text || symbol.address() == 0 {
            return Err(CompileError::Tool(format!(
                "linked systems symbol `{required}` lacks executable placement"
            )));
        }
    }
    let storage = file
        .symbol_by_name("topal_bootstrap_storage")
        .ok_or_else(|| {
            CompileError::Tool("linked systems ELF omits generated bootstrap storage".into())
        })?;
    let storage_section = storage
        .section_index()
        .and_then(|index| file.section_by_index(index).ok())
        .ok_or_else(|| {
            CompileError::Tool("linked bootstrap storage has no allocated section".into())
        })?;
    if storage_section.kind() != SectionKind::UninitializedData
        || storage.size() != storage_capacity
        || storage.address() % 4096 != 0
    {
        return Err(CompileError::Tool(
            "linked systems ELF violates checked bootstrap-storage placement".into(),
        ));
    }
    if file.section_by_name(".gnu_debuglink").is_none() {
        return Err(CompileError::Tool(
            "linked systems ELF omits its canonical debug companion link".into(),
        ));
    }
    Ok(())
}

fn inspect_bootstrap_reservation_floor(file: &object::File<'_>) -> Result<(), CompileError> {
    for segment in file.segments() {
        let end = segment
            .address()
            .checked_add(segment.size())
            .ok_or_else(|| CompileError::Tool("linked systems segment range overflows".into()))?;
        if end > X86_SYSTEMS_ALLOCATABLE_FLOOR {
            return Err(CompileError::Tool(format!(
                "linked systems segment extends beyond reserved bootstrap floor {X86_SYSTEMS_ALLOCATABLE_FLOOR:#x}"
            )));
        }
    }
    Ok(())
}

fn inspect_bootstrap_memory_instructions(file: &object::File<'_>) -> Result<(), CompileError> {
    let entry = file
        .symbol_by_name(X86_SYSTEMS_KERNEL_ENTRY)
        .ok_or_else(|| CompileError::Tool("generated bootstrap root is absent".into()))?;
    let section = entry
        .section_index()
        .and_then(|index| file.section_by_index(index).ok())
        .ok_or_else(|| {
            CompileError::Tool("generated bootstrap root has no executable section".into())
        })?;
    let section_data = section.data().map_err(|error| {
        CompileError::Tool(format!(
            "cannot inspect generated bootstrap root bytes: {error}"
        ))
    })?;
    let start =
        usize::try_from(entry.address().saturating_sub(section.address())).map_err(|_| {
            CompileError::Tool("generated bootstrap root offset exceeds host range".into())
        })?;
    let size = usize::try_from(entry.size()).map_err(|_| {
        CompileError::Tool("generated bootstrap root size exceeds host range".into())
    })?;
    let end = start
        .checked_add(size)
        .ok_or_else(|| CompileError::Tool("generated bootstrap root range overflows".into()))?;
    let root = section_data.get(start..end).ok_or_else(|| {
        CompileError::Tool("generated bootstrap root range exceeds its section".into())
    })?;
    let retains_store = root
        .windows(7)
        .any(|bytes| bytes[..2] == [0xc6, 0x05] && bytes[6] == 90);
    let retains_load_and_check = root
        .windows(15)
        .any(|bytes| bytes[..3] == [0x0f, 0xb6, 0x05] && bytes[7..11] == [0x3c, 90, 0x0f, 0x85]);
    if !retains_store || !retains_load_and_check {
        return Err(CompileError::Tool(
            "linked bootstrap root omits its checked byte store/load path".into(),
        ));
    }
    Ok(())
}

fn inspect_debug_and_map(debug: &[u8], map: &[u8]) -> Result<(), CompileError> {
    let debug_file = object::File::parse(debug).map_err(|error| {
        CompileError::Tool(format!("cannot parse systems debug artifact: {error}"))
    })?;
    if debug_file.format() != BinaryFormat::Elf || debug_file.architecture() != Architecture::X86_64
    {
        return Err(CompileError::Tool(
            "systems debug artifact does not match the kernel ELF architecture".into(),
        ));
    }
    if debug_file
        .symbol_by_name(X86_SYSTEMS_KERNEL_ENTRY)
        .is_none()
    {
        return Err(CompileError::Tool(
            "systems debug artifact omits the generated bootstrap symbol".into(),
        ));
    }
    let map = std::str::from_utf8(map)
        .map_err(|error| CompileError::Tool(format!("systems link map is not UTF-8: {error}")))?;
    for symbol in [
        X86_SYSTEMS_KERNEL_ENTRY,
        X86_SYSTEMS_DEBUG_BREAK_ENTRY,
        X86_SYSTEMS_BOOT_MEMORY_SYMBOL,
        "topal_x86_systems_uart16550_write",
        "topal_bootstrap_storage",
    ] {
        if !map.contains(symbol) {
            return Err(CompileError::Tool(format!(
                "systems link map omits generated symbol `{symbol}`"
            )));
        }
    }
    for input_section in [
        X86_SYSTEMS_ROOT_TEXT_SECTION,
        X86_SYSTEMS_PROVIDER_TEXT_SECTION,
        X86_SYSTEMS_BOOTSTRAP_STORAGE_SECTION,
    ] {
        if !map.contains(input_section) {
            return Err(CompileError::Tool(format!(
                "systems link map omits semantic input section `{input_section}`"
            )));
        }
    }
    Ok(())
}

fn linked_placements(kernel: &[u8]) -> Result<Vec<SystemsArtifactPlacement>, CompileError> {
    let file = object::File::parse(kernel)
        .map_err(|error| CompileError::Tool(format!("cannot parse linked systems ELF: {error}")))?;
    let mut placements = Vec::new();
    for (semantic_identity, symbol_name) in [
        ("topal.systems.entry.bootstrap/1", X86_SYSTEMS_KERNEL_ENTRY),
        (
            "topal.systems.entry.synchronous.debug-break/1",
            X86_SYSTEMS_DEBUG_BREAK_ENTRY,
        ),
        (SYSTEMS_BOOT_MEMORY_DESCRIBE, X86_SYSTEMS_BOOT_MEMORY_SYMBOL),
        (SYSTEMS_CONSOLE_WRITE, "topal_x86_systems_uart16550_write"),
        (SYSTEMS_DEBUG_BREAK, "topal_x86_systems_debug_break"),
        (
            SYSTEMS_RESUME_DEBUG_BREAK,
            "topal_x86_systems_interrupt_return",
        ),
        (SYSTEMS_FATAL, "topal_x86_systems_fatal"),
        (
            SYSTEMS_BOOTSTRAP_STORAGE_PROVISION,
            "topal_bootstrap_storage",
        ),
        (
            SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE,
            "topal_bootstrap_storage",
        ),
        (
            SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
            "topal_bootstrap_storage",
        ),
    ] {
        let symbol = file.symbol_by_name(symbol_name).ok_or_else(|| {
            CompileError::Tool(format!(
                "linked systems ELF omits placement symbol `{symbol_name}`"
            ))
        })?;
        let section = symbol
            .section_index()
            .and_then(|index| file.section_by_index(index).ok())
            .ok_or_else(|| {
                CompileError::Tool(format!(
                    "linked systems placement symbol `{symbol_name}` has no section"
                ))
            })?;
        placements.push(SystemsArtifactPlacement {
            semantic_identity: semantic_identity.into(),
            symbol: symbol_name.into(),
            section: section.name().unwrap_or("<unnamed>").into(),
            address: symbol.address(),
            size: symbol.size(),
            alignment: section.align(),
        });
    }
    placements.sort_by(|left, right| {
        left.semantic_identity
            .cmp(&right.semantic_identity)
            .then_with(|| left.symbol.cmp(&right.symbol))
    });
    Ok(placements)
}

#[derive(Clone, Copy)]
enum ProviderSymbol {
    DescribeBootMemory,
    Uart16550Write,
    DebugBreak,
    InterruptReturn,
    Fatal,
    BootstrapStorage,
}

impl ProviderSymbol {
    const fn name(self) -> &'static str {
        match self {
            Self::DescribeBootMemory => X86_SYSTEMS_BOOT_MEMORY_SYMBOL,
            Self::Uart16550Write => "topal_x86_systems_uart16550_write",
            Self::DebugBreak => "topal_x86_systems_debug_break",
            Self::InterruptReturn => "topal_x86_systems_interrupt_return",
            Self::Fatal => "topal_x86_systems_fatal",
            Self::BootstrapStorage => "topal_bootstrap_storage",
        }
    }

    const fn kind(self) -> SymbolKind {
        match self {
            Self::BootstrapStorage => SymbolKind::Data,
            Self::DescribeBootMemory
            | Self::Uart16550Write
            | Self::DebugBreak
            | Self::InterruptReturn
            | Self::Fatal => SymbolKind::Text,
        }
    }
}

#[derive(Clone, Copy)]
enum SavedRegister {
    Rax,
    Rbx,
    Rcx,
    Rdx,
    Rbp,
    Rsi,
    Rdi,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}

const SAVED_REGISTERS: [SavedRegister; 15] = [
    SavedRegister::Rax,
    SavedRegister::Rbx,
    SavedRegister::Rcx,
    SavedRegister::Rdx,
    SavedRegister::Rbp,
    SavedRegister::Rsi,
    SavedRegister::Rdi,
    SavedRegister::R8,
    SavedRegister::R9,
    SavedRegister::R10,
    SavedRegister::R11,
    SavedRegister::R12,
    SavedRegister::R13,
    SavedRegister::R14,
    SavedRegister::R15,
];

struct PendingRelocation {
    offset: u64,
    target: ProviderSymbol,
    addend: i64,
    kind: RelocationKind,
    encoding: RelocationEncoding,
}

#[derive(Default)]
struct RootEncoder {
    bytes: Vec<u8>,
    relocations: Vec<PendingRelocation>,
    bootstrap_region_offset: Option<u64>,
}

#[derive(Clone, Copy)]
struct ProviderSymbols {
    describe_boot_memory: SymbolId,
    uart16550_write: SymbolId,
    debug_break: SymbolId,
    interrupt_return: SymbolId,
    fatal: SymbolId,
    bootstrap_storage: Option<SymbolId>,
}

impl RootEncoder {
    fn describe_boot_memory(&mut self) -> Result<(), CompileError> {
        self.call(ProviderSymbol::DescribeBootMemory);
        self.bytes.extend_from_slice(&[0x84, 0xc0, 0x0f, 0x84]);
        self.relocations.push(PendingRelocation {
            offset: u64::try_from(self.bytes.len()).map_err(|_| {
                CompileError::Tool("generated root relocation offset exceeds u64".into())
            })?,
            target: ProviderSymbol::Fatal,
            addend: -4,
            kind: RelocationKind::PltRelative,
            encoding: RelocationEncoding::X86Branch,
        });
        self.bytes.extend_from_slice(&[0; 4]);
        Ok(())
    }

    fn console_write(&mut self, text: &str) {
        for byte in text.as_bytes() {
            self.bytes.push(0xbf);
            self.bytes
                .extend_from_slice(&u32::from(*byte).to_le_bytes());
            self.call(ProviderSymbol::Uart16550Write);
        }
    }

    fn call(&mut self, target: ProviderSymbol) {
        self.bytes.push(0xe8);
        self.relocations.push(PendingRelocation {
            offset: self.bytes.len() as u64,
            target,
            addend: -4,
            kind: RelocationKind::PltRelative,
            encoding: RelocationEncoding::X86Branch,
        });
        self.bytes.extend_from_slice(&[0; 4]);
    }

    fn jump(&mut self, target: ProviderSymbol) {
        self.bytes.push(0xe9);
        self.relocations.push(PendingRelocation {
            offset: self.bytes.len() as u64,
            target,
            addend: -4,
            kind: RelocationKind::PltRelative,
            encoding: RelocationEncoding::X86Branch,
        });
        self.bytes.extend_from_slice(&[0; 4]);
    }

    fn push(&mut self, register: SavedRegister) {
        let (rex, opcode) = register_encoding(register, 0x50);
        if let Some(rex) = rex {
            self.bytes.push(rex);
        }
        self.bytes.push(opcode);
    }

    fn pop(&mut self, register: SavedRegister) {
        let (rex, opcode) = register_encoding(register, 0x58);
        if let Some(rex) = rex {
            self.bytes.push(rex);
        }
        self.bytes.push(opcode);
    }

    fn allocate_bootstrap_region(&mut self) -> Result<(), CompileError> {
        if self.bootstrap_region_offset.replace(0).is_some() {
            return Err(CompileError::Tool(
                "x86 root lowering encountered overlapping bootstrap regions".into(),
            ));
        }
        Ok(())
    }

    fn store_bootstrap_byte(&mut self, offset: u64, value: u8) -> Result<(), CompileError> {
        let storage_offset = self.bootstrap_storage_offset(offset)?;
        self.bytes.extend_from_slice(&[0xc6, 0x05]);
        self.rip_relative_storage(storage_offset, 1)?;
        self.bytes.push(value);
        Ok(())
    }

    fn load_bootstrap_byte_equals(
        &mut self,
        offset: u64,
        expected: u8,
    ) -> Result<(), CompileError> {
        let storage_offset = self.bootstrap_storage_offset(offset)?;
        self.bytes.extend_from_slice(&[0x0f, 0xb6, 0x05]);
        self.rip_relative_storage(storage_offset, 0)?;
        self.bytes.extend_from_slice(&[0x3c, expected, 0x0f, 0x85]);
        self.relocations.push(PendingRelocation {
            offset: u64::try_from(self.bytes.len()).map_err(|_| {
                CompileError::Tool("generated root relocation offset exceeds u64".into())
            })?,
            target: ProviderSymbol::Fatal,
            addend: -4,
            kind: RelocationKind::PltRelative,
            encoding: RelocationEncoding::X86Branch,
        });
        self.bytes.extend_from_slice(&[0; 4]);
        Ok(())
    }

    fn release_bootstrap_region(&mut self) -> Result<(), CompileError> {
        if self.bootstrap_region_offset.take().is_none() {
            return Err(CompileError::Tool(
                "x86 root lowering encountered release without a live bootstrap region".into(),
            ));
        }
        Ok(())
    }

    fn bootstrap_storage_offset(&self, offset: u64) -> Result<u64, CompileError> {
        self.bootstrap_region_offset
            .ok_or_else(|| {
                CompileError::Tool(
                    "x86 root lowering encountered byte access without a live region".into(),
                )
            })?
            .checked_add(offset)
            .ok_or_else(|| CompileError::Tool("bootstrap storage offset overflows".into()))
    }

    fn rip_relative_storage(
        &mut self,
        storage_offset: u64,
        trailing_bytes: i64,
    ) -> Result<(), CompileError> {
        let storage_offset = i64::try_from(storage_offset).map_err(|_| {
            CompileError::Tool("bootstrap storage offset exceeds x86 relocation range".into())
        })?;
        self.relocations.push(PendingRelocation {
            offset: u64::try_from(self.bytes.len()).map_err(|_| {
                CompileError::Tool("generated root relocation offset exceeds u64".into())
            })?,
            target: ProviderSymbol::BootstrapStorage,
            addend: storage_offset - 4 - trailing_bytes,
            kind: RelocationKind::Relative,
            encoding: RelocationEncoding::X86RipRelative,
        });
        self.bytes.extend_from_slice(&[0; 4]);
        Ok(())
    }
}

const fn register_encoding(register: SavedRegister, base: u8) -> (Option<u8>, u8) {
    let index = match register {
        SavedRegister::Rax => 0,
        SavedRegister::Rcx => 1,
        SavedRegister::Rdx => 2,
        SavedRegister::Rbx => 3,
        SavedRegister::Rbp => 5,
        SavedRegister::Rsi => 6,
        SavedRegister::Rdi => 7,
        SavedRegister::R8 => 8,
        SavedRegister::R9 => 9,
        SavedRegister::R10 => 10,
        SavedRegister::R11 => 11,
        SavedRegister::R12 => 12,
        SavedRegister::R13 => 13,
        SavedRegister::R14 => 14,
        SavedRegister::R15 => 15,
    };
    if index < 8 {
        (None, base + index)
    } else {
        (Some(0x41), base + index - 8)
    }
}

fn generate_root_object(program: &CompilerSystemsProgram) -> Result<Vec<u8>, CompileError> {
    generate_x86_64_systems_provider_object(program)?;
    let mut object = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
    object.add_file_symbol(b"topal-generated-x86-systems-root".to_vec());
    let text = object.add_section(
        Vec::new(),
        X86_SYSTEMS_ROOT_TEXT_SECTION.as_bytes().to_vec(),
        SectionKind::Text,
    );
    let uses_bootstrap_storage = program
        .bootstrap
        .handler
        .operations
        .iter()
        .any(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::BootstrapAllocate { .. }
                    | CompilerSystemsOperation::BootstrapStoreByte { .. }
                    | CompilerSystemsOperation::BootstrapLoadByteEquals { .. }
                    | CompilerSystemsOperation::BootstrapRelease
            )
        });
    let symbols = ProviderSymbols {
        describe_boot_memory: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::DescribeBootMemory,
        ),
        uart16550_write: undefined_provider_symbol(&mut object, ProviderSymbol::Uart16550Write),
        debug_break: undefined_provider_symbol(&mut object, ProviderSymbol::DebugBreak),
        interrupt_return: undefined_provider_symbol(&mut object, ProviderSymbol::InterruptReturn),
        fatal: undefined_provider_symbol(&mut object, ProviderSymbol::Fatal),
        bootstrap_storage: uses_bootstrap_storage
            .then(|| undefined_provider_symbol(&mut object, ProviderSymbol::BootstrapStorage)),
    };

    let mut bootstrap = RootEncoder::default();
    encode_operations(&mut bootstrap, &program.bootstrap.handler.operations)?;
    bootstrap.jump(ProviderSymbol::Fatal);
    append_root(
        &mut object,
        text,
        X86_SYSTEMS_KERNEL_ENTRY,
        &bootstrap,
        symbols,
    )?;

    let mut debug_break = RootEncoder::default();
    for register in SAVED_REGISTERS {
        debug_break.push(register);
    }
    encode_operations(&mut debug_break, &program.debug_break.handler.operations)?;
    match program.debug_break.handler.disposition {
        CompilerSystemsDisposition::Resume => {
            for register in SAVED_REGISTERS.into_iter().rev() {
                debug_break.pop(register);
            }
            debug_break.jump(ProviderSymbol::InterruptReturn);
        }
        CompilerSystemsDisposition::Fatal { .. } => debug_break.jump(ProviderSymbol::Fatal),
    }
    append_root(
        &mut object,
        text,
        X86_SYSTEMS_DEBUG_BREAK_ENTRY,
        &debug_break,
        symbols,
    )?;
    object
        .write()
        .map_err(|error| CompileError::Tool(format!("cannot encode systems root ELF: {error}")))
}

fn encode_operations(
    encoder: &mut RootEncoder,
    operations: &[CompilerSystemsOperation],
) -> Result<(), CompileError> {
    for operation in operations {
        match operation {
            CompilerSystemsOperation::DescribeBootMemory { .. } => {
                encoder.describe_boot_memory()?;
            }
            CompilerSystemsOperation::ConsoleWrite { text } => encoder.console_write(text),
            CompilerSystemsOperation::DebugBreak => encoder.call(ProviderSymbol::DebugBreak),
            CompilerSystemsOperation::BootstrapAllocate { .. } => {
                encoder.allocate_bootstrap_region()?;
            }
            CompilerSystemsOperation::BootstrapStoreByte {
                offset_bytes,
                value,
            } => encoder.store_bootstrap_byte(*offset_bytes, *value)?,
            CompilerSystemsOperation::BootstrapLoadByteEquals {
                offset_bytes,
                expected,
                ..
            } => encoder.load_bootstrap_byte_equals(*offset_bytes, *expected)?,
            CompilerSystemsOperation::BootstrapRelease => encoder.release_bootstrap_region()?,
        }
    }
    if encoder.bootstrap_region_offset.is_some() {
        return Err(CompileError::Tool(
            "x86 root lowering ended with a live bootstrap region".into(),
        ));
    }
    Ok(())
}

fn undefined_provider_symbol(object: &mut Object<'_>, symbol: ProviderSymbol) -> SymbolId {
    object.add_symbol(Symbol {
        name: symbol.name().as_bytes().to_vec(),
        value: 0,
        size: 0,
        kind: symbol.kind(),
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Undefined,
        flags: SymbolFlags::None,
    })
}

fn append_root(
    object: &mut Object<'_>,
    section: SectionId,
    name: &str,
    encoded: &RootEncoder,
    provider_symbols: ProviderSymbols,
) -> Result<(), CompileError> {
    let start = object.append_section_data(section, &encoded.bytes, 16);
    object.add_symbol(Symbol {
        name: name.as_bytes().to_vec(),
        value: start,
        size: encoded.bytes.len() as u64,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(section),
        flags: SymbolFlags::None,
    });
    for relocation in &encoded.relocations {
        let symbol = match relocation.target {
            ProviderSymbol::DescribeBootMemory => provider_symbols.describe_boot_memory,
            ProviderSymbol::Uart16550Write => provider_symbols.uart16550_write,
            ProviderSymbol::DebugBreak => provider_symbols.debug_break,
            ProviderSymbol::InterruptReturn => provider_symbols.interrupt_return,
            ProviderSymbol::Fatal => provider_symbols.fatal,
            ProviderSymbol::BootstrapStorage => {
                provider_symbols.bootstrap_storage.ok_or_else(|| {
                    CompileError::Tool(
                        "x86 root lowering emitted a storage relocation without storage use".into(),
                    )
                })?
            }
        };
        object
            .add_relocation(
                section,
                Relocation {
                    offset: start + relocation.offset,
                    symbol,
                    addend: relocation.addend,
                    flags: RelocationFlags::Generic {
                        kind: relocation.kind,
                        encoding: relocation.encoding,
                        size: 32,
                    },
                },
            )
            .map_err(|error| {
                CompileError::Tool(format!("cannot encode systems root relocation: {error}"))
            })?;
    }
    Ok(())
}

fn semantic_trace(program: &CompilerSystemsProgram) -> Result<Vec<String>, CompileError> {
    let transitions = model_systems_transitions(program)
        .map_err(|error| CompileError::Tool(format!("invalid systems transition model: {error}")))?
        .into_iter()
        .map(|transition| {
            let identity = transition.semantic_identity();
            match transition {
                CompilerSystemsTransition::ProvisionBootstrapStorage {
                    capacity_bytes,
                    alignment_bytes,
                } => format!("{identity}:capacity={capacity_bytes}:alignment={alignment_bytes}"),
                CompilerSystemsTransition::ConsoleWrite { text } => {
                    format!("{identity}:sha256:{}", sha256(text.as_bytes()))
                }
                CompilerSystemsTransition::Fatal { message } => {
                    format!("{identity}:sha256:{}", sha256(message.as_bytes()))
                }
                CompilerSystemsTransition::AllocateBootstrapRegion { request } => format!(
                    "{identity}:bytes={}:alignment={}:placement={}",
                    request.byte_count,
                    request.alignment_bytes,
                    request.placement.semantic_identity()
                ),
                CompilerSystemsTransition::StoreBootstrapByte {
                    offset_bytes,
                    value,
                }
                | CompilerSystemsTransition::LoadBootstrapByte {
                    offset_bytes,
                    value,
                } => format!("{identity}:offset={offset_bytes}:value={value}"),
                CompilerSystemsTransition::EnterBootstrap
                | CompilerSystemsTransition::DescribeBootMemory
                | CompilerSystemsTransition::ObserveDebugBreak
                | CompilerSystemsTransition::EnterDebugBreak
                | CompilerSystemsTransition::ResumeDebugBreak
                | CompilerSystemsTransition::ReleaseBootstrapRegion => identity.into(),
            }
        })
        .collect();
    Ok(transitions)
}

fn allocate_stage(parent: &Path) -> Result<PathBuf, CompileError> {
    for _ in 0..100 {
        let sequence = NEXT_STAGE.fetch_add(1, Ordering::Relaxed);
        let stage = parent.join(format!(
            ".topal-systems-artifact-{}-{sequence}",
            std::process::id()
        ));
        match fs::create_dir(&stage) {
            Ok(()) => return Ok(stage),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(io_error("create systems artifact stage", &stage)(error)),
        }
    }
    Err(CompileError::Io(
        "cannot allocate a unique systems artifact stage".into(),
    ))
}

fn run_tool(command: &mut Command) -> Result<(), CompileError> {
    let display = format!("{command:?}");
    let output = command.output().map_err(|error| {
        CompileError::Tool(format!(
            "cannot execute systems artifact tool {display}: {error}"
        ))
    })?;
    if output.status.success() {
        Ok(())
    } else {
        Err(CompileError::Tool(format!(
            "systems artifact tool failed ({display}):\n{}",
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

fn io_error<'a>(
    action: &'static str,
    path: &'a Path,
) -> impl FnOnce(std::io::Error) -> CompileError + 'a {
    move |error| CompileError::Io(format!("cannot {action} {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use object::{Object as _, ObjectSection as _, ObjectSymbol as _};
    use topal_language::compiler::{
        CompilerSystemsTargetSelection, SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
        SYSTEMS_BOOTSTRAP_STORAGE_RELEASE, analyze_systems_for_compiler,
    };

    use super::*;

    const SOURCE: &str = include_str!("../../../linux-kernel/kernel/arch/x86_64/toolchain-gate.t");

    fn program() -> CompilerSystemsProgram {
        analyze_systems_for_compiler(
            SOURCE,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap()
    }

    #[test]
    fn generated_root_has_only_typed_provider_dependencies() {
        // TOPAL-COMP-SYSTEMS-X64-001, TOPAL-SYSTEMS-MACHINE-001.
        let root = generate_root_object(&program()).unwrap();
        let file = object::File::parse(root.as_slice()).unwrap();
        let undefined = file
            .symbols()
            .filter(object::ObjectSymbol::is_undefined)
            .map(|symbol| symbol.name().unwrap().to_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            undefined,
            [
                "topal_x86_systems_describe_boot_memory",
                "topal_x86_systems_uart16550_write",
                "topal_x86_systems_debug_break",
                "topal_x86_systems_interrupt_return",
                "topal_x86_systems_fatal",
                "topal_bootstrap_storage",
            ]
        );
        let text = file.section_by_name(X86_SYSTEMS_ROOT_TEXT_SECTION).unwrap();
        let relocation_targets = text
            .relocations()
            .map(|(_, relocation)| {
                let object::RelocationTarget::Symbol(symbol) = relocation.target() else {
                    panic!("root relocations target only provider symbols");
                };
                file.symbol_by_index(symbol).unwrap().name().unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(relocation_targets.len(), 102);
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_BOOT_MEMORY_SYMBOL)
                .count(),
            1
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == "topal_x86_systems_uart16550_write")
                .count(),
            94
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == "topal_bootstrap_storage")
                .count(),
            2
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == "topal_x86_systems_fatal")
                .count(),
            3
        );
        let root = file.symbol_by_name(X86_SYSTEMS_KERNEL_ENTRY).unwrap();
        let root_section = file
            .section_by_index(root.section_index().unwrap())
            .unwrap();
        let data = root_section.data().unwrap();
        let start = usize::try_from(root.address() - root_section.address()).unwrap();
        let end = start + usize::try_from(root.size()).unwrap();
        let root_bytes = &data[start..end];
        assert!(
            root_bytes
                .windows(7)
                .any(|bytes| bytes[..2] == [0xc6, 0x05] && bytes[6] == 90),
            "root must retain the real byte store"
        );
        assert!(
            root_bytes.windows(15).any(|bytes| {
                bytes[..3] == [0x0f, 0xb6, 0x05] && bytes[7..11] == [0x3c, 90, 0x0f, 0x85]
            }),
            "root must retain the real byte load and mismatch branch"
        );
        assert!(file.symbol_by_name(X86_SYSTEMS_KERNEL_ENTRY).is_some());
        assert!(file.symbol_by_name(X86_SYSTEMS_DEBUG_BREAK_ENTRY).is_some());
    }

    #[test]
    fn publishes_one_closed_freestanding_artifact_directory() {
        // TOPAL-COMP-SYSTEMS-ARTIFACT-001, TOPAL-COMP-SYSTEMS-TEST-001.
        let tools = LlvmTools::discover(None).unwrap();
        let parent = std::env::temp_dir().join(format!(
            "topal-systems-artifact-test-{}-{}",
            std::process::id(),
            NEXT_STAGE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&parent).unwrap();
        let destination = parent.join("published");
        let published = publish_x86_64_systems_artifact(&program(), &tools, &destination).unwrap();
        assert_eq!(
            fs::read_dir(&destination).unwrap().count(),
            4,
            "only canonical outputs are published"
        );
        assert!(published.kernel.is_file());
        assert!(published.debug.is_file());
        assert!(published.map.is_file());
        assert!(published.provenance.is_file());
        let encoded = fs::read(&published.provenance).unwrap();
        let decoded: SystemsArtifactProvenance = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded, published.record);
        assert_eq!(decoded.schema, X86_SYSTEMS_ARTIFACT_REVISION);
        assert_eq!(decoded.target, "x86_64-unknown-none");
        assert_eq!(decoded.outputs.len(), 3);
        assert_eq!(decoded.placements.len(), 10);
        assert_eq!(decoded.bootstrap_storage_capacity, 65_536);
        assert_eq!(decoded.bootstrap_storage_alignment, 4096);
        assert_eq!(decoded.semantic_trace.len(), 15);
        assert!(decoded.semantic_trace[0].starts_with(SYSTEMS_BOOTSTRAP_STORAGE_PROVISION));
        assert_eq!(decoded.semantic_trace[1], "topal.systems.entry.bootstrap/1");
        assert_eq!(decoded.semantic_trace[2], SYSTEMS_BOOT_MEMORY_DESCRIBE);
        assert!(decoded.semantic_trace[3].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[4].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(decoded.semantic_trace[5], SYSTEMS_DEBUG_BREAK);
        assert_eq!(
            decoded.semantic_trace[6],
            "topal.systems.entry.synchronous.debug-break/1"
        );
        assert_eq!(decoded.semantic_trace[7], SYSTEMS_RESUME_DEBUG_BREAK);
        assert!(decoded.semantic_trace[8].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[9].starts_with(SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE));
        assert!(decoded.semantic_trace[10].starts_with(SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE));
        assert!(decoded.semantic_trace[11].starts_with(SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE));
        assert!(decoded.semantic_trace[12].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[13],
            SYSTEMS_BOOTSTRAP_STORAGE_RELEASE
        );
        assert!(decoded.semantic_trace[14].starts_with(SYSTEMS_FATAL));
        let repeated_destination = parent.join("repeated");
        let repeated =
            publish_x86_64_systems_artifact(&program(), &tools, &repeated_destination).unwrap();
        for (first, second) in [
            (&published.kernel, &repeated.kernel),
            (&published.debug, &repeated.debug),
            (&published.map, &repeated.map),
            (&published.provenance, &repeated.provenance),
        ] {
            assert_eq!(fs::read(first).unwrap(), fs::read(second).unwrap());
        }
        fs::remove_dir_all(parent).unwrap();
    }
}
