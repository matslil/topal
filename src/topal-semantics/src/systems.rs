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
    SYSTEMS_KERNEL_MAP, SYSTEMS_KERNEL_MAPPING_LOAD_BYTE, SYSTEMS_KERNEL_MAPPING_STORE_BYTE,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SystemsDisposition {
    Resume,
    PreemptKernelContext,
    RetireKernelContextToCaller,
    DispatchInitialUserSyscall {
        getpid_result: u64,
        uname_sysname: String,
        uname_nodename: String,
        uname_release: String,
        uname_version: String,
        uname_machine: String,
        uname_domainname: String,
        nanosleep_seconds: u64,
        nanosleep_nanoseconds: u64,
        nanosleep_remainder_candidate: u64,
        clock_gettime_result: u64,
        clock_getres_result: u64,
        write_descriptor: u64,
        write_maximum_bytes: u64,
        read_descriptor: u64,
        read_maximum_bytes: u64,
        initial_descriptor_count: u64,
        bad_descriptor_error: u64,
        too_many_open_files_error: u64,
        console_input_description: u64,
        console_output_description: u64,
        console_error_description: u64,
        invalid_argument_error: u64,
        fcntl_getfd_command: u64,
        fcntl_setfd_command: u64,
        fcntl_getfl_command: u64,
        close_on_exec_flag: u64,
        read_only_status: u64,
        write_only_status: u64,
        stat_layout_bytes: u64,
        console_device: u64,
        console_inode: u64,
        console_mode: u64,
        console_special_device: u64,
        console_block_size: u64,
        terminal_layout_bytes: u64,
        terminal_get_attributes_request: u64,
        terminal_control_flags: u64,
        terminal_minimum_input: u64,
        not_a_terminal_error: u64,
        window_size_layout_bytes: u64,
        window_size_query_request: u64,
        seek_maximum_whence: u64,
        illegal_seek_error: u64,
        brk_data_page_bytes: u64,
        getppid_result: u64,
        gettid_result: u64,
        getuid_result: u64,
        geteuid_result: u64,
        getgid_result: u64,
        getegid_result: u64,
        arch_set_fs_result: u64,
        arch_get_fs_result: u64,
        set_tid_address_result: u64,
        set_robust_list_result: u64,
        getrlimit_result: u64,
        prlimit64_result: u64,
        rt_sigprocmask_result: u64,
        anonymous_page_bytes: u64,
        madvise_result: u64,
        mprotect_result: u64,
        munmap_result: u64,
        exit_group_status: u64,
        unsupported_syscall_error: u64,
        sched_yield_result: u64,
    },
    Fatal {
        message: String,
    },
}

impl SystemsDisposition {
    #[must_use]
    pub const fn semantic_identity(&self, context: SystemsContextKind) -> &'static str {
        match self {
            Self::Resume if matches!(context, SystemsContextKind::DebugBreak) => {
                SYSTEMS_RESUME_DEBUG_BREAK
            }
            Self::Resume if matches!(context, SystemsContextKind::DeadlineInterrupt) => {
                SYSTEMS_RESUME_DEADLINE
            }
            Self::Resume => SYSTEMS_RESUME_LOCAL_NOTIFICATION,
            Self::PreemptKernelContext => SYSTEMS_KERNEL_CONTEXT_PREEMPT_CURRENT,
            Self::RetireKernelContextToCaller => SYSTEMS_KERNEL_CONTEXT_RETIRE,
            Self::DispatchInitialUserSyscall { .. } => SYSTEMS_USER_SYSCALL_DISPATCH,
            Self::Fatal { .. } => SYSTEMS_FATAL,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsHandler {
    pub name: String,
    pub context: SystemsContextKind,
    pub operations: Vec<SystemsOperation>,
    pub disposition: SystemsDisposition,
    pub effects: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsEntry {
    pub kind: SystemsEntryKind,
    pub handler: SystemsHandler,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsProgram {
    pub target: SystemsTargetSelection,
    pub bootstrap_storage: BootstrapStorageDescriptor,
    pub bootstrap: SystemsEntry,
    pub debug_break: SystemsEntry,
    pub local_notification: SystemsEntry,
    pub deadline_notification: SystemsEntry,
    pub kernel_thread: SystemsEntry,
    pub completion_thread: SystemsEntry,
    pub terminal_thread: SystemsEntry,
    pub user_syscall: SystemsEntry,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SystemsTransition {
    ProvisionBootstrapStorage {
        capacity_bytes: u64,
        alignment_bytes: u64,
    },
    EnterBootstrap,
    DescribeBootMemory,
    CreateFrameAllocator,
    AllocatePhysicalFrames {
        request: PhysicalFrameRequest,
    },
    ReleasePhysicalFrames,
    PrepareStaticImageConstruction,
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
    },
    StoreKernelMappingByte {
        offset_bytes: u64,
        value: u8,
    },
    LoadKernelMappingByte {
        offset_bytes: u64,
        value: u8,
    },
    UnmapKernelFrames,
    BeginTranslationUpdate {
        request: TranslationUpdateRequest,
    },
    CommitTranslationUpdate,
    ActivateTranslationSpace,
    BeginTranslationEdit {
        kind: TranslationEditKind,
    },
    MapTranslationFrames {
        request: TranslationMappingRequest,
    },
    UnmapTranslationMapping,
    CommitTranslationEdit {
        kind: TranslationEditKind,
    },
    EnterCritical {
        domain: CriticalDomain,
        nesting_identity: u64,
    },
    RestoreCritical {
        domain: CriticalDomain,
        nesting_identity: u64,
    },
    ConsoleWrite {
        text: String,
    },
    ObservePciConfigurationDword {
        transaction_identity: u64,
        request: PciConfigurationDwordRequest,
    },
    UpdatePciConfigurationWord {
        transaction_identity: u64,
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
        transaction_identity: u64,
        request: PciRegionDmaReferenceRequest,
    },
    AccessPciRegionRegister {
        transaction_identity: u64,
        request: PciRegionRegisterRequest,
    },
    ObserveDebugBreak,
    EnterDebugBreak,
    ResumeDebugBreak,
    SendLocalNotification {
        event_identity: u64,
    },
    BeginLocalNotificationWait {
        event_identity: u64,
    },
    ObserveLocalNotification {
        event_identity: u64,
    },
    EnterLocalNotificationInterrupt {
        event_identity: u64,
    },
    CompleteLocalNotificationInterrupt {
        event_identity: u64,
    },
    ResumeLocalNotificationInterrupt {
        event_identity: u64,
    },
    EndLocalNotificationWait {
        event_identity: u64,
    },
    ObserveMonotonicClock {
        observation_identity: u64,
    },
    ConstructDeadline {
        source_observation_identity: u64,
        duration_nanoseconds: u64,
    },
    ArmDeadline {
        event_identity: u64,
    },
    BeginDeadlineWait {
        event_identity: u64,
    },
    ObserveDeadline {
        event_identity: u64,
    },
    EnterDeadlineInterrupt {
        event_identity: u64,
    },
    CompleteDeadlineInterrupt {
        event_identity: u64,
    },
    ResumeDeadlineInterrupt {
        event_identity: u64,
    },
    PreemptKernelContext {
        event_identity: u64,
        context_identity: u64,
        caller_identity: u64,
    },
    ResumePreemptionDispatcher {
        caller_identity: u64,
    },
    EndDeadlineWait {
        event_identity: u64,
    },
    CreateKernelContext {
        context_identity: u64,
    },
    CreateKernelCompletion {
        completion_identity: u64,
    },
    CreateRunnableQueue {
        capacity: usize,
    },
    EnqueueRunnableContext {
        context_identity: u64,
    },
    DequeueRunnableContext {
        context_identity: u64,
    },
    TransferKernelContext {
        context_identity: u64,
        caller_identity: u64,
    },
    TransferKernelContextUntilDeadline {
        context_identity: u64,
        caller_identity: u64,
        event_identity: u64,
    },
    TransferKernelContextUntilCompletion {
        context_identity: u64,
        caller_identity: u64,
        completion_identity: u64,
    },
    AwaitKernelCompletion {
        context_identity: u64,
        completion_identity: u64,
    },
    BlockKernelContext {
        context_identity: u64,
        caller_identity: u64,
        completion_identity: u64,
    },
    ResumeCompletionDispatcher {
        caller_identity: u64,
    },
    SignalBlockedKernelCompletion {
        context_identity: u64,
        caller_identity: u64,
        completion_identity: u64,
    },
    RestoreBlockedKernelContext {
        context_identity: u64,
        caller_identity: u64,
        completion_identity: u64,
    },
    AwaitDeadlinePreemption {
        context_identity: u64,
    },
    TakePreemptedKernelContext {
        context_identity: u64,
        event_identity: u64,
    },
    RestorePreemptedKernelContext {
        context_identity: u64,
        caller_identity: u64,
        event_identity: u64,
    },
    EnterKernelThread {
        context_identity: u64,
    },
    RetireKernelContextToCaller {
        context_identity: u64,
        caller_identity: u64,
    },
    ResumeKernelContextCaller {
        caller_identity: u64,
    },
    ReclaimKernelContext {
        context_identity: u64,
    },
    ConsumeEmptyRunnableQueue,
    ConsumeKernelCompletion {
        completion_identity: u64,
    },
    PrepareUserAddressSpace {
        address_space_identity: u64,
    },
    VerifyUserAddressSpaceMapping {
        address_space_identity: u64,
    },
    MapUserAddressSpace {
        address_space_identity: u64,
        mapping_identity: u64,
    },
    ReserveAnonymousUserMapping {
        address_space_identity: u64,
        reservation_identity: u64,
    },
    StoreUserMappingByte {
        mapping_identity: u64,
    },
    InstallUserMappingEntryTrampoline {
        mapping_identity: u64,
    },
    ProtectUserMappingReadOnly {
        mapping_identity: u64,
    },
    LoadUserMappingByte {
        mapping_identity: u64,
    },
    VerifyUserMappingEntryTrampoline {
        mapping_identity: u64,
    },
    PermitUserMappingExecute {
        mapping_identity: u64,
    },
    SealUserMappingImmutableData {
        mapping_identity: u64,
    },
    PermitUserMappingWritableData {
        mapping_identity: u64,
    },
    UnmapUserAddressSpace {
        address_space_identity: u64,
        mapping_identity: u64,
    },
    ReleaseAnonymousUserMapping {
        address_space_identity: u64,
        reservation_identity: u64,
    },
    PrepareUserProbe {
        process_identity: u64,
        probe_identity: u64,
    },
    EnterUserExecution {
        process_identity: u64,
        kernel_continuation_identity: u64,
    },
    EnterUserSyscall {
        entry_identity: u64,
        number: u64,
        arguments: [u64; 6],
    },
    BeginUserTransfer {
        entry_identity: u64,
        transfer_identity: u64,
        candidate: u64,
        extent: u64,
    },
    CopyToUserComplete {
        transfer_identity: u64,
        copied: u64,
    },
    CopyToUserFault {
        transfer_identity: u64,
        copied_prefix: u64,
    },
    CopyFromUserComplete {
        transfer_identity: u64,
        copied: u64,
        seconds: i64,
        nanoseconds: i64,
    },
    CopyFromUserFault {
        transfer_identity: u64,
        copied_prefix: u64,
    },
    EndUserTransfer {
        entry_identity: u64,
        transfer_identity: u64,
    },
    ReturnToUser {
        entry_identity: u64,
        result: u64,
    },
    TerminateUser {
        entry_identity: u64,
        status: u64,
    },
    ResumeUserKernelContinuation {
        kernel_continuation_identity: u64,
        status: u64,
    },
    CompleteUserExecution {
        process_identity: u64,
    },
    ReclaimUserAddressSpace {
        address_space_identity: u64,
    },
    AllocateBootstrapRegion {
        request: BootstrapStorageRequest,
    },
    StoreBootstrapByte {
        offset_bytes: u64,
        value: u8,
    },
    LoadBootstrapByte {
        offset_bytes: u64,
        value: u8,
    },
    CreateAtomicWord {
        request: AtomicWordRequest,
    },
    CompareExchangeAtomicWord {
        expected: u64,
        desired: u64,
        observed: u64,
        exchanged: bool,
        success_order: AtomicOrder,
        failure_order: AtomicOrder,
    },
    LoadAtomicWord {
        value: u64,
        order: AtomicOrder,
    },
    EndAtomicWord,
    ReleaseBootstrapRegion,
    Fatal {
        message: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsModelError {
    pub code: &'static str,
    pub message: String,
}

impl SystemsModelError {
    pub(crate) fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for SystemsModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for SystemsModelError {}

impl SystemsTransition {
    #[must_use]
    pub const fn semantic_identity(&self) -> &'static str {
        match self {
            Self::ProvisionBootstrapStorage { .. } => SYSTEMS_BOOTSTRAP_STORAGE_PROVISION,
            Self::EnterBootstrap => "topal.systems.entry.bootstrap/1",
            Self::DescribeBootMemory => SYSTEMS_BOOT_MEMORY_DESCRIBE,
            Self::CreateFrameAllocator => SYSTEMS_FRAME_ALLOCATOR_CREATE,
            Self::AllocatePhysicalFrames { .. } => SYSTEMS_FRAMES_ALLOCATE,
            Self::ReleasePhysicalFrames => SYSTEMS_FRAMES_RELEASE,
            Self::PrepareStaticImageConstruction => SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PREPARE,
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
            Self::StoreKernelMappingByte { .. } => SYSTEMS_KERNEL_MAPPING_STORE_BYTE,
            Self::LoadKernelMappingByte { .. } => SYSTEMS_KERNEL_MAPPING_LOAD_BYTE,
            Self::UnmapKernelFrames => SYSTEMS_KERNEL_UNMAP,
            Self::BeginTranslationUpdate { .. } => SYSTEMS_TRANSLATION_BEGIN,
            Self::CommitTranslationUpdate => SYSTEMS_TRANSLATION_COMMIT,
            Self::ActivateTranslationSpace => SYSTEMS_TRANSLATION_ACTIVATE,
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
            Self::AccessPciRegionRegister { request, .. } => match request.direction {
                PciRegionRegisterDirection::ReadEquals => SYSTEMS_PCI_REGION_READ,
                PciRegionRegisterDirection::ReadEventuallyEquals => {
                    SYSTEMS_PCI_REGION_READ_EVENTUALLY
                }
                PciRegionRegisterDirection::Write => SYSTEMS_PCI_REGION_WRITE,
            },
            Self::ObserveDebugBreak => SYSTEMS_DEBUG_BREAK,
            Self::EnterDebugBreak => "topal.systems.entry.synchronous.debug-break/1",
            Self::ResumeDebugBreak => SYSTEMS_RESUME_DEBUG_BREAK,
            Self::SendLocalNotification { .. } => SYSTEMS_LOCAL_NOTIFICATION_SEND,
            Self::BeginLocalNotificationWait { .. } | Self::EndLocalNotificationWait { .. } => {
                SYSTEMS_LOCAL_NOTIFICATION_WAIT
            }
            Self::ObserveLocalNotification { .. } => {
                "topal.systems.observation.local-notification/1"
            }
            Self::EnterLocalNotificationInterrupt { .. } => {
                "topal.systems.entry.external.local-notification/1"
            }
            Self::CompleteLocalNotificationInterrupt { .. } => SYSTEMS_LOCAL_NOTIFICATION_COMPLETE,
            Self::ResumeLocalNotificationInterrupt { .. } => SYSTEMS_RESUME_LOCAL_NOTIFICATION,
            Self::ObserveMonotonicClock { .. } => SYSTEMS_MONOTONIC_CLOCK_NOW,
            Self::ConstructDeadline { .. } => SYSTEMS_DEADLINE_AFTER,
            Self::ArmDeadline { .. } => SYSTEMS_DEADLINE_ARM,
            Self::BeginDeadlineWait { .. } | Self::EndDeadlineWait { .. } => SYSTEMS_DEADLINE_WAIT,
            Self::ObserveDeadline { .. } => "topal.systems.observation.deadline/1",
            Self::EnterDeadlineInterrupt { .. } => "topal.systems.entry.external.deadline/1",
            Self::CompleteDeadlineInterrupt { .. } => SYSTEMS_DEADLINE_COMPLETE,
            Self::ResumeDeadlineInterrupt { .. } => SYSTEMS_RESUME_DEADLINE,
            Self::PreemptKernelContext { .. } => SYSTEMS_KERNEL_CONTEXT_PREEMPT_CURRENT,
            Self::ResumePreemptionDispatcher { .. } => {
                SYSTEMS_KERNEL_CONTEXT_TRANSFER_UNTIL_DEADLINE
            }
            Self::CreateKernelContext { .. } => SYSTEMS_KERNEL_CONTEXT_CREATE,
            Self::CreateKernelCompletion { .. } => SYSTEMS_KERNEL_COMPLETION_CREATE,
            Self::CreateRunnableQueue { .. } => SYSTEMS_KERNEL_RUNNABLE_QUEUE_CREATE,
            Self::EnqueueRunnableContext { .. } => SYSTEMS_KERNEL_RUNNABLE_QUEUE_ENQUEUE,
            Self::DequeueRunnableContext { .. } => SYSTEMS_KERNEL_RUNNABLE_QUEUE_DEQUEUE,
            Self::TransferKernelContext { .. } => SYSTEMS_KERNEL_CONTEXT_TRANSFER,
            Self::TransferKernelContextUntilDeadline { .. } => {
                SYSTEMS_KERNEL_CONTEXT_TRANSFER_UNTIL_DEADLINE
            }
            Self::TransferKernelContextUntilCompletion { .. }
            | Self::AwaitKernelCompletion { .. }
            | Self::BlockKernelContext { .. }
            | Self::ResumeCompletionDispatcher { .. } => SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE,
            Self::SignalBlockedKernelCompletion { .. }
            | Self::RestoreBlockedKernelContext { .. } => SYSTEMS_KERNEL_COMPLETION_SIGNAL,
            Self::AwaitDeadlinePreemption { .. } => {
                SYSTEMS_KERNEL_CONTEXT_AWAIT_DEADLINE_PREEMPTION
            }
            Self::TakePreemptedKernelContext { .. } => SYSTEMS_KERNEL_CONTEXT_TAKE_PREEMPTED,
            Self::RestorePreemptedKernelContext { .. } => SYSTEMS_KERNEL_CONTEXT_TRANSFER,
            Self::EnterKernelThread { .. } => SYSTEMS_KERNEL_THREAD_ENTRY,
            Self::RetireKernelContextToCaller { .. } => SYSTEMS_KERNEL_CONTEXT_RETIRE,
            Self::ResumeKernelContextCaller { .. } => SYSTEMS_KERNEL_CONTEXT_TRANSFER,
            Self::ReclaimKernelContext { .. } => SYSTEMS_KERNEL_CONTEXT_RECLAIM,
            Self::ConsumeEmptyRunnableQueue => SYSTEMS_KERNEL_RUNNABLE_QUEUE_CONSUME,
            Self::ConsumeKernelCompletion { .. } => SYSTEMS_KERNEL_COMPLETION_CONSUME,
            Self::PrepareUserAddressSpace { .. } => SYSTEMS_USER_ADDRESS_SPACE_PREPARE,
            Self::VerifyUserAddressSpaceMapping { .. } => SYSTEMS_USER_ADDRESS_SPACE_VERIFY_MAPPING,
            Self::MapUserAddressSpace { .. } => SYSTEMS_USER_ADDRESS_SPACE_MAP,
            Self::ReserveAnonymousUserMapping { .. } => {
                SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS
            }
            Self::StoreUserMappingByte { .. } => SYSTEMS_USER_MAPPING_STORE_BYTE,
            Self::InstallUserMappingEntryTrampoline { .. } => {
                SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE
            }
            Self::ProtectUserMappingReadOnly { .. } => SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY,
            Self::LoadUserMappingByte { .. } => SYSTEMS_USER_MAPPING_LOAD_BYTE,
            Self::VerifyUserMappingEntryTrampoline { .. } => {
                SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE
            }
            Self::PermitUserMappingExecute { .. } => SYSTEMS_USER_MAPPING_PERMIT_EXECUTE,
            Self::SealUserMappingImmutableData { .. } => SYSTEMS_USER_MAPPING_SEAL_IMMUTABLE_DATA,
            Self::PermitUserMappingWritableData { .. } => SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA,
            Self::UnmapUserAddressSpace { .. } => SYSTEMS_USER_ADDRESS_SPACE_UNMAP,
            Self::ReleaseAnonymousUserMapping { .. } => {
                SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS
            }
            Self::PrepareUserProbe { .. } => SYSTEMS_USER_CONTEXT_PREPARE,
            Self::EnterUserExecution { .. } => SYSTEMS_USER_CONTEXT_ENTER,
            Self::EnterUserSyscall { .. } => SYSTEMS_USER_SYSCALL_ENTRY,
            Self::BeginUserTransfer { .. } => SYSTEMS_USER_TRANSFER_BEGIN,
            Self::CopyToUserComplete { .. } | Self::CopyToUserFault { .. } => SYSTEMS_USER_COPY_TO,
            Self::CopyFromUserComplete { .. } | Self::CopyFromUserFault { .. } => {
                SYSTEMS_USER_COPY_FROM
            }
            Self::EndUserTransfer { .. } => SYSTEMS_USER_TRANSFER_END,
            Self::ReturnToUser { .. } => SYSTEMS_USER_SYSCALL_RETURN,
            Self::TerminateUser { .. } => SYSTEMS_USER_TERMINATE,
            Self::ResumeUserKernelContinuation { .. } => SYSTEMS_USER_CONTEXT_ENTER,
            Self::CompleteUserExecution { .. } => SYSTEMS_USER_CONTEXT_COMPLETE,
            Self::ReclaimUserAddressSpace { .. } => SYSTEMS_USER_ADDRESS_SPACE_RECLAIM,
            Self::AllocateBootstrapRegion { .. } => SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
            Self::StoreBootstrapByte { .. } => SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE,
            Self::LoadBootstrapByte { .. } => SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
            Self::CreateAtomicWord { .. } => SYSTEMS_ATOMIC_WORD_CREATE,
            Self::CompareExchangeAtomicWord { .. } => SYSTEMS_ATOMIC_COMPARE_EXCHANGE,
            Self::LoadAtomicWord { .. } => SYSTEMS_ATOMIC_LOAD,
            Self::EndAtomicWord => SYSTEMS_ATOMIC_END,
            Self::ReleaseBootstrapRegion => SYSTEMS_BOOTSTRAP_STORAGE_RELEASE,
            Self::Fatal { .. } => SYSTEMS_FATAL,
        }
    }
}

/// Validate that a program belongs to the sealed initial systems profile.
///
/// # Errors
///
/// Returns a stable systems-model diagnostic when the target identity, entry
/// contexts, admitted operations, dispositions, or derived effects do not
/// match the initial profile.
pub fn validate_systems_program(program: &SystemsProgram) -> Result<(), SystemsModelError> {
    if program.target != SystemsTargetSelection::initial_x86_64_qemu() {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-TARGET",
            "the initial systems model requires its exact target, board, and profile identity",
        ));
    }
    program.bootstrap_storage.validate()?;
    validate_entry(
        &program.bootstrap,
        SystemsEntryKind::Bootstrap,
        SystemsContextKind::Bootstrap,
    )?;
    validate_entry(
        &program.debug_break,
        SystemsEntryKind::SynchronousExceptionDebugBreak,
        SystemsContextKind::DebugBreak,
    )?;
    validate_entry(
        &program.local_notification,
        SystemsEntryKind::ExternalInterruptLocalNotification,
        SystemsContextKind::LocalNotificationInterrupt,
    )?;
    validate_entry(
        &program.deadline_notification,
        SystemsEntryKind::ExternalInterruptDeadline,
        SystemsContextKind::DeadlineInterrupt,
    )?;
    validate_entry(
        &program.kernel_thread,
        SystemsEntryKind::ResumedKernelThread,
        SystemsContextKind::KernelThread,
    )?;
    validate_entry(
        &program.completion_thread,
        SystemsEntryKind::ResumedKernelThread,
        SystemsContextKind::KernelThread,
    )?;
    validate_entry(
        &program.terminal_thread,
        SystemsEntryKind::ResumedKernelThread,
        SystemsContextKind::KernelThread,
    )?;
    validate_entry(
        &program.user_syscall,
        SystemsEntryKind::UserSyscall,
        SystemsContextKind::UserSyscall,
    )?;
    let names = [
        &program.bootstrap.handler.name,
        &program.debug_break.handler.name,
        &program.local_notification.handler.name,
        &program.deadline_notification.handler.name,
        &program.kernel_thread.handler.name,
        &program.completion_thread.handler.name,
        &program.terminal_thread.handler.name,
        &program.user_syscall.handler.name,
    ];
    if names
        .iter()
        .enumerate()
        .any(|(index, name)| names.iter().skip(index + 1).any(|other| *name == *other))
    {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-ENTRY-SET",
            "bootstrap, debug-break, local-notification, deadline, preemptible-thread, completion-thread, terminal-thread, and user-syscall entries require distinct handlers",
        ));
    }
    validate_local_notification_handler(&program.local_notification.handler)?;
    validate_deadline_handler(&program.deadline_notification.handler)?;
    validate_kernel_thread_handler(
        &program.kernel_thread.handler,
        KernelThreadProfile::Preemptible,
    )?;
    validate_kernel_thread_handler(
        &program.completion_thread.handler,
        KernelThreadProfile::CompletionBlocking,
    )?;
    validate_kernel_thread_handler(
        &program.terminal_thread.handler,
        KernelThreadProfile::Terminal,
    )?;
    validate_user_syscall_handler(&program.user_syscall.handler)?;
    validate_bootstrap_storage_operations(program)?;
    Ok(())
}

fn validate_entry(
    entry: &SystemsEntry,
    required_kind: SystemsEntryKind,
    required_context: SystemsContextKind,
) -> Result<(), SystemsModelError> {
    if entry.kind != required_kind || entry.handler.context != required_context {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-HANDLER",
            "systems entry kind and affine context do not match",
        ));
    }
    if entry.handler.name.is_empty() {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-HANDLER",
            "a systems entry requires a statically named handler",
        ));
    }
    if required_context != SystemsContextKind::Bootstrap
        && entry.handler.operations.iter().any(|operation| {
            let bootstrap_only = matches!(
                operation,
                SystemsOperation::DescribeBootMemory { .. }
                    | SystemsOperation::CreateFrameAllocator { .. }
                    | SystemsOperation::AllocatePhysicalFrames { .. }
                    | SystemsOperation::ReleasePhysicalFrames
                    | SystemsOperation::PrepareStaticImageConstruction { .. }
                    | SystemsOperation::ZeroStaticImageConstructionPayload
                    | SystemsOperation::ZeroStaticImageStackBacking
                    | SystemsOperation::PublishStaticImageStackLeaf
                    | SystemsOperation::InitializeStaticImageStackEntry
                    | SystemsOperation::AddStaticImageStackPageSize
                    | SystemsOperation::AddStaticImageStackEntryFact
                    | SystemsOperation::AddStaticImageStackProgramHeaders
                    | SystemsOperation::AddStaticImageStackExecutableName
                    | SystemsOperation::AddStaticImageStackCredentials
                    | SystemsOperation::AddStaticImageStackSecureMode
                    | SystemsOperation::AddStaticImageStackClockTickRate
                    | SystemsOperation::AddStaticImageStackPlatform
                    | SystemsOperation::AddStaticImageStackHardwareCapabilities
                    | SystemsOperation::AddStaticImageStackLoadMetadata
                    | SystemsOperation::AuthorizeStaticImageStackRandomSlot
                    | SystemsOperation::RetainStaticImagePreparedProcess
                    | SystemsOperation::QualifyStaticImageRetainedDisjointWork
                    | SystemsOperation::AuthorizeStaticImageEntropyHandoff
                    | SystemsOperation::AuthorizeStaticImageEntropyAcquisition
                    | SystemsOperation::QualifyStaticImageEntropyDevice
                    | SystemsOperation::QualifyStaticImageEntropyTransport
                    | SystemsOperation::QualifyStaticImageEntropyNegotiation
                    | SystemsOperation::QualifyStaticImageEntropyQueue
                    | SystemsOperation::QualifyStaticImageEntropyBusMaster
                    | SystemsOperation::QualifyStaticImageEntropyDmaBuffers
                    | SystemsOperation::QualifyStaticImageEntropySplitRing
                    | SystemsOperation::QualifyStaticImageEntropyDmaPreparation
                    | SystemsOperation::QualifyStaticImageEntropyQueueAddresses
                    | SystemsOperation::QualifyStaticImageEntropyQueueActivation
                    | SystemsOperation::QualifyStaticImageEntropyRequestCompletion
                    | SystemsOperation::QualifyStaticImageEntropySampleCapture
                    | SystemsOperation::QualifyStaticImageEntropySampleQuality
                    | SystemsOperation::QualifyStaticImageEntropyPoolLifecycle
                    | SystemsOperation::QualifyStaticImageEntropyGeneratorLifecycle
                    | SystemsOperation::DeliverStaticImageEntropyInitialRandom
                    | SystemsOperation::EnterStaticImageProcess
                    | SystemsOperation::RestoreStaticImageProcessEntry
                    | SystemsOperation::RevokeStaticImageEntropyInitialRandom
                    | SystemsOperation::RevokeStaticImageEntropyAcquisition
                    | SystemsOperation::RevokeStaticImageEntropyHandoff
                    | SystemsOperation::RestoreStaticImagePreparedProcess
                    | SystemsOperation::RevokeStaticImageStackRandomSlot
                    | SystemsOperation::RemoveStaticImageStackLoadMetadata
                    | SystemsOperation::RemoveStaticImageStackHardwareCapabilities
                    | SystemsOperation::RemoveStaticImageStackPlatform
                    | SystemsOperation::RemoveStaticImageStackClockTickRate
                    | SystemsOperation::RemoveStaticImageStackSecureMode
                    | SystemsOperation::RemoveStaticImageStackCredentials
                    | SystemsOperation::RemoveStaticImageStackExecutableName
                    | SystemsOperation::RemoveStaticImageStackProgramHeaders
                    | SystemsOperation::RemoveStaticImageStackEntryFact
                    | SystemsOperation::RemoveStaticImageStackPageSize
                    | SystemsOperation::ClearStaticImageStackEntry
                    | SystemsOperation::RevokeStaticImageStackLeaf
                    | SystemsOperation::CopyStaticImageConstructionPayload
                    | SystemsOperation::SealStaticImageConstructionPermissions
                    | SystemsOperation::AttachStaticImageAddressSpace
                    | SystemsOperation::ActivateStaticImageAddressSpace
                    | SystemsOperation::RestoreStaticImageAddressSpace
                    | SystemsOperation::AuthorizeStaticImageEntry
                    | SystemsOperation::AuthorizeStaticImageStackSlot
                    | SystemsOperation::RevokeStaticImageStackSlot
                    | SystemsOperation::RevokeStaticImageEntry
                    | SystemsOperation::DetachStaticImageAddressSpace
                    | SystemsOperation::ReclaimStaticImageConstructionHierarchy
                    | SystemsOperation::RevokeStaticImageConstructionMetadata
                    | SystemsOperation::ReclaimStaticImageConstructionPayload
                    | SystemsOperation::MapKernelFrames { .. }
                    | SystemsOperation::KernelMappingStoreByte { .. }
                    | SystemsOperation::KernelMappingLoadByteEquals { .. }
                    | SystemsOperation::UnmapKernelFrames
                    | SystemsOperation::BeginTranslationUpdate { .. }
                    | SystemsOperation::CommitTranslationUpdate { .. }
                    | SystemsOperation::ActivateTranslationSpace { .. }
                    | SystemsOperation::BeginTranslationEdit { .. }
                    | SystemsOperation::MapTranslationFrames { .. }
                    | SystemsOperation::UnmapTranslationMapping
                    | SystemsOperation::CommitTranslationEdit { .. }
                    | SystemsOperation::EnterCritical { .. }
                    | SystemsOperation::RestoreCritical { .. }
                    | SystemsOperation::DebugBreak
                    | SystemsOperation::BootstrapAllocate { .. }
                    | SystemsOperation::BootstrapStoreByte { .. }
                    | SystemsOperation::BootstrapLoadByteEquals { .. }
                    | SystemsOperation::AtomicWordCreate { .. }
                    | SystemsOperation::AtomicCompareExchangeEquals { .. }
                    | SystemsOperation::AtomicLoadEquals { .. }
                    | SystemsOperation::AtomicWordEnd
                    | SystemsOperation::BootstrapRelease
                    | SystemsOperation::SendLocalNotification
                    | SystemsOperation::WaitLocalNotification
                    | SystemsOperation::ObserveMonotonicClock
                    | SystemsOperation::ConstructDeadline { .. }
                    | SystemsOperation::ArmDeadline
                    | SystemsOperation::WaitDeadline
                    | SystemsOperation::CreateKernelContext
                    | SystemsOperation::CreateKernelCompletion
                    | SystemsOperation::CreateRunnableQueue { .. }
                    | SystemsOperation::EnqueueRunnableContext
                    | SystemsOperation::DequeueRunnableContext
                    | SystemsOperation::TransferKernelContextUntilDeadline
                    | SystemsOperation::TransferKernelContextUntilCompletion
                    | SystemsOperation::TakePreemptedKernelContext
                    | SystemsOperation::SignalBlockedKernelCompletion
                    | SystemsOperation::ReclaimKernelContext
                    | SystemsOperation::ConsumeKernelCompletion
                    | SystemsOperation::ConsumeEmptyRunnableQueue
                    | SystemsOperation::PrepareUserAddressSpace
                    | SystemsOperation::VerifyUserAddressSpaceMapping
                    | SystemsOperation::MapUserAddressSpace
                    | SystemsOperation::StoreUserMappingByte
                    | SystemsOperation::LoadUserMappingByte
                    | SystemsOperation::UnmapUserAddressSpace
                    | SystemsOperation::PrepareUserProbe { .. }
                    | SystemsOperation::EnterUserContext
                    | SystemsOperation::CompleteUserExecution
                    | SystemsOperation::ReclaimUserAddressSpace
            );
            bootstrap_only
                || (required_context != SystemsContextKind::KernelThread
                    && matches!(operation, SystemsOperation::TransferKernelContext))
                || (required_context == SystemsContextKind::DebugBreak
                    && matches!(
                        operation,
                        SystemsOperation::CompleteLocalNotification
                            | SystemsOperation::CompleteDeadline
                    ))
                || (required_context == SystemsContextKind::LocalNotificationInterrupt
                    && !matches!(operation, SystemsOperation::CompleteLocalNotification))
                || (required_context == SystemsContextKind::DeadlineInterrupt
                    && !matches!(operation, SystemsOperation::CompleteDeadline))
                || (required_context == SystemsContextKind::KernelThread
                    && !matches!(
                        operation,
                        SystemsOperation::ConsoleWrite { .. }
                            | SystemsOperation::TransferKernelContext
                            | SystemsOperation::AwaitDeadlinePreemption
                            | SystemsOperation::AwaitKernelCompletion
                    ))
                || (required_context == SystemsContextKind::UserSyscall
                    && !matches!(
                        operation,
                        SystemsOperation::CopyInitialUnameToUser
                            | SystemsOperation::CopyInitialTimespecFromUser
                    ))
        })
    {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-OPERATION",
            "operation is not admitted by this live special-entry context",
        ));
    }
    match (required_context, &entry.handler.disposition) {
        (SystemsContextKind::Bootstrap, SystemsDisposition::Fatal { .. })
        | (
            SystemsContextKind::DebugBreak,
            SystemsDisposition::Resume | SystemsDisposition::Fatal { .. },
        )
        | (
            SystemsContextKind::LocalNotificationInterrupt,
            SystemsDisposition::Resume | SystemsDisposition::Fatal { .. },
        )
        | (
            SystemsContextKind::DeadlineInterrupt,
            SystemsDisposition::PreemptKernelContext | SystemsDisposition::Fatal { .. },
        )
        | (SystemsContextKind::KernelThread, SystemsDisposition::RetireKernelContextToCaller) => {}
        (
            SystemsContextKind::UserSyscall,
            SystemsDisposition::DispatchInitialUserSyscall { .. },
        ) => {}
        (SystemsContextKind::Bootstrap, SystemsDisposition::Resume)
        | (
            SystemsContextKind::Bootstrap
            | SystemsContextKind::DebugBreak
            | SystemsContextKind::LocalNotificationInterrupt
            | SystemsContextKind::DeadlineInterrupt
            | SystemsContextKind::UserSyscall,
            SystemsDisposition::RetireKernelContextToCaller,
        )
        | (
            SystemsContextKind::Bootstrap
            | SystemsContextKind::DebugBreak
            | SystemsContextKind::LocalNotificationInterrupt
            | SystemsContextKind::KernelThread
            | SystemsContextKind::UserSyscall,
            SystemsDisposition::PreemptKernelContext,
        )
        | (SystemsContextKind::DeadlineInterrupt, SystemsDisposition::Resume)
        | (SystemsContextKind::UserSyscall, SystemsDisposition::Resume)
        | (SystemsContextKind::UserSyscall, SystemsDisposition::Fatal { .. })
        | (
            SystemsContextKind::Bootstrap
            | SystemsContextKind::DebugBreak
            | SystemsContextKind::LocalNotificationInterrupt
            | SystemsContextKind::DeadlineInterrupt
            | SystemsContextKind::KernelThread,
            SystemsDisposition::DispatchInitialUserSyscall { .. },
        )
        | (SystemsContextKind::KernelThread, _) => {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-DISPOSITION",
                "resume is admitted only by a live resumable special-entry context",
            ));
        }
    }

    let mut expected_effects = entry
        .handler
        .operations
        .iter()
        .map(|operation| operation.semantic_identity().to_owned())
        .collect::<Vec<_>>();
    expected_effects.push(
        entry
            .handler
            .disposition
            .semantic_identity(required_context)
            .to_owned(),
    );
    expected_effects.sort();
    expected_effects.dedup();
    if entry.handler.effects != expected_effects {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-EFFECTS",
            format!(
                "systems handler `{}` effects must equal its sorted, deduplicated semantic identities",
                entry.handler.name
            ),
        ));
    }
    Ok(())
}

fn validate_local_notification_handler(handler: &SystemsHandler) -> Result<(), SystemsModelError> {
    if handler.operations != [SystemsOperation::CompleteLocalNotification]
        || handler.disposition != SystemsDisposition::Resume
    {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-LOCAL-INTERRUPT",
            "the local-notification handler must consume completion authority before resume",
        ));
    }
    Ok(())
}

fn validate_deadline_handler(handler: &SystemsHandler) -> Result<(), SystemsModelError> {
    if handler.operations != [SystemsOperation::CompleteDeadline]
        || handler.disposition != SystemsDisposition::PreemptKernelContext
    {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-DEADLINE-EVENT",
            "the deadline handler must consume completion authority before preempting the bound context",
        ));
    }
    Ok(())
}

fn validate_user_syscall_handler(handler: &SystemsHandler) -> Result<(), SystemsModelError> {
    if handler.operations
        != [
            SystemsOperation::CopyInitialUnameToUser,
            SystemsOperation::CopyInitialTimespecFromUser,
        ]
        || handler.disposition
            != (SystemsDisposition::DispatchInitialUserSyscall {
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
        return Err(SystemsModelError::new(
            "E-SYSTEMS-USER-CONTEXT",
            "the initial user-syscall handler must dispatch the sealed implemented calls and unsupported-syscall fallback",
        ));
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum KernelThreadProfile {
    Preemptible,
    CompletionBlocking,
    Terminal,
}

fn validate_kernel_thread_handler(
    handler: &SystemsHandler,
    profile: KernelThreadProfile,
) -> Result<(), SystemsModelError> {
    let expected = match profile {
        KernelThreadProfile::Preemptible => vec![
            SystemsOperation::ConsoleWrite {
                text: "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_ENTERED".into(),
            },
            SystemsOperation::AwaitDeadlinePreemption,
            SystemsOperation::ConsoleWrite {
                text: "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_RESUMED".into(),
            },
        ],
        KernelThreadProfile::CompletionBlocking => vec![
            SystemsOperation::ConsoleWrite {
                text: "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_ENTERED".into(),
            },
            SystemsOperation::AwaitKernelCompletion,
            SystemsOperation::ConsoleWrite {
                text: "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_RESUMED".into(),
            },
        ],
        KernelThreadProfile::Terminal => vec![SystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_CONTEXT_TERMINAL_ENTERED".into(),
        }],
    };
    if handler.operations != expected
        || handler.disposition != SystemsDisposition::RetireKernelContextToCaller
    {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-CONTEXT-TRANSFER",
            "preemptible, completion-blocking, and terminal kernel-thread handlers must implement their exact transfer protocols",
        ));
    }
    Ok(())
}

#[derive(Default)]
struct BootstrapAuthorityState {
    memory_described: bool,
    allocator_created: bool,
    memory_ownership: BootstrapMemoryOwnership,
    retained_static_process: bool,
    entropy_pending_static_process: bool,
    entropy_acquiring_static_process: bool,
    randomized_static_process: bool,
    completed_static_process: bool,
    translation: BootstrapTranslationState,
    critical_stack: Vec<(CriticalDomain, u64)>,
    next_critical_identity: u64,
    local_notification: BootstrapLocalNotificationState,
    monotonic_clock_observations: u8,
    deadline: BootstrapDeadlineState,
    kernel_context: BootstrapKernelContextState,
    kernel_completion: BootstrapKernelCompletionState,
    user_execution: BootstrapUserExecutionState,
    resident_user_image_ready: bool,
    anonymous_user_mapping_reserved: bool,
    initial_boot_resource_validated: bool,
    multipage_user_executable_completed: bool,
    constructed_user_read_completed: bool,
    constructed_user_close_completed: bool,
    constructed_user_dup_completed: bool,
    constructed_user_fcntl_completed: bool,
    constructed_user_fstat_completed: bool,
    constructed_user_tcgets_completed: bool,
    constructed_user_winsize_completed: bool,
    constructed_user_lseek_completed: bool,
    constructed_user_image_entered: bool,
    constructed_user_initial_stack_validated: bool,
    constructed_user_initial_auxv_validated: bool,
    constructed_user_initial_entry_auxv_validated: bool,
    constructed_user_initial_execfn_auxv_validated: bool,
    constructed_user_initial_credential_auxv_validated: bool,
    constructed_user_elf_identity_validated: bool,
    external_user_elf_loaded: bool,
    external_user_elf_copy_verified: bool,
    runtime_user_code_extent_validated: bool,
    runtime_user_entry_validated: bool,
    runtime_user_data_extent_validated: bool,
    external_user_bss_validated: bool,
    constructed_user_image_data_observed: bool,
    constructed_user_image_copy_to_completed: bool,
    constructed_user_image_copy_from_completed: bool,
    constructed_user_image_syscall_completed: bool,
    resident_user_image_retained: bool,
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
    user_random_service_completed: bool,
    initial_user_random_delivered: bool,
}

#[derive(Default, Eq, PartialEq)]
enum BootstrapLocalNotificationState {
    #[default]
    Fresh,
    Pending,
    Completed,
}

#[derive(Default, Eq, PartialEq)]
enum BootstrapDeadlineState {
    #[default]
    Fresh,
    Constructed,
    Armed,
    Completed,
}

#[derive(Default, Eq, PartialEq)]
enum BootstrapKernelContextState {
    #[default]
    Fresh,
    PreemptibleCreated,
    CompletionCreated,
    AllSuspended,
    QueueCreated,
    PreemptibleEnqueued,
    CompletionEnqueued,
    AllEnqueued,
    PreemptibleDequeued,
    PreemptedOutcome,
    PreemptedTaken,
    PreemptedEnqueued,
    CompletionDequeued,
    CompletionBlocked,
    TerminalDequeued,
    TerminalCompleted,
    TerminalReclaimed,
    CompletionSignaled,
    CompletionEnqueuedAgain,
    PreemptibleDequeuedAgain,
    PreemptibleCompleted,
    PreemptibleReclaimed,
    CompletionDequeuedAgain,
    CompletionCompleted,
    CompletionReclaimed,
    QueueConsumed,
}

#[derive(Default, Eq, PartialEq)]
enum BootstrapKernelCompletionState {
    #[default]
    Fresh,
    Created,
    Blocked,
    Signaled,
    Consumed,
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
enum BootstrapUserExecutionState {
    #[default]
    Fresh,
    AddressSpacePrepared(PhysicalFrameRequest),
    MappingLive(PhysicalFrameRequest, u8),
    MappingStored(PhysicalFrameRequest, u8),
    MappingEntryInstalled(PhysicalFrameRequest, u8),
    MappingProtected(PhysicalFrameRequest, u8),
    MappingVerified(PhysicalFrameRequest, u8),
    MappingExecutable(PhysicalFrameRequest, u8),
    MappingImmutableData(PhysicalFrameRequest, u8),
    MappingWritableData(PhysicalFrameRequest, u8),
    AddressSpaceVerified(PhysicalFrameRequest),
    PreparedWithResidentImage(PhysicalFrameRequest),
    CompletedWithResidentImage(PhysicalFrameRequest),
    CompletedWithExecutableImage(PhysicalFrameRequest),
    CompletedImageRevoked(PhysicalFrameRequest),
    Revoked(PhysicalFrameRequest),
    Consumed,
}

#[derive(Clone, Copy, Default, Eq, PartialEq)]
enum BootstrapMemoryOwnership {
    #[default]
    None,
    Frames(PhysicalFrameRequest),
    UserMapping(PhysicalFrameRequest),
    Mapping(PhysicalFrameRequest),
    ProvisionalMapping(PhysicalFrameRequest),
    ProvisionalFrames(PhysicalFrameRequest),
    StaticElfMappingResources,
    StaticElfConstructionLive,
    StaticElfConstructionZeroed,
    StaticElfConstructionPopulated,
    StaticElfConstructionSealed,
    StaticElfConstructionSealedWithHierarchy,
    StaticElfConstructionAttached,
    StaticElfConstructionActive,
    StaticElfConstructionRestored,
    StaticElfConstructionEntryAuthorized,
    StaticElfConstructionStackSlotAuthorized,
    StaticElfConstructionStackSlotAuthorizedWithBacking,
    StaticElfConstructionStackBackingZeroed,
    StaticElfConstructionStackLeafPublished,
    StaticElfConstructionStackEntryInitialized,
    StaticElfConstructionStackPageSizeAdded,
    StaticElfConstructionStackEntryFactAdded,
    StaticElfConstructionStackProgramHeadersAdded,
    StaticElfConstructionStackExecutableNameAdded,
    StaticElfConstructionStackCredentialsAdded,
    StaticElfConstructionStackSecureModeAdded,
    StaticElfConstructionStackClockTickRateAdded,
    StaticElfConstructionStackPlatformAdded,
    StaticElfConstructionStackHardwareCapabilitiesAdded,
    StaticElfConstructionStackLoadMetadataAdded,
    StaticElfConstructionStackRandomSlotAuthorized,
    StaticElfConstructionEntryRestored,
    StaticElfConstructionDetached,
    StaticElfConstructionHierarchyReturned,
    StaticElfConstructionMetadataReturned,
    StaticElfConstructionRevoked,
}

const INITIAL_SINGLE_FRAME_REQUEST: PhysicalFrameRequest = PhysicalFrameRequest {
    frame_count: 1,
    alignment_frames: 1,
};

const INITIAL_TWO_FRAME_REQUEST: PhysicalFrameRequest = PhysicalFrameRequest {
    frame_count: 2,
    alignment_frames: 1,
};

const INITIAL_MULTI_FRAME_REQUEST: PhysicalFrameRequest = PhysicalFrameRequest {
    frame_count: 3,
    alignment_frames: 1,
};

const STATIC_ELF_HIERARCHY_FRAME_REQUEST: PhysicalFrameRequest = PhysicalFrameRequest {
    frame_count: 5,
    alignment_frames: 1,
};

const POST_RANDOM_ANONYMOUS_EXTENT_REQUEST: PhysicalFrameRequest = PhysicalFrameRequest {
    frame_count: 34,
    alignment_frames: 1,
};

const STATIC_ELF_FRAME_REQUEST: PhysicalFrameRequest = PhysicalFrameRequest {
    frame_count: 503,
    alignment_frames: 1,
};

#[derive(Default, Eq, PartialEq)]
enum BootstrapTranslationState {
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

impl BootstrapAuthorityState {
    #[allow(clippy::too_many_lines)] // The sealed bootstrap transition order is audited together.
    fn observe(
        &mut self,
        index: usize,
        operation: &SystemsOperation,
    ) -> Result<bool, SystemsModelError> {
        if self.observe_critical(operation)? {
            return Ok(true);
        }
        if self.observe_local_notification(operation)? {
            return Ok(true);
        }
        if self.observe_deadline(operation)? {
            return Ok(true);
        }
        if self.observe_kernel_context(operation)? {
            return Ok(true);
        }
        if self.observe_user_execution(operation)? {
            return Ok(true);
        }
        match operation {
            SystemsOperation::DescribeBootMemory { .. } => {
                if index != 0 || self.memory_described {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-BOOT-MEMORY",
                        "boot memory must be described exactly once as the first bootstrap operation",
                    ));
                }
                self.memory_described = true;
            }
            SystemsOperation::CreateFrameAllocator { .. } => {
                if index != 1 || !self.memory_described || self.allocator_created {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-FRAMES",
                        "the frame allocator must consume memory description exactly once as the second bootstrap operation",
                    ));
                }
                self.allocator_created = true;
            }
            SystemsOperation::AllocatePhysicalFrames { request, .. } => {
                if !self.allocator_created {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-FRAMES",
                        "physical-frame allocation requires one allocator and an admitted static ownership state",
                    ));
                }
                self.memory_ownership = match (self.memory_ownership, *request) {
                    (
                        BootstrapMemoryOwnership::None,
                        request @ (INITIAL_SINGLE_FRAME_REQUEST
                        | INITIAL_TWO_FRAME_REQUEST
                        | INITIAL_MULTI_FRAME_REQUEST
                        | POST_RANDOM_ANONYMOUS_EXTENT_REQUEST
                        | STATIC_ELF_FRAME_REQUEST),
                    ) => BootstrapMemoryOwnership::Frames(request),
                    (
                        BootstrapMemoryOwnership::Frames(STATIC_ELF_FRAME_REQUEST),
                        INITIAL_SINGLE_FRAME_REQUEST,
                    ) => BootstrapMemoryOwnership::StaticElfMappingResources,
                    (
                        BootstrapMemoryOwnership::StaticElfConstructionSealed,
                        STATIC_ELF_HIERARCHY_FRAME_REQUEST,
                    ) => BootstrapMemoryOwnership::StaticElfConstructionSealedWithHierarchy,
                    (
                        BootstrapMemoryOwnership::StaticElfConstructionStackSlotAuthorized,
                        INITIAL_TWO_FRAME_REQUEST,
                    ) => {
                        BootstrapMemoryOwnership::StaticElfConstructionStackSlotAuthorizedWithBacking
                    }
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-FRAMES",
                            "physical-frame allocation requires an admitted static extent shape and exact nested ownership",
                        ));
                    }
                };
            }
            SystemsOperation::ReleasePhysicalFrames => {
                if matches!(
                    self.memory_ownership,
                    BootstrapMemoryOwnership::UserMapping(_)
                ) {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "physical-frame release is invalid while a user mapping owns the extent",
                    ));
                }
                self.memory_ownership = match self.memory_ownership {
                    BootstrapMemoryOwnership::StaticElfMappingResources => {
                        BootstrapMemoryOwnership::Frames(STATIC_ELF_FRAME_REQUEST)
                    }
                    BootstrapMemoryOwnership::StaticElfConstructionMetadataReturned => {
                        BootstrapMemoryOwnership::StaticElfConstructionRevoked
                    }
                    BootstrapMemoryOwnership::StaticElfConstructionSealedWithHierarchy => {
                        BootstrapMemoryOwnership::StaticElfConstructionSealed
                    }
                    BootstrapMemoryOwnership::StaticElfConstructionStackBackingZeroed => {
                        BootstrapMemoryOwnership::StaticElfConstructionStackSlotAuthorized
                    }
                    BootstrapMemoryOwnership::StaticElfConstructionHierarchyReturned => {
                        BootstrapMemoryOwnership::StaticElfConstructionSealed
                    }
                    BootstrapMemoryOwnership::Frames(_) => BootstrapMemoryOwnership::None,
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-FRAMES",
                            "physical-frame release requires the next exact live extent",
                        ));
                    }
                };
            }
            SystemsOperation::PrepareStaticImageConstruction { .. } => {
                if self.memory_ownership != BootstrapMemoryOwnership::StaticElfMappingResources {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image construction requires the exact live payload and metadata resources",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::StaticElfConstructionLive;
            }
            SystemsOperation::ZeroStaticImageConstructionPayload => {
                if self.memory_ownership != BootstrapMemoryOwnership::StaticElfConstructionLive {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image payload zeroing requires one live uninitialized construction",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::StaticElfConstructionZeroed;
            }
            SystemsOperation::CopyStaticImageConstructionPayload => {
                if self.memory_ownership != BootstrapMemoryOwnership::StaticElfConstructionZeroed {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image payload copy requires one completely zeroed construction",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::StaticElfConstructionPopulated;
            }
            SystemsOperation::SealStaticImageConstructionPermissions => {
                if self.memory_ownership != BootstrapMemoryOwnership::StaticElfConstructionPopulated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image permission sealing requires one populated construction",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::StaticElfConstructionSealed;
            }
            SystemsOperation::AttachStaticImageAddressSpace => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionSealedWithHierarchy
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image address-space attachment requires one sealed construction and its live five-frame hierarchy",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::StaticElfConstructionAttached;
            }
            SystemsOperation::ActivateStaticImageAddressSpace => {
                if self.memory_ownership != BootstrapMemoryOwnership::StaticElfConstructionAttached
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image address-space activation requires one fresh attached construction",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::StaticElfConstructionActive;
            }
            SystemsOperation::RestoreStaticImageAddressSpace => {
                if self.memory_ownership != BootstrapMemoryOwnership::StaticElfConstructionActive {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image address-space restoration requires one active construction",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::StaticElfConstructionRestored;
            }
            SystemsOperation::AuthorizeStaticImageEntry => {
                if self.memory_ownership != BootstrapMemoryOwnership::StaticElfConstructionRestored
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entry authorization requires one restored attached construction",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionEntryAuthorized;
            }
            SystemsOperation::RevokeStaticImageEntry => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionEntryAuthorized
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entry revocation requires one authorized construction",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionEntryRestored;
            }
            SystemsOperation::AuthorizeStaticImageStackSlot => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionEntryAuthorized
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack-slot authorization requires one entry-authorized construction",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackSlotAuthorized;
            }
            SystemsOperation::ZeroStaticImageStackBacking => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackSlotAuthorizedWithBacking
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack-backing zeroing requires one slot-authorized construction with live backing",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackBackingZeroed;
            }
            SystemsOperation::PublishStaticImageStackLeaf => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackBackingZeroed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack-leaf publication requires one zeroed stack backing",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackLeafPublished;
            }
            SystemsOperation::InitializeStaticImageStackEntry => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackLeafPublished
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack-entry initialization requires one published stack leaf",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackEntryInitialized;
            }
            SystemsOperation::AddStaticImageStackPageSize => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackEntryInitialized
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack page-size addition requires one initialized stack entry",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackPageSizeAdded;
            }
            SystemsOperation::AddStaticImageStackEntryFact => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackPageSizeAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack entry-fact addition requires one live page-size fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackEntryFactAdded;
            }
            SystemsOperation::AddStaticImageStackProgramHeaders => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackEntryFactAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack program-header addition requires one live entry fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackProgramHeadersAdded;
            }
            SystemsOperation::AddStaticImageStackExecutableName => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackProgramHeadersAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack executable-name addition requires one live program-header group",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackExecutableNameAdded;
            }
            SystemsOperation::AddStaticImageStackCredentials => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackExecutableNameAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack credential addition requires one live executable-name fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackCredentialsAdded;
            }
            SystemsOperation::AddStaticImageStackSecureMode => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackCredentialsAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack secure-mode addition requires one live credential group",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackSecureModeAdded;
            }
            SystemsOperation::AddStaticImageStackClockTickRate => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackSecureModeAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack clock-tick-rate addition requires one live security fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackClockTickRateAdded;
            }
            SystemsOperation::AddStaticImageStackPlatform => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackClockTickRateAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack platform addition requires one live clock-tick-rate fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackPlatformAdded;
            }
            SystemsOperation::RemoveStaticImageStackPlatform => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackPlatformAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack platform removal requires one live platform fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackClockTickRateAdded;
            }
            SystemsOperation::AddStaticImageStackHardwareCapabilities => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackPlatformAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack hardware-capability addition requires one live platform fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackHardwareCapabilitiesAdded;
            }
            SystemsOperation::AddStaticImageStackLoadMetadata => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackHardwareCapabilitiesAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack load-metadata addition requires one live hardware-capability group",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackLoadMetadataAdded;
            }
            SystemsOperation::AuthorizeStaticImageStackRandomSlot => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackLoadMetadataAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack random-slot authorization requires one live load-metadata group",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackRandomSlotAuthorized;
            }
            SystemsOperation::RetainStaticImagePreparedProcess => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackRandomSlotAuthorized
                    || self.retained_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image prepared-process retention requires one live random-slot-authorized construction",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::None;
                self.retained_static_process = true;
            }
            SystemsOperation::QualifyStaticImageRetainedDisjointWork => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.retained_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image retained disjoint work requires the exact retained process and closed allocator channel",
                    ));
                }
            }
            SystemsOperation::AuthorizeStaticImageEntropyHandoff => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.retained_static_process
                    || self.entropy_pending_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy handoff authorization requires the exact retained process",
                    ));
                }
                self.retained_static_process = false;
                self.entropy_pending_static_process = true;
            }
            SystemsOperation::AuthorizeStaticImageEntropyAcquisition => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.entropy_pending_static_process
                    || self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy acquisition authorization requires the exact entropy-pending process",
                    ));
                }
                self.entropy_pending_static_process = false;
                self.entropy_acquiring_static_process = true;
            }
            SystemsOperation::QualifyStaticImageEntropyDevice => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy-device qualification requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropyTransport => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy-transport qualification requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropyNegotiation => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy-transport negotiation requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropyQueue => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy-queue discovery requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropyBusMaster => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy bus-master qualification requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropyDmaBuffers => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy DMA-buffer coexistence requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropySplitRing => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy split-ring coexistence requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropyDmaPreparation => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy DMA-preparation coexistence requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropyQueueAddresses => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy queue-address coexistence requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropyQueueActivation => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy queue-activation coexistence requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropyRequestCompletion => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy request-completion coexistence requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropySampleCapture => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy-sample capture coexistence requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropySampleQuality => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy-sample quality coexistence requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropyPoolLifecycle => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy-pool lifecycle coexistence requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::QualifyStaticImageEntropyGeneratorLifecycle => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy-generator lifecycle coexistence requires the exact acquisition owner",
                    ));
                }
            }
            SystemsOperation::DeliverStaticImageEntropyInitialRandom => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                    || self.randomized_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image initial-random delivery requires the exact acquisition owner",
                    ));
                }
                self.entropy_acquiring_static_process = false;
                self.randomized_static_process = true;
            }
            SystemsOperation::EnterStaticImageProcess => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.randomized_static_process
                    || self.completed_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image process entry requires the exact randomized owner",
                    ));
                }
                self.randomized_static_process = false;
                self.completed_static_process = true;
            }
            SystemsOperation::RestoreStaticImageProcessEntry => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.randomized_static_process
                    || !self.completed_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image process-entry restoration requires the exact completed owner",
                    ));
                }
                self.completed_static_process = false;
                self.randomized_static_process = true;
            }
            SystemsOperation::RevokeStaticImageEntropyInitialRandom => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || self.entropy_acquiring_static_process
                    || !self.randomized_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image initial-random revocation requires the exact randomized owner",
                    ));
                }
                self.randomized_static_process = false;
                self.entropy_acquiring_static_process = true;
            }
            SystemsOperation::RevokeStaticImageEntropyAcquisition => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.entropy_pending_static_process
                    || !self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy acquisition revocation requires the exact acquisition owner",
                    ));
                }
                self.entropy_acquiring_static_process = false;
                self.entropy_pending_static_process = true;
            }
            SystemsOperation::RevokeStaticImageEntropyHandoff => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.retained_static_process
                    || !self.entropy_pending_static_process
                    || self.entropy_acquiring_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image entropy handoff revocation requires the exact entropy-pending process",
                    ));
                }
                self.entropy_pending_static_process = false;
                self.retained_static_process = true;
            }
            SystemsOperation::RestoreStaticImagePreparedProcess => {
                if self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.retained_static_process
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image prepared-process restoration requires the exact retained process and closed allocator channel",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackRandomSlotAuthorized;
                self.retained_static_process = false;
            }
            SystemsOperation::RevokeStaticImageStackRandomSlot => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackRandomSlotAuthorized
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack random-slot revocation requires one live random-slot authority",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackLoadMetadataAdded;
            }
            SystemsOperation::RemoveStaticImageStackLoadMetadata => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackLoadMetadataAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack load-metadata removal requires one live load-metadata group",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackHardwareCapabilitiesAdded;
            }
            SystemsOperation::RemoveStaticImageStackHardwareCapabilities => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackHardwareCapabilitiesAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack hardware-capability removal requires one live capability group",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackPlatformAdded;
            }
            SystemsOperation::RemoveStaticImageStackClockTickRate => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackClockTickRateAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack clock-tick-rate removal requires one live accounting-rate fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackSecureModeAdded;
            }
            SystemsOperation::RemoveStaticImageStackSecureMode => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackSecureModeAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack secure-mode removal requires one live security fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackCredentialsAdded;
            }
            SystemsOperation::RemoveStaticImageStackCredentials => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackCredentialsAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack credential removal requires one live credential group",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackExecutableNameAdded;
            }
            SystemsOperation::RemoveStaticImageStackExecutableName => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackExecutableNameAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack executable-name removal requires one live executable-name fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackProgramHeadersAdded;
            }
            SystemsOperation::RemoveStaticImageStackProgramHeaders => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackProgramHeadersAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack program-header removal requires one live program-header group",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackEntryFactAdded;
            }
            SystemsOperation::RemoveStaticImageStackEntryFact => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackEntryFactAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack entry-fact removal requires one live entry fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackPageSizeAdded;
            }
            SystemsOperation::RemoveStaticImageStackPageSize => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackPageSizeAdded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack page-size removal requires one live page-size fact",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackEntryInitialized;
            }
            SystemsOperation::ClearStaticImageStackEntry => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackEntryInitialized
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack-entry clearing requires one initialized stack entry",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackLeafPublished;
            }
            SystemsOperation::RevokeStaticImageStackLeaf => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackLeafPublished
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack-leaf revocation requires one published stack leaf",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionStackBackingZeroed;
            }
            SystemsOperation::RevokeStaticImageStackSlot => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionStackSlotAuthorized
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image stack-slot revocation requires one slot-authorized construction",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionEntryAuthorized;
            }
            SystemsOperation::DetachStaticImageAddressSpace => {
                if self.memory_ownership
                    != BootstrapMemoryOwnership::StaticElfConstructionEntryRestored
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image address-space detachment requires one attached inactive construction",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::StaticElfConstructionDetached;
            }
            SystemsOperation::ReclaimStaticImageConstructionHierarchy => {
                if self.memory_ownership != BootstrapMemoryOwnership::StaticElfConstructionDetached
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image hierarchy reclaim requires one detached construction",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionHierarchyReturned;
            }
            SystemsOperation::RevokeStaticImageConstructionMetadata => {
                if self.memory_ownership != BootstrapMemoryOwnership::StaticElfConstructionSealed {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image revocation requires one sealed initial-leaf construction",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::StaticElfConstructionMetadataReturned;
            }
            SystemsOperation::ReclaimStaticImageConstructionPayload => {
                if self.memory_ownership != BootstrapMemoryOwnership::StaticElfConstructionRevoked {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STATIC-IMAGE",
                        "static-image payload reclaim requires complete revocation and metadata release",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::Frames(STATIC_ELF_FRAME_REQUEST);
            }
            SystemsOperation::MapKernelFrames { request, .. } => {
                let BootstrapMemoryOwnership::Frames(frame_request) = self.memory_ownership else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-MAPPING",
                        "kernel mapping requires one live single-frame extent and the sealed mapping policy",
                    ));
                };
                if frame_request != INITIAL_SINGLE_FRAME_REQUEST
                    || *request != KernelMappingRequest::initial_read_write()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-MAPPING",
                        "kernel mapping requires one live single-frame extent and the sealed mapping policy",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::Mapping(frame_request);
            }
            SystemsOperation::KernelMappingStoreByte { offset_bytes, .. }
            | SystemsOperation::KernelMappingLoadByteEquals { offset_bytes, .. } => {
                if !matches!(self.memory_ownership, BootstrapMemoryOwnership::Mapping(_))
                    || *offset_bytes >= 4096
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-MAPPING",
                        "kernel mapping byte access requires one live mapping and an in-bounds offset",
                    ));
                }
            }
            SystemsOperation::UnmapKernelFrames => {
                let BootstrapMemoryOwnership::Mapping(frame_request) = self.memory_ownership else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-MAPPING",
                        "kernel unmap requires one live mapping",
                    ));
                };
                self.memory_ownership = BootstrapMemoryOwnership::Frames(frame_request);
            }
            operation @ (SystemsOperation::BeginTranslationUpdate { .. }
            | SystemsOperation::CommitTranslationUpdate { .. }
            | SystemsOperation::ActivateTranslationSpace { .. }
            | SystemsOperation::BeginTranslationEdit { .. }
            | SystemsOperation::MapTranslationFrames { .. }
            | SystemsOperation::UnmapTranslationMapping
            | SystemsOperation::CommitTranslationEdit { .. }) => {
                self.observe_translation(operation)?;
            }
            SystemsOperation::ObserveMonotonicClock => {
                if self.local_notification != BootstrapLocalNotificationState::Completed
                    || self.monotonic_clock_observations >= 2
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-MONOTONIC-CLOCK",
                        "monotonic-clock observation requires the resumed context and admits exactly two initial observations",
                    ));
                }
                self.monotonic_clock_observations += 1;
            }
            SystemsOperation::ObservePciConfigurationDword { request } => {
                let expected = PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                    .get(self.pci_configuration_observations);
                if !self.allocator_created
                    || !self.critical_stack.is_empty()
                    || !request.is_well_formed()
                    || expected != Some(request)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-PCI-CONFIG",
                        "the initial PCI configuration observations require the exact sealed Q35 virtio-rng capability-discovery sequence and a restored bootstrap context",
                    ));
                }
                self.pci_configuration_observations += 1;
            }
            SystemsOperation::AccessPciRegionRegister { request } => {
                let negotiation_len =
                    PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION.len();
                let discovery_len =
                    PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY.len();
                let activation_len =
                    PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION.len();
                let notification_len =
                    PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION.len();
                let expected = if self.pci_region_register_accesses < negotiation_len {
                    PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                        .get(self.pci_region_register_accesses)
                } else if self.pci_region_register_accesses < negotiation_len + discovery_len {
                    PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY
                        .get(self.pci_region_register_accesses - negotiation_len)
                } else if self.pci_region_register_accesses
                    < negotiation_len + discovery_len + activation_len
                {
                    PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
                        .get(self.pci_region_register_accesses - negotiation_len - discovery_len)
                } else if self.pci_region_register_accesses
                    < negotiation_len + discovery_len + activation_len + notification_len
                {
                    PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION.get(
                        self.pci_region_register_accesses
                            - negotiation_len
                            - discovery_len
                            - activation_len,
                    )
                } else {
                    PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL.get(
                        self.pci_region_register_accesses
                            - negotiation_len
                            - discovery_len
                            - activation_len
                            - notification_len,
                    )
                };
                if self.pci_configuration_observations
                    != PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY.len()
                    || !self.critical_stack.is_empty()
                    || (self.pci_region_register_accesses >= negotiation_len + discovery_len
                        && self.dma_references_published
                            != PciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
                                .len())
                    || !request.is_well_formed()
                    || expected != Some(request)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-PCI-REGION",
                        "PCI region access requires the completed capability discovery and exact sealed virtio-rng negotiation, queue-discovery, queue-activation, queue-notification, and completion-signal sequences",
                    ));
                }
                self.pci_region_register_accesses += 1;
            }
            SystemsOperation::UpdatePciConfigurationWord { request } => {
                if self.pci_region_register_accesses
                    != PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION.len()
                        + PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY.len()
                    || self.pci_bus_master_authorized
                    || !self.critical_stack.is_empty()
                    || !request.is_well_formed()
                    || *request
                        != PciConfigurationWordUpdateRequest::INITIAL_Q35_VIRTIO_RNG_BUS_MASTER
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-PCI-CONFIG-UPDATE",
                        "PCI configuration update requires complete queue discovery and the exact sealed bus-master command transition",
                    ));
                }
                self.pci_bus_master_authorized = true;
            }
            SystemsOperation::CreateDmaBuffer { request } => {
                let expected =
                    DmaBufferRequest::INITIAL_Q35_VIRTIO_RNG.get(self.dma_buffers_created);
                if !self.pci_bus_master_authorized
                    || !self.critical_stack.is_empty()
                    || !request.is_well_formed()
                    || expected != Some(request)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-DMA-BUFFER",
                        "DMA-buffer creation requires bus-master authorization and the exact sealed CPU-owned virtio-rng buffer sequence",
                    ));
                }
                self.dma_buffers_created += 1;
            }
            SystemsOperation::BindDmaBufferReference { request } => {
                if self.dma_buffers_created != DmaBufferRequest::INITIAL_Q35_VIRTIO_RNG.len()
                    || self.dma_construction_operations != 0
                    || !request.is_well_formed()
                    || *request != DmaBufferReferenceRequest::INITIAL_Q35_VIRTIO_RNG_DESCRIPTOR
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-DMA-CONSTRUCT",
                        "DMA reference binding requires both CPU-owned buffers and the exact sealed virtio-rng descriptor reference",
                    ));
                }
                self.dma_construction_operations = 1;
            }
            SystemsOperation::StoreDmaBufferInteger { request } => {
                let expected_index = self.dma_construction_operations.checked_sub(1);
                let expected = expected_index.and_then(|index| {
                    DmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING.get(index)
                });
                if self.dma_buffers_created != DmaBufferRequest::INITIAL_Q35_VIRTIO_RNG.len()
                    || !request.is_well_formed()
                    || expected != Some(request)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-DMA-CONSTRUCT",
                        "DMA integer store requires the exact sealed CPU-owned virtio-rng split-ring construction sequence",
                    ));
                }
                self.dma_construction_operations += 1;
            }
            SystemsOperation::PrepareDmaBufferForDevice { request } => {
                let expected = DmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG
                    .get(self.dma_buffers_prepared);
                if self.dma_construction_operations
                    != 1 + DmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING.len()
                    || !request.is_well_formed()
                    || expected != Some(request)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-DMA-PREPARE",
                        "DMA device preparation requires complete split-ring construction and the exact sealed payload-then-control ownership sequence",
                    ));
                }
                self.dma_buffers_prepared += 1;
            }
            SystemsOperation::CompleteDmaBufferForCpu { request } => {
                let required_region_accesses =
                    PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION.len()
                        + PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY.len()
                        + PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION.len()
                        + PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION.len()
                        + PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL.len();
                let completes_control = !self.dma_control_completed
                    && *request == DmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[1]
                    && self.dma_used_ring_reads == 0;
                let completes_payload = self.dma_control_completed
                    && !self.dma_payload_completed
                    && self.dma_used_ring_reads
                        == DmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING.len()
                    && *request == DmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0];
                if self.pci_region_register_accesses != required_region_accesses
                    || !request.is_well_formed()
                    || (!completes_control && !completes_payload)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-DMA-COMPLETE",
                        "DMA completion requires the exact sealed control-then-payload ownership sequence around virtio-rng used-ring validation",
                    ));
                }
                if completes_control {
                    self.dma_control_completed = true;
                } else {
                    self.dma_payload_completed = true;
                }
            }
            SystemsOperation::LoadDmaBufferIntegerEquals { request } => {
                let expected = DmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING
                    .get(self.dma_used_ring_reads);
                if !self.dma_control_completed
                    || !request.is_well_formed()
                    || expected != Some(request)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-DMA-USED-RING",
                        "DMA integer read requires CPU-owned control memory and the exact sealed virtio-rng used-ring validation sequence",
                    ));
                }
                self.dma_used_ring_reads += 1;
            }
            SystemsOperation::CaptureEntropySample { request } => {
                if !self.dma_payload_completed
                    || self.entropy_sample_captured
                    || !request.is_well_formed()
                    || *request != EntropySampleCaptureRequest::INITIAL_Q35_VIRTIO_RNG
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-SAMPLE",
                        "entropy-sample capture requires the exact sealed uncredited virtio-rng payload range after CPU completion",
                    ));
                }
                self.entropy_sample_captured = true;
            }
            SystemsOperation::AssessEntropySampleQuality { request } => {
                if !self.entropy_sample_captured
                    || self.entropy_sample_assessed
                    || !request.is_well_formed()
                    || *request != EntropySampleQualityRequest::INITIAL_Q35_VIRTIO_RNG
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-QUALITY",
                        "entropy-sample quality assessment requires the exact sealed Q35 trust evidence and full-quality credit after capture",
                    ));
                }
                self.entropy_sample_assessed = true;
            }
            SystemsOperation::CreateEntropyPool { request } => {
                if !self.entropy_sample_assessed
                    || self.entropy_pool_created
                    || !request.is_well_formed()
                    || *request != EntropyPoolRequest::INITIAL_Q35
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-POOL",
                        "entropy-pool creation requires the exact sealed empty 256-bit pool after sample assessment",
                    ));
                }
                self.entropy_pool_created = true;
            }
            SystemsOperation::MixEntropyPoolSample { request } => {
                if !self.entropy_sample_assessed
                    || !self.entropy_pool_created
                    || self.entropy_pool_mixed
                    || !request.is_well_formed()
                    || *request != EntropyPoolMixRequest::INITIAL_Q35
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-MIX",
                        "entropy-pool mixing requires the exact sealed assessed sample, empty pool, conditioner, and contribution",
                    ));
                }
                self.entropy_pool_mixed = true;
            }
            SystemsOperation::CreditEntropyPoolContribution { request } => {
                if !self.entropy_pool_mixed
                    || self.entropy_pool_credited
                    || !request.is_well_formed()
                    || *request != EntropyPoolCreditRequest::INITIAL_Q35
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-CREDIT",
                        "entropy-pool credit transfer requires the exact sealed mixed pool and live first contribution",
                    ));
                }
                self.entropy_pool_credited = true;
            }
            SystemsOperation::InitializeEntropyGenerator { request } => {
                if !self.entropy_pool_credited
                    || self.entropy_generator_ready
                    || !request.is_well_formed()
                    || *request != EntropyGeneratorRequest::INITIAL_Q35
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-GENERATOR",
                        "entropy-generator initialization requires the exact sealed credited pool and generation-one ChaCha20 request",
                    ));
                }
                self.entropy_generator_ready = true;
            }
            SystemsOperation::ExtractEntropyGenerator { request } => {
                if !self.entropy_generator_ready || !request.is_well_formed() {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-EXTRACT",
                        "entropy-generator extraction requires the next exact sealed ready ChaCha20 generation",
                    ));
                }
                if *request == EntropyGeneratorExtractRequest::INITIAL_Q35
                    && !self.entropy_generator_extracted
                {
                    self.entropy_generator_extracted = true;
                } else if *request == EntropyGeneratorExtractRequest::INITIAL_Q35_GETRANDOM
                    && self.entropy_generator_extracted
                    && self.initial_auxv_random_delivered
                    && !self.initial_getrandom_extracted
                {
                    self.initial_getrandom_extracted = true;
                } else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-EXTRACT",
                        "entropy-generator extraction requires the next exact sealed ready ChaCha20 generation",
                    ));
                }
            }
            SystemsOperation::DeliverInitialAuxvRandom { request } => {
                if !self.entropy_generator_extracted
                    || self.initial_auxv_random_delivered
                    || !request.is_well_formed()
                    || *request != EntropyOutputDeliveryRequest::INITIAL_AUXV_RANDOM
                    || !matches!(
                        self.user_execution,
                        BootstrapUserExecutionState::PreparedWithResidentImage(_)
                    )
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-DELIVER",
                        "initial AT_RANDOM delivery requires the exact live first output and prepared resident initial process before entry",
                    ));
                }
                self.initial_auxv_random_delivered = true;
            }
            SystemsOperation::ReserveInitialUserRandom { request } => {
                if !self.initial_getrandom_extracted
                    || self.initial_user_random_reserved
                    || !request.is_well_formed()
                    || *request != EntropyOutputReservationRequest::INITIAL_GETRANDOM
                    || !matches!(
                        self.user_execution,
                        BootstrapUserExecutionState::PreparedWithResidentImage(_)
                    )
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-RESERVE",
                        "initial getrandom reservation requires the exact live second output and prepared resident initial process before entry",
                    ));
                }
                self.initial_user_random_reserved = true;
            }
            SystemsOperation::BindReusableUserRandom { request } => {
                if !self.initial_user_random_reserved
                    || self.user_random_service_bound
                    || self.user_random_service_completed
                    || !request.is_well_formed()
                    || *request != EntropyUserRandomServiceRequest::INITIAL_GETRANDOM
                    || !matches!(
                        self.user_execution,
                        BootstrapUserExecutionState::PreparedWithResidentImage(_)
                    )
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-USER-SERVICE",
                        "reusable getrandom binding requires the generation-three generator, live initial reservation, and prepared resident initial process before entry",
                    ));
                }
                self.initial_user_random_reserved = false;
                self.user_random_service_bound = true;
            }
            SystemsOperation::ExpandSmallUserRandomRequests { request } => {
                if !self.user_random_service_bound
                    || self.user_random_small_requests_enabled
                    || self.user_random_service_completed
                    || !request.is_well_formed()
                    || *request != EntropyUserRandomSmallRequest::INITIAL_GETRANDOM
                    || !matches!(
                        self.user_execution,
                        BootstrapUserExecutionState::PreparedWithResidentImage(_)
                    )
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-USER-SMALL-REQUEST",
                        "small-request getrandom refinement requires the exact live reusable service and prepared resident initial process before entry",
                    ));
                }
                self.user_random_small_requests_enabled = true;
            }
            SystemsOperation::ExpandLargeUserRandomRequests { request } => {
                if !self.user_random_service_bound
                    || !self.user_random_small_requests_enabled
                    || self.user_random_large_requests_enabled
                    || self.user_random_service_completed
                    || !request.is_well_formed()
                    || *request != EntropyUserRandomSmallRequest::LARGE_GETRANDOM
                    || !matches!(
                        self.user_execution,
                        BootstrapUserExecutionState::PreparedWithResidentImage(_)
                    )
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-ENTROPY-USER-LARGE-REQUEST",
                        "large-request getrandom refinement requires the exact live small-request service and prepared resident initial process before entry",
                    ));
                }
                self.user_random_large_requests_enabled = true;
            }
            SystemsOperation::PublishPciRegionDmaReference { request } => {
                let expected = PciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
                    .get(self.dma_references_published);
                if self.dma_buffers_prepared
                    != DmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG.len()
                    || !request.is_well_formed()
                    || expected != Some(request)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-DMA-PUBLISH",
                        "PCI-region DMA-reference publication requires both device-owned buffers and the exact sealed descriptor-driver-device register sequence",
                    ));
                }
                self.dma_references_published += 1;
            }
            _ if !self.allocator_created => {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-FRAMES",
                    "bootstrap operations require a frame-allocator context",
                ));
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn observe_local_notification(
        &mut self,
        operation: &SystemsOperation,
    ) -> Result<bool, SystemsModelError> {
        match operation {
            SystemsOperation::SendLocalNotification => {
                if !self.allocator_created
                    || self.local_notification != BootstrapLocalNotificationState::Fresh
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-LOCAL-INTERRUPT",
                        "local notification send requires one restored processor context and no pending event",
                    ));
                }
                self.local_notification = BootstrapLocalNotificationState::Pending;
            }
            SystemsOperation::WaitLocalNotification => {
                if self.local_notification != BootstrapLocalNotificationState::Pending {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-LOCAL-INTERRUPT",
                        "local notification wait requires the matching affine pending session",
                    ));
                }
                self.local_notification = BootstrapLocalNotificationState::Completed;
            }
            SystemsOperation::CompleteLocalNotification => {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-LOCAL-INTERRUPT",
                    "local notification completion is admitted only by its external-interrupt entry",
                ));
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn observe_deadline(
        &mut self,
        operation: &SystemsOperation,
    ) -> Result<bool, SystemsModelError> {
        match operation {
            SystemsOperation::ConstructDeadline {
                duration_nanoseconds,
            } => {
                if self.monotonic_clock_observations != 2
                    || self.deadline != BootstrapDeadlineState::Fresh
                    || *duration_nanoseconds != 1_000_000
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-DEADLINE-EVENT",
                        "the initial deadline must derive once from the second observation and exact duration 1[ms]",
                    ));
                }
                self.deadline = BootstrapDeadlineState::Constructed;
            }
            SystemsOperation::ArmDeadline => {
                if self.deadline != BootstrapDeadlineState::Constructed
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-DEADLINE-EVENT",
                        "deadline arm requires the restored context and one fresh same-clock deadline",
                    ));
                }
                self.deadline = BootstrapDeadlineState::Armed;
            }
            SystemsOperation::WaitDeadline => {
                if self.deadline != BootstrapDeadlineState::Armed {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-DEADLINE-EVENT",
                        "deadline wait requires the matching affine armed event",
                    ));
                }
                self.deadline = BootstrapDeadlineState::Completed;
            }
            SystemsOperation::CompleteDeadline => {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-DEADLINE-EVENT",
                    "deadline completion is admitted only by its external-interrupt entry",
                ));
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn observe_critical(
        &mut self,
        operation: &SystemsOperation,
    ) -> Result<bool, SystemsModelError> {
        if !self.critical_stack.is_empty()
            && !matches!(
                operation,
                SystemsOperation::ConsoleWrite { .. }
                    | SystemsOperation::EnterCritical { .. }
                    | SystemsOperation::RestoreCritical { .. }
            )
        {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-CRITICAL",
                "a local-interrupt critical context admits only nonblocking scoped operations and matching restoration",
            ));
        }
        match operation {
            SystemsOperation::EnterCritical { domain, .. } => {
                if !self.allocator_created || *domain != CriticalDomain::LocalMaskableInterrupts {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CRITICAL",
                        "critical entry requires the live processor context and the local-maskable-interrupts domain",
                    ));
                }
                self.next_critical_identity += 1;
                self.critical_stack
                    .push((*domain, self.next_critical_identity));
            }
            SystemsOperation::RestoreCritical { domain } => {
                let Some((current_domain, _)) = self.critical_stack.pop() else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CRITICAL",
                        "critical restore requires live affine restoration authority",
                    ));
                };
                if current_domain != *domain {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CRITICAL",
                        "critical scopes must restore their matching domain in last-in-first-out order",
                    ));
                }
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn observe_kernel_context(
        &mut self,
        operation: &SystemsOperation,
    ) -> Result<bool, SystemsModelError> {
        match operation {
            SystemsOperation::CreateKernelCompletion => {
                if self.deadline != BootstrapDeadlineState::Constructed
                    || self.kernel_completion != BootstrapKernelCompletionState::Fresh
                    || self.kernel_context != BootstrapKernelContextState::Fresh
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "one-shot completion creation requires the restored dispatcher before worker creation",
                    ));
                }
                self.kernel_completion = BootstrapKernelCompletionState::Created;
            }
            SystemsOperation::CreateKernelContext => {
                if self.deadline != BootstrapDeadlineState::Constructed
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "kernel-context creation requires one constructed deadline and restored processor context",
                    ));
                }
                self.kernel_context = match self.kernel_context {
                    BootstrapKernelContextState::Fresh => {
                        BootstrapKernelContextState::PreemptibleCreated
                    }
                    BootstrapKernelContextState::PreemptibleCreated => {
                        BootstrapKernelContextState::CompletionCreated
                    }
                    BootstrapKernelContextState::CompletionCreated => {
                        BootstrapKernelContextState::AllSuspended
                    }
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-CONTEXT-TRANSFER",
                            "the completion profile creates exactly three contexts before dispatch",
                        ));
                    }
                };
            }
            SystemsOperation::TransferKernelContext => {
                if !self.critical_stack.is_empty() {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "kernel-context transfer requires no live critical scope",
                    ));
                }
                self.kernel_context = match self.kernel_context {
                    BootstrapKernelContextState::TerminalDequeued => {
                        BootstrapKernelContextState::TerminalCompleted
                    }
                    BootstrapKernelContextState::PreemptibleDequeuedAgain => {
                        BootstrapKernelContextState::PreemptibleCompleted
                    }
                    BootstrapKernelContextState::CompletionDequeuedAgain => {
                        BootstrapKernelContextState::CompletionCompleted
                    }
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-CONTEXT-TRANSFER",
                            "ordinary kernel-context transfers must follow terminal, preemptible, then awakened-completion redispatch order",
                        ));
                    }
                };
            }
            SystemsOperation::TransferKernelContextUntilDeadline => {
                if !self.critical_stack.is_empty()
                    || self.deadline != BootstrapDeadlineState::Constructed
                    || self.kernel_context != BootstrapKernelContextState::PreemptibleDequeued
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "deadline-preemptible transfer requires the first FIFO context, constructed deadline, and no critical scope",
                    ));
                }
                self.deadline = BootstrapDeadlineState::Completed;
                self.kernel_context = BootstrapKernelContextState::PreemptedOutcome;
            }
            SystemsOperation::TakePreemptedKernelContext => {
                if self.kernel_context != BootstrapKernelContextState::PreemptedOutcome
                    || self.deadline != BootstrapDeadlineState::Completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "taking a preempted context requires the matching completed deadline transfer",
                    ));
                }
                self.kernel_context = BootstrapKernelContextState::PreemptedTaken;
            }
            SystemsOperation::TransferKernelContextUntilCompletion => {
                if !self.critical_stack.is_empty()
                    || self.kernel_context != BootstrapKernelContextState::CompletionDequeued
                    || self.kernel_completion != BootstrapKernelCompletionState::Created
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "completion-bound transfer requires the selected fresh block-once worker and restored dispatcher",
                    ));
                }
                self.kernel_context = BootstrapKernelContextState::CompletionBlocked;
                self.kernel_completion = BootstrapKernelCompletionState::Blocked;
            }
            SystemsOperation::SignalBlockedKernelCompletion => {
                if self.kernel_context != BootstrapKernelContextState::TerminalReclaimed
                    || self.kernel_completion != BootstrapKernelCompletionState::Blocked
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "completion signal requires the matching blocked worker after terminal reclamation",
                    ));
                }
                self.kernel_context = BootstrapKernelContextState::CompletionSignaled;
                self.kernel_completion = BootstrapKernelCompletionState::Signaled;
            }
            SystemsOperation::CreateRunnableQueue { capacity } => {
                if self.kernel_context != BootstrapKernelContextState::AllSuspended
                    || *capacity != 3
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "the initial runnable queue requires all three suspended contexts and capacity three",
                    ));
                }
                self.kernel_context = BootstrapKernelContextState::QueueCreated;
            }
            SystemsOperation::EnqueueRunnableContext => {
                self.kernel_context = match self.kernel_context {
                    BootstrapKernelContextState::QueueCreated => {
                        BootstrapKernelContextState::PreemptibleEnqueued
                    }
                    BootstrapKernelContextState::PreemptibleEnqueued => {
                        BootstrapKernelContextState::CompletionEnqueued
                    }
                    BootstrapKernelContextState::CompletionEnqueued => {
                        BootstrapKernelContextState::AllEnqueued
                    }
                    BootstrapKernelContextState::PreemptedTaken => {
                        BootstrapKernelContextState::PreemptedEnqueued
                    }
                    BootstrapKernelContextState::CompletionSignaled => {
                        BootstrapKernelContextState::CompletionEnqueuedAgain
                    }
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-CONTEXT-TRANSFER",
                            "runnable enqueue must preserve preemptible, completion, terminal, preemptible, completion FIFO order",
                        ));
                    }
                };
            }
            SystemsOperation::DequeueRunnableContext => {
                self.kernel_context = match self.kernel_context {
                    BootstrapKernelContextState::AllEnqueued => {
                        BootstrapKernelContextState::PreemptibleDequeued
                    }
                    BootstrapKernelContextState::PreemptedEnqueued => {
                        BootstrapKernelContextState::CompletionDequeued
                    }
                    BootstrapKernelContextState::CompletionBlocked => {
                        BootstrapKernelContextState::TerminalDequeued
                    }
                    BootstrapKernelContextState::CompletionEnqueuedAgain => {
                        BootstrapKernelContextState::PreemptibleDequeuedAgain
                    }
                    BootstrapKernelContextState::PreemptibleReclaimed => {
                        BootstrapKernelContextState::CompletionDequeuedAgain
                    }
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-CONTEXT-TRANSFER",
                            "runnable dequeue must select preemptible, completion, terminal, preemptible, completion FIFO order",
                        ));
                    }
                };
            }
            SystemsOperation::ReclaimKernelContext => {
                self.kernel_context = match self.kernel_context {
                    BootstrapKernelContextState::TerminalCompleted => {
                        BootstrapKernelContextState::TerminalReclaimed
                    }
                    BootstrapKernelContextState::PreemptibleCompleted => {
                        BootstrapKernelContextState::PreemptibleReclaimed
                    }
                    BootstrapKernelContextState::CompletionCompleted => {
                        BootstrapKernelContextState::CompletionReclaimed
                    }
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-CONTEXT-TRANSFER",
                            "kernel-context reclaim requires the matching completed transfer",
                        ));
                    }
                };
            }
            SystemsOperation::ConsumeEmptyRunnableQueue => {
                if self.kernel_context != BootstrapKernelContextState::CompletionReclaimed
                    || self.kernel_completion != BootstrapKernelCompletionState::Consumed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "runnable queue consumption requires an empty queue, three reclaimed contexts, and consumed completion",
                    ));
                }
                self.kernel_context = BootstrapKernelContextState::QueueConsumed;
            }
            SystemsOperation::ConsumeKernelCompletion => {
                if self.kernel_context != BootstrapKernelContextState::CompletionReclaimed
                    || self.kernel_completion != BootstrapKernelCompletionState::Signaled
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "one-shot completion consumption requires its awakened worker to be reclaimed",
                    ));
                }
                self.kernel_completion = BootstrapKernelCompletionState::Consumed;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn observe_user_execution(
        &mut self,
        operation: &SystemsOperation,
    ) -> Result<bool, SystemsModelError> {
        match operation {
            SystemsOperation::PrepareUserAddressSpace => {
                if self.kernel_context != BootstrapKernelContextState::QueueConsumed
                    || self.kernel_completion != BootstrapKernelCompletionState::Consumed
                    || self.user_execution != BootstrapUserExecutionState::Fresh
                    || self.memory_ownership
                        != BootstrapMemoryOwnership::Frames(INITIAL_MULTI_FRAME_REQUEST)
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user-address-space preparation requires completed kernel-context cleanup, its exact three-frame backing, and no live critical scope",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::None;
                self.user_execution =
                    BootstrapUserExecutionState::AddressSpacePrepared(INITIAL_MULTI_FRAME_REQUEST);
            }
            SystemsOperation::PrepareUserProbe { process_identity } => {
                let BootstrapUserExecutionState::MappingWritableData(backing, 2) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user probe preparation requires both finalized image mappings resident",
                    ));
                };
                if *process_identity != 1
                    || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.initial_boot_resource_validated
                    || !self.anonymous_user_mapping_reserved
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user probe preparation requires PID one, the validated boot resource, the ready resident image, no source-owned extent, and no live critical scope",
                    ));
                }
                self.user_execution =
                    BootstrapUserExecutionState::PreparedWithResidentImage(backing);
            }
            SystemsOperation::ReserveAnonymousUserMapping => {
                let BootstrapUserExecutionState::MappingWritableData(backing, 2) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "anonymous reservation requires both finalized image mappings",
                    ));
                };
                if self.memory_ownership
                    != BootstrapMemoryOwnership::Frames(POST_RANDOM_ANONYMOUS_EXTENT_REQUEST)
                    || self.anonymous_user_mapping_reserved
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "anonymous reservation requires the exact source-owned growth extent and no prior reservation",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::None;
                self.anonymous_user_mapping_reserved = true;
                self.user_execution = BootstrapUserExecutionState::MappingWritableData(backing, 2);
            }
            SystemsOperation::VerifyUserAddressSpaceMapping => {
                let BootstrapUserExecutionState::AddressSpacePrepared(backing) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping verification requires the exact unverified prepared address space",
                    ));
                };
                if self.memory_ownership
                    != BootstrapMemoryOwnership::Frames(INITIAL_SINGLE_FRAME_REQUEST)
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping verification requires one disjoint source-owned frame and no live critical scope",
                    ));
                }
                self.user_execution = BootstrapUserExecutionState::AddressSpaceVerified(backing);
            }
            SystemsOperation::MapUserAddressSpace => {
                let (backing, generation) = match self.user_execution {
                    BootstrapUserExecutionState::AddressSpacePrepared(backing) => (backing, 1),
                    BootstrapUserExecutionState::MappingExecutable(backing, 1) => (backing, 2),
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-USER-CONTEXT",
                            "user mapping publication requires the prepared address space at its next exact generation",
                        ));
                    }
                };
                let mapping_request = if generation == 1 {
                    INITIAL_TWO_FRAME_REQUEST
                } else {
                    INITIAL_SINGLE_FRAME_REQUEST
                };
                if self.memory_ownership != BootstrapMemoryOwnership::Frames(mapping_request)
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping publication requires one disjoint source-owned frame and no live critical scope",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::UserMapping(mapping_request);
                self.user_execution = BootstrapUserExecutionState::MappingLive(backing, generation);
            }
            SystemsOperation::StoreUserMappingByte => {
                let BootstrapUserExecutionState::MappingLive(backing, generation) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping store requires the exact live mapping",
                    ));
                };
                if generation != 2
                    || self.memory_ownership
                        != BootstrapMemoryOwnership::UserMapping(INITIAL_SINGLE_FRAME_REQUEST)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping store requires its owned disjoint frame",
                    ));
                }
                self.user_execution =
                    BootstrapUserExecutionState::MappingStored(backing, generation);
            }
            SystemsOperation::InstallUserMappingEntryTrampoline => {
                let BootstrapUserExecutionState::MappingLive(backing, generation) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "entry-trampoline installation requires the exact live mapping",
                    ));
                };
                if generation != 1
                    || self.memory_ownership
                        != BootstrapMemoryOwnership::UserMapping(INITIAL_TWO_FRAME_REQUEST)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "entry-trampoline installation requires the first owned disjoint mapping",
                    ));
                }
                self.user_execution =
                    BootstrapUserExecutionState::MappingEntryInstalled(backing, generation);
            }
            SystemsOperation::ProtectUserMappingReadOnly => {
                let (backing, generation) = match self.user_execution {
                    BootstrapUserExecutionState::MappingStored(backing, generation)
                    | BootstrapUserExecutionState::MappingEntryInstalled(backing, generation) => {
                        (backing, generation)
                    }
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-USER-CONTEXT",
                            "user mapping protection requires the exact populated writable mapping",
                        ));
                    }
                };
                if self.memory_ownership
                    != BootstrapMemoryOwnership::UserMapping(if generation == 1 {
                        INITIAL_TWO_FRAME_REQUEST
                    } else {
                        INITIAL_SINGLE_FRAME_REQUEST
                    })
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping protection requires its owned disjoint frame",
                    ));
                }
                self.user_execution =
                    BootstrapUserExecutionState::MappingProtected(backing, generation);
            }
            SystemsOperation::LoadUserMappingByte => {
                let BootstrapUserExecutionState::MappingProtected(backing, generation) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping load requires the exact protected mapping",
                    ));
                };
                if generation != 2
                    || self.memory_ownership
                        != BootstrapMemoryOwnership::UserMapping(INITIAL_SINGLE_FRAME_REQUEST)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping load requires its owned disjoint frame",
                    ));
                }
                self.user_execution =
                    BootstrapUserExecutionState::MappingVerified(backing, generation);
            }
            SystemsOperation::VerifyUserMappingEntryTrampoline => {
                let BootstrapUserExecutionState::MappingProtected(backing, generation) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "entry-trampoline verification requires the exact protected mapping",
                    ));
                };
                if generation != 1
                    || self.memory_ownership
                        != BootstrapMemoryOwnership::UserMapping(INITIAL_TWO_FRAME_REQUEST)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "entry-trampoline verification requires the first owned disjoint mapping",
                    ));
                }
                self.user_execution =
                    BootstrapUserExecutionState::MappingVerified(backing, generation);
            }
            SystemsOperation::PermitUserMappingExecute => {
                let BootstrapUserExecutionState::MappingVerified(backing, generation) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping execute promotion requires the exact verified mapping",
                    ));
                };
                if generation != 1
                    || self.memory_ownership
                        != BootstrapMemoryOwnership::UserMapping(INITIAL_TWO_FRAME_REQUEST)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping execute promotion requires its owned disjoint frame",
                    ));
                }
                self.user_execution =
                    BootstrapUserExecutionState::MappingExecutable(backing, generation);
                self.memory_ownership = BootstrapMemoryOwnership::None;
            }
            SystemsOperation::SealUserMappingImmutableData => {
                let BootstrapUserExecutionState::MappingVerified(backing, generation) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping data sealing requires the exact verified mapping",
                    ));
                };
                if generation != 2
                    || self.memory_ownership
                        != BootstrapMemoryOwnership::UserMapping(INITIAL_SINGLE_FRAME_REQUEST)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping data sealing requires the second generation and its owned frame",
                    ));
                }
                self.user_execution =
                    BootstrapUserExecutionState::MappingImmutableData(backing, generation);
                self.memory_ownership = BootstrapMemoryOwnership::None;
            }
            SystemsOperation::PermitUserMappingWritableData => {
                let BootstrapUserExecutionState::MappingVerified(backing, generation) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping writable-data promotion requires the exact verified mapping",
                    ));
                };
                if generation != 2
                    || self.memory_ownership
                        != BootstrapMemoryOwnership::UserMapping(INITIAL_SINGLE_FRAME_REQUEST)
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping writable-data promotion requires the second generation and its owned frame",
                    ));
                }
                self.user_execution =
                    BootstrapUserExecutionState::MappingWritableData(backing, generation);
                self.memory_ownership = BootstrapMemoryOwnership::None;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_IMAGE_READY" => {
                if self.user_execution
                    != BootstrapUserExecutionState::MappingWritableData(
                        INITIAL_MULTI_FRAME_REQUEST,
                        2,
                    )
                    || self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.resident_user_image_ready
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user image readiness requires both finalized role mappings to remain resident exactly once",
                    ));
                }
                self.resident_user_image_ready = true;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_BOOT_RESOURCE_OK" => {
                if self.user_execution
                    != BootstrapUserExecutionState::MappingWritableData(
                        INITIAL_MULTI_FRAME_REQUEST,
                        2,
                    )
                    || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || self.initial_boot_resource_validated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "boot-resource witness requires one validated opaque resource with both finalized role mappings resident",
                    ));
                }
                self.initial_boot_resource_validated = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_MULTIPAGE_EXEC_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.initial_boot_resource_validated
                    || self.multipage_user_executable_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "multipage-executable witness requires exact second-page user return with both finalized mappings resident",
                    ));
                }
                self.multipage_user_executable_completed = true;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_READ_OK" => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.initial_boot_resource_validated
                    || !self.multipage_user_executable_completed
                    || self.constructed_user_read_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed read witness requires exact user return with both finalized mappings resident",
                    ));
                }
                self.constructed_user_read_completed = true;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_CLOSE_OK" => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.constructed_user_read_completed
                    || self.constructed_user_close_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "descriptor-close witness requires the completed constructed read and exact user return with both finalized mappings resident",
                    ));
                }
                self.constructed_user_close_completed = true;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_DUP_OK" => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.constructed_user_close_completed
                    || self.constructed_user_dup_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "descriptor-duplication witness requires the close witness and exact user return with both finalized mappings resident",
                    ));
                }
                self.constructed_user_dup_completed = true;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_FCNTL_OK" => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.constructed_user_dup_completed
                    || self.constructed_user_fcntl_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "descriptor-metadata witness requires the duplication witness and exact user return with both finalized mappings resident",
                    ));
                }
                self.constructed_user_fcntl_completed = true;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_FSTAT_OK" => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.constructed_user_fcntl_completed
                    || self.constructed_user_fstat_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "descriptor-stat witness requires the fcntl witness and exact user return with both finalized mappings resident",
                    ));
                }
                self.constructed_user_fstat_completed = true;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_TCGETS_OK" => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.constructed_user_fstat_completed
                    || self.constructed_user_tcgets_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "terminal-attributes witness requires the fstat witness and exact user return with both finalized mappings resident",
                    ));
                }
                self.constructed_user_tcgets_completed = true;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_WINSIZE_OK" => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.constructed_user_tcgets_completed
                    || self.constructed_user_winsize_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "console-geometry witness requires the TCGETS witness and exact user return with both finalized mappings resident",
                    ));
                }
                self.constructed_user_winsize_completed = true;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_LSEEK_OK" => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.constructed_user_winsize_completed
                    || self.constructed_user_lseek_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "console-seek witness requires the window-size witness and exact user return with both finalized mappings resident",
                    ));
                }
                self.constructed_user_lseek_completed = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_ENTRY_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.constructed_user_read_completed
                    || !self.constructed_user_close_completed
                    || !self.constructed_user_dup_completed
                    || !self.constructed_user_fcntl_completed
                    || !self.constructed_user_fstat_completed
                    || !self.constructed_user_tcgets_completed
                    || !self.constructed_user_winsize_completed
                    || !self.constructed_user_lseek_completed
                    || self.constructed_user_image_entered
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed-image entry witness requires exact user return with both finalized mappings resident",
                    ));
                }
                self.constructed_user_image_entered = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_STACK_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.constructed_user_image_entered
                    || self.constructed_user_initial_stack_validated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed-image stack witness requires exact returned entry with both finalized mappings resident",
                    ));
                }
                self.constructed_user_initial_stack_validated = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_AUXV_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.constructed_user_initial_stack_validated
                    || self.constructed_user_initial_auxv_validated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed-image auxiliary-vector witness requires exact returned initial-stack validation with both finalized mappings resident",
                    ));
                }
                self.constructed_user_initial_auxv_validated = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.constructed_user_initial_auxv_validated
                    || self.constructed_user_initial_entry_auxv_validated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed-image entry auxiliary-vector witness requires exact returned page-size validation with both finalized mappings resident",
                    ));
                }
                self.constructed_user_initial_entry_auxv_validated = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.constructed_user_initial_entry_auxv_validated
                    || self.constructed_user_initial_execfn_auxv_validated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed-image executable-name auxiliary-vector witness requires exact returned entry-fact validation with both finalized mappings resident",
                    ));
                }
                self.constructed_user_initial_execfn_auxv_validated = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.constructed_user_initial_execfn_auxv_validated
                    || self.constructed_user_initial_credential_auxv_validated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed-image credential auxiliary-vector witness requires exact returned executable-name validation with both finalized mappings resident",
                    ));
                }
                self.constructed_user_initial_credential_auxv_validated = true;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_IMAGE_ELF_OK" => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.constructed_user_initial_credential_auxv_validated
                    || self.constructed_user_elf_identity_validated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed-image ELF witness requires exact returned credential validation with both finalized mappings resident",
                    ));
                }
                self.constructed_user_elf_identity_validated = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.constructed_user_elf_identity_validated
                    || self.external_user_elf_loaded
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "external ELF-load witness requires the validated resident ELF identity exactly once",
                    ));
                }
                self.external_user_elf_loaded = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_COPY_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.external_user_elf_loaded
                    || self.external_user_elf_copy_verified
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "external ELF-copy witness requires the loaded resource payload exactly once",
                    ));
                }
                self.external_user_elf_copy_verified = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_RUNTIME_EXTENT_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.external_user_elf_copy_verified
                    || self.runtime_user_code_extent_validated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "runtime executable-extent witness requires the verified external copy exactly once",
                    ));
                }
                self.runtime_user_code_extent_validated = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_RUNTIME_ENTRY_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.runtime_user_code_extent_validated
                    || self.runtime_user_entry_validated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "runtime ELF-entry witness requires the validated executable extent exactly once",
                    ));
                }
                self.runtime_user_entry_validated = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_RUNTIME_DATA_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.runtime_user_entry_validated
                    || self.runtime_user_data_extent_validated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "runtime data/BSS-extent witness requires the validated ELF entry exactly once",
                    ));
                }
                self.runtime_user_data_extent_validated = true;
            }
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_IMAGE_BSS_OK" => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.external_user_elf_copy_verified
                    || !self.runtime_user_code_extent_validated
                    || !self.runtime_user_data_extent_validated
                    || self.external_user_bss_validated
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "external ELF data/BSS witness requires the verified resource copy exactly once",
                    ));
                }
                self.external_user_bss_validated = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_DATA_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.constructed_user_elf_identity_validated
                    || !self.external_user_elf_loaded
                    || !self.external_user_elf_copy_verified
                    || !self.runtime_user_code_extent_validated
                    || !self.runtime_user_data_extent_validated
                    || !self.external_user_bss_validated
                    || self.constructed_user_image_data_observed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed-image data witness requires exact returned entry with both finalized mappings resident",
                    ));
                }
                self.constructed_user_image_data_observed = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_COPY_TO_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.constructed_user_image_data_observed
                    || self.constructed_user_image_copy_to_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed-image copy-to-user witness requires exact returned data observation with both finalized mappings resident",
                    ));
                }
                self.constructed_user_image_copy_to_completed = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_COPY_FROM_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.constructed_user_image_copy_to_completed
                    || self.constructed_user_image_copy_from_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed-image copy-from-user witness requires exact returned output completion with both finalized mappings resident",
                    ));
                }
                self.constructed_user_image_copy_from_completed = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_SYSCALL_OK" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.constructed_user_image_copy_from_completed
                    || self.constructed_user_image_syscall_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed-image syscall witness requires exact returned data observation with both finalized mappings resident",
                    ));
                }
                self.constructed_user_image_syscall_completed = true;
            }
            SystemsOperation::ConsoleWrite { text }
                if text == "TOPAL_KERNEL_USER_IMAGE_RETAINED" =>
            {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.constructed_user_image_entered
                    || !self.constructed_user_initial_stack_validated
                    || !self.constructed_user_initial_auxv_validated
                    || !self.constructed_user_initial_entry_auxv_validated
                    || !self.constructed_user_initial_execfn_auxv_validated
                    || !self.constructed_user_initial_credential_auxv_validated
                    || !self.constructed_user_elf_identity_validated
                    || !self.external_user_elf_loaded
                    || !self.external_user_elf_copy_verified
                    || !self.runtime_user_code_extent_validated
                    || !self.runtime_user_data_extent_validated
                    || !self.external_user_bss_validated
                    || !self.constructed_user_image_data_observed
                    || !self.constructed_user_image_copy_to_completed
                    || !self.constructed_user_image_copy_from_completed
                    || !self.constructed_user_image_syscall_completed
                    || self.resident_user_image_retained
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "retained-image witness requires exact user return with both finalized mappings resident",
                    ));
                }
                self.resident_user_image_retained = true;
            }
            SystemsOperation::UnmapUserAddressSpace => {
                let (backing, generation) = match self.user_execution {
                    BootstrapUserExecutionState::CompletedWithExecutableImage(backing) => {
                        (backing, 1)
                    }
                    BootstrapUserExecutionState::CompletedWithResidentImage(backing) => {
                        (backing, 2)
                    }
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-USER-CONTEXT",
                            "user unmap requires exact returned execution and its generation-specific image role",
                        ));
                    }
                };
                if self.memory_ownership != BootstrapMemoryOwnership::None {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user unmap requires its owned disjoint frame",
                    ));
                }
                if generation == 2 && !self.resident_user_image_retained {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "immutable data cannot be unmapped before the post-return retained-image witness",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::Frames(if generation == 1 {
                    INITIAL_TWO_FRAME_REQUEST
                } else {
                    INITIAL_SINGLE_FRAME_REQUEST
                });
                self.user_execution = match generation {
                    1 => BootstrapUserExecutionState::CompletedImageRevoked(backing),
                    2 => BootstrapUserExecutionState::CompletedWithExecutableImage(backing),
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-USER-CONTEXT",
                            "user unmap requires one of the two admitted mapping generations",
                        ));
                    }
                };
            }
            SystemsOperation::ReleaseAnonymousUserMapping => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.anonymous_user_mapping_reserved
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "anonymous release requires returned user execution and its live reservation",
                    ));
                }
                self.anonymous_user_mapping_reserved = false;
                self.memory_ownership =
                    BootstrapMemoryOwnership::Frames(POST_RANDOM_ANONYMOUS_EXTENT_REQUEST);
            }
            SystemsOperation::EnterUserContext => {
                let BootstrapUserExecutionState::PreparedWithResidentImage(backing) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user entry requires the exact prepared probe with both image mappings resident",
                    ));
                };
                if !self.critical_stack.is_empty()
                    || !self.initial_auxv_random_delivered
                    || !self.user_random_service_bound
                    || !self.user_random_small_requests_enabled
                    || !self.user_random_large_requests_enabled
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user entry requires the exact prepared probe, delivered initial AT_RANDOM object, reusable getrandom service, and restored kernel context",
                    ));
                }
                self.user_random_service_bound = false;
                self.user_random_small_requests_enabled = false;
                self.user_random_large_requests_enabled = false;
                self.user_random_service_completed = true;
                self.initial_user_random_delivered = true;
                self.user_execution =
                    BootstrapUserExecutionState::CompletedWithResidentImage(backing);
            }
            SystemsOperation::CompleteUserExecution => {
                let BootstrapUserExecutionState::CompletedImageRevoked(backing) =
                    self.user_execution
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user completion requires terminal exit_group restoration and complete image teardown",
                    ));
                };
                self.user_execution = BootstrapUserExecutionState::Revoked(backing);
            }
            SystemsOperation::ReclaimUserAddressSpace => {
                let BootstrapUserExecutionState::Revoked(backing) = self.user_execution else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user-address-space reclaim requires the exact revoked address space",
                    ));
                };
                self.user_execution = BootstrapUserExecutionState::Consumed;
                self.memory_ownership = BootstrapMemoryOwnership::Frames(backing);
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn observe_translation(
        &mut self,
        operation: &SystemsOperation,
    ) -> Result<(), SystemsModelError> {
        match operation {
            SystemsOperation::BeginTranslationUpdate { request, .. } => {
                if !self.allocator_created
                    || self.memory_ownership != BootstrapMemoryOwnership::None
                    || self.translation != BootstrapTranslationState::BootstrapActive
                    || *request != TranslationUpdateRequest::initial_bootstrap_equivalent()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-TRANSLATION",
                        "translation begin requires the live allocator, no borrowed extent, the bootstrap translation, and the sealed request",
                    ));
                }
                self.translation = BootstrapTranslationState::Update;
            }
            SystemsOperation::CommitTranslationUpdate { .. } => {
                if self.translation != BootstrapTranslationState::Update {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-TRANSLATION",
                        "translation commit requires one live exclusive update",
                    ));
                }
                self.translation = BootstrapTranslationState::InactiveSpace;
            }
            SystemsOperation::ActivateTranslationSpace { .. } => {
                if self.translation != BootstrapTranslationState::InactiveSpace {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-TRANSLATION",
                        "translation activation requires one committed inactive space",
                    ));
                }
                self.translation = BootstrapTranslationState::ReplacementActive;
            }
            operation @ (SystemsOperation::BeginTranslationEdit { .. }
            | SystemsOperation::MapTranslationFrames { .. }
            | SystemsOperation::UnmapTranslationMapping
            | SystemsOperation::CommitTranslationEdit { .. }) => {
                self.observe_translation_edit(operation)?;
            }
            _ => unreachable!("caller selects translation operations"),
        }
        Ok(())
    }

    fn observe_translation_edit(
        &mut self,
        operation: &SystemsOperation,
    ) -> Result<(), SystemsModelError> {
        match operation {
            SystemsOperation::BeginTranslationEdit { kind, .. } => match kind {
                TranslationEditKind::Map
                    if self.translation == BootstrapTranslationState::ReplacementActive
                        && self.memory_ownership
                            == BootstrapMemoryOwnership::Frames(INITIAL_SINGLE_FRAME_REQUEST) =>
                {
                    self.translation = BootstrapTranslationState::MapEdit;
                }
                TranslationEditKind::Unmap
                    if self.translation == BootstrapTranslationState::MappedActive
                        && matches!(
                            self.memory_ownership,
                            BootstrapMemoryOwnership::Mapping(_)
                        ) =>
                {
                    self.translation = BootstrapTranslationState::UnmapEdit;
                }
                _ => {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-TRANSLATION-EDIT",
                        "translation edit begin requires the matching refined context and owned resource",
                    ));
                }
            },
            SystemsOperation::MapTranslationFrames { request, .. } => {
                let BootstrapMemoryOwnership::Frames(frame_request) = self.memory_ownership else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-TRANSLATION-EDIT",
                        "translation edit map requires the sealed policy, map edit, and live extent",
                    ));
                };
                if self.translation != BootstrapTranslationState::MapEdit
                    || frame_request != INITIAL_SINGLE_FRAME_REQUEST
                    || *request != TranslationMappingRequest::initial_read_write()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-TRANSLATION-EDIT",
                        "translation edit map requires the sealed policy, map edit, and live extent",
                    ));
                }
                self.translation = BootstrapTranslationState::MapStaged;
                self.memory_ownership = BootstrapMemoryOwnership::ProvisionalMapping(frame_request);
            }
            SystemsOperation::UnmapTranslationMapping => {
                let BootstrapMemoryOwnership::Mapping(frame_request) = self.memory_ownership else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-TRANSLATION-EDIT",
                        "translation edit unmap requires the matching edit and live mapping",
                    ));
                };
                if self.translation != BootstrapTranslationState::UnmapEdit {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-TRANSLATION-EDIT",
                        "translation edit unmap requires the matching edit and live mapping",
                    ));
                }
                self.translation = BootstrapTranslationState::UnmapStaged;
                self.memory_ownership = BootstrapMemoryOwnership::ProvisionalFrames(frame_request);
            }
            SystemsOperation::CommitTranslationEdit { kind, .. } => match kind {
                TranslationEditKind::Map
                    if self.translation == BootstrapTranslationState::MapStaged =>
                {
                    let BootstrapMemoryOwnership::ProvisionalMapping(frame_request) =
                        self.memory_ownership
                    else {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-TRANSLATION-EDIT",
                            "translation edit commit requires one complete matching candidate",
                        ));
                    };
                    self.translation = BootstrapTranslationState::MappedActive;
                    self.memory_ownership = BootstrapMemoryOwnership::Mapping(frame_request);
                }
                TranslationEditKind::Unmap
                    if self.translation == BootstrapTranslationState::UnmapStaged =>
                {
                    let BootstrapMemoryOwnership::ProvisionalFrames(frame_request) =
                        self.memory_ownership
                    else {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-TRANSLATION-EDIT",
                            "translation edit commit requires one complete matching candidate",
                        ));
                    };
                    self.translation = BootstrapTranslationState::RemovedActive;
                    self.memory_ownership = BootstrapMemoryOwnership::Frames(frame_request);
                }
                _ => {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-TRANSLATION-EDIT",
                        "translation edit commit requires one complete matching candidate",
                    ));
                }
            },
            _ => unreachable!("caller selects translation-edit operations"),
        }
        Ok(())
    }

    fn complete(self) -> Result<(), SystemsModelError> {
        if !self.critical_stack.is_empty() {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-CRITICAL-LIVE",
                "bootstrap handler consumes its context while critical restoration authority remains live",
            ));
        }
        if !self.memory_described {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-BOOT-MEMORY",
                "the bootstrap handler must consume its entered context through boot-memory description",
            ));
        }
        if !self.allocator_created {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-FRAMES",
                "the bootstrap handler must consume memory description through frame-allocator creation",
            ));
        }
        match self.memory_ownership {
            BootstrapMemoryOwnership::None
                if !self.retained_static_process
                    && !self.entropy_pending_static_process
                    && !self.entropy_acquiring_static_process => {}
            BootstrapMemoryOwnership::None => {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-FRAMES-LIVE",
                    "bootstrap handler consumes its context while a retained static process remains live",
                ));
            }
            BootstrapMemoryOwnership::Frames(_)
            | BootstrapMemoryOwnership::StaticElfMappingResources
            | BootstrapMemoryOwnership::StaticElfConstructionLive
            | BootstrapMemoryOwnership::StaticElfConstructionZeroed
            | BootstrapMemoryOwnership::StaticElfConstructionPopulated
            | BootstrapMemoryOwnership::StaticElfConstructionSealed
            | BootstrapMemoryOwnership::StaticElfConstructionSealedWithHierarchy
            | BootstrapMemoryOwnership::StaticElfConstructionAttached
            | BootstrapMemoryOwnership::StaticElfConstructionActive
            | BootstrapMemoryOwnership::StaticElfConstructionRestored
            | BootstrapMemoryOwnership::StaticElfConstructionEntryAuthorized
            | BootstrapMemoryOwnership::StaticElfConstructionStackSlotAuthorized
            | BootstrapMemoryOwnership::StaticElfConstructionStackSlotAuthorizedWithBacking
            | BootstrapMemoryOwnership::StaticElfConstructionStackBackingZeroed
            | BootstrapMemoryOwnership::StaticElfConstructionStackLeafPublished
            | BootstrapMemoryOwnership::StaticElfConstructionStackEntryInitialized
            | BootstrapMemoryOwnership::StaticElfConstructionStackPageSizeAdded
            | BootstrapMemoryOwnership::StaticElfConstructionStackEntryFactAdded
            | BootstrapMemoryOwnership::StaticElfConstructionStackProgramHeadersAdded
            | BootstrapMemoryOwnership::StaticElfConstructionStackExecutableNameAdded
            | BootstrapMemoryOwnership::StaticElfConstructionStackCredentialsAdded
            | BootstrapMemoryOwnership::StaticElfConstructionStackSecureModeAdded
            | BootstrapMemoryOwnership::StaticElfConstructionStackClockTickRateAdded
            | BootstrapMemoryOwnership::StaticElfConstructionStackPlatformAdded
            | BootstrapMemoryOwnership::StaticElfConstructionStackHardwareCapabilitiesAdded
            | BootstrapMemoryOwnership::StaticElfConstructionStackLoadMetadataAdded
            | BootstrapMemoryOwnership::StaticElfConstructionStackRandomSlotAuthorized
            | BootstrapMemoryOwnership::StaticElfConstructionEntryRestored
            | BootstrapMemoryOwnership::StaticElfConstructionDetached
            | BootstrapMemoryOwnership::StaticElfConstructionHierarchyReturned
            | BootstrapMemoryOwnership::StaticElfConstructionMetadataReturned
            | BootstrapMemoryOwnership::StaticElfConstructionRevoked => {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-FRAMES-LIVE",
                    "bootstrap handler consumes its context while a physical-frame extent remains live",
                ));
            }
            BootstrapMemoryOwnership::Mapping(_) => {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-MAPPING-LIVE",
                    "bootstrap handler consumes its context while a kernel mapping remains live",
                ));
            }
            BootstrapMemoryOwnership::UserMapping(_) => {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "bootstrap handler consumes its context while a user mapping remains live",
                ));
            }
            BootstrapMemoryOwnership::ProvisionalMapping(_)
            | BootstrapMemoryOwnership::ProvisionalFrames(_) => {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-TRANSLATION-EDIT-LIVE",
                    "bootstrap handler consumes its context while a translation edit candidate remains live",
                ));
            }
        }
        if matches!(
            self.translation,
            BootstrapTranslationState::Update
                | BootstrapTranslationState::InactiveSpace
                | BootstrapTranslationState::MapEdit
                | BootstrapTranslationState::MapStaged
                | BootstrapTranslationState::UnmapEdit
                | BootstrapTranslationState::UnmapStaged
        ) {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-TRANSLATION-LIVE",
                "bootstrap handler consumes its context while a translation update, inactive space, or edit remains live",
            ));
        }
        if self.local_notification != BootstrapLocalNotificationState::Completed {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-LOCAL-INTERRUPT-LIVE",
                "bootstrap completion requires one completed local-notification send/wait lifecycle",
            ));
        }
        if self.monotonic_clock_observations != 2 {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-MONOTONIC-CLOCK-LIVE",
                "bootstrap completion requires exactly two monotonic-clock observations",
            ));
        }
        if self.pci_configuration_observations
            != PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY.len()
        {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-PCI-CONFIG-LIVE",
                "bootstrap completion requires the complete sealed virtio-rng PCI capability-discovery sequence",
            ));
        }
        if !self.pci_bus_master_authorized {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-PCI-CONFIG-UPDATE-LIVE",
                "bootstrap completion requires the sealed PCI bus-master authorization",
            ));
        }
        if self.dma_buffers_created != DmaBufferRequest::INITIAL_Q35_VIRTIO_RNG.len() {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-DMA-BUFFER-LIVE",
                "bootstrap completion requires both sealed CPU-owned virtio-rng DMA buffers",
            ));
        }
        if self.dma_construction_operations
            != 1 + DmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING.len()
        {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-DMA-CONSTRUCT-LIVE",
                "bootstrap completion requires the exact CPU-owned virtio-rng split-ring construction",
            ));
        }
        if self.dma_buffers_prepared != DmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG.len() {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-DMA-PREPARE-LIVE",
                "bootstrap completion requires the exact payload-then-control DMA device-preparation sequence",
            ));
        }
        if self.dma_references_published
            != PciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES.len()
        {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-DMA-PUBLISH-LIVE",
                "bootstrap completion requires all three sealed virtio-rng queue-address publications",
            ));
        }
        if self.pci_region_register_accesses
            != PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION.len()
                + PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY.len()
                + PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION.len()
                + PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION.len()
                + PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL.len()
        {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-PCI-REGION-LIVE",
                "bootstrap completion requires the complete sealed virtio-rng feature-negotiation, queue-discovery, queue-activation, queue-notification, and completion-signal sequences",
            ));
        }
        if !self.dma_control_completed
            || self.dma_used_ring_reads
                != DmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING.len()
        {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-DMA-USED-RING-LIVE",
                "bootstrap completion requires CPU return and exact validation of the sealed virtio-rng used ring",
            ));
        }
        if !self.dma_payload_completed {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-DMA-PAYLOAD-LIVE",
                "bootstrap completion requires CPU return of the completed virtio-rng payload",
            ));
        }
        if !self.entropy_sample_captured {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ENTROPY-SAMPLE-LIVE",
                "bootstrap completion requires capture of the sealed uncredited virtio-rng sample",
            ));
        }
        if !self.entropy_sample_assessed {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ENTROPY-QUALITY-LIVE",
                "bootstrap completion requires assessment of the captured virtio-rng sample under explicit Q35 trust evidence",
            ));
        }
        if !self.entropy_pool_created {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ENTROPY-POOL-LIVE",
                "bootstrap completion requires creation of the sealed empty entropy pool",
            ));
        }
        if !self.entropy_pool_mixed {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ENTROPY-MIX-LIVE",
                "bootstrap completion requires the sealed first entropy-pool mix",
            ));
        }
        if !self.entropy_pool_credited {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ENTROPY-CREDIT-LIVE",
                "bootstrap completion requires transfer of the sealed entropy contribution",
            ));
        }
        if !self.entropy_generator_ready {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ENTROPY-GENERATOR-LIVE",
                "bootstrap completion requires the sealed ready entropy generator",
            ));
        }
        if !self.entropy_generator_extracted {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ENTROPY-EXTRACT-LIVE",
                "bootstrap completion requires the sealed first entropy-generator extraction",
            ));
        }
        if self.deadline != BootstrapDeadlineState::Completed {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-DEADLINE-EVENT-LIVE",
                "bootstrap completion requires one completed deadline arm/wait lifecycle",
            ));
        }
        if self.kernel_context != BootstrapKernelContextState::QueueConsumed {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-CONTEXT-TRANSFER-LIVE",
                "bootstrap completion requires all worker contexts to be reclaimed and the runnable queue consumed empty",
            ));
        }
        if self.kernel_completion != BootstrapKernelCompletionState::Consumed {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-CONTEXT-TRANSFER-LIVE",
                "bootstrap completion requires the one-shot completion state to be consumed",
            ));
        }
        if self.user_execution != BootstrapUserExecutionState::Consumed {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-USER-CONTEXT-LIVE",
                "bootstrap completion requires one consumed getpid/exit_group user execution",
            ));
        }
        if !self.initial_auxv_random_delivered {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ENTROPY-DELIVER-LIVE",
                "bootstrap completion requires affine delivery of the first entropy output to initial AT_RANDOM",
            ));
        }
        if !self.initial_getrandom_extracted
            || self.initial_user_random_reserved
            || self.user_random_service_bound
            || self.user_random_small_requests_enabled
            || self.user_random_large_requests_enabled
            || !self.user_random_service_completed
            || !self.initial_user_random_delivered
        {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-ENTROPY-RESERVE-LIVE",
                "bootstrap completion requires affine reservation, reusable-service binding, repeated delivery, and teardown of the initial getrandom service",
            ));
        }
        Ok(())
    }
}

fn validate_atomic_operation(
    operation: &SystemsOperation,
    storage: &mut BootstrapStorageState,
    region: &mut Option<BootstrapRegion>,
    atomic: &mut Option<AtomicWordLocation>,
) -> Result<bool, SystemsModelError> {
    match operation {
        SystemsOperation::AtomicWordCreate { request } => {
            if atomic.is_some() || *request != AtomicWordRequest::initial() {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-ATOMIC",
                    "the initial slice admits one live atomic word with the sealed request",
                ));
            }
            let owned_region = region.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-ATOMIC-LIFETIME",
                    "atomic word creation requires one exclusively owned ordinary region",
                )
            })?;
            *atomic = Some(AtomicWordLocation::create(storage, owned_region, *request)?);
        }
        SystemsOperation::AtomicCompareExchangeEquals {
            expected,
            desired,
            success_order,
            failure_order,
            ..
        } => {
            if (*expected, *desired, *success_order, *failure_order)
                != (41, 42, AtomicOrder::AcquireRelease, AtomicOrder::Acquire)
            {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-ATOMIC",
                    "the initial compare/exchange requires expected 41, desired 42, acquire-release success, and acquire failure",
                ));
            }
            atomic
                .as_mut()
                .ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-ATOMIC-LIFETIME",
                        "compare/exchange requires one live atomic word",
                    )
                })?
                .compare_exchange(storage, *expected, *desired, *success_order, *failure_order)?;
        }
        SystemsOperation::AtomicLoadEquals { order, .. } => {
            if *order != AtomicOrder::Acquire {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-ATOMIC-ORDER",
                    "the initial atomic load requires acquire order",
                ));
            }
            atomic
                .as_ref()
                .ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-ATOMIC-LIFETIME",
                        "atomic load requires one live atomic word",
                    )
                })?
                .load(storage, *order)?;
        }
        SystemsOperation::AtomicWordEnd => {
            let location = atomic.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-ATOMIC-LIFETIME",
                    "atomic end requires one live atomic word",
                )
            })?;
            if region.replace(location.end()).is_some() {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-ATOMIC-LIFETIME",
                    "atomic end cannot overwrite ordinary region ownership",
                ));
            }
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn validate_bootstrap_storage_operations(
    program: &SystemsProgram,
) -> Result<(), SystemsModelError> {
    let mut storage = BootstrapStorageState::new(
        program.bootstrap_storage.clone(),
        "systems-program-validation",
    )?;
    let mut region: Option<BootstrapRegion> = None;
    let mut context_stacks: Vec<BootstrapRegion> = Vec::new();
    let mut atomic: Option<AtomicWordLocation> = None;
    let mut authority = BootstrapAuthorityState::default();
    for (index, operation) in program.bootstrap.handler.operations.iter().enumerate() {
        let authority_operation = authority.observe(index, operation)?;
        if validate_kernel_context_storage_operation(
            operation,
            &mut storage,
            &mut region,
            &mut context_stacks,
            &authority.kernel_context,
        )? {
            continue;
        }
        if authority_operation {
            continue;
        }
        if validate_atomic_operation(operation, &mut storage, &mut region, &mut atomic)? {
            continue;
        }
        validate_bootstrap_storage_operation(operation, &mut storage, &mut region)?;
    }
    authority.complete()?;
    validate_bootstrap_owned_completion(region.as_ref(), atomic.as_ref(), &context_stacks)
}

fn validate_kernel_context_storage_operation(
    operation: &SystemsOperation,
    storage: &mut BootstrapStorageState,
    region: &mut Option<BootstrapRegion>,
    context_stacks: &mut Vec<BootstrapRegion>,
    state: &BootstrapKernelContextState,
) -> Result<bool, SystemsModelError> {
    match operation {
        SystemsOperation::CreateKernelCompletion => {}
        SystemsOperation::CreateKernelContext => {
            let stack = region.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "kernel-context creation requires one live exclusive stack region",
                )
            })?;
            if stack.byte_count() != 16_384
                || stack.alignment_bytes() != 16
                || stack.placement() != crate::BootstrapStoragePlacement::BootstrapReclaimable
            {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "the initial kernel context requires one 16 KiB, 16-byte-aligned bootstrap-reclaimable stack",
                ));
            }
            context_stacks.push(stack);
        }
        SystemsOperation::TransferKernelContext
        | SystemsOperation::TransferKernelContextUntilDeadline
        | SystemsOperation::TransferKernelContextUntilCompletion => {
            if context_stacks.is_empty() {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "kernel-context transfer requires one live suspended context stack",
                ));
            }
        }
        SystemsOperation::CreateRunnableQueue { .. }
        | SystemsOperation::EnqueueRunnableContext
        | SystemsOperation::DequeueRunnableContext
        | SystemsOperation::TakePreemptedKernelContext
        | SystemsOperation::SignalBlockedKernelCompletion
        | SystemsOperation::ConsumeKernelCompletion
        | SystemsOperation::ConsumeEmptyRunnableQueue => {}
        SystemsOperation::ReclaimKernelContext => {
            let index = match state {
                BootstrapKernelContextState::TerminalReclaimed => 2,
                BootstrapKernelContextState::PreemptibleReclaimed
                | BootstrapKernelContextState::CompletionReclaimed => 0,
                _ => {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "kernel-context reclaim requires one completed context stack",
                    ));
                }
            };
            if context_stacks.len() <= index {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "kernel-context reclaim requires its owned context stack",
                ));
            }
            storage.release(context_stacks.remove(index))?;
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn validate_bootstrap_storage_operation(
    operation: &SystemsOperation,
    storage: &mut BootstrapStorageState,
    region: &mut Option<BootstrapRegion>,
) -> Result<(), SystemsModelError> {
    match operation {
        SystemsOperation::BootstrapAllocate { request } => {
            if region.is_some() {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-STORAGE-LIFETIME",
                    "the initial executable slice admits one live bootstrap region",
                ));
            }
            *region = Some(storage.allocate(*request).map_err(|code| {
                SystemsModelError::new(
                    "E-SYSTEMS-STORAGE-REQUEST",
                    format!("static bootstrap allocation cannot succeed: {code:?}"),
                )
            })?);
        }
        SystemsOperation::BootstrapStoreByte {
            offset_bytes,
            value,
        } => storage.store_byte(
            region.as_ref().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-STORAGE-LIFETIME",
                    "bootstrap byte store requires one live region",
                )
            })?,
            *offset_bytes,
            *value,
        )?,
        SystemsOperation::BootstrapLoadByteEquals { offset_bytes, .. } => {
            let _ = storage.load_byte(
                region.as_ref().ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-STORAGE-LIFETIME",
                        "bootstrap byte load requires one live region",
                    )
                })?,
                *offset_bytes,
            )?;
        }
        SystemsOperation::BootstrapRelease => {
            let released = region.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-STORAGE-LIFETIME",
                    "bootstrap release requires one live region",
                )
            })?;
            storage.release(released)?;
        }
        SystemsOperation::ConsoleWrite { .. } | SystemsOperation::DebugBreak => {}
        SystemsOperation::PrepareUserAddressSpace
        | SystemsOperation::VerifyUserAddressSpaceMapping
        | SystemsOperation::MapUserAddressSpace
        | SystemsOperation::ReserveAnonymousUserMapping
        | SystemsOperation::StoreUserMappingByte
        | SystemsOperation::InstallUserMappingEntryTrampoline
        | SystemsOperation::ProtectUserMappingReadOnly
        | SystemsOperation::PermitUserMappingExecute
        | SystemsOperation::SealUserMappingImmutableData
        | SystemsOperation::PermitUserMappingWritableData
        | SystemsOperation::LoadUserMappingByte
        | SystemsOperation::VerifyUserMappingEntryTrampoline
        | SystemsOperation::UnmapUserAddressSpace
        | SystemsOperation::ReleaseAnonymousUserMapping
        | SystemsOperation::PrepareUserProbe { .. }
        | SystemsOperation::EnterUserContext
        | SystemsOperation::CompleteUserExecution
        | SystemsOperation::ReclaimUserAddressSpace => {}
        _ => unreachable!("authority or atomic operations continue above"),
    }
    Ok(())
}

fn validate_bootstrap_owned_completion(
    region: Option<&BootstrapRegion>,
    atomic: Option<&AtomicWordLocation>,
    context_stacks: &[BootstrapRegion],
) -> Result<(), SystemsModelError> {
    if region.is_some() {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-STORAGE-LIVE",
            "bootstrap handler consumes its context while a region remains live",
        ));
    }
    if atomic.is_some() {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-ATOMIC-LIVE",
            "bootstrap handler consumes its context while an atomic location remains live",
        ));
    }
    if !context_stacks.is_empty() {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-CONTEXT-TRANSFER-LIVE",
            "bootstrap handler consumes its context while a suspended or completed kernel context remains live",
        ));
    }
    Ok(())
}

/// Execute the deterministic abstract transition model for one checked root.
///
/// # Errors
///
/// Returns a stable systems-model diagnostic when the program is outside the
/// sealed initial profile.
pub fn model_systems_transitions(
    program: &SystemsProgram,
) -> Result<Vec<SystemsTransition>, SystemsModelError> {
    validate_systems_program(program)?;
    let mut transitions = initial_systems_transitions(program);
    let mut storage = BootstrapStorageState::new(
        program.bootstrap_storage.clone(),
        "systems-transition-model",
    )?;
    let mut region: Option<BootstrapRegion> = None;
    let mut atomic: Option<AtomicWordLocation> = None;
    let mut context_stacks: Vec<BootstrapRegion> = Vec::new();
    let mut critical_stack = Vec::new();
    let mut next_critical_identity = 1_u64;
    let mut local_notification =
        LocalNotificationProtocol::new("initial-local-notification-source");
    let mut pending_local_notification = None;
    let mut next_clock_observation_identity = 1_u64;
    let mut next_pci_transaction_identity = 1_u64;
    let mut next_pci_region_transaction_identity = 1_u64;
    let mut next_pci_update_transaction_identity = 1_u64;
    let mut deadline_state = BootstrapDeadlineState::Fresh;
    let mut next_deadline_event_identity = 1_u64;
    let mut kernel_context_state = BootstrapKernelContextState::Fresh;
    let mut user_protocol = SystemsUserExecutionProtocol::initial_linux_x86_64();
    let mut prepared_user_space: Option<PreparedUserAddressSpace> = None;
    let mut user_mapping: Option<UserMapping> = None;
    let mut resident_code_mapping: Option<UserMapping> = None;
    let mut anonymous_reservation: Option<ReservedAnonymousUserMapping> = None;
    let mut suspended_user: Option<SuspendedUserContext> = None;
    let mut completed_user: Option<CompletedUserExecution> = None;
    let mut revoked_user_space: Option<RevokedUserAddressSpace> = None;
    for operation in &program.bootstrap.handler.operations {
        if model_critical_transition(
            operation,
            &mut critical_stack,
            &mut next_critical_identity,
            &mut transitions,
        )? {
            continue;
        }
        if model_local_notification_transition(
            operation,
            &program.local_notification.handler,
            &mut local_notification,
            &mut pending_local_notification,
            &mut transitions,
        )? {
            continue;
        }
        if model_monotonic_clock_transition(
            operation,
            &mut next_clock_observation_identity,
            &mut transitions,
        )? {
            continue;
        }
        if model_deadline_transition(
            operation,
            &program.deadline_notification.handler,
            &mut deadline_state,
            &mut next_deadline_event_identity,
            &mut transitions,
        )? {
            continue;
        }
        if model_kernel_context_transition(
            operation,
            &program.kernel_thread.handler,
            &program.completion_thread.handler,
            &program.terminal_thread.handler,
            &program.deadline_notification.handler,
            &mut storage,
            &mut region,
            &mut context_stacks,
            &mut kernel_context_state,
            &mut deadline_state,
            &mut next_deadline_event_identity,
            &mut transitions,
        )? {
            continue;
        }
        if model_user_execution_transition(
            operation,
            &program.user_syscall.handler,
            &mut user_protocol,
            &mut prepared_user_space,
            &mut user_mapping,
            &mut resident_code_mapping,
            &mut anonymous_reservation,
            &mut suspended_user,
            &mut completed_user,
            &mut revoked_user_space,
            &mut transitions,
        )? {
            continue;
        }
        if model_bootstrap_operation(
            operation,
            &program.debug_break.handler,
            &mut storage,
            &mut region,
            &mut atomic,
            &mut next_pci_transaction_identity,
            &mut next_pci_region_transaction_identity,
            &mut next_pci_update_transaction_identity,
            &mut transitions,
        )? {
            return Ok(transitions);
        }
    }
    model_bootstrap_disposition(&program.bootstrap.handler.disposition, &mut transitions);
    Ok(transitions)
}

fn model_user_execution_transition(
    operation: &SystemsOperation,
    handler: &SystemsHandler,
    protocol: &mut SystemsUserExecutionProtocol,
    prepared: &mut Option<PreparedUserAddressSpace>,
    mapping: &mut Option<UserMapping>,
    resident_code: &mut Option<UserMapping>,
    anonymous_reservation: &mut Option<ReservedAnonymousUserMapping>,
    suspended: &mut Option<SuspendedUserContext>,
    completed: &mut Option<CompletedUserExecution>,
    revoked: &mut Option<RevokedUserAddressSpace>,
    transitions: &mut Vec<SystemsTransition>,
) -> Result<bool, SystemsModelError> {
    match operation {
        SystemsOperation::PrepareUserAddressSpace => {
            if prepared.is_some() || suspended.is_some() || completed.is_some() || revoked.is_some()
            {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "user model admits exactly one live address-space lifecycle",
                ));
            }
            *prepared = Some(protocol.prepare_address_space(INITIAL_MULTI_FRAME_REQUEST)?);
            transitions.push(SystemsTransition::PrepareUserAddressSpace {
                address_space_identity: 1,
            });
        }
        SystemsOperation::VerifyUserAddressSpaceMapping => {
            if prepared.is_none() || suspended.is_some() || completed.is_some() || revoked.is_some()
            {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "user mapping model requires one prepared address space",
                ));
            }
            transitions.push(SystemsTransition::VerifyUserAddressSpaceMapping {
                address_space_identity: 1,
            });
        }
        SystemsOperation::MapUserAddressSpace => {
            if prepared.is_none()
                || mapping.is_some()
                || (resident_code.is_some()
                    && resident_code.as_ref().map(UserMapping::identity) != Some(1))
                || suspended.is_some()
                || completed.is_some()
                || revoked.is_some()
            {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "user mapping model requires one prepared address space",
                ));
            }
            let mapping_request = if resident_code.is_none() {
                INITIAL_TWO_FRAME_REQUEST
            } else {
                INITIAL_SINGLE_FRAME_REQUEST
            };
            let fresh_mapping = protocol.map_user_frame(
                prepared.as_ref().expect("checked prepared user space"),
                mapping_request,
            )?;
            let mapping_identity = fresh_mapping.identity();
            *mapping = Some(fresh_mapping);
            transitions.push(SystemsTransition::MapUserAddressSpace {
                address_space_identity: 1,
                mapping_identity,
            });
        }
        SystemsOperation::ReserveAnonymousUserMapping => {
            if prepared.is_none()
                || mapping.is_none()
                || resident_code.is_none()
                || anonymous_reservation.is_some()
                || suspended.is_some()
                || completed.is_some()
                || revoked.is_some()
            {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "anonymous reservation requires both resident image mappings and one prepared address space",
                ));
            }
            let reservation = protocol.reserve_anonymous_user_frame(
                prepared.as_ref().expect("checked prepared user space"),
                INITIAL_SINGLE_FRAME_REQUEST,
            )?;
            let reservation_identity = reservation.identity();
            *anonymous_reservation = Some(reservation);
            transitions.push(SystemsTransition::ReserveAnonymousUserMapping {
                address_space_identity: 1,
                reservation_identity,
            });
        }
        SystemsOperation::StoreUserMappingByte => {
            *mapping = Some(protocol.store_user_mapping_byte(mapping.take().ok_or_else(
                || {
                    SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping store requires one live affine mapping",
                    )
                },
            )?)?);
            let mapping_identity = mapping
                .as_ref()
                .expect("stored mapping was just restored")
                .identity();
            transitions.push(SystemsTransition::StoreUserMappingByte { mapping_identity });
        }
        SystemsOperation::InstallUserMappingEntryTrampoline => {
            *mapping = Some(protocol.install_user_mapping_entry_trampoline(
                mapping.take().ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "entry-trampoline installation requires one live affine mapping",
                    )
                })?,
            )?);
            let mapping_identity = mapping
                .as_ref()
                .expect("entry-installed mapping was just restored")
                .identity();
            transitions
                .push(SystemsTransition::InstallUserMappingEntryTrampoline { mapping_identity });
        }
        SystemsOperation::ProtectUserMappingReadOnly => {
            *mapping = Some(protocol.protect_user_mapping_read_only(
                mapping.take().ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping protection requires one stored affine mapping",
                    )
                })?,
            )?);
            let mapping_identity = mapping
                .as_ref()
                .expect("protected mapping was just restored")
                .identity();
            transitions.push(SystemsTransition::ProtectUserMappingReadOnly { mapping_identity });
        }
        SystemsOperation::LoadUserMappingByte => {
            *mapping = Some(protocol.load_user_mapping_byte(mapping.take().ok_or_else(
                || {
                    SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping load requires one protected affine mapping",
                    )
                },
            )?)?);
            let mapping_identity = mapping
                .as_ref()
                .expect("verified mapping was just restored")
                .identity();
            transitions.push(SystemsTransition::LoadUserMappingByte { mapping_identity });
        }
        SystemsOperation::VerifyUserMappingEntryTrampoline => {
            *mapping = Some(protocol.verify_user_mapping_entry_trampoline(
                mapping.take().ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "entry-trampoline verification requires one protected affine mapping",
                    )
                })?,
            )?);
            let mapping_identity = mapping
                .as_ref()
                .expect("entry-verified mapping was just restored")
                .identity();
            transitions
                .push(SystemsTransition::VerifyUserMappingEntryTrampoline { mapping_identity });
        }
        SystemsOperation::PermitUserMappingExecute => {
            if resident_code.is_some() {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "user model admits exactly one resident executable-code mapping",
                ));
            }
            *resident_code = Some(protocol.permit_user_mapping_execute(
                mapping.take().ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping execute promotion requires one verified affine mapping",
                    )
                })?,
            )?);
            let mapping_identity = resident_code
                .as_ref()
                .expect("executable mapping was just retained")
                .identity();
            transitions.push(SystemsTransition::PermitUserMappingExecute { mapping_identity });
        }
        SystemsOperation::SealUserMappingImmutableData => {
            *mapping = Some(protocol.seal_user_mapping_immutable_data(
                mapping.take().ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping data sealing requires one verified affine mapping",
                    )
                })?,
            )?);
            let mapping_identity = mapping
                .as_ref()
                .expect("immutable-data mapping was just restored")
                .identity();
            transitions.push(SystemsTransition::SealUserMappingImmutableData { mapping_identity });
        }
        SystemsOperation::PermitUserMappingWritableData => {
            *mapping = Some(protocol.permit_user_mapping_writable_data(
                mapping.take().ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping writable-data promotion requires one verified affine mapping",
                    )
                })?,
            )?);
            let mapping_identity = mapping
                .as_ref()
                .expect("writable-data mapping was just restored")
                .identity();
            transitions.push(SystemsTransition::PermitUserMappingWritableData { mapping_identity });
        }
        SystemsOperation::UnmapUserAddressSpace => {
            let mapping = if mapping.is_some() {
                mapping.take().expect("mapping presence checked")
            } else {
                resident_code.take().ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user unmap requires one role-specific affine mapping",
                    )
                })?
            };
            let mapping_identity = mapping.identity();
            let returned = if let Some(address_space) = prepared.as_ref() {
                protocol.unmap_user_frame(address_space, mapping)?
            } else {
                protocol.unmap_completed_user_frame(
                    completed.as_ref().ok_or_else(|| {
                        SystemsModelError::new(
                            "E-SYSTEMS-USER-CONTEXT",
                            "user unmap requires one prepared or exactly returned address space",
                        )
                    })?,
                    mapping,
                )?
            };
            let expected_return = if mapping_identity == 1 {
                INITIAL_TWO_FRAME_REQUEST
            } else {
                INITIAL_SINGLE_FRAME_REQUEST
            };
            if returned != expected_return {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "user unmap returned the wrong physical-frame extent",
                ));
            }
            transitions.push(SystemsTransition::UnmapUserAddressSpace {
                address_space_identity: 1,
                mapping_identity,
            });
        }
        SystemsOperation::ReleaseAnonymousUserMapping => {
            let reservation = anonymous_reservation.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "anonymous release requires one live affine reservation",
                )
            })?;
            let reservation_identity = reservation.identity();
            let returned = protocol.release_anonymous_user_frame(
                completed.as_ref().ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "anonymous release requires completed user execution",
                    )
                })?,
                reservation,
            )?;
            if returned != INITIAL_SINGLE_FRAME_REQUEST {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "anonymous release returned the wrong physical-frame extent",
                ));
            }
            transitions.push(SystemsTransition::ReleaseAnonymousUserMapping {
                address_space_identity: 1,
                reservation_identity,
            });
        }
        SystemsOperation::PrepareUserProbe { process_identity } => {
            if mapping.is_none()
                || resident_code.is_none()
                || anonymous_reservation.is_none()
                || suspended.is_some()
                || completed.is_some()
                || revoked.is_some()
            {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "user probe requires exactly one resident executable-code and immutable-data mapping",
                ));
            }
            *suspended = Some(protocol.prepare_probe(
                *process_identity,
                prepared.take().ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user probe model requires one prepared address space",
                    )
                })?,
            )?);
            transitions.push(SystemsTransition::PrepareUserProbe {
                process_identity: *process_identity,
                probe_identity: 1,
            });
        }
        SystemsOperation::EnterUserContext => {
            let suspended_context = suspended.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "user entry model requires one prepared suspended context",
                )
            })?;
            let active = protocol.enter(suspended_context)?;
            transitions.push(SystemsTransition::EnterUserExecution {
                process_identity: 1,
                kernel_continuation_identity: 1,
            });
            let (
                getpid_result,
                uname_value,
                nanosleep_value,
                remainder_candidate,
                exit_group_status,
            ) = match &handler.disposition {
                SystemsDisposition::DispatchInitialUserSyscall {
                    getpid_result,
                    uname_sysname,
                    uname_nodename,
                    uname_release,
                    uname_version,
                    uname_machine,
                    uname_domainname,
                    nanosleep_seconds,
                    nanosleep_nanoseconds,
                    nanosleep_remainder_candidate,
                    clock_gettime_result: _,
                    clock_getres_result: _,
                    write_descriptor: _,
                    write_maximum_bytes: _,
                    read_descriptor: _,
                    read_maximum_bytes: _,
                    initial_descriptor_count: _,
                    bad_descriptor_error: _,
                    too_many_open_files_error: _,
                    console_input_description: _,
                    console_output_description: _,
                    console_error_description: _,
                    invalid_argument_error: _,
                    fcntl_getfd_command: _,
                    fcntl_setfd_command: _,
                    fcntl_getfl_command: _,
                    close_on_exec_flag: _,
                    read_only_status: _,
                    write_only_status: _,
                    stat_layout_bytes: _,
                    console_device: _,
                    console_inode: _,
                    console_mode: _,
                    console_special_device: _,
                    console_block_size: _,
                    terminal_layout_bytes: _,
                    terminal_get_attributes_request: _,
                    terminal_control_flags: _,
                    terminal_minimum_input: _,
                    not_a_terminal_error: _,
                    window_size_layout_bytes: _,
                    window_size_query_request: _,
                    seek_maximum_whence: _,
                    illegal_seek_error: _,
                    brk_data_page_bytes: _,
                    getppid_result: _,
                    gettid_result: _,
                    getuid_result: _,
                    geteuid_result: _,
                    getgid_result: _,
                    getegid_result: _,
                    arch_set_fs_result: _,
                    arch_get_fs_result: _,
                    set_tid_address_result: _,
                    set_robust_list_result: _,
                    getrlimit_result: _,
                    prlimit64_result: _,
                    rt_sigprocmask_result: _,
                    anonymous_page_bytes: _,
                    madvise_result: _,
                    mprotect_result: _,
                    munmap_result: _,
                    exit_group_status,
                    unsupported_syscall_error: _,
                    sched_yield_result: _,
                } => (
                    *getpid_result,
                    LinuxUtsName {
                        sysname: uname_sysname.clone(),
                        nodename: uname_nodename.clone(),
                        release: uname_release.clone(),
                        version: uname_version.clone(),
                        machine: uname_machine.clone(),
                        domainname: uname_domainname.clone(),
                    },
                    LinuxTimespec {
                        seconds: i64::try_from(*nanosleep_seconds).map_err(|_| {
                            SystemsModelError::new(
                                "E-SYSTEMS-USER-TRANSFER",
                                "sealed nanosleep seconds exceed the native signed layout",
                            )
                        })?,
                        nanoseconds: i64::try_from(*nanosleep_nanoseconds).map_err(|_| {
                            SystemsModelError::new(
                                "E-SYSTEMS-USER-TRANSFER",
                                "sealed nanosleep nanoseconds exceed the native signed layout",
                            )
                        })?,
                    },
                    *nanosleep_remainder_candidate,
                    *exit_group_status,
                ),
                _ => {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user entry model requires the checked initial syscall dispatcher",
                    ));
                }
            };
            let getpid = protocol.enter_syscall(active, crate::LINUX_X86_64_GETPID, [0; 6])?;
            transitions.push(SystemsTransition::EnterUserSyscall {
                entry_identity: 1,
                number: crate::LINUX_X86_64_GETPID,
                arguments: [0; 6],
            });
            let active = protocol.return_getpid(getpid, getpid_result)?;
            transitions.push(SystemsTransition::ReturnToUser {
                entry_identity: 1,
                result: getpid_result,
            });
            let uname = protocol.enter_syscall(
                active,
                LINUX_X86_64_UNAME,
                [INITIAL_USER_UNAME_VALID_CANDIDATE, 0, 0, 0, 0, 0],
            )?;
            transitions.push(SystemsTransition::EnterUserSyscall {
                entry_identity: 2,
                number: LINUX_X86_64_UNAME,
                arguments: [INITIAL_USER_UNAME_VALID_CANDIDATE, 0, 0, 0, 0, 0],
            });
            let scope = protocol.begin_uname_transfer(uname)?;
            transitions.push(SystemsTransition::BeginUserTransfer {
                entry_identity: 2,
                transfer_identity: 1,
                candidate: INITIAL_USER_UNAME_VALID_CANDIDATE,
                extent: LINUX_NEW_UTSNAME_BYTES,
            });
            let outcome = protocol.copy_initial_utsname_to_user(
                scope,
                &uname_value,
                LINUX_NEW_UTSNAME_BYTES,
            )?;
            transitions.push(SystemsTransition::CopyToUserComplete {
                transfer_identity: 1,
                copied: LINUX_NEW_UTSNAME_BYTES,
            });
            let active = protocol.return_uname(outcome, 0)?;
            transitions.push(SystemsTransition::EndUserTransfer {
                entry_identity: 2,
                transfer_identity: 1,
            });
            transitions.push(SystemsTransition::ReturnToUser {
                entry_identity: 2,
                result: 0,
            });
            let uname = protocol.enter_syscall(
                active,
                LINUX_X86_64_UNAME,
                [INITIAL_USER_UNAME_FAULT_CANDIDATE, 0, 0, 0, 0, 0],
            )?;
            transitions.push(SystemsTransition::EnterUserSyscall {
                entry_identity: 3,
                number: LINUX_X86_64_UNAME,
                arguments: [INITIAL_USER_UNAME_FAULT_CANDIDATE, 0, 0, 0, 0, 0],
            });
            let scope = protocol.begin_uname_transfer(uname)?;
            transitions.push(SystemsTransition::BeginUserTransfer {
                entry_identity: 3,
                transfer_identity: 2,
                candidate: INITIAL_USER_UNAME_FAULT_CANDIDATE,
                extent: LINUX_NEW_UTSNAME_BYTES,
            });
            let outcome = protocol.copy_initial_utsname_to_user(scope, &uname_value, 256)?;
            transitions.push(SystemsTransition::CopyToUserFault {
                transfer_identity: 2,
                copied_prefix: 256,
            });
            let active = protocol.return_uname(outcome, LINUX_X86_64_EFAULT_RESULT)?;
            transitions.push(SystemsTransition::EndUserTransfer {
                entry_identity: 3,
                transfer_identity: 2,
            });
            transitions.push(SystemsTransition::ReturnToUser {
                entry_identity: 3,
                result: LINUX_X86_64_EFAULT_RESULT,
            });
            let nanosleep = protocol.enter_syscall(
                active,
                LINUX_X86_64_NANOSLEEP,
                [
                    INITIAL_USER_TIMESPEC_VALID_CANDIDATE,
                    remainder_candidate,
                    0,
                    0,
                    0,
                    0,
                ],
            )?;
            transitions.push(SystemsTransition::EnterUserSyscall {
                entry_identity: 4,
                number: LINUX_X86_64_NANOSLEEP,
                arguments: [
                    INITIAL_USER_TIMESPEC_VALID_CANDIDATE,
                    remainder_candidate,
                    0,
                    0,
                    0,
                    0,
                ],
            });
            let scope = protocol.begin_nanosleep_transfer(nanosleep)?;
            transitions.push(SystemsTransition::BeginUserTransfer {
                entry_identity: 4,
                transfer_identity: 3,
                candidate: INITIAL_USER_TIMESPEC_VALID_CANDIDATE,
                extent: LINUX_TIMESPEC_BYTES,
            });
            let outcome = protocol.copy_initial_timespec_from_user(
                scope,
                Some(nanosleep_value),
                LINUX_TIMESPEC_BYTES,
            )?;
            transitions.push(SystemsTransition::CopyFromUserComplete {
                transfer_identity: 3,
                copied: LINUX_TIMESPEC_BYTES,
                seconds: nanosleep_value.seconds,
                nanoseconds: nanosleep_value.nanoseconds,
            });
            let active = protocol.return_nanosleep(outcome, 0)?;
            transitions.push(SystemsTransition::EndUserTransfer {
                entry_identity: 4,
                transfer_identity: 3,
            });
            transitions.push(SystemsTransition::ReturnToUser {
                entry_identity: 4,
                result: 0,
            });
            let nanosleep = protocol.enter_syscall(
                active,
                LINUX_X86_64_NANOSLEEP,
                [
                    INITIAL_USER_TIMESPEC_FAULT_CANDIDATE,
                    remainder_candidate,
                    0,
                    0,
                    0,
                    0,
                ],
            )?;
            transitions.push(SystemsTransition::EnterUserSyscall {
                entry_identity: 5,
                number: LINUX_X86_64_NANOSLEEP,
                arguments: [
                    INITIAL_USER_TIMESPEC_FAULT_CANDIDATE,
                    remainder_candidate,
                    0,
                    0,
                    0,
                    0,
                ],
            });
            let scope = protocol.begin_nanosleep_transfer(nanosleep)?;
            transitions.push(SystemsTransition::BeginUserTransfer {
                entry_identity: 5,
                transfer_identity: 4,
                candidate: INITIAL_USER_TIMESPEC_FAULT_CANDIDATE,
                extent: LINUX_TIMESPEC_BYTES,
            });
            let outcome = protocol.copy_initial_timespec_from_user(scope, None, 8)?;
            transitions.push(SystemsTransition::CopyFromUserFault {
                transfer_identity: 4,
                copied_prefix: 8,
            });
            let active = protocol.return_nanosleep(outcome, LINUX_X86_64_EFAULT_RESULT)?;
            transitions.push(SystemsTransition::EndUserTransfer {
                entry_identity: 5,
                transfer_identity: 4,
            });
            transitions.push(SystemsTransition::ReturnToUser {
                entry_identity: 5,
                result: LINUX_X86_64_EFAULT_RESULT,
            });
            let exit = protocol.enter_syscall(
                active,
                crate::LINUX_X86_64_EXIT_GROUP,
                [exit_group_status, 0, 0, 0, 0, 0],
            )?;
            transitions.push(SystemsTransition::EnterUserSyscall {
                entry_identity: 6,
                number: crate::LINUX_X86_64_EXIT_GROUP,
                arguments: [exit_group_status, 0, 0, 0, 0, 0],
            });
            *completed = Some(protocol.terminate_exit_group(exit, exit_group_status)?);
            transitions.push(SystemsTransition::TerminateUser {
                entry_identity: 6,
                status: exit_group_status,
            });
            transitions.push(SystemsTransition::ResumeUserKernelContinuation {
                kernel_continuation_identity: 1,
                status: exit_group_status,
            });
        }
        SystemsOperation::CompleteUserExecution => {
            if mapping.is_some() || resident_code.is_some() {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "user completion requires both resident image mappings revoked",
                ));
            }
            let summary = protocol.complete(completed.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "user completion model requires terminal user execution",
                )
            })?)?;
            transitions.push(SystemsTransition::CompleteUserExecution {
                process_identity: summary.process_identity,
            });
            *revoked = Some(summary.revoked_address_space);
        }
        SystemsOperation::ReclaimUserAddressSpace => {
            let backing = protocol.reclaim_address_space(revoked.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "user-address-space reclaim model requires one revoked space",
                )
            })?)?;
            if backing != INITIAL_MULTI_FRAME_REQUEST {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-USER-CONTEXT",
                    "user-address-space reclaim returned the wrong backing shape",
                ));
            }
            transitions.push(SystemsTransition::ReclaimUserAddressSpace {
                address_space_identity: 1,
            });
        }
        _ => return Ok(false),
    }
    Ok(true)
}

#[allow(clippy::too_many_arguments)]
fn model_kernel_context_transition(
    operation: &SystemsOperation,
    preemptible_handler: &SystemsHandler,
    completion_handler: &SystemsHandler,
    terminal_handler: &SystemsHandler,
    deadline_handler: &SystemsHandler,
    storage: &mut BootstrapStorageState,
    region: &mut Option<BootstrapRegion>,
    context_stacks: &mut Vec<BootstrapRegion>,
    state: &mut BootstrapKernelContextState,
    deadline_state: &mut BootstrapDeadlineState,
    next_deadline_event_identity: &mut u64,
    transitions: &mut Vec<SystemsTransition>,
) -> Result<bool, SystemsModelError> {
    match operation {
        SystemsOperation::CreateKernelCompletion => {
            if *state != BootstrapKernelContextState::Fresh {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "kernel-completion model creates exactly one completion before workers",
                ));
            }
            transitions.push(SystemsTransition::CreateKernelCompletion {
                completion_identity: 1,
            });
        }
        SystemsOperation::CreateKernelContext => {
            context_stacks.push(region.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "kernel-context model requires one live stack region",
                )
            })?);
            let context_identity = context_stacks.len() as u64;
            *state = match *state {
                BootstrapKernelContextState::Fresh => {
                    BootstrapKernelContextState::PreemptibleCreated
                }
                BootstrapKernelContextState::PreemptibleCreated => {
                    BootstrapKernelContextState::CompletionCreated
                }
                BootstrapKernelContextState::CompletionCreated => {
                    BootstrapKernelContextState::AllSuspended
                }
                _ => {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "kernel-context model creates exactly three worker contexts",
                    ));
                }
            };
            transitions.push(SystemsTransition::CreateKernelContext { context_identity });
        }
        SystemsOperation::CreateRunnableQueue { capacity } => {
            if *state != BootstrapKernelContextState::AllSuspended || *capacity != 3 {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "kernel-context model requires capacity three after creating all workers",
                ));
            }
            *state = BootstrapKernelContextState::QueueCreated;
            transitions.push(SystemsTransition::CreateRunnableQueue {
                capacity: *capacity,
            });
        }
        SystemsOperation::EnqueueRunnableContext => {
            let (context_identity, next_state) = match *state {
                BootstrapKernelContextState::QueueCreated => {
                    (1, BootstrapKernelContextState::PreemptibleEnqueued)
                }
                BootstrapKernelContextState::PreemptibleEnqueued => {
                    (2, BootstrapKernelContextState::CompletionEnqueued)
                }
                BootstrapKernelContextState::CompletionEnqueued => {
                    (3, BootstrapKernelContextState::AllEnqueued)
                }
                BootstrapKernelContextState::PreemptedTaken => {
                    (1, BootstrapKernelContextState::PreemptedEnqueued)
                }
                BootstrapKernelContextState::CompletionSignaled => {
                    (2, BootstrapKernelContextState::CompletionEnqueuedAgain)
                }
                _ => {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "kernel-context model enqueue violates runnable FIFO ownership",
                    ));
                }
            };
            *state = next_state;
            transitions.push(SystemsTransition::EnqueueRunnableContext { context_identity });
        }
        SystemsOperation::DequeueRunnableContext => {
            let (context_identity, next_state) = match *state {
                BootstrapKernelContextState::AllEnqueued => {
                    (1, BootstrapKernelContextState::PreemptibleDequeued)
                }
                BootstrapKernelContextState::PreemptedEnqueued => {
                    (2, BootstrapKernelContextState::CompletionDequeued)
                }
                BootstrapKernelContextState::CompletionBlocked => {
                    (3, BootstrapKernelContextState::TerminalDequeued)
                }
                BootstrapKernelContextState::CompletionEnqueuedAgain => {
                    (1, BootstrapKernelContextState::PreemptibleDequeuedAgain)
                }
                BootstrapKernelContextState::PreemptibleReclaimed => {
                    (2, BootstrapKernelContextState::CompletionDequeuedAgain)
                }
                _ => {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "kernel-context model dequeue violates runnable FIFO ownership",
                    ));
                }
            };
            *state = next_state;
            transitions.push(SystemsTransition::DequeueRunnableContext { context_identity });
        }
        SystemsOperation::TransferKernelContextUntilDeadline => {
            if *state != BootstrapKernelContextState::PreemptibleDequeued
                || *deadline_state != BootstrapDeadlineState::Constructed
            {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "deadline-preemptible model transfer requires the first selected context and constructed deadline",
                ));
            }
            let [
                SystemsOperation::ConsoleWrite { text: entered },
                SystemsOperation::AwaitDeadlinePreemption,
                SystemsOperation::ConsoleWrite { .. },
            ] = preemptible_handler.operations.as_slice()
            else {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "preemptible handler must enter, await matching preemption, and resume",
                ));
            };
            if deadline_handler.operations != [SystemsOperation::CompleteDeadline]
                || deadline_handler.disposition != SystemsDisposition::PreemptKernelContext
            {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-DEADLINE-EVENT",
                    "deadline-preemption handler must complete and preempt the bound context",
                ));
            }
            let event_identity = *next_deadline_event_identity;
            *next_deadline_event_identity = event_identity.checked_add(1).ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-DEADLINE-EVENT",
                    "deadline-event identity exhausted",
                )
            })?;
            transitions.extend([
                SystemsTransition::ArmDeadline { event_identity },
                SystemsTransition::TransferKernelContextUntilDeadline {
                    context_identity: 1,
                    caller_identity: 1,
                    event_identity,
                },
                SystemsTransition::EnterKernelThread {
                    context_identity: 1,
                },
                SystemsTransition::ConsoleWrite {
                    text: entered.clone(),
                },
                SystemsTransition::AwaitDeadlinePreemption {
                    context_identity: 1,
                },
                SystemsTransition::ObserveDeadline { event_identity },
                SystemsTransition::EnterDeadlineInterrupt { event_identity },
                SystemsTransition::CompleteDeadlineInterrupt { event_identity },
                SystemsTransition::PreemptKernelContext {
                    event_identity,
                    context_identity: 1,
                    caller_identity: 1,
                },
                SystemsTransition::ResumePreemptionDispatcher { caller_identity: 1 },
            ]);
            *deadline_state = BootstrapDeadlineState::Completed;
            *state = BootstrapKernelContextState::PreemptedOutcome;
        }
        SystemsOperation::TakePreemptedKernelContext => {
            if *state != BootstrapKernelContextState::PreemptedOutcome
                || *deadline_state != BootstrapDeadlineState::Completed
            {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "taking the preempted model context requires its completed event outcome",
                ));
            }
            transitions.push(SystemsTransition::TakePreemptedKernelContext {
                context_identity: 1,
                event_identity: next_deadline_event_identity.saturating_sub(1),
            });
            *state = BootstrapKernelContextState::PreemptedTaken;
        }
        SystemsOperation::TransferKernelContextUntilCompletion => {
            if *state != BootstrapKernelContextState::CompletionDequeued {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "completion-bound model transfer requires the selected block-once worker",
                ));
            }
            let [
                SystemsOperation::ConsoleWrite { text: entered },
                SystemsOperation::AwaitKernelCompletion,
                SystemsOperation::ConsoleWrite { .. },
            ] = completion_handler.operations.as_slice()
            else {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "completion worker must enter, await its one-shot completion, and resume",
                ));
            };
            transitions.extend([
                SystemsTransition::TransferKernelContextUntilCompletion {
                    context_identity: 2,
                    caller_identity: 2,
                    completion_identity: 1,
                },
                SystemsTransition::EnterKernelThread {
                    context_identity: 2,
                },
                SystemsTransition::ConsoleWrite {
                    text: entered.clone(),
                },
                SystemsTransition::AwaitKernelCompletion {
                    context_identity: 2,
                    completion_identity: 1,
                },
                SystemsTransition::BlockKernelContext {
                    context_identity: 2,
                    caller_identity: 2,
                    completion_identity: 1,
                },
                SystemsTransition::ResumeCompletionDispatcher { caller_identity: 2 },
            ]);
            *state = BootstrapKernelContextState::CompletionBlocked;
        }
        SystemsOperation::SignalBlockedKernelCompletion => {
            if *state != BootstrapKernelContextState::TerminalReclaimed {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "completion signal requires the blocked worker after terminal reclamation",
                ));
            }
            transitions.push(SystemsTransition::SignalBlockedKernelCompletion {
                context_identity: 2,
                caller_identity: 2,
                completion_identity: 1,
            });
            *state = BootstrapKernelContextState::CompletionSignaled;
        }
        SystemsOperation::TransferKernelContext => match *state {
            BootstrapKernelContextState::TerminalDequeued => {
                let [SystemsOperation::ConsoleWrite { text }] =
                    terminal_handler.operations.as_slice()
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "terminal handler must write its entry marker before retirement",
                    ));
                };
                transitions.extend([
                    SystemsTransition::TransferKernelContext {
                        context_identity: 3,
                        caller_identity: 3,
                    },
                    SystemsTransition::EnterKernelThread {
                        context_identity: 3,
                    },
                    SystemsTransition::ConsoleWrite { text: text.clone() },
                    SystemsTransition::RetireKernelContextToCaller {
                        context_identity: 3,
                        caller_identity: 3,
                    },
                    SystemsTransition::ResumeKernelContextCaller { caller_identity: 3 },
                ]);
                *state = BootstrapKernelContextState::TerminalCompleted;
            }
            BootstrapKernelContextState::PreemptibleDequeuedAgain => {
                let [_, _, SystemsOperation::ConsoleWrite { text }] =
                    preemptible_handler.operations.as_slice()
                else {
                    unreachable!("validated preemptible handler shape")
                };
                transitions.extend([
                    SystemsTransition::TransferKernelContext {
                        context_identity: 1,
                        caller_identity: 4,
                    },
                    SystemsTransition::RestorePreemptedKernelContext {
                        context_identity: 1,
                        caller_identity: 4,
                        event_identity: next_deadline_event_identity.saturating_sub(1),
                    },
                    SystemsTransition::ConsoleWrite { text: text.clone() },
                    SystemsTransition::RetireKernelContextToCaller {
                        context_identity: 1,
                        caller_identity: 4,
                    },
                    SystemsTransition::ResumeKernelContextCaller { caller_identity: 4 },
                ]);
                *state = BootstrapKernelContextState::PreemptibleCompleted;
            }
            BootstrapKernelContextState::CompletionDequeuedAgain => {
                let [_, _, SystemsOperation::ConsoleWrite { text }] =
                    completion_handler.operations.as_slice()
                else {
                    unreachable!("validated completion handler shape")
                };
                transitions.extend([
                    SystemsTransition::TransferKernelContext {
                        context_identity: 2,
                        caller_identity: 5,
                    },
                    SystemsTransition::RestoreBlockedKernelContext {
                        context_identity: 2,
                        caller_identity: 5,
                        completion_identity: 1,
                    },
                    SystemsTransition::ConsoleWrite { text: text.clone() },
                    SystemsTransition::RetireKernelContextToCaller {
                        context_identity: 2,
                        caller_identity: 5,
                    },
                    SystemsTransition::ResumeKernelContextCaller { caller_identity: 5 },
                ]);
                *state = BootstrapKernelContextState::CompletionCompleted;
            }
            _ => {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "kernel-context model requires preemptible, completion, terminal, preemptible, completion FIFO order",
                ));
            }
        },
        SystemsOperation::ReclaimKernelContext => {
            let (stack_index, context_identity, next_state) = match *state {
                BootstrapKernelContextState::TerminalCompleted => {
                    (2, 3, BootstrapKernelContextState::TerminalReclaimed)
                }
                BootstrapKernelContextState::PreemptibleCompleted => {
                    (0, 1, BootstrapKernelContextState::PreemptibleReclaimed)
                }
                BootstrapKernelContextState::CompletionCompleted => {
                    (0, 2, BootstrapKernelContextState::CompletionReclaimed)
                }
                _ => {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "kernel-context reclaim requires one completed worker stack",
                    ));
                }
            };
            if context_stacks.len() <= stack_index {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "kernel-context reclaim lost its owned worker stack",
                ));
            }
            let stack = context_stacks.remove(stack_index);
            storage.release(stack)?;
            *state = next_state;
            transitions.push(SystemsTransition::ReclaimKernelContext { context_identity });
        }
        SystemsOperation::ConsumeEmptyRunnableQueue => {
            if *state != BootstrapKernelContextState::CompletionReclaimed {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "kernel-context model consumes only an empty queue after all three reclamations",
                ));
            }
            *state = BootstrapKernelContextState::QueueConsumed;
            transitions.push(SystemsTransition::ConsumeEmptyRunnableQueue);
        }
        SystemsOperation::ConsumeKernelCompletion => {
            if *state != BootstrapKernelContextState::CompletionReclaimed {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "kernel-completion model consumes completion only after awakened worker reclamation",
                ));
            }
            transitions.push(SystemsTransition::ConsumeKernelCompletion {
                completion_identity: 1,
            });
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn model_deadline_transition(
    operation: &SystemsOperation,
    handler: &SystemsHandler,
    state: &mut BootstrapDeadlineState,
    next_event_identity: &mut u64,
    transitions: &mut Vec<SystemsTransition>,
) -> Result<bool, SystemsModelError> {
    match operation {
        SystemsOperation::ConstructDeadline {
            duration_nanoseconds,
        } => {
            *state = BootstrapDeadlineState::Constructed;
            transitions.push(SystemsTransition::ConstructDeadline {
                source_observation_identity: 2,
                duration_nanoseconds: *duration_nanoseconds,
            });
        }
        SystemsOperation::ArmDeadline => {
            let event_identity = *next_event_identity;
            *next_event_identity = event_identity.checked_add(1).ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-DEADLINE-EVENT",
                    "deadline-event identity exhausted",
                )
            })?;
            *state = BootstrapDeadlineState::Armed;
            transitions.push(SystemsTransition::ArmDeadline { event_identity });
        }
        SystemsOperation::WaitDeadline => {
            let event_identity = (*next_event_identity).saturating_sub(1);
            transitions.extend([
                SystemsTransition::BeginDeadlineWait { event_identity },
                SystemsTransition::ObserveDeadline { event_identity },
                SystemsTransition::EnterDeadlineInterrupt { event_identity },
            ]);
            for handler_operation in &handler.operations {
                if !matches!(handler_operation, SystemsOperation::CompleteDeadline) {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-DEADLINE-EVENT",
                        "deadline handler contains an operation outside its completion protocol",
                    ));
                }
                transitions.push(SystemsTransition::CompleteDeadlineInterrupt { event_identity });
            }
            if handler.disposition != SystemsDisposition::Resume {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-DEADLINE-EVENT",
                    "deadline handler must resume after completion",
                ));
            }
            transitions.extend([
                SystemsTransition::ResumeDeadlineInterrupt { event_identity },
                SystemsTransition::EndDeadlineWait { event_identity },
            ]);
            *state = BootstrapDeadlineState::Completed;
        }
        SystemsOperation::CompleteDeadline => {
            unreachable!("deadline completion is modeled inside the matching wait")
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn model_monotonic_clock_transition(
    operation: &SystemsOperation,
    next_observation_identity: &mut u64,
    transitions: &mut Vec<SystemsTransition>,
) -> Result<bool, SystemsModelError> {
    if !matches!(operation, SystemsOperation::ObserveMonotonicClock) {
        return Ok(false);
    }
    let observation_identity = *next_observation_identity;
    *next_observation_identity = observation_identity.checked_add(1).ok_or_else(|| {
        SystemsModelError::new(
            "E-SYSTEMS-MONOTONIC-CLOCK",
            "monotonic-clock observation identity exhausted",
        )
    })?;
    transitions.push(SystemsTransition::ObserveMonotonicClock {
        observation_identity,
    });
    Ok(true)
}

fn model_bootstrap_operation(
    operation: &SystemsOperation,
    debug_break_handler: &SystemsHandler,
    storage: &mut BootstrapStorageState,
    region: &mut Option<BootstrapRegion>,
    atomic: &mut Option<AtomicWordLocation>,
    next_pci_transaction_identity: &mut u64,
    next_pci_region_transaction_identity: &mut u64,
    next_pci_update_transaction_identity: &mut u64,
    transitions: &mut Vec<SystemsTransition>,
) -> Result<bool, SystemsModelError> {
    match operation {
        SystemsOperation::DescribeBootMemory { .. } => {
            transitions.push(SystemsTransition::DescribeBootMemory);
        }
        SystemsOperation::CreateFrameAllocator { .. } => {
            transitions.push(SystemsTransition::CreateFrameAllocator);
        }
        SystemsOperation::AllocatePhysicalFrames { request, .. } => {
            transitions.push(SystemsTransition::AllocatePhysicalFrames { request: *request });
        }
        SystemsOperation::ReleasePhysicalFrames => {
            transitions.push(SystemsTransition::ReleasePhysicalFrames);
        }
        SystemsOperation::PrepareStaticImageConstruction { .. } => {
            transitions.push(SystemsTransition::PrepareStaticImageConstruction);
        }
        SystemsOperation::ZeroStaticImageConstructionPayload => {
            transitions.push(SystemsTransition::ZeroStaticImageConstructionPayload);
        }
        SystemsOperation::ZeroStaticImageStackBacking => {
            transitions.push(SystemsTransition::ZeroStaticImageStackBacking);
        }
        SystemsOperation::PublishStaticImageStackLeaf => {
            transitions.push(SystemsTransition::PublishStaticImageStackLeaf);
        }
        SystemsOperation::InitializeStaticImageStackEntry => {
            transitions.push(SystemsTransition::InitializeStaticImageStackEntry);
        }
        SystemsOperation::AddStaticImageStackPageSize => {
            transitions.push(SystemsTransition::AddStaticImageStackPageSize);
        }
        SystemsOperation::AddStaticImageStackEntryFact => {
            transitions.push(SystemsTransition::AddStaticImageStackEntryFact);
        }
        SystemsOperation::AddStaticImageStackProgramHeaders => {
            transitions.push(SystemsTransition::AddStaticImageStackProgramHeaders);
        }
        SystemsOperation::AddStaticImageStackExecutableName => {
            transitions.push(SystemsTransition::AddStaticImageStackExecutableName);
        }
        SystemsOperation::AddStaticImageStackCredentials => {
            transitions.push(SystemsTransition::AddStaticImageStackCredentials);
        }
        SystemsOperation::AddStaticImageStackSecureMode => {
            transitions.push(SystemsTransition::AddStaticImageStackSecureMode);
        }
        SystemsOperation::AddStaticImageStackClockTickRate => {
            transitions.push(SystemsTransition::AddStaticImageStackClockTickRate);
        }
        SystemsOperation::AddStaticImageStackPlatform => {
            transitions.push(SystemsTransition::AddStaticImageStackPlatform);
        }
        SystemsOperation::AddStaticImageStackHardwareCapabilities => {
            transitions.push(SystemsTransition::AddStaticImageStackHardwareCapabilities);
        }
        SystemsOperation::AddStaticImageStackLoadMetadata => {
            transitions.push(SystemsTransition::AddStaticImageStackLoadMetadata);
        }
        SystemsOperation::AuthorizeStaticImageStackRandomSlot => {
            transitions.push(SystemsTransition::AuthorizeStaticImageStackRandomSlot);
        }
        SystemsOperation::RetainStaticImagePreparedProcess => {
            transitions.push(SystemsTransition::RetainStaticImagePreparedProcess);
        }
        SystemsOperation::QualifyStaticImageRetainedDisjointWork => {
            transitions.push(SystemsTransition::QualifyStaticImageRetainedDisjointWork);
        }
        SystemsOperation::AuthorizeStaticImageEntropyHandoff => {
            transitions.push(SystemsTransition::AuthorizeStaticImageEntropyHandoff);
        }
        SystemsOperation::AuthorizeStaticImageEntropyAcquisition => {
            transitions.push(SystemsTransition::AuthorizeStaticImageEntropyAcquisition);
        }
        SystemsOperation::QualifyStaticImageEntropyDevice => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyDevice);
        }
        SystemsOperation::QualifyStaticImageEntropyTransport => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyTransport);
        }
        SystemsOperation::QualifyStaticImageEntropyNegotiation => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyNegotiation);
        }
        SystemsOperation::QualifyStaticImageEntropyQueue => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyQueue);
        }
        SystemsOperation::QualifyStaticImageEntropyBusMaster => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyBusMaster);
        }
        SystemsOperation::QualifyStaticImageEntropyDmaBuffers => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyDmaBuffers);
        }
        SystemsOperation::QualifyStaticImageEntropySplitRing => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropySplitRing);
        }
        SystemsOperation::QualifyStaticImageEntropyDmaPreparation => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyDmaPreparation);
        }
        SystemsOperation::QualifyStaticImageEntropyQueueAddresses => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyQueueAddresses);
        }
        SystemsOperation::QualifyStaticImageEntropyQueueActivation => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyQueueActivation);
        }
        SystemsOperation::QualifyStaticImageEntropyRequestCompletion => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyRequestCompletion);
        }
        SystemsOperation::QualifyStaticImageEntropySampleCapture => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropySampleCapture);
        }
        SystemsOperation::QualifyStaticImageEntropySampleQuality => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropySampleQuality);
        }
        SystemsOperation::QualifyStaticImageEntropyPoolLifecycle => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyPoolLifecycle);
        }
        SystemsOperation::QualifyStaticImageEntropyGeneratorLifecycle => {
            transitions.push(SystemsTransition::QualifyStaticImageEntropyGeneratorLifecycle);
        }
        SystemsOperation::DeliverStaticImageEntropyInitialRandom => {
            transitions.push(SystemsTransition::DeliverStaticImageEntropyInitialRandom);
        }
        SystemsOperation::EnterStaticImageProcess => {
            transitions.push(SystemsTransition::EnterStaticImageProcess);
        }
        SystemsOperation::RestoreStaticImageProcessEntry => {
            transitions.push(SystemsTransition::RestoreStaticImageProcessEntry);
        }
        SystemsOperation::RevokeStaticImageEntropyInitialRandom => {
            transitions.push(SystemsTransition::RevokeStaticImageEntropyInitialRandom);
        }
        SystemsOperation::RevokeStaticImageEntropyAcquisition => {
            transitions.push(SystemsTransition::RevokeStaticImageEntropyAcquisition);
        }
        SystemsOperation::RevokeStaticImageEntropyHandoff => {
            transitions.push(SystemsTransition::RevokeStaticImageEntropyHandoff);
        }
        SystemsOperation::RestoreStaticImagePreparedProcess => {
            transitions.push(SystemsTransition::RestoreStaticImagePreparedProcess);
        }
        SystemsOperation::RevokeStaticImageStackRandomSlot => {
            transitions.push(SystemsTransition::RevokeStaticImageStackRandomSlot);
        }
        SystemsOperation::RemoveStaticImageStackLoadMetadata => {
            transitions.push(SystemsTransition::RemoveStaticImageStackLoadMetadata);
        }
        SystemsOperation::RemoveStaticImageStackHardwareCapabilities => {
            transitions.push(SystemsTransition::RemoveStaticImageStackHardwareCapabilities);
        }
        SystemsOperation::RemoveStaticImageStackPlatform => {
            transitions.push(SystemsTransition::RemoveStaticImageStackPlatform);
        }
        SystemsOperation::RemoveStaticImageStackClockTickRate => {
            transitions.push(SystemsTransition::RemoveStaticImageStackClockTickRate);
        }
        SystemsOperation::RemoveStaticImageStackSecureMode => {
            transitions.push(SystemsTransition::RemoveStaticImageStackSecureMode);
        }
        SystemsOperation::RemoveStaticImageStackCredentials => {
            transitions.push(SystemsTransition::RemoveStaticImageStackCredentials);
        }
        SystemsOperation::RemoveStaticImageStackExecutableName => {
            transitions.push(SystemsTransition::RemoveStaticImageStackExecutableName);
        }
        SystemsOperation::RemoveStaticImageStackProgramHeaders => {
            transitions.push(SystemsTransition::RemoveStaticImageStackProgramHeaders);
        }
        SystemsOperation::RemoveStaticImageStackEntryFact => {
            transitions.push(SystemsTransition::RemoveStaticImageStackEntryFact);
        }
        SystemsOperation::RemoveStaticImageStackPageSize => {
            transitions.push(SystemsTransition::RemoveStaticImageStackPageSize);
        }
        SystemsOperation::ClearStaticImageStackEntry => {
            transitions.push(SystemsTransition::ClearStaticImageStackEntry);
        }
        SystemsOperation::RevokeStaticImageStackLeaf => {
            transitions.push(SystemsTransition::RevokeStaticImageStackLeaf);
        }
        SystemsOperation::CopyStaticImageConstructionPayload => {
            transitions.push(SystemsTransition::CopyStaticImageConstructionPayload);
        }
        SystemsOperation::SealStaticImageConstructionPermissions => {
            transitions.push(SystemsTransition::SealStaticImageConstructionPermissions);
        }
        SystemsOperation::AttachStaticImageAddressSpace => {
            transitions.push(SystemsTransition::AttachStaticImageAddressSpace);
        }
        SystemsOperation::ActivateStaticImageAddressSpace => {
            transitions.push(SystemsTransition::ActivateStaticImageAddressSpace);
        }
        SystemsOperation::RestoreStaticImageAddressSpace => {
            transitions.push(SystemsTransition::RestoreStaticImageAddressSpace);
        }
        SystemsOperation::AuthorizeStaticImageEntry => {
            transitions.push(SystemsTransition::AuthorizeStaticImageEntry);
        }
        SystemsOperation::AuthorizeStaticImageStackSlot => {
            transitions.push(SystemsTransition::AuthorizeStaticImageStackSlot);
        }
        SystemsOperation::RevokeStaticImageStackSlot => {
            transitions.push(SystemsTransition::RevokeStaticImageStackSlot);
        }
        SystemsOperation::RevokeStaticImageEntry => {
            transitions.push(SystemsTransition::RevokeStaticImageEntry);
        }
        SystemsOperation::DetachStaticImageAddressSpace => {
            transitions.push(SystemsTransition::DetachStaticImageAddressSpace);
        }
        SystemsOperation::ReclaimStaticImageConstructionHierarchy => {
            transitions.push(SystemsTransition::ReclaimStaticImageConstructionHierarchy);
        }
        SystemsOperation::RevokeStaticImageConstructionMetadata => {
            transitions.push(SystemsTransition::RevokeStaticImageConstructionMetadata);
        }
        SystemsOperation::ReclaimStaticImageConstructionPayload => {
            transitions.push(SystemsTransition::ReclaimStaticImageConstructionPayload);
        }
        SystemsOperation::MapKernelFrames { request, .. } => {
            transitions.push(SystemsTransition::MapKernelFrames { request: *request });
        }
        SystemsOperation::KernelMappingStoreByte {
            offset_bytes,
            value,
        } => transitions.push(SystemsTransition::StoreKernelMappingByte {
            offset_bytes: *offset_bytes,
            value: *value,
        }),
        SystemsOperation::KernelMappingLoadByteEquals {
            offset_bytes,
            expected,
            ..
        } => transitions.push(SystemsTransition::LoadKernelMappingByte {
            offset_bytes: *offset_bytes,
            value: *expected,
        }),
        SystemsOperation::UnmapKernelFrames => {
            transitions.push(SystemsTransition::UnmapKernelFrames);
        }
        SystemsOperation::BeginTranslationUpdate { request, .. } => {
            transitions.push(SystemsTransition::BeginTranslationUpdate { request: *request });
        }
        SystemsOperation::CommitTranslationUpdate { .. } => {
            transitions.push(SystemsTransition::CommitTranslationUpdate);
        }
        SystemsOperation::ActivateTranslationSpace { .. } => {
            transitions.push(SystemsTransition::ActivateTranslationSpace);
        }
        SystemsOperation::BeginTranslationEdit { kind, .. } => {
            transitions.push(SystemsTransition::BeginTranslationEdit { kind: *kind });
        }
        SystemsOperation::MapTranslationFrames { request, .. } => {
            transitions.push(SystemsTransition::MapTranslationFrames { request: *request });
        }
        SystemsOperation::UnmapTranslationMapping => {
            transitions.push(SystemsTransition::UnmapTranslationMapping);
        }
        SystemsOperation::CommitTranslationEdit { kind, .. } => {
            transitions.push(SystemsTransition::CommitTranslationEdit { kind: *kind });
        }
        SystemsOperation::ConsoleWrite { text } => {
            transitions.push(SystemsTransition::ConsoleWrite { text: text.clone() });
        }
        SystemsOperation::ObservePciConfigurationDword { request } => {
            let transaction_identity = *next_pci_transaction_identity;
            *next_pci_transaction_identity =
                transaction_identity.checked_add(1).ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-PCI-CONFIG",
                        "PCI configuration transaction identity exhausted",
                    )
                })?;
            transitions.push(SystemsTransition::ObservePciConfigurationDword {
                transaction_identity,
                request: *request,
            });
        }
        SystemsOperation::UpdatePciConfigurationWord { request } => {
            let transaction_identity = *next_pci_update_transaction_identity;
            *next_pci_update_transaction_identity =
                transaction_identity.checked_add(1).ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-PCI-CONFIG-UPDATE",
                        "PCI configuration update transaction identity exhausted",
                    )
                })?;
            transitions.push(SystemsTransition::UpdatePciConfigurationWord {
                transaction_identity,
                request: *request,
            });
        }
        SystemsOperation::CreateDmaBuffer { request } => {
            transitions.push(SystemsTransition::CreateDmaBuffer { request: *request });
        }
        SystemsOperation::BindDmaBufferReference { request } => {
            transitions.push(SystemsTransition::BindDmaBufferReference { request: *request });
        }
        SystemsOperation::StoreDmaBufferInteger { request } => {
            transitions.push(SystemsTransition::StoreDmaBufferInteger { request: *request });
        }
        SystemsOperation::PrepareDmaBufferForDevice { request } => {
            transitions.push(SystemsTransition::PrepareDmaBufferForDevice { request: *request });
        }
        SystemsOperation::CompleteDmaBufferForCpu { request } => {
            transitions.push(SystemsTransition::CompleteDmaBufferForCpu { request: *request });
        }
        SystemsOperation::LoadDmaBufferIntegerEquals { request } => {
            transitions.push(SystemsTransition::LoadDmaBufferIntegerEquals { request: *request });
        }
        SystemsOperation::CaptureEntropySample { request } => {
            transitions.push(SystemsTransition::CaptureEntropySample { request: *request });
        }
        SystemsOperation::AssessEntropySampleQuality { request } => {
            transitions.push(SystemsTransition::AssessEntropySampleQuality { request: *request });
        }
        SystemsOperation::CreateEntropyPool { request } => {
            transitions.push(SystemsTransition::CreateEntropyPool { request: *request });
        }
        SystemsOperation::MixEntropyPoolSample { request } => {
            transitions.push(SystemsTransition::MixEntropyPoolSample { request: *request });
        }
        SystemsOperation::CreditEntropyPoolContribution { request } => {
            transitions
                .push(SystemsTransition::CreditEntropyPoolContribution { request: *request });
        }
        SystemsOperation::InitializeEntropyGenerator { request } => {
            transitions.push(SystemsTransition::InitializeEntropyGenerator { request: *request });
        }
        SystemsOperation::ExtractEntropyGenerator { request } => {
            transitions.push(SystemsTransition::ExtractEntropyGenerator { request: *request });
        }
        SystemsOperation::DeliverInitialAuxvRandom { request } => {
            transitions.push(SystemsTransition::DeliverInitialAuxvRandom { request: *request });
        }
        SystemsOperation::ReserveInitialUserRandom { request } => {
            transitions.push(SystemsTransition::ReserveInitialUserRandom { request: *request });
        }
        SystemsOperation::BindReusableUserRandom { request } => {
            transitions.push(SystemsTransition::BindReusableUserRandom { request: *request });
        }
        SystemsOperation::ExpandSmallUserRandomRequests { request } => {
            transitions
                .push(SystemsTransition::ExpandSmallUserRandomRequests { request: *request });
        }
        SystemsOperation::ExpandLargeUserRandomRequests { request } => {
            transitions
                .push(SystemsTransition::ExpandLargeUserRandomRequests { request: *request });
        }
        SystemsOperation::PublishPciRegionDmaReference { request } => {
            let transaction_identity = *next_pci_region_transaction_identity;
            *next_pci_region_transaction_identity =
                transaction_identity.checked_add(1).ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-DMA-PUBLISH",
                        "PCI-region DMA-reference transaction identity exhausted",
                    )
                })?;
            transitions.push(SystemsTransition::PublishPciRegionDmaReference {
                transaction_identity,
                request: *request,
            });
        }
        SystemsOperation::AccessPciRegionRegister { request } => {
            let transaction_identity = *next_pci_region_transaction_identity;
            *next_pci_region_transaction_identity =
                transaction_identity.checked_add(1).ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-PCI-REGION",
                        "PCI region transaction identity exhausted",
                    )
                })?;
            transitions.push(SystemsTransition::AccessPciRegionRegister {
                transaction_identity,
                request: *request,
            });
        }
        SystemsOperation::DebugBreak => {
            return Ok(model_debug_break(debug_break_handler, transitions));
        }
        _ => {
            return model_bootstrap_storage_operation(
                operation,
                storage,
                region,
                atomic,
                transitions,
            );
        }
    }
    Ok(false)
}

fn model_local_notification_transition(
    operation: &SystemsOperation,
    handler: &SystemsHandler,
    protocol: &mut LocalNotificationProtocol,
    pending: &mut Option<u64>,
    transitions: &mut Vec<SystemsTransition>,
) -> Result<bool, SystemsModelError> {
    match operation {
        SystemsOperation::SendLocalNotification => {
            let event_identity = protocol.send(LocalInterruptMaskState::Disabled)?;
            if pending.replace(event_identity).is_some() {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-LOCAL-INTERRUPT",
                    "local-notification model encountered a second pending event",
                ));
            }
            transitions.push(SystemsTransition::SendLocalNotification { event_identity });
        }
        SystemsOperation::WaitLocalNotification => {
            let event_identity = pending.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-LOCAL-INTERRUPT",
                    "local-notification model wait lost its pending event",
                )
            })?;
            transitions.push(SystemsTransition::BeginLocalNotificationWait { event_identity });
            protocol.enter(event_identity)?;
            transitions.push(SystemsTransition::ObserveLocalNotification { event_identity });
            transitions.push(SystemsTransition::EnterLocalNotificationInterrupt { event_identity });
            for handler_operation in &handler.operations {
                if !matches!(
                    handler_operation,
                    SystemsOperation::CompleteLocalNotification
                ) {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-LOCAL-INTERRUPT",
                        "local-notification model encountered an unsupported handler operation",
                    ));
                }
                protocol.complete(event_identity)?;
                transitions
                    .push(SystemsTransition::CompleteLocalNotificationInterrupt { event_identity });
            }
            if handler.disposition != SystemsDisposition::Resume {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-LOCAL-INTERRUPT",
                    "local-notification model requires resume after completion",
                ));
            }
            protocol.resume(event_identity)?;
            transitions
                .push(SystemsTransition::ResumeLocalNotificationInterrupt { event_identity });
            let restored = protocol.finish_wait(event_identity)?;
            if restored != LocalInterruptMaskState::Disabled {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-LOCAL-INTERRUPT",
                    "local-notification wait did not restore the sealed initial mask state",
                ));
            }
            transitions.push(SystemsTransition::EndLocalNotificationWait { event_identity });
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn model_bootstrap_disposition(
    disposition: &SystemsDisposition,
    transitions: &mut Vec<SystemsTransition>,
) {
    if let SystemsDisposition::Fatal { message } = disposition {
        transitions.push(SystemsTransition::Fatal {
            message: message.clone(),
        });
    }
}

fn initial_systems_transitions(program: &SystemsProgram) -> Vec<SystemsTransition> {
    vec![
        SystemsTransition::ProvisionBootstrapStorage {
            capacity_bytes: program.bootstrap_storage.capacity_bytes,
            alignment_bytes: program.bootstrap_storage.alignment_bytes,
        },
        SystemsTransition::EnterBootstrap,
    ]
}

fn model_critical_transition(
    operation: &SystemsOperation,
    critical_stack: &mut Vec<(CriticalDomain, u64)>,
    next_nesting_identity: &mut u64,
    transitions: &mut Vec<SystemsTransition>,
) -> Result<bool, SystemsModelError> {
    match operation {
        SystemsOperation::EnterCritical { domain, .. } => {
            let nesting_identity = *next_nesting_identity;
            *next_nesting_identity += 1;
            critical_stack.push((*domain, nesting_identity));
            transitions.push(SystemsTransition::EnterCritical {
                domain: *domain,
                nesting_identity,
            });
        }
        SystemsOperation::RestoreCritical { domain } => {
            let Some((entered_domain, nesting_identity)) = critical_stack.pop() else {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CRITICAL",
                    "validated critical restoration lost its matching entry",
                ));
            };
            if entered_domain != *domain {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CRITICAL",
                    "validated critical restoration changed its domain",
                ));
            }
            transitions.push(SystemsTransition::RestoreCritical {
                domain: *domain,
                nesting_identity,
            });
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn model_debug_break(handler: &SystemsHandler, transitions: &mut Vec<SystemsTransition>) -> bool {
    transitions.push(SystemsTransition::ObserveDebugBreak);
    transitions.push(SystemsTransition::EnterDebugBreak);
    for operation in &handler.operations {
        if let SystemsOperation::ConsoleWrite { text } = operation {
            transitions.push(SystemsTransition::ConsoleWrite { text: text.clone() });
        }
    }
    match &handler.disposition {
        SystemsDisposition::Resume => {
            transitions.push(SystemsTransition::ResumeDebugBreak);
            false
        }
        SystemsDisposition::Fatal { message } => {
            transitions.push(SystemsTransition::Fatal {
                message: message.clone(),
            });
            true
        }
        SystemsDisposition::RetireKernelContextToCaller => {
            unreachable!("debug-break validation rejects kernel-thread retirement")
        }
        SystemsDisposition::PreemptKernelContext => {
            unreachable!("debug-break validation rejects kernel-context preemption")
        }
        SystemsDisposition::DispatchInitialUserSyscall { .. } => {
            unreachable!("debug-break validation rejects user-syscall dispatch")
        }
    }
}

fn model_atomic_operation(
    operation: &SystemsOperation,
    storage: &mut BootstrapStorageState,
    region: &mut Option<BootstrapRegion>,
    atomic: &mut Option<AtomicWordLocation>,
    transitions: &mut Vec<SystemsTransition>,
) -> Result<Option<bool>, SystemsModelError> {
    match operation {
        SystemsOperation::AtomicWordCreate { request } => {
            let owned_region = region.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-ATOMIC-LIFETIME",
                    "atomic word creation requires one live ordinary region",
                )
            })?;
            *atomic = Some(AtomicWordLocation::create(storage, owned_region, *request)?);
            transitions.push(SystemsTransition::CreateAtomicWord { request: *request });
        }
        SystemsOperation::AtomicCompareExchangeEquals {
            expected,
            desired,
            success_order,
            failure_order,
            failure_message,
        } => {
            let outcome = atomic
                .as_mut()
                .ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-ATOMIC-LIFETIME",
                        "compare/exchange requires one live atomic word",
                    )
                })?
                .compare_exchange(storage, *expected, *desired, *success_order, *failure_order)?;
            let (observed, exchanged) = match outcome {
                AtomicCompareExchangeResult::Exchanged { previous } => (previous, true),
                AtomicCompareExchangeResult::Observed { actual } => (actual, false),
            };
            transitions.push(SystemsTransition::CompareExchangeAtomicWord {
                expected: *expected,
                desired: *desired,
                observed,
                exchanged,
                success_order: *success_order,
                failure_order: *failure_order,
            });
            if !exchanged {
                transitions.push(SystemsTransition::Fatal {
                    message: failure_message.clone(),
                });
                return Ok(Some(true));
            }
        }
        SystemsOperation::AtomicLoadEquals {
            order,
            expected,
            failure_message,
        } => {
            let value = atomic
                .as_ref()
                .ok_or_else(|| {
                    SystemsModelError::new(
                        "E-SYSTEMS-ATOMIC-LIFETIME",
                        "atomic load requires one live atomic word",
                    )
                })?
                .load(storage, *order)?;
            transitions.push(SystemsTransition::LoadAtomicWord {
                value,
                order: *order,
            });
            if value != *expected {
                transitions.push(SystemsTransition::Fatal {
                    message: failure_message.clone(),
                });
                return Ok(Some(true));
            }
        }
        SystemsOperation::AtomicWordEnd => {
            let location = atomic.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-ATOMIC-LIFETIME",
                    "atomic end requires one live atomic word",
                )
            })?;
            *region = Some(location.end());
            transitions.push(SystemsTransition::EndAtomicWord);
        }
        _ => return Ok(None),
    }
    Ok(Some(false))
}

fn model_bootstrap_storage_operation(
    operation: &SystemsOperation,
    storage: &mut BootstrapStorageState,
    region: &mut Option<BootstrapRegion>,
    atomic: &mut Option<AtomicWordLocation>,
    transitions: &mut Vec<SystemsTransition>,
) -> Result<bool, SystemsModelError> {
    if let Some(terminal) = model_atomic_operation(operation, storage, region, atomic, transitions)?
    {
        return Ok(terminal);
    }
    match operation {
        SystemsOperation::DescribeBootMemory { .. } => {
            unreachable!("boot-memory refinement is modeled by the caller")
        }
        SystemsOperation::CreateFrameAllocator { .. }
        | SystemsOperation::AllocatePhysicalFrames { .. }
        | SystemsOperation::ReleasePhysicalFrames
        | SystemsOperation::PrepareStaticImageConstruction { .. }
        | SystemsOperation::ZeroStaticImageConstructionPayload
        | SystemsOperation::ZeroStaticImageStackBacking
        | SystemsOperation::PublishStaticImageStackLeaf
        | SystemsOperation::InitializeStaticImageStackEntry
        | SystemsOperation::AddStaticImageStackPageSize
        | SystemsOperation::AddStaticImageStackEntryFact
        | SystemsOperation::AddStaticImageStackProgramHeaders
        | SystemsOperation::AddStaticImageStackExecutableName
        | SystemsOperation::AddStaticImageStackCredentials
        | SystemsOperation::AddStaticImageStackSecureMode
        | SystemsOperation::AddStaticImageStackClockTickRate
        | SystemsOperation::AddStaticImageStackPlatform
        | SystemsOperation::AddStaticImageStackHardwareCapabilities
        | SystemsOperation::AddStaticImageStackLoadMetadata
        | SystemsOperation::AuthorizeStaticImageStackRandomSlot
        | SystemsOperation::RetainStaticImagePreparedProcess
        | SystemsOperation::QualifyStaticImageRetainedDisjointWork
        | SystemsOperation::AuthorizeStaticImageEntropyHandoff
        | SystemsOperation::AuthorizeStaticImageEntropyAcquisition
        | SystemsOperation::QualifyStaticImageEntropyDevice
        | SystemsOperation::QualifyStaticImageEntropyTransport
        | SystemsOperation::QualifyStaticImageEntropyNegotiation
        | SystemsOperation::QualifyStaticImageEntropyQueue
        | SystemsOperation::QualifyStaticImageEntropyBusMaster
        | SystemsOperation::QualifyStaticImageEntropyDmaBuffers
        | SystemsOperation::QualifyStaticImageEntropySplitRing
        | SystemsOperation::QualifyStaticImageEntropyDmaPreparation
        | SystemsOperation::QualifyStaticImageEntropyQueueAddresses
        | SystemsOperation::QualifyStaticImageEntropyQueueActivation
        | SystemsOperation::QualifyStaticImageEntropyRequestCompletion
        | SystemsOperation::QualifyStaticImageEntropySampleCapture
        | SystemsOperation::QualifyStaticImageEntropySampleQuality
        | SystemsOperation::QualifyStaticImageEntropyPoolLifecycle
        | SystemsOperation::QualifyStaticImageEntropyGeneratorLifecycle
        | SystemsOperation::DeliverStaticImageEntropyInitialRandom
        | SystemsOperation::EnterStaticImageProcess
        | SystemsOperation::RestoreStaticImageProcessEntry
        | SystemsOperation::RevokeStaticImageEntropyInitialRandom
        | SystemsOperation::RevokeStaticImageEntropyAcquisition
        | SystemsOperation::RevokeStaticImageEntropyHandoff
        | SystemsOperation::RestoreStaticImagePreparedProcess
        | SystemsOperation::RevokeStaticImageStackRandomSlot
        | SystemsOperation::RemoveStaticImageStackLoadMetadata
        | SystemsOperation::RemoveStaticImageStackHardwareCapabilities
        | SystemsOperation::RemoveStaticImageStackPlatform
        | SystemsOperation::RemoveStaticImageStackClockTickRate
        | SystemsOperation::RemoveStaticImageStackSecureMode
        | SystemsOperation::RemoveStaticImageStackCredentials
        | SystemsOperation::RemoveStaticImageStackExecutableName
        | SystemsOperation::RemoveStaticImageStackProgramHeaders
        | SystemsOperation::RemoveStaticImageStackEntryFact
        | SystemsOperation::RemoveStaticImageStackPageSize
        | SystemsOperation::ClearStaticImageStackEntry
        | SystemsOperation::RevokeStaticImageStackLeaf
        | SystemsOperation::CopyStaticImageConstructionPayload
        | SystemsOperation::SealStaticImageConstructionPermissions
        | SystemsOperation::AttachStaticImageAddressSpace
        | SystemsOperation::ActivateStaticImageAddressSpace
        | SystemsOperation::RestoreStaticImageAddressSpace
        | SystemsOperation::AuthorizeStaticImageEntry
        | SystemsOperation::AuthorizeStaticImageStackSlot
        | SystemsOperation::RevokeStaticImageStackSlot
        | SystemsOperation::RevokeStaticImageEntry
        | SystemsOperation::DetachStaticImageAddressSpace
        | SystemsOperation::ReclaimStaticImageConstructionHierarchy
        | SystemsOperation::RevokeStaticImageConstructionMetadata
        | SystemsOperation::ReclaimStaticImageConstructionPayload
        | SystemsOperation::MapKernelFrames { .. }
        | SystemsOperation::KernelMappingStoreByte { .. }
        | SystemsOperation::KernelMappingLoadByteEquals { .. }
        | SystemsOperation::UnmapKernelFrames => {
            unreachable!("physical-frame and mapping operations are modeled by the caller")
        }
        SystemsOperation::BeginTranslationUpdate { .. }
        | SystemsOperation::CommitTranslationUpdate { .. }
        | SystemsOperation::ActivateTranslationSpace { .. }
        | SystemsOperation::BeginTranslationEdit { .. }
        | SystemsOperation::MapTranslationFrames { .. }
        | SystemsOperation::UnmapTranslationMapping
        | SystemsOperation::CommitTranslationEdit { .. } => {
            unreachable!("translation operations are modeled by the caller")
        }
        SystemsOperation::EnterCritical { .. } | SystemsOperation::RestoreCritical { .. } => {
            unreachable!("critical operations are modeled by the caller")
        }
        SystemsOperation::SendLocalNotification
        | SystemsOperation::WaitLocalNotification
        | SystemsOperation::CompleteLocalNotification => {
            unreachable!("local-notification operations are modeled by the caller")
        }
        SystemsOperation::ObserveMonotonicClock => {
            unreachable!("monotonic-clock observations are modeled by the caller")
        }
        SystemsOperation::ObservePciConfigurationDword { .. }
        | SystemsOperation::UpdatePciConfigurationWord { .. }
        | SystemsOperation::CreateDmaBuffer { .. }
        | SystemsOperation::BindDmaBufferReference { .. }
        | SystemsOperation::StoreDmaBufferInteger { .. }
        | SystemsOperation::PrepareDmaBufferForDevice { .. }
        | SystemsOperation::CompleteDmaBufferForCpu { .. }
        | SystemsOperation::LoadDmaBufferIntegerEquals { .. }
        | SystemsOperation::CaptureEntropySample { .. }
        | SystemsOperation::AssessEntropySampleQuality { .. }
        | SystemsOperation::CreateEntropyPool { .. }
        | SystemsOperation::MixEntropyPoolSample { .. }
        | SystemsOperation::CreditEntropyPoolContribution { .. }
        | SystemsOperation::InitializeEntropyGenerator { .. }
        | SystemsOperation::ExtractEntropyGenerator { .. }
        | SystemsOperation::DeliverInitialAuxvRandom { .. }
        | SystemsOperation::ReserveInitialUserRandom { .. }
        | SystemsOperation::BindReusableUserRandom { .. }
        | SystemsOperation::ExpandSmallUserRandomRequests { .. }
        | SystemsOperation::ExpandLargeUserRandomRequests { .. }
        | SystemsOperation::PublishPciRegionDmaReference { .. }
        | SystemsOperation::AccessPciRegionRegister { .. } => {
            unreachable!("PCI configuration and region accesses are modeled by the caller")
        }
        SystemsOperation::ConstructDeadline { .. }
        | SystemsOperation::ArmDeadline
        | SystemsOperation::WaitDeadline
        | SystemsOperation::CompleteDeadline => {
            unreachable!("deadline-event operations are modeled by the caller")
        }
        SystemsOperation::CreateKernelContext
        | SystemsOperation::CreateKernelCompletion
        | SystemsOperation::CreateRunnableQueue { .. }
        | SystemsOperation::EnqueueRunnableContext
        | SystemsOperation::DequeueRunnableContext
        | SystemsOperation::TransferKernelContextUntilDeadline
        | SystemsOperation::TransferKernelContextUntilCompletion
        | SystemsOperation::TakePreemptedKernelContext
        | SystemsOperation::TransferKernelContext
        | SystemsOperation::AwaitDeadlinePreemption
        | SystemsOperation::AwaitKernelCompletion
        | SystemsOperation::SignalBlockedKernelCompletion
        | SystemsOperation::ReclaimKernelContext
        | SystemsOperation::ConsumeKernelCompletion
        | SystemsOperation::ConsumeEmptyRunnableQueue => {
            unreachable!("kernel-context operations are modeled by the caller")
        }
        SystemsOperation::PrepareUserAddressSpace
        | SystemsOperation::VerifyUserAddressSpaceMapping
        | SystemsOperation::MapUserAddressSpace
        | SystemsOperation::ReserveAnonymousUserMapping
        | SystemsOperation::StoreUserMappingByte
        | SystemsOperation::InstallUserMappingEntryTrampoline
        | SystemsOperation::ProtectUserMappingReadOnly
        | SystemsOperation::PermitUserMappingExecute
        | SystemsOperation::SealUserMappingImmutableData
        | SystemsOperation::PermitUserMappingWritableData
        | SystemsOperation::LoadUserMappingByte
        | SystemsOperation::VerifyUserMappingEntryTrampoline
        | SystemsOperation::UnmapUserAddressSpace
        | SystemsOperation::ReleaseAnonymousUserMapping
        | SystemsOperation::PrepareUserProbe { .. }
        | SystemsOperation::EnterUserContext
        | SystemsOperation::CompleteUserExecution
        | SystemsOperation::ReclaimUserAddressSpace
        | SystemsOperation::CopyInitialUnameToUser
        | SystemsOperation::CopyInitialTimespecFromUser => {
            unreachable!("user-execution operations are modeled by the caller")
        }
        SystemsOperation::BootstrapAllocate { request } => {
            *region = Some(storage.allocate(*request).map_err(|code| {
                SystemsModelError::new(
                    "E-SYSTEMS-STORAGE-REQUEST",
                    format!("checked bootstrap allocation failed during modeling: {code:?}"),
                )
            })?);
            transitions.push(SystemsTransition::AllocateBootstrapRegion { request: *request });
        }
        SystemsOperation::BootstrapStoreByte {
            offset_bytes,
            value,
        } => {
            storage.store_byte(
                required_region(region.as_ref(), "store")?,
                *offset_bytes,
                *value,
            )?;
            transitions.push(SystemsTransition::StoreBootstrapByte {
                offset_bytes: *offset_bytes,
                value: *value,
            });
        }
        SystemsOperation::BootstrapLoadByteEquals {
            offset_bytes,
            expected,
            failure_message,
        } => {
            let value =
                storage.load_byte(required_region(region.as_ref(), "load")?, *offset_bytes)?;
            transitions.push(SystemsTransition::LoadBootstrapByte {
                offset_bytes: *offset_bytes,
                value,
            });
            if value != *expected {
                transitions.push(SystemsTransition::Fatal {
                    message: failure_message.clone(),
                });
                return Ok(true);
            }
        }
        SystemsOperation::BootstrapRelease => {
            storage.release(region.take().ok_or_else(|| {
                SystemsModelError::new(
                    "E-SYSTEMS-STORAGE-LIFETIME",
                    "bootstrap release requires one live region",
                )
            })?)?;
            transitions.push(SystemsTransition::ReleaseBootstrapRegion);
        }
        SystemsOperation::ConsoleWrite { .. } | SystemsOperation::DebugBreak => {
            unreachable!("non-storage operations are modeled by the caller")
        }
        SystemsOperation::AtomicWordCreate { .. }
        | SystemsOperation::AtomicCompareExchangeEquals { .. }
        | SystemsOperation::AtomicLoadEquals { .. }
        | SystemsOperation::AtomicWordEnd => {
            unreachable!("atomic operations are modeled above")
        }
    }
    Ok(false)
}

fn required_region<'a>(
    region: Option<&'a BootstrapRegion>,
    operation: &str,
) -> Result<&'a BootstrapRegion, SystemsModelError> {
    region.ok_or_else(|| {
        SystemsModelError::new(
            "E-SYSTEMS-STORAGE-LIFETIME",
            format!("bootstrap byte {operation} requires one live region"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BootstrapStoragePlacement;

    fn program(debug_disposition: SystemsDisposition) -> SystemsProgram {
        let target = SystemsTargetSelection::initial_x86_64_qemu();
        let mut bootstrap_effects = vec![
            SYSTEMS_BOOT_MEMORY_DESCRIBE.into(),
            SYSTEMS_CONSOLE_WRITE.into(),
            SYSTEMS_FATAL.into(),
            SYSTEMS_LOCAL_NOTIFICATION_SEND.into(),
            SYSTEMS_LOCAL_NOTIFICATION_WAIT.into(),
            SYSTEMS_MONOTONIC_CLOCK_NOW.into(),
            SYSTEMS_DMA_BUFFER_CREATE.into(),
            SYSTEMS_DMA_BUFFER_BIND_REFERENCE.into(),
            SYSTEMS_DMA_BUFFER_COMPLETE_CPU.into(),
            SYSTEMS_DMA_BUFFER_LOAD_INTEGER.into(),
            SYSTEMS_ENTROPY_SAMPLE_CAPTURE_DMA.into(),
            SYSTEMS_ENTROPY_SAMPLE_ASSESS_QUALITY.into(),
            SYSTEMS_ENTROPY_POOL_CREATE.into(),
            SYSTEMS_ENTROPY_POOL_MIX_SAMPLE.into(),
            SYSTEMS_ENTROPY_POOL_CREDIT_CONTRIBUTION.into(),
            SYSTEMS_ENTROPY_GENERATOR_INITIALIZE.into(),
            SYSTEMS_ENTROPY_GENERATOR_EXTRACT.into(),
            SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM.into(),
            SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM.into(),
            SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE.into(),
            SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS.into(),
            SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS.into(),
            SYSTEMS_DMA_BUFFER_PREPARE_DEVICE.into(),
            SYSTEMS_DMA_BUFFER_STORE_INTEGER.into(),
            SYSTEMS_PCI_CONFIGURATION_READ_DWORD.into(),
            SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD.into(),
            SYSTEMS_PCI_REGION_READ.into(),
            SYSTEMS_PCI_REGION_READ_EVENTUALLY.into(),
            SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE.into(),
            SYSTEMS_PCI_REGION_WRITE.into(),
            SYSTEMS_DEADLINE_AFTER.into(),
            SYSTEMS_DEBUG_BREAK.into(),
            SYSTEMS_FRAME_ALLOCATOR_CREATE.into(),
            SYSTEMS_FRAMES_ALLOCATE.into(),
            SYSTEMS_FRAMES_RELEASE.into(),
            SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE.into(),
            SYSTEMS_KERNEL_CONTEXT_CREATE.into(),
            SYSTEMS_KERNEL_COMPLETION_CREATE.into(),
            SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE.into(),
            SYSTEMS_KERNEL_COMPLETION_SIGNAL.into(),
            SYSTEMS_KERNEL_COMPLETION_CONSUME.into(),
            SYSTEMS_KERNEL_CONTEXT_TRANSFER.into(),
            SYSTEMS_KERNEL_CONTEXT_TRANSFER_UNTIL_DEADLINE.into(),
            SYSTEMS_KERNEL_CONTEXT_TAKE_PREEMPTED.into(),
            SYSTEMS_KERNEL_CONTEXT_RECLAIM.into(),
            SYSTEMS_KERNEL_RUNNABLE_QUEUE_CREATE.into(),
            SYSTEMS_KERNEL_RUNNABLE_QUEUE_ENQUEUE.into(),
            SYSTEMS_KERNEL_RUNNABLE_QUEUE_DEQUEUE.into(),
            SYSTEMS_KERNEL_RUNNABLE_QUEUE_CONSUME.into(),
            SYSTEMS_USER_ADDRESS_SPACE_PREPARE.into(),
            SYSTEMS_USER_ADDRESS_SPACE_RECLAIM.into(),
            SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS.into(),
            SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS.into(),
            SYSTEMS_USER_ADDRESS_SPACE_MAP.into(),
            SYSTEMS_USER_MAPPING_STORE_BYTE.into(),
            SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE.into(),
            SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY.into(),
            SYSTEMS_USER_MAPPING_LOAD_BYTE.into(),
            SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE.into(),
            SYSTEMS_USER_MAPPING_PERMIT_EXECUTE.into(),
            SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA.into(),
            SYSTEMS_USER_ADDRESS_SPACE_UNMAP.into(),
            SYSTEMS_USER_CONTEXT_PREPARE.into(),
            SYSTEMS_USER_CONTEXT_ENTER.into(),
            SYSTEMS_USER_CONTEXT_COMPLETE.into(),
        ];
        bootstrap_effects.sort();
        let mut local_notification_effects = vec![
            SYSTEMS_RESUME_LOCAL_NOTIFICATION.into(),
            SYSTEMS_LOCAL_NOTIFICATION_COMPLETE.into(),
        ];
        local_notification_effects.sort();
        let mut deadline_effects = vec![
            SYSTEMS_DEADLINE_COMPLETE.into(),
            SYSTEMS_KERNEL_CONTEXT_PREEMPT_CURRENT.into(),
        ];
        deadline_effects.sort();
        let mut kernel_thread_effects = vec![
            SYSTEMS_CONSOLE_WRITE.into(),
            SYSTEMS_KERNEL_CONTEXT_AWAIT_DEADLINE_PREEMPTION.into(),
            SYSTEMS_KERNEL_CONTEXT_RETIRE.into(),
        ];
        kernel_thread_effects.sort();
        let mut completion_thread_effects = vec![
            SYSTEMS_CONSOLE_WRITE.into(),
            SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE.into(),
            SYSTEMS_KERNEL_CONTEXT_RETIRE.into(),
        ];
        completion_thread_effects.sort();
        let mut terminal_thread_effects = vec![
            SYSTEMS_CONSOLE_WRITE.into(),
            SYSTEMS_KERNEL_CONTEXT_RETIRE.into(),
        ];
        terminal_thread_effects.sort();
        SystemsProgram {
            target,
            bootstrap_storage: BootstrapStorageDescriptor {
                capacity_bytes: 65_536,
                alignment_bytes: 4096,
            },
            bootstrap: SystemsEntry {
                kind: SystemsEntryKind::Bootstrap,
                handler: SystemsHandler {
                    name: "boot".into(),
                    context: SystemsContextKind::Bootstrap,
                    operations: vec![
                        SystemsOperation::DescribeBootMemory {
                            failure_message: "memory description failed".into(),
                        },
                        SystemsOperation::CreateFrameAllocator {
                            failure_message: "frame allocator failed".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "boot".into(),
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [0],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [1],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [2],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [3],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [4],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [5],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [6],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [7],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [8],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [9],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [10],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [11],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [12],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [13],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [14],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [15],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [16],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [17],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [18],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [19],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [20],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [21],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [22],
                        },
                        SystemsOperation::ObservePciConfigurationDword {
                            request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
                                [23],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                                [0],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                                [1],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                                [2],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                                [3],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                                [4],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                                [5],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                                [6],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                                [7],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                                [8],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                                [9],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                                [10],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                                [11],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                                [12],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                                [13],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                                [14],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
                                [15],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request:
                                PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY[0],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request:
                                PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY[1],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request:
                                PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY[2],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request:
                                PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY[3],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request:
                                PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY[4],
                        },
                        SystemsOperation::UpdatePciConfigurationWord {
                            request:
                                PciConfigurationWordUpdateRequest::INITIAL_Q35_VIRTIO_RNG_BUS_MASTER,
                        },
                        SystemsOperation::CreateDmaBuffer {
                            request: DmaBufferRequest::INITIAL_Q35_VIRTIO_RNG[0],
                        },
                        SystemsOperation::CreateDmaBuffer {
                            request: DmaBufferRequest::INITIAL_Q35_VIRTIO_RNG[1],
                        },
                        SystemsOperation::BindDmaBufferReference {
                            request: DmaBufferReferenceRequest::INITIAL_Q35_VIRTIO_RNG_DESCRIPTOR,
                        },
                        SystemsOperation::StoreDmaBufferInteger {
                            request:
                                DmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING[0],
                        },
                        SystemsOperation::StoreDmaBufferInteger {
                            request:
                                DmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING[1],
                        },
                        SystemsOperation::StoreDmaBufferInteger {
                            request:
                                DmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING[2],
                        },
                        SystemsOperation::StoreDmaBufferInteger {
                            request:
                                DmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING[3],
                        },
                        SystemsOperation::PrepareDmaBufferForDevice {
                            request: DmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0],
                        },
                        SystemsOperation::PrepareDmaBufferForDevice {
                            request: DmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[1],
                        },
                        SystemsOperation::PublishPciRegionDmaReference {
                            request:
                                PciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
                                    [0],
                        },
                        SystemsOperation::PublishPciRegionDmaReference {
                            request:
                                PciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
                                    [1],
                        },
                        SystemsOperation::PublishPciRegionDmaReference {
                            request:
                                PciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
                                    [2],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request:
                                PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION[0],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request:
                                PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION[1],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request:
                                PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION[2],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request:
                                PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION[3],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request:
                                PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION
                                    [0],
                        },
                        SystemsOperation::AccessPciRegionRegister {
                            request:
                                PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL
                                    [0],
                        },
                        SystemsOperation::CompleteDmaBufferForCpu {
                            request: DmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[1],
                        },
                        SystemsOperation::LoadDmaBufferIntegerEquals {
                            request: DmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING
                                [0],
                        },
                        SystemsOperation::LoadDmaBufferIntegerEquals {
                            request: DmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING
                                [1],
                        },
                        SystemsOperation::LoadDmaBufferIntegerEquals {
                            request: DmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING
                                [2],
                        },
                        SystemsOperation::LoadDmaBufferIntegerEquals {
                            request: DmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING
                                [3],
                        },
                        SystemsOperation::CompleteDmaBufferForCpu {
                            request: DmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0],
                        },
                        SystemsOperation::CaptureEntropySample {
                            request: EntropySampleCaptureRequest::INITIAL_Q35_VIRTIO_RNG,
                        },
                        SystemsOperation::AssessEntropySampleQuality {
                            request: EntropySampleQualityRequest::INITIAL_Q35_VIRTIO_RNG,
                        },
                        SystemsOperation::CreateEntropyPool {
                            request: EntropyPoolRequest::INITIAL_Q35,
                        },
                        SystemsOperation::MixEntropyPoolSample {
                            request: EntropyPoolMixRequest::INITIAL_Q35,
                        },
                        SystemsOperation::CreditEntropyPoolContribution {
                            request: EntropyPoolCreditRequest::INITIAL_Q35,
                        },
                        SystemsOperation::InitializeEntropyGenerator {
                            request: EntropyGeneratorRequest::INITIAL_Q35,
                        },
                        SystemsOperation::ExtractEntropyGenerator {
                            request: EntropyGeneratorExtractRequest::INITIAL_Q35,
                        },
                        SystemsOperation::DebugBreak,
                        SystemsOperation::SendLocalNotification,
                        SystemsOperation::WaitLocalNotification,
                        SystemsOperation::ObserveMonotonicClock,
                        SystemsOperation::ObserveMonotonicClock,
                        SystemsOperation::ConstructDeadline {
                            duration_nanoseconds: 1_000_000,
                        },
                        SystemsOperation::CreateKernelCompletion,
                        SystemsOperation::BootstrapAllocate {
                            request: BootstrapStorageRequest {
                                byte_count: 16_384,
                                alignment_bytes: 16,
                                placement: BootstrapStoragePlacement::BootstrapReclaimable,
                            },
                        },
                        SystemsOperation::CreateKernelContext,
                        SystemsOperation::BootstrapAllocate {
                            request: BootstrapStorageRequest {
                                byte_count: 16_384,
                                alignment_bytes: 16,
                                placement: BootstrapStoragePlacement::BootstrapReclaimable,
                            },
                        },
                        SystemsOperation::CreateKernelContext,
                        SystemsOperation::BootstrapAllocate {
                            request: BootstrapStorageRequest {
                                byte_count: 16_384,
                                alignment_bytes: 16,
                                placement: BootstrapStoragePlacement::BootstrapReclaimable,
                            },
                        },
                        SystemsOperation::CreateKernelContext,
                        SystemsOperation::CreateRunnableQueue { capacity: 3 },
                        SystemsOperation::EnqueueRunnableContext,
                        SystemsOperation::EnqueueRunnableContext,
                        SystemsOperation::EnqueueRunnableContext,
                        SystemsOperation::DequeueRunnableContext,
                        SystemsOperation::TransferKernelContextUntilDeadline,
                        SystemsOperation::TakePreemptedKernelContext,
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_DEADLINE_PREEMPTED".into(),
                        },
                        SystemsOperation::EnqueueRunnableContext,
                        SystemsOperation::DequeueRunnableContext,
                        SystemsOperation::TransferKernelContextUntilCompletion,
                        SystemsOperation::DequeueRunnableContext,
                        SystemsOperation::TransferKernelContext,
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_CONTEXT_TERMINAL_RETIRED".into(),
                        },
                        SystemsOperation::ReclaimKernelContext,
                        SystemsOperation::SignalBlockedKernelCompletion,
                        SystemsOperation::EnqueueRunnableContext,
                        SystemsOperation::DequeueRunnableContext,
                        SystemsOperation::TransferKernelContext,
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_RETIRED".into(),
                        },
                        SystemsOperation::ReclaimKernelContext,
                        SystemsOperation::DequeueRunnableContext,
                        SystemsOperation::TransferKernelContext,
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_RETIRED".into(),
                        },
                        SystemsOperation::ReclaimKernelContext,
                        SystemsOperation::ConsumeKernelCompletion,
                        SystemsOperation::ConsumeEmptyRunnableQueue,
                        SystemsOperation::AllocatePhysicalFrames {
                            request: INITIAL_MULTI_FRAME_REQUEST,
                            failure_message: "user backing allocation failed".into(),
                        },
                        SystemsOperation::PrepareUserAddressSpace,
                        SystemsOperation::AllocatePhysicalFrames {
                            request: INITIAL_TWO_FRAME_REQUEST,
                            failure_message: "disjoint frame allocation failed".into(),
                        },
                        SystemsOperation::MapUserAddressSpace,
                        SystemsOperation::InstallUserMappingEntryTrampoline,
                        SystemsOperation::ProtectUserMappingReadOnly,
                        SystemsOperation::VerifyUserMappingEntryTrampoline,
                        SystemsOperation::PermitUserMappingExecute,
                        SystemsOperation::AllocatePhysicalFrames {
                            request: INITIAL_SINGLE_FRAME_REQUEST,
                            failure_message: "data frame allocation failed".into(),
                        },
                        SystemsOperation::MapUserAddressSpace,
                        SystemsOperation::StoreUserMappingByte,
                        SystemsOperation::ProtectUserMappingReadOnly,
                        SystemsOperation::LoadUserMappingByte,
                        SystemsOperation::PermitUserMappingWritableData,
                        SystemsOperation::AllocatePhysicalFrames {
                            request: POST_RANDOM_ANONYMOUS_EXTENT_REQUEST,
                            failure_message: "anonymous frame allocation failed".into(),
                        },
                        SystemsOperation::ReserveAnonymousUserMapping,
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_IMAGE_READY".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_BOOT_RESOURCE_OK".into(),
                        },
                        SystemsOperation::PrepareUserProbe {
                            process_identity: 1,
                        },
                        SystemsOperation::DeliverInitialAuxvRandom {
                            request: EntropyOutputDeliveryRequest::INITIAL_AUXV_RANDOM,
                        },
                        SystemsOperation::ExtractEntropyGenerator {
                            request: EntropyGeneratorExtractRequest::INITIAL_Q35_GETRANDOM,
                        },
                        SystemsOperation::ReserveInitialUserRandom {
                            request: EntropyOutputReservationRequest::INITIAL_GETRANDOM,
                        },
                        SystemsOperation::BindReusableUserRandom {
                            request: EntropyUserRandomServiceRequest::INITIAL_GETRANDOM,
                        },
                        SystemsOperation::ExpandSmallUserRandomRequests {
                            request: EntropyUserRandomSmallRequest::INITIAL_GETRANDOM,
                        },
                        SystemsOperation::ExpandLargeUserRandomRequests {
                            request: EntropyUserRandomSmallRequest::LARGE_GETRANDOM,
                        },
                        SystemsOperation::EnterUserContext,
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_IMAGE_MULTIPAGE_EXEC_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_GETRANDOM_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_GETRANDOM_REPEATED_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_GETRANDOM_FULL_OUTPUT_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_GETRANDOM_SMALL_REQUEST_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_GETRANDOM_LARGE_REQUEST_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_READ_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_CLOSE_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_DUP_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_FCNTL_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_FSTAT_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_TCGETS_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_WINSIZE_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_LSEEK_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_IMAGE_ENTRY_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_IMAGE_STACK_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_IMAGE_AUXV_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_IMAGE_ELF_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_COPY_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_IMAGE_RUNTIME_EXTENT_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_IMAGE_RUNTIME_ENTRY_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_IMAGE_RUNTIME_DATA_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_IMAGE_BSS_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_IMAGE_DATA_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_IMAGE_COPY_TO_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_IMAGE_COPY_FROM_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_IMAGE_SYSCALL_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_IMAGE_RETAINED".into(),
                        },
                        SystemsOperation::ReleaseAnonymousUserMapping,
                        SystemsOperation::ReleasePhysicalFrames,
                        SystemsOperation::UnmapUserAddressSpace,
                        SystemsOperation::ReleasePhysicalFrames,
                        SystemsOperation::UnmapUserAddressSpace,
                        SystemsOperation::ReleasePhysicalFrames,
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_MAPPING_OK".into(),
                        },
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_FRAME_DISJOINT_OK".into(),
                        },
                        SystemsOperation::CompleteUserExecution,
                        SystemsOperation::ReclaimUserAddressSpace,
                        SystemsOperation::ReleasePhysicalFrames,
                    ],
                    disposition: SystemsDisposition::Fatal {
                        message: "done".into(),
                    },
                    effects: bootstrap_effects,
                },
            },
            debug_break: SystemsEntry {
                kind: SystemsEntryKind::SynchronousExceptionDebugBreak,
                handler: SystemsHandler {
                    name: "debug".into(),
                    context: SystemsContextKind::DebugBreak,
                    operations: Vec::new(),
                    effects: vec![
                        debug_disposition
                            .semantic_identity(SystemsContextKind::DebugBreak)
                            .into(),
                    ],
                    disposition: debug_disposition,
                },
            },
            local_notification: SystemsEntry {
                kind: SystemsEntryKind::ExternalInterruptLocalNotification,
                handler: SystemsHandler {
                    name: "local-notification".into(),
                    context: SystemsContextKind::LocalNotificationInterrupt,
                    operations: vec![SystemsOperation::CompleteLocalNotification],
                    disposition: SystemsDisposition::Resume,
                    effects: local_notification_effects,
                },
            },
            deadline_notification: SystemsEntry {
                kind: SystemsEntryKind::ExternalInterruptDeadline,
                handler: SystemsHandler {
                    name: "deadline-notification".into(),
                    context: SystemsContextKind::DeadlineInterrupt,
                    operations: vec![SystemsOperation::CompleteDeadline],
                    disposition: SystemsDisposition::PreemptKernelContext,
                    effects: deadline_effects,
                },
            },
            kernel_thread: SystemsEntry {
                kind: SystemsEntryKind::ResumedKernelThread,
                handler: SystemsHandler {
                    name: "kernel-thread".into(),
                    context: SystemsContextKind::KernelThread,
                    operations: vec![
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_ENTERED".into(),
                        },
                        SystemsOperation::AwaitDeadlinePreemption,
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_RESUMED".into(),
                        },
                    ],
                    disposition: SystemsDisposition::RetireKernelContextToCaller,
                    effects: kernel_thread_effects,
                },
            },
            completion_thread: SystemsEntry {
                kind: SystemsEntryKind::ResumedKernelThread,
                handler: SystemsHandler {
                    name: "completion-thread".into(),
                    context: SystemsContextKind::KernelThread,
                    operations: vec![
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_ENTERED".into(),
                        },
                        SystemsOperation::AwaitKernelCompletion,
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_RESUMED".into(),
                        },
                    ],
                    disposition: SystemsDisposition::RetireKernelContextToCaller,
                    effects: completion_thread_effects,
                },
            },
            terminal_thread: SystemsEntry {
                kind: SystemsEntryKind::ResumedKernelThread,
                handler: SystemsHandler {
                    name: "terminal-thread".into(),
                    context: SystemsContextKind::KernelThread,
                    operations: vec![SystemsOperation::ConsoleWrite {
                        text: "TOPAL_KERNEL_CONTEXT_TERMINAL_ENTERED".into(),
                    }],
                    disposition: SystemsDisposition::RetireKernelContextToCaller,
                    effects: terminal_thread_effects,
                },
            },
            user_syscall: SystemsEntry {
                kind: SystemsEntryKind::UserSyscall,
                handler: SystemsHandler {
                    name: "user-syscall".into(),
                    context: SystemsContextKind::UserSyscall,
                    operations: vec![
                        SystemsOperation::CopyInitialUnameToUser,
                        SystemsOperation::CopyInitialTimespecFromUser,
                    ],
                    disposition: SystemsDisposition::DispatchInitialUserSyscall {
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
                    },
                    effects: vec![
                        SYSTEMS_USER_SYSCALL_DISPATCH.into(),
                        SYSTEMS_USER_COPY_FROM.into(),
                        SYSTEMS_USER_COPY_TO.into(),
                    ],
                },
            },
        }
    }

    fn refresh_bootstrap_effects(program: &mut SystemsProgram) {
        let mut effects = program
            .bootstrap
            .handler
            .operations
            .iter()
            .map(|operation| operation.semantic_identity().to_owned())
            .collect::<Vec<_>>();
        effects.push(
            program
                .bootstrap
                .handler
                .disposition
                .semantic_identity(SystemsContextKind::Bootstrap)
                .to_owned(),
        );
        effects.sort();
        effects.dedup();
        program.bootstrap.handler.effects = effects;
    }

    fn remove_context_lifecycle(program: &mut SystemsProgram) {
        program
            .bootstrap
            .handler
            .operations
            .retain(|operation| match operation {
                SystemsOperation::ExtractEntropyGenerator { request }
                    if *request == EntropyGeneratorExtractRequest::INITIAL_Q35_GETRANDOM =>
                {
                    false
                }
                SystemsOperation::CreateKernelContext
                | SystemsOperation::CreateKernelCompletion
                | SystemsOperation::CreateRunnableQueue { .. }
                | SystemsOperation::EnqueueRunnableContext
                | SystemsOperation::DequeueRunnableContext
                | SystemsOperation::TransferKernelContextUntilDeadline
                | SystemsOperation::TransferKernelContextUntilCompletion
                | SystemsOperation::TakePreemptedKernelContext
                | SystemsOperation::SignalBlockedKernelCompletion
                | SystemsOperation::TransferKernelContext
                | SystemsOperation::ReclaimKernelContext
                | SystemsOperation::ConsumeKernelCompletion
                | SystemsOperation::ConsumeEmptyRunnableQueue
                | SystemsOperation::PrepareUserAddressSpace
                | SystemsOperation::VerifyUserAddressSpaceMapping
                | SystemsOperation::MapUserAddressSpace
                | SystemsOperation::ReserveAnonymousUserMapping
                | SystemsOperation::StoreUserMappingByte
                | SystemsOperation::InstallUserMappingEntryTrampoline
                | SystemsOperation::ProtectUserMappingReadOnly
                | SystemsOperation::PermitUserMappingExecute
                | SystemsOperation::SealUserMappingImmutableData
                | SystemsOperation::PermitUserMappingWritableData
                | SystemsOperation::LoadUserMappingByte
                | SystemsOperation::VerifyUserMappingEntryTrampoline
                | SystemsOperation::UnmapUserAddressSpace
                | SystemsOperation::ReleaseAnonymousUserMapping
                | SystemsOperation::PrepareUserProbe { .. }
                | SystemsOperation::DeliverInitialAuxvRandom { .. }
                | SystemsOperation::ReserveInitialUserRandom { .. }
                | SystemsOperation::BindReusableUserRandom { .. }
                | SystemsOperation::ExpandSmallUserRandomRequests { .. }
                | SystemsOperation::ExpandLargeUserRandomRequests { .. }
                | SystemsOperation::EnterUserContext
                | SystemsOperation::CompleteUserExecution
                | SystemsOperation::ReclaimUserAddressSpace
                | SystemsOperation::AllocatePhysicalFrames { .. }
                | SystemsOperation::ReleasePhysicalFrames => false,
                SystemsOperation::ConsoleWrite { text }
                    if text.starts_with("TOPAL_KERNEL_CONTEXT_")
                        || text == "TOPAL_KERNEL_FRAME_DISJOINT_OK"
                        || text == "TOPAL_KERNEL_USER_MAPPING_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_READY"
                        || text == "TOPAL_KERNEL_BOOT_RESOURCE_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_MULTIPAGE_EXEC_OK"
                        || text == "TOPAL_KERNEL_USER_READ_OK"
                        || text == "TOPAL_KERNEL_USER_CLOSE_OK"
                        || text == "TOPAL_KERNEL_USER_DUP_OK"
                        || text == "TOPAL_KERNEL_USER_FCNTL_OK"
                        || text == "TOPAL_KERNEL_USER_FSTAT_OK"
                        || text == "TOPAL_KERNEL_USER_TCGETS_OK"
                        || text == "TOPAL_KERNEL_USER_WINSIZE_OK"
                        || text == "TOPAL_KERNEL_USER_LSEEK_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_ENTRY_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_STACK_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_AUXV_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_ELF_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_COPY_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_RUNTIME_EXTENT_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_RUNTIME_ENTRY_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_RUNTIME_DATA_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_BSS_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_DATA_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_COPY_TO_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_COPY_FROM_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_SYSCALL_OK"
                        || text == "TOPAL_KERNEL_USER_IMAGE_RETAINED" =>
                {
                    false
                }
                SystemsOperation::BootstrapAllocate { request } if request.byte_count == 16_384 => {
                    false
                }
                _ => true,
            });
        refresh_bootstrap_effects(program);
    }

    #[test]
    fn resumed_debug_break_returns_to_bootstrap_before_fatal() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-OBSERVATION-001.
        let transitions = model_systems_transitions(&program(SystemsDisposition::Resume))
            .unwrap()
            .into_iter()
            .filter(|transition| {
                !matches!(
                    transition,
                    SystemsTransition::ObservePciConfigurationDword { .. }
                        | SystemsTransition::AccessPciRegionRegister { .. }
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            transitions,
            [
                SystemsTransition::ProvisionBootstrapStorage {
                    capacity_bytes: 65_536,
                    alignment_bytes: 4096,
                },
                SystemsTransition::EnterBootstrap,
                SystemsTransition::DescribeBootMemory,
                SystemsTransition::CreateFrameAllocator,
                SystemsTransition::ConsoleWrite {
                    text: "boot".into(),
                },
                SystemsTransition::UpdatePciConfigurationWord {
                    transaction_identity: 1,
                    request: PciConfigurationWordUpdateRequest::INITIAL_Q35_VIRTIO_RNG_BUS_MASTER,
                },
                SystemsTransition::CreateDmaBuffer {
                    request: DmaBufferRequest::INITIAL_Q35_VIRTIO_RNG[0],
                },
                SystemsTransition::CreateDmaBuffer {
                    request: DmaBufferRequest::INITIAL_Q35_VIRTIO_RNG[1],
                },
                SystemsTransition::BindDmaBufferReference {
                    request: DmaBufferReferenceRequest::INITIAL_Q35_VIRTIO_RNG_DESCRIPTOR,
                },
                SystemsTransition::StoreDmaBufferInteger {
                    request: DmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING[0],
                },
                SystemsTransition::StoreDmaBufferInteger {
                    request: DmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING[1],
                },
                SystemsTransition::StoreDmaBufferInteger {
                    request: DmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING[2],
                },
                SystemsTransition::StoreDmaBufferInteger {
                    request: DmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING[3],
                },
                SystemsTransition::PrepareDmaBufferForDevice {
                    request: DmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0],
                },
                SystemsTransition::PrepareDmaBufferForDevice {
                    request: DmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[1],
                },
                SystemsTransition::PublishPciRegionDmaReference {
                    transaction_identity: 22,
                    request: PciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
                        [0],
                },
                SystemsTransition::PublishPciRegionDmaReference {
                    transaction_identity: 23,
                    request: PciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
                        [1],
                },
                SystemsTransition::PublishPciRegionDmaReference {
                    transaction_identity: 24,
                    request: PciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
                        [2],
                },
                SystemsTransition::CompleteDmaBufferForCpu {
                    request: DmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[1],
                },
                SystemsTransition::LoadDmaBufferIntegerEquals {
                    request: DmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING[0],
                },
                SystemsTransition::LoadDmaBufferIntegerEquals {
                    request: DmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING[1],
                },
                SystemsTransition::LoadDmaBufferIntegerEquals {
                    request: DmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING[2],
                },
                SystemsTransition::LoadDmaBufferIntegerEquals {
                    request: DmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING[3],
                },
                SystemsTransition::CompleteDmaBufferForCpu {
                    request: DmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0],
                },
                SystemsTransition::CaptureEntropySample {
                    request: EntropySampleCaptureRequest::INITIAL_Q35_VIRTIO_RNG,
                },
                SystemsTransition::AssessEntropySampleQuality {
                    request: EntropySampleQualityRequest::INITIAL_Q35_VIRTIO_RNG,
                },
                SystemsTransition::CreateEntropyPool {
                    request: EntropyPoolRequest::INITIAL_Q35,
                },
                SystemsTransition::MixEntropyPoolSample {
                    request: EntropyPoolMixRequest::INITIAL_Q35,
                },
                SystemsTransition::CreditEntropyPoolContribution {
                    request: EntropyPoolCreditRequest::INITIAL_Q35,
                },
                SystemsTransition::InitializeEntropyGenerator {
                    request: EntropyGeneratorRequest::INITIAL_Q35,
                },
                SystemsTransition::ExtractEntropyGenerator {
                    request: EntropyGeneratorExtractRequest::INITIAL_Q35,
                },
                SystemsTransition::ObserveDebugBreak,
                SystemsTransition::EnterDebugBreak,
                SystemsTransition::ResumeDebugBreak,
                SystemsTransition::SendLocalNotification { event_identity: 1 },
                SystemsTransition::BeginLocalNotificationWait { event_identity: 1 },
                SystemsTransition::ObserveLocalNotification { event_identity: 1 },
                SystemsTransition::EnterLocalNotificationInterrupt { event_identity: 1 },
                SystemsTransition::CompleteLocalNotificationInterrupt { event_identity: 1 },
                SystemsTransition::ResumeLocalNotificationInterrupt { event_identity: 1 },
                SystemsTransition::EndLocalNotificationWait { event_identity: 1 },
                SystemsTransition::ObserveMonotonicClock {
                    observation_identity: 1,
                },
                SystemsTransition::ObserveMonotonicClock {
                    observation_identity: 2,
                },
                SystemsTransition::ConstructDeadline {
                    source_observation_identity: 2,
                    duration_nanoseconds: 1_000_000,
                },
                SystemsTransition::CreateKernelCompletion {
                    completion_identity: 1,
                },
                SystemsTransition::AllocateBootstrapRegion {
                    request: BootstrapStorageRequest {
                        byte_count: 16_384,
                        alignment_bytes: 16,
                        placement: BootstrapStoragePlacement::BootstrapReclaimable,
                    },
                },
                SystemsTransition::CreateKernelContext {
                    context_identity: 1,
                },
                SystemsTransition::AllocateBootstrapRegion {
                    request: BootstrapStorageRequest {
                        byte_count: 16_384,
                        alignment_bytes: 16,
                        placement: BootstrapStoragePlacement::BootstrapReclaimable,
                    },
                },
                SystemsTransition::CreateKernelContext {
                    context_identity: 2,
                },
                SystemsTransition::AllocateBootstrapRegion {
                    request: BootstrapStorageRequest {
                        byte_count: 16_384,
                        alignment_bytes: 16,
                        placement: BootstrapStoragePlacement::BootstrapReclaimable,
                    },
                },
                SystemsTransition::CreateKernelContext {
                    context_identity: 3,
                },
                SystemsTransition::CreateRunnableQueue { capacity: 3 },
                SystemsTransition::EnqueueRunnableContext {
                    context_identity: 1,
                },
                SystemsTransition::EnqueueRunnableContext {
                    context_identity: 2,
                },
                SystemsTransition::EnqueueRunnableContext {
                    context_identity: 3,
                },
                SystemsTransition::DequeueRunnableContext {
                    context_identity: 1,
                },
                SystemsTransition::ArmDeadline { event_identity: 1 },
                SystemsTransition::TransferKernelContextUntilDeadline {
                    context_identity: 1,
                    caller_identity: 1,
                    event_identity: 1,
                },
                SystemsTransition::EnterKernelThread {
                    context_identity: 1,
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_ENTERED".into(),
                },
                SystemsTransition::AwaitDeadlinePreemption {
                    context_identity: 1,
                },
                SystemsTransition::ObserveDeadline { event_identity: 1 },
                SystemsTransition::EnterDeadlineInterrupt { event_identity: 1 },
                SystemsTransition::CompleteDeadlineInterrupt { event_identity: 1 },
                SystemsTransition::PreemptKernelContext {
                    event_identity: 1,
                    context_identity: 1,
                    caller_identity: 1,
                },
                SystemsTransition::ResumePreemptionDispatcher { caller_identity: 1 },
                SystemsTransition::TakePreemptedKernelContext {
                    context_identity: 1,
                    event_identity: 1,
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_DEADLINE_PREEMPTED".into(),
                },
                SystemsTransition::EnqueueRunnableContext {
                    context_identity: 1,
                },
                SystemsTransition::DequeueRunnableContext {
                    context_identity: 2,
                },
                SystemsTransition::TransferKernelContextUntilCompletion {
                    context_identity: 2,
                    caller_identity: 2,
                    completion_identity: 1,
                },
                SystemsTransition::EnterKernelThread {
                    context_identity: 2,
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_ENTERED".into(),
                },
                SystemsTransition::AwaitKernelCompletion {
                    context_identity: 2,
                    completion_identity: 1,
                },
                SystemsTransition::BlockKernelContext {
                    context_identity: 2,
                    caller_identity: 2,
                    completion_identity: 1,
                },
                SystemsTransition::ResumeCompletionDispatcher { caller_identity: 2 },
                SystemsTransition::DequeueRunnableContext {
                    context_identity: 3,
                },
                SystemsTransition::TransferKernelContext {
                    context_identity: 3,
                    caller_identity: 3,
                },
                SystemsTransition::EnterKernelThread {
                    context_identity: 3,
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_CONTEXT_TERMINAL_ENTERED".into(),
                },
                SystemsTransition::RetireKernelContextToCaller {
                    context_identity: 3,
                    caller_identity: 3,
                },
                SystemsTransition::ResumeKernelContextCaller { caller_identity: 3 },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_CONTEXT_TERMINAL_RETIRED".into(),
                },
                SystemsTransition::ReclaimKernelContext {
                    context_identity: 3,
                },
                SystemsTransition::SignalBlockedKernelCompletion {
                    context_identity: 2,
                    caller_identity: 2,
                    completion_identity: 1,
                },
                SystemsTransition::EnqueueRunnableContext {
                    context_identity: 2,
                },
                SystemsTransition::DequeueRunnableContext {
                    context_identity: 1,
                },
                SystemsTransition::TransferKernelContext {
                    context_identity: 1,
                    caller_identity: 4,
                },
                SystemsTransition::RestorePreemptedKernelContext {
                    context_identity: 1,
                    caller_identity: 4,
                    event_identity: 1,
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_RESUMED".into(),
                },
                SystemsTransition::RetireKernelContextToCaller {
                    context_identity: 1,
                    caller_identity: 4,
                },
                SystemsTransition::ResumeKernelContextCaller { caller_identity: 4 },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_RETIRED".into(),
                },
                SystemsTransition::ReclaimKernelContext {
                    context_identity: 1,
                },
                SystemsTransition::DequeueRunnableContext {
                    context_identity: 2,
                },
                SystemsTransition::TransferKernelContext {
                    context_identity: 2,
                    caller_identity: 5,
                },
                SystemsTransition::RestoreBlockedKernelContext {
                    context_identity: 2,
                    caller_identity: 5,
                    completion_identity: 1,
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_RESUMED".into(),
                },
                SystemsTransition::RetireKernelContextToCaller {
                    context_identity: 2,
                    caller_identity: 5,
                },
                SystemsTransition::ResumeKernelContextCaller { caller_identity: 5 },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_RETIRED".into(),
                },
                SystemsTransition::ReclaimKernelContext {
                    context_identity: 2,
                },
                SystemsTransition::ConsumeKernelCompletion {
                    completion_identity: 1,
                },
                SystemsTransition::ConsumeEmptyRunnableQueue,
                SystemsTransition::AllocatePhysicalFrames {
                    request: INITIAL_MULTI_FRAME_REQUEST,
                },
                SystemsTransition::PrepareUserAddressSpace {
                    address_space_identity: 1,
                },
                SystemsTransition::AllocatePhysicalFrames {
                    request: INITIAL_TWO_FRAME_REQUEST,
                },
                SystemsTransition::MapUserAddressSpace {
                    address_space_identity: 1,
                    mapping_identity: 1,
                },
                SystemsTransition::InstallUserMappingEntryTrampoline {
                    mapping_identity: 1,
                },
                SystemsTransition::ProtectUserMappingReadOnly {
                    mapping_identity: 1,
                },
                SystemsTransition::VerifyUserMappingEntryTrampoline {
                    mapping_identity: 1,
                },
                SystemsTransition::PermitUserMappingExecute {
                    mapping_identity: 1,
                },
                SystemsTransition::AllocatePhysicalFrames {
                    request: INITIAL_SINGLE_FRAME_REQUEST,
                },
                SystemsTransition::MapUserAddressSpace {
                    address_space_identity: 1,
                    mapping_identity: 2,
                },
                SystemsTransition::StoreUserMappingByte {
                    mapping_identity: 2,
                },
                SystemsTransition::ProtectUserMappingReadOnly {
                    mapping_identity: 2,
                },
                SystemsTransition::LoadUserMappingByte {
                    mapping_identity: 2,
                },
                SystemsTransition::PermitUserMappingWritableData {
                    mapping_identity: 2,
                },
                SystemsTransition::AllocatePhysicalFrames {
                    request: POST_RANDOM_ANONYMOUS_EXTENT_REQUEST,
                },
                SystemsTransition::ReserveAnonymousUserMapping {
                    address_space_identity: 1,
                    reservation_identity: 3,
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_READY".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_BOOT_RESOURCE_OK".into(),
                },
                SystemsTransition::PrepareUserProbe {
                    process_identity: 1,
                    probe_identity: 1,
                },
                SystemsTransition::DeliverInitialAuxvRandom {
                    request: EntropyOutputDeliveryRequest::INITIAL_AUXV_RANDOM,
                },
                SystemsTransition::ExtractEntropyGenerator {
                    request: EntropyGeneratorExtractRequest::INITIAL_Q35_GETRANDOM,
                },
                SystemsTransition::ReserveInitialUserRandom {
                    request: EntropyOutputReservationRequest::INITIAL_GETRANDOM,
                },
                SystemsTransition::BindReusableUserRandom {
                    request: EntropyUserRandomServiceRequest::INITIAL_GETRANDOM,
                },
                SystemsTransition::ExpandSmallUserRandomRequests {
                    request: EntropyUserRandomSmallRequest::INITIAL_GETRANDOM,
                },
                SystemsTransition::ExpandLargeUserRandomRequests {
                    request: EntropyUserRandomSmallRequest::LARGE_GETRANDOM,
                },
                SystemsTransition::EnterUserExecution {
                    process_identity: 1,
                    kernel_continuation_identity: 1,
                },
                SystemsTransition::EnterUserSyscall {
                    entry_identity: 1,
                    number: 39,
                    arguments: [0; 6],
                },
                SystemsTransition::ReturnToUser {
                    entry_identity: 1,
                    result: 1,
                },
                SystemsTransition::EnterUserSyscall {
                    entry_identity: 2,
                    number: 63,
                    arguments: [INITIAL_USER_UNAME_VALID_CANDIDATE, 0, 0, 0, 0, 0],
                },
                SystemsTransition::BeginUserTransfer {
                    entry_identity: 2,
                    transfer_identity: 1,
                    candidate: INITIAL_USER_UNAME_VALID_CANDIDATE,
                    extent: LINUX_NEW_UTSNAME_BYTES,
                },
                SystemsTransition::CopyToUserComplete {
                    transfer_identity: 1,
                    copied: LINUX_NEW_UTSNAME_BYTES,
                },
                SystemsTransition::EndUserTransfer {
                    entry_identity: 2,
                    transfer_identity: 1,
                },
                SystemsTransition::ReturnToUser {
                    entry_identity: 2,
                    result: 0,
                },
                SystemsTransition::EnterUserSyscall {
                    entry_identity: 3,
                    number: 63,
                    arguments: [INITIAL_USER_UNAME_FAULT_CANDIDATE, 0, 0, 0, 0, 0],
                },
                SystemsTransition::BeginUserTransfer {
                    entry_identity: 3,
                    transfer_identity: 2,
                    candidate: INITIAL_USER_UNAME_FAULT_CANDIDATE,
                    extent: LINUX_NEW_UTSNAME_BYTES,
                },
                SystemsTransition::CopyToUserFault {
                    transfer_identity: 2,
                    copied_prefix: 256,
                },
                SystemsTransition::EndUserTransfer {
                    entry_identity: 3,
                    transfer_identity: 2,
                },
                SystemsTransition::ReturnToUser {
                    entry_identity: 3,
                    result: LINUX_X86_64_EFAULT_RESULT,
                },
                SystemsTransition::EnterUserSyscall {
                    entry_identity: 4,
                    number: LINUX_X86_64_NANOSLEEP,
                    arguments: [INITIAL_USER_TIMESPEC_VALID_CANDIDATE, 0, 0, 0, 0, 0],
                },
                SystemsTransition::BeginUserTransfer {
                    entry_identity: 4,
                    transfer_identity: 3,
                    candidate: INITIAL_USER_TIMESPEC_VALID_CANDIDATE,
                    extent: LINUX_TIMESPEC_BYTES,
                },
                SystemsTransition::CopyFromUserComplete {
                    transfer_identity: 3,
                    copied: LINUX_TIMESPEC_BYTES,
                    seconds: 0,
                    nanoseconds: 0,
                },
                SystemsTransition::EndUserTransfer {
                    entry_identity: 4,
                    transfer_identity: 3,
                },
                SystemsTransition::ReturnToUser {
                    entry_identity: 4,
                    result: 0,
                },
                SystemsTransition::EnterUserSyscall {
                    entry_identity: 5,
                    number: LINUX_X86_64_NANOSLEEP,
                    arguments: [INITIAL_USER_TIMESPEC_FAULT_CANDIDATE, 0, 0, 0, 0, 0],
                },
                SystemsTransition::BeginUserTransfer {
                    entry_identity: 5,
                    transfer_identity: 4,
                    candidate: INITIAL_USER_TIMESPEC_FAULT_CANDIDATE,
                    extent: LINUX_TIMESPEC_BYTES,
                },
                SystemsTransition::CopyFromUserFault {
                    transfer_identity: 4,
                    copied_prefix: 8,
                },
                SystemsTransition::EndUserTransfer {
                    entry_identity: 5,
                    transfer_identity: 4,
                },
                SystemsTransition::ReturnToUser {
                    entry_identity: 5,
                    result: LINUX_X86_64_EFAULT_RESULT,
                },
                SystemsTransition::EnterUserSyscall {
                    entry_identity: 6,
                    number: 231,
                    arguments: [0; 6],
                },
                SystemsTransition::TerminateUser {
                    entry_identity: 6,
                    status: 0,
                },
                SystemsTransition::ResumeUserKernelContinuation {
                    kernel_continuation_identity: 1,
                    status: 0,
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_MULTIPAGE_EXEC_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_GETRANDOM_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_GETRANDOM_REPEATED_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_GETRANDOM_FULL_OUTPUT_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_GETRANDOM_SMALL_REQUEST_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_GETRANDOM_LARGE_REQUEST_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_READ_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_CLOSE_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_DUP_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_FCNTL_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_FSTAT_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_TCGETS_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_WINSIZE_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_LSEEK_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_ENTRY_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_STACK_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_AUXV_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_ELF_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_COPY_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_RUNTIME_EXTENT_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_RUNTIME_ENTRY_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_RUNTIME_DATA_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_BSS_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_DATA_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_COPY_TO_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_COPY_FROM_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_SYSCALL_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_RETAINED".into(),
                },
                SystemsTransition::ReleaseAnonymousUserMapping {
                    address_space_identity: 1,
                    reservation_identity: 3,
                },
                SystemsTransition::ReleasePhysicalFrames,
                SystemsTransition::UnmapUserAddressSpace {
                    address_space_identity: 1,
                    mapping_identity: 2,
                },
                SystemsTransition::ReleasePhysicalFrames,
                SystemsTransition::UnmapUserAddressSpace {
                    address_space_identity: 1,
                    mapping_identity: 1,
                },
                SystemsTransition::ReleasePhysicalFrames,
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_MAPPING_OK".into(),
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_FRAME_DISJOINT_OK".into(),
                },
                SystemsTransition::CompleteUserExecution {
                    process_identity: 1,
                },
                SystemsTransition::ReclaimUserAddressSpace {
                    address_space_identity: 1,
                },
                SystemsTransition::ReleasePhysicalFrames,
                SystemsTransition::Fatal {
                    message: "done".into(),
                },
            ]
        );
    }

    #[test]
    fn prepared_user_space_requires_two_mapping_generations_and_released_extent() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-FRAMES-001,
        // TOPAL-SYSTEMS-CONTEXT-006.
        let mut live_at_probe = program(SystemsDisposition::Resume);
        let scratch_release = live_at_probe
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| matches!(operation, SystemsOperation::PrepareUserAddressSpace))
            .unwrap()
            + 3;
        let prepare_probe = live_at_probe
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| matches!(operation, SystemsOperation::PrepareUserProbe { .. }))
            .unwrap();
        live_at_probe
            .bootstrap
            .handler
            .operations
            .swap(scratch_release, prepare_probe);
        assert_eq!(
            validate_systems_program(&live_at_probe).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut third_extent = program(SystemsDisposition::Resume);
        let scratch_allocation = third_extent
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| matches!(operation, SystemsOperation::PrepareUserAddressSpace))
            .unwrap()
            + 2;
        third_extent.bootstrap.handler.operations.insert(
            scratch_allocation,
            SystemsOperation::AllocatePhysicalFrames {
                request: INITIAL_SINGLE_FRAME_REQUEST,
                failure_message: "third frame allocation failed".into(),
            },
        );
        assert_eq!(
            validate_systems_program(&third_extent).unwrap_err().code,
            "E-SYSTEMS-FRAMES"
        );

        let mut missing_unmap = program(SystemsDisposition::Resume);
        let unmap = missing_unmap
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| matches!(operation, SystemsOperation::UnmapUserAddressSpace))
            .unwrap();
        missing_unmap.bootstrap.handler.operations.remove(unmap);
        refresh_bootstrap_effects(&mut missing_unmap);
        assert_eq!(
            validate_systems_program(&missing_unmap).unwrap_err().code,
            "E-SYSTEMS-FRAMES"
        );

        let mut duplicate_store = program(SystemsDisposition::Resume);
        let store = duplicate_store
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| matches!(operation, SystemsOperation::StoreUserMappingByte))
            .unwrap();
        duplicate_store
            .bootstrap
            .handler
            .operations
            .insert(store + 1, SystemsOperation::StoreUserMappingByte);
        assert_eq!(
            validate_systems_program(&duplicate_store).unwrap_err().code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut missing_protection = program(SystemsDisposition::Resume);
        let protection = missing_protection
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| matches!(operation, SystemsOperation::ProtectUserMappingReadOnly))
            .unwrap();
        missing_protection
            .bootstrap
            .handler
            .operations
            .remove(protection);
        refresh_bootstrap_effects(&mut missing_protection);
        assert_eq!(
            validate_systems_program(&missing_protection)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut duplicate_protection = program(SystemsDisposition::Resume);
        let protection = duplicate_protection
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| matches!(operation, SystemsOperation::ProtectUserMappingReadOnly))
            .unwrap();
        duplicate_protection
            .bootstrap
            .handler
            .operations
            .insert(protection + 1, SystemsOperation::ProtectUserMappingReadOnly);
        assert_eq!(
            validate_systems_program(&duplicate_protection)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut reordered_access = program(SystemsDisposition::Resume);
        let store = reordered_access
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| matches!(operation, SystemsOperation::StoreUserMappingByte))
            .unwrap();
        reordered_access
            .bootstrap
            .handler
            .operations
            .swap(store, store + 1);
        assert_eq!(
            validate_systems_program(&reordered_access)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut code_promoted_to_writable_data = program(SystemsDisposition::Resume);
        let execute = code_promoted_to_writable_data
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| matches!(operation, SystemsOperation::PermitUserMappingExecute))
            .unwrap();
        code_promoted_to_writable_data.bootstrap.handler.operations[execute] =
            SystemsOperation::PermitUserMappingWritableData;
        refresh_bootstrap_effects(&mut code_promoted_to_writable_data);
        assert_eq!(
            validate_systems_program(&code_promoted_to_writable_data)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut data_promoted_to_code = program(SystemsDisposition::Resume);
        let seal = data_promoted_to_code
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(operation, SystemsOperation::PermitUserMappingWritableData)
            })
            .unwrap();
        data_promoted_to_code.bootstrap.handler.operations[seal] =
            SystemsOperation::PermitUserMappingExecute;
        refresh_bootstrap_effects(&mut data_promoted_to_code);
        assert_eq!(
            validate_systems_program(&data_promoted_to_code)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut late_image_readiness = program(SystemsDisposition::Resume);
        let readiness = late_image_readiness
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    SystemsOperation::ConsoleWrite { text }
                        if text == "TOPAL_KERNEL_USER_IMAGE_READY"
                )
            })
            .unwrap();
        late_image_readiness
            .bootstrap
            .handler
            .operations
            .swap(readiness, readiness + 1);
        assert_eq!(
            validate_systems_program(&late_image_readiness)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut duplicate_image_readiness = program(SystemsDisposition::Resume);
        let readiness = duplicate_image_readiness
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    SystemsOperation::ConsoleWrite { text }
                        if text == "TOPAL_KERNEL_USER_IMAGE_READY"
                )
            })
            .unwrap();
        duplicate_image_readiness
            .bootstrap
            .handler
            .operations
            .insert(
                readiness + 1,
                SystemsOperation::ConsoleWrite {
                    text: "TOPAL_KERNEL_USER_IMAGE_READY".into(),
                },
            );
        assert_eq!(
            validate_systems_program(&duplicate_image_readiness)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut late_initial_stack = program(SystemsDisposition::Resume);
        let initial_stack = late_initial_stack
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    SystemsOperation::ConsoleWrite { text }
                        if text == "TOPAL_KERNEL_USER_IMAGE_STACK_OK"
                )
            })
            .unwrap();
        late_initial_stack
            .bootstrap
            .handler
            .operations
            .swap(initial_stack, initial_stack + 1);
        assert_eq!(
            validate_systems_program(&late_initial_stack)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut late_initial_auxv = program(SystemsDisposition::Resume);
        let initial_auxv = late_initial_auxv
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    SystemsOperation::ConsoleWrite { text }
                        if text == "TOPAL_KERNEL_USER_IMAGE_AUXV_OK"
                )
            })
            .unwrap();
        late_initial_auxv
            .bootstrap
            .handler
            .operations
            .swap(initial_auxv, initial_auxv + 1);
        assert_eq!(
            validate_systems_program(&late_initial_auxv)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut late_initial_entry_auxv = program(SystemsDisposition::Resume);
        let initial_entry_auxv = late_initial_entry_auxv
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    SystemsOperation::ConsoleWrite { text }
                        if text == "TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_OK"
                )
            })
            .unwrap();
        late_initial_entry_auxv
            .bootstrap
            .handler
            .operations
            .swap(initial_entry_auxv, initial_entry_auxv + 1);
        assert_eq!(
            validate_systems_program(&late_initial_entry_auxv)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut late_initial_execfn_auxv = program(SystemsDisposition::Resume);
        let initial_execfn_auxv = late_initial_execfn_auxv
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    SystemsOperation::ConsoleWrite { text }
                        if text == "TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_OK"
                )
            })
            .unwrap();
        late_initial_execfn_auxv
            .bootstrap
            .handler
            .operations
            .swap(initial_execfn_auxv, initial_execfn_auxv + 1);
        assert_eq!(
            validate_systems_program(&late_initial_execfn_auxv)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut late_initial_credential_auxv = program(SystemsDisposition::Resume);
        let initial_credential_auxv = late_initial_credential_auxv
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    SystemsOperation::ConsoleWrite { text }
                        if text == "TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_OK"
                )
            })
            .unwrap();
        late_initial_credential_auxv
            .bootstrap
            .handler
            .operations
            .swap(initial_credential_auxv, initial_credential_auxv + 1);
        assert_eq!(
            validate_systems_program(&late_initial_credential_auxv)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut late_elf_identity = program(SystemsDisposition::Resume);
        let elf_identity = late_elf_identity
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    SystemsOperation::ConsoleWrite { text }
                        if text == "TOPAL_KERNEL_USER_IMAGE_ELF_OK"
                )
            })
            .unwrap();
        late_elf_identity
            .bootstrap
            .handler
            .operations
            .swap(elf_identity, elf_identity + 1);
        assert_eq!(
            validate_systems_program(&late_elf_identity)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut late_image_copy_to = program(SystemsDisposition::Resume);
        let image_copy_to = late_image_copy_to
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    SystemsOperation::ConsoleWrite { text }
                        if text == "TOPAL_KERNEL_USER_IMAGE_COPY_TO_OK"
                )
            })
            .unwrap();
        late_image_copy_to
            .bootstrap
            .handler
            .operations
            .swap(image_copy_to, image_copy_to + 1);
        assert_eq!(
            validate_systems_program(&late_image_copy_to)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut late_image_copy_from = program(SystemsDisposition::Resume);
        let image_copy_from = late_image_copy_from
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    SystemsOperation::ConsoleWrite { text }
                        if text == "TOPAL_KERNEL_USER_IMAGE_COPY_FROM_OK"
                )
            })
            .unwrap();
        late_image_copy_from
            .bootstrap
            .handler
            .operations
            .swap(image_copy_from, image_copy_from + 1);
        assert_eq!(
            validate_systems_program(&late_image_copy_from)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut late_image_syscall = program(SystemsDisposition::Resume);
        let image_syscall = late_image_syscall
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    SystemsOperation::ConsoleWrite { text }
                        if text == "TOPAL_KERNEL_USER_IMAGE_SYSCALL_OK"
                )
            })
            .unwrap();
        late_image_syscall
            .bootstrap
            .handler
            .operations
            .swap(image_syscall, image_syscall + 1);
        assert_eq!(
            validate_systems_program(&late_image_syscall)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut late_image_retention = program(SystemsDisposition::Resume);
        let retained = late_image_retention
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    SystemsOperation::ConsoleWrite { text }
                        if text == "TOPAL_KERNEL_USER_IMAGE_RETAINED"
                )
            })
            .unwrap();
        late_image_retention
            .bootstrap
            .handler
            .operations
            .swap(retained, retained + 1);
        assert_eq!(
            validate_systems_program(&late_image_retention)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut completion_before_teardown = program(SystemsDisposition::Resume);
        let completion = completion_before_teardown
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| matches!(operation, SystemsOperation::CompleteUserExecution))
            .unwrap();
        let retained = completion_before_teardown
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    SystemsOperation::ConsoleWrite { text }
                        if text == "TOPAL_KERNEL_USER_IMAGE_RETAINED"
                )
            })
            .unwrap();
        completion_before_teardown
            .bootstrap
            .handler
            .operations
            .swap(completion, retained + 1);
        assert_eq!(
            validate_systems_program(&completion_before_teardown)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );

        let mut third_generation = program(SystemsDisposition::Resume);
        let release = third_generation
            .bootstrap
            .handler
            .operations
            .iter()
            .enumerate()
            .filter(|(_, operation)| matches!(operation, SystemsOperation::UnmapUserAddressSpace))
            .nth(1)
            .map(|(index, _)| index + 1)
            .unwrap();
        third_generation.bootstrap.handler.operations.splice(
            release..release,
            [
                SystemsOperation::MapUserAddressSpace,
                SystemsOperation::StoreUserMappingByte,
                SystemsOperation::ProtectUserMappingReadOnly,
                SystemsOperation::LoadUserMappingByte,
                SystemsOperation::PermitUserMappingExecute,
                SystemsOperation::UnmapUserAddressSpace,
            ],
        );
        assert_eq!(
            validate_systems_program(&third_generation)
                .unwrap_err()
                .code,
            "E-SYSTEMS-USER-CONTEXT"
        );
    }

    #[test]
    fn local_notification_requires_matching_wait_completion_and_resume() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-LOCAL-INTERRUPT-001.
        let mut live = program(SystemsDisposition::Resume);
        remove_context_lifecycle(&mut live);
        live.bootstrap
            .handler
            .operations
            .retain(|operation| !matches!(operation, SystemsOperation::WaitLocalNotification));
        live.bootstrap
            .handler
            .effects
            .retain(|effect| effect != SYSTEMS_LOCAL_NOTIFICATION_WAIT);
        live.bootstrap
            .handler
            .operations
            .retain(|operation| !matches!(operation, SystemsOperation::ObserveMonotonicClock));
        live.bootstrap
            .handler
            .effects
            .retain(|effect| effect != SYSTEMS_MONOTONIC_CLOCK_NOW);
        live.bootstrap.handler.operations.retain(|operation| {
            !matches!(
                operation,
                SystemsOperation::ConstructDeadline { .. }
                    | SystemsOperation::ArmDeadline
                    | SystemsOperation::WaitDeadline
            )
        });
        live.bootstrap.handler.effects.retain(|effect| {
            !matches!(
                effect.as_str(),
                SYSTEMS_DEADLINE_AFTER | SYSTEMS_DEADLINE_ARM | SYSTEMS_DEADLINE_WAIT
            )
        });
        assert_eq!(
            validate_systems_program(&live).unwrap_err().code,
            "E-SYSTEMS-LOCAL-INTERRUPT-LIVE"
        );

        let mut duplicate = program(SystemsDisposition::Resume);
        duplicate
            .bootstrap
            .handler
            .operations
            .insert(5, SystemsOperation::SendLocalNotification);
        assert_eq!(
            validate_systems_program(&duplicate).unwrap_err().code,
            "E-SYSTEMS-LOCAL-INTERRUPT"
        );

        let mut incomplete = program(SystemsDisposition::Resume);
        incomplete.local_notification.handler.operations.clear();
        incomplete.local_notification.handler.effects =
            vec![SYSTEMS_RESUME_LOCAL_NOTIFICATION.into()];
        assert_eq!(
            validate_systems_program(&incomplete).unwrap_err().code,
            "E-SYSTEMS-LOCAL-INTERRUPT"
        );
    }

    #[test]
    fn monotonic_clock_requires_two_post_interrupt_observations() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-MONOTONIC-CLOCK-001.
        let mut missing = program(SystemsDisposition::Resume);
        remove_context_lifecycle(&mut missing);
        missing.bootstrap.handler.operations.retain(|operation| {
            !matches!(
                operation,
                SystemsOperation::ConstructDeadline { .. }
                    | SystemsOperation::ArmDeadline
                    | SystemsOperation::WaitDeadline
            )
        });
        let observation = missing
            .bootstrap
            .handler
            .operations
            .iter()
            .rposition(|operation| matches!(operation, SystemsOperation::ObserveMonotonicClock))
            .unwrap();
        missing.bootstrap.handler.operations.remove(observation);
        refresh_bootstrap_effects(&mut missing);
        assert_eq!(
            validate_systems_program(&missing).unwrap_err().code,
            "E-SYSTEMS-MONOTONIC-CLOCK-LIVE"
        );

        let mut early = program(SystemsDisposition::Resume);
        let observation_index = early
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| matches!(operation, SystemsOperation::ObserveMonotonicClock))
            .unwrap();
        let observation = early.bootstrap.handler.operations.remove(observation_index);
        early.bootstrap.handler.operations.insert(4, observation);
        assert_eq!(
            validate_systems_program(&early).unwrap_err().code,
            "E-SYSTEMS-MONOTONIC-CLOCK"
        );

        let mut extra = program(SystemsDisposition::Resume);
        extra
            .bootstrap
            .handler
            .operations
            .push(SystemsOperation::ObserveMonotonicClock);
        assert_eq!(
            validate_systems_program(&extra).unwrap_err().code,
            "E-SYSTEMS-MONOTONIC-CLOCK"
        );
    }

    #[test]
    fn deadline_requires_exact_construction_preemption_and_completion() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-DEADLINE-EVENT-001,
        // TOPAL-SYSTEMS-CONTEXT-004.
        let mut wrong_duration = program(SystemsDisposition::Resume);
        wrong_duration.bootstrap.handler.operations[8] = SystemsOperation::ConstructDeadline {
            duration_nanoseconds: 2_000_000,
        };
        assert_eq!(
            validate_systems_program(&wrong_duration).unwrap_err().code,
            "E-SYSTEMS-DEADLINE-EVENT"
        );

        let mut missing_preemption = program(SystemsDisposition::Resume);
        remove_context_lifecycle(&mut missing_preemption);
        assert_eq!(
            validate_systems_program(&missing_preemption)
                .unwrap_err()
                .code,
            "E-SYSTEMS-DEADLINE-EVENT-LIVE"
        );

        let mut incomplete = program(SystemsDisposition::Resume);
        incomplete.deadline_notification.handler.operations.clear();
        incomplete.deadline_notification.handler.effects =
            vec![SYSTEMS_KERNEL_CONTEXT_PREEMPT_CURRENT.into()];
        assert_eq!(
            validate_systems_program(&incomplete).unwrap_err().code,
            "E-SYSTEMS-DEADLINE-EVENT"
        );
    }

    #[test]
    fn kernel_context_requires_create_transfer_retirement_and_reclaim() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-CONTEXT-001.
        let mut missing_reclaim = program(SystemsDisposition::Resume);
        let final_reclaim = missing_reclaim
            .bootstrap
            .handler
            .operations
            .iter()
            .rposition(|operation| matches!(operation, SystemsOperation::ReclaimKernelContext))
            .unwrap();
        missing_reclaim
            .bootstrap
            .handler
            .operations
            .remove(final_reclaim);
        missing_reclaim
            .bootstrap
            .handler
            .operations
            .retain(|operation| {
                !matches!(
                    operation,
                    SystemsOperation::ConsoleWrite { text }
                        if text == "TOPAL_KERNEL_USER_IMAGE_READY"
                            || text == "TOPAL_KERNEL_BOOT_RESOURCE_OK"
                            || text == "TOPAL_KERNEL_USER_READ_OK"
                            || text == "TOPAL_KERNEL_USER_CLOSE_OK"
                            || text == "TOPAL_KERNEL_USER_DUP_OK"
                            || text == "TOPAL_KERNEL_USER_FCNTL_OK"
                            || text == "TOPAL_KERNEL_USER_FSTAT_OK"
                            || text == "TOPAL_KERNEL_USER_TCGETS_OK"
                            || text == "TOPAL_KERNEL_USER_WINSIZE_OK"
                            || text == "TOPAL_KERNEL_USER_LSEEK_OK"
                            || text == "TOPAL_KERNEL_USER_IMAGE_ENTRY_OK"
                            || text == "TOPAL_KERNEL_USER_IMAGE_STACK_OK"
                            || text == "TOPAL_KERNEL_USER_IMAGE_AUXV_OK"
                            || text == "TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_OK"
                            || text == "TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_OK"
                            || text == "TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_OK"
                            || text == "TOPAL_KERNEL_USER_IMAGE_ELF_OK"
                            || text == "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_OK"
                            || text == "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_COPY_OK"
                            || text == "TOPAL_KERNEL_USER_IMAGE_RUNTIME_EXTENT_OK"
                            || text == "TOPAL_KERNEL_USER_IMAGE_RUNTIME_ENTRY_OK"
                            || text == "TOPAL_KERNEL_USER_IMAGE_RUNTIME_DATA_OK"
                            || text == "TOPAL_KERNEL_USER_IMAGE_BSS_OK"
                            || text == "TOPAL_KERNEL_USER_IMAGE_DATA_OK"
                            || text == "TOPAL_KERNEL_USER_IMAGE_COPY_TO_OK"
                            || text == "TOPAL_KERNEL_USER_IMAGE_COPY_FROM_OK"
                            || text == "TOPAL_KERNEL_USER_IMAGE_SYSCALL_OK"
                            || text == "TOPAL_KERNEL_USER_IMAGE_MULTIPAGE_EXEC_OK"
                            || text == "TOPAL_KERNEL_USER_IMAGE_RETAINED"
                )
            });
        missing_reclaim
            .bootstrap
            .handler
            .operations
            .retain(|operation| {
                !matches!(
                    operation,
                    SystemsOperation::ExtractEntropyGenerator { request }
                        if *request == EntropyGeneratorExtractRequest::INITIAL_Q35_GETRANDOM
                ) && !matches!(
                    operation,
                    SystemsOperation::ConsumeKernelCompletion
                        | SystemsOperation::ConsumeEmptyRunnableQueue
                        | SystemsOperation::PrepareUserAddressSpace
                        | SystemsOperation::VerifyUserAddressSpaceMapping
                        | SystemsOperation::MapUserAddressSpace
                        | SystemsOperation::ReserveAnonymousUserMapping
                        | SystemsOperation::StoreUserMappingByte
                        | SystemsOperation::InstallUserMappingEntryTrampoline
                        | SystemsOperation::ProtectUserMappingReadOnly
                        | SystemsOperation::PermitUserMappingExecute
                        | SystemsOperation::SealUserMappingImmutableData
                        | SystemsOperation::PermitUserMappingWritableData
                        | SystemsOperation::LoadUserMappingByte
                        | SystemsOperation::VerifyUserMappingEntryTrampoline
                        | SystemsOperation::UnmapUserAddressSpace
                        | SystemsOperation::ReleaseAnonymousUserMapping
                        | SystemsOperation::PrepareUserProbe { .. }
                        | SystemsOperation::DeliverInitialAuxvRandom { .. }
                        | SystemsOperation::ReserveInitialUserRandom { .. }
                        | SystemsOperation::BindReusableUserRandom { .. }
                        | SystemsOperation::ExpandSmallUserRandomRequests { .. }
                        | SystemsOperation::ExpandLargeUserRandomRequests { .. }
                        | SystemsOperation::EnterUserContext
                        | SystemsOperation::CompleteUserExecution
                        | SystemsOperation::ReclaimUserAddressSpace
                        | SystemsOperation::AllocatePhysicalFrames { .. }
                        | SystemsOperation::ReleasePhysicalFrames
                )
            });
        refresh_bootstrap_effects(&mut missing_reclaim);
        assert_eq!(
            validate_systems_program(&missing_reclaim).unwrap_err().code,
            "E-SYSTEMS-CONTEXT-TRANSFER-LIVE"
        );

        let mut transfer_before_create = program(SystemsDisposition::Resume);
        let create = transfer_before_create
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| matches!(operation, SystemsOperation::CreateKernelContext))
            .unwrap();
        let transfer = transfer_before_create
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| matches!(operation, SystemsOperation::TransferKernelContext))
            .or_else(|| {
                transfer_before_create
                    .bootstrap
                    .handler
                    .operations
                    .iter()
                    .position(|operation| {
                        matches!(
                            operation,
                            SystemsOperation::TransferKernelContextUntilDeadline
                        )
                    })
            })
            .unwrap();
        transfer_before_create
            .bootstrap
            .handler
            .operations
            .swap(create, transfer);
        assert_eq!(
            validate_systems_program(&transfer_before_create)
                .unwrap_err()
                .code,
            "E-SYSTEMS-CONTEXT-TRANSFER"
        );

        let mut wrong_handler = program(SystemsDisposition::Resume);
        wrong_handler.kernel_thread.handler.operations[0] = SystemsOperation::ConsoleWrite {
            text: "wrong".into(),
        };
        wrong_handler.kernel_thread.handler.effects = vec![
            SYSTEMS_KERNEL_CONTEXT_AWAIT_DEADLINE_PREEMPTION.into(),
            SYSTEMS_CONSOLE_WRITE.into(),
            SYSTEMS_KERNEL_CONTEXT_RETIRE.into(),
        ];
        assert_eq!(
            validate_systems_program(&wrong_handler).unwrap_err().code,
            "E-SYSTEMS-CONTEXT-TRANSFER"
        );
    }

    #[test]
    fn models_checked_bootstrap_region_contents_and_release() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-STORAGE-001.
        let mut program = program(SystemsDisposition::Resume);
        program.bootstrap.handler.operations.extend([
            SystemsOperation::BootstrapAllocate {
                request: BootstrapStorageRequest {
                    byte_count: 64,
                    alignment_bytes: 8,
                    placement: BootstrapStoragePlacement::BootstrapReclaimable,
                },
            },
            SystemsOperation::BootstrapStoreByte {
                offset_bytes: 7,
                value: 90,
            },
            SystemsOperation::BootstrapLoadByteEquals {
                offset_bytes: 7,
                expected: 90,
                failure_message: "mismatch".into(),
            },
            SystemsOperation::ConsoleWrite {
                text: "memory ok".into(),
            },
            SystemsOperation::BootstrapRelease,
        ]);
        refresh_bootstrap_effects(&mut program);
        let transitions = model_systems_transitions(&program).unwrap();
        assert!(
            transitions.contains(&SystemsTransition::StoreBootstrapByte {
                offset_bytes: 7,
                value: 90,
            })
        );
        assert!(transitions.contains(&SystemsTransition::LoadBootstrapByte {
            offset_bytes: 7,
            value: 90,
        }));
        assert!(transitions.contains(&SystemsTransition::ReleaseBootstrapRegion));

        let release = program.bootstrap.handler.operations.pop().unwrap();
        program
            .bootstrap
            .handler
            .effects
            .retain(|effect| effect != SYSTEMS_BOOTSTRAP_STORAGE_RELEASE);
        assert_eq!(
            validate_systems_program(&program).unwrap_err().code,
            "E-SYSTEMS-STORAGE-LIVE"
        );
        program.bootstrap.handler.operations.push(release);
        program
            .bootstrap
            .handler
            .effects
            .push(SYSTEMS_BOOTSTRAP_STORAGE_RELEASE.into());
        program.bootstrap.handler.effects.sort();
        let store = program
            .bootstrap
            .handler
            .operations
            .iter_mut()
            .find(|operation| matches!(operation, SystemsOperation::BootstrapStoreByte { .. }))
            .expect("test program contains a bootstrap store");
        let SystemsOperation::BootstrapStoreByte { offset_bytes, .. } = store else {
            unreachable!("matching operation was selected")
        };
        *offset_bytes = 64;
        assert_eq!(
            validate_systems_program(&program).unwrap_err().code,
            "E-SYSTEMS-STORAGE-BOUNDS"
        );
    }

    #[test]
    fn atomic_location_excludes_plain_region_ownership_until_end() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-ATOMIC-001,
        // TOPAL-SYSTEMS-STORAGE-001.
        let mut live = program(SystemsDisposition::Resume);
        live.bootstrap.handler.operations.extend([
            SystemsOperation::BootstrapAllocate {
                request: BootstrapStorageRequest {
                    byte_count: 64,
                    alignment_bytes: 8,
                    placement: BootstrapStoragePlacement::BootstrapReclaimable,
                },
            },
            SystemsOperation::AtomicWordCreate {
                request: AtomicWordRequest::initial(),
            },
        ]);
        live.bootstrap.handler.effects.extend([
            SYSTEMS_ATOMIC_WORD_CREATE.into(),
            SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE.into(),
        ]);
        live.bootstrap.handler.effects.sort();
        live.bootstrap.handler.effects.dedup();
        assert_eq!(
            validate_systems_program(&live).unwrap_err().code,
            "E-SYSTEMS-ATOMIC-LIVE"
        );

        live.bootstrap
            .handler
            .operations
            .push(SystemsOperation::BootstrapRelease);
        live.bootstrap
            .handler
            .effects
            .push(SYSTEMS_BOOTSTRAP_STORAGE_RELEASE.into());
        live.bootstrap.handler.effects.sort();
        live.bootstrap.handler.effects.dedup();
        assert_eq!(
            validate_systems_program(&live).unwrap_err().code,
            "E-SYSTEMS-STORAGE-LIFETIME"
        );
    }

    #[test]
    fn models_affine_physical_frame_allocation_and_release() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-FRAMES-001.
        let mut program = program(SystemsDisposition::Resume);
        let request = PhysicalFrameRequest {
            frame_count: 3,
            alignment_frames: 1,
        };
        program.bootstrap.handler.operations.splice(
            2..2,
            [
                SystemsOperation::AllocatePhysicalFrames {
                    request,
                    failure_message: "frame allocation failed".into(),
                },
                SystemsOperation::ReleasePhysicalFrames,
            ],
        );
        program.bootstrap.handler.effects.extend([
            SYSTEMS_FRAMES_ALLOCATE.into(),
            SYSTEMS_FRAMES_RELEASE.into(),
        ]);
        program.bootstrap.handler.effects.sort();
        program.bootstrap.handler.effects.dedup();
        let transitions = model_systems_transitions(&program).unwrap();
        assert!(transitions.contains(&SystemsTransition::AllocatePhysicalFrames { request }));
        assert!(transitions.contains(&SystemsTransition::ReleasePhysicalFrames));

        let mut unsupported = program.clone();
        let SystemsOperation::AllocatePhysicalFrames {
            request: unsupported_request,
            ..
        } = &mut unsupported.bootstrap.handler.operations[2]
        else {
            unreachable!("test inserted the allocation at index two")
        };
        unsupported_request.frame_count = 4;
        assert_eq!(
            validate_systems_program(&unsupported).unwrap_err().code,
            "E-SYSTEMS-FRAMES"
        );

        let mut mapped = program.clone();
        mapped.bootstrap.handler.operations.insert(
            3,
            SystemsOperation::MapKernelFrames {
                request: KernelMappingRequest::initial_read_write(),
                failure_message: "mapping failed".into(),
            },
        );
        refresh_bootstrap_effects(&mut mapped);
        assert_eq!(
            validate_systems_program(&mapped).unwrap_err().code,
            "E-SYSTEMS-MAPPING"
        );

        program.bootstrap.handler.operations.drain(2..4);
        program
            .bootstrap
            .handler
            .operations
            .push(SystemsOperation::AllocatePhysicalFrames {
                request,
                failure_message: "late frame allocation failed".into(),
            });
        refresh_bootstrap_effects(&mut program);
        assert_eq!(
            validate_systems_program(&program).unwrap_err().code,
            "E-SYSTEMS-FRAMES-LIVE"
        );
    }

    #[test]
    fn models_exact_static_elf_mapping_resource_ownership() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-STATIC-ELF-MAPPING-RESOURCES-001.
        let mut program = program(SystemsDisposition::Resume);
        let operations = [
            SystemsOperation::AllocatePhysicalFrames {
                request: STATIC_ELF_FRAME_REQUEST,
                failure_message: "payload allocation failed".into(),
            },
            SystemsOperation::AllocatePhysicalFrames {
                request: INITIAL_SINGLE_FRAME_REQUEST,
                failure_message: "metadata allocation failed".into(),
            },
            SystemsOperation::ReleasePhysicalFrames,
            SystemsOperation::ReleasePhysicalFrames,
        ];
        program
            .bootstrap
            .handler
            .operations
            .splice(2..2, operations.clone());
        refresh_bootstrap_effects(&mut program);
        assert!(validate_systems_program(&program).is_ok());

        let mut wrong_metadata = program.clone();
        let SystemsOperation::AllocatePhysicalFrames { request, .. } =
            &mut wrong_metadata.bootstrap.handler.operations[3]
        else {
            unreachable!("test inserted metadata allocation at index three")
        };
        *request = INITIAL_TWO_FRAME_REQUEST;
        assert_eq!(
            validate_systems_program(&wrong_metadata).unwrap_err().code,
            "E-SYSTEMS-FRAMES"
        );

        let mut incomplete = program;
        incomplete.bootstrap.handler.operations.remove(5);
        refresh_bootstrap_effects(&mut incomplete);
        assert_eq!(
            validate_systems_program(&incomplete).unwrap_err().code,
            "E-SYSTEMS-FRAMES"
        );
    }

    #[test]
    fn models_static_elf_initial_leaf_construction_and_revocation() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-STATIC-ELF-LEAVES-001,
        // TOPAL-SYSTEMS-STATIC-ELF-ZERO-001, TOPAL-SYSTEMS-STATIC-ELF-COPY-001.
        let mut program = program(SystemsDisposition::Resume);
        program.bootstrap.handler.operations.splice(
            2..2,
            [
                SystemsOperation::AllocatePhysicalFrames {
                    request: STATIC_ELF_FRAME_REQUEST,
                    failure_message: "payload allocation failed".into(),
                },
                SystemsOperation::AllocatePhysicalFrames {
                    request: INITIAL_SINGLE_FRAME_REQUEST,
                    failure_message: "metadata allocation failed".into(),
                },
                SystemsOperation::PrepareStaticImageConstruction {
                    failure_message: "leaf construction failed".into(),
                },
                SystemsOperation::ZeroStaticImageConstructionPayload,
                SystemsOperation::CopyStaticImageConstructionPayload,
                SystemsOperation::SealStaticImageConstructionPermissions,
                SystemsOperation::RevokeStaticImageConstructionMetadata,
                SystemsOperation::ReleasePhysicalFrames,
                SystemsOperation::ReclaimStaticImageConstructionPayload,
                SystemsOperation::ReleasePhysicalFrames,
            ],
        );
        refresh_bootstrap_effects(&mut program);
        let transitions = model_systems_transitions(&program).unwrap();
        assert!(transitions.contains(&SystemsTransition::PrepareStaticImageConstruction));
        assert!(transitions.contains(&SystemsTransition::ZeroStaticImageConstructionPayload));
        assert!(transitions.contains(&SystemsTransition::CopyStaticImageConstructionPayload));
        assert!(transitions.contains(&SystemsTransition::SealStaticImageConstructionPermissions));
        assert!(transitions.contains(&SystemsTransition::RevokeStaticImageConstructionMetadata));
        assert!(transitions.contains(&SystemsTransition::ReclaimStaticImageConstructionPayload));

        let mut reclaim_before_metadata_release = program.clone();
        reclaim_before_metadata_release
            .bootstrap
            .handler
            .operations
            .swap(9, 10);
        assert_eq!(
            validate_systems_program(&reclaim_before_metadata_release)
                .unwrap_err()
                .code,
            "E-SYSTEMS-STATIC-IMAGE"
        );

        let mut revoke_before_zero = program.clone();
        revoke_before_zero.bootstrap.handler.operations.remove(5);
        refresh_bootstrap_effects(&mut revoke_before_zero);
        assert_eq!(
            validate_systems_program(&revoke_before_zero)
                .unwrap_err()
                .code,
            "E-SYSTEMS-STATIC-IMAGE"
        );

        let mut revoke_before_copy = program.clone();
        revoke_before_copy.bootstrap.handler.operations.remove(6);
        refresh_bootstrap_effects(&mut revoke_before_copy);
        assert_eq!(
            validate_systems_program(&revoke_before_copy)
                .unwrap_err()
                .code,
            "E-SYSTEMS-STATIC-IMAGE"
        );

        let mut revoke_before_seal = program.clone();
        revoke_before_seal.bootstrap.handler.operations.remove(7);
        refresh_bootstrap_effects(&mut revoke_before_seal);
        assert_eq!(
            validate_systems_program(&revoke_before_seal)
                .unwrap_err()
                .code,
            "E-SYSTEMS-STATIC-IMAGE"
        );

        let mut live = program;
        live.bootstrap.handler.operations.truncate(5);
        refresh_bootstrap_effects(&mut live);
        assert_eq!(
            validate_systems_program(&live).unwrap_err().code,
            "E-SYSTEMS-FRAMES-LIVE"
        );
    }

    #[test]
    fn models_static_elf_stack_entry_lifecycle() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-STATIC-ELF-STACK-LEAF-001,
        // TOPAL-SYSTEMS-STATIC-ELF-STACK-ENTRY-001.
        let mut program = program(SystemsDisposition::Resume);
        program.bootstrap.handler.operations.splice(
            2..2,
            [
                SystemsOperation::AllocatePhysicalFrames {
                    request: STATIC_ELF_FRAME_REQUEST,
                    failure_message: "payload allocation failed".into(),
                },
                SystemsOperation::AllocatePhysicalFrames {
                    request: INITIAL_SINGLE_FRAME_REQUEST,
                    failure_message: "metadata allocation failed".into(),
                },
                SystemsOperation::PrepareStaticImageConstruction {
                    failure_message: "leaf construction failed".into(),
                },
                SystemsOperation::ZeroStaticImageConstructionPayload,
                SystemsOperation::CopyStaticImageConstructionPayload,
                SystemsOperation::SealStaticImageConstructionPermissions,
                SystemsOperation::AllocatePhysicalFrames {
                    request: STATIC_ELF_HIERARCHY_FRAME_REQUEST,
                    failure_message: "hierarchy allocation failed".into(),
                },
                SystemsOperation::AttachStaticImageAddressSpace,
                SystemsOperation::ActivateStaticImageAddressSpace,
                SystemsOperation::RestoreStaticImageAddressSpace,
                SystemsOperation::AuthorizeStaticImageEntry,
                SystemsOperation::AuthorizeStaticImageStackSlot,
                SystemsOperation::AllocatePhysicalFrames {
                    request: INITIAL_TWO_FRAME_REQUEST,
                    failure_message: "stack backing allocation failed".into(),
                },
                SystemsOperation::ZeroStaticImageStackBacking,
                SystemsOperation::PublishStaticImageStackLeaf,
                SystemsOperation::InitializeStaticImageStackEntry,
                SystemsOperation::AddStaticImageStackPageSize,
                SystemsOperation::AddStaticImageStackEntryFact,
                SystemsOperation::AddStaticImageStackProgramHeaders,
                SystemsOperation::AddStaticImageStackExecutableName,
                SystemsOperation::AddStaticImageStackCredentials,
                SystemsOperation::AddStaticImageStackSecureMode,
                SystemsOperation::AddStaticImageStackClockTickRate,
                SystemsOperation::AddStaticImageStackPlatform,
                SystemsOperation::AddStaticImageStackHardwareCapabilities,
                SystemsOperation::AddStaticImageStackLoadMetadata,
                SystemsOperation::AuthorizeStaticImageStackRandomSlot,
                SystemsOperation::RetainStaticImagePreparedProcess,
                SystemsOperation::QualifyStaticImageRetainedDisjointWork,
                SystemsOperation::AuthorizeStaticImageEntropyHandoff,
                SystemsOperation::AuthorizeStaticImageEntropyAcquisition,
                SystemsOperation::QualifyStaticImageEntropyDevice,
                SystemsOperation::QualifyStaticImageEntropyTransport,
                SystemsOperation::QualifyStaticImageEntropyNegotiation,
                SystemsOperation::QualifyStaticImageEntropyQueue,
                SystemsOperation::QualifyStaticImageEntropyBusMaster,
                SystemsOperation::QualifyStaticImageEntropyDmaBuffers,
                SystemsOperation::QualifyStaticImageEntropySplitRing,
                SystemsOperation::QualifyStaticImageEntropyDmaPreparation,
                SystemsOperation::QualifyStaticImageEntropyQueueAddresses,
                SystemsOperation::QualifyStaticImageEntropyQueueActivation,
                SystemsOperation::QualifyStaticImageEntropyRequestCompletion,
                SystemsOperation::QualifyStaticImageEntropySampleCapture,
                SystemsOperation::QualifyStaticImageEntropySampleQuality,
                SystemsOperation::QualifyStaticImageEntropyPoolLifecycle,
                SystemsOperation::QualifyStaticImageEntropyGeneratorLifecycle,
                SystemsOperation::DeliverStaticImageEntropyInitialRandom,
                SystemsOperation::EnterStaticImageProcess,
                SystemsOperation::RestoreStaticImageProcessEntry,
                SystemsOperation::RevokeStaticImageEntropyInitialRandom,
                SystemsOperation::RevokeStaticImageEntropyAcquisition,
                SystemsOperation::RevokeStaticImageEntropyHandoff,
                SystemsOperation::RestoreStaticImagePreparedProcess,
                SystemsOperation::RevokeStaticImageStackRandomSlot,
                SystemsOperation::RemoveStaticImageStackLoadMetadata,
                SystemsOperation::RemoveStaticImageStackHardwareCapabilities,
                SystemsOperation::RemoveStaticImageStackPlatform,
                SystemsOperation::RemoveStaticImageStackClockTickRate,
                SystemsOperation::RemoveStaticImageStackSecureMode,
                SystemsOperation::RemoveStaticImageStackCredentials,
                SystemsOperation::RemoveStaticImageStackExecutableName,
                SystemsOperation::RemoveStaticImageStackProgramHeaders,
                SystemsOperation::RemoveStaticImageStackEntryFact,
                SystemsOperation::RemoveStaticImageStackPageSize,
                SystemsOperation::ClearStaticImageStackEntry,
                SystemsOperation::RevokeStaticImageStackLeaf,
                SystemsOperation::ReleasePhysicalFrames,
                SystemsOperation::RevokeStaticImageStackSlot,
                SystemsOperation::RevokeStaticImageEntry,
                SystemsOperation::DetachStaticImageAddressSpace,
                SystemsOperation::ReclaimStaticImageConstructionHierarchy,
                SystemsOperation::ReleasePhysicalFrames,
                SystemsOperation::RevokeStaticImageConstructionMetadata,
                SystemsOperation::ReleasePhysicalFrames,
                SystemsOperation::ReclaimStaticImageConstructionPayload,
                SystemsOperation::ReleasePhysicalFrames,
            ],
        );
        refresh_bootstrap_effects(&mut program);
        let transitions = model_systems_transitions(&program).unwrap();
        assert!(transitions.contains(&SystemsTransition::PublishStaticImageStackLeaf));
        assert!(transitions.contains(&SystemsTransition::InitializeStaticImageStackEntry));
        assert!(transitions.contains(&SystemsTransition::AddStaticImageStackPageSize));
        assert!(transitions.contains(&SystemsTransition::AddStaticImageStackEntryFact));
        assert!(transitions.contains(&SystemsTransition::AddStaticImageStackProgramHeaders));
        assert!(transitions.contains(&SystemsTransition::AddStaticImageStackExecutableName));
        assert!(transitions.contains(&SystemsTransition::AddStaticImageStackCredentials));
        assert!(transitions.contains(&SystemsTransition::AddStaticImageStackSecureMode));
        assert!(transitions.contains(&SystemsTransition::AddStaticImageStackClockTickRate));
        assert!(transitions.contains(&SystemsTransition::AddStaticImageStackPlatform));
        assert!(transitions.contains(&SystemsTransition::AddStaticImageStackHardwareCapabilities));
        assert!(transitions.contains(&SystemsTransition::AddStaticImageStackLoadMetadata));
        assert!(transitions.contains(&SystemsTransition::AuthorizeStaticImageStackRandomSlot));
        assert!(transitions.contains(&SystemsTransition::RetainStaticImagePreparedProcess));
        assert!(transitions.contains(&SystemsTransition::QualifyStaticImageRetainedDisjointWork));
        assert!(transitions.contains(&SystemsTransition::AuthorizeStaticImageEntropyHandoff));
        assert!(transitions.contains(&SystemsTransition::AuthorizeStaticImageEntropyAcquisition));
        assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropyDevice));
        assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropyTransport));
        assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropyNegotiation));
        assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropyQueue));
        assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropyBusMaster));
        assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropyDmaBuffers));
        assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropySplitRing));
        assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropyDmaPreparation));
        assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropyQueueAddresses));
        assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropyQueueActivation));
        assert!(
            transitions.contains(&SystemsTransition::QualifyStaticImageEntropyRequestCompletion)
        );
        assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropySampleCapture));
        assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropySampleQuality));
        assert!(transitions.contains(&SystemsTransition::QualifyStaticImageEntropyPoolLifecycle));
        assert!(
            transitions.contains(&SystemsTransition::QualifyStaticImageEntropyGeneratorLifecycle)
        );
        assert!(transitions.contains(&SystemsTransition::DeliverStaticImageEntropyInitialRandom));
        assert!(transitions.contains(&SystemsTransition::EnterStaticImageProcess));
        assert!(transitions.contains(&SystemsTransition::RestoreStaticImageProcessEntry));
        assert!(transitions.contains(&SystemsTransition::RevokeStaticImageEntropyInitialRandom));
        assert!(transitions.contains(&SystemsTransition::RevokeStaticImageEntropyAcquisition));
        assert!(transitions.contains(&SystemsTransition::RevokeStaticImageEntropyHandoff));
        assert!(transitions.contains(&SystemsTransition::RestoreStaticImagePreparedProcess));
        assert!(transitions.contains(&SystemsTransition::RevokeStaticImageStackRandomSlot));
        assert!(transitions.contains(&SystemsTransition::RemoveStaticImageStackLoadMetadata));
        assert!(
            transitions.contains(&SystemsTransition::RemoveStaticImageStackHardwareCapabilities)
        );
        assert!(transitions.contains(&SystemsTransition::RemoveStaticImageStackPlatform));
        assert!(transitions.contains(&SystemsTransition::RemoveStaticImageStackClockTickRate));
        assert!(transitions.contains(&SystemsTransition::RemoveStaticImageStackSecureMode));
        assert!(transitions.contains(&SystemsTransition::RemoveStaticImageStackCredentials));
        assert!(transitions.contains(&SystemsTransition::RemoveStaticImageStackExecutableName));
        assert!(transitions.contains(&SystemsTransition::RemoveStaticImageStackProgramHeaders));
        assert!(transitions.contains(&SystemsTransition::RemoveStaticImageStackEntryFact));
        assert!(transitions.contains(&SystemsTransition::RemoveStaticImageStackPageSize));
        assert!(transitions.contains(&SystemsTransition::ClearStaticImageStackEntry));
        assert!(transitions.contains(&SystemsTransition::RevokeStaticImageStackLeaf));

        let mut release_while_initialized = program.clone();
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        release_while_initialized
            .bootstrap
            .handler
            .operations
            .remove(19);
        refresh_bootstrap_effects(&mut release_while_initialized);
        assert_eq!(
            validate_systems_program(&release_while_initialized)
                .unwrap_err()
                .code,
            "E-SYSTEMS-FRAMES"
        );

        let mut duplicate_publication = program.clone();
        duplicate_publication
            .bootstrap
            .handler
            .operations
            .insert(17, SystemsOperation::PublishStaticImageStackLeaf);
        refresh_bootstrap_effects(&mut duplicate_publication);
        assert_eq!(
            validate_systems_program(&duplicate_publication)
                .unwrap_err()
                .code,
            "E-SYSTEMS-STATIC-IMAGE"
        );

        let mut duplicate_initialization = program.clone();
        duplicate_initialization
            .bootstrap
            .handler
            .operations
            .insert(18, SystemsOperation::InitializeStaticImageStackEntry);
        refresh_bootstrap_effects(&mut duplicate_initialization);
        assert_eq!(
            validate_systems_program(&duplicate_initialization)
                .unwrap_err()
                .code,
            "E-SYSTEMS-STATIC-IMAGE"
        );

        let mut remove_load_metadata_before_addition = program;
        remove_load_metadata_before_addition
            .bootstrap
            .handler
            .operations
            .swap(27, 28);
        assert_eq!(
            validate_systems_program(&remove_load_metadata_before_addition)
                .unwrap_err()
                .code,
            "E-SYSTEMS-STATIC-IMAGE"
        );
    }

    #[test]
    fn critical_scope_requires_lifo_restore_before_blocking_or_completion() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-CRITICAL-001.
        let mut blocked = program(SystemsDisposition::Resume);
        blocked.bootstrap.handler.operations.insert(
            2,
            SystemsOperation::EnterCritical {
                domain: CriticalDomain::LocalMaskableInterrupts,
                failure_message: "critical entry failed".into(),
            },
        );
        blocked
            .bootstrap
            .handler
            .effects
            .push(SYSTEMS_CRITICAL_ENTER.into());
        blocked.bootstrap.handler.effects.sort();
        assert_eq!(
            validate_systems_program(&blocked).unwrap_err().code,
            "E-SYSTEMS-CRITICAL"
        );

        blocked.bootstrap.handler.operations.insert(
            3,
            SystemsOperation::RestoreCritical {
                domain: CriticalDomain::LocalMaskableInterrupts,
            },
        );
        blocked
            .bootstrap
            .handler
            .effects
            .push(SYSTEMS_CRITICAL_RESTORE.into());
        blocked.bootstrap.handler.effects.sort();
        assert!(validate_systems_program(&blocked).is_ok());

        let mut live = program(SystemsDisposition::Resume);
        live.bootstrap.handler.operations.truncate(2);
        live.bootstrap
            .handler
            .operations
            .push(SystemsOperation::EnterCritical {
                domain: CriticalDomain::LocalMaskableInterrupts,
                failure_message: "critical entry failed".into(),
            });
        live.bootstrap.handler.effects = vec![
            SYSTEMS_BOOT_MEMORY_DESCRIBE.into(),
            SYSTEMS_CRITICAL_ENTER.into(),
            SYSTEMS_FATAL.into(),
            SYSTEMS_FRAME_ALLOCATOR_CREATE.into(),
        ];
        assert_eq!(
            validate_systems_program(&live).unwrap_err().code,
            "E-SYSTEMS-CRITICAL-LIVE"
        );

        let mut restore_without_entry = program(SystemsDisposition::Resume);
        restore_without_entry.bootstrap.handler.operations.insert(
            2,
            SystemsOperation::RestoreCritical {
                domain: CriticalDomain::LocalMaskableInterrupts,
            },
        );
        restore_without_entry
            .bootstrap
            .handler
            .effects
            .push(SYSTEMS_CRITICAL_RESTORE.into());
        restore_without_entry.bootstrap.handler.effects.sort();
        assert_eq!(
            validate_systems_program(&restore_without_entry)
                .unwrap_err()
                .code,
            "E-SYSTEMS-CRITICAL"
        );
    }

    #[test]
    fn fatal_debug_break_cannot_resume_bootstrap() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-DISPOSITION-001.
        let transitions = model_systems_transitions(&program(SystemsDisposition::Fatal {
            message: "fault".into(),
        }))
        .unwrap();
        assert_eq!(
            transitions.last(),
            Some(&SystemsTransition::Fatal {
                message: "fault".into(),
            })
        );
        assert!(!transitions.contains(&SystemsTransition::Fatal {
            message: "done".into(),
        }));
    }

    #[test]
    fn rejects_invalid_context_operations_and_effect_claims() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-AUTHORITY-001.
        let mut invalid_context = program(SystemsDisposition::Resume);
        invalid_context
            .debug_break
            .handler
            .operations
            .push(SystemsOperation::DebugBreak);
        assert_eq!(
            model_systems_transitions(&invalid_context)
                .unwrap_err()
                .code,
            "E-SYSTEMS-OPERATION"
        );

        let mut invalid_effects = program(SystemsDisposition::Resume);
        invalid_effects.bootstrap.handler.effects.clear();
        assert_eq!(
            model_systems_transitions(&invalid_effects)
                .unwrap_err()
                .code,
            "E-SYSTEMS-EFFECTS"
        );
    }

    #[test]
    fn rejects_wrong_target_and_bootstrap_resume() {
        // TOPAL-SYSTEMS-QUALIFY-001, TOPAL-SYSTEMS-DISPOSITION-001.
        let mut wrong_target = program(SystemsDisposition::Resume);
        wrong_target.target.board = "another-board".into();
        assert_eq!(
            validate_systems_program(&wrong_target).unwrap_err().code,
            "E-SYSTEMS-TARGET"
        );

        let mut invalid_disposition = program(SystemsDisposition::Resume);
        invalid_disposition.bootstrap.handler.disposition = SystemsDisposition::Resume;
        invalid_disposition.bootstrap.handler.effects = vec![
            SYSTEMS_CONSOLE_WRITE.into(),
            SYSTEMS_DEBUG_BREAK.into(),
            SYSTEMS_RESUME_DEBUG_BREAK.into(),
        ];
        assert_eq!(
            validate_systems_program(&invalid_disposition)
                .unwrap_err()
                .code,
            "E-SYSTEMS-DISPOSITION"
        );
    }

    #[test]
    fn rejects_invalid_bootstrap_storage_descriptor() {
        // TOPAL-SYSTEMS-STORAGE-001.
        let mut invalid = program(SystemsDisposition::Resume);
        invalid.bootstrap_storage.alignment_bytes = 3;
        assert_eq!(
            validate_systems_program(&invalid).unwrap_err().code,
            "E-SYSTEMS-STORAGE"
        );
    }
}
