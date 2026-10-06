//! Generated ELF object for the sealed initial x86-64 systems provider.

use object::write::{Object, Symbol, SymbolSection};
use object::{
    Architecture, BinaryFormat, Endianness, SectionKind, SymbolFlags, SymbolKind, SymbolScope,
};
use topal_language::compiler::CompilerSystemsProgram;

use crate::{
    CompileError, X86_SYSTEMS_PROVIDER_REVISION, X86SystemsProviderPlan,
    plan_x86_64_systems_provider,
};

pub const X86_SYSTEMS_PROVIDER_OBJECT_REVISION: &str = "topal.provider-object.x86_64-qemu-pc-q35/1";
pub const X86_SYSTEMS_PROVIDER_TEXT_SECTION: &str = ".text.topal.systems.provider";
pub const X86_SYSTEMS_BOOTSTRAP_STORAGE_SECTION: &str = ".bss.topal.bootstrap";
pub const X86_SYSTEMS_PROVIDER_NOTE_SECTION: &str = ".note.topal.provider";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum X86Instruction {
    MovePortArgumentToDx,
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
            Self::MovePortArgumentToDx => output.extend_from_slice(&[0x66, 0x89, 0xfa]),
            Self::MoveByteArgumentToAl => output.extend_from_slice(&[0x40, 0x88, 0xf0]),
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
        "topal_x86_systems_out8",
        &[
            X86Instruction::MovePortArgumentToDx,
            X86Instruction::MoveByteArgumentToAl,
            X86Instruction::OutputByteToPort,
            X86Instruction::Return,
        ],
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
                "topal_x86_systems_out8",
                &[0x66, 0x89, 0xfa, 0x40, 0x88, 0xf0, 0xee, 0xc3][..],
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
    }
}
