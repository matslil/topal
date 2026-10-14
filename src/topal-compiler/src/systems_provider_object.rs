//! Generated ELF object for the sealed initial x86-64 systems provider.

use std::collections::BTreeMap;

use object::write::{Object, Symbol, SymbolSection};
use object::{
    Architecture, BinaryFormat, Endianness, SectionKind, SymbolFlags, SymbolKind, SymbolScope,
};
use topal_language::compiler::CompilerSystemsProgram;

use crate::systems_boot_resource::{
    X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET, generate_initial_boot_resource,
};
use crate::{
    CompileError, X86_SYSTEMS_PROVIDER_REVISION, X86SystemsProviderPlan,
    plan_x86_64_systems_provider,
};

pub const X86_SYSTEMS_PROVIDER_OBJECT_REVISION: &str =
    "topal.provider-object.x86_64-qemu-pc-q35/55";
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
pub const X86_SYSTEMS_CONTEXT_AWAIT_DEADLINE_PREEMPTION_SYMBOL: &str =
    "topal_x86_systems_context_await_deadline_preemption";
pub const X86_SYSTEMS_CONTEXT_PREEMPT_CURRENT_SYMBOL: &str =
    "topal_x86_systems_context_preempt_current";
pub const X86_SYSTEMS_CONTEXT_RETIRE_SYMBOL: &str = "topal_x86_systems_context_retire";
pub const X86_SYSTEMS_CONTEXT_RECLAIM_SYMBOL: &str = "topal_x86_systems_context_reclaim";
pub const X86_SYSTEMS_USER_ADDRESS_SPACE_PREPARE_SYMBOL: &str =
    "topal_x86_systems_user_prepare_initial_address_space";
pub const X86_SYSTEMS_USER_ADDRESS_SPACE_VERIFY_MAPPING_SYMBOL: &str =
    "topal_x86_systems_user_verify_initial_mapping";
pub const X86_SYSTEMS_USER_ADDRESS_SPACE_MAP_SYMBOL: &str =
    "topal_x86_systems_user_map_initial_frame";
pub const X86_SYSTEMS_USER_MAPPING_STORE_BYTE_SYMBOL: &str =
    "topal_x86_systems_user_store_initial_mapping_byte";
pub const X86_SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE_SYMBOL: &str =
    "topal_x86_systems_user_install_initial_entry_trampoline";
pub const X86_SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY_SYMBOL: &str =
    "topal_x86_systems_user_protect_initial_mapping_read_only";
pub const X86_SYSTEMS_USER_MAPPING_LOAD_BYTE_SYMBOL: &str =
    "topal_x86_systems_user_load_initial_mapping_byte";
pub const X86_SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE_SYMBOL: &str =
    "topal_x86_systems_user_verify_initial_entry_trampoline";
pub const X86_SYSTEMS_USER_MAPPING_PERMIT_EXECUTE_SYMBOL: &str =
    "topal_x86_systems_user_permit_initial_mapping_execute";
pub const X86_SYSTEMS_USER_MAPPING_SEAL_IMMUTABLE_DATA_SYMBOL: &str =
    "topal_x86_systems_user_seal_initial_mapping_immutable_data";
pub const X86_SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA_SYMBOL: &str =
    "topal_x86_systems_user_permit_initial_mapping_writable_data";
pub const X86_SYSTEMS_USER_ADDRESS_SPACE_UNMAP_SYMBOL: &str =
    "topal_x86_systems_user_unmap_initial_frame";
pub const X86_SYSTEMS_USER_PREPARE_SYMBOL: &str = "topal_x86_systems_user_prepare_initial_probe";
pub const X86_SYSTEMS_USER_ENTER_SYMBOL: &str = "topal_x86_systems_user_enter_initial_context";
pub const X86_SYSTEMS_USER_COMPLETE_SYMBOL: &str =
    "topal_x86_systems_user_complete_initial_backing";
pub const X86_SYSTEMS_USER_ADDRESS_SPACE_RECLAIM_SYMBOL: &str =
    "topal_x86_systems_user_reclaim_initial_address_space";
pub const X86_SYSTEMS_USER_SYSCALL_SYMBOL: &str = "topal_x86_systems_user_dispatch_initial_syscall";
pub const X86_SYSTEMS_USER_COPY_FAULT_SYMBOL: &str = "topal_x86_systems_user_copy_page_fault";
pub const X86_SYSTEMS_ALLOCATABLE_FLOOR: u64 = 16 * 1024 * 1024;
pub(crate) const INITIAL_USER_CODE_ADDRESS: u64 = 0x0060_4000;
const INITIAL_USER_PROGRAM_HEADER_ADDRESS: u64 = INITIAL_USER_CODE_ADDRESS + 64;
const INITIAL_USER_ENTRY_ADDRESS: u64 = INITIAL_USER_CODE_ADDRESS + 256;
const INITIAL_USER_DATA_ADDRESS: u64 = 0x0060_5000;
pub(crate) const INITIAL_USER_DATA_FILE_OFFSET: usize = 4096;
pub(crate) const INITIAL_USER_ENTRY_ELF_OFFSET: usize = 24;
pub(crate) const INITIAL_USER_CODE_FILE_SIZE_ELF_OFFSET: usize = 96;
pub(crate) const INITIAL_USER_CODE_MEMORY_SIZE_ELF_OFFSET: usize = 104;
pub(crate) const INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET: usize = 152;
pub(crate) const INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET: usize = 160;
pub(crate) const INITIAL_USER_DATA_CAPACITY: usize = 4096;
const INITIAL_USER_STACK_ADDRESS: u64 = 0x0060_1f20;
const INITIAL_USER_UNAME_CANDIDATE: u32 = 0x0060_1e00;
const INITIAL_USER_NANOSLEEP_CANDIDATE: u32 = 0x0060_1ff0;

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
    append_user_functions(&mut object, text)?;
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
        (
            X86_SYSTEMS_CONTEXT_AWAIT_DEADLINE_PREEMPTION_SYMBOL,
            context_await_deadline_preemption()?,
        ),
        (
            X86_SYSTEMS_CONTEXT_PREEMPT_CURRENT_SYMBOL,
            context_preempt_current()?,
        ),
        (X86_SYSTEMS_CONTEXT_RETIRE_SYMBOL, context_retire()?),
        (X86_SYSTEMS_CONTEXT_RECLAIM_SYMBOL, context_reclaim()?),
    ] {
        append_encoded_function(object, text, name, &encoded);
    }
    Ok(())
}

fn append_user_functions(
    object: &mut Object<'_>,
    text: object::write::SectionId,
) -> Result<(), CompileError> {
    for (name, encoded) in [
        (
            X86_SYSTEMS_USER_ADDRESS_SPACE_PREPARE_SYMBOL,
            user_prepare_initial_address_space()?,
        ),
        (
            X86_SYSTEMS_USER_ADDRESS_SPACE_VERIFY_MAPPING_SYMBOL,
            user_verify_initial_mapping()?,
        ),
        (
            X86_SYSTEMS_USER_ADDRESS_SPACE_MAP_SYMBOL,
            user_map_initial_frame()?,
        ),
        (
            X86_SYSTEMS_USER_MAPPING_STORE_BYTE_SYMBOL,
            user_store_initial_mapping_byte()?,
        ),
        (
            X86_SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE_SYMBOL,
            user_install_initial_entry_trampoline()?,
        ),
        (
            X86_SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY_SYMBOL,
            user_protect_initial_mapping_read_only()?,
        ),
        (
            X86_SYSTEMS_USER_MAPPING_LOAD_BYTE_SYMBOL,
            user_load_initial_mapping_byte()?,
        ),
        (
            X86_SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE_SYMBOL,
            user_verify_initial_entry_trampoline()?,
        ),
        (
            X86_SYSTEMS_USER_MAPPING_PERMIT_EXECUTE_SYMBOL,
            user_permit_initial_mapping_execute()?,
        ),
        (
            X86_SYSTEMS_USER_MAPPING_SEAL_IMMUTABLE_DATA_SYMBOL,
            user_seal_initial_mapping_immutable_data()?,
        ),
        (
            X86_SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA_SYMBOL,
            user_permit_initial_mapping_writable_data()?,
        ),
        (
            X86_SYSTEMS_USER_ADDRESS_SPACE_UNMAP_SYMBOL,
            user_unmap_initial_frame()?,
        ),
        (
            X86_SYSTEMS_USER_PREPARE_SYMBOL,
            user_prepare_initial_probe()?,
        ),
        (X86_SYSTEMS_USER_ENTER_SYMBOL, user_enter_initial_context()?),
        (
            X86_SYSTEMS_USER_COMPLETE_SYMBOL,
            user_complete_initial_backing()?,
        ),
        (
            X86_SYSTEMS_USER_ADDRESS_SPACE_RECLAIM_SYMBOL,
            user_reclaim_initial_address_space()?,
        ),
        (
            X86_SYSTEMS_USER_SYSCALL_SYMBOL,
            user_dispatch_initial_syscall()?,
        ),
        (
            X86_SYSTEMS_USER_COPY_FAULT_SYMBOL,
            user_copy_page_fault_entry()?,
        ),
    ] {
        append_encoded_function(object, text, name, &encoded);
    }
    Ok(())
}

fn user_prepare_initial_address_space() -> Result<Vec<u8>, CompileError> {
    let mut probe = initial_user_probe()?;
    let expected_boot_resource = generate_initial_boot_resource(&initial_user_elf_file()?)?;
    probe.resize(probe.len().next_multiple_of(8), 0x90);
    if probe.len() > 4096 {
        return Err(CompileError::Tool(
            "initial user probe exceeds its sealed executable page".into(),
        ));
    }
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xd2]); // allocator-owned backing base
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc2, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xfa, 0x00, 0xd0, 0xff, 0x3f]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x49, 0x89, 0xd2]); // retain backing identity in r10
    code.bytes(&[0x80, 0x3f, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // retained Linux boot_params
    code.jump_if(0x84, "fail");
    code.bytes(&[0x83, 0xbe, 0xc0, 0x00, 0x00, 0x00, 0x00]); // ext_ramdisk_image
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xbe, 0xc4, 0x00, 0x00, 0x00, 0x00]); // ext_ramdisk_size
    code.jump_if(0x85, "fail");
    code.bytes(&[0x44, 0x8b, 0x86, 0x18, 0x02, 0x00, 0x00]); // ramdisk_image
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x8b, 0x8e, 0x1c, 0x02, 0x00, 0x00]); // ramdisk_size
    code.bytes(&[0x81, 0xf9]);
    code.bytes(
        &u32::try_from(expected_boot_resource.len())
            .map_err(|_| CompileError::Tool("initial boot resource exceeds u32".into()))?
            .to_le_bytes(),
    );
    code.jump_if(0x85, "fail");
    code.bytes(&[0x4c, 0x89, 0xc0, 0x48, 0x01, 0xc8]); // resource end
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x3d, 0x00, 0x00, 0x00, 0x40]); // within initial identity map
    code.jump_if(0x87, "fail");
    let image_end =
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + initial_user_elf_file()?.len();
    let metadata_offsets = (0..X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize)
        .chain(
            X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
                ..X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + 256,
        )
        .chain(image_end..expected_boot_resource.len())
        .filter(|offset| {
            let code_size_fields = X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
                + INITIAL_USER_CODE_FILE_SIZE_ELF_OFFSET
                ..X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
                    + INITIAL_USER_CODE_MEMORY_SIZE_ELF_OFFSET
                    + 8;
            let entry_field = X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
                + INITIAL_USER_ENTRY_ELF_OFFSET
                ..X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
                    + INITIAL_USER_ENTRY_ELF_OFFSET
                    + 8;
            let data_size_fields = X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
                + INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET
                ..X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
                    + INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET
                    + 8;
            !entry_field.contains(offset)
                && !code_size_fields.contains(offset)
                && !data_size_fields.contains(offset)
        });
    for offset in metadata_offsets {
        let expected = expected_boot_resource[offset];
        code.bytes(&[0x41, 0x80, 0xb8]); // cmp byte ptr [r8 + displacement], immediate
        code.bytes(
            &u32::try_from(offset)
                .map_err(|_| CompileError::Tool("boot resource offset exceeds u32".into()))?
                .to_le_bytes(),
        );
        code.bytes(&[expected]);
        code.jump_if(0x85, "fail");
    }
    let code_file_size_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + INITIAL_USER_CODE_FILE_SIZE_ELF_OFFSET,
    )
    .map_err(|_| CompileError::Tool("external code-size offset exceeds u32".into()))?;
    let code_memory_size_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
            + INITIAL_USER_CODE_MEMORY_SIZE_ELF_OFFSET,
    )
    .map_err(|_| CompileError::Tool("external memory-size offset exceeds u32".into()))?;
    code.bytes(&[0x49, 0x8b, 0x80]); // executable p_filesz
    code.bytes(&code_file_size_offset.to_le_bytes());
    code.bytes(&[0x49, 0x3b, 0x80]); // p_filesz == p_memsz
    code.bytes(&code_memory_size_offset.to_le_bytes());
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x3d, 0x08, 0x01, 0x00, 0x00]); // includes entry qword
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x3d, 0x00, 0x10, 0x00, 0x00]); // one base page
    code.jump_if(0x87, "fail");
    code.bytes(&[0xa8, 0x07]); // qword complete
    code.jump_if(0x85, "fail");
    let entry_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + INITIAL_USER_ENTRY_ELF_OFFSET,
    )
    .map_err(|_| CompileError::Tool("external entry offset exceeds u32".into()))?;
    code.bytes(&[0x4d, 0x8b, 0x98]); // ELF e_entry
    code.bytes(&entry_offset.to_le_bytes());
    code.bytes(&[0x49, 0x81, 0xfb]);
    code.bytes(&(INITIAL_USER_CODE_ADDRESS as u32 + 256).to_le_bytes());
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x05]);
    code.bytes(&(INITIAL_USER_CODE_ADDRESS as u32).to_le_bytes());
    code.bytes(&[0x49, 0x39, 0xc3]);
    code.jump_if(0x83, "fail");
    let data_file_size_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET,
    )
    .map_err(|_| CompileError::Tool("external data-size offset exceeds u32".into()))?;
    let data_memory_size_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
            + INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET,
    )
    .map_err(|_| CompileError::Tool("external BSS-size offset exceeds u32".into()))?;
    code.bytes(&[0x49, 0x8b, 0x80]); // data p_filesz
    code.bytes(&data_file_size_offset.to_le_bytes());
    code.bytes(&[0x48, 0x83, 0xf8, 0x08]); // at least one initialized qword
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x3d, 0x00, 0x10, 0x00, 0x00]); // one file page
    code.jump_if(0x87, "fail");
    code.bytes(&[0xa8, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x8b, 0x90]); // data p_memsz
    code.bytes(&data_memory_size_offset.to_le_bytes());
    code.bytes(&[0x48, 0x39, 0xd0]); // p_filesz <= p_memsz
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x81, 0xfa, 0x00, 0x10, 0x00, 0x00]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0xf6, 0xc2, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x89, 0xf9]); // retain state in r9
    code.bytes(&[0x4c, 0x89, 0x57, 0x68]); // retain backing identity
    code.bytes(&[0x4c, 0x89, 0x9f, 0x88, 0x00, 0x00, 0x00]); // selected entry
    code.bytes(&[0x4d, 0x89, 0xd0]); // publish through supervisor identity mapping
    for (index, chunk) in probe.chunks_exact(8).enumerate() {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(chunk);
        let displacement = u32::try_from(index * 8)
            .map_err(|_| CompileError::Tool("user probe displacement exceeds u32".into()))?;
        code.bytes(&[0x49, 0x89, 0x80]);
        code.bytes(&displacement.to_le_bytes());
    }
    for (index, chunk) in initial_user_stack_bytes().chunks_exact(8).enumerate() {
        code.bytes(&[0x48, 0xb8]);
        code.bytes(chunk);
        let displacement = u32::try_from(0x1f20 + index * 8).map_err(|_| {
            CompileError::Tool("initial user stack displacement exceeds u32".into())
        })?;
        code.bytes(&[0x49, 0x89, 0x82]);
        code.bytes(&displacement.to_le_bytes());
    }
    code.bytes(&[0x4d, 0x8b, 0x99, 0x88, 0x00, 0x00, 0x00]);
    code.bytes(&[0x4d, 0x89, 0x9a, 0x58, 0x1f, 0x00, 0x00]); // runtime AT_ENTRY
    code.bytes(&[0x4d, 0x8d, 0x9a, 0x00, 0x20, 0x00, 0x00]); // leaf table
    code.bytes(&[0x4c, 0x89, 0xdf, 0x31, 0xc0]);
    code.bytes(&[0xb9, 0x00, 0x02, 0x00, 0x00, 0xfc, 0xf3, 0x48, 0xab]); // clear frame
    code.bytes(&[0x4c, 0x89, 0xcf]); // restore state
    code.bytes(&[0x4c, 0x89, 0xd2, 0x48, 0x83, 0xca, 0x05]); // RX code leaf
    code.bytes(&[0x49, 0x89, 0x13]);
    code.bytes(&[0x49, 0x8d, 0x92, 0x00, 0x10, 0x00, 0x00]); // stack frame
    code.bytes(&[0x48, 0xb9]);
    code.bytes(&0x8000_0000_0000_0007_u64.to_le_bytes());
    code.bytes(&[0x48, 0x09, 0xca, 0x49, 0x89, 0x53, 0x08]); // RW/NX stack leaf
    code.bytes(&[0x0f, 0x20, 0xd8]); // active root from CR3
    code.bytes(&[0x48, 0x25, 0x00, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x4d, 0x89, 0xda, 0x49, 0x83, 0xca, 0x07]);
    code.bytes(&[0x4c, 0x89, 0x90, 0x18, 0x20, 0x00, 0x00]); // publish user PT
    code.bytes(&[0x0f, 0xae, 0xf0]); // publish before invalidation
    for address in [0x0060_0000_u64, 0x0060_1000, 0x0060_2000] {
        code.bytes(&[0x49, 0xb8]);
        code.bytes(&address.to_le_bytes());
        code.bytes(&[0x41, 0x0f, 0x01, 0x38]); // invlpg [r8]
    }

    code.bytes(&[0xc6, 0x87, 0x70, 0x00, 0x00, 0x00, 0x01]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn user_verify_initial_mapping() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc2, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x80, 0x3f, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x01]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x8b, 0x4f, 0x68]);
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x49, 0x89, 0xd0]);
    code.bytes(&[0x49, 0x81, 0xc0, 0x00, 0x10, 0x00, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x49, 0x89, 0xc9]);
    code.bytes(&[0x49, 0x81, 0xc1, 0x00, 0x30, 0x00, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x49, 0x39, 0xc8]);
    code.jump_if(0x86, "disjoint");
    code.bytes(&[0x4c, 0x39, 0xca]);
    code.jump_if(0x83, "disjoint");
    code.jump("fail");

    code.bind("disjoint")?;
    code.bytes(&[0x48, 0x81, 0xc1, 0x00, 0x20, 0x00, 0x00]);
    code.bytes(&[0x48, 0x83, 0x79, 0x10, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x89, 0xd0]);
    code.bytes(&[0x49, 0xb9]);
    code.bytes(&0x8000_0000_0000_0007_u64.to_le_bytes());
    code.bytes(&[0x4d, 0x09, 0xc8, 0x4c, 0x89, 0x41, 0x10]);
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x0060_2000_u64.to_le_bytes());
    code.bytes(&[0x41, 0x0f, 0x01, 0x3a]);
    code.bytes(&[0x41, 0xc6, 0x02, 0x6d]);
    code.bytes(&[0x41, 0x80, 0x3a, 0x6d]);
    code.jump_if(0x85, "cleanup-fail");
    code.bytes(&[0x48, 0xc7, 0x41, 0x10, 0, 0, 0, 0]);
    code.bytes(&[0x0f, 0xae, 0xf0, 0x41, 0x0f, 0x01, 0x3a]);
    code.bytes(&[0xc6, 0x87, 0x70, 0x00, 0x00, 0x00, 0x0b]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);

    code.bind("cleanup-fail")?;
    code.bytes(&[0x48, 0xc7, 0x41, 0x10, 0, 0, 0, 0]);
    code.bytes(&[0x0f, 0xae, 0xf0, 0x41, 0x0f, 0x01, 0x3a]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn user_map_initial_frame() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc2, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x80, 0x3f, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x01]);
    code.jump_if(0x84, "generation-one");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x0e]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0xb8, 0x11, 0x00, 0x00, 0x00]);
    code.jump("generation-accepted");
    code.bind("generation-one")?;
    code.bytes(&[0xb8, 0x04, 0x00, 0x00, 0x00]);
    code.bind("generation-accepted")?;
    code.bytes(&[0x3c, 0x04]);
    code.jump_if(0x84, "check-generation-one-frame");
    code.bytes(&[0x48, 0x83, 0xbf, 0x80, 0x00, 0x00, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x3b, 0x57, 0x78]);
    code.jump_if(0x84, "fail");
    code.jump("frame-slot-clear");
    code.bind("check-generation-one-frame")?;
    code.bytes(&[0x48, 0x83, 0x7f, 0x78, 0x00]);
    code.jump_if(0x85, "fail");
    code.bind("frame-slot-clear")?;
    code.bytes(&[0x48, 0x8b, 0x4f, 0x68]);
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x49, 0x89, 0xd0]);
    code.bytes(&[0x49, 0x81, 0xc0, 0x00, 0x10, 0x00, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x49, 0x89, 0xc9]);
    code.bytes(&[0x49, 0x81, 0xc1, 0x00, 0x30, 0x00, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x49, 0x39, 0xc8]);
    code.jump_if(0x86, "disjoint");
    code.bytes(&[0x4c, 0x39, 0xca]);
    code.jump_if(0x83, "disjoint");
    code.jump("fail");

    code.bind("disjoint")?;
    code.bytes(&[0x48, 0x81, 0xc1, 0x00, 0x20, 0x00, 0x00]);
    code.bytes(&[0x3c, 0x04]);
    code.jump_if(0x84, "pte-slot-selected");
    code.bytes(&[0x48, 0x83, 0xc1, 0x08]);
    code.bind("pte-slot-selected")?;
    code.bytes(&[0x48, 0x83, 0x79, 0x20, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x89, 0xd0]);
    code.bytes(&[0x49, 0xb9]);
    code.bytes(&0x8000_0000_0000_0007_u64.to_le_bytes());
    code.bytes(&[0x4d, 0x09, 0xc8, 0x4c, 0x89, 0x41, 0x20]);
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x3c, 0x04]);
    code.jump_if(0x84, "publish-generation-one");
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x0060_5000_u64.to_le_bytes());
    code.bytes(&[0x48, 0x89, 0x97, 0x80, 0x00, 0x00, 0x00]);
    code.jump("publish-mapping");
    code.bind("publish-generation-one")?;
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x0060_4000_u64.to_le_bytes());
    code.bytes(&[0x48, 0x89, 0x57, 0x78]);
    code.bind("publish-mapping")?;
    code.bytes(&[0x41, 0x0f, 0x01, 0x3a]);
    code.bytes(&[0x88, 0x87, 0x70, 0x00, 0x00, 0x00]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);

    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn user_store_initial_mapping_byte() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x11]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // retained Linux boot_params
    code.jump_if(0x84, "fail");
    emit_initial_boot_resource_handoff_validation(&mut code)?;
    let data_payload_offset = X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET
        .checked_add(
            u32::try_from(INITIAL_USER_DATA_FILE_OFFSET)
                .map_err(|_| CompileError::Tool("external user data offset exceeds u32".into()))?,
        )
        .ok_or_else(|| CompileError::Tool("external user data offset overflow".into()))?;
    let data_file_size_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET,
    )
    .map_err(|_| CompileError::Tool("external data-size offset exceeds u32".into()))?;
    let data_memory_size_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
            + INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET,
    )
    .map_err(|_| CompileError::Tool("external BSS-size offset exceeds u32".into()))?;
    code.bytes(&[0x49, 0x8b, 0x88]); // runtime p_filesz
    code.bytes(&data_file_size_offset.to_le_bytes());
    code.bytes(&[0x4d, 0x8b, 0x98]); // runtime p_memsz
    code.bytes(&data_memory_size_offset.to_le_bytes());
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&INITIAL_USER_DATA_ADDRESS.to_le_bytes());
    code.bytes(&[0x4c, 0x01, 0xd8]); // initial break = data base + p_memsz
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x3d, 0x00, 0x60, 0x60, 0x00]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x89, 0x87, 0x90, 0x00, 0x00, 0x00]); // current break
    code.bytes(&[0x48, 0x89, 0x87, 0x98, 0x00, 0x00, 0x00]); // initial break
    code.bytes(&[0x41, 0xbb, 0x00, 0x10, 0x00, 0x00]);
    code.bytes(&[0x49, 0x29, 0xcb]); // zero bytes = page size - p_filesz
    code.bytes(&[0x49, 0x81, 0xc0]);
    code.bytes(&data_payload_offset.to_le_bytes());
    code.bytes(&[0x49, 0xba]);
    code.bytes(&INITIAL_USER_DATA_ADDRESS.to_le_bytes());
    code.bind("data-copy-loop")?;
    code.bytes(&[0x49, 0x8b, 0x00]);
    code.bytes(&[0x49, 0x89, 0x02]);
    code.bytes(&[0x49, 0x83, 0xc0, 0x08]);
    code.bytes(&[0x49, 0x83, 0xc2, 0x08]);
    code.bytes(&[0x48, 0x83, 0xe9, 0x08]);
    code.jump_if(0x85, "data-copy-loop");
    code.bytes(&[0x4d, 0x85, 0xdb]);
    code.jump_if(0x84, "data-complete");
    code.bind("bss-zero-loop")?;
    code.bytes(&[0x49, 0xc7, 0x02, 0, 0, 0, 0]);
    code.bytes(&[0x49, 0x83, 0xc2, 0x08]);
    code.bytes(&[0x49, 0x83, 0xeb, 0x08]);
    code.jump_if(0x85, "bss-zero-loop");
    code.bind("data-complete")?;
    code.bytes(&[0xc6, 0x87, 0x70, 0x00, 0x00, 0x00, 0x12]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

struct InitialUserEntryPayload {
    bytes: Vec<u8>,
    continuation_address: u64,
    uname_return_address: u64,
    nanosleep_return_address: u64,
}

fn initial_user_stack_bytes() -> [u8; 224] {
    let mut bytes = [0_u8; 224];
    bytes[0..8].copy_from_slice(&1_u64.to_le_bytes());
    bytes[8..16].copy_from_slice(&0x0060_1ff0_u64.to_le_bytes());
    bytes[32..40].copy_from_slice(&6_u64.to_le_bytes());
    bytes[40..48].copy_from_slice(&4096_u64.to_le_bytes());
    bytes[48..56].copy_from_slice(&9_u64.to_le_bytes());
    bytes[56..64].copy_from_slice(&0_u64.to_le_bytes());
    bytes[64..72].copy_from_slice(&31_u64.to_le_bytes());
    bytes[72..80].copy_from_slice(&0x0060_1ff0_u64.to_le_bytes());
    for (offset, tag) in [(80, 11_u64), (96, 12), (112, 13), (128, 14)] {
        bytes[offset..offset + 8].copy_from_slice(&tag.to_le_bytes());
    }
    bytes[144..152].copy_from_slice(&3_u64.to_le_bytes());
    bytes[152..160].copy_from_slice(&INITIAL_USER_PROGRAM_HEADER_ADDRESS.to_le_bytes());
    bytes[160..168].copy_from_slice(&4_u64.to_le_bytes());
    bytes[168..176].copy_from_slice(&56_u64.to_le_bytes());
    bytes[176..184].copy_from_slice(&5_u64.to_le_bytes());
    bytes[184..192].copy_from_slice(&3_u64.to_le_bytes());
    bytes[208..219].copy_from_slice(b"topal-probe");
    bytes
}

fn initial_user_elf_prefix(image_size: u64) -> [u8; 256] {
    let mut bytes = [0_u8; 256];
    bytes[0..16].copy_from_slice(&[0x7f, b'E', b'L', b'F', 2, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    bytes[16..18].copy_from_slice(&2_u16.to_le_bytes()); // ET_EXEC
    bytes[18..20].copy_from_slice(&62_u16.to_le_bytes()); // EM_X86_64
    bytes[20..24].copy_from_slice(&1_u32.to_le_bytes());
    bytes[24..32].copy_from_slice(&INITIAL_USER_ENTRY_ADDRESS.to_le_bytes());
    bytes[32..40].copy_from_slice(&64_u64.to_le_bytes());
    bytes[52..54].copy_from_slice(&64_u16.to_le_bytes());
    bytes[54..56].copy_from_slice(&56_u16.to_le_bytes());
    bytes[56..58].copy_from_slice(&3_u16.to_le_bytes());
    bytes[58..60].copy_from_slice(&64_u16.to_le_bytes());

    bytes[64..68].copy_from_slice(&1_u32.to_le_bytes()); // PT_LOAD
    bytes[68..72].copy_from_slice(&5_u32.to_le_bytes()); // PF_R | PF_X
    bytes[80..88].copy_from_slice(&INITIAL_USER_CODE_ADDRESS.to_le_bytes());
    bytes[88..96].copy_from_slice(&INITIAL_USER_CODE_ADDRESS.to_le_bytes());
    bytes[96..104].copy_from_slice(&image_size.to_le_bytes());
    bytes[104..112].copy_from_slice(&image_size.to_le_bytes());
    bytes[112..120].copy_from_slice(&4096_u64.to_le_bytes());

    bytes[120..124].copy_from_slice(&1_u32.to_le_bytes()); // PT_LOAD data
    bytes[124..128].copy_from_slice(&6_u32.to_le_bytes()); // PF_R | PF_W
    bytes[128..136].copy_from_slice(&(INITIAL_USER_DATA_FILE_OFFSET as u64).to_le_bytes());
    bytes[136..144].copy_from_slice(&INITIAL_USER_DATA_ADDRESS.to_le_bytes());
    bytes[144..152].copy_from_slice(&INITIAL_USER_DATA_ADDRESS.to_le_bytes());
    bytes[152..160].copy_from_slice(&8_u64.to_le_bytes());
    bytes[160..168].copy_from_slice(&16_u64.to_le_bytes());
    bytes[168..176].copy_from_slice(&4096_u64.to_le_bytes());

    bytes[176..180].copy_from_slice(&0x6474_e551_u32.to_le_bytes()); // PT_GNU_STACK
    bytes[180..184].copy_from_slice(&6_u32.to_le_bytes()); // PF_R | PF_W
    bytes[224..232].copy_from_slice(&16_u64.to_le_bytes());
    bytes
}

pub(crate) fn initial_user_code_image() -> Result<Vec<u8>, CompileError> {
    let payload = initial_user_entry_payload()?;
    let image_size = 256_u64
        .checked_add(u64::try_from(payload.bytes.len()).unwrap_or(u64::MAX))
        .ok_or_else(|| CompileError::Tool("constructed ELF image size overflow".into()))?;
    let mut bytes = initial_user_elf_prefix(image_size).to_vec();
    bytes.extend(payload.bytes);
    if bytes.len() > 4096 {
        return Err(CompileError::Tool(
            "constructed ELF image exceeds its sealed executable page".into(),
        ));
    }
    Ok(bytes)
}

pub(crate) fn initial_user_elf_file() -> Result<Vec<u8>, CompileError> {
    let mut bytes = initial_user_code_image()?;
    bytes.resize(
        INITIAL_USER_DATA_FILE_OFFSET + INITIAL_USER_DATA_CAPACITY,
        0,
    );
    bytes[INITIAL_USER_DATA_FILE_OFFSET] = 0x6d;
    Ok(bytes)
}

fn initial_user_entry_payload() -> Result<InitialUserEntryPayload, CompileError> {
    let mut entry = X86FunctionEncoder::default();
    entry.bytes(&[0x48, 0x83, 0x3c, 0x24, 0x01]); // argc = 1
    entry.jump_if(0x85, "entry-fail");
    entry.bytes(&[0x48, 0x81, 0x7c, 0x24, 0x08]);
    entry.bytes(&0x0060_1ff0_u32.to_le_bytes()); // argv[0]
    entry.jump_if(0x85, "entry-fail");
    for displacement in [0x10_u8, 0x18] {
        entry.bytes(&[0x48, 0x83, 0x7c, 0x24, displacement, 0x00]);
        entry.jump_if(0x85, "entry-fail");
    }
    entry.bytes(&[0x48, 0x83, 0x7c, 0x24, 0x20, 0x06]); // AT_PAGESZ
    entry.jump_if(0x85, "entry-fail");
    entry.bytes(&[0x48, 0x81, 0x7c, 0x24, 0x28]);
    entry.bytes(&4096_u32.to_le_bytes()); // provider base-page size
    entry.jump_if(0x85, "entry-fail");
    entry.bytes(&[0x48, 0x83, 0x7c, 0x24, 0x30, 0x09]); // AT_ENTRY
    entry.jump_if(0x85, "entry-fail");
    entry.bytes(&[0x49, 0xbb]);
    entry.bytes(&INITIAL_USER_CODE_ADDRESS.to_le_bytes());
    entry.bytes(&[0x49, 0x8b, 0x43, 0x18]); // resident ELF e_entry
    entry.bytes(&[0x48, 0x39, 0x44, 0x24, 0x38]);
    entry.jump_if(0x85, "entry-fail");
    entry.bytes(&[0x48, 0x83, 0x7c, 0x24, 0x40, 0x1f]); // AT_EXECFN
    entry.jump_if(0x85, "entry-fail");
    entry.bytes(&[0x48, 0x81, 0x7c, 0x24, 0x48]);
    entry.bytes(&0x0060_1ff0_u32.to_le_bytes()); // same string as argv[0]
    entry.jump_if(0x85, "entry-fail");
    for (displacement, tag) in [(0x50_u8, 11_u8), (0x60, 12), (0x70, 13)] {
        entry.bytes(&[0x48, 0x83, 0x7c, 0x24, displacement, tag]);
        entry.jump_if(0x85, "entry-fail");
        entry.bytes(&[0x48, 0x83, 0x7c, 0x24, displacement + 8, 0x00]);
        entry.jump_if(0x85, "entry-fail");
    }
    for (displacement, expected) in [(0x80_u32, 14_u8), (0x88, 0), (0x90, 3)] {
        entry.bytes(&[0x48, 0x83, 0xbc, 0x24]);
        entry.bytes(&displacement.to_le_bytes());
        entry.bytes(&[expected]);
        entry.jump_if(0x85, "entry-fail");
    }
    entry.bytes(&[0x48, 0x81, 0xbc, 0x24]);
    entry.bytes(&0x98_u32.to_le_bytes());
    entry.bytes(&(INITIAL_USER_PROGRAM_HEADER_ADDRESS as u32).to_le_bytes());
    entry.jump_if(0x85, "entry-fail");
    for (displacement, expected) in [
        (0xa0_u32, 4_u8),
        (0xa8, 56),
        (0xb0, 5),
        (0xb8, 3),
        (0xc0, 0),
        (0xc8, 0),
    ] {
        entry.bytes(&[0x48, 0x83, 0xbc, 0x24]);
        entry.bytes(&displacement.to_le_bytes());
        entry.bytes(&[expected]);
        entry.jump_if(0x85, "entry-fail");
    }
    entry.bytes(&[0x48, 0xb8]);
    entry.bytes(&u64::from_le_bytes(*b"topal-pr").to_le_bytes());
    entry.bytes(&[0x48, 0x39, 0x84, 0x24]);
    entry.bytes(&0xd0_u32.to_le_bytes());
    entry.jump_if(0x85, "entry-fail");
    entry.bytes(&[0x81, 0xbc, 0x24]);
    entry.bytes(&0xd8_u32.to_le_bytes());
    entry.bytes(&u32::from_le_bytes(*b"obe\0").to_le_bytes());
    entry.jump_if(0x85, "entry-fail");
    entry.bytes(&[0x48, 0x81, 0xc4]);
    entry.bytes(&224_u32.to_le_bytes()); // consume bounded initial block
    entry.bytes(&[0x49, 0xba]);
    entry.bytes(&INITIAL_USER_DATA_ADDRESS.to_le_bytes());
    entry.bytes(&[0x41, 0x80, 0x3a, 0x6d]);
    entry.jump_if(0x85, "entry-fail");
    entry.bytes(&[0x49, 0xbb]);
    entry.bytes(&INITIAL_USER_CODE_ADDRESS.to_le_bytes());
    entry.bytes(&[0x49, 0x8b, 0x8b]); // runtime data p_filesz
    entry.bytes(
        &u32::try_from(INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET)
            .unwrap()
            .to_le_bytes(),
    );
    entry.bytes(&[0x49, 0x8b, 0x93]); // runtime data p_memsz
    entry.bytes(
        &u32::try_from(INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET)
            .unwrap()
            .to_le_bytes(),
    );
    entry.bytes(&[0x49, 0x01, 0xca]); // first BSS qword
    entry.bytes(&[0x48, 0x29, 0xca]); // complete BSS byte count
    entry.bytes(&[0x48, 0x85, 0xd2]);
    entry.jump_if(0x84, "bss-complete");
    entry.bind("bss-loop")?;
    entry.bytes(&[0x49, 0x83, 0x3a, 0x00]);
    entry.jump_if(0x85, "entry-fail");
    entry.bytes(&[0x49, 0x83, 0xc2, 0x08]);
    entry.bytes(&[0x48, 0x83, 0xea, 0x08]);
    entry.jump_if(0x85, "bss-loop");
    entry.bind("bss-complete")?;
    entry.bytes(&[0x49, 0xba]);
    entry.bytes(&(INITIAL_USER_DATA_ADDRESS + 8).to_le_bytes());
    entry.bytes(&[0x41, 0xc6, 0x02, 0xa5]);
    entry.bytes(&[0x41, 0x80, 0x3a, 0xa5]);
    entry.jump_if(0x85, "entry-fail");
    entry.bytes(&[0x48, 0xb8]);
    entry.bytes(&0x0060_0000_u64.to_le_bytes());
    entry.bytes(&[0xff, 0xe0]);
    entry.bind("entry-fail")?;
    entry.bytes(&[0x0f, 0x0b]);
    let mut bytes = entry.finish()?;
    bytes.resize(bytes.len().next_multiple_of(32), 0x90);
    let continuation_address =
        INITIAL_USER_ENTRY_ADDRESS + u64::try_from(bytes.len()).unwrap_or(u64::MAX);

    let mut continuation = X86FunctionEncoder::default();
    continuation.bytes(&[0x48, 0x81, 0xec, 0x00, 0x02, 0x00, 0x00]); // bounded output
    continuation.bytes(&[0x48, 0x89, 0xe7]); // candidate = rsp
    continuation.bytes(&[0xb8, 0x3f, 0x00, 0x00, 0x00, 0x0f, 0x05]); // uname
    let uname_return_address =
        continuation_address + u64::try_from(continuation.bytes.len()).unwrap_or(u64::MAX);
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "uname-buffer-fail");
    continuation.bytes(&[0x81, 0x3c, 0x24]);
    continuation.bytes(&u32::from_le_bytes(*b"Linu").to_le_bytes());
    continuation.jump_if(0x85, "uname-buffer-fail");
    continuation.bytes(&[0x80, 0x7c, 0x24, 0x04, b'x']);
    continuation.jump_if(0x85, "uname-buffer-fail");
    continuation.bytes(&[0x48, 0x81, 0xc4, 0x00, 0x02, 0x00, 0x00]); // release output
    continuation.bytes(&[0x48, 0x83, 0xec, 0x10]); // bounded native timespec
    continuation.bytes(&[0x31, 0xc0, 0x48, 0x89, 0x04, 0x24]);
    continuation.bytes(&[0x48, 0x89, 0x44, 0x24, 0x08]);
    continuation.bytes(&[0x48, 0x89, 0xe7, 0x31, 0xf6]);
    continuation.bytes(&[0xb8, 0x23, 0x00, 0x00, 0x00, 0x0f, 0x05]); // nanosleep
    let nanosleep_return_address =
        continuation_address + u64::try_from(continuation.bytes.len()).unwrap_or(u64::MAX);
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "nanosleep-buffer-fail");
    continuation.bytes(&[0x48, 0x83, 0xc4, 0x10]); // release input
    continuation.bytes(&[0x48, 0x83, 0xec, 0x20]); // bounded stdout payload
    for (offset, value) in [
        (0_u8, u64::from_le_bytes(*b"TOPAL_KE")),
        (8, u64::from_le_bytes(*b"RNEL_USE")),
        (16, u64::from_le_bytes(*b"R_WRITE_")),
    ] {
        continuation.bytes(&[0x48, 0xb8]);
        continuation.bytes(&value.to_le_bytes());
        if offset == 0 {
            continuation.bytes(&[0x48, 0x89, 0x04, 0x24]);
        } else {
            continuation.bytes(&[0x48, 0x89, 0x44, 0x24, offset]);
        }
    }
    continuation.bytes(&[0x66, 0xc7, 0x44, 0x24, 0x18, b'O', b'K']);
    continuation.bytes(&[0xbf, 0x01, 0x00, 0x00, 0x00]); // stdout
    continuation.bytes(&[0x48, 0x89, 0xe6]); // candidate = rsp
    continuation.bytes(&[0xba, 0x1a, 0x00, 0x00, 0x00]); // 26 bytes
    continuation.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0x0f, 0x05]); // write
    continuation.bytes(&[0x48, 0x83, 0xf8, 0x1a]);
    continuation.jump_if(0x85, "write-buffer-fail");
    continuation.bytes(&[0x48, 0x83, 0xc4, 0x20]); // release stdout payload
    continuation.bytes(&[0x48, 0x83, 0xec, 0x10]); // bounded stdin byte
    continuation.bytes(&[0x31, 0xff]); // stdin
    continuation.bytes(&[0x48, 0x89, 0xe6]); // candidate = rsp
    continuation.bytes(&[0xba, 0x04, 0x00, 0x00, 0x00]); // four bytes
    continuation.bytes(&[0x31, 0xc0, 0x0f, 0x05]); // read
    continuation.bytes(&[0x48, 0x83, 0xf8, 0x04]);
    continuation.jump_if(0x85, "read-buffer-fail");
    continuation.bytes(&[0x81, 0x3c, 0x24]);
    continuation.bytes(&u32::from_le_bytes(*b"read").to_le_bytes());
    continuation.jump_if(0x85, "read-buffer-fail");
    continuation.bytes(&[0x48, 0x83, 0xc4, 0x10]); // release stdin byte
    continuation.bytes(&[0x31, 0xff]); // brk(0) query
    continuation.bytes(&[0xb8, 0x0c, 0x00, 0x00, 0x00, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x3d, 0x10, 0x50, 0x60, 0x00]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xbf, 0x20, 0x50, 0x60, 0x00]); // grow 16 bytes
    continuation.bytes(&[0xb8, 0x0c, 0x00, 0x00, 0x00, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x3d, 0x20, 0x50, 0x60, 0x00]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xc6, 0x04, 0x25, 0x10, 0x50, 0x60, 0x00, 0xa5]);
    continuation.bytes(&[0x80, 0x3c, 0x25, 0x10, 0x50, 0x60, 0x00, 0xa5]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xbf, 0x10, 0x50, 0x60, 0x00]); // shrink to initial
    continuation.bytes(&[0xb8, 0x0c, 0x00, 0x00, 0x00, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x3d, 0x10, 0x50, 0x60, 0x00]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xbf, 0x01, 0x60, 0x60, 0x00]); // beyond reserve
    continuation.bytes(&[0xb8, 0x0c, 0x00, 0x00, 0x00, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x3d, 0x10, 0x50, 0x60, 0x00]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xb8, 0x27, 0x00, 0x00, 0x00, 0x0f, 0x05]); // getpid
    continuation.bytes(&[0x48, 0x83, 0xf8, 0x01]);
    continuation.jump_if(0x85, "fail");
    for (number, expected) in [
        (110_u32, 0_u8), // getppid
        (186, 1),        // gettid
        (102, 0),        // getuid
        (107, 0),        // geteuid
        (104, 0),        // getgid
        (108, 0),        // getegid
    ] {
        continuation.bytes(&[0xb8]);
        continuation.bytes(&number.to_le_bytes());
        continuation.bytes(&[0x0f, 0x05]);
        if expected == 0 {
            continuation.bytes(&[0x48, 0x85, 0xc0]);
        } else {
            continuation.bytes(&[0x48, 0x83, 0xf8, expected]);
        }
        continuation.jump_if(0x85, "fail");
    }
    continuation.bytes(&[0x31, 0xff]);
    continuation.bytes(&[0xb8, 0xe7, 0x00, 0x00, 0x00, 0x0f, 0x05]); // exit_group(0)
    continuation.bytes(&[0x0f, 0x0b]);
    continuation.bind("uname-buffer-fail")?;
    continuation.bytes(&[0x48, 0x81, 0xc4, 0x00, 0x02, 0x00, 0x00]);
    continuation.jump("fail");
    continuation.bind("nanosleep-buffer-fail")?;
    continuation.bytes(&[0x48, 0x83, 0xc4, 0x10]);
    continuation.jump("fail");
    continuation.bind("write-buffer-fail")?;
    continuation.bytes(&[0x48, 0x83, 0xc4, 0x20]);
    continuation.jump("fail");
    continuation.bind("read-buffer-fail")?;
    continuation.bytes(&[0x48, 0x83, 0xc4, 0x10]);
    continuation.bind("fail")?;
    continuation.bytes(&[0x0f, 0x0b]);
    bytes.extend(continuation.finish()?);
    bytes.resize(bytes.len().next_multiple_of(8), 0x90);
    if bytes.len() > 4096 {
        return Err(CompileError::Tool(
            "constructed user entry payload exceeds its sealed executable page".into(),
        ));
    }
    Ok(InitialUserEntryPayload {
        bytes,
        continuation_address,
        uname_return_address,
        nanosleep_return_address,
    })
}

fn user_install_initial_entry_trampoline() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x04]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x83, 0x7f, 0x78, 0x00]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // retained Linux boot_params
    code.jump_if(0x84, "fail");
    emit_initial_boot_resource_handoff_validation(&mut code)?;
    let code_file_size_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + INITIAL_USER_CODE_FILE_SIZE_ELF_OFFSET,
    )
    .map_err(|_| CompileError::Tool("external code-size offset exceeds u32".into()))?;
    code.bytes(&[0x49, 0x8b, 0x88]); // validated runtime extent
    code.bytes(&code_file_size_offset.to_le_bytes());
    code.bytes(&[0x49, 0x81, 0xc0]); // `init` payload in r8
    code.bytes(&X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET.to_le_bytes());
    code.bytes(&[0x49, 0xba]);
    code.bytes(&INITIAL_USER_CODE_ADDRESS.to_le_bytes());
    code.bind("copy-loop")?;
    code.bytes(&[0x49, 0x8b, 0x00]);
    code.bytes(&[0x49, 0x89, 0x02]);
    code.bytes(&[0x49, 0x83, 0xc0, 0x08]);
    code.bytes(&[0x49, 0x83, 0xc2, 0x08]);
    code.bytes(&[0x48, 0x83, 0xe9, 0x08]);
    code.jump_if(0x85, "copy-loop");
    code.bytes(&[0xc6, 0x87, 0x70, 0x00, 0x00, 0x00, 0x05]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn user_protect_initial_mapping_read_only() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x05]);
    code.jump_if(0x84, "generation-one");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x12]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0xb8, 0x13, 0x00, 0x00, 0x00]);
    code.jump("generation-accepted");
    code.bind("generation-one")?;
    code.bytes(&[0xb8, 0x0c, 0x00, 0x00, 0x00]);
    code.bind("generation-accepted")?;
    code.bytes(&[0x3c, 0x0c]);
    code.jump_if(0x84, "check-generation-one-frame");
    code.bytes(&[0x48, 0x83, 0xbf, 0x80, 0x00, 0x00, 0x00, 0x00]);
    code.jump_if(0x84, "fail");
    code.jump("frame-present");
    code.bind("check-generation-one-frame")?;
    code.bytes(&[0x48, 0x83, 0x7f, 0x78, 0x00]);
    code.jump_if(0x84, "fail");
    code.bind("frame-present")?;
    code.bytes(&[0x48, 0x8b, 0x4f, 0x68]);
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xc1, 0x00, 0x20, 0x00, 0x00]);
    code.bytes(&[0x3c, 0x0c]);
    code.jump_if(0x84, "pte-selected");
    code.bytes(&[0x48, 0x83, 0xc1, 0x08]);
    code.bind("pte-selected")?;
    code.bytes(&[0x4c, 0x8b, 0x41, 0x20]);
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x4d, 0x89, 0xc1, 0x41, 0x83, 0xe1, 0x07]);
    code.bytes(&[0x41, 0x80, 0xf9, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x83, 0xe0, 0xfd]);
    code.bytes(&[0x4c, 0x89, 0x41, 0x20]);
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x3c, 0x0c]);
    code.jump_if(0x84, "invalidate-generation-one");
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x0060_5000_u64.to_le_bytes());
    code.jump("invalidate-mapping");
    code.bind("invalidate-generation-one")?;
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x0060_4000_u64.to_le_bytes());
    code.bind("invalidate-mapping")?;
    code.bytes(&[0x41, 0x0f, 0x01, 0x3a]);
    code.bytes(&[0x88, 0x87, 0x70, 0x00, 0x00, 0x00]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn user_load_initial_mapping_byte() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x13]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // retained Linux boot_params
    code.jump_if(0x84, "fail");
    emit_initial_boot_resource_handoff_validation(&mut code)?;
    let data_payload_offset = X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET
        .checked_add(
            u32::try_from(INITIAL_USER_DATA_FILE_OFFSET)
                .map_err(|_| CompileError::Tool("external user data offset exceeds u32".into()))?,
        )
        .ok_or_else(|| CompileError::Tool("external user data offset overflow".into()))?;
    let data_file_size_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET,
    )
    .map_err(|_| CompileError::Tool("external data-size offset exceeds u32".into()))?;
    let data_memory_size_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
            + INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET,
    )
    .map_err(|_| CompileError::Tool("external BSS-size offset exceeds u32".into()))?;
    code.bytes(&[0x4d, 0x8b, 0x88]); // runtime p_filesz
    code.bytes(&data_file_size_offset.to_le_bytes());
    code.bytes(&[0x4d, 0x8b, 0x98]); // runtime p_memsz
    code.bytes(&data_memory_size_offset.to_le_bytes());
    code.bytes(&[0x4d, 0x29, 0xcb]); // BSS bytes = p_memsz - p_filesz
    code.bytes(&[0x49, 0x81, 0xc0]);
    code.bytes(&data_payload_offset.to_le_bytes());
    code.bytes(&[0xb8, 0x14, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x8b, 0x4f, 0x68]);
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xc1, 0x18, 0x20, 0x00, 0x00]);
    code.bytes(&[0x49, 0xba]);
    code.bytes(&INITIAL_USER_DATA_ADDRESS.to_le_bytes());
    code.bind("data-compare-loop")?;
    code.bytes(&[0x49, 0x8b, 0x00]);
    code.bytes(&[0x49, 0x39, 0x02]);
    code.jump_if(0x85, "cleanup-fail");
    code.bytes(&[0x49, 0x83, 0xc0, 0x08]);
    code.bytes(&[0x49, 0x83, 0xc2, 0x08]);
    code.bytes(&[0x49, 0x83, 0xe9, 0x08]);
    code.jump_if(0x85, "data-compare-loop");
    code.bytes(&[0x4d, 0x85, 0xdb]);
    code.jump_if(0x84, "data-verified");
    code.bind("bss-compare-loop")?;
    code.bytes(&[0x49, 0x83, 0x3a, 0x00]);
    code.jump_if(0x85, "cleanup-fail");
    code.bytes(&[0x49, 0x83, 0xc2, 0x08]);
    code.bytes(&[0x49, 0x83, 0xeb, 0x08]);
    code.jump_if(0x85, "bss-compare-loop");
    code.bind("data-verified")?;
    code.bytes(&[0xc6, 0x87, 0x70, 0x00, 0x00, 0x00, 0x14]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);

    code.bind("cleanup-fail")?;
    code.bytes(&[0x48, 0xc7, 0x41, 0x10, 0, 0, 0, 0]);
    code.bytes(&[0x0f, 0xae, 0xf0, 0x41, 0x0f, 0x01, 0x3a]);
    code.bytes(&[0x48, 0xc7, 0x87, 0x80, 0x00, 0x00, 0x00, 0, 0, 0, 0]);
    code.bytes(&[0xc6, 0x87, 0x70, 0x00, 0x00, 0x00, 0x0e]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn user_verify_initial_entry_trampoline() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x0c]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x83, 0x7f, 0x78, 0x00]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // retained Linux boot_params
    code.jump_if(0x84, "fail");
    emit_initial_boot_resource_handoff_validation(&mut code)?;
    let code_file_size_offset = u32::try_from(
        X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + INITIAL_USER_CODE_FILE_SIZE_ELF_OFFSET,
    )
    .map_err(|_| CompileError::Tool("external code-size offset exceeds u32".into()))?;
    code.bytes(&[0x49, 0x8b, 0x88]); // validated runtime extent
    code.bytes(&code_file_size_offset.to_le_bytes());
    code.bytes(&[0x49, 0x81, 0xc0]); // `init` payload in r8
    code.bytes(&X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET.to_le_bytes());
    code.bytes(&[0x49, 0xba]);
    code.bytes(&INITIAL_USER_CODE_ADDRESS.to_le_bytes());
    code.bind("compare-loop")?;
    code.bytes(&[0x49, 0x8b, 0x00]);
    code.bytes(&[0x49, 0x39, 0x02]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x83, 0xc0, 0x08]);
    code.bytes(&[0x49, 0x83, 0xc2, 0x08]);
    code.bytes(&[0x48, 0x83, 0xe9, 0x08]);
    code.jump_if(0x85, "compare-loop");
    code.bytes(&[0xc6, 0x87, 0x70, 0x00, 0x00, 0x00, 0x06]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn user_permit_initial_mapping_execute() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x06]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x83, 0x7f, 0x78, 0x00]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x8b, 0x4f, 0x68]);
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xc1, 0x10, 0x20, 0x00, 0x00]);
    code.bytes(&[0x4c, 0x8b, 0x41, 0x10]);
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x41, 0xf6, 0xc0, 0x02]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x0f, 0xba, 0xe0, 0x3f]);
    code.jump_if(0x83, "fail");
    code.bytes(&[0x49, 0x0f, 0xba, 0xf0, 0x3f]);
    code.bytes(&[0x4c, 0x89, 0x41, 0x10]);
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x0060_4000_u64.to_le_bytes());
    code.bytes(&[0x41, 0x0f, 0x01, 0x3a]);
    code.bytes(&[0xc6, 0x87, 0x70, 0x00, 0x00, 0x00, 0x0e]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn user_seal_initial_mapping_immutable_data() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x14]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x83, 0xbf, 0x80, 0x00, 0x00, 0x00, 0x00]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x8b, 0x4f, 0x68]);
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xc1, 0x18, 0x20, 0x00, 0x00]);
    code.bytes(&[0x4c, 0x8b, 0x41, 0x10]);
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x41, 0xf6, 0xc0, 0x02]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x0f, 0xba, 0xe0, 0x3f]);
    code.jump_if(0x83, "fail");
    code.bytes(&[0xc6, 0x87, 0x70, 0x00, 0x00, 0x00, 0x15]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn user_permit_initial_mapping_writable_data() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x14]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x83, 0xbf, 0x80, 0x00, 0x00, 0x00, 0x00]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x8b, 0x4f, 0x68]);
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xc1, 0x18, 0x20, 0x00, 0x00]);
    code.bytes(&[0x4c, 0x8b, 0x41, 0x10]);
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x41, 0xf6, 0xc0, 0x02]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x0f, 0xba, 0xe0, 0x3f]);
    code.jump_if(0x83, "fail");
    code.bytes(&[0x49, 0x83, 0xc8, 0x02]);
    code.bytes(&[0x4c, 0x89, 0x41, 0x10]);
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x49, 0xba]);
    code.bytes(&INITIAL_USER_DATA_ADDRESS.to_le_bytes());
    code.bytes(&[0x41, 0x0f, 0x01, 0x3a]);
    code.bytes(&[0xc6, 0x87, 0x70, 0x00, 0x00, 0x00, 0x16]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn user_unmap_initial_frame() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x0e]);
    code.jump_if(0x84, "unmap-code");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x15]);
    code.jump_if(0x84, "unmap-data");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x16]);
    code.jump_if(0x85, "fail");
    code.bind("unmap-data")?;
    code.bytes(&[0x48, 0xc7, 0x87, 0x90, 0x00, 0x00, 0x00, 0, 0, 0, 0]);
    code.bytes(&[0x48, 0xc7, 0x87, 0x98, 0x00, 0x00, 0x00, 0, 0, 0, 0]);
    code.bytes(&[0xb8, 0x0e, 0x00, 0x00, 0x00]);
    code.bytes(&[0x4c, 0x8b, 0x87, 0x80, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x8b, 0x4f, 0x68]);
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xc1, 0x18, 0x20, 0x00, 0x00]);
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x0060_5000_u64.to_le_bytes());
    code.jump("mapping-selected");
    code.bind("unmap-code")?;
    code.bytes(&[0xb8, 0x0b, 0x00, 0x00, 0x00]);
    code.bytes(&[0x4c, 0x8b, 0x47, 0x78]);
    code.bytes(&[0x48, 0x8b, 0x4f, 0x68]);
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xc1, 0x10, 0x20, 0x00, 0x00]);
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x0060_4000_u64.to_le_bytes());
    code.bind("mapping-selected")?;
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0xc7, 0x41, 0x10, 0, 0, 0, 0]);
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x41, 0x0f, 0x01, 0x3a]);
    code.bytes(&[0x3c, 0x0e]);
    code.jump_if(0x84, "clear-data-frame");
    code.bytes(&[0x48, 0xc7, 0x47, 0x78, 0, 0, 0, 0]);
    code.jump("frame-cleared");
    code.bind("clear-data-frame")?;
    code.bytes(&[0x48, 0xc7, 0x87, 0x80, 0x00, 0x00, 0x00, 0, 0, 0, 0]);
    code.bind("frame-cleared")?;
    code.bytes(&[0x88, 0x87, 0x70, 0x00, 0x00, 0x00]);
    code.bind("returned")?;
    code.bytes(&[0x4c, 0x89, 0xc0, 0xc3]);

    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn user_prepare_initial_probe() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x3f, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x16]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0xb9]);
    code.bytes(&0xc000_0080_u32.to_le_bytes()); // IA32_EFER
    code.bytes(&[0x0f, 0x32, 0x83, 0xc8, 0x01, 0x0f, 0x30]); // enable SCE
    emit_wrmsr_immediate(&mut code, 0xc000_0081, 0x001b_0010_0000_0000); // STAR
    emit_wrmsr_register(&mut code, 0xc000_0082, &[0x48, 0x89, 0xf0]); // LSTAR = RSI
    emit_wrmsr_immediate(&mut code, 0xc000_0084, 0x200); // mask IF on entry
    emit_wrmsr_register(&mut code, 0xc000_0102, &[0x48, 0x89, 0xf8]); // kernel GS base
    code.bytes(&[0xc6, 0x07, 0x01]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn user_complete_initial_backing() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x3f, 0x08]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x4c, 0x8b, 0x4f, 0x68]); // retained backing base
    code.bytes(&[0x4d, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x0f, 0x20, 0xd8]);
    code.bytes(&[0x48, 0x25, 0x00, 0xf0, 0xff, 0xff]);
    code.bytes(&[0x48, 0xc7, 0x80, 0x18, 0x20, 0x00, 0x00, 0, 0, 0, 0]);
    code.bytes(&[0x0f, 0xae, 0xf0]);
    for address in [0x0060_0000_u64, 0x0060_1000, 0x0060_2000] {
        code.bytes(&[0x48, 0xba]);
        code.bytes(&address.to_le_bytes());
        code.bytes(&[0x0f, 0x01, 0x3a]); // invlpg [rdx]
    }
    code.bytes(&[0xc6, 0x87, 0x70, 0x00, 0x00, 0x00, 0x02]);
    code.bytes(&[0xc6, 0x07, 0x09]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn user_reclaim_initial_address_space() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x3f, 0x09]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x02]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x8b, 0x47, 0x68]);
    code.bytes(&[0x48, 0x85, 0xc0]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0xc7, 0x47, 0x68, 0, 0, 0, 0]);
    code.bytes(&[0xc6, 0x87, 0x70, 0x00, 0x00, 0x00, 0x03]);
    code.bytes(&[0xc6, 0x07, 0x0a, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn initial_user_probe() -> Result<Vec<u8>, CompileError> {
    let continuation_address = initial_user_entry_payload()?.continuation_address;
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0xb8, 0x27, 0x00, 0x00, 0x00, 0x0f, 0x05]); // getpid
    code.bytes(&[0x48, 0x83, 0xf8, 0x01]);
    code.jump_if(0x85, "probe-fail");

    code.bytes(&[0x48, 0x81, 0xec, 0x00, 0x02, 0x00, 0x00]); // 512-byte buffer
    code.bytes(&[0x48, 0x89, 0xe7]);
    code.bytes(&[0xb8, 0x3f, 0x00, 0x00, 0x00, 0x0f, 0x05]); // uname(valid)
    code.bytes(&[0x48, 0x85, 0xc0]);
    code.jump_if(0x85, "probe-fail");
    code.bytes(&[0x81, 0x3c, 0x24]);
    code.bytes(&u32::from_le_bytes(*b"Linu").to_le_bytes());
    code.jump_if(0x85, "probe-fail");
    code.bytes(&[0x80, 0x7c, 0x24, 0x04, b'x']);
    code.jump_if(0x85, "probe-fail");
    code.bytes(&[0x81, 0xbc, 0x24]);
    code.bytes(&260_u32.to_le_bytes());
    code.bytes(&u32::from_le_bytes(*b"x86_").to_le_bytes());
    code.jump_if(0x85, "probe-fail");
    code.bytes(&[0x48, 0x81, 0xc4, 0x00, 0x02, 0x00, 0x00]);

    code.bytes(&[0xbf]);
    code.bytes(&0x0060_1f00_u32.to_le_bytes());
    code.bytes(&[0xb8, 0x3f, 0x00, 0x00, 0x00, 0x0f, 0x05]); // uname(guard crossing)
    code.bytes(&[0x48, 0x83, 0xf8, 0xf2]); // -EFAULT
    code.jump_if(0x85, "probe-fail");

    code.bytes(&[0x48, 0x83, 0xec, 0x10]); // native timespec staging
    code.bytes(&[0x31, 0xc0, 0x48, 0x89, 0x04, 0x24]);
    code.bytes(&[0x48, 0x89, 0x44, 0x24, 0x08]);
    code.bytes(&[0x48, 0x89, 0xe7, 0x31, 0xf6]);
    code.bytes(&[0xb8, 0x23, 0x00, 0x00, 0x00, 0x0f, 0x05]); // nanosleep(valid zero)
    code.bytes(&[0x48, 0x85, 0xc0]);
    code.jump_if(0x85, "probe-fail");
    code.bytes(&[0x48, 0x83, 0xc4, 0x10]);

    code.bytes(&[0xbf]);
    code.bytes(&0x0060_1ff8_u32.to_le_bytes());
    code.bytes(&[0x31, 0xf6, 0xb8, 0x23, 0x00, 0x00, 0x00, 0x0f, 0x05]);
    code.bytes(&[0x48, 0x83, 0xf8, 0xf2]); // -EFAULT
    code.jump_if(0x85, "probe-fail");

    code.bytes(&[0x48, 0xb8]);
    code.bytes(&continuation_address.to_le_bytes());
    code.bytes(&[0xff, 0xe0, 0x0f, 0x0b]);
    code.bind("probe-fail")?;
    code.bytes(&[
        0xbf, 0x01, 0x00, 0x00, 0x00, 0xb8, 0xe7, 0x00, 0x00, 0x00, 0x0f, 0x05, 0x0f, 0x0b,
    ]);
    code.finish()
}

fn emit_wrmsr_immediate(code: &mut X86FunctionEncoder, msr: u32, value: u64) {
    code.bytes(&[0xb9]);
    code.bytes(&msr.to_le_bytes());
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&value.to_le_bytes());
    code.bytes(&[0x48, 0x89, 0xc2, 0x48, 0xc1, 0xea, 0x20, 0x0f, 0x30]);
}

fn emit_wrmsr_register(code: &mut X86FunctionEncoder, msr: u32, load_rax: &[u8]) {
    code.bytes(&[0xb9]);
    code.bytes(&msr.to_le_bytes());
    code.bytes(load_rax);
    code.bytes(&[0x48, 0x89, 0xc2, 0x48, 0xc1, 0xea, 0x20, 0x0f, 0x30]);
}

fn user_enter_initial_context() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x3f, 0x01]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x89, 0x67, 0x08]); // continuation RSP
    code.bytes(&[0xc6, 0x07, 0x02]);
    code.bytes(&[0x6a, 0x23]); // user SS
    code.bytes(&[0x48, 0xb8]);
    code.bytes(&INITIAL_USER_STACK_ADDRESS.to_le_bytes());
    code.bytes(&[0x50]); // user RSP
    code.bytes(&[0x6a, 0x02]); // user RFLAGS, interrupts masked
    code.bytes(&[0x6a, 0x2b]); // user CS
    code.bytes(&[0x48, 0x8b, 0x87, 0x88, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x85, 0xc0]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x50, 0x48, 0xcf]); // user RIP; iretq
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn user_dispatch_initial_syscall() -> Result<Vec<u8>, CompileError> {
    let payload = initial_user_entry_payload()?;
    let uname_return_address = u32::try_from(payload.uname_return_address).map_err(|_| {
        CompileError::Tool("constructed uname return address exceeds x86 imm32".into())
    })?;
    let nanosleep_return_address =
        u32::try_from(payload.nanosleep_return_address).map_err(|_| {
            CompileError::Tool("constructed nanosleep return address exceeds x86 imm32".into())
        })?;
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x0f, 0x01, 0xf8]); // swapgs
    code.bytes(&[0x65, 0x48, 0x89, 0x24, 0x25, 0x10, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0x48, 0x89, 0x0c, 0x25, 0x30, 0x00, 0x00, 0x00]); // user RCX
    code.bytes(&[0x65, 0x4c, 0x89, 0x1c, 0x25, 0x38, 0x00, 0x00, 0x00]); // user R11
    code.bytes(&[0x65, 0x4c, 0x89, 0x04, 0x25, 0x40, 0x00, 0x00, 0x00]); // user R8
    code.bytes(&[0x65, 0x4c, 0x89, 0x0c, 0x25, 0x48, 0x00, 0x00, 0x00]); // user R9
    code.bytes(&[0x65, 0x4c, 0x89, 0x14, 0x25, 0x50, 0x00, 0x00, 0x00]); // user R10
    code.bytes(&[0x65, 0x48, 0x89, 0x14, 0x25, 0x58, 0x00, 0x00, 0x00]); // user RDX
    code.bytes(&[0x65, 0x48, 0x8b, 0x24, 0x25, 0x08, 0x00, 0x00, 0x00]);
    code.bytes(&[0x85, 0xc0]);
    code.jump_if(0x84, "read");
    code.bytes(&[0x83, 0xf8, 0x01]);
    code.jump_if(0x84, "write");
    code.bytes(&[0x83, 0xf8, 0x0c]);
    code.jump_if(0x84, "brk");
    code.bytes(&[0x83, 0xf8, 0x27]);
    code.jump_if(0x84, "getpid");
    for number in [102_u8, 104, 107, 108, 110] {
        code.bytes(&[0x83, 0xf8, number]);
        code.jump_if(0x84, "identity-zero");
    }
    code.bytes(&[0x3d, 0xba, 0x00, 0x00, 0x00]);
    code.jump_if(0x84, "identity-one");
    code.bytes(&[0x83, 0xf8, 0x3f]);
    code.jump_if(0x84, "uname");
    code.bytes(&[0x83, 0xf8, 0x23]);
    code.jump_if(0x84, "nanosleep");
    code.bytes(&[0x3d, 0xe7, 0x00, 0x00, 0x00]);
    code.jump_if(0x84, "exit-group");
    code.jump("fail");

    code.bind("getpid")?;
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0060_0007_u32.to_le_bytes());
    code.jump_if(0x84, "getpid-provider-probe");
    code.bytes(&[0x41, 0xba, 0x01, 0x00, 0x00, 0x00]);
    code.jump("scalar-identity");

    code.bind("identity-zero")?;
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.jump("scalar-identity");
    code.bind("identity-one")?;
    code.bytes(&[0x41, 0xba, 0x01, 0x00, 0x00, 0x00]);
    code.bind("scalar-identity")?;
    code.bytes(&[0x48, 0x81, 0xf9, 0x00, 0x40, 0x60, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x81, 0xf9, 0x00, 0x50, 0x60, 0x00]);
    code.jump_if(0x83, "fail");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x00, 0x00, 0x00, 0x00, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x4c, 0x89, 0xd0]);
    code.jump("return-user");

    code.bind("getpid-provider-probe")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x00, 0x00, 0x00, 0x00, 0x02]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x00, 0x00, 0x00, 0x00, 0x03]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00]);
    code.jump("return-user");

    code.bind("brk")?;
    code.bytes(&[0x48, 0x81, 0xf9, 0x00, 0x40, 0x60, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x81, 0xf9, 0x00, 0x50, 0x60, 0x00]);
    code.jump_if(0x83, "fail");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x00, 0x00, 0x00, 0x00, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25, 0x90, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0x4c, 0x8b, 0x04, 0x25, 0x98, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x85, 0xc0]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x4c, 0x39, 0xc0]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x3d, 0x00, 0x60, 0x60, 0x00]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "return-user");
    code.bytes(&[0x4c, 0x39, 0xc7]);
    code.jump_if(0x82, "return-user");
    code.bytes(&[0x48, 0x81, 0xff, 0x00, 0x60, 0x60, 0x00]);
    code.jump_if(0x87, "return-user");
    code.bytes(&[0x65, 0x48, 0x89, 0x3c, 0x25, 0x90, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x89, 0xf8]);
    code.jump("return-user");

    code.bind("uname")?;
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&uname_return_address.to_le_bytes());
    code.jump_if(0x84, "uname-constructed");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x00, 0x00, 0x00, 0x00, 0x03]);
    code.jump_if(0x84, "uname-valid");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x00, 0x00, 0x00, 0x00, 0x04]);
    code.jump_if(0x84, "uname-fault");
    code.jump("fail");

    code.bind("uname-constructed")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x00, 0x00, 0x00, 0x00, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xff]);
    code.bytes(&INITIAL_USER_UNAME_CANDIDATE.to_le_bytes());
    code.jump_if(0x85, "fail");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x19, 0x00, 0x00, 0x00, 0x07]);
    code.jump("copy-to-user");

    code.bind("uname-valid")?;
    code.bytes(&[0x48, 0x81, 0xff]);
    code.bytes(&INITIAL_USER_UNAME_CANDIDATE.to_le_bytes());
    code.jump_if(0x85, "fail");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x19, 0x00, 0x00, 0x00, 0x04]);
    code.jump("copy-to-user");

    code.bind("uname-fault")?;
    code.bytes(&[0x48, 0x81, 0xff]);
    code.bytes(&0x0060_1f00_u32.to_le_bytes());
    code.jump_if(0x85, "fail");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x19, 0x00, 0x00, 0x00, 0x05]);

    code.bind("copy-to-user")?;
    code.bytes(&[0x49, 0x89, 0xf8]); // destination candidate
    code.rip_lea(&[0x4c, 0x8d, 0x0d], "uts-data"); // source bytes in R9
    code.bytes(&[0x45, 0x31, 0xd2]); // copied prefix in R10
    code.rip_lea(&[0x48, 0x8d, 0x05], "copy-fault-site");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x20, 0x00, 0x00, 0x00]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "copy-recovery");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x28, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x01]);
    code.bind("copy-loop")?;
    code.bytes(&[0x43, 0x8a, 0x04, 0x11]); // source[index]
    code.bind("copy-fault-site")?;
    code.bytes(&[0x43, 0x88, 0x04, 0x10]); // user candidate[index]
    code.bytes(&[0x49, 0xff, 0xc2]);
    code.bytes(&[0x49, 0x81, 0xfa]);
    code.bytes(&390_u32.to_le_bytes());
    code.jump_if(0x85, "copy-loop");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0x31, 0xc0]);
    code.jump("copy-complete");

    code.bind("copy-recovery")?;
    code.bytes(&[0x65, 0x4c, 0x89, 0x14, 0x25, 0x60, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0xc7, 0xc0, 0xf2, 0xff, 0xff, 0xff]); // -EFAULT
    code.bind("copy-complete")?;
    code.bytes(&[0x65, 0x8a, 0x14, 0x25, 0x19, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0x88, 0x14, 0x25, 0x00, 0x00, 0x00, 0x00]);
    code.jump("return-user");

    code.bind("nanosleep")?;
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&nanosleep_return_address.to_le_bytes());
    code.jump_if(0x84, "nanosleep-constructed");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x00, 0x00, 0x00, 0x00, 0x05]);
    code.jump_if(0x84, "nanosleep-valid");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x00, 0x00, 0x00, 0x00, 0x06]);
    code.jump_if(0x84, "nanosleep-fault");
    code.jump("fail");

    code.bind("nanosleep-constructed")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x00, 0x00, 0x00, 0x00, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xff]);
    code.bytes(&INITIAL_USER_NANOSLEEP_CANDIDATE.to_le_bytes());
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x19, 0x00, 0x00, 0x00, 0x07]);
    code.jump("copy-from-user");

    code.bind("nanosleep-valid")?;
    code.bytes(&[0x48, 0x81, 0xff]);
    code.bytes(&INITIAL_USER_NANOSLEEP_CANDIDATE.to_le_bytes());
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x19, 0x00, 0x00, 0x00, 0x06]);
    code.jump("copy-from-user");

    code.bind("nanosleep-fault")?;
    code.bytes(&[0x48, 0x81, 0xff]);
    code.bytes(&0x0060_1ff8_u32.to_le_bytes());
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x19, 0x00, 0x00, 0x00, 0x07]);

    code.bind("copy-from-user")?;
    code.bytes(&[0x48, 0x83, 0xec, 0x10]); // private kernel staging
    code.bytes(&[0x49, 0x89, 0xf8]); // source candidate
    code.bytes(&[0x45, 0x31, 0xd2]); // copied prefix in R10
    code.rip_lea(&[0x48, 0x8d, 0x05], "copy-from-fault-site");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x20, 0x00, 0x00, 0x00]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "copy-from-recovery");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x28, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x02]);
    code.bind("copy-from-loop")?;
    code.bind("copy-from-fault-site")?;
    code.bytes(&[0x43, 0x8a, 0x04, 0x10]); // user candidate[index]
    code.bytes(&[0x42, 0x88, 0x04, 0x14]); // private staging[index]
    code.bytes(&[0x49, 0xff, 0xc2]);
    code.bytes(&[0x49, 0x83, 0xfa, 0x10]);
    code.jump_if(0x85, "copy-from-loop");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x83, 0x3c, 0x24, 0x00]);
    code.jump_if(0x85, "copy-from-invalid");
    code.bytes(&[0x48, 0x83, 0x7c, 0x24, 0x08, 0x00]);
    code.jump_if(0x85, "copy-from-invalid");
    code.bytes(&[0x48, 0x83, 0xc4, 0x10, 0x31, 0xc0]);
    code.jump("copy-from-complete");

    code.bind("copy-from-recovery")?;
    code.bytes(&[0x65, 0x4c, 0x89, 0x14, 0x25, 0x60, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x83, 0xc4, 0x10]);
    code.bytes(&[0x48, 0xc7, 0xc0, 0xf2, 0xff, 0xff, 0xff]); // -EFAULT
    code.bind("copy-from-complete")?;
    code.bytes(&[0x65, 0x8a, 0x14, 0x25, 0x19, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0x88, 0x14, 0x25, 0x00, 0x00, 0x00, 0x00]);
    code.jump("return-user");

    code.bind("copy-from-invalid")?;
    code.bytes(&[0x48, 0x83, 0xc4, 0x10]);
    code.jump("fail");

    code.bind("write")?;
    code.bytes(&[0x48, 0x81, 0xf9, 0x00, 0x40, 0x60, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x81, 0xf9, 0x00, 0x50, 0x60, 0x00]);
    code.jump_if(0x83, "fail");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x00, 0x00, 0x00, 0x00, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x83, 0xff, 0x01]);
    code.jump_if(0x85, "write-ebadf");
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x84, "write-einval");
    code.bytes(&[0x48, 0x81, 0xfa, 0x00, 0x01, 0x00, 0x00]);
    code.jump_if(0x87, "write-einval");
    code.bytes(&[0x49, 0x89, 0xf0]); // source candidate
    code.bytes(&[0x49, 0x89, 0xf1, 0x49, 0x01, 0xd1]); // exclusive end
    code.jump_if(0x82, "write-efault");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x00, 0x60, 0x00]);
    code.jump_if(0x82, "write-second-range");
    code.bytes(&[0x49, 0x81, 0xf9, 0x00, 0x20, 0x60, 0x00]);
    code.jump_if(0x86, "write-candidate-ok");
    code.bind("write-second-range")?;
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x40, 0x60, 0x00]);
    code.jump_if(0x82, "write-efault");
    code.bytes(&[0x49, 0x81, 0xf9, 0x00, 0x60, 0x60, 0x00]);
    code.jump_if(0x87, "write-efault");
    code.bind("write-candidate-ok")?;
    code.bytes(&[0x48, 0x81, 0xec, 0x08, 0x01, 0x00, 0x00]); // 256 + length
    code.bytes(&[0x48, 0x89, 0x94, 0x24, 0x00, 0x01, 0x00, 0x00]);
    code.bytes(&[0x45, 0x31, 0xd2]); // copied prefix
    code.rip_lea(&[0x48, 0x8d, 0x05], "write-copy-fault-site");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x20, 0x00, 0x00, 0x00]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "write-copy-recovery");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x28, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x02]);
    code.bind("write-copy-loop")?;
    code.bind("write-copy-fault-site")?;
    code.bytes(&[0x43, 0x8a, 0x04, 0x10]);
    code.bytes(&[0x42, 0x88, 0x04, 0x14]);
    code.bytes(&[0x49, 0xff, 0xc2]);
    code.bytes(&[0x4c, 0x3b, 0x94, 0x24, 0x00, 0x01, 0x00, 0x00]);
    code.jump_if(0x85, "write-copy-loop");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.bind("write-uart-loop")?;
    code.bytes(&[0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20]);
    code.jump_if(0x84, "write-uart-loop");
    code.bytes(&[0x66, 0xba, 0xf8, 0x03]);
    code.bytes(&[0x42, 0x8a, 0x04, 0x14, 0xee]);
    code.bytes(&[0x49, 0xff, 0xc2]);
    code.bytes(&[0x4c, 0x3b, 0x94, 0x24, 0x00, 0x01, 0x00, 0x00]);
    code.jump_if(0x85, "write-uart-loop");
    code.bytes(&[0x48, 0x8b, 0x84, 0x24, 0x00, 0x01, 0x00, 0x00]);
    code.bytes(&[0x48, 0x81, 0xc4, 0x08, 0x01, 0x00, 0x00]);
    code.jump("return-user");

    code.bind("write-copy-recovery")?;
    code.bytes(&[0x65, 0x4c, 0x89, 0x14, 0x25, 0x60, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x81, 0xc4, 0x08, 0x01, 0x00, 0x00]);
    code.bind("write-efault")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xf2, 0xff, 0xff, 0xff]);
    code.jump("return-user");
    code.bind("write-ebadf")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xf7, 0xff, 0xff, 0xff]);
    code.jump("return-user");
    code.bind("write-einval")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xea, 0xff, 0xff, 0xff]);
    code.jump("return-user");

    code.bind("read")?;
    code.bytes(&[0x48, 0x81, 0xf9, 0x00, 0x40, 0x60, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x81, 0xf9, 0x00, 0x50, 0x60, 0x00]);
    code.jump_if(0x83, "fail");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x00, 0x00, 0x00, 0x00, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x85, "read-ebadf");
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x84, "read-einval");
    code.bytes(&[0x48, 0x81, 0xfa, 0x00, 0x01, 0x00, 0x00]);
    code.jump_if(0x87, "read-einval");
    code.bytes(&[0x49, 0x89, 0xf0]); // destination candidate
    code.bytes(&[0x49, 0x89, 0xf1, 0x49, 0x01, 0xd1]); // exclusive end
    code.jump_if(0x82, "read-efault");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x10, 0x60, 0x00]);
    code.jump_if(0x82, "read-second-range");
    code.bytes(&[0x49, 0x81, 0xf9, 0x00, 0x20, 0x60, 0x00]);
    code.jump_if(0x86, "read-candidate-ok");
    code.bind("read-second-range")?;
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x50, 0x60, 0x00]);
    code.jump_if(0x82, "read-efault");
    code.bytes(&[0x49, 0x81, 0xf9, 0x00, 0x60, 0x60, 0x00]);
    code.jump_if(0x87, "read-efault");
    code.bind("read-candidate-ok")?;
    code.bytes(&[0x48, 0x81, 0xec, 0x08, 0x01, 0x00, 0x00]); // 256 + length
    code.bytes(&[0x48, 0x89, 0x94, 0x24, 0x00, 0x01, 0x00, 0x00]);
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.bind("read-receive-loop")?;
    code.bind("read-uart-wait")?;
    code.bytes(&[0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x01]);
    code.jump_if(0x84, "read-uart-wait");
    code.bytes(&[0x66, 0xba, 0xf8, 0x03, 0xec]);
    code.bytes(&[0x42, 0x88, 0x04, 0x14]);
    code.bytes(&[0x49, 0xff, 0xc2]);
    code.bytes(&[0x4c, 0x3b, 0x94, 0x24, 0x00, 0x01, 0x00, 0x00]);
    code.jump_if(0x85, "read-receive-loop");
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "read-copy-fault-site");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x20, 0x00, 0x00, 0x00]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "read-copy-recovery");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x28, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x01]);
    code.bind("read-copy-loop")?;
    code.bytes(&[0x42, 0x8a, 0x04, 0x14]);
    code.bind("read-copy-fault-site")?;
    code.bytes(&[0x43, 0x88, 0x04, 0x10]);
    code.bytes(&[0x49, 0xff, 0xc2]);
    code.bytes(&[0x4c, 0x3b, 0x94, 0x24, 0x00, 0x01, 0x00, 0x00]);
    code.jump_if(0x85, "read-copy-loop");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x8b, 0x84, 0x24, 0x00, 0x01, 0x00, 0x00]);
    code.bytes(&[0x48, 0x81, 0xc4, 0x08, 0x01, 0x00, 0x00]);
    code.jump("return-user");

    code.bind("read-copy-recovery")?;
    code.bytes(&[0x65, 0x4c, 0x89, 0x14, 0x25, 0x60, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x81, 0xc4, 0x08, 0x01, 0x00, 0x00]);
    code.bind("read-efault")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xf2, 0xff, 0xff, 0xff]);
    code.jump("return-user");
    code.bind("read-ebadf")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xf7, 0xff, 0xff, 0xff]);
    code.jump("return-user");
    code.bind("read-einval")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xea, 0xff, 0xff, 0xff]);
    code.jump("return-user");

    code.bind("return-user")?;
    code.bytes(&[0x65, 0x4c, 0x8b, 0x04, 0x25, 0x40, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0x4c, 0x8b, 0x0c, 0x25, 0x48, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0x4c, 0x8b, 0x14, 0x25, 0x50, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0x48, 0x8b, 0x14, 0x25, 0x58, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0x48, 0x8b, 0x0c, 0x25, 0x30, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0x4c, 0x8b, 0x1c, 0x25, 0x38, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0x48, 0x8b, 0x24, 0x25, 0x10, 0x00, 0x00, 0x00]);
    code.bytes(&[0x0f, 0x01, 0xf8, 0x48, 0x0f, 0x07]); // swapgs; sysretq

    code.bind("exit-group")?;
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x00, 0x00, 0x00, 0x00, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x00, 0x00, 0x00, 0x00, 0x08]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00]);
    code.bytes(&[0x0f, 0x01, 0xf8, 0xc3]); // restore kernel GS base; return continuation

    code.bind("fail")?;
    code.bytes(&[0xfa, 0xf4, 0xeb, 0xfd]);
    code.bind("uts-data")?;
    code.bytes(&initial_utsname_bytes());
    code.finish()
}

fn initial_utsname_bytes() -> [u8; 390] {
    let mut bytes = [0_u8; 390];
    for (index, value) in [
        "Linux",
        "topal",
        "7.2.9-topal",
        "#1 Topal",
        "x86_64",
        "(none)",
    ]
    .into_iter()
    .enumerate()
    {
        let start = index * 65;
        bytes[start..start + value.len()].copy_from_slice(value.as_bytes());
    }
    bytes
}

fn user_copy_page_fault_entry() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x18, 0x00, 0x00, 0x00, 0x01]);
    code.jump_if(0x84, "fault-copy-to");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x18, 0x00, 0x00, 0x00, 0x02]);
    code.jump_if(0x84, "fault-copy-from");
    code.jump("fault-fatal");
    code.bind("fault-copy-to")?;
    code.bytes(&[0x48, 0x83, 0x3c, 0x24, 0x02]); // non-present supervisor write
    code.jump_if(0x85, "fault-fatal");
    code.jump("fault-site");
    code.bind("fault-copy-from")?;
    code.bytes(&[0x48, 0x83, 0x3c, 0x24, 0x00]); // non-present supervisor read
    code.jump_if(0x85, "fault-fatal");
    code.bind("fault-site")?;
    code.bytes(&[0x48, 0x8b, 0x44, 0x24, 0x08]);
    code.bytes(&[0x65, 0x48, 0x3b, 0x04, 0x25, 0x20, 0x00, 0x00, 0x00]);
    code.jump_if(0x85, "fault-fatal");
    code.bytes(&[0x0f, 0x20, 0xd0]); // CR2
    code.bytes(&[0x48, 0x3d]);
    code.bytes(&0x0060_2000_u32.to_le_bytes());
    code.jump_if(0x85, "fault-fatal");
    code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25, 0x28, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x89, 0x44, 0x24, 0x08]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x83, 0xc4, 0x08, 0x48, 0xcf]); // discard error; iretq
    code.bind("fault-fatal")?;
    code.bytes(&[0xfa, 0xf4, 0xeb, 0xfd]);
    code.finish()
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
    code.jump_if(0x84, "select-suspended");
    code.bytes(&[0x80, 0x7f, 0x18, 0x02]); // active cooperative caller
    code.jump_if(0x85, "fail");
    code.bytes(&[0x9c, 0x58, 0xa9, 0x00, 0x02, 0x00, 0x00]); // require IF=0
    code.jump_if(0x85, "fail");
    code.bytes(&[0xc6, 0x47, 0x18, 0x01]); // worker becomes suspended
    code.bytes(&[0x53, 0x55, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57]);
    code.bytes(&[0x48, 0x89, 0x67, 0x08]); // suspend worker RSP
    code.bytes(&[0x48, 0x8b, 0x27]); // select dispatcher RSP
    code.bytes(&[
        0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x41, 0x5c, 0x5d, 0x5b, 0xc3,
    ]);
    code.bind("select-suspended")?;
    code.bytes(&[0x9c, 0x58, 0xa9, 0x00, 0x02, 0x00, 0x00]); // require IF=0
    code.jump_if(0x85, "fail");
    code.bytes(&[0xc6, 0x47, 0x18, 0x02]); // selected worker becomes active
    code.bytes(&[0x53, 0x55, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57]);
    code.bytes(&[0x48, 0x89, 0x27]); // suspend dispatcher RSP
    code.bytes(&[0x48, 0x8b, 0x67, 0x08]); // select worker RSP
    code.bytes(&[
        0x41, 0x5f, 0x41, 0x5e, 0x41, 0x5d, 0x41, 0x5c, 0x5d, 0x5b, 0xc3,
    ]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn context_await_deadline_preemption() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x7f, 0x18, 0x02]); // active worker
    code.jump_if(0x85, "fail");
    code.bytes(&[0xfb, 0xf4, 0xfa]); // sti; hlt; restore sealed IF=0 state
    code.bytes(&[0x80, 0x7f, 0x18, 0x02]); // restored only by redispatch
    code.jump_if(0x85, "fail");
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0xfa, 0x31, 0xc0, 0xc3]);
    code.finish()
}

fn context_preempt_current() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x7f, 0x18, 0x02]); // active worker
    code.jump_if(0x85, "fail");
    code.bytes(&[0x9c, 0x58, 0xa9, 0x00, 0x02, 0x00, 0x00]); // require IF=0
    code.jump_if(0x85, "fail");
    code.bytes(&[0xc6, 0x47, 0x18, 0x01]); // interrupted worker becomes suspended
    code.bytes(&[0x53, 0x55, 0x41, 0x54, 0x41, 0x55, 0x41, 0x56, 0x41, 0x57]);
    code.bytes(&[0x48, 0x89, 0x67, 0x08]); // retain interrupt-handler RSP
    code.bytes(&[0x48, 0x8b, 0x27]); // restore exact dispatcher RSP
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00]); // complete transfer call
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

fn emit_initial_boot_resource_handoff_validation(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    let expected = generate_initial_boot_resource(&initial_user_elf_file()?)?;
    let expected_size = u32::try_from(expected.len())
        .map_err(|_| CompileError::Tool("initial boot resource exceeds u32".into()))?;
    code.bytes(&[0x83, 0xbe, 0xc0, 0x00, 0x00, 0x00, 0x00]); // ext_ramdisk_image
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xbe, 0xc4, 0x00, 0x00, 0x00, 0x00]); // ext_ramdisk_size
    code.jump_if(0x85, "fail");
    code.bytes(&[0x44, 0x8b, 0x86, 0x18, 0x02, 0x00, 0x00]); // resource base
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x8b, 0x8e, 0x1c, 0x02, 0x00, 0x00]); // resource size
    code.bytes(&[0x81, 0xf9]);
    code.bytes(&expected_size.to_le_bytes());
    code.jump_if(0x85, "fail");
    code.bytes(&[0x4d, 0x89, 0xc1, 0x49, 0x01, 0xc9]); // resource end
    code.jump_if(0x82, "fail");
    code.bytes(&[0x49, 0x81, 0xf9, 0x00, 0x00, 0x00, 0x40]); // initial map limit
    code.jump_if(0x87, "fail");
    code.bytes(&[0x41, 0x81, 0x38, 0x30, 0x37, 0x30, 0x37]); // `0707`
    code.jump_if(0x85, "fail");
    code.bytes(&[0x66, 0x41, 0x81, 0x78, 0x04, 0x30, 0x31]); // `01`
    code.jump_if(0x85, "fail");
    Ok(())
}

fn boot_memory_validator() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xf6]); // test rsi, rsi
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0xf0, 0xff, 0x00]); // cmp rsi, 0x00fff000
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x83, 0xbe, 0x50, 0x02, 0x00, 0x00, 0x00]); // setup_data == 0
    code.jump_if(0x85, "fail");
    emit_initial_boot_resource_handoff_validation(&mut code)?;
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
    code.bytes(&[0x41, 0x54]); // preserve r12
    code.bytes(&[0x49, 0x89, 0xcc]); // retain optional excluded base
    code.bytes(&[0x4d, 0x85, 0xe4]);
    code.jump_if(0x84, "count-admitted");
    code.bytes(&[0x83, 0xfa, 0x01]); // exclusion admits only one frame
    code.jump_if(0x85, "fail");
    code.bind("count-admitted")?;
    code.bytes(&[0x83, 0xfa, 0x01]); // frame count == 1
    code.jump_if(0x84, "count-ready");
    code.bytes(&[0x83, 0xfa, 0x03]); // frame count == 3
    code.jump_if(0x85, "fail");
    code.bind("count-ready")?;
    code.bytes(&[0x48, 0xc1, 0xe2, 0x0c]); // byte count = frames * 4096
    code.bytes(&[0x49, 0x89, 0xd0]); // retain byte count in r8
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
    emit_initial_boot_resource_handoff_validation(&mut code)?;
    code.bytes(&[0x49, 0x89, 0xd0]); // restore byte count after handoff validation
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
    code.bytes(&[0x49, 0x89, 0xc2]); // extent end = candidate
    code.bytes(&[0x4d, 0x01, 0xc2]); // extent end += byte count
    code.jump_if(0x82, "fail");
    code.bytes(&[0x4d, 0x85, 0xe4]); // optional live user backing exclusion
    code.jump_if(0x84, "extent-ready");
    code.bytes(&[0x4d, 0x89, 0xe1]); // excluded end = excluded base
    code.bytes(&[0x49, 0x81, 0xc1, 0x00, 0x30, 0x00, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x4d, 0x39, 0xe2]); // extent end <= excluded base
    code.jump_if(0x86, "extent-ready");
    code.bytes(&[0x4c, 0x39, 0xc8]); // candidate >= excluded end
    code.jump_if(0x83, "extent-ready");
    code.bytes(&[0x4c, 0x89, 0xc8]); // advance candidate past exclusion
    code.bytes(&[0x49, 0x89, 0xc2]);
    code.bytes(&[0x4d, 0x01, 0xc2]);
    code.jump_if(0x82, "fail");
    code.bind("extent-ready")?;
    code.bytes(&[0x44, 0x8b, 0x8e, 0x18, 0x02, 0x00, 0x00]); // resource base
    code.bytes(&[0x44, 0x8b, 0x9e, 0x1c, 0x02, 0x00, 0x00]); // resource size
    code.bytes(&[0x4d, 0x01, 0xcb]); // resource end
    code.bytes(&[0x4d, 0x39, 0xca]); // extent end <= resource base
    code.jump_if(0x86, "resource-disjoint");
    code.bytes(&[0x4c, 0x39, 0xd8]); // candidate >= resource end
    code.jump_if(0x83, "resource-disjoint");
    code.jump("next");
    code.bind("resource-disjoint")?;
    code.bytes(&[0x4d, 0x39, 0xda]); // extent end <= RAM end
    code.jump_if(0x87, "next");
    code.bytes(&[0x49, 0x81, 0xfa, 0x00, 0x00, 0x00, 0x40]); // end <= 1 GiB
    code.jump_if(0x87, "next");
    code.bytes(&[0x41, 0x5c, 0xc3]); // restore r12 and return physical base

    code.bind("zero-length")?;
    code.bytes(&[0x41, 0x83, 0xfa, 0x01]);
    code.jump_if(0x84, "fail");

    code.bind("next")?;
    code.bytes(&[0x48, 0x83, 0xc2, 0x14]);
    code.bytes(&[0xff, 0xc9]);
    code.jump_if(0x85, "loop");

    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0x41, 0x5c, 0xc3]);
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
    emit_initial_boot_resource_handoff_validation(&mut code)?;
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
    code.bytes(&[0x49, 0x89, 0xc2]); // end of 5 pages = candidate
    code.bytes(&[0x49, 0x81, 0xc2, 0x00, 0x50, 0x00, 0x00]);
    code.jump_if(0x82, "next");
    code.bytes(&[0x44, 0x8b, 0x8e, 0x18, 0x02, 0x00, 0x00]); // resource base
    code.bytes(&[0x44, 0x8b, 0x9e, 0x1c, 0x02, 0x00, 0x00]); // resource size
    code.bytes(&[0x4d, 0x01, 0xcb]); // resource end
    code.bytes(&[0x4d, 0x39, 0xca]); // extent end <= resource base
    code.jump_if(0x86, "resource-disjoint");
    code.bytes(&[0x4c, 0x39, 0xd8]); // candidate >= resource end
    code.jump_if(0x83, "resource-disjoint");
    code.jump("next");
    code.bind("resource-disjoint")?;
    code.bytes(&[0x4d, 0x39, 0xda]); // candidate + 5 pages <= RAM end
    code.jump_if(0x87, "next");
    code.bytes(&[0x48, 0x3d, 0x00, 0xb0, 0xff, 0x3f]); // all backing below 1 GiB
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
    code.bytes(&[0xb9, 0x00, 0x0a, 0x00, 0x00]); // 2560 qwords
    code.bytes(&[0xfc, 0xf3, 0x48, 0xab]); // cld; rep stosq
    code.bytes(&[0x4d, 0x8d, 0x88, 0x00, 0x10, 0x00, 0x00]); // pdpt
    code.bytes(&[0x4c, 0x89, 0xc8, 0x48, 0x83, 0xc8, 0x07]);
    code.bytes(&[0x49, 0x89, 0x00]); // pml4[0]
    code.bytes(&[0x4d, 0x8d, 0x88, 0x00, 0x20, 0x00, 0x00]); // pd
    code.bytes(&[0x4c, 0x89, 0xc8, 0x48, 0x83, 0xc8, 0x07]);
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
    code.bytes(&[0x48, 0x81, 0xff, 0x00, 0xb0, 0xff, 0x3f]);
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

    fn rip_lea(&mut self, opcode: &[u8], label: &'static str) {
        self.bytes.extend_from_slice(opcode);
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
                .windows(7)
                .any(|bytes| bytes == [0x49, 0x81, 0xfa, 0x00, 0x00, 0x00, 0x40]),
            "selector must constrain the complete extent to bootstrap identity mappings"
        );
        assert!(selector.windows(3).any(|bytes| bytes == [0x83, 0xfa, 0x01]));
        assert!(selector.windows(3).any(|bytes| bytes == [0x83, 0xfa, 0x03]));
        assert!(
            selector
                .windows(4)
                .any(|bytes| bytes == [0x48, 0xc1, 0xe2, 0x0c]),
            "selector must convert the admitted frame count to a byte extent"
        );
        assert!(
            selector.windows(3).any(|bytes| bytes == [0x4d, 0x39, 0xda]),
            "selector must require the complete extent to fit its E820 RAM run"
        );
        assert_eq!(&selector[..5], &[0x41, 0x54, 0x49, 0x89, 0xcc]);
        assert!(
            selector
                .windows(7)
                .any(|bytes| bytes == [0x49, 0x81, 0xc1, 0x00, 0x30, 0x00, 0x00]),
            "selector must derive the end of the exact three-frame exclusion"
        );
        assert!(
            selector.windows(3).any(|bytes| bytes == [0x4d, 0x39, 0xe2])
                && selector.windows(3).any(|bytes| bytes == [0x4c, 0x39, 0xc8])
                && selector.windows(3).any(|bytes| bytes == [0x4c, 0x89, 0xc8]),
            "selector must detect overlap and advance beyond the live user backing"
        );
        assert!(
            selector.windows(3).any(|bytes| bytes == [0x48, 0x39, 0xf8]),
            "selector must apply its provider-private allocation floor"
        );
        assert_eq!(
            selector
                .windows(3)
                .filter(|bytes| *bytes == [0x41, 0x5c, 0xc3])
                .count(),
            2,
            "success and failure must both restore r12"
        );
        assert_eq!(
            &selector[selector.len() - 5..],
            &[0x31, 0xc0, 0x41, 0x5c, 0xc3]
        );

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
                .any(|bytes| bytes == [0x49, 0x81, 0xc2, 0x00, 0x50, 0x00, 0x00]),
            "translation backing selection must reserve five complete pages"
        );
        assert!(
            translation_selector
                .windows(6)
                .any(|bytes| { bytes == [0x48, 0x3d, 0x00, 0xb0, 0xff, 0x3f] })
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
        assert!(
            !builder
                .windows(8)
                .any(|bytes| bytes == 0x0060_0007_u64.to_le_bytes())
        );
        assert!(
            !builder
                .windows(8)
                .any(|bytes| bytes == 0x8000_0000_0060_1007_u64.to_le_bytes())
        );

        let user_address_prepare =
            symbol_bytes(&file, X86_SYSTEMS_USER_ADDRESS_SPACE_PREPARE_SYMBOL);
        let user_prepare = symbol_bytes(&file, X86_SYSTEMS_USER_PREPARE_SYMBOL);
        for msr in [
            0xc000_0080_u32,
            0xc000_0081,
            0xc000_0082,
            0xc000_0084,
            0xc000_0102,
        ] {
            assert!(
                user_prepare
                    .windows(4)
                    .any(|bytes| bytes == msr.to_le_bytes()),
                "user preparation must configure MSR {msr:#x}"
            );
        }
        assert!(
            user_prepare
                .windows(7)
                .any(|bytes| bytes == [0x80, 0xbf, 0x70, 0, 0, 0, 0x16]),
            "user preparation must require both finalized image mappings resident"
        );
        assert!(
            user_address_prepare
                .windows(4)
                .any(|bytes| bytes == [0x41, 0x0f, 0x01, 0x38]),
            "address-space preparation must invalidate the sealed probe page after publication"
        );
        assert!(
            user_address_prepare
                .windows(4)
                .any(|bytes| bytes == [0x4c, 0x89, 0x57, 0x68]),
            "address-space preparation must retain the allocator-owned backing identity"
        );
        let code_file_size_offset = u32::try_from(
            X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
                + INITIAL_USER_CODE_FILE_SIZE_ELF_OFFSET,
        )
        .unwrap();
        let code_memory_size_offset = u32::try_from(
            X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
                + INITIAL_USER_CODE_MEMORY_SIZE_ELF_OFFSET,
        )
        .unwrap();
        assert!(
            user_address_prepare.windows(7).any(|bytes| {
                bytes[0..3] == [0x49, 0x8b, 0x80]
                    && bytes[3..7] == code_file_size_offset.to_le_bytes()
            }) && user_address_prepare.windows(7).any(|bytes| {
                bytes[0..3] == [0x49, 0x3b, 0x80]
                    && bytes[3..7] == code_memory_size_offset.to_le_bytes()
            }) && user_address_prepare
                .windows(6)
                .any(|bytes| bytes == [0x48, 0x3d, 0x08, 0x01, 0, 0])
                && user_address_prepare
                    .windows(6)
                    .any(|bytes| bytes == [0x48, 0x3d, 0, 0x10, 0, 0])
                && user_address_prepare
                    .windows(2)
                    .any(|bytes| bytes == [0xa8, 0x07]),
            "address-space preparation must validate a bounded equal qword-complete runtime code extent"
        );
        let data_file_size_offset = u32::try_from(
            X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
                + INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET,
        )
        .unwrap();
        let data_memory_size_offset = u32::try_from(
            X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
                + INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET,
        )
        .unwrap();
        assert!(
            user_address_prepare.windows(7).any(|bytes| {
                bytes[0..3] == [0x49, 0x8b, 0x80]
                    && bytes[3..7] == data_file_size_offset.to_le_bytes()
            }) && user_address_prepare.windows(7).any(|bytes| {
                bytes[0..3] == [0x49, 0x8b, 0x90]
                    && bytes[3..7] == data_memory_size_offset.to_le_bytes()
            }) && user_address_prepare
                .windows(4)
                .any(|bytes| bytes == [0x48, 0x83, 0xf8, 0x08])
                && user_address_prepare
                    .windows(3)
                    .any(|bytes| bytes == [0x48, 0x39, 0xd0])
                && user_address_prepare
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0x81, 0xfa, 0, 0x10, 0, 0]),
            "address-space preparation must validate bounded ordered qword-complete data and BSS extents"
        );
        let entry_offset = u32::try_from(
            X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + INITIAL_USER_ENTRY_ELF_OFFSET,
        )
        .unwrap();
        assert!(
            user_address_prepare.windows(7).any(|bytes| {
                bytes[0..3] == [0x4d, 0x8b, 0x98] && bytes[3..7] == entry_offset.to_le_bytes()
            }) && user_address_prepare
                .windows(7)
                .any(|bytes| bytes == [0x4c, 0x89, 0x9f, 0x88, 0, 0, 0])
                && user_address_prepare
                    .windows(7)
                    .any(|bytes| bytes == [0x4d, 0x89, 0x9a, 0x58, 0x1f, 0, 0]),
            "address-space preparation must validate and retain one ELF entry for AT_ENTRY"
        );
        let initial_stack = initial_user_stack_bytes();
        assert_eq!(
            u64::from_le_bytes(initial_stack[0..8].try_into().unwrap()),
            1
        );
        assert_eq!(
            u64::from_le_bytes(initial_stack[8..16].try_into().unwrap()),
            0x0060_1ff0
        );
        assert_eq!(initial_stack[16..32], [0; 16]);
        assert_eq!(
            u64::from_le_bytes(initial_stack[32..40].try_into().unwrap()),
            6
        );
        assert_eq!(
            u64::from_le_bytes(initial_stack[40..48].try_into().unwrap()),
            4096
        );
        assert_eq!(
            u64::from_le_bytes(initial_stack[48..56].try_into().unwrap()),
            9
        );
        assert_eq!(
            u64::from_le_bytes(initial_stack[56..64].try_into().unwrap()),
            0
        );
        assert_eq!(
            u64::from_le_bytes(initial_stack[64..72].try_into().unwrap()),
            31
        );
        assert_eq!(
            u64::from_le_bytes(initial_stack[72..80].try_into().unwrap()),
            0x0060_1ff0
        );
        for (offset, tag) in [(80, 11_u64), (96, 12), (112, 13), (128, 14)] {
            assert_eq!(
                u64::from_le_bytes(initial_stack[offset..offset + 8].try_into().unwrap()),
                tag
            );
            assert_eq!(initial_stack[offset + 8..offset + 16], [0; 8]);
        }
        for (offset, tag, value) in [
            (144, 3_u64, INITIAL_USER_PROGRAM_HEADER_ADDRESS),
            (160, 4, 56),
            (176, 5, 3),
        ] {
            assert_eq!(
                u64::from_le_bytes(initial_stack[offset..offset + 8].try_into().unwrap()),
                tag
            );
            assert_eq!(
                u64::from_le_bytes(initial_stack[offset + 8..offset + 16].try_into().unwrap()),
                value
            );
        }
        assert_eq!(initial_stack[192..208], [0; 16]);
        assert_eq!(&initial_stack[208..219], b"topal-probe");
        assert_eq!(initial_stack[219], 0);
        assert!(
            initial_stack
                .chunks_exact(8)
                .all(|chunk| user_address_prepare.windows(8).any(|bytes| bytes == chunk)),
            "address-space preparation must initialize every word of the sealed initial stack"
        );
        assert!(
            user_address_prepare
                .windows(7)
                .any(|bytes| bytes == [0xc6, 0x87, 0x70, 0x00, 0x00, 0x00, 0x01]),
            "address-space state must not overlap the retained backing identity"
        );
        assert!(
            user_address_prepare
                .windows(8)
                .any(|bytes| bytes == 0x8000_0000_0000_0007_u64.to_le_bytes())
        );
        let user_mapping =
            symbol_bytes(&file, X86_SYSTEMS_USER_ADDRESS_SPACE_VERIFY_MAPPING_SYMBOL);
        assert!(
            user_mapping
                .windows(8)
                .any(|bytes| bytes == 0x8000_0000_0000_0007_u64.to_le_bytes()),
            "mapping verification must install an RW/NX user leaf"
        );
        assert!(
            user_mapping
                .windows(4)
                .any(|bytes| bytes == [0x41, 0xc6, 0x02, 0x6d])
                && user_mapping
                    .windows(4)
                    .any(|bytes| bytes == [0x41, 0x80, 0x3a, 0x6d]),
            "mapping verification must perform its fixed byte store and load"
        );
        assert!(
            user_mapping
                .windows(4)
                .filter(|bytes| *bytes == [0x41, 0x0f, 0x01, 0x3a])
                .count()
                >= 2,
            "mapping verification must invalidate after publication and revocation"
        );
        let user_map = symbol_bytes(&file, X86_SYSTEMS_USER_ADDRESS_SPACE_MAP_SYMBOL);
        assert!(
            user_map
                .windows(8)
                .any(|bytes| bytes == 0x8000_0000_0000_0007_u64.to_le_bytes())
                && user_map
                    .windows(4)
                    .any(|bytes| bytes == [0x48, 0x89, 0x57, 0x78]),
            "user map must publish RW/NX authority and retain its exact frame identity"
        );
        assert!(
            user_map
                .windows(5)
                .any(|bytes| bytes == [0xb8, 0x04, 0, 0, 0])
                && user_map
                    .windows(5)
                    .any(|bytes| bytes == [0xb8, 0x11, 0, 0, 0])
                && user_map
                    .windows(6)
                    .any(|bytes| bytes == [0x88, 0x87, 0x70, 0, 0, 0])
                && user_map
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0x89, 0x97, 0x80, 0, 0, 0])
                && user_map
                    .windows(8)
                    .any(|bytes| bytes == 0x0060_5000_u64.to_le_bytes())
                && user_map
                    .windows(8)
                    .any(|bytes| bytes == 0x0060_4000_u64.to_le_bytes())
                && !user_map
                    .windows(8)
                    .any(|bytes| bytes == 0x0060_2000_u64.to_le_bytes())
        );
        let user_store = symbol_bytes(&file, X86_SYSTEMS_USER_MAPPING_STORE_BYTE_SYMBOL);
        let data_payload_offset = X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET
            + u32::try_from(INITIAL_USER_DATA_FILE_OFFSET).unwrap();
        assert!(
            user_store
                .windows(3)
                .any(|bytes| bytes == [0x49, 0x8b, 0x00])
                && user_store
                    .windows(3)
                    .any(|bytes| bytes == [0x49, 0x89, 0x02])
                && user_store
                    .windows(7)
                    .any(|bytes| bytes == [0x49, 0xc7, 0x02, 0, 0, 0, 0])
                && user_store
                    .windows(4)
                    .any(|bytes| bytes == [0x48, 0x83, 0xe9, 0x08])
                && user_store
                    .windows(4)
                    .any(|bytes| bytes == [0x49, 0x83, 0xeb, 0x08])
                && user_store
                    .windows(4)
                    .any(|bytes| bytes == data_payload_offset.to_le_bytes())
                && user_store
                    .windows(6)
                    .any(|bytes| bytes == [0x41, 0xbb, 0x00, 0x10, 0x00, 0x00])
                && user_store
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0x89, 0x87, 0x90, 0, 0, 0])
                && user_store
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0x89, 0x87, 0x98, 0, 0, 0])
                && user_store
                    .windows(7)
                    .any(|bytes| bytes == [0xc6, 0x87, 0x70, 0, 0, 0, 0x12])
                && user_store
                    .windows(8)
                    .any(|bytes| bytes == INITIAL_USER_DATA_ADDRESS.to_le_bytes()),
            "user store must copy runtime data, derive both break states, and zero the complete BSS and future-heap page tail"
        );
        let user_install = symbol_bytes(
            &file,
            X86_SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE_SYMBOL,
        );
        let entry_payload = initial_user_entry_payload().unwrap();
        let code_image = initial_user_code_image().unwrap();
        let elf_file = initial_user_elf_file().unwrap();
        assert_eq!(code_image.len(), 256 + entry_payload.bytes.len());
        assert!(
            entry_payload
                .bytes
                .windows(8)
                .any(|bytes| bytes == (INITIAL_USER_DATA_ADDRESS + 8).to_le_bytes())
                && entry_payload
                    .bytes
                    .windows(4)
                    .any(|bytes| bytes == [0x41, 0xc6, 0x02, 0xa5])
                && entry_payload
                    .bytes
                    .windows(4)
                    .any(|bytes| bytes == [0x41, 0x80, 0x3a, 0xa5]),
            "constructed user code must write and read back one BSS byte"
        );
        assert_eq!(
            entry_payload
                .bytes
                .windows(7)
                .filter(|bytes| *bytes == [0xb8, 0x0c, 0, 0, 0, 0x0f, 0x05])
                .count(),
            4,
            "constructed user code must query, grow, shrink, and reject one program break"
        );
        assert!(
            entry_payload
                .bytes
                .windows(8)
                .any(|bytes| bytes == [0xc6, 0x04, 0x25, 0x10, 0x50, 0x60, 0x00, 0xa5])
                && entry_payload
                    .bytes
                    .windows(8)
                    .any(|bytes| bytes == [0x80, 0x3c, 0x25, 0x10, 0x50, 0x60, 0x00, 0xa5]),
            "constructed user code must write and read the first newly admitted heap byte"
        );
        assert_eq!(
            elf_file.len(),
            INITIAL_USER_DATA_FILE_OFFSET + INITIAL_USER_DATA_CAPACITY
        );
        assert_eq!(&code_image[0..4], b"\x7fELF");
        assert_eq!(
            u16::from_le_bytes(code_image[18..20].try_into().unwrap()),
            62
        );
        assert_eq!(
            u64::from_le_bytes(code_image[24..32].try_into().unwrap()),
            INITIAL_USER_ENTRY_ADDRESS
        );
        assert_eq!(
            u64::from_le_bytes(code_image[32..40].try_into().unwrap()),
            64
        );
        assert_eq!(
            u16::from_le_bytes(code_image[54..56].try_into().unwrap()),
            56
        );
        assert_eq!(
            u16::from_le_bytes(code_image[56..58].try_into().unwrap()),
            3
        );
        assert_eq!(
            u64::from_le_bytes(code_image[80..88].try_into().unwrap()),
            INITIAL_USER_CODE_ADDRESS
        );
        assert_eq!(
            u64::from_le_bytes(code_image[96..104].try_into().unwrap()),
            u64::try_from(code_image.len()).unwrap()
        );
        assert_eq!(
            u32::from_le_bytes(code_image[120..124].try_into().unwrap()),
            1
        );
        assert_eq!(
            u32::from_le_bytes(code_image[124..128].try_into().unwrap()),
            6
        );
        assert_eq!(
            u64::from_le_bytes(code_image[128..136].try_into().unwrap()),
            u64::try_from(INITIAL_USER_DATA_FILE_OFFSET).unwrap()
        );
        assert_eq!(
            u64::from_le_bytes(code_image[136..144].try_into().unwrap()),
            INITIAL_USER_DATA_ADDRESS
        );
        assert_eq!(
            u64::from_le_bytes(code_image[152..160].try_into().unwrap()),
            8
        );
        assert_eq!(
            u64::from_le_bytes(code_image[160..168].try_into().unwrap()),
            16
        );
        assert_eq!(
            u32::from_le_bytes(code_image[176..180].try_into().unwrap()),
            0x6474_e551
        );
        assert_eq!(
            u32::from_le_bytes(code_image[180..184].try_into().unwrap()),
            6
        );
        assert_eq!(elf_file[INITIAL_USER_DATA_FILE_OFFSET], 0x6d);
        assert!(
            elf_file[INITIAL_USER_DATA_FILE_OFFSET + 1..]
                .iter()
                .all(|byte| *byte == 0)
        );
        assert_eq!(&code_image[256..], entry_payload.bytes.as_slice());
        assert!(
            user_install
                .windows(8)
                .any(|bytes| bytes == INITIAL_USER_CODE_ADDRESS.to_le_bytes())
                && user_install
                    .windows(3)
                    .filter(|bytes| *bytes == [0x49, 0x8b, 0x00])
                    .count()
                    == 1
                && user_install
                    .windows(3)
                    .filter(|bytes| *bytes == [0x49, 0x89, 0x02])
                    .count()
                    == 1
                && user_install
                    .windows(4)
                    .any(|bytes| bytes == [0x49, 0x83, 0xc0, 0x08])
                && user_install
                    .windows(4)
                    .any(|bytes| bytes == [0x49, 0x83, 0xc2, 0x08])
                && user_install
                    .windows(4)
                    .any(|bytes| bytes == [0x48, 0x83, 0xe9, 0x08])
                && user_install.windows(7).any(|bytes| {
                    bytes
                        == [
                            0x49,
                            0x81,
                            0xc0,
                            X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as u8,
                            0,
                            0,
                            0,
                        ]
                })
                && user_install
                    .windows(7)
                    .any(|bytes| bytes == [0xc6, 0x87, 0x70, 0, 0, 0, 0x05]),
            "entry installation must copy the runtime-derived bounded extent from the external ELF member"
        );
        assert!(
            entry_payload
                .bytes
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x83, 0x7c, 0x24, 0x20, 0x06, 0x0f])
                && entry_payload
                    .bytes
                    .windows(9)
                    .any(|bytes| bytes == [0x48, 0x81, 0x7c, 0x24, 0x28, 0x00, 0x10, 0x00, 0x00])
                && entry_payload
                    .bytes
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0x83, 0x7c, 0x24, 0x30, 0x09, 0x0f])
                && entry_payload
                    .bytes
                    .windows(4)
                    .any(|bytes| bytes == [0x49, 0x8b, 0x43, 0x18])
                && entry_payload
                    .bytes
                    .windows(5)
                    .any(|bytes| bytes == [0x48, 0x39, 0x44, 0x24, 0x38])
                && entry_payload
                    .bytes
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0x83, 0x7c, 0x24, 0x40, 0x1f, 0x0f])
                && entry_payload
                    .bytes
                    .windows(9)
                    .any(|bytes| bytes == [0x48, 0x81, 0x7c, 0x24, 0x48, 0xf0, 0x1f, 0x60, 0x00])
                && entry_payload
                    .bytes
                    .windows(6)
                    .any(|bytes| bytes == [0x48, 0x83, 0x7c, 0x24, 0x50, 0x0b])
                && entry_payload
                    .bytes
                    .windows(6)
                    .any(|bytes| bytes == [0x48, 0x83, 0x7c, 0x24, 0x60, 0x0c])
                && entry_payload
                    .bytes
                    .windows(6)
                    .any(|bytes| bytes == [0x48, 0x83, 0x7c, 0x24, 0x70, 0x0d])
                && entry_payload
                    .bytes
                    .windows(9)
                    .any(|bytes| bytes == [0x48, 0x83, 0xbc, 0x24, 0x80, 0, 0, 0, 0x0e])
                && entry_payload
                    .bytes
                    .windows(12)
                    .any(|bytes| bytes
                        == [0x48, 0x81, 0xbc, 0x24, 0x98, 0, 0, 0, 0x40, 0x40, 0x60, 0])
                && entry_payload
                    .bytes
                    .windows(9)
                    .any(|bytes| bytes == [0x48, 0x83, 0xbc, 0x24, 0xa0, 0, 0, 0, 4])
                && entry_payload
                    .bytes
                    .windows(9)
                    .any(|bytes| bytes == [0x48, 0x83, 0xbc, 0x24, 0xa8, 0, 0, 0, 56])
                && entry_payload
                    .bytes
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0x81, 0xc4, 0xe0, 0, 0, 0]),
            "constructed entry must validate process facts and resident program-header metadata before consuming the complete bounded stack"
        );
        assert!(
            entry_payload
                .bytes
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x81, 0xec, 0x00, 0x02, 0x00, 0x00])
                && entry_payload
                    .bytes
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0x81, 0xc4, 0x00, 0x02, 0x00, 0x00])
                && entry_payload
                    .bytes
                    .windows(7)
                    .any(|bytes| bytes == [0xb8, 0x3f, 0, 0, 0, 0x0f, 0x05])
                && entry_payload
                    .bytes
                    .windows(7)
                    .any(|bytes| bytes == [0xb8, 0x23, 0, 0, 0, 0x0f, 0x05])
                && entry_payload
                    .bytes
                    .windows(4)
                    .any(|bytes| bytes == [0x48, 0x83, 0xec, 0x10])
                && entry_payload
                    .bytes
                    .windows(4)
                    .any(|bytes| bytes == [0x48, 0x83, 0xc4, 0x10])
                && entry_payload
                    .bytes
                    .windows(6)
                    .any(|bytes| bytes == [0x31, 0xc0, 0x48, 0x89, 0x04, 0x24])
                && entry_payload
                    .bytes
                    .windows(7)
                    .any(|bytes| bytes == [0xb8, 0x27, 0, 0, 0, 0x0f, 0x05])
                && entry_payload.bytes.windows(4).any(|bytes| bytes == b"Linu")
                && entry_payload
                    .bytes
                    .windows(2)
                    .any(|bytes| bytes == [0x31, 0xff]),
            "constructed transfer payload must qualify uname output, zero-duration nanosleep input, general bounded stdout write and stdin read before getpid and status-zero exit"
        );
        let user_syscall = symbol_bytes(&file, X86_SYSTEMS_USER_SYSCALL_SYMBOL);
        assert!(
            entry_payload
                .bytes
                .windows(7)
                .any(|bytes| bytes == [0xb8, 0x01, 0, 0, 0, 0x0f, 0x05])
        );
        assert!(
            entry_payload
                .bytes
                .windows(8)
                .any(|bytes| bytes == b"TOPAL_KE")
        );
        assert!(
            entry_payload
                .bytes
                .windows(4)
                .any(|bytes| bytes == [0x48, 0x83, 0xec, 0x20])
        );
        assert!(
            entry_payload
                .bytes
                .windows(4)
                .any(|bytes| bytes == [0x48, 0x83, 0xc4, 0x20])
        );
        assert!(
            !user_syscall.windows(8).any(|bytes| bytes == b"TOPAL_KE")
                && !user_syscall.windows(8).any(|bytes| bytes == b"RNEL_USE")
                && !user_syscall.windows(8).any(|bytes| bytes == b"R_WRITE_"),
            "the provider must not use the application payload as a write authorization oracle"
        );
        assert!(
            user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x81, 0xfa, 0x00, 0x01, 0x00, 0x00])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0x81, 0xfe, 0x00, 0x00, 0x60, 0x00])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x49, 0x81, 0xf9, 0x00, 0x20, 0x60, 0x00])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0x81, 0xfe, 0x00, 0x40, 0x60, 0x00])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x49, 0x81, 0xf9, 0x00, 0x60, 0x60, 0x00]),
            "bounded write must reject zero and oversized lengths and admit only complete readable mapping ranges"
        );
        assert!(
            user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x81, 0xec, 0x08, 0x01, 0x00, 0x00])
                && user_syscall
                    .windows(8)
                    .any(|bytes| bytes == [0x48, 0x89, 0x94, 0x24, 0x00, 0x01, 0x00, 0x00])
                && user_syscall
                    .windows(8)
                    .filter(|bytes| { *bytes == [0x4c, 0x3b, 0x94, 0x24, 0x00, 0x01, 0x00, 0x00] })
                    .count()
                    == 4
                && user_syscall
                    .windows(8)
                    .any(|bytes| bytes == [0x48, 0x8b, 0x84, 0x24, 0x00, 0x01, 0x00, 0x00]),
            "bounded write must finish dynamic private staging before output and return the requested length"
        );
        for errno in [0xf7_u8, 0xea, 0xf2] {
            assert!(
                user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0xc7, 0xc0, errno, 0xff, 0xff, 0xff]),
                "bounded write must retain distinct EBADF, EINVAL, and EFAULT results"
            );
        }
        assert!(
            user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x20])
        );
        assert!(
            user_syscall
                .windows(5)
                .any(|bytes| bytes == [0x42, 0x8a, 0x04, 0x14, 0xee])
        );
        assert!(
            entry_payload
                .bytes
                .windows(4)
                .any(|bytes| bytes == [0x31, 0xc0, 0x0f, 0x05])
                && entry_payload
                    .bytes
                    .windows(4)
                    .any(|bytes| bytes == [0x48, 0x83, 0xf8, 0x04])
                && entry_payload.bytes.windows(4).any(|bytes| bytes == b"read"),
            "constructed code must request and validate the four-byte qualification input"
        );
        assert!(
            user_syscall
                .windows(7)
                .filter(|bytes| *bytes == [0x48, 0x81, 0xfa, 0x00, 0x01, 0x00, 0x00])
                .count()
                == 2
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0x81, 0xfe, 0x00, 0x10, 0x60, 0x00])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x49, 0x81, 0xf9, 0x00, 0x20, 0x60, 0x00])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0x81, 0xfe, 0x00, 0x50, 0x60, 0x00])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x49, 0x81, 0xf9, 0x00, 0x60, 0x60, 0x00]),
            "bounded read must reject zero and oversized lengths and admit only complete writable mapping ranges"
        );
        let receive_stage = user_syscall
            .windows(4)
            .rposition(|bytes| bytes == [0x42, 0x88, 0x04, 0x14])
            .expect("bounded read must stage received bytes privately");
        let user_publish = user_syscall
            .windows(4)
            .rposition(|bytes| bytes == [0x43, 0x88, 0x04, 0x10])
            .expect("bounded read must copy staged bytes to the user destination");
        assert!(
            receive_stage < user_publish
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x66, 0xba, 0xfd, 0x03, 0xec, 0xa8, 0x01])
                && user_syscall
                    .windows(5)
                    .any(|bytes| bytes == [0x66, 0xba, 0xf8, 0x03, 0xec])
                && user_syscall
                    .windows(4)
                    .any(|bytes| bytes == [0x42, 0x8a, 0x04, 0x14])
                && user_syscall
                    .windows(8)
                    .filter(|bytes| { *bytes == [0x4c, 0x3b, 0x94, 0x24, 0x00, 0x01, 0x00, 0x00] })
                    .count()
                    == 4,
            "bounded read must receive a dynamic exact count into private staging before the dynamic user copy"
        );
        for errno in [0xf7_u8, 0xea, 0xf2] {
            assert!(
                user_syscall
                    .windows(7)
                    .filter(|bytes| { *bytes == [0x48, 0xc7, 0xc0, errno, 0xff, 0xff, 0xff] })
                    .count()
                    >= 2,
                "bounded read and write must each retain EBADF, EINVAL, and EFAULT results"
            );
        }
        assert!(
            user_syscall
                .windows(3)
                .any(|bytes| bytes == [0x83, 0xf8, 0x0c])
                && user_syscall
                    .windows(9)
                    .any(|bytes| bytes == [0x65, 0x48, 0x8b, 0x04, 0x25, 0x90, 0, 0, 0])
                && user_syscall
                    .windows(9)
                    .any(|bytes| bytes == [0x65, 0x4c, 0x8b, 0x04, 0x25, 0x98, 0, 0, 0])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0x81, 0xff, 0x00, 0x60, 0x60, 0x00])
                && user_syscall
                    .windows(9)
                    .any(|bytes| bytes == [0x65, 0x48, 0x89, 0x3c, 0x25, 0x90, 0, 0, 0]),
            "brk must use private current and initial state and publish only an in-page request"
        );
        let user_protect = symbol_bytes(&file, X86_SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY_SYMBOL);
        assert!(
            user_protect
                .windows(4)
                .any(|bytes| bytes == [0x49, 0x83, 0xe0, 0xfd])
                && user_protect
                    .windows(3)
                    .any(|bytes| bytes == [0x0f, 0xae, 0xf0])
                && user_protect
                    .windows(4)
                    .any(|bytes| bytes == [0x41, 0x0f, 0x01, 0x3a])
                && user_protect
                    .windows(5)
                    .any(|bytes| bytes == [0xb8, 0x0c, 0, 0, 0])
                && user_protect
                    .windows(5)
                    .any(|bytes| bytes == [0xb8, 0x13, 0, 0, 0]),
            "user protection must revoke the writable bit, order publication, invalidate the target, and retain generation"
        );
        let user_load = symbol_bytes(&file, X86_SYSTEMS_USER_MAPPING_LOAD_BYTE_SYMBOL);
        assert!(
            user_load
                .windows(3)
                .any(|bytes| bytes == [0x49, 0x8b, 0x00])
                && user_load
                    .windows(3)
                    .any(|bytes| bytes == [0x49, 0x39, 0x02])
                && user_load
                    .windows(4)
                    .any(|bytes| bytes == [0x49, 0x83, 0x3a, 0x00])
                && user_load
                    .windows(4)
                    .any(|bytes| bytes == [0x49, 0x83, 0xe9, 0x08])
                && user_load
                    .windows(4)
                    .any(|bytes| bytes == [0x49, 0x83, 0xeb, 0x08])
                && user_load
                    .windows(4)
                    .any(|bytes| bytes == data_payload_offset.to_le_bytes())
                && user_load
                    .windows(7)
                    .any(|bytes| bytes == [0x80, 0xbf, 0x70, 0, 0, 0, 0x13])
                && user_load
                    .windows(5)
                    .any(|bytes| bytes == [0xb8, 0x14, 0, 0, 0])
                && user_load
                    .windows(11)
                    .any(|bytes| bytes == [0x48, 0xc7, 0x87, 0x80, 0, 0, 0, 0, 0, 0, 0]),
            "user load must compare complete runtime initialized-data and BSS extents, revoking on mismatch"
        );
        let user_verify = symbol_bytes(
            &file,
            X86_SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE_SYMBOL,
        );
        assert!(
            user_verify
                .windows(8)
                .any(|bytes| bytes == INITIAL_USER_CODE_ADDRESS.to_le_bytes())
                && user_verify
                    .windows(3)
                    .filter(|bytes| *bytes == [0x49, 0x8b, 0x00])
                    .count()
                    == 1
                && user_verify
                    .windows(3)
                    .filter(|bytes| *bytes == [0x49, 0x39, 0x02])
                    .count()
                    == 1
                && user_verify
                    .windows(4)
                    .any(|bytes| bytes == [0x49, 0x83, 0xc0, 0x08])
                && user_verify
                    .windows(4)
                    .any(|bytes| bytes == [0x49, 0x83, 0xc2, 0x08])
                && user_verify
                    .windows(4)
                    .any(|bytes| bytes == [0x48, 0x83, 0xe9, 0x08])
                && user_verify
                    .windows(7)
                    .any(|bytes| bytes == [0xc6, 0x87, 0x70, 0, 0, 0, 0x06]),
            "entry verification must compare the complete runtime-derived extent to the external ELF source"
        );
        let user_execute = symbol_bytes(&file, X86_SYSTEMS_USER_MAPPING_PERMIT_EXECUTE_SYMBOL);
        assert!(
            user_execute
                .windows(5)
                .any(|bytes| bytes == [0x49, 0x0f, 0xba, 0xe0, 0x3f])
                && user_execute
                    .windows(5)
                    .any(|bytes| bytes == [0x49, 0x0f, 0xba, 0xf0, 0x3f])
                && user_execute
                    .windows(4)
                    .any(|bytes| bytes == [0x41, 0x0f, 0x01, 0x3a])
                && user_execute
                    .windows(7)
                    .any(|bytes| bytes == [0xc6, 0x87, 0x70, 0, 0, 0, 0x0e]),
            "user execute promotion must accept only generation one, require NX, clear it, and invalidate the target"
        );
        let user_data = symbol_bytes(&file, X86_SYSTEMS_USER_MAPPING_SEAL_IMMUTABLE_DATA_SYMBOL);
        assert!(
            user_data
                .windows(7)
                .any(|bytes| bytes == [0x80, 0xbf, 0x70, 0, 0, 0, 0x14])
                && user_data
                    .windows(4)
                    .any(|bytes| bytes == [0x41, 0xf6, 0xc0, 0x02])
                && user_data
                    .windows(5)
                    .any(|bytes| bytes == [0x49, 0x0f, 0xba, 0xe0, 0x3f])
                && user_data
                    .windows(7)
                    .any(|bytes| bytes == [0xc6, 0x87, 0x70, 0, 0, 0, 0x15]),
            "immutable-data sealing must accept only generation two and preserve its RO/NX permissions"
        );
        let writable_data =
            symbol_bytes(&file, X86_SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA_SYMBOL);
        assert!(
            writable_data
                .windows(7)
                .any(|bytes| bytes == [0x80, 0xbf, 0x70, 0, 0, 0, 0x14])
                && writable_data
                    .windows(4)
                    .any(|bytes| bytes == [0x41, 0xf6, 0xc0, 0x02])
                && writable_data
                    .windows(5)
                    .any(|bytes| bytes == [0x49, 0x0f, 0xba, 0xe0, 0x3f])
                && writable_data
                    .windows(4)
                    .any(|bytes| bytes == [0x49, 0x83, 0xc8, 0x02])
                && writable_data
                    .windows(3)
                    .any(|bytes| bytes == [0x0f, 0xae, 0xf0])
                && writable_data
                    .windows(4)
                    .any(|bytes| bytes == [0x41, 0x0f, 0x01, 0x3a])
                && writable_data
                    .windows(7)
                    .any(|bytes| bytes == [0xc6, 0x87, 0x70, 0, 0, 0, 0x16]),
            "writable-data promotion must accept verified generation two, restore write while retaining NX, and invalidate"
        );
        let user_unmap = symbol_bytes(&file, X86_SYSTEMS_USER_ADDRESS_SPACE_UNMAP_SYMBOL);
        assert!(
            user_unmap
                .windows(8)
                .any(|bytes| bytes == [0x48, 0xc7, 0x41, 0x10, 0, 0, 0, 0])
                && user_unmap
                    .windows(4)
                    .any(|bytes| bytes == [0x41, 0x0f, 0x01, 0x3a])
                && user_unmap
                    .windows(8)
                    .any(|bytes| bytes == [0x48, 0xc7, 0x47, 0x78, 0, 0, 0, 0])
                && user_unmap
                    .windows(11)
                    .any(|bytes| bytes == [0x48, 0xc7, 0x87, 0x80, 0, 0, 0, 0, 0, 0, 0])
                && user_unmap
                    .windows(11)
                    .any(|bytes| bytes == [0x48, 0xc7, 0x87, 0x90, 0, 0, 0, 0, 0, 0, 0])
                && user_unmap
                    .windows(11)
                    .any(|bytes| bytes == [0x48, 0xc7, 0x87, 0x98, 0, 0, 0, 0, 0, 0, 0])
                && user_unmap
                    .windows(8)
                    .any(|bytes| bytes == 0x0060_5000_u64.to_le_bytes())
                && user_unmap
                    .windows(8)
                    .any(|bytes| bytes == 0x0060_4000_u64.to_le_bytes())
                && !user_unmap
                    .windows(8)
                    .any(|bytes| bytes == 0x0060_2000_u64.to_le_bytes()),
            "user unmap must revoke, invalidate, clear break and mapping state, and return the frame"
        );
        let user_complete = symbol_bytes(&file, X86_SYSTEMS_USER_COMPLETE_SYMBOL);
        assert!(
            user_complete
                .windows(11)
                .any(|bytes| bytes == [0x48, 0xc7, 0x80, 0x18, 0x20, 0x00, 0x00, 0, 0, 0, 0])
        );
        assert_eq!(
            user_complete
                .windows(3)
                .filter(|bytes| *bytes == [0x0f, 0x01, 0x3a])
                .count(),
            3,
            "user completion must invalidate code, stack, and guard locations"
        );
        let user_reclaim = symbol_bytes(&file, X86_SYSTEMS_USER_ADDRESS_SPACE_RECLAIM_SYMBOL);
        assert!(
            user_reclaim
                .windows(4)
                .any(|bytes| bytes == [0x48, 0x8b, 0x47, 0x68]),
            "address-space reclaim must return the retained backing identity"
        );
        let user_enter = symbol_bytes(&file, X86_SYSTEMS_USER_ENTER_SYMBOL);
        assert!(user_enter.windows(2).any(|bytes| bytes == [0x48, 0xcf]));
        assert_eq!(&user_enter[user_enter.len() - 3..], &[0x31, 0xc0, 0xc3]);
        assert!(
            user_enter
                .windows(8)
                .any(|bytes| bytes == INITIAL_USER_STACK_ADDRESS.to_le_bytes())
        );
        assert!(
            user_enter
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x8b, 0x87, 0x88, 0, 0, 0])
                && !user_enter
                    .windows(8)
                    .any(|bytes| bytes == INITIAL_USER_ENTRY_ADDRESS.to_le_bytes()),
            "initial user instruction fetch must use the retained runtime ELF entry"
        );
        let user_syscall = symbol_bytes(&file, X86_SYSTEMS_USER_SYSCALL_SYMBOL);
        assert_eq!(
            user_syscall
                .windows(3)
                .filter(|bytes| *bytes == [0x0f, 0x01, 0xf8])
                .count(),
            3,
            "syscall entry must swap GS on entry and both exits"
        );
        assert!(
            user_syscall
                .windows(3)
                .any(|bytes| bytes == [0x48, 0x0f, 0x07]),
            "returning user syscalls must use SYSRETQ"
        );
        assert!(
            user_syscall
                .windows(4)
                .any(|bytes| bytes == 0x0060_0007_u32.to_le_bytes())
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0x81, 0xf9, 0x00, 0x40, 0x60, 0x00])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0x81, 0xf9, 0x00, 0x50, 0x60, 0x00]),
            "scalar identity calls must retain the provider probe and admit finalized executable continuations"
        );
        for (number, expected) in [
            (39_u32, 1_u8),
            (102, 0),
            (104, 0),
            (107, 0),
            (108, 0),
            (110, 0),
            (186, 1),
        ] {
            let mut invocation = [0_u8; 7];
            invocation[0] = 0xb8;
            invocation[1..5].copy_from_slice(&number.to_le_bytes());
            invocation[5..7].copy_from_slice(&[0x0f, 0x05]);
            let invocation_position = entry_payload
                .bytes
                .windows(7)
                .position(|bytes| bytes == invocation)
                .expect("constructed image must invoke every scalar identity syscall");
            if expected == 0 {
                assert_eq!(
                    &entry_payload.bytes[invocation_position + 7..invocation_position + 10],
                    &[0x48, 0x85, 0xc0]
                );
            } else {
                assert_eq!(
                    &entry_payload.bytes[invocation_position + 7..invocation_position + 11],
                    &[0x48, 0x83, 0xf8, 0x01]
                );
            }
            if number != 186 {
                assert!(
                    user_syscall
                        .windows(3)
                        .any(|bytes| bytes == [0x83, 0xf8, number as u8])
                );
            } else {
                assert!(
                    user_syscall
                        .windows(5)
                        .any(|bytes| bytes == [0x3d, 0xba, 0, 0, 0])
                );
            }
        }
        assert!(
            user_syscall.windows(4).any(|bytes| bytes
                == u32::try_from(entry_payload.uname_return_address)
                    .unwrap()
                    .to_le_bytes())
                && user_syscall
                    .windows(4)
                    .any(|bytes| bytes == INITIAL_USER_UNAME_CANDIDATE.to_le_bytes()),
            "constructed uname must admit only its sealed return site and exact writable stack candidate"
        );
        assert!(
            user_syscall.windows(4).any(|bytes| bytes
                == u32::try_from(entry_payload.nanosleep_return_address)
                    .unwrap()
                    .to_le_bytes())
                && user_syscall
                    .windows(4)
                    .any(|bytes| bytes == INITIAL_USER_NANOSLEEP_CANDIDATE.to_le_bytes()),
            "constructed nanosleep must admit only its sealed return site and exact readable stack candidate"
        );
        assert!(
            user_syscall
                .windows(4)
                .any(|bytes| bytes == 390_u32.to_le_bytes()),
            "copy_to_user must retain the exact new_utsname extent"
        );
        assert!(
            user_syscall.windows(8).any(|bytes| bytes == b"7.2.9-to"),
            "uname bytes must come from the checked source-owned value"
        );
        let user_probe = initial_user_probe().unwrap();
        assert!(
            user_probe
                .windows(8)
                .any(|bytes| bytes == entry_payload.continuation_address.to_le_bytes()),
            "the provider probe must transfer to the sealed constructed-image syscall continuation"
        );
        assert!(
            user_probe
                .windows(7)
                .filter(|bytes| *bytes == [0xb8, 0x23, 0x00, 0x00, 0x00, 0x0f, 0x05])
                .count()
                >= 2,
            "the generated user probe must invoke both nanosleep qualification paths"
        );
        assert!(
            user_syscall
                .windows(4)
                .any(|bytes| bytes == [0x49, 0x83, 0xfa, 0x10]),
            "copy_from_user must retain the exact native timespec extent"
        );
        assert!(
            user_syscall
                .windows(4)
                .any(|bytes| bytes == [0x43, 0x8a, 0x04, 0x10]),
            "copy_from_user must load through the generated user candidate site"
        );
        assert!(
            user_syscall
                .windows(4)
                .any(|bytes| bytes == [0x42, 0x88, 0x04, 0x14]),
            "copy_from_user must stage bytes in private kernel storage"
        );
        assert!(
            user_syscall
                .windows(9)
                .any(|bytes| { bytes == [0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x02] }),
            "copy_from_user must publish only its private read-direction recovery state"
        );
        let copy_fault = symbol_bytes(&file, X86_SYSTEMS_USER_COPY_FAULT_SYMBOL);
        assert!(
            copy_fault
                .windows(3)
                .any(|bytes| bytes == [0x0f, 0x20, 0xd0])
        );
        assert!(copy_fault.windows(2).any(|bytes| bytes == [0x48, 0xcf]));
        assert!(
            copy_fault
                .windows(5)
                .any(|bytes| bytes == [0x48, 0x83, 0x3c, 0x24, 0x02]),
            "copy_to_user recovery must require a supervisor write fault"
        );
        assert!(
            copy_fault
                .windows(5)
                .any(|bytes| bytes == [0x48, 0x83, 0x3c, 0x24, 0x00]),
            "copy_from_user recovery must require a supervisor read fault"
        );
        assert!(
            copy_fault
                .windows(4)
                .any(|bytes| bytes == 0x0060_2000_u32.to_le_bytes()),
            "copy fault recovery must validate the exact guard-page fault address"
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
        assert!(
            context_transfer
                .windows(4)
                .any(|bytes| bytes == [0x48, 0x89, 0x67, 0x08]),
            "cooperative handoff must suspend the running worker stack pointer"
        );
        assert!(
            context_transfer
                .windows(3)
                .any(|bytes| bytes == [0x48, 0x8b, 0x27]),
            "cooperative handoff must select the suspended dispatcher stack pointer"
        );
        for state in [0x01, 0x02] {
            assert!(
                context_transfer
                    .windows(4)
                    .any(|bytes| bytes == [0xc6, 0x47, 0x18, state]),
                "context transfer must publish both suspended and active states"
            );
        }
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
