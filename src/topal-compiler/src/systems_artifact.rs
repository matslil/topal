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
    CompilerAtomicOrder, CompilerAtomicWordRequest, CompilerBootstrapStoragePlacement,
    CompilerBootstrapStorageRequest, CompilerCriticalDomain, CompilerDmaBufferIntegerLoadRequest,
    CompilerDmaBufferIntegerStoreRequest, CompilerDmaBufferPreparationRequest,
    CompilerDmaBufferReferenceRequest, CompilerDmaBufferRequest, CompilerDmaBufferRole,
    CompilerDmaDirection, CompilerEntropyGeneratorExtractRequest, CompilerEntropyGeneratorRequest,
    CompilerEntropyOutputDeliveryRequest, CompilerEntropyOutputReservationRequest,
    CompilerEntropyPoolCreditRequest, CompilerEntropyPoolMixRequest, CompilerEntropyPoolRequest,
    CompilerEntropySampleCaptureRequest, CompilerEntropySampleQualityRequest,
    CompilerEntropyUserRandomServiceRequest, CompilerEntropyUserRandomSmallRequest,
    CompilerKernelMappingRequest, CompilerPciConfigurationDwordRequest,
    CompilerPciConfigurationWordUpdateRequest, CompilerPciRegionDmaReferenceRequest,
    CompilerPciRegionRegisterDirection, CompilerPciRegionRegisterRequest,
    CompilerSystemsDisposition, CompilerSystemsOperation, CompilerSystemsProgram,
    CompilerSystemsTransition, CompilerTranslationEditKind, CompilerTranslationMappingRequest,
    CompilerTranslationUpdateRequest, SYSTEMS_ATOMIC_COMPARE_EXCHANGE, SYSTEMS_ATOMIC_END,
    SYSTEMS_ATOMIC_LOAD, SYSTEMS_ATOMIC_WORD_CREATE, SYSTEMS_BOOT_MEMORY_DESCRIBE,
    SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE, SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE,
    SYSTEMS_BOOTSTRAP_STORAGE_PROVISION, SYSTEMS_CONSOLE_WRITE, SYSTEMS_CRITICAL_ENTER,
    SYSTEMS_CRITICAL_RESTORE, SYSTEMS_DEADLINE_AFTER, SYSTEMS_DEADLINE_ARM,
    SYSTEMS_DEADLINE_COMPLETE, SYSTEMS_DEADLINE_WAIT, SYSTEMS_DEBUG_BREAK, SYSTEMS_FATAL,
    SYSTEMS_FRAMES_ALLOCATE, SYSTEMS_KERNEL_CONTEXT_CREATE, SYSTEMS_KERNEL_CONTEXT_RECLAIM,
    SYSTEMS_KERNEL_CONTEXT_RETIRE, SYSTEMS_KERNEL_CONTEXT_TRANSFER, SYSTEMS_KERNEL_MAP,
    SYSTEMS_KERNEL_MAPPING_LOAD_BYTE, SYSTEMS_KERNEL_MAPPING_STORE_BYTE, SYSTEMS_KERNEL_UNMAP,
    SYSTEMS_LOCAL_NOTIFICATION_COMPLETE, SYSTEMS_LOCAL_NOTIFICATION_SEND,
    SYSTEMS_LOCAL_NOTIFICATION_WAIT, SYSTEMS_MONOTONIC_CLOCK_NOW, SYSTEMS_RESUME_DEADLINE,
    SYSTEMS_RESUME_DEBUG_BREAK, SYSTEMS_RESUME_LOCAL_NOTIFICATION, SYSTEMS_TRANSLATION_ACTIVATE,
    SYSTEMS_TRANSLATION_BEGIN, SYSTEMS_TRANSLATION_COMMIT, SYSTEMS_TRANSLATION_EDIT_BEGIN,
    SYSTEMS_TRANSLATION_EDIT_COMMIT, SYSTEMS_TRANSLATION_EDIT_MAP, SYSTEMS_TRANSLATION_EDIT_UNMAP,
    SYSTEMS_USER_ADDRESS_SPACE_MAP, SYSTEMS_USER_ADDRESS_SPACE_PREPARE,
    SYSTEMS_USER_ADDRESS_SPACE_RECLAIM, SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS,
    SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS, SYSTEMS_USER_ADDRESS_SPACE_UNMAP,
    SYSTEMS_USER_ADDRESS_SPACE_VERIFY_MAPPING, SYSTEMS_USER_CONTEXT_COMPLETE,
    SYSTEMS_USER_CONTEXT_ENTER, SYSTEMS_USER_CONTEXT_PREPARE, SYSTEMS_USER_COPY_FROM,
    SYSTEMS_USER_COPY_TO, SYSTEMS_USER_MAPPING_LOAD_BYTE, SYSTEMS_USER_MAPPING_PERMIT_EXECUTE,
    SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA, SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY,
    SYSTEMS_USER_MAPPING_SEAL_IMMUTABLE_DATA, SYSTEMS_USER_MAPPING_STORE_BYTE,
    SYSTEMS_USER_SYSCALL_DISPATCH, model_systems_transitions,
};

#[cfg(test)]
use topal_language::compiler::{
    SYSTEMS_DMA_BUFFER_BIND_REFERENCE, SYSTEMS_DMA_BUFFER_COMPLETE_CPU, SYSTEMS_DMA_BUFFER_CREATE,
    SYSTEMS_DMA_BUFFER_PREPARE_DEVICE, SYSTEMS_DMA_BUFFER_STORE_INTEGER,
    SYSTEMS_ENTROPY_GENERATOR_EXTRACT, SYSTEMS_ENTROPY_GENERATOR_INITIALIZE,
    SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM,
    SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM, SYSTEMS_ENTROPY_POOL_CREATE,
    SYSTEMS_ENTROPY_POOL_CREDIT_CONTRIBUTION, SYSTEMS_ENTROPY_POOL_MIX_SAMPLE,
    SYSTEMS_ENTROPY_SAMPLE_ASSESS_QUALITY, SYSTEMS_ENTROPY_SAMPLE_CAPTURE_DMA,
    SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE, SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS,
    SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS, SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE,
    SYSTEMS_KERNEL_COMPLETION_CONSUME, SYSTEMS_KERNEL_COMPLETION_CREATE,
    SYSTEMS_KERNEL_COMPLETION_SIGNAL, SYSTEMS_KERNEL_CONTEXT_AWAIT_DEADLINE_PREEMPTION,
    SYSTEMS_KERNEL_CONTEXT_PREEMPT_CURRENT, SYSTEMS_KERNEL_CONTEXT_TAKE_PREEMPTED,
    SYSTEMS_KERNEL_CONTEXT_TRANSFER_UNTIL_DEADLINE, SYSTEMS_KERNEL_RUNNABLE_QUEUE_CONSUME,
    SYSTEMS_KERNEL_RUNNABLE_QUEUE_CREATE, SYSTEMS_KERNEL_RUNNABLE_QUEUE_DEQUEUE,
    SYSTEMS_KERNEL_RUNNABLE_QUEUE_ENQUEUE, SYSTEMS_PCI_CONFIGURATION_READ_DWORD,
    SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD, SYSTEMS_PCI_REGION_READ,
    SYSTEMS_PCI_REGION_READ_EVENTUALLY, SYSTEMS_PCI_REGION_WRITE,
    SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ACTIVATE_ADDRESS_SPACE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_CLOCK_TICK_RATE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_CREDENTIALS,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_ENTRY_FACT,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_EXECUTABLE_NAME,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_HARDWARE_CAPABILITIES,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_LOAD_METADATA,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PAGESZ,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PLATFORM,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PROGRAM_HEADERS,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_SECURE_MODE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ATTACH_ADDRESS_SPACE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTROPY_ACQUISITION,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTROPY_HANDOFF,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTRY,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_RANDOM_SLOT,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_SLOT,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_CLEAR_STACK_ENTRY,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_COPY_PAYLOAD,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_DELIVER_ENTROPY_INITIAL_RANDOM,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_DETACH_ADDRESS_SPACE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_INITIALIZE_STACK_ENTRY,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PREPARE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PUBLISH_STACK_LEAF,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_BUS_MASTER,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DEVICE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DMA_BUFFERS,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DMA_PREPARATION,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_GENERATOR_LIFECYCLE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_NEGOTIATION,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_POOL_LIFECYCLE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE_ACTIVATION,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE_ADDRESSES,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_REQUEST_COMPLETION,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SAMPLE_CAPTURE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SAMPLE_QUALITY,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SPLIT_RING,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_TRANSPORT,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_RETAINED_DISJOINT_WORK,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_HIERARCHY,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_PAYLOAD,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_CLOCK_TICK_RATE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_CREDENTIALS,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_ENTRY_FACT,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_EXECUTABLE_NAME,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_HARDWARE_CAPABILITIES,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_LOAD_METADATA,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PAGESZ,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PLATFORM,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PROGRAM_HEADERS,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_SECURE_MODE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESTORE_ADDRESS_SPACE,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESTORE_PREPARED_PROCESS,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RETAIN_PREPARED_PROCESS,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_ACQUISITION,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_HANDOFF,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_INITIAL_RANDOM,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTRY,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_METADATA,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_LEAF,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_RANDOM_SLOT,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_SLOT,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_SEAL_PERMISSIONS,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_PAYLOAD,
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_STACK_BACKING, SYSTEMS_STATIC_IMAGE_PROCESS_ENTER,
    SYSTEMS_STATIC_IMAGE_PROCESS_RESTORE_ENTRY, SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE,
    SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE, SYSTEMS_USER_TRANSFER_BEGIN,
    SYSTEMS_USER_TRANSFER_END,
};

use crate::artifact::sha256;
use crate::systems_provider_object::{
    INITIAL_USER_CODE_ADDRESS, INITIAL_USER_CODE_FILE_SIZE_ELF_OFFSET,
    INITIAL_USER_CODE_MEMORY_SIZE_ELF_OFFSET, INITIAL_USER_DATA_CAPACITY,
    INITIAL_USER_DATA_FILE_OFFSET, INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET,
    INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET, INITIAL_USER_ENTRY_ELF_OFFSET, initial_user_elf_file,
};
use crate::{
    CompileError, DigestEntry, LlvmTools, X86_SYSTEMS_ALLOCATABLE_FLOOR,
    X86_SYSTEMS_ATOMIC_COMPARE_EXCHANGE_SYMBOL, X86_SYSTEMS_ATOMIC_CREATE_SYMBOL,
    X86_SYSTEMS_ATOMIC_LOAD_SYMBOL, X86_SYSTEMS_BOOT_MEMORY_SYMBOL,
    X86_SYSTEMS_BOOTSTRAP_STORAGE_SECTION, X86_SYSTEMS_CONTEXT_AWAIT_DEADLINE_PREEMPTION_SYMBOL,
    X86_SYSTEMS_CONTEXT_CREATE_SYMBOL, X86_SYSTEMS_CONTEXT_PREEMPT_CURRENT_SYMBOL,
    X86_SYSTEMS_CONTEXT_RECLAIM_SYMBOL, X86_SYSTEMS_CONTEXT_RETIRE_SYMBOL,
    X86_SYSTEMS_CONTEXT_TRANSFER_SYMBOL, X86_SYSTEMS_CRITICAL_ENTER_SYMBOL,
    X86_SYSTEMS_CRITICAL_RESTORE_SYMBOL, X86_SYSTEMS_DEADLINE_AFTER_SYMBOL,
    X86_SYSTEMS_DEADLINE_ARM_SYMBOL, X86_SYSTEMS_DEADLINE_COMPLETE_SYMBOL,
    X86_SYSTEMS_DEADLINE_WAIT_SYMBOL, X86_SYSTEMS_DMA_BUFFER_BIND_REFERENCE_SYMBOL,
    X86_SYSTEMS_DMA_BUFFER_COMPLETE_CPU_SYMBOL, X86_SYSTEMS_DMA_BUFFER_CREATE_SYMBOL,
    X86_SYSTEMS_DMA_BUFFER_LOAD_INTEGER_SYMBOL, X86_SYSTEMS_DMA_BUFFER_PREPARE_DEVICE_SYMBOL,
    X86_SYSTEMS_DMA_BUFFER_STORE_INTEGER_SYMBOL, X86_SYSTEMS_DMA_ENTROPY_PAYLOAD_SYMBOL,
    X86_SYSTEMS_DMA_VIRTQUEUE_CONTROL_SYMBOL, X86_SYSTEMS_ENTROPY_GENERATOR_EXTRACT_SYMBOL,
    X86_SYSTEMS_ENTROPY_GENERATOR_INITIALIZE_SYMBOL, X86_SYSTEMS_ENTROPY_GENERATOR_KEY_SYMBOL,
    X86_SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM_SYMBOL,
    X86_SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM_SYMBOL,
    X86_SYSTEMS_ENTROPY_OUTPUT_SYMBOL, X86_SYSTEMS_ENTROPY_POOL_MIX_SYMBOL,
    X86_SYSTEMS_ENTROPY_POOL_STATE_SYMBOL, X86_SYSTEMS_ENTROPY_SAMPLE_CAPTURE_SYMBOL,
    X86_SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE_SYMBOL,
    X86_SYSTEMS_ENTROPY_USER_RANDOM_SERVICE_SYMBOL, X86_SYSTEMS_FRAME_ALLOCATE_SYMBOL,
    X86_SYSTEMS_LOCAL_NOTIFICATION_COMPLETE_SYMBOL, X86_SYSTEMS_LOCAL_NOTIFICATION_SEND_SYMBOL,
    X86_SYSTEMS_LOCAL_NOTIFICATION_WAIT_SYMBOL, X86_SYSTEMS_MONOTONIC_CLOCK_NOW_SYMBOL,
    X86_SYSTEMS_PCI_CONFIGURATION_READ_DWORD_SYMBOL,
    X86_SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD_SYMBOL,
    X86_SYSTEMS_PCI_REGION_READ_EVENTUALLY_SYMBOL, X86_SYSTEMS_PCI_REGION_READ_SYMBOL,
    X86_SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE_SYMBOL, X86_SYSTEMS_PCI_REGION_WRITE_SYMBOL,
    X86_SYSTEMS_PLATFORM_ABI, X86_SYSTEMS_PROVIDER_OBJECT_REVISION, X86_SYSTEMS_PROVIDER_REVISION,
    X86_SYSTEMS_PROVIDER_TEXT_SECTION, X86_SYSTEMS_RAW_ENTROPY_SAMPLE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ACTIVATE_ADDRESS_SPACE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_CLOCK_TICK_RATE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_CREDENTIALS_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_ENTRY_FACT_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_EXECUTABLE_NAME_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_HARDWARE_CAPABILITIES_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_LOAD_METADATA_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_PAGESZ_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_PLATFORM_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_PROGRAM_HEADERS_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_SECURE_MODE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ATTACH_ADDRESS_SPACE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_ENTROPY_ACQUISITION_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_ENTROPY_HANDOFF_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_ENTRY_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_STACK_RANDOM_SLOT_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_STACK_SLOT_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_CLEAR_STACK_ENTRY_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_COPY_PAYLOAD_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_DELIVER_ENTROPY_INITIAL_RANDOM_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_DETACH_ADDRESS_SPACE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_INITIALIZE_STACK_ENTRY_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_PREPARE_SYMBOL, X86_SYSTEMS_STATIC_IMAGE_PROCESS_ENTER_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_PROCESS_RESTORE_ENTRY_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_PUBLISH_STACK_LEAF_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_BUS_MASTER_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_DEVICE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_DMA_BUFFERS_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_DMA_PREPARATION_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_GENERATOR_LIFECYCLE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_NEGOTIATION_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_POOL_LIFECYCLE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_QUEUE_ACTIVATION_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_QUEUE_ADDRESSES_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_QUEUE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_REQUEST_COMPLETION_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_SAMPLE_CAPTURE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_SAMPLE_QUALITY_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_SPLIT_RING_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_TRANSPORT_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_RETAINED_DISJOINT_WORK_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_CLOCK_TICK_RATE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_CREDENTIALS_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_ENTRY_FACT_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_EXECUTABLE_NAME_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_HARDWARE_CAPABILITIES_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_LOAD_METADATA_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_PAGESZ_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_PLATFORM_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_PROGRAM_HEADERS_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_SECURE_MODE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_RESTORE_ADDRESS_SPACE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_RESTORE_PREPARED_PROCESS_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_RETAIN_PREPARED_PROCESS_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTROPY_ACQUISITION_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTROPY_HANDOFF_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTROPY_INITIAL_RANDOM_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTRY_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REVOKE_STACK_LEAF_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REVOKE_STACK_RANDOM_SLOT_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REVOKE_STACK_SLOT_SYMBOL, X86_SYSTEMS_STATIC_IMAGE_REVOKE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_SEAL_PERMISSIONS_SYMBOL, X86_SYSTEMS_STATIC_IMAGE_ZERO_PAYLOAD_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ZERO_STACK_BACKING_SYMBOL,
    X86_SYSTEMS_STATIC_PROCESS_BRK_RESERVE_SYMBOL,
    X86_SYSTEMS_STATIC_PROCESS_DESCRIPTOR_TABLE_SYMBOL,
    X86_SYSTEMS_STATIC_PROCESS_EXEC_SNAPSHOT_SYMBOL, X86_SYSTEMS_TRANSLATION_ACTIVATE_SYMBOL,
    X86_SYSTEMS_TRANSLATION_BEGIN_SYMBOL, X86_SYSTEMS_TRANSLATION_COMMIT_SYMBOL,
    X86_SYSTEMS_TRANSLATION_EDIT_BEGIN_SYMBOL, X86_SYSTEMS_TRANSLATION_EDIT_COMMIT_SYMBOL,
    X86_SYSTEMS_TRANSLATION_EDIT_MAP_SYMBOL, X86_SYSTEMS_TRANSLATION_EDIT_UNMAP_SYMBOL,
    X86_SYSTEMS_USER_ADDRESS_SPACE_MAP_SYMBOL, X86_SYSTEMS_USER_ADDRESS_SPACE_PREPARE_SYMBOL,
    X86_SYSTEMS_USER_ADDRESS_SPACE_RECLAIM_SYMBOL,
    X86_SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS_SYMBOL,
    X86_SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS_SYMBOL,
    X86_SYSTEMS_USER_ADDRESS_SPACE_UNMAP_SYMBOL,
    X86_SYSTEMS_USER_ADDRESS_SPACE_VERIFY_MAPPING_SYMBOL, X86_SYSTEMS_USER_COMPLETE_SYMBOL,
    X86_SYSTEMS_USER_COPY_FAULT_SYMBOL, X86_SYSTEMS_USER_ENTER_SYMBOL,
    X86_SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE_SYMBOL,
    X86_SYSTEMS_USER_MAPPING_LOAD_BYTE_SYMBOL, X86_SYSTEMS_USER_MAPPING_PERMIT_EXECUTE_SYMBOL,
    X86_SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA_SYMBOL,
    X86_SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY_SYMBOL,
    X86_SYSTEMS_USER_MAPPING_SEAL_IMMUTABLE_DATA_SYMBOL,
    X86_SYSTEMS_USER_MAPPING_STORE_BYTE_SYMBOL,
    X86_SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE_SYMBOL, X86_SYSTEMS_USER_PREPARE_SYMBOL,
    X86_SYSTEMS_USER_SYSCALL_SYMBOL, generate_x86_64_systems_provider_object,
};

pub const X86_SYSTEMS_ARTIFACT_REVISION: &str = "topal.systems-artifact.x86_64-qemu-pc-q35/255";
pub const X86_SYSTEMS_ROOT_OBJECT_REVISION: &str = "topal.systems-root-object.x86_64/241";
pub const X86_SYSTEMS_ROOT_TEXT_SECTION: &str = ".text.topal.systems.root";
pub const X86_SYSTEMS_KERNEL_ENTRY: &str = "_topal_kernel_entry";
pub const X86_SYSTEMS_DEBUG_BREAK_ENTRY: &str = "topal_x86_systems_debug_break_entry";
pub const X86_SYSTEMS_LOCAL_NOTIFICATION_ENTRY: &str = "topal_x86_systems_local_notification_entry";
pub const X86_SYSTEMS_DEADLINE_ENTRY: &str = "topal_x86_systems_deadline_entry";
pub const X86_SYSTEMS_KERNEL_THREAD_ENTRY: &str = "topal_x86_systems_kernel_thread_entry";
pub const X86_SYSTEMS_COMPLETION_THREAD_ENTRY: &str = "topal_x86_systems_completion_thread_entry";
pub const X86_SYSTEMS_TERMINAL_THREAD_ENTRY: &str = "topal_x86_systems_terminal_thread_entry";
pub const SYSTEMS_KERNEL_FILE: &str = "kernel.elf";
pub const SYSTEMS_DEBUG_FILE: &str = "kernel.debug";
pub const SYSTEMS_MAP_FILE: &str = "kernel.map";
pub const SYSTEMS_INITIAL_USER_IMAGE_FILE: &str = "initial-user.elf";
pub const SYSTEMS_STATIC_USER_IMAGE_FILE: &str = "static-user.elf";
pub const SYSTEMS_PROVENANCE_FILE: &str = "provenance.json";

const REQUIRED_LINKED_TEXT_SYMBOLS: [&str; 103] = [
    X86_SYSTEMS_KERNEL_ENTRY,
    X86_SYSTEMS_DEBUG_BREAK_ENTRY,
    X86_SYSTEMS_LOCAL_NOTIFICATION_ENTRY,
    X86_SYSTEMS_DEADLINE_ENTRY,
    X86_SYSTEMS_KERNEL_THREAD_ENTRY,
    X86_SYSTEMS_COMPLETION_THREAD_ENTRY,
    X86_SYSTEMS_TERMINAL_THREAD_ENTRY,
    X86_SYSTEMS_BOOT_MEMORY_SYMBOL,
    X86_SYSTEMS_FRAME_ALLOCATE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_PREPARE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ZERO_PAYLOAD_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ZERO_STACK_BACKING_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_PUBLISH_STACK_LEAF_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_INITIALIZE_STACK_ENTRY_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_PAGESZ_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_ENTRY_FACT_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_PROGRAM_HEADERS_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_EXECUTABLE_NAME_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_CREDENTIALS_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_SECURE_MODE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_CLOCK_TICK_RATE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_PLATFORM_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_HARDWARE_CAPABILITIES_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_LOAD_METADATA_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_STACK_RANDOM_SLOT_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_RETAIN_PREPARED_PROCESS_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_RETAINED_DISJOINT_WORK_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_ENTROPY_HANDOFF_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_ENTROPY_ACQUISITION_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_DEVICE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_TRANSPORT_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_NEGOTIATION_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_QUEUE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_BUS_MASTER_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_DMA_BUFFERS_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_SPLIT_RING_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_DMA_PREPARATION_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTROPY_ACQUISITION_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTROPY_HANDOFF_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_RESTORE_PREPARED_PROCESS_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REVOKE_STACK_RANDOM_SLOT_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_LOAD_METADATA_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_HARDWARE_CAPABILITIES_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_PLATFORM_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_CLOCK_TICK_RATE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_SECURE_MODE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_CREDENTIALS_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_EXECUTABLE_NAME_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_PROGRAM_HEADERS_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_ENTRY_FACT_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_PAGESZ_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_CLEAR_STACK_ENTRY_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REVOKE_STACK_LEAF_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_COPY_PAYLOAD_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_SEAL_PERMISSIONS_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ATTACH_ADDRESS_SPACE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_ACTIVATE_ADDRESS_SPACE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_RESTORE_ADDRESS_SPACE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_ENTRY_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_STACK_SLOT_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REVOKE_STACK_SLOT_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTRY_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_DETACH_ADDRESS_SPACE_SYMBOL,
    X86_SYSTEMS_STATIC_IMAGE_REVOKE_SYMBOL,
    X86_SYSTEMS_TRANSLATION_BEGIN_SYMBOL,
    X86_SYSTEMS_TRANSLATION_COMMIT_SYMBOL,
    X86_SYSTEMS_TRANSLATION_ACTIVATE_SYMBOL,
    X86_SYSTEMS_TRANSLATION_EDIT_BEGIN_SYMBOL,
    X86_SYSTEMS_TRANSLATION_EDIT_MAP_SYMBOL,
    X86_SYSTEMS_TRANSLATION_EDIT_UNMAP_SYMBOL,
    X86_SYSTEMS_TRANSLATION_EDIT_COMMIT_SYMBOL,
    X86_SYSTEMS_CRITICAL_ENTER_SYMBOL,
    X86_SYSTEMS_CRITICAL_RESTORE_SYMBOL,
    X86_SYSTEMS_ATOMIC_CREATE_SYMBOL,
    X86_SYSTEMS_ATOMIC_COMPARE_EXCHANGE_SYMBOL,
    X86_SYSTEMS_ATOMIC_LOAD_SYMBOL,
    X86_SYSTEMS_LOCAL_NOTIFICATION_SEND_SYMBOL,
    X86_SYSTEMS_LOCAL_NOTIFICATION_WAIT_SYMBOL,
    X86_SYSTEMS_LOCAL_NOTIFICATION_COMPLETE_SYMBOL,
    X86_SYSTEMS_MONOTONIC_CLOCK_NOW_SYMBOL,
    X86_SYSTEMS_DEADLINE_AFTER_SYMBOL,
    X86_SYSTEMS_DEADLINE_ARM_SYMBOL,
    X86_SYSTEMS_DEADLINE_WAIT_SYMBOL,
    X86_SYSTEMS_DEADLINE_COMPLETE_SYMBOL,
    X86_SYSTEMS_CONTEXT_CREATE_SYMBOL,
    X86_SYSTEMS_CONTEXT_TRANSFER_SYMBOL,
    X86_SYSTEMS_CONTEXT_AWAIT_DEADLINE_PREEMPTION_SYMBOL,
    X86_SYSTEMS_CONTEXT_PREEMPT_CURRENT_SYMBOL,
    X86_SYSTEMS_CONTEXT_RETIRE_SYMBOL,
    X86_SYSTEMS_CONTEXT_RECLAIM_SYMBOL,
    X86_SYSTEMS_USER_ADDRESS_SPACE_PREPARE_SYMBOL,
    X86_SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS_SYMBOL,
    X86_SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS_SYMBOL,
    X86_SYSTEMS_USER_PREPARE_SYMBOL,
    X86_SYSTEMS_USER_ENTER_SYMBOL,
    X86_SYSTEMS_USER_COMPLETE_SYMBOL,
    X86_SYSTEMS_USER_ADDRESS_SPACE_RECLAIM_SYMBOL,
    X86_SYSTEMS_USER_SYSCALL_SYMBOL,
    X86_SYSTEMS_USER_COPY_FAULT_SYMBOL,
    X86_SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE_SYMBOL,
    "topal_x86_systems_uart16550_write",
    "topal_x86_systems_interrupt_return",
    "topal_x86_systems_fatal",
];

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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kernel_library: Option<crate::SystemsSourceLibraryArtifact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region_library: Option<crate::SystemsRegionLibraryArtifact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region_runtime: Option<crate::SystemsRootRegionBinding>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::systems_region_library::deserialize_present_region_field"
    )]
    pub tail_root: Option<crate::SystemsTailRootBinding>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::systems_region_library::deserialize_present_region_field"
    )]
    pub frame_root: Option<crate::SystemsFrameRootBinding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_position_library: Option<crate::SystemsRegionLibraryArtifact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_position_runtime: Option<crate::SystemsFilePositionRegionBinding>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::systems_region_library::deserialize_present_region_field"
    )]
    pub ram_page_pool_library: Option<crate::SystemsRegionLibraryArtifact>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::systems_region_library::deserialize_present_region_field"
    )]
    pub ram_page_pool_runtime: Option<crate::SystemsRamPageRegionBinding>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::systems_region_library::deserialize_present_region_field"
    )]
    pub inode_mode_library: Option<crate::SystemsRegionLibraryArtifact>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::systems_region_library::deserialize_present_region_field"
    )]
    pub inode_mode_runtime: Option<crate::SystemsInodeRegionBinding>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::systems_region_library::deserialize_present_region_field"
    )]
    pub namespace_library: Option<crate::SystemsRegionLibraryArtifact>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::systems_region_library::deserialize_present_region_field"
    )]
    pub namespace_runtime: Option<crate::SystemsNamespaceRegionBinding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub time_library: Option<crate::SystemsNumericLibraryArtifact>,
    pub image_library: Option<crate::SystemsNumericLibraryArtifact>,
    pub dispatch_library: Option<crate::SystemsNumericLibraryArtifact>,
    pub process_identity_library: Option<crate::SystemsNumericLibraryArtifact>,
    pub volatile_files: crate::SystemsVolatileFileProfile,
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
    pub initial_user_image: PathBuf,
    pub static_user_image: Option<PathBuf>,
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
    let initial_user_image = initial_user_elf_file()?;
    publish_x86_64_systems_artifact_with_initial_user_image(
        program,
        tools,
        &initial_user_image,
        destination,
    )
}

/// Link, inspect, and atomically publish a systems ELF set with a supplied
/// bounded-profile initial user image.
///
/// # Errors
///
/// Returns an error before publication when the supplied image is outside the
/// qualified x86-64 envelope or when normal artifact production fails.
pub fn publish_x86_64_systems_artifact_with_initial_user_image(
    program: &CompilerSystemsProgram,
    tools: &LlvmTools,
    initial_user_image: &[u8],
    destination: &Path,
) -> Result<PublishedSystemsArtifact, CompileError> {
    publish_x86_64_systems_artifact_with_user_images(
        program,
        tools,
        initial_user_image,
        None,
        destination,
    )
}

/// Link, inspect, and atomically publish a systems ELF set with a supplied
/// approved-shape static user image and the generated boot user image.
///
/// # Errors
///
/// Returns an error before publication when the static image is outside the
/// qualified x86-64 envelope or when normal artifact production fails.
pub fn publish_x86_64_systems_artifact_with_static_user_image(
    program: &CompilerSystemsProgram,
    tools: &LlvmTools,
    static_user_image: &[u8],
    destination: &Path,
) -> Result<PublishedSystemsArtifact, CompileError> {
    let initial_user_image = initial_user_elf_file()?;
    publish_x86_64_systems_artifact_with_user_images(
        program,
        tools,
        &initial_user_image,
        Some(static_user_image),
        destination,
    )
}

/// Link, inspect, and atomically publish a systems ELF set with independently
/// supplied boot and static user images.
///
/// # Errors
///
/// Returns an error before publication when either supplied image is outside
/// its qualified x86-64 envelope or when normal artifact production fails.
pub fn publish_x86_64_systems_artifact_with_user_images(
    program: &CompilerSystemsProgram,
    tools: &LlvmTools,
    initial_user_image: &[u8],
    static_user_image: Option<&[u8]>,
    destination: &Path,
) -> Result<PublishedSystemsArtifact, CompileError> {
    publish_x86_64_systems_artifact_with_source_library(
        program,
        tools,
        Some(initial_user_image),
        static_user_image,
        None,
        destination,
    )
}

/// Publish a systems root with its explicit immutable source-library snapshot.
///
/// # Errors
/// Rejects missing, mismatched or unqualified source before atomic publication.
pub fn publish_x86_64_systems_artifact_with_source_library(
    program: &CompilerSystemsProgram,
    tools: &LlvmTools,
    initial_user_image: Option<&[u8]>,
    static_user_image: Option<&[u8]>,
    source_library: Option<&crate::SystemsSourceLibraryInput>,
    destination: &Path,
) -> Result<PublishedSystemsArtifact, CompileError> {
    publish_x86_64_systems_artifact_with_libraries(
        program,
        tools,
        initial_user_image,
        static_user_image,
        source_library,
        None,
        destination,
    )
}

/// Explicit immutable inputs for each policy selected by a systems root.
#[derive(Clone, Copy, Default)]
pub struct SystemsKernelLibraries<'a> {
    pub region: Option<&'a crate::SystemsSourceLibraryInput>,
    pub file_position: Option<&'a crate::SystemsSourceLibraryInput>,
    pub ram_page_pool: Option<&'a crate::SystemsSourceLibraryInput>,
    pub inode_mode: Option<&'a crate::SystemsSourceLibraryInput>,
    pub namespace: Option<&'a crate::SystemsSourceLibraryInput>,
    pub policy: Option<&'a crate::SystemsSourceLibraryInput>,
    pub time: Option<&'a crate::SystemsSourceLibraryInput>,
    pub image: Option<&'a crate::SystemsSourceLibraryInput>,
    pub dispatch: Option<&'a crate::SystemsSourceLibraryInput>,
    pub process_identity: Option<&'a crate::SystemsSourceLibraryInput>,
}

/// Publish both explicit source-library snapshots with their owned kernel root.
///
/// # Errors
/// Rejects missing, mismatched, unsupported or unbounded source before publication.
pub fn publish_x86_64_systems_artifact_with_libraries(
    program: &CompilerSystemsProgram,
    tools: &LlvmTools,
    initial_user_image: Option<&[u8]>,
    static_user_image: Option<&[u8]>,
    source_library: Option<&crate::SystemsSourceLibraryInput>,
    time_library: Option<&crate::SystemsSourceLibraryInput>,
    destination: &Path,
) -> Result<PublishedSystemsArtifact, CompileError> {
    publish_x86_64_systems_artifact_with_kernel_libraries(
        program,
        tools,
        initial_user_image,
        static_user_image,
        SystemsKernelLibraries {
            file_position: None,
            ram_page_pool: None,
            inode_mode: None,
            namespace: None,
            region: None,
            policy: source_library,
            time: time_library,
            image: None,
            dispatch: None,
            process_identity: None,
        },
        destination,
    )
}

/// Publish the complete explicit policy set without ambient source discovery.
///
/// # Errors
/// Rejects missing/mismatched snapshots and unqualified resources before publication.
pub fn publish_x86_64_systems_artifact_with_kernel_libraries(
    program: &CompilerSystemsProgram,
    tools: &LlvmTools,
    initial_user_image: Option<&[u8]>,
    static_user_image: Option<&[u8]>,
    libraries: SystemsKernelLibraries<'_>,
    destination: &Path,
) -> Result<PublishedSystemsArtifact, CompileError> {
    publish_with_root_source(
        program,
        tools,
        initial_user_image,
        static_user_image,
        libraries,
        destination,
        None,
        None,
    )
}

/// Publish the tail profile with its complete immutable checked root source.
///
/// # Errors
/// Refuses incomplete source/native/caller reconstruction before publication.
pub fn publish_x86_64_systems_tail_artifact(
    root_source: &str,
    tools: &LlvmTools,
    initial_user_image: Option<&[u8]>,
    static_user_image: Option<&[u8]>,
    libraries: SystemsKernelLibraries<'_>,
    destination: &Path,
) -> Result<PublishedSystemsArtifact, CompileError> {
    let program = topal_language::compiler::analyze_systems_for_compiler(
        root_source,
        &topal_language::compiler::CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .map_err(CompileError::Diagnostic)?;
    if !program.region_library.as_ref().is_some_and(|r|r.interface==topal_language::compiler::CompilerSystemsSourceLibraryInterface::BorrowedByteRegionTailActionV1) {
        return Err(CompileError::Tool("E-SYSTEMS-TAIL-BINDING: explicit tail source root required".into()));
    }
    publish_with_root_source(
        &program,
        tools,
        initial_user_image,
        static_user_image,
        libraries,
        destination,
        Some(root_source),
        None,
    )
}

/// Publish a frame-policy kernel from complete immutable root/library inputs.
///
/// # Errors
/// Refuses incomplete source/native/caller/load reconstruction before publication.
pub fn publish_x86_64_systems_frame_artifact(
    root_source: &str,
    frame_library: &crate::SystemsSourceLibraryInput,
    tools: &LlvmTools,
    initial_user_image: Option<&[u8]>,
    static_user_image: Option<&[u8]>,
    libraries: SystemsKernelLibraries<'_>,
    destination: &Path,
) -> Result<PublishedSystemsArtifact, CompileError> {
    let checked = crate::systems_frame_policy_binding::check(root_source, frame_library)?;
    publish_with_root_source(
        &checked.program,
        tools,
        initial_user_image,
        static_user_image,
        libraries,
        destination,
        None,
        Some((root_source, frame_library)),
    )
}

fn publish_with_root_source(
    program: &CompilerSystemsProgram,
    tools: &LlvmTools,
    initial_user_image: Option<&[u8]>,
    static_user_image: Option<&[u8]>,
    libraries: SystemsKernelLibraries<'_>,
    destination: &Path,
    tail_source: Option<&str>,
    frame_source: Option<(&str, &crate::SystemsSourceLibraryInput)>,
) -> Result<PublishedSystemsArtifact, CompileError> {
    if program.frame_allocator_library.is_some() != frame_source.is_some()
        || tail_source.is_some() && frame_source.is_some()
    {
        return Err(CompileError::Tool("E-SYSTEMS-FRAME-POLICY-BINDING: kernel publication requires complete source/native/allocator/caller reconstruction".into()));
    }
    let checked_frame = frame_source
        .map(|(root, input)| crate::systems_frame_policy_binding::check(root, input))
        .transpose()?;
    if checked_frame
        .as_ref()
        .is_some_and(|c| c.program != *program)
    {
        return Err(CompileError::Tool(
            "E-SYSTEMS-FRAME-RECONSTRUCTION: root source/program differ".into(),
        ));
    }
    let SystemsKernelLibraries {
        region: region_library,
        file_position: file_position_library,
        ram_page_pool: ram_page_pool_library,
        inode_mode: inode_mode_library,
        namespace: namespace_library,
        policy: source_library,
        time: time_library,
        image: image_library,
        dispatch: dispatch_library,
        process_identity: process_identity_library,
    } = libraries;
    if tail_source.is_none() && program.region_library.as_ref().is_some_and(|r| r.interface == topal_language::compiler::CompilerSystemsSourceLibraryInterface::BorrowedByteRegionTailActionV1) {
        return Err(CompileError::Tool("E-SYSTEMS-TAIL-KERNEL: canonical publication requires the complete kernel tail owner source/native/caller binding".into()));
    }
    if program.namespace_library.is_some() != namespace_library.is_some() {
        return Err(CompileError::Tool(
            "E-SYSTEMS-NAMESPACE: declaration requires its matching explicit source snapshot"
                .into(),
        ));
    }
    if program.inode_mode_library.is_some() != inode_mode_library.is_some() {
        return Err(CompileError::Tool(
            "E-SYSTEMS-INODE-MODE: declaration requires its matching explicit source snapshot"
                .into(),
        ));
    }
    if program.ram_page_pool_library.is_some() != ram_page_pool_library.is_some() {
        return Err(CompileError::Tool("E-SYSTEMS-REGION-PAGE: declaration requires its matching explicit page source snapshot".into()));
    }
    if program.file_position_library.is_some() != file_position_library.is_some() {
        return Err(CompileError::Tool("E-SYSTEMS-REGION-ACTION: declaration requires its matching explicit action source snapshot".into()));
    }
    if program.region_library.is_some() != region_library.is_some() {
        return Err(CompileError::Tool("E-SYSTEMS-REGION-ROOT: declaration requires its matching explicit region source snapshot".into()));
    }
    if program.process_identity_library.is_some() != process_identity_library.is_some() {
        return Err(CompileError::Tool("E-SYSTEMS-LIBRARY: declared process identity import requires its matching explicit source snapshot".into()));
    }
    if program.dispatch_library.is_some() != dispatch_library.is_some() {
        return Err(CompileError::Tool("E-SYSTEMS-LIBRARY: declared dispatch import requires its matching explicit source snapshot".into()));
    }
    if program.image_library.is_some() != image_library.is_some() {
        return Err(CompileError::Tool("E-SYSTEMS-LIBRARY: declared image import requires its matching explicit source snapshot".into()));
    }
    if program.time_library.is_some() != time_library.is_some() {
        return Err(CompileError::Tool("E-SYSTEMS-LIBRARY: declared numeric import requires its matching explicit source snapshot".into()));
    }
    if program.kernel_library.is_some() != source_library.is_some() {
        return Err(CompileError::Tool(
            "E-SYSTEMS-LIBRARY: declared import requires its matching explicit source snapshot"
                .into(),
        ));
    }
    let generated;
    let initial_user_image = match initial_user_image {
        Some(image) => image,
        None => {
            generated = initial_user_elf_file()?;
            &generated
        }
    };
    validate_x86_64_initial_user_image(initial_user_image)?;
    if let Some(image) = static_user_image {
        validate_x86_64_static_user_image(image)?;
    }
    let generated_provider = match &checked_frame {
        Some(checked) => crate::systems_provider_object::generate_frame_memory_provider(checked)?,
        None => generate_x86_64_systems_provider_object(program)?,
    };
    let root_object = match &checked_frame {
        Some(checked) => generate_frame_root_checked(checked)?,
        None => generate_root_object(program)?,
    };
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
        initial_user_image,
        static_user_image,
        source_library,
        time_library,
        image_library,
        dispatch_library,
        process_identity_library,
        region_library,
        file_position_library,
        ram_page_pool_library,
        inode_mode_library,
        namespace_library,
        &stage,
        tail_source,
        frame_source,
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
        initial_user_image: destination.join(SYSTEMS_INITIAL_USER_IMAGE_FILE),
        static_user_image: static_user_image
            .map(|_| destination.join(SYSTEMS_STATIC_USER_IMAGE_FILE)),
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
    initial_user_image: &[u8],
    static_user_image: Option<&[u8]>,
    source_library: Option<&crate::SystemsSourceLibraryInput>,
    time_library: Option<&crate::SystemsSourceLibraryInput>,
    image_library: Option<&crate::SystemsSourceLibraryInput>,
    dispatch_library: Option<&crate::SystemsSourceLibraryInput>,
    process_identity_library: Option<&crate::SystemsSourceLibraryInput>,
    region_library: Option<&crate::SystemsSourceLibraryInput>,
    file_position_library: Option<&crate::SystemsSourceLibraryInput>,
    ram_page_pool_library: Option<&crate::SystemsSourceLibraryInput>,
    inode_mode_library: Option<&crate::SystemsSourceLibraryInput>,
    namespace_library: Option<&crate::SystemsSourceLibraryInput>,
    stage: &Path,
    tail_source: Option<&str>,
    frame_source: Option<(&str, &crate::SystemsSourceLibraryInput)>,
) -> Result<SystemsArtifactProvenance, CompileError> {
    let work = stage.join("work");
    fs::create_dir(&work).map_err(io_error("create artifact work directory", &work))?;
    let provider_path = work.join("provider.o");
    let root_path = work.join("root.o");
    fs::write(&provider_path, provider_object)
        .map_err(io_error("write provider object", &provider_path))?;
    fs::write(&root_path, root_object).map_err(io_error("write root object", &root_path))?;
    let mut tail = tail_source
        .map(|root| {
            let source = region_library.ok_or_else(|| {
                CompileError::Tool("E-SYSTEMS-TAIL-BINDING: missing source snapshot".into())
            })?;
            let fresh = crate::systems_tail_publication::rebuild(
                root,
                &source.into(),
                tools,
                &work.join("tail"),
            )?;
            if fresh.program != *program {
                return Err(CompileError::Tool(
                    "E-SYSTEMS-TAIL-BINDING: root source/program disagree".into(),
                ));
            }
            Ok(fresh)
        })
        .transpose()?;
    let mut frame = frame_source
        .map(|(root, input)| {
            let fresh = crate::systems_frame_publication::rebuild(
                root,
                &input.into(),
                tools,
                &work.join("frame"),
            )?;
            if fresh.program != *program {
                return Err(CompileError::Tool(
                    "E-SYSTEMS-FRAME-RECONSTRUCTION: source/program differ".into(),
                ));
            }
            Ok(fresh)
        })
        .transpose()?;
    let bound_provider = frame
        .as_ref()
        .map(|f| f.objects[1].as_slice())
        .or_else(|| tail.as_ref().map(|t| t.objects[1].as_slice()))
        .unwrap_or(provider_object)
        .to_vec();
    let provider_object = bound_provider.as_slice();
    let bound_root = frame
        .as_ref()
        .map(|f| f.objects[0].as_slice())
        .or_else(|| tail.as_ref().map(|t| t.objects[0].as_slice()))
        .unwrap_or(root_object)
        .to_vec();
    let root_object = bound_root.as_slice();
    fs::write(&root_path, root_object).map_err(io_error("write bound root object", &root_path))?;
    fs::write(&provider_path, provider_object)
        .map_err(io_error("write bound provider object", &provider_path))?;

    let mut library = program
        .kernel_library
        .as_ref()
        .zip(source_library)
        .map(|(requirement, input)| {
            crate::systems_library::build_source_library(requirement, input, tools, &work)
        })
        .transpose()?;
    let region = program
        .region_library
        .as_ref()
        .filter(|r|r.interface!=topal_language::compiler::CompilerSystemsSourceLibraryInterface::BorrowedByteRegionTailActionV1)
        .zip(region_library)
        .map(|(requirement, input)| {
            if requirement.source != input.source {
                return Err(CompileError::Tool(
                    "E-SYSTEMS-REGION-ROOT: source path does not match the declared entry".into(),
                ));
            }
            if requirement.interface == topal_language::compiler::CompilerSystemsSourceLibraryInterface::BorrowedByteRegionInstanceActionV1 {
                crate::build_systems_region_instance_library(input, &requirement.export, 0,
                    crate::systems_region_library::ROOT_INSTANCE, tools, &work)
            } else {
                crate::systems_region_library::build_systems_region_library(input, &requirement.export, tools, &work)
            }
        })
        .transpose()?;
    let file_position = program
        .file_position_library
        .as_ref()
        .zip(file_position_library)
        .map(|(requirement, input)| {
            if requirement.source != input.source {
                return Err(CompileError::Tool(
                    "E-SYSTEMS-REGION-ACTION: source path does not match the declared entry".into(),
                ));
            }
            let action_work = work.join("file-position");
            fs::create_dir(&action_work)
                .map_err(io_error("create action object work space", &action_work))?;
            if requirement.interface == topal_language::compiler::CompilerSystemsSourceLibraryInterface::BorrowedByteRegionInstanceActionV1 {
                crate::build_systems_region_instance_library(input, &requirement.export, 2,
                    crate::systems_region_library::FILE_INSTANCE, tools, &action_work)
            } else if requirement.interface == topal_language::compiler::CompilerSystemsSourceLibraryInterface::BorrowedByteRegionFramedActionV1 {
                crate::build_systems_region_framed_action_library(
                    input, &requirement.export, 2, tools, &action_work,
                )
            } else {
                crate::build_systems_region_action_library(
                    input, &requirement.export, tools, &action_work,
                )
            }
        })
        .transpose()?;
    let ram_page_pool = program
        .ram_page_pool_library
        .as_ref()
        .zip(ram_page_pool_library)
        .map(|(requirement, input)| {
            if requirement.source != input.source {
                return Err(CompileError::Tool(
                    "E-SYSTEMS-REGION-PAGE: source path does not match the declared entry".into(),
                ));
            }
            let page_work = work.join("ram-page-pool");
            fs::create_dir(&page_work)
                .map_err(io_error("create page object work space", &page_work))?;
            crate::build_systems_region_instance_library(
                input,
                &requirement.export,
                2,
                crate::systems_region_library::PAGE_INSTANCE,
                tools,
                &page_work,
            )
        })
        .transpose()?;
    let inode_mode = program
        .inode_mode_library
        .as_ref()
        .zip(inode_mode_library)
        .map(|(requirement, input)| {
            if requirement.source != input.source {
                return Err(CompileError::Tool(
                    "E-SYSTEMS-INODE-MODE: source path does not match the declared entry".into(),
                ));
            }
            let inode_work = work.join("inode-mode");
            fs::create_dir(&inode_work)
                .map_err(io_error("create inode object work space", &inode_work))?;
            crate::build_systems_region_instance_library(
                input,
                &requirement.export,
                2,
                crate::systems_region_library::INODE_INSTANCE,
                tools,
                &inode_work,
            )
        })
        .transpose()?;
    let namespace = program
        .namespace_library
        .as_ref()
        .zip(namespace_library)
        .map(|(requirement, input)| {
            if requirement.source != input.source {
                return Err(CompileError::Tool(
                    "E-SYSTEMS-NAMESPACE: source path does not match the declared entry".into(),
                ));
            }
            let namespace_work = work.join("namespace");
            fs::create_dir(&namespace_work).map_err(io_error(
                "create namespace object work space",
                &namespace_work,
            ))?;
            crate::build_systems_region_instance_library(
                input,
                &requirement.export,
                2,
                crate::systems_region_library::NAMESPACE_INSTANCE,
                tools,
                &namespace_work,
            )
        })
        .transpose()?;
    if program.time_library.is_some()
        || program.image_library.is_some()
        || program.dispatch_library.is_some()
        || program.process_identity_library.is_some()
    {
        crate::systems_numeric_library::inspect_numeric_provider(tools, &provider_path)?;
    }
    let mut numeric = program
        .time_library
        .as_ref()
        .zip(time_library)
        .map(|(requirement, input)| {
            crate::systems_numeric_library::build_numeric_library(requirement, input, tools, &work)
        })
        .transpose()?;
    let mut image_numeric = program
        .image_library
        .as_ref()
        .zip(image_library)
        .map(|(requirement, input)| {
            crate::systems_numeric_library::build_numeric_library(requirement, input, tools, &work)
        })
        .transpose()?;
    let mut dispatch_numeric = program
        .dispatch_library
        .as_ref()
        .zip(dispatch_library)
        .map(|(requirement, input)| {
            crate::systems_numeric_library::build_numeric_library(requirement, input, tools, &work)
        })
        .transpose()?;
    let mut identity_numeric = program
        .process_identity_library
        .as_ref()
        .zip(process_identity_library)
        .map(|(requirement, input)| {
            crate::systems_numeric_library::build_numeric_library(requirement, input, tools, &work)
        })
        .transpose()?;
    if let Some(frame) = frame.as_mut() {
        let sources = crate::systems_frame_publication::library_sources(SystemsKernelLibraries {
            policy: source_library,
            time: time_library,
            image: image_library,
            dispatch: dispatch_library,
            process_identity: process_identity_library,
            region: region_library,
            file_position: file_position_library,
            ram_page_pool: ram_page_pool_library,
            inode_mode: inode_mode_library,
            namespace: namespace_library,
        });
        frame.complete_libraries(&sources, tools, &work.join("frame"))?;
        for (role, object, path) in [
            (
                "library-kernel",
                library.as_ref().map(|l| l.object.as_slice()),
                "kernel-library.o",
            ),
            (
                "library-time",
                numeric.as_ref().map(|l| l.object.as_slice()),
                "time-library.o",
            ),
            (
                "library-image",
                image_numeric.as_ref().map(|l| l.object.as_slice()),
                "image-library.o",
            ),
            (
                "library-dispatch",
                dispatch_numeric.as_ref().map(|l| l.object.as_slice()),
                "dispatch-library.o",
            ),
            (
                "library-process-identity",
                identity_numeric.as_ref().map(|l| l.object.as_slice()),
                "process-identity-library.o",
            ),
            (
                "library-region",
                region.as_ref().map(|l| l.object.as_slice()),
                "kernel-region.o",
            ),
            (
                "library-file-position",
                file_position.as_ref().map(|l| l.object.as_slice()),
                "file-position/kernel-region.o",
            ),
            (
                "library-ram-page-pool",
                ram_page_pool.as_ref().map(|l| l.object.as_slice()),
                "ram-page-pool/kernel-region.o",
            ),
            (
                "library-inode-mode",
                inode_mode.as_ref().map(|l| l.object.as_slice()),
                "inode-mode/kernel-region.o",
            ),
            (
                "library-namespace",
                namespace.as_ref().map(|l| l.object.as_slice()),
                "namespace/kernel-region.o",
            ),
        ] {
            if let Some(object) = object {
                if frame.record.other_original_objects.get(role) != Some(&sha256(object)) {
                    return Err(CompileError::Tool(format!(
                        "E-SYSTEMS-FRAME-RECONSTRUCTION: metadata/native source producer differs for {role}"
                    )));
                }
                fs::copy(work.join(format!("frame/{role}.o")), work.join(path))
                    .map_err(io_error("install reconstructed library object", &work))?;
            }
        }
    }
    let linked = build_linked_outputs(tools, &work, stage)?;
    let frame_end = if let Some(frame) = frame.as_mut() {
        frame.bind(&linked.kernel)?;
        let file = object::File::parse(linked.kernel.as_slice())
            .map_err(|e| CompileError::Tool(e.to_string()))?;
        Some(
            file.segments()
                .map(|s| s.address() + s.size())
                .max()
                .unwrap()
                .max(0x1000000),
        )
    } else {
        None
    };
    if let Some(library) = library.as_mut() {
        library.record.linked_inputs = Some(crate::systems_policy_inputs::inspect(
            &linked.kernel,
            "kernel-library",
            &library.record.object_sha256,
            &library.record.sections,
        )?);
    }
    for (library, slot) in [
        (numeric.as_mut(), "time-library"),
        (image_numeric.as_mut(), "image-library"),
        (dispatch_numeric.as_mut(), "dispatch-library"),
        (identity_numeric.as_mut(), "process-identity-library"),
    ] {
        if let Some(library) = library {
            library.record.linked_inputs = Some(crate::systems_policy_inputs::inspect(
                &linked.kernel,
                slot,
                &library.record.object_sha256,
                &library.record.sections,
            )?);
        }
    }
    let ram_page_pool_runtime = ram_page_pool
        .as_ref()
        .map(|page| {
            crate::systems_provider_object::ram_page_region::inspect_linked(
                &linked.kernel,
                &page.record,
            )
        })
        .transpose()?;
    let inode_mode_runtime = inode_mode
        .as_ref()
        .map(|inode| {
            crate::systems_provider_object::inode_region::inspect_linked(
                &linked.kernel,
                &inode.record,
                program.dispatch_library.as_ref().is_some_and(|library|
                    library.interface == topal_language::compiler::CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV6
                ),
            )
        })
        .transpose()?;
    let namespace_runtime = namespace
        .as_ref()
        .map(|namespace| {
            crate::systems_provider_object::namespace_region::inspect_linked(
                &linked.kernel,
                &namespace.record,
                program.dispatch_library.as_ref().is_some_and(|library|
                    library.interface == topal_language::compiler::CompilerSystemsSourceLibraryInterface::StaticLinuxDispatchV6
                ),
            )
        })
        .transpose()?;
    let file_position_runtime = file_position
        .as_ref()
        .map(|action| {
            crate::systems_provider_object::file_region::inspect_linked(
                &linked.kernel,
                &action.record,
            )
        })
        .transpose()?;
    let region_runtime = region
        .as_ref()
        .map(|region| {
            crate::systems_provider_object::root_region::inspect_linked(
                &linked.kernel,
                &region.record,
                plan.bootstrap_placement.capacity_bytes,
            )
        })
        .transpose()?;
    if let Some(identity) = &identity_numeric {
        crate::systems_numeric_library::inspect_linked_numeric_library(
            &linked.kernel,
            &identity.record,
        )?;
    }
    crate::systems_provider_object::inspect_linked_process_backing(&linked.kernel)?;
    if let Some(numeric) = &numeric {
        crate::systems_numeric_library::inspect_linked_numeric_library(
            &linked.kernel,
            &numeric.record,
        )?;
    }
    if let Some(image) = &image_numeric {
        crate::systems_numeric_library::inspect_linked_numeric_library(
            &linked.kernel,
            &image.record,
        )?;
    }
    if let Some(library) = &library {
        crate::systems_library::inspect_linked_library(&linked.kernel, &library.record)?;
    }
    if let Some(dispatch) = &dispatch_numeric {
        crate::systems_numeric_library::inspect_linked_numeric_library(
            &linked.kernel,
            &dispatch.record,
        )?;
    }
    inspect_linked_kernel(
        &linked.kernel,
        &linked.debug,
        &linked.map,
        plan.bootstrap_placement.capacity_bytes,
        frame_end,
        {
            let operations = &program.bootstrap.handler.operations;
            let memory = operations.iter().any(|op| {
                matches!(
                    op,
                    CompilerSystemsOperation::BootstrapStoreByte { .. }
                        | CompilerSystemsOperation::BootstrapLoadByteEquals { .. }
                        | CompilerSystemsOperation::AtomicWordCreate { .. }
                        | CompilerSystemsOperation::AtomicCompareExchangeEquals { .. }
                        | CompilerSystemsOperation::AtomicLoadEquals { .. }
                )
            });
            if memory {
                let store = operations.iter().find_map(|op| {
                    if let CompilerSystemsOperation::BootstrapStoreByte { value, .. } = op {
                        Some(*value)
                    } else {
                        None
                    }
                });
                let load = operations.iter().find_map(|op| {
                    if let CompilerSystemsOperation::BootstrapLoadByteEquals { expected, .. } = op {
                        Some(*expected)
                    } else {
                        None
                    }
                });
                Some(store.zip(load).ok_or_else(|| {
                    CompileError::Tool("checked bootstrap memory probe is incomplete".into())
                })?)
            } else {
                None
            }
        },
    )?;
    let placements = linked_placements(&linked.kernel)?;
    let initial_user_image_path = stage.join(SYSTEMS_INITIAL_USER_IMAGE_FILE);
    fs::write(&initial_user_image_path, initial_user_image).map_err(io_error(
        "write initial user image",
        &initial_user_image_path,
    ))?;
    if let Some(image) = static_user_image {
        let static_user_image_path = stage.join(SYSTEMS_STATIC_USER_IMAGE_FILE);
        fs::write(&static_user_image_path, image)
            .map_err(io_error("write static user image", &static_user_image_path))?;
    }
    let mut record = artifact_provenance(
        program,
        tools,
        plan,
        provider_object,
        root_object,
        &linked.kernel,
        &linked.debug,
        &linked.map,
        initial_user_image,
        static_user_image,
        placements,
    )?;
    if let Some(tail) = tail.as_mut() {
        tail.bind(&linked.kernel)?;
    }
    if let Some(page) = ram_page_pool {
        for (name, bytes) in [
            ("ram-page-pool-library.t", page.source.as_bytes()),
            ("ram-page-pool-library.ll", page.llvm.as_bytes()),
            (
                "ram-page-pool-library.retained.ll",
                page.retained_llvm.as_bytes(),
            ),
            ("ram-page-pool-library.o", page.object.as_slice()),
            (
                "ram-page-pool-library.modules.json",
                page.module_snapshot.as_slice(),
            ),
        ] {
            fs::write(stage.join(name), bytes)
                .map_err(io_error("write page source artifact", stage))?;
            record.outputs.push(DigestEntry {
                identity: name.into(),
                sha256: sha256(bytes),
            });
        }
        record.inputs.push(DigestEntry {
            identity: format!("kernel-ram-page-pool-source:{}", page.record.source),
            sha256: page.record.source_sha256.clone(),
        });
        record
            .inputs
            .extend(page.record.modules.iter().map(|module| DigestEntry {
                identity: format!("kernel-ram-page-pool-source-module:{}", module.source),
                sha256: module.sha256.clone(),
            }));
        record.ram_page_pool_library = Some(page.record);
        record.ram_page_pool_runtime = ram_page_pool_runtime;
        record.inputs.sort_by(|a, b| a.identity.cmp(&b.identity));
        record.outputs.sort_by(|a, b| a.identity.cmp(&b.identity));
    }
    if let Some(inode) = inode_mode {
        for (name, bytes) in [
            ("inode-mode-library.t", inode.source.as_bytes()),
            ("inode-mode-library.ll", inode.llvm.as_bytes()),
            (
                "inode-mode-library.retained.ll",
                inode.retained_llvm.as_bytes(),
            ),
            ("inode-mode-library.o", inode.object.as_slice()),
            (
                "inode-mode-library.modules.json",
                inode.module_snapshot.as_slice(),
            ),
        ] {
            fs::write(stage.join(name), bytes)
                .map_err(io_error("write inode source artifact", stage))?;
            record.outputs.push(DigestEntry {
                identity: name.into(),
                sha256: sha256(bytes),
            });
        }
        record.inputs.push(DigestEntry {
            identity: format!("kernel-inode-mode-source:{}", inode.record.source),
            sha256: inode.record.source_sha256.clone(),
        });
        record
            .inputs
            .extend(inode.record.modules.iter().map(|module| DigestEntry {
                identity: format!("kernel-inode-mode-source-module:{}", module.source),
                sha256: module.sha256.clone(),
            }));
        record.inode_mode_library = Some(inode.record);
        record.inode_mode_runtime = inode_mode_runtime;
        record.inputs.sort_by(|a, b| a.identity.cmp(&b.identity));
        record.outputs.sort_by(|a, b| a.identity.cmp(&b.identity));
    }
    if let Some(namespace) = namespace {
        for (name, bytes) in [
            ("namespace-library.t", namespace.source.as_bytes()),
            ("namespace-library.ll", namespace.llvm.as_bytes()),
            (
                "namespace-library.retained.ll",
                namespace.retained_llvm.as_bytes(),
            ),
            ("namespace-library.o", namespace.object.as_slice()),
            (
                "namespace-library.modules.json",
                namespace.module_snapshot.as_slice(),
            ),
        ] {
            fs::write(stage.join(name), bytes)
                .map_err(io_error("write namespace source artifact", stage))?;
            record.outputs.push(DigestEntry {
                identity: name.into(),
                sha256: sha256(bytes),
            });
        }
        record.inputs.push(DigestEntry {
            identity: format!("kernel-namespace-source:{}", namespace.record.source),
            sha256: namespace.record.source_sha256.clone(),
        });
        record
            .inputs
            .extend(namespace.record.modules.iter().map(|module| DigestEntry {
                identity: format!("kernel-namespace-source-module:{}", module.source),
                sha256: module.sha256.clone(),
            }));
        record.namespace_library = Some(namespace.record);
        record.namespace_runtime = namespace_runtime;
        record.inputs.sort_by(|a, b| a.identity.cmp(&b.identity));
        record.outputs.sort_by(|a, b| a.identity.cmp(&b.identity));
    }
    if let Some(action) = file_position {
        for (name, bytes) in [
            ("file-position-library.t", action.source.as_bytes()),
            ("file-position-library.ll", action.llvm.as_bytes()),
            (
                "file-position-library.retained.ll",
                action.retained_llvm.as_bytes(),
            ),
            ("file-position-library.o", action.object.as_slice()),
            (
                "file-position-library.modules.json",
                action.module_snapshot.as_slice(),
            ),
        ] {
            fs::write(stage.join(name), bytes)
                .map_err(io_error("write action source artifact", stage))?;
            record.outputs.push(DigestEntry {
                identity: name.into(),
                sha256: sha256(bytes),
            });
        }
        record.inputs.push(DigestEntry {
            identity: format!("kernel-file-position-source:{}", action.record.source),
            sha256: action.record.source_sha256.clone(),
        });
        record
            .inputs
            .extend(action.record.modules.iter().map(|module| DigestEntry {
                identity: format!("kernel-file-position-source-module:{}", module.source),
                sha256: module.sha256.clone(),
            }));
        record.file_position_library = Some(action.record);
        record.file_position_runtime = file_position_runtime;
        record.inputs.sort_by(|a, b| a.identity.cmp(&b.identity));
        record.outputs.sort_by(|a, b| a.identity.cmp(&b.identity));
    }
    if let Some(region) = region {
        for (name, bytes) in [
            ("region-library.t", region.source.as_bytes()),
            ("region-library.ll", region.llvm.as_bytes()),
            (
                "region-library.retained.ll",
                region.retained_llvm.as_bytes(),
            ),
            ("region-library.o", region.object.as_slice()),
            (
                "region-library.modules.json",
                region.module_snapshot.as_slice(),
            ),
        ] {
            fs::write(stage.join(name), bytes)
                .map_err(io_error("write region source artifact", stage))?;
            record.outputs.push(DigestEntry {
                identity: name.into(),
                sha256: sha256(bytes),
            });
        }
        record.inputs.push(DigestEntry {
            identity: format!("kernel-region-source:{}", region.record.source),
            sha256: region.record.source_sha256.clone(),
        });
        record
            .inputs
            .extend(region.record.modules.iter().map(|module| DigestEntry {
                identity: format!("kernel-region-source-module:{}", module.source),
                sha256: module.sha256.clone(),
            }));
        record.region_library = Some(region.record);
        record.region_runtime = region_runtime;
        record.inputs.sort_by(|a, b| a.identity.cmp(&b.identity));
        record.outputs.sort_by(|a, b| a.identity.cmp(&b.identity));
    }
    if let Some(library) = library {
        for (name, bytes) in [
            ("kernel-library.t", library.source.as_bytes()),
            ("kernel-library.ll", library.llvm.as_bytes()),
            ("kernel-library.o", library.object.as_slice()),
            (
                "kernel-library.modules.json",
                library.module_snapshot.as_slice(),
            ),
        ] {
            fs::write(stage.join(name), bytes)
                .map_err(io_error("write source library artifact", stage))?;
            record.outputs.push(DigestEntry {
                identity: name.into(),
                sha256: sha256(bytes),
            });
        }
        record.inputs.push(DigestEntry {
            identity: format!("kernel-source:{}", library.record.source),
            sha256: library.record.source_sha256.clone(),
        });
        record
            .inputs
            .extend(library.record.modules.iter().map(|m| DigestEntry {
                identity: format!("kernel-source-module:{}", m.source),
                sha256: m.sha256.clone(),
            }));
        record.inputs.sort_by(|a, b| a.identity.cmp(&b.identity));
        record.outputs.sort_by(|a, b| a.identity.cmp(&b.identity));
        record.kernel_library = Some(library.record);
    }
    for (numeric, stem, input_prefix) in [
        (numeric, "time-library", "kernel-time"),
        (image_numeric, "image-library", "kernel-image"),
        (dispatch_numeric, "dispatch-library", "kernel-dispatch"),
        (
            identity_numeric,
            "process-identity-library",
            "kernel-process-identity",
        ),
    ] {
        if let Some(numeric) = numeric {
            for (name, bytes) in [
                (format!("{stem}.t"), numeric.source.as_bytes()),
                (format!("{stem}.ll"), numeric.llvm.as_bytes()),
                (
                    format!("{stem}.retained.ll"),
                    numeric.retained_llvm.as_bytes(),
                ),
                (format!("{stem}.o"), numeric.object.as_slice()),
                (
                    format!("{stem}.modules.json"),
                    numeric.module_snapshot.as_slice(),
                ),
            ] {
                fs::write(stage.join(&name), bytes)
                    .map_err(io_error("write numeric library artifact", stage))?;
                record.outputs.push(DigestEntry {
                    identity: name.into(),
                    sha256: sha256(bytes),
                });
            }
            record.inputs.push(DigestEntry {
                identity: format!("{input_prefix}-source:{}", numeric.record.source),
                sha256: numeric.record.source_sha256.clone(),
            });
            record
                .inputs
                .extend(numeric.record.modules.iter().map(|m| DigestEntry {
                    identity: format!("{input_prefix}-source-module:{}", m.source),
                    sha256: m.sha256.clone(),
                }));
            record.inputs.sort_by(|a, b| a.identity.cmp(&b.identity));
            record.outputs.sort_by(|a, b| a.identity.cmp(&b.identity));
            if stem == "time-library" {
                record.time_library = Some(numeric.record);
            } else if stem == "image-library" {
                record.image_library = Some(numeric.record);
            } else if stem == "process-identity-library" {
                record.process_identity_library = Some(numeric.record);
            } else {
                record.dispatch_library = Some(numeric.record);
            }
        }
    }
    if let Some(tail) = tail {
        crate::systems_tail_publication::publish_outputs(
            tail,
            &work.join("tail"),
            stage,
            &mut record,
        )?;
    }
    if let Some(frame) = frame {
        crate::systems_frame_publication::publish_outputs(
            frame,
            &work.join("frame"),
            stage,
            &mut record,
        )?;
    }
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

include!("systems_artifact_static_image.rs");

include!("systems_artifact_elf_plan.rs");

include!("systems_artifact_linked_outputs.rs");

include!("systems_artifact_root_state.rs");

include!("systems_artifact_root_object.rs");

include!("systems_artifact_operations.rs");

include!("systems_artifact_trace.rs");

#[cfg(test)]
mod tests {

    include!("systems_artifact_tests_plans.rs");

    include!("systems_artifact_tests_publication.rs");
}
