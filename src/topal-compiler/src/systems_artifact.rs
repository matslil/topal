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
    X86_SYSTEMS_STATIC_PROCESS_BRK_RESERVE_SYMBOL, X86_SYSTEMS_TRANSLATION_ACTIVATE_SYMBOL,
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

pub const X86_SYSTEMS_ARTIFACT_REVISION: &str = "topal.systems-artifact.x86_64-qemu-pc-q35/178";
pub const X86_SYSTEMS_ROOT_OBJECT_REVISION: &str = "topal.systems-root-object.x86_64/167";
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
    validate_x86_64_initial_user_image(initial_user_image)?;
    if let Some(image) = static_user_image {
        validate_x86_64_static_user_image(image)?;
    }
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
        initial_user_image,
        static_user_image,
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
                        | CompilerSystemsOperation::AtomicWordCreate { .. }
                        | CompilerSystemsOperation::AtomicCompareExchangeEquals { .. }
                        | CompilerSystemsOperation::AtomicLoadEquals { .. }
                )
            }),
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
    let record = artifact_provenance(
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

const STATIC_ELF_PAGE_SIZE: u64 = 4096;
const STATIC_ELF_LEAF_WINDOW_SIZE: u64 = 2 * 1024 * 1024;
const STATIC_ELF_MAX_PROGRAM_HEADERS: u16 = 16;
const STATIC_ELF_MAX_USER_ADDRESS: u64 = 0x0000_8000_0000_0000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StaticElfLoadRole {
    Headers,
    Executable,
    ImmutableData,
    WritableData,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StaticElfIdentity {
    class: u8,
    data_encoding: u8,
    version: u8,
    file_type: u16,
    machine: u16,
}

#[allow(dead_code)] // The complete checked plan is retained for the following mapping increment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StaticElfInterval {
    start: u64,
    end: u64,
}

#[allow(dead_code)] // The complete checked plan is retained for the following mapping increment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StaticElfLoadSegment {
    role: StaticElfLoadRole,
    file: StaticElfInterval,
    memory: StaticElfInterval,
    pages: StaticElfInterval,
    alignment: u64,
}

#[allow(dead_code)] // The complete checked plan is retained for the following mapping increment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StaticElfTlsPlan {
    file: StaticElfInterval,
    memory: StaticElfInterval,
    alignment: u64,
}

#[allow(dead_code)] // The complete checked plan is retained for the following mapping increment.
#[derive(Clone, Debug, Eq, PartialEq)]
struct StaticElfLoadPlan {
    identity: StaticElfIdentity,
    entry: u64,
    segments: Vec<StaticElfLoadSegment>,
    pages: StaticElfInterval,
    page_count: u64,
    tls: Option<StaticElfTlsPlan>,
    relro_pages: Option<StaticElfInterval>,
    stack_executable: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StaticElfPagePermission {
    ReadWriteNoExecute,
    ReadOnlyNoExecute,
    ReadOnlyExecute,
}

#[allow(dead_code)] // Retained for the following affine mapping increment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StaticElfPageRecord {
    virtual_address: u64,
    frame_ordinal: u64,
    source_role: StaticElfLoadRole,
    initial_permission: StaticElfPagePermission,
    entry_permission: StaticElfPagePermission,
}

#[allow(dead_code)] // Retained for the following affine mapping increment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct StaticElfCopyOperation {
    file: StaticElfInterval,
    destination: StaticElfInterval,
}

#[allow(dead_code)] // Retained for the following affine mapping increment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StaticElfConstructionOperation {
    Zero(StaticElfInterval),
    Copy(StaticElfCopyOperation),
}

#[allow(dead_code)] // Retained for the following affine mapping increment.
#[derive(Clone, Debug, Eq, PartialEq)]
struct StaticElfPagePlan {
    frame_count: u64,
    alignment_frames: u64,
    construction: Vec<StaticElfConstructionOperation>,
    pages: Vec<StaticElfPageRecord>,
    entry: u64,
    tls: Option<StaticElfTlsPlan>,
    relro_pages: Option<StaticElfInterval>,
}

/// Construct deterministic clean-room bytes with the qualified static-ELF metadata shape.
///
/// The payload is not an application and exists only for structural qualification.
#[must_use]
pub fn qualified_x86_64_static_user_image_fixture() -> Vec<u8> {
    fn put_u16(image: &mut [u8], offset: usize, value: u16) {
        image[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn put_u32(image: &mut [u8], offset: usize, value: u32) {
        image[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    fn put_u64(image: &mut [u8], offset: usize, value: u64) {
        image[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }
    fn put_program_header(
        image: &mut [u8],
        index: usize,
        row: (u32, u32, u64, u64, u64, u64, u64),
    ) {
        let (kind, flags, file_offset, virtual_address, file_size, memory_size, alignment) = row;
        let offset = 64 + index * 56;
        put_u32(image, offset, kind);
        put_u32(image, offset + 4, flags);
        put_u64(image, offset + 8, file_offset);
        put_u64(image, offset + 16, virtual_address);
        put_u64(image, offset + 24, virtual_address);
        put_u64(image, offset + 32, file_size);
        put_u64(image, offset + 40, memory_size);
        put_u64(image, offset + 48, alignment);
    }

    let mut image = vec![0_u8; 2_024_544];
    image[0..16].copy_from_slice(&[0x7f, b'E', b'L', b'F', 2, 1, 1, 3, 0, 0, 0, 0, 0, 0, 0, 0]);
    put_u16(&mut image, 16, 2);
    put_u16(&mut image, 18, 62);
    put_u32(&mut image, 20, 1);
    put_u64(&mut image, 24, 0x004a_6580);
    put_u64(&mut image, 32, 64);
    put_u16(&mut image, 52, 64);
    put_u16(&mut image, 54, 56);
    put_u16(&mut image, 56, 11);
    for (index, row) in [
        (1, 4, 0, 0x0040_0000, 0x6e0, 0x6e0, 0x1000),
        (1, 5, 0x1000, 0x0040_1000, 0x18_b285, 0x18_b285, 0x1000),
        (1, 4, 0x18_d000, 0x0058_d000, 0x5_7040, 0x5_7040, 0x1000),
        (1, 6, 0x1e_4660, 0x005e_5660, 0x9728, 0x1_0b68, 0x1000),
        (4, 4, 0x2a8, 0x0040_02a8, 0x20, 0x20, 8),
        (4, 4, 0x2c8, 0x0040_02c8, 0x24, 0x24, 4),
        (4, 4, 0x1e_4020, 0x005e_4020, 0x20, 0x20, 4),
        (7, 4, 0x1e_4660, 0x005e_5660, 0x40, 0x82, 8),
        (0x6474_e553, 4, 0x2a8, 0x0040_02a8, 0x20, 0x20, 8),
        (0x6474_e551, 6, 0, 0, 0, 0, 0x10),
        (0x6474_e552, 4, 0x1e_4660, 0x005e_5660, 0x79a0, 0x79a0, 1),
    ]
    .into_iter()
    .enumerate()
    {
        put_program_header(&mut image, index, row);
    }
    let entry_offset = 0x0a_6580;
    let entry_code = [
        0xb8, 0x0c, 0, 0, 0, // brk
        0x31, 0xff, // request = 0
        0x0f, 0x05, // syscall
        0x48, 0x3d, 0x00, 0x70, 0x5f, 0x00, // derived initial break
        0x75, 0xfe, // fail closed on a different query result
        0xbf, 0x80, 0x7d, 0x5f, 0x00, // exact growth request
        0xb8, 0x0c, 0, 0, 0, // brk
        0x0f, 0x05, // syscall
        0x48, 0x3d, 0x80, 0x7d, 0x5f, 0x00, // admitted current break
        0x75, 0xfe, // fail closed on a different growth result
        0xc6, 0x04, 0x25, 0x00, 0x70, 0x5f, 0x00, 0x5a, // first admitted byte
        0x80, 0x3c, 0x25, 0x00, 0x70, 0x5f, 0x00, 0x5a, // read it back
        0x75, 0xfe, // fail closed on inaccessible or changed backing
        0xc6, 0x04, 0x25, 0x00, 0x74, 0x5f, 0x00, 0x6b, // TLS-base byte
        0xbf, 0x02, 0x10, 0x00, 0x00, // ARCH_SET_FS
        0xbe, 0x00, 0x74, 0x5f, 0x00, // selected initial FS base
        0xb8, 0x9e, 0, 0, 0, // arch_prctl
        0x0f, 0x05, // syscall
        0x48, 0x85, 0xc0, // exact zero result
        0x75, 0xfe, // fail closed on rejected installation
        0x64, 0x8a, 0x04, 0x25, 0, 0, 0, 0, // load FS-relative byte zero
        0x3c, 0x6b, // installed base selects the known byte
        0x75, 0xfe, // fail closed on wrong physical FS base
        0xc7, 0x04, 0x25, 0xd0, 0x76, 0x5f, 0x00, 1, 0, 0, 0, // exit word
        0xbf, 0xd0, 0x76, 0x5f, 0x00, // exact candidate
        0xb8, 0xda, 0, 0, 0, // set_tid_address
        0x0f, 0x05, // syscall
        0x48, 0x83, 0xf8, 0x01, // sealed TID one
        0x75, 0xfe, // fail closed on a different result
        0x48, 0xc7, 0x04, 0x25, 0xe0, 0x76, 0x5f, 0x00, 0xe0, 0x76, 0x5f, 0x00, 0x48, 0xc7, 0x04,
        0x25, 0xe8, 0x76, 0x5f, 0x00, 0xe0, 0xff, 0xff, 0xff, 0x48, 0xc7, 0x04, 0x25, 0xf0, 0x76,
        0x5f, 0x00, 0, 0, 0, 0, 0xbf, 0xe0, 0x76, 0x5f, 0x00, // exact robust-list head
        0xbe, 0x18, 0, 0, 0, // native head size
        0xb8, 0x11, 0x01, 0, 0, // set_robust_list
        0x0f, 0x05, // syscall
        0x48, 0x85, 0xc0, // exact zero result
        0x75, 0xfe, // fail closed on rejected registration
        0xbf, 0x00, 0x73, 0x5f, 0x00, // observed rseq candidate
        0xbe, 0x21, 0, 0, 0, // observed rseq length
        0x31, 0xd2, // flags = 0
        0x41, 0xba, 0x53, 0x30, 0x05, 0x53, // observed rseq signature
        0xb8, 0x4e, 0x01, 0, 0, // rseq
        0x0f, 0x05, // syscall
        0x48, 0x83, 0xf8, 0xda, // exact -ENOSYS result
        0x75, 0xfe, // fail closed on another result
        0x48, 0x81, 0xec, 0x40, 0x01, 0, 0, // temporary pair below the ABI block
        0x31, 0xff, // pid = self
        0xbe, 0x03, 0, 0, 0, // RLIMIT_STACK
        0x31, 0xd2, // no new limit
        0x49, 0x89, 0xe2, // old limit = temporary pair
        0xb8, 0x2e, 0x01, 0, 0, // prlimit64
        0x0f, 0x05, // syscall
        0x48, 0x85, 0xc0, // exact zero result
        0x75, 0xfe, // fail closed on rejection
        0x48, 0x81, 0x3c, 0x24, 0x00, 0x00, 0x80, 0x00, // 8 MiB soft limit
        0x75, 0xfe, // fail closed on wrong soft limit
        0x48, 0x83, 0x7c, 0x24, 0x08, 0xff, // infinite hard limit
        0x75, 0xfe, // fail closed on wrong hard limit
        0x48, 0x89, 0x04, 0x24, // clear soft-limit scratch
        0x48, 0x89, 0x44, 0x24, 0x08, // clear hard-limit scratch
        0x48, 0x81, 0xc4, 0x40, 0x01, 0, 0, // restore initial stack pointer
        0xbf, 0x9c, 0xff, 0xff, 0xff, // AT_FDCWD
        0xbe, 0xda, 0xfd, 0x5b, 0x00, // observed path candidate
        0xba, 0x00, 0xe0, 0x5f, 0x00, // bounded output candidate
        0x41, 0xba, 0x00, 0x10, 0, 0, // observed output length
        0xb8, 0x0b, 0x01, 0, 0, // readlinkat
        0x0f, 0x05, // syscall
        0x48, 0x83, 0xf8, 0xda, // exact -ENOSYS result
        0x75, 0xfe, // fail closed on another result
        0xbf, 0x68, 0x5e, 0x5f, 0x00, // bounded random output candidate
        0xbe, 0x08, 0, 0, 0, // eight requested bytes
        0xba, 0x01, 0, 0, 0, // GRND_NONBLOCK
        0xb8, 0x3e, 0x01, 0, 0, // getrandom
        0x0f, 0x05, // syscall
        0x48, 0x83, 0xf8, 0x08, // exact eight-byte result
        0x75, 0xfe, // fail closed on another result
        0x4c, 0x8b, 0x14, 0x25, 0x68, 0x5e, 0x5f, 0x00, // observe opaque result
        0x48, 0xc7, 0x04, 0x25, 0x68, 0x5e, 0x5f, 0x00, 0, 0, 0, 0, // clear fixture scratch
        0xbf, 0x70, 0x5e, 0x5f, 0x00, // second bounded output candidate
        0xbe, 0x08, 0, 0, 0, // eight requested bytes
        0xba, 0x01, 0, 0, 0, // GRND_NONBLOCK
        0xb8, 0x3e, 0x01, 0, 0, // getrandom
        0x0f, 0x05, // syscall
        0x48, 0x83, 0xf8, 0x08, // exact eight-byte result
        0x75, 0xfe, // fail closed on another result
        0x4c, 0x8b, 0x14, 0x25, 0x70, 0x5e, 0x5f, 0x00, // observe fresh opaque result
        0x48, 0xc7, 0x04, 0x25, 0x70, 0x5e, 0x5f, 0x00, 0, 0, 0, 0, // clear fixture scratch
        0xb8, 0x0c, 0, 0, 0, // post-random brk query
        0x31, 0xff, // request = 0
        0x0f, 0x05, // syscall
        0x48, 0x3d, 0x80, 0x7d, 0x5f, 0x00, // established current break
        0x75, 0xfe, // fail closed on a different query result
        0xbf, 0x80, 0x8d, 0x61, 0x00, // exact post-random growth request
        0xb8, 0x0c, 0, 0, 0, // brk
        0x0f, 0x05, // syscall
        0x48, 0x3d, 0x80, 0x8d, 0x61, 0x00, // admitted current break
        0x75, 0xfe, // fail closed on a different growth result
        0xc6, 0x04, 0x25, 0x7f, 0x8d, 0x61, 0x00, 0x6d, // final admitted byte
        0x80, 0x3c, 0x25, 0x7f, 0x8d, 0x61, 0x00, 0x6d, // read it back
        0x75, 0xfe, // fail closed on inaccessible or changed backing
        0xbf, 0x00, 0x90, 0x61, 0x00, // exact page-alignment growth request
        0xb8, 0x0c, 0, 0, 0, // brk
        0x0f, 0x05, // syscall
        0x48, 0x3d, 0x00, 0x90, 0x61, 0x00, // aligned current break
        0x75, 0xfe, // fail closed on a different growth result
        0xc6, 0x04, 0x25, 0xff, 0x8f, 0x61, 0x00, 0x71, // final newly admitted byte
        0x80, 0x3c, 0x25, 0xff, 0x8f, 0x61, 0x00, 0x71, // read it back
        0x75, 0xfe, // fail closed on inaccessible or changed backing
        0xbf, 0x00, 0x50, 0x5e, 0x00, // exact GNU RELRO start
        0xbe, 0x00, 0x80, 0x00, 0x00, // exact eight-page length
        0xba, 0x01, 0x00, 0x00, 0x00, // PROT_READ
        0xb8, 0x0a, 0x00, 0x00, 0x00, // mprotect
        0x0f, 0x05, // syscall
        0x48, 0x85, 0xc0, // exact zero result
        0x75, 0xfe, // fail closed on rejection
        0x48, 0x81, 0xec, 0x40, 0x01, 0, 0, // qualified temporary stack area
        0xbf, 0x10, 0x00, 0x00, 0x00, // PR_GET_NAME
        0x48, 0x89, 0xe6, // exact output candidate
        0xb8, 0x9d, 0x00, 0x00, 0x00, // prctl
        0x0f, 0x05, // syscall
        0x48, 0x85, 0xc0, // exact zero result
        0x75, 0xfe, // fail closed on rejection
        0x48, 0xb8, 0x62, 0x75, 0x73, 0x79, 0x62, 0x6f, 0x78, 0x00, // "busybox\0"
        0x48, 0x39, 0x04, 0x24, // exact first word
        0x75, 0xfe, 0x48, 0x83, 0x7c, 0x24, 0x08, 0x00, // zero-padded second word
        0x75, 0xfe, 0x48, 0xc7, 0x04, 0x24, 0, 0, 0, 0, // clear first scratch word
        0x48, 0xc7, 0x44, 0x24, 0x08, 0, 0, 0, 0, // clear second scratch word
        0x48, 0x81, 0xc4, 0x40, 0x01, 0, 0, // restore initial stack pointer
        0xb8, 0x66, 0x00, 0x00, 0x00, // getuid
        0x0f, 0x05, // syscall
        0x48, 0x85, 0xc0, // exact zero result
        0x75, 0xfe, // fail closed on another result
        0x48, 0x81, 0xec, 0x20, 0x02, 0x00, 0x00, // scratch below initial vector
        0xbf, 0x9c, 0xff, 0xff, 0xff, // AT_FDCWD
        0xbe, 0x00, 0x69, 0x4a, 0x00, // immutable configuration path
        0x48, 0x89, 0xe2, // complete native stat destination
        0x45, 0x31, 0xd2, // flags = 0
        0xb8, 0x06, 0x01, 0x00, 0x00, // newfstatat
        0x0f, 0x05, // syscall
        0x48, 0x83, 0xf8, 0xfe, // exact -ENOENT result
        0x75, 0xfe, // fail closed on another result
        0x48, 0x81, 0xc4, 0x20, 0x02, 0x00, 0x00, // restore initial stack pointer
        0xb8, 0x68, 0x00, 0x00, 0x00, // getgid
        0x0f, 0x05, 0x48, 0x85, 0xc0, 0x75, 0xfe, // require zero
        0x31, 0xff, // exact sealed group identity
        0xb8, 0x6a, 0x00, 0x00, 0x00, // setgid
        0x0f, 0x05, 0x48, 0x85, 0xc0, 0x75, 0xfe, // require zero
        0x31, 0xff, // exact sealed user identity
        0xb8, 0x69, 0x00, 0x00, 0x00, // setuid
        0x0f, 0x05, 0x48, 0x85, 0xc0, 0x75, 0xfe, // require zero
        0x48, 0x81, 0xec, 0x40, 0x02, 0x00, 0x00, // disjoint stack-prefix scratch
        0xc6, 0x04, 0x24, 0x0a, // line feed
        0xbf, 0x01, 0x00, 0x00, 0x00, // stdout
        0x48, 0x89, 0xe6, // candidate
        0xba, 0x01, 0x00, 0x00, 0x00, // length one
        0xb8, 0x01, 0x00, 0x00, 0x00, // write
        0x0f, 0x05, 0x48, 0x83, 0xf8, 0x01, 0x75, 0xfe, // require one
        0xc6, 0x04, 0x24, 0x00, // clear line feed
        0x48, 0x81, 0xc4, 0x40, 0x02, 0x00, 0x00, // restore initial stack pointer
        0x31, 0xff, // status = 0
        0xb8, 0xe7, 0, 0, 0, // exit_group
        0x0f, 0x05, // syscall
        0x0f, 0x0b, // unreachable
    ];
    image[entry_offset..entry_offset + entry_code.len()].copy_from_slice(&entry_code);
    image[0x0a_6900..0x0a_6912].copy_from_slice(b"/etc/busybox.conf\0");
    image
}

fn elf_u16(image: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(
        image[offset..offset + 2]
            .try_into()
            .expect("validated ELF header extent"),
    )
}

fn elf_u32(image: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        image[offset..offset + 4]
            .try_into()
            .expect("validated ELF header extent"),
    )
}

fn checked_elf_interval(
    start: u64,
    size: u64,
    description: &str,
) -> Result<StaticElfInterval, CompileError> {
    let end = start.checked_add(size).ok_or_else(|| {
        CompileError::Tool(format!(
            "bounded static ELF {description} interval overflows"
        ))
    })?;
    Ok(StaticElfInterval { start, end })
}

fn page_floor(value: u64) -> u64 {
    value & !(STATIC_ELF_PAGE_SIZE - 1)
}

fn page_ceil(value: u64, description: &str) -> Result<u64, CompileError> {
    value
        .checked_add(STATIC_ELF_PAGE_SIZE - 1)
        .map(page_floor)
        .ok_or_else(|| {
            CompileError::Tool(format!(
                "bounded static ELF {description} page interval overflows"
            ))
        })
}

#[allow(clippy::too_many_lines)] // Keeps one fail-closed validation transaction auditable.
fn derive_x86_64_static_elf_load_plan(image: &[u8]) -> Result<StaticElfLoadPlan, CompileError> {
    if image.len() < 64 {
        return Err(CompileError::Tool(
            "bounded static ELF requires a complete 64-byte ELF header".into(),
        ));
    }
    if image[0..4] != [0x7f, b'E', b'L', b'F']
        || image[4] != 2
        || image[5] != 1
        || image[6] != 1
        || elf_u16(image, 16) != 2
        || elf_u16(image, 18) != 62
        || elf_u32(image, 20) != 1
        || elf_u16(image, 52) != 64
        || elf_u16(image, 54) != 56
    {
        return Err(CompileError::Tool(
            "bounded static ELF requires current little-endian ELF64 ET_EXEC/EM_X86_64 identity and canonical header sizes"
                .into(),
        ));
    }

    let entry = image_u64(image, 24);
    let program_header_offset = image_u64(image, 32);
    let program_header_count = elf_u16(image, 56);
    if program_header_count == 0 || program_header_count > STATIC_ELF_MAX_PROGRAM_HEADERS {
        return Err(CompileError::Tool(format!(
            "bounded static ELF program-header count must be 1..={STATIC_ELF_MAX_PROGRAM_HEADERS}"
        )));
    }
    let table_bytes = u64::from(program_header_count)
        .checked_mul(56)
        .ok_or_else(|| CompileError::Tool("bounded static ELF table size overflows".into()))?;
    let table = checked_elf_interval(program_header_offset, table_bytes, "program-header table")?;
    if table.start < 64 || table.end > image.len() as u64 {
        return Err(CompileError::Tool(
            "bounded static ELF program-header table lies outside the complete input".into(),
        ));
    }

    let roles = [
        StaticElfLoadRole::Headers,
        StaticElfLoadRole::Executable,
        StaticElfLoadRole::ImmutableData,
        StaticElfLoadRole::WritableData,
    ];
    let expected_flags = [4_u32, 5, 4, 6];
    let mut segments = Vec::new();
    let mut tls = None;
    let mut relro_pages = None;
    let mut stack_executable = None;

    for index in 0..usize::from(program_header_count) {
        let offset = usize::try_from(program_header_offset).map_err(|_| {
            CompileError::Tool("bounded static ELF table offset exceeds host limits".into())
        })? + index * 56;
        let kind = elf_u32(image, offset);
        let flags = elf_u32(image, offset + 4);
        let file_offset = image_u64(image, offset + 8);
        let virtual_address = image_u64(image, offset + 16);
        let file_size = image_u64(image, offset + 32);
        let memory_size = image_u64(image, offset + 40);
        let alignment = image_u64(image, offset + 48);

        match kind {
            1 => {
                let segment_index = segments.len();
                if segment_index >= roles.len() || flags != expected_flags[segment_index] {
                    return Err(CompileError::Tool(
                        "bounded static ELF requires exactly ordered R, R/X, R, and R/W load roles without W+X"
                            .into(),
                    ));
                }
                if memory_size == 0 || file_size > memory_size {
                    return Err(CompileError::Tool(
                        "bounded static ELF load sizes require 0 < p_memsz and p_filesz <= p_memsz"
                            .into(),
                    ));
                }
                if alignment < STATIC_ELF_PAGE_SIZE || !alignment.is_power_of_two() {
                    return Err(CompileError::Tool(
                        "bounded static ELF load alignment must be a power of two at least 4096"
                            .into(),
                    ));
                }
                if file_offset % STATIC_ELF_PAGE_SIZE != virtual_address % STATIC_ELF_PAGE_SIZE {
                    return Err(CompileError::Tool(
                        "bounded static ELF load file and virtual page offsets are incongruent"
                            .into(),
                    ));
                }
                let file = checked_elf_interval(file_offset, file_size, "load file")?;
                let memory = checked_elf_interval(virtual_address, memory_size, "load memory")?;
                if file.end > image.len() as u64
                    || memory.start < STATIC_ELF_PAGE_SIZE
                    || memory.end > STATIC_ELF_MAX_USER_ADDRESS
                {
                    return Err(CompileError::Tool(
                        "bounded static ELF load interval lies outside its file or user envelope"
                            .into(),
                    ));
                }
                let pages = StaticElfInterval {
                    start: page_floor(memory.start),
                    end: page_ceil(memory.end, "load memory")?,
                };
                if segments
                    .last()
                    .is_some_and(|prior: &StaticElfLoadSegment| prior.pages.end > pages.start)
                {
                    return Err(CompileError::Tool(
                        "bounded static ELF load page intervals overlap or are unordered".into(),
                    ));
                }
                segments.push(StaticElfLoadSegment {
                    role: roles[segment_index],
                    file,
                    memory,
                    pages,
                    alignment,
                });
            }
            2 | 3 => {
                return Err(CompileError::Tool(
                    "bounded static ELF does not admit PT_DYNAMIC or PT_INTERP".into(),
                ));
            }
            7 => {
                if tls.is_some() || memory_size == 0 || file_size > memory_size {
                    return Err(CompileError::Tool(
                        "bounded static ELF admits at most one nonempty valid PT_TLS".into(),
                    ));
                }
                if alignment == 0 || !alignment.is_power_of_two() {
                    return Err(CompileError::Tool(
                        "bounded static ELF TLS alignment must be a nonzero power of two".into(),
                    ));
                }
                let file = checked_elf_interval(file_offset, file_size, "TLS file")?;
                let memory = checked_elf_interval(virtual_address, memory_size, "TLS memory")?;
                if file.end > image.len() as u64 {
                    return Err(CompileError::Tool(
                        "bounded static ELF TLS file interval exceeds the complete input".into(),
                    ));
                }
                tls = Some(StaticElfTlsPlan {
                    file,
                    memory,
                    alignment,
                });
            }
            0x6474_e551 => {
                if stack_executable.replace(flags & 1 != 0).is_some() {
                    return Err(CompileError::Tool(
                        "bounded static ELF requires exactly one PT_GNU_STACK".into(),
                    ));
                }
            }
            0x6474_e552 => {
                if relro_pages.is_some() || memory_size == 0 {
                    return Err(CompileError::Tool(
                        "bounded static ELF admits at most one nonempty PT_GNU_RELRO".into(),
                    ));
                }
                let memory = checked_elf_interval(virtual_address, memory_size, "RELRO memory")?;
                relro_pages = Some(StaticElfInterval {
                    start: page_floor(memory.start),
                    end: page_ceil(memory.end, "RELRO memory")?,
                });
            }
            _ => {}
        }
    }

    if segments.len() != roles.len() {
        return Err(CompileError::Tool(
            "bounded static ELF requires exactly four PT_LOAD entries".into(),
        ));
    }
    let executable = segments[1];
    if entry < executable.memory.start || entry >= executable.memory.end {
        return Err(CompileError::Tool(
            "bounded static ELF entry lies outside the R/X load interval".into(),
        ));
    }
    let writable = segments[3];
    if let Some(tls_plan) = tls
        && (tls_plan.file.start < writable.file.start
            || tls_plan.file.end > writable.file.end
            || tls_plan.memory.start < writable.memory.start
            || tls_plan.memory.end > writable.memory.end)
    {
        return Err(CompileError::Tool(
            "bounded static ELF TLS intervals lie outside the R/W load role".into(),
        ));
    }
    if let Some(relro) = relro_pages
        && (relro.start < writable.pages.start
            || relro.end > writable.pages.end
            || relro.start == relro.end)
    {
        return Err(CompileError::Tool(
            "bounded static ELF RELRO pages lie outside the R/W load role".into(),
        ));
    }
    let Some(stack_executable) = stack_executable else {
        return Err(CompileError::Tool(
            "bounded static ELF requires exactly one PT_GNU_STACK".into(),
        ));
    };
    if stack_executable {
        return Err(CompileError::Tool(
            "bounded static ELF requires a non-executable stack".into(),
        ));
    }

    let pages = StaticElfInterval {
        start: segments[0].pages.start,
        end: segments[3].pages.end,
    };
    let page_count = (pages.end - pages.start) / STATIC_ELF_PAGE_SIZE;
    if page_count == 0
        || page_count > 512
        || page_floor(pages.start) / STATIC_ELF_LEAF_WINDOW_SIZE
            != (pages.end - 1) / STATIC_ELF_LEAF_WINDOW_SIZE
    {
        return Err(CompileError::Tool(
            "bounded static ELF load span exceeds one 512-leaf 2 MiB window".into(),
        ));
    }

    Ok(StaticElfLoadPlan {
        identity: StaticElfIdentity {
            class: image[4],
            data_encoding: image[5],
            version: image[6],
            file_type: elf_u16(image, 16),
            machine: elf_u16(image, 18),
        },
        entry,
        segments,
        pages,
        page_count,
        tls,
        relro_pages,
        stack_executable,
    })
}

#[allow(clippy::too_many_lines)] // Keeps one inherited-plan validation transaction auditable.
fn derive_x86_64_static_elf_page_plan(
    load: &StaticElfLoadPlan,
) -> Result<StaticElfPagePlan, CompileError> {
    if load.segments.len() != 4
        || load.page_count == 0
        || load.page_count > 512
        || !load.pages.start.is_multiple_of(STATIC_ELF_PAGE_SIZE)
        || !load.pages.end.is_multiple_of(STATIC_ELF_PAGE_SIZE)
        || load.pages.end <= load.pages.start
        || (load.pages.end - load.pages.start) / STATIC_ELF_PAGE_SIZE != load.page_count
    {
        return Err(CompileError::Tool(
            "bounded static ELF page realization has an inconsistent inherited span".into(),
        ));
    }

    let expected_roles = [
        StaticElfLoadRole::Headers,
        StaticElfLoadRole::Executable,
        StaticElfLoadRole::ImmutableData,
        StaticElfLoadRole::WritableData,
    ];
    let mut next_page = load.pages.start;
    for (segment, expected_role) in load.segments.iter().zip(expected_roles) {
        if segment.role != expected_role
            || segment.pages.start != next_page
            || !segment.pages.start.is_multiple_of(STATIC_ELF_PAGE_SIZE)
            || !segment.pages.end.is_multiple_of(STATIC_ELF_PAGE_SIZE)
            || segment.pages.end <= segment.pages.start
            || segment.memory.start < segment.pages.start
            || segment.memory.end > segment.pages.end
            || segment.file.end < segment.file.start
            || segment.memory.end < segment.memory.start
            || segment.file.end - segment.file.start > segment.memory.end - segment.memory.start
        {
            return Err(CompileError::Tool(
                "bounded static ELF page realization requires four exact adjacent inherited roles"
                    .into(),
            ));
        }
        next_page = segment.pages.end;
    }
    if next_page != load.pages.end {
        return Err(CompileError::Tool(
            "bounded static ELF page realization does not cover the complete inherited span".into(),
        ));
    }

    let writable = load.segments[3];
    if let Some(relro) = load.relro_pages
        && (!relro.start.is_multiple_of(STATIC_ELF_PAGE_SIZE)
            || !relro.end.is_multiple_of(STATIC_ELF_PAGE_SIZE)
            || relro.start >= relro.end
            || relro.start < writable.pages.start
            || relro.end > writable.pages.end)
    {
        return Err(CompileError::Tool(
            "bounded static ELF page realization has invalid inherited RELRO pages".into(),
        ));
    }

    let mut pages = Vec::with_capacity(usize::try_from(load.page_count).map_err(|_| {
        CompileError::Tool("bounded static ELF page count exceeds host limits".into())
    })?);
    for segment in &load.segments {
        let mut virtual_address = segment.pages.start;
        while virtual_address < segment.pages.end {
            let entry_permission = match segment.role {
                StaticElfLoadRole::Executable => StaticElfPagePermission::ReadOnlyExecute,
                StaticElfLoadRole::WritableData => StaticElfPagePermission::ReadWriteNoExecute,
                StaticElfLoadRole::Headers | StaticElfLoadRole::ImmutableData => {
                    StaticElfPagePermission::ReadOnlyNoExecute
                }
            };
            pages.push(StaticElfPageRecord {
                virtual_address,
                frame_ordinal: (virtual_address - load.pages.start) / STATIC_ELF_PAGE_SIZE,
                source_role: segment.role,
                initial_permission: StaticElfPagePermission::ReadWriteNoExecute,
                entry_permission,
            });
            virtual_address += STATIC_ELF_PAGE_SIZE;
        }
    }
    let derived_page_count = u64::try_from(pages.len()).map_err(|_| {
        CompileError::Tool("bounded static ELF realized page count exceeds u64".into())
    })?;
    if derived_page_count != load.page_count
        || pages.iter().enumerate().any(|(index, page)| {
            let ordinal = u64::try_from(index).expect("bounded to 512 page records");
            page.frame_ordinal != ordinal
                || page.virtual_address != load.pages.start + ordinal * STATIC_ELF_PAGE_SIZE
        })
    {
        return Err(CompileError::Tool(
            "bounded static ELF page realization has incomplete page ordinals".into(),
        ));
    }

    let mut construction = Vec::with_capacity(5);
    construction.push(StaticElfConstructionOperation::Zero(load.pages));
    for segment in &load.segments {
        let copy_size = segment.file.end - segment.file.start;
        let destination =
            checked_elf_interval(segment.memory.start, copy_size, "copy destination")?;
        if destination.end > segment.memory.end {
            return Err(CompileError::Tool(
                "bounded static ELF copy exceeds its inherited memory role".into(),
            ));
        }
        construction.push(StaticElfConstructionOperation::Copy(
            StaticElfCopyOperation {
                file: segment.file,
                destination,
            },
        ));
    }

    Ok(StaticElfPagePlan {
        frame_count: load.page_count,
        alignment_frames: 1,
        construction,
        pages,
        entry: load.entry,
        tls: load.tls,
        relro_pages: load.relro_pages,
    })
}

fn validate_x86_64_static_user_image(image: &[u8]) -> Result<(), CompileError> {
    const EXPECTED_FILE_BYTES: usize = 2_024_544;
    const EXPECTED_ENTRY: u64 = 0x004a_6580;
    let load = derive_x86_64_static_elf_load_plan(image)?;
    let pages = derive_x86_64_static_elf_page_plan(&load)?;
    let expected_segments = [
        StaticElfLoadSegment {
            role: StaticElfLoadRole::Headers,
            file: StaticElfInterval {
                start: 0,
                end: 0x6e0,
            },
            memory: StaticElfInterval {
                start: 0x0040_0000,
                end: 0x0040_06e0,
            },
            pages: StaticElfInterval {
                start: 0x0040_0000,
                end: 0x0040_1000,
            },
            alignment: 0x1000,
        },
        StaticElfLoadSegment {
            role: StaticElfLoadRole::Executable,
            file: StaticElfInterval {
                start: 0x1000,
                end: 0x18_c285,
            },
            memory: StaticElfInterval {
                start: 0x0040_1000,
                end: 0x0058_c285,
            },
            pages: StaticElfInterval {
                start: 0x0040_1000,
                end: 0x0058_d000,
            },
            alignment: 0x1000,
        },
        StaticElfLoadSegment {
            role: StaticElfLoadRole::ImmutableData,
            file: StaticElfInterval {
                start: 0x18_d000,
                end: 0x1e_4040,
            },
            memory: StaticElfInterval {
                start: 0x0058_d000,
                end: 0x005e_4040,
            },
            pages: StaticElfInterval {
                start: 0x0058_d000,
                end: 0x005e_5000,
            },
            alignment: 0x1000,
        },
        StaticElfLoadSegment {
            role: StaticElfLoadRole::WritableData,
            file: StaticElfInterval {
                start: 0x1e_4660,
                end: 0x1e_dd88,
            },
            memory: StaticElfInterval {
                start: 0x005e_5660,
                end: 0x005f_61c8,
            },
            pages: StaticElfInterval {
                start: 0x005e_5000,
                end: 0x005f_7000,
            },
            alignment: 0x1000,
        },
    ];
    let expected_tls = Some(StaticElfTlsPlan {
        file: StaticElfInterval {
            start: 0x1e_4660,
            end: 0x1e_46a0,
        },
        memory: StaticElfInterval {
            start: 0x005e_5660,
            end: 0x005e_56e2,
        },
        alignment: 8,
    });
    let expected_relro = Some(StaticElfInterval {
        start: 0x005e_5000,
        end: 0x005e_d000,
    });
    let expected_program_headers = [
        (1, 4, 0, 0x0040_0000, 0x6e0, 0x6e0, 0x1000),
        (1, 5, 0x1000, 0x0040_1000, 0x18_b285, 0x18_b285, 0x1000),
        (1, 4, 0x18_d000, 0x0058_d000, 0x5_7040, 0x5_7040, 0x1000),
        (1, 6, 0x1e_4660, 0x005e_5660, 0x9728, 0x1_0b68, 0x1000),
        (4, 4, 0x2a8, 0x0040_02a8, 0x20, 0x20, 8),
        (4, 4, 0x2c8, 0x0040_02c8, 0x24, 0x24, 4),
        (4, 4, 0x1e_4020, 0x005e_4020, 0x20, 0x20, 4),
        (7, 4, 0x1e_4660, 0x005e_5660, 0x40, 0x82, 8),
        (0x6474_e553, 4, 0x2a8, 0x0040_02a8, 0x20, 0x20, 8),
        (0x6474_e551, 6, 0, 0, 0, 0, 0x10),
        (0x6474_e552, 4, 0x1e_4660, 0x005e_5660, 0x79a0, 0x79a0, 1),
    ];
    let exact_program_headers = expected_program_headers.iter().enumerate().all(
        |(index, &(kind, flags, offset, virtual_address, file_size, memory_size, alignment))| {
            let header = 64 + index * 56;
            elf_u32(image, header) == kind
                && elf_u32(image, header + 4) == flags
                && image_u64(image, header + 8) == offset
                && image_u64(image, header + 16) == virtual_address
                && image_u64(image, header + 24) == virtual_address
                && image_u64(image, header + 32) == file_size
                && image_u64(image, header + 40) == memory_size
                && image_u64(image, header + 48) == alignment
        },
    );
    let section_header_shape = match (
        image_u64(image, 40),
        elf_u16(image, 58),
        elf_u16(image, 60),
        elf_u16(image, 62),
    ) {
        (0, 0, 0, 0) => true,
        (0x1e_dea0, 64, 23, 22) => {
            let directory_bytes = 64_usize.checked_mul(23);
            let directory_end =
                directory_bytes.and_then(|bytes| 0x1e_dea0_usize.checked_add(bytes));
            directory_end == Some(image.len())
        }
        _ => false,
    };

    if image.len() != EXPECTED_FILE_BYTES
        || image_u64(image, 32) != 64
        || elf_u16(image, 56) != 11
        || !section_header_shape
        || !exact_program_headers
        || load.entry != EXPECTED_ENTRY
        || load.segments.as_slice() != expected_segments
        || load.pages
            != (StaticElfInterval {
                start: 0x0040_0000,
                end: 0x005f_7000,
            })
        || load.page_count != 503
        || load.tls != expected_tls
        || load.relro_pages != expected_relro
        || load.stack_executable
        || pages.frame_count != 503
        || pages.entry != EXPECTED_ENTRY
    {
        return Err(CompileError::Tool(
            "static user image does not match the approved 503-page x86-64 shape".into(),
        ));
    }
    Ok(())
}

fn validate_x86_64_initial_user_image(image: &[u8]) -> Result<(), CompileError> {
    let reference = initial_user_elf_file()?;
    if image.len() != reference.len() {
        if let Ok(load_plan) = derive_x86_64_static_elf_load_plan(image) {
            let page_plan = derive_x86_64_static_elf_page_plan(&load_plan)?;
            return Err(CompileError::Tool(format!(
                "bounded static ELF page plan covers one {}-frame affine extent at entry {:#x}, but this image is not runtime-selectable until its mapping lifecycle is implemented",
                page_plan.frame_count, page_plan.entry
            )));
        }
        return Err(CompileError::Tool(format!(
            "initial user image has {} bytes; the qualified x86-64 profile requires {}",
            image.len(),
            reference.len()
        )));
    }

    let variable_header_ranges = [
        INITIAL_USER_ENTRY_ELF_OFFSET..INITIAL_USER_ENTRY_ELF_OFFSET + 8,
        INITIAL_USER_CODE_FILE_SIZE_ELF_OFFSET..INITIAL_USER_CODE_FILE_SIZE_ELF_OFFSET + 8,
        INITIAL_USER_CODE_MEMORY_SIZE_ELF_OFFSET..INITIAL_USER_CODE_MEMORY_SIZE_ELF_OFFSET + 8,
        INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET..INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET + 8,
        INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET..INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET + 8,
    ];
    for offset in 0..256 {
        if variable_header_ranges
            .iter()
            .any(|range| range.contains(&offset))
        {
            continue;
        }
        if image[offset] != reference[offset] {
            return Err(CompileError::Tool(format!(
                "initial user image changes immutable x86-64 ELF metadata at byte {offset}"
            )));
        }
    }

    let code_file_size = image_u64(image, INITIAL_USER_CODE_FILE_SIZE_ELF_OFFSET);
    let code_memory_size = image_u64(image, INITIAL_USER_CODE_MEMORY_SIZE_ELF_OFFSET);
    if code_file_size != code_memory_size
        || !(4360..=8192).contains(&code_file_size)
        || code_file_size % 8 != 0
    {
        return Err(CompileError::Tool(
            "initial user executable extent must be equal file/memory sizes of 4360..8192 bytes aligned to 8"
                .into(),
        ));
    }

    let entry = image_u64(image, INITIAL_USER_ENTRY_ELF_OFFSET);
    let code_end = INITIAL_USER_CODE_ADDRESS
        .checked_add(code_file_size)
        .ok_or_else(|| CompileError::Tool("initial user executable extent overflows".into()))?;
    if entry < INITIAL_USER_CODE_ADDRESS + 4096 + 256 || entry >= code_end {
        return Err(CompileError::Tool(
            "initial user entry must lie inside the executable payload extent".into(),
        ));
    }

    let data_file_size = image_u64(image, INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET);
    let data_memory_size = image_u64(image, INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET);
    if !(8..=INITIAL_USER_DATA_CAPACITY as u64).contains(&data_file_size)
        || data_file_size % 8 != 0
        || data_memory_size < data_file_size
        || data_memory_size > INITIAL_USER_DATA_CAPACITY as u64
        || data_memory_size % 8 != 0
    {
        return Err(CompileError::Tool(
            "initial user data extent requires an 8-byte-aligned file size of 8..4096 and memory size between it and 4096"
                .into(),
        ));
    }
    if INITIAL_USER_DATA_FILE_OFFSET + INITIAL_USER_DATA_CAPACITY != image.len() {
        return Err(CompileError::Tool(
            "initial user data segment is outside the qualified file envelope".into(),
        ));
    }
    Ok(())
}

fn image_u64(image: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(
        image[offset..offset + 8]
            .try_into()
            .expect("validated ELF extent"),
    )
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
    initial_user_image: &[u8],
    static_user_image: Option<&[u8]>,
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
        DigestEntry {
            identity: SYSTEMS_INITIAL_USER_IMAGE_FILE.into(),
            sha256: sha256(initial_user_image),
        },
    ];
    if let Some(image) = static_user_image {
        outputs.push(DigestEntry {
            identity: SYSTEMS_STATIC_USER_IMAGE_FILE.into(),
            sha256: sha256(image),
        });
    }
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
        exception_entries: vec![
            X86_SYSTEMS_DEBUG_BREAK_ENTRY.into(),
            X86_SYSTEMS_LOCAL_NOTIFICATION_ENTRY.into(),
            X86_SYSTEMS_DEADLINE_ENTRY.into(),
            X86_SYSTEMS_USER_COPY_FAULT_SYMBOL.into(),
        ],
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
    for required in REQUIRED_LINKED_TEXT_SYMBOLS {
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
        X86_SYSTEMS_KERNEL_THREAD_ENTRY,
        X86_SYSTEMS_COMPLETION_THREAD_ENTRY,
        X86_SYSTEMS_TERMINAL_THREAD_ENTRY,
        X86_SYSTEMS_BOOT_MEMORY_SYMBOL,
        X86_SYSTEMS_FRAME_ALLOCATE_SYMBOL,
        X86_SYSTEMS_TRANSLATION_BEGIN_SYMBOL,
        X86_SYSTEMS_TRANSLATION_COMMIT_SYMBOL,
        X86_SYSTEMS_TRANSLATION_ACTIVATE_SYMBOL,
        X86_SYSTEMS_TRANSLATION_EDIT_BEGIN_SYMBOL,
        X86_SYSTEMS_TRANSLATION_EDIT_MAP_SYMBOL,
        X86_SYSTEMS_TRANSLATION_EDIT_UNMAP_SYMBOL,
        X86_SYSTEMS_TRANSLATION_EDIT_COMMIT_SYMBOL,
        X86_SYSTEMS_CRITICAL_ENTER_SYMBOL,
        X86_SYSTEMS_CRITICAL_RESTORE_SYMBOL,
        X86_SYSTEMS_MONOTONIC_CLOCK_NOW_SYMBOL,
        X86_SYSTEMS_CONTEXT_CREATE_SYMBOL,
        X86_SYSTEMS_CONTEXT_TRANSFER_SYMBOL,
        X86_SYSTEMS_CONTEXT_RETIRE_SYMBOL,
        X86_SYSTEMS_CONTEXT_RECLAIM_SYMBOL,
        X86_SYSTEMS_USER_COPY_FAULT_SYMBOL,
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

#[allow(clippy::too_many_lines)] // The closed semantic-to-symbol placement ledger stays auditable together.
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
        (
            "topal.systems.entry.external.local-notification/1",
            X86_SYSTEMS_LOCAL_NOTIFICATION_ENTRY,
        ),
        (
            "topal.systems.entry.resumed.kernel-thread/1",
            X86_SYSTEMS_KERNEL_THREAD_ENTRY,
        ),
        (
            "topal.systems.entry.resumed.completion-thread/1",
            X86_SYSTEMS_COMPLETION_THREAD_ENTRY,
        ),
        (
            "topal.systems.entry.resumed.terminal-thread/1",
            X86_SYSTEMS_TERMINAL_THREAD_ENTRY,
        ),
        (SYSTEMS_BOOT_MEMORY_DESCRIBE, X86_SYSTEMS_BOOT_MEMORY_SYMBOL),
        (SYSTEMS_FRAMES_ALLOCATE, X86_SYSTEMS_FRAME_ALLOCATE_SYMBOL),
        (SYSTEMS_KERNEL_MAP, X86_SYSTEMS_FRAME_ALLOCATE_SYMBOL),
        (SYSTEMS_KERNEL_MAPPING_STORE_BYTE, X86_SYSTEMS_KERNEL_ENTRY),
        (SYSTEMS_KERNEL_MAPPING_LOAD_BYTE, X86_SYSTEMS_KERNEL_ENTRY),
        (SYSTEMS_KERNEL_UNMAP, X86_SYSTEMS_KERNEL_ENTRY),
        (
            SYSTEMS_TRANSLATION_BEGIN,
            X86_SYSTEMS_TRANSLATION_BEGIN_SYMBOL,
        ),
        (
            SYSTEMS_TRANSLATION_COMMIT,
            X86_SYSTEMS_TRANSLATION_COMMIT_SYMBOL,
        ),
        (
            SYSTEMS_TRANSLATION_ACTIVATE,
            X86_SYSTEMS_TRANSLATION_ACTIVATE_SYMBOL,
        ),
        (
            SYSTEMS_TRANSLATION_EDIT_BEGIN,
            X86_SYSTEMS_TRANSLATION_EDIT_BEGIN_SYMBOL,
        ),
        (
            SYSTEMS_TRANSLATION_EDIT_MAP,
            X86_SYSTEMS_TRANSLATION_EDIT_MAP_SYMBOL,
        ),
        (
            SYSTEMS_TRANSLATION_EDIT_UNMAP,
            X86_SYSTEMS_TRANSLATION_EDIT_UNMAP_SYMBOL,
        ),
        (
            SYSTEMS_TRANSLATION_EDIT_COMMIT,
            X86_SYSTEMS_TRANSLATION_EDIT_COMMIT_SYMBOL,
        ),
        (SYSTEMS_CRITICAL_ENTER, X86_SYSTEMS_CRITICAL_ENTER_SYMBOL),
        (
            SYSTEMS_CRITICAL_RESTORE,
            X86_SYSTEMS_CRITICAL_RESTORE_SYMBOL,
        ),
        (SYSTEMS_ATOMIC_WORD_CREATE, X86_SYSTEMS_ATOMIC_CREATE_SYMBOL),
        (
            SYSTEMS_ATOMIC_COMPARE_EXCHANGE,
            X86_SYSTEMS_ATOMIC_COMPARE_EXCHANGE_SYMBOL,
        ),
        (SYSTEMS_ATOMIC_LOAD, X86_SYSTEMS_ATOMIC_LOAD_SYMBOL),
        (SYSTEMS_ATOMIC_END, X86_SYSTEMS_KERNEL_ENTRY),
        (
            SYSTEMS_LOCAL_NOTIFICATION_SEND,
            X86_SYSTEMS_LOCAL_NOTIFICATION_SEND_SYMBOL,
        ),
        (
            SYSTEMS_LOCAL_NOTIFICATION_WAIT,
            X86_SYSTEMS_LOCAL_NOTIFICATION_WAIT_SYMBOL,
        ),
        (
            SYSTEMS_LOCAL_NOTIFICATION_COMPLETE,
            X86_SYSTEMS_LOCAL_NOTIFICATION_COMPLETE_SYMBOL,
        ),
        (
            SYSTEMS_MONOTONIC_CLOCK_NOW,
            X86_SYSTEMS_MONOTONIC_CLOCK_NOW_SYMBOL,
        ),
        (SYSTEMS_DEADLINE_AFTER, X86_SYSTEMS_DEADLINE_AFTER_SYMBOL),
        (SYSTEMS_DEADLINE_ARM, X86_SYSTEMS_DEADLINE_ARM_SYMBOL),
        (SYSTEMS_DEADLINE_WAIT, X86_SYSTEMS_DEADLINE_WAIT_SYMBOL),
        (
            SYSTEMS_DEADLINE_COMPLETE,
            X86_SYSTEMS_DEADLINE_COMPLETE_SYMBOL,
        ),
        (
            SYSTEMS_RESUME_DEADLINE,
            "topal_x86_systems_interrupt_return",
        ),
        (
            SYSTEMS_KERNEL_CONTEXT_CREATE,
            X86_SYSTEMS_CONTEXT_CREATE_SYMBOL,
        ),
        (
            SYSTEMS_KERNEL_CONTEXT_TRANSFER,
            X86_SYSTEMS_CONTEXT_TRANSFER_SYMBOL,
        ),
        (
            SYSTEMS_KERNEL_CONTEXT_RETIRE,
            X86_SYSTEMS_CONTEXT_RETIRE_SYMBOL,
        ),
        (
            SYSTEMS_KERNEL_CONTEXT_RECLAIM,
            X86_SYSTEMS_CONTEXT_RECLAIM_SYMBOL,
        ),
        (
            SYSTEMS_USER_ADDRESS_SPACE_PREPARE,
            X86_SYSTEMS_USER_ADDRESS_SPACE_PREPARE_SYMBOL,
        ),
        (
            SYSTEMS_USER_ADDRESS_SPACE_VERIFY_MAPPING,
            X86_SYSTEMS_USER_ADDRESS_SPACE_VERIFY_MAPPING_SYMBOL,
        ),
        (
            SYSTEMS_USER_ADDRESS_SPACE_MAP,
            X86_SYSTEMS_USER_ADDRESS_SPACE_MAP_SYMBOL,
        ),
        (
            SYSTEMS_USER_MAPPING_STORE_BYTE,
            X86_SYSTEMS_USER_MAPPING_STORE_BYTE_SYMBOL,
        ),
        (
            SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY,
            X86_SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY_SYMBOL,
        ),
        (
            SYSTEMS_USER_MAPPING_LOAD_BYTE,
            X86_SYSTEMS_USER_MAPPING_LOAD_BYTE_SYMBOL,
        ),
        (
            SYSTEMS_USER_MAPPING_PERMIT_EXECUTE,
            X86_SYSTEMS_USER_MAPPING_PERMIT_EXECUTE_SYMBOL,
        ),
        (
            SYSTEMS_USER_MAPPING_SEAL_IMMUTABLE_DATA,
            X86_SYSTEMS_USER_MAPPING_SEAL_IMMUTABLE_DATA_SYMBOL,
        ),
        (
            SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA,
            X86_SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA_SYMBOL,
        ),
        (
            SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS,
            X86_SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS_SYMBOL,
        ),
        (
            SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS,
            X86_SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS_SYMBOL,
        ),
        (
            SYSTEMS_USER_ADDRESS_SPACE_UNMAP,
            X86_SYSTEMS_USER_ADDRESS_SPACE_UNMAP_SYMBOL,
        ),
        (
            SYSTEMS_USER_CONTEXT_PREPARE,
            X86_SYSTEMS_USER_PREPARE_SYMBOL,
        ),
        (SYSTEMS_USER_CONTEXT_ENTER, X86_SYSTEMS_USER_ENTER_SYMBOL),
        (
            SYSTEMS_USER_CONTEXT_COMPLETE,
            X86_SYSTEMS_USER_COMPLETE_SYMBOL,
        ),
        (
            SYSTEMS_USER_ADDRESS_SPACE_RECLAIM,
            X86_SYSTEMS_USER_ADDRESS_SPACE_RECLAIM_SYMBOL,
        ),
        (SYSTEMS_USER_COPY_TO, X86_SYSTEMS_USER_SYSCALL_SYMBOL),
        (SYSTEMS_USER_COPY_FROM, X86_SYSTEMS_USER_SYSCALL_SYMBOL),
        (
            SYSTEMS_USER_SYSCALL_DISPATCH,
            X86_SYSTEMS_USER_SYSCALL_SYMBOL,
        ),
        (SYSTEMS_CONSOLE_WRITE, "topal_x86_systems_uart16550_write"),
        (SYSTEMS_DEBUG_BREAK, "topal_x86_systems_debug_break"),
        (
            SYSTEMS_RESUME_DEBUG_BREAK,
            "topal_x86_systems_interrupt_return",
        ),
        (
            SYSTEMS_RESUME_LOCAL_NOTIFICATION,
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
    AllocatePhysicalFrames,
    PrepareStaticImage,
    ZeroStaticImagePayload,
    ZeroStaticImageStackBacking,
    PublishStaticImageStackLeaf,
    InitializeStaticImageStackEntry,
    AddStaticImageStackPageSize,
    AddStaticImageStackEntryFact,
    AddStaticImageStackProgramHeaders,
    AddStaticImageStackExecutableName,
    AddStaticImageStackCredentials,
    AddStaticImageStackSecureMode,
    AddStaticImageStackClockTickRate,
    AddStaticImageStackPlatform,
    AddStaticImageStackHardwareCapabilities,
    AddStaticImageStackLoadMetadata,
    AuthorizeStaticImageStackRandomSlot,
    RetainStaticImagePreparedProcess,
    QualifyStaticImageRetainedDisjointWork,
    AuthorizeStaticImageEntropyHandoff,
    AuthorizeStaticImageEntropyAcquisition,
    QualifyStaticImageEntropyDevice,
    QualifyStaticImageEntropyTransport,
    QualifyStaticImageEntropyNegotiation,
    QualifyStaticImageEntropyQueue,
    QualifyStaticImageEntropyBusMaster,
    QualifyStaticImageEntropyDmaBuffers,
    QualifyStaticImageEntropySplitRing,
    QualifyStaticImageEntropyDmaPreparation,
    QualifyStaticImageEntropyQueueAddresses,
    QualifyStaticImageEntropyQueueActivation,
    QualifyStaticImageEntropyRequestCompletion,
    QualifyStaticImageEntropySampleCapture,
    QualifyStaticImageEntropySampleQuality,
    QualifyStaticImageEntropyPoolLifecycle,
    QualifyStaticImageEntropyGeneratorLifecycle,
    DeliverStaticImageEntropyInitialRandom,
    EnterStaticImageProcess,
    RestoreStaticImageProcessEntry,
    RevokeStaticImageEntropyInitialRandom,
    RevokeStaticImageEntropyAcquisition,
    RevokeStaticImageEntropyHandoff,
    RestoreStaticImagePreparedProcess,
    RevokeStaticImageStackRandomSlot,
    RemoveStaticImageStackLoadMetadata,
    RemoveStaticImageStackHardwareCapabilities,
    RemoveStaticImageStackPlatform,
    RemoveStaticImageStackClockTickRate,
    RemoveStaticImageStackSecureMode,
    RemoveStaticImageStackCredentials,
    RemoveStaticImageStackExecutableName,
    RemoveStaticImageStackProgramHeaders,
    RemoveStaticImageStackEntryFact,
    RemoveStaticImageStackPageSize,
    ClearStaticImageStackEntry,
    RevokeStaticImageStackLeaf,
    CopyStaticImagePayload,
    SealStaticImagePermissions,
    AttachStaticImageAddressSpace,
    ActivateStaticImageAddressSpace,
    RestoreStaticImageAddressSpace,
    AuthorizeStaticImageEntry,
    AuthorizeStaticImageStackSlot,
    RevokeStaticImageStackSlot,
    RevokeStaticImageEntry,
    DetachStaticImageAddressSpace,
    RevokeStaticImage,
    BeginTranslation,
    CommitTranslation,
    ActivateTranslation,
    BeginTranslationEdit,
    MapTranslationFrames,
    UnmapTranslationMapping,
    CommitTranslationEdit,
    EnterCritical,
    RestoreCritical,
    AtomicWordCreate,
    AtomicWordCompareExchange,
    AtomicWordLoad,
    SendLocalNotification,
    WaitLocalNotification,
    CompleteLocalNotification,
    ObserveMonotonicClock,
    ObservePciConfigurationDword,
    UpdatePciConfigurationWord,
    CreateDmaBuffer,
    BindDmaBufferReference,
    StoreDmaBufferInteger,
    PrepareDmaBufferForDevice,
    CompleteDmaBufferForCpu,
    LoadDmaBufferInteger,
    CaptureEntropySample,
    MixEntropyPoolSample,
    InitializeEntropyGenerator,
    ExtractEntropyGenerator,
    DeliverInitialAuxvRandom,
    ReserveInitialUserRandom,
    BindReusableUserRandom,
    PublishPciRegionDmaReference,
    ReadPciRegionRegister,
    ReadPciRegionRegisterEventually,
    WritePciRegionRegister,
    ConstructDeadline,
    ArmDeadline,
    WaitDeadline,
    CompleteDeadline,
    CreateKernelContext,
    TransferKernelContext,
    AwaitDeadlinePreemption,
    PreemptCurrentKernelContext,
    RetireKernelContext,
    ReclaimKernelContext,
    PrepareUserAddressSpace,
    VerifyUserAddressSpaceMapping,
    MapUserAddressSpace,
    StoreUserMappingByte,
    InstallUserMappingEntryTrampoline,
    ProtectUserMappingReadOnly,
    LoadUserMappingByte,
    VerifyUserMappingEntryTrampoline,
    PermitUserMappingExecute,
    SealUserMappingImmutableData,
    PermitUserMappingWritableData,
    ReserveAnonymousUserMapping,
    ReleaseAnonymousUserMapping,
    UnmapUserAddressSpace,
    PrepareUserProbe,
    EnterUserContext,
    CompleteUserBacking,
    ReclaimUserAddressSpace,
    UserSyscall,
    KernelThreadEntry,
    CompletionThreadEntry,
    TerminalThreadEntry,
    Uart16550Write,
    DebugBreak,
    InterruptReturn,
    Fatal,
    BootstrapStorage,
    DmaVirtqueueControl,
    DmaEntropyPayload,
    RawEntropySample,
    EntropyPoolState,
    EntropyGeneratorKey,
    EntropyOutput,
    EntropyUserRandomService,
    StaticProcessBrkReserve,
}

impl ProviderSymbol {
    const fn name(self) -> &'static str {
        match self {
            Self::DescribeBootMemory => X86_SYSTEMS_BOOT_MEMORY_SYMBOL,
            Self::AllocatePhysicalFrames => X86_SYSTEMS_FRAME_ALLOCATE_SYMBOL,
            Self::PrepareStaticImage => X86_SYSTEMS_STATIC_IMAGE_PREPARE_SYMBOL,
            Self::ZeroStaticImagePayload => X86_SYSTEMS_STATIC_IMAGE_ZERO_PAYLOAD_SYMBOL,
            Self::ZeroStaticImageStackBacking => X86_SYSTEMS_STATIC_IMAGE_ZERO_STACK_BACKING_SYMBOL,
            Self::PublishStaticImageStackLeaf => X86_SYSTEMS_STATIC_IMAGE_PUBLISH_STACK_LEAF_SYMBOL,
            Self::InitializeStaticImageStackEntry => {
                X86_SYSTEMS_STATIC_IMAGE_INITIALIZE_STACK_ENTRY_SYMBOL
            }
            Self::AddStaticImageStackPageSize => X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_PAGESZ_SYMBOL,
            Self::AddStaticImageStackEntryFact => {
                X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_ENTRY_FACT_SYMBOL
            }
            Self::AddStaticImageStackProgramHeaders => {
                X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_PROGRAM_HEADERS_SYMBOL
            }
            Self::AddStaticImageStackExecutableName => {
                X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_EXECUTABLE_NAME_SYMBOL
            }
            Self::AddStaticImageStackCredentials => {
                X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_CREDENTIALS_SYMBOL
            }
            Self::AddStaticImageStackSecureMode => {
                X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_SECURE_MODE_SYMBOL
            }
            Self::AddStaticImageStackClockTickRate => {
                X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_CLOCK_TICK_RATE_SYMBOL
            }
            Self::AddStaticImageStackPlatform => X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_PLATFORM_SYMBOL,
            Self::AddStaticImageStackHardwareCapabilities => {
                X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_HARDWARE_CAPABILITIES_SYMBOL
            }
            Self::AddStaticImageStackLoadMetadata => {
                X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_LOAD_METADATA_SYMBOL
            }
            Self::AuthorizeStaticImageStackRandomSlot => {
                X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_STACK_RANDOM_SLOT_SYMBOL
            }
            Self::RetainStaticImagePreparedProcess => {
                X86_SYSTEMS_STATIC_IMAGE_RETAIN_PREPARED_PROCESS_SYMBOL
            }
            Self::QualifyStaticImageRetainedDisjointWork => {
                X86_SYSTEMS_STATIC_IMAGE_QUALIFY_RETAINED_DISJOINT_WORK_SYMBOL
            }
            Self::AuthorizeStaticImageEntropyHandoff => {
                X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_ENTROPY_HANDOFF_SYMBOL
            }
            Self::AuthorizeStaticImageEntropyAcquisition => {
                X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_ENTROPY_ACQUISITION_SYMBOL
            }
            Self::QualifyStaticImageEntropyDevice => {
                X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_DEVICE_SYMBOL
            }
            Self::QualifyStaticImageEntropyTransport => {
                X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_TRANSPORT_SYMBOL
            }
            Self::QualifyStaticImageEntropyNegotiation => {
                X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_NEGOTIATION_SYMBOL
            }
            Self::QualifyStaticImageEntropyQueue => {
                X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_QUEUE_SYMBOL
            }
            Self::QualifyStaticImageEntropyBusMaster => {
                X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_BUS_MASTER_SYMBOL
            }
            Self::QualifyStaticImageEntropyDmaBuffers => {
                X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_DMA_BUFFERS_SYMBOL
            }
            Self::QualifyStaticImageEntropySplitRing => {
                X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_SPLIT_RING_SYMBOL
            }
            Self::QualifyStaticImageEntropyDmaPreparation => {
                X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_DMA_PREPARATION_SYMBOL
            }
            Self::QualifyStaticImageEntropyQueueAddresses => {
                X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_QUEUE_ADDRESSES_SYMBOL
            }
            Self::QualifyStaticImageEntropyQueueActivation => {
                X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_QUEUE_ACTIVATION_SYMBOL
            }
            Self::QualifyStaticImageEntropyRequestCompletion => {
                X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_REQUEST_COMPLETION_SYMBOL
            }
            Self::QualifyStaticImageEntropySampleCapture => {
                X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_SAMPLE_CAPTURE_SYMBOL
            }
            Self::QualifyStaticImageEntropySampleQuality => {
                X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_SAMPLE_QUALITY_SYMBOL
            }
            Self::QualifyStaticImageEntropyPoolLifecycle => {
                X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_POOL_LIFECYCLE_SYMBOL
            }
            Self::QualifyStaticImageEntropyGeneratorLifecycle => {
                X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_GENERATOR_LIFECYCLE_SYMBOL
            }
            Self::DeliverStaticImageEntropyInitialRandom => {
                X86_SYSTEMS_STATIC_IMAGE_DELIVER_ENTROPY_INITIAL_RANDOM_SYMBOL
            }
            Self::EnterStaticImageProcess => X86_SYSTEMS_STATIC_IMAGE_PROCESS_ENTER_SYMBOL,
            Self::RestoreStaticImageProcessEntry => {
                X86_SYSTEMS_STATIC_IMAGE_PROCESS_RESTORE_ENTRY_SYMBOL
            }
            Self::RevokeStaticImageEntropyInitialRandom => {
                X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTROPY_INITIAL_RANDOM_SYMBOL
            }
            Self::RevokeStaticImageEntropyAcquisition => {
                X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTROPY_ACQUISITION_SYMBOL
            }
            Self::RevokeStaticImageEntropyHandoff => {
                X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTROPY_HANDOFF_SYMBOL
            }
            Self::RestoreStaticImagePreparedProcess => {
                X86_SYSTEMS_STATIC_IMAGE_RESTORE_PREPARED_PROCESS_SYMBOL
            }
            Self::RevokeStaticImageStackRandomSlot => {
                X86_SYSTEMS_STATIC_IMAGE_REVOKE_STACK_RANDOM_SLOT_SYMBOL
            }
            Self::RemoveStaticImageStackLoadMetadata => {
                X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_LOAD_METADATA_SYMBOL
            }
            Self::RemoveStaticImageStackHardwareCapabilities => {
                X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_HARDWARE_CAPABILITIES_SYMBOL
            }
            Self::RemoveStaticImageStackPlatform => {
                X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_PLATFORM_SYMBOL
            }
            Self::RemoveStaticImageStackClockTickRate => {
                X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_CLOCK_TICK_RATE_SYMBOL
            }
            Self::RemoveStaticImageStackSecureMode => {
                X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_SECURE_MODE_SYMBOL
            }
            Self::RemoveStaticImageStackCredentials => {
                X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_CREDENTIALS_SYMBOL
            }
            Self::RemoveStaticImageStackExecutableName => {
                X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_EXECUTABLE_NAME_SYMBOL
            }
            Self::RemoveStaticImageStackProgramHeaders => {
                X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_PROGRAM_HEADERS_SYMBOL
            }
            Self::RemoveStaticImageStackEntryFact => {
                X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_ENTRY_FACT_SYMBOL
            }
            Self::RemoveStaticImageStackPageSize => {
                X86_SYSTEMS_STATIC_IMAGE_REMOVE_STACK_PAGESZ_SYMBOL
            }
            Self::ClearStaticImageStackEntry => X86_SYSTEMS_STATIC_IMAGE_CLEAR_STACK_ENTRY_SYMBOL,
            Self::RevokeStaticImageStackLeaf => X86_SYSTEMS_STATIC_IMAGE_REVOKE_STACK_LEAF_SYMBOL,
            Self::CopyStaticImagePayload => X86_SYSTEMS_STATIC_IMAGE_COPY_PAYLOAD_SYMBOL,
            Self::SealStaticImagePermissions => X86_SYSTEMS_STATIC_IMAGE_SEAL_PERMISSIONS_SYMBOL,
            Self::AttachStaticImageAddressSpace => {
                X86_SYSTEMS_STATIC_IMAGE_ATTACH_ADDRESS_SPACE_SYMBOL
            }
            Self::ActivateStaticImageAddressSpace => {
                X86_SYSTEMS_STATIC_IMAGE_ACTIVATE_ADDRESS_SPACE_SYMBOL
            }
            Self::RestoreStaticImageAddressSpace => {
                X86_SYSTEMS_STATIC_IMAGE_RESTORE_ADDRESS_SPACE_SYMBOL
            }
            Self::AuthorizeStaticImageEntry => X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_ENTRY_SYMBOL,
            Self::AuthorizeStaticImageStackSlot => {
                X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_STACK_SLOT_SYMBOL
            }
            Self::RevokeStaticImageStackSlot => X86_SYSTEMS_STATIC_IMAGE_REVOKE_STACK_SLOT_SYMBOL,
            Self::RevokeStaticImageEntry => X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTRY_SYMBOL,
            Self::DetachStaticImageAddressSpace => {
                X86_SYSTEMS_STATIC_IMAGE_DETACH_ADDRESS_SPACE_SYMBOL
            }
            Self::RevokeStaticImage => X86_SYSTEMS_STATIC_IMAGE_REVOKE_SYMBOL,
            Self::BeginTranslation => X86_SYSTEMS_TRANSLATION_BEGIN_SYMBOL,
            Self::CommitTranslation => X86_SYSTEMS_TRANSLATION_COMMIT_SYMBOL,
            Self::ActivateTranslation => X86_SYSTEMS_TRANSLATION_ACTIVATE_SYMBOL,
            Self::BeginTranslationEdit => X86_SYSTEMS_TRANSLATION_EDIT_BEGIN_SYMBOL,
            Self::MapTranslationFrames => X86_SYSTEMS_TRANSLATION_EDIT_MAP_SYMBOL,
            Self::UnmapTranslationMapping => X86_SYSTEMS_TRANSLATION_EDIT_UNMAP_SYMBOL,
            Self::CommitTranslationEdit => X86_SYSTEMS_TRANSLATION_EDIT_COMMIT_SYMBOL,
            Self::EnterCritical => X86_SYSTEMS_CRITICAL_ENTER_SYMBOL,
            Self::RestoreCritical => X86_SYSTEMS_CRITICAL_RESTORE_SYMBOL,
            Self::AtomicWordCreate => X86_SYSTEMS_ATOMIC_CREATE_SYMBOL,
            Self::AtomicWordCompareExchange => X86_SYSTEMS_ATOMIC_COMPARE_EXCHANGE_SYMBOL,
            Self::AtomicWordLoad => X86_SYSTEMS_ATOMIC_LOAD_SYMBOL,
            Self::SendLocalNotification => X86_SYSTEMS_LOCAL_NOTIFICATION_SEND_SYMBOL,
            Self::WaitLocalNotification => X86_SYSTEMS_LOCAL_NOTIFICATION_WAIT_SYMBOL,
            Self::CompleteLocalNotification => X86_SYSTEMS_LOCAL_NOTIFICATION_COMPLETE_SYMBOL,
            Self::ObserveMonotonicClock => X86_SYSTEMS_MONOTONIC_CLOCK_NOW_SYMBOL,
            Self::ObservePciConfigurationDword => X86_SYSTEMS_PCI_CONFIGURATION_READ_DWORD_SYMBOL,
            Self::UpdatePciConfigurationWord => X86_SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD_SYMBOL,
            Self::CreateDmaBuffer => X86_SYSTEMS_DMA_BUFFER_CREATE_SYMBOL,
            Self::BindDmaBufferReference => X86_SYSTEMS_DMA_BUFFER_BIND_REFERENCE_SYMBOL,
            Self::StoreDmaBufferInteger => X86_SYSTEMS_DMA_BUFFER_STORE_INTEGER_SYMBOL,
            Self::PrepareDmaBufferForDevice => X86_SYSTEMS_DMA_BUFFER_PREPARE_DEVICE_SYMBOL,
            Self::CompleteDmaBufferForCpu => X86_SYSTEMS_DMA_BUFFER_COMPLETE_CPU_SYMBOL,
            Self::LoadDmaBufferInteger => X86_SYSTEMS_DMA_BUFFER_LOAD_INTEGER_SYMBOL,
            Self::CaptureEntropySample => X86_SYSTEMS_ENTROPY_SAMPLE_CAPTURE_SYMBOL,
            Self::MixEntropyPoolSample => X86_SYSTEMS_ENTROPY_POOL_MIX_SYMBOL,
            Self::InitializeEntropyGenerator => X86_SYSTEMS_ENTROPY_GENERATOR_INITIALIZE_SYMBOL,
            Self::ExtractEntropyGenerator => X86_SYSTEMS_ENTROPY_GENERATOR_EXTRACT_SYMBOL,
            Self::DeliverInitialAuxvRandom => {
                X86_SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM_SYMBOL
            }
            Self::ReserveInitialUserRandom => {
                X86_SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM_SYMBOL
            }
            Self::BindReusableUserRandom => X86_SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE_SYMBOL,
            Self::PublishPciRegionDmaReference => X86_SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE_SYMBOL,
            Self::ReadPciRegionRegister => X86_SYSTEMS_PCI_REGION_READ_SYMBOL,
            Self::ReadPciRegionRegisterEventually => X86_SYSTEMS_PCI_REGION_READ_EVENTUALLY_SYMBOL,
            Self::WritePciRegionRegister => X86_SYSTEMS_PCI_REGION_WRITE_SYMBOL,
            Self::ConstructDeadline => X86_SYSTEMS_DEADLINE_AFTER_SYMBOL,
            Self::ArmDeadline => X86_SYSTEMS_DEADLINE_ARM_SYMBOL,
            Self::WaitDeadline => X86_SYSTEMS_DEADLINE_WAIT_SYMBOL,
            Self::CompleteDeadline => X86_SYSTEMS_DEADLINE_COMPLETE_SYMBOL,
            Self::CreateKernelContext => X86_SYSTEMS_CONTEXT_CREATE_SYMBOL,
            Self::TransferKernelContext => X86_SYSTEMS_CONTEXT_TRANSFER_SYMBOL,
            Self::AwaitDeadlinePreemption => X86_SYSTEMS_CONTEXT_AWAIT_DEADLINE_PREEMPTION_SYMBOL,
            Self::PreemptCurrentKernelContext => X86_SYSTEMS_CONTEXT_PREEMPT_CURRENT_SYMBOL,
            Self::RetireKernelContext => X86_SYSTEMS_CONTEXT_RETIRE_SYMBOL,
            Self::ReclaimKernelContext => X86_SYSTEMS_CONTEXT_RECLAIM_SYMBOL,
            Self::PrepareUserAddressSpace => X86_SYSTEMS_USER_ADDRESS_SPACE_PREPARE_SYMBOL,
            Self::VerifyUserAddressSpaceMapping => {
                X86_SYSTEMS_USER_ADDRESS_SPACE_VERIFY_MAPPING_SYMBOL
            }
            Self::MapUserAddressSpace => X86_SYSTEMS_USER_ADDRESS_SPACE_MAP_SYMBOL,
            Self::StoreUserMappingByte => X86_SYSTEMS_USER_MAPPING_STORE_BYTE_SYMBOL,
            Self::InstallUserMappingEntryTrampoline => {
                X86_SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE_SYMBOL
            }
            Self::ProtectUserMappingReadOnly => X86_SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY_SYMBOL,
            Self::LoadUserMappingByte => X86_SYSTEMS_USER_MAPPING_LOAD_BYTE_SYMBOL,
            Self::VerifyUserMappingEntryTrampoline => {
                X86_SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE_SYMBOL
            }
            Self::PermitUserMappingExecute => X86_SYSTEMS_USER_MAPPING_PERMIT_EXECUTE_SYMBOL,
            Self::SealUserMappingImmutableData => {
                X86_SYSTEMS_USER_MAPPING_SEAL_IMMUTABLE_DATA_SYMBOL
            }
            Self::PermitUserMappingWritableData => {
                X86_SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA_SYMBOL
            }
            Self::ReserveAnonymousUserMapping => {
                X86_SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS_SYMBOL
            }
            Self::ReleaseAnonymousUserMapping => {
                X86_SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS_SYMBOL
            }
            Self::UnmapUserAddressSpace => X86_SYSTEMS_USER_ADDRESS_SPACE_UNMAP_SYMBOL,
            Self::PrepareUserProbe => X86_SYSTEMS_USER_PREPARE_SYMBOL,
            Self::EnterUserContext => X86_SYSTEMS_USER_ENTER_SYMBOL,
            Self::CompleteUserBacking => X86_SYSTEMS_USER_COMPLETE_SYMBOL,
            Self::ReclaimUserAddressSpace => X86_SYSTEMS_USER_ADDRESS_SPACE_RECLAIM_SYMBOL,
            Self::UserSyscall => X86_SYSTEMS_USER_SYSCALL_SYMBOL,
            Self::KernelThreadEntry => X86_SYSTEMS_KERNEL_THREAD_ENTRY,
            Self::CompletionThreadEntry => X86_SYSTEMS_COMPLETION_THREAD_ENTRY,
            Self::TerminalThreadEntry => X86_SYSTEMS_TERMINAL_THREAD_ENTRY,
            Self::Uart16550Write => "topal_x86_systems_uart16550_write",
            Self::DebugBreak => "topal_x86_systems_debug_break",
            Self::InterruptReturn => "topal_x86_systems_interrupt_return",
            Self::Fatal => "topal_x86_systems_fatal",
            Self::BootstrapStorage => "topal_bootstrap_storage",
            Self::DmaVirtqueueControl => X86_SYSTEMS_DMA_VIRTQUEUE_CONTROL_SYMBOL,
            Self::DmaEntropyPayload => X86_SYSTEMS_DMA_ENTROPY_PAYLOAD_SYMBOL,
            Self::RawEntropySample => X86_SYSTEMS_RAW_ENTROPY_SAMPLE_SYMBOL,
            Self::EntropyPoolState => X86_SYSTEMS_ENTROPY_POOL_STATE_SYMBOL,
            Self::EntropyGeneratorKey => X86_SYSTEMS_ENTROPY_GENERATOR_KEY_SYMBOL,
            Self::EntropyOutput => X86_SYSTEMS_ENTROPY_OUTPUT_SYMBOL,
            Self::EntropyUserRandomService => X86_SYSTEMS_ENTROPY_USER_RANDOM_SERVICE_SYMBOL,
            Self::StaticProcessBrkReserve => X86_SYSTEMS_STATIC_PROCESS_BRK_RESERVE_SYMBOL,
        }
    }

    const fn kind(self) -> SymbolKind {
        match self {
            Self::BootstrapStorage
            | Self::DmaVirtqueueControl
            | Self::DmaEntropyPayload
            | Self::RawEntropySample
            | Self::EntropyPoolState
            | Self::EntropyGeneratorKey
            | Self::EntropyOutput
            | Self::EntropyUserRandomService
            | Self::StaticProcessBrkReserve => SymbolKind::Data,
            Self::DescribeBootMemory
            | Self::AllocatePhysicalFrames
            | Self::PrepareStaticImage
            | Self::ZeroStaticImagePayload
            | Self::ZeroStaticImageStackBacking
            | Self::PublishStaticImageStackLeaf
            | Self::InitializeStaticImageStackEntry
            | Self::AddStaticImageStackPageSize
            | Self::AddStaticImageStackEntryFact
            | Self::AddStaticImageStackProgramHeaders
            | Self::AddStaticImageStackExecutableName
            | Self::AddStaticImageStackCredentials
            | Self::AddStaticImageStackSecureMode
            | Self::AddStaticImageStackClockTickRate
            | Self::AddStaticImageStackPlatform
            | Self::AddStaticImageStackHardwareCapabilities
            | Self::AddStaticImageStackLoadMetadata
            | Self::AuthorizeStaticImageStackRandomSlot
            | Self::RetainStaticImagePreparedProcess
            | Self::QualifyStaticImageRetainedDisjointWork
            | Self::AuthorizeStaticImageEntropyHandoff
            | Self::AuthorizeStaticImageEntropyAcquisition
            | Self::QualifyStaticImageEntropyDevice
            | Self::QualifyStaticImageEntropyTransport
            | Self::QualifyStaticImageEntropyNegotiation
            | Self::QualifyStaticImageEntropyQueue
            | Self::QualifyStaticImageEntropyBusMaster
            | Self::QualifyStaticImageEntropyDmaBuffers
            | Self::QualifyStaticImageEntropySplitRing
            | Self::QualifyStaticImageEntropyDmaPreparation
            | Self::QualifyStaticImageEntropyQueueAddresses
            | Self::QualifyStaticImageEntropyQueueActivation
            | Self::QualifyStaticImageEntropyRequestCompletion
            | Self::QualifyStaticImageEntropySampleCapture
            | Self::QualifyStaticImageEntropySampleQuality
            | Self::QualifyStaticImageEntropyPoolLifecycle
            | Self::QualifyStaticImageEntropyGeneratorLifecycle
            | Self::DeliverStaticImageEntropyInitialRandom
            | Self::EnterStaticImageProcess
            | Self::RestoreStaticImageProcessEntry
            | Self::RevokeStaticImageEntropyInitialRandom
            | Self::RevokeStaticImageEntropyAcquisition
            | Self::RevokeStaticImageEntropyHandoff
            | Self::RestoreStaticImagePreparedProcess
            | Self::RevokeStaticImageStackRandomSlot
            | Self::RemoveStaticImageStackLoadMetadata
            | Self::RemoveStaticImageStackHardwareCapabilities
            | Self::RemoveStaticImageStackPlatform
            | Self::RemoveStaticImageStackClockTickRate
            | Self::RemoveStaticImageStackSecureMode
            | Self::RemoveStaticImageStackCredentials
            | Self::RemoveStaticImageStackExecutableName
            | Self::RemoveStaticImageStackProgramHeaders
            | Self::RemoveStaticImageStackEntryFact
            | Self::RemoveStaticImageStackPageSize
            | Self::ClearStaticImageStackEntry
            | Self::RevokeStaticImageStackLeaf
            | Self::CopyStaticImagePayload
            | Self::SealStaticImagePermissions
            | Self::AttachStaticImageAddressSpace
            | Self::ActivateStaticImageAddressSpace
            | Self::RestoreStaticImageAddressSpace
            | Self::AuthorizeStaticImageEntry
            | Self::AuthorizeStaticImageStackSlot
            | Self::RevokeStaticImageStackSlot
            | Self::RevokeStaticImageEntry
            | Self::DetachStaticImageAddressSpace
            | Self::RevokeStaticImage
            | Self::BeginTranslation
            | Self::CommitTranslation
            | Self::ActivateTranslation
            | Self::BeginTranslationEdit
            | Self::MapTranslationFrames
            | Self::UnmapTranslationMapping
            | Self::CommitTranslationEdit
            | Self::EnterCritical
            | Self::RestoreCritical
            | Self::AtomicWordCreate
            | Self::AtomicWordCompareExchange
            | Self::AtomicWordLoad
            | Self::SendLocalNotification
            | Self::WaitLocalNotification
            | Self::CompleteLocalNotification
            | Self::ObserveMonotonicClock
            | Self::ObservePciConfigurationDword
            | Self::UpdatePciConfigurationWord
            | Self::CreateDmaBuffer
            | Self::BindDmaBufferReference
            | Self::StoreDmaBufferInteger
            | Self::PrepareDmaBufferForDevice
            | Self::CompleteDmaBufferForCpu
            | Self::LoadDmaBufferInteger
            | Self::CaptureEntropySample
            | Self::MixEntropyPoolSample
            | Self::InitializeEntropyGenerator
            | Self::ExtractEntropyGenerator
            | Self::DeliverInitialAuxvRandom
            | Self::ReserveInitialUserRandom
            | Self::BindReusableUserRandom
            | Self::PublishPciRegionDmaReference
            | Self::ReadPciRegionRegister
            | Self::ReadPciRegionRegisterEventually
            | Self::WritePciRegionRegister
            | Self::ConstructDeadline
            | Self::ArmDeadline
            | Self::WaitDeadline
            | Self::CompleteDeadline
            | Self::CreateKernelContext
            | Self::TransferKernelContext
            | Self::AwaitDeadlinePreemption
            | Self::PreemptCurrentKernelContext
            | Self::RetireKernelContext
            | Self::ReclaimKernelContext
            | Self::PrepareUserAddressSpace
            | Self::VerifyUserAddressSpaceMapping
            | Self::MapUserAddressSpace
            | Self::StoreUserMappingByte
            | Self::InstallUserMappingEntryTrampoline
            | Self::ProtectUserMappingReadOnly
            | Self::LoadUserMappingByte
            | Self::VerifyUserMappingEntryTrampoline
            | Self::PermitUserMappingExecute
            | Self::SealUserMappingImmutableData
            | Self::PermitUserMappingWritableData
            | Self::ReserveAnonymousUserMapping
            | Self::ReleaseAnonymousUserMapping
            | Self::UnmapUserAddressSpace
            | Self::PrepareUserProbe
            | Self::EnterUserContext
            | Self::CompleteUserBacking
            | Self::ReclaimUserAddressSpace
            | Self::UserSyscall
            | Self::KernelThreadEntry
            | Self::CompletionThreadEntry
            | Self::TerminalThreadEntry
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
    frame_allocator_created: bool,
    physical_frame_count: Option<u64>,
    retained_static_elf_payload: bool,
    static_image: RootStaticImageState,
    kernel_mapping_live: bool,
    translation: RootTranslationState,
    critical: RootCriticalState,
    bootstrap_region_offset: Option<u64>,
    atomic_word_offset: Option<u64>,
    local_notification: RootLocalNotificationState,
    pci_configuration_observations: usize,
    pci_region_register_accesses: usize,
    pci_bus_master_authorized: bool,
    dma_buffers_created: usize,
    dma_construction_operations: usize,
    dma_buffers_prepared: usize,
    dma_references_published: usize,
    dma_control_completed: bool,
    dma_used_ring_reads: usize,
    dma_payload_completed: bool,
    entropy_sample_captured: bool,
    entropy_sample_assessed: bool,
    entropy_pool_created: bool,
    entropy_pool_mixed: bool,
    entropy_pool_credited: bool,
    entropy_generator_ready: bool,
    entropy_generator_extracted: bool,
    initial_auxv_random_delivered: bool,
    initial_getrandom_extracted: bool,
    initial_user_random_reserved: bool,
    user_random_service_bound: bool,
    user_random_small_requests_enabled: bool,
    user_random_large_requests_enabled: bool,
    monotonic_observations: u8,
    deadline: RootDeadlineState,
    kernel_context: RootKernelContextState,
    bootstrap_next_offset: u64,
    context_stack_offsets: [Option<u64>; 3],
    completion: RootCompletionState,
    user_execution: RootUserExecutionState,
    role: RootRole,
}

#[derive(Default, Eq, PartialEq)]
enum RootRole {
    #[default]
    Ordinary,
    BootstrapWithLocalNotification,
    LocalNotificationHandler,
    DeadlineHandler,
    CooperativeKernelThreadHandler,
    CompletionKernelThreadHandler,
    TerminalKernelThreadHandler,
}

#[derive(Default, Eq, PartialEq)]
enum RootTranslationState {
    #[default]
    BootstrapActive,
    Update,
    InactiveSpace,
    ReplacementActive,
    MapEdit,
    MapStaged,
    MappedActive,
    UnmapEdit,
    UnmapStaged,
    RemovedActive,
}

#[derive(Default, Eq, PartialEq)]
enum RootCriticalState {
    #[default]
    Restored,
    Live,
}

#[derive(Default, Eq, PartialEq)]
enum RootStaticImageState {
    #[default]
    Fresh,
    Live,
    Zeroed,
    Populated,
    Sealed,
    Attached,
    Active,
    Restored,
    EntryAuthorized,
    StackSlotAuthorized,
    StackBackingZeroed,
    StackLeafPublished,
    StackEntryInitialized,
    StackPageSizeAdded,
    StackEntryFactAdded,
    StackProgramHeadersAdded,
    StackExecutableNameAdded,
    StackCredentialsAdded,
    StackSecureModeAdded,
    StackClockTickRateAdded,
    StackPlatformAdded,
    StackHardwareCapabilitiesAdded,
    StackLoadMetadataAdded,
    StackRandomSlotAuthorized,
    PreparedProcessRetained,
    EntropyPendingProcess,
    EntropyAcquiringProcess,
    RandomizedProcess,
    CompletedProcess,
    EntryRestored,
    Detached,
    HierarchyReturned,
    MetadataReturned,
    PayloadReturned,
}

#[derive(Default, Eq, PartialEq)]
enum RootLocalNotificationState {
    #[default]
    Idle,
    Pending,
    Completed,
}

#[derive(Default, Eq, PartialEq)]
enum RootDeadlineState {
    #[default]
    Fresh,
    Constructed,
    Armed,
    Completed,
}

#[derive(Default, Eq, PartialEq)]
enum RootKernelContextState {
    #[default]
    Fresh,
    CooperativeCreated,
    CompletionCreated,
    AllSuspended,
    QueueCreated,
    CooperativeEnqueued,
    CompletionEnqueued,
    AllEnqueued,
    CooperativeDequeued,
    CooperativeYielded,
    PreemptedTaken,
    HandoffEnqueued,
    CompletionDequeued,
    CompletionBlocked,
    TerminalDequeued,
    TerminalCompleted,
    TerminalReclaimed,
    CompletionSignaled,
    CompletionEnqueuedAgain,
    CooperativeDequeuedAgain,
    CooperativeCompleted,
    CooperativeReclaimed,
    CompletionDequeuedAgain,
    CompletionCompleted,
    CompletionReclaimed,
    QueueConsumed,
}

#[derive(Default, Eq, PartialEq)]
enum RootCompletionState {
    #[default]
    Fresh,
    Created,
    Blocked,
    Signaled,
    Consumed,
}

#[derive(Default, Eq, PartialEq)]
enum RootUserExecutionState {
    #[default]
    Fresh,
    AddressSpacePrepared,
    UserMappingLive(u8),
    UserMappingStored(u8),
    UserMappingEntryInstalled(u8),
    UserMappingProtected(u8),
    UserMappingVerified(u8),
    UserMappingExecutable(u8),
    UserMappingImmutableData(u8),
    UserMappingWritableData(u8),
    AnonymousUserMappingReserved,
    AddressSpaceVerified,
    PreparedWithResidentImage,
    RandomizedEntryReady,
    UserRandomReservedEntryReady,
    UserRandomServiceEntryReady,
    CompletedWithResidentImage,
    CompletedWithResidentImageAnonymousReleased,
    CompletedWithExecutableImage,
    CompletedImageRevoked,
    Revoked,
    Consumed,
}

#[derive(Clone, Copy)]
struct ProviderSymbols {
    describe_boot_memory: SymbolId,
    allocate_physical_frames: SymbolId,
    prepare_static_image: SymbolId,
    zero_static_image_payload: SymbolId,
    zero_static_image_stack_backing: SymbolId,
    publish_static_image_stack_leaf: SymbolId,
    initialize_static_image_stack_entry: SymbolId,
    add_static_image_stack_pagesz: SymbolId,
    add_static_image_stack_entry_fact: SymbolId,
    add_static_image_stack_program_headers: SymbolId,
    add_static_image_stack_executable_name: SymbolId,
    add_static_image_stack_credentials: SymbolId,
    add_static_image_stack_secure_mode: SymbolId,
    add_static_image_stack_clock_tick_rate: SymbolId,
    add_static_image_stack_platform: SymbolId,
    add_static_image_stack_hardware_capabilities: SymbolId,
    add_static_image_stack_load_metadata: SymbolId,
    authorize_static_image_stack_random_slot: SymbolId,
    retain_static_image_prepared_process: SymbolId,
    qualify_static_image_retained_disjoint_work: SymbolId,
    authorize_static_image_entropy_handoff: SymbolId,
    authorize_static_image_entropy_acquisition: SymbolId,
    qualify_static_image_entropy_device: SymbolId,
    qualify_static_image_entropy_transport: SymbolId,
    qualify_static_image_entropy_negotiation: SymbolId,
    qualify_static_image_entropy_queue: SymbolId,
    qualify_static_image_entropy_bus_master: SymbolId,
    qualify_static_image_entropy_dma_buffers: SymbolId,
    qualify_static_image_entropy_split_ring: SymbolId,
    qualify_static_image_entropy_dma_preparation: SymbolId,
    qualify_static_image_entropy_queue_addresses: SymbolId,
    qualify_static_image_entropy_queue_activation: SymbolId,
    qualify_static_image_entropy_request_completion: SymbolId,
    qualify_static_image_entropy_sample_capture: SymbolId,
    qualify_static_image_entropy_sample_quality: SymbolId,
    qualify_static_image_entropy_pool_lifecycle: SymbolId,
    qualify_static_image_entropy_generator_lifecycle: SymbolId,
    deliver_static_image_entropy_initial_random: SymbolId,
    enter_static_image_process: SymbolId,
    restore_static_image_process_entry: SymbolId,
    revoke_static_image_entropy_initial_random: SymbolId,
    revoke_static_image_entropy_acquisition: SymbolId,
    revoke_static_image_entropy_handoff: SymbolId,
    restore_static_image_prepared_process: SymbolId,
    revoke_static_image_stack_random_slot: SymbolId,
    remove_static_image_stack_load_metadata: SymbolId,
    remove_static_image_stack_hardware_capabilities: SymbolId,
    remove_static_image_stack_platform: SymbolId,
    remove_static_image_stack_clock_tick_rate: SymbolId,
    remove_static_image_stack_secure_mode: SymbolId,
    remove_static_image_stack_credentials: SymbolId,
    remove_static_image_stack_executable_name: SymbolId,
    remove_static_image_stack_program_headers: SymbolId,
    remove_static_image_stack_entry_fact: SymbolId,
    remove_static_image_stack_pagesz: SymbolId,
    clear_static_image_stack_entry: SymbolId,
    revoke_static_image_stack_leaf: SymbolId,
    copy_static_image_payload: SymbolId,
    seal_static_image_permissions: SymbolId,
    attach_static_image_address_space: SymbolId,
    activate_static_image_address_space: SymbolId,
    restore_static_image_address_space: SymbolId,
    authorize_static_image_entry: SymbolId,
    authorize_static_image_stack_slot: SymbolId,
    revoke_static_image_stack_slot: SymbolId,
    revoke_static_image_entry: SymbolId,
    detach_static_image_address_space: SymbolId,
    revoke_static_image: SymbolId,
    begin_translation: SymbolId,
    commit_translation: SymbolId,
    activate_translation: SymbolId,
    begin_translation_edit: SymbolId,
    map_translation_frames: SymbolId,
    unmap_translation_mapping: SymbolId,
    commit_translation_edit: SymbolId,
    enter_critical: SymbolId,
    restore_critical: SymbolId,
    atomic_word_create: SymbolId,
    atomic_word_compare_exchange: SymbolId,
    atomic_word_load: SymbolId,
    send_local_notification: SymbolId,
    wait_local_notification: SymbolId,
    complete_local_notification: SymbolId,
    observe_monotonic_clock: SymbolId,
    observe_pci_configuration_dword: SymbolId,
    update_pci_configuration_word: SymbolId,
    create_dma_buffer: SymbolId,
    bind_dma_buffer_reference: SymbolId,
    store_dma_buffer_integer: SymbolId,
    prepare_dma_buffer_for_device: SymbolId,
    complete_dma_buffer_for_cpu: SymbolId,
    load_dma_buffer_integer: SymbolId,
    capture_entropy_sample: SymbolId,
    mix_entropy_pool_sample: SymbolId,
    initialize_entropy_generator: SymbolId,
    extract_entropy_generator: SymbolId,
    deliver_initial_auxv_random: SymbolId,
    reserve_initial_user_random: SymbolId,
    bind_reusable_user_random: SymbolId,
    publish_pci_region_dma_reference: SymbolId,
    dma_virtqueue_control: SymbolId,
    dma_entropy_payload: SymbolId,
    raw_entropy_sample: SymbolId,
    entropy_pool_state: SymbolId,
    entropy_generator_key: SymbolId,
    entropy_output: SymbolId,
    entropy_user_random_service: SymbolId,
    static_process_brk_reserve: SymbolId,
    read_pci_region_register: SymbolId,
    read_pci_region_register_eventually: SymbolId,
    write_pci_region_register: SymbolId,
    construct_deadline: SymbolId,
    arm_deadline: SymbolId,
    wait_deadline: SymbolId,
    complete_deadline: SymbolId,
    create_kernel_context: SymbolId,
    transfer_kernel_context: SymbolId,
    await_deadline_preemption: SymbolId,
    preempt_current_kernel_context: SymbolId,
    retire_kernel_context: SymbolId,
    reclaim_kernel_context: SymbolId,
    prepare_user_address_space: SymbolId,
    verify_user_address_space_mapping: SymbolId,
    map_user_address_space: SymbolId,
    store_user_mapping_byte: SymbolId,
    install_user_mapping_entry_trampoline: SymbolId,
    protect_user_mapping_read_only: SymbolId,
    load_user_mapping_byte: SymbolId,
    verify_user_mapping_entry_trampoline: SymbolId,
    permit_user_mapping_execute: SymbolId,
    seal_user_mapping_immutable_data: SymbolId,
    permit_user_mapping_writable_data: SymbolId,
    reserve_anonymous_user_mapping: SymbolId,
    release_anonymous_user_mapping: SymbolId,
    unmap_user_address_space: SymbolId,
    prepare_user_probe: SymbolId,
    enter_user_context: SymbolId,
    complete_user_backing: SymbolId,
    reclaim_user_address_space: SymbolId,
    user_syscall: SymbolId,
    kernel_thread_entry: Option<SymbolId>,
    completion_thread_entry: Option<SymbolId>,
    terminal_thread_entry: Option<SymbolId>,
    uart16550_write: SymbolId,
    debug_break: SymbolId,
    interrupt_return: SymbolId,
    fatal: SymbolId,
    bootstrap_storage: Option<SymbolId>,
}

impl RootEncoder {
    fn describe_boot_memory(&mut self) -> Result<(), CompileError> {
        self.bytes.extend_from_slice(&[0x49, 0x89, 0xf5]); // retain boot params in r13
        self.call_checked_bool(ProviderSymbol::DescribeBootMemory)
    }

    fn create_frame_allocator(&mut self) -> Result<(), CompileError> {
        if self.frame_allocator_created {
            return Err(CompileError::Tool(
                "x86 root lowering encountered duplicate frame-allocator creation".into(),
            ));
        }
        self.frame_allocator_created = true;
        Ok(())
    }

    fn allocate_physical_frames(
        &mut self,
        frame_count: u64,
        alignment_frames: u64,
    ) -> Result<(), CompileError> {
        let nested_static_elf_metadata = self.physical_frame_count == Some(503)
            && frame_count == 1
            && !self.retained_static_elf_payload;
        let nested_static_elf_hierarchy = self.static_image == RootStaticImageState::Sealed
            && self.physical_frame_count.is_none()
            && frame_count == 5
            && !self.retained_static_elf_payload;
        let nested_static_elf_stack_backing = self.static_image
            == RootStaticImageState::StackSlotAuthorized
            && self.physical_frame_count.is_none()
            && frame_count == 1
            && !self.retained_static_elf_payload;
        let retained_static_process_work = self.static_image
            == RootStaticImageState::PreparedProcessRetained
            && self.physical_frame_count.is_none()
            && frame_count == 1
            && !self.retained_static_elf_payload;
        let post_random_anonymous_extent = self.user_execution
            == RootUserExecutionState::UserMappingWritableData(2)
            && self.physical_frame_count.is_none()
            && frame_count == 34;
        if !self.frame_allocator_created
            || (self.physical_frame_count.is_some() && !nested_static_elf_metadata)
            || self.kernel_mapping_live
            || !matches!(frame_count, 1..=3 | 5 | 34 | 503)
            || alignment_frames != 1
            || (frame_count == 5 && !nested_static_elf_hierarchy)
            || (self.static_image == RootStaticImageState::Sealed && !nested_static_elf_hierarchy)
            || (self.static_image == RootStaticImageState::StackSlotAuthorized
                && !nested_static_elf_stack_backing)
            || (frame_count == 34 && !post_random_anonymous_extent)
            || (self.user_execution == RootUserExecutionState::AddressSpacePrepared
                && frame_count != 2)
        {
            return Err(CompileError::Tool(
                "x86 root lowering requires one live allocator and an admitted static frame-extent request".into(),
            ));
        }
        let resident_code = self.user_execution == RootUserExecutionState::UserMappingExecutable(1);
        if retained_static_process_work {
            self.bytes.extend_from_slice(&[0x48, 0x83, 0xec, 0x10]);
            self.bytes.extend_from_slice(&[0x48, 0x89, 0x1c, 0x24]);
            self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf1]);
            self.bytes.extend_from_slice(&[0x4c, 0x8d, 0x83]);
            self.bytes.extend_from_slice(&0x1000_u32.to_le_bytes());
        } else if nested_static_elf_stack_backing {
            self.bytes.extend_from_slice(&[0x48, 0x89, 0xdd]); // retain hierarchy in rbp
            self.bytes
                .extend_from_slice(&[0x48, 0x8d, 0xbb, 0x00, 0x50, 0x00, 0x00]); // floor after hierarchy
            self.bytes.extend_from_slice(&[0x31, 0xc9]); // no separate exclusion
            self.bytes.extend_from_slice(&[0x45, 0x31, 0xc0]);
        } else if nested_static_elf_hierarchy {
            self.bytes.extend_from_slice(&[0x49, 0x89, 0xdc]); // retain leaf metadata in r12
            self.bytes
                .extend_from_slice(&[0x48, 0x8d, 0xbb, 0x00, 0x10, 0x00, 0x00]); // floor after leaf table
            self.bytes.extend_from_slice(&[0x31, 0xc9]); // no separate exclusion
            self.bytes.extend_from_slice(&[0x45, 0x31, 0xc0]);
        } else if nested_static_elf_metadata {
            self.bytes.extend_from_slice(&[0x48, 0x89, 0xd9]); // payload base in rcx
            self.bytes.extend_from_slice(&[0x4c, 0x8d, 0x83]); // payload end in r8
            self.bytes.extend_from_slice(&0x001f_7000_u32.to_le_bytes());
            self.bytes.extend_from_slice(&[0x49, 0x89, 0xde]); // retain payload in r14
        } else if matches!(
            self.user_execution,
            RootUserExecutionState::AddressSpacePrepared
                | RootUserExecutionState::UserMappingExecutable(1)
        ) {
            self.load_bootstrap_storage_address(176)?;
            self.bytes.extend_from_slice(&[0x48, 0x8b, 0x4f, 0x68]); // excluded backing base in rcx
            self.bytes.extend_from_slice(&[0x4c, 0x8d, 0x81]); // excluded backing end in r8
            self.bytes.extend_from_slice(&0x0000_3000_u32.to_le_bytes());
        } else {
            self.bytes.extend_from_slice(&[0x31, 0xc9]); // no excluded extent
            self.bytes.extend_from_slice(&[0x45, 0x31, 0xc0]); // no excluded end
        }
        if nested_static_elf_hierarchy || nested_static_elf_stack_backing {
            // The exact monotonic candidate and empty exclusion were prepared above.
        } else if resident_code {
            self.bytes.extend_from_slice(&[0x48, 0x8b, 0x7f, 0x78]); // code frame base
            self.bytes
                .extend_from_slice(&[0x48, 0x81, 0xc7, 0x00, 0x20, 0x00, 0x00]); // after code extent
        } else {
            match self.translation {
                RootTranslationState::BootstrapActive => {
                    self.bytes
                        .extend_from_slice(&[0xbf, 0x00, 0x00, 0x00, 0x01]);
                }
                RootTranslationState::ReplacementActive | RootTranslationState::RemovedActive => {
                    self.bytes
                        .extend_from_slice(&[0x49, 0x8d, 0xbf, 0x00, 0x50, 0x00, 0x00]);
                }
                _ => {
                    return Err(CompileError::Tool(
                    "x86 root lowering encountered frame allocation in an unavailable translation state".into(),
                ));
                }
            }
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xee]); // rsi = retained boot params
        let frame_count = u32::try_from(frame_count)
            .map_err(|_| CompileError::Tool("physical frame count exceeds u32".into()))?;
        self.bytes.push(0xba); // edx = frame count
        self.bytes.extend_from_slice(&frame_count.to_le_bytes());
        self.call(ProviderSymbol::AllocatePhysicalFrames);
        self.bytes.extend_from_slice(&[0x48, 0x85, 0xc0]); // test rax, rax
        self.jump_to_fatal_if(0x84)?;
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xc3]); // retain physical base in rbx
        self.physical_frame_count = Some(u64::from(frame_count));
        if nested_static_elf_metadata {
            self.retained_static_elf_payload = true;
        }
        Ok(())
    }

    fn map_kernel_frames(
        &mut self,
        request: CompilerKernelMappingRequest,
    ) -> Result<(), CompileError> {
        if self.physical_frame_count != Some(1)
            || self.kernel_mapping_live
            || request != CompilerKernelMappingRequest::initial_read_write()
        {
            return Err(CompileError::Tool(
                "x86 root lowering requires one live frame extent and the sealed mapping policy"
                    .into(),
            ));
        }
        self.physical_frame_count = None;
        self.kernel_mapping_live = true;
        Ok(())
    }

    fn store_kernel_mapping_byte(
        &mut self,
        offset_bytes: u64,
        value: u8,
    ) -> Result<(), CompileError> {
        if !self.kernel_mapping_live || offset_bytes >= 4096 {
            return Err(CompileError::Tool(
                "x86 root lowering requires an in-bounds live kernel mapping store".into(),
            ));
        }
        let offset = u32::try_from(offset_bytes)
            .map_err(|_| CompileError::Tool("kernel mapping offset exceeds u32".into()))?;
        if self.translation == RootTranslationState::MappedActive {
            self.bytes.extend_from_slice(&[0x41, 0xc6, 0x86]); // [r14+disp32]
        } else {
            self.bytes.extend_from_slice(&[0xc6, 0x83]); // [rbx+disp32]
        }
        self.bytes.extend_from_slice(&offset.to_le_bytes());
        self.bytes.push(value);
        Ok(())
    }

    fn load_kernel_mapping_byte_equals(
        &mut self,
        offset_bytes: u64,
        expected: u8,
    ) -> Result<(), CompileError> {
        if !self.kernel_mapping_live || offset_bytes >= 4096 {
            return Err(CompileError::Tool(
                "x86 root lowering requires an in-bounds live kernel mapping load".into(),
            ));
        }
        let offset = u32::try_from(offset_bytes)
            .map_err(|_| CompileError::Tool("kernel mapping offset exceeds u32".into()))?;
        if self.translation == RootTranslationState::MappedActive {
            self.bytes.extend_from_slice(&[0x41, 0x0f, 0xb6, 0x86]); // [r14+disp32]
        } else {
            self.bytes.extend_from_slice(&[0x0f, 0xb6, 0x83]); // [rbx+disp32]
        }
        self.bytes.extend_from_slice(&offset.to_le_bytes());
        self.bytes.extend_from_slice(&[0x3c, expected]); // cmp al, imm8
        self.jump_to_fatal_if(0x85)
    }

    fn unmap_kernel_frames(&mut self) -> Result<(), CompileError> {
        if !self.kernel_mapping_live || self.physical_frame_count.is_some() {
            return Err(CompileError::Tool(
                "x86 root lowering encountered unmap without a live kernel mapping".into(),
            ));
        }
        self.kernel_mapping_live = false;
        self.physical_frame_count = Some(1);
        Ok(())
    }

    fn release_physical_frames(&mut self) -> Result<(), CompileError> {
        let Some(frame_count) = self.physical_frame_count else {
            return Err(CompileError::Tool(
                "x86 root lowering encountered release without a live physical-frame extent".into(),
            ));
        };
        if matches!(
            self.static_image,
            RootStaticImageState::Sealed | RootStaticImageState::HierarchyReturned
        ) && frame_count == 5
        {
            self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe3]); // restore leaf metadata
            self.bytes.extend_from_slice(&[0x4d, 0x31, 0xe4]); // clear hierarchy identity
            self.bytes.extend_from_slice(&[0x31, 0xd2]);
            self.physical_frame_count = None;
            self.static_image = RootStaticImageState::Sealed;
            return Ok(());
        }
        if self.static_image == RootStaticImageState::StackBackingZeroed && frame_count == 1 {
            self.bytes.extend_from_slice(&[0x48, 0x31, 0xdb]); // clear backing identity
            self.bytes.extend_from_slice(&[0x48, 0x89, 0xeb]); // restore hierarchy identity
            self.bytes.extend_from_slice(&[0x48, 0x31, 0xed]); // clear retained hierarchy copy
            self.bytes.extend_from_slice(&[0x31, 0xd2]);
            self.physical_frame_count = None;
            self.static_image = RootStaticImageState::StackSlotAuthorized;
            return Ok(());
        }
        if self.static_image == RootStaticImageState::PreparedProcessRetained && frame_count == 1 {
            self.bytes.extend_from_slice(&[0x48, 0x31, 0xdb]);
            self.bytes.extend_from_slice(&[0x48, 0x8b, 0x1c, 0x24]);
            self.bytes.extend_from_slice(&[0x48, 0x83, 0xc4, 0x10]);
            self.bytes.extend_from_slice(&[0x31, 0xd2]);
            self.physical_frame_count = None;
            return Ok(());
        }
        if self.retained_static_elf_payload {
            if frame_count != 1 {
                return Err(CompileError::Tool(
                    "x86 root lowering requires metadata release before its retained static ELF payload".into(),
                ));
            }
            self.bytes.extend_from_slice(&[0x48, 0x31, 0xdb]); // clear metadata base
            self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf3]); // restore payload base
            self.bytes.extend_from_slice(&[0x4d, 0x31, 0xf6]); // clear retained payload register
            self.bytes.extend_from_slice(&[0x31, 0xd2]); // clear transient frame count
            self.physical_frame_count = Some(503);
            self.retained_static_elf_payload = false;
            return Ok(());
        }
        self.physical_frame_count = None;
        self.bytes.extend_from_slice(&[0x48, 0x31, 0xdb]); // clear retained physical base
        self.bytes.extend_from_slice(&[0x31, 0xd2]); // clear transient frame count
        Ok(())
    }

    fn prepare_static_image_construction(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.translation != RootTranslationState::RemovedActive
            || self.static_image != RootStaticImageState::Fresh
            || self.physical_frame_count != Some(1)
            || !self.retained_static_elf_payload
        {
            return Err(CompileError::Tool(
                "x86 static-image preparation requires the exact live payload and metadata extents"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]); // payload base
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xde]); // metadata base
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]); // 503 leaves
        self.call_checked_bool(ProviderSymbol::PrepareStaticImage)?;
        self.physical_frame_count = None;
        self.retained_static_elf_payload = false;
        self.static_image = RootStaticImageState::Live;
        Ok(())
    }

    fn revoke_static_image_construction(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::Sealed
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
        {
            return Err(CompileError::Tool(
                "x86 static-image revocation requires one live inactive construction".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]); // payload base
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xde]); // metadata base
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]); // 503 leaves
        self.call_checked_bool(ProviderSymbol::RevokeStaticImage)?;
        self.physical_frame_count = Some(1);
        self.retained_static_elf_payload = true;
        self.static_image = RootStaticImageState::MetadataReturned;
        Ok(())
    }

    fn zero_static_image_payload(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::Live
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
        {
            return Err(CompileError::Tool(
                "x86 static-image payload zeroing requires one live uninitialized construction"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]); // payload base
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xde]); // metadata base
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]); // 503 leaves
        self.call_checked_bool(ProviderSymbol::ZeroStaticImagePayload)?;
        self.static_image = RootStaticImageState::Zeroed;
        Ok(())
    }

    fn copy_static_image_payload(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::Zeroed
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
        {
            return Err(CompileError::Tool(
                "x86 static-image payload copy requires one completely zeroed construction".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]); // payload base
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xde]); // metadata base
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]); // 503 leaves
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe9]); // boot params
        self.call_checked_bool(ProviderSymbol::CopyStaticImagePayload)?;
        self.static_image = RootStaticImageState::Populated;
        Ok(())
    }

    fn seal_static_image_permissions(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::Populated
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
        {
            return Err(CompileError::Tool(
                "x86 static-image permission sealing requires one populated construction".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]); // payload base
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xde]); // metadata base
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]); // 503 leaves
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe9]); // boot params
        self.call_checked_bool(ProviderSymbol::SealStaticImagePermissions)?;
        self.static_image = RootStaticImageState::Sealed;
        Ok(())
    }

    fn attach_static_image_address_space(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::Sealed
            || self.physical_frame_count != Some(5)
            || self.retained_static_elf_payload
        {
            return Err(CompileError::Tool(
                "x86 static-image address-space attachment requires one sealed construction and its live hierarchy"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]); // payload base
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]); // leaf metadata base
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]); // 503 leaves
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xd9]); // hierarchy base
        self.call_checked_bool(ProviderSymbol::AttachStaticImageAddressSpace)?;
        self.physical_frame_count = None;
        self.static_image = RootStaticImageState::Attached;
        Ok(())
    }

    fn detach_static_image_address_space(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::EntryRestored
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
        {
            return Err(CompileError::Tool(
                "x86 static-image address-space detachment requires one attached inactive construction"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xd9]);
        self.call_checked_bool(ProviderSymbol::DetachStaticImageAddressSpace)?;
        self.static_image = RootStaticImageState::Detached;
        Ok(())
    }

    fn activate_static_image_address_space(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::Attached
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image address-space activation requires one fresh attached construction and the qualified bootstrap root"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xd9]); // static root
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]); // bootstrap root
        self.call_checked_bool(ProviderSymbol::ActivateStaticImageAddressSpace)?;
        self.static_image = RootStaticImageState::Active;
        Ok(())
    }

    fn restore_static_image_address_space(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::Active
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image address-space restoration requires one active construction and the qualified bootstrap root"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xd9]);
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]);
        self.call_checked_bool(ProviderSymbol::RestoreStaticImageAddressSpace)?;
        self.static_image = RootStaticImageState::Restored;
        Ok(())
    }

    fn authorize_static_image_entry(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::Restored
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image entry authorization requires one restored attached construction and the qualified bootstrap root"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xd9]);
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]);
        self.call_checked_bool(ProviderSymbol::AuthorizeStaticImageEntry)?;
        self.static_image = RootStaticImageState::EntryAuthorized;
        Ok(())
    }

    fn authorize_static_image_stack_slot(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::EntryAuthorized
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image stack-slot authorization requires one entry-authorized construction and the qualified bootstrap root"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xd9]);
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]);
        self.call_checked_bool(ProviderSymbol::AuthorizeStaticImageStackSlot)?;
        self.static_image = RootStaticImageState::StackSlotAuthorized;
        Ok(())
    }

    fn revoke_static_image_stack_slot(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::StackSlotAuthorized
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image stack-slot revocation requires one slot-authorized construction and the qualified bootstrap root"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xd9]);
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]);
        self.call_checked_bool(ProviderSymbol::RevokeStaticImageStackSlot)?;
        self.static_image = RootStaticImageState::EntryAuthorized;
        Ok(())
    }

    fn zero_static_image_stack_backing(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::StackSlotAuthorized
            || self.physical_frame_count != Some(1)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image stack-backing zeroing requires one slot-authorized construction and its live one-frame backing"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xdf]); // opaque backing base
        self.call_checked_bool(ProviderSymbol::ZeroStaticImageStackBacking)?;
        self.static_image = RootStaticImageState::StackBackingZeroed;
        Ok(())
    }

    fn publish_static_image_stack_leaf(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::StackBackingZeroed
            || self.physical_frame_count != Some(1)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image stack-leaf publication requires one zeroed backing under the inactive qualified hierarchy"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]); // payload
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]); // leaf metadata
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xe9]); // hierarchy
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]); // bootstrap root
        self.bytes.extend_from_slice(&[0x49, 0x89, 0xd9]); // backing
        self.call_checked_bool(ProviderSymbol::PublishStaticImageStackLeaf)?;
        self.static_image = RootStaticImageState::StackLeafPublished;
        Ok(())
    }

    fn initialize_static_image_stack_entry(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::StackLeafPublished
            || self.physical_frame_count != Some(1)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image stack-entry initialization requires one published stack leaf under the inactive qualified hierarchy"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xe9]);
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]);
        self.bytes.extend_from_slice(&[0x49, 0x89, 0xd9]);
        self.call_checked_bool(ProviderSymbol::InitializeStaticImageStackEntry)?;
        self.static_image = RootStaticImageState::StackEntryInitialized;
        Ok(())
    }

    fn clear_static_image_stack_entry(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::StackEntryInitialized
            || self.physical_frame_count != Some(1)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image stack-entry clearing requires one initialized stack entry under the inactive qualified hierarchy"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xe9]);
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]);
        self.bytes.extend_from_slice(&[0x49, 0x89, 0xd9]);
        self.call_checked_bool(ProviderSymbol::ClearStaticImageStackEntry)?;
        self.static_image = RootStaticImageState::StackLeafPublished;
        Ok(())
    }

    fn add_static_image_stack_pagesz(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::StackEntryInitialized
            || self.physical_frame_count != Some(1)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image AT_PAGESZ addition requires one initialized stack entry under the inactive qualified hierarchy"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xe9]);
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]);
        self.bytes.extend_from_slice(&[0x49, 0x89, 0xd9]);
        self.call_checked_bool(ProviderSymbol::AddStaticImageStackPageSize)?;
        self.static_image = RootStaticImageState::StackPageSizeAdded;
        Ok(())
    }

    fn add_static_image_stack_entry_fact(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::StackPageSizeAdded
            || self.physical_frame_count != Some(1)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image AT_ENTRY addition requires one page-size-qualified stack entry under the inactive qualified hierarchy"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xe9]);
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]);
        self.bytes.extend_from_slice(&[0x49, 0x89, 0xd9]);
        self.call_checked_bool(ProviderSymbol::AddStaticImageStackEntryFact)?;
        self.static_image = RootStaticImageState::StackEntryFactAdded;
        Ok(())
    }

    fn transition_static_image_stack_program_headers(
        &mut self,
        add: bool,
    ) -> Result<(), CompileError> {
        let required = if add {
            RootStaticImageState::StackEntryFactAdded
        } else {
            RootStaticImageState::StackProgramHeadersAdded
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count != Some(1)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image program-header transition requires its exact qualified stack owner under the inactive hierarchy"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xe9]);
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]);
        self.bytes.extend_from_slice(&[0x49, 0x89, 0xd9]);
        self.call_checked_bool(if add {
            ProviderSymbol::AddStaticImageStackProgramHeaders
        } else {
            ProviderSymbol::RemoveStaticImageStackProgramHeaders
        })?;
        self.static_image = if add {
            RootStaticImageState::StackProgramHeadersAdded
        } else {
            RootStaticImageState::StackEntryFactAdded
        };
        Ok(())
    }

    fn add_static_image_stack_program_headers(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_program_headers(true)
    }

    fn transition_static_image_stack_executable_name(
        &mut self,
        add: bool,
    ) -> Result<(), CompileError> {
        let required = if add {
            RootStaticImageState::StackProgramHeadersAdded
        } else {
            RootStaticImageState::StackExecutableNameAdded
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count != Some(1)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image executable-name transition requires its exact qualified stack owner under the inactive hierarchy".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if add {
            ProviderSymbol::AddStaticImageStackExecutableName
        } else {
            ProviderSymbol::RemoveStaticImageStackExecutableName
        })?;
        self.static_image = if add {
            RootStaticImageState::StackExecutableNameAdded
        } else {
            RootStaticImageState::StackProgramHeadersAdded
        };
        Ok(())
    }

    fn add_static_image_stack_executable_name(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_executable_name(true)
    }
    fn remove_static_image_stack_executable_name(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_executable_name(false)
    }

    fn transition_static_image_stack_credentials(&mut self, add: bool) -> Result<(), CompileError> {
        let required = if add {
            RootStaticImageState::StackExecutableNameAdded
        } else {
            RootStaticImageState::StackCredentialsAdded
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count != Some(1)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image credential transition requires its exact qualified stack owner under the inactive hierarchy".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if add {
            ProviderSymbol::AddStaticImageStackCredentials
        } else {
            ProviderSymbol::RemoveStaticImageStackCredentials
        })?;
        self.static_image = if add {
            RootStaticImageState::StackCredentialsAdded
        } else {
            RootStaticImageState::StackExecutableNameAdded
        };
        Ok(())
    }

    fn add_static_image_stack_credentials(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_credentials(true)
    }

    fn remove_static_image_stack_credentials(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_credentials(false)
    }

    fn transition_static_image_stack_secure_mode(&mut self, add: bool) -> Result<(), CompileError> {
        let required = if add {
            RootStaticImageState::StackCredentialsAdded
        } else {
            RootStaticImageState::StackSecureModeAdded
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count != Some(1)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image secure-mode transition requires its exact qualified stack owner under the inactive hierarchy".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if add {
            ProviderSymbol::AddStaticImageStackSecureMode
        } else {
            ProviderSymbol::RemoveStaticImageStackSecureMode
        })?;
        self.static_image = if add {
            RootStaticImageState::StackSecureModeAdded
        } else {
            RootStaticImageState::StackCredentialsAdded
        };
        Ok(())
    }

    fn add_static_image_stack_secure_mode(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_secure_mode(true)
    }

    fn remove_static_image_stack_secure_mode(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_secure_mode(false)
    }

    fn transition_static_image_stack_clock_tick_rate(
        &mut self,
        add: bool,
    ) -> Result<(), CompileError> {
        let required = if add {
            RootStaticImageState::StackSecureModeAdded
        } else {
            RootStaticImageState::StackClockTickRateAdded
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count != Some(1)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image clock-tick-rate transition requires its exact qualified stack owner under the inactive hierarchy".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if add {
            ProviderSymbol::AddStaticImageStackClockTickRate
        } else {
            ProviderSymbol::RemoveStaticImageStackClockTickRate
        })?;
        self.static_image = if add {
            RootStaticImageState::StackClockTickRateAdded
        } else {
            RootStaticImageState::StackSecureModeAdded
        };
        Ok(())
    }

    fn add_static_image_stack_clock_tick_rate(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_clock_tick_rate(true)
    }

    fn remove_static_image_stack_clock_tick_rate(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_clock_tick_rate(false)
    }

    fn transition_static_image_stack_platform(&mut self, add: bool) -> Result<(), CompileError> {
        let required = if add {
            RootStaticImageState::StackClockTickRateAdded
        } else {
            RootStaticImageState::StackPlatformAdded
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count != Some(1)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image platform transition requires its exact qualified stack owner under the inactive hierarchy".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if add {
            ProviderSymbol::AddStaticImageStackPlatform
        } else {
            ProviderSymbol::RemoveStaticImageStackPlatform
        })?;
        self.static_image = if add {
            RootStaticImageState::StackPlatformAdded
        } else {
            RootStaticImageState::StackClockTickRateAdded
        };
        Ok(())
    }

    fn add_static_image_stack_platform(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_platform(true)
    }

    fn remove_static_image_stack_platform(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_platform(false)
    }

    fn transition_static_image_stack_hardware_capabilities(
        &mut self,
        add: bool,
    ) -> Result<(), CompileError> {
        let required = if add {
            RootStaticImageState::StackPlatformAdded
        } else {
            RootStaticImageState::StackHardwareCapabilitiesAdded
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count != Some(1)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image hardware-capability transition requires its exact qualified stack owner under the inactive hierarchy".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if add {
            ProviderSymbol::AddStaticImageStackHardwareCapabilities
        } else {
            ProviderSymbol::RemoveStaticImageStackHardwareCapabilities
        })?;
        self.static_image = if add {
            RootStaticImageState::StackHardwareCapabilitiesAdded
        } else {
            RootStaticImageState::StackPlatformAdded
        };
        Ok(())
    }

    fn add_static_image_stack_hardware_capabilities(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_hardware_capabilities(true)
    }

    fn remove_static_image_stack_hardware_capabilities(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_hardware_capabilities(false)
    }

    fn transition_static_image_stack_load_metadata(
        &mut self,
        add: bool,
    ) -> Result<(), CompileError> {
        let required = if add {
            RootStaticImageState::StackHardwareCapabilitiesAdded
        } else {
            RootStaticImageState::StackLoadMetadataAdded
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count != Some(1)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image load-metadata transition requires its exact qualified stack owner under the inactive hierarchy".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if add {
            ProviderSymbol::AddStaticImageStackLoadMetadata
        } else {
            ProviderSymbol::RemoveStaticImageStackLoadMetadata
        })?;
        self.static_image = if add {
            RootStaticImageState::StackLoadMetadataAdded
        } else {
            RootStaticImageState::StackHardwareCapabilitiesAdded
        };
        Ok(())
    }

    fn add_static_image_stack_load_metadata(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_load_metadata(true)
    }

    fn remove_static_image_stack_load_metadata(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_load_metadata(false)
    }

    fn transition_static_image_stack_random_slot(
        &mut self,
        authorize: bool,
    ) -> Result<(), CompileError> {
        let required = if authorize {
            RootStaticImageState::StackLoadMetadataAdded
        } else {
            RootStaticImageState::StackRandomSlotAuthorized
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count != Some(1)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image random-slot transition requires its exact qualified stack owner under the inactive hierarchy".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if authorize {
            ProviderSymbol::AuthorizeStaticImageStackRandomSlot
        } else {
            ProviderSymbol::RevokeStaticImageStackRandomSlot
        })?;
        self.static_image = if authorize {
            RootStaticImageState::StackRandomSlotAuthorized
        } else {
            RootStaticImageState::StackLoadMetadataAdded
        };
        Ok(())
    }

    fn authorize_static_image_stack_random_slot(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_random_slot(true)
    }

    fn revoke_static_image_stack_random_slot(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_random_slot(false)
    }

    fn transition_static_image_prepared_process(
        &mut self,
        retain: bool,
    ) -> Result<(), CompileError> {
        let required = if retain {
            RootStaticImageState::StackRandomSlotAuthorized
        } else {
            RootStaticImageState::PreparedProcessRetained
        };
        let required_frames = if retain { Some(1) } else { None };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count != required_frames
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image prepared-process transition requires its exact retained owner under the inactive hierarchy".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if retain {
            ProviderSymbol::RetainStaticImagePreparedProcess
        } else {
            ProviderSymbol::RestoreStaticImagePreparedProcess
        })?;
        self.static_image = if retain {
            RootStaticImageState::PreparedProcessRetained
        } else {
            RootStaticImageState::StackRandomSlotAuthorized
        };
        self.physical_frame_count = if retain { None } else { Some(1) };
        Ok(())
    }

    fn retain_static_image_prepared_process(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_prepared_process(true)
    }

    fn restore_static_image_prepared_process(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_prepared_process(false)
    }

    fn qualify_static_image_retained_disjoint_work(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::PreparedProcessRetained
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image retained disjoint work requires the exact retained owner and closed allocator channel".into()));
        }
        self.allocate_physical_frames(1, 1)?;
        self.release_physical_frames()?;
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(ProviderSymbol::QualifyStaticImageRetainedDisjointWork)
    }

    fn transition_static_image_entropy_handoff(
        &mut self,
        authorize: bool,
    ) -> Result<(), CompileError> {
        let required = if authorize {
            RootStaticImageState::PreparedProcessRetained
        } else {
            RootStaticImageState::EntropyPendingProcess
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image entropy-handoff transition requires its exact opaque process owner".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if authorize {
            ProviderSymbol::AuthorizeStaticImageEntropyHandoff
        } else {
            ProviderSymbol::RevokeStaticImageEntropyHandoff
        })?;
        self.static_image = if authorize {
            RootStaticImageState::EntropyPendingProcess
        } else {
            RootStaticImageState::PreparedProcessRetained
        };
        Ok(())
    }

    fn authorize_static_image_entropy_handoff(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_entropy_handoff(true)
    }

    fn revoke_static_image_entropy_handoff(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_entropy_handoff(false)
    }

    fn transition_static_image_entropy_acquisition(
        &mut self,
        authorize: bool,
    ) -> Result<(), CompileError> {
        let required = if authorize {
            RootStaticImageState::EntropyPendingProcess
        } else {
            RootStaticImageState::EntropyAcquiringProcess
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != required
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image entropy-acquisition transition requires its exact opaque process owner".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if authorize {
            ProviderSymbol::AuthorizeStaticImageEntropyAcquisition
        } else {
            ProviderSymbol::RevokeStaticImageEntropyAcquisition
        })?;
        self.static_image = if authorize {
            RootStaticImageState::EntropyAcquiringProcess
        } else {
            RootStaticImageState::EntropyPendingProcess
        };
        Ok(())
    }

    fn authorize_static_image_entropy_acquisition(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_entropy_acquisition(true)
    }

    fn qualify_static_image_entropy_device(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::EntropyAcquiringProcess
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image entropy-device qualification requires the exact acquisition owner".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(ProviderSymbol::QualifyStaticImageEntropyDevice)?;
        self.bytes.push(0xbf);
        self.bytes.extend_from_slice(&0x8000_2800_u32.to_le_bytes());
        self.bytes.push(0xbe);
        self.bytes.extend_from_slice(&0x1044_1af4_u32.to_le_bytes());
        self.call_checked_bool(ProviderSymbol::ObservePciConfigurationDword)
    }

    fn qualify_static_image_entropy_transport(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::EntropyAcquiringProcess
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image entropy-transport qualification requires the exact acquisition owner".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(ProviderSymbol::QualifyStaticImageEntropyTransport)?;
        for request in CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
            .iter()
            .skip(1)
        {
            let selector = 0x8000_0000
                | (u32::from(request.bus) << 16)
                | (u32::from(request.device) << 11)
                | (u32::from(request.function) << 8)
                | u32::from(request.register_offset_bytes);
            self.bytes.push(0xbf);
            self.bytes.extend_from_slice(&selector.to_le_bytes());
            self.bytes.push(0xbe);
            self.bytes
                .extend_from_slice(&request.expected_value.to_le_bytes());
            self.call_checked_bool(ProviderSymbol::ObservePciConfigurationDword)?;
        }
        Ok(())
    }

    fn qualify_static_image_entropy_negotiation(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::EntropyAcquiringProcess
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image entropy-transport negotiation requires the exact acquisition owner".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(ProviderSymbol::QualifyStaticImageEntropyNegotiation)?;
        for request in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        for request in [
            CompilerPciRegionRegisterRequest::initial(
                0x14,
                1,
                CompilerPciRegionRegisterDirection::Write,
                0,
            ),
            CompilerPciRegionRegisterRequest::initial(
                0x14,
                1,
                CompilerPciRegionRegisterDirection::ReadEquals,
                0,
            ),
        ] {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        Ok(())
    }

    fn qualify_static_image_entropy_queue(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::EntropyAcquiringProcess
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image entropy-queue discovery requires the exact acquisition owner"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(ProviderSymbol::QualifyStaticImageEntropyQueue)?;
        for request in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        for request in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        for request in [
            CompilerPciRegionRegisterRequest::initial(
                0x14,
                1,
                CompilerPciRegionRegisterDirection::Write,
                0,
            ),
            CompilerPciRegionRegisterRequest::initial(
                0x14,
                1,
                CompilerPciRegionRegisterDirection::ReadEquals,
                0,
            ),
        ] {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        Ok(())
    }

    fn qualify_static_image_entropy_bus_master(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::EntropyAcquiringProcess
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image entropy bus-master qualification requires the exact acquisition owner".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(ProviderSymbol::QualifyStaticImageEntropyBusMaster)?;
        for request in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        for request in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        let enable = CompilerPciConfigurationWordUpdateRequest::INITIAL_Q35_VIRTIO_RNG_BUS_MASTER;
        self.emit_unaccounted_pci_configuration_word_update(enable)?;
        self.emit_unaccounted_pci_configuration_word_update(
            CompilerPciConfigurationWordUpdateRequest {
                expected_masked_value: enable.desired_masked_value,
                desired_masked_value: enable.expected_masked_value,
                ..enable
            },
        )?;
        for request in [
            CompilerPciRegionRegisterRequest::initial(
                0x14,
                1,
                CompilerPciRegionRegisterDirection::Write,
                0,
            ),
            CompilerPciRegionRegisterRequest::initial(
                0x14,
                1,
                CompilerPciRegionRegisterDirection::ReadEquals,
                0,
            ),
        ] {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        Ok(())
    }

    fn qualify_static_image_entropy_dma_buffers(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::EntropyAcquiringProcess
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool("x86 static-image entropy DMA-buffer coexistence requires the exact acquisition owner".into()));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(ProviderSymbol::QualifyStaticImageEntropyDmaBuffers)?;
        for request in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        for request in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        let enable = CompilerPciConfigurationWordUpdateRequest::INITIAL_Q35_VIRTIO_RNG_BUS_MASTER;
        self.emit_unaccounted_pci_configuration_word_update(enable)?;
        for request in CompilerDmaBufferRequest::INITIAL_Q35_VIRTIO_RNG {
            self.emit_unaccounted_dma_buffer_create(request)?;
        }
        self.emit_unaccounted_pci_configuration_word_update(
            CompilerPciConfigurationWordUpdateRequest {
                expected_masked_value: enable.desired_masked_value,
                desired_masked_value: enable.expected_masked_value,
                ..enable
            },
        )?;
        for request in [
            CompilerPciRegionRegisterRequest::initial(
                0x14,
                1,
                CompilerPciRegionRegisterDirection::Write,
                0,
            ),
            CompilerPciRegionRegisterRequest::initial(
                0x14,
                1,
                CompilerPciRegionRegisterDirection::ReadEquals,
                0,
            ),
        ] {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        Ok(())
    }

    fn qualify_static_image_entropy_split_ring(&mut self) -> Result<(), CompileError> {
        self.qualify_static_image_entropy_ring_transaction(
            ProviderSymbol::QualifyStaticImageEntropySplitRing,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        )
    }

    fn qualify_static_image_entropy_dma_preparation(&mut self) -> Result<(), CompileError> {
        self.qualify_static_image_entropy_ring_transaction(
            ProviderSymbol::QualifyStaticImageEntropyDmaPreparation,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        )
    }

    fn qualify_static_image_entropy_queue_addresses(&mut self) -> Result<(), CompileError> {
        self.qualify_static_image_entropy_ring_transaction(
            ProviderSymbol::QualifyStaticImageEntropyQueueAddresses,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
        )
    }

    fn qualify_static_image_entropy_queue_activation(&mut self) -> Result<(), CompileError> {
        self.qualify_static_image_entropy_ring_transaction(
            ProviderSymbol::QualifyStaticImageEntropyQueueActivation,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
            false,
        )
    }

    fn qualify_static_image_entropy_request_completion(&mut self) -> Result<(), CompileError> {
        self.qualify_static_image_entropy_ring_transaction(
            ProviderSymbol::QualifyStaticImageEntropyRequestCompletion,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            false,
        )
    }

    fn qualify_static_image_entropy_sample_capture(&mut self) -> Result<(), CompileError> {
        self.qualify_static_image_entropy_ring_transaction(
            ProviderSymbol::QualifyStaticImageEntropySampleCapture,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
        )
    }

    fn qualify_static_image_entropy_sample_quality(&mut self) -> Result<(), CompileError> {
        self.qualify_static_image_entropy_ring_transaction(
            ProviderSymbol::QualifyStaticImageEntropySampleQuality,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
        )
    }

    fn qualify_static_image_entropy_pool_lifecycle(&mut self) -> Result<(), CompileError> {
        self.qualify_static_image_entropy_ring_transaction(
            ProviderSymbol::QualifyStaticImageEntropyPoolLifecycle,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
        )
    }

    fn qualify_static_image_entropy_generator_lifecycle(&mut self) -> Result<(), CompileError> {
        self.qualify_static_image_entropy_ring_transaction(
            ProviderSymbol::QualifyStaticImageEntropyGeneratorLifecycle,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
        )
    }

    fn deliver_static_image_entropy_initial_random(&mut self) -> Result<(), CompileError> {
        self.qualify_static_image_entropy_ring_transaction(
            ProviderSymbol::DeliverStaticImageEntropyInitialRandom,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
        )?;
        self.static_image = RootStaticImageState::RandomizedProcess;
        Ok(())
    }

    fn revoke_static_image_entropy_initial_random(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::RandomizedProcess
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image initial-random revocation requires the exact randomized owner"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xdf]);
        self.call_checked_bool(ProviderSymbol::RevokeStaticImageEntropyInitialRandom)?;
        self.static_image = RootStaticImageState::EntropyAcquiringProcess;
        Ok(())
    }

    fn enter_static_image_process(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::RandomizedProcess
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image process entry requires the exact randomized owner".into(),
            ));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, // payload
            0x4c, 0x89, 0xe6, // leaf metadata
            0xba, 0xf7, 0x01, 0, 0, // 503 leaves
            0x48, 0x89, 0xe9, // static root
            0x4d, 0x89, 0xf8, // bootstrap root
            0x49, 0x89, 0xd9, // stack backing
            0x48, 0x8d, 0x05, // private brk reserve
        ]);
        self.rip_relative_symbol(ProviderSymbol::StaticProcessBrkReserve, 0)?;
        self.bytes.extend_from_slice(&[
            0x50, // eighth System V argument
            0x48, 0x8d, 0x05, // private context
        ]);
        self.rip_relative_storage(512, 0)?;
        self.bytes.push(0x50); // seventh System V argument
        self.call_checked_bool(ProviderSymbol::EnterStaticImageProcess)?;
        self.bytes.extend_from_slice(&[0x48, 0x83, 0xc4, 0x10]);
        self.static_image = RootStaticImageState::CompletedProcess;
        Ok(())
    }

    fn restore_static_image_process_entry(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::CompletedProcess
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image process-entry restoration requires the exact completed owner"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(512)?;
        self.call_checked_bool(ProviderSymbol::RestoreStaticImageProcessEntry)?;
        self.static_image = RootStaticImageState::RandomizedProcess;
        Ok(())
    }

    fn qualify_static_image_entropy_ring_transaction(
        &mut self,
        provider: ProviderSymbol,
        prepare: bool,
        publish: bool,
        activate: bool,
        complete_request: bool,
        capture_sample: bool,
        assess_sample: bool,
        pool_lifecycle: bool,
        generator_lifecycle: bool,
        deliver_initial_random: bool,
    ) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::EntropyAcquiringProcess
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image entropy ring transaction requires the exact acquisition owner"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[
            0x4c, 0x89, 0xf7, 0x4c, 0x89, 0xe6, 0xba, 0xf7, 0x01, 0, 0, 0x48, 0x89, 0xe9, 0x4d,
            0x89, 0xf8, 0x49, 0x89, 0xd9,
        ]);
        self.call_checked_bool(if deliver_initial_random {
            ProviderSymbol::QualifyStaticImageEntropyGeneratorLifecycle
        } else {
            provider
        })?;
        for request in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        for request in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY {
            self.emit_unaccounted_pci_region_register_access(request)?;
        }
        let enable = CompilerPciConfigurationWordUpdateRequest::INITIAL_Q35_VIRTIO_RNG_BUS_MASTER;
        self.emit_unaccounted_pci_configuration_word_update(enable)?;
        for request in CompilerDmaBufferRequest::INITIAL_Q35_VIRTIO_RNG {
            self.emit_unaccounted_dma_buffer_create(request)?;
        }
        self.emit_unaccounted_dma_buffer_bind_reference(
            CompilerDmaBufferReferenceRequest::INITIAL_Q35_VIRTIO_RNG_DESCRIPTOR,
        )?;
        for request in CompilerDmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING {
            self.emit_unaccounted_dma_buffer_integer_store(request)?;
        }
        if activate && !complete_request {
            self.emit_unaccounted_dma_buffer_integer_store(CompilerDmaBufferIntegerStoreRequest {
                buffer_identity: 1,
                offset_bytes: 130,
                width_bytes: 2,
                value: 0,
            })?;
            self.emit_unaccounted_dma_buffer_integer_load(CompilerDmaBufferIntegerLoadRequest {
                buffer_identity: 1,
                offset_bytes: 130,
                width_bytes: 2,
                expected_value: 0,
            })?;
        }
        if prepare {
            for request in CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG {
                self.emit_unaccounted_dma_buffer_ownership(
                    request,
                    ProviderSymbol::PrepareDmaBufferForDevice,
                )?;
            }
        }
        if publish {
            for request in
                CompilerPciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
            {
                self.emit_unaccounted_pci_region_dma_reference(request)?;
            }
        }
        if activate {
            for request in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
            {
                self.emit_unaccounted_pci_region_register_access(request)?;
            }
            if complete_request {
                for request in
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION
                {
                    self.emit_unaccounted_pci_region_register_access(request)?;
                }
                for request in
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL
                {
                    self.emit_unaccounted_pci_region_register_access(request)?;
                }
                self.emit_unaccounted_dma_buffer_ownership(
                    CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[1],
                    ProviderSymbol::CompleteDmaBufferForCpu,
                )?;
                for request in CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING
                {
                    self.emit_unaccounted_dma_buffer_integer_load(request)?;
                }
                self.emit_unaccounted_dma_buffer_ownership(
                    CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0],
                    ProviderSymbol::CompleteDmaBufferForCpu,
                )?;
                if capture_sample {
                    self.emit_unaccounted_entropy_sample_capture(
                        CompilerEntropySampleCaptureRequest::INITIAL_Q35_VIRTIO_RNG,
                    )?;
                }
                if assess_sample {
                    self.emit_unaccounted_entropy_sample_quality(
                        CompilerEntropySampleQualityRequest::INITIAL_Q35_VIRTIO_RNG,
                    )?;
                }
                if pool_lifecycle {
                    self.emit_unaccounted_entropy_pool_create(
                        CompilerEntropyPoolRequest::INITIAL_Q35,
                    )?;
                    self.emit_unaccounted_entropy_pool_mix(
                        CompilerEntropyPoolMixRequest::INITIAL_Q35,
                    )?;
                    self.emit_unaccounted_entropy_pool_credit(
                        CompilerEntropyPoolCreditRequest::INITIAL_Q35,
                    )?;
                }
                if generator_lifecycle {
                    self.emit_unaccounted_entropy_generator_initialize(
                        CompilerEntropyGeneratorRequest::INITIAL_Q35,
                    )?;
                    self.emit_unaccounted_entropy_generator_extract(
                        CompilerEntropyGeneratorExtractRequest::INITIAL_Q35,
                    )?;
                }
                if deliver_initial_random {
                    self.bytes.extend_from_slice(&[0x48, 0x89, 0xdf]);
                    self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
                    self.rip_relative_symbol(ProviderSymbol::EntropyOutput, 0)?;
                    self.bytes.extend_from_slice(&[0x48, 0x8d, 0x15]);
                    self.rip_relative_storage(512, 0)?;
                    self.bytes.extend_from_slice(&[0x48, 0x8d, 0x0d]);
                    self.rip_relative_symbol(ProviderSymbol::EntropyGeneratorKey, 0)?;
                    self.call_checked_bool(provider)?;
                }
            }
            for request in [
                CompilerPciRegionRegisterRequest::initial(
                    0x14,
                    1,
                    CompilerPciRegionRegisterDirection::Write,
                    0,
                ),
                CompilerPciRegionRegisterRequest::initial(
                    0x14,
                    1,
                    CompilerPciRegionRegisterDirection::ReadEquals,
                    0,
                ),
                CompilerPciRegionRegisterRequest::initial(
                    0x1c,
                    2,
                    CompilerPciRegionRegisterDirection::ReadEquals,
                    0,
                ),
            ] {
                self.emit_unaccounted_pci_region_register_access(request)?;
            }
            for register_offset_bytes in [32, 40, 48] {
                for request in [
                    CompilerPciRegionRegisterRequest::initial(
                        register_offset_bytes,
                        4,
                        CompilerPciRegionRegisterDirection::ReadEquals,
                        0,
                    ),
                    CompilerPciRegionRegisterRequest::initial(
                        register_offset_bytes + 4,
                        4,
                        CompilerPciRegionRegisterDirection::ReadEquals,
                        0,
                    ),
                ] {
                    self.emit_unaccounted_pci_region_register_access(request)?;
                }
            }
        } else if publish {
            for register_offset_bytes in [48, 40, 32] {
                for request in [
                    CompilerPciRegionRegisterRequest::initial(
                        register_offset_bytes + 4,
                        4,
                        CompilerPciRegionRegisterDirection::Write,
                        0,
                    ),
                    CompilerPciRegionRegisterRequest::initial(
                        register_offset_bytes,
                        4,
                        CompilerPciRegionRegisterDirection::Write,
                        0,
                    ),
                    CompilerPciRegionRegisterRequest::initial(
                        register_offset_bytes,
                        4,
                        CompilerPciRegionRegisterDirection::ReadEquals,
                        0,
                    ),
                    CompilerPciRegionRegisterRequest::initial(
                        register_offset_bytes + 4,
                        4,
                        CompilerPciRegionRegisterDirection::ReadEquals,
                        0,
                    ),
                ] {
                    self.emit_unaccounted_pci_region_register_access(request)?;
                }
            }
        }
        if prepare && !complete_request {
            for request in [
                CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[1],
                CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0],
            ] {
                self.emit_unaccounted_dma_buffer_ownership(
                    request,
                    ProviderSymbol::CompleteDmaBufferForCpu,
                )?;
            }
        }
        if complete_request {
            for offset_bytes in [0, 8, 16, 24] {
                self.emit_unaccounted_dma_buffer_integer_store(
                    CompilerDmaBufferIntegerStoreRequest {
                        buffer_identity: 2,
                        offset_bytes,
                        width_bytes: 8,
                        value: 0,
                    },
                )?;
            }
            for offset_bytes in [0, 8, 16, 24] {
                self.emit_unaccounted_dma_buffer_integer_load(
                    CompilerDmaBufferIntegerLoadRequest {
                        buffer_identity: 2,
                        offset_bytes,
                        width_bytes: 8,
                        expected_value: 0,
                    },
                )?;
            }
        }
        for request in [
            CompilerDmaBufferIntegerStoreRequest {
                buffer_identity: 1,
                offset_bytes: 156,
                width_bytes: 4,
                value: 0,
            },
            CompilerDmaBufferIntegerStoreRequest {
                buffer_identity: 1,
                offset_bytes: 152,
                width_bytes: 4,
                value: 0,
            },
            CompilerDmaBufferIntegerStoreRequest {
                buffer_identity: 1,
                offset_bytes: 150,
                width_bytes: 2,
                value: 0,
            },
            CompilerDmaBufferIntegerStoreRequest {
                buffer_identity: 1,
                offset_bytes: 148,
                width_bytes: 2,
                value: 0,
            },
            CompilerDmaBufferIntegerStoreRequest {
                buffer_identity: 1,
                offset_bytes: 130,
                width_bytes: 2,
                value: 0,
            },
            CompilerDmaBufferIntegerStoreRequest {
                buffer_identity: 1,
                offset_bytes: 132,
                width_bytes: 2,
                value: 0,
            },
            CompilerDmaBufferIntegerStoreRequest {
                buffer_identity: 1,
                offset_bytes: 12,
                width_bytes: 2,
                value: 0,
            },
            CompilerDmaBufferIntegerStoreRequest {
                buffer_identity: 1,
                offset_bytes: 8,
                width_bytes: 4,
                value: 0,
            },
            CompilerDmaBufferIntegerStoreRequest {
                buffer_identity: 1,
                offset_bytes: 0,
                width_bytes: 8,
                value: 0,
            },
        ] {
            if complete_request || request.offset_bytes < 148 {
                self.emit_unaccounted_dma_buffer_integer_store(request)?;
            }
        }
        for request in [
            CompilerDmaBufferIntegerLoadRequest {
                buffer_identity: 1,
                offset_bytes: 156,
                width_bytes: 4,
                expected_value: 0,
            },
            CompilerDmaBufferIntegerLoadRequest {
                buffer_identity: 1,
                offset_bytes: 152,
                width_bytes: 4,
                expected_value: 0,
            },
            CompilerDmaBufferIntegerLoadRequest {
                buffer_identity: 1,
                offset_bytes: 150,
                width_bytes: 2,
                expected_value: 0,
            },
            CompilerDmaBufferIntegerLoadRequest {
                buffer_identity: 1,
                offset_bytes: 148,
                width_bytes: 2,
                expected_value: 0,
            },
            CompilerDmaBufferIntegerLoadRequest {
                buffer_identity: 1,
                offset_bytes: 130,
                width_bytes: 2,
                expected_value: 0,
            },
            CompilerDmaBufferIntegerLoadRequest {
                buffer_identity: 1,
                offset_bytes: 132,
                width_bytes: 2,
                expected_value: 0,
            },
            CompilerDmaBufferIntegerLoadRequest {
                buffer_identity: 1,
                offset_bytes: 12,
                width_bytes: 2,
                expected_value: 0,
            },
            CompilerDmaBufferIntegerLoadRequest {
                buffer_identity: 1,
                offset_bytes: 8,
                width_bytes: 4,
                expected_value: 0,
            },
            CompilerDmaBufferIntegerLoadRequest {
                buffer_identity: 1,
                offset_bytes: 0,
                width_bytes: 8,
                expected_value: 0,
            },
        ] {
            if complete_request || request.offset_bytes < 148 {
                self.emit_unaccounted_dma_buffer_integer_load(request)?;
            }
        }
        if capture_sample {
            self.erase_entropy_sample_storage()?;
        }
        if pool_lifecycle {
            self.erase_entropy_pool_storage()?;
        }
        if generator_lifecycle {
            self.erase_entropy_generator_storage()?;
            self.erase_entropy_output_storage()?;
        }
        self.emit_unaccounted_pci_configuration_word_update(
            CompilerPciConfigurationWordUpdateRequest {
                expected_masked_value: enable.desired_masked_value,
                desired_masked_value: enable.expected_masked_value,
                ..enable
            },
        )?;
        if !activate {
            for request in [
                CompilerPciRegionRegisterRequest::initial(
                    0x14,
                    1,
                    CompilerPciRegionRegisterDirection::Write,
                    0,
                ),
                CompilerPciRegionRegisterRequest::initial(
                    0x14,
                    1,
                    CompilerPciRegionRegisterDirection::ReadEquals,
                    0,
                ),
            ] {
                self.emit_unaccounted_pci_region_register_access(request)?;
            }
        }
        Ok(())
    }

    fn erase_entropy_sample_storage(&mut self) -> Result<(), CompileError> {
        self.erase_entropy_storage(ProviderSymbol::RawEntropySample)
    }

    fn erase_entropy_pool_storage(&mut self) -> Result<(), CompileError> {
        self.erase_entropy_storage(ProviderSymbol::EntropyPoolState)
    }

    fn erase_entropy_generator_storage(&mut self) -> Result<(), CompileError> {
        self.erase_entropy_storage(ProviderSymbol::EntropyGeneratorKey)
    }

    fn erase_entropy_output_storage(&mut self) -> Result<(), CompileError> {
        self.erase_entropy_storage(ProviderSymbol::EntropyOutput)
    }

    fn erase_entropy_storage(&mut self, symbol: ProviderSymbol) -> Result<(), CompileError> {
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(symbol, 0)?;
        self.bytes
            .extend_from_slice(&[0x31, 0xc0, 0x48, 0x89, 0x07]);
        for offset in [8_u8, 16, 24] {
            self.bytes.extend_from_slice(&[0x48, 0x89, 0x47, offset]);
        }
        self.bytes.extend_from_slice(&[0x48, 0x83, 0x3f, 0]);
        self.jump_to_fatal_if(0x85)?;
        for offset in [8_u8, 16, 24] {
            self.bytes.extend_from_slice(&[0x48, 0x83, 0x7f, offset, 0]);
            self.jump_to_fatal_if(0x85)?;
        }
        Ok(())
    }

    fn emit_unaccounted_pci_region_dma_reference(
        &mut self,
        request: CompilerPciRegionDmaReferenceRequest,
    ) -> Result<(), CompileError> {
        let selector = 0x8000_0000
            | (u32::from(request.bus) << 16)
            | (u32::from(request.device) << 11)
            | (u32::from(request.function) << 8)
            | u32::from(request.access_capability_offset_bytes);
        let target_offset = request
            .region_offset_bytes
            .checked_add(request.register_offset_bytes)
            .ok_or_else(|| {
                CompileError::Tool("PCI DMA-reference target offset overflowed".into())
            })?;
        self.bytes.push(0xbf);
        self.bytes.extend_from_slice(&selector.to_le_bytes());
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&u32::from(request.bar).to_le_bytes());
        self.bytes.push(0xba);
        self.bytes.extend_from_slice(&target_offset.to_le_bytes());
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x0d]);
        self.rip_relative_symbol(ProviderSymbol::DmaVirtqueueControl, 0)?;
        self.bytes.extend_from_slice(&[0x41, 0xb8]);
        self.bytes
            .extend_from_slice(&(request.buffer_offset_bytes as u32).to_le_bytes());
        self.call_checked_bool(ProviderSymbol::PublishPciRegionDmaReference)
    }

    fn emit_unaccounted_pci_configuration_word_update(
        &mut self,
        request: CompilerPciConfigurationWordUpdateRequest,
    ) -> Result<(), CompileError> {
        let selector = 0x8000_0000
            | (u32::from(request.bus) << 16)
            | (u32::from(request.device) << 11)
            | (u32::from(request.function) << 8)
            | u32::from(request.register_offset_bytes & 0xfc);
        self.bytes.push(0xbf);
        self.bytes.extend_from_slice(&selector.to_le_bytes());
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&u32::from(request.mask).to_le_bytes());
        self.bytes.push(0xba);
        self.bytes
            .extend_from_slice(&u32::from(request.expected_masked_value).to_le_bytes());
        self.bytes.push(0xb9);
        self.bytes
            .extend_from_slice(&u32::from(request.desired_masked_value).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]);
        self.bytes
            .extend_from_slice(&u32::from(request.register_offset_bytes & 0x02).to_le_bytes());
        self.call_checked_bool(ProviderSymbol::UpdatePciConfigurationWord)
    }

    fn emit_unaccounted_dma_buffer_create(
        &mut self,
        request: CompilerDmaBufferRequest,
    ) -> Result<(), CompileError> {
        let packed_metadata = match request.role {
            CompilerDmaBufferRole::VirtqueueControl => 0x01_u32,
            CompilerDmaBufferRole::EntropyPayload => 0x12_u32,
        };
        self.bytes.push(0xbf);
        self.bytes
            .extend_from_slice(&(request.buffer_identity as u32).to_le_bytes());
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&(request.device_identity as u32).to_le_bytes());
        self.bytes.push(0xba);
        self.bytes
            .extend_from_slice(&(request.translation_domain_identity as u32).to_le_bytes());
        self.bytes.push(0xb9);
        self.bytes
            .extend_from_slice(&(request.byte_count as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]);
        self.bytes
            .extend_from_slice(&(request.alignment_bytes as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb9]);
        self.bytes.extend_from_slice(&packed_metadata.to_le_bytes());
        self.call_checked_bool(ProviderSymbol::CreateDmaBuffer)
    }

    fn emit_unaccounted_dma_buffer_bind_reference(
        &mut self,
        request: CompilerDmaBufferReferenceRequest,
    ) -> Result<(), CompileError> {
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::DmaVirtqueueControl, 0)?;
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&(request.destination_offset_bytes as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x15]);
        self.rip_relative_symbol(ProviderSymbol::DmaEntropyPayload, 0)?;
        self.bytes.push(0xb9);
        self.bytes
            .extend_from_slice(&(request.referenced_offset_bytes as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]);
        self.bytes
            .extend_from_slice(&u32::from(request.width_bytes).to_le_bytes());
        self.call_checked_bool(ProviderSymbol::BindDmaBufferReference)
    }

    fn emit_unaccounted_dma_buffer_integer_store(
        &mut self,
        request: CompilerDmaBufferIntegerStoreRequest,
    ) -> Result<(), CompileError> {
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::DmaVirtqueueControl, 0)?;
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&(request.offset_bytes as u32).to_le_bytes());
        self.bytes.push(0xba);
        self.bytes
            .extend_from_slice(&u32::from(request.width_bytes).to_le_bytes());
        self.bytes.extend_from_slice(&[0x48, 0xb9]);
        self.bytes.extend_from_slice(&request.value.to_le_bytes());
        self.call_checked_bool(ProviderSymbol::StoreDmaBufferInteger)
    }

    fn emit_unaccounted_dma_buffer_integer_load(
        &mut self,
        request: CompilerDmaBufferIntegerLoadRequest,
    ) -> Result<(), CompileError> {
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::DmaVirtqueueControl, 0)?;
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&(request.offset_bytes as u32).to_le_bytes());
        self.bytes.push(0xba);
        self.bytes
            .extend_from_slice(&u32::from(request.width_bytes).to_le_bytes());
        self.bytes.extend_from_slice(&[0x48, 0xb9]);
        self.bytes
            .extend_from_slice(&request.expected_value.to_le_bytes());
        self.call_checked_bool(ProviderSymbol::LoadDmaBufferInteger)
    }

    fn emit_unaccounted_dma_buffer_ownership(
        &mut self,
        request: CompilerDmaBufferPreparationRequest,
        provider: ProviderSymbol,
    ) -> Result<(), CompileError> {
        let symbol = match request.role {
            CompilerDmaBufferRole::VirtqueueControl => ProviderSymbol::DmaVirtqueueControl,
            CompilerDmaBufferRole::EntropyPayload => ProviderSymbol::DmaEntropyPayload,
        };
        let packed_metadata = match request.role {
            CompilerDmaBufferRole::VirtqueueControl => 0x01_u32,
            CompilerDmaBufferRole::EntropyPayload => 0x12_u32,
        };
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(symbol, 0)?;
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&(request.buffer_identity as u32).to_le_bytes());
        self.bytes.push(0xba);
        self.bytes
            .extend_from_slice(&(request.device_identity as u32).to_le_bytes());
        self.bytes.push(0xb9);
        self.bytes
            .extend_from_slice(&(request.translation_domain_identity as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]);
        self.bytes.extend_from_slice(&packed_metadata.to_le_bytes());
        self.call_checked_bool(provider)
    }

    fn emit_unaccounted_entropy_sample_capture(
        &mut self,
        request: CompilerEntropySampleCaptureRequest,
    ) -> Result<(), CompileError> {
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::DmaEntropyPayload, 0)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
        self.rip_relative_symbol(ProviderSymbol::RawEntropySample, 0)?;
        self.bytes.push(0xba);
        self.bytes
            .extend_from_slice(&(request.sample_identity as u32).to_le_bytes());
        self.bytes.push(0xb9);
        self.bytes
            .extend_from_slice(&(request.buffer_identity as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]);
        self.bytes
            .extend_from_slice(&(request.byte_count as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb9]);
        self.bytes
            .extend_from_slice(&(request.credited_entropy_bits as u32).to_le_bytes());
        self.call_checked_bool(ProviderSymbol::CaptureEntropySample)
    }

    fn emit_unaccounted_entropy_sample_quality(
        &mut self,
        request: CompilerEntropySampleQualityRequest,
    ) -> Result<(), CompileError> {
        if !request.is_well_formed()
            || request != CompilerEntropySampleQualityRequest::INITIAL_Q35_VIRTIO_RNG
        {
            return Err(CompileError::Tool(
                "x86 static-image entropy quality requires the exact target-independent Q35 trust transition".into(),
            ));
        }
        Ok(())
    }

    fn emit_unaccounted_entropy_pool_create(
        &mut self,
        request: CompilerEntropyPoolRequest,
    ) -> Result<(), CompileError> {
        if !request.is_well_formed() || request != CompilerEntropyPoolRequest::INITIAL_Q35 {
            return Err(CompileError::Tool(
                "x86 static-image entropy-pool creation requires the exact target-independent empty pool".into(),
            ));
        }
        Ok(())
    }

    fn emit_unaccounted_entropy_pool_mix(
        &mut self,
        request: CompilerEntropyPoolMixRequest,
    ) -> Result<(), CompileError> {
        if !request.is_well_formed() || request != CompilerEntropyPoolMixRequest::INITIAL_Q35 {
            return Err(CompileError::Tool(
                "x86 static-image entropy-pool mixing requires the exact first BLAKE2s contribution".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::RawEntropySample, 0)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
        self.rip_relative_symbol(ProviderSymbol::EntropyPoolState, 0)?;
        self.call_checked_bool(ProviderSymbol::MixEntropyPoolSample)
    }

    fn emit_unaccounted_entropy_pool_credit(
        &mut self,
        request: CompilerEntropyPoolCreditRequest,
    ) -> Result<(), CompileError> {
        if !request.is_well_formed() || request != CompilerEntropyPoolCreditRequest::INITIAL_Q35 {
            return Err(CompileError::Tool(
                "x86 static-image entropy-pool credit requires the exact target-independent first contribution".into(),
            ));
        }
        Ok(())
    }

    fn emit_unaccounted_entropy_generator_initialize(
        &mut self,
        request: CompilerEntropyGeneratorRequest,
    ) -> Result<(), CompileError> {
        if !request.is_well_formed() || request != CompilerEntropyGeneratorRequest::INITIAL_Q35 {
            return Err(CompileError::Tool(
                "x86 static-image entropy-generator initialization requires the exact first ChaCha20 generation".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::EntropyPoolState, 0)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
        self.rip_relative_symbol(ProviderSymbol::EntropyGeneratorKey, 0)?;
        self.call_checked_bool(ProviderSymbol::InitializeEntropyGenerator)
    }

    fn emit_unaccounted_entropy_generator_extract(
        &mut self,
        request: CompilerEntropyGeneratorExtractRequest,
    ) -> Result<(), CompileError> {
        if !request.is_well_formed()
            || request != CompilerEntropyGeneratorExtractRequest::INITIAL_Q35
        {
            return Err(CompileError::Tool(
                "x86 static-image entropy-generator extraction requires the exact first ChaCha20 output".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::EntropyGeneratorKey, 0)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
        self.rip_relative_symbol(ProviderSymbol::EntropyOutput, 0)?;
        self.call_checked_bool(ProviderSymbol::ExtractEntropyGenerator)
    }

    fn emit_unaccounted_pci_region_register_access(
        &mut self,
        request: CompilerPciRegionRegisterRequest,
    ) -> Result<(), CompileError> {
        let selector = 0x8000_0000
            | (u32::from(request.bus) << 16)
            | (u32::from(request.device) << 11)
            | (u32::from(request.function) << 8)
            | u32::from(request.access_capability_offset_bytes);
        let target_offset = request
            .region_offset_bytes
            .checked_add(request.register_offset_bytes)
            .ok_or_else(|| CompileError::Tool("PCI region target offset overflowed".into()))?;
        self.bytes.push(0xbf);
        self.bytes.extend_from_slice(&selector.to_le_bytes());
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&u32::from(request.bar).to_le_bytes());
        self.bytes.push(0xba);
        self.bytes.extend_from_slice(&target_offset.to_le_bytes());
        self.bytes.push(0xb9);
        self.bytes
            .extend_from_slice(&u32::from(request.width_bytes).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]);
        self.bytes.extend_from_slice(&request.value.to_le_bytes());
        let symbol = match request.direction {
            CompilerPciRegionRegisterDirection::ReadEquals => ProviderSymbol::ReadPciRegionRegister,
            CompilerPciRegionRegisterDirection::ReadEventuallyEquals => {
                ProviderSymbol::ReadPciRegionRegisterEventually
            }
            CompilerPciRegionRegisterDirection::Write => ProviderSymbol::WritePciRegionRegister,
        };
        self.call_checked_bool(symbol)
    }

    fn revoke_static_image_entropy_acquisition(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_entropy_acquisition(false)
    }

    fn remove_static_image_stack_program_headers(&mut self) -> Result<(), CompileError> {
        self.transition_static_image_stack_program_headers(false)
    }

    fn remove_static_image_stack_entry_fact(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::StackEntryFactAdded
            || self.physical_frame_count != Some(1)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image AT_ENTRY removal requires one entry-fact-qualified stack entry under the inactive qualified hierarchy"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xe9]);
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]);
        self.bytes.extend_from_slice(&[0x49, 0x89, 0xd9]);
        self.call_checked_bool(ProviderSymbol::RemoveStaticImageStackEntryFact)?;
        self.static_image = RootStaticImageState::StackPageSizeAdded;
        Ok(())
    }

    fn remove_static_image_stack_pagesz(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::StackPageSizeAdded
            || self.physical_frame_count != Some(1)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image AT_PAGESZ removal requires one extended stack entry under the inactive qualified hierarchy"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xe9]);
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]);
        self.bytes.extend_from_slice(&[0x49, 0x89, 0xd9]);
        self.call_checked_bool(ProviderSymbol::RemoveStaticImageStackPageSize)?;
        self.static_image = RootStaticImageState::StackEntryInitialized;
        Ok(())
    }

    fn revoke_static_image_stack_leaf(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::StackLeafPublished
            || self.physical_frame_count != Some(1)
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image stack-leaf revocation requires one published leaf under the inactive qualified hierarchy"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xe9]);
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]);
        self.bytes.extend_from_slice(&[0x49, 0x89, 0xd9]);
        self.call_checked_bool(ProviderSymbol::RevokeStaticImageStackLeaf)?;
        self.static_image = RootStaticImageState::StackBackingZeroed;
        Ok(())
    }

    fn revoke_static_image_entry(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.static_image != RootStaticImageState::EntryAuthorized
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
            || self.translation != RootTranslationState::RemovedActive
        {
            return Err(CompileError::Tool(
                "x86 static-image entry revocation requires one authorized construction and the qualified bootstrap root"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf7]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
        self.bytes
            .extend_from_slice(&[0xba, 0xf7, 0x01, 0x00, 0x00]);
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xd9]);
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xf8]);
        self.call_checked_bool(ProviderSymbol::RevokeStaticImageEntry)?;
        self.static_image = RootStaticImageState::EntryRestored;
        Ok(())
    }

    fn reclaim_static_image_hierarchy(&mut self) -> Result<(), CompileError> {
        if self.static_image != RootStaticImageState::Detached
            || self.physical_frame_count.is_some()
            || self.retained_static_elf_payload
        {
            return Err(CompileError::Tool(
                "x86 static-image hierarchy reclaim requires one detached construction".into(),
            ));
        }
        self.physical_frame_count = Some(5);
        self.static_image = RootStaticImageState::HierarchyReturned;
        Ok(())
    }

    fn reclaim_static_image_payload(&mut self) -> Result<(), CompileError> {
        if self.static_image != RootStaticImageState::MetadataReturned
            || self.physical_frame_count != Some(503)
            || self.retained_static_elf_payload
        {
            return Err(CompileError::Tool(
                "x86 static-image payload reclaim requires revoked leaves and released metadata"
                    .into(),
            ));
        }
        self.static_image = RootStaticImageState::PayloadReturned;
        Ok(())
    }

    fn begin_translation(
        &mut self,
        request: CompilerTranslationUpdateRequest,
    ) -> Result<(), CompileError> {
        if !self.frame_allocator_created
            || self.physical_frame_count.is_some()
            || self.kernel_mapping_live
            || self.translation != RootTranslationState::BootstrapActive
            || request != CompilerTranslationUpdateRequest::initial_bootstrap_equivalent()
        {
            return Err(CompileError::Tool(
                "x86 root lowering requires the live allocator, no borrowed frames, and the sealed translation request".into(),
            ));
        }
        self.call(ProviderSymbol::BeginTranslation);
        self.bytes.extend_from_slice(&[0x48, 0x85, 0xc0]);
        self.jump_to_fatal_if(0x84)?;
        self.bytes.extend_from_slice(&[0x49, 0x89, 0xc4]); // retain root in r12
        self.translation = RootTranslationState::Update;
        Ok(())
    }

    fn commit_translation(&mut self) -> Result<(), CompileError> {
        if self.translation != RootTranslationState::Update {
            return Err(CompileError::Tool(
                "x86 root lowering encountered translation commit without an update".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe7]); // rdi = r12
        self.call_checked_bool(ProviderSymbol::CommitTranslation)?;
        self.translation = RootTranslationState::InactiveSpace;
        Ok(())
    }

    fn activate_translation(&mut self) -> Result<(), CompileError> {
        if self.translation != RootTranslationState::InactiveSpace {
            return Err(CompileError::Tool(
                "x86 root lowering encountered translation activation without an inactive space"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe7]); // rdi = r12
        self.call_checked_bool(ProviderSymbol::ActivateTranslation)?;
        self.bytes.extend_from_slice(&[0x4d, 0x89, 0xe7]); // retain active root in r15
        self.bytes.extend_from_slice(&[0x4d, 0x31, 0xe4]); // consume inactive-space token
        self.translation = RootTranslationState::ReplacementActive;
        Ok(())
    }

    fn begin_translation_edit(
        &mut self,
        kind: CompilerTranslationEditKind,
    ) -> Result<(), CompileError> {
        match kind {
            CompilerTranslationEditKind::Map
                if self.translation == RootTranslationState::ReplacementActive
                    && self.physical_frame_count == Some(1)
                    && !self.kernel_mapping_live =>
            {
                self.bytes.extend_from_slice(&[0x48, 0x89, 0xdf]); // payload in rdi
                self.bytes.extend_from_slice(&[0x4c, 0x89, 0xee]); // boot params in rsi
                self.bytes.extend_from_slice(&[0x31, 0xd2]); // map kind
                self.call(ProviderSymbol::BeginTranslationEdit);
                self.bytes.extend_from_slice(&[0x48, 0x85, 0xc0]);
                self.jump_to_fatal_if(0x84)?;
                self.bytes.extend_from_slice(&[0x49, 0x89, 0xc4]); // metadata in r12
                self.translation = RootTranslationState::MapEdit;
            }
            CompilerTranslationEditKind::Unmap
                if self.translation == RootTranslationState::MappedActive
                    && self.kernel_mapping_live
                    && self.physical_frame_count.is_none() =>
            {
                self.bytes.extend_from_slice(&[0x4c, 0x89, 0xff]); // active root
                self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf6]); // mapping
                self.bytes
                    .extend_from_slice(&[0xba, 0x01, 0x00, 0x00, 0x00]);
                self.call_checked_bool(ProviderSymbol::BeginTranslationEdit)?;
                self.translation = RootTranslationState::UnmapEdit;
            }
            _ => {
                return Err(CompileError::Tool(
                    "x86 root lowering encountered translation edit begin without matching active authority".into(),
                ));
            }
        }
        Ok(())
    }

    fn map_translation_frames(
        &mut self,
        request: CompilerTranslationMappingRequest,
    ) -> Result<(), CompileError> {
        if self.translation != RootTranslationState::MapEdit
            || self.physical_frame_count != Some(1)
            || self.kernel_mapping_live
            || request != CompilerTranslationMappingRequest::initial_read_write()
        {
            return Err(CompileError::Tool(
                "x86 root lowering requires a map edit, live frame, and sealed active mapping policy".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe7]); // metadata PD
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xde]); // payload frame
        self.call_checked_bool(ProviderSymbol::MapTranslationFrames)?;
        self.physical_frame_count = None;
        self.translation = RootTranslationState::MapStaged;
        Ok(())
    }

    fn unmap_translation_mapping(&mut self) -> Result<(), CompileError> {
        if self.translation != RootTranslationState::UnmapEdit
            || !self.kernel_mapping_live
            || self.physical_frame_count.is_some()
        {
            return Err(CompileError::Tool(
                "x86 root lowering requires an unmap edit and its live mapping".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xff]);
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf6]);
        self.call_checked_bool(ProviderSymbol::UnmapTranslationMapping)?;
        self.kernel_mapping_live = false;
        self.translation = RootTranslationState::UnmapStaged;
        Ok(())
    }

    fn commit_translation_edit(
        &mut self,
        kind: CompilerTranslationEditKind,
    ) -> Result<(), CompileError> {
        match kind {
            CompilerTranslationEditKind::Map
                if self.translation == RootTranslationState::MapStaged =>
            {
                self.bytes.extend_from_slice(&[0x4c, 0x89, 0xff]);
                self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe6]);
                self.bytes.extend_from_slice(&[0x31, 0xd2]);
                self.call(ProviderSymbol::CommitTranslationEdit);
                self.bytes.extend_from_slice(&[0x48, 0x85, 0xc0]);
                self.jump_to_fatal_if(0x84)?;
                self.bytes.extend_from_slice(&[0x49, 0x89, 0xc6]); // opaque mapping
                self.bytes.extend_from_slice(&[0x4d, 0x31, 0xe4]);
                self.kernel_mapping_live = true;
                self.translation = RootTranslationState::MappedActive;
            }
            CompilerTranslationEditKind::Unmap
                if self.translation == RootTranslationState::UnmapStaged =>
            {
                self.bytes.extend_from_slice(&[0x4c, 0x89, 0xff]);
                self.bytes.extend_from_slice(&[0x4c, 0x89, 0xf6]);
                self.bytes
                    .extend_from_slice(&[0xba, 0x01, 0x00, 0x00, 0x00]);
                self.call_checked_bool(ProviderSymbol::CommitTranslationEdit)?;
                self.bytes.extend_from_slice(&[0x4d, 0x31, 0xf6]);
                self.physical_frame_count = Some(1);
                self.translation = RootTranslationState::RemovedActive;
            }
            _ => {
                return Err(CompileError::Tool(
                    "x86 root lowering encountered translation edit commit without a matching candidate".into(),
                ));
            }
        }
        Ok(())
    }

    fn enter_critical(&mut self, domain: CompilerCriticalDomain) -> Result<(), CompileError> {
        if domain != CompilerCriticalDomain::LocalMaskableInterrupts
            || self.critical == RootCriticalState::Live
            || self.physical_frame_count.is_some()
            || self.kernel_mapping_live
            || self.translation != RootTranslationState::RemovedActive
            || self.static_image != RootStaticImageState::PayloadReturned
        {
            return Err(CompileError::Tool(
                "x86 root lowering requires the restored active context and one non-nested local-interrupt critical scope".into(),
            ));
        }
        self.call(ProviderSymbol::EnterCritical);
        self.bytes.extend_from_slice(&[0x48, 0x85, 0xc0]); // validate opaque token
        self.jump_to_fatal_if(0x84)?;
        self.bytes.extend_from_slice(&[0x49, 0x89, 0xc4]); // retain token in r12
        self.critical = RootCriticalState::Live;
        Ok(())
    }

    fn restore_critical(&mut self, domain: CompilerCriticalDomain) -> Result<(), CompileError> {
        if domain != CompilerCriticalDomain::LocalMaskableInterrupts
            || self.critical != RootCriticalState::Live
        {
            return Err(CompileError::Tool(
                "x86 root lowering encountered critical restoration without matching affine authority".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xe7]); // opaque token in rdi
        self.call_checked_bool(ProviderSymbol::RestoreCritical)?;
        self.bytes.extend_from_slice(&[0x4d, 0x31, 0xe4]); // consume token
        self.critical = RootCriticalState::Restored;
        Ok(())
    }

    fn complete(&self) -> Result<(), CompileError> {
        if self.atomic_word_offset.is_some() {
            return Err(CompileError::Tool(
                "x86 root lowering ended with a live atomic word".into(),
            ));
        }
        if self.bootstrap_region_offset.is_some() {
            return Err(CompileError::Tool(
                "x86 root lowering ended with a live bootstrap region".into(),
            ));
        }
        if self.physical_frame_count.is_some() {
            return Err(CompileError::Tool(
                "x86 root lowering ended with a live physical-frame extent".into(),
            ));
        }
        if self.kernel_mapping_live {
            return Err(CompileError::Tool(
                "x86 root lowering ended with a live kernel mapping".into(),
            ));
        }
        if self.critical == RootCriticalState::Live {
            return Err(CompileError::Tool(
                "x86 root lowering ended with live critical restoration authority".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && self.pci_configuration_observations
                != CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY.len()
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without its complete sealed PCI capability-discovery sequence".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && self.pci_region_register_accesses
                != CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION.len()
                    + CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY.len()
                    + CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
                        .len()
                    + CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION
                        .len()
                    + CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL
                        .len()
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without its complete sealed virtio-rng feature-negotiation, queue-discovery, queue-activation, queue-notification, and completion-signal sequences".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification && !self.pci_bus_master_authorized
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without its sealed PCI bus-master authorization".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && self.dma_buffers_created != CompilerDmaBufferRequest::INITIAL_Q35_VIRTIO_RNG.len()
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without both sealed CPU-owned virtio-rng DMA buffers"
                    .into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && self.dma_construction_operations
                != 1 + CompilerDmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING.len()
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without its complete CPU-owned virtio-rng split ring"
                    .into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && self.dma_buffers_prepared
                != CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG.len()
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without its complete payload-then-control DMA preparation sequence"
                    .into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && self.dma_references_published
                != CompilerPciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
                    .len()
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without all three sealed virtio-rng queue-address publications"
                    .into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && (!self.dma_control_completed
                || self.dma_used_ring_reads
                    != CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING.len())
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without CPU return and exact validation of the sealed virtio-rng used ring".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification && !self.dma_payload_completed {
            return Err(CompileError::Tool(
                "x86 root lowering ended without CPU return of the completed virtio-rng payload"
                    .into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification && !self.entropy_sample_captured {
            return Err(CompileError::Tool(
                "x86 root lowering ended without the sealed uncredited virtio-rng sample capture"
                    .into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification && !self.entropy_sample_assessed {
            return Err(CompileError::Tool(
                "x86 root lowering ended without the sealed trusted-host virtio-rng sample assessment"
                    .into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification && !self.entropy_pool_created {
            return Err(CompileError::Tool(
                "x86 root lowering ended without the sealed empty entropy pool".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification && !self.entropy_pool_mixed {
            return Err(CompileError::Tool(
                "x86 root lowering ended without the sealed first entropy-pool mix".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification && !self.entropy_pool_credited {
            return Err(CompileError::Tool(
                "x86 root lowering ended without the sealed first entropy-credit transfer".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification && !self.entropy_generator_ready {
            return Err(CompileError::Tool(
                "x86 root lowering ended without the sealed generation-one entropy generator"
                    .into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && !self.entropy_generator_extracted
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without the sealed first entropy-generator extraction"
                    .into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && !self.initial_auxv_random_delivered
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without affine initial AT_RANDOM delivery".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && (!self.initial_getrandom_extracted
                || self.initial_user_random_reserved
                || !self.user_random_service_bound
                || !self.user_random_small_requests_enabled
                || !self.user_random_large_requests_enabled)
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without the sealed reusable getrandom service".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && self.local_notification != RootLocalNotificationState::Completed
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without completing its local-notification lifecycle"
                    .into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification && self.monotonic_observations != 2
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without exactly two monotonic-clock observations".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && self.deadline != RootDeadlineState::Completed
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without completing its deadline lifecycle".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && self.kernel_context != RootKernelContextState::QueueConsumed
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended without reclaiming all contexts and consuming its runnable queue"
                    .into(),
            ));
        }
        if self.context_stack_offsets != [None, None, None] {
            return Err(CompileError::Tool(
                "x86 root lowering ended with a live cooperative context stack".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && self.completion != RootCompletionState::Consumed
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended with a live one-shot completion".into(),
            ));
        }
        if self.role == RootRole::BootstrapWithLocalNotification
            && self.user_execution != RootUserExecutionState::Consumed
        {
            return Err(CompileError::Tool(
                "x86 root lowering ended with a live initial user execution".into(),
            ));
        }
        if matches!(
            self.translation,
            RootTranslationState::Update
                | RootTranslationState::InactiveSpace
                | RootTranslationState::MapEdit
                | RootTranslationState::MapStaged
                | RootTranslationState::UnmapEdit
                | RootTranslationState::UnmapStaged
        ) {
            return Err(CompileError::Tool(
                "x86 root lowering ended with an incomplete translation lifecycle".into(),
            ));
        }
        Ok(())
    }

    fn call_checked_bool(&mut self, target: ProviderSymbol) -> Result<(), CompileError> {
        self.call(target);
        self.bytes.extend_from_slice(&[0x84, 0xc0]);
        self.jump_to_fatal_if(0x84)
    }

    fn jump_to_fatal_if(&mut self, condition: u8) -> Result<(), CompileError> {
        self.bytes.extend_from_slice(&[0x0f, condition]);
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

    fn allocate_bootstrap_region(
        &mut self,
        request: CompilerBootstrapStorageRequest,
    ) -> Result<(), CompileError> {
        if self.bootstrap_region_offset.is_some()
            || request.placement != CompilerBootstrapStoragePlacement::BootstrapReclaimable
            || !matches!(
                (request.byte_count, request.alignment_bytes),
                (64, 8) | (16_384, 16)
            )
        {
            return Err(CompileError::Tool(
                "x86 root lowering encountered an overlapping or unsealed bootstrap region".into(),
            ));
        }
        let mask = request.alignment_bytes - 1;
        let mut offset = self
            .bootstrap_next_offset
            .checked_add(mask)
            .ok_or_else(|| CompileError::Tool("bootstrap placement overflows".into()))?
            & !mask;
        if request.byte_count == 16_384 {
            offset = offset.max(4096); // provider-private event state occupies the low page
        }
        self.bootstrap_next_offset = offset
            .checked_add(request.byte_count)
            .ok_or_else(|| CompileError::Tool("bootstrap placement overflows".into()))?;
        self.bootstrap_region_offset = Some(offset);
        Ok(())
    }

    fn store_bootstrap_byte(&mut self, offset: u64, value: u8) -> Result<(), CompileError> {
        if self.atomic_word_offset.is_some() {
            return Err(CompileError::Tool(
                "x86 root lowering rejected plain access while an atomic word is live".into(),
            ));
        }
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
        if self.atomic_word_offset.is_some() {
            return Err(CompileError::Tool(
                "x86 root lowering rejected plain access while an atomic word is live".into(),
            ));
        }
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
        if self.atomic_word_offset.is_some() {
            return Err(CompileError::Tool(
                "x86 root lowering rejected region release while an atomic word is live".into(),
            ));
        }
        if self.bootstrap_region_offset.take().is_none() {
            return Err(CompileError::Tool(
                "x86 root lowering encountered release without a live bootstrap region".into(),
            ));
        }
        Ok(())
    }

    fn create_atomic_word(
        &mut self,
        request: CompilerAtomicWordRequest,
    ) -> Result<(), CompileError> {
        if request != CompilerAtomicWordRequest::initial() || self.atomic_word_offset.is_some() {
            return Err(CompileError::Tool(
                "x86 root lowering requires one live atomic word with the sealed request".into(),
            ));
        }
        let storage_offset = self.bootstrap_storage_offset(request.offset_bytes)?;
        self.load_bootstrap_storage_address(storage_offset)?;
        self.bytes.push(0xbe); // mov esi, imm32
        let initial_value = u32::try_from(request.initial_value)
            .map_err(|_| CompileError::Tool("atomic initial value exceeds x86 imm32".into()))?;
        self.bytes.extend_from_slice(&initial_value.to_le_bytes());
        self.call_checked_bool(ProviderSymbol::AtomicWordCreate)?;
        self.atomic_word_offset = Some(request.offset_bytes);
        Ok(())
    }

    fn compare_exchange_atomic_word(
        &mut self,
        expected: u64,
        desired: u64,
        success_order: CompilerAtomicOrder,
        failure_order: CompilerAtomicOrder,
    ) -> Result<(), CompileError> {
        if self.atomic_word_offset != Some(8)
            || (expected, desired, success_order, failure_order)
                != (
                    41,
                    42,
                    CompilerAtomicOrder::AcquireRelease,
                    CompilerAtomicOrder::Acquire,
                )
        {
            return Err(CompileError::Tool(
                "x86 root lowering requires the sealed atomic compare/exchange".into(),
            ));
        }
        self.load_bootstrap_storage_address(8)?;
        self.bytes.push(0xbe); // mov esi, imm32
        let expected = u32::try_from(expected)
            .map_err(|_| CompileError::Tool("atomic expected value exceeds x86 imm32".into()))?;
        self.bytes.extend_from_slice(&expected.to_le_bytes());
        self.bytes.push(0xba); // mov edx, imm32
        let desired = u32::try_from(desired)
            .map_err(|_| CompileError::Tool("atomic desired value exceeds x86 imm32".into()))?;
        self.bytes.extend_from_slice(&desired.to_le_bytes());
        self.call_checked_bool(ProviderSymbol::AtomicWordCompareExchange)
    }

    fn load_atomic_word_equals(
        &mut self,
        order: CompilerAtomicOrder,
        expected: u64,
    ) -> Result<(), CompileError> {
        if self.atomic_word_offset != Some(8)
            || order != CompilerAtomicOrder::Acquire
            || expected != 42
        {
            return Err(CompileError::Tool(
                "x86 root lowering requires the sealed acquire atomic load".into(),
            ));
        }
        self.load_bootstrap_storage_address(8)?;
        self.call(ProviderSymbol::AtomicWordLoad);
        self.bytes.extend_from_slice(&[0x48, 0x83, 0xf8, 0x2a]); // cmp rax, 42
        self.jump_to_fatal_if(0x85)
    }

    fn end_atomic_word(&mut self) -> Result<(), CompileError> {
        if self.atomic_word_offset.take() != Some(8) {
            return Err(CompileError::Tool(
                "x86 root lowering encountered atomic end without a live location".into(),
            ));
        }
        Ok(())
    }

    fn send_local_notification(&mut self) -> Result<(), CompileError> {
        if self.local_notification != RootLocalNotificationState::Idle
            || self.bootstrap_region_offset.is_some()
            || self.atomic_word_offset.is_some()
            || self.critical != RootCriticalState::Restored
        {
            return Err(CompileError::Tool(
                "x86 root lowering requires a restored context and no live bootstrap region before local-notification send".into(),
            ));
        }
        self.load_bootstrap_storage_address(16)?;
        self.call_checked_bool(ProviderSymbol::SendLocalNotification)?;
        self.local_notification = RootLocalNotificationState::Pending;
        Ok(())
    }

    fn wait_local_notification(&mut self) -> Result<(), CompileError> {
        if self.local_notification != RootLocalNotificationState::Pending {
            return Err(CompileError::Tool(
                "x86 root lowering encountered local-notification wait without pending authority"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(16)?;
        self.call_checked_bool(ProviderSymbol::WaitLocalNotification)?;
        self.local_notification = RootLocalNotificationState::Completed;
        Ok(())
    }

    fn complete_local_notification(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::LocalNotificationHandler {
            return Err(CompileError::Tool(
                "x86 local-notification completion is admitted only in its external entry".into(),
            ));
        }
        self.load_bootstrap_storage_address(16)?;
        self.call(ProviderSymbol::CompleteLocalNotification);
        Ok(())
    }

    fn observe_monotonic_clock(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.local_notification != RootLocalNotificationState::Completed
            || self.monotonic_observations >= 2
        {
            return Err(CompileError::Tool(
                "x86 monotonic-clock observation requires the completed local-notification lifecycle and its sealed two-read sequence".into(),
            ));
        }
        self.load_bootstrap_storage_address(24)?;
        self.call(ProviderSymbol::ObserveMonotonicClock);
        self.bytes.extend_from_slice(&[0x85, 0xc9]); // test provider status in ecx
        self.jump_to_fatal_if(0x84)?;
        self.monotonic_observations += 1;
        Ok(())
    }

    fn observe_pci_configuration_dword(
        &mut self,
        request: CompilerPciConfigurationDwordRequest,
    ) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || !self.frame_allocator_created
            || self.critical != RootCriticalState::Restored
            || CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                .get(self.pci_configuration_observations)
                != Some(&request)
        {
            return Err(CompileError::Tool(
                "x86 PCI configuration observation requires the restored bootstrap context and exact sealed Q35 virtio-rng capability-discovery sequence".into(),
            ));
        }
        let selector = 0x8000_0000
            | (u32::from(request.bus) << 16)
            | (u32::from(request.device) << 11)
            | (u32::from(request.function) << 8)
            | u32::from(request.register_offset_bytes);
        self.bytes.push(0xbf); // selector in edi
        self.bytes.extend_from_slice(&selector.to_le_bytes());
        self.bytes.push(0xbe); // expected value in esi
        self.bytes
            .extend_from_slice(&request.expected_value.to_le_bytes());
        self.call_checked_bool(ProviderSymbol::ObservePciConfigurationDword)?;
        self.pci_configuration_observations += 1;
        Ok(())
    }

    fn update_pci_configuration_word(
        &mut self,
        request: CompilerPciConfigurationWordUpdateRequest,
    ) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.pci_region_register_accesses
                != CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION.len()
                    + CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY.len()
            || self.pci_bus_master_authorized
            || !request.is_well_formed()
            || request
                != CompilerPciConfigurationWordUpdateRequest::INITIAL_Q35_VIRTIO_RNG_BUS_MASTER
        {
            return Err(CompileError::Tool(
                "x86 PCI configuration update requires complete queue discovery and the exact sealed bus-master command transition".into(),
            ));
        }
        let selector = 0x8000_0000
            | (u32::from(request.bus) << 16)
            | (u32::from(request.device) << 11)
            | (u32::from(request.function) << 8)
            | u32::from(request.register_offset_bytes & 0xfc);
        self.bytes.push(0xbf); // aligned configuration selector in edi
        self.bytes.extend_from_slice(&selector.to_le_bytes());
        self.bytes.push(0xbe); // mask in esi
        self.bytes
            .extend_from_slice(&u32::from(request.mask).to_le_bytes());
        self.bytes.push(0xba); // expected masked value in edx
        self.bytes
            .extend_from_slice(&u32::from(request.expected_masked_value).to_le_bytes());
        self.bytes.push(0xb9); // desired masked value in ecx
        self.bytes
            .extend_from_slice(&u32::from(request.desired_masked_value).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]); // data-port offset in r8d
        self.bytes
            .extend_from_slice(&u32::from(request.register_offset_bytes & 0x02).to_le_bytes());
        self.call_checked_bool(ProviderSymbol::UpdatePciConfigurationWord)?;
        self.pci_bus_master_authorized = true;
        Ok(())
    }

    fn create_dma_buffer(&mut self, request: CompilerDmaBufferRequest) -> Result<(), CompileError> {
        let expected =
            CompilerDmaBufferRequest::INITIAL_Q35_VIRTIO_RNG.get(self.dma_buffers_created);
        if self.role != RootRole::BootstrapWithLocalNotification
            || !self.pci_bus_master_authorized
            || !request.is_well_formed()
            || expected != Some(&request)
        {
            return Err(CompileError::Tool(
                "x86 DMA-buffer creation requires bus-master authorization and the exact sealed virtio-rng buffer sequence".into(),
            ));
        }
        let packed_metadata = match request.role {
            CompilerDmaBufferRole::VirtqueueControl => 0x01_u32,
            CompilerDmaBufferRole::EntropyPayload => 0x12_u32,
        };
        self.bytes.push(0xbf); // buffer identity in edi
        self.bytes
            .extend_from_slice(&(request.buffer_identity as u32).to_le_bytes());
        self.bytes.push(0xbe); // device identity in esi
        self.bytes
            .extend_from_slice(&(request.device_identity as u32).to_le_bytes());
        self.bytes.push(0xba); // translation-domain identity in edx
        self.bytes
            .extend_from_slice(&(request.translation_domain_identity as u32).to_le_bytes());
        self.bytes.push(0xb9); // byte count in ecx
        self.bytes
            .extend_from_slice(&(request.byte_count as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]); // alignment in r8d
        self.bytes
            .extend_from_slice(&(request.alignment_bytes as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb9]); // packed checked metadata in r9d
        self.bytes.extend_from_slice(&packed_metadata.to_le_bytes());
        self.call_checked_bool(ProviderSymbol::CreateDmaBuffer)?;
        self.dma_buffers_created += 1;
        Ok(())
    }

    fn bind_dma_buffer_reference(
        &mut self,
        request: CompilerDmaBufferReferenceRequest,
    ) -> Result<(), CompileError> {
        if self.dma_buffers_created != CompilerDmaBufferRequest::INITIAL_Q35_VIRTIO_RNG.len()
            || self.dma_construction_operations != 0
            || !request.is_well_formed()
            || request != CompilerDmaBufferReferenceRequest::INITIAL_Q35_VIRTIO_RNG_DESCRIPTOR
        {
            return Err(CompileError::Tool(
                "x86 DMA reference binding requires the exact sealed virtio-rng descriptor reference".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::DmaVirtqueueControl, 0)?;
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&(request.destination_offset_bytes as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x15]);
        self.rip_relative_symbol(ProviderSymbol::DmaEntropyPayload, 0)?;
        self.bytes.push(0xb9);
        self.bytes
            .extend_from_slice(&(request.referenced_offset_bytes as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]);
        self.bytes
            .extend_from_slice(&u32::from(request.width_bytes).to_le_bytes());
        self.call_checked_bool(ProviderSymbol::BindDmaBufferReference)?;
        self.dma_construction_operations = 1;
        Ok(())
    }

    fn store_dma_buffer_integer(
        &mut self,
        request: CompilerDmaBufferIntegerStoreRequest,
    ) -> Result<(), CompileError> {
        let expected = self
            .dma_construction_operations
            .checked_sub(1)
            .and_then(|index| {
                CompilerDmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING.get(index)
            });
        if expected != Some(&request) || !request.is_well_formed() {
            return Err(CompileError::Tool(
                "x86 DMA integer store requires the exact sealed virtio-rng split-ring sequence"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::DmaVirtqueueControl, 0)?;
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&(request.offset_bytes as u32).to_le_bytes());
        self.bytes.push(0xba);
        self.bytes
            .extend_from_slice(&u32::from(request.width_bytes).to_le_bytes());
        self.bytes.extend_from_slice(&[0x48, 0xb9]);
        self.bytes.extend_from_slice(&request.value.to_le_bytes());
        self.call_checked_bool(ProviderSymbol::StoreDmaBufferInteger)?;
        self.dma_construction_operations += 1;
        Ok(())
    }

    fn prepare_dma_buffer_for_device(
        &mut self,
        request: CompilerDmaBufferPreparationRequest,
    ) -> Result<(), CompileError> {
        let expected = CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG
            .get(self.dma_buffers_prepared);
        if self.dma_construction_operations
            != 1 + CompilerDmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING.len()
            || expected != Some(&request)
            || !request.is_well_formed()
        {
            return Err(CompileError::Tool(
                "x86 DMA device preparation requires complete split-ring construction and the exact sealed payload-then-control sequence"
                    .into(),
            ));
        }
        let symbol = match request.role {
            CompilerDmaBufferRole::VirtqueueControl => ProviderSymbol::DmaVirtqueueControl,
            CompilerDmaBufferRole::EntropyPayload => ProviderSymbol::DmaEntropyPayload,
        };
        let packed_metadata = match request.role {
            CompilerDmaBufferRole::VirtqueueControl => 0x01_u32,
            CompilerDmaBufferRole::EntropyPayload => 0x12_u32,
        };
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(symbol, 0)?;
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&(request.buffer_identity as u32).to_le_bytes());
        self.bytes.push(0xba);
        self.bytes
            .extend_from_slice(&(request.device_identity as u32).to_le_bytes());
        self.bytes.push(0xb9);
        self.bytes
            .extend_from_slice(&(request.translation_domain_identity as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]);
        self.bytes.extend_from_slice(&packed_metadata.to_le_bytes());
        self.call_checked_bool(ProviderSymbol::PrepareDmaBufferForDevice)?;
        self.dma_buffers_prepared += 1;
        Ok(())
    }

    fn complete_dma_buffer_for_cpu(
        &mut self,
        request: CompilerDmaBufferPreparationRequest,
    ) -> Result<(), CompileError> {
        let required_accesses = CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
            .len()
            + CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY.len()
            + CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION.len()
            + CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION.len()
            + CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL.len();
        let completes_control = !self.dma_control_completed
            && request == CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[1]
            && self.dma_used_ring_reads == 0;
        let completes_payload = self.dma_control_completed
            && !self.dma_payload_completed
            && self.dma_used_ring_reads
                == CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING.len()
            && request == CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0];
        if self.pci_region_register_accesses != required_accesses
            || !request.is_well_formed()
            || (!completes_control && !completes_payload)
        {
            return Err(CompileError::Tool(
                "x86 DMA completion requires the exact sealed control-then-payload ownership sequence around used-ring validation".into(),
            ));
        }
        let (symbol, packed_metadata) = match request.role {
            CompilerDmaBufferRole::VirtqueueControl => {
                (ProviderSymbol::DmaVirtqueueControl, 0x01_u32)
            }
            CompilerDmaBufferRole::EntropyPayload => (ProviderSymbol::DmaEntropyPayload, 0x12_u32),
        };
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(symbol, 0)?;
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&(request.buffer_identity as u32).to_le_bytes());
        self.bytes.push(0xba);
        self.bytes
            .extend_from_slice(&(request.device_identity as u32).to_le_bytes());
        self.bytes.push(0xb9);
        self.bytes
            .extend_from_slice(&(request.translation_domain_identity as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]);
        self.bytes.extend_from_slice(&packed_metadata.to_le_bytes());
        self.call_checked_bool(ProviderSymbol::CompleteDmaBufferForCpu)?;
        if completes_control {
            self.dma_control_completed = true;
        } else {
            self.dma_payload_completed = true;
        }
        Ok(())
    }

    fn load_dma_buffer_integer(
        &mut self,
        request: CompilerDmaBufferIntegerLoadRequest,
    ) -> Result<(), CompileError> {
        let expected = CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING
            .get(self.dma_used_ring_reads);
        if !self.dma_control_completed || expected != Some(&request) || !request.is_well_formed() {
            return Err(CompileError::Tool(
                "x86 DMA integer read requires the exact sealed virtio-rng used-ring sequence"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::DmaVirtqueueControl, 0)?;
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&(request.offset_bytes as u32).to_le_bytes());
        self.bytes.push(0xba);
        self.bytes
            .extend_from_slice(&u32::from(request.width_bytes).to_le_bytes());
        self.bytes.extend_from_slice(&[0x48, 0xb9]);
        self.bytes
            .extend_from_slice(&request.expected_value.to_le_bytes());
        self.call_checked_bool(ProviderSymbol::LoadDmaBufferInteger)?;
        self.dma_used_ring_reads += 1;
        Ok(())
    }

    fn capture_entropy_sample(
        &mut self,
        request: CompilerEntropySampleCaptureRequest,
    ) -> Result<(), CompileError> {
        if !self.dma_payload_completed
            || self.entropy_sample_captured
            || !request.is_well_formed()
            || request != CompilerEntropySampleCaptureRequest::INITIAL_Q35_VIRTIO_RNG
        {
            return Err(CompileError::Tool(
                "x86 entropy-sample capture requires the exact sealed uncredited virtio-rng payload range after CPU completion".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::DmaEntropyPayload, 0)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
        self.rip_relative_symbol(ProviderSymbol::RawEntropySample, 0)?;
        self.bytes.push(0xba);
        self.bytes
            .extend_from_slice(&(request.sample_identity as u32).to_le_bytes());
        self.bytes.push(0xb9);
        self.bytes
            .extend_from_slice(&(request.buffer_identity as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]);
        self.bytes
            .extend_from_slice(&(request.byte_count as u32).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb9]);
        self.bytes
            .extend_from_slice(&(request.credited_entropy_bits as u32).to_le_bytes());
        self.call_checked_bool(ProviderSymbol::CaptureEntropySample)?;
        self.entropy_sample_captured = true;
        Ok(())
    }

    fn assess_entropy_sample_quality(
        &mut self,
        request: CompilerEntropySampleQualityRequest,
    ) -> Result<(), CompileError> {
        if !self.entropy_sample_captured
            || self.entropy_sample_assessed
            || !request.is_well_formed()
            || request != CompilerEntropySampleQualityRequest::INITIAL_Q35_VIRTIO_RNG
        {
            return Err(CompileError::Tool(
                "x86 entropy-sample quality assessment requires the exact sealed Q35 trust evidence and full-quality credit after capture".into(),
            ));
        }
        self.entropy_sample_assessed = true;
        Ok(())
    }

    fn create_entropy_pool(
        &mut self,
        request: CompilerEntropyPoolRequest,
    ) -> Result<(), CompileError> {
        if !self.entropy_sample_assessed
            || self.entropy_pool_created
            || !request.is_well_formed()
            || request != CompilerEntropyPoolRequest::INITIAL_Q35
        {
            return Err(CompileError::Tool(
                "x86 entropy-pool creation requires the exact sealed empty 256-bit pool after sample assessment".into(),
            ));
        }
        self.entropy_pool_created = true;
        Ok(())
    }

    fn mix_entropy_pool_sample(
        &mut self,
        request: CompilerEntropyPoolMixRequest,
    ) -> Result<(), CompileError> {
        if !self.entropy_sample_assessed
            || !self.entropy_pool_created
            || self.entropy_pool_mixed
            || !request.is_well_formed()
            || request != CompilerEntropyPoolMixRequest::INITIAL_Q35
        {
            return Err(CompileError::Tool(
                "x86 entropy-pool mixing requires the exact sealed BLAKE2s first-generation request after assessment and empty-pool creation".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::RawEntropySample, 0)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
        self.rip_relative_symbol(ProviderSymbol::EntropyPoolState, 0)?;
        self.call_checked_bool(ProviderSymbol::MixEntropyPoolSample)?;
        self.entropy_pool_mixed = true;
        Ok(())
    }

    fn credit_entropy_pool_contribution(
        &mut self,
        request: CompilerEntropyPoolCreditRequest,
    ) -> Result<(), CompileError> {
        if !self.entropy_pool_mixed
            || self.entropy_pool_credited
            || !request.is_well_formed()
            || request != CompilerEntropyPoolCreditRequest::INITIAL_Q35
        {
            return Err(CompileError::Tool(
                "x86 entropy-credit transfer requires the exact sealed mixed pool and first contribution".into(),
            ));
        }
        self.entropy_pool_credited = true;
        Ok(())
    }

    fn initialize_entropy_generator(
        &mut self,
        request: CompilerEntropyGeneratorRequest,
    ) -> Result<(), CompileError> {
        if !self.entropy_pool_credited
            || self.entropy_generator_ready
            || !request.is_well_formed()
            || request != CompilerEntropyGeneratorRequest::INITIAL_Q35
        {
            return Err(CompileError::Tool(
                "x86 entropy-generator initialization requires the exact sealed credited pool and generation-one ChaCha20 request".into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::EntropyPoolState, 0)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
        self.rip_relative_symbol(ProviderSymbol::EntropyGeneratorKey, 0)?;
        self.call_checked_bool(ProviderSymbol::InitializeEntropyGenerator)?;
        self.entropy_generator_ready = true;
        Ok(())
    }

    fn extract_entropy_generator(
        &mut self,
        request: CompilerEntropyGeneratorExtractRequest,
    ) -> Result<(), CompileError> {
        if !self.entropy_generator_ready || !request.is_well_formed() {
            return Err(CompileError::Tool(
                "x86 entropy-generator extraction requires the next exact sealed ready ChaCha20 generation".into(),
            ));
        }
        if request == CompilerEntropyGeneratorExtractRequest::INITIAL_Q35 {
            if self.entropy_generator_extracted {
                return Err(CompileError::Tool(
                    "x86 first entropy-generator extraction was already consumed".into(),
                ));
            }
        } else if request == CompilerEntropyGeneratorExtractRequest::INITIAL_Q35_GETRANDOM {
            if !self.entropy_generator_extracted
                || !self.initial_auxv_random_delivered
                || self.initial_getrandom_extracted
            {
                return Err(CompileError::Tool(
                    "x86 second entropy-generator extraction requires consumed first output after initial AT_RANDOM delivery".into(),
                ));
            }
        } else {
            return Err(CompileError::Tool(
                "x86 entropy-generator extraction request is not admitted by the initial profile"
                    .into(),
            ));
        }
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]);
        self.rip_relative_symbol(ProviderSymbol::EntropyGeneratorKey, 0)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
        self.rip_relative_symbol(ProviderSymbol::EntropyOutput, 0)?;
        self.call_checked_bool(ProviderSymbol::ExtractEntropyGenerator)?;
        if request == CompilerEntropyGeneratorExtractRequest::INITIAL_Q35 {
            self.entropy_generator_extracted = true;
        } else {
            self.initial_getrandom_extracted = true;
        }
        Ok(())
    }

    fn deliver_initial_auxv_random(
        &mut self,
        request: CompilerEntropyOutputDeliveryRequest,
    ) -> Result<(), CompileError> {
        if !self.entropy_generator_extracted
            || self.initial_auxv_random_delivered
            || self.user_execution != RootUserExecutionState::PreparedWithResidentImage
            || !request.is_well_formed()
            || request != CompilerEntropyOutputDeliveryRequest::INITIAL_AUXV_RANDOM
        {
            return Err(CompileError::Tool(
                "x86 initial AT_RANDOM delivery requires the exact live first output and prepared resident initial process"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
        self.rip_relative_symbol(ProviderSymbol::EntropyOutput, 0)?;
        self.call_checked_bool(ProviderSymbol::DeliverInitialAuxvRandom)?;
        self.initial_auxv_random_delivered = true;
        self.user_execution = RootUserExecutionState::RandomizedEntryReady;
        Ok(())
    }

    fn reserve_initial_user_random(
        &mut self,
        request: CompilerEntropyOutputReservationRequest,
    ) -> Result<(), CompileError> {
        if !self.initial_getrandom_extracted
            || self.initial_user_random_reserved
            || self.user_execution != RootUserExecutionState::RandomizedEntryReady
            || !request.is_well_formed()
            || request != CompilerEntropyOutputReservationRequest::INITIAL_GETRANDOM
        {
            return Err(CompileError::Tool(
                "x86 initial getrandom reservation requires the exact live second output and randomized entry-ready process"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
        self.rip_relative_symbol(ProviderSymbol::EntropyOutput, 0)?;
        self.call_checked_bool(ProviderSymbol::ReserveInitialUserRandom)?;
        self.initial_user_random_reserved = true;
        self.user_execution = RootUserExecutionState::UserRandomReservedEntryReady;
        Ok(())
    }

    fn bind_reusable_user_random(
        &mut self,
        request: CompilerEntropyUserRandomServiceRequest,
    ) -> Result<(), CompileError> {
        if !self.initial_user_random_reserved
            || self.user_random_service_bound
            || self.user_execution != RootUserExecutionState::UserRandomReservedEntryReady
            || !request.is_well_formed()
            || request != CompilerEntropyUserRandomServiceRequest::INITIAL_GETRANDOM
        {
            return Err(CompileError::Tool(
                "x86 reusable getrandom binding requires the exact live generation-three reservation and entry-ready process"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]);
        self.rip_relative_symbol(ProviderSymbol::EntropyGeneratorKey, 0)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x15]);
        self.rip_relative_symbol(ProviderSymbol::EntropyUserRandomService, 0)?;
        self.call_checked_bool(ProviderSymbol::BindReusableUserRandom)?;
        self.initial_user_random_reserved = false;
        self.user_random_service_bound = true;
        self.user_execution = RootUserExecutionState::UserRandomServiceEntryReady;
        Ok(())
    }

    fn expand_small_user_random_requests(
        &mut self,
        request: CompilerEntropyUserRandomSmallRequest,
    ) -> Result<(), CompileError> {
        if !self.user_random_service_bound
            || self.user_random_small_requests_enabled
            || self.user_execution != RootUserExecutionState::UserRandomServiceEntryReady
            || !request.is_well_formed()
            || request != CompilerEntropyUserRandomSmallRequest::INITIAL_GETRANDOM
        {
            return Err(CompileError::Tool(
                "x86 small-request getrandom refinement requires the exact live reusable service before entry"
                    .into(),
            ));
        }
        self.user_random_small_requests_enabled = true;
        Ok(())
    }

    fn expand_large_user_random_requests(
        &mut self,
        request: CompilerEntropyUserRandomSmallRequest,
    ) -> Result<(), CompileError> {
        if !self.user_random_service_bound
            || !self.user_random_small_requests_enabled
            || self.user_random_large_requests_enabled
            || self.user_execution != RootUserExecutionState::UserRandomServiceEntryReady
            || !request.is_well_formed()
            || request != CompilerEntropyUserRandomSmallRequest::LARGE_GETRANDOM
        {
            return Err(CompileError::Tool(
                "x86 large-request getrandom refinement requires the exact live small-request service before entry"
                    .into(),
            ));
        }
        self.user_random_large_requests_enabled = true;
        Ok(())
    }

    fn publish_pci_region_dma_reference(
        &mut self,
        request: CompilerPciRegionDmaReferenceRequest,
    ) -> Result<(), CompileError> {
        let expected = CompilerPciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
            .get(self.dma_references_published);
        if self.dma_buffers_prepared
            != CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG.len()
            || expected != Some(&request)
            || !request.is_well_formed()
        {
            return Err(CompileError::Tool(
                "x86 PCI-region DMA-reference publication requires both device-owned buffers and the exact sealed queue-address sequence"
                    .into(),
            ));
        }
        let selector = 0x8000_0000
            | (u32::from(request.bus) << 16)
            | (u32::from(request.device) << 11)
            | (u32::from(request.function) << 8)
            | u32::from(request.access_capability_offset_bytes);
        let target_offset = request
            .region_offset_bytes
            .checked_add(request.register_offset_bytes)
            .ok_or_else(|| {
                CompileError::Tool("PCI DMA-reference target offset overflowed".into())
            })?;
        self.bytes.push(0xbf);
        self.bytes.extend_from_slice(&selector.to_le_bytes());
        self.bytes.push(0xbe);
        self.bytes
            .extend_from_slice(&u32::from(request.bar).to_le_bytes());
        self.bytes.push(0xba);
        self.bytes.extend_from_slice(&target_offset.to_le_bytes());
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x0d]);
        self.rip_relative_symbol(ProviderSymbol::DmaVirtqueueControl, 0)?;
        self.bytes.extend_from_slice(&[0x41, 0xb8]);
        self.bytes
            .extend_from_slice(&(request.buffer_offset_bytes as u32).to_le_bytes());
        self.call_checked_bool(ProviderSymbol::PublishPciRegionDmaReference)?;
        self.dma_references_published += 1;
        Ok(())
    }

    fn access_pci_region_register(
        &mut self,
        request: CompilerPciRegionRegisterRequest,
    ) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.pci_configuration_observations
                != CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY.len()
            || {
                let negotiation_len =
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION.len();
                let discovery_len =
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY.len();
                let activation_len =
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION.len();
                let notification_len =
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION
                        .len();
                let expected = if self.pci_region_register_accesses < negotiation_len {
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                        .get(self.pci_region_register_accesses)
                } else if self.pci_region_register_accesses < negotiation_len + discovery_len {
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY
                        .get(self.pci_region_register_accesses - negotiation_len)
                } else if self.pci_region_register_accesses
                    < negotiation_len + discovery_len + activation_len
                {
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
                        .get(self.pci_region_register_accesses - negotiation_len - discovery_len)
                } else if self.pci_region_register_accesses
                    < negotiation_len + discovery_len + activation_len + notification_len
                {
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION.get(
                        self.pci_region_register_accesses
                            - negotiation_len
                            - discovery_len
                            - activation_len,
                    )
                } else {
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL.get(
                        self.pci_region_register_accesses
                            - negotiation_len
                            - discovery_len
                            - activation_len
                            - notification_len,
                    )
                };
                expected != Some(&request)
                    || (self.pci_region_register_accesses >= negotiation_len + discovery_len
                        && self.dma_references_published
                            != CompilerPciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES.len())
            }
        {
            return Err(CompileError::Tool(
                "x86 PCI region access requires completed capability discovery and the exact sealed virtio-rng negotiation, queue-discovery, queue-activation, queue-notification, and completion-signal sequences".into(),
            ));
        }
        let selector = 0x8000_0000
            | (u32::from(request.bus) << 16)
            | (u32::from(request.device) << 11)
            | (u32::from(request.function) << 8)
            | u32::from(request.access_capability_offset_bytes);
        let target_offset = request
            .region_offset_bytes
            .checked_add(request.register_offset_bytes)
            .ok_or_else(|| CompileError::Tool("PCI region target offset overflowed".into()))?;
        self.bytes.push(0xbf); // configuration access selector in edi
        self.bytes.extend_from_slice(&selector.to_le_bytes());
        self.bytes.push(0xbe); // BAR index in esi
        self.bytes
            .extend_from_slice(&u32::from(request.bar).to_le_bytes());
        self.bytes.push(0xba); // BAR-relative target offset in edx
        self.bytes.extend_from_slice(&target_offset.to_le_bytes());
        self.bytes.push(0xb9); // exact access width in ecx
        self.bytes
            .extend_from_slice(&u32::from(request.width_bytes).to_le_bytes());
        self.bytes.extend_from_slice(&[0x41, 0xb8]); // value in r8d
        self.bytes.extend_from_slice(&request.value.to_le_bytes());
        let symbol = match request.direction {
            CompilerPciRegionRegisterDirection::ReadEquals => ProviderSymbol::ReadPciRegionRegister,
            CompilerPciRegionRegisterDirection::ReadEventuallyEquals => {
                ProviderSymbol::ReadPciRegionRegisterEventually
            }
            CompilerPciRegionRegisterDirection::Write => ProviderSymbol::WritePciRegionRegister,
        };
        self.call_checked_bool(symbol)?;
        self.pci_region_register_accesses += 1;
        Ok(())
    }

    fn construct_deadline(&mut self, duration_nanoseconds: u64) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.monotonic_observations != 2
            || self.deadline != RootDeadlineState::Fresh
            || duration_nanoseconds != 1_000_000
        {
            return Err(CompileError::Tool(
                "x86 deadline construction requires the second clock observation and exact duration 1[ms]".into(),
            ));
        }
        self.load_bootstrap_storage_address(24)?;
        self.call_checked_bool(ProviderSymbol::ConstructDeadline)?;
        self.deadline = RootDeadlineState::Constructed;
        Ok(())
    }

    fn arm_deadline(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.deadline != RootDeadlineState::Constructed
            || self.critical != RootCriticalState::Restored
        {
            return Err(CompileError::Tool(
                "x86 deadline arm requires one constructed deadline and restored processor context"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(24)?;
        self.call_checked_bool(ProviderSymbol::ArmDeadline)?;
        self.deadline = RootDeadlineState::Armed;
        Ok(())
    }

    fn wait_deadline(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.deadline != RootDeadlineState::Armed
        {
            return Err(CompileError::Tool(
                "x86 deadline wait requires one matching armed event".into(),
            ));
        }
        self.load_bootstrap_storage_address(24)?;
        self.call_checked_bool(ProviderSymbol::WaitDeadline)?;
        self.deadline = RootDeadlineState::Completed;
        Ok(())
    }

    fn complete_deadline(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::DeadlineHandler {
            return Err(CompileError::Tool(
                "x86 deadline completion is admitted only in its external entry".into(),
            ));
        }
        self.load_bootstrap_storage_address(24)?;
        self.call_checked_bool(ProviderSymbol::CompleteDeadline)
    }

    fn create_kernel_context(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.deadline != RootDeadlineState::Constructed
        {
            return Err(CompileError::Tool(
                "x86 kernel-context creation requires one constructed deadline and its sealed stack region"
                    .into(),
            ));
        }
        let (index, state_offset, stack_offset, entry) = match self.kernel_context {
            RootKernelContextState::Fresh if self.bootstrap_region_offset == Some(4096) => {
                (0, 80, 4096, ProviderSymbol::KernelThreadEntry)
            }
            RootKernelContextState::CooperativeCreated
                if self.bootstrap_region_offset == Some(20_480) =>
            {
                (1, 112, 20_480, ProviderSymbol::CompletionThreadEntry)
            }
            RootKernelContextState::CompletionCreated
                if self.bootstrap_region_offset == Some(36_864) =>
            {
                (2, 144, 36_864, ProviderSymbol::TerminalThreadEntry)
            }
            _ => {
                return Err(CompileError::Tool(
                    "x86 profile creates exactly three contexts from its sealed disjoint stacks"
                        .into(),
                ));
            }
        };
        self.load_bootstrap_storage_address(state_offset)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]); // lea rsi, stack
        self.rip_relative_storage(stack_offset, 0)?;
        self.bytes
            .extend_from_slice(&[0xba, 0x00, 0x40, 0x00, 0x00]); // 16 KiB
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x0d]); // lea rcx, worker entry
        self.rip_relative_symbol(entry, 0)?;
        self.call_checked_bool(ProviderSymbol::CreateKernelContext)?;
        self.context_stack_offsets[index] = self.bootstrap_region_offset.take();
        self.kernel_context = match index {
            0 => RootKernelContextState::CooperativeCreated,
            1 => RootKernelContextState::CompletionCreated,
            2 => RootKernelContextState::AllSuspended,
            _ => unreachable!("closed context index"),
        };
        Ok(())
    }

    fn create_kernel_completion(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.completion != RootCompletionState::Fresh
            || self.kernel_context != RootKernelContextState::Fresh
        {
            return Err(CompileError::Tool(
                "x86 completion creation requires one fresh dispatcher state".into(),
            ));
        }
        self.completion = RootCompletionState::Created;
        Ok(())
    }

    fn transfer_kernel_context(&mut self) -> Result<(), CompileError> {
        if self.role == RootRole::CooperativeKernelThreadHandler {
            self.load_bootstrap_storage_address(80)?;
            self.call_checked_bool(ProviderSymbol::TransferKernelContext)?;
            return Ok(());
        }
        if self.role != RootRole::BootstrapWithLocalNotification {
            return Err(CompileError::Tool(
                "x86 cooperative transfer requires the dispatcher or cooperative worker".into(),
            ));
        }
        let (state_offset, next_state) = match self.kernel_context {
            RootKernelContextState::CooperativeDequeued
                if self.context_stack_offsets == [Some(4096), Some(20_480), Some(36_864)] =>
            {
                (80, RootKernelContextState::CooperativeYielded)
            }
            RootKernelContextState::TerminalDequeued
                if self.context_stack_offsets == [Some(4096), Some(20_480), Some(36_864)] =>
            {
                (144, RootKernelContextState::TerminalCompleted)
            }
            RootKernelContextState::CooperativeDequeuedAgain
                if self.context_stack_offsets == [Some(4096), Some(20_480), None] =>
            {
                (80, RootKernelContextState::CooperativeCompleted)
            }
            RootKernelContextState::CompletionDequeuedAgain
                if self.context_stack_offsets == [None, Some(20_480), None] =>
            {
                (112, RootKernelContextState::CompletionCompleted)
            }
            _ => {
                return Err(CompileError::Tool(
                    "x86 kernel-context transfers must follow terminal, preemptible, then awakened-completion order"
                        .into(),
                ));
            }
        };
        self.load_bootstrap_storage_address(state_offset)?;
        self.call_checked_bool(ProviderSymbol::TransferKernelContext)?;
        self.kernel_context = next_state;
        Ok(())
    }

    fn transfer_kernel_context_until_deadline(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.kernel_context != RootKernelContextState::CooperativeDequeued
            || self.context_stack_offsets != [Some(4096), Some(20_480), Some(36_864)]
            || self.deadline != RootDeadlineState::Constructed
            || self.critical != RootCriticalState::Restored
        {
            return Err(CompileError::Tool(
                "x86 deadline-bound transfer requires the first selected worker, constructed deadline, and restored processor context".into(),
            ));
        }
        self.load_bootstrap_storage_address(24)?;
        self.call_checked_bool(ProviderSymbol::ArmDeadline)?;
        self.deadline = RootDeadlineState::Armed;
        self.load_bootstrap_storage_address(80)?;
        self.call_checked_bool(ProviderSymbol::TransferKernelContext)?;
        self.deadline = RootDeadlineState::Completed;
        self.kernel_context = RootKernelContextState::CooperativeYielded;
        Ok(())
    }

    fn await_deadline_preemption(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::CooperativeKernelThreadHandler {
            return Err(CompileError::Tool(
                "x86 deadline-preemption await is admitted only in its bound worker".into(),
            ));
        }
        self.load_bootstrap_storage_address(80)?;
        self.call_checked_bool(ProviderSymbol::AwaitDeadlinePreemption)
    }

    fn transfer_kernel_context_until_completion(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.kernel_context != RootKernelContextState::CompletionDequeued
            || self.completion != RootCompletionState::Created
            || self.context_stack_offsets != [Some(4096), Some(20_480), Some(36_864)]
            || self.critical != RootCriticalState::Restored
        {
            return Err(CompileError::Tool(
                "x86 completion-bound transfer requires the fresh selected block-once worker"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(112)?;
        self.call_checked_bool(ProviderSymbol::TransferKernelContext)?;
        self.kernel_context = RootKernelContextState::CompletionBlocked;
        self.completion = RootCompletionState::Blocked;
        Ok(())
    }

    fn await_kernel_completion(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::CompletionKernelThreadHandler {
            return Err(CompileError::Tool(
                "x86 completion await is admitted only in its bound block-once worker".into(),
            ));
        }
        self.load_bootstrap_storage_address(112)?;
        self.call_checked_bool(ProviderSymbol::TransferKernelContext)
    }

    fn signal_blocked_kernel_completion(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.kernel_context != RootKernelContextState::TerminalReclaimed
            || self.completion != RootCompletionState::Blocked
        {
            return Err(CompileError::Tool(
                "x86 completion signal requires the exact blocked worker after terminal reclamation"
                    .into(),
            ));
        }
        self.kernel_context = RootKernelContextState::CompletionSignaled;
        self.completion = RootCompletionState::Signaled;
        Ok(())
    }

    fn consume_kernel_completion(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.kernel_context != RootKernelContextState::CompletionReclaimed
            || self.completion != RootCompletionState::Signaled
        {
            return Err(CompileError::Tool(
                "x86 completion consumption requires its awakened worker reclamation".into(),
            ));
        }
        self.completion = RootCompletionState::Consumed;
        Ok(())
    }

    fn take_preempted_kernel_context(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.kernel_context != RootKernelContextState::CooperativeYielded
            || self.deadline != RootDeadlineState::Completed
        {
            return Err(CompileError::Tool(
                "x86 preempted-context take requires the completed bound deadline transfer".into(),
            ));
        }
        self.kernel_context = RootKernelContextState::PreemptedTaken;
        Ok(())
    }

    fn preempt_current_kernel_context(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::DeadlineHandler {
            return Err(CompileError::Tool(
                "x86 deadline preemption is admitted only in its external entry".into(),
            ));
        }
        self.load_bootstrap_storage_address(80)?;
        self.call_checked_bool(ProviderSymbol::PreemptCurrentKernelContext)
    }

    fn create_runnable_queue(&mut self, capacity: usize) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.kernel_context != RootKernelContextState::AllSuspended
            || capacity != 3
        {
            return Err(CompileError::Tool(
                "x86 root runnable queue requires three suspended contexts and capacity three"
                    .into(),
            ));
        }
        self.kernel_context = RootKernelContextState::QueueCreated;
        Ok(())
    }

    fn enqueue_runnable_context(&mut self) -> Result<(), CompileError> {
        self.kernel_context = match self.kernel_context {
            RootKernelContextState::QueueCreated => RootKernelContextState::CooperativeEnqueued,
            RootKernelContextState::CooperativeEnqueued => {
                RootKernelContextState::CompletionEnqueued
            }
            RootKernelContextState::CompletionEnqueued => RootKernelContextState::AllEnqueued,
            RootKernelContextState::PreemptedTaken => RootKernelContextState::HandoffEnqueued,
            RootKernelContextState::CompletionSignaled => {
                RootKernelContextState::CompletionEnqueuedAgain
            }
            _ => {
                return Err(CompileError::Tool(
                    "x86 root runnable enqueue violates the sealed three-worker FIFO ownership"
                        .into(),
                ));
            }
        };
        Ok(())
    }

    fn dequeue_runnable_context(&mut self) -> Result<(), CompileError> {
        self.kernel_context = match self.kernel_context {
            RootKernelContextState::AllEnqueued => RootKernelContextState::CooperativeDequeued,
            RootKernelContextState::HandoffEnqueued => RootKernelContextState::CompletionDequeued,
            RootKernelContextState::CompletionBlocked => RootKernelContextState::TerminalDequeued,
            RootKernelContextState::CompletionEnqueuedAgain => {
                RootKernelContextState::CooperativeDequeuedAgain
            }
            RootKernelContextState::CooperativeReclaimed => {
                RootKernelContextState::CompletionDequeuedAgain
            }
            _ => {
                return Err(CompileError::Tool(
                    "x86 root runnable dequeue violates the sealed three-worker FIFO ownership"
                        .into(),
                ));
            }
        };
        Ok(())
    }

    fn consume_empty_runnable_queue(&mut self) -> Result<(), CompileError> {
        if self.kernel_context != RootKernelContextState::CompletionReclaimed
            || self.completion != RootCompletionState::Consumed
        {
            return Err(CompileError::Tool(
                "x86 root runnable queue must be empty after all context and completion reclamations".into(),
            ));
        }
        self.kernel_context = RootKernelContextState::QueueConsumed;
        Ok(())
    }

    fn retire_kernel_context(&mut self) -> Result<(), CompileError> {
        let state_offset = match self.role {
            RootRole::CooperativeKernelThreadHandler => 80,
            RootRole::CompletionKernelThreadHandler => 112,
            RootRole::TerminalKernelThreadHandler => 144,
            _ => {
                return Err(CompileError::Tool(
                    "x86 kernel-context retirement is admitted only in a resumed entry".into(),
                ));
            }
        };
        self.load_bootstrap_storage_address(state_offset)?;
        self.jump(ProviderSymbol::RetireKernelContext);
        Ok(())
    }

    fn reclaim_kernel_context(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification {
            return Err(CompileError::Tool(
                "x86 kernel-context reclaim requires one completed transfer".into(),
            ));
        }
        let (index, state_offset, next_state) = match self.kernel_context {
            RootKernelContextState::TerminalCompleted
                if self.context_stack_offsets[2] == Some(36_864) =>
            {
                (2, 144, RootKernelContextState::TerminalReclaimed)
            }
            RootKernelContextState::CooperativeCompleted
                if self.context_stack_offsets[0] == Some(4096) =>
            {
                (0, 80, RootKernelContextState::CooperativeReclaimed)
            }
            RootKernelContextState::CompletionCompleted
                if self.context_stack_offsets[1] == Some(20_480) =>
            {
                (1, 112, RootKernelContextState::CompletionReclaimed)
            }
            _ => {
                return Err(CompileError::Tool(
                    "x86 kernel-context reclaim requires the next completed FIFO context".into(),
                ));
            }
        };
        self.load_bootstrap_storage_address(state_offset)?;
        self.call_checked_bool(ProviderSymbol::ReclaimKernelContext)?;
        self.context_stack_offsets[index] = None;
        self.kernel_context = next_state;
        Ok(())
    }

    fn prepare_user_address_space(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.kernel_context != RootKernelContextState::QueueConsumed
            || self.completion != RootCompletionState::Consumed
            || self.user_execution != RootUserExecutionState::Fresh
            || self.physical_frame_count != Some(3)
            || self.translation != RootTranslationState::RemovedActive
            || self.critical != RootCriticalState::Restored
        {
            return Err(CompileError::Tool(
                "x86 user-address-space preparation requires completed kernel cleanup, exact three-frame backing, and the restored replacement translation".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xda]); // backing base in rdx
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xee]); // retained boot params in rsi
        self.call_checked_bool(ProviderSymbol::PrepareUserAddressSpace)?;
        self.bytes.extend_from_slice(&[0x48, 0x31, 0xdb]);
        self.physical_frame_count = None;
        self.user_execution = RootUserExecutionState::AddressSpacePrepared;
        Ok(())
    }

    fn prepare_user_probe(&mut self, process_identity: u64) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::AnonymousUserMappingReserved
            || self.physical_frame_count.is_some()
            || process_identity != 1
        {
            return Err(CompileError::Tool(
                "x86 user probe preparation requires PID one, both finalized image mappings, and no source-owned extent".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x35]); // syscall entry in RSI
        self.rip_relative_symbol(ProviderSymbol::UserSyscall, 0)?;
        self.call_checked_bool(ProviderSymbol::PrepareUserProbe)?;
        self.user_execution = RootUserExecutionState::PreparedWithResidentImage;
        Ok(())
    }

    fn verify_user_address_space_mapping(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::AddressSpacePrepared
            || self.physical_frame_count != Some(2)
        {
            return Err(CompileError::Tool(
                "x86 user mapping verification requires the prepared address space and its two-frame code extent".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xda]); // disjoint frame base in rdx
        self.call_checked_bool(ProviderSymbol::VerifyUserAddressSpaceMapping)?;
        self.user_execution = RootUserExecutionState::AddressSpaceVerified;
        Ok(())
    }

    fn map_user_address_space(&mut self) -> Result<(), CompileError> {
        let generation = match self.user_execution {
            RootUserExecutionState::AddressSpacePrepared => 1,
            RootUserExecutionState::UserMappingExecutable(1) => 2,
            _ => 0,
        };
        if self.role != RootRole::BootstrapWithLocalNotification
            || generation == 0
            || self.physical_frame_count != Some(if generation == 1 { 2 } else { 1 })
        {
            return Err(CompileError::Tool(
                "x86 user map requires the prepared address space and exact role extent".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xda]);
        self.call_checked_bool(ProviderSymbol::MapUserAddressSpace)?;
        self.bytes.extend_from_slice(&[0x48, 0x31, 0xdb]);
        self.physical_frame_count = None;
        self.user_execution = RootUserExecutionState::UserMappingLive(generation);
        Ok(())
    }

    fn store_user_mapping_byte(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::UserMappingLive(2)
            || self.physical_frame_count.is_some()
        {
            return Err(CompileError::Tool(
                "x86 user mapping store requires the exact live mapping".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xee]); // retained boot params in rsi
        self.call_checked_bool(ProviderSymbol::StoreUserMappingByte)?;
        let RootUserExecutionState::UserMappingLive(generation) = self.user_execution else {
            unreachable!("live mapping checked above");
        };
        self.user_execution = RootUserExecutionState::UserMappingStored(generation);
        Ok(())
    }

    fn install_user_mapping_entry_trampoline(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::UserMappingLive(1)
            || self.physical_frame_count.is_some()
        {
            return Err(CompileError::Tool(
                "x86 entry-trampoline installation requires the first exact live mapping".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xee]); // retained boot params in rsi
        self.call_checked_bool(ProviderSymbol::InstallUserMappingEntryTrampoline)?;
        self.user_execution = RootUserExecutionState::UserMappingEntryInstalled(1);
        Ok(())
    }

    fn protect_user_mapping_read_only(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || !matches!(
                self.user_execution,
                RootUserExecutionState::UserMappingStored(_)
                    | RootUserExecutionState::UserMappingEntryInstalled(_)
            )
            || self.physical_frame_count.is_some()
        {
            return Err(CompileError::Tool(
                "x86 user mapping protection requires the exact stored writable mapping".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.call_checked_bool(ProviderSymbol::ProtectUserMappingReadOnly)?;
        let generation = match self.user_execution {
            RootUserExecutionState::UserMappingStored(generation)
            | RootUserExecutionState::UserMappingEntryInstalled(generation) => generation,
            _ => unreachable!("populated mapping checked above"),
        };
        self.user_execution = RootUserExecutionState::UserMappingProtected(generation);
        Ok(())
    }

    fn load_user_mapping_byte(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::UserMappingProtected(2)
            || self.physical_frame_count.is_some()
        {
            return Err(CompileError::Tool(
                "x86 user mapping load requires the exact protected mapping".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xee]); // retained boot params in rsi
        self.call_checked_bool(ProviderSymbol::LoadUserMappingByte)?;
        let RootUserExecutionState::UserMappingProtected(generation) = self.user_execution else {
            unreachable!("protected mapping checked above");
        };
        self.user_execution = RootUserExecutionState::UserMappingVerified(generation);
        Ok(())
    }

    fn verify_user_mapping_entry_trampoline(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::UserMappingProtected(1)
            || self.physical_frame_count.is_some()
        {
            return Err(CompileError::Tool(
                "x86 entry-trampoline verification requires the first exact protected mapping"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x4c, 0x89, 0xee]); // retained boot params in rsi
        self.call_checked_bool(ProviderSymbol::VerifyUserMappingEntryTrampoline)?;
        self.user_execution = RootUserExecutionState::UserMappingVerified(1);
        Ok(())
    }

    fn permit_user_mapping_execute(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::UserMappingVerified(1)
            || self.physical_frame_count.is_some()
        {
            return Err(CompileError::Tool(
                "x86 user mapping execute promotion requires the exact verified mapping".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.call_checked_bool(ProviderSymbol::PermitUserMappingExecute)?;
        let RootUserExecutionState::UserMappingVerified(generation) = self.user_execution else {
            unreachable!("verified mapping checked above");
        };
        self.user_execution = RootUserExecutionState::UserMappingExecutable(generation);
        Ok(())
    }

    fn seal_user_mapping_immutable_data(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::UserMappingVerified(2)
            || self.physical_frame_count.is_some()
        {
            return Err(CompileError::Tool(
                "x86 immutable-data seal requires the exact second verified mapping".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.call_checked_bool(ProviderSymbol::SealUserMappingImmutableData)?;
        self.user_execution = RootUserExecutionState::UserMappingImmutableData(2);
        Ok(())
    }

    fn permit_user_mapping_writable_data(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::UserMappingVerified(2)
            || self.physical_frame_count.is_some()
        {
            return Err(CompileError::Tool(
                "x86 writable-data promotion requires the exact second verified mapping".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.call_checked_bool(ProviderSymbol::PermitUserMappingWritableData)?;
        self.user_execution = RootUserExecutionState::UserMappingWritableData(2);
        Ok(())
    }

    fn reserve_anonymous_user_mapping(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::UserMappingWritableData(2)
            || self.physical_frame_count != Some(34)
        {
            return Err(CompileError::Tool(
                "x86 anonymous reservation requires both finalized image mappings and one source-owned frame"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xde]); // anonymous frame in rsi
        self.call_checked_bool(ProviderSymbol::ReserveAnonymousUserMapping)?;
        self.bytes.extend_from_slice(&[0x48, 0x31, 0xdb]);
        self.physical_frame_count = None;
        self.user_execution = RootUserExecutionState::AnonymousUserMappingReserved;
        Ok(())
    }

    fn release_anonymous_user_mapping(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::CompletedWithResidentImage
            || self.physical_frame_count.is_some()
        {
            return Err(CompileError::Tool(
                "x86 anonymous release requires returned user execution and its live reservation"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.call(ProviderSymbol::ReleaseAnonymousUserMapping);
        self.bytes.extend_from_slice(&[0x48, 0x85, 0xc0]);
        self.jump_to_fatal_if(0x84)?;
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xc3]);
        self.physical_frame_count = Some(34);
        self.user_execution = RootUserExecutionState::CompletedWithResidentImageAnonymousReleased;
        Ok(())
    }

    fn unmap_user_address_space(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || !matches!(
                self.user_execution,
                RootUserExecutionState::CompletedWithExecutableImage
                    | RootUserExecutionState::CompletedWithResidentImageAnonymousReleased
            )
            || self.physical_frame_count.is_some()
        {
            return Err(CompileError::Tool(
                "x86 user unmap requires returned execution with the exact finalized code or data mapping".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.call(ProviderSymbol::UnmapUserAddressSpace);
        self.bytes.extend_from_slice(&[0x48, 0x85, 0xc0]);
        self.jump_to_fatal_if(0x84)?;
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xc3]);
        let generation = match self.user_execution {
            RootUserExecutionState::CompletedWithExecutableImage => 1,
            RootUserExecutionState::CompletedWithResidentImageAnonymousReleased => 2,
            _ => unreachable!("finalized mapping checked above"),
        };
        self.physical_frame_count = Some(if generation == 1 { 2 } else { 1 });
        self.user_execution = match generation {
            1 => RootUserExecutionState::CompletedImageRevoked,
            2 => RootUserExecutionState::CompletedWithExecutableImage,
            _ => unreachable!("only two mapping generations are constructed"),
        };
        Ok(())
    }

    fn enter_user_context(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::UserRandomServiceEntryReady
            || !self.user_random_service_bound
            || !self.user_random_small_requests_enabled
            || !self.user_random_large_requests_enabled
        {
            return Err(CompileError::Tool(
                "x86 user entry requires the randomized and reusable-getrandom-bound entry-ready context with both resident image mappings"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.call_checked_bool(ProviderSymbol::EnterUserContext)?;
        self.user_execution = RootUserExecutionState::CompletedWithResidentImage;
        Ok(())
    }

    fn complete_user_execution(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::CompletedImageRevoked
        {
            return Err(CompileError::Tool(
                "x86 user completion requires terminal exit_group restoration and image teardown"
                    .into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.call_checked_bool(ProviderSymbol::CompleteUserBacking)?;
        self.user_execution = RootUserExecutionState::Revoked;
        Ok(())
    }

    fn reclaim_user_address_space(&mut self) -> Result<(), CompileError> {
        if self.role != RootRole::BootstrapWithLocalNotification
            || self.user_execution != RootUserExecutionState::Revoked
        {
            return Err(CompileError::Tool(
                "x86 user-address-space reclaim requires the exact revoked space".into(),
            ));
        }
        self.load_bootstrap_storage_address(176)?;
        self.call(ProviderSymbol::ReclaimUserAddressSpace);
        self.bytes.extend_from_slice(&[0x48, 0x85, 0xc0]);
        self.jump_to_fatal_if(0x84)?;
        self.bytes.extend_from_slice(&[0x48, 0x89, 0xc3]);
        self.physical_frame_count = Some(3);
        self.user_execution = RootUserExecutionState::Consumed;
        Ok(())
    }

    fn load_bootstrap_storage_address(&mut self, storage_offset: u64) -> Result<(), CompileError> {
        self.bytes.extend_from_slice(&[0x48, 0x8d, 0x3d]); // lea rdi, [rip+disp32]
        self.rip_relative_storage(storage_offset, 0)
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

    fn rip_relative_symbol(
        &mut self,
        target: ProviderSymbol,
        trailing_bytes: i64,
    ) -> Result<(), CompileError> {
        self.relocations.push(PendingRelocation {
            offset: u64::try_from(self.bytes.len()).map_err(|_| {
                CompileError::Tool("generated root relocation offset exceeds u64".into())
            })?,
            target,
            addend: -4 - trailing_bytes,
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

#[allow(clippy::too_many_lines)] // Every typed provider dependency is constructed in one closed root boundary.
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
                    | CompilerSystemsOperation::AtomicWordCreate { .. }
                    | CompilerSystemsOperation::AtomicCompareExchangeEquals { .. }
                    | CompilerSystemsOperation::AtomicLoadEquals { .. }
                    | CompilerSystemsOperation::AtomicWordEnd
                    | CompilerSystemsOperation::SendLocalNotification
                    | CompilerSystemsOperation::WaitLocalNotification
                    | CompilerSystemsOperation::ObserveMonotonicClock
                    | CompilerSystemsOperation::ConstructDeadline { .. }
                    | CompilerSystemsOperation::ArmDeadline
                    | CompilerSystemsOperation::WaitDeadline
                    | CompilerSystemsOperation::BootstrapRelease
                    | CompilerSystemsOperation::PrepareUserAddressSpace
                    | CompilerSystemsOperation::MapUserAddressSpace
                    | CompilerSystemsOperation::StoreUserMappingByte
                    | CompilerSystemsOperation::InstallUserMappingEntryTrampoline
                    | CompilerSystemsOperation::ProtectUserMappingReadOnly
                    | CompilerSystemsOperation::LoadUserMappingByte
                    | CompilerSystemsOperation::VerifyUserMappingEntryTrampoline
                    | CompilerSystemsOperation::PermitUserMappingExecute
                    | CompilerSystemsOperation::SealUserMappingImmutableData
                    | CompilerSystemsOperation::PermitUserMappingWritableData
                    | CompilerSystemsOperation::UnmapUserAddressSpace
                    | CompilerSystemsOperation::PrepareUserProbe { .. }
                    | CompilerSystemsOperation::EnterUserContext
                    | CompilerSystemsOperation::CompleteUserExecution
            )
        })
        || program
            .local_notification
            .handler
            .operations
            .iter()
            .any(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::CompleteLocalNotification
                )
            })
        || program
            .deadline_notification
            .handler
            .operations
            .iter()
            .any(|operation| matches!(operation, CompilerSystemsOperation::CompleteDeadline));
    let mut symbols = ProviderSymbols {
        describe_boot_memory: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::DescribeBootMemory,
        ),
        allocate_physical_frames: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AllocatePhysicalFrames,
        ),
        prepare_static_image: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::PrepareStaticImage,
        ),
        zero_static_image_payload: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ZeroStaticImagePayload,
        ),
        zero_static_image_stack_backing: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ZeroStaticImageStackBacking,
        ),
        publish_static_image_stack_leaf: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::PublishStaticImageStackLeaf,
        ),
        initialize_static_image_stack_entry: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::InitializeStaticImageStackEntry,
        ),
        add_static_image_stack_pagesz: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AddStaticImageStackPageSize,
        ),
        add_static_image_stack_entry_fact: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AddStaticImageStackEntryFact,
        ),
        add_static_image_stack_program_headers: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AddStaticImageStackProgramHeaders,
        ),
        add_static_image_stack_executable_name: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AddStaticImageStackExecutableName,
        ),
        add_static_image_stack_credentials: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AddStaticImageStackCredentials,
        ),
        add_static_image_stack_secure_mode: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AddStaticImageStackSecureMode,
        ),
        add_static_image_stack_clock_tick_rate: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AddStaticImageStackClockTickRate,
        ),
        add_static_image_stack_platform: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AddStaticImageStackPlatform,
        ),
        add_static_image_stack_hardware_capabilities: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AddStaticImageStackHardwareCapabilities,
        ),
        add_static_image_stack_load_metadata: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AddStaticImageStackLoadMetadata,
        ),
        authorize_static_image_stack_random_slot: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AuthorizeStaticImageStackRandomSlot,
        ),
        retain_static_image_prepared_process: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RetainStaticImagePreparedProcess,
        ),
        qualify_static_image_retained_disjoint_work: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageRetainedDisjointWork,
        ),
        authorize_static_image_entropy_handoff: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AuthorizeStaticImageEntropyHandoff,
        ),
        authorize_static_image_entropy_acquisition: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AuthorizeStaticImageEntropyAcquisition,
        ),
        qualify_static_image_entropy_device: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyDevice,
        ),
        qualify_static_image_entropy_transport: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyTransport,
        ),
        qualify_static_image_entropy_negotiation: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyNegotiation,
        ),
        qualify_static_image_entropy_queue: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyQueue,
        ),
        qualify_static_image_entropy_bus_master: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyBusMaster,
        ),
        qualify_static_image_entropy_dma_buffers: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyDmaBuffers,
        ),
        qualify_static_image_entropy_split_ring: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropySplitRing,
        ),
        qualify_static_image_entropy_dma_preparation: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyDmaPreparation,
        ),
        qualify_static_image_entropy_queue_addresses: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyQueueAddresses,
        ),
        qualify_static_image_entropy_queue_activation: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyQueueActivation,
        ),
        qualify_static_image_entropy_request_completion: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyRequestCompletion,
        ),
        qualify_static_image_entropy_sample_capture: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropySampleCapture,
        ),
        qualify_static_image_entropy_sample_quality: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropySampleQuality,
        ),
        qualify_static_image_entropy_pool_lifecycle: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyPoolLifecycle,
        ),
        qualify_static_image_entropy_generator_lifecycle: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::QualifyStaticImageEntropyGeneratorLifecycle,
        ),
        deliver_static_image_entropy_initial_random: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::DeliverStaticImageEntropyInitialRandom,
        ),
        enter_static_image_process: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::EnterStaticImageProcess,
        ),
        restore_static_image_process_entry: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RestoreStaticImageProcessEntry,
        ),
        revoke_static_image_entropy_initial_random: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RevokeStaticImageEntropyInitialRandom,
        ),
        revoke_static_image_entropy_acquisition: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RevokeStaticImageEntropyAcquisition,
        ),
        revoke_static_image_entropy_handoff: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RevokeStaticImageEntropyHandoff,
        ),
        restore_static_image_prepared_process: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RestoreStaticImagePreparedProcess,
        ),
        revoke_static_image_stack_random_slot: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RevokeStaticImageStackRandomSlot,
        ),
        remove_static_image_stack_load_metadata: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RemoveStaticImageStackLoadMetadata,
        ),
        remove_static_image_stack_hardware_capabilities: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RemoveStaticImageStackHardwareCapabilities,
        ),
        remove_static_image_stack_platform: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RemoveStaticImageStackPlatform,
        ),
        remove_static_image_stack_clock_tick_rate: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RemoveStaticImageStackClockTickRate,
        ),
        remove_static_image_stack_secure_mode: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RemoveStaticImageStackSecureMode,
        ),
        remove_static_image_stack_credentials: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RemoveStaticImageStackCredentials,
        ),
        remove_static_image_stack_executable_name: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RemoveStaticImageStackExecutableName,
        ),
        remove_static_image_stack_program_headers: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RemoveStaticImageStackProgramHeaders,
        ),
        remove_static_image_stack_entry_fact: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RemoveStaticImageStackEntryFact,
        ),
        remove_static_image_stack_pagesz: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RemoveStaticImageStackPageSize,
        ),
        clear_static_image_stack_entry: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ClearStaticImageStackEntry,
        ),
        revoke_static_image_stack_leaf: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RevokeStaticImageStackLeaf,
        ),
        copy_static_image_payload: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::CopyStaticImagePayload,
        ),
        seal_static_image_permissions: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::SealStaticImagePermissions,
        ),
        attach_static_image_address_space: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AttachStaticImageAddressSpace,
        ),
        activate_static_image_address_space: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ActivateStaticImageAddressSpace,
        ),
        restore_static_image_address_space: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RestoreStaticImageAddressSpace,
        ),
        authorize_static_image_entry: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AuthorizeStaticImageEntry,
        ),
        authorize_static_image_stack_slot: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AuthorizeStaticImageStackSlot,
        ),
        revoke_static_image_stack_slot: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RevokeStaticImageStackSlot,
        ),
        revoke_static_image_entry: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RevokeStaticImageEntry,
        ),
        detach_static_image_address_space: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::DetachStaticImageAddressSpace,
        ),
        revoke_static_image: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RevokeStaticImage,
        ),
        begin_translation: undefined_provider_symbol(&mut object, ProviderSymbol::BeginTranslation),
        commit_translation: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::CommitTranslation,
        ),
        activate_translation: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ActivateTranslation,
        ),
        begin_translation_edit: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::BeginTranslationEdit,
        ),
        map_translation_frames: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::MapTranslationFrames,
        ),
        unmap_translation_mapping: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::UnmapTranslationMapping,
        ),
        commit_translation_edit: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::CommitTranslationEdit,
        ),
        enter_critical: undefined_provider_symbol(&mut object, ProviderSymbol::EnterCritical),
        restore_critical: undefined_provider_symbol(&mut object, ProviderSymbol::RestoreCritical),
        atomic_word_create: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AtomicWordCreate,
        ),
        atomic_word_compare_exchange: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AtomicWordCompareExchange,
        ),
        atomic_word_load: undefined_provider_symbol(&mut object, ProviderSymbol::AtomicWordLoad),
        send_local_notification: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::SendLocalNotification,
        ),
        wait_local_notification: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::WaitLocalNotification,
        ),
        complete_local_notification: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::CompleteLocalNotification,
        ),
        observe_monotonic_clock: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ObserveMonotonicClock,
        ),
        observe_pci_configuration_dword: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ObservePciConfigurationDword,
        ),
        update_pci_configuration_word: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::UpdatePciConfigurationWord,
        ),
        create_dma_buffer: undefined_provider_symbol(&mut object, ProviderSymbol::CreateDmaBuffer),
        bind_dma_buffer_reference: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::BindDmaBufferReference,
        ),
        store_dma_buffer_integer: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::StoreDmaBufferInteger,
        ),
        prepare_dma_buffer_for_device: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::PrepareDmaBufferForDevice,
        ),
        complete_dma_buffer_for_cpu: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::CompleteDmaBufferForCpu,
        ),
        load_dma_buffer_integer: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::LoadDmaBufferInteger,
        ),
        capture_entropy_sample: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::CaptureEntropySample,
        ),
        mix_entropy_pool_sample: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::MixEntropyPoolSample,
        ),
        initialize_entropy_generator: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::InitializeEntropyGenerator,
        ),
        extract_entropy_generator: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ExtractEntropyGenerator,
        ),
        deliver_initial_auxv_random: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::DeliverInitialAuxvRandom,
        ),
        reserve_initial_user_random: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ReserveInitialUserRandom,
        ),
        bind_reusable_user_random: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::BindReusableUserRandom,
        ),
        publish_pci_region_dma_reference: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::PublishPciRegionDmaReference,
        ),
        dma_virtqueue_control: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::DmaVirtqueueControl,
        ),
        dma_entropy_payload: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::DmaEntropyPayload,
        ),
        raw_entropy_sample: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RawEntropySample,
        ),
        entropy_pool_state: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::EntropyPoolState,
        ),
        entropy_generator_key: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::EntropyGeneratorKey,
        ),
        entropy_output: undefined_provider_symbol(&mut object, ProviderSymbol::EntropyOutput),
        entropy_user_random_service: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::EntropyUserRandomService,
        ),
        static_process_brk_reserve: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::StaticProcessBrkReserve,
        ),
        read_pci_region_register: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ReadPciRegionRegister,
        ),
        read_pci_region_register_eventually: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ReadPciRegionRegisterEventually,
        ),
        write_pci_region_register: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::WritePciRegionRegister,
        ),
        construct_deadline: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ConstructDeadline,
        ),
        arm_deadline: undefined_provider_symbol(&mut object, ProviderSymbol::ArmDeadline),
        wait_deadline: undefined_provider_symbol(&mut object, ProviderSymbol::WaitDeadline),
        complete_deadline: undefined_provider_symbol(&mut object, ProviderSymbol::CompleteDeadline),
        create_kernel_context: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::CreateKernelContext,
        ),
        transfer_kernel_context: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::TransferKernelContext,
        ),
        await_deadline_preemption: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::AwaitDeadlinePreemption,
        ),
        preempt_current_kernel_context: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::PreemptCurrentKernelContext,
        ),
        retire_kernel_context: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::RetireKernelContext,
        ),
        reclaim_kernel_context: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ReclaimKernelContext,
        ),
        prepare_user_address_space: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::PrepareUserAddressSpace,
        ),
        verify_user_address_space_mapping: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::VerifyUserAddressSpaceMapping,
        ),
        map_user_address_space: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::MapUserAddressSpace,
        ),
        store_user_mapping_byte: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::StoreUserMappingByte,
        ),
        install_user_mapping_entry_trampoline: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::InstallUserMappingEntryTrampoline,
        ),
        protect_user_mapping_read_only: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ProtectUserMappingReadOnly,
        ),
        load_user_mapping_byte: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::LoadUserMappingByte,
        ),
        verify_user_mapping_entry_trampoline: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::VerifyUserMappingEntryTrampoline,
        ),
        permit_user_mapping_execute: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::PermitUserMappingExecute,
        ),
        seal_user_mapping_immutable_data: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::SealUserMappingImmutableData,
        ),
        permit_user_mapping_writable_data: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::PermitUserMappingWritableData,
        ),
        reserve_anonymous_user_mapping: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ReserveAnonymousUserMapping,
        ),
        release_anonymous_user_mapping: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ReleaseAnonymousUserMapping,
        ),
        unmap_user_address_space: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::UnmapUserAddressSpace,
        ),
        prepare_user_probe: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::PrepareUserProbe,
        ),
        complete_user_backing: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::CompleteUserBacking,
        ),
        reclaim_user_address_space: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::ReclaimUserAddressSpace,
        ),
        enter_user_context: undefined_provider_symbol(
            &mut object,
            ProviderSymbol::EnterUserContext,
        ),
        user_syscall: undefined_provider_symbol(&mut object, ProviderSymbol::UserSyscall),
        kernel_thread_entry: None,
        completion_thread_entry: None,
        terminal_thread_entry: None,
        uart16550_write: undefined_provider_symbol(&mut object, ProviderSymbol::Uart16550Write),
        debug_break: undefined_provider_symbol(&mut object, ProviderSymbol::DebugBreak),
        interrupt_return: undefined_provider_symbol(&mut object, ProviderSymbol::InterruptReturn),
        fatal: undefined_provider_symbol(&mut object, ProviderSymbol::Fatal),
        bootstrap_storage: uses_bootstrap_storage
            .then(|| undefined_provider_symbol(&mut object, ProviderSymbol::BootstrapStorage)),
    };

    let mut kernel_thread = RootEncoder {
        role: RootRole::CooperativeKernelThreadHandler,
        ..RootEncoder::default()
    };
    encode_operations(
        &mut kernel_thread,
        &program.kernel_thread.handler.operations,
    )?;
    match program.kernel_thread.handler.disposition {
        CompilerSystemsDisposition::RetireKernelContextToCaller => {
            kernel_thread.retire_kernel_context()?;
        }
        CompilerSystemsDisposition::Fatal { .. } => kernel_thread.jump(ProviderSymbol::Fatal),
        CompilerSystemsDisposition::Resume => {
            return Err(CompileError::Tool(
                "x86 kernel-thread root cannot resume as an interrupt entry".into(),
            ));
        }
        CompilerSystemsDisposition::PreemptKernelContext => {
            return Err(CompileError::Tool(
                "x86 kernel-thread root cannot preempt itself as a deadline entry".into(),
            ));
        }
        CompilerSystemsDisposition::DispatchInitialUserSyscall { .. } => {
            return Err(CompileError::Tool(
                "x86 kernel-thread root cannot dispatch a user syscall".into(),
            ));
        }
    }
    let kernel_thread_entry = append_root(
        &mut object,
        text,
        X86_SYSTEMS_KERNEL_THREAD_ENTRY,
        &kernel_thread,
        symbols,
    )?;
    symbols.kernel_thread_entry = Some(kernel_thread_entry);

    let mut completion_thread = RootEncoder {
        role: RootRole::CompletionKernelThreadHandler,
        ..RootEncoder::default()
    };
    encode_operations(
        &mut completion_thread,
        &program.completion_thread.handler.operations,
    )?;
    match program.completion_thread.handler.disposition {
        CompilerSystemsDisposition::RetireKernelContextToCaller => {
            completion_thread.retire_kernel_context()?;
        }
        CompilerSystemsDisposition::Fatal { .. } => {
            completion_thread.jump(ProviderSymbol::Fatal);
        }
        CompilerSystemsDisposition::Resume
        | CompilerSystemsDisposition::PreemptKernelContext
        | CompilerSystemsDisposition::DispatchInitialUserSyscall { .. } => {
            return Err(CompileError::Tool(
                "x86 completion-thread root requires retirement disposition".into(),
            ));
        }
    }
    let completion_thread_entry = append_root(
        &mut object,
        text,
        X86_SYSTEMS_COMPLETION_THREAD_ENTRY,
        &completion_thread,
        symbols,
    )?;
    symbols.completion_thread_entry = Some(completion_thread_entry);

    let mut terminal_thread = RootEncoder {
        role: RootRole::TerminalKernelThreadHandler,
        ..RootEncoder::default()
    };
    encode_operations(
        &mut terminal_thread,
        &program.terminal_thread.handler.operations,
    )?;
    match program.terminal_thread.handler.disposition {
        CompilerSystemsDisposition::RetireKernelContextToCaller => {
            terminal_thread.retire_kernel_context()?;
        }
        CompilerSystemsDisposition::Fatal { .. } => terminal_thread.jump(ProviderSymbol::Fatal),
        CompilerSystemsDisposition::Resume => {
            return Err(CompileError::Tool(
                "x86 terminal-thread root cannot resume as an interrupt entry".into(),
            ));
        }
        CompilerSystemsDisposition::PreemptKernelContext => {
            return Err(CompileError::Tool(
                "x86 terminal-thread root cannot preempt a kernel context".into(),
            ));
        }
        CompilerSystemsDisposition::DispatchInitialUserSyscall { .. } => {
            return Err(CompileError::Tool(
                "x86 terminal-thread root cannot dispatch a user syscall".into(),
            ));
        }
    }
    let terminal_thread_entry = append_root(
        &mut object,
        text,
        X86_SYSTEMS_TERMINAL_THREAD_ENTRY,
        &terminal_thread,
        symbols,
    )?;
    symbols.terminal_thread_entry = Some(terminal_thread_entry);

    if program.user_syscall.handler.operations
        != [
            CompilerSystemsOperation::CopyInitialUnameToUser,
            CompilerSystemsOperation::CopyInitialTimespecFromUser,
        ]
        || program.user_syscall.handler.disposition
            != (CompilerSystemsDisposition::DispatchInitialUserSyscall {
                getpid_result: 1,
                uname_sysname: "Linux".into(),
                uname_nodename: "topal".into(),
                uname_release: "7.2.9-topal".into(),
                uname_version: "#1 Topal".into(),
                uname_machine: "x86_64".into(),
                uname_domainname: "(none)".into(),
                nanosleep_seconds: 0,
                nanosleep_nanoseconds: 0,
                nanosleep_remainder_candidate: 0,
                clock_gettime_result: 0,
                clock_getres_result: 0,
                write_descriptor: 1,
                write_maximum_bytes: 256,
                read_descriptor: 0,
                read_maximum_bytes: 256,
                initial_descriptor_count: 3,
                bad_descriptor_error: 9,
                too_many_open_files_error: 24,
                console_input_description: 1,
                console_output_description: 2,
                console_error_description: 3,
                invalid_argument_error: 22,
                fcntl_getfd_command: 1,
                fcntl_setfd_command: 2,
                fcntl_getfl_command: 3,
                close_on_exec_flag: 1,
                read_only_status: 0,
                write_only_status: 1,
                stat_layout_bytes: 144,
                console_device: 0,
                console_inode: 1,
                console_mode: 8576,
                console_special_device: 17408,
                console_block_size: 4096,
                terminal_layout_bytes: 36,
                terminal_get_attributes_request: 21505,
                terminal_control_flags: 2239,
                terminal_minimum_input: 1,
                not_a_terminal_error: 25,
                window_size_layout_bytes: 8,
                window_size_query_request: 21523,
                seek_maximum_whence: 4,
                illegal_seek_error: 29,
                brk_data_page_bytes: 4096,
                getppid_result: 0,
                gettid_result: 1,
                getuid_result: 0,
                geteuid_result: 0,
                getgid_result: 0,
                getegid_result: 0,
                arch_set_fs_result: 0,
                arch_get_fs_result: 0,
                set_tid_address_result: 1,
                set_robust_list_result: 0,
                getrlimit_result: 0,
                prlimit64_result: 0,
                rt_sigprocmask_result: 0,
                anonymous_page_bytes: 4096,
                madvise_result: 0,
                mprotect_result: 0,
                munmap_result: 0,
                exit_group_status: 0,
                unsupported_syscall_error: 38,
                sched_yield_result: 0,
            })
    {
        return Err(CompileError::Tool(
            "x86 initial user-syscall entry requires the sealed implemented-call dispatch and unsupported-syscall fallback"
                .into(),
        ));
    }

    let mut bootstrap = RootEncoder {
        role: RootRole::BootstrapWithLocalNotification,
        ..RootEncoder::default()
    };
    encode_operations(&mut bootstrap, &program.bootstrap.handler.operations)?;
    bootstrap.jump(ProviderSymbol::Fatal);
    append_root(
        &mut object,
        text,
        X86_SYSTEMS_KERNEL_ENTRY,
        &bootstrap,
        symbols,
    )?;

    let mut deadline = RootEncoder {
        role: RootRole::DeadlineHandler,
        ..RootEncoder::default()
    };
    for register in SAVED_REGISTERS {
        deadline.push(register);
    }
    encode_operations(
        &mut deadline,
        &program.deadline_notification.handler.operations,
    )?;
    match program.deadline_notification.handler.disposition {
        CompilerSystemsDisposition::Resume => {
            for register in SAVED_REGISTERS.into_iter().rev() {
                deadline.pop(register);
            }
            deadline.jump(ProviderSymbol::InterruptReturn);
        }
        CompilerSystemsDisposition::Fatal { .. } => deadline.jump(ProviderSymbol::Fatal),
        CompilerSystemsDisposition::PreemptKernelContext => {
            deadline.preempt_current_kernel_context()?;
            for register in SAVED_REGISTERS.into_iter().rev() {
                deadline.pop(register);
            }
            deadline.jump(ProviderSymbol::InterruptReturn);
        }
        CompilerSystemsDisposition::RetireKernelContextToCaller => {
            return Err(CompileError::Tool(
                "x86 deadline entry cannot retire a kernel context".into(),
            ));
        }
        CompilerSystemsDisposition::DispatchInitialUserSyscall { .. } => {
            return Err(CompileError::Tool(
                "x86 deadline entry cannot dispatch a user syscall".into(),
            ));
        }
    }
    append_root(
        &mut object,
        text,
        X86_SYSTEMS_DEADLINE_ENTRY,
        &deadline,
        symbols,
    )?;

    let mut local_notification = RootEncoder {
        role: RootRole::LocalNotificationHandler,
        ..RootEncoder::default()
    };
    for register in SAVED_REGISTERS {
        local_notification.push(register);
    }
    encode_operations(
        &mut local_notification,
        &program.local_notification.handler.operations,
    )?;
    match program.local_notification.handler.disposition {
        CompilerSystemsDisposition::Resume => {
            for register in SAVED_REGISTERS.into_iter().rev() {
                local_notification.pop(register);
            }
            local_notification.jump(ProviderSymbol::InterruptReturn);
        }
        CompilerSystemsDisposition::Fatal { .. } => {
            local_notification.jump(ProviderSymbol::Fatal);
        }
        CompilerSystemsDisposition::RetireKernelContextToCaller => {
            return Err(CompileError::Tool(
                "x86 local-notification entry cannot retire a kernel context".into(),
            ));
        }
        CompilerSystemsDisposition::PreemptKernelContext => {
            return Err(CompileError::Tool(
                "x86 local-notification entry cannot preempt a kernel context".into(),
            ));
        }
        CompilerSystemsDisposition::DispatchInitialUserSyscall { .. } => {
            return Err(CompileError::Tool(
                "x86 local-notification entry cannot dispatch a user syscall".into(),
            ));
        }
    }
    append_root(
        &mut object,
        text,
        X86_SYSTEMS_LOCAL_NOTIFICATION_ENTRY,
        &local_notification,
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
        CompilerSystemsDisposition::RetireKernelContextToCaller => {
            return Err(CompileError::Tool(
                "x86 debug-break entry cannot retire a kernel context".into(),
            ));
        }
        CompilerSystemsDisposition::PreemptKernelContext => {
            return Err(CompileError::Tool(
                "x86 debug-break entry cannot preempt a kernel context".into(),
            ));
        }
        CompilerSystemsDisposition::DispatchInitialUserSyscall { .. } => {
            return Err(CompileError::Tool(
                "x86 debug-break entry cannot dispatch a user syscall".into(),
            ));
        }
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
            CompilerSystemsOperation::CreateFrameAllocator { .. } => {
                encoder.create_frame_allocator()?;
            }
            CompilerSystemsOperation::AllocatePhysicalFrames { request, .. } => {
                encoder.allocate_physical_frames(request.frame_count, request.alignment_frames)?;
            }
            CompilerSystemsOperation::ReleasePhysicalFrames => {
                encoder.release_physical_frames()?;
            }
            CompilerSystemsOperation::PrepareStaticImageConstruction { .. } => {
                encoder.prepare_static_image_construction()?;
            }
            CompilerSystemsOperation::ZeroStaticImageConstructionPayload => {
                encoder.zero_static_image_payload()?;
            }
            CompilerSystemsOperation::ZeroStaticImageStackBacking => {
                encoder.zero_static_image_stack_backing()?;
            }
            CompilerSystemsOperation::PublishStaticImageStackLeaf => {
                encoder.publish_static_image_stack_leaf()?;
            }
            CompilerSystemsOperation::InitializeStaticImageStackEntry => {
                encoder.initialize_static_image_stack_entry()?;
            }
            CompilerSystemsOperation::AddStaticImageStackPageSize => {
                encoder.add_static_image_stack_pagesz()?;
            }
            CompilerSystemsOperation::AddStaticImageStackEntryFact => {
                encoder.add_static_image_stack_entry_fact()?;
            }
            CompilerSystemsOperation::AddStaticImageStackProgramHeaders => {
                encoder.add_static_image_stack_program_headers()?;
            }
            CompilerSystemsOperation::AddStaticImageStackExecutableName => {
                encoder.add_static_image_stack_executable_name()?;
            }
            CompilerSystemsOperation::AddStaticImageStackCredentials => {
                encoder.add_static_image_stack_credentials()?;
            }
            CompilerSystemsOperation::AddStaticImageStackSecureMode => {
                encoder.add_static_image_stack_secure_mode()?;
            }
            CompilerSystemsOperation::AddStaticImageStackClockTickRate => {
                encoder.add_static_image_stack_clock_tick_rate()?;
            }
            CompilerSystemsOperation::AddStaticImageStackPlatform => {
                encoder.add_static_image_stack_platform()?;
            }
            CompilerSystemsOperation::AddStaticImageStackHardwareCapabilities => {
                encoder.add_static_image_stack_hardware_capabilities()?;
            }
            CompilerSystemsOperation::AddStaticImageStackLoadMetadata => {
                encoder.add_static_image_stack_load_metadata()?;
            }
            CompilerSystemsOperation::AuthorizeStaticImageStackRandomSlot => {
                encoder.authorize_static_image_stack_random_slot()?;
            }
            CompilerSystemsOperation::RetainStaticImagePreparedProcess => {
                encoder.retain_static_image_prepared_process()?;
            }
            CompilerSystemsOperation::QualifyStaticImageRetainedDisjointWork => {
                encoder.qualify_static_image_retained_disjoint_work()?;
            }
            CompilerSystemsOperation::AuthorizeStaticImageEntropyHandoff => {
                encoder.authorize_static_image_entropy_handoff()?;
            }
            CompilerSystemsOperation::AuthorizeStaticImageEntropyAcquisition => {
                encoder.authorize_static_image_entropy_acquisition()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyDevice => {
                encoder.qualify_static_image_entropy_device()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyTransport => {
                encoder.qualify_static_image_entropy_transport()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyNegotiation => {
                encoder.qualify_static_image_entropy_negotiation()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyQueue => {
                encoder.qualify_static_image_entropy_queue()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyBusMaster => {
                encoder.qualify_static_image_entropy_bus_master()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyDmaBuffers => {
                encoder.qualify_static_image_entropy_dma_buffers()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropySplitRing => {
                encoder.qualify_static_image_entropy_split_ring()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyDmaPreparation => {
                encoder.qualify_static_image_entropy_dma_preparation()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyQueueAddresses => {
                encoder.qualify_static_image_entropy_queue_addresses()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyQueueActivation => {
                encoder.qualify_static_image_entropy_queue_activation()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyRequestCompletion => {
                encoder.qualify_static_image_entropy_request_completion()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropySampleCapture => {
                encoder.qualify_static_image_entropy_sample_capture()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropySampleQuality => {
                encoder.qualify_static_image_entropy_sample_quality()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyPoolLifecycle => {
                encoder.qualify_static_image_entropy_pool_lifecycle()?;
            }
            CompilerSystemsOperation::QualifyStaticImageEntropyGeneratorLifecycle => {
                encoder.qualify_static_image_entropy_generator_lifecycle()?;
            }
            CompilerSystemsOperation::DeliverStaticImageEntropyInitialRandom => {
                encoder.deliver_static_image_entropy_initial_random()?;
            }
            CompilerSystemsOperation::EnterStaticImageProcess => {
                encoder.enter_static_image_process()?;
            }
            CompilerSystemsOperation::RestoreStaticImageProcessEntry => {
                encoder.restore_static_image_process_entry()?;
            }
            CompilerSystemsOperation::RevokeStaticImageEntropyInitialRandom => {
                encoder.revoke_static_image_entropy_initial_random()?;
            }
            CompilerSystemsOperation::RevokeStaticImageEntropyAcquisition => {
                encoder.revoke_static_image_entropy_acquisition()?;
            }
            CompilerSystemsOperation::RevokeStaticImageEntropyHandoff => {
                encoder.revoke_static_image_entropy_handoff()?;
            }
            CompilerSystemsOperation::RestoreStaticImagePreparedProcess => {
                encoder.restore_static_image_prepared_process()?;
            }
            CompilerSystemsOperation::RevokeStaticImageStackRandomSlot => {
                encoder.revoke_static_image_stack_random_slot()?;
            }
            CompilerSystemsOperation::RemoveStaticImageStackLoadMetadata => {
                encoder.remove_static_image_stack_load_metadata()?;
            }
            CompilerSystemsOperation::RemoveStaticImageStackHardwareCapabilities => {
                encoder.remove_static_image_stack_hardware_capabilities()?;
            }
            CompilerSystemsOperation::RemoveStaticImageStackPlatform => {
                encoder.remove_static_image_stack_platform()?;
            }
            CompilerSystemsOperation::RemoveStaticImageStackClockTickRate => {
                encoder.remove_static_image_stack_clock_tick_rate()?;
            }
            CompilerSystemsOperation::RemoveStaticImageStackSecureMode => {
                encoder.remove_static_image_stack_secure_mode()?;
            }
            CompilerSystemsOperation::RemoveStaticImageStackCredentials => {
                encoder.remove_static_image_stack_credentials()?;
            }
            CompilerSystemsOperation::RemoveStaticImageStackExecutableName => {
                encoder.remove_static_image_stack_executable_name()?;
            }
            CompilerSystemsOperation::RemoveStaticImageStackProgramHeaders => {
                encoder.remove_static_image_stack_program_headers()?;
            }
            CompilerSystemsOperation::RemoveStaticImageStackEntryFact => {
                encoder.remove_static_image_stack_entry_fact()?;
            }
            CompilerSystemsOperation::RemoveStaticImageStackPageSize => {
                encoder.remove_static_image_stack_pagesz()?;
            }
            CompilerSystemsOperation::ClearStaticImageStackEntry => {
                encoder.clear_static_image_stack_entry()?;
            }
            CompilerSystemsOperation::RevokeStaticImageStackLeaf => {
                encoder.revoke_static_image_stack_leaf()?;
            }
            CompilerSystemsOperation::CopyStaticImageConstructionPayload => {
                encoder.copy_static_image_payload()?;
            }
            CompilerSystemsOperation::SealStaticImageConstructionPermissions => {
                encoder.seal_static_image_permissions()?;
            }
            CompilerSystemsOperation::AttachStaticImageAddressSpace => {
                encoder.attach_static_image_address_space()?;
            }
            CompilerSystemsOperation::ActivateStaticImageAddressSpace => {
                encoder.activate_static_image_address_space()?;
            }
            CompilerSystemsOperation::RestoreStaticImageAddressSpace => {
                encoder.restore_static_image_address_space()?;
            }
            CompilerSystemsOperation::AuthorizeStaticImageEntry => {
                encoder.authorize_static_image_entry()?;
            }
            CompilerSystemsOperation::AuthorizeStaticImageStackSlot => {
                encoder.authorize_static_image_stack_slot()?;
            }
            CompilerSystemsOperation::RevokeStaticImageStackSlot => {
                encoder.revoke_static_image_stack_slot()?;
            }
            CompilerSystemsOperation::RevokeStaticImageEntry => {
                encoder.revoke_static_image_entry()?;
            }
            CompilerSystemsOperation::DetachStaticImageAddressSpace => {
                encoder.detach_static_image_address_space()?;
            }
            CompilerSystemsOperation::ReclaimStaticImageConstructionHierarchy => {
                encoder.reclaim_static_image_hierarchy()?;
            }
            CompilerSystemsOperation::RevokeStaticImageConstructionMetadata => {
                encoder.revoke_static_image_construction()?;
            }
            CompilerSystemsOperation::ReclaimStaticImageConstructionPayload => {
                encoder.reclaim_static_image_payload()?;
            }
            CompilerSystemsOperation::MapKernelFrames { request, .. } => {
                encoder.map_kernel_frames(*request)?;
            }
            CompilerSystemsOperation::KernelMappingStoreByte {
                offset_bytes,
                value,
            } => encoder.store_kernel_mapping_byte(*offset_bytes, *value)?,
            CompilerSystemsOperation::KernelMappingLoadByteEquals {
                offset_bytes,
                expected,
                ..
            } => encoder.load_kernel_mapping_byte_equals(*offset_bytes, *expected)?,
            CompilerSystemsOperation::UnmapKernelFrames => encoder.unmap_kernel_frames()?,
            CompilerSystemsOperation::BeginTranslationUpdate { request, .. } => {
                encoder.begin_translation(*request)?;
            }
            CompilerSystemsOperation::CommitTranslationUpdate { .. } => {
                encoder.commit_translation()?;
            }
            CompilerSystemsOperation::ActivateTranslationSpace { .. } => {
                encoder.activate_translation()?;
            }
            CompilerSystemsOperation::BeginTranslationEdit { kind, .. } => {
                encoder.begin_translation_edit(*kind)?;
            }
            CompilerSystemsOperation::MapTranslationFrames { request, .. } => {
                encoder.map_translation_frames(*request)?;
            }
            CompilerSystemsOperation::UnmapTranslationMapping => {
                encoder.unmap_translation_mapping()?;
            }
            CompilerSystemsOperation::CommitTranslationEdit { kind, .. } => {
                encoder.commit_translation_edit(*kind)?;
            }
            CompilerSystemsOperation::EnterCritical { domain, .. } => {
                encoder.enter_critical(*domain)?;
            }
            CompilerSystemsOperation::RestoreCritical { domain } => {
                encoder.restore_critical(*domain)?;
            }
            CompilerSystemsOperation::ConsoleWrite { text } => encoder.console_write(text),
            CompilerSystemsOperation::DebugBreak => encoder.call(ProviderSymbol::DebugBreak),
            CompilerSystemsOperation::BootstrapAllocate { request } => {
                encoder.allocate_bootstrap_region(*request)?;
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
            CompilerSystemsOperation::AtomicWordCreate { request } => {
                encoder.create_atomic_word(*request)?;
            }
            CompilerSystemsOperation::AtomicCompareExchangeEquals {
                expected,
                desired,
                success_order,
                failure_order,
                ..
            } => encoder.compare_exchange_atomic_word(
                *expected,
                *desired,
                *success_order,
                *failure_order,
            )?,
            CompilerSystemsOperation::AtomicLoadEquals {
                order, expected, ..
            } => encoder.load_atomic_word_equals(*order, *expected)?,
            CompilerSystemsOperation::AtomicWordEnd => encoder.end_atomic_word()?,
            CompilerSystemsOperation::BootstrapRelease => encoder.release_bootstrap_region()?,
            CompilerSystemsOperation::SendLocalNotification => encoder.send_local_notification()?,
            CompilerSystemsOperation::WaitLocalNotification => encoder.wait_local_notification()?,
            CompilerSystemsOperation::CompleteLocalNotification => {
                encoder.complete_local_notification()?;
            }
            CompilerSystemsOperation::ObserveMonotonicClock => {
                encoder.observe_monotonic_clock()?;
            }
            CompilerSystemsOperation::ObservePciConfigurationDword { request } => {
                encoder.observe_pci_configuration_dword(*request)?;
            }
            CompilerSystemsOperation::UpdatePciConfigurationWord { request } => {
                encoder.update_pci_configuration_word(*request)?;
            }
            CompilerSystemsOperation::CreateDmaBuffer { request } => {
                encoder.create_dma_buffer(*request)?;
            }
            CompilerSystemsOperation::BindDmaBufferReference { request } => {
                encoder.bind_dma_buffer_reference(*request)?;
            }
            CompilerSystemsOperation::StoreDmaBufferInteger { request } => {
                encoder.store_dma_buffer_integer(*request)?;
            }
            CompilerSystemsOperation::PrepareDmaBufferForDevice { request } => {
                encoder.prepare_dma_buffer_for_device(*request)?;
            }
            CompilerSystemsOperation::CompleteDmaBufferForCpu { request } => {
                encoder.complete_dma_buffer_for_cpu(*request)?;
            }
            CompilerSystemsOperation::LoadDmaBufferIntegerEquals { request } => {
                encoder.load_dma_buffer_integer(*request)?;
            }
            CompilerSystemsOperation::CaptureEntropySample { request } => {
                encoder.capture_entropy_sample(*request)?;
            }
            CompilerSystemsOperation::AssessEntropySampleQuality { request } => {
                encoder.assess_entropy_sample_quality(*request)?;
            }
            CompilerSystemsOperation::CreateEntropyPool { request } => {
                encoder.create_entropy_pool(*request)?;
            }
            CompilerSystemsOperation::MixEntropyPoolSample { request } => {
                encoder.mix_entropy_pool_sample(*request)?;
            }
            CompilerSystemsOperation::CreditEntropyPoolContribution { request } => {
                encoder.credit_entropy_pool_contribution(*request)?;
            }
            CompilerSystemsOperation::InitializeEntropyGenerator { request } => {
                encoder.initialize_entropy_generator(*request)?;
            }
            CompilerSystemsOperation::ExtractEntropyGenerator { request } => {
                encoder.extract_entropy_generator(*request)?;
            }
            CompilerSystemsOperation::DeliverInitialAuxvRandom { request } => {
                encoder.deliver_initial_auxv_random(*request)?;
            }
            CompilerSystemsOperation::ReserveInitialUserRandom { request } => {
                encoder.reserve_initial_user_random(*request)?;
            }
            CompilerSystemsOperation::BindReusableUserRandom { request } => {
                encoder.bind_reusable_user_random(*request)?;
            }
            CompilerSystemsOperation::ExpandSmallUserRandomRequests { request } => {
                encoder.expand_small_user_random_requests(*request)?;
            }
            CompilerSystemsOperation::ExpandLargeUserRandomRequests { request } => {
                encoder.expand_large_user_random_requests(*request)?;
            }
            CompilerSystemsOperation::PublishPciRegionDmaReference { request } => {
                encoder.publish_pci_region_dma_reference(*request)?;
            }
            CompilerSystemsOperation::AccessPciRegionRegister { request } => {
                encoder.access_pci_region_register(*request)?;
            }
            CompilerSystemsOperation::ConstructDeadline {
                duration_nanoseconds,
            } => encoder.construct_deadline(*duration_nanoseconds)?,
            CompilerSystemsOperation::ArmDeadline => encoder.arm_deadline()?,
            CompilerSystemsOperation::WaitDeadline => encoder.wait_deadline()?,
            CompilerSystemsOperation::CompleteDeadline => encoder.complete_deadline()?,
            CompilerSystemsOperation::CreateKernelContext => encoder.create_kernel_context()?,
            CompilerSystemsOperation::CreateKernelCompletion => {
                encoder.create_kernel_completion()?;
            }
            CompilerSystemsOperation::CreateRunnableQueue { capacity } => {
                encoder.create_runnable_queue(*capacity)?;
            }
            CompilerSystemsOperation::EnqueueRunnableContext => {
                encoder.enqueue_runnable_context()?;
            }
            CompilerSystemsOperation::DequeueRunnableContext => {
                encoder.dequeue_runnable_context()?;
            }
            CompilerSystemsOperation::TransferKernelContextUntilDeadline => {
                encoder.transfer_kernel_context_until_deadline()?;
            }
            CompilerSystemsOperation::TransferKernelContextUntilCompletion => {
                encoder.transfer_kernel_context_until_completion()?;
            }
            CompilerSystemsOperation::TakePreemptedKernelContext => {
                encoder.take_preempted_kernel_context()?;
            }
            CompilerSystemsOperation::TransferKernelContext => encoder.transfer_kernel_context()?,
            CompilerSystemsOperation::AwaitDeadlinePreemption => {
                encoder.await_deadline_preemption()?;
            }
            CompilerSystemsOperation::AwaitKernelCompletion => {
                encoder.await_kernel_completion()?;
            }
            CompilerSystemsOperation::SignalBlockedKernelCompletion => {
                encoder.signal_blocked_kernel_completion()?;
            }
            CompilerSystemsOperation::ReclaimKernelContext => encoder.reclaim_kernel_context()?,
            CompilerSystemsOperation::ConsumeKernelCompletion => {
                encoder.consume_kernel_completion()?;
            }
            CompilerSystemsOperation::ConsumeEmptyRunnableQueue => {
                encoder.consume_empty_runnable_queue()?;
            }
            CompilerSystemsOperation::PrepareUserAddressSpace => {
                encoder.prepare_user_address_space()?;
            }
            CompilerSystemsOperation::VerifyUserAddressSpaceMapping => {
                encoder.verify_user_address_space_mapping()?;
            }
            CompilerSystemsOperation::MapUserAddressSpace => {
                encoder.map_user_address_space()?;
            }
            CompilerSystemsOperation::StoreUserMappingByte => {
                encoder.store_user_mapping_byte()?;
            }
            CompilerSystemsOperation::InstallUserMappingEntryTrampoline => {
                encoder.install_user_mapping_entry_trampoline()?;
            }
            CompilerSystemsOperation::ProtectUserMappingReadOnly => {
                encoder.protect_user_mapping_read_only()?;
            }
            CompilerSystemsOperation::LoadUserMappingByte => {
                encoder.load_user_mapping_byte()?;
            }
            CompilerSystemsOperation::VerifyUserMappingEntryTrampoline => {
                encoder.verify_user_mapping_entry_trampoline()?;
            }
            CompilerSystemsOperation::PermitUserMappingExecute => {
                encoder.permit_user_mapping_execute()?;
            }
            CompilerSystemsOperation::SealUserMappingImmutableData => {
                encoder.seal_user_mapping_immutable_data()?;
            }
            CompilerSystemsOperation::PermitUserMappingWritableData => {
                encoder.permit_user_mapping_writable_data()?;
            }
            CompilerSystemsOperation::ReserveAnonymousUserMapping => {
                encoder.reserve_anonymous_user_mapping()?;
            }
            CompilerSystemsOperation::ReleaseAnonymousUserMapping => {
                encoder.release_anonymous_user_mapping()?;
            }
            CompilerSystemsOperation::UnmapUserAddressSpace => {
                encoder.unmap_user_address_space()?;
            }
            CompilerSystemsOperation::PrepareUserProbe { process_identity } => {
                encoder.prepare_user_probe(*process_identity)?;
            }
            CompilerSystemsOperation::EnterUserContext => encoder.enter_user_context()?,
            CompilerSystemsOperation::CompleteUserExecution => {
                encoder.complete_user_execution()?;
            }
            CompilerSystemsOperation::ReclaimUserAddressSpace => {
                encoder.reclaim_user_address_space()?;
            }
            CompilerSystemsOperation::CopyInitialUnameToUser
            | CompilerSystemsOperation::CopyInitialTimespecFromUser => {
                return Err(CompileError::Tool(
                    "x86 user transfer lowers only inside the sealed user-syscall entry".into(),
                ));
            }
        }
    }
    encoder.complete()
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
) -> Result<SymbolId, CompileError> {
    let start = object.append_section_data(section, &encoded.bytes, 16);
    let root_symbol = object.add_symbol(Symbol {
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
            ProviderSymbol::AllocatePhysicalFrames => provider_symbols.allocate_physical_frames,
            ProviderSymbol::PrepareStaticImage => provider_symbols.prepare_static_image,
            ProviderSymbol::ZeroStaticImagePayload => provider_symbols.zero_static_image_payload,
            ProviderSymbol::ZeroStaticImageStackBacking => {
                provider_symbols.zero_static_image_stack_backing
            }
            ProviderSymbol::PublishStaticImageStackLeaf => {
                provider_symbols.publish_static_image_stack_leaf
            }
            ProviderSymbol::InitializeStaticImageStackEntry => {
                provider_symbols.initialize_static_image_stack_entry
            }
            ProviderSymbol::AddStaticImageStackPageSize => {
                provider_symbols.add_static_image_stack_pagesz
            }
            ProviderSymbol::AddStaticImageStackEntryFact => {
                provider_symbols.add_static_image_stack_entry_fact
            }
            ProviderSymbol::AddStaticImageStackProgramHeaders => {
                provider_symbols.add_static_image_stack_program_headers
            }
            ProviderSymbol::AddStaticImageStackExecutableName => {
                provider_symbols.add_static_image_stack_executable_name
            }
            ProviderSymbol::AddStaticImageStackCredentials => {
                provider_symbols.add_static_image_stack_credentials
            }
            ProviderSymbol::AddStaticImageStackSecureMode => {
                provider_symbols.add_static_image_stack_secure_mode
            }
            ProviderSymbol::AddStaticImageStackClockTickRate => {
                provider_symbols.add_static_image_stack_clock_tick_rate
            }
            ProviderSymbol::AddStaticImageStackPlatform => {
                provider_symbols.add_static_image_stack_platform
            }
            ProviderSymbol::AddStaticImageStackHardwareCapabilities => {
                provider_symbols.add_static_image_stack_hardware_capabilities
            }
            ProviderSymbol::AddStaticImageStackLoadMetadata => {
                provider_symbols.add_static_image_stack_load_metadata
            }
            ProviderSymbol::AuthorizeStaticImageStackRandomSlot => {
                provider_symbols.authorize_static_image_stack_random_slot
            }
            ProviderSymbol::RetainStaticImagePreparedProcess => {
                provider_symbols.retain_static_image_prepared_process
            }
            ProviderSymbol::QualifyStaticImageRetainedDisjointWork => {
                provider_symbols.qualify_static_image_retained_disjoint_work
            }
            ProviderSymbol::AuthorizeStaticImageEntropyHandoff => {
                provider_symbols.authorize_static_image_entropy_handoff
            }
            ProviderSymbol::AuthorizeStaticImageEntropyAcquisition => {
                provider_symbols.authorize_static_image_entropy_acquisition
            }
            ProviderSymbol::QualifyStaticImageEntropyDevice => {
                provider_symbols.qualify_static_image_entropy_device
            }
            ProviderSymbol::QualifyStaticImageEntropyTransport => {
                provider_symbols.qualify_static_image_entropy_transport
            }
            ProviderSymbol::QualifyStaticImageEntropyNegotiation => {
                provider_symbols.qualify_static_image_entropy_negotiation
            }
            ProviderSymbol::QualifyStaticImageEntropyQueue => {
                provider_symbols.qualify_static_image_entropy_queue
            }
            ProviderSymbol::QualifyStaticImageEntropyBusMaster => {
                provider_symbols.qualify_static_image_entropy_bus_master
            }
            ProviderSymbol::QualifyStaticImageEntropyDmaBuffers => {
                provider_symbols.qualify_static_image_entropy_dma_buffers
            }
            ProviderSymbol::QualifyStaticImageEntropySplitRing => {
                provider_symbols.qualify_static_image_entropy_split_ring
            }
            ProviderSymbol::QualifyStaticImageEntropyDmaPreparation => {
                provider_symbols.qualify_static_image_entropy_dma_preparation
            }
            ProviderSymbol::QualifyStaticImageEntropyQueueAddresses => {
                provider_symbols.qualify_static_image_entropy_queue_addresses
            }
            ProviderSymbol::QualifyStaticImageEntropyQueueActivation => {
                provider_symbols.qualify_static_image_entropy_queue_activation
            }
            ProviderSymbol::QualifyStaticImageEntropyRequestCompletion => {
                provider_symbols.qualify_static_image_entropy_request_completion
            }
            ProviderSymbol::QualifyStaticImageEntropySampleCapture => {
                provider_symbols.qualify_static_image_entropy_sample_capture
            }
            ProviderSymbol::QualifyStaticImageEntropySampleQuality => {
                provider_symbols.qualify_static_image_entropy_sample_quality
            }
            ProviderSymbol::QualifyStaticImageEntropyPoolLifecycle => {
                provider_symbols.qualify_static_image_entropy_pool_lifecycle
            }
            ProviderSymbol::QualifyStaticImageEntropyGeneratorLifecycle => {
                provider_symbols.qualify_static_image_entropy_generator_lifecycle
            }
            ProviderSymbol::DeliverStaticImageEntropyInitialRandom => {
                provider_symbols.deliver_static_image_entropy_initial_random
            }
            ProviderSymbol::EnterStaticImageProcess => provider_symbols.enter_static_image_process,
            ProviderSymbol::RestoreStaticImageProcessEntry => {
                provider_symbols.restore_static_image_process_entry
            }
            ProviderSymbol::RevokeStaticImageEntropyInitialRandom => {
                provider_symbols.revoke_static_image_entropy_initial_random
            }
            ProviderSymbol::RevokeStaticImageEntropyAcquisition => {
                provider_symbols.revoke_static_image_entropy_acquisition
            }
            ProviderSymbol::RevokeStaticImageEntropyHandoff => {
                provider_symbols.revoke_static_image_entropy_handoff
            }
            ProviderSymbol::RestoreStaticImagePreparedProcess => {
                provider_symbols.restore_static_image_prepared_process
            }
            ProviderSymbol::RevokeStaticImageStackRandomSlot => {
                provider_symbols.revoke_static_image_stack_random_slot
            }
            ProviderSymbol::RemoveStaticImageStackLoadMetadata => {
                provider_symbols.remove_static_image_stack_load_metadata
            }
            ProviderSymbol::RemoveStaticImageStackHardwareCapabilities => {
                provider_symbols.remove_static_image_stack_hardware_capabilities
            }
            ProviderSymbol::RemoveStaticImageStackPlatform => {
                provider_symbols.remove_static_image_stack_platform
            }
            ProviderSymbol::RemoveStaticImageStackClockTickRate => {
                provider_symbols.remove_static_image_stack_clock_tick_rate
            }
            ProviderSymbol::RemoveStaticImageStackSecureMode => {
                provider_symbols.remove_static_image_stack_secure_mode
            }
            ProviderSymbol::RemoveStaticImageStackCredentials => {
                provider_symbols.remove_static_image_stack_credentials
            }
            ProviderSymbol::RemoveStaticImageStackExecutableName => {
                provider_symbols.remove_static_image_stack_executable_name
            }
            ProviderSymbol::RemoveStaticImageStackProgramHeaders => {
                provider_symbols.remove_static_image_stack_program_headers
            }
            ProviderSymbol::RemoveStaticImageStackEntryFact => {
                provider_symbols.remove_static_image_stack_entry_fact
            }
            ProviderSymbol::RemoveStaticImageStackPageSize => {
                provider_symbols.remove_static_image_stack_pagesz
            }
            ProviderSymbol::ClearStaticImageStackEntry => {
                provider_symbols.clear_static_image_stack_entry
            }
            ProviderSymbol::RevokeStaticImageStackLeaf => {
                provider_symbols.revoke_static_image_stack_leaf
            }
            ProviderSymbol::CopyStaticImagePayload => provider_symbols.copy_static_image_payload,
            ProviderSymbol::SealStaticImagePermissions => {
                provider_symbols.seal_static_image_permissions
            }
            ProviderSymbol::AttachStaticImageAddressSpace => {
                provider_symbols.attach_static_image_address_space
            }
            ProviderSymbol::ActivateStaticImageAddressSpace => {
                provider_symbols.activate_static_image_address_space
            }
            ProviderSymbol::RestoreStaticImageAddressSpace => {
                provider_symbols.restore_static_image_address_space
            }
            ProviderSymbol::AuthorizeStaticImageEntry => {
                provider_symbols.authorize_static_image_entry
            }
            ProviderSymbol::AuthorizeStaticImageStackSlot => {
                provider_symbols.authorize_static_image_stack_slot
            }
            ProviderSymbol::RevokeStaticImageStackSlot => {
                provider_symbols.revoke_static_image_stack_slot
            }
            ProviderSymbol::RevokeStaticImageEntry => provider_symbols.revoke_static_image_entry,
            ProviderSymbol::DetachStaticImageAddressSpace => {
                provider_symbols.detach_static_image_address_space
            }
            ProviderSymbol::RevokeStaticImage => provider_symbols.revoke_static_image,
            ProviderSymbol::BeginTranslation => provider_symbols.begin_translation,
            ProviderSymbol::CommitTranslation => provider_symbols.commit_translation,
            ProviderSymbol::ActivateTranslation => provider_symbols.activate_translation,
            ProviderSymbol::BeginTranslationEdit => provider_symbols.begin_translation_edit,
            ProviderSymbol::MapTranslationFrames => provider_symbols.map_translation_frames,
            ProviderSymbol::UnmapTranslationMapping => provider_symbols.unmap_translation_mapping,
            ProviderSymbol::CommitTranslationEdit => provider_symbols.commit_translation_edit,
            ProviderSymbol::EnterCritical => provider_symbols.enter_critical,
            ProviderSymbol::RestoreCritical => provider_symbols.restore_critical,
            ProviderSymbol::AtomicWordCreate => provider_symbols.atomic_word_create,
            ProviderSymbol::AtomicWordCompareExchange => {
                provider_symbols.atomic_word_compare_exchange
            }
            ProviderSymbol::AtomicWordLoad => provider_symbols.atomic_word_load,
            ProviderSymbol::SendLocalNotification => provider_symbols.send_local_notification,
            ProviderSymbol::WaitLocalNotification => provider_symbols.wait_local_notification,
            ProviderSymbol::CompleteLocalNotification => {
                provider_symbols.complete_local_notification
            }
            ProviderSymbol::ObserveMonotonicClock => provider_symbols.observe_monotonic_clock,
            ProviderSymbol::ObservePciConfigurationDword => {
                provider_symbols.observe_pci_configuration_dword
            }
            ProviderSymbol::UpdatePciConfigurationWord => {
                provider_symbols.update_pci_configuration_word
            }
            ProviderSymbol::CreateDmaBuffer => provider_symbols.create_dma_buffer,
            ProviderSymbol::BindDmaBufferReference => provider_symbols.bind_dma_buffer_reference,
            ProviderSymbol::StoreDmaBufferInteger => provider_symbols.store_dma_buffer_integer,
            ProviderSymbol::PrepareDmaBufferForDevice => {
                provider_symbols.prepare_dma_buffer_for_device
            }
            ProviderSymbol::CompleteDmaBufferForCpu => provider_symbols.complete_dma_buffer_for_cpu,
            ProviderSymbol::LoadDmaBufferInteger => provider_symbols.load_dma_buffer_integer,
            ProviderSymbol::CaptureEntropySample => provider_symbols.capture_entropy_sample,
            ProviderSymbol::MixEntropyPoolSample => provider_symbols.mix_entropy_pool_sample,
            ProviderSymbol::InitializeEntropyGenerator => {
                provider_symbols.initialize_entropy_generator
            }
            ProviderSymbol::ExtractEntropyGenerator => provider_symbols.extract_entropy_generator,
            ProviderSymbol::DeliverInitialAuxvRandom => {
                provider_symbols.deliver_initial_auxv_random
            }
            ProviderSymbol::ReserveInitialUserRandom => {
                provider_symbols.reserve_initial_user_random
            }
            ProviderSymbol::BindReusableUserRandom => provider_symbols.bind_reusable_user_random,
            ProviderSymbol::PublishPciRegionDmaReference => {
                provider_symbols.publish_pci_region_dma_reference
            }
            ProviderSymbol::DmaVirtqueueControl => provider_symbols.dma_virtqueue_control,
            ProviderSymbol::DmaEntropyPayload => provider_symbols.dma_entropy_payload,
            ProviderSymbol::RawEntropySample => provider_symbols.raw_entropy_sample,
            ProviderSymbol::EntropyPoolState => provider_symbols.entropy_pool_state,
            ProviderSymbol::EntropyGeneratorKey => provider_symbols.entropy_generator_key,
            ProviderSymbol::EntropyOutput => provider_symbols.entropy_output,
            ProviderSymbol::EntropyUserRandomService => {
                provider_symbols.entropy_user_random_service
            }
            ProviderSymbol::StaticProcessBrkReserve => provider_symbols.static_process_brk_reserve,
            ProviderSymbol::ReadPciRegionRegister => provider_symbols.read_pci_region_register,
            ProviderSymbol::ReadPciRegionRegisterEventually => {
                provider_symbols.read_pci_region_register_eventually
            }
            ProviderSymbol::WritePciRegionRegister => provider_symbols.write_pci_region_register,
            ProviderSymbol::ConstructDeadline => provider_symbols.construct_deadline,
            ProviderSymbol::ArmDeadline => provider_symbols.arm_deadline,
            ProviderSymbol::WaitDeadline => provider_symbols.wait_deadline,
            ProviderSymbol::CompleteDeadline => provider_symbols.complete_deadline,
            ProviderSymbol::CreateKernelContext => provider_symbols.create_kernel_context,
            ProviderSymbol::TransferKernelContext => provider_symbols.transfer_kernel_context,
            ProviderSymbol::AwaitDeadlinePreemption => provider_symbols.await_deadline_preemption,
            ProviderSymbol::PreemptCurrentKernelContext => {
                provider_symbols.preempt_current_kernel_context
            }
            ProviderSymbol::RetireKernelContext => provider_symbols.retire_kernel_context,
            ProviderSymbol::ReclaimKernelContext => provider_symbols.reclaim_kernel_context,
            ProviderSymbol::PrepareUserAddressSpace => provider_symbols.prepare_user_address_space,
            ProviderSymbol::VerifyUserAddressSpaceMapping => {
                provider_symbols.verify_user_address_space_mapping
            }
            ProviderSymbol::MapUserAddressSpace => provider_symbols.map_user_address_space,
            ProviderSymbol::StoreUserMappingByte => provider_symbols.store_user_mapping_byte,
            ProviderSymbol::InstallUserMappingEntryTrampoline => {
                provider_symbols.install_user_mapping_entry_trampoline
            }
            ProviderSymbol::ProtectUserMappingReadOnly => {
                provider_symbols.protect_user_mapping_read_only
            }
            ProviderSymbol::LoadUserMappingByte => provider_symbols.load_user_mapping_byte,
            ProviderSymbol::VerifyUserMappingEntryTrampoline => {
                provider_symbols.verify_user_mapping_entry_trampoline
            }
            ProviderSymbol::PermitUserMappingExecute => {
                provider_symbols.permit_user_mapping_execute
            }
            ProviderSymbol::SealUserMappingImmutableData => {
                provider_symbols.seal_user_mapping_immutable_data
            }
            ProviderSymbol::PermitUserMappingWritableData => {
                provider_symbols.permit_user_mapping_writable_data
            }
            ProviderSymbol::ReserveAnonymousUserMapping => {
                provider_symbols.reserve_anonymous_user_mapping
            }
            ProviderSymbol::ReleaseAnonymousUserMapping => {
                provider_symbols.release_anonymous_user_mapping
            }
            ProviderSymbol::UnmapUserAddressSpace => provider_symbols.unmap_user_address_space,
            ProviderSymbol::PrepareUserProbe => provider_symbols.prepare_user_probe,
            ProviderSymbol::EnterUserContext => provider_symbols.enter_user_context,
            ProviderSymbol::CompleteUserBacking => provider_symbols.complete_user_backing,
            ProviderSymbol::ReclaimUserAddressSpace => provider_symbols.reclaim_user_address_space,
            ProviderSymbol::UserSyscall => provider_symbols.user_syscall,
            ProviderSymbol::KernelThreadEntry => {
                provider_symbols.kernel_thread_entry.ok_or_else(|| {
                    CompileError::Tool(
                        "x86 bootstrap root references an unplaced kernel-thread entry".into(),
                    )
                })?
            }
            ProviderSymbol::CompletionThreadEntry => {
                provider_symbols.completion_thread_entry.ok_or_else(|| {
                    CompileError::Tool(
                        "x86 bootstrap root references an unplaced completion-thread entry".into(),
                    )
                })?
            }
            ProviderSymbol::TerminalThreadEntry => {
                provider_symbols.terminal_thread_entry.ok_or_else(|| {
                    CompileError::Tool(
                        "x86 bootstrap root references an unplaced terminal-thread entry".into(),
                    )
                })?
            }
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
    Ok(root_symbol)
}

#[allow(clippy::too_many_lines)] // Exhaustive transition detail remains beside its canonical trace spelling.
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
                CompilerSystemsTransition::ObservePciConfigurationDword {
                    transaction_identity,
                    request,
                } => format!(
                    "{identity}:transaction={transaction_identity}:segment={}:bus={}:device={}:function={}:offset={}:expected={:#010x}",
                    request.segment,
                    request.bus,
                    request.device,
                    request.function,
                    request.register_offset_bytes,
                    request.expected_value
                ),
                CompilerSystemsTransition::UpdatePciConfigurationWord {
                    transaction_identity,
                    request,
                } => format!(
                    "{identity}:transaction={transaction_identity}:segment={}:bus={}:device={}:function={}:offset={}:mask={:#06x}:expected={:#06x}:desired={:#06x}",
                    request.segment,
                    request.bus,
                    request.device,
                    request.function,
                    request.register_offset_bytes,
                    request.mask,
                    request.expected_masked_value,
                    request.desired_masked_value
                ),
                CompilerSystemsTransition::CreateDmaBuffer { request } => {
                    let direction = match request.direction {
                        CompilerDmaDirection::Bidirectional => "bidirectional",
                        CompilerDmaDirection::DeviceToMemory => "device-to-memory",
                    };
                    let role = match request.role {
                        CompilerDmaBufferRole::VirtqueueControl => "virtqueue-control",
                        CompilerDmaBufferRole::EntropyPayload => "entropy-payload",
                    };
                    format!(
                        "{identity}:buffer={}:device={}:domain={}:bytes={}:alignment={}:direction={direction}:visibility=provider-managed:role={role}",
                        request.buffer_identity,
                        request.device_identity,
                        request.translation_domain_identity,
                        request.byte_count,
                        request.alignment_bytes,
                    )
                }
                CompilerSystemsTransition::BindDmaBufferReference { request } => format!(
                    "{identity}:destination-buffer={}:destination-offset={}:referenced-buffer={}:referenced-offset={}:width={}",
                    request.destination_buffer_identity,
                    request.destination_offset_bytes,
                    request.referenced_buffer_identity,
                    request.referenced_offset_bytes,
                    request.width_bytes,
                ),
                CompilerSystemsTransition::StoreDmaBufferInteger { request } => format!(
                    "{identity}:buffer={}:offset={}:width={}:value={}",
                    request.buffer_identity,
                    request.offset_bytes,
                    request.width_bytes,
                    request.value,
                ),
                CompilerSystemsTransition::PrepareDmaBufferForDevice { request } => {
                    let direction = match request.direction {
                        CompilerDmaDirection::Bidirectional => "bidirectional",
                        CompilerDmaDirection::DeviceToMemory => "device-to-memory",
                    };
                    let role = match request.role {
                        CompilerDmaBufferRole::VirtqueueControl => "virtqueue-control",
                        CompilerDmaBufferRole::EntropyPayload => "entropy-payload",
                    };
                    format!(
                        "{identity}:buffer={}:device={}:domain={}:direction={direction}:visibility=provider-managed:role={role}",
                        request.buffer_identity,
                        request.device_identity,
                        request.translation_domain_identity,
                    )
                }
                CompilerSystemsTransition::CompleteDmaBufferForCpu { request } => {
                    let direction = match request.direction {
                        CompilerDmaDirection::Bidirectional => "bidirectional",
                        CompilerDmaDirection::DeviceToMemory => "device-to-memory",
                    };
                    let role = match request.role {
                        CompilerDmaBufferRole::VirtqueueControl => "virtqueue-control",
                        CompilerDmaBufferRole::EntropyPayload => "entropy-payload",
                    };
                    format!(
                        "{identity}:buffer={}:device={}:domain={}:direction={direction}:visibility=provider-managed:role={role}",
                        request.buffer_identity,
                        request.device_identity,
                        request.translation_domain_identity,
                    )
                }
                CompilerSystemsTransition::LoadDmaBufferIntegerEquals { request } => format!(
                    "{identity}:buffer={}:offset={}:width={}:expected={}",
                    request.buffer_identity,
                    request.offset_bytes,
                    request.width_bytes,
                    request.expected_value,
                ),
                CompilerSystemsTransition::CaptureEntropySample { request } => format!(
                    "{identity}:sample={}:buffer={}:offset={}:bytes={}:device={}:domain={}:direction=device-to-memory:visibility=provider-managed:role=entropy-payload:credited-entropy-bits={}",
                    request.sample_identity,
                    request.buffer_identity,
                    request.offset_bytes,
                    request.byte_count,
                    request.device_identity,
                    request.translation_domain_identity,
                    request.credited_entropy_bits,
                ),
                CompilerSystemsTransition::AssessEntropySampleQuality { request } => format!(
                    "{identity}:sample={}:source={}:trust-evidence={}:bytes={}:quality-parts-per-1024={}:credited-entropy-bits={}",
                    request.sample_identity,
                    request.source_identity,
                    request.trust_evidence_identity,
                    request.byte_count,
                    request.quality_parts_per_1024,
                    request.credited_entropy_bits,
                ),
                CompilerSystemsTransition::CreateEntropyPool { request } => format!(
                    "{identity}:pool={}:credit-capacity-bits={}:initial-credited-entropy-bits={}",
                    request.pool_identity,
                    request.credit_capacity_bits,
                    request.initial_credited_entropy_bits,
                ),
                CompilerSystemsTransition::MixEntropyPoolSample { request } => format!(
                    "{identity}:pool={}:sample={}:conditioner={}:contribution={}:bytes={}:contribution-entropy-bits={}",
                    request.pool_identity,
                    request.sample_identity,
                    request.conditioner_identity,
                    request.contribution_identity,
                    request.byte_count,
                    request.contribution_entropy_bits,
                ),
                CompilerSystemsTransition::CreditEntropyPoolContribution { request } => format!(
                    "{identity}:pool={}:contribution={}:conditioner-origin={}:prior-credited-entropy-bits={}:contribution-entropy-bits={}:credit-capacity-bits={}:resulting-credited-entropy-bits={}",
                    request.pool_identity,
                    request.contribution_identity,
                    request.conditioner_origin_identity,
                    request.prior_credited_entropy_bits,
                    request.contribution_entropy_bits,
                    request.credit_capacity_bits,
                    request.resulting_credited_entropy_bits,
                ),
                CompilerSystemsTransition::InitializeEntropyGenerator { request } => format!(
                    "{identity}:pool={}:generator={}:algorithm={}:generation={}:key-bytes={}:required-entropy-bits={}:observed-pool-entropy-bits={}",
                    request.pool_identity,
                    request.generator_identity,
                    request.algorithm_identity,
                    request.generation,
                    request.key_byte_count,
                    request.required_entropy_bits,
                    request.observed_pool_entropy_bits,
                ),
                CompilerSystemsTransition::ExtractEntropyGenerator { request } => format!(
                    "{identity}:generator={}:algorithm={}:prior-generation={}:resulting-generation={}:output={}:block-bytes={}:output-offset={}:output-bytes={}:counter={}:nonce={}",
                    request.generator_identity,
                    request.algorithm_identity,
                    request.prior_generation,
                    request.resulting_generation,
                    request.output_identity,
                    request.block_byte_count,
                    request.output_offset_bytes,
                    request.output_byte_count,
                    request.counter,
                    request.nonce_identity,
                ),
                CompilerSystemsTransition::DeliverInitialAuxvRandom { request } => format!(
                    "{identity}:output={}:source-bytes={}:selected-offset={}:delivered-bytes={}:discarded-offset={}:discarded-bytes={}:address-space={}:interface={}:auxiliary-tag={}:single-use={}",
                    request.output_identity,
                    request.source_byte_count,
                    request.selected_offset_bytes,
                    request.delivered_byte_count,
                    request.discarded_offset_bytes,
                    request.discarded_byte_count,
                    request.address_space_identity,
                    request.interface_identity,
                    request.auxiliary_tag,
                    request.single_use,
                ),
                CompilerSystemsTransition::ReserveInitialUserRandom { request } => format!(
                    "{identity}:output={}:source-bytes={}:selected-offset={}:maximum-delivery-bytes={}:discarded-offset={}:discarded-bytes={}:process={}:interface={}:service={}:supported-flags={}:incompatible-flags={}:single-use={}",
                    request.output_identity,
                    request.source_byte_count,
                    request.selected_offset_bytes,
                    request.maximum_delivery_byte_count,
                    request.discarded_offset_bytes,
                    request.discarded_byte_count,
                    request.process_identity,
                    request.interface_identity,
                    request.service_identity,
                    request.supported_flag_mask,
                    request.incompatible_flag_mask,
                    request.single_use,
                ),
                CompilerSystemsTransition::BindReusableUserRandom { request } => format!(
                    "{identity}:generator={}:generation={}:initial-output={}:block-bytes={}:output-bytes={}:maximum-delivery-bytes={}:process={}:interface={}:service={}:supported-flags={}:incompatible-flags={}:reserved-first-output={}:reusable={}",
                    request.generator_identity,
                    request.generation,
                    request.initial_output_identity,
                    request.block_byte_count,
                    request.output_byte_count,
                    request.maximum_delivery_byte_count,
                    request.process_identity,
                    request.interface_identity,
                    request.service_identity,
                    request.supported_flag_mask,
                    request.incompatible_flag_mask,
                    request.reserved_first_output,
                    request.reusable,
                ),
                CompilerSystemsTransition::ExpandSmallUserRandomRequests { request } => format!(
                    "{identity}:process={}:interface={}:service={}:algorithm={}:output-bytes={}:maximum-output-count={}:maximum-delivery-bytes={}:complete-request={}",
                    request.process_identity,
                    request.interface_identity,
                    request.service_identity,
                    request.algorithm_identity,
                    request.output_byte_count,
                    request.maximum_output_count,
                    request.maximum_delivery_byte_count,
                    request.complete_request,
                ),
                CompilerSystemsTransition::ExpandLargeUserRandomRequests { request } => format!(
                    "{identity}:process={}:interface={}:service={}:algorithm={}:output-bytes={}:maximum-output-count={}:maximum-delivery-bytes={}:complete-request={}",
                    request.process_identity,
                    request.interface_identity,
                    request.service_identity,
                    request.algorithm_identity,
                    request.output_byte_count,
                    request.maximum_output_count,
                    request.maximum_delivery_byte_count,
                    request.complete_request,
                ),
                CompilerSystemsTransition::PublishPciRegionDmaReference {
                    transaction_identity,
                    request,
                } => format!(
                    "{identity}:transaction={transaction_identity}:segment={}:bus={}:device={}:function={}:access-capability-offset={}:bar={}:region-offset={}:region-length={}:register-offset={}:width={}:buffer={}:buffer-offset={}:device-identity={}:domain={}",
                    request.segment,
                    request.bus,
                    request.device,
                    request.function,
                    request.access_capability_offset_bytes,
                    request.bar,
                    request.region_offset_bytes,
                    request.region_length_bytes,
                    request.register_offset_bytes,
                    request.width_bytes,
                    request.buffer_identity,
                    request.buffer_offset_bytes,
                    request.device_identity,
                    request.translation_domain_identity,
                ),
                CompilerSystemsTransition::AccessPciRegionRegister {
                    transaction_identity,
                    request,
                } => format!(
                    "{}:transaction={transaction_identity}:segment={}:bus={}:device={}:function={}:access-capability-offset={}:bar={}:region-offset={}:region-length={}:register-offset={}:width={}:value={:#010x}",
                    identity,
                    request.segment,
                    request.bus,
                    request.device,
                    request.function,
                    request.access_capability_offset_bytes,
                    request.bar,
                    request.region_offset_bytes,
                    request.region_length_bytes,
                    request.register_offset_bytes,
                    request.width_bytes,
                    request.value
                ),
                CompilerSystemsTransition::Fatal { message } => {
                    format!("{identity}:sha256:{}", sha256(message.as_bytes()))
                }
                CompilerSystemsTransition::AllocateBootstrapRegion { request } => format!(
                    "{identity}:bytes={}:alignment={}:placement={}",
                    request.byte_count,
                    request.alignment_bytes,
                    request.placement.semantic_identity()
                ),
                CompilerSystemsTransition::AllocatePhysicalFrames { request } => format!(
                    "{identity}:frames={}:alignment={}",
                    request.frame_count, request.alignment_frames
                ),
                CompilerSystemsTransition::MapKernelFrames { .. } => {
                    format!("{identity}:rights=read-write:execution=denied:memory-kind=normal")
                }
                CompilerSystemsTransition::MapTranslationFrames { .. } => format!(
                    "{identity}:rights=read-write:execution=denied:memory-kind=normal:placement=provider-selected"
                ),
                CompilerSystemsTransition::BeginTranslationUpdate { .. } => {
                    format!(
                        "{identity}:template=bootstrap-equivalent:page-policy=provider-selected"
                    )
                }
                CompilerSystemsTransition::BeginTranslationEdit { kind }
                | CompilerSystemsTransition::CommitTranslationEdit { kind } => format!(
                    "{identity}:kind={}",
                    match kind {
                        CompilerTranslationEditKind::Map => "map",
                        CompilerTranslationEditKind::Unmap => "unmap",
                    }
                ),
                CompilerSystemsTransition::EnterCritical {
                    domain,
                    nesting_identity,
                }
                | CompilerSystemsTransition::RestoreCritical {
                    domain,
                    nesting_identity,
                } => format!(
                    "{identity}:domain={}:nesting={nesting_identity}",
                    match domain {
                        topal_language::CompilerCriticalDomain::LocalMaskableInterrupts => {
                            "local-maskable-interrupts"
                        }
                    }
                ),
                CompilerSystemsTransition::CreateAtomicWord { request } => format!(
                    "{identity}:offset={}:initial={}:domain=cpu-shared",
                    request.offset_bytes, request.initial_value
                ),
                CompilerSystemsTransition::CompareExchangeAtomicWord {
                    expected,
                    desired,
                    observed,
                    exchanged,
                    success_order,
                    failure_order,
                } => format!(
                    "{identity}:expected={expected}:desired={desired}:observed={observed}:exchanged={exchanged}:success-order={success_order:?}:failure-order={failure_order:?}"
                ),
                CompilerSystemsTransition::LoadAtomicWord { value, order } => {
                    format!("{identity}:value={value}:order={order:?}")
                }
                CompilerSystemsTransition::ConstructDeadline {
                    source_observation_identity,
                    duration_nanoseconds,
                } => format!(
                    "{identity}:source-observation={source_observation_identity}:duration-nanoseconds={duration_nanoseconds}"
                ),
                CompilerSystemsTransition::CreateKernelContext { context_identity }
                | CompilerSystemsTransition::EnterKernelThread { context_identity }
                | CompilerSystemsTransition::ReclaimKernelContext { context_identity } => {
                    format!("{identity}:context={context_identity}")
                }
                CompilerSystemsTransition::CreateKernelCompletion {
                    completion_identity,
                }
                | CompilerSystemsTransition::ConsumeKernelCompletion {
                    completion_identity,
                } => format!("{identity}:completion={completion_identity}"),
                CompilerSystemsTransition::CreateRunnableQueue { capacity } => {
                    format!("{identity}:capacity={capacity}")
                }
                CompilerSystemsTransition::EnqueueRunnableContext { context_identity }
                | CompilerSystemsTransition::DequeueRunnableContext { context_identity } => {
                    format!("{identity}:context={context_identity}")
                }
                CompilerSystemsTransition::TransferKernelContext {
                    context_identity,
                    caller_identity,
                }
                | CompilerSystemsTransition::RetireKernelContextToCaller {
                    context_identity,
                    caller_identity,
                } => format!(
                    "{identity}:context={context_identity}:caller={caller_identity}"
                ),
                CompilerSystemsTransition::TransferKernelContextUntilDeadline {
                    context_identity,
                    caller_identity,
                    event_identity,
                }
                | CompilerSystemsTransition::PreemptKernelContext {
                    context_identity,
                    caller_identity,
                    event_identity,
                }
                | CompilerSystemsTransition::RestorePreemptedKernelContext {
                    context_identity,
                    caller_identity,
                    event_identity,
                } => format!(
                    "{identity}:context={context_identity}:caller={caller_identity}:event={event_identity}"
                ),
                CompilerSystemsTransition::TransferKernelContextUntilCompletion {
                    context_identity,
                    caller_identity,
                    completion_identity,
                }
                | CompilerSystemsTransition::BlockKernelContext {
                    context_identity,
                    caller_identity,
                    completion_identity,
                }
                | CompilerSystemsTransition::SignalBlockedKernelCompletion {
                    context_identity,
                    caller_identity,
                    completion_identity,
                }
                | CompilerSystemsTransition::RestoreBlockedKernelContext {
                    context_identity,
                    caller_identity,
                    completion_identity,
                } => format!(
                    "{identity}:context={context_identity}:caller={caller_identity}:completion={completion_identity}"
                ),
                CompilerSystemsTransition::AwaitDeadlinePreemption { context_identity } => {
                    format!("{identity}:context={context_identity}")
                }
                CompilerSystemsTransition::AwaitKernelCompletion {
                    context_identity,
                    completion_identity,
                } => format!(
                    "{identity}:context={context_identity}:completion={completion_identity}"
                ),
                CompilerSystemsTransition::PrepareUserAddressSpace {
                    address_space_identity,
                }
                | CompilerSystemsTransition::VerifyUserAddressSpaceMapping {
                    address_space_identity,
                }
                | CompilerSystemsTransition::ReclaimUserAddressSpace {
                    address_space_identity,
                } => format!("{identity}:address-space={address_space_identity}"),
                CompilerSystemsTransition::MapUserAddressSpace {
                    address_space_identity,
                    mapping_identity,
                }
                | CompilerSystemsTransition::UnmapUserAddressSpace {
                    address_space_identity,
                    mapping_identity,
                } => format!(
                    "{identity}:address-space={address_space_identity}:mapping={mapping_identity}"
                ),
                CompilerSystemsTransition::ReserveAnonymousUserMapping {
                    address_space_identity,
                    reservation_identity,
                }
                | CompilerSystemsTransition::ReleaseAnonymousUserMapping {
                    address_space_identity,
                    reservation_identity,
                } => format!(
                    "{identity}:address-space={address_space_identity}:reservation={reservation_identity}"
                ),
                CompilerSystemsTransition::StoreUserMappingByte { mapping_identity }
                | CompilerSystemsTransition::InstallUserMappingEntryTrampoline {
                    mapping_identity,
                }
                | CompilerSystemsTransition::ProtectUserMappingReadOnly { mapping_identity }
                | CompilerSystemsTransition::LoadUserMappingByte { mapping_identity }
                | CompilerSystemsTransition::VerifyUserMappingEntryTrampoline {
                    mapping_identity,
                } => {
                    format!("{identity}:mapping={mapping_identity}")
                }
                CompilerSystemsTransition::PermitUserMappingExecute { mapping_identity }
                | CompilerSystemsTransition::SealUserMappingImmutableData { mapping_identity }
                | CompilerSystemsTransition::PermitUserMappingWritableData {
                    mapping_identity,
                } => {
                    format!("{identity}:mapping={mapping_identity}")
                }
                CompilerSystemsTransition::PrepareUserProbe {
                    process_identity,
                    probe_identity,
                } => format!(
                    "{identity}:process={process_identity}:probe={probe_identity}"
                ),
                CompilerSystemsTransition::EnterUserExecution {
                    process_identity,
                    kernel_continuation_identity,
                } => format!(
                    "{identity}:process={process_identity}:continuation={kernel_continuation_identity}"
                ),
                CompilerSystemsTransition::EnterUserSyscall {
                    entry_identity,
                    number,
                    arguments,
                } => format!(
                    "{identity}:entry={entry_identity}:number={number}:arguments={arguments:?}"
                ),
                CompilerSystemsTransition::BeginUserTransfer {
                    entry_identity,
                    transfer_identity,
                    candidate,
                    extent,
                } => format!(
                    "{identity}:entry={entry_identity}:transfer={transfer_identity}:candidate={candidate}:extent={extent}"
                ),
                CompilerSystemsTransition::CopyToUserComplete {
                    transfer_identity,
                    copied,
                } => format!(
                    "{identity}:transfer={transfer_identity}:copied={copied}"
                ),
                CompilerSystemsTransition::CopyToUserFault {
                    transfer_identity,
                    copied_prefix,
                } => format!(
                    "{identity}:transfer={transfer_identity}:copied-prefix={copied_prefix}"
                ),
                CompilerSystemsTransition::CopyFromUserComplete {
                    transfer_identity,
                    copied,
                    seconds,
                    nanoseconds,
                } => format!(
                    "{identity}:transfer={transfer_identity}:copied={copied}:seconds={seconds}:nanoseconds={nanoseconds}"
                ),
                CompilerSystemsTransition::CopyFromUserFault {
                    transfer_identity,
                    copied_prefix,
                } => format!(
                    "{identity}:transfer={transfer_identity}:copied-prefix={copied_prefix}"
                ),
                CompilerSystemsTransition::EndUserTransfer {
                    entry_identity,
                    transfer_identity,
                } => format!(
                    "{identity}:entry={entry_identity}:transfer={transfer_identity}"
                ),
                CompilerSystemsTransition::ReturnToUser {
                    entry_identity,
                    result,
                } => format!("{identity}:entry={entry_identity}:result={result}"),
                CompilerSystemsTransition::TerminateUser {
                    entry_identity,
                    status,
                } => format!("{identity}:entry={entry_identity}:status={status}"),
                CompilerSystemsTransition::ResumeUserKernelContinuation {
                    kernel_continuation_identity,
                    status,
                } => format!(
                    "{identity}:continuation={kernel_continuation_identity}:status={status}"
                ),
                CompilerSystemsTransition::CompleteUserExecution { process_identity } => {
                    format!("{identity}:process={process_identity}")
                }
                CompilerSystemsTransition::TakePreemptedKernelContext {
                    context_identity,
                    event_identity,
                } => format!(
                    "{identity}:context={context_identity}:event={event_identity}"
                ),
                CompilerSystemsTransition::ResumeKernelContextCaller { caller_identity }
                | CompilerSystemsTransition::ResumePreemptionDispatcher { caller_identity }
                | CompilerSystemsTransition::ResumeCompletionDispatcher { caller_identity } => {
                    format!("{identity}:caller={caller_identity}")
                }
                CompilerSystemsTransition::StoreBootstrapByte {
                    offset_bytes,
                    value,
                }
                | CompilerSystemsTransition::LoadBootstrapByte {
                    offset_bytes,
                    value,
                }
                | CompilerSystemsTransition::StoreKernelMappingByte {
                    offset_bytes,
                    value,
                }
                | CompilerSystemsTransition::LoadKernelMappingByte {
                    offset_bytes,
                    value,
                } => format!("{identity}:offset={offset_bytes}:value={value}"),
                CompilerSystemsTransition::EnterBootstrap
                | CompilerSystemsTransition::DescribeBootMemory
                | CompilerSystemsTransition::CreateFrameAllocator
                | CompilerSystemsTransition::ReleasePhysicalFrames
                | CompilerSystemsTransition::PrepareStaticImageConstruction
                | CompilerSystemsTransition::ZeroStaticImageConstructionPayload
                | CompilerSystemsTransition::ZeroStaticImageStackBacking
                | CompilerSystemsTransition::PublishStaticImageStackLeaf
                | CompilerSystemsTransition::InitializeStaticImageStackEntry
                | CompilerSystemsTransition::AddStaticImageStackPageSize
                | CompilerSystemsTransition::AddStaticImageStackEntryFact
                | CompilerSystemsTransition::AddStaticImageStackProgramHeaders
                | CompilerSystemsTransition::AddStaticImageStackExecutableName
                | CompilerSystemsTransition::AddStaticImageStackCredentials
                | CompilerSystemsTransition::AddStaticImageStackSecureMode
                | CompilerSystemsTransition::AddStaticImageStackClockTickRate
                | CompilerSystemsTransition::AddStaticImageStackPlatform
                | CompilerSystemsTransition::AddStaticImageStackHardwareCapabilities
                | CompilerSystemsTransition::AddStaticImageStackLoadMetadata
                | CompilerSystemsTransition::AuthorizeStaticImageStackRandomSlot
                | CompilerSystemsTransition::RetainStaticImagePreparedProcess
                | CompilerSystemsTransition::QualifyStaticImageRetainedDisjointWork
                | CompilerSystemsTransition::AuthorizeStaticImageEntropyHandoff
                | CompilerSystemsTransition::AuthorizeStaticImageEntropyAcquisition
                | CompilerSystemsTransition::QualifyStaticImageEntropyDevice
                | CompilerSystemsTransition::QualifyStaticImageEntropyTransport
                | CompilerSystemsTransition::QualifyStaticImageEntropyNegotiation
                | CompilerSystemsTransition::QualifyStaticImageEntropyQueue
                | CompilerSystemsTransition::QualifyStaticImageEntropyBusMaster
                | CompilerSystemsTransition::QualifyStaticImageEntropyDmaBuffers
                | CompilerSystemsTransition::QualifyStaticImageEntropySplitRing
                | CompilerSystemsTransition::QualifyStaticImageEntropyDmaPreparation
                | CompilerSystemsTransition::QualifyStaticImageEntropyQueueAddresses
                | CompilerSystemsTransition::QualifyStaticImageEntropyQueueActivation
                | CompilerSystemsTransition::QualifyStaticImageEntropyRequestCompletion
                | CompilerSystemsTransition::QualifyStaticImageEntropySampleCapture
                | CompilerSystemsTransition::QualifyStaticImageEntropySampleQuality
                | CompilerSystemsTransition::QualifyStaticImageEntropyPoolLifecycle
                | CompilerSystemsTransition::QualifyStaticImageEntropyGeneratorLifecycle
                | CompilerSystemsTransition::DeliverStaticImageEntropyInitialRandom
                | CompilerSystemsTransition::EnterStaticImageProcess
                | CompilerSystemsTransition::RestoreStaticImageProcessEntry
                | CompilerSystemsTransition::RevokeStaticImageEntropyInitialRandom
                | CompilerSystemsTransition::RevokeStaticImageEntropyAcquisition
                | CompilerSystemsTransition::RevokeStaticImageEntropyHandoff
                | CompilerSystemsTransition::RestoreStaticImagePreparedProcess
                | CompilerSystemsTransition::RevokeStaticImageStackRandomSlot
                | CompilerSystemsTransition::RemoveStaticImageStackLoadMetadata
                | CompilerSystemsTransition::RemoveStaticImageStackHardwareCapabilities
                | CompilerSystemsTransition::RemoveStaticImageStackPlatform
                | CompilerSystemsTransition::RemoveStaticImageStackClockTickRate
                | CompilerSystemsTransition::RemoveStaticImageStackSecureMode
                | CompilerSystemsTransition::RemoveStaticImageStackCredentials
                | CompilerSystemsTransition::RemoveStaticImageStackExecutableName
                | CompilerSystemsTransition::RemoveStaticImageStackProgramHeaders
                | CompilerSystemsTransition::RemoveStaticImageStackEntryFact
                | CompilerSystemsTransition::RemoveStaticImageStackPageSize
                | CompilerSystemsTransition::ClearStaticImageStackEntry
                | CompilerSystemsTransition::RevokeStaticImageStackLeaf
                | CompilerSystemsTransition::CopyStaticImageConstructionPayload
                | CompilerSystemsTransition::SealStaticImageConstructionPermissions
                | CompilerSystemsTransition::AttachStaticImageAddressSpace
                | CompilerSystemsTransition::ActivateStaticImageAddressSpace
                | CompilerSystemsTransition::RestoreStaticImageAddressSpace
                | CompilerSystemsTransition::AuthorizeStaticImageEntry
                | CompilerSystemsTransition::AuthorizeStaticImageStackSlot
                | CompilerSystemsTransition::RevokeStaticImageStackSlot
                | CompilerSystemsTransition::RevokeStaticImageEntry
                | CompilerSystemsTransition::DetachStaticImageAddressSpace
                | CompilerSystemsTransition::ReclaimStaticImageConstructionHierarchy
                | CompilerSystemsTransition::RevokeStaticImageConstructionMetadata
                | CompilerSystemsTransition::ReclaimStaticImageConstructionPayload
                | CompilerSystemsTransition::UnmapKernelFrames
                | CompilerSystemsTransition::CommitTranslationUpdate
                | CompilerSystemsTransition::ActivateTranslationSpace
                | CompilerSystemsTransition::UnmapTranslationMapping
                | CompilerSystemsTransition::ObserveDebugBreak
                | CompilerSystemsTransition::EnterDebugBreak
                | CompilerSystemsTransition::ResumeDebugBreak
                | CompilerSystemsTransition::SendLocalNotification { .. }
                | CompilerSystemsTransition::BeginLocalNotificationWait { .. }
                | CompilerSystemsTransition::ObserveLocalNotification { .. }
                | CompilerSystemsTransition::EnterLocalNotificationInterrupt { .. }
                | CompilerSystemsTransition::CompleteLocalNotificationInterrupt { .. }
                | CompilerSystemsTransition::ResumeLocalNotificationInterrupt { .. }
                | CompilerSystemsTransition::EndLocalNotificationWait { .. }
                | CompilerSystemsTransition::ObserveMonotonicClock { .. }
                | CompilerSystemsTransition::ArmDeadline { .. }
                | CompilerSystemsTransition::BeginDeadlineWait { .. }
                | CompilerSystemsTransition::ObserveDeadline { .. }
                | CompilerSystemsTransition::EnterDeadlineInterrupt { .. }
                | CompilerSystemsTransition::CompleteDeadlineInterrupt { .. }
                | CompilerSystemsTransition::ResumeDeadlineInterrupt { .. }
                | CompilerSystemsTransition::EndDeadlineWait { .. }
                | CompilerSystemsTransition::EndAtomicWord
                | CompilerSystemsTransition::ConsumeEmptyRunnableQueue
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
        SYSTEMS_BOOTSTRAP_STORAGE_RELEASE, SYSTEMS_DEADLINE_AFTER, SYSTEMS_DEADLINE_ARM,
        SYSTEMS_DEADLINE_COMPLETE, SYSTEMS_FRAME_ALLOCATOR_CREATE, SYSTEMS_FRAMES_RELEASE,
        SYSTEMS_KERNEL_MAP, SYSTEMS_KERNEL_MAPPING_LOAD_BYTE, SYSTEMS_KERNEL_MAPPING_STORE_BYTE,
        SYSTEMS_KERNEL_UNMAP, analyze_systems_for_compiler,
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

    fn put_u16(image: &mut [u8], offset: usize, value: u16) {
        image[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn put_u32(image: &mut [u8], offset: usize, value: u32) {
        image[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn put_u64(image: &mut [u8], offset: usize, value: u64) {
        image[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }

    fn busybox_shaped_static_elf() -> Vec<u8> {
        qualified_x86_64_static_user_image_fixture()
    }

    #[test]
    fn derives_the_pinned_busybox_shaped_static_elf_plan() {
        // TOPAL-SYSTEMS-STATIC-ELF-PLAN-001, TOPAL-COMP-SYSTEMS-ARTIFACT-001.
        let image = busybox_shaped_static_elf();
        let plan = derive_x86_64_static_elf_load_plan(&image).unwrap();
        assert_eq!(
            plan.identity,
            StaticElfIdentity {
                class: 2,
                data_encoding: 1,
                version: 1,
                file_type: 2,
                machine: 62,
            }
        );
        assert_eq!(plan.entry, 0x004a_6580);
        assert_eq!(plan.page_count, 503);
        assert_eq!(plan.pages.start, 0x0040_0000);
        assert_eq!(plan.pages.end, 0x005f_7000);
        assert_eq!(
            plan.segments
                .iter()
                .map(|segment| segment.role)
                .collect::<Vec<_>>(),
            vec![
                StaticElfLoadRole::Headers,
                StaticElfLoadRole::Executable,
                StaticElfLoadRole::ImmutableData,
                StaticElfLoadRole::WritableData,
            ]
        );
        assert_eq!(plan.segments[1].pages.start, 0x0040_1000);
        assert_eq!(plan.segments[1].pages.end, 0x0058_d000);
        assert_eq!(plan.segments[3].memory.end, 0x005f_61c8);
        assert_eq!(
            plan.tls,
            Some(StaticElfTlsPlan {
                file: StaticElfInterval {
                    start: 0x1e_4660,
                    end: 0x1e_46a0,
                },
                memory: StaticElfInterval {
                    start: 0x005e_5660,
                    end: 0x005e_56e2,
                },
                alignment: 8,
            })
        );
        assert_eq!(
            plan.relro_pages,
            Some(StaticElfInterval {
                start: 0x005e_5000,
                end: 0x005e_d000,
            })
        );
        assert!(!plan.stack_executable);
        let error = validate_x86_64_initial_user_image(&image).unwrap_err();
        assert!(error.to_string().contains("503-frame affine extent"));
        assert!(error.to_string().contains("not runtime-selectable"));
    }

    #[test]
    fn static_user_image_validation_requires_the_exact_approved_shape() {
        // TOPAL-SYSTEMS-STATIC-ELF-ARTIFACT-001,
        // TOPAL-SYSTEMS-STATIC-ELF-RESIDENT-HEADER-VARIANTS-001,
        // TOPAL-COMP-SYSTEMS-TEST-001.
        let mut image = busybox_shaped_static_elf();
        image[0x1000] = 0xa5;
        validate_x86_64_static_user_image(&image).unwrap();

        let mut external_header = image.clone();
        put_u64(&mut external_header, 40, 0x1e_dea0);
        put_u16(&mut external_header, 58, 64);
        put_u16(&mut external_header, 60, 23);
        put_u16(&mut external_header, 62, 22);
        validate_x86_64_static_user_image(&external_header).unwrap();

        let rejected = |mutate: fn(&mut Vec<u8>)| {
            let mut malformed = image.clone();
            mutate(&mut malformed);
            assert!(validate_x86_64_static_user_image(&malformed).is_err());
        };
        rejected(|candidate| {
            candidate.pop();
        });
        rejected(|candidate| put_u16(candidate, 56, 10));
        rejected(|candidate| put_u64(candidate, 24, 0x004a_6581));
        rejected(|candidate| put_u64(candidate, 64 + 2 * 56 + 32, 0x5_703f));
        rejected(|candidate| put_u64(candidate, 64 + 7 * 56 + 40, 0x81));
        rejected(|candidate| put_u64(candidate, 64 + 10 * 56 + 40, 0x799f));
        rejected(|candidate| put_u64(candidate, 40, 0x1e_dea0));
        rejected(|candidate| {
            put_u64(candidate, 40, 0x1e_dea0);
            put_u16(candidate, 58, 64);
            put_u16(candidate, 60, 24);
            put_u16(candidate, 62, 22);
        });
    }

    #[test]
    fn realizes_the_pinned_static_elf_as_ordered_wx_safe_pages() {
        // TOPAL-SYSTEMS-STATIC-ELF-PAGE-PLAN-001, TOPAL-COMP-SYSTEMS-ARTIFACT-001.
        let load = derive_x86_64_static_elf_load_plan(&busybox_shaped_static_elf()).unwrap();
        let plan = derive_x86_64_static_elf_page_plan(&load).unwrap();
        assert_eq!(plan.frame_count, 503);
        assert_eq!(plan.alignment_frames, 1);
        assert_eq!(plan.pages.len(), 503);
        assert_eq!(plan.pages.first().unwrap().virtual_address, 0x0040_0000);
        assert_eq!(plan.pages.last().unwrap().virtual_address, 0x005f_6000);
        assert!(plan.pages.iter().enumerate().all(|(index, page)| {
            page.frame_ordinal == u64::try_from(index).unwrap()
                && page.initial_permission == StaticElfPagePermission::ReadWriteNoExecute
        }));
        assert_eq!(
            plan.pages
                .iter()
                .filter(|page| {
                    page.entry_permission == StaticElfPagePermission::ReadOnlyExecute
                })
                .count(),
            396
        );
        assert_eq!(
            plan.pages
                .iter()
                .filter(|page| {
                    page.entry_permission == StaticElfPagePermission::ReadOnlyNoExecute
                })
                .count(),
            89
        );
        assert_eq!(
            plan.pages
                .iter()
                .filter(|page| {
                    page.entry_permission == StaticElfPagePermission::ReadWriteNoExecute
                })
                .count(),
            18
        );
        assert_eq!(
            plan.construction.first(),
            Some(&StaticElfConstructionOperation::Zero(StaticElfInterval {
                start: 0x0040_0000,
                end: 0x005f_7000,
            }))
        );
        assert_eq!(plan.construction.len(), 5);
        assert!(
            plan.construction[1..]
                .iter()
                .all(|operation| matches!(operation, StaticElfConstructionOperation::Copy(_)))
        );
        assert_eq!(
            plan.construction.last(),
            Some(&StaticElfConstructionOperation::Copy(
                StaticElfCopyOperation {
                    file: StaticElfInterval {
                        start: 0x1e_4660,
                        end: 0x1e_dd88,
                    },
                    destination: StaticElfInterval {
                        start: 0x005e_5660,
                        end: 0x005e_ed88,
                    },
                }
            ))
        );
    }

    #[test]
    fn rejects_inconsistent_inherited_static_elf_page_facts() {
        // TOPAL-SYSTEMS-STATIC-ELF-PAGE-PLAN-001, TOPAL-COMP-SYSTEMS-TEST-001.
        let valid = derive_x86_64_static_elf_load_plan(&busybox_shaped_static_elf()).unwrap();
        let rejected = |mutate: fn(&mut StaticElfLoadPlan)| {
            let mut load = valid.clone();
            mutate(&mut load);
            assert!(derive_x86_64_static_elf_page_plan(&load).is_err());
        };

        rejected(|load| load.page_count = 502);
        rejected(|load| load.pages.end += STATIC_ELF_PAGE_SIZE);
        rejected(|load| load.segments[1].pages.start += STATIC_ELF_PAGE_SIZE);
        rejected(|load| load.segments[2].role = StaticElfLoadRole::WritableData);
        rejected(|load| load.segments[3].file.end = load.segments[3].memory.end + 1);
        rejected(|load| {
            load.relro_pages = Some(StaticElfInterval {
                start: load.segments[2].pages.start,
                end: load.segments[2].pages.end,
            });
        });
    }

    #[test]
    fn rejects_malformed_bounded_static_elf_plans_before_publication() {
        // TOPAL-SYSTEMS-STATIC-ELF-PLAN-001, TOPAL-COMP-SYSTEMS-TEST-001.
        let valid = busybox_shaped_static_elf();
        let rejected = |mutate: fn(&mut [u8])| {
            let mut image = valid.clone();
            mutate(&mut image);
            assert!(derive_x86_64_static_elf_load_plan(&image).is_err());
        };

        rejected(|image| image[4] = 1);
        rejected(|image| put_u16(image, 56, 17));
        rejected(|image| put_u32(image, 64 + 4 * 56, 3));
        rejected(|image| put_u32(image, 64 + 4 * 56, 2));
        rejected(|image| put_u32(image, 64 + 56 + 4, 7));
        rejected(|image| put_u64(image, 64 + 3 * 56 + 32, 0x2_0000));
        rejected(|image| put_u64(image, 64 + 2 * 56 + 48, 0x3000));
        rejected(|image| put_u64(image, 64 + 2 * 56 + 8, 0x18_d001));
        rejected(|image| put_u64(image, 64 + 8, 0x1e_e000));
        rejected(|image| put_u64(image, 64 + 2 * 56 + 16, 0x0058_c000));
        rejected(|image| put_u64(image, 24, 0x0040_0000));
        rejected(|image| put_u32(image, 64 + 9 * 56 + 4, 7));
        rejected(|image| put_u32(image, 64 + 6 * 56, 7));
        rejected(|image| put_u64(image, 64 + 7 * 56 + 16, 0x0040_0000));
        rejected(|image| {
            put_u32(image, 64 + 6 * 56, 0x6474_e552);
            put_u64(image, 64 + 6 * 56 + 16, 0x005e_5000);
            put_u64(image, 64 + 6 * 56 + 40, 0x1000);
        });
        rejected(|image| put_u64(image, 64 + 10 * 56 + 16, 0x0040_0000));
        rejected(|image| put_u64(image, 64 + 3 * 56 + 40, 0x1_a9a1));
        rejected(|image| put_u32(image, 64 + 9 * 56, 4));
    }

    #[test]
    #[allow(clippy::too_many_lines)] // Root dependencies, relocations, and access bytes form one structural audit.
    fn generated_root_has_only_typed_provider_dependencies() {
        // TOPAL-COMP-SYSTEMS-X64-001, TOPAL-SYSTEMS-MACHINE-001.
        let root = generate_root_object(&program()).unwrap();
        let file = object::File::parse(root.as_slice()).unwrap();
        let undefined = file
            .symbols()
            .filter(object::ObjectSymbol::is_undefined)
            .map(|symbol| symbol.name().unwrap().to_owned())
            .collect::<Vec<_>>();
        assert_root_dependencies(&undefined);
        let text = file.section_by_name(X86_SYSTEMS_ROOT_TEXT_SECTION).unwrap();
        let root_bytes = text.data().unwrap();
        assert!(
            root_bytes
                .windows(5)
                .any(|bytes| bytes == [0xba, 0xf7, 0x01, 0x00, 0x00]),
            "checked root must request the exact 503-frame static ELF extent"
        );
        assert!(
            root_bytes
                .windows(5)
                .any(|bytes| bytes == [0xba, 0x04, 0x00, 0x00, 0x00]),
            "checked root must request the exact five-frame static ELF hierarchy extent"
        );
        assert!(
            root_bytes
                .windows(7)
                .any(|bytes| bytes == [0x4c, 0x8d, 0x83, 0x00, 0x70, 0x1f, 0x00]),
            "checked root must pass the complete 503-frame payload exclusion"
        );
        assert!(
            root_bytes
                .windows(3)
                .any(|bytes| bytes == [0x49, 0x89, 0xde])
                && root_bytes
                    .windows(3)
                    .any(|bytes| bytes == [0x4c, 0x89, 0xf3]),
            "checked root must retain and restore the exact payload owner"
        );
        assert!(
            root_bytes
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x8d, 0xbb, 0x00, 0x10, 0x00, 0x00])
                && root_bytes
                    .windows(3)
                    .any(|bytes| bytes == [0x49, 0x89, 0xdc])
                && root_bytes
                    .windows(3)
                    .any(|bytes| bytes == [0x4c, 0x89, 0xe3]),
            "checked root must place the hierarchy after the leaf table and preserve its owner"
        );
        assert!(
            root_bytes
                .windows(7)
                .any(|bytes| bytes == [0x48, 0x8d, 0xbb, 0x00, 0x50, 0x00, 0x00])
                && root_bytes
                    .windows(3)
                    .any(|bytes| bytes == [0x48, 0x89, 0xdd])
                && root_bytes.windows(9).any(|bytes| {
                    bytes == [0x48, 0x31, 0xdb, 0x48, 0x89, 0xeb, 0x48, 0x31, 0xed]
                }),
            "checked root must allocate stack backing after the hierarchy, clear it, and restore the hierarchy owner"
        );
        assert!(
            root_bytes
                .windows(3)
                .filter(|bytes| *bytes == [0x49, 0x89, 0xd9])
                .count()
                >= 2,
            "checked root must pass the opaque backing identity to both stack-leaf transitions"
        );
        let relocation_targets = text
            .relocations()
            .map(|(_, relocation)| {
                let object::RelocationTarget::Symbol(symbol) = relocation.target() else {
                    panic!("root relocations target only provider symbols");
                };
                file.symbol_by_index(symbol).unwrap().name().unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(relocation_targets.len(), 9482);
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_BOOT_MEMORY_SYMBOL)
                .count(),
            1
        );
        for target in [
            X86_SYSTEMS_LOCAL_NOTIFICATION_SEND_SYMBOL,
            X86_SYSTEMS_LOCAL_NOTIFICATION_WAIT_SYMBOL,
            X86_SYSTEMS_LOCAL_NOTIFICATION_COMPLETE_SYMBOL,
        ] {
            assert_eq!(
                relocation_targets
                    .iter()
                    .filter(|actual| **actual == target)
                    .count(),
                1
            );
        }
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_MONOTONIC_CLOCK_NOW_SYMBOL)
                .count(),
            2
        );
        for target in [
            X86_SYSTEMS_DEADLINE_AFTER_SYMBOL,
            X86_SYSTEMS_DEADLINE_ARM_SYMBOL,
            X86_SYSTEMS_DEADLINE_COMPLETE_SYMBOL,
        ] {
            assert_eq!(
                relocation_targets
                    .iter()
                    .filter(|actual| **actual == target)
                    .count(),
                1
            );
        }
        for (target, expected) in [
            (X86_SYSTEMS_CONTEXT_CREATE_SYMBOL, 3),
            (X86_SYSTEMS_CONTEXT_TRANSFER_SYMBOL, 6),
            (X86_SYSTEMS_CONTEXT_AWAIT_DEADLINE_PREEMPTION_SYMBOL, 1),
            (X86_SYSTEMS_CONTEXT_PREEMPT_CURRENT_SYMBOL, 1),
            (X86_SYSTEMS_CONTEXT_RETIRE_SYMBOL, 3),
            (X86_SYSTEMS_CONTEXT_RECLAIM_SYMBOL, 3),
        ] {
            assert_eq!(
                relocation_targets
                    .iter()
                    .filter(|actual| **actual == target)
                    .count(),
                expected
            );
        }
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_KERNEL_THREAD_ENTRY)
                .count(),
            1
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_COMPLETION_THREAD_ENTRY)
                .count(),
            1
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_TERMINAL_THREAD_ENTRY)
                .count(),
            1
        );
        for target in [
            X86_SYSTEMS_ATOMIC_CREATE_SYMBOL,
            X86_SYSTEMS_ATOMIC_COMPARE_EXCHANGE_SYMBOL,
            X86_SYSTEMS_ATOMIC_LOAD_SYMBOL,
        ] {
            assert_eq!(
                relocation_targets
                    .iter()
                    .filter(|actual| **actual == target)
                    .count(),
                1
            );
        }
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_FRAME_ALLOCATE_SYMBOL)
                .count(),
            12
        );
        for target in [
            X86_SYSTEMS_STATIC_IMAGE_PREPARE_SYMBOL,
            X86_SYSTEMS_STATIC_IMAGE_ZERO_PAYLOAD_SYMBOL,
            X86_SYSTEMS_STATIC_IMAGE_ZERO_STACK_BACKING_SYMBOL,
            X86_SYSTEMS_STATIC_IMAGE_PUBLISH_STACK_LEAF_SYMBOL,
            X86_SYSTEMS_STATIC_IMAGE_INITIALIZE_STACK_ENTRY_SYMBOL,
            X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_PAGESZ_SYMBOL,
            X86_SYSTEMS_STATIC_IMAGE_ADD_STACK_ENTRY_FACT_SYMBOL,
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
            X86_SYSTEMS_STATIC_IMAGE_PROCESS_ENTER_SYMBOL,
            X86_SYSTEMS_STATIC_IMAGE_PROCESS_RESTORE_ENTRY_SYMBOL,
        ] {
            assert_eq!(
                relocation_targets
                    .iter()
                    .filter(|actual| **actual == target)
                    .count(),
                1
            );
        }
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_USER_ADDRESS_SPACE_VERIFY_MAPPING_SYMBOL)
                .count(),
            0
        );
        for target in [
            X86_SYSTEMS_USER_ADDRESS_SPACE_MAP_SYMBOL,
            X86_SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY_SYMBOL,
            X86_SYSTEMS_USER_ADDRESS_SPACE_UNMAP_SYMBOL,
        ] {
            assert_eq!(
                relocation_targets
                    .iter()
                    .filter(|actual| **actual == target)
                    .count(),
                2
            );
        }
        for target in [
            X86_SYSTEMS_USER_MAPPING_STORE_BYTE_SYMBOL,
            X86_SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE_SYMBOL,
            X86_SYSTEMS_USER_MAPPING_LOAD_BYTE_SYMBOL,
            X86_SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE_SYMBOL,
            X86_SYSTEMS_USER_MAPPING_PERMIT_EXECUTE_SYMBOL,
            X86_SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA_SYMBOL,
        ] {
            assert_eq!(
                relocation_targets
                    .iter()
                    .filter(|actual| **actual == target)
                    .count(),
                1
            );
        }
        assert_translation_relocations(&relocation_targets);
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_CRITICAL_ENTER_SYMBOL)
                .count(),
            1
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_CRITICAL_RESTORE_SYMBOL)
                .count(),
            1
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == "topal_x86_systems_uart16550_write")
                .count(),
            6858
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_PCI_CONFIGURATION_READ_DWORD_SYMBOL)
                .count(),
            48
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD_SYMBOL)
                .count(),
            25
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_DMA_BUFFER_CREATE_SYMBOL)
                .count(),
            24
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_DMA_BUFFER_BIND_REFERENCE_SYMBOL)
                .count(),
            11
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_DMA_BUFFER_STORE_INTEGER_SYMBOL)
                .count(),
            143
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_DMA_BUFFER_PREPARE_DEVICE_SYMBOL)
                .count(),
            20
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_DMA_BUFFER_COMPLETE_CPU_SYMBOL)
                .count(),
            20
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_DMA_BUFFER_LOAD_INTEGER_SYMBOL)
                .count(),
            127
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_ENTROPY_SAMPLE_CAPTURE_SYMBOL)
                .count(),
            6
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_ENTROPY_POOL_MIX_SYMBOL)
                .count(),
            4
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE_SYMBOL)
                .count(),
            27
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_RAW_ENTROPY_SAMPLE_SYMBOL)
                .count(),
            15
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_ENTROPY_POOL_STATE_SYMBOL)
                .count(),
            10
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_ENTROPY_GENERATOR_KEY_SYMBOL)
                .count(),
            11
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_ENTROPY_USER_RANDOM_SERVICE_SYMBOL)
                .count(),
            1
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_ENTROPY_OUTPUT_SYMBOL)
                .count(),
            9
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_DMA_VIRTQUEUE_CONTROL_SYMBOL)
                .count(),
            328
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_DMA_ENTROPY_PAYLOAD_SYMBOL)
                .count(),
            37
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_PCI_REGION_READ_SYMBOL)
                .count(),
            231
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_PCI_REGION_READ_EVENTUALLY_SYMBOL)
                .count(),
            7
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == X86_SYSTEMS_PCI_REGION_WRITE_SYMBOL)
                .count(),
            207
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == "topal_bootstrap_storage")
                .count(),
            60
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == "topal_x86_systems_fatal")
                .count(),
            1096
        );
        assert_eq!(
            relocation_targets
                .iter()
                .filter(|target| **target == "topal_x86_systems_interrupt_return")
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
            root_bytes
                .windows(7)
                .any(|bytes| bytes == [0xc6, 0x83, 0, 0, 0, 0, 165]),
            "root must store through the selected physical frame mapping"
        );
        assert!(
            root_bytes.windows(11).any(|bytes| {
                bytes[..7] == [0x0f, 0xb6, 0x83, 0, 0, 0, 0]
                    && bytes[7..11] == [0x3c, 165, 0x0f, 0x85]
            }),
            "root must load through the selected physical frame mapping"
        );
        assert!(
            root_bytes
                .windows(8)
                .any(|bytes| bytes == [0x41, 0xc6, 0x86, 0, 0, 0, 0, 60]),
            "root must store through the committed active translation mapping"
        );
        assert!(
            root_bytes.windows(12).any(|bytes| {
                bytes[..8] == [0x41, 0x0f, 0xb6, 0x86, 0, 0, 0, 0]
                    && bytes[8..12] == [0x3c, 60, 0x0f, 0x85]
            }),
            "root must load through the committed active translation mapping"
        );
        assert!(
            root_bytes.windows(15).any(|bytes| {
                bytes[..3] == [0x0f, 0xb6, 0x05] && bytes[7..11] == [0x3c, 90, 0x0f, 0x85]
            }),
            "root must retain the real byte load and mismatch branch"
        );
        assert!(
            root_bytes
                .windows(9)
                .any(|bytes| bytes == [0x31, 0xc0, 0x48, 0x89, 0x07, 0x48, 0x89, 0x47, 8]),
            "root must overwrite the provider-private raw entropy sample"
        );
        assert!(
            root_bytes
                .windows(6)
                .any(|bytes| bytes == [0x48, 0x83, 0x3f, 0, 0x0f, 0x85]),
            "root must verify raw entropy-sample erasure and fail closed"
        );
        assert_eq!(
            root_bytes
                .windows(7)
                .filter(|bytes| bytes[..3] == [0x48, 0x8d, 0x3d])
                .count(),
            395,
            "root must derive opaque storage for DMA, atomic, notification, clock, deadline, context, and user operations"
        );
        assert!(
            root_bytes
                .windows(4)
                .any(|bytes| bytes == [0x48, 0x8b, 0x4f, 0x68]),
            "root must pass the prepared user backing only as a provider-private exclusion"
        );
        assert!(
            root_bytes
                .windows(4)
                .any(|bytes| bytes == [0x48, 0x83, 0xf8, 0x2a]),
            "root must retain the acquire-load confirmation"
        );
        assert!(file.symbol_by_name(X86_SYSTEMS_KERNEL_ENTRY).is_some());
        assert!(file.symbol_by_name(X86_SYSTEMS_DEBUG_BREAK_ENTRY).is_some());
        assert!(
            file.symbol_by_name(X86_SYSTEMS_LOCAL_NOTIFICATION_ENTRY)
                .is_some()
        );
        assert!(file.symbol_by_name(X86_SYSTEMS_DEADLINE_ENTRY).is_some());
        assert!(
            file.symbol_by_name(X86_SYSTEMS_KERNEL_THREAD_ENTRY)
                .is_some()
        );
        assert!(
            file.symbol_by_name(X86_SYSTEMS_COMPLETION_THREAD_ENTRY)
                .is_some()
        );
        assert!(
            file.symbol_by_name(X86_SYSTEMS_TERMINAL_THREAD_ENTRY)
                .is_some()
        );
    }

    fn assert_translation_relocations(relocation_targets: &[&str]) {
        for target in [
            X86_SYSTEMS_TRANSLATION_BEGIN_SYMBOL,
            X86_SYSTEMS_TRANSLATION_COMMIT_SYMBOL,
            X86_SYSTEMS_TRANSLATION_ACTIVATE_SYMBOL,
            X86_SYSTEMS_TRANSLATION_EDIT_MAP_SYMBOL,
            X86_SYSTEMS_TRANSLATION_EDIT_UNMAP_SYMBOL,
        ] {
            assert_eq!(
                relocation_targets
                    .iter()
                    .filter(|actual| **actual == target)
                    .count(),
                1
            );
        }
        for (target, expected) in [
            (X86_SYSTEMS_TRANSLATION_EDIT_BEGIN_SYMBOL, 2),
            (X86_SYSTEMS_TRANSLATION_EDIT_COMMIT_SYMBOL, 2),
        ] {
            assert_eq!(
                relocation_targets
                    .iter()
                    .filter(|actual| **actual == target)
                    .count(),
                expected
            );
        }
    }

    fn assert_root_dependencies(undefined: &[String]) {
        assert_eq!(
            undefined,
            [
                "topal_x86_systems_describe_boot_memory",
                "topal_x86_systems_allocate_physical_frames",
                "topal_x86_systems_static_image_prepare_initial_leaves",
                "topal_x86_systems_static_image_zero_payload",
                "topal_x86_systems_static_image_zero_stack_backing",
                "topal_x86_systems_static_image_publish_stack_leaf",
                "topal_x86_systems_static_image_initialize_stack_entry",
                "topal_x86_systems_static_image_add_stack_pagesz",
                "topal_x86_systems_static_image_add_stack_entry_fact",
                "topal_x86_systems_static_image_add_stack_program_headers",
                "topal_x86_systems_static_image_add_stack_executable_name",
                "topal_x86_systems_static_image_add_stack_credentials",
                "topal_x86_systems_static_image_add_stack_secure_mode",
                "topal_x86_systems_static_image_add_stack_clock_tick_rate",
                "topal_x86_systems_static_image_add_stack_platform",
                "topal_x86_systems_static_image_add_stack_hardware_capabilities",
                "topal_x86_systems_static_image_add_stack_load_metadata",
                "topal_x86_systems_static_image_authorize_stack_random_slot",
                "topal_x86_systems_static_image_retain_prepared_process",
                "topal_x86_systems_static_image_qualify_retained_disjoint_work",
                "topal_x86_systems_static_image_authorize_entropy_handoff",
                "topal_x86_systems_static_image_authorize_entropy_acquisition",
                "topal_x86_systems_static_image_qualify_entropy_device",
                "topal_x86_systems_static_image_qualify_entropy_transport",
                "topal_x86_systems_static_image_qualify_entropy_negotiation",
                "topal_x86_systems_static_image_qualify_entropy_queue",
                "topal_x86_systems_static_image_qualify_entropy_bus_master",
                "topal_x86_systems_static_image_qualify_entropy_dma_buffers",
                "topal_x86_systems_static_image_qualify_entropy_split_ring",
                "topal_x86_systems_static_image_qualify_entropy_dma_preparation",
                "topal_x86_systems_static_image_qualify_entropy_queue_addresses",
                "topal_x86_systems_static_image_qualify_entropy_queue_activation",
                "topal_x86_systems_static_image_qualify_entropy_request_completion",
                "topal_x86_systems_static_image_qualify_entropy_sample_capture",
                "topal_x86_systems_static_image_qualify_entropy_sample_quality",
                "topal_x86_systems_static_image_qualify_entropy_pool_lifecycle",
                "topal_x86_systems_static_image_qualify_entropy_generator_lifecycle",
                "topal_x86_systems_static_image_deliver_entropy_initial_random",
                "topal_x86_systems_static_image_process_enter",
                "topal_x86_systems_static_image_process_restore_entry",
                "topal_x86_systems_static_image_revoke_entropy_initial_random",
                "topal_x86_systems_static_image_revoke_entropy_acquisition",
                "topal_x86_systems_static_image_revoke_entropy_handoff",
                "topal_x86_systems_static_image_restore_prepared_process",
                "topal_x86_systems_static_image_revoke_stack_random_slot",
                "topal_x86_systems_static_image_remove_stack_load_metadata",
                "topal_x86_systems_static_image_remove_stack_hardware_capabilities",
                "topal_x86_systems_static_image_remove_stack_platform",
                "topal_x86_systems_static_image_remove_stack_clock_tick_rate",
                "topal_x86_systems_static_image_remove_stack_secure_mode",
                "topal_x86_systems_static_image_remove_stack_credentials",
                "topal_x86_systems_static_image_remove_stack_executable_name",
                "topal_x86_systems_static_image_remove_stack_program_headers",
                "topal_x86_systems_static_image_remove_stack_entry_fact",
                "topal_x86_systems_static_image_remove_stack_pagesz",
                "topal_x86_systems_static_image_clear_stack_entry",
                "topal_x86_systems_static_image_revoke_stack_leaf",
                "topal_x86_systems_static_image_copy_payload",
                "topal_x86_systems_static_image_seal_permissions",
                "topal_x86_systems_static_image_attach_address_space",
                "topal_x86_systems_static_image_activate_address_space",
                "topal_x86_systems_static_image_restore_address_space",
                "topal_x86_systems_static_image_authorize_entry",
                "topal_x86_systems_static_image_authorize_stack_slot",
                "topal_x86_systems_static_image_revoke_stack_slot",
                "topal_x86_systems_static_image_revoke_entry",
                "topal_x86_systems_static_image_detach_address_space",
                "topal_x86_systems_static_image_revoke_initial_leaves",
                "topal_x86_systems_begin_bootstrap_translation",
                "topal_x86_systems_commit_bootstrap_translation",
                "topal_x86_systems_activate_bootstrap_translation",
                "topal_x86_systems_begin_active_translation_edit",
                "topal_x86_systems_stage_active_translation_map",
                "topal_x86_systems_stage_active_translation_unmap",
                "topal_x86_systems_commit_active_translation_edit",
                "topal_x86_systems_enter_local_interrupt_critical",
                "topal_x86_systems_restore_local_interrupt_critical",
                "topal_x86_systems_atomic_word_create",
                "topal_x86_systems_atomic_word_compare_exchange",
                "topal_x86_systems_atomic_word_load_acquire",
                "topal_x86_systems_local_notification_send",
                "topal_x86_systems_local_notification_wait",
                "topal_x86_systems_local_notification_complete",
                "topal_x86_systems_monotonic_clock_now",
                "topal_x86_systems_pci_configuration_read_dword_equals",
                "topal_x86_systems_pci_configuration_update_word_masked",
                "topal_x86_systems_dma_buffer_create",
                "topal_x86_systems_dma_buffer_bind_reference",
                "topal_x86_systems_dma_buffer_store_integer",
                "topal_x86_systems_dma_buffer_prepare_device",
                "topal_x86_systems_dma_buffer_complete_cpu",
                "topal_x86_systems_dma_buffer_load_integer_equals",
                "topal_x86_systems_entropy_capture_dma_sample",
                "topal_x86_systems_entropy_mix_blake2s_pool_sample",
                "topal_x86_systems_entropy_initialize_chacha20_generator",
                "topal_x86_systems_entropy_extract_chacha20_generator",
                "topal_x86_systems_entropy_deliver_initial_auxv_random",
                "topal_x86_systems_entropy_reserve_initial_user_random",
                "topal_x86_systems_entropy_bind_reusable_user_random",
                "topal_x86_systems_pci_region_configuration_gateway_write_dma_reference",
                "topal_x86_systems_dma_virtqueue_control",
                "topal_x86_systems_dma_entropy_payload",
                "topal_x86_systems_raw_entropy_sample",
                "topal_x86_systems_entropy_pool_state",
                "topal_x86_systems_entropy_generator_key",
                "topal_x86_systems_entropy_output",
                "topal_x86_systems_entropy_user_random_service",
                "topal_x86_static_process_brk_reserve",
                "topal_x86_systems_pci_region_configuration_gateway_read_equals",
                "topal_x86_systems_pci_region_configuration_gateway_read_eventually_equals",
                "topal_x86_systems_pci_region_configuration_gateway_write",
                "topal_x86_systems_deadline_after",
                "topal_x86_systems_deadline_arm",
                "topal_x86_systems_deadline_wait",
                "topal_x86_systems_deadline_complete",
                "topal_x86_systems_context_create",
                "topal_x86_systems_context_transfer",
                "topal_x86_systems_context_await_deadline_preemption",
                "topal_x86_systems_context_preempt_current",
                "topal_x86_systems_context_retire",
                "topal_x86_systems_context_reclaim",
                "topal_x86_systems_user_prepare_initial_address_space",
                "topal_x86_systems_user_verify_initial_mapping",
                "topal_x86_systems_user_map_initial_frame",
                "topal_x86_systems_user_store_initial_mapping_byte",
                "topal_x86_systems_user_install_initial_entry_trampoline",
                "topal_x86_systems_user_protect_initial_mapping_read_only",
                "topal_x86_systems_user_load_initial_mapping_byte",
                "topal_x86_systems_user_verify_initial_entry_trampoline",
                "topal_x86_systems_user_permit_initial_mapping_execute",
                "topal_x86_systems_user_seal_initial_mapping_immutable_data",
                "topal_x86_systems_user_permit_initial_mapping_writable_data",
                "topal_x86_systems_user_reserve_initial_anonymous_mapping",
                "topal_x86_systems_user_release_initial_anonymous_mapping",
                "topal_x86_systems_user_unmap_initial_frame",
                "topal_x86_systems_user_prepare_initial_probe",
                "topal_x86_systems_user_complete_initial_backing",
                "topal_x86_systems_user_reclaim_initial_address_space",
                "topal_x86_systems_user_enter_initial_context",
                "topal_x86_systems_user_dispatch_initial_syscall",
                "topal_x86_systems_uart16550_write",
                "topal_x86_systems_debug_break",
                "topal_x86_systems_interrupt_return",
                "topal_x86_systems_fatal",
                "topal_bootstrap_storage",
            ]
        );
    }

    #[test]
    fn supplied_initial_user_image_validation_is_bounded_and_payload_neutral() {
        // TOPAL-COMP-SYSTEMS-ARTIFACT-001.
        let canonical = initial_user_elf_file().unwrap();
        let mut supplied = canonical.clone();
        supplied[INITIAL_USER_DATA_FILE_OFFSET + 1] = 0xa5;
        validate_x86_64_initial_user_image(&supplied).unwrap();

        let mut wrong_size = supplied.clone();
        wrong_size.pop();
        assert!(validate_x86_64_initial_user_image(&wrong_size).is_err());

        let mut wrong_metadata = supplied.clone();
        wrong_metadata[0] = 0;
        assert!(validate_x86_64_initial_user_image(&wrong_metadata).is_err());

        let mut wrong_code_extent = supplied.clone();
        wrong_code_extent[INITIAL_USER_CODE_MEMORY_SIZE_ELF_OFFSET
            ..INITIAL_USER_CODE_MEMORY_SIZE_ELF_OFFSET + 8]
            .copy_from_slice(&4096_u64.to_le_bytes());
        assert!(validate_x86_64_initial_user_image(&wrong_code_extent).is_err());

        let mut wrong_entry = supplied.clone();
        wrong_entry[INITIAL_USER_ENTRY_ELF_OFFSET..INITIAL_USER_ENTRY_ELF_OFFSET + 8]
            .copy_from_slice(&(INITIAL_USER_CODE_ADDRESS + 4096 + 256 - 1).to_le_bytes());
        assert!(validate_x86_64_initial_user_image(&wrong_entry).is_err());

        let mut wrong_data_extent = supplied;
        wrong_data_extent
            [INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET..INITIAL_USER_DATA_FILE_SIZE_ELF_OFFSET + 8]
            .copy_from_slice(&24_u64.to_le_bytes());
        wrong_data_extent[INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET
            ..INITIAL_USER_DATA_MEMORY_SIZE_ELF_OFFSET + 8]
            .copy_from_slice(&16_u64.to_le_bytes());
        assert!(validate_x86_64_initial_user_image(&wrong_data_extent).is_err());
    }

    #[test]
    #[allow(clippy::too_many_lines)] // The complete transition artifact trace is one structural audit.
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
            5,
            "only canonical outputs are published"
        );
        assert!(published.kernel.is_file());
        assert!(published.debug.is_file());
        assert!(published.map.is_file());
        assert!(published.initial_user_image.is_file());
        assert!(published.static_user_image.is_none());
        assert!(published.provenance.is_file());
        let encoded = fs::read(&published.provenance).unwrap();
        let mut decoded: SystemsArtifactProvenance = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(decoded, published.record);
        assert_eq!(decoded.schema, X86_SYSTEMS_ARTIFACT_REVISION);
        assert_eq!(decoded.target, "x86_64-unknown-none");
        assert_eq!(decoded.outputs.len(), 4);
        assert_eq!(decoded.placements.len(), 65);
        assert_eq!(decoded.bootstrap_storage_capacity, 65_536);
        assert_eq!(decoded.bootstrap_storage_alignment, 4096);
        assert_eq!(decoded.semantic_trace.len(), 483);
        assert!(
            decoded
                .semantic_trace
                .iter()
                .any(|entry| { entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PREPARE) })
        );
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ATTACH_ADDRESS_SPACE)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ACTIVATE_ADDRESS_SPACE)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESTORE_ADDRESS_SPACE)
        }));
        assert!(
            decoded
                .semantic_trace
                .iter()
                .any(|entry| entry.starts_with(SYSTEMS_STATIC_IMAGE_PROCESS_ENTER))
        );
        assert!(
            decoded
                .semantic_trace
                .iter()
                .any(|entry| { entry.starts_with(SYSTEMS_STATIC_IMAGE_PROCESS_RESTORE_ENTRY) })
        );
        assert!(
            decoded.semantic_trace.iter().any(|entry| {
                entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTRY)
            })
        );
        assert!(
            decoded
                .semantic_trace
                .iter()
                .any(|entry| { entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTRY) })
        );
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_SLOT)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_SLOT)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_DETACH_ADDRESS_SPACE)
        }));
        assert!(
            decoded.semantic_trace.iter().any(|entry| {
                entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_METADATA)
            })
        );
        assert!(
            decoded
                .semantic_trace
                .iter()
                .any(|entry| { entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_PAYLOAD) })
        );
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_STACK_BACKING)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PUBLISH_STACK_LEAF)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_INITIALIZE_STACK_ENTRY)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PAGESZ)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_ENTRY_FACT)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_EXECUTABLE_NAME)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_CREDENTIALS)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_SECURE_MODE)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_CLOCK_TICK_RATE)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PLATFORM)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_HARDWARE_CAPABILITIES)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_LOAD_METADATA)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_RANDOM_SLOT)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_RANDOM_SLOT)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_LOAD_METADATA)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_HARDWARE_CAPABILITIES)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PLATFORM)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_CLOCK_TICK_RATE)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_SECURE_MODE)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_CREDENTIALS)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_EXECUTABLE_NAME)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_ENTRY_FACT)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PAGESZ)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_CLEAR_STACK_ENTRY)
        }));
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_LEAF)
        }));
        assert!(
            decoded
                .semantic_trace
                .iter()
                .any(|entry| { entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_COPY_PAYLOAD) })
        );
        assert!(decoded.semantic_trace.iter().any(|entry| {
            entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_SEAL_PERMISSIONS)
        }));
        assert!(
            decoded.semantic_trace.iter().any(|entry| {
                entry.starts_with(SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_PAYLOAD)
            })
        );
        assert!(
            decoded
                .semantic_trace
                .iter()
                .any(|entry| entry.starts_with(SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM))
        );
        assert!(
            decoded
                .semantic_trace
                .iter()
                .any(|entry| entry.starts_with(SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE))
        );
        assert!(
            decoded
                .semantic_trace
                .iter()
                .any(|entry| entry.starts_with(SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS))
        );
        assert!(
            decoded
                .semantic_trace
                .iter()
                .any(|entry| entry.starts_with(SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS))
        );
        assert!(decoded.semantic_trace[27].starts_with(SYSTEMS_FRAMES_ALLOCATE));
        assert!(decoded.semantic_trace[28].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(decoded.semantic_trace[29], SYSTEMS_FRAMES_RELEASE);
        assert!(decoded.semantic_trace[30].starts_with(SYSTEMS_FRAMES_ALLOCATE));
        assert!(decoded.semantic_trace[31].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[32].starts_with(SYSTEMS_FRAMES_ALLOCATE));
        assert!(decoded.semantic_trace[33].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[34],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PREPARE
        );
        assert!(decoded.semantic_trace[35].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[36],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_PAYLOAD
        );
        assert!(decoded.semantic_trace[37].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[38],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_COPY_PAYLOAD
        );
        assert!(decoded.semantic_trace[39].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[40],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_SEAL_PERMISSIONS
        );
        assert!(decoded.semantic_trace[41].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[42],
            "topal.systems.frames.allocate/1:frames=5:alignment=1"
        );
        assert!(decoded.semantic_trace[43].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[44],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ATTACH_ADDRESS_SPACE
        );
        assert!(decoded.semantic_trace[45].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[46],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ACTIVATE_ADDRESS_SPACE
        );
        assert!(decoded.semantic_trace[47].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[48],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESTORE_ADDRESS_SPACE
        );
        assert!(decoded.semantic_trace[49].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[50],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTRY
        );
        assert!(decoded.semantic_trace[51].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[52],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_SLOT
        );
        assert!(decoded.semantic_trace[53].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[54],
            "topal.systems.frames.allocate/1:frames=1:alignment=1"
        );
        assert!(decoded.semantic_trace[55].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[56],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_STACK_BACKING
        );
        assert!(decoded.semantic_trace[57].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[58],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PUBLISH_STACK_LEAF
        );
        assert!(decoded.semantic_trace[59].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[60],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_INITIALIZE_STACK_ENTRY
        );
        assert!(decoded.semantic_trace[61].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[62],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PAGESZ
        );
        assert!(decoded.semantic_trace[63].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[64],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_ENTRY_FACT
        );
        assert!(decoded.semantic_trace[65].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[66],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PROGRAM_HEADERS
        );
        assert!(decoded.semantic_trace[67].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[68],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_EXECUTABLE_NAME
        );
        assert!(decoded.semantic_trace[69].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[70],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_CREDENTIALS
        );
        assert!(decoded.semantic_trace[71].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[72],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_SECURE_MODE
        );
        assert!(decoded.semantic_trace[73].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[74],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_CLOCK_TICK_RATE
        );
        assert!(decoded.semantic_trace[75].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[76],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PLATFORM
        );
        assert!(decoded.semantic_trace[77].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[78],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_HARDWARE_CAPABILITIES
        );
        assert!(decoded.semantic_trace[79].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[80],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_LOAD_METADATA
        );
        assert!(decoded.semantic_trace[81].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[82],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_RANDOM_SLOT
        );
        assert!(decoded.semantic_trace[83].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[84],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RETAIN_PREPARED_PROCESS
        );
        assert!(decoded.semantic_trace[85].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[86],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_RETAINED_DISJOINT_WORK
        );
        assert!(decoded.semantic_trace[87].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[88],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTROPY_HANDOFF
        );
        assert!(decoded.semantic_trace[89].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[90],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTROPY_ACQUISITION
        );
        assert!(decoded.semantic_trace[91].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[92],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DEVICE
        );
        assert!(decoded.semantic_trace[93].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[94],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_TRANSPORT
        );
        assert!(decoded.semantic_trace[95].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[96],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_NEGOTIATION
        );
        assert!(decoded.semantic_trace[97].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[98],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE
        );
        assert!(decoded.semantic_trace[99].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[100],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_BUS_MASTER
        );
        assert!(decoded.semantic_trace[101].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[102],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DMA_BUFFERS
        );
        assert!(decoded.semantic_trace[103].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[104],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SPLIT_RING
        );
        assert!(decoded.semantic_trace[105].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[106],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DMA_PREPARATION
        );
        assert!(decoded.semantic_trace[107].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[108],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE_ADDRESSES
        );
        assert!(decoded.semantic_trace[109].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[110],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE_ACTIVATION
        );
        assert!(decoded.semantic_trace[111].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[112],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_REQUEST_COMPLETION
        );
        assert!(decoded.semantic_trace[113].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[114],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SAMPLE_CAPTURE
        );
        assert!(decoded.semantic_trace[115].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[116],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SAMPLE_QUALITY
        );
        assert!(decoded.semantic_trace[117].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[118],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_POOL_LIFECYCLE
        );
        assert!(decoded.semantic_trace[119].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[120],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_GENERATOR_LIFECYCLE
        );
        assert!(decoded.semantic_trace[121].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[122],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_DELIVER_ENTROPY_INITIAL_RANDOM
        );
        assert!(decoded.semantic_trace[123].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[124],
            SYSTEMS_STATIC_IMAGE_PROCESS_ENTER
        );
        assert!(decoded.semantic_trace[125].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[126].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[127].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[128].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[129].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[130].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[131].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[132].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[133].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[134].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[135].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[136].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[137].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[138].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[139].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[140].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[141].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[142].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[143].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[144],
            SYSTEMS_STATIC_IMAGE_PROCESS_RESTORE_ENTRY
        );
        assert!(decoded.semantic_trace[145].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[146],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_INITIAL_RANDOM
        );
        assert!(decoded.semantic_trace[147].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[148],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_ACQUISITION
        );
        assert!(decoded.semantic_trace[149].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[150],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_HANDOFF
        );
        assert!(decoded.semantic_trace[151].starts_with(SYSTEMS_CONSOLE_WRITE));
        decoded.semantic_trace.drain(88..152);
        assert_eq!(
            decoded.semantic_trace[88],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESTORE_PREPARED_PROCESS
        );
        assert!(decoded.semantic_trace[89].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[90],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_RANDOM_SLOT
        );
        assert!(decoded.semantic_trace[91].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[92],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_LOAD_METADATA
        );
        assert!(decoded.semantic_trace[93].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[94],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_HARDWARE_CAPABILITIES
        );
        assert!(decoded.semantic_trace[95].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[96],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PLATFORM
        );
        assert!(decoded.semantic_trace[97].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[98],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_CLOCK_TICK_RATE
        );
        assert!(decoded.semantic_trace[99].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[100],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_SECURE_MODE
        );
        assert!(decoded.semantic_trace[101].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[102],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_CREDENTIALS
        );
        assert!(decoded.semantic_trace[103].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[104],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_EXECUTABLE_NAME
        );
        assert!(decoded.semantic_trace[105].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[106],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PROGRAM_HEADERS
        );
        assert!(decoded.semantic_trace[107].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[108],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_ENTRY_FACT
        );
        assert!(decoded.semantic_trace[109].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[110],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PAGESZ
        );
        assert!(decoded.semantic_trace[111].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[112],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_CLEAR_STACK_ENTRY
        );
        assert!(decoded.semantic_trace[113].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[114],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_LEAF
        );
        assert!(decoded.semantic_trace[115].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(decoded.semantic_trace[116], SYSTEMS_FRAMES_RELEASE);
        assert_eq!(
            decoded.semantic_trace[117],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_SLOT
        );
        assert!(decoded.semantic_trace[118].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[119],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTRY
        );
        assert!(decoded.semantic_trace[120].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[121],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_DETACH_ADDRESS_SPACE
        );
        assert!(decoded.semantic_trace[122].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[123],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_HIERARCHY
        );
        assert_eq!(decoded.semantic_trace[124], SYSTEMS_FRAMES_RELEASE);
        assert_eq!(
            decoded.semantic_trace[125],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_METADATA
        );
        assert_eq!(decoded.semantic_trace[126], SYSTEMS_FRAMES_RELEASE);
        assert_eq!(
            decoded.semantic_trace[127],
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_PAYLOAD
        );
        assert_eq!(decoded.semantic_trace[128], SYSTEMS_FRAMES_RELEASE);
        decoded.semantic_trace.drain(27..129);
        for (index, request) in
            CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                .iter()
                .enumerate()
        {
            assert_eq!(
                decoded.semantic_trace[31 + index],
                format!(
                    "{SYSTEMS_PCI_CONFIGURATION_READ_DWORD}:transaction={}:segment=0:bus=0:device=5:function=0:offset={}:expected={:#010x}",
                    index + 1,
                    request.register_offset_bytes,
                    request.expected_value
                )
            );
        }
        assert_eq!(
            decoded.semantic_trace[55],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_PCI_VIRTIO_RNG_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[56],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_VIRTIO_PCI_CAPABILITIES_OK")
            )
        );
        decoded.semantic_trace.drain(31..57);
        for (index, request) in CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
            .iter()
            .enumerate()
        {
            let identity = match request.direction {
                CompilerPciRegionRegisterDirection::ReadEquals => SYSTEMS_PCI_REGION_READ,
                CompilerPciRegionRegisterDirection::ReadEventuallyEquals => {
                    SYSTEMS_PCI_REGION_READ_EVENTUALLY
                }
                CompilerPciRegionRegisterDirection::Write => SYSTEMS_PCI_REGION_WRITE,
            };
            assert_eq!(
                decoded.semantic_trace[31 + index],
                format!(
                    "{identity}:transaction={}:segment=0:bus=0:device=5:function=0:access-capability-offset=132:bar=4:region-offset=0:region-length=4096:register-offset={}:width={}:value={:#010x}",
                    index + 1,
                    request.register_offset_bytes,
                    request.width_bytes,
                    request.value
                )
            );
        }
        assert_eq!(
            decoded.semantic_trace[47],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_VIRTIO_PCI_NEGOTIATED")
            )
        );
        decoded.semantic_trace.drain(31..48);
        for (index, request) in
            CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY
                .iter()
                .enumerate()
        {
            let identity = match request.direction {
                CompilerPciRegionRegisterDirection::ReadEquals => SYSTEMS_PCI_REGION_READ,
                CompilerPciRegionRegisterDirection::ReadEventuallyEquals => {
                    SYSTEMS_PCI_REGION_READ_EVENTUALLY
                }
                CompilerPciRegionRegisterDirection::Write => SYSTEMS_PCI_REGION_WRITE,
            };
            assert_eq!(
                decoded.semantic_trace[31 + index],
                format!(
                    "{identity}:transaction={}:segment=0:bus=0:device=5:function=0:access-capability-offset=132:bar=4:region-offset=0:region-length=4096:register-offset={}:width={}:value={:#010x}",
                    index + 17,
                    request.register_offset_bytes,
                    request.width_bytes,
                    request.value
                )
            );
        }
        assert_eq!(
            decoded.semantic_trace[36],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_VIRTIO_QUEUE_DISCOVERED")
            )
        );
        decoded.semantic_trace.drain(31..37);
        assert_eq!(
            decoded.semantic_trace[31],
            format!(
                "{SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD}:transaction=1:segment=0:bus=0:device=5:function=0:offset=4:mask=0x0004:expected=0x0000:desired=0x0004"
            )
        );
        assert_eq!(
            decoded.semantic_trace[32],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_PCI_BUS_MASTER_OK")
            )
        );
        decoded.semantic_trace.drain(31..33);
        assert_eq!(
            decoded.semantic_trace[31],
            format!(
                "{SYSTEMS_DMA_BUFFER_CREATE}:buffer=1:device=1:domain=1:bytes=4096:alignment=4096:direction=bidirectional:visibility=provider-managed:role=virtqueue-control"
            )
        );
        assert_eq!(
            decoded.semantic_trace[32],
            format!(
                "{SYSTEMS_DMA_BUFFER_CREATE}:buffer=2:device=1:domain=1:bytes=4096:alignment=4096:direction=device-to-memory:visibility=provider-managed:role=entropy-payload"
            )
        );
        assert_eq!(
            decoded.semantic_trace[33],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_DMA_BUFFERS_OK")
            )
        );
        decoded.semantic_trace.drain(31..34);
        assert_eq!(
            decoded.semantic_trace[31],
            format!(
                "{SYSTEMS_DMA_BUFFER_BIND_REFERENCE}:destination-buffer=1:destination-offset=0:referenced-buffer=2:referenced-offset=0:width=8"
            )
        );
        for (index, request) in
            CompilerDmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING
                .iter()
                .enumerate()
        {
            assert_eq!(
                decoded.semantic_trace[32 + index],
                format!(
                    "{SYSTEMS_DMA_BUFFER_STORE_INTEGER}:buffer={}:offset={}:width={}:value={}",
                    request.buffer_identity,
                    request.offset_bytes,
                    request.width_bytes,
                    request.value,
                )
            );
        }
        assert_eq!(
            decoded.semantic_trace[36],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_VIRTIO_SPLIT_RING_OK")
            )
        );
        decoded.semantic_trace.drain(31..37);
        for (index, request) in CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG
            .iter()
            .enumerate()
        {
            let direction = match request.direction {
                CompilerDmaDirection::Bidirectional => "bidirectional",
                CompilerDmaDirection::DeviceToMemory => "device-to-memory",
            };
            let role = match request.role {
                CompilerDmaBufferRole::VirtqueueControl => "virtqueue-control",
                CompilerDmaBufferRole::EntropyPayload => "entropy-payload",
            };
            assert_eq!(
                decoded.semantic_trace[31 + index],
                format!(
                    "{SYSTEMS_DMA_BUFFER_PREPARE_DEVICE}:buffer={}:device={}:domain={}:direction={direction}:visibility=provider-managed:role={role}",
                    request.buffer_identity,
                    request.device_identity,
                    request.translation_domain_identity,
                )
            );
        }
        assert_eq!(
            decoded.semantic_trace[33],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_DMA_DEVICE_OWNED")
            )
        );
        decoded.semantic_trace.drain(31..34);
        for (index, request) in
            CompilerPciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
                .iter()
                .enumerate()
        {
            assert_eq!(
                decoded.semantic_trace[31 + index],
                format!(
                    "{SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE}:transaction={}:segment=0:bus=0:device=5:function=0:access-capability-offset=132:bar=4:region-offset=0:region-length=4096:register-offset={}:width=8:buffer=1:buffer-offset={}:device-identity=1:domain=1",
                    index + 22,
                    request.register_offset_bytes,
                    request.buffer_offset_bytes,
                )
            );
        }
        assert_eq!(
            decoded.semantic_trace[34],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_VIRTIO_QUEUE_ADDRESSES_OK")
            )
        );
        decoded.semantic_trace.drain(31..35);
        assert!(decoded.semantic_trace[118].starts_with(SYSTEMS_FRAMES_ALLOCATE));
        assert!(decoded.semantic_trace[119].starts_with(SYSTEMS_USER_ADDRESS_SPACE_PREPARE));
        assert!(decoded.semantic_trace[120].starts_with(SYSTEMS_FRAMES_ALLOCATE));
        assert!(decoded.semantic_trace[121].starts_with(SYSTEMS_USER_ADDRESS_SPACE_MAP));
        assert!(
            decoded.semantic_trace[122].starts_with(SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE)
        );
        assert!(decoded.semantic_trace[123].starts_with(SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY));
        assert!(
            decoded.semantic_trace[124].starts_with(SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE)
        );
        assert!(decoded.semantic_trace[125].starts_with(SYSTEMS_USER_MAPPING_PERMIT_EXECUTE));
        assert!(decoded.semantic_trace[126].starts_with(SYSTEMS_FRAMES_ALLOCATE));
        assert!(decoded.semantic_trace[127].starts_with(SYSTEMS_USER_ADDRESS_SPACE_MAP));
        assert!(decoded.semantic_trace[128].starts_with(SYSTEMS_USER_MAPPING_STORE_BYTE));
        assert!(decoded.semantic_trace[129].starts_with(SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY));
        assert!(decoded.semantic_trace[130].starts_with(SYSTEMS_USER_MAPPING_LOAD_BYTE));
        assert!(decoded.semantic_trace[131].starts_with(SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA));
        assert!(decoded.semantic_trace[132].starts_with(SYSTEMS_FRAMES_ALLOCATE));
        assert!(
            decoded.semantic_trace[133].starts_with(SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS)
        );
        assert_eq!(
            decoded.semantic_trace[134],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_READY")
            )
        );
        assert_eq!(
            decoded.semantic_trace[135],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_BOOT_RESOURCE_OK")
            )
        );
        assert!(decoded.semantic_trace[136].starts_with(SYSTEMS_USER_CONTEXT_PREPARE));
        assert_eq!(
            decoded.semantic_trace[201],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_MULTIPAGE_EXEC_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[202],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_GETRANDOM_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[203],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_GETRANDOM_REPEATED_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[204],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_GETRANDOM_FULL_OUTPUT_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[205],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_GETRANDOM_SMALL_REQUEST_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[206],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_GETRANDOM_LARGE_REQUEST_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[207],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_READ_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[208],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_CLOSE_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[209],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_DUP_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[210],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_FCNTL_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[211],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_FSTAT_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[212],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_TCGETS_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[213],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_WINSIZE_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[214],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_LSEEK_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[218],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_ENTRY_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[219],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_STACK_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[220],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_AUXV_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[221],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[222],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[223],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[224],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_ELF_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[225],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_EXTERNAL_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[226],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_EXTERNAL_COPY_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[227],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_RUNTIME_EXTENT_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[228],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_RUNTIME_ENTRY_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[229],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_RUNTIME_DATA_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[230],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_BSS_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[231],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_DATA_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[232],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_WRITABLE_DATA_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[233],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_COPY_TO_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[234],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_COPY_FROM_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[235],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_SYSCALL_OK")
            )
        );
        assert_eq!(
            decoded.semantic_trace[236],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_RETAINED")
            )
        );
        assert!(
            decoded.semantic_trace[237].starts_with(SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS)
        );
        assert_eq!(decoded.semantic_trace[238], SYSTEMS_FRAMES_RELEASE);
        assert!(decoded.semantic_trace[239].starts_with(SYSTEMS_USER_ADDRESS_SPACE_UNMAP));
        assert_eq!(decoded.semantic_trace[240], SYSTEMS_FRAMES_RELEASE);
        assert!(decoded.semantic_trace[241].starts_with(SYSTEMS_USER_ADDRESS_SPACE_UNMAP));
        assert_eq!(decoded.semantic_trace[242], SYSTEMS_FRAMES_RELEASE);
        assert!(decoded.semantic_trace[243].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[244].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[246].starts_with(SYSTEMS_USER_ADDRESS_SPACE_RECLAIM));
        assert_eq!(decoded.semantic_trace[247], SYSTEMS_FRAMES_RELEASE);
        assert!(decoded.semantic_trace[248].starts_with(SYSTEMS_CONSOLE_WRITE));
        for (index, request) in
            CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
                .iter()
                .enumerate()
        {
            let identity = match request.direction {
                CompilerPciRegionRegisterDirection::ReadEquals => SYSTEMS_PCI_REGION_READ,
                CompilerPciRegionRegisterDirection::ReadEventuallyEquals => {
                    SYSTEMS_PCI_REGION_READ_EVENTUALLY
                }
                CompilerPciRegionRegisterDirection::Write => SYSTEMS_PCI_REGION_WRITE,
            };
            assert_eq!(
                decoded.semantic_trace[137 + index],
                format!(
                    "{identity}:transaction={}:segment=0:bus=0:device=5:function=0:access-capability-offset=132:bar=4:region-offset=0:region-length=4096:register-offset={}:width={}:value={:#010x}",
                    index + 25,
                    request.register_offset_bytes,
                    request.width_bytes,
                    request.value
                )
            );
        }
        assert_eq!(
            decoded.semantic_trace[141],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_VIRTIO_QUEUE_ACTIVE")
            )
        );
        let notification =
            CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION[0];
        assert_eq!(
            decoded.semantic_trace[142],
            format!(
                "{SYSTEMS_PCI_REGION_WRITE}:transaction=29:segment=0:bus=0:device=5:function=0:access-capability-offset={}:bar=4:region-offset=12288:region-length=4096:register-offset=0:width=2:value={:#010x}",
                notification.access_capability_offset_bytes, notification.value
            )
        );
        assert_eq!(
            decoded.semantic_trace[143],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_VIRTIO_QUEUE_NOTIFIED")
            )
        );
        let completion_signal =
            CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL[0];
        assert_eq!(
            decoded.semantic_trace[144],
            format!(
                "{SYSTEMS_PCI_REGION_READ_EVENTUALLY}:transaction=30:segment=0:bus=0:device=5:function=0:access-capability-offset={}:bar=4:region-offset=4096:region-length=4096:register-offset=0:width=1:value={:#010x}",
                completion_signal.access_capability_offset_bytes, completion_signal.value
            )
        );
        assert_eq!(
            decoded.semantic_trace[145],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_VIRTIO_COMPLETION_SIGNAL")
            )
        );
        assert!(decoded.semantic_trace[152].starts_with(SYSTEMS_DMA_BUFFER_COMPLETE_CPU));
        assert_eq!(
            decoded.semantic_trace[153],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_VIRTIO_PAYLOAD_CPU_OWNED")
            )
        );
        assert_eq!(
            decoded.semantic_trace[154],
            format!(
                "{SYSTEMS_ENTROPY_SAMPLE_CAPTURE_DMA}:sample=1:buffer=2:offset=0:bytes=32:device=1:domain=1:direction=device-to-memory:visibility=provider-managed:role=entropy-payload:credited-entropy-bits=0"
            )
        );
        assert_eq!(
            decoded.semantic_trace[155],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_ENTROPY_SAMPLE_CAPTURED")
            )
        );
        assert_eq!(
            decoded.semantic_trace[156],
            format!(
                "{SYSTEMS_ENTROPY_SAMPLE_ASSESS_QUALITY}:sample=1:source=1:trust-evidence=1:bytes=32:quality-parts-per-1024=1024:credited-entropy-bits=256"
            )
        );
        assert_eq!(
            decoded.semantic_trace[157],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_ENTROPY_SAMPLE_CREDITED")
            )
        );
        assert_eq!(
            decoded.semantic_trace[158],
            format!(
                "{SYSTEMS_ENTROPY_POOL_CREATE}:pool=1:credit-capacity-bits=256:initial-credited-entropy-bits=0"
            )
        );
        assert_eq!(
            decoded.semantic_trace[159],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_ENTROPY_POOL_CREATED")
            )
        );
        assert_eq!(
            decoded.semantic_trace[160],
            format!(
                "{SYSTEMS_ENTROPY_POOL_MIX_SAMPLE}:pool=1:sample=1:conditioner=1:contribution=1:bytes=32:contribution-entropy-bits=256"
            )
        );
        assert_eq!(
            decoded.semantic_trace[161],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_ENTROPY_POOL_MIXED")
            )
        );
        assert_eq!(
            decoded.semantic_trace[162],
            format!(
                "{SYSTEMS_ENTROPY_POOL_CREDIT_CONTRIBUTION}:pool=1:contribution=1:conditioner-origin=1:prior-credited-entropy-bits=0:contribution-entropy-bits=256:credit-capacity-bits=256:resulting-credited-entropy-bits=256"
            )
        );
        assert_eq!(
            decoded.semantic_trace[163],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_ENTROPY_POOL_CREDITED")
            )
        );
        assert_eq!(
            decoded.semantic_trace[164],
            format!(
                "{SYSTEMS_ENTROPY_GENERATOR_INITIALIZE}:pool=1:generator=1:algorithm=1:generation=1:key-bytes=32:required-entropy-bits=256:observed-pool-entropy-bits=256"
            )
        );
        assert_eq!(
            decoded.semantic_trace[165],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_ENTROPY_GENERATOR_READY")
            )
        );
        assert_eq!(
            decoded.semantic_trace[166],
            format!(
                "{SYSTEMS_ENTROPY_GENERATOR_EXTRACT}:generator=1:algorithm=1:prior-generation=1:resulting-generation=2:output=1:block-bytes=64:output-offset=32:output-bytes=32:counter=0:nonce=0"
            )
        );
        assert_eq!(
            decoded.semantic_trace[167],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_ENTROPY_OUTPUT_READY")
            )
        );
        assert!(
            decoded.semantic_trace[168]
                .starts_with(SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM)
        );
        assert_eq!(
            decoded.semantic_trace[169],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_AUXV_RANDOM_OK")
            )
        );
        decoded.semantic_trace[137..249].rotate_left(33);
        decoded.semantic_trace.drain(213..216);
        decoded.semantic_trace.drain(171..206);
        decoded.semantic_trace.drain(120..136);
        decoded.semantic_trace.drain(118..120);
        assert!(decoded.semantic_trace[0].starts_with(SYSTEMS_BOOTSTRAP_STORAGE_PROVISION));
        assert_eq!(decoded.semantic_trace[1], "topal.systems.entry.bootstrap/1");
        assert_eq!(decoded.semantic_trace[2], SYSTEMS_BOOT_MEMORY_DESCRIBE);
        assert_eq!(decoded.semantic_trace[3], SYSTEMS_FRAME_ALLOCATOR_CREATE);
        assert!(decoded.semantic_trace[4].starts_with(SYSTEMS_FRAMES_ALLOCATE));
        assert!(decoded.semantic_trace[5].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[6].starts_with(SYSTEMS_KERNEL_MAP));
        assert!(decoded.semantic_trace[7].starts_with(SYSTEMS_KERNEL_MAPPING_STORE_BYTE));
        assert!(decoded.semantic_trace[8].starts_with(SYSTEMS_KERNEL_MAPPING_LOAD_BYTE));
        assert_eq!(decoded.semantic_trace[9], SYSTEMS_KERNEL_UNMAP);
        assert!(decoded.semantic_trace[10].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(decoded.semantic_trace[11], SYSTEMS_FRAMES_RELEASE);
        assert!(decoded.semantic_trace[12].starts_with(SYSTEMS_TRANSLATION_BEGIN));
        assert_eq!(decoded.semantic_trace[13], SYSTEMS_TRANSLATION_COMMIT);
        assert_eq!(decoded.semantic_trace[14], SYSTEMS_TRANSLATION_ACTIVATE);
        assert!(decoded.semantic_trace[15].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[16].starts_with(SYSTEMS_FRAMES_ALLOCATE));
        assert!(decoded.semantic_trace[17].starts_with(SYSTEMS_TRANSLATION_EDIT_BEGIN));
        assert!(decoded.semantic_trace[18].starts_with(SYSTEMS_TRANSLATION_EDIT_MAP));
        assert!(decoded.semantic_trace[19].starts_with(SYSTEMS_TRANSLATION_EDIT_COMMIT));
        assert!(decoded.semantic_trace[20].starts_with(SYSTEMS_KERNEL_MAPPING_STORE_BYTE));
        assert!(decoded.semantic_trace[21].starts_with(SYSTEMS_KERNEL_MAPPING_LOAD_BYTE));
        assert!(decoded.semantic_trace[22].starts_with(SYSTEMS_TRANSLATION_EDIT_BEGIN));
        assert_eq!(decoded.semantic_trace[23], SYSTEMS_TRANSLATION_EDIT_UNMAP);
        assert!(decoded.semantic_trace[24].starts_with(SYSTEMS_TRANSLATION_EDIT_COMMIT));
        assert!(decoded.semantic_trace[25].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(decoded.semantic_trace[26], SYSTEMS_FRAMES_RELEASE);
        assert!(decoded.semantic_trace[27].starts_with(SYSTEMS_CRITICAL_ENTER));
        assert!(decoded.semantic_trace[28].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[29].starts_with(SYSTEMS_CRITICAL_RESTORE));
        assert!(decoded.semantic_trace[30].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[31].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(decoded.semantic_trace[32], SYSTEMS_DEBUG_BREAK);
        assert_eq!(
            decoded.semantic_trace[33],
            "topal.systems.entry.synchronous.debug-break/1"
        );
        assert_eq!(decoded.semantic_trace[34], SYSTEMS_RESUME_DEBUG_BREAK);
        assert!(decoded.semantic_trace[35].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[36].starts_with(SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE));
        assert!(decoded.semantic_trace[37].starts_with(SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE));
        assert!(decoded.semantic_trace[38].starts_with(SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE));
        assert!(decoded.semantic_trace[39].starts_with(SYSTEMS_ATOMIC_WORD_CREATE));
        assert!(decoded.semantic_trace[40].starts_with(SYSTEMS_ATOMIC_COMPARE_EXCHANGE));
        assert!(decoded.semantic_trace[41].starts_with(SYSTEMS_ATOMIC_LOAD));
        assert_eq!(decoded.semantic_trace[42], SYSTEMS_ATOMIC_END);
        assert!(decoded.semantic_trace[43].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[44].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[45],
            SYSTEMS_BOOTSTRAP_STORAGE_RELEASE
        );
        assert!(decoded.semantic_trace[46].starts_with(SYSTEMS_LOCAL_NOTIFICATION_SEND));
        assert!(decoded.semantic_trace[47].starts_with(SYSTEMS_LOCAL_NOTIFICATION_WAIT));
        assert_eq!(
            decoded.semantic_trace[48],
            "topal.systems.observation.local-notification/1"
        );
        assert_eq!(
            decoded.semantic_trace[49],
            "topal.systems.entry.external.local-notification/1"
        );
        assert!(decoded.semantic_trace[50].starts_with(SYSTEMS_LOCAL_NOTIFICATION_COMPLETE));
        assert_eq!(
            decoded.semantic_trace[51],
            SYSTEMS_RESUME_LOCAL_NOTIFICATION
        );
        assert!(decoded.semantic_trace[52].starts_with(SYSTEMS_LOCAL_NOTIFICATION_WAIT));
        assert!(decoded.semantic_trace[53].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(decoded.semantic_trace[54], SYSTEMS_MONOTONIC_CLOCK_NOW);
        assert_eq!(decoded.semantic_trace[55], SYSTEMS_MONOTONIC_CLOCK_NOW);
        assert!(decoded.semantic_trace[56].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[57].starts_with(SYSTEMS_DEADLINE_AFTER));
        assert!(decoded.semantic_trace[58].starts_with(SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE));
        assert!(decoded.semantic_trace[59].starts_with(SYSTEMS_KERNEL_COMPLETION_CREATE));
        assert!(decoded.semantic_trace[60].starts_with(SYSTEMS_KERNEL_CONTEXT_CREATE));
        assert!(decoded.semantic_trace[61].starts_with(SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE));
        assert!(decoded.semantic_trace[62].starts_with(SYSTEMS_KERNEL_CONTEXT_CREATE));
        assert!(decoded.semantic_trace[63].starts_with(SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE));
        assert!(decoded.semantic_trace[64].starts_with(SYSTEMS_KERNEL_CONTEXT_CREATE));
        assert!(decoded.semantic_trace[65].starts_with(SYSTEMS_KERNEL_RUNNABLE_QUEUE_CREATE));
        assert!(decoded.semantic_trace[66].starts_with(SYSTEMS_KERNEL_RUNNABLE_QUEUE_ENQUEUE));
        assert!(decoded.semantic_trace[67].starts_with(SYSTEMS_KERNEL_RUNNABLE_QUEUE_ENQUEUE));
        assert!(decoded.semantic_trace[68].starts_with(SYSTEMS_KERNEL_RUNNABLE_QUEUE_ENQUEUE));
        assert!(decoded.semantic_trace[69].starts_with(SYSTEMS_KERNEL_RUNNABLE_QUEUE_DEQUEUE));
        assert_eq!(decoded.semantic_trace[70], SYSTEMS_DEADLINE_ARM);
        assert!(
            decoded.semantic_trace[71].starts_with(SYSTEMS_KERNEL_CONTEXT_TRANSFER_UNTIL_DEADLINE)
        );
        assert!(
            decoded.semantic_trace[72].starts_with("topal.systems.entry.resumed.kernel-thread/1")
        );
        assert!(decoded.semantic_trace[73].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(
            decoded.semantic_trace[74]
                .starts_with(SYSTEMS_KERNEL_CONTEXT_AWAIT_DEADLINE_PREEMPTION)
        );
        assert_eq!(
            decoded.semantic_trace[75],
            "topal.systems.observation.deadline/1"
        );
        assert_eq!(
            decoded.semantic_trace[76],
            "topal.systems.entry.external.deadline/1"
        );
        assert_eq!(decoded.semantic_trace[77], SYSTEMS_DEADLINE_COMPLETE);
        assert!(decoded.semantic_trace[78].starts_with(SYSTEMS_KERNEL_CONTEXT_PREEMPT_CURRENT));
        assert!(
            decoded.semantic_trace[79].starts_with(SYSTEMS_KERNEL_CONTEXT_TRANSFER_UNTIL_DEADLINE)
        );
        assert!(decoded.semantic_trace[80].starts_with(SYSTEMS_KERNEL_CONTEXT_TAKE_PREEMPTED));
        assert!(decoded.semantic_trace[81].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[82].starts_with(SYSTEMS_KERNEL_RUNNABLE_QUEUE_ENQUEUE));
        assert!(decoded.semantic_trace[83].starts_with(SYSTEMS_KERNEL_RUNNABLE_QUEUE_DEQUEUE));
        assert!(decoded.semantic_trace[84].starts_with(SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE));
        assert!(
            decoded.semantic_trace[85].starts_with("topal.systems.entry.resumed.kernel-thread/1")
        );
        assert!(decoded.semantic_trace[86].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[87].starts_with(SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE));
        assert!(decoded.semantic_trace[88].starts_with(SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE));
        assert!(decoded.semantic_trace[89].starts_with(SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE));
        assert!(decoded.semantic_trace[90].starts_with(SYSTEMS_KERNEL_RUNNABLE_QUEUE_DEQUEUE));
        assert!(decoded.semantic_trace[91].starts_with(SYSTEMS_KERNEL_CONTEXT_TRANSFER));
        assert!(
            decoded.semantic_trace[92].starts_with("topal.systems.entry.resumed.kernel-thread/1")
        );
        assert!(decoded.semantic_trace[93].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[94].starts_with(SYSTEMS_KERNEL_CONTEXT_RETIRE));
        assert!(decoded.semantic_trace[95].starts_with(SYSTEMS_KERNEL_CONTEXT_TRANSFER));
        assert!(decoded.semantic_trace[96].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[97].starts_with(SYSTEMS_KERNEL_CONTEXT_RECLAIM));
        assert!(decoded.semantic_trace[98].starts_with(SYSTEMS_KERNEL_COMPLETION_SIGNAL));
        assert!(decoded.semantic_trace[99].starts_with(SYSTEMS_KERNEL_RUNNABLE_QUEUE_ENQUEUE));
        assert!(decoded.semantic_trace[100].starts_with(SYSTEMS_KERNEL_RUNNABLE_QUEUE_DEQUEUE));
        assert!(decoded.semantic_trace[101].starts_with(SYSTEMS_KERNEL_CONTEXT_TRANSFER));
        assert!(decoded.semantic_trace[102].starts_with(SYSTEMS_KERNEL_CONTEXT_TRANSFER));
        assert!(decoded.semantic_trace[103].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[104].starts_with(SYSTEMS_KERNEL_CONTEXT_RETIRE));
        assert!(decoded.semantic_trace[105].starts_with(SYSTEMS_KERNEL_CONTEXT_TRANSFER));
        assert!(decoded.semantic_trace[106].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[107].starts_with(SYSTEMS_KERNEL_CONTEXT_RECLAIM));
        assert!(decoded.semantic_trace[108].starts_with(SYSTEMS_KERNEL_RUNNABLE_QUEUE_DEQUEUE));
        assert!(decoded.semantic_trace[109].starts_with(SYSTEMS_KERNEL_CONTEXT_TRANSFER));
        assert!(decoded.semantic_trace[110].starts_with(SYSTEMS_KERNEL_COMPLETION_SIGNAL));
        assert!(decoded.semantic_trace[111].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[112].starts_with(SYSTEMS_KERNEL_CONTEXT_RETIRE));
        assert!(decoded.semantic_trace[113].starts_with(SYSTEMS_KERNEL_CONTEXT_TRANSFER));
        assert!(decoded.semantic_trace[114].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[115].starts_with(SYSTEMS_KERNEL_CONTEXT_RECLAIM));
        assert!(decoded.semantic_trace[116].starts_with(SYSTEMS_KERNEL_COMPLETION_CONSUME));
        assert_eq!(
            decoded.semantic_trace[117],
            SYSTEMS_KERNEL_RUNNABLE_QUEUE_CONSUME
        );
        assert!(decoded.semantic_trace[118].starts_with(SYSTEMS_USER_CONTEXT_PREPARE));
        assert!(decoded.semantic_trace[119].starts_with(SYSTEMS_ENTROPY_GENERATOR_EXTRACT));
        assert!(
            decoded.semantic_trace[120]
                .starts_with(SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM)
        );
        assert!(decoded.semantic_trace[121].starts_with(SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE));
        decoded.semantic_trace.remove(121);
        assert!(
            decoded.semantic_trace[121]
                .starts_with(SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS)
        );
        decoded.semantic_trace.remove(121);
        assert!(
            decoded.semantic_trace[121]
                .starts_with(SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS)
        );
        decoded.semantic_trace.remove(121);
        assert!(decoded.semantic_trace[121].starts_with(SYSTEMS_USER_CONTEXT_ENTER));
        assert!(decoded.semantic_trace[122].starts_with("topal.systems.entry.user-syscall/1"));
        assert!(
            decoded.semantic_trace[123]
                .starts_with("topal.systems.disposition.user-syscall.return/1")
        );
        assert!(decoded.semantic_trace[124].starts_with("topal.systems.entry.user-syscall/1"));
        assert!(decoded.semantic_trace[125].starts_with(SYSTEMS_USER_TRANSFER_BEGIN));
        assert!(decoded.semantic_trace[126].starts_with(SYSTEMS_USER_COPY_TO));
        assert!(decoded.semantic_trace[127].starts_with(SYSTEMS_USER_TRANSFER_END));
        assert!(
            decoded.semantic_trace[128]
                .starts_with("topal.systems.disposition.user-syscall.return/1")
        );
        assert!(decoded.semantic_trace[129].starts_with("topal.systems.entry.user-syscall/1"));
        assert!(decoded.semantic_trace[130].starts_with(SYSTEMS_USER_TRANSFER_BEGIN));
        assert!(decoded.semantic_trace[131].starts_with(SYSTEMS_USER_COPY_TO));
        assert!(decoded.semantic_trace[132].starts_with(SYSTEMS_USER_TRANSFER_END));
        assert!(
            decoded.semantic_trace[133]
                .starts_with("topal.systems.disposition.user-syscall.return/1")
        );
        assert!(decoded.semantic_trace[134].starts_with("topal.systems.entry.user-syscall/1"));
        assert!(decoded.semantic_trace[135].starts_with(SYSTEMS_USER_TRANSFER_BEGIN));
        assert!(decoded.semantic_trace[136].starts_with(SYSTEMS_USER_COPY_FROM));
        assert!(decoded.semantic_trace[137].starts_with(SYSTEMS_USER_TRANSFER_END));
        assert!(
            decoded.semantic_trace[138]
                .starts_with("topal.systems.disposition.user-syscall.return/1")
        );
        assert!(decoded.semantic_trace[139].starts_with("topal.systems.entry.user-syscall/1"));
        assert!(decoded.semantic_trace[140].starts_with(SYSTEMS_USER_TRANSFER_BEGIN));
        assert!(decoded.semantic_trace[141].starts_with(SYSTEMS_USER_COPY_FROM));
        assert!(decoded.semantic_trace[142].starts_with(SYSTEMS_USER_TRANSFER_END));
        assert!(
            decoded.semantic_trace[143]
                .starts_with("topal.systems.disposition.user-syscall.return/1")
        );
        assert!(decoded.semantic_trace[144].starts_with("topal.systems.entry.user-syscall/1"));
        assert!(
            decoded.semantic_trace[145]
                .starts_with("topal.systems.disposition.user-syscall.terminate/1")
        );
        assert!(decoded.semantic_trace[146].starts_with(SYSTEMS_USER_CONTEXT_ENTER));
        assert_eq!(
            decoded.semantic_trace[147],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_IMAGE_MULTIPAGE_EXEC_OK")
            )
        );
        decoded.semantic_trace.remove(147);
        assert!(decoded.semantic_trace[147].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert_eq!(
            decoded.semantic_trace[148],
            format!(
                "{SYSTEMS_CONSOLE_WRITE}:sha256:{}",
                sha256(b"TOPAL_KERNEL_USER_GETRANDOM_REPEATED_OK")
            )
        );
        decoded.semantic_trace.remove(148);
        assert!(decoded.semantic_trace[148].starts_with(SYSTEMS_USER_ADDRESS_SPACE_UNMAP));
        assert_eq!(decoded.semantic_trace[149], SYSTEMS_FRAMES_RELEASE);
        assert!(decoded.semantic_trace[150].starts_with(SYSTEMS_USER_ADDRESS_SPACE_UNMAP));
        assert_eq!(decoded.semantic_trace[151], SYSTEMS_FRAMES_RELEASE);
        assert!(decoded.semantic_trace[152].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(decoded.semantic_trace[153].starts_with(SYSTEMS_CONSOLE_WRITE));
        assert!(
            decoded.semantic_trace[154].starts_with("topal.systems.context.user.complete/1"),
            "normalized user tail: {:?}",
            &decoded.semantic_trace[148..=158]
        );
        assert!(decoded.semantic_trace[188].starts_with(SYSTEMS_FATAL));
        let repeated_destination = parent.join("repeated");
        let repeated =
            publish_x86_64_systems_artifact(&program(), &tools, &repeated_destination).unwrap();
        for (first, second) in [
            (&published.kernel, &repeated.kernel),
            (&published.debug, &repeated.debug),
            (&published.map, &repeated.map),
            (&published.initial_user_image, &repeated.initial_user_image),
            (&published.provenance, &repeated.provenance),
        ] {
            assert_eq!(fs::read(first).unwrap(), fs::read(second).unwrap());
        }

        let static_destination = parent.join("with-static-user");
        let static_image = busybox_shaped_static_elf();
        let with_static = publish_x86_64_systems_artifact_with_static_user_image(
            &program(),
            &tools,
            &static_image,
            &static_destination,
        )
        .unwrap();
        assert_eq!(fs::read_dir(&static_destination).unwrap().count(), 6);
        let static_path = with_static.static_user_image.unwrap();
        assert_eq!(fs::read(&static_path).unwrap(), static_image);
        assert_eq!(with_static.record.outputs.len(), 5);
        assert_eq!(
            with_static
                .record
                .outputs
                .iter()
                .find(|output| output.identity == SYSTEMS_STATIC_USER_IMAGE_FILE)
                .unwrap()
                .sha256,
            sha256(&static_image)
        );
        fs::remove_dir_all(parent).unwrap();
    }
}
