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

pub const X86_SYSTEMS_PROVIDER_OBJECT_REVISION: &str =
    "topal.provider-object.x86_64-qemu-pc-q35/12";
pub const X86_SYSTEMS_PROVIDER_TEXT_SECTION: &str = ".text.topal.systems.provider";
pub const X86_SYSTEMS_BOOTSTRAP_STORAGE_SECTION: &str = ".bss.topal.bootstrap";
pub const X86_SYSTEMS_PROVIDER_NOTE_SECTION: &str = ".note.topal.provider";
pub const X86_SYSTEMS_BOOT_MEMORY_SYMBOL: &str = "topal_x86_systems_describe_boot_memory";
pub const X86_SYSTEMS_FRAME_ALLOCATE_SYMBOL: &str = "topal_x86_systems_allocate_physical_frames";
pub const X86_SYSTEMS_TRANSLATION_BEGIN_SYMBOL: &str =
    "topal_x86_systems_begin_bootstrap_translation";
pub const X86_SYSTEMS_TRANSLATION_COMMIT_SYMBOL: &str =
    "topal_x86_systems_commit_bootstrap_translation";
pub const X86_SYSTEMS_TRANSLATION_ACTIVATE_SYMBOL: &str =
    "topal_x86_systems_activate_bootstrap_translation";
pub const X86_SYSTEMS_TRANSLATION_EDIT_BEGIN_SYMBOL: &str =
    "topal_x86_systems_begin_active_translation_edit";
pub const X86_SYSTEMS_TRANSLATION_EDIT_MAP_SYMBOL: &str =
    "topal_x86_systems_stage_active_translation_map";
pub const X86_SYSTEMS_TRANSLATION_EDIT_UNMAP_SYMBOL: &str =
    "topal_x86_systems_stage_active_translation_unmap";
pub const X86_SYSTEMS_TRANSLATION_EDIT_COMMIT_SYMBOL: &str =
    "topal_x86_systems_commit_active_translation_edit";
pub const X86_SYSTEMS_CRITICAL_ENTER_SYMBOL: &str =
    "topal_x86_systems_enter_local_interrupt_critical";
pub const X86_SYSTEMS_CRITICAL_RESTORE_SYMBOL: &str =
    "topal_x86_systems_restore_local_interrupt_critical";
pub const X86_SYSTEMS_ATOMIC_CREATE_SYMBOL: &str = "topal_x86_systems_atomic_word_create";
pub const X86_SYSTEMS_ATOMIC_COMPARE_EXCHANGE_SYMBOL: &str =
    "topal_x86_systems_atomic_word_compare_exchange";
pub const X86_SYSTEMS_ATOMIC_LOAD_SYMBOL: &str = "topal_x86_systems_atomic_word_load_acquire";
pub const X86_SYSTEMS_LOCAL_NOTIFICATION_SEND_SYMBOL: &str =
    "topal_x86_systems_local_notification_send";
pub const X86_SYSTEMS_LOCAL_NOTIFICATION_WAIT_SYMBOL: &str =
    "topal_x86_systems_local_notification_wait";
pub const X86_SYSTEMS_LOCAL_NOTIFICATION_COMPLETE_SYMBOL: &str =
    "topal_x86_systems_local_notification_complete";
pub const X86_SYSTEMS_MONOTONIC_CLOCK_NOW_SYMBOL: &str = "topal_x86_systems_monotonic_clock_now";
pub const X86_SYSTEMS_DEADLINE_AFTER_SYMBOL: &str = "topal_x86_systems_deadline_after";
pub const X86_SYSTEMS_DEADLINE_ARM_SYMBOL: &str = "topal_x86_systems_deadline_arm";
pub const X86_SYSTEMS_DEADLINE_WAIT_SYMBOL: &str = "topal_x86_systems_deadline_wait";
pub const X86_SYSTEMS_DEADLINE_COMPLETE_SYMBOL: &str = "topal_x86_systems_deadline_complete";
pub const X86_SYSTEMS_CONTEXT_CREATE_SYMBOL: &str = "topal_x86_systems_context_create";
pub const X86_SYSTEMS_CONTEXT_TRANSFER_SYMBOL: &str = "topal_x86_systems_context_transfer";
pub const X86_SYSTEMS_CONTEXT_RETIRE_SYMBOL: &str = "topal_x86_systems_context_retire";
pub const X86_SYSTEMS_CONTEXT_RECLAIM_SYMBOL: &str = "topal_x86_systems_context_reclaim";
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
        &physical_frame_selector()?,
    );
    append_translation_functions(&mut object, text)?;
    append_critical_functions(&mut object, text);
    append_atomic_functions(&mut object, text);
    append_local_notification_functions(&mut object, text);
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_MONOTONIC_CLOCK_NOW_SYMBOL,
        &monotonic_clock_now()?,
    );
    append_deadline_functions(&mut object, text)?;
    append_context_functions(&mut object, text)?;
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

fn append_deadline_functions(
    object: &mut Object<'_>,
    text: object::write::SectionId,
) -> Result<(), CompileError> {
    for (name, encoded) in [
        (X86_SYSTEMS_DEADLINE_AFTER_SYMBOL, deadline_after()?),
        (X86_SYSTEMS_DEADLINE_ARM_SYMBOL, deadline_arm()?),
        (X86_SYSTEMS_DEADLINE_WAIT_SYMBOL, deadline_wait()),
        (X86_SYSTEMS_DEADLINE_COMPLETE_SYMBOL, deadline_complete()?),
    ] {
        append_encoded_function(object, text, name, &encoded);
    }
    Ok(())
}

fn append_context_functions(
    object: &mut Object<'_>,
    text: object::write::SectionId,
) -> Result<(), CompileError> {
    for (name, encoded) in [
        (X86_SYSTEMS_CONTEXT_CREATE_SYMBOL, context_create()?),
        (X86_SYSTEMS_CONTEXT_TRANSFER_SYMBOL, context_transfer()?),
        (X86_SYSTEMS_CONTEXT_RETIRE_SYMBOL, context_retire()?),
        (X86_SYSTEMS_CONTEXT_RECLAIM_SYMBOL, context_reclaim()?),
    ] {
        append_encoded_function(object, text, name, &encoded);
    }
    Ok(())
}

fn context_create() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]); // provider-private context state
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // stack base
    code.jump_if(0x84, "fail");
    code.bytes(&[0x40, 0xf6, 0xc6, 0x0f]); // 16-byte aligned stack
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xfa, 0x00, 0x40, 0x00, 0x00]); // 16 KiB
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x85, 0xc9]); // typed entry address
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x7f, 0x18, 0x00]); // fresh state
    code.jump_if(0x85, "fail");
    code.bytes(&[0x9c, 0x58, 0xa9, 0x00, 0x02, 0x00, 0x00]); // require IF=0
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x89, 0x77, 0x10]); // retain stack base
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x01, 0xd0]); // stack end
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x83, 0xe8, 0x40]); // initial saved RSP
    code.bytes(&[0x45, 0x31, 0xc0]); // zero callee-saved register images
    for displacement in [0_u8, 8, 16, 24, 32, 40] {
        code.bytes(&[0x4c, 0x89, 0x40, displacement]);
    }
    code.bytes(&[0x48, 0x89, 0x48, 0x30]); // synthetic return enters worker
    code.bytes(&[0x48, 0x89, 0x47, 0x08]); // suspended worker RSP
    code.bytes(&[0xc6, 0x47, 0x18, 0x01]); // suspended
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn context_transfer() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x7f, 0x18, 0x01]); // suspended
    code.jump_if(0x85, "fail");
    code.bytes(&[0x9c, 0x58, 0xa9, 0x00, 0x02, 0x00, 0x00]); // require IF=0
    code.jump_if(0x85, "fail");
    code.bytes(&[0xc6, 0x47, 0x18, 0x02]); // active
    code.bytes(&[0x53, 0x55, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57]);
    code.bytes(&[0x48, 0x89, 0x27]); // suspend caller RSP
    code.bytes(&[0x48, 0x8b, 0x67, 0x08]); // select worker RSP
    code.bytes(&[
        0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x41, 0x5c, 0x5d, 0x5b, 0xc3,
    ]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn context_retire() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x7f, 0x18, 0x02]); // active
    code.jump_if(0x85, "fail");
    code.bytes(&[0x9c, 0x58, 0xa9, 0x00, 0x02, 0x00, 0x00]); // require IF=0
    code.jump_if(0x85, "fail");
    code.bytes(&[0xc6, 0x47, 0x18, 0x03]); // completed
    code.bytes(&[0x48, 0x8b, 0x27]); // resume exact caller RSP
    code.bytes(&[0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x41, 0x5c, 0x5d, 0x5b]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn context_reclaim() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x7f, 0x18, 0x03]); // completed
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0xc7, 0x07, 0, 0, 0, 0]); // consume caller RSP
    code.bytes(&[0x48, 0xc7, 0x47, 0x08, 0, 0, 0, 0]); // consume worker RSP
    code.bytes(&[0xc6, 0x47, 0x18, 0x04]); // reclaimed
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn append_critical_functions(object: &mut Object<'_>, text: object::write::SectionId) {
    append_encoded_function(
        object,
        text,
        X86_SYSTEMS_CRITICAL_ENTER_SYMBOL,
        &critical_enter(),
    );
    append_encoded_function(
        object,
        text,
        X86_SYSTEMS_CRITICAL_RESTORE_SYMBOL,
        &critical_restore(),
    );
}

fn append_atomic_functions(object: &mut Object<'_>, text: object::write::SectionId) {
    append_encoded_function(
        object,
        text,
        X86_SYSTEMS_ATOMIC_CREATE_SYMBOL,
        &atomic_word_create(),
    );
    append_encoded_function(
        object,
        text,
        X86_SYSTEMS_ATOMIC_COMPARE_EXCHANGE_SYMBOL,
        &atomic_word_compare_exchange(),
    );
    append_encoded_function(
        object,
        text,
        X86_SYSTEMS_ATOMIC_LOAD_SYMBOL,
        &atomic_word_load_acquire(),
    );
}

fn append_local_notification_functions(object: &mut Object<'_>, text: object::write::SectionId) {
    append_encoded_function(
        object,
        text,
        X86_SYSTEMS_LOCAL_NOTIFICATION_SEND_SYMBOL,
        &local_notification_send(),
    );
    append_encoded_function(
        object,
        text,
        X86_SYSTEMS_LOCAL_NOTIFICATION_WAIT_SYMBOL,
        &local_notification_wait(),
    );
    append_encoded_function(
        object,
        text,
        X86_SYSTEMS_LOCAL_NOTIFICATION_COMPLETE_SYMBOL,
        &local_notification_complete(),
    );
}

fn local_notification_send() -> Vec<u8> {
    let mut code = Vec::new();
    code.extend_from_slice(&[0x9c, 0x58]); // pushfq; pop rax
    code.extend_from_slice(&[0xa9, 0x00, 0x02, 0x00, 0x00]); // test eax, IF
    code.extend_from_slice(&[0x75, 0x41]); // initial profile requires IF clear
    code.extend_from_slice(&[0xc6, 0x07, 0x00]); // completion flag = 0
    code.extend_from_slice(&[0xb9, 0x1b, 0x00, 0x00, 0x00]); // IA32_APIC_BASE
    code.extend_from_slice(&[0x0f, 0x32]); // rdmsr
    code.extend_from_slice(&[0xa9, 0x00, 0x04, 0x00, 0x00]); // reject x2APIC mode
    code.extend_from_slice(&[0x75, 0x30]);
    code.extend_from_slice(&[0x0d, 0x00, 0x08, 0x00, 0x00]); // global APIC enable
    code.extend_from_slice(&[0x0f, 0x30]); // wrmsr
    code.extend_from_slice(&[0x48, 0xba]); // mov rdx, local APIC SVR
    code.extend_from_slice(&0xfee0_00f0_u64.to_le_bytes());
    code.extend_from_slice(&[0x8b, 0x02]); // mov eax, [rdx]
    code.extend_from_slice(&[0x0d, 0x00, 0x01, 0x00, 0x00]); // software enable
    code.extend_from_slice(&[0x89, 0x02]); // mov [rdx], eax
    code.extend_from_slice(&[0x48, 0xba]); // mov rdx, local APIC ICR low
    code.extend_from_slice(&0xfee0_0300_u64.to_le_bytes());
    code.extend_from_slice(&[0xc7, 0x02]); // fixed, assert, edge, self shorthand
    code.extend_from_slice(&0x0004_40f1_u32.to_le_bytes());
    code.extend_from_slice(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.extend_from_slice(&[0x31, 0xc0, 0xc3]); // fail
    code
}

fn local_notification_wait() -> Vec<u8> {
    vec![
        0xfb, // sti; the interrupt shadow covers the following hlt
        0xf4, // hlt until an admitted external interrupt
        0xfa, // restore the sealed prior disabled state
        0x80, 0x3f, 0x01, // cmp byte ptr [rdi], 1
        0x75, 0xf8, // wait again after an unrelated wake
        0xc6, 0x07, 0x00, // consume completion flag
        0xb8, 0x01, 0x00, 0x00, 0x00, // success
        0xc3,
    ]
}

fn local_notification_complete() -> Vec<u8> {
    let mut code = Vec::new();
    code.extend_from_slice(&[0x48, 0xb8]); // mov rax, local APIC EOI
    code.extend_from_slice(&0xfee0_00b0_u64.to_le_bytes());
    code.extend_from_slice(&[0xc7, 0x00, 0x00, 0x00, 0x00, 0x00]); // EOI = 0
    code.extend_from_slice(&[0xc6, 0x07, 0x01, 0xc3]); // completion flag = 1; ret
    code
}

fn monotonic_clock_now() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]); // test provider-private state address
    code.jump_if(0x84, "fail");
    code.bytes(&[0x40, 0xf6, 0xc7, 0x07]); // test dil, 7
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0xb8]); // r8 = HPET MMIO base
    code.bytes(&0x0000_0000_fed0_0000_u64.to_le_bytes());
    code.bytes(&[0x49, 0x8b, 0x00]); // general capabilities
    code.bytes(&[0xa9, 0x00, 0x20, 0x00, 0x00]); // require a 64-bit main counter
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x89, 0xc2, 0x48, 0xc1, 0xea, 0x20]);
    code.bytes(&[0x81, 0xfa, 0x80, 0x96, 0x98, 0x00]); // Q35 HPET: 10 ns period
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x8b, 0x40, 0x10]); // general configuration
    code.bytes(&[0x48, 0x83, 0xc8, 0x01]); // enable main counter
    code.bytes(&[0x49, 0x89, 0x40, 0x10]);
    code.bytes(&[0x49, 0x8b, 0x80, 0xf0, 0x00, 0x00, 0x00]); // raw counter
    code.bytes(&[0x80, 0x7f, 0x10, 0x00]); // initialized?
    code.jump_if(0x85, "subsequent");
    code.bytes(&[0x48, 0x89, 0x07]); // retain raw low word
    code.bytes(&[0x31, 0xd2]); // first epoch is zero
    code.bytes(&[0x48, 0x89, 0x57, 0x08]);
    code.bytes(&[0xc6, 0x47, 0x10, 0x01]);
    code.jump("success");

    code.bind("subsequent")?;
    code.bytes(&[0x48, 0x8b, 0x57, 0x08]); // retained wrap epoch
    code.bytes(&[0x48, 0x3b, 0x07]); // unsigned raw >= previous raw
    code.jump_if(0x83, "retain");
    code.bytes(&[0x4c, 0x8b, 0x0f]); // a decrease is only a qualified boundary wrap
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0xc000_0000_0000_0000_u64.to_le_bytes());
    code.bytes(&[0x4d, 0x39, 0xd1]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x4000_0000_0000_0000_u64.to_le_bytes());
    code.bytes(&[0x4c, 0x39, 0xd0]);
    code.jump_if(0x83, "fail");
    code.bytes(&[0x48, 0x83, 0xc2, 0x01]); // extend a 64-bit wrap
    code.jump_if(0x82, "fail");
    code.bind("retain")?;
    code.bytes(&[0x48, 0x89, 0x07, 0x48, 0x89, 0x57, 0x08]);
    code.bind("success")?;
    code.bytes(&[0xb9, 0x01, 0x00, 0x00, 0x00, 0xc3]); // rcx = success
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc9, 0xc3]);
    code.finish()
}

fn deadline_after() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]); // private clock/deadline state
    code.jump_if(0x84, "fail");
    code.bytes(&[0x40, 0xf6, 0xc7, 0x07]); // 8-byte aligned
    code.jump_if(0x85, "fail");
    code.bytes(&[0x80, 0x7f, 0x10, 0x01]); // clock initialized
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x8b, 0x07]); // last accepted raw counter
    code.bytes(&[0x48, 0x8b, 0x57, 0x08]); // wrap epoch
    code.bytes(&[0x48, 0x05]);
    code.bytes(&100_000_u32.to_le_bytes()); // 1 ms at the validated 10 ns period
    code.bytes(&[0x48, 0x83, 0xd2, 0x00]); // extend carry into epoch
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x89, 0x47, 0x18]); // scheduled raw
    code.bytes(&[0x48, 0x89, 0x57, 0x20]); // scheduled epoch
    code.bytes(&[0xc6, 0x47, 0x28, 0x01]); // constructed
    code.bytes(&[0xc6, 0x47, 0x29, 0x00]); // not completed
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn deadline_arm() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x40, 0xf6, 0xc7, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x80, 0x7f, 0x28, 0x01]); // exactly one constructed event
    code.jump_if(0x85, "fail");
    code.bytes(&[0x9c, 0x58]); // require the sealed prior IF=0 state
    code.bytes(&[0xa9, 0x00, 0x02, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0xb0, 0xff, 0xe6, 0x21, 0xe6, 0xa1]); // mask both legacy PICs
    code.bytes(&[0x49, 0xb8]);
    code.bytes(&0x0000_0000_fed0_0000_u64.to_le_bytes()); // HPET
    code.bytes(&[0x49, 0x8b, 0x00]); // capabilities
    code.bytes(&[0xa9, 0x00, 0x20, 0x00, 0x00]); // 64-bit main counter
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x89, 0xc2, 0x48, 0xc1, 0xea, 0x20]);
    code.bytes(&[0x81, 0xfa, 0x80, 0x96, 0x98, 0x00]); // 10 ns period
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x8b, 0x90, 0x00, 0x01, 0x00, 0x00]); // timer 0 config/cap
    code.bytes(&[0x48, 0xc1, 0xea, 0x20]);
    code.bytes(&[0xf7, 0xc2, 0x04, 0x00, 0x00, 0x00]); // route 2 capability
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x8b, 0x47, 0x20]); // scheduled epoch
    code.bytes(&[0x48, 0x3b, 0x47, 0x08]); // same current epoch in initial slice
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x8b, 0x88, 0xf0, 0x00, 0x00, 0x00]); // current raw
    code.bytes(&[0x48, 0x8b, 0x47, 0x18]); // scheduled raw
    code.bytes(&[0x48, 0x39, 0xc1]);
    code.jump_if(0x83, "immediate"); // current >= scheduled

    code.bytes(&[0x49, 0xb9]);
    code.bytes(&0x0000_0000_fec0_0000_u64.to_le_bytes()); // I/O APIC
    code.bytes(&[0x41, 0xc7, 0x01, 0x14, 0x00, 0x00, 0x00]); // redirection low 2
    code.bytes(&[0x41, 0xc7, 0x41, 0x10]);
    code.bytes(&0x0001_00f2_u32.to_le_bytes()); // masked, fixed vector f2
    code.bytes(&[0x41, 0xc7, 0x01, 0x15, 0x00, 0x00, 0x00]); // redirection high 2
    code.bytes(&[0x41, 0xc7, 0x41, 0x10, 0x00, 0x00, 0x00, 0x00]); // APIC ID 0
    code.bytes(&[0x41, 0xc7, 0x80, 0x00, 0x01, 0x00, 0x00]);
    code.bytes(&0x0000_0400_u32.to_le_bytes()); // edge, one-shot, route 2, disabled
    code.bytes(&[0x49, 0x89, 0x80, 0x08, 0x01, 0x00, 0x00]); // comparator
    code.bytes(&[0x49, 0x8b, 0x50, 0x10]);
    code.bytes(&[0x48, 0x83, 0xca, 0x01]); // enable HPET globally
    code.bytes(&[0x49, 0x89, 0x50, 0x10]);
    code.bytes(&[0x41, 0xc7, 0x01, 0x14, 0x00, 0x00, 0x00]);
    code.bytes(&[0x41, 0xc7, 0x41, 0x10]);
    code.bytes(&0x0000_00f2_u32.to_le_bytes()); // unmask route
    code.bytes(&[0x41, 0xc7, 0x80, 0x00, 0x01, 0x00, 0x00]);
    code.bytes(&0x0000_0404_u32.to_le_bytes()); // enable timer interrupt
    code.jump("armed");

    code.bind("immediate")?;
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0000_0000_fee0_0300_u64.to_le_bytes()); // local APIC ICR low
    code.bytes(&[0xc7, 0x00]);
    code.bytes(&0x0004_40f2_u32.to_le_bytes()); // fixed self notification

    code.bind("armed")?;
    code.bytes(&[0xc6, 0x47, 0x29, 0x00]);
    code.bytes(&[0xc6, 0x47, 0x28, 0x02]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn deadline_wait() -> Vec<u8> {
    vec![
        0xfb, 0xf4, 0xfa, // sti; hlt; restore sealed prior IF=0 state
        0x80, 0x7f, 0x29, 0x01, // completion flag
        0x75, 0xf7, // unrelated wake: wait again
        0xc6, 0x47, 0x29, 0x00, // consume completion flag
        0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3,
    ]
}

fn deadline_complete() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x7f, 0x28, 0x02]); // armed
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0xb8]);
    code.bytes(&0x0000_0000_fed0_0000_u64.to_le_bytes());
    code.bytes(&[0x49, 0x8b, 0x80, 0xf0, 0x00, 0x00, 0x00]); // observed raw
    code.bytes(&[0x48, 0x3b, 0x47, 0x18]); // no earlier than scheduled
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x89, 0x47, 0x30]); // retain observed raw
    code.bytes(&[0x41, 0x81, 0xa0, 0x00, 0x01, 0x00, 0x00]);
    code.bytes(&0xffff_fffb_u32.to_le_bytes()); // disable timer 0 interrupt
    code.bytes(&[0x41, 0xc7, 0x40, 0x20, 0x01, 0x00, 0x00, 0x00]); // clear status
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0000_0000_fee0_00b0_u64.to_le_bytes());
    code.bytes(&[0xc7, 0x00, 0x00, 0x00, 0x00, 0x00]); // local APIC EOI
    code.bytes(&[0xc6, 0x47, 0x28, 0x03]); // completed
    code.bytes(&[0xc6, 0x47, 0x29, 0x01]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn atomic_word_create() -> Vec<u8> {
    vec![
        0x48, 0xf7, 0xc7, 0x07, 0x00, 0x00, 0x00, // test rdi, 7
        0x75, 0x09, // jnz failure
        0x48, 0x89, 0x37, // mov [rdi], rsi
        0xb8, 0x01, 0x00, 0x00, 0x00, // mov eax, 1
        0xc3, // ret
        0x31, 0xc0, // failure: xor eax, eax
        0xc3, // ret
    ]
}

fn atomic_word_compare_exchange() -> Vec<u8> {
    vec![
        0x48, 0xf7, 0xc7, 0x07, 0x00, 0x00, 0x00, // test rdi, 7
        0x75, 0x0f, // jnz failure
        0x48, 0x89, 0xf0, // mov rax, rsi
        0xf0, 0x48, 0x0f, 0xb1, 0x17, // lock cmpxchg [rdi], rdx
        0x0f, 0x94, 0xc0, // sete al
        0x0f, 0xb6, 0xc0, // movzx eax, al
        0xc3, // ret
        0x31, 0xc0, // failure: xor eax, eax
        0xc3, // ret
    ]
}

fn atomic_word_load_acquire() -> Vec<u8> {
    vec![
        0x48, 0xf7, 0xc7, 0x07, 0x00, 0x00, 0x00, // test rdi, 7
        0x75, 0x04, // jnz failure
        0x48, 0x8b, 0x07, // mov rax, [rdi]
        0xc3, // ret
        0x31, 0xc0, // failure: xor eax, eax
        0xc3, // ret
    ]
}

fn critical_enter() -> Vec<u8> {
    vec![
        0x9c, // pushfq
        0x58, // pop rax -- opaque exact prior flags token
        0xfa, // cli
        0xc3, // ret
    ]
}

fn critical_restore() -> Vec<u8> {
    vec![
        0x40, 0xf6, 0xc7, 0x02, // test dil, 2 -- validate architectural fixed bit
        0x74, 0x16, // jz failure
        0xf7, 0xc7, 0x00, 0x02, 0x00, 0x00, // test edi, RFLAGS.IF
        0x74, 0x07, // jz restore-disabled
        0xfb, // sti
        0xb8, 0x01, 0x00, 0x00, 0x00, // mov eax, 1
        0xc3, // ret
        0xfa, // restore-disabled: cli
        0xb8, 0x01, 0x00, 0x00, 0x00, // mov eax, 1
        0xc3, // ret
        0x31, 0xc0, // failure: xor eax, eax
        0xc3, // ret
    ]
}

fn append_translation_functions(
    object: &mut Object<'_>,
    text: object::write::SectionId,
) -> Result<(), CompileError> {
    for (name, encoded) in [
        (
            X86_SYSTEMS_TRANSLATION_BEGIN_SYMBOL,
            translation_backing_selector()?,
        ),
        (
            X86_SYSTEMS_TRANSLATION_COMMIT_SYMBOL,
            translation_space_builder()?,
        ),
        (
            X86_SYSTEMS_TRANSLATION_ACTIVATE_SYMBOL,
            translation_space_activator()?,
        ),
        (
            X86_SYSTEMS_TRANSLATION_EDIT_BEGIN_SYMBOL,
            translation_edit_begin()?,
        ),
        (
            X86_SYSTEMS_TRANSLATION_EDIT_MAP_SYMBOL,
            translation_edit_map()?,
        ),
        (
            X86_SYSTEMS_TRANSLATION_EDIT_UNMAP_SYMBOL,
            translation_edit_unmap()?,
        ),
        (
            X86_SYSTEMS_TRANSLATION_EDIT_COMMIT_SYMBOL,
            translation_edit_commit()?,
        ),
    ] {
        append_encoded_function(object, text, name, &encoded);
    }
    Ok(())
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

fn physical_frame_selector() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]); // test private allocation floor
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xff, 0x00, 0x00, 0x00, 0x01]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // test rsi, rsi
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0xf0, 0xff, 0x00]); // cmp rsi, 0x00fff000
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x83, 0xbe, 0x50, 0x02, 0x00, 0x00, 0x00]); // setup_data == 0
    code.jump_if(0x85, "fail");
    code.bytes(&[0x0f, 0xb6, 0x8e, 0xe8, 0x01, 0x00, 0x00]); // e820_entries
    code.bytes(&[0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x81, 0xf9, 0x80, 0x00, 0x00, 0x00]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x8d, 0x96, 0xd0, 0x02, 0x00, 0x00]); // first E820 entry

    code.bind("loop")?;
    code.bytes(&[0x48, 0x8b, 0x02]); // candidate = base
    code.bytes(&[0x4c, 0x8b, 0x4a, 0x08]); // length
    code.bytes(&[0x44, 0x8b, 0x52, 0x10]); // type
    code.bytes(&[0x4d, 0x85, 0xc9]);
    code.jump_if(0x84, "zero-length");
    code.bytes(&[0x49, 0x89, 0xc3]); // end = base
    code.bytes(&[0x4d, 0x01, 0xcb]); // end += length
    code.jump_if(0x82, "fail");
    code.bytes(&[0x41, 0x83, 0xfa, 0x01]); // E820 RAM
    code.jump_if(0x85, "next");
    code.bytes(&[0x48, 0x39, 0xf8]); // base >= private floor
    code.jump_if(0x83, "base-ready");
    code.bytes(&[0x48, 0x89, 0xf8]); // candidate = private floor
    code.bind("base-ready")?;
    code.bytes(&[0x48, 0x05, 0xff, 0x0f, 0x00, 0x00]); // align candidate up
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x25, 0x00, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x49, 0x81, 0xe3, 0x00, 0xf0, 0xff, 0xff]); // align end down
    code.bytes(&[0x4c, 0x39, 0xd8]); // candidate < end
    code.jump_if(0x83, "next");
    code.bytes(&[0x48, 0x3d, 0x00, 0xf0, 0xff, 0x3f]); // identity-mapped page <= 1 GiB
    code.jump_if(0x87, "next");
    code.bytes(&[0xc3]); // return physical base in rax

    code.bind("zero-length")?;
    code.bytes(&[0x41, 0x83, 0xfa, 0x01]);
    code.jump_if(0x84, "fail");

    code.bind("next")?;
    code.bytes(&[0x48, 0x83, 0xc2, 0x14]);
    code.bytes(&[0xff, 0xc9]);
    code.jump_if(0x85, "loop");

    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn translation_backing_selector() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xf6]); // test boot_params, boot_params
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0xf0, 0xff, 0x00]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x83, 0xbe, 0x50, 0x02, 0x00, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x0f, 0xb6, 0x8e, 0xe8, 0x01, 0x00, 0x00]);
    code.bytes(&[0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x81, 0xf9, 0x80, 0x00, 0x00, 0x00]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x8d, 0x96, 0xd0, 0x02, 0x00, 0x00]);

    code.bind("loop")?;
    code.bytes(&[0x48, 0x8b, 0x02]); // candidate = base
    code.bytes(&[0x4c, 0x8b, 0x4a, 0x08]); // length
    code.bytes(&[0x44, 0x8b, 0x52, 0x10]); // type
    code.bytes(&[0x4d, 0x85, 0xc9]);
    code.jump_if(0x84, "zero-length");
    code.bytes(&[0x49, 0x89, 0xc3]); // end = base
    code.bytes(&[0x4d, 0x01, 0xcb]); // end += length
    code.jump_if(0x82, "fail");
    code.bytes(&[0x41, 0x83, 0xfa, 0x01]);
    code.jump_if(0x85, "next");
    code.bytes(&[0x48, 0x3d, 0x00, 0x00, 0x00, 0x01]);
    code.jump_if(0x83, "base-ready");
    code.bytes(&[0xb8, 0x00, 0x00, 0x00, 0x01]);
    code.bind("base-ready")?;
    code.bytes(&[0x48, 0x05, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x25, 0x00, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x49, 0x81, 0xe3, 0x00, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x49, 0x89, 0xc2]); // end of 4 pages = candidate
    code.bytes(&[0x49, 0x81, 0xc2, 0x00, 0x40, 0x00, 0x00]);
    code.jump_if(0x82, "next");
    code.bytes(&[0x4d, 0x39, 0xda]); // candidate + 4 pages <= RAM end
    code.jump_if(0x87, "next");
    code.bytes(&[0x48, 0x3d, 0x00, 0xc0, 0xff, 0x3f]); // all backing below 1 GiB
    code.jump_if(0x87, "next");
    code.bytes(&[0xc3]);

    code.bind("zero-length")?;
    code.bytes(&[0x41, 0x83, 0xfa, 0x01]);
    code.jump_if(0x84, "fail");
    code.bind("next")?;
    code.bytes(&[0x48, 0x83, 0xc2, 0x14]);
    code.bytes(&[0xff, 0xc9]);
    code.jump_if(0x85, "loop");
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn translation_space_builder() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x49, 0x89, 0xf8]); // r8 = root
    code.bytes(&[0x31, 0xc0]); // zero value
    code.bytes(&[0xb9, 0x00, 0x08, 0x00, 0x00]); // 2048 qwords
    code.bytes(&[0xfc, 0xf3, 0x48, 0xab]); // cld; rep stosq
    code.bytes(&[0x4d, 0x8d, 0x88, 0x00, 0x10, 0x00, 0x00]); // pdpt
    code.bytes(&[0x4c, 0x89, 0xc8, 0x48, 0x83, 0xc8, 0x03]);
    code.bytes(&[0x49, 0x89, 0x00]); // pml4[0]
    code.bytes(&[0x4d, 0x8d, 0x88, 0x00, 0x20, 0x00, 0x00]); // pd
    code.bytes(&[0x4c, 0x89, 0xc8, 0x48, 0x83, 0xc8, 0x03]);
    code.bytes(&[0x49, 0x89, 0x80, 0x00, 0x10, 0x00, 0x00]); // pdpt[0]
    code.bytes(&[0x4d, 0x8d, 0x90, 0x00, 0x30, 0x00, 0x00]); // local APIC PD
    code.bytes(&[0x4c, 0x89, 0xd0, 0x48, 0x83, 0xc8, 0x03]);
    code.bytes(&[0x49, 0x89, 0x80, 0x18, 0x10, 0x00, 0x00]); // pdpt[3]
    code.bytes(&[0x49, 0x8d, 0xb8, 0x00, 0x20, 0x00, 0x00]);
    code.bytes(&[0x31, 0xc9]);
    code.bind("leaves")?;
    code.bytes(&[0x48, 0x89, 0xc8, 0x48, 0xc1, 0xe0, 0x15]);
    code.bytes(&[0x48, 0x0d, 0x83, 0x00, 0x00, 0x00]); // present/write/2MiB
    code.bytes(&[0x48, 0x89, 0x04, 0xcf]);
    code.bytes(&[0xff, 0xc1, 0x81, 0xf9, 0x00, 0x02, 0x00, 0x00]);
    code.jump_if(0x85, "leaves");
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0000_0000_fec0_009b_u64.to_le_bytes()); // HPET 2 MiB MMIO leaf
    code.bytes(&[0x49, 0x89, 0x82, 0xb0, 0x0f, 0x00, 0x00]); // PDE[502]
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&0x0000_0000_fee0_009b_u64.to_le_bytes()); // APIC 2 MiB MMIO leaf
    code.bytes(&[0x49, 0x89, 0x82, 0xb8, 0x0f, 0x00, 0x00]); // PDE[503]
    code.bytes(&[0x0f, 0xae, 0xf0]); // mfence publishes the inactive hierarchy
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.finish()
}

fn translation_space_activator() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xff, 0x00, 0xd0, 0xff, 0x3f]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0xb9, 0x80, 0x00, 0x00, 0xc0]); // IA32_EFER
    code.bytes(&[0x0f, 0x32]); // rdmsr
    code.bytes(&[0x0d, 0x00, 0x08, 0x00, 0x00]); // enable NXE
    code.bytes(&[0x0f, 0x30]); // wrmsr
    code.bytes(&[0x0f, 0x22, 0xdf]); // mov cr3, rdi
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn translation_edit_begin() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x83, 0xfa, 0x01]); // unmap edit kind
    code.jump_if(0x84, "unmap");
    code.bytes(&[0x85, 0xd2]); // map edit kind is zero
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x85, 0xff]); // payload frame
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xc7, 0x00, 0x10, 0x00, 0x00]); // metadata floor
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // boot params
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0xf0, 0xff, 0x00]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x83, 0xbe, 0x50, 0x02, 0x00, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x0f, 0xb6, 0x8e, 0xe8, 0x01, 0x00, 0x00]);
    code.bytes(&[0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x81, 0xf9, 0x80, 0x00, 0x00, 0x00]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x8d, 0x96, 0xd0, 0x02, 0x00, 0x00]);

    code.bind("loop")?;
    code.bytes(&[0x48, 0x8b, 0x02]);
    code.bytes(&[0x4c, 0x8b, 0x4a, 0x08]);
    code.bytes(&[0x44, 0x8b, 0x52, 0x10]);
    code.bytes(&[0x4d, 0x85, 0xc9]);
    code.jump_if(0x84, "zero-length");
    code.bytes(&[0x49, 0x89, 0xc3]);
    code.bytes(&[0x4d, 0x01, 0xcb]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x41, 0x83, 0xfa, 0x01]);
    code.jump_if(0x85, "next");
    code.bytes(&[0x48, 0x39, 0xf8]);
    code.jump_if(0x83, "base-ready");
    code.bytes(&[0x48, 0x89, 0xf8]);
    code.bind("base-ready")?;
    code.bytes(&[0x48, 0x05, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x25, 0x00, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x49, 0x81, 0xe3, 0x00, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x49, 0x89, 0xc2]);
    code.bytes(&[0x49, 0x81, 0xc2, 0x00, 0x20, 0x00, 0x00]); // two pages
    code.jump_if(0x82, "next");
    code.bytes(&[0x4d, 0x39, 0xda]);
    code.jump_if(0x87, "next");
    code.bytes(&[0x48, 0x3d, 0x00, 0xe0, 0xff, 0x3f]);
    code.jump_if(0x87, "next");
    code.bytes(&[0xc3]);

    code.bind("zero-length")?;
    code.bytes(&[0x41, 0x83, 0xfa, 0x01]);
    code.jump_if(0x84, "fail");
    code.bind("next")?;
    code.bytes(&[0x48, 0x83, 0xc2, 0x14]);
    code.bytes(&[0xff, 0xc9]);
    code.jump_if(0x85, "loop");
    code.jump("fail");

    code.bind("unmap")?;
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x00, 0x00, 0x40]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn translation_edit_map() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc6, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x89, 0xf8]); // retain PD
    code.bytes(&[0x4c, 0x8d, 0x8f, 0x00, 0x10, 0x00, 0x00]); // PT
    code.bytes(&[0x31, 0xc0]);
    code.bytes(&[0xb9, 0x00, 0x04, 0x00, 0x00]);
    code.bytes(&[0xfc, 0xf3, 0x48, 0xab]); // clear two pages
    code.bytes(&[0x4c, 0x89, 0xc8, 0x48, 0x83, 0xc8, 0x03]);
    code.bytes(&[0x49, 0x89, 0x00]); // PD[0] -> PT
    code.bytes(&[0x48, 0x89, 0xf0]);
    code.bytes(&[0x48, 0xba, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]);
    code.bytes(&[0x48, 0x09, 0xd0]);
    code.bytes(&[0x49, 0x89, 0x01]); // PT[0] -> payload, writable and NX
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn translation_edit_unmap() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x00, 0x00, 0x40]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x83, 0xbf, 0x08, 0x10, 0x00, 0x00, 0x00]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn translation_edit_commit() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xfa, 0x01]);
    code.jump_if(0x84, "unmap");
    code.bytes(&[0x85, 0xd2]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc6, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x83, 0xc8, 0x03]);
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x48, 0x89, 0x87, 0x08, 0x10, 0x00, 0x00]); // publish PDPT[1]
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0xb8, 0x00, 0x00, 0x00, 0x40, 0xc3]); // opaque mapping VA

    code.bind("unmap")?;
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x00, 0x00, 0x40]);
    code.jump_if(0x85, "fail");
    code.bytes(&[
        0x48, 0xc7, 0x87, 0x08, 0x10, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ]);
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x0f, 0x01, 0x3e]); // invlpg [rsi]
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
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
    #[allow(clippy::too_many_lines)] // The complete sealed provider instruction audit remains co-located.
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
            (X86_SYSTEMS_CRITICAL_ENTER_SYMBOL, &[0x9c, 0x58, 0xfa, 0xc3]),
            (
                X86_SYSTEMS_CRITICAL_RESTORE_SYMBOL,
                &[
                    0x40, 0xf6, 0xc7, 0x02, 0x74, 0x16, 0xf7, 0xc7, 0x00, 0x02, 0x00, 0x00, 0x74,
                    0x07, 0xfb, 0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3, 0xfa, 0xb8, 0x01, 0x00, 0x00,
                    0x00, 0xc3, 0x31, 0xc0, 0xc3,
                ],
            ),
            (
                X86_SYSTEMS_ATOMIC_CREATE_SYMBOL,
                &[
                    0x48, 0xf7, 0xc7, 0x07, 0x00, 0x00, 0x00, 0x75, 0x09, 0x48, 0x89, 0x37, 0xb8,
                    0x01, 0x00, 0x00, 0x00, 0xc3, 0x31, 0xc0, 0xc3,
                ],
            ),
            (
                X86_SYSTEMS_ATOMIC_COMPARE_EXCHANGE_SYMBOL,
                &[
                    0x48, 0xf7, 0xc7, 0x07, 0x00, 0x00, 0x00, 0x75, 0x0f, 0x48, 0x89, 0xf0, 0xf0,
                    0x48, 0x0f, 0xb1, 0x17, 0x0f, 0x94, 0xc0, 0x0f, 0xb6, 0xc0, 0xc3, 0x31, 0xc0,
                    0xc3,
                ],
            ),
            (
                X86_SYSTEMS_ATOMIC_LOAD_SYMBOL,
                &[
                    0x48, 0xf7, 0xc7, 0x07, 0x00, 0x00, 0x00, 0x75, 0x04, 0x48, 0x8b, 0x07, 0xc3,
                    0x31, 0xc0, 0xc3,
                ],
            ),
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
        assert!(selector.size() > 100);
        let selector_start = usize::try_from(selector.address()).unwrap();
        let selector_end = selector_start + usize::try_from(selector.size()).unwrap();
        let selector = &data[selector_start..selector_end];
        assert_ne!(selector, &data[start..end]);
        assert!(
            selector
                .windows(6)
                .any(|bytes| bytes == [0x48, 0x3d, 0x00, 0xf0, 0xff, 0x3f]),
            "selector must constrain the returned page to bootstrap identity mappings"
        );
        assert!(
            selector.windows(3).any(|bytes| bytes == [0x48, 0x39, 0xf8]),
            "selector must apply its provider-private allocation floor"
        );
        assert_eq!(&selector[selector.len() - 3..], &[0x31, 0xc0, 0xc3]);

        let translation_selector = file
            .symbol_by_name(X86_SYSTEMS_TRANSLATION_BEGIN_SYMBOL)
            .unwrap();
        let translation_selector_start = usize::try_from(translation_selector.address()).unwrap();
        let translation_selector_end =
            translation_selector_start + usize::try_from(translation_selector.size()).unwrap();
        let translation_selector = &data[translation_selector_start..translation_selector_end];
        assert!(
            translation_selector
                .windows(7)
                .any(|bytes| bytes == [0x49, 0x81, 0xc2, 0x00, 0x40, 0x00, 0x00]),
            "translation backing selection must reserve four complete pages"
        );
        assert!(
            translation_selector
                .windows(6)
                .any(|bytes| { bytes == [0x48, 0x3d, 0x00, 0xc0, 0xff, 0x3f] })
        );

        let builder = file
            .symbol_by_name(X86_SYSTEMS_TRANSLATION_COMMIT_SYMBOL)
            .unwrap();
        let builder_start = usize::try_from(builder.address()).unwrap();
        let builder_end = builder_start + usize::try_from(builder.size()).unwrap();
        let builder = &data[builder_start..builder_end];
        assert!(
            builder
                .windows(4)
                .any(|bytes| bytes == [0xfc, 0xf3, 0x48, 0xab])
        );
        assert!(
            builder
                .windows(6)
                .any(|bytes| bytes == [0x81, 0xf9, 0x00, 0x02, 0x00, 0x00])
        );
        assert!(builder.windows(3).any(|bytes| bytes == [0x0f, 0xae, 0xf0]));
        assert!(
            builder
                .windows(8)
                .any(|bytes| { bytes == 0x0000_0000_fec0_009b_u64.to_le_bytes() })
        );
        assert!(
            builder
                .windows(8)
                .any(|bytes| { bytes == 0x0000_0000_fee0_009b_u64.to_le_bytes() })
        );
        assert!(
            builder
                .windows(7)
                .any(|bytes| bytes == [0x49, 0x89, 0x80, 0x18, 0x10, 0x00, 0x00]),
            "replacement translation must publish the local-APIC directory through PDPT[3]"
        );

        let send = symbol_bytes(&file, X86_SYSTEMS_LOCAL_NOTIFICATION_SEND_SYMBOL);
        assert!(send.windows(2).any(|bytes| bytes == [0x0f, 0x32]));
        assert!(send.windows(2).any(|bytes| bytes == [0x0f, 0x30]));
        assert!(
            send.windows(8)
                .any(|bytes| bytes == 0xfee0_0300_u64.to_le_bytes())
        );
        assert!(
            send.windows(4)
                .any(|bytes| bytes == 0x0004_40f1_u32.to_le_bytes())
        );
        let wait = symbol_bytes(&file, X86_SYSTEMS_LOCAL_NOTIFICATION_WAIT_SYMBOL);
        assert!(wait.windows(3).any(|bytes| bytes == [0xfb, 0xf4, 0xfa]));
        let complete = symbol_bytes(&file, X86_SYSTEMS_LOCAL_NOTIFICATION_COMPLETE_SYMBOL);
        assert!(
            complete
                .windows(8)
                .any(|bytes| bytes == 0xfee0_00b0_u64.to_le_bytes())
        );
        let clock = symbol_bytes(&file, X86_SYSTEMS_MONOTONIC_CLOCK_NOW_SYMBOL);
        assert!(
            clock
                .windows(8)
                .any(|bytes| bytes == 0x0000_0000_fed0_0000_u64.to_le_bytes())
        );
        assert!(
            clock
                .windows(6)
                .any(|bytes| bytes == [0x81, 0xfa, 0x80, 0x96, 0x98, 0x00])
        );
        assert!(
            clock
                .windows(7)
                .any(|bytes| bytes == [0x49, 0x8b, 0x80, 0xf0, 0, 0, 0])
        );
        assert!(
            clock
                .windows(8)
                .any(|bytes| bytes == 0xc000_0000_0000_0000_u64.to_le_bytes())
        );
        assert!(
            clock
                .windows(8)
                .any(|bytes| bytes == 0x4000_0000_0000_0000_u64.to_le_bytes())
        );
        let deadline_after = symbol_bytes(&file, X86_SYSTEMS_DEADLINE_AFTER_SYMBOL);
        assert!(
            deadline_after
                .windows(4)
                .any(|bytes| bytes == 100_000_u32.to_le_bytes()),
            "deadline construction must add exactly one millisecond of validated HPET ticks"
        );
        let deadline_arm = symbol_bytes(&file, X86_SYSTEMS_DEADLINE_ARM_SYMBOL);
        for address in [
            0x0000_0000_fed0_0000_u64,
            0x0000_0000_fec0_0000_u64,
            0x0000_0000_fee0_0300_u64,
        ] {
            assert!(
                deadline_arm
                    .windows(8)
                    .any(|bytes| bytes == address.to_le_bytes())
            );
        }
        assert!(
            deadline_arm
                .windows(4)
                .any(|bytes| bytes == 0x0000_0404_u32.to_le_bytes()),
            "deadline arm must select one-shot timer 0 delivery on Q35 route 2"
        );
        assert!(
            deadline_arm
                .windows(4)
                .any(|bytes| bytes == 0x0004_40f2_u32.to_le_bytes()),
            "an already-expired deadline must become immediately deliverable"
        );
        assert!(
            deadline_arm
                .windows(6)
                .any(|bytes| bytes == [0xb0, 0xff, 0xe6, 0x21, 0xe6, 0xa1]),
            "deadline wait must mask unrelated legacy PIC delivery"
        );
        let deadline_wait = symbol_bytes(&file, X86_SYSTEMS_DEADLINE_WAIT_SYMBOL);
        assert!(
            deadline_wait
                .windows(3)
                .any(|bytes| bytes == [0xfb, 0xf4, 0xfa])
        );
        let deadline_complete = symbol_bytes(&file, X86_SYSTEMS_DEADLINE_COMPLETE_SYMBOL);
        assert!(
            deadline_complete
                .windows(8)
                .any(|bytes| bytes == 0x0000_0000_fed0_0000_u64.to_le_bytes())
        );
        assert!(
            deadline_complete
                .windows(8)
                .any(|bytes| bytes == 0x0000_0000_fee0_00b0_u64.to_le_bytes())
        );
        let context_create = symbol_bytes(&file, X86_SYSTEMS_CONTEXT_CREATE_SYMBOL);
        assert!(
            context_create
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x81, 0xfa, 0x00, 0x40, 0x00, 0x00]),
            "context creation must validate the exact 16 KiB stack extent"
        );
        assert!(
            context_create
                .windows(4)
                .any(|bytes| bytes == [0xc6, 0x47, 0x18, 0x01]),
            "context creation must publish exactly one suspended worker"
        );
        let context_transfer = symbol_bytes(&file, X86_SYSTEMS_CONTEXT_TRANSFER_SYMBOL);
        assert!(
            context_transfer.windows(10).any(|bytes| {
                bytes == [0x53, 0x55, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57]
            }),
            "context transfer must save the x86-64 callee-saved continuation"
        );
        assert!(
            context_transfer
                .windows(4)
                .any(|bytes| bytes == [0x48, 0x8b, 0x67, 0x08]),
            "context transfer must select the provider-private worker stack pointer"
        );
        let context_retire = symbol_bytes(&file, X86_SYSTEMS_CONTEXT_RETIRE_SYMBOL);
        assert!(
            context_retire
                .windows(4)
                .any(|bytes| bytes == [0xc6, 0x47, 0x18, 0x03])
        );
        assert!(
            context_retire
                .windows(3)
                .any(|bytes| bytes == [0x48, 0x8b, 0x27]),
            "retirement must resume the exact saved caller stack pointer"
        );
        let context_reclaim = symbol_bytes(&file, X86_SYSTEMS_CONTEXT_RECLAIM_SYMBOL);
        assert!(
            context_reclaim
                .windows(4)
                .any(|bytes| bytes == [0xc6, 0x47, 0x18, 0x04]),
            "reclaim must terminally consume the completed context state"
        );

        let activator = file
            .symbol_by_name(X86_SYSTEMS_TRANSLATION_ACTIVATE_SYMBOL)
            .unwrap();
        let activator_start = usize::try_from(activator.address()).unwrap();
        let activator_end = activator_start + usize::try_from(activator.size()).unwrap();
        let activator = &data[activator_start..activator_end];
        assert!(
            activator
                .windows(3)
                .any(|bytes| bytes == [0x0f, 0x22, 0xdf]),
            "translation activation must retain the qualified CR3 transition"
        );
        assert!(
            activator
                .windows(5)
                .any(|bytes| bytes == [0x0d, 0x00, 0x08, 0x00, 0x00]),
            "translation activation must enable NX before admitting non-executable leaves"
        );

        let edit_begin = file
            .symbol_by_name(X86_SYSTEMS_TRANSLATION_EDIT_BEGIN_SYMBOL)
            .unwrap();
        let start = usize::try_from(edit_begin.address()).unwrap();
        let end = start + usize::try_from(edit_begin.size()).unwrap();
        let edit_begin = &data[start..end];
        assert!(
            edit_begin
                .windows(7)
                .any(|bytes| bytes == [0x49, 0x81, 0xc2, 0x00, 0x20, 0x00, 0x00]),
            "map-edit begin must reserve two complete metadata pages"
        );

        let edit_map = file
            .symbol_by_name(X86_SYSTEMS_TRANSLATION_EDIT_MAP_SYMBOL)
            .unwrap();
        let start = usize::try_from(edit_map.address()).unwrap();
        let end = start + usize::try_from(edit_map.size()).unwrap();
        let edit_map = &data[start..end];
        assert!(
            edit_map
                .windows(10)
                .any(|bytes| bytes == [0x48, 0xba, 0x03, 0, 0, 0, 0, 0, 0, 0x80]),
            "staged leaf must be present, writable, and non-executable"
        );

        let edit_unmap = file
            .symbol_by_name(X86_SYSTEMS_TRANSLATION_EDIT_UNMAP_SYMBOL)
            .unwrap();
        let start = usize::try_from(edit_unmap.address()).unwrap();
        let end = start + usize::try_from(edit_unmap.size()).unwrap();
        let edit_unmap = &data[start..end];
        assert!(
            edit_unmap
                .windows(8)
                .any(|bytes| bytes == [0x48, 0x83, 0xbf, 0x08, 0x10, 0, 0, 0]),
            "unmap staging must validate the published parent entry"
        );

        let edit_commit = file
            .symbol_by_name(X86_SYSTEMS_TRANSLATION_EDIT_COMMIT_SYMBOL)
            .unwrap();
        let start = usize::try_from(edit_commit.address()).unwrap();
        let end = start + usize::try_from(edit_commit.size()).unwrap();
        let edit_commit = &data[start..end];
        assert!(
            edit_commit
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x89, 0x87, 0x08, 0x10, 0, 0]),
            "map commit must publish PDPT[1] only after staging"
        );
        assert!(
            edit_commit
                .windows(3)
                .any(|bytes| bytes == [0x0f, 0x01, 0x3e]),
            "unmap commit must invalidate the private mapping before returning frames"
        );
    }
}
