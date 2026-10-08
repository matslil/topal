//! Generated ELF object for the sealed initial x86-64 systems provider.

use std::collections::BTreeMap;

use object::write::{Object, Symbol, SymbolSection};
use object::{
    Architecture, BinaryFormat, Endianness, SectionKind, SymbolFlags, SymbolKind, SymbolScope,
};
use topal_language::compiler::CompilerSystemsProgram;

use crate::{
    CompileError, X86_SYSTEMS_PROVIDER_REVISION, X86SystemsProviderPlan,
    plan_x86_64_systems_provider,
};

pub const X86_SYSTEMS_PROVIDER_OBJECT_REVISION: &str = "topal.provider-object.x86_64-qemu-pc-q35/3";
pub const X86_SYSTEMS_PROVIDER_TEXT_SECTION: &str = ".text.topal.systems.provider";
pub const X86_SYSTEMS_BOOTSTRAP_STORAGE_SECTION: &str = ".bss.topal.bootstrap";
pub const X86_SYSTEMS_PROVIDER_NOTE_SECTION: &str = ".note.topal.provider";
pub const X86_SYSTEMS_BOOT_MEMORY_SYMBOL: &str = "topal_x86_systems_describe_boot_memory";
pub const X86_SYSTEMS_FRAME_ALLOCATE_SYMBOL: &str = "topal_x86_systems_allocate_physical_frames";
pub const X86_SYSTEMS_ALLOCATABLE_FLOOR: u64 = 16 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum X86Instruction {
    MoveUartLineStatusPortToDx,
    InputByteFromPort,
    TestTransmitterHoldingRegisterEmpty,
    RetryWhileTransmitterBusy,
    MoveUartDataPortToDx,
    MoveByteArgumentToAl,
    OutputByteToPort,
    Breakpoint,
    InterruptReturn,
    DisableInterrupts,
    Halt,
    JumpBackToHalt,
    Return,
}

impl X86Instruction {
    fn encode(self, output: &mut Vec<u8>) {
        match self {
            Self::MoveUartLineStatusPortToDx => {
                output.extend_from_slice(&[0x66, 0xba, 0xfd, 0x03]);
            }
            Self::InputByteFromPort => output.push(0xec),
            Self::TestTransmitterHoldingRegisterEmpty => {
                output.extend_from_slice(&[0xa8, 0x20]);
            }
            Self::RetryWhileTransmitterBusy => output.extend_from_slice(&[0x74, 0xfb]),
            Self::MoveUartDataPortToDx => {
                output.extend_from_slice(&[0x66, 0xba, 0xf8, 0x03]);
            }
            Self::MoveByteArgumentToAl => output.extend_from_slice(&[0x40, 0x88, 0xf8]),
            Self::OutputByteToPort => output.push(0xee),
            Self::Breakpoint => output.push(0xcc),
            Self::InterruptReturn => output.extend_from_slice(&[0x48, 0xcf]),
            Self::DisableInterrupts => output.push(0xfa),
            Self::Halt => output.push(0xf4),
            Self::JumpBackToHalt => output.extend_from_slice(&[0xeb, 0xfd]),
            Self::Return => output.push(0xc3),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedSystemsProviderObject {
    pub plan: X86SystemsProviderPlan,
    pub bytes: Vec<u8>,
}

/// Generate one relocatable ELF64 provider object from a checked systems root.
///
/// # Errors
///
/// Returns a tool error when provider planning fails or the ELF writer cannot
/// encode the closed object.
pub fn generate_x86_64_systems_provider_object(
    program: &CompilerSystemsProgram,
) -> Result<GeneratedSystemsProviderObject, CompileError> {
    let plan = plan_x86_64_systems_provider(program)?;
    let mut object = Object::new(BinaryFormat::Elf, Architecture::X86_64, Endianness::Little);
    object.add_file_symbol(b"topal-generated-x86-systems-provider".to_vec());

    let text = object.add_section(
        Vec::new(),
        X86_SYSTEMS_PROVIDER_TEXT_SECTION.as_bytes().to_vec(),
        SectionKind::Text,
    );
    append_function(
        &mut object,
        text,
        "topal_x86_systems_uart16550_write",
        &[
            X86Instruction::MoveUartLineStatusPortToDx,
            X86Instruction::InputByteFromPort,
            X86Instruction::TestTransmitterHoldingRegisterEmpty,
            X86Instruction::RetryWhileTransmitterBusy,
            X86Instruction::MoveUartDataPortToDx,
            X86Instruction::MoveByteArgumentToAl,
            X86Instruction::OutputByteToPort,
            X86Instruction::Return,
        ],
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_BOOT_MEMORY_SYMBOL,
        &boot_memory_validator()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_FRAME_ALLOCATE_SYMBOL,
        &boot_memory_validator()?,
    );
    append_function(
        &mut object,
        text,
        "topal_x86_systems_debug_break",
        &[X86Instruction::Breakpoint, X86Instruction::Return],
    );
    append_function(
        &mut object,
        text,
        "topal_x86_systems_interrupt_return",
        &[X86Instruction::InterruptReturn],
    );
    append_function(
        &mut object,
        text,
        "topal_x86_systems_fatal",
        &[
            X86Instruction::DisableInterrupts,
            X86Instruction::Halt,
            X86Instruction::JumpBackToHalt,
        ],
    );

    let storage = object.add_section(
        Vec::new(),
        X86_SYSTEMS_BOOTSTRAP_STORAGE_SECTION.as_bytes().to_vec(),
        SectionKind::UninitializedData,
    );
    let storage_offset = object.append_section_bss(
        storage,
        plan.bootstrap_placement.capacity_bytes,
        plan.bootstrap_placement.alignment_bytes,
    );
    object.add_symbol(Symbol {
        name: b"topal_bootstrap_storage".to_vec(),
        value: storage_offset,
        size: plan.bootstrap_placement.capacity_bytes,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(storage),
        flags: SymbolFlags::None,
    });

    let note = object.add_section(
        Vec::new(),
        X86_SYSTEMS_PROVIDER_NOTE_SECTION.as_bytes().to_vec(),
        SectionKind::Note,
    );
    let note_description = provider_note(&plan);
    object.append_section_data(note, &elf_note(note_description.as_bytes())?, 4);

    let bytes = object.write().map_err(|error| {
        CompileError::Tool(format!("cannot encode systems provider ELF: {error}"))
    })?;
    Ok(GeneratedSystemsProviderObject { plan, bytes })
}

fn append_function(
    object: &mut Object<'_>,
    section: object::write::SectionId,
    name: &str,
    instructions: &[X86Instruction],
) {
    let mut encoded = Vec::new();
    for instruction in instructions {
        instruction.encode(&mut encoded);
    }
    let offset = object.append_section_data(section, &encoded, 16);
    object.add_symbol(Symbol {
        name: name.as_bytes().to_vec(),
        value: offset,
        size: encoded.len() as u64,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(section),
        flags: SymbolFlags::None,
    });
}

fn append_encoded_function(
    object: &mut Object<'_>,
    section: object::write::SectionId,
    name: &str,
    encoded: &[u8],
) {
    let offset = object.append_section_data(section, encoded, 16);
    object.add_symbol(Symbol {
        name: name.as_bytes().to_vec(),
        value: offset,
        size: encoded.len() as u64,
        kind: SymbolKind::Text,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(section),
        flags: SymbolFlags::None,
    });
}

fn boot_memory_validator() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xf6]); // test rsi, rsi
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0xf0, 0xff, 0x00]); // cmp rsi, 0x00fff000
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x83, 0xbe, 0x50, 0x02, 0x00, 0x00, 0x00]); // setup_data == 0
    code.jump_if(0x85, "fail");
    code.bytes(&[0x0f, 0xb6, 0x8e, 0xe8, 0x01, 0x00, 0x00]); // e820_entries
    code.bytes(&[0x85, 0xc9]); // test ecx, ecx
    code.jump_if(0x84, "fail");
    code.bytes(&[0x81, 0xf9, 0x80, 0x00, 0x00, 0x00]); // cmp ecx, 128
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x8d, 0x96, 0xd0, 0x02, 0x00, 0x00]); // first E820 entry
    code.bytes(&[0x45, 0x31, 0xc0]); // xor r8d, r8d

    code.bind("loop")?;
    code.bytes(&[0x48, 0x8b, 0x02]); // base
    code.bytes(&[0x4c, 0x8b, 0x4a, 0x08]); // length
    code.bytes(&[0x44, 0x8b, 0x52, 0x10]); // type
    code.bytes(&[0x4d, 0x85, 0xc9]); // test length
    code.jump_if(0x84, "zero-length");
    code.bytes(&[0x49, 0x89, 0xc3]); // end = base
    code.bytes(&[0x4d, 0x01, 0xcb]); // end += length
    code.jump_if(0x82, "fail");
    code.bytes(&[0x41, 0x83, 0xfa, 0x01]); // E820 RAM
    code.jump_if(0x85, "next");
    code.bytes(&[0x48, 0x3d, 0x00, 0x00, 0x00, 0x01]); // base >= 16 MiB
    code.jump_if(0x83, "base-ready");
    code.bytes(&[0xb8, 0x00, 0x00, 0x00, 0x01]); // candidate = 16 MiB
    code.bind("base-ready")?;
    code.bytes(&[0x48, 0x05, 0xff, 0x0f, 0x00, 0x00]); // align candidate up
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x25, 0x00, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x49, 0x81, 0xe3, 0x00, 0xf0, 0xff, 0xff]); // align end down
    code.bytes(&[0x4c, 0x39, 0xd8]); // cmp candidate, end
    code.jump_if(0x83, "next");
    code.bytes(&[0x41, 0xb0, 0x01]); // one allocatable page exists
    code.jump("next");

    code.bind("zero-length")?;
    code.bytes(&[0x41, 0x83, 0xfa, 0x01]);
    code.jump_if(0x84, "fail"); // empty RAM is malformed

    code.bind("next")?;
    code.bytes(&[0x48, 0x83, 0xc2, 0x14]); // next 20-byte entry
    code.bytes(&[0xff, 0xc9]); // dec ecx
    code.jump_if(0x85, "loop");
    code.bytes(&[0x45, 0x84, 0xc0]); // test r8b, r8b
    code.jump_if(0x84, "fail");
    code.bytes(&[0xb0, 0x01, 0xc3]); // success

    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]); // failure
    code.finish()
}

#[derive(Default)]
struct X86FunctionEncoder {
    bytes: Vec<u8>,
    labels: BTreeMap<&'static str, usize>,
    fixups: Vec<(usize, &'static str)>,
}

impl X86FunctionEncoder {
    fn bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    fn bind(&mut self, label: &'static str) -> Result<(), CompileError> {
        if self.labels.insert(label, self.bytes.len()).is_some() {
            return Err(CompileError::Tool(format!(
                "duplicate generated x86 label `{label}`"
            )));
        }
        Ok(())
    }

    fn jump_if(&mut self, condition: u8, label: &'static str) {
        self.bytes.extend_from_slice(&[0x0f, condition]);
        self.fixup(label);
    }

    fn jump(&mut self, label: &'static str) {
        self.bytes.push(0xe9);
        self.fixup(label);
    }

    fn fixup(&mut self, label: &'static str) {
        let offset = self.bytes.len();
        self.fixups.push((offset, label));
        self.bytes.extend_from_slice(&[0; 4]);
    }

    fn finish(mut self) -> Result<Vec<u8>, CompileError> {
        for (offset, label) in self.fixups {
            let target = *self.labels.get(label).ok_or_else(|| {
                CompileError::Tool(format!("missing generated x86 label `{label}`"))
            })?;
            let displacement = i64::try_from(target).unwrap_or(i64::MAX)
                - i64::try_from(offset + 4).unwrap_or(i64::MIN);
            let displacement = i32::try_from(displacement).map_err(|_| {
                CompileError::Tool(format!("generated x86 jump to `{label}` is out of range"))
            })?;
            self.bytes[offset..offset + 4].copy_from_slice(&displacement.to_le_bytes());
        }
        Ok(self.bytes)
    }
}

fn provider_note(plan: &X86SystemsProviderPlan) -> String {
    format!(
        "revision={X86_SYSTEMS_PROVIDER_OBJECT_REVISION}\nprovider={}\ntarget={}\nboard={}\nprofile={}\nmachine-cpu={}\ncodegen-cpu={}\nplatform-abi={}\ndata-layout={}\nobject-format={}\nrelocation-model={}\ncode-model={}\nstorage-capacity={}\nstorage-alignment={}\n",
        X86_SYSTEMS_PROVIDER_REVISION,
        plan.target,
        plan.board,
        plan.profile,
        plan.machine_cpu_model,
        plan.codegen_cpu,
        plan.platform_abi,
        plan.data_layout,
        plan.object_format,
        plan.relocation_model,
        plan.code_model,
        plan.bootstrap_placement.capacity_bytes,
        plan.bootstrap_placement.alignment_bytes,
    )
}

fn elf_note(description: &[u8]) -> Result<Vec<u8>, CompileError> {
    const NAME: &[u8] = b"TOPAL\0";
    const PROVIDER_NOTE_TYPE: u32 = 1;

    let description_size = u32::try_from(description.len()).map_err(|_| {
        CompileError::Tool("systems provider ELF note exceeds the ELF32 field limit".into())
    })?;
    let mut note = Vec::with_capacity(
        12 + aligned_note_size(NAME.len()) + aligned_note_size(description.len()),
    );
    note.extend_from_slice(&6_u32.to_le_bytes());
    note.extend_from_slice(&description_size.to_le_bytes());
    note.extend_from_slice(&PROVIDER_NOTE_TYPE.to_le_bytes());
    append_note_field(&mut note, NAME);
    append_note_field(&mut note, description);
    Ok(note)
}

const fn aligned_note_size(size: usize) -> usize {
    (size + 3) & !3
}

fn append_note_field(note: &mut Vec<u8>, field: &[u8]) {
    note.extend_from_slice(field);
    note.resize(
        note.len() + (aligned_note_size(field.len()) - field.len()),
        0,
    );
}

#[cfg(test)]
mod tests {
    use object::{Object as _, ObjectSection as _, ObjectSymbol as _};
    use topal_language::compiler::{CompilerSystemsTargetSelection, analyze_systems_for_compiler};

    use super::*;

    const SOURCE: &str = include_str!("../../../linux-kernel/kernel/arch/x86_64/toolchain-gate.t");

    fn generated() -> GeneratedSystemsProviderObject {
        let program = analyze_systems_for_compiler(
            SOURCE,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap();
        generate_x86_64_systems_provider_object(&program).unwrap()
    }

    #[test]
    fn emits_a_closed_relocatable_elf64_provider_object() {
        // TOPAL-COMP-SYSTEMS-X64-001, TOPAL-SYSTEMS-ARTIFACT-001.
        let first = generated();
        let repeated = generated();
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
            0
        );
        assert_eq!(
            file.symbols()
                .filter(object::ObjectSymbol::is_undefined)
                .count(),
            0
        );
        assert_eq!(
            file.section_by_name(X86_SYSTEMS_PROVIDER_TEXT_SECTION)
                .unwrap()
                .kind(),
            SectionKind::Text
        );
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
        let note =
            std::str::from_utf8(&note[description_start..description_start + description_size])
                .unwrap();
        assert!(note.contains(X86_SYSTEMS_PROVIDER_OBJECT_REVISION));
        assert!(note.contains(X86_SYSTEMS_PROVIDER_REVISION));
        assert!(note.contains("target=x86_64-unknown-none"));
        assert!(note.contains("machine-cpu=qemu64-v1"));
        assert!(!note.contains("linux-gnu"));
    }

    #[test]
    fn emits_only_the_sealed_provider_instruction_sequences() {
        // TOPAL-SYSTEMS-MACHINE-001, TOPAL-SYSTEMS-DISPOSITION-001.
        let generated = generated();
        let file = object::File::parse(generated.bytes.as_slice()).unwrap();
        for (name, expected) in [
            (
                "topal_x86_systems_uart16550_write",
                &[
                    0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20, 0x74, 0xfb, 0x66, 0xba, 0xf8, 0x03,
                    0x40, 0x88, 0xf8, 0xee, 0xc3,
                ][..],
            ),
            ("topal_x86_systems_debug_break", &[0xcc, 0xc3]),
            ("topal_x86_systems_interrupt_return", &[0x48, 0xcf]),
            ("topal_x86_systems_fatal", &[0xfa, 0xf4, 0xeb, 0xfd]),
        ] {
            let symbol = file.symbol_by_name(name).unwrap();
            assert_eq!(symbol.size(), expected.len() as u64);
            let section = file
                .section_by_index(symbol.section_index().unwrap())
                .unwrap();
            let data = section.data().unwrap();
            let start = usize::try_from(symbol.address()).unwrap();
            assert_eq!(&data[start..start + expected.len()], expected);
        }
        let validator = file.symbol_by_name(X86_SYSTEMS_BOOT_MEMORY_SYMBOL).unwrap();
        assert!(validator.size() > 100);
        let section = file
            .section_by_index(validator.section_index().unwrap())
            .unwrap();
        let data = section.data().unwrap();
        let start = usize::try_from(validator.address()).unwrap();
        let end = start + usize::try_from(validator.size()).unwrap();
        assert_eq!(&data[end - 3..end], &[0x31, 0xc0, 0xc3]);
        let selector = file
            .symbol_by_name(X86_SYSTEMS_FRAME_ALLOCATE_SYMBOL)
            .unwrap();
        assert_eq!(selector.size(), validator.size());
        let selector_start = usize::try_from(selector.address()).unwrap();
        let selector_end = selector_start + usize::try_from(selector.size()).unwrap();
        assert_eq!(&data[selector_start..selector_end], &data[start..end]);
    }
}
