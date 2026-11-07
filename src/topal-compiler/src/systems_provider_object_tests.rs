#[cfg(all(target_arch = "x86_64", target_family = "unix"))]
use std::process::Command;

use object::{Object as _, ObjectSection as _, ObjectSymbol as _};
use topal_language::compiler::{CompilerSystemsTargetSelection, analyze_systems_for_compiler};

use super::*;

const SOURCE: &str = include_str!("../../../linux-kernel/kernel/arch/x86_64/toolchain-gate.t");

fn generated() -> GeneratedSystemsProviderObject {
    generated_with_page_source(true)
}

fn generated_with_page_source(selected: bool) -> GeneratedSystemsProviderObject {
    generated_with_sources(selected, true)
}

fn generated_with_sources(source_page: bool, source_inode: bool) -> GeneratedSystemsProviderObject {
    let mut program = analyze_systems_for_compiler(
        SOURCE,
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    // These machine-instruction/relocation controls retain the exact V5
    // provider. V6's additional namespace closure is qualified independently.
    program.namespace_library = None;
    let dispatch = program.dispatch_library.as_mut().unwrap();
    dispatch.interface =
        topal_language::compiler::CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV5;
    dispatch.source = "arch/x86_64/policy/static-linux-dispatch-v5.t".into();
    if !source_page {
        program.ram_page_pool_library = None;
    }
    if !source_inode {
        program.inode_mode_library = None;
        let dispatch = program.dispatch_library.as_mut().unwrap();
        dispatch.interface =
            topal_language::compiler::CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV4;
        dispatch.source = "arch/x86_64/policy/static-linux-dispatch-v4.t".into();
    }
    generate_x86_64_systems_provider_object(&program).unwrap()
}

fn symbol_bytes(file: &object::File<'_>, name: &str) -> Vec<u8> {
    let symbol = file.symbol_by_name(name).unwrap();
    let section = file
        .section_by_index(symbol.section_index().unwrap())
        .unwrap();
    let data = section.data().unwrap();
    let start = usize::try_from(symbol.address()).unwrap();
    let end = start + usize::try_from(symbol.size()).unwrap();
    data[start..end].to_vec()
}

#[cfg(all(target_arch = "x86_64", target_family = "unix"))]
fn check_entropy_mix_against_independent_reference(bytes: &[u8]) {
    let encoded = bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let script = r#"
import ctypes
import hashlib
import mmap
import sys

code = bytes.fromhex(sys.argv[1])
memory = mmap.mmap(
    -1,
    len(code),
    prot=mmap.PROT_READ | mmap.PROT_WRITE | mmap.PROT_EXEC,
)
memory.write(code)
address = ctypes.addressof(ctypes.c_char.from_buffer(memory))
function = ctypes.CFUNCTYPE(
    ctypes.c_uint32,
    ctypes.POINTER(ctypes.c_ubyte),
    ctypes.POINTER(ctypes.c_ubyte),
)(address)
sample = (ctypes.c_ubyte * 32)(*range(32))
pool = (ctypes.c_ubyte * 32)()
assert function(sample, pool) == 1
assert bytes(sample) == bytes(32)
expected = hashlib.blake2s(
    bytes(range(32)),
    digest_size=32,
    person=b"TopalRNG",
).digest()
assert expected.hex() == "dd8ab1645ca92dd40c801f4f9ac8a4852159709249e4cdb4cf7cab58e899e24e"
assert bytes(pool) == expected
"#;
    let output = Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(encoded)
        .output()
        .expect("Python is available for the independent BLAKE2s reference check");
    assert!(
        output.status.success(),
        "generated entropy mix failed its executable reference check: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(all(target_arch = "x86_64", target_family = "unix"))]
fn check_entropy_extract_against_independent_reference(bytes: &[u8]) {
    let encoded = bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let script = r#"
import ctypes
import mmap
import sys

code = bytes.fromhex(sys.argv[1])
memory = mmap.mmap(
    -1,
    len(code),
    prot=mmap.PROT_READ | mmap.PROT_WRITE | mmap.PROT_EXEC,
)
memory.write(code)
address = ctypes.addressof(ctypes.c_char.from_buffer(memory))
function = ctypes.CFUNCTYPE(
    ctypes.c_uint32,
    ctypes.POINTER(ctypes.c_ubyte),
    ctypes.POINTER(ctypes.c_ubyte),
)(address)
key = (ctypes.c_ubyte * 32)()
output = (ctypes.c_ubyte * 32)()
assert function(key, output) == 1
block = bytes.fromhex(
    "76b8e0ada0f13d90405d6ae55386bd28"
    "bdd219b8a08ded1aa836efcc8b770dc7"
    "da41597c5157488d7724e03fb8d84a37"
    "6a43b8f41518a11cc387b669b2ee6586"
)
assert bytes(key) == block[:32]
assert bytes(output) == block[32:]
"#;
    let output = Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(encoded)
        .output()
        .expect("Python is available for the independent ChaCha20 reference check");
    assert!(
        output.status.success(),
        "generated entropy extraction failed its executable reference check: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn emits_a_closed_relocatable_elf64_provider_object() {
    // TOPAL-COMP-SYSTEMS-X64-001, TOPAL-SYSTEMS-ARTIFACT-001;
    // TOPAL-COMP-SYSTEMS-REGION-PAGE-001. Source imports are explicit for each
    // selected caller; retaining native page allocation keeps its old closure.
    for (source_page, source_inode) in [(false, false), (false, true), (true, false), (true, true)]
    {
        let first = generated_with_sources(source_page, source_inode);
        let repeated = generated_with_sources(source_page, source_inode);
        assert_eq!(first.bytes, repeated.bytes);
        let file = object::File::parse(first.bytes.as_slice()).unwrap();
        assert_eq!(file.format(), BinaryFormat::Elf);
        assert_eq!(file.architecture(), Architecture::X86_64);
        assert!(file.is_little_endian());
        assert_eq!(file.kind(), object::ObjectKind::Relocatable);
        assert_eq!(
            file.sections()
                .flat_map(|section| section.relocations())
                .count(),
            51 + if source_page { 29 } else { 0 } + if source_inode { 26 } else { 0 },
            "source page/inode: {source_page}/{source_inode}"
        );
        let mut undefined = std::collections::BTreeSet::from([
            crate::systems_region_library::ROOT_FRAME_EXPORT,
            crate::systems_region_library::FILE_FRAME_EXPORT,
        ]);
        if source_page {
            undefined.insert(crate::systems_region_library::PAGE_FRAME_EXPORT);
        }
        if source_inode {
            undefined.insert(crate::systems_region_library::INODE_FRAME_EXPORT);
        }
        if source_page || source_inode {
            // The page controller precedes process storage in the object. Its
            // forward data reference resolves to this same object's sole strong
            // definition; it is not an additional external source/host import.
            let definitions = file
                .symbols()
                .filter(|symbol| {
                    symbol.is_definition()
                        && symbol.name().ok()
                            == Some(X86_SYSTEMS_STATIC_PROCESS_DESCRIPTOR_TABLE_SYMBOL)
                })
                .collect::<Vec<_>>();
            assert_eq!(definitions.len(), 1);
            let definition = &definitions[0];
            assert_eq!(definition.kind(), SymbolKind::Data);
            assert_eq!(definition.scope(), SymbolScope::Linkage);
            assert!(!definition.is_weak());
            assert_eq!(definition.size(), process_runtime::children::STORAGE_BYTES);
            assert_eq!(definition.address() % 4096, 0);
            let section = file
                .section_by_index(definition.section_index().unwrap())
                .unwrap();
            assert_eq!(
                section.name().unwrap(),
                X86_SYSTEMS_STATIC_PROCESS_STORAGE_SECTION
            );
            assert_eq!(section.kind(), SectionKind::UninitializedData);
            undefined.insert(X86_SYSTEMS_STATIC_PROCESS_DESCRIPTOR_TABLE_SYMBOL);
        }
        assert_eq!(
            file.symbols()
                .filter(object::ObjectSymbol::is_undefined)
                .map(|symbol| symbol.name().unwrap())
                .collect::<std::collections::BTreeSet<_>>(),
            undefined,
            "source page/inode: {source_page}/{source_inode}"
        );
        let frames = file.section_by_name(".debug_frame").unwrap();
        // ELF readers classify nonallocated debugger sections as Other.
        assert_eq!(frames.kind(), SectionKind::Other);
        assert!(
            matches!(frames.flags(), object::SectionFlags::Elf { sh_flags, .. } if sh_flags.0 & 2 == 0)
        );
        assert_eq!(
            file.section_by_name(X86_SYSTEMS_PROVIDER_TEXT_SECTION)
                .unwrap()
                .kind(),
            SectionKind::Text
        );
    }
}

#[test]
fn retains_checked_storage_and_provider_provenance() {
    // TOPAL-SYSTEMS-STORAGE-001, TOPAL-SYSTEMS-QUALIFY-001.
    let generated = generated();
    let file = object::File::parse(generated.bytes.as_slice()).unwrap();
    let storage = file
        .section_by_name(X86_SYSTEMS_BOOTSTRAP_STORAGE_SECTION)
        .unwrap();
    assert_eq!(storage.kind(), SectionKind::UninitializedData);
    assert_eq!(storage.size(), 65_536);
    assert_eq!(storage.align(), 4096);
    let dma_storage = file
        .section_by_name(X86_SYSTEMS_DMA_STORAGE_SECTION)
        .unwrap();
    assert_eq!(dma_storage.kind(), SectionKind::UninitializedData);
    assert_eq!(dma_storage.size(), 8192);
    assert_eq!(dma_storage.align(), 4096);
    let control = file
        .symbol_by_name(X86_SYSTEMS_DMA_VIRTQUEUE_CONTROL_SYMBOL)
        .unwrap();
    let payload = file
        .symbol_by_name(X86_SYSTEMS_DMA_ENTROPY_PAYLOAD_SYMBOL)
        .unwrap();
    assert_eq!(control.size(), 4096);
    assert_eq!(payload.size(), 4096);
    assert_eq!(payload.address() - control.address(), 4096);
    let entropy_storage = file
        .section_by_name(X86_SYSTEMS_ENTROPY_STORAGE_SECTION)
        .unwrap();
    assert_eq!(entropy_storage.kind(), SectionKind::UninitializedData);
    assert_eq!(entropy_storage.size(), 160);
    assert_eq!(entropy_storage.align(), 32);
    let raw_sample = file
        .symbol_by_name(X86_SYSTEMS_RAW_ENTROPY_SAMPLE_SYMBOL)
        .unwrap();
    assert_eq!(raw_sample.size(), 32);
    assert_eq!(raw_sample.address(), 0);
    assert_eq!(raw_sample.section_index(), entropy_storage.index().into());
    let pool_state = file
        .symbol_by_name(X86_SYSTEMS_ENTROPY_POOL_STATE_SYMBOL)
        .unwrap();
    assert_eq!(pool_state.size(), 32);
    assert_eq!(pool_state.address(), 32);
    assert_eq!(pool_state.section_index(), entropy_storage.index().into());
    let generator_key = file
        .symbol_by_name(X86_SYSTEMS_ENTROPY_GENERATOR_KEY_SYMBOL)
        .unwrap();
    assert_eq!(generator_key.size(), 32);
    assert_eq!(generator_key.address(), 64);
    assert_eq!(
        generator_key.section_index(),
        entropy_storage.index().into()
    );
    let entropy_output = file
        .symbol_by_name(X86_SYSTEMS_ENTROPY_OUTPUT_SYMBOL)
        .unwrap();
    assert_eq!(entropy_output.size(), 32);
    assert_eq!(entropy_output.address(), 96);
    assert_eq!(
        entropy_output.section_index(),
        entropy_storage.index().into()
    );
    let user_random_service = file
        .symbol_by_name(X86_SYSTEMS_ENTROPY_USER_RANDOM_SERVICE_SYMBOL)
        .unwrap();
    assert_eq!(user_random_service.size(), 32);
    assert_eq!(user_random_service.address(), 128);
    assert_eq!(
        user_random_service.section_index(),
        entropy_storage.index().into()
    );
    let note = file
        .section_by_name(X86_SYSTEMS_PROVIDER_NOTE_SECTION)
        .unwrap()
        .data()
        .unwrap();
    assert_eq!(u32::from_le_bytes(note[0..4].try_into().unwrap()), 6);
    let description_size = u32::from_le_bytes(note[4..8].try_into().unwrap()) as usize;
    assert_eq!(u32::from_le_bytes(note[8..12].try_into().unwrap()), 1);
    assert_eq!(&note[12..18], b"TOPAL\0");
    let description_start = 12 + aligned_note_size(6);
    let note = std::str::from_utf8(&note[description_start..description_start + description_size])
        .unwrap();
    assert!(note.contains(X86_SYSTEMS_PROVIDER_OBJECT_REVISION));
    assert!(note.contains(X86_SYSTEMS_PROVIDER_REVISION));
    assert!(note.contains("target=x86_64-unknown-none"));
    assert!(note.contains("machine-cpu=qemu64-v1"));
    assert!(!note.contains("linux-gnu"));
}

#[test]
#[allow(clippy::too_many_lines)]
fn emits_only_the_sealed_provider_instruction_sequences() {
    // TOPAL-SYSTEMS-MACHINE-001, TOPAL-SYSTEMS-DISPOSITION-001.
    let generated = generated();
    let file = object::File::parse(generated.bytes.as_slice()).unwrap();
    audit_provider_devices(&file);
    let data = audit_provider_bootstrap(&file);
    audit_provider_entropy_authority(&file);
    let (process_enter, process_restore, process_fixture_image) =
        audit_provider_process_entry(&file);
    audit_provider_process_syscalls(
        &process_enter,
        &process_restore,
        &process_fixture_image[0x0a_7000..0x0a_a000],
    );
    audit_provider_image_mapping(&file, data);
    let entry_payload = audit_provider_user_execution(&file);
    audit_provider_user_completion(&file, data, &entry_payload);
}
include!("systems_provider_audit_devices.rs");
include!("systems_provider_audit_bootstrap.rs");
include!("systems_provider_audit_entropy_authority.rs");
include!("systems_provider_audit_process_entry.rs");
include!("systems_provider_audit_process_syscalls.rs");
include!("systems_provider_audit_image_mapping.rs");
include!("systems_provider_audit_user_execution.rs");
include!("systems_provider_audit_user_completion.rs");
