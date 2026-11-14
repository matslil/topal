//! Architecture-independent reference model for the initial systems profile.

use std::fmt;

use crate::{
    AtomicCompareExchangeResult, AtomicOrder, AtomicWordLocation, AtomicWordRequest,
    BootstrapRegion, BootstrapStorageDescriptor, BootstrapStorageRequest, BootstrapStorageState,
    CompletedUserExecution, CriticalDomain, INITIAL_USER_TIMESPEC_FAULT_CANDIDATE,
    INITIAL_USER_TIMESPEC_VALID_CANDIDATE, INITIAL_USER_UNAME_FAULT_CANDIDATE,
    INITIAL_USER_UNAME_VALID_CANDIDATE, KernelMappingRequest, LINUX_NEW_UTSNAME_BYTES,
    LINUX_TIMESPEC_BYTES, LINUX_X86_64_EFAULT_RESULT, LINUX_X86_64_NANOSLEEP, LINUX_X86_64_UNAME,
    LinuxTimespec, LinuxUtsName, LocalInterruptMaskState, LocalNotificationProtocol,
    PhysicalFrameRequest, PreparedUserAddressSpace, ReservedAnonymousUserMapping,
    RevokedUserAddressSpace, SYSTEMS_ATOMIC_COMPARE_EXCHANGE, SYSTEMS_ATOMIC_END,
    SYSTEMS_ATOMIC_LOAD, SYSTEMS_ATOMIC_WORD_CREATE, SYSTEMS_BOOT_MEMORY_DESCRIBE,
    SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE, SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE,
    SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE, SYSTEMS_BOOTSTRAP_STORAGE_PROVISION,
    SYSTEMS_BOOTSTRAP_STORAGE_RELEASE, SYSTEMS_CRITICAL_ENTER, SYSTEMS_CRITICAL_RESTORE,
    SYSTEMS_DEADLINE_AFTER, SYSTEMS_DEADLINE_ARM, SYSTEMS_DEADLINE_COMPLETE, SYSTEMS_DEADLINE_WAIT,
    SYSTEMS_FRAME_ALLOCATOR_CREATE, SYSTEMS_FRAMES_ALLOCATE, SYSTEMS_FRAMES_RELEASE,
    SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE, SYSTEMS_KERNEL_COMPLETION_CONSUME,
    SYSTEMS_KERNEL_COMPLETION_CREATE, SYSTEMS_KERNEL_COMPLETION_SIGNAL,
    SYSTEMS_KERNEL_CONTEXT_AWAIT_DEADLINE_PREEMPTION, SYSTEMS_KERNEL_CONTEXT_CREATE,
    SYSTEMS_KERNEL_CONTEXT_PREEMPT_CURRENT, SYSTEMS_KERNEL_CONTEXT_RECLAIM,
    SYSTEMS_KERNEL_CONTEXT_RETIRE, SYSTEMS_KERNEL_CONTEXT_TAKE_PREEMPTED,
    SYSTEMS_KERNEL_CONTEXT_TRANSFER, SYSTEMS_KERNEL_CONTEXT_TRANSFER_UNTIL_DEADLINE,
    SYSTEMS_KERNEL_IMAGE_DISCARD_BACKING, SYSTEMS_KERNEL_IMAGE_RESERVE_BACKING, SYSTEMS_KERNEL_MAP,
    SYSTEMS_KERNEL_MAPPING_LOAD_BYTE, SYSTEMS_KERNEL_MAPPING_STORE_BYTE,
    SYSTEMS_KERNEL_RUNNABLE_QUEUE_CONSUME, SYSTEMS_KERNEL_RUNNABLE_QUEUE_CREATE,
    SYSTEMS_KERNEL_RUNNABLE_QUEUE_DEQUEUE, SYSTEMS_KERNEL_RUNNABLE_QUEUE_ENQUEUE,
    SYSTEMS_KERNEL_THREAD_ENTRY, SYSTEMS_KERNEL_UNMAP, SYSTEMS_LOCAL_NOTIFICATION_COMPLETE,
    SYSTEMS_LOCAL_NOTIFICATION_SEND, SYSTEMS_LOCAL_NOTIFICATION_WAIT, SYSTEMS_MONOTONIC_CLOCK_NOW,
    SYSTEMS_RESUME_DEADLINE, SYSTEMS_RESUME_LOCAL_NOTIFICATION,
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
    SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESERVE_HIERARCHY,
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
    SYSTEMS_STATIC_IMAGE_PROCESS_RESTORE_ENTRY, SYSTEMS_TRANSLATION_ACTIVATE,
    SYSTEMS_TRANSLATION_BEGIN, SYSTEMS_TRANSLATION_COMMIT, SYSTEMS_TRANSLATION_EDIT_BEGIN,
    SYSTEMS_TRANSLATION_EDIT_COMMIT, SYSTEMS_TRANSLATION_EDIT_MAP, SYSTEMS_TRANSLATION_EDIT_UNMAP,
    SYSTEMS_USER_ADDRESS_SPACE_MAP, SYSTEMS_USER_ADDRESS_SPACE_PREPARE,
    SYSTEMS_USER_ADDRESS_SPACE_RECLAIM, SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS,
    SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS, SYSTEMS_USER_ADDRESS_SPACE_UNMAP,
    SYSTEMS_USER_ADDRESS_SPACE_VERIFY_MAPPING, SYSTEMS_USER_CONTEXT_COMPLETE,
    SYSTEMS_USER_CONTEXT_ENTER, SYSTEMS_USER_CONTEXT_PREPARE, SYSTEMS_USER_COPY_FROM,
    SYSTEMS_USER_COPY_TO, SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE,
    SYSTEMS_USER_MAPPING_LOAD_BYTE, SYSTEMS_USER_MAPPING_PERMIT_EXECUTE,
    SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA, SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY,
    SYSTEMS_USER_MAPPING_SEAL_IMMUTABLE_DATA, SYSTEMS_USER_MAPPING_STORE_BYTE,
    SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE, SYSTEMS_USER_SYSCALL_DISPATCH,
    SYSTEMS_USER_SYSCALL_ENTRY, SYSTEMS_USER_SYSCALL_RETURN, SYSTEMS_USER_TERMINATE,
    SYSTEMS_USER_TRANSFER_BEGIN, SYSTEMS_USER_TRANSFER_END, SuspendedUserContext,
    SystemsUserExecutionProtocol, TranslationEditKind, TranslationMappingRequest,
    TranslationUpdateRequest, UserMapping,
};

pub const INITIAL_SYSTEMS_TARGET: &str = "x86_64-unknown-none";
pub const INITIAL_SYSTEMS_BOARD: &str = "topal-qemu-pc-q35-10.2";
pub const INITIAL_SYSTEMS_PROFILE: &str = "topal.systems.x86_64-qemu-pc-q35-10.2/1";
pub const SYSTEMS_CONSOLE_WRITE: &str = "topal.systems.device.console.write/1";
pub const SYSTEMS_REGION_LIBRARY_APPLY: &str = "topal.systems.region-library.apply/1";
pub const SYSTEMS_PCI_CONFIGURATION_READ_DWORD: &str =
    "topal.systems.device.pci-configuration.read-dword/1";
pub const SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD: &str =
    "topal.systems.device.pci-configuration.update-word/1";
pub const SYSTEMS_DMA_BUFFER_CREATE: &str = "topal.systems.dma.buffer.create/1";
pub const SYSTEMS_DMA_BUFFER_BIND_REFERENCE: &str = "topal.systems.dma.buffer.bind-reference/1";
pub const SYSTEMS_DMA_BUFFER_STORE_INTEGER: &str = "topal.systems.dma.buffer.store-integer/1";
pub const SYSTEMS_DMA_BUFFER_PREPARE_DEVICE: &str = "topal.systems.dma.buffer.prepare-device/1";
pub const SYSTEMS_DMA_BUFFER_COMPLETE_CPU: &str = "topal.systems.dma.buffer.complete-cpu/1";
pub const SYSTEMS_DMA_BUFFER_LOAD_INTEGER: &str = "topal.systems.dma.buffer.load-integer/1";
pub const SYSTEMS_ENTROPY_SAMPLE_CAPTURE_DMA: &str = "topal.systems.entropy.sample.capture-dma/1";
pub const SYSTEMS_ENTROPY_SAMPLE_ASSESS_QUALITY: &str =
    "topal.systems.entropy.sample.assess-quality/1";
pub const SYSTEMS_ENTROPY_POOL_CREATE: &str = "topal.systems.entropy.pool.create/1";
pub const SYSTEMS_ENTROPY_POOL_MIX_SAMPLE: &str = "topal.systems.entropy.pool.mix-sample/1";
pub const SYSTEMS_ENTROPY_POOL_CREDIT_CONTRIBUTION: &str =
    "topal.systems.entropy.pool.credit-contribution/1";
pub const SYSTEMS_ENTROPY_GENERATOR_INITIALIZE: &str =
    "topal.systems.entropy.generator.initialize/1";
pub const SYSTEMS_ENTROPY_GENERATOR_EXTRACT: &str = "topal.systems.entropy.generator.extract/1";
pub const SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM: &str =
    "topal.systems.entropy.output.deliver-initial-auxv-random/1";
pub const SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM: &str =
    "topal.systems.entropy.output.reserve-initial-user-random/1";
pub const SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE: &str =
    "topal.systems.entropy.user-random.bind-reusable/1";
pub const SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS: &str =
    "topal.systems.entropy.user-random.expand-small-requests/1";
pub const SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS: &str =
    "topal.systems.entropy.user-random.expand-large-requests/1";
pub const SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE: &str =
    "topal.systems.device.pci-region.write-dma-reference/1";
pub const SYSTEMS_PCI_REGION_READ: &str = "topal.systems.device.pci-region.read/1";
pub const SYSTEMS_PCI_REGION_READ_EVENTUALLY: &str =
    "topal.systems.device.pci-region.read-eventually/1";
pub const SYSTEMS_PCI_REGION_WRITE: &str = "topal.systems.device.pci-region.write/1";
pub const SYSTEMS_DEBUG_BREAK: &str = "topal.systems.machine.debug-break/1";
pub const SYSTEMS_RESUME_DEBUG_BREAK: &str = "topal.systems.disposition.resume-debug-break/1";
pub const SYSTEMS_FATAL: &str = "topal.systems.disposition.fatal/1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsTargetSelection {
    pub target: String,
    pub board: String,
    pub profile: String,
}

impl SystemsTargetSelection {
    #[must_use]
    pub fn initial_x86_64_qemu() -> Self {
        Self {
            target: INITIAL_SYSTEMS_TARGET.into(),
            board: INITIAL_SYSTEMS_BOARD.into(),
            profile: INITIAL_SYSTEMS_PROFILE.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemsEntryKind {
    Bootstrap,
    SynchronousExceptionDebugBreak,
    ExternalInterruptLocalNotification,
    ExternalInterruptDeadline,
    ResumedKernelThread,
    UserSyscall,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemsContextKind {
    Bootstrap,
    DebugBreak,
    LocalNotificationInterrupt,
    DeadlineInterrupt,
    KernelThread,
    UserSyscall,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PciConfigurationDwordRequest {
    pub segment: u16,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub register_offset_bytes: u8,
    pub expected_value: u32,
}

impl PciConfigurationDwordRequest {
    pub const INITIAL_Q35_VIRTIO_RNG_DISCOVERY: [Self; 24] = [
        Self::initial_q35_virtio_rng(0x00, 0x1044_1af4),
        Self::initial_q35_virtio_rng(0x34, 0x0000_0098),
        Self::initial_q35_virtio_rng(0x98, 0x0001_8411),
        Self::initial_q35_virtio_rng(0x84, 0x0514_7009),
        Self::initial_q35_virtio_rng(0x88, 0x0000_0000),
        Self::initial_q35_virtio_rng(0x8c, 0x0000_0000),
        Self::initial_q35_virtio_rng(0x90, 0x0000_0000),
        Self::initial_q35_virtio_rng(0x70, 0x0214_6009),
        Self::initial_q35_virtio_rng(0x74, 0x0000_0004),
        Self::initial_q35_virtio_rng(0x78, 0x0000_3000),
        Self::initial_q35_virtio_rng(0x7c, 0x0000_1000),
        Self::initial_q35_virtio_rng(0x80, 0x0000_0004),
        Self::initial_q35_virtio_rng(0x60, 0x0410_5009),
        Self::initial_q35_virtio_rng(0x64, 0x0000_0004),
        Self::initial_q35_virtio_rng(0x68, 0x0000_2000),
        Self::initial_q35_virtio_rng(0x6c, 0x0000_1000),
        Self::initial_q35_virtio_rng(0x50, 0x0310_4009),
        Self::initial_q35_virtio_rng(0x54, 0x0000_0004),
        Self::initial_q35_virtio_rng(0x58, 0x0000_1000),
        Self::initial_q35_virtio_rng(0x5c, 0x0000_1000),
        Self::initial_q35_virtio_rng(0x40, 0x0110_0009),
        Self::initial_q35_virtio_rng(0x44, 0x0000_0004),
        Self::initial_q35_virtio_rng(0x48, 0x0000_0000),
        Self::initial_q35_virtio_rng(0x4c, 0x0000_1000),
    ];

    #[must_use]
    pub const fn initial_q35_virtio_rng(register_offset_bytes: u8, expected_value: u32) -> Self {
        Self {
            segment: 0,
            bus: 0,
            device: 5,
            function: 0,
            register_offset_bytes,
            expected_value,
        }
    }

    #[must_use]
    pub const fn initial_q35_virtio_rng_identity() -> Self {
        Self::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[0]
    }

    #[must_use]
    pub const fn is_well_formed(self) -> bool {
        self.device <= 31
            && self.function <= 7
            && self.register_offset_bytes <= 252
            && self.register_offset_bytes % 4 == 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PciConfigurationWordUpdateRequest {
    pub segment: u16,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub register_offset_bytes: u8,
    pub mask: u16,
    pub expected_masked_value: u16,
    pub desired_masked_value: u16,
}

impl PciConfigurationWordUpdateRequest {
    pub const INITIAL_Q35_VIRTIO_RNG_BUS_MASTER: Self = Self {
        segment: 0,
        bus: 0,
        device: 5,
        function: 0,
        register_offset_bytes: 0x04,
        mask: 0x0004,
        expected_masked_value: 0,
        desired_masked_value: 0x0004,
    };

    #[must_use]
    pub const fn is_well_formed(self) -> bool {
        self.device <= 31
            && self.function <= 7
            && self.register_offset_bytes <= 254
            && self.register_offset_bytes % 2 == 0
            && self.mask != 0
            && self.expected_masked_value & !self.mask == 0
            && self.desired_masked_value & !self.mask == 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DmaDirection {
    Bidirectional,
    DeviceToMemory,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DmaVisibilityPolicy {
    ProviderManaged,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DmaBufferRole {
    VirtqueueControl,
    EntropyPayload,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DmaBufferRequest {
    pub buffer_identity: u64,
    pub device_identity: u64,
    pub translation_domain_identity: u64,
    pub byte_count: u64,
    pub alignment_bytes: u64,
    pub direction: DmaDirection,
    pub visibility: DmaVisibilityPolicy,
    pub role: DmaBufferRole,
}

impl DmaBufferRequest {
    pub const INITIAL_Q35_VIRTIO_RNG: [Self; 2] = [
        Self {
            buffer_identity: 1,
            device_identity: 1,
            translation_domain_identity: 1,
            byte_count: 4096,
            alignment_bytes: 4096,
            direction: DmaDirection::Bidirectional,
            visibility: DmaVisibilityPolicy::ProviderManaged,
            role: DmaBufferRole::VirtqueueControl,
        },
        Self {
            buffer_identity: 2,
            device_identity: 1,
            translation_domain_identity: 1,
            byte_count: 4096,
            alignment_bytes: 4096,
            direction: DmaDirection::DeviceToMemory,
            visibility: DmaVisibilityPolicy::ProviderManaged,
            role: DmaBufferRole::EntropyPayload,
        },
    ];

    #[must_use]
    pub const fn is_well_formed(self) -> bool {
        self.buffer_identity != 0
            && self.device_identity != 0
            && self.translation_domain_identity != 0
            && self.byte_count != 0
            && self.alignment_bytes.is_power_of_two()
            && self.byte_count % self.alignment_bytes == 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DmaBufferReferenceRequest {
    pub destination_buffer_identity: u64,
    pub destination_offset_bytes: u64,
    pub referenced_buffer_identity: u64,
    pub referenced_offset_bytes: u64,
    pub width_bytes: u8,
}

impl DmaBufferReferenceRequest {
    pub const INITIAL_Q35_VIRTIO_RNG_DESCRIPTOR: Self = Self {
        destination_buffer_identity: 1,
        destination_offset_bytes: 0,
        referenced_buffer_identity: 2,
        referenced_offset_bytes: 0,
        width_bytes: 8,
    };

    #[must_use]
    pub const fn is_well_formed(self) -> bool {
        self.destination_buffer_identity != 0
            && self.referenced_buffer_identity != 0
            && self.width_bytes == 8
            && self.destination_offset_bytes <= 4096 - 8
            && self.destination_offset_bytes.is_multiple_of(8)
            && self.referenced_offset_bytes < 4096
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DmaBufferIntegerStoreRequest {
    pub buffer_identity: u64,
    pub offset_bytes: u64,
    pub width_bytes: u8,
    pub value: u64,
}

impl DmaBufferIntegerStoreRequest {
    pub const INITIAL_Q35_VIRTIO_RNG_SPLIT_RING: [Self; 4] = [
        Self {
            buffer_identity: 1,
            offset_bytes: 8,
            width_bytes: 4,
            value: 32,
        },
        Self {
            buffer_identity: 1,
            offset_bytes: 12,
            width_bytes: 2,
            value: 2,
        },
        Self {
            buffer_identity: 1,
            offset_bytes: 132,
            width_bytes: 2,
            value: 0,
        },
        Self {
            buffer_identity: 1,
            offset_bytes: 130,
            width_bytes: 2,
            value: 1,
        },
    ];

    #[must_use]
    pub const fn is_well_formed(self) -> bool {
        let width = self.width_bytes as u64;
        self.buffer_identity != 0
            && matches!(self.width_bytes, 1 | 2 | 4 | 8)
            && self.offset_bytes <= 4096 - width
            && self.offset_bytes.is_multiple_of(width)
            && (self.width_bytes == 8 || self.value < (1_u64 << (self.width_bytes * 8)))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DmaBufferPreparationRequest {
    pub buffer_identity: u64,
    pub device_identity: u64,
    pub translation_domain_identity: u64,
    pub direction: DmaDirection,
    pub visibility: DmaVisibilityPolicy,
    pub role: DmaBufferRole,
}

impl DmaBufferPreparationRequest {
    pub const INITIAL_Q35_VIRTIO_RNG: [Self; 2] = [
        Self {
            buffer_identity: 2,
            device_identity: 1,
            translation_domain_identity: 1,
            direction: DmaDirection::DeviceToMemory,
            visibility: DmaVisibilityPolicy::ProviderManaged,
            role: DmaBufferRole::EntropyPayload,
        },
        Self {
            buffer_identity: 1,
            device_identity: 1,
            translation_domain_identity: 1,
            direction: DmaDirection::Bidirectional,
            visibility: DmaVisibilityPolicy::ProviderManaged,
            role: DmaBufferRole::VirtqueueControl,
        },
    ];

    #[must_use]
    pub const fn is_well_formed(self) -> bool {
        self.buffer_identity != 0
            && self.device_identity != 0
            && self.translation_domain_identity != 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DmaBufferIntegerLoadRequest {
    pub buffer_identity: u64,
    pub offset_bytes: u64,
    pub width_bytes: u8,
    pub expected_value: u64,
}

impl DmaBufferIntegerLoadRequest {
    pub const INITIAL_Q35_VIRTIO_RNG_USED_RING: [Self; 4] = [
        Self {
            buffer_identity: 1,
            offset_bytes: 148,
            width_bytes: 2,
            expected_value: 0,
        },
        Self {
            buffer_identity: 1,
            offset_bytes: 150,
            width_bytes: 2,
            expected_value: 1,
        },
        Self {
            buffer_identity: 1,
            offset_bytes: 152,
            width_bytes: 4,
            expected_value: 0,
        },
        Self {
            buffer_identity: 1,
            offset_bytes: 156,
            width_bytes: 4,
            expected_value: 32,
        },
    ];

    #[must_use]
    pub const fn is_well_formed(self) -> bool {
        let width = self.width_bytes as u64;
        self.buffer_identity != 0
            && matches!(self.width_bytes, 1 | 2 | 4 | 8)
            && self.offset_bytes <= 4096 - width
            && self.offset_bytes.is_multiple_of(width)
            && (self.width_bytes == 8 || self.expected_value < (1_u64 << (self.width_bytes * 8)))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntropySampleCaptureRequest {
    pub sample_identity: u64,
    pub buffer_identity: u64,
    pub offset_bytes: u64,
    pub byte_count: u64,
    pub device_identity: u64,
    pub translation_domain_identity: u64,
    pub direction: DmaDirection,
    pub visibility: DmaVisibilityPolicy,
    pub role: DmaBufferRole,
    pub credited_entropy_bits: u64,
}

impl EntropySampleCaptureRequest {
    pub const INITIAL_Q35_VIRTIO_RNG: Self = Self {
        sample_identity: 1,
        buffer_identity: 2,
        offset_bytes: 0,
        byte_count: 32,
        device_identity: 1,
        translation_domain_identity: 1,
        direction: DmaDirection::DeviceToMemory,
        visibility: DmaVisibilityPolicy::ProviderManaged,
        role: DmaBufferRole::EntropyPayload,
        credited_entropy_bits: 0,
    };

    #[must_use]
    pub const fn is_well_formed(self) -> bool {
        self.sample_identity != 0
            && self.buffer_identity != 0
            && self.byte_count != 0
            && self.offset_bytes <= 4096
            && self.byte_count <= 4096 - self.offset_bytes
            && self.device_identity != 0
            && self.translation_domain_identity != 0
            && self.credited_entropy_bits <= self.byte_count.saturating_mul(8)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntropySampleQualityRequest {
    pub sample_identity: u64,
    pub source_identity: u64,
    pub trust_evidence_identity: u64,
    pub byte_count: u64,
    pub quality_parts_per_1024: u64,
    pub credited_entropy_bits: u64,
}

impl EntropySampleQualityRequest {
    pub const INITIAL_Q35_VIRTIO_RNG: Self = Self {
        sample_identity: 1,
        source_identity: 1,
        trust_evidence_identity: 1,
        byte_count: 32,
        quality_parts_per_1024: 1024,
        credited_entropy_bits: 256,
    };

    #[must_use]
    pub const fn is_well_formed(self) -> bool {
        let sample_bits = (self.byte_count as u128) * 8;
        let derived_credit = sample_bits * (self.quality_parts_per_1024 as u128) / 1024;
        self.sample_identity != 0
            && self.source_identity != 0
            && self.trust_evidence_identity != 0
            && self.byte_count != 0
            && self.quality_parts_per_1024 <= 1024
            && derived_credit <= sample_bits
            && derived_credit == self.credited_entropy_bits as u128
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntropyPoolRequest {
    pub pool_identity: u64,
    pub credit_capacity_bits: u64,
    pub initial_credited_entropy_bits: u64,
}

impl EntropyPoolRequest {
    pub const INITIAL_Q35: Self = Self {
        pool_identity: 1,
        credit_capacity_bits: 256,
        initial_credited_entropy_bits: 0,
    };

    #[must_use]
    pub const fn is_well_formed(self) -> bool {
        self.pool_identity != 0
            && self.credit_capacity_bits != 0
            && self.initial_credited_entropy_bits == 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntropyPoolMixRequest {
    pub pool_identity: u64,
    pub sample_identity: u64,
    pub conditioner_identity: u64,
    pub contribution_identity: u64,
    pub byte_count: u64,
    pub contribution_entropy_bits: u64,
}

impl EntropyPoolMixRequest {
    pub const INITIAL_Q35: Self = Self {
        pool_identity: 1,
        sample_identity: 1,
        conditioner_identity: 1,
        contribution_identity: 1,
        byte_count: 32,
        contribution_entropy_bits: 256,
    };

    #[must_use]
    pub const fn is_well_formed(self) -> bool {
        self.pool_identity != 0
            && self.sample_identity != 0
            && self.conditioner_identity != 0
            && self.contribution_identity != 0
            && self.byte_count != 0
            && self.contribution_entropy_bits != 0
            && self.contribution_entropy_bits <= self.byte_count.saturating_mul(8)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntropyPoolCreditRequest {
    pub pool_identity: u64,
    pub contribution_identity: u64,
    pub conditioner_origin_identity: u64,
    pub prior_credited_entropy_bits: u64,
    pub contribution_entropy_bits: u64,
    pub credit_capacity_bits: u64,
    pub resulting_credited_entropy_bits: u64,
}

impl EntropyPoolCreditRequest {
    pub const INITIAL_Q35: Self = Self {
        pool_identity: 1,
        contribution_identity: 1,
        conditioner_origin_identity: 1,
        prior_credited_entropy_bits: 0,
        contribution_entropy_bits: 256,
        credit_capacity_bits: 256,
        resulting_credited_entropy_bits: 256,
    };

    #[must_use]
    pub const fn is_well_formed(self) -> bool {
        self.pool_identity != 0
            && self.contribution_identity != 0
            && self.conditioner_origin_identity != 0
            && self.contribution_entropy_bits != 0
            && self.credit_capacity_bits != 0
            && match self
                .prior_credited_entropy_bits
                .checked_add(self.contribution_entropy_bits)
            {
                Some(result) => {
                    result == self.resulting_credited_entropy_bits
                        && result <= self.credit_capacity_bits
                }
                None => false,
            }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntropyGeneratorRequest {
    pub pool_identity: u64,
    pub generator_identity: u64,
    pub algorithm_identity: u64,
    pub generation: u64,
    pub key_byte_count: u64,
    pub required_entropy_bits: u64,
    pub observed_pool_entropy_bits: u64,
}

impl EntropyGeneratorRequest {
    pub const INITIAL_Q35: Self = Self {
        pool_identity: 1,
        generator_identity: 1,
        algorithm_identity: 1,
        generation: 1,
        key_byte_count: 32,
        required_entropy_bits: 256,
        observed_pool_entropy_bits: 256,
    };

    #[must_use]
    pub const fn is_well_formed(self) -> bool {
        self.pool_identity != 0
            && self.generator_identity != 0
            && self.algorithm_identity != 0
            && self.generation != 0
            && self.key_byte_count != 0
            && self.required_entropy_bits != 0
            && self.observed_pool_entropy_bits >= self.required_entropy_bits
            && self.required_entropy_bits <= self.key_byte_count.saturating_mul(8)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntropyGeneratorExtractRequest {
    pub generator_identity: u64,
    pub algorithm_identity: u64,
    pub prior_generation: u64,
    pub resulting_generation: u64,
    pub output_identity: u64,
    pub block_byte_count: u64,
    pub output_offset_bytes: u64,
    pub output_byte_count: u64,
    pub counter: u64,
    pub nonce_identity: u64,
}

impl EntropyGeneratorExtractRequest {
    pub const INITIAL_Q35: Self = Self {
        generator_identity: 1,
        algorithm_identity: 1,
        prior_generation: 1,
        resulting_generation: 2,
        output_identity: 1,
        block_byte_count: 64,
        output_offset_bytes: 32,
        output_byte_count: 32,
        counter: 0,
        nonce_identity: 0,
    };

    pub const INITIAL_Q35_GETRANDOM: Self = Self {
        generator_identity: 1,
        algorithm_identity: 1,
        prior_generation: 2,
        resulting_generation: 3,
        output_identity: 2,
        block_byte_count: 64,
        output_offset_bytes: 32,
        output_byte_count: 32,
        counter: 0,
        nonce_identity: 0,
    };

    #[must_use]
    pub const fn is_well_formed(self) -> bool {
        self.generator_identity != 0
            && self.algorithm_identity != 0
            && self.prior_generation != 0
            && self.output_identity != 0
            && self.block_byte_count != 0
            && self.output_byte_count != 0
            && match self.prior_generation.checked_add(1) {
                Some(result) => result == self.resulting_generation,
                None => false,
            }
            && match self.output_offset_bytes.checked_add(self.output_byte_count) {
                Some(end) => end == self.block_byte_count,
                None => false,
            }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntropyOutputReservationRequest {
    pub output_identity: u64,
    pub source_byte_count: u64,
    pub selected_offset_bytes: u64,
    pub maximum_delivery_byte_count: u64,
    pub discarded_offset_bytes: u64,
    pub discarded_byte_count: u64,
    pub process_identity: u64,
    pub interface_identity: u64,
    pub service_identity: u64,
    pub supported_flag_mask: u64,
    pub incompatible_flag_mask: u64,
    pub single_use: bool,
}

impl EntropyOutputReservationRequest {
    pub const INITIAL_GETRANDOM: Self = Self {
        output_identity: 2,
        source_byte_count: 32,
        selected_offset_bytes: 0,
        maximum_delivery_byte_count: 32,
        discarded_offset_bytes: 32,
        discarded_byte_count: 0,
        process_identity: 1,
        interface_identity: 1,
        service_identity: 1,
        supported_flag_mask: 7,
        incompatible_flag_mask: 6,
        single_use: true,
    };

    #[must_use]
    pub const fn is_well_formed(self) -> bool {
        self.output_identity != 0
            && self.source_byte_count != 0
            && self.maximum_delivery_byte_count != 0
            && self.process_identity != 0
            && self.interface_identity != 0
            && self.service_identity != 0
            && self.supported_flag_mask != 0
            && self.incompatible_flag_mask != 0
            && self.incompatible_flag_mask & !self.supported_flag_mask == 0
            && self.single_use
            && match self
                .selected_offset_bytes
                .checked_add(self.maximum_delivery_byte_count)
            {
                Some(end) => end == self.discarded_offset_bytes,
                None => false,
            }
            && match self
                .discarded_offset_bytes
                .checked_add(self.discarded_byte_count)
            {
                Some(end) => end == self.source_byte_count,
                None => false,
            }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntropyUserRandomServiceRequest {
    pub generator_identity: u64,
    pub generation: u64,
    pub initial_output_identity: u64,
    pub block_byte_count: u64,
    pub output_byte_count: u64,
    pub maximum_delivery_byte_count: u64,
    pub process_identity: u64,
    pub interface_identity: u64,
    pub service_identity: u64,
    pub supported_flag_mask: u64,
    pub incompatible_flag_mask: u64,
    pub reserved_first_output: bool,
    pub reusable: bool,
}

impl EntropyUserRandomServiceRequest {
    pub const INITIAL_GETRANDOM: Self = Self {
        generator_identity: 1,
        generation: 3,
        initial_output_identity: 2,
        block_byte_count: 64,
        output_byte_count: 32,
        maximum_delivery_byte_count: 32,
        process_identity: 1,
        interface_identity: 1,
        service_identity: 1,
        supported_flag_mask: 7,
        incompatible_flag_mask: 6,
        reserved_first_output: true,
        reusable: true,
    };

    #[must_use]
    pub const fn is_well_formed(self) -> bool {
        self.generator_identity != 0
            && self.generation != 0
            && self.initial_output_identity != 0
            && self.block_byte_count != 0
            && self.output_byte_count != 0
            && self.maximum_delivery_byte_count != 0
            && self.maximum_delivery_byte_count <= self.output_byte_count
            && self.output_byte_count < self.block_byte_count
            && self.process_identity != 0
            && self.interface_identity != 0
            && self.service_identity != 0
            && self.supported_flag_mask != 0
            && self.incompatible_flag_mask != 0
            && self.incompatible_flag_mask & !self.supported_flag_mask == 0
            && self.reserved_first_output
            && self.reusable
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntropyUserRandomSmallRequest {
    pub process_identity: u64,
    pub interface_identity: u64,
    pub service_identity: u64,
    pub algorithm_identity: u64,
    pub output_byte_count: u64,
    pub maximum_output_count: u64,
    pub maximum_delivery_byte_count: u64,
    pub complete_request: bool,
}

impl EntropyUserRandomSmallRequest {
    pub const INITIAL_GETRANDOM: Self = Self {
        process_identity: 1,
        interface_identity: 1,
        service_identity: 1,
        algorithm_identity: 1,
        output_byte_count: 32,
        maximum_output_count: 8,
        maximum_delivery_byte_count: 256,
        complete_request: true,
    };

    pub const LARGE_GETRANDOM: Self = Self {
        process_identity: 1,
        interface_identity: 1,
        service_identity: 1,
        algorithm_identity: 1,
        output_byte_count: 32,
        maximum_output_count: 16,
        maximum_delivery_byte_count: 512,
        complete_request: true,
    };

    #[must_use]
    pub const fn is_well_formed(self) -> bool {
        self.process_identity != 0
            && self.interface_identity != 0
            && self.service_identity != 0
            && self.algorithm_identity != 0
            && self.output_byte_count != 0
            && self.maximum_output_count != 0
            && self.complete_request
            && match self
                .output_byte_count
                .checked_mul(self.maximum_output_count)
            {
                Some(maximum) => maximum == self.maximum_delivery_byte_count,
                None => false,
            }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntropyOutputDeliveryRequest {
    pub output_identity: u64,
    pub source_byte_count: u64,
    pub selected_offset_bytes: u64,
    pub delivered_byte_count: u64,
    pub discarded_offset_bytes: u64,
    pub discarded_byte_count: u64,
    pub address_space_identity: u64,
    pub interface_identity: u64,
    pub auxiliary_tag: u64,
    pub single_use: bool,
}

impl EntropyOutputDeliveryRequest {
    pub const INITIAL_AUXV_RANDOM: Self = Self {
        output_identity: 1,
        source_byte_count: 32,
        selected_offset_bytes: 0,
        delivered_byte_count: 16,
        discarded_offset_bytes: 16,
        discarded_byte_count: 16,
        address_space_identity: 1,
        interface_identity: 1,
        auxiliary_tag: 25,
        single_use: true,
    };

    #[must_use]
    pub const fn is_well_formed(self) -> bool {
        self.output_identity != 0
            && self.source_byte_count != 0
            && self.delivered_byte_count != 0
            && self.address_space_identity != 0
            && self.interface_identity != 0
            && self.auxiliary_tag != 0
            && self.single_use
            && match self
                .selected_offset_bytes
                .checked_add(self.delivered_byte_count)
            {
                Some(end) => end == self.discarded_offset_bytes,
                None => false,
            }
            && match self
                .discarded_offset_bytes
                .checked_add(self.discarded_byte_count)
            {
                Some(end) => end == self.source_byte_count,
                None => false,
            }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PciRegionDmaReferenceRequest {
    pub segment: u16,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub access_capability_offset_bytes: u8,
    pub bar: u8,
    pub region_offset_bytes: u32,
    pub region_length_bytes: u32,
    pub register_offset_bytes: u32,
    pub width_bytes: u8,
    pub buffer_identity: u64,
    pub buffer_offset_bytes: u64,
    pub device_identity: u64,
    pub translation_domain_identity: u64,
}

impl PciRegionDmaReferenceRequest {
    const fn initial(register_offset_bytes: u32, buffer_offset_bytes: u64) -> Self {
        Self {
            segment: 0,
            bus: 0,
            device: 5,
            function: 0,
            access_capability_offset_bytes: 132,
            bar: 4,
            region_offset_bytes: 0,
            region_length_bytes: 4096,
            register_offset_bytes,
            width_bytes: 8,
            buffer_identity: 1,
            buffer_offset_bytes,
            device_identity: 1,
            translation_domain_identity: 1,
        }
    }

    pub const INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES: [Self; 3] = [
        Self::initial(32, 0),
        Self::initial(40, 128),
        Self::initial(48, 148),
    ];

    #[must_use]
    pub const fn is_well_formed(self) -> bool {
        self.device <= 31
            && self.function <= 7
            && self.access_capability_offset_bytes <= 252
            && self.access_capability_offset_bytes.is_multiple_of(4)
            && self.bar <= 5
            && self.region_length_bytes != 0
            && self.width_bytes == 8
            && self.register_offset_bytes <= self.region_length_bytes - 8
            && self.register_offset_bytes.is_multiple_of(8)
            && self.buffer_identity != 0
            && self.buffer_offset_bytes <= 4096 - 8
            && self.device_identity != 0
            && self.translation_domain_identity != 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PciRegionRegisterDirection {
    ReadEquals,
    ReadEventuallyEquals,
    Write,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PciRegionRegisterRequest {
    pub segment: u16,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub access_capability_offset_bytes: u8,
    pub bar: u8,
    pub region_offset_bytes: u32,
    pub region_length_bytes: u32,
    pub register_offset_bytes: u32,
    pub width_bytes: u8,
    pub direction: PciRegionRegisterDirection,
    pub value: u32,
}

impl PciRegionRegisterRequest {
    pub const INITIAL_Q35_VIRTIO_RNG_NEGOTIATION: [Self; 16] = [
        Self::initial(0x14, 1, PciRegionRegisterDirection::Write, 0),
        Self::initial(0x14, 1, PciRegionRegisterDirection::ReadEquals, 0),
        Self::initial(0x14, 1, PciRegionRegisterDirection::Write, 1),
        Self::initial(0x14, 1, PciRegionRegisterDirection::ReadEquals, 1),
        Self::initial(0x14, 1, PciRegionRegisterDirection::Write, 3),
        Self::initial(0x14, 1, PciRegionRegisterDirection::ReadEquals, 3),
        Self::initial(0x00, 4, PciRegionRegisterDirection::Write, 0),
        Self::initial(0x04, 4, PciRegionRegisterDirection::ReadEquals, 0x3000_0000),
        Self::initial(0x00, 4, PciRegionRegisterDirection::Write, 1),
        Self::initial(0x04, 4, PciRegionRegisterDirection::ReadEquals, 0x0000_0101),
        Self::initial(0x08, 4, PciRegionRegisterDirection::Write, 0),
        Self::initial(0x0c, 4, PciRegionRegisterDirection::Write, 0),
        Self::initial(0x08, 4, PciRegionRegisterDirection::Write, 1),
        Self::initial(0x0c, 4, PciRegionRegisterDirection::Write, 1),
        Self::initial(0x14, 1, PciRegionRegisterDirection::Write, 11),
        Self::initial(0x14, 1, PciRegionRegisterDirection::ReadEquals, 11),
    ];

    pub const INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY: [Self; 5] = [
        Self::initial(0x12, 2, PciRegionRegisterDirection::ReadEquals, 1),
        Self::initial(0x16, 2, PciRegionRegisterDirection::Write, 0),
        Self::initial(0x18, 2, PciRegionRegisterDirection::ReadEquals, 8),
        Self::initial(0x1c, 2, PciRegionRegisterDirection::ReadEquals, 0),
        Self::initial(0x1e, 2, PciRegionRegisterDirection::ReadEquals, 0),
    ];

    pub const INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION: [Self; 4] = [
        Self::initial(0x1c, 2, PciRegionRegisterDirection::Write, 1),
        Self::initial(0x1c, 2, PciRegionRegisterDirection::ReadEquals, 1),
        Self::initial(0x14, 1, PciRegionRegisterDirection::Write, 15),
        Self::initial(0x14, 1, PciRegionRegisterDirection::ReadEquals, 15),
    ];

    pub const INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION: [Self; 1] = [Self {
        segment: 0,
        bus: 0,
        device: 5,
        function: 0,
        access_capability_offset_bytes: 0x84,
        bar: 4,
        region_offset_bytes: 0x3000,
        region_length_bytes: 0x1000,
        register_offset_bytes: 0,
        width_bytes: 2,
        direction: PciRegionRegisterDirection::Write,
        value: 0,
    }];

    pub const INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL: [Self; 1] = [Self {
        segment: 0,
        bus: 0,
        device: 5,
        function: 0,
        access_capability_offset_bytes: 0x84,
        bar: 4,
        region_offset_bytes: 0x1000,
        region_length_bytes: 0x1000,
        register_offset_bytes: 0,
        width_bytes: 1,
        direction: PciRegionRegisterDirection::ReadEventuallyEquals,
        value: 1,
    }];

    #[must_use]
    pub const fn initial(
        register_offset_bytes: u32,
        width_bytes: u8,
        direction: PciRegionRegisterDirection,
        value: u32,
    ) -> Self {
        Self {
            segment: 0,
            bus: 0,
            device: 5,
            function: 0,
            access_capability_offset_bytes: 0x84,
            bar: 4,
            region_offset_bytes: 0,
            region_length_bytes: 0x1000,
            register_offset_bytes,
            width_bytes,
            direction,
            value,
        }
    }

    #[must_use]
    pub const fn is_well_formed(self) -> bool {
        let value_fits = match self.width_bytes {
            1 => self.value <= u8::MAX as u32,
            2 => self.value <= u16::MAX as u32,
            4 => true,
            _ => false,
        };
        let Some(end) = self
            .register_offset_bytes
            .checked_add(self.width_bytes as u32)
        else {
            return false;
        };
        self.device <= 31
            && self.function <= 7
            && self.access_capability_offset_bytes <= 236
            && self.access_capability_offset_bytes % 4 == 0
            && self.bar <= 5
            && self.region_length_bytes != 0
            && end <= self.region_length_bytes
            && self.register_offset_bytes % self.width_bytes as u32 == 0
            && value_fits
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SystemsOperation {
    DescribeBootMemory {
        failure_message: String,
    },
    CreateFrameAllocator {
        failure_message: String,
    },
    AllocatePhysicalFrames {
        request: PhysicalFrameRequest,
        failure_message: String,
    },
    ReserveStaticImageHierarchy {
        failure_message: String,
    },
    ReserveKernelImageBacking {
        failure_message: String,
    },
    DiscardKernelImageBacking,
    ReleasePhysicalFrames,
    PrepareStaticImageConstruction {
        failure_message: String,
    },
    ZeroStaticImageConstructionPayload,
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
    CopyStaticImageConstructionPayload,
    SealStaticImageConstructionPermissions,
    AttachStaticImageAddressSpace,
    ActivateStaticImageAddressSpace,
    RestoreStaticImageAddressSpace,
    AuthorizeStaticImageEntry,
    AuthorizeStaticImageStackSlot,
    RevokeStaticImageStackSlot,
    RevokeStaticImageEntry,
    DetachStaticImageAddressSpace,
    ReclaimStaticImageConstructionHierarchy,
    RevokeStaticImageConstructionMetadata,
    ReclaimStaticImageConstructionPayload,
    MapKernelFrames {
        request: KernelMappingRequest,
        failure_message: String,
    },
    KernelMappingStoreByte {
        offset_bytes: u64,
        value: u8,
    },
    KernelMappingLoadByteEquals {
        offset_bytes: u64,
        expected: u8,
        failure_message: String,
    },
    UnmapKernelFrames,
    BeginTranslationUpdate {
        request: TranslationUpdateRequest,
        failure_message: String,
    },
    CommitTranslationUpdate {
        failure_message: String,
    },
    ActivateTranslationSpace {
        failure_message: String,
    },
    BeginTranslationEdit {
        kind: TranslationEditKind,
        failure_message: String,
    },
    MapTranslationFrames {
        request: TranslationMappingRequest,
        failure_message: String,
    },
    UnmapTranslationMapping,
    CommitTranslationEdit {
        kind: TranslationEditKind,
        failure_message: String,
    },
    EnterCritical {
        domain: CriticalDomain,
        failure_message: String,
    },
    RestoreCritical {
        domain: CriticalDomain,
    },
    ConsoleWrite {
        text: String,
    },
    ObservePciConfigurationDword {
        request: PciConfigurationDwordRequest,
    },
    UpdatePciConfigurationWord {
        request: PciConfigurationWordUpdateRequest,
    },
    CreateDmaBuffer {
        request: DmaBufferRequest,
    },
    BindDmaBufferReference {
        request: DmaBufferReferenceRequest,
    },
    StoreDmaBufferInteger {
        request: DmaBufferIntegerStoreRequest,
    },
    PrepareDmaBufferForDevice {
        request: DmaBufferPreparationRequest,
    },
    CompleteDmaBufferForCpu {
        request: DmaBufferPreparationRequest,
    },
    LoadDmaBufferIntegerEquals {
        request: DmaBufferIntegerLoadRequest,
    },
    CaptureEntropySample {
        request: EntropySampleCaptureRequest,
    },
    AssessEntropySampleQuality {
        request: EntropySampleQualityRequest,
    },
    CreateEntropyPool {
        request: EntropyPoolRequest,
    },
    MixEntropyPoolSample {
        request: EntropyPoolMixRequest,
    },
    CreditEntropyPoolContribution {
        request: EntropyPoolCreditRequest,
    },
    InitializeEntropyGenerator {
        request: EntropyGeneratorRequest,
    },
    ExtractEntropyGenerator {
        request: EntropyGeneratorExtractRequest,
    },
    DeliverInitialAuxvRandom {
        request: EntropyOutputDeliveryRequest,
    },
    ReserveInitialUserRandom {
        request: EntropyOutputReservationRequest,
    },
    BindReusableUserRandom {
        request: EntropyUserRandomServiceRequest,
    },
    ExpandSmallUserRandomRequests {
        request: EntropyUserRandomSmallRequest,
    },
    ExpandLargeUserRandomRequests {
        request: EntropyUserRandomSmallRequest,
    },
    PublishPciRegionDmaReference {
        request: PciRegionDmaReferenceRequest,
    },
    AccessPciRegionRegister {
        request: PciRegionRegisterRequest,
    },
    DebugBreak,
    BootstrapAllocate {
        request: BootstrapStorageRequest,
    },
    BootstrapStoreByte {
        offset_bytes: u64,
        value: u8,
    },
    ApplyBootstrapRegionLibrary,
    BootstrapLoadByteEquals {
        offset_bytes: u64,
        expected: u8,
        failure_message: String,
    },
    AtomicWordCreate {
        request: AtomicWordRequest,
    },
    AtomicCompareExchangeEquals {
        expected: u64,
        desired: u64,
        success_order: AtomicOrder,
        failure_order: AtomicOrder,
        failure_message: String,
    },
    AtomicLoadEquals {
        order: AtomicOrder,
        expected: u64,
        failure_message: String,
    },
    AtomicWordEnd,
    BootstrapRelease,
    SendLocalNotification,
    WaitLocalNotification,
    CompleteLocalNotification,
    ObserveMonotonicClock,
    ConstructDeadline {
        duration_nanoseconds: u64,
    },
    ArmDeadline,
    WaitDeadline,
    CompleteDeadline,
    CreateKernelContext,
    CreateKernelCompletion,
    CreateRunnableQueue {
        capacity: usize,
    },
    EnqueueRunnableContext,
    DequeueRunnableContext,
    TransferKernelContextUntilDeadline,
    TransferKernelContextUntilCompletion,
    TakePreemptedKernelContext,
    TransferKernelContext,
    AwaitDeadlinePreemption,
    AwaitKernelCompletion,
    SignalBlockedKernelCompletion,
    ReclaimKernelContext,
    ConsumeKernelCompletion,
    ConsumeEmptyRunnableQueue,
    PrepareUserAddressSpace,
    VerifyUserAddressSpaceMapping,
    MapUserAddressSpace,
    ReserveAnonymousUserMapping,
    StoreUserMappingByte,
    InstallUserMappingEntryTrampoline,
    ProtectUserMappingReadOnly,
    LoadUserMappingByte,
    VerifyUserMappingEntryTrampoline,
    PermitUserMappingExecute,
    SealUserMappingImmutableData,
    PermitUserMappingWritableData,
    UnmapUserAddressSpace,
    ReleaseAnonymousUserMapping,
    PrepareUserProbe {
        process_identity: u64,
    },
    EnterUserContext,
    CompleteUserExecution,
    ReclaimUserAddressSpace,
    CopyInitialUnameToUser,
    CopyInitialTimespecFromUser,
}

impl SystemsOperation {
    #[must_use]
    pub const fn semantic_identity(&self) -> &'static str {
        match self {
            Self::DescribeBootMemory { .. } => SYSTEMS_BOOT_MEMORY_DESCRIBE,
            Self::CreateFrameAllocator { .. } => SYSTEMS_FRAME_ALLOCATOR_CREATE,
            Self::AllocatePhysicalFrames { .. } => SYSTEMS_FRAMES_ALLOCATE,
            Self::ReserveKernelImageBacking { .. } => SYSTEMS_KERNEL_IMAGE_RESERVE_BACKING,
            Self::DiscardKernelImageBacking => SYSTEMS_KERNEL_IMAGE_DISCARD_BACKING,
            Self::ReserveStaticImageHierarchy { .. } => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESERVE_HIERARCHY
            }
            Self::ReleasePhysicalFrames => SYSTEMS_FRAMES_RELEASE,
            Self::PrepareStaticImageConstruction { .. } => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PREPARE
            }
            Self::ZeroStaticImageConstructionPayload => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_PAYLOAD
            }
            Self::ZeroStaticImageStackBacking => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_STACK_BACKING
            }
            Self::PublishStaticImageStackLeaf => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PUBLISH_STACK_LEAF
            }
            Self::InitializeStaticImageStackEntry => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_INITIALIZE_STACK_ENTRY
            }
            Self::AddStaticImageStackPageSize => SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PAGESZ,
            Self::AddStaticImageStackEntryFact => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_ENTRY_FACT
            }
            Self::AddStaticImageStackProgramHeaders => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PROGRAM_HEADERS
            }
            Self::AddStaticImageStackExecutableName => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_EXECUTABLE_NAME
            }
            Self::AddStaticImageStackCredentials => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_CREDENTIALS
            }
            Self::AddStaticImageStackSecureMode => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_SECURE_MODE
            }
            Self::AddStaticImageStackClockTickRate => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_CLOCK_TICK_RATE
            }
            Self::AddStaticImageStackPlatform => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PLATFORM
            }
            Self::AddStaticImageStackHardwareCapabilities => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_HARDWARE_CAPABILITIES
            }
            Self::AddStaticImageStackLoadMetadata => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_LOAD_METADATA
            }
            Self::AuthorizeStaticImageStackRandomSlot => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_RANDOM_SLOT
            }
            Self::RetainStaticImagePreparedProcess => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RETAIN_PREPARED_PROCESS
            }
            Self::QualifyStaticImageRetainedDisjointWork => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_RETAINED_DISJOINT_WORK
            }
            Self::AuthorizeStaticImageEntropyHandoff => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTROPY_HANDOFF
            }
            Self::AuthorizeStaticImageEntropyAcquisition => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTROPY_ACQUISITION
            }
            Self::QualifyStaticImageEntropyDevice => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DEVICE
            }
            Self::QualifyStaticImageEntropyTransport => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_TRANSPORT
            }
            Self::QualifyStaticImageEntropyNegotiation => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_NEGOTIATION
            }
            Self::QualifyStaticImageEntropyQueue => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE
            }
            Self::QualifyStaticImageEntropyBusMaster => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_BUS_MASTER
            }
            Self::QualifyStaticImageEntropyDmaBuffers => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DMA_BUFFERS
            }
            Self::QualifyStaticImageEntropySplitRing => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SPLIT_RING
            }
            Self::QualifyStaticImageEntropyDmaPreparation => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DMA_PREPARATION
            }
            Self::QualifyStaticImageEntropyQueueAddresses => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE_ADDRESSES
            }
            Self::QualifyStaticImageEntropyQueueActivation => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE_ACTIVATION
            }
            Self::QualifyStaticImageEntropyRequestCompletion => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_REQUEST_COMPLETION
            }
            Self::QualifyStaticImageEntropySampleCapture => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SAMPLE_CAPTURE
            }
            Self::QualifyStaticImageEntropySampleQuality => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SAMPLE_QUALITY
            }
            Self::QualifyStaticImageEntropyPoolLifecycle => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_POOL_LIFECYCLE
            }
            Self::QualifyStaticImageEntropyGeneratorLifecycle => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_GENERATOR_LIFECYCLE
            }
            Self::DeliverStaticImageEntropyInitialRandom => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_DELIVER_ENTROPY_INITIAL_RANDOM
            }
            Self::EnterStaticImageProcess => SYSTEMS_STATIC_IMAGE_PROCESS_ENTER,
            Self::RestoreStaticImageProcessEntry => SYSTEMS_STATIC_IMAGE_PROCESS_RESTORE_ENTRY,
            Self::RevokeStaticImageEntropyInitialRandom => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_INITIAL_RANDOM
            }
            Self::RevokeStaticImageEntropyAcquisition => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_ACQUISITION
            }
            Self::RevokeStaticImageEntropyHandoff => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_HANDOFF
            }
            Self::RestoreStaticImagePreparedProcess => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESTORE_PREPARED_PROCESS
            }
            Self::RevokeStaticImageStackRandomSlot => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_RANDOM_SLOT
            }
            Self::RemoveStaticImageStackLoadMetadata => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_LOAD_METADATA
            }
            Self::RemoveStaticImageStackHardwareCapabilities => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_HARDWARE_CAPABILITIES
            }
            Self::RemoveStaticImageStackPlatform => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PLATFORM
            }
            Self::RemoveStaticImageStackClockTickRate => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_CLOCK_TICK_RATE
            }
            Self::RemoveStaticImageStackSecureMode => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_SECURE_MODE
            }
            Self::RemoveStaticImageStackCredentials => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_CREDENTIALS
            }
            Self::RemoveStaticImageStackExecutableName => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_EXECUTABLE_NAME
            }
            Self::RemoveStaticImageStackProgramHeaders => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PROGRAM_HEADERS
            }
            Self::RemoveStaticImageStackEntryFact => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_ENTRY_FACT
            }
            Self::RemoveStaticImageStackPageSize => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PAGESZ
            }
            Self::ClearStaticImageStackEntry => SYSTEMS_STATIC_IMAGE_CONSTRUCTION_CLEAR_STACK_ENTRY,
            Self::RevokeStaticImageStackLeaf => SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_LEAF,
            Self::CopyStaticImageConstructionPayload => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_COPY_PAYLOAD
            }
            Self::SealStaticImageConstructionPermissions => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_SEAL_PERMISSIONS
            }
            Self::AttachStaticImageAddressSpace => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ATTACH_ADDRESS_SPACE
            }
            Self::ActivateStaticImageAddressSpace => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ACTIVATE_ADDRESS_SPACE
            }
            Self::RestoreStaticImageAddressSpace => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESTORE_ADDRESS_SPACE
            }
            Self::AuthorizeStaticImageEntry => SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTRY,
            Self::AuthorizeStaticImageStackSlot => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_SLOT
            }
            Self::RevokeStaticImageStackSlot => SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_SLOT,
            Self::RevokeStaticImageEntry => SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTRY,
            Self::DetachStaticImageAddressSpace => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_DETACH_ADDRESS_SPACE
            }
            Self::ReclaimStaticImageConstructionHierarchy => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_HIERARCHY
            }
            Self::RevokeStaticImageConstructionMetadata => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_METADATA
            }
            Self::ReclaimStaticImageConstructionPayload => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_PAYLOAD
            }
            Self::MapKernelFrames { .. } => SYSTEMS_KERNEL_MAP,
            Self::KernelMappingStoreByte { .. } => SYSTEMS_KERNEL_MAPPING_STORE_BYTE,
            Self::KernelMappingLoadByteEquals { .. } => SYSTEMS_KERNEL_MAPPING_LOAD_BYTE,
            Self::UnmapKernelFrames => SYSTEMS_KERNEL_UNMAP,
            Self::BeginTranslationUpdate { .. } => SYSTEMS_TRANSLATION_BEGIN,
            Self::CommitTranslationUpdate { .. } => SYSTEMS_TRANSLATION_COMMIT,
            Self::ActivateTranslationSpace { .. } => SYSTEMS_TRANSLATION_ACTIVATE,
            Self::BeginTranslationEdit { .. } => SYSTEMS_TRANSLATION_EDIT_BEGIN,
            Self::MapTranslationFrames { .. } => SYSTEMS_TRANSLATION_EDIT_MAP,
            Self::UnmapTranslationMapping => SYSTEMS_TRANSLATION_EDIT_UNMAP,
            Self::CommitTranslationEdit { .. } => SYSTEMS_TRANSLATION_EDIT_COMMIT,
            Self::EnterCritical { .. } => SYSTEMS_CRITICAL_ENTER,
            Self::RestoreCritical { .. } => SYSTEMS_CRITICAL_RESTORE,
            Self::ConsoleWrite { .. } => SYSTEMS_CONSOLE_WRITE,
            Self::ObservePciConfigurationDword { .. } => SYSTEMS_PCI_CONFIGURATION_READ_DWORD,
            Self::UpdatePciConfigurationWord { .. } => SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD,
            Self::CreateDmaBuffer { .. } => SYSTEMS_DMA_BUFFER_CREATE,
            Self::BindDmaBufferReference { .. } => SYSTEMS_DMA_BUFFER_BIND_REFERENCE,
            Self::StoreDmaBufferInteger { .. } => SYSTEMS_DMA_BUFFER_STORE_INTEGER,
            Self::PrepareDmaBufferForDevice { .. } => SYSTEMS_DMA_BUFFER_PREPARE_DEVICE,
            Self::CompleteDmaBufferForCpu { .. } => SYSTEMS_DMA_BUFFER_COMPLETE_CPU,
            Self::LoadDmaBufferIntegerEquals { .. } => SYSTEMS_DMA_BUFFER_LOAD_INTEGER,
            Self::CaptureEntropySample { .. } => SYSTEMS_ENTROPY_SAMPLE_CAPTURE_DMA,
            Self::AssessEntropySampleQuality { .. } => SYSTEMS_ENTROPY_SAMPLE_ASSESS_QUALITY,
            Self::CreateEntropyPool { .. } => SYSTEMS_ENTROPY_POOL_CREATE,
            Self::MixEntropyPoolSample { .. } => SYSTEMS_ENTROPY_POOL_MIX_SAMPLE,
            Self::CreditEntropyPoolContribution { .. } => SYSTEMS_ENTROPY_POOL_CREDIT_CONTRIBUTION,
            Self::InitializeEntropyGenerator { .. } => SYSTEMS_ENTROPY_GENERATOR_INITIALIZE,
            Self::ExtractEntropyGenerator { .. } => SYSTEMS_ENTROPY_GENERATOR_EXTRACT,
            Self::DeliverInitialAuxvRandom { .. } => {
                SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM
            }
            Self::ReserveInitialUserRandom { .. } => {
                SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM
            }
            Self::BindReusableUserRandom { .. } => SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE,
            Self::ExpandSmallUserRandomRequests { .. } => {
                SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS
            }
            Self::ExpandLargeUserRandomRequests { .. } => {
                SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS
            }
            Self::PublishPciRegionDmaReference { .. } => SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE,
            Self::AccessPciRegionRegister { request } => match request.direction {
                PciRegionRegisterDirection::ReadEquals => SYSTEMS_PCI_REGION_READ,
                PciRegionRegisterDirection::ReadEventuallyEquals => {
                    SYSTEMS_PCI_REGION_READ_EVENTUALLY
                }
                PciRegionRegisterDirection::Write => SYSTEMS_PCI_REGION_WRITE,
            },
            Self::DebugBreak => SYSTEMS_DEBUG_BREAK,
            Self::BootstrapAllocate { .. } => SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
            Self::BootstrapStoreByte { .. } => SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE,
            Self::ApplyBootstrapRegionLibrary => SYSTEMS_REGION_LIBRARY_APPLY,
            Self::BootstrapLoadByteEquals { .. } => SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
            Self::AtomicWordCreate { .. } => SYSTEMS_ATOMIC_WORD_CREATE,
            Self::AtomicCompareExchangeEquals { .. } => SYSTEMS_ATOMIC_COMPARE_EXCHANGE,
            Self::AtomicLoadEquals { .. } => SYSTEMS_ATOMIC_LOAD,
            Self::AtomicWordEnd => SYSTEMS_ATOMIC_END,
            Self::BootstrapRelease => SYSTEMS_BOOTSTRAP_STORAGE_RELEASE,
            Self::SendLocalNotification => SYSTEMS_LOCAL_NOTIFICATION_SEND,
            Self::WaitLocalNotification => SYSTEMS_LOCAL_NOTIFICATION_WAIT,
            Self::CompleteLocalNotification => SYSTEMS_LOCAL_NOTIFICATION_COMPLETE,
            Self::ObserveMonotonicClock => SYSTEMS_MONOTONIC_CLOCK_NOW,
            Self::ConstructDeadline { .. } => SYSTEMS_DEADLINE_AFTER,
            Self::ArmDeadline => SYSTEMS_DEADLINE_ARM,
            Self::WaitDeadline => SYSTEMS_DEADLINE_WAIT,
            Self::CompleteDeadline => SYSTEMS_DEADLINE_COMPLETE,
            Self::CreateKernelContext => SYSTEMS_KERNEL_CONTEXT_CREATE,
            Self::CreateKernelCompletion => SYSTEMS_KERNEL_COMPLETION_CREATE,
            Self::CreateRunnableQueue { .. } => SYSTEMS_KERNEL_RUNNABLE_QUEUE_CREATE,
            Self::EnqueueRunnableContext => SYSTEMS_KERNEL_RUNNABLE_QUEUE_ENQUEUE,
            Self::DequeueRunnableContext => SYSTEMS_KERNEL_RUNNABLE_QUEUE_DEQUEUE,
            Self::TransferKernelContextUntilDeadline => {
                SYSTEMS_KERNEL_CONTEXT_TRANSFER_UNTIL_DEADLINE
            }
            Self::TransferKernelContextUntilCompletion => SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE,
            Self::TakePreemptedKernelContext => SYSTEMS_KERNEL_CONTEXT_TAKE_PREEMPTED,
            Self::TransferKernelContext => SYSTEMS_KERNEL_CONTEXT_TRANSFER,
            Self::AwaitDeadlinePreemption => SYSTEMS_KERNEL_CONTEXT_AWAIT_DEADLINE_PREEMPTION,
            Self::AwaitKernelCompletion => SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE,
            Self::SignalBlockedKernelCompletion => SYSTEMS_KERNEL_COMPLETION_SIGNAL,
            Self::ReclaimKernelContext => SYSTEMS_KERNEL_CONTEXT_RECLAIM,
            Self::ConsumeKernelCompletion => SYSTEMS_KERNEL_COMPLETION_CONSUME,
            Self::ConsumeEmptyRunnableQueue => SYSTEMS_KERNEL_RUNNABLE_QUEUE_CONSUME,
            Self::PrepareUserAddressSpace => SYSTEMS_USER_ADDRESS_SPACE_PREPARE,
            Self::VerifyUserAddressSpaceMapping => SYSTEMS_USER_ADDRESS_SPACE_VERIFY_MAPPING,
            Self::MapUserAddressSpace => SYSTEMS_USER_ADDRESS_SPACE_MAP,
            Self::ReserveAnonymousUserMapping => SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS,
            Self::StoreUserMappingByte => SYSTEMS_USER_MAPPING_STORE_BYTE,
            Self::InstallUserMappingEntryTrampoline => {
                SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE
            }
            Self::ProtectUserMappingReadOnly => SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY,
            Self::LoadUserMappingByte => SYSTEMS_USER_MAPPING_LOAD_BYTE,
            Self::VerifyUserMappingEntryTrampoline => SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE,
            Self::PermitUserMappingExecute => SYSTEMS_USER_MAPPING_PERMIT_EXECUTE,
            Self::SealUserMappingImmutableData => SYSTEMS_USER_MAPPING_SEAL_IMMUTABLE_DATA,
            Self::PermitUserMappingWritableData => SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA,
            Self::UnmapUserAddressSpace => SYSTEMS_USER_ADDRESS_SPACE_UNMAP,
            Self::ReleaseAnonymousUserMapping => SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS,
            Self::PrepareUserProbe { .. } => SYSTEMS_USER_CONTEXT_PREPARE,
            Self::EnterUserContext => SYSTEMS_USER_CONTEXT_ENTER,
            Self::CompleteUserExecution => SYSTEMS_USER_CONTEXT_COMPLETE,
            Self::ReclaimUserAddressSpace => SYSTEMS_USER_ADDRESS_SPACE_RECLAIM,
            Self::CopyInitialUnameToUser => SYSTEMS_USER_COPY_TO,
            Self::CopyInitialTimespecFromUser => SYSTEMS_USER_COPY_FROM,
        }
    }
}

include!("systems_model_program.rs");
include!("systems_model_authority.rs");
include!("systems_model_authority_protocols.rs");
include!("systems_model_transitions.rs");
include!("systems_model_storage_transitions.rs");

#[cfg(test)]
mod tests {
    include!("systems_model_tests_regions_and_contexts.rs");
    include!("systems_model_tests_memory_and_protocols.rs");
    include!("systems_model_tests_hierarchy_backing.rs");
    include!("systems_model_tests_image_backing.rs");
}
