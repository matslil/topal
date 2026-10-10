//! Linux x86 boot-protocol adapter for the initial systems artifact.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use object::{
    Architecture, BinaryFormat, Object as _, ObjectSegment as _, ObjectSymbol as _, SymbolKind,
};
use serde::{Deserialize, Serialize};

use crate::artifact::sha256;
use crate::{
    CompileError, SYSTEMS_KERNEL_FILE, SystemsArtifactProvenance, X86_SYSTEMS_ARTIFACT_REVISION,
    X86_SYSTEMS_DEBUG_BREAK_ENTRY, X86_SYSTEMS_KERNEL_ENTRY, X86_SYSTEMS_LOCAL_NOTIFICATION_ENTRY,
    X86_SYSTEMS_PROVIDER_REVISION,
};

pub const X86_LINUX_BOOT_ADAPTER_REVISION: &str =
    "topal.boot-adapter.linux-x86-protocol-2.15-q35/4";
pub const X86_LINUX_BOOT_PROTOCOL: u16 = 0x020f;
pub const X86_LINUX_SETUP_SECTORS: u8 = 4;
pub const X86_PROTECTED_PAYLOAD_ADDRESS: u64 = 0x0010_0000;
pub const X86_KERNEL_MINIMUM_ADDRESS: u64 = 0x0020_0000;
pub const X86_INITIAL_STACK_TOP: u64 = 0x0018_0000;
pub const X86_INITIAL_IDENTITY_LIMIT: u64 = 0x4000_0000;
pub const X86_BOOT_IMAGE_FILE: &str = "bzImage";
pub const X86_BOOT_PROVENANCE_FILE: &str = "boot-provenance.json";

const SETUP_BYTES: usize = 5 * 512;
const REAL_MODE_ENTRY_OFFSET: usize = 0x26c;
const REAL_MODE_CONTINUATION_SOURCE_OFFSET: usize = 0x300;
const REAL_MODE_CONTINUATION_RELOCATION_OFFSET: u16 = 0x7000;
const BOOT_PARAMS_E820_COUNT_OFFSET: u16 = 0x01e8;
const BOOT_PARAMS_E820_TABLE_OFFSET: u16 = 0x02d0;
const BOOT_PARAMS_E820_ENTRY_BYTES: u16 = 20;
const BOOT_PARAMS_E820_MAX_ENTRIES: u8 = 128;
const PAGE_TABLE_PML4_ADDRESS: u64 = 0x0010_1000;
const PAGE_TABLE_PDPT_ADDRESS: u64 = 0x0010_2000;
const PAGE_TABLE_PD_ADDRESS: u64 = 0x0010_3000;
const GDT_ADDRESS: u64 = 0x0010_4000;
const IDT_ADDRESS: u64 = 0x0010_5000;
const IDT_DESCRIPTOR_ADDRESS: u64 = 0x0010_6000;
const LOCAL_APIC_PD_ADDRESS: u64 = 0x0010_7000;
const TRANSITION_RESERVED_END: u64 = 0x0010_8000;
const HPET_MMIO_LEAF_ADDRESS: u64 = 0xfec0_0000;
const LOCAL_APIC_PAGE_ADDRESS: u64 = 0xfee0_0000;
const LOCAL_NOTIFICATION_VECTOR: usize = 0xf1;
const MAX_PROTECTED_PAYLOAD_BYTES: u64 = 64 * 1024 * 1024;

static NEXT_BOOT_STAGE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct X86LinuxBootImageProvenance {
    pub schema: String,
    pub protocol: u16,
    pub setup_sectors: u8,
    pub protected_payload_address: u64,
    pub kernel_minimum_address: u64,
    pub initial_stack_top: u64,
    pub identity_map_limit: u64,
    pub local_apic_page_address: u64,
    pub target: String,
    pub board: String,
    pub machine_cpu: String,
    pub provider: String,
    pub linked_kernel_sha256: String,
    pub boot_image_sha256: String,
    pub protected_payload_bytes: u64,
    pub kernel_entry: u64,
    pub debug_break_entry: u64,
    pub local_notification_entry: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedX86LinuxBootImage {
    pub bytes: Vec<u8>,
    pub provenance: X86LinuxBootImageProvenance,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishedX86LinuxBootImage {
    pub directory: PathBuf,
    pub image: PathBuf,
    pub provenance: PathBuf,
    pub record: X86LinuxBootImageProvenance,
}

struct KernelLoadSegment<'data> {
    address: u64,
    size: u64,
    executable: bool,
    data: &'data [u8],
}

/// Package a validated linked systems ELF as the approved Linux x86 `bzImage`.
///
/// # Errors
///
/// Returns an error when provenance does not match the kernel, ELF placement is
/// outside the approved initial layout, required entries are absent, or an
/// encoded boot-protocol field would overflow.
pub fn generate_x86_64_linux_boot_image(
    kernel: &[u8],
    artifact: &SystemsArtifactProvenance,
) -> Result<GeneratedX86LinuxBootImage, CompileError> {
    validate_artifact_input(kernel, artifact)?;
    let file = object::File::parse(kernel)
        .map_err(|error| CompileError::Tool(format!("cannot parse linked kernel ELF: {error}")))?;
    if file.format() != BinaryFormat::Elf
        || file.architecture() != Architecture::X86_64
        || !file.is_little_endian()
        || file.kind() != object::ObjectKind::Executable
    {
        return Err(CompileError::Tool(
            "boot adapter requires an executable little-endian ELF64 x86-64 kernel".into(),
        ));
    }
    let segments = collect_load_segments(&file)?;
    let kernel_entry = required_executable_symbol(&file, &segments, X86_SYSTEMS_KERNEL_ENTRY)?;
    if kernel_entry != file.entry() {
        return Err(CompileError::Tool(
            "kernel ELF entry does not select the generated bootstrap symbol".into(),
        ));
    }
    let debug_break_entry =
        required_executable_symbol(&file, &segments, X86_SYSTEMS_DEBUG_BREAK_ENTRY)?;
    let local_notification_entry =
        required_executable_symbol(&file, &segments, X86_SYSTEMS_LOCAL_NOTIFICATION_ENTRY)?;
    let mut protected = materialize_protected_payload(&segments)?;
    install_transition_support(
        &mut protected,
        kernel_entry,
        debug_break_entry,
        local_notification_entry,
    )?;
    pad_to(&mut protected, 16);

    let protected_size = u32::try_from(protected.len()).map_err(|_| {
        CompileError::Tool("protected payload exceeds the Linux boot header limit".into())
    })?;
    let syssize = protected_size.div_ceil(16);
    let setup = build_setup_header(syssize, protected_size)?;
    let mut bytes = Vec::with_capacity(setup.len() + protected.len());
    bytes.extend_from_slice(&setup);
    bytes.extend_from_slice(&protected);
    let linked_kernel_sha256 = sha256(kernel);
    let boot_image_sha256 = sha256(&bytes);
    Ok(GeneratedX86LinuxBootImage {
        bytes,
        provenance: X86LinuxBootImageProvenance {
            schema: X86_LINUX_BOOT_ADAPTER_REVISION.into(),
            protocol: X86_LINUX_BOOT_PROTOCOL,
            setup_sectors: X86_LINUX_SETUP_SECTORS,
            protected_payload_address: X86_PROTECTED_PAYLOAD_ADDRESS,
            kernel_minimum_address: X86_KERNEL_MINIMUM_ADDRESS,
            initial_stack_top: X86_INITIAL_STACK_TOP,
            identity_map_limit: X86_INITIAL_IDENTITY_LIMIT,
            local_apic_page_address: LOCAL_APIC_PAGE_ADDRESS,
            target: artifact.target.clone(),
            board: artifact.board.clone(),
            machine_cpu: artifact.machine_cpu.clone(),
            provider: artifact.provider.clone(),
            linked_kernel_sha256,
            boot_image_sha256,
            protected_payload_bytes: u64::from(protected_size),
            kernel_entry,
            debug_break_entry,
            local_notification_entry,
        },
    })
}

/// Atomically publish the generated boot image and its canonical provenance.
///
/// # Errors
///
/// Returns an error without publishing the destination when generation,
/// staging, provenance encoding, or the final directory rename fails.
pub fn publish_x86_64_linux_boot_image(
    kernel: &[u8],
    artifact: &SystemsArtifactProvenance,
    destination: &Path,
) -> Result<PublishedX86LinuxBootImage, CompileError> {
    let generated = generate_x86_64_linux_boot_image(kernel, artifact)?;
    let parent = destination.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(io_error("create boot-image parent", parent))?;
    if destination.exists() {
        return Err(CompileError::Io(format!(
            "boot-image destination already exists: {}",
            destination.display()
        )));
    }
    let stage = allocate_stage(parent)?;
    let result = (|| {
        let image = stage.join(X86_BOOT_IMAGE_FILE);
        fs::write(&image, &generated.bytes).map_err(io_error("write staged boot image", &image))?;
        let mut encoded = serde_json::to_vec_pretty(&generated.provenance).map_err(|error| {
            CompileError::Tool(format!("cannot encode boot-image provenance: {error}"))
        })?;
        encoded.push(b'\n');
        let provenance = stage.join(X86_BOOT_PROVENANCE_FILE);
        fs::write(&provenance, encoded)
            .map_err(io_error("write staged boot provenance", &provenance))?;
        Ok::<(), CompileError>(())
    })();
    if let Err(error) = result {
        let _ = fs::remove_dir_all(&stage);
        return Err(error);
    }
    if let Err(error) = fs::rename(&stage, destination) {
        let _ = fs::remove_dir_all(&stage);
        return Err(CompileError::Io(format!(
            "cannot atomically publish x86 boot image {}: {error}",
            destination.display()
        )));
    }
    Ok(PublishedX86LinuxBootImage {
        directory: destination.to_owned(),
        image: destination.join(X86_BOOT_IMAGE_FILE),
        provenance: destination.join(X86_BOOT_PROVENANCE_FILE),
        record: generated.provenance,
    })
}

fn validate_artifact_input(
    kernel: &[u8],
    artifact: &SystemsArtifactProvenance,
) -> Result<(), CompileError> {
    if artifact.schema != X86_SYSTEMS_ARTIFACT_REVISION
        || artifact.target != "x86_64-unknown-none"
        || artifact.board != "topal-qemu-pc-q35-10.2"
        || artifact.machine_cpu != "qemu64-v1"
        || artifact.provider != X86_SYSTEMS_PROVIDER_REVISION
    {
        return Err(CompileError::Tool(
            "boot adapter input provenance is outside the approved x86 QEMU profile".into(),
        ));
    }
    let expected = artifact
        .outputs
        .iter()
        .find(|output| output.identity == SYSTEMS_KERNEL_FILE)
        .ok_or_else(|| {
            CompileError::Tool("artifact provenance omits the linked kernel digest".into())
        })?;
    if expected.sha256 != sha256(kernel) {
        return Err(CompileError::Tool(
            "linked kernel bytes do not match artifact provenance".into(),
        ));
    }
    Ok(())
}

fn collect_load_segments<'data>(
    file: &object::File<'data>,
) -> Result<Vec<KernelLoadSegment<'data>>, CompileError> {
    let mut segments = file
        .segments()
        .map(|segment| {
            let data = segment.data().map_err(|error| {
                CompileError::Tool(format!("cannot read linked kernel segment: {error}"))
            })?;
            let (file_offset, _) = segment.file_range();
            let alignment = segment.align().max(1);
            if segment.address() < X86_KERNEL_MINIMUM_ADDRESS
                || alignment < 4096
                || segment.address() % alignment != file_offset % alignment
                || segment.size()
                    < u64::try_from(data.len()).map_err(|_| {
                        CompileError::Tool("kernel segment size does not fit u64".into())
                    })?
            {
                return Err(CompileError::Tool(format!(
                    "linked kernel segment at {:#x} violates the approved load layout",
                    segment.address()
                )));
            }
            let end = segment
                .address()
                .checked_add(segment.size())
                .ok_or_else(|| CompileError::Tool("kernel segment address overflows".into()))?;
            if end > X86_INITIAL_IDENTITY_LIMIT {
                return Err(CompileError::Tool(format!(
                    "linked kernel segment ends beyond the initial identity map: {end:#x}"
                )));
            }
            Ok(KernelLoadSegment {
                address: segment.address(),
                size: segment.size(),
                executable: segment.permissions().executable(),
                data,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if segments.is_empty() {
        return Err(CompileError::Tool(
            "linked kernel ELF contains no loadable segment".into(),
        ));
    }
    segments.sort_by_key(|segment| segment.address);
    for pair in segments.windows(2) {
        let first_end = pair[0].address + pair[0].size;
        if first_end > pair[1].address {
            return Err(CompileError::Tool(
                "linked kernel load segments overlap in physical memory".into(),
            ));
        }
    }
    Ok(segments)
}

fn required_executable_symbol(
    file: &object::File<'_>,
    segments: &[KernelLoadSegment<'_>],
    name: &str,
) -> Result<u64, CompileError> {
    let symbol = file.symbol_by_name(name).ok_or_else(|| {
        CompileError::Tool(format!("linked kernel omits required boot symbol `{name}`"))
    })?;
    let address = symbol.address();
    if symbol.kind() != SymbolKind::Text
        || symbol.size() == 0
        || !segments.iter().any(|segment| {
            segment.executable
                && address >= segment.address
                && address < segment.address + segment.size
        })
    {
        return Err(CompileError::Tool(format!(
            "linked kernel symbol `{name}` is not inside an executable load segment"
        )));
    }
    Ok(address)
}

fn materialize_protected_payload(
    segments: &[KernelLoadSegment<'_>],
) -> Result<Vec<u8>, CompileError> {
    let kernel_end = segments
        .iter()
        .map(|segment| segment.address + segment.size)
        .max()
        .expect("nonempty segments were checked");
    let protected_size = kernel_end - X86_PROTECTED_PAYLOAD_ADDRESS;
    if protected_size > MAX_PROTECTED_PAYLOAD_BYTES {
        return Err(CompileError::Tool(format!(
            "protected payload exceeds the initial {} MiB packaging limit",
            MAX_PROTECTED_PAYLOAD_BYTES / (1024 * 1024)
        )));
    }
    let protected_size = usize::try_from(protected_size)
        .map_err(|_| CompileError::Tool("protected payload does not fit host size".into()))?;
    let mut protected = vec![0; protected_size];
    for segment in segments {
        let offset =
            usize::try_from(segment.address - X86_PROTECTED_PAYLOAD_ADDRESS).map_err(|_| {
                CompileError::Tool("kernel segment offset does not fit host size".into())
            })?;
        let data_end = offset.checked_add(segment.data.len()).ok_or_else(|| {
            CompileError::Tool("kernel segment file extent overflows payload".into())
        })?;
        let memory_end = offset
            .checked_add(usize::try_from(segment.size).map_err(|_| {
                CompileError::Tool("kernel segment memory extent does not fit host size".into())
            })?)
            .ok_or_else(|| CompileError::Tool("kernel segment memory extent overflows".into()))?;
        if data_end > protected.len() || memory_end > protected.len() {
            return Err(CompileError::Tool(
                "kernel segment exceeds protected payload bounds".into(),
            ));
        }
        protected[offset..data_end].copy_from_slice(segment.data);
    }
    Ok(protected)
}

fn install_transition_support(
    protected: &mut [u8],
    kernel_entry: u64,
    debug_break_entry: u64,
    local_notification_entry: u64,
) -> Result<(), CompileError> {
    if X86_KERNEL_MINIMUM_ADDRESS < TRANSITION_RESERVED_END {
        return Err(CompileError::Tool(
            "approved transition and kernel ranges overlap".into(),
        ));
    }
    let transition = transition_code(kernel_entry)?;
    if transition.len() > 4096 {
        return Err(CompileError::Tool(
            "generated transition code exceeds its reserved page".into(),
        ));
    }
    protected[..transition.len()].copy_from_slice(&transition);
    install_page_tables(protected)?;
    install_gdt(protected)?;
    install_idt(protected, debug_break_entry, local_notification_entry)?;
    Ok(())
}

fn transition_code(kernel_entry: u64) -> Result<Vec<u8>, CompileError> {
    let mut code = Vec::new();

    // 32-bit protected entry selected by setup_header.code32_start.
    code.extend_from_slice(&[0x66, 0xb8, 0x18, 0x00]);
    code.extend_from_slice(&[0x8e, 0xd8, 0x8e, 0xc0, 0x8e, 0xd0]);
    code.extend_from_slice(&[0xbc]);
    code.extend_from_slice(&narrow_u32(X86_INITIAL_STACK_TOP, "initial stack")?.to_le_bytes());
    code.extend_from_slice(&[0x0f, 0x20, 0xe0, 0x83, 0xc8, 0x20, 0x0f, 0x22, 0xe0]);
    code.push(0xb8);
    code.extend_from_slice(&narrow_u32(PAGE_TABLE_PML4_ADDRESS, "PML4")?.to_le_bytes());
    code.extend_from_slice(&[0x0f, 0x22, 0xd8]);
    code.push(0xb9);
    code.extend_from_slice(&0xc000_0080_u32.to_le_bytes());
    code.extend_from_slice(&[0x0f, 0x32, 0x0d]);
    code.extend_from_slice(&0x100_u32.to_le_bytes());
    code.extend_from_slice(&[0x0f, 0x30, 0x0f, 0x20, 0xc0, 0x0d]);
    code.extend_from_slice(&0x8000_0000_u32.to_le_bytes());
    code.extend_from_slice(&[0x0f, 0x22, 0xc0]);
    let far_jump_offset = code.len() + 1;
    code.extend_from_slice(&[0xea, 0, 0, 0, 0, 0x10, 0x00]);
    pad_to(&mut code, 16);
    let long_entry =
        X86_PROTECTED_PAYLOAD_ADDRESS
            .checked_add(u64::try_from(code.len()).map_err(|_| {
                CompileError::Tool("transition code length does not fit u64".into())
            })?)
            .ok_or_else(|| CompileError::Tool("long-mode entry address overflows".into()))?;
    code[far_jump_offset..far_jump_offset + 4]
        .copy_from_slice(&narrow_u32(long_entry, "long-mode entry")?.to_le_bytes());

    // 64-bit handoff: retain RSI, install the generated IDT, and jump to entry.
    code.extend_from_slice(&[0x66, 0xb8, 0x18, 0x00]);
    code.extend_from_slice(&[
        0x8e, 0xd8, 0x8e, 0xc0, 0x8e, 0xd0, 0x8e, 0xe0, 0x8e, 0xe8, 0xfc,
    ]);
    code.extend_from_slice(&[0x48, 0xbc]);
    code.extend_from_slice(&X86_INITIAL_STACK_TOP.to_le_bytes());
    code.extend_from_slice(&[0x48, 0xb8]);
    code.extend_from_slice(&IDT_DESCRIPTOR_ADDRESS.to_le_bytes());
    code.extend_from_slice(&[0x0f, 0x01, 0x18, 0x31, 0xed, 0x31, 0xdb, 0x31, 0xff]);
    code.extend_from_slice(&[0x48, 0xb8]);
    code.extend_from_slice(&kernel_entry.to_le_bytes());
    code.extend_from_slice(&[0xff, 0xe0]);
    Ok(code)
}

fn install_page_tables(protected: &mut [u8]) -> Result<(), CompileError> {
    let pml4 = payload_offset(PAGE_TABLE_PML4_ADDRESS, 4096, protected.len())?;
    let pdpt = payload_offset(PAGE_TABLE_PDPT_ADDRESS, 4096, protected.len())?;
    let pd = payload_offset(PAGE_TABLE_PD_ADDRESS, 4096, protected.len())?;
    let apic_pd = payload_offset(LOCAL_APIC_PD_ADDRESS, 4096, protected.len())?;
    write_u64(protected, pml4, PAGE_TABLE_PDPT_ADDRESS | 0x3);
    write_u64(protected, pdpt, PAGE_TABLE_PD_ADDRESS | 0x3);
    write_u64(protected, pdpt + 3 * 8, LOCAL_APIC_PD_ADDRESS | 0x3);
    for index in 0..512_usize {
        let physical = u64::try_from(index)
            .map_err(|_| CompileError::Tool("page-table index does not fit u64".into()))?
            * 0x20_0000;
        write_u64(protected, pd + index * 8, physical | 0x83);
    }
    let apic_index = usize::try_from((LOCAL_APIC_PAGE_ADDRESS >> 21) & 0x1ff)
        .map_err(|_| CompileError::Tool("local-APIC page index does not fit host size".into()))?;
    let hpet_index = usize::try_from((HPET_MMIO_LEAF_ADDRESS >> 21) & 0x1ff)
        .map_err(|_| CompileError::Tool("HPET page index does not fit host size".into()))?;
    write_u64(
        protected,
        apic_pd + hpet_index * 8,
        HPET_MMIO_LEAF_ADDRESS | 0x9b,
    );
    write_u64(
        protected,
        apic_pd + apic_index * 8,
        LOCAL_APIC_PAGE_ADDRESS | 0x9b,
    );
    Ok(())
}

fn install_gdt(protected: &mut [u8]) -> Result<(), CompileError> {
    let gdt = payload_offset(GDT_ADDRESS, 32, protected.len())?;
    let descriptors = [
        0_u64,
        0x00cf_9a00_0000_ffff,
        0x00af_9a00_0000_ffff,
        0x00cf_9200_0000_ffff,
    ];
    for (index, descriptor) in descriptors.into_iter().enumerate() {
        write_u64(protected, gdt + index * 8, descriptor);
    }
    Ok(())
}

fn install_idt(
    protected: &mut [u8],
    debug_break_entry: u64,
    local_notification_entry: u64,
) -> Result<(), CompileError> {
    let idt = payload_offset(IDT_ADDRESS, 4096, protected.len())?;
    let gate = idt + 3 * 16;
    write_u16(
        protected,
        gate,
        u16::try_from(debug_break_entry & 0xffff)
            .map_err(|_| CompileError::Tool("debug entry low word does not fit u16".into()))?,
    );
    write_u16(protected, gate + 2, 0x10);
    protected[gate + 4] = 0;
    protected[gate + 5] = 0x8f;
    write_u16(
        protected,
        gate + 6,
        u16::try_from((debug_break_entry >> 16) & 0xffff)
            .map_err(|_| CompileError::Tool("debug entry middle word does not fit u16".into()))?,
    );
    write_u32(
        protected,
        gate + 8,
        u32::try_from(debug_break_entry >> 32)
            .map_err(|_| CompileError::Tool("debug entry high word does not fit u32".into()))?,
    );
    write_u32(protected, gate + 12, 0);

    let gate = idt + LOCAL_NOTIFICATION_VECTOR * 16;
    write_u16(
        protected,
        gate,
        u16::try_from(local_notification_entry & 0xffff).map_err(|_| {
            CompileError::Tool("local-notification entry low word does not fit u16".into())
        })?,
    );
    write_u16(protected, gate + 2, 0x10);
    protected[gate + 4] = 0;
    protected[gate + 5] = 0x8e;
    write_u16(
        protected,
        gate + 6,
        u16::try_from((local_notification_entry >> 16) & 0xffff).map_err(|_| {
            CompileError::Tool("local-notification entry middle word does not fit u16".into())
        })?,
    );
    write_u32(
        protected,
        gate + 8,
        u32::try_from(local_notification_entry >> 32).map_err(|_| {
            CompileError::Tool("local-notification entry high word does not fit u32".into())
        })?,
    );
    write_u32(protected, gate + 12, 0);

    let descriptor = payload_offset(IDT_DESCRIPTOR_ADDRESS, 10, protected.len())?;
    write_u16(protected, descriptor, 4095);
    write_u64(protected, descriptor + 2, IDT_ADDRESS);
    Ok(())
}

fn build_setup_header(syssize: u32, protected_size: u32) -> Result<Vec<u8>, CompileError> {
    let mut setup = vec![0; SETUP_BYTES];
    setup[0x1f1] = X86_LINUX_SETUP_SECTORS;
    write_u32(&mut setup, 0x1f4, syssize);
    write_u16(&mut setup, 0x1fa, 0xffff);
    write_u16(&mut setup, 0x1fe, 0xaa55);
    setup[0x200] = 0xeb;
    setup[0x201] = u8::try_from(REAL_MODE_ENTRY_OFFSET - 0x202)
        .map_err(|_| CompileError::Tool("real-mode setup entry exceeds short jump".into()))?;
    setup[0x202..0x206].copy_from_slice(b"HdrS");
    write_u16(&mut setup, 0x206, X86_LINUX_BOOT_PROTOCOL);
    setup[0x210] = 0xff;
    setup[0x211] = 0x81;
    write_u32(
        &mut setup,
        0x214,
        narrow_u32(X86_PROTECTED_PAYLOAD_ADDRESS, "protected payload")?,
    );
    write_u16(&mut setup, 0x224, 0xfe00);
    write_u32(&mut setup, 0x22c, 0x37ff_ffff);
    write_u32(&mut setup, 0x230, 4096);
    setup[0x234] = 0;
    setup[0x235] = 12;
    write_u16(&mut setup, 0x236, 1);
    write_u32(&mut setup, 0x238, 2048);
    write_u64(&mut setup, 0x258, X86_PROTECTED_PAYLOAD_ADDRESS);
    write_u32(&mut setup, 0x260, protected_size);

    let continuation = real_mode_continuation_code()?;
    let code = real_mode_setup_code(continuation.len())?;
    let code_end = REAL_MODE_ENTRY_OFFSET + code.len();
    if code_end > usize::from(BOOT_PARAMS_E820_TABLE_OFFSET) {
        return Err(CompileError::Tool(
            "generated real-mode setup overlaps the E820 table".into(),
        ));
    }
    setup[REAL_MODE_ENTRY_OFFSET..code_end].copy_from_slice(&code);
    let continuation_end = REAL_MODE_CONTINUATION_SOURCE_OFFSET
        .checked_add(continuation.len())
        .ok_or_else(|| CompileError::Tool("real-mode continuation extent overflows".into()))?;
    if continuation_end > setup.len() {
        return Err(CompileError::Tool(
            "generated real-mode continuation exceeds the setup image".into(),
        ));
    }
    setup[REAL_MODE_CONTINUATION_SOURCE_OFFSET..continuation_end].copy_from_slice(&continuation);
    Ok(setup)
}

fn real_mode_setup_code(continuation_bytes: usize) -> Result<Vec<u8>, CompileError> {
    let mut code = vec![
        0xfa, // disable interrupts
        0x0e, 0x58, // derive setup base segment from CS - 0x20
        0x83, 0xe8, 0x20, 0x8e, 0xd8, 0x8e, 0xc0, 0x8e, 0xd0, 0xbc, 0x00, 0x7c, // setup stack
        0xfc, // copy forward
    ];

    // Copy the continuation out of the zeropage range which E820 overwrites.
    code.push(0xbe); // mov si, source
    code.extend_from_slice(
        &u16::try_from(REAL_MODE_CONTINUATION_SOURCE_OFFSET)
            .map_err(|_| CompileError::Tool("continuation source exceeds u16".into()))?
            .to_le_bytes(),
    );
    code.push(0xbf); // mov di, relocation
    code.extend_from_slice(&REAL_MODE_CONTINUATION_RELOCATION_OFFSET.to_le_bytes());
    code.push(0xb9); // mov cx, length
    code.extend_from_slice(
        &u16::try_from(continuation_bytes)
            .map_err(|_| CompileError::Tool("continuation length exceeds u16".into()))?
            .to_le_bytes(),
    );
    code.extend_from_slice(&[0xf3, 0xa4]); // rep movsb

    // Populate the Linux zeropage E820 table before leaving BIOS real mode.
    code.extend_from_slice(&[0x66, 0x31, 0xdb]); // xor ebx, ebx (continuation)
    code.extend_from_slice(&[0x31, 0xed]); // xor bp, bp (entry count)
    code.push(0xbf); // mov di, table offset
    code.extend_from_slice(&BOOT_PARAMS_E820_TABLE_OFFSET.to_le_bytes());
    let e820_loop = code.len();
    code.extend_from_slice(&[0x66, 0xb8, 0x20, 0xe8, 0x00, 0x00]); // eax = E820h
    code.extend_from_slice(&[0x66, 0xba, 0x50, 0x41, 0x4d, 0x53]); // edx = "SMAP"
    code.extend_from_slice(&[0x66, 0x31, 0xc9, 0xb1]); // ecx = entry size
    code.push(
        u8::try_from(BOOT_PARAMS_E820_ENTRY_BYTES)
            .map_err(|_| CompileError::Tool("E820 entry size exceeds imm8".into()))?,
    );
    code.extend_from_slice(&[0xcd, 0x15]);
    let carry_to_failure = short_jump_placeholder(&mut code, 0x72); // jc failure
    code.extend_from_slice(&[0x66, 0x3d, 0x50, 0x41, 0x4d, 0x53]);
    let signature_to_failure = short_jump_placeholder(&mut code, 0x75); // jne failure
    code.extend_from_slice(&[0x66, 0x83, 0xf9]);
    code.push(
        u8::try_from(BOOT_PARAMS_E820_ENTRY_BYTES)
            .map_err(|_| CompileError::Tool("E820 entry size exceeds imm8".into()))?,
    );
    let short_entry_to_failure = short_jump_placeholder(&mut code, 0x72); // jb failure
    code.extend_from_slice(&[0x45]); // inc bp
    code.extend_from_slice(&[0x83, 0xc7]); // add di, entry size
    code.push(
        u8::try_from(BOOT_PARAMS_E820_ENTRY_BYTES)
            .map_err(|_| CompileError::Tool("E820 entry size exceeds imm8".into()))?,
    );
    code.extend_from_slice(&[0x66, 0x85, 0xdb]); // test ebx, ebx
    let complete_to_done = short_jump_placeholder(&mut code, 0x74); // je done
    code.extend_from_slice(&[0x81, 0xfd]); // cmp bp, max
    code.extend_from_slice(&u16::from(BOOT_PARAMS_E820_MAX_ENTRIES).to_le_bytes());
    let continuation_to_loop = short_jump_placeholder(&mut code, 0x75); // jne loop
    let e820_failure = code.len();
    code.extend_from_slice(&[0x31, 0xed]); // any truncated result is unusable
    let e820_done = code.len();
    for jump in [
        carry_to_failure,
        signature_to_failure,
        short_entry_to_failure,
    ] {
        patch_short_jump(&mut code, jump, e820_failure)?;
    }
    patch_short_jump(&mut code, complete_to_done, e820_done)?;
    patch_short_jump(&mut code, continuation_to_loop, e820_loop)?;

    code.extend_from_slice(&[0x89, 0xe8, 0xa2]); // publish the complete entry count
    code.extend_from_slice(&BOOT_PARAMS_E820_COUNT_OFFSET.to_le_bytes());
    code.extend_from_slice(&[0x1e, 0x68]); // transfer to the relocated continuation
    code.extend_from_slice(&REAL_MODE_CONTINUATION_RELOCATION_OFFSET.to_le_bytes());
    code.push(0xcb); // retf with CS = boot-parameter segment
    Ok(code)
}

fn real_mode_continuation_code() -> Result<Vec<u8>, CompileError> {
    let mut code = vec![
        0xfa, 0x0e, 0x58, // derive all data segments from relocated CS
        0x8e, 0xd8, 0x8e, 0xc0, 0x8e, 0xd0, 0x66, 0x0f, 0xb7, 0xf0, 0x66, 0xc1, 0xe6, 0x04, 0xe4,
        0x92, 0x0c, 0x02, 0xe6, 0x92, // enable A20 through port 0x92
        0x2e, 0x0f, 0x01, 0x16,
    ];
    let descriptor_offset = usize::from(REAL_MODE_CONTINUATION_RELOCATION_OFFSET)
        .checked_add(code.len() + 2 + 10 + 8)
        .ok_or_else(|| CompileError::Tool("relocated GDT descriptor offset overflows".into()))?;
    code.extend_from_slice(
        &u16::try_from(descriptor_offset)
            .map_err(|_| CompileError::Tool("relocated GDT descriptor exceeds u16".into()))?
            .to_le_bytes(),
    );
    code.extend_from_slice(&[0x0f, 0x20, 0xc0, 0x66, 0x83, 0xc8, 0x01, 0x0f, 0x22, 0xc0]);
    code.extend_from_slice(&[0x66, 0xea]);
    code.extend_from_slice(
        &narrow_u32(X86_PROTECTED_PAYLOAD_ADDRESS, "protected payload")?.to_le_bytes(),
    );
    code.extend_from_slice(&0x08_u16.to_le_bytes());
    code.extend_from_slice(&31_u16.to_le_bytes());
    code.extend_from_slice(&narrow_u32(GDT_ADDRESS, "GDT")?.to_le_bytes());
    Ok(code)
}

fn short_jump_placeholder(code: &mut Vec<u8>, opcode: u8) -> usize {
    code.push(opcode);
    let displacement = code.len();
    code.push(0);
    displacement
}

fn patch_short_jump(
    code: &mut [u8],
    displacement_offset: usize,
    target: usize,
) -> Result<(), CompileError> {
    let origin = displacement_offset
        .checked_add(1)
        .ok_or_else(|| CompileError::Tool("real-mode jump origin overflows".into()))?;
    let displacement =
        i64::try_from(target).unwrap_or(i64::MAX) - i64::try_from(origin).unwrap_or(i64::MIN);
    code[displacement_offset] = i8::try_from(displacement)
        .map_err(|_| CompileError::Tool("real-mode short jump is out of range".into()))?
        .to_le_bytes()[0];
    Ok(())
}

fn narrow_u32(value: u64, description: &str) -> Result<u32, CompileError> {
    u32::try_from(value)
        .map_err(|_| CompileError::Tool(format!("{description} address does not fit u32")))
}

fn payload_offset(address: u64, size: usize, payload_len: usize) -> Result<usize, CompileError> {
    let offset = usize::try_from(address - X86_PROTECTED_PAYLOAD_ADDRESS)
        .map_err(|_| CompileError::Tool("transition address does not fit host size".into()))?;
    if offset.checked_add(size).is_none_or(|end| end > payload_len) {
        return Err(CompileError::Tool(
            "transition structure exceeds protected payload bounds".into(),
        ));
    }
    Ok(offset)
}

fn pad_to(bytes: &mut Vec<u8>, alignment: usize) {
    let remainder = bytes.len() % alignment;
    if remainder != 0 {
        bytes.resize(bytes.len() + alignment - remainder, 0);
    }
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn allocate_stage(parent: &Path) -> Result<PathBuf, CompileError> {
    for _ in 0..100 {
        let sequence = NEXT_BOOT_STAGE.fetch_add(1, Ordering::Relaxed);
        let stage = parent.join(format!(
            ".topal-x86-boot-image-{}-{sequence}",
            std::process::id()
        ));
        match fs::create_dir(&stage) {
            Ok(()) => return Ok(stage),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(io_error("create boot-image stage", &stage)(error)),
        }
    }
    Err(CompileError::Io(
        "cannot allocate a unique x86 boot-image stage".into(),
    ))
}

fn io_error<'a>(
    action: &'static str,
    path: &'a Path,
) -> impl FnOnce(std::io::Error) -> CompileError + 'a {
    move |error| CompileError::Io(format!("cannot {action} {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use topal_language::compiler::{CompilerSystemsTargetSelection, analyze_systems_for_compiler};

    use super::*;
    use crate::{LlvmTools, publish_x86_64_systems_artifact};

    const SOURCE: &str = include_str!("../../../linux-kernel/kernel/arch/x86_64/toolchain-gate.t");

    fn linked_kernel(parent: &Path) -> (Vec<u8>, SystemsArtifactProvenance) {
        let program = analyze_systems_for_compiler(
            SOURCE,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        let tools = LlvmTools::discover(None).unwrap();
        let linked =
            publish_x86_64_systems_artifact(&program, &tools, &parent.join("linked")).unwrap();
        (fs::read(linked.kernel).unwrap(), linked.record)
    }

    fn temporary_parent() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "topal-x86-boot-image-test-{}-{}",
            std::process::id(),
            NEXT_BOOT_STAGE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        path
    }

    #[test]
    fn generates_the_approved_linux_boot_header_and_transition_layout() {
        // TOPAL-COMP-SYSTEMS-ARTIFACT-001, LK-X64-BOOT-001.
        let parent = temporary_parent();
        let (kernel, artifact) = linked_kernel(&parent);
        let generated = generate_x86_64_linux_boot_image(&kernel, &artifact).unwrap();
        assert_eq!(generated.bytes[0x1f1], X86_LINUX_SETUP_SECTORS);
        assert_eq!(&generated.bytes[0x1fe..0x200], &[0x55, 0xaa]);
        assert_eq!(&generated.bytes[0x202..0x206], b"HdrS");
        assert_eq!(
            u16::from_le_bytes(generated.bytes[0x206..0x208].try_into().unwrap()),
            X86_LINUX_BOOT_PROTOCOL
        );
        assert_eq!(generated.bytes[0x211], 0x81);
        let real_mode =
            &generated.bytes[REAL_MODE_ENTRY_OFFSET..usize::from(BOOT_PARAMS_E820_TABLE_OFFSET)];
        assert!(real_mode.windows(2).any(|bytes| bytes == [0xcd, 0x15]));
        assert!(real_mode.windows(5).any(|bytes| {
            bytes
                == [
                    0x89,
                    0xe8,
                    0xa2,
                    BOOT_PARAMS_E820_COUNT_OFFSET.to_le_bytes()[0],
                    BOOT_PARAMS_E820_COUNT_OFFSET.to_le_bytes()[1],
                ]
        }));
        assert_eq!(
            &generated.bytes
                [REAL_MODE_CONTINUATION_SOURCE_OFFSET..REAL_MODE_CONTINUATION_SOURCE_OFFSET + 3],
            &[0xfa, 0x0e, 0x58]
        );
        assert_eq!(
            u32::from_le_bytes(generated.bytes[0x214..0x218].try_into().unwrap()),
            u32::try_from(X86_PROTECTED_PAYLOAD_ADDRESS).unwrap()
        );
        assert_eq!(
            u16::from_le_bytes(generated.bytes[0x236..0x238].try_into().unwrap()),
            1
        );
        let protected = &generated.bytes[SETUP_BYTES..];
        let pml4 =
            usize::try_from(PAGE_TABLE_PML4_ADDRESS - X86_PROTECTED_PAYLOAD_ADDRESS).unwrap();
        assert_eq!(
            u64::from_le_bytes(protected[pml4..pml4 + 8].try_into().unwrap()),
            PAGE_TABLE_PDPT_ADDRESS | 3
        );
        let gate = usize::try_from(IDT_ADDRESS - X86_PROTECTED_PAYLOAD_ADDRESS).unwrap() + 3 * 16;
        assert_eq!(protected[gate + 5], 0x8f);
        let local_gate = usize::try_from(IDT_ADDRESS - X86_PROTECTED_PAYLOAD_ADDRESS).unwrap()
            + LOCAL_NOTIFICATION_VECTOR * 16;
        assert_eq!(protected[local_gate + 5], 0x8e);
        let pdpt =
            usize::try_from(PAGE_TABLE_PDPT_ADDRESS - X86_PROTECTED_PAYLOAD_ADDRESS).unwrap();
        assert_eq!(
            u64::from_le_bytes(protected[pdpt + 24..pdpt + 32].try_into().unwrap()),
            LOCAL_APIC_PD_ADDRESS | 3
        );
        let apic_pd =
            usize::try_from(LOCAL_APIC_PD_ADDRESS - X86_PROTECTED_PAYLOAD_ADDRESS).unwrap();
        let hpet_index = usize::try_from((HPET_MMIO_LEAF_ADDRESS >> 21) & 0x1ff).unwrap();
        assert_eq!(
            u64::from_le_bytes(
                protected[apic_pd + hpet_index * 8..apic_pd + hpet_index * 8 + 8]
                    .try_into()
                    .unwrap()
            ),
            HPET_MMIO_LEAF_ADDRESS | 0x9b
        );
        let apic_index = usize::try_from((LOCAL_APIC_PAGE_ADDRESS >> 21) & 0x1ff).unwrap();
        assert_eq!(
            u64::from_le_bytes(
                protected[apic_pd + apic_index * 8..apic_pd + apic_index * 8 + 8]
                    .try_into()
                    .unwrap()
            ),
            LOCAL_APIC_PAGE_ADDRESS | 0x9b
        );
        assert_eq!(
            generated.provenance.local_notification_entry,
            u64::from(u16::from_le_bytes(
                protected[local_gate..local_gate + 2].try_into().unwrap()
            )) | (u64::from(u16::from_le_bytes(
                protected[local_gate + 6..local_gate + 8]
                    .try_into()
                    .unwrap()
            )) << 16)
                | (u64::from(u32::from_le_bytes(
                    protected[local_gate + 8..local_gate + 12]
                        .try_into()
                        .unwrap()
                )) << 32)
        );
        assert_eq!(generated.provenance.linked_kernel_sha256, sha256(&kernel));
        assert_eq!(
            generated.provenance.boot_image_sha256,
            sha256(&generated.bytes)
        );
        fs::remove_dir_all(parent).unwrap();
    }

    #[test]
    fn atomically_publishes_a_deterministic_image_and_provenance_pair() {
        // TOPAL-SYSTEMS-ARTIFACT-001, TOPAL-SYSTEMS-QUALIFY-001.
        let parent = temporary_parent();
        let (kernel, artifact) = linked_kernel(&parent);
        let first =
            publish_x86_64_linux_boot_image(&kernel, &artifact, &parent.join("first")).unwrap();
        let second =
            publish_x86_64_linux_boot_image(&kernel, &artifact, &parent.join("second")).unwrap();
        assert_eq!(fs::read_dir(&first.directory).unwrap().count(), 2);
        assert_eq!(
            fs::read(&first.image).unwrap(),
            fs::read(&second.image).unwrap()
        );
        assert_eq!(
            fs::read(&first.provenance).unwrap(),
            fs::read(&second.provenance).unwrap()
        );
        let decoded: X86LinuxBootImageProvenance =
            serde_json::from_slice(&fs::read(&first.provenance).unwrap()).unwrap();
        assert_eq!(decoded, first.record);
        let expected_entry = artifact
            .placements
            .iter()
            .find(|placement| placement.symbol == X86_SYSTEMS_KERNEL_ENTRY)
            .unwrap()
            .address;
        assert_eq!(decoded.kernel_entry, expected_entry);
        fs::remove_dir_all(parent).unwrap();
    }
}
