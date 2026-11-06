//! Generated ELF object for the sealed initial x86-64 systems provider.

use std::collections::BTreeMap;

#[path = "systems_file_region.rs"]
pub(crate) mod file_region;
#[path = "systems_process_runtime.rs"]
mod process_runtime;
#[path = "systems_ram_page_region.rs"]
pub(crate) mod ram_page_region;
#[path = "systems_root_region.rs"]
pub(crate) mod root_region;
#[path = "systems_x86_function.rs"]
mod x86_function;
use process_runtime::file_descriptions as process_files;

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
    "topal.provider-object.x86_64-qemu-pc-q35/231";
pub const X86_SYSTEMS_PROVIDER_TEXT_SECTION: &str = ".text.topal.systems.provider";
pub const X86_SYSTEMS_BOOTSTRAP_STORAGE_SECTION: &str = ".bss.topal.bootstrap";
pub const X86_SYSTEMS_DMA_STORAGE_SECTION: &str = ".bss.topal.dma";
pub const X86_SYSTEMS_ENTROPY_STORAGE_SECTION: &str = ".bss.topal.entropy";
pub const X86_SYSTEMS_STATIC_PROCESS_STORAGE_SECTION: &str = ".bss.topal.process";
pub const X86_SYSTEMS_STATIC_PROCESS_BRK_RESERVE_SYMBOL: &str =
    "topal_x86_static_process_brk_reserve";
pub const X86_SYSTEMS_STATIC_PROCESS_EXEC_SNAPSHOT_SYMBOL: &str =
    "topal_x86_static_process_exec_snapshot";
pub const X86_SYSTEMS_STATIC_PROCESS_DESCRIPTOR_TABLE_SYMBOL: &str =
    "topal_x86_static_process_descriptor_table";
pub const X86_SYSTEMS_PROVIDER_NOTE_SECTION: &str = ".note.topal.provider";

pub(crate) fn volatile_file_profile(source_page: bool) -> crate::SystemsVolatileFileProfile {
    process_runtime::volatile_files::profile(source_page)
}

/// Verify the expanded process/child backing before it can reach a boot image.
pub(crate) fn inspect_linked_process_backing(bytes: &[u8]) -> Result<(), CompileError> {
    use object::{Object as _, ObjectSection as _, ObjectSymbol as _};
    let invalid = || CompileError::Tool("unqualified linked process/child backing".into());
    let file = object::File::parse(bytes).map_err(|_| invalid())?;
    if file.format() != BinaryFormat::Elf
        || file.architecture() != Architecture::X86_64
        || !file.is_little_endian()
        || file.kind() != object::ObjectKind::Executable
    {
        return Err(invalid());
    }
    let symbols = file
        .symbols()
        .filter(|s| {
            s.is_definition()
                && s.name().ok() == Some(X86_SYSTEMS_STATIC_PROCESS_DESCRIPTOR_TABLE_SYMBOL)
        })
        .collect::<Vec<_>>();
    if symbols.len() != 1 {
        return Err(invalid());
    }
    let symbol = &symbols[0];
    let end = symbol
        .address()
        .checked_add(symbol.size())
        .ok_or_else(invalid)?;
    if symbol.kind() != SymbolKind::Data
        || symbol.size() != process_runtime::children::STORAGE_BYTES
        || symbol.address() % 4096 != 0
        || symbol.address() < 0x200000
        || end > 0x40000000
        || (symbol.address() < 0x800000 && end > 0x400000)
    {
        return Err(invalid());
    }
    let section = file
        .section_by_index(symbol.section_index().ok_or_else(invalid)?)
        .map_err(|_| invalid())?;
    if !matches!(section.flags(),object::SectionFlags::Elf { sh_flags,.. } if sh_flags.0 & 7 == 3) {
        return Err(invalid());
    }
    Ok(())
}

pub const X86_SYSTEMS_BOOT_MEMORY_SYMBOL: &str = "topal_x86_systems_describe_boot_memory";
pub const X86_SYSTEMS_FRAME_ALLOCATE_SYMBOL: &str = "topal_x86_systems_allocate_physical_frames";
pub const X86_SYSTEMS_STATIC_IMAGE_PREPARE_SYMBOL: &str =
    "topal_x86_systems_static_image_prepare_initial_leaves";
pub const X86_SYSTEMS_STATIC_IMAGE_ZERO_PAYLOAD_SYMBOL: &str =
    "topal_x86_systems_static_image_zero_payload";
pub const X86_SYSTEMS_STATIC_IMAGE_ZERO_STACK_BACKING_SYMBOL: &str =
    "topal_x86_systems_static_image_zero_stack_backing";
pub const X86_SYSTEMS_STATIC_IMAGE_PUBLISH_STACK_LEAF_SYMBOL: &str =
    "topal_x86_systems_static_image_publish_stack_leaf";
pub const X86_SYSTEMS_STATIC_IMAGE_REVOKE_STACK_LEAF_SYMBOL: &str =
    "topal_x86_systems_static_image_revoke_stack_leaf";
pub const X86_SYSTEMS_STATIC_IMAGE_INITIALIZE_STACK_ENTRY_SYMBOL: &str =
    "topal_x86_systems_static_image_initialize_stack_entry";
pub const X86_SYSTEMS_STATIC_IMAGE_CLEAR_STACK_ENTRY_SYMBOL: &str =
    "topal_x86_systems_static_image_clear_stack_entry";
pub const X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_PAGESZ_SYMBOL: &str =
    "topal_x86_systems_static_image_add_stack_pagesz";
pub const X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_ENTRY_FACT_SYMBOL: &str =
    "topal_x86_systems_static_image_add_stack_entry_fact";
pub const X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_ENTRY_FACT_SYMBOL: &str =
    "topal_x86_systems_static_image_remove_stack_entry_fact";
pub const X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_PROGRAM_HEADERS_SYMBOL: &str =
    "topal_x86_systems_static_image_add_stack_program_headers";
pub const X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_PROGRAM_HEADERS_SYMBOL: &str =
    "topal_x86_systems_static_image_remove_stack_program_headers";
pub const X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_EXECUTABLE_NAME_SYMBOL: &str =
    "topal_x86_systems_static_image_add_stack_executable_name";
pub const X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_EXECUTABLE_NAME_SYMBOL: &str =
    "topal_x86_systems_static_image_remove_stack_executable_name";
pub const X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_CREDENTIALS_SYMBOL: &str =
    "topal_x86_systems_static_image_add_stack_credentials";
pub const X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_CREDENTIALS_SYMBOL: &str =
    "topal_x86_systems_static_image_remove_stack_credentials";
pub const X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_SECURE_MODE_SYMBOL: &str =
    "topal_x86_systems_static_image_add_stack_secure_mode";
pub const X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_SECURE_MODE_SYMBOL: &str =
    "topal_x86_systems_static_image_remove_stack_secure_mode";
pub const X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_CLOCK_TICK_RATE_SYMBOL: &str =
    "topal_x86_systems_static_image_add_stack_clock_tick_rate";
pub const X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_CLOCK_TICK_RATE_SYMBOL: &str =
    "topal_x86_systems_static_image_remove_stack_clock_tick_rate";
pub const X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_PLATFORM_SYMBOL: &str =
    "topal_x86_systems_static_image_add_stack_platform";
pub const X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_PLATFORM_SYMBOL: &str =
    "topal_x86_systems_static_image_remove_stack_platform";
pub const X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_HARDWARE_CAPABILITIES_SYMBOL: &str =
    "topal_x86_systems_static_image_add_stack_hardware_capabilities";
pub const X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_HARDWARE_CAPABILITIES_SYMBOL: &str =
    "topal_x86_systems_static_image_remove_stack_hardware_capabilities";
pub const X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_LOAD_METADATA_SYMBOL: &str =
    "topal_x86_systems_static_image_add_stack_load_metadata";
pub const X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_LOAD_METADATA_SYMBOL: &str =
    "topal_x86_systems_static_image_remove_stack_load_metadata";
pub const X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_STACK_RANDOM_SLOT_SYMBOL: &str =
    "topal_x86_systems_static_image_authorize_stack_random_slot";
pub const X86_SYSTEMS_STATIC_IMAGE_RETAIN_PREPARED_PROCESS_SYMBOL: &str =
    "topal_x86_systems_static_image_retain_prepared_process";
pub const X86_SYSTEMS_STATIC_IMAGE_QUALIFY_RETAINED_DISJOINT_WORK_SYMBOL: &str =
    "topal_x86_systems_static_image_qualify_retained_disjoint_work";
pub const X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_ENTROPY_HANDOFF_SYMBOL: &str =
    "topal_x86_systems_static_image_authorize_entropy_handoff";
pub const X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_ENTROPY_ACQUISITION_SYMBOL: &str =
    "topal_x86_systems_static_image_authorize_entropy_acquisition";
pub const X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_DEVICE_SYMBOL: &str =
    "topal_x86_systems_static_image_qualify_entropy_device";
pub const X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_TRANSPORT_SYMBOL: &str =
    "topal_x86_systems_static_image_qualify_entropy_transport";
pub const X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_NEGOTIATION_SYMBOL: &str =
    "topal_x86_systems_static_image_qualify_entropy_negotiation";
pub const X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_QUEUE_SYMBOL: &str =
    "topal_x86_systems_static_image_qualify_entropy_queue";
pub const X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_BUS_MASTER_SYMBOL: &str =
    "topal_x86_systems_static_image_qualify_entropy_bus_master";
pub const X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_DMA_BUFFERS_SYMBOL: &str =
    "topal_x86_systems_static_image_qualify_entropy_dma_buffers";
pub const X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_SPLIT_RING_SYMBOL: &str =
    "topal_x86_systems_static_image_qualify_entropy_split_ring";
pub const X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_DMA_PREPARATION_SYMBOL: &str =
    "topal_x86_systems_static_image_qualify_entropy_dma_preparation";
pub const X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_QUEUE_ADDRESSES_SYMBOL: &str =
    "topal_x86_systems_static_image_qualify_entropy_queue_addresses";
pub const X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_QUEUE_ACTIVATION_SYMBOL: &str =
    "topal_x86_systems_static_image_qualify_entropy_queue_activation";
pub const X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_REQUEST_COMPLETION_SYMBOL: &str =
    "topal_x86_systems_static_image_qualify_entropy_request_completion";
pub const X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_SAMPLE_CAPTURE_SYMBOL: &str =
    "topal_x86_systems_static_image_qualify_entropy_sample_capture";
pub const X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_SAMPLE_QUALITY_SYMBOL: &str =
    "topal_x86_systems_static_image_qualify_entropy_sample_quality";
pub const X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_POOL_LIFECYCLE_SYMBOL: &str =
    "topal_x86_systems_static_image_qualify_entropy_pool_lifecycle";
pub const X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_GENERATOR_LIFECYCLE_SYMBOL: &str =
    "topal_x86_systems_static_image_qualify_entropy_generator_lifecycle";
pub const X86_SYSTEMS_STATIC_IMAGE_DELIVER_ENTROPY_INITIAL_RANDOM_SYMBOL: &str =
    "topal_x86_systems_static_image_deliver_entropy_initial_random";
pub const X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTROPY_INITIAL_RANDOM_SYMBOL: &str =
    "topal_x86_systems_static_image_revoke_entropy_initial_random";
pub const X86_SYSTEMS_STATIC_IMAGE_PROCESS_ENTER_SYMBOL: &str =
    "topal_x86_systems_static_image_process_enter";
pub const X86_SYSTEMS_STATIC_IMAGE_PROCESS_RESTORE_ENTRY_SYMBOL: &str =
    "topal_x86_systems_static_image_process_restore_entry";
pub const X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTROPY_ACQUISITION_SYMBOL: &str =
    "topal_x86_systems_static_image_revoke_entropy_acquisition";
pub const X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTROPY_HANDOFF_SYMBOL: &str =
    "topal_x86_systems_static_image_revoke_entropy_handoff";
pub const X86_SYSTEMS_STATIC_IMAGE_RESTORE_PREPARED_PROCESS_SYMBOL: &str =
    "topal_x86_systems_static_image_restore_prepared_process";
pub const X86_SYSTEMS_STATIC_IMAGE_REVOKE_STACK_RANDOM_SLOT_SYMBOL: &str =
    "topal_x86_systems_static_image_revoke_stack_random_slot";
pub const X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_PAGESZ_SYMBOL: &str =
    "topal_x86_systems_static_image_remove_stack_pagesz";
pub const X86_SYSTEMS_STATIC_IMAGE_COPY_PAYLOAD_SYMBOL: &str =
    "topal_x86_systems_static_image_copy_payload";
pub const X86_SYSTEMS_STATIC_IMAGE_SEAL_PERMISSIONS_SYMBOL: &str =
    "topal_x86_systems_static_image_seal_permissions";
pub const X86_SYSTEMS_STATIC_IMAGE_ATTACH_ADDRESS_SPACE_SYMBOL: &str =
    "topal_x86_systems_static_image_attach_address_space";
pub const X86_SYSTEMS_STATIC_IMAGE_ACTIVATE_ADDRESS_SPACE_SYMBOL: &str =
    "topal_x86_systems_static_image_activate_address_space";
pub const X86_SYSTEMS_STATIC_IMAGE_RESTORE_ADDRESS_SPACE_SYMBOL: &str =
    "topal_x86_systems_static_image_restore_address_space";
pub const X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_ENTRY_SYMBOL: &str =
    "topal_x86_systems_static_image_authorize_entry";
pub const X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_STACK_SLOT_SYMBOL: &str =
    "topal_x86_systems_static_image_authorize_stack_slot";
pub const X86_SYSTEMS_STATIC_IMAGE_REVOKE_STACK_SLOT_SYMBOL: &str =
    "topal_x86_systems_static_image_revoke_stack_slot";
pub const X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTRY_SYMBOL: &str =
    "topal_x86_systems_static_image_revoke_entry";
pub const X86_SYSTEMS_STATIC_IMAGE_DETACH_ADDRESS_SPACE_SYMBOL: &str =
    "topal_x86_systems_static_image_detach_address_space";
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

include!("systems_provider_emit_devices.rs");

include!("systems_provider_emit_publication.rs");

include!("systems_provider_emit_user_memory.rs");

include!("systems_provider_emit_user_payload.rs");

include!("systems_provider_emit_user_context.rs");

include!("systems_provider_emit_user_syscalls.rs");

include!("systems_provider_emit_contexts_clocks.rs");

include!("systems_provider_emit_encoding.rs");

include!("systems_provider_emit_static_image_leaves.rs");

include!("systems_provider_emit_stack_headers.rs");

include!("systems_provider_emit_stack_credentials.rs");

include!("systems_provider_emit_stack_platform.rs");

include!("systems_provider_emit_entropy_authority.rs");

include!("systems_provider_emit_process_entry.rs");

include!("systems_provider_emit_image_mapping.rs");

include!("systems_provider_emit_translation.rs");

#[cfg(test)]
mod tests {

    include!("systems_provider_object_tests.rs");
}
