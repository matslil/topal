//! Generated ELF object for the sealed initial x86-64 systems provider.

use std::collections::BTreeMap;

use object::write::{Object, Symbol, SymbolSection};
use object::{
    Architecture, BinaryFormat, Endianness, SectionKind, SymbolFlags, SymbolKind, SymbolScope,
};
use topal_language::compiler::CompilerSystemsProgram;

use crate::systems_artifact::qualified_x86_64_static_user_image_fixture;
use crate::systems_boot_resource::{
    X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET, X86_STATIC_BOOT_RESOURCE_PAYLOAD_OFFSET,
    generate_initial_boot_resource,
};
use crate::{
    CompileError, X86_SYSTEMS_PROVIDER_REVISION, X86SystemsProviderPlan,
    plan_x86_64_systems_provider,
};

pub const X86_SYSTEMS_PROVIDER_OBJECT_REVISION: &str =
    "topal.provider-object.x86_64-qemu-pc-q35/105";
pub const X86_SYSTEMS_PROVIDER_TEXT_SECTION: &str = ".text.topal.systems.provider";
pub const X86_SYSTEMS_BOOTSTRAP_STORAGE_SECTION: &str = ".bss.topal.bootstrap";
pub const X86_SYSTEMS_DMA_STORAGE_SECTION: &str = ".bss.topal.dma";
pub const X86_SYSTEMS_ENTROPY_STORAGE_SECTION: &str = ".bss.topal.entropy";
pub const X86_SYSTEMS_PROVIDER_NOTE_SECTION: &str = ".note.topal.provider";
pub const X86_SYSTEMS_BOOT_MEMORY_SYMBOL: &str = "topal_x86_systems_describe_boot_memory";
pub const X86_SYSTEMS_FRAME_ALLOCATE_SYMBOL: &str = "topal_x86_systems_allocate_physical_frames";
pub const X86_SYSTEMS_STATIC_IMAGE_PREPARE_SYMBOL: &str =
    "topal_x86_systems_static_image_prepare_initial_leaves";
pub const X86_SYSTEMS_STATIC_IMAGE_ZERO_PAYLOAD_SYMBOL: &str =
    "topal_x86_systems_static_image_zero_payload";
pub const X86_SYSTEMS_STATIC_IMAGE_REVOKE_SYMBOL: &str =
    "topal_x86_systems_static_image_revoke_initial_leaves";
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
pub const X86_SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS_SYMBOL: &str =
    "topal_x86_systems_user_reserve_initial_anonymous_mapping";
pub const X86_SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS_SYMBOL: &str =
    "topal_x86_systems_user_release_initial_anonymous_mapping";
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
pub const X86_SYSTEMS_PCI_CONFIGURATION_READ_DWORD_SYMBOL: &str =
    "topal_x86_systems_pci_configuration_read_dword_equals";
pub const X86_SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD_SYMBOL: &str =
    "topal_x86_systems_pci_configuration_update_word_masked";
pub const X86_SYSTEMS_DMA_BUFFER_CREATE_SYMBOL: &str = "topal_x86_systems_dma_buffer_create";
pub const X86_SYSTEMS_DMA_BUFFER_BIND_REFERENCE_SYMBOL: &str =
    "topal_x86_systems_dma_buffer_bind_reference";
pub const X86_SYSTEMS_DMA_BUFFER_STORE_INTEGER_SYMBOL: &str =
    "topal_x86_systems_dma_buffer_store_integer";
pub const X86_SYSTEMS_DMA_BUFFER_PREPARE_DEVICE_SYMBOL: &str =
    "topal_x86_systems_dma_buffer_prepare_device";
pub const X86_SYSTEMS_DMA_BUFFER_COMPLETE_CPU_SYMBOL: &str =
    "topal_x86_systems_dma_buffer_complete_cpu";
pub const X86_SYSTEMS_DMA_BUFFER_LOAD_INTEGER_SYMBOL: &str =
    "topal_x86_systems_dma_buffer_load_integer_equals";
pub const X86_SYSTEMS_ENTROPY_SAMPLE_CAPTURE_SYMBOL: &str =
    "topal_x86_systems_entropy_capture_dma_sample";
pub const X86_SYSTEMS_ENTROPY_POOL_MIX_SYMBOL: &str =
    "topal_x86_systems_entropy_mix_blake2s_pool_sample";
pub const X86_SYSTEMS_ENTROPY_GENERATOR_INITIALIZE_SYMBOL: &str =
    "topal_x86_systems_entropy_initialize_chacha20_generator";
pub const X86_SYSTEMS_ENTROPY_GENERATOR_EXTRACT_SYMBOL: &str =
    "topal_x86_systems_entropy_extract_chacha20_generator";
pub const X86_SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM_SYMBOL: &str =
    "topal_x86_systems_entropy_deliver_initial_auxv_random";
pub const X86_SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM_SYMBOL: &str =
    "topal_x86_systems_entropy_reserve_initial_user_random";
pub const X86_SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE_SYMBOL: &str =
    "topal_x86_systems_entropy_bind_reusable_user_random";
pub const X86_SYSTEMS_DMA_VIRTQUEUE_CONTROL_SYMBOL: &str =
    "topal_x86_systems_dma_virtqueue_control";
pub const X86_SYSTEMS_DMA_ENTROPY_PAYLOAD_SYMBOL: &str = "topal_x86_systems_dma_entropy_payload";
pub const X86_SYSTEMS_RAW_ENTROPY_SAMPLE_SYMBOL: &str = "topal_x86_systems_raw_entropy_sample";
pub const X86_SYSTEMS_ENTROPY_POOL_STATE_SYMBOL: &str = "topal_x86_systems_entropy_pool_state";
pub const X86_SYSTEMS_ENTROPY_GENERATOR_KEY_SYMBOL: &str =
    "topal_x86_systems_entropy_generator_key";
pub const X86_SYSTEMS_ENTROPY_OUTPUT_SYMBOL: &str = "topal_x86_systems_entropy_output";
pub const X86_SYSTEMS_ENTROPY_USER_RANDOM_SERVICE_SYMBOL: &str =
    "topal_x86_systems_entropy_user_random_service";
pub const X86_SYSTEMS_PCI_REGION_READ_SYMBOL: &str =
    "topal_x86_systems_pci_region_configuration_gateway_read_equals";
pub const X86_SYSTEMS_PCI_REGION_READ_EVENTUALLY_SYMBOL: &str =
    "topal_x86_systems_pci_region_configuration_gateway_read_eventually_equals";
pub const X86_SYSTEMS_PCI_REGION_WRITE_SYMBOL: &str =
    "topal_x86_systems_pci_region_configuration_gateway_write";
pub const X86_SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE_SYMBOL: &str =
    "topal_x86_systems_pci_region_configuration_gateway_write_dma_reference";
pub const X86_SYSTEMS_ALLOCATABLE_FLOOR: u64 = 16 * 1024 * 1024;
pub(crate) const INITIAL_USER_CODE_ADDRESS: u64 = 0x0060_3000;
const INITIAL_USER_PROGRAM_HEADER_ADDRESS: u64 = INITIAL_USER_CODE_ADDRESS + 64;
const INITIAL_USER_ENTRY_ADDRESS: u64 = INITIAL_USER_CODE_ADDRESS + 4096 + 256;
const INITIAL_USER_DATA_ADDRESS: u64 = 0x0060_5000;
const INITIAL_USER_CODE_CAPACITY: usize = 8192;
pub(crate) const INITIAL_USER_DATA_FILE_OFFSET: usize = INITIAL_USER_CODE_CAPACITY;
pub(crate) const INITIAL_USER_ENTRY_ELF_OFFSET: usize = 24;
pub(crate) const INITIAL_USER_CODE_FILE_SIZE_ELF_OFFSET: usize = 96;
pub(crate) const INITIAL_USER_CODE_MEMORY_SIZE_ELF_OFFSET: usize = 104;
pub(crate) const INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET: usize = 152;
pub(crate) const INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET: usize = 160;
pub(crate) const INITIAL_USER_DATA_CAPACITY: usize = 4096;
const INITIAL_USER_STACK_ADDRESS: u64 = 0x0060_1eb0;
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
    MovePciAddressPortToDx,
    MoveDwordSelectorToEax,
    OutputDwordToPort,
    MovePciDataPortToDx,
    InputDwordFromPort,
    CompareDwordWithExpected,
    SetEqualAl,
    ZeroExtendAlToEax,
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
            Self::MovePciAddressPortToDx => {
                output.extend_from_slice(&[0x66, 0xba, 0xf8, 0x0c]);
            }
            Self::MoveDwordSelectorToEax => output.extend_from_slice(&[0x89, 0xf8]),
            Self::OutputDwordToPort => output.push(0xef),
            Self::MovePciDataPortToDx => {
                output.extend_from_slice(&[0x66, 0xba, 0xfc, 0x0c]);
            }
            Self::InputDwordFromPort => output.push(0xed),
            Self::CompareDwordWithExpected => output.extend_from_slice(&[0x39, 0xf0]),
            Self::SetEqualAl => output.extend_from_slice(&[0x0f, 0x94, 0xc0]),
            Self::ZeroExtendAlToEax => output.extend_from_slice(&[0x0f, 0xb6, 0xc0]),
            Self::Breakpoint => output.push(0xcc),
            Self::InterruptReturn => output.extend_from_slice(&[0x48, 0xcf]),
            Self::DisableInterrupts => output.push(0xfa),
            Self::Halt => output.push(0xf4),
            Self::JumpBackToHalt => output.extend_from_slice(&[0xeb, 0xfd]),
            Self::Return => output.push(0xc3),
        }
    }
}

#[derive(Clone, Copy)]
enum PciRegionGatewayOperation {
    ReadEquals,
    ReadEventuallyEquals,
    Write,
}

fn pci_region_configuration_gateway(
    operation: PciRegionGatewayOperation,
) -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x41, 0x89, 0xd1]); // preserve target offset in r9d
    code.bytes(&[0x41, 0x89, 0xfa]); // preserve configuration selector in r10d

    for (selector_delta, value) in [(4_u8, 0_u8), (8, 1), (12, 2)] {
        code.bytes(&[0x41, 0x8d, 0x42, selector_delta]); // selector in eax
        code.bytes(&[0x66, 0xba, 0xf8, 0x0c, 0xef]); // select through 0xcf8
        code.bytes(&[0x66, 0xba, 0xfc, 0x0c]); // configuration data port
        match value {
            0 => code.bytes(&[0x89, 0xf0]),       // BAR index from esi
            1 => code.bytes(&[0x44, 0x89, 0xc8]), // target offset from r9d
            2 => code.bytes(&[0x89, 0xc8]),       // width from ecx
            _ => unreachable!(),
        }
        code.bytes(&[0xef]);
    }

    code.bytes(&[0x41, 0x8d, 0x42, 0x10]); // configuration data selector
    code.bytes(&[0x66, 0xba, 0xf8, 0x0c, 0xef]);
    code.bytes(&[0x66, 0xba, 0xfc, 0x0c]);
    if matches!(operation, PciRegionGatewayOperation::ReadEventuallyEquals) {
        code.bytes(&[0x41, 0xbb]); // bounded attempts in r11d
        code.bytes(&1_000_000_u32.to_le_bytes());
    }
    code.bytes(&[0x83, 0xf9, 0x01]); // exact byte access
    code.jump_if(0x84, "byte");
    code.bytes(&[0x83, 0xf9, 0x02]); // exact word access
    code.jump_if(0x84, "word");
    code.bytes(&[0x83, 0xf9, 0x04]); // exact dword access
    code.jump_if(0x85, "fail");
    match operation {
        PciRegionGatewayOperation::ReadEquals => {
            code.bytes(&[0xed, 0x44, 0x39, 0xc0]); // in eax, dx; cmp eax, r8d
            code.jump_if(0x85, "fail");
        }
        PciRegionGatewayOperation::ReadEventuallyEquals => {
            code.bind("dword-retry")?;
            code.bytes(&[0xed, 0x44, 0x39, 0xc0]);
            code.jump_if(0x84, "success");
            code.bytes(&[0x41, 0xff, 0xcb]); // dec r11d
            code.jump_if(0x85, "dword-retry");
            code.jump("fail");
        }
        PciRegionGatewayOperation::Write => {
            code.bytes(&[0x44, 0x89, 0xc0, 0xef]); // mov eax, r8d; out dx, eax
        }
    }
    code.jump("success");

    code.bind("word")?;
    match operation {
        PciRegionGatewayOperation::ReadEquals => {
            code.bytes(&[0x66, 0xed, 0x66, 0x44, 0x39, 0xc0]); // in ax, dx; cmp ax, r8w
            code.jump_if(0x85, "fail");
        }
        PciRegionGatewayOperation::ReadEventuallyEquals => {
            code.bind("word-retry")?;
            code.bytes(&[0x66, 0xed, 0x66, 0x44, 0x39, 0xc0]);
            code.jump_if(0x84, "success");
            code.bytes(&[0x41, 0xff, 0xcb]);
            code.jump_if(0x85, "word-retry");
            code.jump("fail");
        }
        PciRegionGatewayOperation::Write => {
            code.bytes(&[0x44, 0x89, 0xc0, 0x66, 0xef]); // mov eax, r8d; out dx, ax
        }
    }
    code.jump("success");

    code.bind("byte")?;
    match operation {
        PciRegionGatewayOperation::ReadEquals => {
            code.bytes(&[0xec, 0x44, 0x38, 0xc0]); // in al, dx; cmp al, r8b
            code.jump_if(0x85, "fail");
        }
        PciRegionGatewayOperation::ReadEventuallyEquals => {
            code.bind("byte-retry")?;
            code.bytes(&[0xec, 0x44, 0x38, 0xc0]);
            code.jump_if(0x84, "success");
            code.bytes(&[0x41, 0xff, 0xcb]);
            code.jump_if(0x85, "byte-retry");
            code.jump("fail");
        }
        PciRegionGatewayOperation::Write => {
            code.bytes(&[0x44, 0x89, 0xc0, 0xee]); // mov eax, r8d; out dx, al
        }
    }

    code.bind("success")?;
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn pci_configuration_update_word() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x41, 0x89, 0xd1]); // preserve expected value in r9d
    code.bytes(&[0x41, 0x89, 0xca]); // preserve desired value in r10d
    code.bytes(&[0x89, 0xf8]); // selector in eax
    code.bytes(&[0x66, 0xba, 0xf8, 0x0c, 0xef]); // select through 0xcf8
    code.bytes(&[0x66, 0xba, 0xfc, 0x0c]); // base configuration data port
    code.bytes(&[0x66, 0x44, 0x01, 0xc2]); // add word-within-dword offset
    code.bytes(&[0x66, 0xed, 0x0f, 0xb7, 0xc0]); // read command word
    code.bytes(&[0x41, 0x89, 0xc3]); // preserve current word in r11d
    code.bytes(&[0x21, 0xf0, 0x44, 0x39, 0xc8]); // mask and compare pre-state
    code.jump_if(0x85, "fail");
    code.bytes(&[0xf7, 0xd6, 0x41, 0x21, 0xf3]); // retain unmasked current bits
    code.bytes(&[0x45, 0x09, 0xd3, 0x44, 0x89, 0xd8]); // apply desired bits
    code.bytes(&[0x66, 0xef]); // exact word write
    code.bytes(&[0xf7, 0xd6]); // restore mask
    code.bytes(&[0x66, 0xed, 0x0f, 0xb7, 0xc0]); // exact word readback
    code.bytes(&[0x21, 0xf0, 0x44, 0x39, 0xd0]); // mask and compare post-state
    code.jump_if(0x85, "fail");
    code.bytes(&[0xb8, 1, 0, 0, 0]);
    code.jump("return");
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0]);
    code.bind("return")?;
    code.bytes(&[0xc3]);
    code.finish()
}

fn dma_buffer_create() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x83, 0xff, 0x01]); // cmp edi, 1
    code.jump_if(0x85, "second");
    code.bytes(&[0x83, 0xfe, 0x01]); // cmp esi, 1
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xfa, 0x01]); // cmp edx, 1
    code.jump_if(0x85, "fail");
    code.bytes(&[0x81, 0xf9, 0x00, 0x10, 0x00, 0x00]); // cmp ecx, 4096
    code.jump_if(0x85, "fail");
    code.bytes(&[0x41, 0x81, 0xf8, 0x00, 0x10, 0x00, 0x00]); // cmp r8d, 4096
    code.jump_if(0x85, "fail");
    code.bytes(&[0x41, 0x83, 0xf9, 0x01]); // packed bidirectional/control metadata
    code.jump_if(0x85, "fail");
    code.jump("success");
    code.bind("second")?;
    code.bytes(&[0x83, 0xff, 0x02]); // cmp edi, 2
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xfe, 0x01]); // cmp esi, 1
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xfa, 0x01]); // cmp edx, 1
    code.jump_if(0x85, "fail");
    code.bytes(&[0x81, 0xf9, 0x00, 0x10, 0x00, 0x00]); // cmp ecx, 4096
    code.jump_if(0x85, "fail");
    code.bytes(&[0x41, 0x81, 0xf8, 0x00, 0x10, 0x00, 0x00]); // cmp r8d, 4096
    code.jump_if(0x85, "fail");
    code.bytes(&[0x41, 0x83, 0xf9, 0x12]); // packed device-write/payload metadata
    code.jump_if(0x85, "fail");
    code.bind("success")?;
    code.bytes(&[0xb8, 1, 0, 0, 0]);
    code.bytes(&[0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn dma_buffer_bind_reference() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x41, 0x83, 0xf8, 0x08]); // cmp r8d, 8
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xfe, 0xf8, 0x0f, 0x00, 0x00]); // cmp rsi, 4088
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0xf7, 0xc6, 0x07, 0x00, 0x00, 0x00]); // test rsi, 7
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xf9, 0xff, 0x0f, 0x00, 0x00]); // cmp rcx, 4095
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x01, 0xca]); // add rdx, rcx
    code.bytes(&[0x48, 0x89, 0x14, 0x37]); // mov [rdi+rsi], rdx
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn dma_buffer_store_integer() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x83, 0xfa, 0x02]); // cmp edx, 2
    code.jump_if(0x85, "dword");
    code.bytes(&[0x48, 0x81, 0xfe, 0xfe, 0x0f, 0x00, 0x00]); // cmp rsi, 4094
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0xf7, 0xc6, 0x01, 0x00, 0x00, 0x00]); // test rsi, 1
    code.jump_if(0x85, "fail");
    code.bytes(&[0x66, 0x89, 0x0c, 0x37]); // mov [rdi+rsi], cx
    code.jump("success");
    code.bind("dword")?;
    code.bytes(&[0x83, 0xfa, 0x04]); // cmp edx, 4
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xfe, 0xfc, 0x0f, 0x00, 0x00]); // cmp rsi, 4092
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0xf7, 0xc6, 0x03, 0x00, 0x00, 0x00]); // test rsi, 3
    code.jump_if(0x85, "fail");
    code.bytes(&[0x89, 0x0c, 0x37]); // mov [rdi+rsi], ecx
    code.bind("success")?;
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn dma_buffer_prepare_device() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00]); // page-aligned storage
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xfa, 0x01]); // device identity one
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xf9, 0x01]); // translation domain one
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xfe, 0x02]); // payload identity
    code.jump_if(0x85, "control");
    code.bytes(&[0x41, 0x83, 0xf8, 0x12]); // device-write/payload metadata
    code.jump_if(0x85, "fail");
    code.jump("success");
    code.bind("control")?;
    code.bytes(&[0x83, 0xfe, 0x01]); // control identity
    code.jump_if(0x85, "fail");
    code.bytes(&[0x41, 0x83, 0xf8, 0x01]); // bidirectional/control metadata
    code.jump_if(0x85, "fail");
    code.bind("success")?;
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn dma_buffer_load_integer() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x83, 0xfa, 0x02]);
    code.jump_if(0x85, "dword");
    code.bytes(&[0x48, 0x81, 0xfe, 0xfe, 0x0f, 0x00, 0x00]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0xf7, 0xc6, 0x01, 0x00, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x0f, 0xb7, 0x04, 0x37, 0x39, 0xc8]);
    code.jump_if(0x85, "fail");
    code.jump("success");
    code.bind("dword")?;
    code.bytes(&[0x83, 0xfa, 0x04]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x81, 0xfe, 0xfc, 0x0f, 0x00, 0x00]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0xf7, 0xc6, 0x03, 0x00, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x8b, 0x04, 0x37, 0x39, 0xc8]);
    code.jump_if(0x85, "fail");
    code.bind("success")?;
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn entropy_capture_dma_sample() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x83, 0xfa, 0x01]); // sample identity one
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xf9, 0x02]); // payload buffer identity two
    code.jump_if(0x85, "fail");
    code.bytes(&[0x41, 0x83, 0xf8, 0x20]); // exact 32-byte range
    code.jump_if(0x85, "fail");
    code.bytes(&[0x45, 0x85, 0xc9]); // explicitly uncredited
    code.jump_if(0x85, "fail");
    for offset in [0_u8, 8, 16, 24] {
        if offset == 0 {
            code.bytes(&[0x48, 0x8b, 0x07]);
            code.bytes(&[0x48, 0x89, 0x06]);
            code.bytes(&[0x48, 0x3b, 0x06]);
        } else {
            code.bytes(&[0x48, 0x8b, 0x47, offset]);
            code.bytes(&[0x48, 0x89, 0x46, offset]);
            code.bytes(&[0x48, 0x3b, 0x46, offset]);
        }
        code.jump_if(0x85, "fail");
    }
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

const BLAKE2S_IV: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];

const BLAKE2S_SIGMA: [[usize; 16]; 10] = [
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [14, 10, 4, 8, 9, 15, 13, 6, 1, 12, 0, 2, 11, 7, 5, 3],
    [11, 8, 12, 0, 5, 2, 15, 13, 10, 14, 3, 6, 7, 1, 9, 4],
    [7, 9, 3, 1, 13, 12, 11, 14, 2, 6, 5, 10, 4, 0, 15, 8],
    [9, 0, 5, 7, 2, 4, 10, 15, 14, 1, 11, 12, 6, 8, 3, 13],
    [2, 12, 6, 10, 0, 11, 8, 3, 4, 13, 7, 5, 15, 14, 1, 9],
    [12, 5, 1, 15, 14, 13, 4, 10, 0, 7, 6, 3, 9, 2, 8, 11],
    [13, 11, 7, 14, 12, 1, 3, 9, 5, 0, 15, 4, 8, 6, 2, 10],
    [6, 15, 14, 9, 11, 3, 0, 8, 12, 2, 13, 7, 1, 4, 10, 5],
    [10, 2, 8, 4, 7, 6, 1, 5, 15, 11, 9, 14, 3, 12, 13, 0],
];

fn emit_stack_dword_load(code: &mut X86FunctionEncoder, register: u8, offset: usize) {
    let displacement = u8::try_from(offset).expect("BLAKE2s stack offset fits disp8");
    match register {
        0 => code.bytes(&[0x8b, 0x44, 0x24, displacement]), // eax
        1 => code.bytes(&[0x8b, 0x4c, 0x24, displacement]), // ecx
        2 => code.bytes(&[0x8b, 0x54, 0x24, displacement]), // edx
        8 => code.bytes(&[0x44, 0x8b, 0x44, 0x24, displacement]), // r8d
        _ => unreachable!(),
    }
}

fn emit_stack_dword_store(code: &mut X86FunctionEncoder, register: u8, offset: usize) {
    let displacement = u8::try_from(offset).expect("BLAKE2s stack offset fits disp8");
    match register {
        0 => code.bytes(&[0x89, 0x44, 0x24, displacement]), // eax
        1 => code.bytes(&[0x89, 0x4c, 0x24, displacement]), // ecx
        2 => code.bytes(&[0x89, 0x54, 0x24, displacement]), // edx
        8 => code.bytes(&[0x44, 0x89, 0x44, 0x24, displacement]), // r8d
        _ => unreachable!(),
    }
}

fn emit_blake2s_g(code: &mut X86FunctionEncoder, lanes: [usize; 4], message_words: [usize; 2]) {
    let [a, b, c, d] = lanes.map(|lane| lane * 4);
    let [x, y] = message_words.map(|word| 64 + word * 4);
    emit_stack_dword_load(code, 0, a);
    emit_stack_dword_load(code, 1, b);
    emit_stack_dword_load(code, 2, c);
    emit_stack_dword_load(code, 8, d);
    code.bytes(&[0x01, 0xc8]); // a += b
    code.bytes(&[0x03, 0x44, 0x24, u8::try_from(x).unwrap()]); // a += x
    code.bytes(&[0x41, 0x31, 0xc0, 0x41, 0xc1, 0xc8, 16]); // d = ror(d ^ a, 16)
    code.bytes(&[0x44, 0x01, 0xc2]); // c += d
    code.bytes(&[0x31, 0xd1, 0xc1, 0xc9, 12]); // b = ror(b ^ c, 12)
    code.bytes(&[0x01, 0xc8]); // a += b
    code.bytes(&[0x03, 0x44, 0x24, u8::try_from(y).unwrap()]); // a += y
    code.bytes(&[0x41, 0x31, 0xc0, 0x41, 0xc1, 0xc8, 8]); // d = ror(d ^ a, 8)
    code.bytes(&[0x44, 0x01, 0xc2]); // c += d
    code.bytes(&[0x31, 0xd1, 0xc1, 0xc9, 7]); // b = ror(b ^ c, 7)
    emit_stack_dword_store(code, 0, a);
    emit_stack_dword_store(code, 1, b);
    emit_stack_dword_store(code, 2, c);
    emit_stack_dword_store(code, 8, d);
}

fn entropy_mix_blake2s_pool_sample() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]); // sample pointer
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // pool pointer
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xf7]); // storage must be distinct
    code.jump_if(0x84, "fail");
    code.bytes(&[0x49, 0x89, 0xfa, 0x49, 0x89, 0xf3]); // r10=sample, r11=pool
    code.bytes(&[0x48, 0x81, 0xec, 0x80, 0, 0, 0]); // v[16], m[16]

    for word in 0_u8..8 {
        code.bytes(&[0x41, 0x8b, 0x42, word * 4]);
        code.bytes(&[0x89, 0x44, 0x24, 64 + word * 4]);
    }
    for word in 8_u8..16 {
        code.bytes(&[0xc7, 0x44, 0x24, 64 + word * 4, 0, 0, 0, 0]);
    }

    let mut initial = [0_u32; 16];
    initial[..8].copy_from_slice(&BLAKE2S_IV);
    initial[0] ^= 0x0101_0020; // digest length 32, fanout one, depth one
    initial[6] ^= u32::from_le_bytes(*b"Topa");
    initial[7] ^= u32::from_le_bytes(*b"lRNG");
    initial[8..].copy_from_slice(&BLAKE2S_IV);
    initial[12] ^= 32; // final block contains 32 bytes
    initial[14] ^= u32::MAX; // final-block flag
    for (lane, value) in initial.into_iter().enumerate() {
        code.bytes(&[0xc7, 0x44, 0x24, u8::try_from(lane * 4).unwrap()]);
        code.bytes(&value.to_le_bytes());
    }

    for sigma in BLAKE2S_SIGMA {
        for (lanes, pair) in [
            ([0, 4, 8, 12], [sigma[0], sigma[1]]),
            ([1, 5, 9, 13], [sigma[2], sigma[3]]),
            ([2, 6, 10, 14], [sigma[4], sigma[5]]),
            ([3, 7, 11, 15], [sigma[6], sigma[7]]),
            ([0, 5, 10, 15], [sigma[8], sigma[9]]),
            ([1, 6, 11, 12], [sigma[10], sigma[11]]),
            ([2, 7, 8, 13], [sigma[12], sigma[13]]),
            ([3, 4, 9, 14], [sigma[14], sigma[15]]),
        ] {
            emit_blake2s_g(&mut code, lanes, pair);
        }
    }

    let mut hash = BLAKE2S_IV;
    hash[0] ^= 0x0101_0020;
    hash[6] ^= u32::from_le_bytes(*b"Topa");
    hash[7] ^= u32::from_le_bytes(*b"lRNG");
    for (word, value) in hash.into_iter().enumerate() {
        emit_stack_dword_load(&mut code, 0, word * 4);
        code.bytes(&[0x33, 0x44, 0x24, u8::try_from((word + 8) * 4).unwrap()]);
        code.bytes(&[0x35]);
        code.bytes(&value.to_le_bytes());
        code.bytes(&[0x41, 0x89, 0x43, u8::try_from(word * 4).unwrap()]);
    }

    code.bytes(&[0x49, 0x8b, 0x03]);
    for offset in [8_u8, 16, 24] {
        code.bytes(&[0x49, 0x0b, 0x43, offset]);
    }
    code.bytes(&[0x48, 0x85, 0xc0]);
    code.jump_if(0x84, "fail_stack");

    code.bytes(&[0x31, 0xc0]);
    for offset in [0_u8, 8, 16, 24] {
        code.bytes(&[0x49, 0x89, 0x42, offset]);
    }
    for offset in [0_u8, 8, 16, 24] {
        code.bytes(&[0x49, 0x0b, 0x42, offset]);
    }
    code.bytes(&[0x48, 0x85, 0xc0]);
    code.jump_if(0x85, "fail_stack");
    code.bytes(&[0x48, 0x81, 0xc4, 0x80, 0, 0, 0, 0xb8, 1, 0, 0, 0, 0xc3]);

    code.bind("fail_stack")?;
    code.bytes(&[0x31, 0xc0]);
    for offset in [0_u8, 8, 16, 24] {
        code.bytes(&[0x49, 0x89, 0x43, offset, 0x49, 0x89, 0x42, offset]);
    }
    code.bytes(&[0x48, 0x81, 0xc4, 0x80, 0, 0, 0]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn entropy_initialize_chacha20_generator() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xf7]);
    code.jump_if(0x84, "fail");
    for offset in [0_u8, 8, 16, 24] {
        if offset == 0 {
            code.bytes(&[0x48, 0x8b, 0x07, 0x48, 0x89, 0x06, 0x48, 0x3b, 0x06]);
        } else {
            code.bytes(&[
                0x48, 0x8b, 0x47, offset, 0x48, 0x89, 0x46, offset, 0x48, 0x3b, 0x46, offset,
            ]);
        }
        code.jump_if(0x85, "fail_wipe");
    }
    code.bytes(&[0x31, 0xc0]);
    for offset in [0_u8, 8, 16, 24] {
        if offset == 0 {
            code.bytes(&[0x48, 0x89, 0x07]);
        } else {
            code.bytes(&[0x48, 0x89, 0x47, offset]);
        }
    }
    for offset in [0_u8, 8, 16, 24] {
        if offset == 0 {
            code.bytes(&[0x48, 0x0b, 0x07]);
        } else {
            code.bytes(&[0x48, 0x0b, 0x47, offset]);
        }
    }
    code.bytes(&[0x48, 0x85, 0xc0]);
    code.jump_if(0x85, "fail_wipe");
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail_wipe")?;
    code.bytes(&[0x31, 0xc0]);
    for offset in [0_u8, 8, 16, 24] {
        if offset == 0 {
            code.bytes(&[0x48, 0x89, 0x07, 0x48, 0x89, 0x06]);
        } else {
            code.bytes(&[0x48, 0x89, 0x47, offset, 0x48, 0x89, 0x46, offset]);
        }
    }
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn emit_chacha20_quarter_round(code: &mut X86FunctionEncoder, lanes: [usize; 4]) {
    let [a, b, c, d] = lanes.map(|lane| 64 + lane * 4);
    emit_stack_dword_load(code, 0, a);
    emit_stack_dword_load(code, 1, b);
    emit_stack_dword_load(code, 2, c);
    emit_stack_dword_load(code, 8, d);
    code.bytes(&[0x01, 0xc8]); // a += b
    code.bytes(&[0x41, 0x31, 0xc0, 0x41, 0xc1, 0xc0, 16]); // d = rol(d ^ a, 16)
    code.bytes(&[0x44, 0x01, 0xc2]); // c += d
    code.bytes(&[0x31, 0xd1, 0xc1, 0xc1, 12]); // b = rol(b ^ c, 12)
    code.bytes(&[0x01, 0xc8]); // a += b
    code.bytes(&[0x41, 0x31, 0xc0, 0x41, 0xc1, 0xc0, 8]); // d = rol(d ^ a, 8)
    code.bytes(&[0x44, 0x01, 0xc2]); // c += d
    code.bytes(&[0x31, 0xd1, 0xc1, 0xc1, 7]); // b = rol(b ^ c, 7)
    emit_stack_dword_store(code, 0, a);
    emit_stack_dword_store(code, 1, b);
    emit_stack_dword_store(code, 2, c);
    emit_stack_dword_store(code, 8, d);
}

fn entropy_extract_chacha20_generator() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]); // key pointer
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // output pointer
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xf7]); // storage must be distinct
    code.jump_if(0x84, "fail");
    code.bytes(&[0x49, 0x89, 0xfa, 0x49, 0x89, 0xf3]); // r10=key, r11=output
    code.bytes(&[0x48, 0x81, 0xec, 0x80, 0, 0, 0]); // input[16], working[16]

    for (word, value) in [0x6170_7865_u32, 0x3320_646e, 0x7962_2d32, 0x6b20_6574]
        .into_iter()
        .enumerate()
    {
        code.bytes(&[0xc7, 0x44, 0x24, u8::try_from(word * 4).unwrap()]);
        code.bytes(&value.to_le_bytes());
    }
    for word in 0_u8..8 {
        code.bytes(&[0x41, 0x8b, 0x42, word * 4]);
        code.bytes(&[0x89, 0x44, 0x24, 16 + word * 4]);
    }
    for word in 12_u8..16 {
        code.bytes(&[0xc7, 0x44, 0x24, word * 4, 0, 0, 0, 0]);
    }
    for word in 0..16 {
        emit_stack_dword_load(&mut code, 0, word * 4);
        emit_stack_dword_store(&mut code, 0, 64 + word * 4);
    }

    for _ in 0..10 {
        for lanes in [
            [0, 4, 8, 12],
            [1, 5, 9, 13],
            [2, 6, 10, 14],
            [3, 7, 11, 15],
            [0, 5, 10, 15],
            [1, 6, 11, 12],
            [2, 7, 8, 13],
            [3, 4, 9, 14],
        ] {
            emit_chacha20_quarter_round(&mut code, lanes);
        }
    }
    for word in 0..16 {
        emit_stack_dword_load(&mut code, 0, 64 + word * 4);
        code.bytes(&[0x03, 0x44, 0x24, u8::try_from(word * 4).unwrap()]);
        emit_stack_dword_store(&mut code, 0, 64 + word * 4);
    }

    // Replace and verify the persistent key before publishing any output.
    for word in 0_u8..8 {
        emit_stack_dword_load(&mut code, 0, 64 + usize::from(word) * 4);
        code.bytes(&[0x41, 0x89, 0x42, word * 4]);
        code.bytes(&[0x41, 0x3b, 0x42, word * 4]);
        code.jump_if(0x85, "fail_stack");
    }
    for word in 0_u8..8 {
        emit_stack_dword_load(&mut code, 0, 96 + usize::from(word) * 4);
        code.bytes(&[0x41, 0x89, 0x43, word * 4]);
        code.bytes(&[0x41, 0x3b, 0x43, word * 4]);
        code.jump_if(0x85, "fail_stack");
    }

    code.bytes(&[0x31, 0xc0]);
    for offset in (0_u8..128).step_by(8) {
        if offset == 0 {
            code.bytes(&[0x48, 0x89, 0x04, 0x24]);
        } else {
            code.bytes(&[0x48, 0x89, 0x44, 0x24, offset]);
        }
    }
    code.bytes(&[0x48, 0x8b, 0x04, 0x24]);
    for offset in (8_u8..128).step_by(8) {
        code.bytes(&[0x48, 0x0b, 0x44, 0x24, offset]);
    }
    code.bytes(&[0x48, 0x85, 0xc0]);
    code.jump_if(0x85, "fail_stack");
    code.bytes(&[0x48, 0x81, 0xc4, 0x80, 0, 0, 0, 0xb8, 1, 0, 0, 0, 0xc3]);

    code.bind("fail_stack")?;
    code.bytes(&[0x31, 0xc0]);
    for offset in [0_u8, 8, 16, 24] {
        code.bytes(&[0x49, 0x89, 0x42, offset, 0x49, 0x89, 0x43, offset]);
    }
    for offset in (0_u8..128).step_by(8) {
        if offset == 0 {
            code.bytes(&[0x48, 0x89, 0x04, 0x24]);
        } else {
            code.bytes(&[0x48, 0x89, 0x44, 0x24, offset]);
        }
    }
    code.bytes(&[0x48, 0x81, 0xc4, 0x80, 0, 0, 0]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn entropy_deliver_initial_auxv_random() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]); // prepared address-space state
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // affine 32-byte output
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xf7]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x3f, 0x01]); // prepared probe, not entered
    code.jump_if(0x85, "fail");
    code.bytes(&[0x80, 0xbf, 0xd0, 0, 0, 0, 0]); // not previously delivered
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x8b, 0x4f, 0x68]); // allocator-owned backing
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");

    for offset in [0_u8, 8] {
        if offset == 0 {
            code.bytes(&[0x48, 0x8b, 0x06]);
        } else {
            code.bytes(&[0x48, 0x8b, 0x46, offset]);
        }
        code.bytes(&[0x48, 0x89, 0x81]);
        code.bytes(&(0x1fe0_u32 + u32::from(offset)).to_le_bytes());
        code.bytes(&[0x48, 0x3b, 0x81]);
        code.bytes(&(0x1fe0_u32 + u32::from(offset)).to_le_bytes());
        code.jump_if(0x85, "fail_wipe");
    }
    code.bytes(&[0x31, 0xc0]);
    for offset in [0_u8, 8, 16, 24] {
        if offset == 0 {
            code.bytes(&[0x48, 0x89, 0x06]);
        } else {
            code.bytes(&[0x48, 0x89, 0x46, offset]);
        }
    }
    code.bytes(&[0x48, 0x8b, 0x06]);
    for offset in [8_u8, 16, 24] {
        code.bytes(&[0x48, 0x0b, 0x46, offset]);
    }
    code.bytes(&[0x48, 0x85, 0xc0]);
    code.jump_if(0x85, "fail_wipe");
    code.bytes(&[0xc6, 0x87, 0xd0, 0, 0, 0, 1]);
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);

    code.bind("fail_wipe")?;
    code.bytes(&[0x31, 0xc0]);
    for offset in [0_u8, 8, 16, 24] {
        if offset == 0 {
            code.bytes(&[0x48, 0x89, 0x06]);
        } else {
            code.bytes(&[0x48, 0x89, 0x46, offset]);
        }
    }
    for offset in [0x1fe0_u32, 0x1fe8] {
        code.bytes(&[0x48, 0x89, 0x81]);
        code.bytes(&offset.to_le_bytes());
    }
    code.bytes(&[0xc6, 0x87, 0xd0, 0, 0, 0, 0]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn entropy_reserve_initial_user_random() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]); // prepared address-space state
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // affine 32-byte output
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xf7]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x3f, 0x01]); // prepared probe, not entered
    code.jump_if(0x85, "fail");
    code.bytes(&[0x80, 0xbf, 0xd0, 0, 0, 0, 1]); // AT_RANDOM delivered
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x83, 0xbf, 0xd8, 0, 0, 0, 0]); // no live pointer
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x89, 0xb7, 0xd8, 0, 0, 0]);
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn entropy_bind_reusable_user_random() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]); // prepared address-space state
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // live generator key
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xd2]); // fresh service state
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xf7]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xd7]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xd6]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0xbf, 0xd0, 0, 0, 0, 1]); // AT_RANDOM delivered
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x8b, 0x8f, 0xd8, 0, 0, 0]); // reserved first output
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xf9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xd1]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x39, 0xf1]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x31, 0xc0]);
    for offset in [0_u8, 8, 16, 24] {
        if offset == 0 {
            code.bytes(&[0x48, 0x0b, 0x02]);
        } else {
            code.bytes(&[0x48, 0x0b, 0x42, offset]);
        }
    }
    code.bytes(&[0x48, 0x85, 0xc0]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x89, 0x32]);
    code.bytes(&[0x48, 0x89, 0x4a, 0x08]);
    code.bytes(&[0xc6, 0x42, 0x10, 0x01]); // reserved first output is live
    code.bytes(&[0x48, 0x89, 0x97, 0xd8, 0, 0, 0]);
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn pci_region_write_dma_reference() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x81, 0xff, 0x84, 0x28, 0x00, 0x80]); // pinned access selector
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xfe, 0x04]); // BAR four
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0xf7, 0xc1, 0xff, 0x0f, 0x00, 0x00]); // page-aligned buffer
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xfa, 0x20]); // descriptor register
    code.jump_if(0x85, "driver");
    code.bytes(&[0x45, 0x85, 0xc0]); // buffer offset zero
    code.jump_if(0x85, "fail");
    code.jump("validated");
    code.bind("driver")?;
    code.bytes(&[0x83, 0xfa, 0x28]); // driver register
    code.jump_if(0x85, "device");
    code.bytes(&[0x41, 0x81, 0xf8, 0x80, 0x00, 0x00, 0x00]); // buffer offset 128
    code.jump_if(0x85, "fail");
    code.jump("validated");
    code.bind("device")?;
    code.bytes(&[0x83, 0xfa, 0x30]); // device register
    code.jump_if(0x85, "fail");
    code.bytes(&[0x41, 0x81, 0xf8, 0x94, 0x00, 0x00, 0x00]); // buffer offset 148
    code.jump_if(0x85, "fail");
    code.bind("validated")?;
    code.bytes(&[0x4c, 0x01, 0xc1, 0x49, 0x89, 0xcb]); // reference in r11
    code.bytes(&[0x41, 0x89, 0xfa, 0x41, 0x89, 0xd1]); // selector r10d, target r9d

    code.bytes(&[0x41, 0x8d, 0x42, 0x04, 0x66, 0xba, 0xf8, 0x0c, 0xef]);
    code.bytes(&[0x66, 0xba, 0xfc, 0x0c, 0x89, 0xf0, 0xef]); // BAR
    code.bytes(&[0x41, 0x8d, 0x42, 0x08, 0x66, 0xba, 0xf8, 0x0c, 0xef]);
    code.bytes(&[0x66, 0xba, 0xfc, 0x0c, 0x44, 0x89, 0xc8, 0xef]); // target
    code.bytes(&[0x41, 0x8d, 0x42, 0x0c, 0x66, 0xba, 0xf8, 0x0c, 0xef]);
    code.bytes(&[0x66, 0xba, 0xfc, 0x0c, 0xb8, 0x04, 0x00, 0x00, 0x00, 0xef]); // dword
    code.bytes(&[0x41, 0x8d, 0x42, 0x10, 0x66, 0xba, 0xf8, 0x0c, 0xef]);
    code.bytes(&[0x66, 0xba, 0xfc, 0x0c, 0x44, 0x89, 0xd8, 0xef]); // low dword

    code.bytes(&[0x41, 0x83, 0xc1, 0x04]); // target high dword
    code.bytes(&[0x41, 0x8d, 0x42, 0x08, 0x66, 0xba, 0xf8, 0x0c, 0xef]);
    code.bytes(&[0x66, 0xba, 0xfc, 0x0c, 0x44, 0x89, 0xc8, 0xef]);
    code.bytes(&[0x41, 0x8d, 0x42, 0x10, 0x66, 0xba, 0xf8, 0x0c, 0xef]);
    code.bytes(&[0x66, 0xba, 0xfc, 0x0c, 0x4c, 0x89, 0xd8]);
    code.bytes(&[0x48, 0xc1, 0xe8, 0x20, 0xef]); // high dword
    code.bytes(&[0xb8, 1, 0, 0, 0, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
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
        X86_SYSTEMS_PCI_REGION_READ_SYMBOL,
        &pci_region_configuration_gateway(PciRegionGatewayOperation::ReadEquals)?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_PCI_REGION_READ_EVENTUALLY_SYMBOL,
        &pci_region_configuration_gateway(PciRegionGatewayOperation::ReadEventuallyEquals)?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_PCI_REGION_WRITE_SYMBOL,
        &pci_region_configuration_gateway(PciRegionGatewayOperation::Write)?,
    );
    append_function(
        &mut object,
        text,
        X86_SYSTEMS_PCI_CONFIGURATION_READ_DWORD_SYMBOL,
        &[
            X86Instruction::MovePciAddressPortToDx,
            X86Instruction::MoveDwordSelectorToEax,
            X86Instruction::OutputDwordToPort,
            X86Instruction::MovePciDataPortToDx,
            X86Instruction::InputDwordFromPort,
            X86Instruction::CompareDwordWithExpected,
            X86Instruction::SetEqualAl,
            X86Instruction::ZeroExtendAlToEax,
            X86Instruction::Return,
        ],
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD_SYMBOL,
        &pci_configuration_update_word()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_DMA_BUFFER_CREATE_SYMBOL,
        &dma_buffer_create()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_DMA_BUFFER_BIND_REFERENCE_SYMBOL,
        &dma_buffer_bind_reference()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_DMA_BUFFER_STORE_INTEGER_SYMBOL,
        &dma_buffer_store_integer()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_DMA_BUFFER_PREPARE_DEVICE_SYMBOL,
        &dma_buffer_prepare_device()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_DMA_BUFFER_COMPLETE_CPU_SYMBOL,
        &dma_buffer_prepare_device()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_DMA_BUFFER_LOAD_INTEGER_SYMBOL,
        &dma_buffer_load_integer()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_ENTROPY_SAMPLE_CAPTURE_SYMBOL,
        &entropy_capture_dma_sample()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_ENTROPY_POOL_MIX_SYMBOL,
        &entropy_mix_blake2s_pool_sample()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_ENTROPY_GENERATOR_INITIALIZE_SYMBOL,
        &entropy_initialize_chacha20_generator()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_ENTROPY_GENERATOR_EXTRACT_SYMBOL,
        &entropy_extract_chacha20_generator()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM_SYMBOL,
        &entropy_deliver_initial_auxv_random()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM_SYMBOL,
        &entropy_reserve_initial_user_random()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE_SYMBOL,
        &entropy_bind_reusable_user_random()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE_SYMBOL,
        &pci_region_write_dma_reference()?,
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
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_PREPARE_SYMBOL,
        &static_image_prepare_initial_leaves()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_ZERO_PAYLOAD_SYMBOL,
        &static_image_zero_payload()?,
    );
    append_encoded_function(
        &mut object,
        text,
        X86_SYSTEMS_STATIC_IMAGE_REVOKE_SYMBOL,
        &static_image_revoke_initial_leaves()?,
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
    let dma_storage = object.add_section(
        Vec::new(),
        X86_SYSTEMS_DMA_STORAGE_SECTION.as_bytes().to_vec(),
        SectionKind::UninitializedData,
    );
    let control_offset = object.append_section_bss(dma_storage, 4096, 4096);
    object.add_symbol(Symbol {
        name: X86_SYSTEMS_DMA_VIRTQUEUE_CONTROL_SYMBOL.as_bytes().to_vec(),
        value: control_offset,
        size: 4096,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(dma_storage),
        flags: SymbolFlags::None,
    });
    let payload_offset = object.append_section_bss(dma_storage, 4096, 4096);
    object.add_symbol(Symbol {
        name: X86_SYSTEMS_DMA_ENTROPY_PAYLOAD_SYMBOL.as_bytes().to_vec(),
        value: payload_offset,
        size: 4096,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(dma_storage),
        flags: SymbolFlags::None,
    });

    let entropy_storage = object.add_section(
        Vec::new(),
        X86_SYSTEMS_ENTROPY_STORAGE_SECTION.as_bytes().to_vec(),
        SectionKind::UninitializedData,
    );
    let sample_offset = object.append_section_bss(entropy_storage, 32, 32);
    object.add_symbol(Symbol {
        name: X86_SYSTEMS_RAW_ENTROPY_SAMPLE_SYMBOL.as_bytes().to_vec(),
        value: sample_offset,
        size: 32,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(entropy_storage),
        flags: SymbolFlags::None,
    });
    let pool_offset = object.append_section_bss(entropy_storage, 32, 32);
    object.add_symbol(Symbol {
        name: X86_SYSTEMS_ENTROPY_POOL_STATE_SYMBOL.as_bytes().to_vec(),
        value: pool_offset,
        size: 32,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(entropy_storage),
        flags: SymbolFlags::None,
    });
    let generator_offset = object.append_section_bss(entropy_storage, 32, 32);
    object.add_symbol(Symbol {
        name: X86_SYSTEMS_ENTROPY_GENERATOR_KEY_SYMBOL.as_bytes().to_vec(),
        value: generator_offset,
        size: 32,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(entropy_storage),
        flags: SymbolFlags::None,
    });
    let output_offset = object.append_section_bss(entropy_storage, 32, 32);
    object.add_symbol(Symbol {
        name: X86_SYSTEMS_ENTROPY_OUTPUT_SYMBOL.as_bytes().to_vec(),
        value: output_offset,
        size: 32,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(entropy_storage),
        flags: SymbolFlags::None,
    });
    let user_random_service_offset = object.append_section_bss(entropy_storage, 32, 32);
    object.add_symbol(Symbol {
        name: X86_SYSTEMS_ENTROPY_USER_RANDOM_SERVICE_SYMBOL
            .as_bytes()
            .to_vec(),
        value: user_random_service_offset,
        size: 32,
        kind: SymbolKind::Data,
        scope: SymbolScope::Linkage,
        weak: false,
        section: SymbolSection::Section(entropy_storage),
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
            X86_SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS_SYMBOL,
            user_reserve_initial_anonymous_mapping()?,
        ),
        (
            X86_SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS_SYMBOL,
            user_release_initial_anonymous_mapping()?,
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
    let static_user_image = qualified_x86_64_static_user_image_fixture();
    let expected_boot_resource =
        generate_initial_boot_resource(&initial_user_elf_file()?, &static_user_image)?;
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
    let static_payload_offset = X86_STATIC_BOOT_RESOURCE_PAYLOAD_OFFSET as usize;
    let static_metadata_end = static_payload_offset + 64 + 11 * 56;
    let static_image_end = static_payload_offset + static_user_image.len();
    let metadata_offsets = (0..X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize)
        .chain(
            X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize
                ..X86_INITIAL_BOOT_RESOURCE_PAYLOAD_OFFSET as usize + 256,
        )
        .chain(image_end..static_payload_offset)
        .chain(static_payload_offset..static_metadata_end)
        .chain(static_image_end..expected_boot_resource.len())
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
    code.bytes(&[0x48, 0x3d, 0x08, 0x11, 0x00, 0x00]); // includes second-page entry qword
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x3d, 0x00, 0x20, 0x00, 0x00]); // two base pages
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
    code.bytes(&(INITIAL_USER_CODE_ADDRESS as u32 + 4096 + 256).to_le_bytes());
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
        let displacement = u32::try_from(0x1eb0 + index * 8).map_err(|_| {
            CompileError::Tool("initial user stack displacement exceeds u32".into())
        })?;
        code.bytes(&[0x49, 0x89, 0x82]);
        code.bytes(&displacement.to_le_bytes());
    }
    code.bytes(&[0x4d, 0x8b, 0x99, 0x88, 0x00, 0x00, 0x00]);
    code.bytes(&[0x4d, 0x89, 0x9a, 0xe8, 0x1e, 0x00, 0x00]); // runtime AT_ENTRY
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

    code.bytes(&[0xc6, 0x87, 0xc0, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0xc6, 0x87, 0xd0, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0xc7, 0x87, 0xd8, 0x00, 0x00, 0x00, 0, 0, 0, 0]);
    code.bytes(&[
        0x48, 0xc7, 0x87, 0xe0, 0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x00,
    ]);
    code.bytes(&[0x48, 0xc7, 0x87, 0xe8, 0x00, 0x00, 0x00, 0, 0, 0, 0]);
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
    code.bytes(&[0x3c, 0x04]);
    code.jump_if(0x84, "code-extent-end");
    code.bytes(&[0x49, 0x81, 0xc0, 0x00, 0x10, 0x00, 0x00]);
    code.jump("extent-end-ready");
    code.bind("code-extent-end")?;
    code.bytes(&[0x49, 0x81, 0xc0, 0x00, 0x20, 0x00, 0x00]);
    code.bind("extent-end-ready")?;
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
    code.bytes(&[0x48, 0x83, 0xc1, 0x10]);
    code.bind("pte-slot-selected")?;
    code.bytes(&[0x48, 0x83, 0x79, 0x18, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x3c, 0x04]);
    code.jump_if(0x85, "leaf-slots-clear");
    code.bytes(&[0x48, 0x83, 0x79, 0x20, 0x00]);
    code.jump_if(0x85, "fail");
    code.bind("leaf-slots-clear")?;
    code.bytes(&[0x49, 0x89, 0xd0]);
    code.bytes(&[0x49, 0xb9]);
    code.bytes(&0x8000_0000_0000_0007_u64.to_le_bytes());
    code.bytes(&[0x4d, 0x09, 0xc8, 0x4c, 0x89, 0x41, 0x18]);
    code.bytes(&[0x3c, 0x04]);
    code.jump_if(0x85, "leaves-published");
    code.bytes(&[0x49, 0x81, 0xc0, 0x00, 0x10, 0x00, 0x00]);
    code.bytes(&[0x4c, 0x89, 0x41, 0x20]);
    code.bind("leaves-published")?;
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x3c, 0x04]);
    code.jump_if(0x84, "publish-generation-one");
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x0060_5000_u64.to_le_bytes());
    code.bytes(&[0x48, 0x89, 0x97, 0x80, 0x00, 0x00, 0x00]);
    code.jump("publish-mapping");
    code.bind("publish-generation-one")?;
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x0060_3000_u64.to_le_bytes());
    code.bytes(&[0x48, 0x89, 0x57, 0x78]);
    code.bind("publish-mapping")?;
    code.bytes(&[0x41, 0x0f, 0x01, 0x3a]);
    code.bytes(&[0x3c, 0x04]);
    code.jump_if(0x85, "mapping-invalidated");
    code.bytes(&[0x49, 0x81, 0xc2, 0x00, 0x10, 0x00, 0x00]);
    code.bytes(&[0x41, 0x0f, 0x01, 0x3a]);
    code.bind("mapping-invalidated")?;
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

fn initial_user_stack_bytes() -> [u8; 336] {
    let mut bytes = [0_u8; 336];
    bytes[0..8].copy_from_slice(&1_u64.to_le_bytes());
    bytes[8..16].copy_from_slice(&0x0060_1fc8_u64.to_le_bytes());
    bytes[32..40].copy_from_slice(&6_u64.to_le_bytes());
    bytes[40..48].copy_from_slice(&4096_u64.to_le_bytes());
    bytes[48..56].copy_from_slice(&9_u64.to_le_bytes());
    bytes[56..64].copy_from_slice(&0_u64.to_le_bytes());
    bytes[64..72].copy_from_slice(&31_u64.to_le_bytes());
    bytes[72..80].copy_from_slice(&0x0060_1fc8_u64.to_le_bytes());
    for (offset, tag) in [(80, 11_u64), (96, 12), (112, 13), (128, 14)] {
        bytes[offset..offset + 8].copy_from_slice(&tag.to_le_bytes());
    }
    bytes[144..152].copy_from_slice(&3_u64.to_le_bytes());
    bytes[152..160].copy_from_slice(&INITIAL_USER_PROGRAM_HEADER_ADDRESS.to_le_bytes());
    bytes[160..168].copy_from_slice(&4_u64.to_le_bytes());
    bytes[168..176].copy_from_slice(&56_u64.to_le_bytes());
    bytes[176..184].copy_from_slice(&5_u64.to_le_bytes());
    bytes[184..192].copy_from_slice(&3_u64.to_le_bytes());
    bytes[192..200].copy_from_slice(&23_u64.to_le_bytes());
    bytes[208..216].copy_from_slice(&17_u64.to_le_bytes());
    bytes[216..224].copy_from_slice(&100_u64.to_le_bytes());
    bytes[224..232].copy_from_slice(&15_u64.to_le_bytes());
    bytes[232..240].copy_from_slice(&0x0060_1fc0_u64.to_le_bytes());
    bytes[240..248].copy_from_slice(&25_u64.to_le_bytes());
    bytes[248..256].copy_from_slice(&0x0060_1fe0_u64.to_le_bytes());
    bytes[272..279].copy_from_slice(b"x86_64\0");
    bytes[280..291].copy_from_slice(b"topal-probe");
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
    let image_size = (4096_u64 + 256)
        .checked_add(u64::try_from(payload.bytes.len()).unwrap_or(u64::MAX))
        .ok_or_else(|| CompileError::Tool("constructed ELF image size overflow".into()))?;
    let mut bytes = initial_user_elf_prefix(image_size).to_vec();
    bytes.resize(4096 + 256, 0x90);
    bytes.extend(payload.bytes);
    if bytes.len() > INITIAL_USER_CODE_CAPACITY {
        return Err(CompileError::Tool(
            "constructed ELF image exceeds its sealed two-page executable extent".into(),
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
    entry.bytes(&0x0060_1fc8_u32.to_le_bytes()); // argv[0]
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
    entry.bytes(&0x0060_1fc8_u32.to_le_bytes()); // same string as argv[0]
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
        (0xc0, 23),
        (0xc8, 0),
        (0xd0, 17),
        (0xd8, 100),
        (0xe0, 15),
        (0xf0, 25),
        (0x100, 0),
        (0x108, 0),
    ] {
        entry.bytes(&[0x48, 0x83, 0xbc, 0x24]);
        entry.bytes(&displacement.to_le_bytes());
        entry.bytes(&[expected]);
        entry.jump_if(0x85, "entry-fail");
    }
    entry.bytes(&[0x48, 0x81, 0xbc, 0x24]);
    entry.bytes(&0xe8_u32.to_le_bytes());
    entry.bytes(&0x0060_1fc0_u32.to_le_bytes());
    entry.jump_if(0x85, "entry-fail");
    entry.bytes(&[0x48, 0x81, 0xbc, 0x24]);
    entry.bytes(&0xf8_u32.to_le_bytes());
    entry.bytes(&0x0060_1fe0_u32.to_le_bytes());
    entry.jump_if(0x85, "entry-fail");
    entry.bytes(&[0x48, 0xb8]);
    entry.bytes(&u64::from_le_bytes(*b"x86_64\0\0").to_le_bytes());
    entry.bytes(&[0x48, 0x39, 0x84, 0x24]);
    entry.bytes(&0x110_u32.to_le_bytes());
    entry.jump_if(0x85, "entry-fail");
    entry.bytes(&[0x48, 0xb8]);
    entry.bytes(&u64::from_le_bytes(*b"topal-pr").to_le_bytes());
    entry.bytes(&[0x48, 0x39, 0x84, 0x24]);
    entry.bytes(&0x118_u32.to_le_bytes());
    entry.jump_if(0x85, "entry-fail");
    entry.bytes(&[0x81, 0xbc, 0x24]);
    entry.bytes(&0x120_u32.to_le_bytes());
    entry.bytes(&u32::from_le_bytes(*b"obe\0").to_le_bytes());
    entry.jump_if(0x85, "entry-fail");
    entry.bytes(&[0x48, 0x8b, 0x84, 0x24]);
    entry.bytes(&0x130_u32.to_le_bytes()); // read complete first random qword
    entry.bytes(&[0x48, 0x8b, 0x8c, 0x24]);
    entry.bytes(&0x138_u32.to_le_bytes()); // read complete second random qword
    entry.bytes(&[0x48, 0x09, 0xc8]); // consume both values without constraining them
    entry.bytes(&[0x48, 0x81, 0xc4]);
    entry.bytes(&336_u32.to_le_bytes()); // consume bounded initial block
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
    continuation.bytes(&[0x48, 0x83, 0xec, 0x20]); // two native timespecs
    continuation.bytes(&[0xbf, 0x01, 0x00, 0x00, 0x00, 0x48, 0x89, 0xe6]);
    continuation.bytes(&[0xb8, 0xe4, 0x00, 0x00, 0x00, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x48, 0x81, 0x7c, 0x24, 0x08]);
    continuation.bytes(&1_000_000_000_u32.to_le_bytes());
    continuation.jump_if(0x83, "fail");
    continuation.bytes(&[0xbf, 0x01, 0x00, 0x00, 0x00]);
    continuation.bytes(&[0x48, 0x8d, 0x74, 0x24, 0x10]);
    continuation.bytes(&[0xb8, 0xe4, 0x00, 0x00, 0x00, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x48, 0x81, 0x7c, 0x24, 0x18]);
    continuation.bytes(&1_000_000_000_u32.to_le_bytes());
    continuation.jump_if(0x83, "fail");
    continuation.bytes(&[0x48, 0x8b, 0x44, 0x24, 0x10, 0x48, 0x3b, 0x04, 0x24]);
    continuation.jump_if(0x87, "clock-gettime-order-ok");
    continuation.jump_if(0x82, "fail");
    continuation.bytes(&[0x48, 0x8b, 0x44, 0x24, 0x18]);
    continuation.bytes(&[0x48, 0x3b, 0x44, 0x24, 0x08]);
    continuation.jump_if(0x82, "fail");
    continuation.bind("clock-gettime-order-ok")?;
    continuation.bytes(&[0x48, 0x83, 0xc4, 0x20]);
    continuation.bytes(&[0x48, 0x83, 0xec, 0x10]);
    continuation.bytes(&[0xbf, 0x01, 0x00, 0x00, 0x00, 0x48, 0x89, 0xe6]);
    continuation.bytes(&[0xb8, 0xe5, 0x00, 0x00, 0x00, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x48, 0x83, 0x3c, 0x24, 0x00]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x48, 0x83, 0x7c, 0x24, 0x08, 0x0a]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xbf, 0x01, 0x00, 0x00, 0x00, 0x31, 0xf6]);
    continuation.bytes(&[0xb8, 0xe5, 0x00, 0x00, 0x00, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x48, 0x83, 0xc4, 0x10]);
    continuation.bytes(&[0xb8, 0x18, 0x00, 0x00, 0x00, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xb8, 0xff, 0xff, 0xff, 0x7f, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x83, 0xf8, 0xda]);
    continuation.jump_if(0x85, "fail");
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
    continuation.bytes(&[0x48, 0x83, 0xec, 0x10]); // native rlimit pair
    continuation.bytes(&[0xbf, 0x03, 0x00, 0x00, 0x00]); // RLIMIT_STACK
    continuation.bytes(&[0x48, 0x89, 0xe6]);
    continuation.bytes(&[0xb8, 0x61, 0x00, 0x00, 0x00, 0x0f, 0x05]); // getrlimit
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "rlimit-buffer-fail");
    continuation.bytes(&[0x48, 0x81, 0x3c, 0x24, 0x00, 0x00, 0x80, 0x00]);
    continuation.jump_if(0x85, "rlimit-buffer-fail");
    continuation.bytes(&[0x48, 0x83, 0x7c, 0x24, 0x08, 0xff]);
    continuation.jump_if(0x85, "rlimit-buffer-fail");
    continuation.bytes(&[0x31, 0xff]); // current process
    continuation.bytes(&[0xbe, 0x03, 0x00, 0x00, 0x00, 0x31, 0xd2]);
    continuation.bytes(&[0x49, 0x89, 0xe2]); // old-limit candidate
    continuation.bytes(&[0xb8, 0x2e, 0x01, 0x00, 0x00, 0x0f, 0x05]); // prlimit64
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "rlimit-buffer-fail");
    continuation.bytes(&[0x48, 0x81, 0x3c, 0x24, 0x00, 0x00, 0x80, 0x00]);
    continuation.jump_if(0x85, "rlimit-buffer-fail");
    continuation.bytes(&[0x48, 0x83, 0x7c, 0x24, 0x08, 0xff]);
    continuation.jump_if(0x85, "rlimit-buffer-fail");
    continuation.bytes(&[0x48, 0x83, 0xc4, 0x10]);
    continuation.bytes(&[0x48, 0x83, 0xec, 0x10]); // signal set and old set
    continuation.bytes(&[0x48, 0xc7, 0x04, 0x24]);
    continuation.bytes(&0x0004_0300_u32.to_le_bytes()); // USR1, KILL, STOP
    continuation.bytes(&[0x48, 0xc7, 0x44, 0x24, 0x08, 0xff, 0xff, 0xff, 0xff]);
    continuation.bytes(&[0xbf, 0x02, 0x00, 0x00, 0x00]); // SIG_SETMASK
    continuation.bytes(&[0x48, 0x89, 0xe6, 0x48, 0x8d, 0x54, 0x24, 0x08]);
    continuation.bytes(&[0x41, 0xba, 0x08, 0x00, 0x00, 0x00]); // native set size
    continuation.bytes(&[0xb8, 0x0e, 0x00, 0x00, 0x00, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "signal-mask-buffer-fail");
    continuation.bytes(&[0x48, 0x83, 0x7c, 0x24, 0x08, 0x00]);
    continuation.jump_if(0x85, "signal-mask-buffer-fail");
    continuation.bytes(&[0xbf, 0x7f, 0x00, 0x00, 0x00, 0x31, 0xf6]); // null-set query
    continuation.bytes(&[0x48, 0x8d, 0x54, 0x24, 0x08]);
    continuation.bytes(&[0x41, 0xba, 0x08, 0x00, 0x00, 0x00]);
    continuation.bytes(&[0xb8, 0x0e, 0x00, 0x00, 0x00, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "signal-mask-buffer-fail");
    continuation.bytes(&[0x48, 0x81, 0x7c, 0x24, 0x08]);
    continuation.bytes(&0x0000_0200_u32.to_le_bytes()); // only SIGUSR1 remains
    continuation.jump_if(0x85, "signal-mask-buffer-fail");
    continuation.bytes(&[0xbf, 0x01, 0x00, 0x00, 0x00]); // SIG_UNBLOCK
    continuation.bytes(&[0x48, 0x89, 0xe6, 0x48, 0x8d, 0x54, 0x24, 0x08]);
    continuation.bytes(&[0x41, 0xba, 0x08, 0x00, 0x00, 0x00]);
    continuation.bytes(&[0xb8, 0x0e, 0x00, 0x00, 0x00, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "signal-mask-buffer-fail");
    continuation.bytes(&[0x48, 0x81, 0x7c, 0x24, 0x08]);
    continuation.bytes(&0x0000_0200_u32.to_le_bytes());
    continuation.jump_if(0x85, "signal-mask-buffer-fail");
    continuation.bytes(&[0x31, 0xff, 0x31, 0xf6]); // final query
    continuation.bytes(&[0x48, 0x8d, 0x54, 0x24, 0x08]);
    continuation.bytes(&[0x41, 0xba, 0x08, 0x00, 0x00, 0x00]);
    continuation.bytes(&[0xb8, 0x0e, 0x00, 0x00, 0x00, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "signal-mask-buffer-fail");
    continuation.bytes(&[0x48, 0x83, 0x7c, 0x24, 0x08, 0x00]);
    continuation.jump_if(0x85, "signal-mask-buffer-fail");
    continuation.bytes(&[0x48, 0x83, 0xc4, 0x10]);
    for generation in 0..2 {
        continuation.bytes(&[0x31, 0xff]);
        continuation.bytes(&[0xbe, 0x00, 0x10, 0x00, 0x00]);
        continuation.bytes(&[0xba, 0x03, 0x00, 0x00, 0x00]);
        continuation.bytes(&[0x41, 0xba, 0x22, 0x00, 0x00, 0x00]);
        continuation.bytes(&[0x49, 0xc7, 0xc0, 0xff, 0xff, 0xff, 0xff]);
        continuation.bytes(&[0x45, 0x31, 0xc9]);
        continuation.bytes(&[0xb8, 0x09, 0x00, 0x00, 0x00, 0x0f, 0x05]); // mmap
        continuation.bytes(&[0x48, 0x3d, 0x00, 0x60, 0x60, 0x00]);
        continuation.jump_if(0x85, "fail");
        continuation.bytes(&[0x49, 0x89, 0xc4]);
        continuation.bytes(&[0x41, 0x80, 0x3c, 0x24, 0x00]);
        continuation.jump_if(0x85, "fail");
        if generation == 0 {
            continuation.bytes(&[0x41, 0xc6, 0x04, 0x24, 0xa5]);
            continuation.bytes(&[0x4c, 0x89, 0xe7]);
            continuation.bytes(&[0xbe, 0x00, 0x10, 0x00, 0x00]);
            continuation.bytes(&[0xba, 0x01, 0x00, 0x00, 0x00]); // read-only
            continuation.bytes(&[0xb8, 0x0a, 0x00, 0x00, 0x00, 0x0f, 0x05]);
            continuation.bytes(&[0x48, 0x85, 0xc0]);
            continuation.jump_if(0x85, "fail");
            continuation.bytes(&[0x41, 0x80, 0x3c, 0x24, 0xa5]);
            continuation.jump_if(0x85, "fail");
            continuation.bytes(&[0x4c, 0x89, 0xe7]);
            continuation.bytes(&[0xbe, 0x00, 0x10, 0x00, 0x00, 0x31, 0xd2]); // none
            continuation.bytes(&[0xb8, 0x0a, 0x00, 0x00, 0x00, 0x0f, 0x05]);
            continuation.bytes(&[0x48, 0x85, 0xc0]);
            continuation.jump_if(0x85, "fail");
            continuation.bytes(&[0x4c, 0x89, 0xe7]);
            continuation.bytes(&[0xbe, 0x00, 0x10, 0x00, 0x00]);
            continuation.bytes(&[0xba, 0x04, 0x00, 0x00, 0x00]); // MADV_DONTNEED
            continuation.bytes(&[0xb8, 0x1c, 0x00, 0x00, 0x00, 0x0f, 0x05]);
            continuation.bytes(&[0x48, 0x85, 0xc0]);
            continuation.jump_if(0x85, "fail");
            continuation.bytes(&[0x31, 0xff]);
            continuation.bytes(&[0xbe, 0x00, 0x10, 0x00, 0x00]);
            continuation.bytes(&[0xba, 0x03, 0x00, 0x00, 0x00]);
            continuation.bytes(&[0x41, 0xba, 0x22, 0x00, 0x00, 0x00]);
            continuation.bytes(&[0x49, 0xc7, 0xc0, 0xff, 0xff, 0xff, 0xff]);
            continuation.bytes(&[0x45, 0x31, 0xc9]);
            continuation.bytes(&[0xb8, 0x09, 0x00, 0x00, 0x00, 0x0f, 0x05]);
            continuation.bytes(&[0x48, 0x83, 0xf8, 0xf4]); // -ENOMEM
            continuation.jump_if(0x85, "fail");
            continuation.bytes(&[0x4c, 0x89, 0xe7]);
            continuation.bytes(&[0xbe, 0x00, 0x10, 0x00, 0x00]);
            continuation.bytes(&[0xba, 0x03, 0x00, 0x00, 0x00]); // read-write
            continuation.bytes(&[0xb8, 0x0a, 0x00, 0x00, 0x00, 0x0f, 0x05]);
            continuation.bytes(&[0x48, 0x85, 0xc0]);
            continuation.jump_if(0x85, "fail");
            continuation.bytes(&[0x41, 0x80, 0x3c, 0x24, 0x00]);
            continuation.jump_if(0x85, "fail");
            continuation.bytes(&[0x41, 0xc6, 0x04, 0x24, 0xa5]);
            continuation.bytes(&[0x4c, 0x89, 0xe7]);
            continuation.bytes(&[0xbe, 0x00, 0x10, 0x00, 0x00, 0x31, 0xd2]); // unsupported advice
            continuation.bytes(&[0xb8, 0x1c, 0x00, 0x00, 0x00, 0x0f, 0x05]);
            continuation.bytes(&[0x48, 0x83, 0xf8, 0xea]); // -EINVAL
            continuation.jump_if(0x85, "fail");
            continuation.bytes(&[0x41, 0x80, 0x3c, 0x24, 0xa5]);
            continuation.jump_if(0x85, "fail");
            continuation.bytes(&[0x4c, 0x89, 0xe7]);
            continuation.bytes(&[0xbe, 0x00, 0x10, 0x00, 0x00]);
            continuation.bytes(&[0xba, 0x04, 0x00, 0x00, 0x00]); // executable rejected
            continuation.bytes(&[0xb8, 0x0a, 0x00, 0x00, 0x00, 0x0f, 0x05]);
            continuation.bytes(&[0x48, 0x83, 0xf8, 0xea]); // -EINVAL
            continuation.jump_if(0x85, "fail");
            continuation.bytes(&[0x41, 0x80, 0x3c, 0x24, 0xa5]);
            continuation.jump_if(0x85, "fail");
        }
        continuation.bytes(&[0x4c, 0x89, 0xe7]);
        continuation.bytes(&[0xbe, 0x00, 0x10, 0x00, 0x00]);
        continuation.bytes(&[0xb8, 0x0b, 0x00, 0x00, 0x00, 0x0f, 0x05]); // munmap
        continuation.bytes(&[0x48, 0x85, 0xc0]);
        continuation.jump_if(0x85, "fail");
    }
    continuation.bytes(&[0xc7, 0x04, 0x25, 0x08, 0x50, 0x60, 0x00]);
    continuation.bytes(&1_u32.to_le_bytes()); // clear-child-TID word starts nonzero
    continuation.bytes(&[0xbf, 0x08, 0x50, 0x60, 0x00]);
    continuation.bytes(&[0xb8, 0xda, 0x00, 0x00, 0x00, 0x0f, 0x05]); // set_tid_address
    continuation.bytes(&[0x48, 0x83, 0xf8, 0x01]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x48, 0x83, 0xec, 0x20]); // native robust_list_head
    continuation.bytes(&[0x48, 0x89, 0x24, 0x24, 0x31, 0xc0]); // self link; zero
    continuation.bytes(&[0x48, 0x89, 0x44, 0x24, 0x08]); // futex offset
    continuation.bytes(&[0x48, 0x89, 0x44, 0x24, 0x10]); // no pending entry
    continuation.bytes(&[0x48, 0x89, 0xe7]);
    continuation.bytes(&[0xbe, 0x18, 0x00, 0x00, 0x00]);
    continuation.bytes(&[0xb8, 0x11, 0x01, 0x00, 0x00, 0x0f, 0x05]); // set_robust_list
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x48, 0x83, 0xec, 0x10]); // ARCH_GET_FS output
    continuation.bytes(&[0xbf, 0x02, 0x10, 0x00, 0x00]); // ARCH_SET_FS
    continuation.bytes(&[0xbe, 0x00, 0x50, 0x60, 0x00]); // readable data base
    continuation.bytes(&[0xb8, 0x9e, 0x00, 0x00, 0x00, 0x0f, 0x05]); // arch_prctl
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fs-buffer-fail");
    continuation.bytes(&[0xbf, 0x03, 0x10, 0x00, 0x00]); // ARCH_GET_FS
    continuation.bytes(&[0x48, 0x89, 0xe6]);
    continuation.bytes(&[0xb8, 0x9e, 0x00, 0x00, 0x00, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fs-buffer-fail");
    continuation.bytes(&[0x48, 0x81, 0x3c, 0x24, 0x00, 0x50, 0x60, 0x00]);
    continuation.jump_if(0x85, "fs-buffer-fail");
    continuation.bytes(&[0xbf, 0x02, 0x10, 0x00, 0x00, 0x31, 0xf6]); // clear FS
    continuation.bytes(&[0xb8, 0x9e, 0x00, 0x00, 0x00, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fs-buffer-fail");
    continuation.bytes(&[0xbf, 0x03, 0x10, 0x00, 0x00]);
    continuation.bytes(&[0x48, 0x89, 0xe6]);
    continuation.bytes(&[0xb8, 0x9e, 0x00, 0x00, 0x00, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fs-buffer-fail");
    continuation.bytes(&[0x48, 0x83, 0x3c, 0x24, 0x00]);
    continuation.jump_if(0x85, "fs-buffer-fail");
    continuation.bytes(&[0x48, 0x83, 0xc4, 0x10]);
    continuation.bytes(&[0x48, 0x81, 0xec, 0x00, 0x01, 0x00, 0x00]); // bounded getrandom result
    continuation.bytes(&[0x31, 0xff, 0x31, 0xf6, 0x31, 0xd2]);
    continuation.bytes(&[0xb8, 0x3e, 0x01, 0x00, 0x00, 0x0f, 0x05]); // zero length
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "getrandom-buffer-fail");
    continuation.bytes(&[0x48, 0x89, 0xe7]);
    continuation.bytes(&[0xbe, 0x10, 0x00, 0x00, 0x00, 0xba, 0x08, 0, 0, 0]);
    continuation.bytes(&[0xb8, 0x3e, 0x01, 0x00, 0x00, 0x0f, 0x05]); // unknown flag
    continuation.bytes(&[0x48, 0x83, 0xf8, 0xea]);
    continuation.jump_if(0x85, "getrandom-buffer-fail");
    continuation.bytes(&[0x48, 0x89, 0xe7]);
    continuation.bytes(&[0xbe, 0x10, 0x00, 0x00, 0x00, 0xba, 0x06, 0, 0, 0]);
    continuation.bytes(&[0xb8, 0x3e, 0x01, 0x00, 0x00, 0x0f, 0x05]); // incompatible flags
    continuation.bytes(&[0x48, 0x83, 0xf8, 0xea]);
    continuation.jump_if(0x85, "getrandom-buffer-fail");
    continuation.bytes(&[0xbf, 0xf8, 0x1f, 0x60, 0x00]); // crosses stack guard
    continuation.bytes(&[0xbe, 0x10, 0x00, 0x00, 0x00, 0x31, 0xd2]);
    continuation.bytes(&[0xb8, 0x3e, 0x01, 0x00, 0x00, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x83, 0xf8, 0xf2]);
    continuation.jump_if(0x85, "getrandom-buffer-fail");
    continuation.bytes(&[0x48, 0x89, 0xe7]);
    continuation.bytes(&[0xbe, 0x10, 0x00, 0x00, 0x00, 0x31, 0xd2]);
    continuation.bytes(&[0xb8, 0x3e, 0x01, 0x00, 0x00, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x83, 0xf8, 0x10]);
    continuation.jump_if(0x85, "getrandom-buffer-fail");
    continuation.bytes(&[0x4c, 0x8b, 0x14, 0x24, 0x4c, 0x8b, 0x5c, 0x24, 0x08]);
    continuation.bytes(&[0x48, 0x89, 0xe7]);
    continuation.bytes(&[0xbe, 0x10, 0x00, 0x00, 0x00, 0x31, 0xd2]);
    continuation.bytes(&[0xb8, 0x3e, 0x01, 0x00, 0x00, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x83, 0xf8, 0x10]);
    continuation.jump_if(0x85, "getrandom-buffer-fail");
    continuation.bytes(&[0x4c, 0x8b, 0x14, 0x24, 0x4c, 0x8b, 0x5c, 0x24, 0x08]);
    continuation.bytes(&[0x48, 0x89, 0xe7]);
    continuation.bytes(&[0xbe, 0x20, 0x00, 0x00, 0x00, 0x31, 0xd2]);
    continuation.bytes(&[0xb8, 0x3e, 0x01, 0x00, 0x00, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x83, 0xf8, 0x20]);
    continuation.jump_if(0x85, "getrandom-buffer-fail");
    continuation.bytes(&[0x4c, 0x8b, 0x14, 0x24, 0x4c, 0x8b, 0x5c, 0x24, 0x08]);
    continuation.bytes(&[0x4c, 0x8b, 0x64, 0x24, 0x10, 0x4c, 0x8b, 0x6c, 0x24, 0x18]);
    continuation.bytes(&[0x48, 0x89, 0xe7]);
    continuation.bytes(&[0xbe, 0x00, 0x01, 0x00, 0x00, 0x31, 0xd2]);
    continuation.bytes(&[0xb8, 0x3e, 0x01, 0x00, 0x00, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x3d, 0x00, 0x01, 0x00, 0x00]);
    continuation.jump_if(0x85, "getrandom-buffer-fail");
    continuation.bytes(&[0x4c, 0x8b, 0x14, 0x24]);
    continuation.bytes(&[0x4c, 0x8b, 0x9c, 0x24, 0xf8, 0x00, 0x00, 0x00]);
    continuation.bytes(&[0xbf, 0x00, 0x10, 0x60, 0x00]); // complete writable data page
    continuation.bytes(&[0xbe, 0x00, 0x02, 0x00, 0x00, 0x31, 0xd2]);
    continuation.bytes(&[0xb8, 0x3e, 0x01, 0x00, 0x00, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x3d, 0x00, 0x02, 0x00, 0x00]);
    continuation.jump_if(0x85, "getrandom-buffer-fail");
    continuation.bytes(&[0x4c, 0x8b, 0x14, 0x25, 0x00, 0x10, 0x60, 0x00]);
    continuation.bytes(&[0x4c, 0x8b, 0x1c, 0x25, 0xf8, 0x11, 0x60, 0x00]);
    continuation.bytes(&[0x48, 0x81, 0xc4, 0x00, 0x01, 0x00, 0x00]);
    continuation.bytes(&[0xbf, 0x01, 0x00, 0x00, 0x00]); // stdout
    continuation.bytes(&[0xbe, 0x00, 0x50, 0x60, 0x00]); // writable stat destination
    continuation.bytes(&[0xb8, 0x05, 0x00, 0x00, 0x00, 0x0f, 0x05]); // fstat(stdout)
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x48, 0x81, 0xec, 0x90, 0x00, 0x00, 0x00]);
    continuation.bytes(&[0x31, 0xc0, 0xb9, 0x12, 0x00, 0x00, 0x00]);
    continuation.bytes(&[0x48, 0x89, 0xe7, 0xf3, 0x48, 0xab]);
    for (offset, value) in [
        (8_u8, 1_u32),
        (16, 1),
        (24, 0x2180),
        (40, 0x4400),
        (56, 4096),
    ] {
        continuation.bytes(&[0xc7, 0x44, 0x24, offset]);
        continuation.bytes(&value.to_le_bytes());
    }
    continuation.bytes(&[0xbe, 0x00, 0x50, 0x60, 0x00]);
    continuation.bytes(&[0x48, 0x89, 0xe7, 0xb9, 0x12, 0x00, 0x00, 0x00]);
    continuation.bytes(&[0xf3, 0x48, 0xa7]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x48, 0x81, 0xc4, 0x90, 0x00, 0x00, 0x00]);
    continuation.bytes(&[0x31, 0xff]);
    continuation.bytes(&[0xbe, 0x01, 0x54, 0x00, 0x00]);
    continuation.bytes(&[0xba, 0x20, 0x51, 0x60, 0x00]);
    continuation.bytes(&[0x6a, 0x10, 0x58, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xbe, 0x20, 0x51, 0x60, 0x00]);
    continuation.rip_lea(&[0x48, 0x8d, 0x3d], "termios-expected");
    continuation.bytes(&[0xb9, 0x24, 0x00, 0x00, 0x00, 0xf3, 0xa6]);
    continuation.jump_if(0x85, "fail");
    continuation.jump("termios-expected-complete");
    continuation.bind("termios-expected")?;
    let mut expected_termios = [0_u8; 36];
    expected_termios[8..12].copy_from_slice(&0x8bf_u32.to_le_bytes());
    expected_termios[23] = 1;
    continuation.bytes(&expected_termios);
    continuation.bind("termios-expected-complete")?;
    continuation.bytes(&[0xbf, 0x01, 0x00, 0x00, 0x00]);
    continuation.bytes(&[0xbe, 0x01, 0x54, 0x00, 0x00]);
    continuation.bytes(&[0xba, 0x50, 0x51, 0x60, 0x00]);
    continuation.bytes(&[0x6a, 0x10, 0x58, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xbe, 0x20, 0x51, 0x60, 0x00]);
    continuation.bytes(&[0xbf, 0x50, 0x51, 0x60, 0x00]);
    continuation.bytes(&[0xb9, 0x24, 0x00, 0x00, 0x00, 0xf3, 0xa6]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x31, 0xdb]);
    continuation.bytes(&[0xbe, 0x13, 0x54, 0x00, 0x00]);
    continuation.bytes(&[0xba, 0x80, 0x51, 0x60, 0x00]);
    continuation.bind("winsize-query-loop")?;
    continuation.bytes(&[0x89, 0xdf, 0x6a, 0x10, 0x58, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x48, 0x83, 0x3a, 0x00]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x83, 0xc2, 0x08, 0xff, 0xc3, 0x83, 0xfb, 0x02]);
    continuation.jump_if(0x85, "winsize-query-loop");
    continuation.bytes(&[0x31, 0xdb, 0x6a, 0x01, 0x5a]);
    continuation.bind("lseek-query-loop")?;
    continuation.bytes(&[0x89, 0xdf, 0x6a, 0x08, 0x58, 0x0f, 0x05]);
    continuation.bytes(&[0x83, 0xf8, 0xe3]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xff, 0xc3, 0x83, 0xfb, 0x02]);
    continuation.jump_if(0x85, "lseek-query-loop");
    continuation.bytes(&[0xb2, 0x05, 0x6a, 0x08, 0x58, 0x0f, 0x05]);
    continuation.bytes(&[0x83, 0xf8, 0xea]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xbf, 0x01, 0x00, 0x00, 0x00]);
    continuation.bytes(&[0xbe, 0xff, 0xff, 0x00, 0x00]);
    continuation.bytes(&[0x48, 0xc7, 0xc2, 0xff, 0xff, 0xff, 0xff]);
    continuation.bytes(&[0x6a, 0x10, 0x58, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x83, 0xf8, 0xe7]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xbf, 0x01, 0, 0, 0, 0xbe, 0x03, 0, 0, 0, 0x31, 0xd2]);
    continuation.bytes(&[0xb8, 0x48, 0, 0, 0, 0x0f, 0x05]); // F_GETFL stdout
    continuation.bytes(&[0x48, 0x83, 0xf8, 0x01]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xbf, 0x01, 0, 0, 0, 0xbe, 0x01, 0, 0, 0, 0x31, 0xd2]);
    continuation.bytes(&[0xb8, 0x48, 0, 0, 0, 0x0f, 0x05]); // F_GETFD clear
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xbf, 0x01, 0, 0, 0, 0xbe, 0x02, 0, 0, 0]);
    continuation.bytes(&[0xba, 0x01, 0, 0, 0, 0xb8, 0x48, 0, 0, 0, 0x0f, 0x05]); // F_SETFD
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xbf, 0x01, 0, 0, 0, 0xbe, 0x01, 0, 0, 0, 0x31, 0xd2]);
    continuation.bytes(&[0xb8, 0x48, 0, 0, 0, 0x0f, 0x05]); // F_GETFD set
    continuation.bytes(&[0x48, 0x83, 0xf8, 0x01]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xbf, 0x01, 0, 0, 0, 0xbe, 0x63, 0, 0, 0, 0x31, 0xd2]);
    continuation.bytes(&[0xb8, 0x48, 0, 0, 0, 0x0f, 0x05]); // unsupported command
    continuation.bytes(&[0x48, 0x83, 0xf8, 0xea]); // -EINVAL
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xbf, 0x01, 0x00, 0x00, 0x00]);
    continuation.bytes(&[0xb8, 0x20, 0x00, 0x00, 0x00, 0x0f, 0x05]); // dup(full table)
    continuation.bytes(&[0x48, 0x83, 0xf8, 0xe8]); // -EMFILE
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x31, 0xff]);
    continuation.bytes(&[0xb8, 0x03, 0x00, 0x00, 0x00, 0x0f, 0x05]); // close(stdin)
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x31, 0xff]);
    continuation.bytes(&[0xbe, 0x00, 0x10, 0x60, 0x00]);
    continuation.bytes(&[0xba, 0x01, 0x00, 0x00, 0x00]);
    continuation.bytes(&[0x31, 0xc0, 0x0f, 0x05]); // read(closed stdin)
    continuation.bytes(&[0x48, 0x83, 0xf8, 0xf7]); // -EBADF
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x31, 0xff]);
    continuation.bytes(&[0xb8, 0x03, 0x00, 0x00, 0x00, 0x0f, 0x05]); // repeated close
    continuation.bytes(&[0x48, 0x83, 0xf8, 0xf7]); // -EBADF
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xbf, 0x01, 0x00, 0x00, 0x00]);
    continuation.bytes(&[0xb8, 0x20, 0x00, 0x00, 0x00, 0x0f, 0x05]); // dup(stdout)
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x31, 0xff]); // duplicated stdout alias in slot zero
    continuation.bytes(&[0xbe, 0x90, 0x50, 0x60, 0x00]);
    continuation.bytes(&[0xb8, 0x05, 0x00, 0x00, 0x00, 0x0f, 0x05]); // fstat(alias)
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xbe, 0x00, 0x50, 0x60, 0x00]);
    continuation.bytes(&[0xbf, 0x90, 0x50, 0x60, 0x00]);
    continuation.bytes(&[0xb9, 0x12, 0x00, 0x00, 0x00, 0xf3, 0x48, 0xa7]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x31, 0xff, 0xbe, 0x01, 0, 0, 0, 0x31, 0xd2]);
    continuation.bytes(&[0xb8, 0x48, 0, 0, 0, 0x0f, 0x05]); // duplicate flag clear
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xbf, 0x01, 0, 0, 0, 0xbe, 0x01, 0, 0, 0, 0x31, 0xd2]);
    continuation.bytes(&[0xb8, 0x48, 0, 0, 0, 0x0f, 0x05]); // source flag retained
    continuation.bytes(&[0x48, 0x83, 0xf8, 0x01]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xbf, 0x01, 0x00, 0x00, 0x00]);
    continuation.bytes(&[0xb8, 0x03, 0x00, 0x00, 0x00, 0x0f, 0x05]); // close source slot
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x31, 0xff, 0xbe, 0x01, 0x00, 0x00, 0x00]);
    continuation.bytes(&[0xb8, 0x21, 0x00, 0x00, 0x00, 0x0f, 0x05]); // dup2(0,1)
    continuation.bytes(&[0x48, 0x83, 0xf8, 0x01]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xbf, 0x01, 0, 0, 0, 0xbe, 0x01, 0, 0, 0, 0x31, 0xd2]);
    continuation.bytes(&[0xb8, 0x48, 0, 0, 0, 0x0f, 0x05]); // dup2 flag clear
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x31, 0xff]);
    continuation.bytes(&[0xbe, 0x00, 0x10, 0x60, 0x00]);
    continuation.bytes(&[0xba, 0x01, 0x00, 0x00, 0x00]);
    continuation.bytes(&[0x31, 0xc0, 0x0f, 0x05]); // read(write-only alias)
    continuation.bytes(&[0x48, 0x83, 0xf8, 0xf7]); // -EBADF
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x31, 0xff]);
    continuation.bytes(&[0xb8, 0x03, 0x00, 0x00, 0x00, 0x0f, 0x05]); // close duplicate
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x48, 0xc7, 0xc7, 0xff, 0xff, 0xff, 0xff, 0x31, 0xf6]);
    continuation.bytes(&[0xb8, 0x21, 0x00, 0x00, 0x00, 0x0f, 0x05]); // invalid dup2
    continuation.bytes(&[0x48, 0x83, 0xf8, 0xf7]); // -EBADF
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0xbf, 0x02, 0x00, 0x00, 0x00, 0x31, 0xf6]);
    continuation.bytes(&[0xb8, 0x21, 0x00, 0x00, 0x00, 0x0f, 0x05]); // dup2(2,0)
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x31, 0xff]);
    continuation.bytes(&[0xb8, 0x03, 0x00, 0x00, 0x00, 0x0f, 0x05]); // close error alias
    continuation.bytes(&[0x48, 0x85, 0xc0]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x31, 0xff]);
    continuation.bytes(&[0xbe, 0x00, 0x50, 0x60, 0x00]);
    continuation.bytes(&[0xb8, 0x05, 0x00, 0x00, 0x00, 0x0f, 0x05]); // fstat(closed slot)
    continuation.bytes(&[0x48, 0x83, 0xf8, 0xf7]); // -EBADF before pointer access
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x31, 0xff]);
    continuation.bytes(&[0x48, 0xc7, 0xc2, 0xff, 0xff, 0xff, 0xff]);
    continuation.bytes(&[0xbe, 0x01, 0x54, 0x00, 0x00]);
    continuation.bytes(&[0x6a, 0x10, 0x58, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x83, 0xf8, 0xf7]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x83, 0xc6, 0x12]);
    continuation.bytes(&[0x6a, 0x10, 0x58, 0x0f, 0x05]);
    continuation.bytes(&[0x48, 0x83, 0xf8, 0xf7]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x6a, 0x05, 0x5a, 0x6a, 0x08, 0x58, 0x0f, 0x05]);
    continuation.bytes(&[0x83, 0xf8, 0xf7]);
    continuation.jump_if(0x85, "fail");
    continuation.bytes(&[0x31, 0xff, 0xbe, 0x01, 0, 0, 0, 0x31, 0xd2]);
    continuation.bytes(&[0xb8, 0x48, 0, 0, 0, 0x0f, 0x05]); // closed-slot F_GETFD
    continuation.bytes(&[0x48, 0x83, 0xf8, 0xf7]); // -EBADF
    continuation.jump_if(0x85, "fail");
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
    continuation.jump("fail");
    continuation.bind("rlimit-buffer-fail")?;
    continuation.bytes(&[0x48, 0x83, 0xc4, 0x10]);
    continuation.jump("fail");
    continuation.bind("signal-mask-buffer-fail")?;
    continuation.bytes(&[0x48, 0x83, 0xc4, 0x10]);
    continuation.jump("fail");
    continuation.bind("fs-buffer-fail")?;
    continuation.bytes(&[0x48, 0x83, 0xc4, 0x10]);
    continuation.jump("fail");
    continuation.bind("getrandom-buffer-fail")?;
    continuation.bytes(&[0x48, 0x81, 0xc4, 0x00, 0x01, 0x00, 0x00]);
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
    code.bytes(&[0x48, 0x83, 0xc1, 0x10]);
    code.bind("pte-selected")?;
    code.bytes(&[0x4c, 0x8b, 0x41, 0x18]);
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x4d, 0x89, 0xc1, 0x41, 0x83, 0xe1, 0x07]);
    code.bytes(&[0x41, 0x80, 0xf9, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x3c, 0x0c]);
    code.jump_if(0x85, "leaf-validation-complete");
    code.bytes(&[0x4c, 0x8b, 0x51, 0x20]);
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x4d, 0x89, 0xd3, 0x41, 0x83, 0xe3, 0x07]);
    code.bytes(&[0x41, 0x80, 0xfb, 0x07]);
    code.jump_if(0x85, "fail");
    code.bind("leaf-validation-complete")?;
    code.bytes(&[0x49, 0x83, 0xe0, 0xfd]);
    code.bytes(&[0x4c, 0x89, 0x41, 0x18]);
    code.bytes(&[0x3c, 0x0c]);
    code.jump_if(0x85, "leaf-protection-complete");
    code.bytes(&[0x49, 0x83, 0xe2, 0xfd]);
    code.bytes(&[0x4c, 0x89, 0x51, 0x20]);
    code.bind("leaf-protection-complete")?;
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x3c, 0x0c]);
    code.jump_if(0x84, "invalidate-generation-one");
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x0060_5000_u64.to_le_bytes());
    code.jump("invalidate-mapping");
    code.bind("invalidate-generation-one")?;
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x0060_3000_u64.to_le_bytes());
    code.bind("invalidate-mapping")?;
    code.bytes(&[0x41, 0x0f, 0x01, 0x3a]);
    code.bytes(&[0x3c, 0x0c]);
    code.jump_if(0x85, "invalidation-complete");
    code.bytes(&[0x49, 0x81, 0xc2, 0x00, 0x10, 0x00, 0x00]);
    code.bytes(&[0x41, 0x0f, 0x01, 0x3a]);
    code.bind("invalidation-complete")?;
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
    code.bytes(&[0x48, 0x81, 0xc1, 0x18, 0x20, 0x00, 0x00]);
    code.bytes(&[0x4c, 0x8b, 0x01]);
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x41, 0xf6, 0xc0, 0x02]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x0f, 0xba, 0xe0, 0x3f]);
    code.jump_if(0x83, "fail");
    code.bytes(&[0x4c, 0x8b, 0x51, 0x08]);
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x41, 0xf6, 0xc2, 0x02]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x0f, 0xba, 0xe2, 0x3f]);
    code.jump_if(0x83, "fail");
    code.bytes(&[0x49, 0x0f, 0xba, 0xf0, 0x3f]);
    code.bytes(&[0x49, 0x0f, 0xba, 0xf2, 0x3f]);
    code.bytes(&[0x4c, 0x89, 0x01]);
    code.bytes(&[0x4c, 0x89, 0x51, 0x08]);
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x0060_3000_u64.to_le_bytes());
    code.bytes(&[0x41, 0x0f, 0x01, 0x3a]);
    code.bytes(&[0x49, 0x81, 0xc2, 0x00, 0x10, 0x00, 0x00]);
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

fn user_reserve_initial_anonymous_mapping() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x16]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x83, 0xbf, 0xc0, 0x00, 0x00, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x83, 0xbf, 0xc8, 0x00, 0x00, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x8b, 0x4f, 0x68]);
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xc1, 0x30, 0x20, 0x00, 0x00]);
    code.bytes(&[0x48, 0x83, 0x39, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x89, 0xb7, 0xc0, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0xc7, 0x87, 0xc8, 0x00, 0x00, 0x00, 1, 0, 0, 0]);
    code.bytes(&[0xc6, 0x87, 0x70, 0x00, 0x00, 0x00, 0x17]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn user_release_initial_anonymous_mapping() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x3f, 0x08]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x17]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x8b, 0x87, 0xc0, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x85, 0xc0]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x83, 0xbf, 0xc8, 0x00, 0x00, 0x00, 0x01]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x8b, 0x4f, 0x68]);
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xc1, 0x30, 0x20, 0x00, 0x00]);
    code.bytes(&[0x48, 0x83, 0x39, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0xc7, 0x87, 0xc0, 0x00, 0x00, 0x00, 0, 0, 0, 0]);
    code.bytes(&[0x48, 0xc7, 0x87, 0xc8, 0x00, 0x00, 0x00, 0, 0, 0, 0]);
    code.bytes(&[0xc6, 0x87, 0x70, 0x00, 0x00, 0x00, 0x16, 0xc3]);
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
    code.bytes(&[0x48, 0x83, 0xbf, 0xa8, 0x00, 0x00, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x83, 0xbf, 0xb0, 0x00, 0x00, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x83, 0xbf, 0xb8, 0x00, 0x00, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0xc7, 0x87, 0x90, 0x00, 0x00, 0x00, 0, 0, 0, 0]);
    code.bytes(&[0x48, 0xc7, 0x87, 0x98, 0x00, 0x00, 0x00, 0, 0, 0, 0]);
    code.bytes(&[0x48, 0xc7, 0x87, 0xa0, 0x00, 0x00, 0x00, 0, 0, 0, 0]);
    code.bytes(&[0x48, 0xc7, 0x87, 0xa8, 0x00, 0x00, 0x00, 0, 0, 0, 0]);
    code.bytes(&[0x48, 0xc7, 0x87, 0xb0, 0x00, 0x00, 0x00, 0, 0, 0, 0]);
    code.bytes(&[0x48, 0xc7, 0x87, 0xb8, 0x00, 0x00, 0x00, 0, 0, 0, 0]);
    emit_wrmsr_immediate(&mut code, 0xc000_0100, 0); // clear IA32_FS_BASE
    code.bytes(&[0xb8, 0x0e, 0x00, 0x00, 0x00]);
    code.bytes(&[0x4c, 0x8b, 0x87, 0x80, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x8b, 0x4f, 0x68]);
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xc1, 0x28, 0x20, 0x00, 0x00]);
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x0060_5000_u64.to_le_bytes());
    code.jump("mapping-selected");
    code.bind("unmap-code")?;
    code.bytes(&[0xb8, 0x0b, 0x00, 0x00, 0x00]);
    code.bytes(&[0x4c, 0x8b, 0x47, 0x78]);
    code.bytes(&[0x48, 0x8b, 0x4f, 0x68]);
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xc1, 0x18, 0x20, 0x00, 0x00]);
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x0060_3000_u64.to_le_bytes());
    code.bind("mapping-selected")?;
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x83, 0x39, 0x00]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0xc7, 0x01, 0, 0, 0, 0]);
    code.bytes(&[0x3c, 0x0b]);
    code.jump_if(0x85, "leaves-revoked");
    code.bytes(&[0x48, 0x83, 0x79, 0x08, 0x00]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0xc7, 0x41, 0x08, 0, 0, 0, 0]);
    code.bind("leaves-revoked")?;
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x41, 0x0f, 0x01, 0x3a]);
    code.bytes(&[0x3c, 0x0b]);
    code.jump_if(0x85, "translations-revoked");
    code.bytes(&[0x49, 0x81, 0xc2, 0x00, 0x10, 0x00, 0x00]);
    code.bytes(&[0x41, 0x0f, 0x01, 0x3a]);
    code.bind("translations-revoked")?;
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
    code.bytes(&[0x80, 0xbf, 0x70, 0x00, 0x00, 0x00, 0x17]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0xb9]);
    code.bytes(&0xc000_0080_u32.to_le_bytes()); // IA32_EFER
    code.bytes(&[0x0f, 0x32, 0x83, 0xc8, 0x01, 0x0f, 0x30]); // enable SCE
    emit_wrmsr_immediate(&mut code, 0xc000_0081, 0x001b_0010_0000_0000); // STAR
    emit_wrmsr_register(&mut code, 0xc000_0082, &[0x48, 0x89, 0xf0]); // LSTAR = RSI
    emit_wrmsr_immediate(&mut code, 0xc000_0084, 0x200); // mask IF on entry
    emit_wrmsr_register(&mut code, 0xc000_0102, &[0x48, 0x89, 0xf8]); // kernel GS base
    code.bytes(&[0x48, 0xc7, 0x87, 0xa0, 0x00, 0x00, 0x00, 0, 0, 0, 0]);
    code.bytes(&[0x48, 0xc7, 0x87, 0xa8, 0x00, 0x00, 0x00, 0, 0, 0, 0]);
    code.bytes(&[0x48, 0xc7, 0x87, 0xb0, 0x00, 0x00, 0x00, 0, 0, 0, 0]);
    code.bytes(&[0x48, 0xc7, 0x87, 0xb8, 0x00, 0x00, 0x00, 0, 0, 0, 0]);
    emit_wrmsr_immediate(&mut code, 0xc000_0100, 0); // initial user FS base
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
    code.bytes(&[0x80, 0xbf, 0xd0, 0, 0, 0, 1]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x83, 0xbf, 0xd8, 0, 0, 0, 0]);
    code.jump_if(0x84, "fail");
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
    code.bytes(&[0x83, 0xf8, 0x03]);
    code.jump_if(0x84, "close");
    code.bytes(&[0x83, 0xf8, 0x05]);
    code.jump_if(0x84, "fstat");
    code.bytes(&[0x83, 0xf8, 0x08]);
    code.jump_if(0x84, "lseek");
    code.bytes(&[0x83, 0xf8, 0x10]);
    code.jump_if(0x84, "ioctl");
    code.bytes(&[0x83, 0xf8, 0x20]);
    code.jump_if(0x84, "dup");
    code.bytes(&[0x83, 0xf8, 0x21]);
    code.jump_if(0x84, "dup2");
    code.bytes(&[0x83, 0xf8, 0x48]);
    code.jump_if(0x84, "fcntl");
    code.bytes(&[0x83, 0xf8, 0x09]);
    code.jump_if(0x84, "mmap");
    code.bytes(&[0x83, 0xf8, 0x0a]);
    code.jump_if(0x84, "mprotect");
    code.bytes(&[0x83, 0xf8, 0x0b]);
    code.jump_if(0x84, "munmap");
    code.bytes(&[0x83, 0xf8, 0x0c]);
    code.jump_if(0x84, "brk");
    code.bytes(&[0x83, 0xf8, 0x0e]);
    code.jump_if(0x84, "rt-sigprocmask");
    code.bytes(&[0x83, 0xf8, 0x1c]);
    code.jump_if(0x84, "madvise");
    code.bytes(&[0x83, 0xf8, 0x18]);
    code.jump_if(0x84, "sched-yield");
    code.bytes(&[0x83, 0xf8, 0x27]);
    code.jump_if(0x84, "getpid");
    for number in [102_u8, 104, 107, 108, 110] {
        code.bytes(&[0x83, 0xf8, number]);
        code.jump_if(0x84, "identity-zero");
    }
    code.bytes(&[0x3d, 0xba, 0x00, 0x00, 0x00]);
    code.jump_if(0x84, "identity-one");
    code.bytes(&[0x3d, 0xe4, 0x00, 0x00, 0x00]);
    code.jump_if(0x84, "clock-gettime");
    code.bytes(&[0x3d, 0xe5, 0x00, 0x00, 0x00]);
    code.jump_if(0x84, "clock-getres");
    code.bytes(&[0x3d, 0x9e, 0x00, 0x00, 0x00]);
    code.jump_if(0x84, "arch-prctl");
    code.bytes(&[0x3d, 0xda, 0x00, 0x00, 0x00]);
    code.jump_if(0x84, "set-tid-address");
    code.bytes(&[0x3d, 0x11, 0x01, 0x00, 0x00]);
    code.jump_if(0x84, "set-robust-list");
    code.bytes(&[0x83, 0xf8, 0x61]);
    code.jump_if(0x84, "getrlimit");
    code.bytes(&[0x3d, 0x2e, 0x01, 0x00, 0x00]);
    code.jump_if(0x84, "prlimit64");
    code.bytes(&[0x3d, 0x3e, 0x01, 0x00, 0x00]);
    code.jump_if(0x84, "getrandom");
    code.bytes(&[0x83, 0xf8, 0x3f]);
    code.jump_if(0x84, "uname");
    code.bytes(&[0x83, 0xf8, 0x23]);
    code.jump_if(0x84, "nanosleep");
    code.bytes(&[0x3d, 0xe7, 0x00, 0x00, 0x00]);
    code.jump_if(0x84, "exit-group");
    code.bytes(&[0x48, 0xc7, 0xc0, 0xda, 0xff, 0xff, 0xff]);
    code.jump("return-user");

    code.bind("sched-yield")?;
    code.bytes(&[0x31, 0xc0]);
    code.jump("return-user");

    code.bind("getrandom")?;
    code.bytes(&[0x65, 0x4c, 0x8b, 0x04, 0x25, 0x58, 0, 0, 0]); // flags
    code.bytes(&[0x4d, 0x89, 0xc1, 0x49, 0x83, 0xe1, 0xf8]); // unknown bits
    code.jump_if(0x85, "getrandom-einval");
    code.bytes(&[0x49, 0x83, 0xe0, 0x06, 0x49, 0x83, 0xf8, 0x06]);
    code.jump_if(0x84, "getrandom-einval");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x02, 0x00, 0x00]);
    code.jump_if(0x87, "getrandom-einval");
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x84, "getrandom-zero");
    code.bytes(&[0x65, 0x48, 0x8b, 0x0c, 0x25, 0xd8, 0, 0, 0]); // private service
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "getrandom-einval");
    code.bytes(&[0x48, 0x81, 0xff, 0x00, 0x10, 0x60, 0x00]);
    code.jump_if(0x82, "getrandom-efault");
    code.bytes(&[0x49, 0x89, 0xfa, 0x49, 0x01, 0xf2]); // checked end
    code.jump_if(0x82, "getrandom-efault");
    code.bytes(&[0x49, 0x81, 0xfa, 0x00, 0x20, 0x60, 0x00]);
    code.jump_if(0x87, "getrandom-efault");
    code.bytes(&[0x56]); // retain exact syscall length until every output succeeds
    code.bind("getrandom-output-loop")?;
    code.bytes(&[0x80, 0x79, 0x10, 0x01]);
    code.jump_if(0x84, "getrandom-output-ready");
    code.bytes(&[0x80, 0x79, 0x10, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x51, 0x57, 0x56]); // retain service, destination, and remaining length
    code.bytes(&[0x48, 0x8b, 0x39, 0x48, 0x8b, 0x71, 0x08]);
    code.call("getrandom-extract-chacha20");
    code.bytes(&[0x5e, 0x5f, 0x59, 0x85, 0xc0]);
    code.jump_if(0x84, "fail");
    code.bind("getrandom-output-ready")?;
    code.bytes(&[0x4c, 0x8b, 0x49, 0x08]);
    code.bytes(&[0x4d, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x89, 0xf2, 0x48, 0x83, 0xfa, 0x20]); // chunk=min(remaining,32)
    code.jump_if(0x86, "getrandom-chunk-ready");
    code.bytes(&[0xba, 0x20, 0x00, 0x00, 0x00]);
    code.bind("getrandom-chunk-ready")?;
    code.bytes(&[0x45, 0x31, 0xc0]);
    code.bind("getrandom-copy-loop")?;
    code.bytes(&[0x49, 0x39, 0xd0]);
    code.jump_if(0x84, "getrandom-copy-complete");
    code.bytes(&[0x43, 0x8a, 0x04, 0x01, 0x42, 0x88, 0x04, 0x07]);
    code.bytes(&[0x42, 0x3a, 0x04, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0xff, 0xc0]);
    code.jump("getrandom-copy-loop");
    code.bind("getrandom-copy-complete")?;
    code.bytes(&[0x31, 0xc0]);
    for offset in [0_u8, 8, 16, 24] {
        if offset == 0 {
            code.bytes(&[0x49, 0x89, 0x01]);
        } else {
            code.bytes(&[0x49, 0x89, 0x41, offset]);
        }
    }
    code.bytes(&[0x4d, 0x8b, 0x01]);
    for offset in [8_u8, 16, 24] {
        code.bytes(&[0x4d, 0x0b, 0x41, offset]);
    }
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0xc6, 0x41, 0x10, 0x00]);
    code.bytes(&[0x48, 0x01, 0xd7, 0x48, 0x29, 0xd6]);
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x85, "getrandom-output-loop");
    code.bytes(&[0x48, 0xff, 0x41, 0x18]);
    code.bytes(&[0x58]);
    code.jump("return-user");
    code.bind("getrandom-zero")?;
    code.bytes(&[0x31, 0xc0]);
    code.jump("return-user");
    code.bind("getrandom-efault")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xf2, 0xff, 0xff, 0xff]);
    code.jump("return-user");
    code.bind("getrandom-einval")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xea, 0xff, 0xff, 0xff]);
    code.jump("return-user");

    code.bind("getpid")?;
    code.bytes(&[0x48, 0x81, 0xf9]);
    code.bytes(&0x0060_0007_u32.to_le_bytes());
    code.jump_if(0x84, "getpid-provider-probe");
    code.bytes(&[0x41, 0xba, 0x01, 0x00, 0x00, 0x00]);
    code.jump("scalar-identity");

    code.bind("mmap")?;
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x85, "mmap-einval");
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x84, "mmap-einval");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x10, 0x00, 0x00]);
    code.jump_if(0x87, "mmap-einval");
    code.bytes(&[0x48, 0x83, 0xfa, 0x03]);
    code.jump_if(0x85, "mmap-einval");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25, 0x50, 0x00, 0x00, 0x00, 0x22]);
    code.jump_if(0x85, "mmap-einval");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25, 0x40, 0x00, 0x00, 0x00, 0xff]);
    code.jump_if(0x85, "mmap-einval");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25, 0x48, 0x00, 0x00, 0x00, 0x00]);
    code.jump_if(0x85, "mmap-einval");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25, 0xc8, 0x00, 0x00, 0x00, 0x01]);
    code.jump_if(0x85, "mmap-enomem");
    code.bytes(&[0x65, 0x4c, 0x8b, 0x14, 0x25, 0xc0, 0x00, 0x00, 0x00]);
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x4d, 0x89, 0xd0, 0x49, 0x81, 0xc0, 0x00, 0x10, 0x00, 0x00]);
    code.bind("mmap-zero-loop")?;
    code.bytes(&[0x49, 0xc7, 0x02, 0, 0, 0, 0, 0x49, 0x83, 0xc2, 0x08]);
    code.bytes(&[0x4d, 0x39, 0xc2]);
    code.jump_if(0x85, "mmap-zero-loop");
    code.bytes(&[0x65, 0x48, 0x8b, 0x0c, 0x25, 0x68, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xc1, 0x30, 0x20, 0x00, 0x00]);
    code.bytes(&[0x48, 0x83, 0x39, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25, 0xc0, 0x00, 0x00, 0x00]);
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x8000_0000_0000_0007_u64.to_le_bytes());
    code.bytes(&[0x4c, 0x09, 0xd0, 0x48, 0x89, 0x01, 0x0f, 0xae, 0xf0]);
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x0060_6000_u64.to_le_bytes());
    code.bytes(&[0x41, 0x0f, 0x01, 0x3a]);
    code.bytes(&[
        0x65, 0x48, 0xc7, 0x04, 0x25, 0xc8, 0x00, 0x00, 0x00, 2, 0, 0, 0,
    ]);
    code.bytes(&[0xb8]);
    code.bytes(&0x0060_6000_u32.to_le_bytes());
    code.jump("return-user");
    code.bind("mmap-enomem")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xf4, 0xff, 0xff, 0xff]);
    code.jump("return-user");
    code.bind("mmap-einval")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xea, 0xff, 0xff, 0xff]);
    code.jump("return-user");

    code.bind("mprotect")?;
    code.bytes(&[0x48, 0x81, 0xff, 0x00, 0x60, 0x60, 0x00]);
    code.jump_if(0x85, "mprotect-enomem");
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x84, "mprotect-einval");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x10, 0x00, 0x00]);
    code.jump_if(0x87, "mprotect-einval");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25, 0xc8, 0x00, 0x00, 0x00, 0x01]);
    code.jump_if(0x84, "mprotect-enomem");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25, 0xc8, 0x00, 0x00, 0x00, 0x02]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25, 0xc8, 0x00, 0x00, 0x00, 0x04]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x48, 0x83, 0xfa, 0x00]);
    code.jump_if(0x84, "mprotect-none");
    code.bytes(&[0x48, 0x83, 0xfa, 0x01]);
    code.jump_if(0x84, "mprotect-read");
    code.bytes(&[0x48, 0x83, 0xfa, 0x03]);
    code.jump_if(0x85, "mprotect-einval");
    code.bytes(&[0x49, 0xbb]);
    code.bytes(&0x8000_0000_0000_0007_u64.to_le_bytes());
    code.bytes(&[0x41, 0xba, 0x02, 0x00, 0x00, 0x00]);
    code.jump("mprotect-publish");
    code.bind("mprotect-read")?;
    code.bytes(&[0x49, 0xbb]);
    code.bytes(&0x8000_0000_0000_0005_u64.to_le_bytes());
    code.bytes(&[0x41, 0xba, 0x03, 0x00, 0x00, 0x00]);
    code.jump("mprotect-publish");
    code.bind("mprotect-none")?;
    code.bytes(&[0x45, 0x31, 0xdb]);
    code.bytes(&[0x41, 0xba, 0x04, 0x00, 0x00, 0x00]);
    code.bind("mprotect-publish")?;
    code.bytes(&[0x65, 0x48, 0x8b, 0x0c, 0x25, 0x68, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xc1, 0x30, 0x20, 0x00, 0x00]);
    code.bytes(&[0x4d, 0x85, 0xdb]);
    code.jump_if(0x84, "mprotect-zero");
    code.bytes(&[0x65, 0x48, 0x8b, 0x04, 0x25, 0xc0, 0x00, 0x00, 0x00]);
    code.bytes(&[0x4c, 0x09, 0xd8]);
    code.jump("mprotect-store");
    code.bind("mprotect-zero")?;
    code.bytes(&[0x31, 0xc0]);
    code.bind("mprotect-store")?;
    code.bytes(&[0x48, 0x89, 0x01, 0x0f, 0xae, 0xf0]);
    code.bytes(&[0x49, 0xbb]);
    code.bytes(&0x0060_6000_u64.to_le_bytes());
    code.bytes(&[0x41, 0x0f, 0x01, 0x3b]);
    code.bytes(&[0x65, 0x4c, 0x89, 0x14, 0x25, 0xc8, 0x00, 0x00, 0x00]);
    code.bytes(&[0x31, 0xc0]);
    code.jump("return-user");
    code.bind("mprotect-enomem")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xf4, 0xff, 0xff, 0xff]);
    code.jump("return-user");
    code.bind("mprotect-einval")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xea, 0xff, 0xff, 0xff]);
    code.jump("return-user");

    code.bind("madvise")?;
    code.bytes(&[0x48, 0x81, 0xff, 0x00, 0x60, 0x60, 0x00]);
    code.jump_if(0x85, "madvise-enomem");
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x84, "madvise-einval");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x10, 0x00, 0x00]);
    code.jump_if(0x87, "madvise-einval");
    code.bytes(&[0x48, 0x83, 0xfa, 0x04]);
    code.jump_if(0x85, "madvise-einval");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25, 0xc8, 0x00, 0x00, 0x00, 0x01]);
    code.jump_if(0x84, "madvise-enomem");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25, 0xc8, 0x00, 0x00, 0x00, 0x02]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25, 0xc8, 0x00, 0x00, 0x00, 0x04]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x65, 0x4c, 0x8b, 0x14, 0x25, 0xc0, 0x00, 0x00, 0x00]);
    code.bytes(&[0x4d, 0x85, 0xd2]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x4d, 0x89, 0xd0, 0x49, 0x81, 0xc0, 0x00, 0x10, 0x00, 0x00]);
    code.bind("madvise-zero-loop")?;
    code.bytes(&[0x49, 0xc7, 0x02, 0, 0, 0, 0, 0x49, 0x83, 0xc2, 0x08]);
    code.bytes(&[0x4d, 0x39, 0xc2]);
    code.jump_if(0x85, "madvise-zero-loop");
    code.bytes(&[0x31, 0xc0]);
    code.jump("return-user");
    code.bind("madvise-enomem")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xf4, 0xff, 0xff, 0xff]);
    code.jump("return-user");
    code.bind("madvise-einval")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xea, 0xff, 0xff, 0xff]);
    code.jump("return-user");

    code.bind("munmap")?;
    code.bytes(&[0x48, 0x81, 0xff, 0x00, 0x60, 0x60, 0x00]);
    code.jump_if(0x85, "munmap-einval");
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x84, "munmap-einval");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x10, 0x00, 0x00]);
    code.jump_if(0x87, "munmap-einval");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25, 0xc8, 0x00, 0x00, 0x00, 0x01]);
    code.jump_if(0x84, "munmap-success");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25, 0xc8, 0x00, 0x00, 0x00, 0x02]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25, 0xc8, 0x00, 0x00, 0x00, 0x04]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x65, 0x48, 0x8b, 0x0c, 0x25, 0x68, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xc1, 0x30, 0x20, 0x00, 0x00]);
    code.bytes(&[0x48, 0xc7, 0x01, 0, 0, 0, 0, 0x0f, 0xae, 0xf0]);
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x0060_6000_u64.to_le_bytes());
    code.bytes(&[0x41, 0x0f, 0x01, 0x3a]);
    code.bytes(&[
        0x65, 0x48, 0xc7, 0x04, 0x25, 0xc8, 0x00, 0x00, 0x00, 1, 0, 0, 0,
    ]);
    code.bind("munmap-success")?;
    code.bytes(&[0x31, 0xc0]);
    code.jump("return-user");
    code.bind("munmap-einval")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xea, 0xff, 0xff, 0xff]);
    code.jump("return-user");

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

    code.bind("arch-prctl")?;
    code.bytes(&[0x48, 0x81, 0xf9, 0x00, 0x40, 0x60, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x81, 0xf9, 0x00, 0x50, 0x60, 0x00]);
    code.jump_if(0x83, "fail");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x00, 0x00, 0x00, 0x00, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x81, 0xff, 0x02, 0x10, 0x00, 0x00]);
    code.jump_if(0x84, "arch-set-fs");
    code.bytes(&[0x81, 0xff, 0x03, 0x10, 0x00, 0x00]);
    code.jump_if(0x84, "arch-get-fs");
    code.bytes(&[0x48, 0xc7, 0xc0, 0xea, 0xff, 0xff, 0xff]); // -EINVAL
    code.jump("return-user");

    code.bind("set-tid-address")?;
    code.bytes(&[0x48, 0x81, 0xf9, 0x00, 0x40, 0x60, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x81, 0xf9, 0x00, 0x50, 0x60, 0x00]);
    code.jump_if(0x83, "fail");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x00, 0x00, 0x00, 0x00, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "set-tid-address-accepted");
    code.bytes(&[0x49, 0x89, 0xf8, 0x49, 0x83, 0xc0, 0x04]);
    code.jump_if(0x82, "set-tid-address-efault");
    code.bytes(&[0x48, 0x81, 0xff, 0x00, 0x00, 0x60, 0x00]);
    code.jump_if(0x82, "set-tid-address-second-range");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x20, 0x60, 0x00]);
    code.jump_if(0x86, "set-tid-address-accepted");
    code.bind("set-tid-address-second-range")?;
    code.bytes(&[0x48, 0x81, 0xff, 0x00, 0x50, 0x60, 0x00]);
    code.jump_if(0x82, "set-tid-address-efault");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x60, 0x60, 0x00]);
    code.jump_if(0x87, "set-tid-address-efault");
    code.bind("set-tid-address-accepted")?;
    code.bytes(&[0x65, 0x48, 0x89, 0x3c, 0x25, 0xa8, 0x00, 0x00, 0x00]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00]);
    code.jump("return-user");
    code.bind("set-tid-address-efault")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xf2, 0xff, 0xff, 0xff]);
    code.jump("return-user");

    code.bind("set-robust-list")?;
    code.bytes(&[0x48, 0x81, 0xf9, 0x00, 0x40, 0x60, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x81, 0xf9, 0x00, 0x50, 0x60, 0x00]);
    code.jump_if(0x83, "fail");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x00, 0x00, 0x00, 0x00, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x83, 0xfe, 0x18]);
    code.jump_if(0x85, "set-robust-list-einval");
    code.bytes(&[0x49, 0x89, 0xf8, 0x49, 0x83, 0xc0, 0x18]);
    code.jump_if(0x82, "set-robust-list-efault");
    code.bytes(&[0x48, 0x81, 0xff, 0x00, 0x00, 0x60, 0x00]);
    code.jump_if(0x82, "set-robust-list-second-range");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x20, 0x60, 0x00]);
    code.jump_if(0x86, "set-robust-list-accepted");
    code.bind("set-robust-list-second-range")?;
    code.bytes(&[0x48, 0x81, 0xff, 0x00, 0x40, 0x60, 0x00]);
    code.jump_if(0x82, "set-robust-list-efault");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x60, 0x60, 0x00]);
    code.jump_if(0x87, "set-robust-list-efault");
    code.bind("set-robust-list-accepted")?;
    code.bytes(&[0x65, 0x48, 0x89, 0x3c, 0x25, 0xb0, 0x00, 0x00, 0x00]);
    code.bytes(&[0x31, 0xc0]);
    code.jump("return-user");
    code.bind("set-robust-list-einval")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xea, 0xff, 0xff, 0xff]);
    code.jump("return-user");
    code.bind("set-robust-list-efault")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xf2, 0xff, 0xff, 0xff]);
    code.jump("return-user");

    code.bind("getrlimit")?;
    code.bytes(&[0x48, 0x81, 0xf9, 0x00, 0x40, 0x60, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x81, 0xf9, 0x00, 0x50, 0x60, 0x00]);
    code.jump_if(0x83, "fail");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x00, 0x00, 0x00, 0x00, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x83, 0xff, 0x03]);
    code.jump_if(0x85, "rlimit-einval");
    code.bytes(&[0x49, 0x89, 0xf1]);
    code.jump("rlimit-copy");

    code.bind("prlimit64")?;
    code.bytes(&[0x48, 0x81, 0xf9, 0x00, 0x40, 0x60, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x81, 0xf9, 0x00, 0x50, 0x60, 0x00]);
    code.jump_if(0x83, "fail");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x00, 0x00, 0x00, 0x00, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x85, 0xff]);
    code.jump_if(0x84, "prlimit64-pid-ok");
    code.bytes(&[0x48, 0x83, 0xff, 0x01]);
    code.jump_if(0x85, "rlimit-esrch");
    code.bind("prlimit64-pid-ok")?;
    code.bytes(&[0x48, 0x83, 0xfe, 0x03]);
    code.jump_if(0x85, "rlimit-einval");
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x85, "rlimit-einval");
    code.bytes(&[0x65, 0x4c, 0x8b, 0x0c, 0x25, 0x50, 0x00, 0x00, 0x00]);

    code.bind("rlimit-copy")?;
    code.bytes(&[0x4d, 0x89, 0xc8, 0x49, 0x83, 0xc0, 0x10]);
    code.jump_if(0x82, "rlimit-efault");
    code.bytes(&[0x49, 0x81, 0xf9, 0x00, 0x00, 0x60, 0x00]);
    code.jump_if(0x82, "rlimit-second-range");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x20, 0x60, 0x00]);
    code.jump_if(0x86, "rlimit-candidate-ok");
    code.bind("rlimit-second-range")?;
    code.bytes(&[0x49, 0x81, 0xf9, 0x00, 0x50, 0x60, 0x00]);
    code.jump_if(0x82, "rlimit-efault");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x60, 0x60, 0x00]);
    code.jump_if(0x87, "rlimit-efault");
    code.bind("rlimit-candidate-ok")?;
    code.rip_lea(&[0x4c, 0x8d, 0x05], "rlimit-data");
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "rlimit-copy-fault-site");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x20, 0x00, 0x00, 0x00]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "rlimit-copy-recovery");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x28, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x01]);
    code.bind("rlimit-copy-loop")?;
    code.bytes(&[0x43, 0x8a, 0x04, 0x10]);
    code.bind("rlimit-copy-fault-site")?;
    code.bytes(&[0x43, 0x88, 0x04, 0x11]);
    code.bytes(&[0x49, 0xff, 0xc2]);
    code.bytes(&[0x49, 0x83, 0xfa, 0x10]);
    code.jump_if(0x85, "rlimit-copy-loop");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0x31, 0xc0]);
    code.jump("return-user");
    code.bind("rlimit-copy-recovery")?;
    code.bytes(&[0x65, 0x4c, 0x89, 0x14, 0x25, 0x60, 0x00, 0x00, 0x00]);
    code.bind("rlimit-efault")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xf2, 0xff, 0xff, 0xff]);
    code.jump("return-user");
    code.bind("rlimit-einval")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xea, 0xff, 0xff, 0xff]);
    code.jump("return-user");
    code.bind("rlimit-esrch")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xfd, 0xff, 0xff, 0xff]);
    code.jump("return-user");

    code.bind("clock-gettime")?;
    code.bytes(&[0x48, 0x83, 0xff, 0x01]);
    code.jump_if(0x85, "clock-gettime-einval");
    code.bytes(&[0x49, 0x89, 0xf1, 0x49, 0x89, 0xf0, 0x49, 0x83, 0xc0, 0x10]);
    code.jump_if(0x82, "clock-gettime-efault");
    code.bytes(&[0x49, 0x81, 0xf9, 0x00, 0x00, 0x60, 0x00]);
    code.jump_if(0x82, "clock-gettime-second-range");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x20, 0x60, 0x00]);
    code.jump_if(0x86, "clock-gettime-candidate-ok");
    code.bind("clock-gettime-second-range")?;
    code.bytes(&[0x49, 0x81, 0xf9, 0x00, 0x50, 0x60, 0x00]);
    code.jump_if(0x82, "clock-gettime-efault");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x60, 0x60, 0x00]);
    code.jump_if(0x87, "clock-gettime-efault");
    code.bind("clock-gettime-candidate-ok")?;
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x0000_0000_fed0_0000_u64.to_le_bytes());
    code.bytes(&[0x49, 0x8b, 0x02]);
    code.bytes(&[0xa9, 0x00, 0x20, 0x00, 0x00]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x89, 0xc2, 0x48, 0xc1, 0xea, 0x20]);
    code.bytes(&[0x81, 0xfa, 0x80, 0x96, 0x98, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x8b, 0x42, 0x10, 0x48, 0x83, 0xc8, 0x01]);
    code.bytes(&[0x49, 0x89, 0x42, 0x10]);
    code.bytes(&[0x49, 0x8b, 0x82, 0xf0, 0x00, 0x00, 0x00]);
    code.bytes(&[0x31, 0xd2, 0x41, 0xba]);
    code.bytes(&100_000_000_u32.to_le_bytes());
    code.bytes(&[0x49, 0xf7, 0xf2]);
    code.bytes(&[0x48, 0x6b, 0xd2, 0x0a]);
    code.bytes(&[0x48, 0x83, 0xec, 0x10, 0x48, 0x89, 0x04, 0x24]);
    code.bytes(&[0x48, 0x89, 0x54, 0x24, 0x08, 0x49, 0x89, 0xe0]);
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "clock-gettime-copy-fault-site");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x20, 0x00, 0x00, 0x00]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "clock-gettime-copy-recovery");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x28, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x01]);
    code.bind("clock-gettime-copy-loop")?;
    code.bytes(&[0x43, 0x8a, 0x04, 0x10]);
    code.bind("clock-gettime-copy-fault-site")?;
    code.bytes(&[0x43, 0x88, 0x04, 0x11, 0x49, 0xff, 0xc2]);
    code.bytes(&[0x49, 0x83, 0xfa, 0x10]);
    code.jump_if(0x85, "clock-gettime-copy-loop");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x83, 0xc4, 0x10, 0x31, 0xc0]);
    code.jump("return-user");
    code.bind("clock-gettime-copy-recovery")?;
    code.bytes(&[0x65, 0x4c, 0x89, 0x14, 0x25, 0x60, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x83, 0xc4, 0x10]);
    code.bind("clock-gettime-efault")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xf2, 0xff, 0xff, 0xff]);
    code.jump("return-user");
    code.bind("clock-gettime-einval")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xea, 0xff, 0xff, 0xff]);
    code.jump("return-user");

    code.bind("clock-getres")?;
    code.bytes(&[0x48, 0x83, 0xff, 0x01]);
    code.jump_if(0x85, "clock-getres-einval");
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x84, "clock-getres-success");
    code.bytes(&[0x49, 0x89, 0xf1, 0x49, 0x89, 0xf0, 0x49, 0x83, 0xc0, 0x10]);
    code.jump_if(0x82, "clock-getres-efault");
    code.bytes(&[0x49, 0x81, 0xf9, 0x00, 0x00, 0x60, 0x00]);
    code.jump_if(0x82, "clock-getres-second-range");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x20, 0x60, 0x00]);
    code.jump_if(0x86, "clock-getres-candidate-ok");
    code.bind("clock-getres-second-range")?;
    code.bytes(&[0x49, 0x81, 0xf9, 0x00, 0x50, 0x60, 0x00]);
    code.jump_if(0x82, "clock-getres-efault");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x60, 0x60, 0x00]);
    code.jump_if(0x87, "clock-getres-efault");
    code.bind("clock-getres-candidate-ok")?;
    code.bytes(&[0x48, 0x83, 0xec, 0x10, 0x48, 0xc7, 0x04, 0x24, 0, 0, 0, 0]);
    code.bytes(&[0x48, 0xc7, 0x44, 0x24, 0x08, 0x0a, 0, 0, 0]);
    code.bytes(&[0x49, 0x89, 0xe0, 0x45, 0x31, 0xd2]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "clock-getres-copy-fault-site");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x20, 0x00, 0x00, 0x00]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "clock-getres-copy-recovery");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x28, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x01]);
    code.bind("clock-getres-copy-loop")?;
    code.bytes(&[0x43, 0x8a, 0x04, 0x10]);
    code.bind("clock-getres-copy-fault-site")?;
    code.bytes(&[0x43, 0x88, 0x04, 0x11, 0x49, 0xff, 0xc2]);
    code.bytes(&[0x49, 0x83, 0xfa, 0x10]);
    code.jump_if(0x85, "clock-getres-copy-loop");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x83, 0xc4, 0x10]);
    code.bind("clock-getres-success")?;
    code.bytes(&[0x31, 0xc0]);
    code.jump("return-user");
    code.bind("clock-getres-copy-recovery")?;
    code.bytes(&[0x65, 0x4c, 0x89, 0x14, 0x25, 0x60, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x83, 0xc4, 0x10]);
    code.bind("clock-getres-efault")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xf2, 0xff, 0xff, 0xff]);
    code.jump("return-user");
    code.bind("clock-getres-einval")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xea, 0xff, 0xff, 0xff]);
    code.jump("return-user");

    code.bind("rt-sigprocmask")?;
    code.bytes(&[0x48, 0x81, 0xf9, 0x00, 0x40, 0x60, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x48, 0x81, 0xf9, 0x00, 0x50, 0x60, 0x00]);
    code.jump_if(0x83, "fail");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0x00, 0x00, 0x00, 0x00, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x65, 0x4c, 0x8b, 0x04, 0x25, 0x50, 0x00, 0x00, 0x00]);
    code.bytes(&[0x49, 0x83, 0xf8, 0x08]);
    code.jump_if(0x85, "signal-mask-einval");
    code.bytes(&[0x65, 0x4c, 0x8b, 0x0c, 0x25, 0xb8, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x84, "signal-mask-output");
    code.bytes(&[0x49, 0x89, 0xf0, 0x49, 0x83, 0xc0, 0x08]);
    code.jump_if(0x82, "signal-mask-efault");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x00, 0x60, 0x00]);
    code.jump_if(0x82, "signal-mask-input-second-range");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x20, 0x60, 0x00]);
    code.jump_if(0x86, "signal-mask-input-ok");
    code.bind("signal-mask-input-second-range")?;
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x50, 0x60, 0x00]);
    code.jump_if(0x82, "signal-mask-efault");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x60, 0x60, 0x00]);
    code.jump_if(0x87, "signal-mask-efault");
    code.bind("signal-mask-input-ok")?;
    code.rip_lea(&[0x48, 0x8d, 0x05], "signal-mask-input-fault-site");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x20, 0x00, 0x00, 0x00]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "signal-mask-input-recovery");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x28, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x02]);
    code.bind("signal-mask-input-fault-site")?;
    code.bytes(&[0x4c, 0x8b, 0x06]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0x49, 0x81, 0xe0, 0xff, 0xfe, 0xfb, 0xff]);
    code.bytes(&[0x83, 0xff, 0x00]);
    code.jump_if(0x84, "signal-mask-block");
    code.bytes(&[0x83, 0xff, 0x01]);
    code.jump_if(0x84, "signal-mask-unblock");
    code.bytes(&[0x83, 0xff, 0x02]);
    code.jump_if(0x85, "signal-mask-einval");
    code.jump("signal-mask-commit");
    code.bind("signal-mask-block")?;
    code.bytes(&[0x4d, 0x09, 0xc8]);
    code.jump("signal-mask-commit");
    code.bind("signal-mask-unblock")?;
    code.bytes(&[0x49, 0xf7, 0xd0, 0x4d, 0x21, 0xc8]);
    code.bind("signal-mask-commit")?;
    code.bytes(&[0x65, 0x4c, 0x89, 0x04, 0x25, 0xb8, 0x00, 0x00, 0x00]);
    code.bind("signal-mask-output")?;
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x84, "signal-mask-success");
    code.bytes(&[0x49, 0x89, 0xd0, 0x49, 0x83, 0xc0, 0x08]);
    code.jump_if(0x82, "signal-mask-efault");
    code.bytes(&[0x48, 0x81, 0xfa, 0x00, 0x00, 0x60, 0x00]);
    code.jump_if(0x82, "signal-mask-output-second-range");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x20, 0x60, 0x00]);
    code.jump_if(0x86, "signal-mask-output-ok");
    code.bind("signal-mask-output-second-range")?;
    code.bytes(&[0x48, 0x81, 0xfa, 0x00, 0x50, 0x60, 0x00]);
    code.jump_if(0x82, "signal-mask-efault");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x60, 0x60, 0x00]);
    code.jump_if(0x87, "signal-mask-efault");
    code.bind("signal-mask-output-ok")?;
    code.rip_lea(&[0x48, 0x8d, 0x05], "signal-mask-output-fault-site");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x20, 0x00, 0x00, 0x00]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "signal-mask-output-recovery");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x28, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x01]);
    code.bind("signal-mask-output-fault-site")?;
    code.bytes(&[0x4c, 0x89, 0x0a]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x00]);
    code.bind("signal-mask-success")?;
    code.bytes(&[0x31, 0xc0]);
    code.jump("return-user");
    code.bind("signal-mask-input-recovery")?;
    code.bytes(&[0x65, 0x4c, 0x89, 0x14, 0x25, 0x60, 0x00, 0x00, 0x00]);
    code.jump("signal-mask-efault");
    code.bind("signal-mask-output-recovery")?;
    code.bytes(&[0x65, 0x4c, 0x89, 0x14, 0x25, 0x60, 0x00, 0x00, 0x00]);
    code.bind("signal-mask-efault")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xf2, 0xff, 0xff, 0xff]);
    code.jump("return-user");
    code.bind("signal-mask-einval")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xea, 0xff, 0xff, 0xff]);
    code.jump("return-user");

    code.bind("arch-set-fs")?;
    code.bytes(&[0x48, 0x85, 0xf6]);
    code.jump_if(0x84, "arch-set-fs-accepted");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x00, 0x60, 0x00]);
    code.jump_if(0x82, "arch-set-fs-second-range");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x20, 0x60, 0x00]);
    code.jump_if(0x82, "arch-set-fs-accepted");
    code.bind("arch-set-fs-second-range")?;
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x40, 0x60, 0x00]);
    code.jump_if(0x82, "arch-set-fs-eperm");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x60, 0x60, 0x00]);
    code.jump_if(0x83, "arch-set-fs-eperm");
    code.bind("arch-set-fs-accepted")?;
    code.bytes(&[0x65, 0x48, 0x89, 0x34, 0x25, 0xa0, 0x00, 0x00, 0x00]);
    code.bytes(&[0xb9, 0x00, 0x01, 0x00, 0xc0]); // IA32_FS_BASE
    code.bytes(&[0x48, 0x89, 0xf0, 0x48, 0x89, 0xc2, 0x48, 0xc1, 0xea, 0x20]);
    code.bytes(&[0x0f, 0x30, 0x31, 0xc0]); // wrmsr; success
    code.jump("return-user");
    code.bind("arch-set-fs-eperm")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xff, 0xff, 0xff, 0xff]); // -EPERM
    code.jump("return-user");

    code.bind("arch-get-fs")?;
    code.bytes(&[0x49, 0x89, 0xf0, 0x49, 0x83, 0xc0, 0x08]);
    code.jump_if(0x82, "arch-get-fs-efault");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x00, 0x60, 0x00]);
    code.jump_if(0x82, "arch-get-fs-second-range");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x20, 0x60, 0x00]);
    code.jump_if(0x86, "arch-get-fs-candidate-ok");
    code.bind("arch-get-fs-second-range")?;
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x50, 0x60, 0x00]);
    code.jump_if(0x82, "arch-get-fs-efault");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x60, 0x60, 0x00]);
    code.jump_if(0x87, "arch-get-fs-efault");
    code.bind("arch-get-fs-candidate-ok")?;
    code.bytes(&[0x49, 0x89, 0xf1]);
    code.bytes(&[0x65, 0x4c, 0x8b, 0x04, 0x25, 0xa0, 0x00, 0x00, 0x00]);
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "arch-get-fs-fault-site");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x20, 0x00, 0x00, 0x00]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "arch-get-fs-recovery");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x28, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x01]);
    code.bind("arch-get-fs-fault-site")?;
    code.bytes(&[0x4d, 0x89, 0x01]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0x31, 0xc0]);
    code.jump("return-user");
    code.bind("arch-get-fs-recovery")?;
    code.bytes(&[0x65, 0x4c, 0x89, 0x14, 0x25, 0x60, 0x00, 0x00, 0x00]);
    code.bind("arch-get-fs-efault")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xf2, 0xff, 0xff, 0xff]); // -EFAULT
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
    code.bytes(&[0x48, 0x83, 0xff, 0x02]);
    code.jump_if(0x87, "write-ebadf");
    code.bytes(&[0x65, 0x44, 0x8a, 0x87, 0xe0, 0x00, 0x00, 0x00]);
    code.bytes(&[0x41, 0x80, 0xf8, 0x02]);
    code.jump_if(0x84, "write-descriptor-ok");
    code.bytes(&[0x41, 0x80, 0xf8, 0x03]);
    code.jump_if(0x85, "write-ebadf");
    code.bind("write-descriptor-ok")?;
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
    code.bytes(&[0x48, 0x83, 0xff, 0x02]);
    code.jump_if(0x87, "read-ebadf");
    code.bytes(&[0x65, 0x44, 0x8a, 0x87, 0xe0, 0x00, 0x00, 0x00]);
    code.bytes(&[0x41, 0x80, 0xf8, 0x01]);
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

    code.bind("close")?;
    code.bytes(&[0x48, 0x83, 0xff, 0x02]);
    code.jump_if(0x87, "close-ebadf");
    code.bytes(&[0x65, 0x44, 0x8a, 0x87, 0xe0, 0x00, 0x00, 0x00]);
    code.bytes(&[0x45, 0x84, 0xc0]);
    code.jump_if(0x84, "close-ebadf");
    code.bytes(&[0x65, 0xc6, 0x87, 0xe0, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0xc6, 0x87, 0xe8, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0x31, 0xc0]);
    code.jump("return-user");
    code.bind("close-ebadf")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xf7, 0xff, 0xff, 0xff]);
    code.jump("return-user");

    code.bind("dup")?;
    code.bytes(&[0x48, 0x83, 0xff, 0x02]);
    code.jump_if(0x87, "close-ebadf");
    code.bytes(&[0x65, 0x44, 0x8a, 0x87, 0xe0, 0x00, 0x00, 0x00]);
    code.bytes(&[0x45, 0x84, 0xc0]);
    code.jump_if(0x84, "close-ebadf");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0xe0, 0x00, 0x00, 0x00, 0x00]);
    code.jump_if(0x84, "dup-zero");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0xe1, 0x00, 0x00, 0x00, 0x00]);
    code.jump_if(0x84, "dup-one");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0xe2, 0x00, 0x00, 0x00, 0x00]);
    code.jump_if(0x84, "dup-two");
    code.bytes(&[0x48, 0xc7, 0xc0, 0xe8, 0xff, 0xff, 0xff]);
    code.jump("return-user");
    code.bind("dup-zero")?;
    code.bytes(&[0x65, 0x44, 0x88, 0x04, 0x25, 0xe0, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0xe8, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0x31, 0xc0]);
    code.jump("return-user");
    code.bind("dup-one")?;
    code.bytes(&[0x65, 0x44, 0x88, 0x04, 0x25, 0xe1, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0xe9, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00]);
    code.jump("return-user");
    code.bind("dup-two")?;
    code.bytes(&[0x65, 0x44, 0x88, 0x04, 0x25, 0xe2, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0xea, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0xb8, 0x02, 0x00, 0x00, 0x00]);
    code.jump("return-user");

    code.bind("dup2")?;
    code.bytes(&[0x48, 0x83, 0xff, 0x02]);
    code.jump_if(0x87, "close-ebadf");
    code.bytes(&[0x65, 0x44, 0x8a, 0x87, 0xe0, 0x00, 0x00, 0x00]);
    code.bytes(&[0x45, 0x84, 0xc0]);
    code.jump_if(0x84, "close-ebadf");
    code.bytes(&[0x48, 0x83, 0xfe, 0x02]);
    code.jump_if(0x87, "close-ebadf");
    code.bytes(&[0x48, 0x39, 0xf7]);
    code.jump_if(0x84, "dup2-same");
    code.bytes(&[0x65, 0x44, 0x88, 0x86, 0xe0, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0xc6, 0x86, 0xe8, 0x00, 0x00, 0x00, 0x00]);
    code.bind("dup2-same")?;
    code.bytes(&[0x48, 0x89, 0xf0]);
    code.jump("return-user");

    code.bind("fcntl")?;
    code.bytes(&[0x48, 0x83, 0xff, 0x02]);
    code.jump_if(0x87, "close-ebadf");
    code.bytes(&[0x65, 0x44, 0x8a, 0x87, 0xe0, 0x00, 0x00, 0x00]);
    code.bytes(&[0x45, 0x84, 0xc0]);
    code.jump_if(0x84, "close-ebadf");
    code.bytes(&[0x48, 0x83, 0xfe, 0x01]);
    code.jump_if(0x84, "fcntl-getfd");
    code.bytes(&[0x48, 0x83, 0xfe, 0x02]);
    code.jump_if(0x84, "fcntl-setfd");
    code.bytes(&[0x48, 0x83, 0xfe, 0x03]);
    code.jump_if(0x84, "fcntl-getfl");
    code.bytes(&[0x48, 0xc7, 0xc0, 0xea, 0xff, 0xff, 0xff]);
    code.jump("return-user");
    code.bind("fcntl-getfd")?;
    code.bytes(&[0x65, 0x0f, 0xb6, 0x87, 0xe8, 0x00, 0x00, 0x00]);
    code.jump("return-user");
    code.bind("fcntl-setfd")?;
    code.bytes(&[0x80, 0xe2, 0x01]);
    code.bytes(&[0x65, 0x88, 0x97, 0xe8, 0x00, 0x00, 0x00]);
    code.bytes(&[0x31, 0xc0]);
    code.jump("return-user");
    code.bind("fcntl-getfl")?;
    code.bytes(&[0x31, 0xc0, 0x41, 0x80, 0xf8, 0x01]);
    code.jump_if(0x84, "return-user");
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00]);
    code.jump("return-user");

    code.bind("fstat")?;
    code.bytes(&[0x48, 0x83, 0xff, 0x02]);
    code.jump_if(0x87, "close-ebadf");
    code.bytes(&[0x65, 0x44, 0x8a, 0x87, 0xe0, 0x00, 0x00, 0x00]);
    code.bytes(&[0x45, 0x84, 0xc0]);
    code.jump_if(0x84, "close-ebadf");
    code.bytes(&[0x49, 0x89, 0xf1]); // destination candidate
    code.bytes(&[0x49, 0x89, 0xf0, 0x49, 0x81, 0xc0, 0x90, 0x00, 0x00, 0x00]);
    code.jump_if(0x82, "fstat-efault");
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x10, 0x60, 0x00]);
    code.jump_if(0x82, "fstat-second-range");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x20, 0x60, 0x00]);
    code.jump_if(0x86, "fstat-candidate-ok");
    code.bind("fstat-second-range")?;
    code.bytes(&[0x48, 0x81, 0xfe, 0x00, 0x50, 0x60, 0x00]);
    code.jump_if(0x82, "fstat-efault");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x60, 0x60, 0x00]);
    code.jump_if(0x87, "fstat-efault");
    code.bind("fstat-candidate-ok")?;
    code.bytes(&[0x48, 0x81, 0xec, 0x90, 0x00, 0x00, 0x00]); // private stat staging
    code.bytes(&[0x31, 0xc0]);
    for offset in (0_u32..144).step_by(8) {
        if offset == 0 {
            code.bytes(&[0x48, 0x89, 0x04, 0x24]);
        } else if offset <= 0x7f {
            code.bytes(&[0x48, 0x89, 0x44, 0x24, offset as u8]);
        } else {
            code.bytes(&[0x48, 0x89, 0x84, 0x24]);
            code.bytes(&offset.to_le_bytes());
        }
    }
    for (offset, value) in [
        (8_u8, 1_u32),
        (16, 1),
        (24, 0x2180),
        (40, 0x4400),
        (56, 4096),
    ] {
        code.bytes(&[0x48, 0xc7, 0x44, 0x24, offset]);
        code.bytes(&value.to_le_bytes());
    }
    code.bytes(&[0x45, 0x31, 0xd2]); // copied prefix
    code.rip_lea(&[0x48, 0x8d, 0x05], "fstat-copy-fault-site");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x20, 0x00, 0x00, 0x00]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "fstat-copy-recovery");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x28, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x01]);
    code.bind("fstat-copy-loop")?;
    code.bytes(&[0x42, 0x8a, 0x04, 0x14]);
    code.bind("fstat-copy-fault-site")?;
    code.bytes(&[0x43, 0x88, 0x04, 0x11]);
    code.bytes(&[0x49, 0xff, 0xc2]);
    code.bytes(&[0x49, 0x81, 0xfa, 0x90, 0x00, 0x00, 0x00]);
    code.jump_if(0x85, "fstat-copy-loop");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x81, 0xc4, 0x90, 0x00, 0x00, 0x00, 0x31, 0xc0]);
    code.jump("return-user");

    code.bind("fstat-copy-recovery")?;
    code.bytes(&[0x65, 0x4c, 0x89, 0x14, 0x25, 0x60, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x81, 0xc4, 0x90, 0x00, 0x00, 0x00]);
    code.bind("fstat-efault")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xf2, 0xff, 0xff, 0xff]);
    code.jump("return-user");

    code.bind("lseek")?;
    code.bytes(&[0x48, 0x83, 0xff, 0x02]);
    code.jump_if(0x87, "close-ebadf");
    code.bytes(&[0x65, 0x44, 0x8a, 0x87, 0xe0, 0x00, 0x00, 0x00]);
    code.bytes(&[0x45, 0x84, 0xc0]);
    code.jump_if(0x84, "close-ebadf");
    code.bytes(&[0x48, 0x83, 0xfa, 0x04]);
    code.jump_if(0x87, "lseek-einval");
    code.bytes(&[0x48, 0xc7, 0xc0, 0xe3, 0xff, 0xff, 0xff]);
    code.jump("return-user");
    code.bind("lseek-einval")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xea, 0xff, 0xff, 0xff]);
    code.jump("return-user");

    code.bind("ioctl")?;
    code.bytes(&[0x48, 0x83, 0xff, 0x02]);
    code.jump_if(0x87, "close-ebadf");
    code.bytes(&[0x65, 0x44, 0x8a, 0x87, 0xe0, 0x00, 0x00, 0x00]);
    code.bytes(&[0x45, 0x84, 0xc0]);
    code.jump_if(0x84, "close-ebadf");
    code.bytes(&[0x81, 0xfe, 0x01, 0x54, 0x00, 0x00]);
    code.jump_if(0x84, "ioctl-tcgets");
    code.bytes(&[0x81, 0xfe, 0x13, 0x54, 0x00, 0x00]);
    code.jump_if(0x84, "ioctl-winsize");
    code.jump("ioctl-enotty");

    code.bind("ioctl-tcgets")?;
    code.bytes(&[0x49, 0x89, 0xd1]);
    code.bytes(&[0x49, 0x89, 0xd0, 0x49, 0x83, 0xc0, 0x24]);
    code.jump_if(0x82, "ioctl-efault");
    code.bytes(&[0x48, 0x81, 0xfa, 0x00, 0x10, 0x60, 0x00]);
    code.jump_if(0x82, "ioctl-second-range");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x20, 0x60, 0x00]);
    code.jump_if(0x86, "ioctl-candidate-ok");
    code.bind("ioctl-second-range")?;
    code.bytes(&[0x48, 0x81, 0xfa, 0x00, 0x50, 0x60, 0x00]);
    code.jump_if(0x82, "ioctl-efault");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x60, 0x60, 0x00]);
    code.jump_if(0x87, "ioctl-efault");
    code.bind("ioctl-candidate-ok")?;
    code.bytes(&[0x48, 0x83, 0xec, 0x28, 0x31, 0xc0]);
    for offset in [0_u8, 8, 16, 24, 32] {
        if offset == 0 {
            code.bytes(&[0x48, 0x89, 0x04, 0x24]);
        } else {
            code.bytes(&[0x48, 0x89, 0x44, 0x24, offset]);
        }
    }
    code.bytes(&[0x48, 0xc7, 0x44, 0x24, 0x08, 0xbf, 0x08, 0x00, 0x00]);
    code.bytes(&[0xc6, 0x44, 0x24, 0x17, 0x01]);
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "ioctl-copy-fault-site");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x20, 0x00, 0x00, 0x00]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "ioctl-copy-recovery");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x28, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x01]);
    code.bind("ioctl-copy-loop")?;
    code.bytes(&[0x42, 0x8a, 0x04, 0x14]);
    code.bind("ioctl-copy-fault-site")?;
    code.bytes(&[0x43, 0x88, 0x04, 0x11]);
    code.bytes(&[0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 0x24]);
    code.jump_if(0x85, "ioctl-copy-loop");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x83, 0xc4, 0x28, 0x31, 0xc0]);
    code.jump("return-user");

    code.bind("ioctl-copy-recovery")?;
    code.bytes(&[0x65, 0x4c, 0x89, 0x14, 0x25, 0x60, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x83, 0xc4, 0x28]);
    code.bind("ioctl-efault")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xf2, 0xff, 0xff, 0xff]);
    code.jump("return-user");

    code.bind("ioctl-winsize")?;
    code.bytes(&[0x49, 0x89, 0xd1]);
    code.bytes(&[0x49, 0x89, 0xd0, 0x49, 0x83, 0xc0, 0x08]);
    code.jump_if(0x82, "ioctl-winsize-efault");
    code.bytes(&[0x48, 0x81, 0xfa, 0x00, 0x10, 0x60, 0x00]);
    code.jump_if(0x82, "ioctl-winsize-second-range");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x20, 0x60, 0x00]);
    code.jump_if(0x86, "ioctl-winsize-candidate-ok");
    code.bind("ioctl-winsize-second-range")?;
    code.bytes(&[0x48, 0x81, 0xfa, 0x00, 0x50, 0x60, 0x00]);
    code.jump_if(0x82, "ioctl-winsize-efault");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x60, 0x60, 0x00]);
    code.jump_if(0x87, "ioctl-winsize-efault");
    code.bind("ioctl-winsize-candidate-ok")?;
    code.bytes(&[0x48, 0x83, 0xec, 0x08, 0x31, 0xc0]);
    code.bytes(&[0x48, 0x89, 0x04, 0x24]);
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "ioctl-winsize-copy-fault-site");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x20, 0x00, 0x00, 0x00]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "ioctl-winsize-copy-recovery");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x28, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x01]);
    code.bind("ioctl-winsize-copy-loop")?;
    code.bytes(&[0x42, 0x8a, 0x04, 0x14]);
    code.bind("ioctl-winsize-copy-fault-site")?;
    code.bytes(&[0x43, 0x88, 0x04, 0x11]);
    code.bytes(&[0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 0x08]);
    code.jump_if(0x85, "ioctl-winsize-copy-loop");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x83, 0xc4, 0x08, 0x31, 0xc0]);
    code.jump("return-user");

    code.bind("ioctl-winsize-copy-recovery")?;
    code.bytes(&[0x65, 0x4c, 0x89, 0x14, 0x25, 0x60, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x83, 0xc4, 0x08]);
    code.bind("ioctl-winsize-efault")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xf2, 0xff, 0xff, 0xff]);
    code.jump("return-user");

    code.bind("ioctl-enotty")?;
    code.bytes(&[0x48, 0xc7, 0xc0, 0xe7, 0xff, 0xff, 0xff]);
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
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0xd0, 0, 0, 0, 1]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x65, 0x48, 0x8b, 0x0c, 0x25, 0xd8, 0, 0, 0]);
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x80, 0x79, 0x10, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x83, 0x79, 0x18, 0x02]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x4c, 0x8b, 0x01, 0x4c, 0x8b, 0x49, 0x08]);
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x4d, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x31, 0xc0]);
    for offset in [0_u8, 8, 16, 24] {
        if offset == 0 {
            code.bytes(&[0x49, 0x89, 0x00, 0x49, 0x89, 0x01, 0x48, 0x89, 0x01]);
        } else {
            code.bytes(&[
                0x49, 0x89, 0x40, offset, 0x49, 0x89, 0x41, offset, 0x48, 0x89, 0x41, offset,
            ]);
        }
    }
    code.bytes(&[0x31, 0xd2]);
    for offset in [0_u8, 8, 16, 24] {
        if offset == 0 {
            code.bytes(&[0x49, 0x0b, 0x10, 0x49, 0x0b, 0x11, 0x48, 0x0b, 0x11]);
        } else {
            code.bytes(&[
                0x49, 0x0b, 0x50, offset, 0x49, 0x0b, 0x51, offset, 0x48, 0x0b, 0x51, offset,
            ]);
        }
    }
    code.bytes(&[0x48, 0x85, 0xd2]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25, 0xd8, 0, 0, 0, 0, 0, 0, 0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0xd0, 0, 0, 0, 0]);
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0xe0, 0, 0, 0, 0]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0xe1, 0, 0, 0, 2]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x65, 0x80, 0x3c, 0x25, 0xe2, 0, 0, 0, 3]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25, 0xe0, 0, 0, 0, 0, 0, 0, 0]);
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25, 0xe0, 0, 0, 0, 0]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25, 0xe8, 0, 0, 0, 0]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x65, 0x48, 0xc7, 0x04, 0x25, 0xe8, 0, 0, 0, 0, 0, 0, 0]);
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25, 0xe8, 0, 0, 0, 0]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25, 0xc8, 0x00, 0x00, 0x00, 0x01]);
    code.jump_if(0x84, "exit-anonymous-already-revoked");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25, 0xc8, 0x00, 0x00, 0x00, 0x02]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25, 0xc8, 0x00, 0x00, 0x00, 0x04]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x65, 0x48, 0x8b, 0x0c, 0x25, 0x68, 0x00, 0x00, 0x00]);
    code.bytes(&[0x48, 0x85, 0xc9]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x81, 0xc1, 0x30, 0x20, 0x00, 0x00]);
    code.bytes(&[0x48, 0xc7, 0x01, 0, 0, 0, 0, 0x0f, 0xae, 0xf0]);
    code.bytes(&[0x49, 0xba]);
    code.bytes(&0x0060_6000_u64.to_le_bytes());
    code.bytes(&[0x41, 0x0f, 0x01, 0x3a]);
    code.bytes(&[
        0x65, 0x48, 0xc7, 0x04, 0x25, 0xc8, 0x00, 0x00, 0x00, 1, 0, 0, 0,
    ]);
    code.bind("exit-anonymous-already-revoked")?;
    code.bytes(&[0x65, 0x48, 0x83, 0x3c, 0x25, 0xc8, 0x00, 0x00, 0x00, 0x01]);
    code.jump_if(0x85, "fail");
    code.bytes(&[
        0x65, 0x48, 0xc7, 0x04, 0x25, 0xb8, 0x00, 0x00, 0x00, 0, 0, 0, 0,
    ]); // discard terminal thread mask
    code.bytes(&[0x65, 0x4c, 0x8b, 0x04, 0x25, 0xb0, 0x00, 0x00, 0x00]);
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "robust-state-cleared");
    code.bytes(&[0x48, 0x83, 0xec, 0x18]);
    code.bytes(&[0x4d, 0x89, 0xc1, 0x45, 0x31, 0xd2]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "robust-copy-fault-site");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x20, 0x00, 0x00, 0x00]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "robust-copy-recovery");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x28, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x02]);
    code.bind("robust-copy-loop")?;
    code.bind("robust-copy-fault-site")?;
    code.bytes(&[0x43, 0x8a, 0x04, 0x11]);
    code.bytes(&[0x42, 0x88, 0x04, 0x14]);
    code.bytes(&[0x49, 0xff, 0xc2]);
    code.bytes(&[0x49, 0x83, 0xfa, 0x18]);
    code.jump_if(0x85, "robust-copy-loop");
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x00]);
    code.bytes(&[0x4c, 0x39, 0x04, 0x24]);
    code.jump_if(0x85, "robust-invalid");
    code.bytes(&[0x48, 0x83, 0x7c, 0x24, 0x10, 0x00]);
    code.jump_if(0x85, "robust-invalid");
    code.bytes(&[0x48, 0x83, 0xc4, 0x18]);
    code.bytes(&[
        0x65, 0x48, 0xc7, 0x04, 0x25, 0xb0, 0x00, 0x00, 0x00, 0, 0, 0, 0,
    ]);
    code.bind("robust-state-cleared")?;
    code.bytes(&[0x65, 0x4c, 0x8b, 0x04, 0x25, 0xa8, 0x00, 0x00, 0x00]);
    code.bytes(&[0x4d, 0x85, 0xc0]);
    code.jump_if(0x84, "clear-tid-state");
    code.bytes(&[0x45, 0x31, 0xd2]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "clear-tid-fault-site");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x20, 0x00, 0x00, 0x00]);
    code.rip_lea(&[0x48, 0x8d, 0x05], "clear-tid-recovery");
    code.bytes(&[0x65, 0x48, 0x89, 0x04, 0x25, 0x28, 0x00, 0x00, 0x00]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x01]);
    code.bind("clear-tid-fault-site")?;
    code.bytes(&[0x41, 0xc7, 0x00, 0, 0, 0, 0]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x18, 0x00, 0x00, 0x00, 0x00]);
    code.bind("clear-tid-state")?;
    code.bytes(&[
        0x65, 0x48, 0xc7, 0x04, 0x25, 0xa8, 0x00, 0x00, 0x00, 0, 0, 0, 0,
    ]);
    code.bytes(&[0x65, 0xc6, 0x04, 0x25, 0x00, 0x00, 0x00, 0x00, 0x08]);
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00]);
    code.bytes(&[0x0f, 0x01, 0xf8, 0xc3]); // restore kernel GS base; return continuation

    code.bind("clear-tid-recovery")?;
    code.bytes(&[0x65, 0x4c, 0x89, 0x14, 0x25, 0x60, 0x00, 0x00, 0x00]);
    code.jump("fail");
    code.bind("robust-copy-recovery")?;
    code.bytes(&[0x65, 0x4c, 0x89, 0x14, 0x25, 0x60, 0x00, 0x00, 0x00]);
    code.bind("robust-invalid")?;
    code.bytes(&[0x48, 0x83, 0xc4, 0x18]);
    code.jump("fail");

    code.bind("fail")?;
    code.bytes(&[0xfa, 0xf4, 0xeb, 0xfd]);
    code.bind("uts-data")?;
    code.bytes(&initial_utsname_bytes());
    code.bind("rlimit-data")?;
    code.bytes(&0x0080_0000_u64.to_le_bytes());
    code.bytes(&u64::MAX.to_le_bytes());
    code.bind("getrandom-extract-chacha20")?;
    code.bytes(&entropy_extract_chacha20_generator()?);
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
    let expected = generate_initial_boot_resource(
        &initial_user_elf_file()?,
        &qualified_x86_64_static_user_image_fixture(),
    )?;
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
    code.bytes(&[0x41, 0x55]); // preserve r13
    code.bytes(&[0x49, 0x89, 0xcc]); // retain optional excluded base
    code.bytes(&[0x4d, 0x89, 0xc5]); // retain optional excluded end
    code.bytes(&[0x4d, 0x85, 0xe4]);
    code.jump_if(0x85, "exclusion-present");
    code.bytes(&[0x4d, 0x85, 0xed]); // absent base requires absent end
    code.jump_if(0x85, "fail");
    code.jump("count-admitted");
    code.bind("exclusion-present")?;
    code.bytes(&[0x4d, 0x85, 0xed]);
    code.jump_if(0x84, "fail");
    code.bytes(&[0x4d, 0x39, 0xec]); // excluded end > excluded base
    code.jump_if(0x83, "fail");
    code.bytes(&[0x41, 0xf7, 0xc4, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x41, 0xf7, 0xc5, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x83, 0xfa, 0x01]); // exclusion admits one or two frames
    code.jump_if(0x84, "count-admitted");
    code.bytes(&[0x83, 0xfa, 0x02]);
    code.jump_if(0x85, "fail");
    code.bind("count-admitted")?;
    code.bytes(&[0x83, 0xfa, 0x01]); // frame count == 1
    code.jump_if(0x84, "count-ready");
    code.bytes(&[0x83, 0xfa, 0x02]); // frame count == 2
    code.jump_if(0x84, "count-ready");
    code.bytes(&[0x83, 0xfa, 0x03]); // frame count == 3
    code.jump_if(0x84, "count-ready");
    code.bytes(&[0x81, 0xfa, 0xf7, 0x01, 0x00, 0x00]); // frame count == 503
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
    code.bytes(&[0x4d, 0x39, 0xe2]); // extent end <= excluded base
    code.jump_if(0x86, "extent-ready");
    code.bytes(&[0x4c, 0x39, 0xe8]); // candidate >= excluded end
    code.jump_if(0x83, "extent-ready");
    code.bytes(&[0x4c, 0x89, 0xe8]); // advance candidate past exclusion
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
    code.bytes(&[0x41, 0x5d, 0x41, 0x5c, 0xc3]); // restore r13/r12 and return physical base

    code.bind("zero-length")?;
    code.bytes(&[0x41, 0x83, 0xfa, 0x01]);
    code.jump_if(0x84, "fail");

    code.bind("next")?;
    code.bytes(&[0x48, 0x83, 0xc2, 0x14]);
    code.bytes(&[0xff, 0xc9]);
    code.jump_if(0x85, "loop");

    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0x41, 0x5d, 0x41, 0x5c, 0xc3]);
    code.finish()
}

fn emit_static_image_resource_validation(
    code: &mut X86FunctionEncoder,
) -> Result<(), CompileError> {
    code.bytes(&[0x48, 0x85, 0xff]); // payload base
    code.jump_if(0x84, "fail");
    code.bytes(&[0x48, 0x85, 0xf6]); // metadata base
    code.jump_if(0x84, "fail");
    code.bytes(&[0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0xf7, 0xc6, 0xff, 0x0f, 0x00, 0x00]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x81, 0xfa, 0xf7, 0x01, 0x00, 0x00]); // exact 503 leaves
    code.jump_if(0x85, "fail");
    code.bytes(&[0x49, 0x89, 0xf8]); // payload end
    code.bytes(&[0x49, 0x81, 0xc0, 0x00, 0x70, 0x1f, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x49, 0x81, 0xf8, 0x00, 0x00, 0x00, 0x40]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x49, 0x89, 0xf1]); // metadata end
    code.bytes(&[0x49, 0x81, 0xc1, 0x00, 0x10, 0x00, 0x00]);
    code.jump_if(0x82, "fail");
    code.bytes(&[0x49, 0x81, 0xf9, 0x00, 0x00, 0x00, 0x40]);
    code.jump_if(0x87, "fail");
    code.bytes(&[0x49, 0x39, 0xf9]); // metadata end <= payload base
    code.jump_if(0x86, "resources-disjoint");
    code.bytes(&[0x4c, 0x39, 0xc6]); // metadata base >= payload end
    code.jump_if(0x82, "fail");
    code.bind("resources-disjoint")
}

fn emit_static_image_leaf_verification(
    code: &mut X86FunctionEncoder,
    loop_label: &'static str,
) -> Result<(), CompileError> {
    code.bytes(&[0x49, 0x89, 0xfa]); // payload cursor
    code.bytes(&[0x48, 0x89, 0xf7]); // metadata cursor
    code.bytes(&[0x49, 0xb9]);
    code.bytes(&0x8000_0000_0000_0007_u64.to_le_bytes()); // present/user/write/NX
    code.bytes(&[0xb9, 0xf7, 0x01, 0x00, 0x00]);
    code.bind(loop_label)?;
    code.bytes(&[0x4c, 0x89, 0xd0]);
    code.bytes(&[0x4c, 0x09, 0xc8]);
    code.bytes(&[0x48, 0x39, 0x07]);
    code.jump_if(0x85, "fail");
    code.bytes(&[0x48, 0x83, 0xc7, 0x08]);
    code.bytes(&[0x49, 0x81, 0xc2, 0x00, 0x10, 0x00, 0x00]);
    code.bytes(&[0xff, 0xc9]);
    code.jump_if(0x85, loop_label);
    code.bytes(&[0x31, 0xc0]);
    code.bytes(&[0xb9, 0x09, 0x00, 0x00, 0x00]);
    code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]); // nine entries remain absent
    code.jump_if(0x85, "fail");
    Ok(())
}

fn static_image_prepare_initial_leaves() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    emit_static_image_resource_validation(&mut code)?;
    code.bytes(&[0x49, 0x89, 0xfa]); // payload cursor
    code.bytes(&[0x49, 0x89, 0xf3]); // retain metadata base
    code.bytes(&[0x48, 0x89, 0xf7]);
    code.bytes(&[0x31, 0xc0]);
    code.bytes(&[0xb9, 0x00, 0x02, 0x00, 0x00]);
    code.bytes(&[0xfc, 0xf3, 0x48, 0xab]); // zero all 512 entries
    code.bytes(&[0x49, 0xb9]);
    code.bytes(&0x8000_0000_0000_0007_u64.to_le_bytes());
    code.bytes(&[0x4c, 0x89, 0xdf]);
    code.bytes(&[0xb9, 0xf7, 0x01, 0x00, 0x00]);
    code.bind("construct-leaves")?;
    code.bytes(&[0x4c, 0x89, 0xd0]);
    code.bytes(&[0x4c, 0x09, 0xc8]);
    code.bytes(&[0x48, 0xab]);
    code.bytes(&[0x49, 0x81, 0xc2, 0x00, 0x10, 0x00, 0x00]);
    code.bytes(&[0xff, 0xc9]);
    code.jump_if(0x85, "construct-leaves");
    code.bytes(&[0x0f, 0xae, 0xf0]); // publish inactive leaves
    code.bytes(&[0x49, 0x81, 0xea, 0x00, 0x70, 0x1f, 0x00]);
    code.bytes(&[0x4c, 0x89, 0xd7]); // restore payload base
    code.bytes(&[0x4c, 0x89, 0xde]); // restore metadata base
    emit_static_image_leaf_verification(&mut code, "verify-leaves")?;
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn static_image_zero_payload() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    emit_static_image_resource_validation(&mut code)?;
    emit_static_image_leaf_verification(&mut code, "verify-leaves-before-zero")?;
    code.bytes(&[0x49, 0x81, 0xea, 0x00, 0x70, 0x1f, 0x00]); // payload base
    code.bytes(&[0x4c, 0x89, 0xd7]);
    code.bytes(&[0x31, 0xc0]);
    code.bytes(&[0xb9, 0x00, 0xee, 0x03, 0x00]); // 503 * 4096 / 8 qwords
    code.bytes(&[0xfc, 0xf3, 0x48, 0xab]); // zero complete payload
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x4c, 0x89, 0xd7]);
    code.bytes(&[0x31, 0xc0]);
    code.bytes(&[0xb9, 0x00, 0xee, 0x03, 0x00]);
    code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]); // verify complete payload
    code.jump_if(0x85, "fail");
    code.bytes(&[0x4c, 0x89, 0xd7]);
    emit_static_image_leaf_verification(&mut code, "verify-leaves-after-zero")?;
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
    code.bind("fail")?;
    code.bytes(&[0x31, 0xc0, 0xc3]);
    code.finish()
}

fn static_image_revoke_initial_leaves() -> Result<Vec<u8>, CompileError> {
    let mut code = X86FunctionEncoder::default();
    emit_static_image_resource_validation(&mut code)?;
    emit_static_image_leaf_verification(&mut code, "verify-leaves")?;
    code.bytes(&[0x48, 0x89, 0xf7]);
    code.bytes(&[0x31, 0xc0]);
    code.bytes(&[0xb9, 0x00, 0x02, 0x00, 0x00]);
    code.bytes(&[0xfc, 0xf3, 0x48, 0xab]); // revoke all 512 entries
    code.bytes(&[0x0f, 0xae, 0xf0]);
    code.bytes(&[0x48, 0x89, 0xf7]);
    code.bytes(&[0x31, 0xc0]);
    code.bytes(&[0xb9, 0x00, 0x02, 0x00, 0x00]);
    code.bytes(&[0xfc, 0xf3, 0x48, 0xaf]); // verify complete revocation
    code.jump_if(0x85, "fail");
    code.bytes(&[0xb8, 0x01, 0x00, 0x00, 0x00, 0xc3]);
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

    fn call(&mut self, label: &'static str) {
        self.bytes.push(0xe8);
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
    #[cfg(all(target_arch = "x86_64", target_family = "unix"))]
    use std::process::Command;

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
            (
                X86_SYSTEMS_PCI_CONFIGURATION_READ_DWORD_SYMBOL,
                &[
                    0x66, 0xba, 0xf8, 0x0c, 0x89, 0xf8, 0xef, 0x66, 0xba, 0xfc, 0x0c, 0xed, 0x39,
                    0xf0, 0x0f, 0x94, 0xc0, 0x0f, 0xb6, 0xc0, 0xc3,
                ],
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
        for (name, read) in [
            (X86_SYSTEMS_PCI_REGION_READ_SYMBOL, true),
            (X86_SYSTEMS_PCI_REGION_READ_EVENTUALLY_SYMBOL, true),
            (X86_SYSTEMS_PCI_REGION_WRITE_SYMBOL, false),
        ] {
            let symbol = file.symbol_by_name(name).unwrap();
            let section = file
                .section_by_index(symbol.section_index().unwrap())
                .unwrap();
            let data = section.data().unwrap();
            let start = usize::try_from(symbol.address()).unwrap();
            let end = start + usize::try_from(symbol.size()).unwrap();
            let bytes = &data[start..end];
            assert_eq!(
                bytes
                    .windows(5)
                    .filter(|window| *window == [0x66, 0xba, 0xf8, 0x0c, 0xef])
                    .count(),
                4,
                "gateway access must select BAR, offset, width, and data fields"
            );
            assert_eq!(
                bytes
                    .windows(4)
                    .filter(|window| *window == [0x66, 0xba, 0xfc, 0x0c])
                    .count(),
                4,
                "gateway access must address BAR, offset, width, and data fields"
            );
            if read {
                assert!(
                    bytes
                        .windows(4)
                        .any(|window| window == [0xed, 0x44, 0x39, 0xc0])
                );
                assert!(
                    bytes
                        .windows(4)
                        .any(|window| window == [0xec, 0x44, 0x38, 0xc0])
                );
                assert!(
                    bytes
                        .windows(6)
                        .any(|window| window == [0x66, 0xed, 0x66, 0x44, 0x39, 0xc0])
                );
            } else {
                assert!(
                    bytes
                        .windows(4)
                        .any(|window| window == [0x44, 0x89, 0xc0, 0xef])
                );
                assert!(
                    bytes
                        .windows(4)
                        .any(|window| window == [0x44, 0x89, 0xc0, 0xee])
                );
                assert!(
                    bytes
                        .windows(5)
                        .any(|window| window == [0x44, 0x89, 0xc0, 0x66, 0xef])
                );
            }
        }
        let eventual = file
            .symbol_by_name(X86_SYSTEMS_PCI_REGION_READ_EVENTUALLY_SYMBOL)
            .unwrap();
        let section = file
            .section_by_index(eventual.section_index().unwrap())
            .unwrap();
        let data = section.data().unwrap();
        let start = usize::try_from(eventual.address()).unwrap();
        let end = start + usize::try_from(eventual.size()).unwrap();
        let bytes = &data[start..end];
        assert_eq!(
            bytes
                .windows(6)
                .filter(|window| *window == [0x41, 0xbb, 0x40, 0x42, 0x0f, 0x00])
                .count(),
            1,
            "eventual read must install the exact 1,000,000-attempt bound"
        );
        assert_eq!(
            bytes
                .windows(3)
                .filter(|window| *window == [0x41, 0xff, 0xcb])
                .count(),
            3,
            "each width-specific eventual-read path must consume one attempt per mismatch"
        );
        let update = file
            .symbol_by_name(X86_SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD_SYMBOL)
            .unwrap();
        let section = file
            .section_by_index(update.section_index().unwrap())
            .unwrap();
        let data = section.data().unwrap();
        let start = usize::try_from(update.address()).unwrap();
        let end = start + usize::try_from(update.size()).unwrap();
        let bytes = &data[start..end];
        assert_eq!(
            bytes
                .windows(5)
                .filter(|window| *window == [0x66, 0xba, 0xf8, 0x0c, 0xef])
                .count(),
            1,
            "masked word update must select configuration address once"
        );
        assert_eq!(
            bytes
                .windows(2)
                .filter(|window| *window == [0x66, 0xed])
                .count(),
            2,
            "masked word update must read pre-state and post-state exactly"
        );
        assert_eq!(
            bytes
                .windows(2)
                .filter(|window| *window == [0x66, 0xef])
                .count(),
            1,
            "masked word update must write exactly one command word"
        );
        assert!(
            bytes
                .windows(4)
                .any(|window| window == [0x66, 0x44, 0x01, 0xc2])
        );
        let dma_create = file
            .symbol_by_name(X86_SYSTEMS_DMA_BUFFER_CREATE_SYMBOL)
            .unwrap();
        let section = file
            .section_by_index(dma_create.section_index().unwrap())
            .unwrap();
        let data = section.data().unwrap();
        let start = usize::try_from(dma_create.address()).unwrap();
        let end = start + usize::try_from(dma_create.size()).unwrap();
        let bytes = &data[start..end];
        assert!(bytes.windows(3).any(|window| window == [0x83, 0xff, 0x01]));
        assert!(bytes.windows(3).any(|window| window == [0x83, 0xff, 0x02]));
        assert_eq!(bytes.iter().filter(|byte| **byte == 0xc3).count(), 2);
        assert!(!bytes.iter().any(|byte| matches!(*byte, 0xec..=0xef)));
        let bind = file
            .symbol_by_name(X86_SYSTEMS_DMA_BUFFER_BIND_REFERENCE_SYMBOL)
            .unwrap();
        let start = usize::try_from(bind.address()).unwrap();
        let end = start + usize::try_from(bind.size()).unwrap();
        let bytes = &data[start..end];
        assert!(
            bytes
                .windows(4)
                .any(|window| window == [0x48, 0x89, 0x14, 0x37])
        );
        assert!(
            bytes
                .windows(7)
                .any(|window| window == [0x48, 0x81, 0xfe, 0xf8, 0x0f, 0x00, 0x00])
        );
        let store = file
            .symbol_by_name(X86_SYSTEMS_DMA_BUFFER_STORE_INTEGER_SYMBOL)
            .unwrap();
        let start = usize::try_from(store.address()).unwrap();
        let end = start + usize::try_from(store.size()).unwrap();
        let bytes = &data[start..end];
        assert!(
            bytes
                .windows(4)
                .any(|window| window == [0x66, 0x89, 0x0c, 0x37])
        );
        assert!(bytes.windows(3).any(|window| window == [0x89, 0x0c, 0x37]));
        let prepare = file
            .symbol_by_name(X86_SYSTEMS_DMA_BUFFER_PREPARE_DEVICE_SYMBOL)
            .unwrap();
        let start = usize::try_from(prepare.address()).unwrap();
        let end = start + usize::try_from(prepare.size()).unwrap();
        let bytes = &data[start..end];
        assert!(
            bytes
                .windows(7)
                .any(|window| window == [0x48, 0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00])
        );
        assert!(bytes.windows(3).any(|window| window == [0x83, 0xfe, 0x02]));
        assert!(bytes.windows(3).any(|window| window == [0x83, 0xfe, 0x01]));
        assert!(
            !bytes
                .windows(4)
                .any(|window| window == [0x48, 0x89, 0x14, 0x37])
        );
        let complete = file
            .symbol_by_name(X86_SYSTEMS_DMA_BUFFER_COMPLETE_CPU_SYMBOL)
            .unwrap();
        let start = usize::try_from(complete.address()).unwrap();
        let end = start + usize::try_from(complete.size()).unwrap();
        let bytes = &data[start..end];
        assert!(
            bytes
                .windows(7)
                .any(|window| window == [0x48, 0xf7, 0xc7, 0xff, 0x0f, 0x00, 0x00])
        );
        assert!(bytes.windows(3).any(|window| window == [0x83, 0xfe, 0x01]));
        assert!(
            !bytes
                .windows(4)
                .any(|window| window == [0x48, 0x89, 0x14, 0x37])
        );
        let load = file
            .symbol_by_name(X86_SYSTEMS_DMA_BUFFER_LOAD_INTEGER_SYMBOL)
            .unwrap();
        let start = usize::try_from(load.address()).unwrap();
        let end = start + usize::try_from(load.size()).unwrap();
        let bytes = &data[start..end];
        assert!(
            bytes
                .windows(6)
                .any(|window| window == [0x0f, 0xb7, 0x04, 0x37, 0x39, 0xc8])
        );
        assert!(
            bytes
                .windows(5)
                .any(|window| window == [0x8b, 0x04, 0x37, 0x39, 0xc8])
        );
        let capture = file
            .symbol_by_name(X86_SYSTEMS_ENTROPY_SAMPLE_CAPTURE_SYMBOL)
            .unwrap();
        let start = usize::try_from(capture.address()).unwrap();
        let end = start + usize::try_from(capture.size()).unwrap();
        let bytes = &data[start..end];
        for validation in [
            &[0x83, 0xfa, 0x01][..],
            &[0x83, 0xf9, 0x02],
            &[0x41, 0x83, 0xf8, 0x20],
            &[0x45, 0x85, 0xc9],
        ] {
            assert!(
                bytes
                    .windows(validation.len())
                    .any(|window| window == validation)
            );
        }
        for (load, store, verify) in [
            (
                &[0x48, 0x8b, 0x07][..],
                &[0x48, 0x89, 0x06][..],
                &[0x48, 0x3b, 0x06][..],
            ),
            (
                &[0x48, 0x8b, 0x47, 8][..],
                &[0x48, 0x89, 0x46, 8][..],
                &[0x48, 0x3b, 0x46, 8][..],
            ),
            (
                &[0x48, 0x8b, 0x47, 16][..],
                &[0x48, 0x89, 0x46, 16][..],
                &[0x48, 0x3b, 0x46, 16][..],
            ),
            (
                &[0x48, 0x8b, 0x47, 24][..],
                &[0x48, 0x89, 0x46, 24][..],
                &[0x48, 0x3b, 0x46, 24][..],
            ),
        ] {
            for sequence in [load, store, verify] {
                assert_eq!(
                    bytes
                        .windows(sequence.len())
                        .filter(|window| *window == sequence)
                        .count(),
                    1,
                    "capture must load, copy, and verify each qword exactly once"
                );
            }
        }
        let mix = file
            .symbol_by_name(X86_SYSTEMS_ENTROPY_POOL_MIX_SYMBOL)
            .unwrap();
        let start = usize::try_from(mix.address()).unwrap();
        let end = start + usize::try_from(mix.size()).unwrap();
        let bytes = &data[start..end];
        assert!(bytes.len() > 5_000, "scalar BLAKE2s lowering is complete");
        assert!(
            bytes
                .windows(7)
                .any(|window| window == [0x48, 0x81, 0xec, 0x80, 0, 0, 0])
        );
        for rotation in [16_u8, 12, 8, 7] {
            assert_eq!(
                bytes
                    .windows(4)
                    .filter(|window| *window == [0x41, 0xc1, 0xc8, rotation])
                    .count()
                    + bytes
                        .windows(3)
                        .filter(|window| *window == [0xc1, 0xc9, rotation])
                        .count(),
                80,
                "each BLAKE2s G half-round uses the selected rotation"
            );
        }
        for value in [0x6b08_e647_u32, 0x7ef3_b6ff, 0x1cae_9f75] {
            assert!(bytes.windows(4).any(|window| window == value.to_le_bytes()));
        }
        for offset in [0_u8, 8, 16, 24] {
            assert!(
                bytes
                    .windows(4)
                    .any(|window| window == [0x49, 0x89, 0x42, offset]),
                "the consumed sample qword is wiped"
            );
            assert!(
                bytes
                    .windows(4)
                    .any(|window| window == [0x49, 0x0b, 0x42, offset]),
                "the consumed sample qword wipe is verified"
            );
        }
        #[cfg(all(target_arch = "x86_64", target_family = "unix"))]
        check_entropy_mix_against_independent_reference(bytes);
        let initialize = file
            .symbol_by_name(X86_SYSTEMS_ENTROPY_GENERATOR_INITIALIZE_SYMBOL)
            .unwrap();
        let start = usize::try_from(initialize.address()).unwrap();
        let end = start + usize::try_from(initialize.size()).unwrap();
        let bytes = &data[start..end];
        for offset in [0_u8, 8, 16, 24] {
            let load = if offset == 0 {
                vec![0x48, 0x8b, 0x07]
            } else {
                vec![0x48, 0x8b, 0x47, offset]
            };
            let publish = if offset == 0 {
                vec![0x48, 0x89, 0x06]
            } else {
                vec![0x48, 0x89, 0x46, offset]
            };
            let wipe = if offset == 0 {
                vec![0x48, 0x89, 0x07]
            } else {
                vec![0x48, 0x89, 0x47, offset]
            };
            for sequence in [load, publish, wipe] {
                assert!(
                    bytes
                        .windows(sequence.len())
                        .any(|window| window == sequence)
                );
            }
        }
        let extract = file
            .symbol_by_name(X86_SYSTEMS_ENTROPY_GENERATOR_EXTRACT_SYMBOL)
            .unwrap();
        let start = usize::try_from(extract.address()).unwrap();
        let end = start + usize::try_from(extract.size()).unwrap();
        let bytes = &data[start..end];
        assert!(bytes.len() > 4_000, "scalar ChaCha20 lowering is complete");
        for value in [0x6170_7865_u32, 0x3320_646e, 0x7962_2d32, 0x6b20_6574] {
            assert!(bytes.windows(4).any(|window| window == value.to_le_bytes()));
        }
        for rotation in [16_u8, 8] {
            assert_eq!(
                bytes
                    .windows(4)
                    .filter(|window| *window == [0x41, 0xc1, 0xc0, rotation])
                    .count(),
                80
            );
        }
        for rotation in [12_u8, 7] {
            assert_eq!(
                bytes
                    .windows(3)
                    .filter(|window| *window == [0xc1, 0xc1, rotation])
                    .count(),
                80
            );
        }
        for offset in [0_u8, 4, 8, 12, 16, 20, 24, 28] {
            assert!(
                bytes
                    .windows(4)
                    .any(|window| window == [0x41, 0x89, 0x42, offset]),
                "the successor key word is published"
            );
            assert!(
                bytes
                    .windows(4)
                    .any(|window| window == [0x41, 0x89, 0x43, offset]),
                "the opaque output word is published"
            );
        }
        #[cfg(all(target_arch = "x86_64", target_family = "unix"))]
        check_entropy_extract_against_independent_reference(bytes);
        let publish = file
            .symbol_by_name(X86_SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE_SYMBOL)
            .unwrap();
        let start = usize::try_from(publish.address()).unwrap();
        let end = start + usize::try_from(publish.size()).unwrap();
        let bytes = &data[start..end];
        assert!(
            bytes
                .windows(6)
                .any(|window| window == [0x81, 0xff, 0x84, 0x28, 0x00, 0x80])
        );
        assert!(bytes.windows(3).any(|window| window == [0x83, 0xfa, 0x20]));
        assert!(bytes.windows(3).any(|window| window == [0x83, 0xfa, 0x28]));
        assert!(bytes.windows(3).any(|window| window == [0x83, 0xfa, 0x30]));
        assert_eq!(
            bytes
                .windows(4)
                .filter(|window| *window == [0x66, 0xba, 0xfc, 0x0c])
                .count(),
            6
        );
        assert!(
            bytes
                .windows(5)
                .any(|window| window == [0x48, 0xc1, 0xe8, 0x20, 0xef])
        );
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
                .windows(6)
                .any(|bytes| bytes == [0x81, 0xfa, 0xf7, 0x01, 0x00, 0x00]),
            "selector must admit the exact 503-frame static ELF extent"
        );
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
        assert_eq!(&selector[..7], &[0x41, 0x54, 0x41, 0x55, 0x49, 0x89, 0xcc]);
        assert!(
            selector
                .windows(7)
                .any(|bytes| bytes == [0x41, 0xf7, 0xc5, 0xff, 0x0f, 0x00, 0x00]),
            "selector must validate the supplied complete exclusion end"
        );
        assert!(
            selector.windows(3).any(|bytes| bytes == [0x4d, 0x39, 0xe2])
                && selector.windows(3).any(|bytes| bytes == [0x4c, 0x39, 0xe8])
                && selector.windows(3).any(|bytes| bytes == [0x4c, 0x89, 0xe8]),
            "selector must detect overlap and advance beyond the complete live exclusion"
        );
        assert!(
            selector.windows(3).any(|bytes| bytes == [0x48, 0x39, 0xf8]),
            "selector must apply its provider-private allocation floor"
        );
        assert_eq!(
            selector
                .windows(5)
                .filter(|bytes| *bytes == [0x41, 0x5d, 0x41, 0x5c, 0xc3])
                .count(),
            2,
            "success and failure must both restore r13 and r12"
        );
        assert_eq!(
            &selector[selector.len() - 7..],
            &[0x31, 0xc0, 0x41, 0x5d, 0x41, 0x5c, 0xc3]
        );

        let prepare = file
            .symbol_by_name(X86_SYSTEMS_STATIC_IMAGE_PREPARE_SYMBOL)
            .unwrap();
        let start = usize::try_from(prepare.address()).unwrap();
        let end = start + usize::try_from(prepare.size()).unwrap();
        let prepare = &data[start..end];
        assert!(
            prepare
                .windows(10)
                .any(|bytes| bytes == [0x49, 0xb9, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]),
            "initial leaves must be present, user-readable, writable, and NX"
        );
        assert_eq!(
            prepare
                .windows(5)
                .filter(|bytes| *bytes == [0xb9, 0xf7, 0x01, 0x00, 0x00])
                .count(),
            2,
            "construction and verification must each cover exactly 503 leaves"
        );
        assert!(
            prepare
                .windows(5)
                .any(|bytes| bytes == [0xb9, 0x09, 0x00, 0x00, 0x00])
        );
        assert!(
            prepare
                .windows(5)
                .any(|bytes| bytes == [0xb9, 0x00, 0x02, 0x00, 0x00])
        );
        assert!(
            prepare
                .windows(4)
                .any(|bytes| bytes == [0xfc, 0xf3, 0x48, 0xab])
        );
        assert!(
            prepare
                .windows(4)
                .any(|bytes| bytes == [0xfc, 0xf3, 0x48, 0xaf])
        );
        assert_eq!(
            prepare
                .windows(3)
                .filter(|bytes| *bytes == [0x0f, 0xae, 0xf0])
                .count(),
            1
        );

        let zero_payload = file
            .symbol_by_name(X86_SYSTEMS_STATIC_IMAGE_ZERO_PAYLOAD_SYMBOL)
            .unwrap();
        let start = usize::try_from(zero_payload.address()).unwrap();
        let end = start + usize::try_from(zero_payload.size()).unwrap();
        let zero_payload = &data[start..end];
        assert_eq!(
            zero_payload
                .windows(5)
                .filter(|bytes| *bytes == [0xb9, 0x00, 0xee, 0x03, 0x00])
                .count(),
            2,
            "payload zeroing and readback must each cover 503 complete pages"
        );
        assert_eq!(
            zero_payload
                .windows(10)
                .filter(|bytes| {
                    *bytes == [0x49, 0xb9, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]
                })
                .count(),
            2,
            "the exact live leaves must be verified before and after zeroing"
        );
        assert_eq!(
            zero_payload
                .windows(4)
                .filter(|bytes| *bytes == [0xfc, 0xf3, 0x48, 0xab])
                .count(),
            1
        );
        assert_eq!(
            zero_payload
                .windows(4)
                .filter(|bytes| *bytes == [0xfc, 0xf3, 0x48, 0xaf])
                .count(),
            3
        );
        assert_eq!(
            zero_payload
                .windows(3)
                .filter(|bytes| *bytes == [0x0f, 0xae, 0xf0])
                .count(),
            1
        );

        let revoke = file
            .symbol_by_name(X86_SYSTEMS_STATIC_IMAGE_REVOKE_SYMBOL)
            .unwrap();
        let start = usize::try_from(revoke.address()).unwrap();
        let end = start + usize::try_from(revoke.size()).unwrap();
        let revoke = &data[start..end];
        assert!(
            revoke
                .windows(10)
                .any(|bytes| bytes == [0x49, 0xb9, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80]),
            "revocation must first verify the exact live leaf policy"
        );
        assert_eq!(
            revoke
                .windows(5)
                .filter(|bytes| *bytes == [0xb9, 0x00, 0x02, 0x00, 0x00])
                .count(),
            2,
            "revocation must clear and then verify all 512 entries"
        );
        assert!(
            revoke
                .windows(4)
                .any(|bytes| bytes == [0xfc, 0xf3, 0x48, 0xab])
        );
        assert_eq!(
            revoke
                .windows(4)
                .filter(|bytes| *bytes == [0xfc, 0xf3, 0x48, 0xaf])
                .count(),
            2
        );
        assert_eq!(
            revoke
                .windows(3)
                .filter(|bytes| *bytes == [0x0f, 0xae, 0xf0])
                .count(),
            1
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
                .any(|bytes| bytes == [0x80, 0xbf, 0x70, 0, 0, 0, 0x17]),
            "user preparation must require the bounded anonymous reservation"
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
                .any(|bytes| bytes == [0x48, 0x3d, 0x08, 0x11, 0, 0])
                && user_address_prepare
                    .windows(6)
                    .any(|bytes| bytes == [0x48, 0x3d, 0, 0x20, 0, 0])
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
                    .any(|bytes| bytes == [0x4d, 0x89, 0x9a, 0xe8, 0x1e, 0, 0]),
            "address-space preparation must validate and retain one ELF entry for AT_ENTRY"
        );
        let initial_stack = initial_user_stack_bytes();
        assert_eq!(
            u64::from_le_bytes(initial_stack[0..8].try_into().unwrap()),
            1
        );
        assert_eq!(
            u64::from_le_bytes(initial_stack[8..16].try_into().unwrap()),
            0x0060_1fc8
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
            0x0060_1fc8
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
        assert_eq!(
            u64::from_le_bytes(initial_stack[192..200].try_into().unwrap()),
            23
        );
        assert_eq!(initial_stack[200..208], [0; 8]);
        assert_eq!(
            u64::from_le_bytes(initial_stack[208..216].try_into().unwrap()),
            17
        );
        assert_eq!(
            u64::from_le_bytes(initial_stack[216..224].try_into().unwrap()),
            100
        );
        assert_eq!(
            u64::from_le_bytes(initial_stack[224..232].try_into().unwrap()),
            15
        );
        assert_eq!(
            u64::from_le_bytes(initial_stack[232..240].try_into().unwrap()),
            0x0060_1fc0
        );
        assert_eq!(
            u64::from_le_bytes(initial_stack[240..248].try_into().unwrap()),
            25
        );
        assert_eq!(
            u64::from_le_bytes(initial_stack[248..256].try_into().unwrap()),
            0x0060_1fe0
        );
        assert_eq!(initial_stack[256..272], [0; 16]);
        assert_eq!(&initial_stack[272..279], b"x86_64\0");
        assert_eq!(&initial_stack[280..291], b"topal-probe");
        assert_eq!(initial_stack[291], 0);
        assert_eq!(initial_stack[304..320], [0; 16]);
        let auxv_random_delivery = symbol_bytes(
            &file,
            X86_SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM_SYMBOL,
        );
        assert!(
            auxv_random_delivery
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x89, 0x81, 0xe0, 0x1f, 0, 0])
                && auxv_random_delivery
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0x89, 0x81, 0xe8, 0x1f, 0, 0])
                && auxv_random_delivery
                    .windows(7)
                    .any(|bytes| bytes == [0xc6, 0x87, 0xd0, 0, 0, 0, 1]),
            "initial AT_RANDOM delivery must copy both selected qwords and publish entry readiness"
        );
        let user_enter = symbol_bytes(&file, X86_SYSTEMS_USER_ENTER_SYMBOL);
        assert!(
            user_enter
                .windows(7)
                .any(|bytes| bytes == [0x80, 0xbf, 0xd0, 0, 0, 0, 1]),
            "initial user entry must reject an address space without AT_RANDOM delivery"
        );
        assert!(
            user_enter
                .windows(8)
                .any(|bytes| bytes == [0x48, 0x83, 0xbf, 0xd8, 0, 0, 0, 0]),
            "initial user entry must reject an address space without a live getrandom reservation"
        );
        let getrandom_reservation = symbol_bytes(
            &file,
            X86_SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM_SYMBOL,
        );
        assert!(
            getrandom_reservation
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x89, 0xb7, 0xd8, 0, 0, 0])
                && getrandom_reservation
                    .windows(7)
                    .any(|bytes| bytes == [0x80, 0xbf, 0xd0, 0, 0, 0, 1]),
            "initial getrandom reservation must retain only a private affine output reference"
        );
        let getrandom_binding =
            symbol_bytes(&file, X86_SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE_SYMBOL);
        assert!(
            getrandom_binding
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x89, 0x97, 0xd8, 0, 0, 0])
                && getrandom_binding
                    .windows(4)
                    .any(|bytes| bytes == [0xc6, 0x42, 0x10, 0x01]),
            "reusable getrandom binding must replace the reservation with sealed service state"
        );
        let user_syscall = symbol_bytes(&file, X86_SYSTEMS_USER_SYSCALL_SYMBOL);
        assert!(
            user_syscall
                .windows(5)
                .any(|bytes| bytes == [0x3d, 0x3e, 0x01, 0, 0])
                && user_syscall
                    .windows(9)
                    .any(|bytes| bytes == [0x65, 0x48, 0x8b, 0x0c, 0x25, 0xd8, 0, 0, 0])
                && user_syscall
                    .windows(4)
                    .any(|bytes| bytes == [0x48, 0xff, 0x41, 0x18]),
            "native syscall 318 must validate and count each reusable service delivery"
        );
        let entry_payload = initial_user_entry_payload().unwrap();
        assert_eq!(
            entry_payload
                .bytes
                .windows(7)
                .filter(|bytes| **bytes == [0xb8, 0x3e, 0x01, 0, 0, 0x0f, 0x05])
                .count(),
            9,
            "ring-3 must preserve the prior eight getrandom calls and add one maximum large request"
        );
        assert!(
            entry_payload
                .bytes
                .windows(9)
                .any(|bytes| bytes == [0x4c, 0x8b, 0x14, 0x24, 0x4c, 0x8b, 0x5c, 0x24, 8]),
            "ring-3 must read both returned qwords without comparing them with constants"
        );
        assert!(
            entry_payload
                .bytes
                .windows(10)
                .any(|bytes| bytes == [0x4c, 0x8b, 0x64, 0x24, 0x10, 0x4c, 0x8b, 0x6c, 0x24, 0x18]),
            "ring-3 must read the complete four-qword provider output without constraining it"
        );
        assert!(
            entry_payload
                .bytes
                .windows(8)
                .any(|bytes| bytes == [0x4c, 0x8b, 0x9c, 0x24, 0xf8, 0x00, 0x00, 0x00]),
            "ring-3 must read the last qword of the maximum 256-byte small request"
        );
        assert!(
            entry_payload
                .bytes
                .windows(8)
                .any(|bytes| bytes == [0x4c, 0x8b, 0x1c, 0x25, 0xf8, 0x11, 0x60, 0x00]),
            "ring-3 must read the last qword of the maximum 512-byte large request"
        );
        assert!(
            user_syscall
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x81, 0xfe, 0x00, 0x02, 0x00, 0x00])
                && user_syscall
                    .windows(4)
                    .any(|bytes| bytes == [0x48, 0x83, 0xfa, 0x20])
                && user_syscall
                    .windows(5)
                    .any(|bytes| bytes == [0xba, 0x20, 0x00, 0x00, 0x00]),
            "getrandom must cap complete requests at 512 bytes and split them into 32-byte outputs"
        );
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
                    .any(|bytes| bytes == 0x0060_3000_u64.to_le_bytes())
                && user_map
                    .windows(4)
                    .filter(|bytes| *bytes == [0x41, 0x0f, 0x01, 0x3a])
                    .count()
                    >= 2
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
        assert_eq!(code_image.len(), 4096 + 256 + entry_payload.bytes.len());
        assert!(
            entry_payload
                .bytes
                .windows(9)
                .any(|bytes| bytes == [0x48, 0x83, 0xbc, 0x24, 0xc0, 0, 0, 0, 23])
                && entry_payload
                    .bytes
                    .windows(9)
                    .any(|bytes| bytes == [0x48, 0x83, 0xbc, 0x24, 0xd0, 0, 0, 0, 17])
                && entry_payload
                    .bytes
                    .windows(9)
                    .any(|bytes| bytes == [0x48, 0x83, 0xbc, 0x24, 0xd8, 0, 0, 0, 100])
                && entry_payload
                    .bytes
                    .windows(12)
                    .any(|bytes| bytes
                        == [0x48, 0x81, 0xbc, 0x24, 0xe8, 0, 0, 0, 0xc0, 0x1f, 0x60, 0])
                && entry_payload
                    .bytes
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0x81, 0xc4, 0x50, 0x01, 0, 0]),
            "constructed user code must validate AT_SECURE, AT_CLKTCK, AT_PLATFORM, and the enlarged initial block"
        );
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
        assert!(code_image[256..4096 + 256].iter().all(|byte| *byte == 0x90));
        assert_eq!(&code_image[4096 + 256..], entry_payload.bytes.as_slice());
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
                    .any(|bytes| bytes == [0x48, 0x81, 0x7c, 0x24, 0x48, 0xc8, 0x1f, 0x60, 0x00])
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
                        == [0x48, 0x81, 0xbc, 0x24, 0x98, 0, 0, 0, 0x40, 0x30, 0x60, 0])
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
                    .any(|bytes| bytes == [0x48, 0x81, 0xc4, 0x50, 0x01, 0, 0]),
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
        let dup2_replace = user_syscall
            .windows(8)
            .rposition(|bytes| bytes == [0x65, 0x44, 0x88, 0x86, 0xe0, 0, 0, 0])
            .expect("dup2 must perform one dynamic destination replacement");
        let dup2_source_lookup = user_syscall[..dup2_replace]
            .windows(8)
            .rposition(|bytes| bytes == [0x65, 0x44, 0x8a, 0x87, 0xe0, 0, 0, 0])
            .expect("dup2 must look up the source description before replacement");
        let dup2_destination_bound = user_syscall[..dup2_replace]
            .windows(4)
            .rposition(|bytes| bytes == [0x48, 0x83, 0xfe, 0x02])
            .expect("dup2 must validate its destination before replacement");
        assert!(
            dup2_source_lookup < dup2_destination_bound && dup2_destination_bound < dup2_replace,
            "dup2 must validate the source and destination before atomic replacement"
        );
        assert!(
            entry_payload
                .bytes
                .windows(7)
                .filter(|bytes| *bytes == [0xb8, 0x03, 0, 0, 0, 0x0f, 0x05])
                .count()
                == 5
                && entry_payload
                    .bytes
                    .windows(7)
                    .filter(|bytes| *bytes == [0xb8, 0x20, 0, 0, 0, 0x0f, 0x05])
                    .count()
                    == 2
                && entry_payload
                    .bytes
                    .windows(7)
                    .filter(|bytes| *bytes == [0xb8, 0x21, 0, 0, 0, 0x0f, 0x05])
                    .count()
                    == 3
                && entry_payload
                    .bytes
                    .windows(4)
                    .filter(|bytes| *bytes == [0x31, 0xc0, 0x0f, 0x05])
                    .count()
                    >= 2
                && user_address_prepare
                    .windows(11)
                    .any(|bytes| bytes == [0x48, 0xc7, 0x87, 0xe0, 0, 0, 0, 1, 2, 3, 0])
                && [0x03_u8, 0x20, 0x21].into_iter().all(|number| {
                    user_syscall
                        .windows(3)
                        .any(|bytes| bytes == [0x83, 0xf8, number])
                })
                && user_syscall
                    .windows(8)
                    .filter(|bytes| *bytes == [0x65, 0x44, 0x8a, 0x87, 0xe0, 0, 0, 0])
                    .count()
                    >= 5
                && [1_u8, 2, 3].into_iter().all(|identity| {
                    user_syscall
                        .windows(4)
                        .any(|bytes| bytes == [0x41, 0x80, 0xf8, identity])
                })
                && user_syscall
                    .windows(8)
                    .any(|bytes| bytes == [0x65, 0xc6, 0x87, 0xe0, 0, 0, 0, 0])
                && [0xe0_u8, 0xe1, 0xe2].into_iter().all(|offset| {
                    user_syscall
                        .windows(9)
                        .any(|bytes| bytes == [0x65, 0x44, 0x88, 0x04, 0x25, offset, 0, 0, 0])
                })
                && user_syscall
                    .windows(8)
                    .any(|bytes| bytes == [0x65, 0x44, 0x88, 0x86, 0xe0, 0, 0, 0])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0xc7, 0xc0, 0xe8, 0xff, 0xff, 0xff])
                && [(0xe0_u8, 0_u8), (0xe1, 2), (0xe2, 3)]
                    .into_iter()
                    .all(|(offset, identity)| {
                        user_syscall.windows(9).any(|bytes| {
                            bytes == [0x65, 0x80, 0x3c, 0x25, offset, 0, 0, 0, identity]
                        })
                    })
                && user_syscall.windows(13).any(|bytes| {
                    bytes == [0x65, 0x48, 0xc7, 0x04, 0x25, 0xe0, 0, 0, 0, 0, 0, 0, 0]
                }),
            "duplication must map fixed description identities, preserve access modes and aliases, reject full or invalid requests, and verify complete terminal clearing"
        );
        let fcntl_first_command = user_syscall
            .windows(4)
            .rposition(|bytes| bytes == [0x48, 0x83, 0xfe, 0x01])
            .expect("fcntl must decode F_GETFD");
        let fcntl_lookup = user_syscall[..fcntl_first_command]
            .windows(8)
            .rposition(|bytes| bytes == [0x65, 0x44, 0x8a, 0x87, 0xe0, 0, 0, 0])
            .expect("fcntl must look up the mapped description");
        assert!(
            fcntl_lookup < fcntl_first_command
                && entry_payload
                    .bytes
                    .windows(7)
                    .filter(|bytes| *bytes == [0xb8, 0x48, 0, 0, 0, 0x0f, 0x05])
                    .count()
                    == 9
                && user_address_prepare
                    .windows(11)
                    .any(|bytes| bytes == [0x48, 0xc7, 0x87, 0xe8, 0, 0, 0, 0, 0, 0, 0])
                && user_syscall
                    .windows(3)
                    .any(|bytes| bytes == [0x83, 0xf8, 0x48])
                && [1_u8, 2, 3].into_iter().all(|command| {
                    user_syscall
                        .windows(4)
                        .any(|bytes| bytes == [0x48, 0x83, 0xfe, command])
                })
                && user_syscall
                    .windows(8)
                    .any(|bytes| bytes == [0x65, 0x0f, 0xb6, 0x87, 0xe8, 0, 0, 0])
                && user_syscall
                    .windows(3)
                    .any(|bytes| bytes == [0x80, 0xe2, 0x01])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x65, 0x88, 0x97, 0xe8, 0, 0, 0])
                && [0xe8_u8, 0xe9, 0xea].into_iter().all(|offset| {
                    user_syscall
                        .windows(9)
                        .any(|bytes| bytes == [0x65, 0xc6, 0x04, 0x25, offset, 0, 0, 0, 0])
                })
                && user_syscall
                    .windows(8)
                    .any(|bytes| bytes == [0x65, 0xc6, 0x86, 0xe8, 0, 0, 0, 0])
                && user_syscall.windows(13).any(|bytes| {
                    bytes == [0x65, 0x48, 0xc7, 0x04, 0x25, 0xe8, 0, 0, 0, 0, 0, 0, 0]
                }),
            "fcntl must separate slot flags from descriptions, decode only after lookup, clear duplication destinations, and erase terminal state"
        );
        let fstat_first_candidate_bound = user_syscall
            .windows(7)
            .rposition(|bytes| bytes == [0x48, 0x81, 0xfe, 0x00, 0x10, 0x60, 0x00])
            .expect("fstat must bound the writable stack range");
        let fstat_lookup = user_syscall[..fstat_first_candidate_bound]
            .windows(8)
            .rposition(|bytes| bytes == [0x65, 0x44, 0x8a, 0x87, 0xe0, 0, 0, 0])
            .expect("fstat must look up the mapped description");
        assert!(
            fstat_lookup < fstat_first_candidate_bound
                && entry_payload
                    .bytes
                    .windows(7)
                    .filter(|bytes| *bytes == [0xb8, 0x05, 0, 0, 0, 0x0f, 0x05])
                    .count()
                    == 3
                && user_syscall
                    .windows(3)
                    .any(|bytes| bytes == [0x83, 0xf8, 0x05])
                && user_syscall
                    .windows(10)
                    .any(|bytes| bytes == [0x49, 0x89, 0xf0, 0x49, 0x81, 0xc0, 0x90, 0, 0, 0])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x49, 0x81, 0xf8, 0x00, 0x20, 0x60, 0x00])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0x81, 0xfe, 0x00, 0x50, 0x60, 0x00])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x49, 0x81, 0xf8, 0x00, 0x60, 0x60, 0x00])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0x81, 0xec, 0x90, 0, 0, 0])
                && user_syscall
                    .windows(9)
                    .any(|bytes| bytes == [0x48, 0xc7, 0x44, 0x24, 0x18, 0x80, 0x21, 0, 0])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x49, 0x81, 0xfa, 0x90, 0, 0, 0])
                && user_syscall
                    .windows(4)
                    .any(|bytes| bytes == [0x43, 0x88, 0x04, 0x11])
                && user_syscall
                    .windows(8)
                    .any(|bytes| bytes == [0x65, 0x4c, 0x89, 0x14, 0x25, 0x60, 0, 0]),
            "fstat must resolve a live description before pointer access, stage the complete native layout, and fault-contain one full writable-range copy"
        );
        let lseek_whence = user_syscall
            .windows(4)
            .rposition(|bytes| bytes == [0x48, 0x83, 0xfa, 0x04])
            .expect("lseek must validate the maximum seek mode");
        let lseek_lookup = user_syscall[..lseek_whence]
            .windows(8)
            .rposition(|bytes| bytes == [0x65, 0x44, 0x8a, 0x87, 0xe0, 0, 0, 0])
            .expect("lseek must look up the mapped description");
        assert!(
            lseek_lookup < lseek_whence
                && user_syscall
                    .windows(3)
                    .any(|bytes| bytes == [0x83, 0xf8, 0x08])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0xc7, 0xc0, 0xe3, 0xff, 0xff, 0xff])
                && entry_payload
                    .bytes
                    .windows(5)
                    .filter(|bytes| *bytes == [0x6a, 0x08, 0x58, 0x0f, 0x05])
                    .count()
                    == 3
                && entry_payload
                    .bytes
                    .windows(5)
                    .any(|bytes| bytes == [0xff, 0xc3, 0x83, 0xfb, 0x02])
                && entry_payload
                    .bytes
                    .windows(2)
                    .any(|bytes| bytes == [0xb2, 0x05]),
            "lseek must resolve descriptors before modes, return ESPIPE for both live console directions, reject unsupported live modes, and retain closed-slot EBADF precedence"
        );
        let ioctl_lookup = user_syscall
            .windows(8)
            .rposition(|bytes| bytes == [0x65, 0x44, 0x8a, 0x87, 0xe0, 0, 0, 0])
            .expect("ioctl must look up the mapped description");
        let ioctl_request = user_syscall[ioctl_lookup..]
            .windows(6)
            .position(|bytes| bytes == [0x81, 0xfe, 0x01, 0x54, 0, 0])
            .map(|position| position + ioctl_lookup)
            .expect("ioctl must decode TCGETS");
        let ioctl_candidate = user_syscall[ioctl_request..]
            .windows(7)
            .position(|bytes| bytes == [0x48, 0x81, 0xfa, 0x00, 0x10, 0x60, 0x00])
            .map(|position| position + ioctl_request)
            .expect("ioctl must bound the writable candidate");
        assert!(
            ioctl_lookup < ioctl_request
                && ioctl_request < ioctl_candidate
                && entry_payload
                    .bytes
                    .windows(5)
                    .filter(|bytes| *bytes == [0xbe, 0x01, 0x54, 0, 0])
                    .count()
                    == 3
                && user_syscall
                    .windows(3)
                    .any(|bytes| bytes == [0x83, 0xf8, 0x10])
                && user_syscall
                    .windows(8)
                    .any(|bytes| bytes == [0x48, 0xc7, 0x44, 0x24, 0x08, 0xbf, 0x08, 0])
                && user_syscall
                    .windows(5)
                    .any(|bytes| bytes == [0xc6, 0x44, 0x24, 0x17, 0x01])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 0x24])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0xc7, 0xc0, 0xe7, 0xff, 0xff, 0xff]),
            "TCGETS must enforce descriptor/request/pointer precedence, stage the exact raw termios, copy 36 bytes under recovery, and return ENOTTY for unknown live-console requests"
        );
        let winsize_request = user_syscall[ioctl_request..]
            .windows(6)
            .position(|bytes| bytes == [0x81, 0xfe, 0x13, 0x54, 0, 0])
            .map(|position| position + ioctl_request)
            .expect("ioctl must decode TIOCGWINSZ");
        let winsize_candidate = user_syscall[winsize_request..]
            .windows(7)
            .position(|bytes| bytes == [0x48, 0x81, 0xfa, 0x00, 0x10, 0x60, 0x00])
            .map(|position| position + winsize_request)
            .expect("TIOCGWINSZ must bound the writable candidate");
        assert!(
            ioctl_lookup < ioctl_request
                && ioctl_request < winsize_request
                && winsize_request < winsize_candidate
                && entry_payload
                    .bytes
                    .windows(5)
                    .filter(|bytes| *bytes == [0xbe, 0x13, 0x54, 0, 0])
                    .count()
                    == 1
                && user_syscall
                    .windows(9)
                    .any(|bytes| bytes == [0x48, 0x83, 0xec, 0x08, 0x31, 0xc0, 0x48, 0x89, 0x04])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x49, 0xff, 0xc2, 0x49, 0x83, 0xfa, 0x08])
                && entry_payload
                    .bytes
                    .windows(5)
                    .any(|bytes| bytes == [0xba, 0x80, 0x51, 0x60, 0])
                && entry_payload
                    .bytes
                    .windows(8)
                    .any(|bytes| bytes == [0x83, 0xc2, 0x08, 0xff, 0xc3, 0x83, 0xfb, 0x02])
                && entry_payload
                    .bytes
                    .windows(3)
                    .any(|bytes| bytes == [0x83, 0xc6, 0x12]),
            "TIOCGWINSZ must follow live-descriptor and request decoding, publish the exact zero geometry under recovery, and prove input/output equality plus closed-slot precedence"
        );
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
        let user_publish = user_syscall
            .windows(4)
            .rposition(|bytes| bytes == [0x43, 0x88, 0x04, 0x10])
            .expect("bounded read must copy staged bytes to the user destination");
        let receive_stage = user_syscall
            .windows(4)
            .enumerate()
            .filter_map(|(position, bytes)| {
                (position < user_publish && bytes == [0x42, 0x88, 0x04, 0x14]).then_some(position)
            })
            .next_back()
            .expect("bounded read must stage received bytes privately");
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
        assert_eq!(
            entry_payload
                .bytes
                .windows(7)
                .filter(|bytes| *bytes == [0xb8, 0x9e, 0, 0, 0, 0x0f, 0x05])
                .count(),
            4,
            "constructed user code must set, get, clear, and re-read its FS base"
        );
        assert!(
            entry_payload
                .bytes
                .windows(8)
                .any(|bytes| bytes == [0x48, 0x81, 0x3c, 0x24, 0, 0x50, 0x60, 0])
                && entry_payload
                    .bytes
                    .windows(5)
                    .any(|bytes| bytes == [0x48, 0x83, 0x3c, 0x24, 0]),
            "constructed user code must validate both the selected and cleared FS values"
        );
        assert!(
            user_syscall
                .windows(5)
                .any(|bytes| bytes == [0x3d, 0x9e, 0, 0, 0])
                && user_syscall
                    .windows(6)
                    .any(|bytes| bytes == [0x81, 0xff, 0x02, 0x10, 0, 0])
                && user_syscall
                    .windows(6)
                    .any(|bytes| bytes == [0x81, 0xff, 0x03, 0x10, 0, 0])
                && user_syscall
                    .windows(9)
                    .any(|bytes| bytes == [0x65, 0x48, 0x89, 0x34, 0x25, 0xa0, 0, 0, 0])
                && user_syscall
                    .windows(9)
                    .any(|bytes| bytes == [0x65, 0x4c, 0x8b, 0x04, 0x25, 0xa0, 0, 0, 0])
                && user_syscall.windows(2).any(|bytes| bytes == [0x0f, 0x30])
                && user_syscall
                    .windows(3)
                    .any(|bytes| bytes == [0x4d, 0x89, 0x01]),
            "arch_prctl must retain and install FS state and publish GET_FS through fault-contained output"
        );
        assert!(
            entry_payload
                .bytes
                .windows(7)
                .any(|bytes| bytes == [0xb8, 0xda, 0, 0, 0, 0x0f, 0x05])
                && entry_payload
                    .bytes
                    .windows(11)
                    .any(|bytes| bytes == [0xc7, 0x04, 0x25, 0x08, 0x50, 0x60, 0, 1, 0, 0, 0])
                && user_syscall
                    .windows(5)
                    .any(|bytes| bytes == [0x3d, 0xda, 0, 0, 0])
                && user_syscall
                    .windows(9)
                    .any(|bytes| bytes == [0x65, 0x48, 0x89, 0x3c, 0x25, 0xa8, 0, 0, 0])
                && user_syscall
                    .windows(9)
                    .any(|bytes| bytes == [0x65, 0x4c, 0x8b, 0x04, 0x25, 0xa8, 0, 0, 0])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x41, 0xc7, 0x00, 0, 0, 0, 0])
                && user_syscall
                    .windows(13)
                    .any(|bytes| bytes == [0x65, 0x48, 0xc7, 0x04, 0x25, 0xa8, 0, 0, 0, 0, 0, 0, 0]),
            "set_tid_address must register one checked word, return TID one, and clear the word and state before terminal exit"
        );
        assert!(
            entry_payload
                .bytes
                .windows(7)
                .any(|bytes| bytes == [0xb8, 0x11, 0x01, 0, 0, 0x0f, 0x05])
                && entry_payload
                    .bytes
                    .windows(4)
                    .any(|bytes| bytes == [0x48, 0x89, 0x24, 0x24])
                && user_syscall
                    .windows(5)
                    .any(|bytes| bytes == [0x3d, 0x11, 0x01, 0, 0])
                && user_syscall
                    .windows(9)
                    .any(|bytes| bytes == [0x65, 0x48, 0x89, 0x3c, 0x25, 0xb0, 0, 0, 0])
                && user_syscall
                    .windows(9)
                    .any(|bytes| bytes == [0x65, 0x4c, 0x8b, 0x04, 0x25, 0xb0, 0, 0, 0])
                && user_syscall
                    .windows(4)
                    .any(|bytes| bytes == [0x49, 0x83, 0xfa, 0x18])
                && user_syscall
                    .windows(4)
                    .any(|bytes| bytes == [0x4c, 0x39, 0x04, 0x24])
                && user_syscall
                    .windows(6)
                    .any(|bytes| bytes == [0x48, 0x83, 0x7c, 0x24, 0x10, 0])
                && user_syscall
                    .windows(13)
                    .any(|bytes| bytes == [0x65, 0x48, 0xc7, 0x04, 0x25, 0xb0, 0, 0, 0, 0, 0, 0, 0]),
            "set_robust_list must register one complete native head and terminally validate its empty state before clearing"
        );
        assert!(
            entry_payload
                .bytes
                .windows(7)
                .any(|bytes| bytes == [0xb8, 0x61, 0, 0, 0, 0x0f, 0x05])
                && entry_payload
                    .bytes
                    .windows(7)
                    .any(|bytes| bytes == [0xb8, 0x2e, 0x01, 0, 0, 0x0f, 0x05])
                && entry_payload
                    .bytes
                    .windows(8)
                    .filter(|bytes| *bytes == [0x48, 0x81, 0x3c, 0x24, 0, 0, 0x80, 0])
                    .count()
                    == 2
                && user_syscall
                    .windows(3)
                    .any(|bytes| bytes == [0x83, 0xf8, 0x61])
                && user_syscall
                    .windows(5)
                    .any(|bytes| bytes == [0x3d, 0x2e, 0x01, 0, 0])
                && user_syscall
                    .windows(4)
                    .any(|bytes| bytes == [0x49, 0x83, 0xfa, 0x10])
                && user_syscall
                    .windows(4)
                    .any(|bytes| bytes == [0x43, 0x88, 0x04, 0x11])
                && user_syscall
                    .windows(8)
                    .any(|bytes| bytes == 0x0080_0000_u64.to_le_bytes())
                && user_syscall
                    .windows(8)
                    .any(|bytes| bytes == u64::MAX.to_le_bytes()),
            "getrlimit and query-only prlimit64 must share complete fault-contained publication of the initial stack limit"
        );
        assert_eq!(
            entry_payload
                .bytes
                .windows(7)
                .filter(|bytes| *bytes == [0xb8, 0x0e, 0, 0, 0, 0x0f, 0x05])
                .count(),
            4,
            "constructed user code must replace, query, unblock, and re-query the signal mask"
        );
        assert!(
            entry_payload
                .bytes
                .windows(4)
                .any(|bytes| bytes == 0x0004_0300_u32.to_le_bytes())
                && entry_payload
                    .bytes
                    .windows(9)
                    .filter(|bytes| {
                        *bytes == [0x48, 0x81, 0x7c, 0x24, 0x08, 0x00, 0x02, 0x00, 0x00]
                    })
                    .count()
                    == 2
                && user_syscall
                    .windows(3)
                    .any(|bytes| bytes == [0x83, 0xf8, 0x0e])
                && user_syscall
                    .windows(4)
                    .any(|bytes| bytes == [0x49, 0x83, 0xf8, 0x08])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x49, 0x81, 0xe0, 0xff, 0xfe, 0xfb, 0xff])
                && [0_u8, 1, 2].into_iter().all(|how| {
                    user_syscall
                        .windows(3)
                        .any(|bytes| bytes == [0x83, 0xff, how])
                })
                && user_syscall
                    .windows(3)
                    .any(|bytes| bytes == [0x4d, 0x09, 0xc8])
                && user_syscall
                    .windows(3)
                    .any(|bytes| bytes == [0x49, 0xf7, 0xd0])
                && user_syscall
                    .windows(3)
                    .any(|bytes| bytes == [0x4d, 0x21, 0xc8])
                && user_syscall
                    .windows(3)
                    .any(|bytes| bytes == [0x4c, 0x8b, 0x06])
                && user_syscall
                    .windows(3)
                    .any(|bytes| bytes == [0x4c, 0x89, 0x0a])
                && user_syscall
                    .windows(9)
                    .any(|bytes| bytes == [0x65, 0x4c, 0x89, 0x04, 0x25, 0xb8, 0, 0, 0])
                && user_syscall
                    .windows(13)
                    .any(|bytes| bytes == [0x65, 0x48, 0xc7, 0x04, 0x25, 0xb8, 0, 0, 0, 0, 0, 0, 0]),
            "rt_sigprocmask must stage native input, remove unblockable bits, update private state, publish the prior word, and clear state at exit"
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
        let reserve_anonymous = symbol_bytes(
            &file,
            X86_SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS_SYMBOL,
        );
        let release_anonymous = symbol_bytes(
            &file,
            X86_SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS_SYMBOL,
        );
        assert!(
            reserve_anonymous
                .windows(7)
                .any(|bytes| bytes == [0x80, 0xbf, 0x70, 0, 0, 0, 0x16])
                && reserve_anonymous
                    .windows(8)
                    .any(|bytes| bytes == [0x48, 0x83, 0xbf, 0xc0, 0, 0, 0, 0])
                && reserve_anonymous
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0x81, 0xc1, 0x30, 0x20, 0, 0])
                && reserve_anonymous
                    .windows(7)
                    .any(|bytes| bytes == [0xc6, 0x87, 0x70, 0, 0, 0, 0x17]),
            "anonymous reservation must consume one frame while retaining a non-present provider-selected PTE"
        );
        assert!(
            release_anonymous
                .windows(7)
                .any(|bytes| bytes == [0x80, 0xbf, 0x70, 0, 0, 0, 0x17])
                && release_anonymous
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0x81, 0xc1, 0x30, 0x20, 0, 0])
                && release_anonymous
                    .windows(11)
                    .any(|bytes| bytes == [0x48, 0xc7, 0x87, 0xc0, 0, 0, 0, 0, 0, 0, 0]),
            "anonymous release must require completed execution, an absent PTE, and return the exact frame"
        );
        let user_unmap = symbol_bytes(&file, X86_SYSTEMS_USER_ADDRESS_SPACE_UNMAP_SYMBOL);
        assert!(
            user_unmap
                .windows(8)
                .any(|bytes| bytes == [0x48, 0xc7, 0x41, 0x08, 0, 0, 0, 0])
                && user_unmap
                    .windows(4)
                    .filter(|bytes| *bytes == [0x41, 0x0f, 0x01, 0x3a])
                    .count()
                    >= 2
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
                    .windows(11)
                    .any(|bytes| bytes == [0x48, 0xc7, 0x87, 0xa0, 0, 0, 0, 0, 0, 0, 0])
                && user_unmap
                    .windows(8)
                    .any(|bytes| bytes == [0x48, 0x83, 0xbf, 0xa8, 0, 0, 0, 0])
                && user_unmap
                    .windows(11)
                    .any(|bytes| bytes == [0x48, 0xc7, 0x87, 0xa8, 0, 0, 0, 0, 0, 0, 0])
                && user_unmap
                    .windows(8)
                    .any(|bytes| bytes == [0x48, 0x83, 0xbf, 0xb0, 0, 0, 0, 0])
                && user_unmap
                    .windows(11)
                    .any(|bytes| bytes == [0x48, 0xc7, 0x87, 0xb0, 0, 0, 0, 0, 0, 0, 0])
                && user_unmap.windows(2).any(|bytes| bytes == [0x0f, 0x30])
                && user_unmap
                    .windows(8)
                    .any(|bytes| bytes == 0x0060_5000_u64.to_le_bytes())
                && user_unmap
                    .windows(8)
                    .any(|bytes| bytes == 0x0060_3000_u64.to_le_bytes())
                && !user_unmap
                    .windows(8)
                    .any(|bytes| bytes == 0x0060_2000_u64.to_le_bytes()),
            "user unmap must revoke, invalidate, clear break, FS, and mapping state, and return the frame"
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
        assert!(
            user_syscall
                .windows(3)
                .any(|bytes| bytes == [0x83, 0xf8, 0x09])
                && user_syscall
                    .windows(3)
                    .any(|bytes| bytes == [0x83, 0xf8, 0x0a])
                && user_syscall
                    .windows(3)
                    .any(|bytes| bytes == [0x83, 0xf8, 0x0b])
                && user_syscall
                    .windows(3)
                    .any(|bytes| bytes == [0x83, 0xf8, 0x1c])
                && user_syscall
                    .windows(3)
                    .any(|bytes| bytes == [0x83, 0xf8, 0x18])
                && user_syscall
                    .windows(5)
                    .any(|bytes| bytes == [0x3d, 0xe4, 0, 0, 0])
                && user_syscall
                    .windows(5)
                    .any(|bytes| bytes == [0x3d, 0xe5, 0, 0, 0])
                && user_syscall
                    .windows(9)
                    .any(|bytes| bytes == [0x48, 0xc7, 0x44, 0x24, 0x08, 0x0a, 0, 0, 0])
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x48, 0xc7, 0xc0, 0xda, 0xff, 0xff, 0xff])
                && user_syscall
                    .windows(8)
                    .any(|bytes| bytes == 0x0060_6000_u64.to_le_bytes())
                && user_syscall
                    .windows(8)
                    .any(|bytes| bytes == 0x8000_0000_0000_0007_u64.to_le_bytes())
                && user_syscall
                    .windows(8)
                    .any(|bytes| bytes == 0x8000_0000_0000_0005_u64.to_le_bytes())
                && user_syscall
                    .windows(7)
                    .any(|bytes| bytes == [0x49, 0xc7, 0x02, 0, 0, 0, 0])
                && user_syscall
                    .windows(9)
                    .any(|bytes| bytes == [0x65, 0x48, 0x83, 0x3c, 0x25, 0xc8, 0, 0, 0]),
            "the userspace adapter must retain bounded VM lifecycle, clock dispatch, and unsupported-syscall failure"
        );
        assert_eq!(
            entry_payload
                .bytes
                .windows(7)
                .filter(|bytes| *bytes == [0xb8, 0x18, 0, 0, 0, 0x0f, 0x05])
                .count(),
            1,
            "constructed code must prove one successful sole-runnable yield"
        );
        assert_eq!(
            entry_payload
                .bytes
                .windows(7)
                .filter(|bytes| *bytes == [0xb8, 0xe4, 0, 0, 0, 0x0f, 0x05])
                .count(),
            2,
            "constructed code must prove two monotonic clock observations"
        );
        assert_eq!(
            entry_payload
                .bytes
                .windows(7)
                .filter(|bytes| *bytes == [0xb8, 0xe5, 0, 0, 0, 0x0f, 0x05])
                .count(),
            2,
            "constructed code must prove exact and null monotonic clock-resolution queries"
        );
        assert_eq!(
            entry_payload
                .bytes
                .windows(7)
                .filter(|bytes| *bytes == [0xb8, 0xff, 0xff, 0xff, 0x7f, 0x0f, 0x05])
                .count(),
            1,
            "constructed code must prove one recoverable unsupported-syscall result"
        );
        assert_eq!(
            entry_payload
                .bytes
                .windows(7)
                .filter(|bytes| *bytes == [0xb8, 0x1c, 0, 0, 0, 0x0f, 0x05])
                .count(),
            2,
            "constructed code must prove inaccessible discard and rejected advice"
        );
        assert_eq!(
            entry_payload
                .bytes
                .windows(7)
                .filter(|bytes| *bytes == [0xb8, 0x0a, 0, 0, 0, 0x0f, 0x05])
                .count(),
            4,
            "constructed code must prove read, none, read-write, and rejected executable protection"
        );
        assert_eq!(
            entry_payload
                .bytes
                .windows(7)
                .filter(|bytes| *bytes == [0xb8, 0x09, 0, 0, 0, 0x0f, 0x05])
                .count(),
            3,
            "constructed code must prove first map, active-map rejection, and remap"
        );
        assert_eq!(
            entry_payload
                .bytes
                .windows(7)
                .filter(|bytes| *bytes == [0xb8, 0x0b, 0, 0, 0, 0x0f, 0x05])
                .count(),
            2,
            "constructed code must release both successful mappings"
        );
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
