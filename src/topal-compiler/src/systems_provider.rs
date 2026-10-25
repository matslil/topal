//! Deterministic x86-64 provider planning for checked systems programs.

use topal_language::compiler::{
    CompilerSystemsProgram, INITIAL_SYSTEMS_BOARD, INITIAL_SYSTEMS_PROFILE, INITIAL_SYSTEMS_TARGET,
    SYSTEMS_ATOMIC_COMPARE_EXCHANGE, SYSTEMS_ATOMIC_END, SYSTEMS_ATOMIC_LOAD,
    SYSTEMS_ATOMIC_WORD_CREATE, SYSTEMS_BOOT_MEMORY_DESCRIBE, SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
    SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE, SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
    SYSTEMS_BOOTSTRAP_STORAGE_COMPLETE, SYSTEMS_BOOTSTRAP_STORAGE_PROVISION,
    SYSTEMS_BOOTSTRAP_STORAGE_RELEASE, SYSTEMS_CONSOLE_WRITE, SYSTEMS_CRITICAL_ENTER,
    SYSTEMS_CRITICAL_RESTORE, SYSTEMS_DEBUG_BREAK, SYSTEMS_DMA_BUFFER_BIND_REFERENCE,
    SYSTEMS_DMA_BUFFER_COMPLETE_CPU, SYSTEMS_DMA_BUFFER_CREATE, SYSTEMS_DMA_BUFFER_LOAD_INTEGER,
    SYSTEMS_DMA_BUFFER_PREPARE_DEVICE, SYSTEMS_DMA_BUFFER_STORE_INTEGER,
    SYSTEMS_ENTROPY_GENERATOR_EXTRACT, SYSTEMS_ENTROPY_GENERATOR_INITIALIZE,
    SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM,
    SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM, SYSTEMS_ENTROPY_POOL_CREATE,
    SYSTEMS_ENTROPY_POOL_CREDIT_CONTRIBUTION, SYSTEMS_ENTROPY_POOL_MIX_SAMPLE,
    SYSTEMS_ENTROPY_SAMPLE_ASSESS_QUALITY, SYSTEMS_ENTROPY_SAMPLE_CAPTURE_DMA,
    SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE, SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS,
    SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS, SYSTEMS_FATAL,
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
    SYSTEMS_KERNEL_UNMAP, SYSTEMS_LOCAL_NOTIFICATION_COMPLETE, SYSTEMS_LOCAL_NOTIFICATION_SEND,
    SYSTEMS_LOCAL_NOTIFICATION_WAIT, SYSTEMS_MONOTONIC_CLOCK_NOW,
    SYSTEMS_PCI_CONFIGURATION_READ_DWORD, SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD,
    SYSTEMS_PCI_REGION_READ, SYSTEMS_PCI_REGION_READ_EVENTUALLY, SYSTEMS_PCI_REGION_WRITE,
    SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE, SYSTEMS_RESUME_DEBUG_BREAK,
    SYSTEMS_RESUME_LOCAL_NOTIFICATION, SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ACTIVATE_ADDRESS_SPACE,
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
    validate_systems_program,
};
use topal_language::compiler::{
    SYSTEMS_DEADLINE_AFTER, SYSTEMS_DEADLINE_ARM, SYSTEMS_DEADLINE_COMPLETE, SYSTEMS_DEADLINE_WAIT,
    SYSTEMS_RESUME_DEADLINE,
};

use crate::CompileError;

pub const X86_SYSTEMS_PROVIDER_REVISION: &str = "topal.provider.x86_64-qemu-pc-q35/158";
pub const X86_SYSTEMS_PLATFORM_ABI: &str = "topal.systems.x86_64-bare/1";
pub const X86_SYSTEMS_DATA_LAYOUT: &str =
    "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum X86SystemsLowering {
    LinuxBootParamsE820,
    RetainedE820FrameAuthority,
    E820StaticFrameExtentSelect,
    ConsumePhysicalFrameExtent,
    ConstructInitialStaticImageLeaves,
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
    ReturnStaticImageHierarchy,
    RevokeInitialStaticImageLeaves,
    ReturnStaticImagePayload,
    AdoptBootstrapIdentityMapping,
    PlainKernelMappingStoreByte,
    PlainKernelMappingLoadByte,
    ConsumeKernelMapping,
    SelectBootstrapTranslationBacking,
    BuildBootstrapEquivalentTranslation,
    ActivateTranslationRoot,
    BeginActiveTranslationEdit,
    StageActiveTranslationMap,
    StageActiveTranslationUnmap,
    CommitActiveTranslationEdit,
    CaptureAndMaskLocalInterrupts,
    RestoreLocalInterruptState,
    InitializeAtomicWord,
    LockedCompareExchangeWord,
    LoadAtomicWordAcquire,
    ConsumeAtomicWord,
    SendLocalApicNotification,
    WaitForLocalNotification,
    CompleteLocalApicNotification,
    ResumeLocalNotification,
    ObserveHpetMonotonicClock,
    ConstructHpetDeadline,
    ArmHpetIoApicDeadline,
    WaitForDeadline,
    CompleteHpetIoApicDeadline,
    ResumeDeadline,
    CreateKernelContext,
    TransferKernelContextUntilDeadline,
    AwaitDeadlinePreemption,
    BlockOneShotCompletion,
    PreemptCurrentKernelContext,
    TransferKernelContext,
    RetireKernelContext,
    ReclaimKernelContext,
    PrepareInitialUserAddressSpace,
    VerifyInitialUserMapping,
    MapInitialUserFrame,
    StoreInitialUserMappingByte,
    InstallInitialUserEntryTrampoline,
    ProtectInitialUserMappingReadOnly,
    LoadInitialUserMappingByte,
    VerifyInitialUserEntryTrampoline,
    PermitInitialUserMappingExecute,
    SealInitialUserMappingImmutableData,
    PermitInitialUserMappingWritableData,
    ReserveInitialAnonymousUserMapping,
    ReleaseInitialAnonymousUserMapping,
    UnmapInitialUserFrame,
    PrepareInitialUserProbe,
    EnterInitialUserContext,
    CompleteInitialUserBacking,
    ReclaimInitialUserAddressSpace,
    CopyInitialUnameToUser,
    CopyInitialTimespecFromUser,
    DispatchInitialUserSyscall,
    StaticBootstrapStorage,
    MonotonicBootstrapAllocate,
    PlainBootstrapRegionStoreByte,
    PlainBootstrapRegionLoadByte,
    ConsumeBootstrapRegion,
    CompleteBootstrapStorage,
    StaticCoherentDmaBuffer,
    BindIdentityTranslatedDmaReference,
    StoreCpuOwnedDmaInteger,
    PrepareCoherentDmaForDevice,
    CompleteCoherentDmaForCpu,
    LoadCpuOwnedDmaInteger,
    CaptureOpaqueRawEntropySample,
    MixBlake2sEntropyPoolSample,
    InitializeChaCha20EntropyGenerator,
    ExtractChaCha20EntropyGenerator,
    DeliverInitialAuxvRandom,
    ReserveInitialUserRandom,
    BindReusableUserRandom,
    PciConfigurationMechanismOneReadDword,
    PciConfigurationMechanismOneUpdateWord,
    PciConfigurationGatewayRegionRead,
    PciConfigurationGatewayRegionReadEventually,
    PciConfigurationGatewayRegionWrite,
    PciConfigurationGatewayDmaReferenceWrite,
    PolledUart16550PortIo,
    BreakpointVector3,
    InterruptReturn,
    InterruptsDisabledHalt,
}

impl X86SystemsLowering {
    #[must_use]
    pub const fn provider_identity(self) -> &'static str {
        match self {
            Self::LinuxBootParamsE820 => "topal.provider.x86_64.boot.linux-e820/1",
            Self::RetainedE820FrameAuthority => {
                "topal.provider.x86_64.frames.retained-e820-authority/1"
            }
            Self::E820StaticFrameExtentSelect => {
                "topal.provider.x86_64.frames.select-static-e820-extent/1"
            }
            Self::ConsumePhysicalFrameExtent => "topal.provider.x86_64.frames.consume-extent/1",
            Self::ConstructInitialStaticImageLeaves => {
                "topal.provider.x86_64.static-image.construct-initial-leaves/1"
            }
            Self::ZeroStaticImagePayload => "topal.provider.x86_64.static-image.zero-payload/1",
            Self::ZeroStaticImageStackBacking => {
                "topal.provider.x86_64.static-image.zero-stack-backing/1"
            }
            Self::PublishStaticImageStackLeaf => {
                "topal.provider.x86_64.static-image.publish-stack-leaf/1"
            }
            Self::InitializeStaticImageStackEntry => {
                "topal.provider.x86_64.static-image.initialize-stack-entry/1"
            }
            Self::AddStaticImageStackPageSize => {
                "topal.provider.x86_64.static-image.add-stack-pagesz/1"
            }
            Self::AddStaticImageStackEntryFact => {
                "topal.provider.x86_64.static-image.add-stack-entry-fact/1"
            }
            Self::AddStaticImageStackProgramHeaders => {
                "topal.provider.x86_64.static-image.add-stack-program-headers/1"
            }
            Self::AddStaticImageStackExecutableName => {
                "topal.provider.x86_64.static-image.add-stack-executable-name/1"
            }
            Self::AddStaticImageStackCredentials => {
                "topal.provider.x86_64.static-image.add-stack-credentials/1"
            }
            Self::AddStaticImageStackSecureMode => {
                "topal.provider.x86_64.static-image.add-stack-secure-mode/1"
            }
            Self::AddStaticImageStackClockTickRate => {
                "topal.provider.x86_64.static-image.add-stack-clock-tick-rate/1"
            }
            Self::AddStaticImageStackPlatform => {
                "topal.provider.x86_64.static-image.add-stack-platform/1"
            }
            Self::AddStaticImageStackHardwareCapabilities => {
                "topal.provider.x86_64.static-image.add-stack-hardware-capabilities/1"
            }
            Self::AddStaticImageStackLoadMetadata => {
                "topal.provider.x86_64.static-image.add-stack-load-metadata/1"
            }
            Self::AuthorizeStaticImageStackRandomSlot => {
                "topal.provider.x86_64.static-image.authorize-stack-random-slot/1"
            }
            Self::RetainStaticImagePreparedProcess => {
                "topal.provider.x86_64.static-image.retain-prepared-process/1"
            }
            Self::QualifyStaticImageRetainedDisjointWork => {
                "topal.provider.x86_64.static-image.qualify-retained-disjoint-work/1"
            }
            Self::AuthorizeStaticImageEntropyHandoff => {
                "topal.provider.x86_64.static-image.authorize-entropy-handoff/1"
            }
            Self::AuthorizeStaticImageEntropyAcquisition => {
                "topal.provider.x86_64.static-image.authorize-entropy-acquisition/1"
            }
            Self::QualifyStaticImageEntropyDevice => {
                "topal.provider.x86_64.static-image.qualify-entropy-device/1"
            }
            Self::QualifyStaticImageEntropyTransport => {
                "topal.provider.x86_64.static-image.qualify-entropy-transport/1"
            }
            Self::QualifyStaticImageEntropyNegotiation => {
                "topal.provider.x86_64.static-image.qualify-entropy-negotiation/1"
            }
            Self::QualifyStaticImageEntropyQueue => {
                "topal.provider.x86_64.static-image.qualify-entropy-queue/1"
            }
            Self::QualifyStaticImageEntropyBusMaster => {
                "topal.provider.x86_64.static-image.qualify-entropy-bus-master/1"
            }
            Self::QualifyStaticImageEntropyDmaBuffers => {
                "topal.provider.x86_64.static-image.qualify-entropy-dma-buffers/1"
            }
            Self::QualifyStaticImageEntropySplitRing => {
                "topal.provider.x86_64.static-image.qualify-entropy-split-ring/1"
            }
            Self::QualifyStaticImageEntropyDmaPreparation => {
                "topal.provider.x86_64.static-image.qualify-entropy-dma-preparation/1"
            }
            Self::QualifyStaticImageEntropyQueueAddresses => {
                "topal.provider.x86_64.static-image.qualify-entropy-queue-addresses/1"
            }
            Self::QualifyStaticImageEntropyQueueActivation => {
                "topal.provider.x86_64.static-image.qualify-entropy-queue-activation/1"
            }
            Self::QualifyStaticImageEntropyRequestCompletion => {
                "topal.provider.x86_64.static-image.qualify-entropy-request-completion/1"
            }
            Self::QualifyStaticImageEntropySampleCapture => {
                "topal.provider.x86_64.static-image.qualify-entropy-sample-capture/1"
            }
            Self::QualifyStaticImageEntropySampleQuality => {
                "topal.provider.x86_64.static-image.qualify-entropy-sample-quality/1"
            }
            Self::QualifyStaticImageEntropyPoolLifecycle => {
                "topal.provider.x86_64.static-image.qualify-entropy-pool-lifecycle/1"
            }
            Self::QualifyStaticImageEntropyGeneratorLifecycle => {
                "topal.provider.x86_64.static-image.qualify-entropy-generator-lifecycle/1"
            }
            Self::DeliverStaticImageEntropyInitialRandom => {
                "topal.provider.x86_64.static-image.deliver-entropy-initial-random/1"
            }
            Self::EnterStaticImageProcess => "topal.provider.x86_64.static-image.process-enter/1",
            Self::RestoreStaticImageProcessEntry => {
                "topal.provider.x86_64.static-image.process-restore-entry/1"
            }
            Self::RevokeStaticImageEntropyInitialRandom => {
                "topal.provider.x86_64.static-image.revoke-entropy-initial-random/1"
            }
            Self::RevokeStaticImageEntropyAcquisition => {
                "topal.provider.x86_64.static-image.revoke-entropy-acquisition/1"
            }
            Self::RevokeStaticImageEntropyHandoff => {
                "topal.provider.x86_64.static-image.revoke-entropy-handoff/1"
            }
            Self::RestoreStaticImagePreparedProcess => {
                "topal.provider.x86_64.static-image.restore-prepared-process/1"
            }
            Self::RevokeStaticImageStackRandomSlot => {
                "topal.provider.x86_64.static-image.revoke-stack-random-slot/1"
            }
            Self::RemoveStaticImageStackLoadMetadata => {
                "topal.provider.x86_64.static-image.remove-stack-load-metadata/1"
            }
            Self::RemoveStaticImageStackHardwareCapabilities => {
                "topal.provider.x86_64.static-image.remove-stack-hardware-capabilities/1"
            }
            Self::RemoveStaticImageStackPlatform => {
                "topal.provider.x86_64.static-image.remove-stack-platform/1"
            }
            Self::RemoveStaticImageStackClockTickRate => {
                "topal.provider.x86_64.static-image.remove-stack-clock-tick-rate/1"
            }
            Self::RemoveStaticImageStackSecureMode => {
                "topal.provider.x86_64.static-image.remove-stack-secure-mode/1"
            }
            Self::RemoveStaticImageStackCredentials => {
                "topal.provider.x86_64.static-image.remove-stack-credentials/1"
            }
            Self::RemoveStaticImageStackExecutableName => {
                "topal.provider.x86_64.static-image.remove-stack-executable-name/1"
            }
            Self::RemoveStaticImageStackProgramHeaders => {
                "topal.provider.x86_64.static-image.remove-stack-program-headers/1"
            }
            Self::RemoveStaticImageStackEntryFact => {
                "topal.provider.x86_64.static-image.remove-stack-entry-fact/1"
            }
            Self::RemoveStaticImageStackPageSize => {
                "topal.provider.x86_64.static-image.remove-stack-pagesz/1"
            }
            Self::ClearStaticImageStackEntry => {
                "topal.provider.x86_64.static-image.clear-stack-entry/1"
            }
            Self::RevokeStaticImageStackLeaf => {
                "topal.provider.x86_64.static-image.revoke-stack-leaf/1"
            }
            Self::CopyStaticImagePayload => "topal.provider.x86_64.static-image.copy-payload/1",
            Self::SealStaticImagePermissions => {
                "topal.provider.x86_64.static-image.seal-permissions/1"
            }
            Self::AttachStaticImageAddressSpace => {
                "topal.provider.x86_64.static-image.attach-address-space/1"
            }
            Self::ActivateStaticImageAddressSpace => {
                "topal.provider.x86_64.static-image.activate-address-space/1"
            }
            Self::RestoreStaticImageAddressSpace => {
                "topal.provider.x86_64.static-image.restore-address-space/1"
            }
            Self::AuthorizeStaticImageEntry => {
                "topal.provider.x86_64.static-image.authorize-entry/1"
            }
            Self::AuthorizeStaticImageStackSlot => {
                "topal.provider.x86_64.static-image.authorize-stack-slot/1"
            }
            Self::RevokeStaticImageStackSlot => {
                "topal.provider.x86_64.static-image.revoke-stack-slot/1"
            }
            Self::RevokeStaticImageEntry => "topal.provider.x86_64.static-image.revoke-entry/1",
            Self::DetachStaticImageAddressSpace => {
                "topal.provider.x86_64.static-image.detach-address-space/1"
            }
            Self::ReturnStaticImageHierarchy => {
                "topal.provider.x86_64.static-image.return-hierarchy/1"
            }
            Self::RevokeInitialStaticImageLeaves => {
                "topal.provider.x86_64.static-image.revoke-initial-leaves/1"
            }
            Self::ReturnStaticImagePayload => "topal.provider.x86_64.static-image.return-payload/1",
            Self::AdoptBootstrapIdentityMapping => {
                "topal.provider.x86_64.mapping.adopt-bootstrap-identity/1"
            }
            Self::PlainKernelMappingStoreByte => "topal.provider.x86_64.mapping.plain-store-byte/1",
            Self::PlainKernelMappingLoadByte => "topal.provider.x86_64.mapping.plain-load-byte/1",
            Self::ConsumeKernelMapping => "topal.provider.x86_64.mapping.consume/1",
            Self::SelectBootstrapTranslationBacking => {
                "topal.provider.x86_64.translation.select-backing/1"
            }
            Self::BuildBootstrapEquivalentTranslation => {
                "topal.provider.x86_64.translation.build-bootstrap-equivalent/1"
            }
            Self::ActivateTranslationRoot => "topal.provider.x86_64.translation.activate-root/1",
            Self::BeginActiveTranslationEdit => "topal.provider.x86_64.translation.edit.begin/1",
            Self::StageActiveTranslationMap => {
                "topal.provider.x86_64.translation.edit.map-kernel/1"
            }
            Self::StageActiveTranslationUnmap => {
                "topal.provider.x86_64.translation.edit.unmap-kernel/1"
            }
            Self::CommitActiveTranslationEdit => "topal.provider.x86_64.translation.edit.commit/1",
            Self::CaptureAndMaskLocalInterrupts => {
                "topal.provider.x86_64.critical.local-interrupts.enter/1"
            }
            Self::RestoreLocalInterruptState => {
                "topal.provider.x86_64.critical.local-interrupts.restore/1"
            }
            Self::InitializeAtomicWord => "topal.provider.x86_64.atomic.word.initialize/1",
            Self::LockedCompareExchangeWord => {
                "topal.provider.x86_64.atomic.word.locked-compare-exchange/1"
            }
            Self::LoadAtomicWordAcquire => "topal.provider.x86_64.atomic.word.load-acquire/1",
            Self::ConsumeAtomicWord => "topal.provider.x86_64.atomic.word.consume/1",
            Self::SendLocalApicNotification => {
                "topal.provider.x86_64.interrupt.local-apic.send-self/1"
            }
            Self::WaitForLocalNotification => "topal.provider.x86_64.interrupt.local-apic.wait/1",
            Self::CompleteLocalApicNotification => {
                "topal.provider.x86_64.interrupt.local-apic.complete/1"
            }
            Self::ResumeLocalNotification => "topal.provider.x86_64.interrupt.local-apic.resume/1",
            Self::ObserveHpetMonotonicClock => "topal.provider.x86_64.time.hpet-monotonic-now/1",
            Self::ConstructHpetDeadline => "topal.provider.x86_64.time.hpet-deadline-after/1",
            Self::ArmHpetIoApicDeadline => "topal.provider.x86_64.time.hpet-ioapic.arm/1",
            Self::WaitForDeadline => "topal.provider.x86_64.time.hpet-ioapic.wait/1",
            Self::CompleteHpetIoApicDeadline => "topal.provider.x86_64.time.hpet-ioapic.complete/1",
            Self::ResumeDeadline => "topal.provider.x86_64.time.hpet-ioapic.resume/1",
            Self::CreateKernelContext => "topal.provider.x86_64.context.create/1",
            Self::TransferKernelContextUntilDeadline => {
                "topal.provider.x86_64.context.transfer-until-hpet-deadline/1"
            }
            Self::AwaitDeadlinePreemption => {
                "topal.provider.x86_64.context.await-deadline-preemption/1"
            }
            Self::BlockOneShotCompletion => {
                "topal.provider.x86_64.context.block-one-shot-completion/1"
            }
            Self::PreemptCurrentKernelContext => "topal.provider.x86_64.context.preempt-current/1",
            Self::TransferKernelContext => "topal.provider.x86_64.context.transfer/1",
            Self::RetireKernelContext => "topal.provider.x86_64.context.retire/1",
            Self::ReclaimKernelContext => "topal.provider.x86_64.context.reclaim/1",
            Self::PrepareInitialUserAddressSpace => {
                "topal.provider.x86_64.user.prepare-initial-address-space/1"
            }
            Self::VerifyInitialUserMapping => "topal.provider.x86_64.user.verify-initial-mapping/1",
            Self::MapInitialUserFrame => "topal.provider.x86_64.user.map-initial-frame/1",
            Self::StoreInitialUserMappingByte => {
                "topal.provider.x86_64.user.store-initial-mapping-byte/1"
            }
            Self::InstallInitialUserEntryTrampoline => {
                "topal.provider.x86_64.user.install-initial-entry-trampoline/1"
            }
            Self::ProtectInitialUserMappingReadOnly => {
                "topal.provider.x86_64.user.protect-initial-mapping-read-only/1"
            }
            Self::LoadInitialUserMappingByte => {
                "topal.provider.x86_64.user.load-initial-mapping-byte/1"
            }
            Self::VerifyInitialUserEntryTrampoline => {
                "topal.provider.x86_64.user.verify-initial-entry-trampoline/1"
            }
            Self::PermitInitialUserMappingExecute => {
                "topal.provider.x86_64.user.permit-initial-mapping-execute/1"
            }
            Self::SealInitialUserMappingImmutableData => {
                "topal.provider.x86_64.user.seal-initial-mapping-immutable-data/1"
            }
            Self::PermitInitialUserMappingWritableData => {
                "topal.provider.x86_64.user.permit-initial-mapping-writable-data/1"
            }
            Self::ReserveInitialAnonymousUserMapping => {
                "topal.provider.x86_64.user.reserve-initial-anonymous-mapping/1"
            }
            Self::ReleaseInitialAnonymousUserMapping => {
                "topal.provider.x86_64.user.release-initial-anonymous-mapping/1"
            }
            Self::UnmapInitialUserFrame => "topal.provider.x86_64.user.unmap-initial-frame/1",
            Self::PrepareInitialUserProbe => "topal.provider.x86_64.user.prepare-initial-probe/1",
            Self::EnterInitialUserContext => "topal.provider.x86_64.user.enter-initial-context/1",
            Self::CompleteInitialUserBacking => {
                "topal.provider.x86_64.user.complete-initial-backing/1"
            }
            Self::ReclaimInitialUserAddressSpace => {
                "topal.provider.x86_64.user.reclaim-initial-address-space/1"
            }
            Self::CopyInitialUnameToUser => {
                "topal.provider.x86_64.user.copy-to-with-page-fault-recovery/1"
            }
            Self::CopyInitialTimespecFromUser => {
                "topal.provider.x86_64.user.copy-from-with-page-fault-recovery/1"
            }
            Self::DispatchInitialUserSyscall => {
                "topal.provider.x86_64.user.dispatch-initial-syscall/1"
            }
            Self::StaticBootstrapStorage => "topal.provider.x86_64.storage.static-nobits/1",
            Self::MonotonicBootstrapAllocate => {
                "topal.provider.x86_64.storage.monotonic-allocate/1"
            }
            Self::PlainBootstrapRegionStoreByte => {
                "topal.provider.x86_64.storage.plain-store-byte/1"
            }
            Self::PlainBootstrapRegionLoadByte => "topal.provider.x86_64.storage.plain-load-byte/1",
            Self::ConsumeBootstrapRegion => "topal.provider.x86_64.storage.consume-region/1",
            Self::CompleteBootstrapStorage => "topal.provider.x86_64.storage.complete-bootstrap/1",
            Self::StaticCoherentDmaBuffer => "topal.provider.x86_64.dma.static-coherent-buffer/1",
            Self::BindIdentityTranslatedDmaReference => {
                "topal.provider.x86_64.dma.bind-identity-reference/1"
            }
            Self::StoreCpuOwnedDmaInteger => "topal.provider.x86_64.dma.store-cpu-owned-integer/1",
            Self::PrepareCoherentDmaForDevice => {
                "topal.provider.x86_64.dma.prepare-coherent-device-ownership/1"
            }
            Self::CompleteCoherentDmaForCpu => {
                "topal.provider.x86_64.dma.complete-coherent-cpu-ownership/1"
            }
            Self::LoadCpuOwnedDmaInteger => "topal.provider.x86_64.dma.load-cpu-owned-integer/1",
            Self::CaptureOpaqueRawEntropySample => {
                "topal.provider.x86_64.entropy.capture-opaque-raw-sample/1"
            }
            Self::MixBlake2sEntropyPoolSample => {
                "topal.provider.x86_64.entropy.mix-blake2s-pool-sample/1"
            }
            Self::InitializeChaCha20EntropyGenerator => {
                "topal.provider.x86_64.entropy.initialize-chacha20-generator/1"
            }
            Self::ExtractChaCha20EntropyGenerator => {
                "topal.provider.x86_64.entropy.extract-chacha20-generator/1"
            }
            Self::DeliverInitialAuxvRandom => {
                "topal.provider.x86_64.entropy.deliver-initial-auxv-random/1"
            }
            Self::ReserveInitialUserRandom => {
                "topal.provider.x86_64.entropy.reserve-initial-user-random/1"
            }
            Self::BindReusableUserRandom => {
                "topal.provider.x86_64.entropy.bind-reusable-user-random/1"
            }
            Self::PciConfigurationMechanismOneReadDword => {
                "topal.provider.x86_64.pci-configuration.mechanism-one-read-dword/1"
            }
            Self::PciConfigurationMechanismOneUpdateWord => {
                "topal.provider.x86_64.pci-configuration.mechanism-one-update-word/1"
            }
            Self::PciConfigurationGatewayRegionRead => {
                "topal.provider.x86_64.pci-region.configuration-gateway-read/1"
            }
            Self::PciConfigurationGatewayRegionReadEventually => {
                "topal.provider.x86_64.pci-region.configuration-gateway-read-eventually/1"
            }
            Self::PciConfigurationGatewayRegionWrite => {
                "topal.provider.x86_64.pci-region.configuration-gateway-write/1"
            }
            Self::PciConfigurationGatewayDmaReferenceWrite => {
                "topal.provider.x86_64.pci-region.configuration-gateway-dma-reference-write/1"
            }
            Self::PolledUart16550PortIo => "topal.provider.x86_64.uart16550.polled-port-io/1",
            Self::BreakpointVector3 => "topal.provider.x86_64.exception.breakpoint-vector-3/1",
            Self::InterruptReturn => "topal.provider.x86_64.exception.interrupt-return/1",
            Self::InterruptsDisabledHalt => {
                "topal.provider.x86_64.fatal.interrupts-disabled-halt/1"
            }
        }
    }

    #[must_use]
    pub const fn requires_privileged_machine_state(self) -> bool {
        matches!(
            self,
            Self::PciConfigurationMechanismOneReadDword
                | Self::PciConfigurationMechanismOneUpdateWord
                | Self::PciConfigurationGatewayRegionRead
                | Self::PciConfigurationGatewayRegionReadEventually
                | Self::PciConfigurationGatewayRegionWrite
                | Self::PolledUart16550PortIo
                | Self::ActivateTranslationRoot
                | Self::CommitActiveTranslationEdit
                | Self::CaptureAndMaskLocalInterrupts
                | Self::RestoreLocalInterruptState
                | Self::SendLocalApicNotification
                | Self::WaitForLocalNotification
                | Self::CompleteLocalApicNotification
                | Self::ResumeLocalNotification
                | Self::ObserveHpetMonotonicClock
                | Self::ArmHpetIoApicDeadline
                | Self::WaitForDeadline
                | Self::CompleteHpetIoApicDeadline
                | Self::ResumeDeadline
                | Self::TransferKernelContextUntilDeadline
                | Self::AwaitDeadlinePreemption
                | Self::BlockOneShotCompletion
                | Self::PreemptCurrentKernelContext
                | Self::PrepareInitialUserAddressSpace
                | Self::VerifyInitialUserMapping
                | Self::MapInitialUserFrame
                | Self::StoreInitialUserMappingByte
                | Self::InstallInitialUserEntryTrampoline
                | Self::ProtectInitialUserMappingReadOnly
                | Self::LoadInitialUserMappingByte
                | Self::VerifyInitialUserEntryTrampoline
                | Self::PermitInitialUserMappingExecute
                | Self::SealInitialUserMappingImmutableData
                | Self::PermitInitialUserMappingWritableData
                | Self::ReserveInitialAnonymousUserMapping
                | Self::ReleaseInitialAnonymousUserMapping
                | Self::UnmapInitialUserFrame
                | Self::PrepareInitialUserProbe
                | Self::EnterInitialUserContext
                | Self::CompleteInitialUserBacking
                | Self::ReclaimInitialUserAddressSpace
                | Self::CopyInitialUnameToUser
                | Self::CopyInitialTimespecFromUser
                | Self::DispatchInitialUserSyscall
                | Self::BindReusableUserRandom
                | Self::InterruptReturn
                | Self::InterruptsDisabledHalt
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsProviderOperationPlan {
    pub semantic_identity: &'static str,
    pub lowering: X86SystemsLowering,
}

impl SystemsProviderOperationPlan {
    #[must_use]
    pub const fn provider_identity(&self) -> &'static str {
        self.lowering.provider_identity()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemsBootstrapPlacementPlan {
    pub capacity_bytes: u64,
    pub alignment_bytes: u64,
    pub semantic_placement: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct X86SystemsProviderPlan {
    pub revision: &'static str,
    pub target: &'static str,
    pub board: &'static str,
    pub profile: &'static str,
    pub machine_cpu_model: &'static str,
    pub codegen_cpu: &'static str,
    pub platform_abi: &'static str,
    pub data_layout: &'static str,
    pub object_format: &'static str,
    pub relocation_model: &'static str,
    pub code_model: &'static str,
    pub bootstrap_placement: SystemsBootstrapPlacementPlan,
    pub operations: Vec<SystemsProviderOperationPlan>,
}

/// Derive the sealed x86-64 provider plan without executable-qualifying it.
///
/// # Errors
///
/// Returns a tool error when the checked program is outside the initial
/// systems profile or contains an operation without a sealed provider mapping.
pub fn plan_x86_64_systems_provider(
    program: &CompilerSystemsProgram,
) -> Result<X86SystemsProviderPlan, CompileError> {
    validate_systems_program(program)
        .map_err(|error| CompileError::Tool(format!("invalid systems program: {error}")))?;

    let mut semantic_identities = program
        .bootstrap
        .handler
        .effects
        .iter()
        .chain(&program.debug_break.handler.effects)
        .chain(&program.local_notification.handler.effects)
        .chain(&program.deadline_notification.handler.effects)
        .chain(&program.kernel_thread.handler.effects)
        .chain(&program.completion_thread.handler.effects)
        .chain(&program.terminal_thread.handler.effects)
        .chain(&program.user_syscall.handler.effects)
        .map(String::as_str)
        .collect::<Vec<_>>();
    semantic_identities.extend([
        SYSTEMS_BOOTSTRAP_STORAGE_PROVISION,
        SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
        SYSTEMS_BOOTSTRAP_STORAGE_RELEASE,
        SYSTEMS_BOOTSTRAP_STORAGE_COMPLETE,
    ]);
    semantic_identities.sort_unstable();
    semantic_identities.dedup();
    semantic_identities.retain(|identity| {
        !matches!(
            *identity,
            SYSTEMS_KERNEL_RUNNABLE_QUEUE_CREATE
                | SYSTEMS_KERNEL_RUNNABLE_QUEUE_ENQUEUE
                | SYSTEMS_KERNEL_RUNNABLE_QUEUE_DEQUEUE
                | SYSTEMS_KERNEL_RUNNABLE_QUEUE_CONSUME
                | SYSTEMS_KERNEL_CONTEXT_TAKE_PREEMPTED
                | SYSTEMS_KERNEL_COMPLETION_CREATE
                | SYSTEMS_KERNEL_COMPLETION_SIGNAL
                | SYSTEMS_KERNEL_COMPLETION_CONSUME
                | SYSTEMS_ENTROPY_SAMPLE_ASSESS_QUALITY
                | SYSTEMS_ENTROPY_POOL_CREATE
                | SYSTEMS_ENTROPY_POOL_CREDIT_CONTRIBUTION
                | SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS
                | SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS
        )
    });

    let operations = semantic_identities
        .into_iter()
        .map(provider_operation)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(X86SystemsProviderPlan {
        revision: X86_SYSTEMS_PROVIDER_REVISION,
        target: INITIAL_SYSTEMS_TARGET,
        board: INITIAL_SYSTEMS_BOARD,
        profile: INITIAL_SYSTEMS_PROFILE,
        machine_cpu_model: "qemu64-v1",
        codegen_cpu: "x86-64",
        platform_abi: X86_SYSTEMS_PLATFORM_ABI,
        data_layout: X86_SYSTEMS_DATA_LAYOUT,
        object_format: "elf64-x86-64",
        relocation_model: "static",
        code_model: "small",
        bootstrap_placement: SystemsBootstrapPlacementPlan {
            capacity_bytes: program.bootstrap_storage.capacity_bytes,
            alignment_bytes: program.bootstrap_storage.alignment_bytes,
            semantic_placement: "topal.systems.placement.bootstrap-reclaimable/1",
        },
        operations,
    })
}

fn provider_operation(identity: &str) -> Result<SystemsProviderOperationPlan, CompileError> {
    let lowering = match identity {
        SYSTEMS_BOOT_MEMORY_DESCRIBE => X86SystemsLowering::LinuxBootParamsE820,
        SYSTEMS_FRAME_ALLOCATOR_CREATE => X86SystemsLowering::RetainedE820FrameAuthority,
        SYSTEMS_FRAMES_ALLOCATE => X86SystemsLowering::E820StaticFrameExtentSelect,
        SYSTEMS_FRAMES_RELEASE => X86SystemsLowering::ConsumePhysicalFrameExtent,
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PREPARE => {
            X86SystemsLowering::ConstructInitialStaticImageLeaves
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_PAYLOAD => {
            X86SystemsLowering::ZeroStaticImagePayload
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_STACK_BACKING => {
            X86SystemsLowering::ZeroStaticImageStackBacking
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PUBLISH_STACK_LEAF => {
            X86SystemsLowering::PublishStaticImageStackLeaf
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_INITIALIZE_STACK_ENTRY => {
            X86SystemsLowering::InitializeStaticImageStackEntry
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PAGESZ => {
            X86SystemsLowering::AddStaticImageStackPageSize
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_ENTRY_FACT => {
            X86SystemsLowering::AddStaticImageStackEntryFact
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PROGRAM_HEADERS => {
            X86SystemsLowering::AddStaticImageStackProgramHeaders
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_EXECUTABLE_NAME => {
            X86SystemsLowering::AddStaticImageStackExecutableName
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_CREDENTIALS => {
            X86SystemsLowering::AddStaticImageStackCredentials
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_SECURE_MODE => {
            X86SystemsLowering::AddStaticImageStackSecureMode
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_CLOCK_TICK_RATE => {
            X86SystemsLowering::AddStaticImageStackClockTickRate
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PLATFORM => {
            X86SystemsLowering::AddStaticImageStackPlatform
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_HARDWARE_CAPABILITIES => {
            X86SystemsLowering::AddStaticImageStackHardwareCapabilities
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_LOAD_METADATA => {
            X86SystemsLowering::AddStaticImageStackLoadMetadata
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_RANDOM_SLOT => {
            X86SystemsLowering::AuthorizeStaticImageStackRandomSlot
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RETAIN_PREPARED_PROCESS => {
            X86SystemsLowering::RetainStaticImagePreparedProcess
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_RETAINED_DISJOINT_WORK => {
            X86SystemsLowering::QualifyStaticImageRetainedDisjointWork
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTROPY_HANDOFF => {
            X86SystemsLowering::AuthorizeStaticImageEntropyHandoff
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTROPY_ACQUISITION => {
            X86SystemsLowering::AuthorizeStaticImageEntropyAcquisition
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DEVICE => {
            X86SystemsLowering::QualifyStaticImageEntropyDevice
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_TRANSPORT => {
            X86SystemsLowering::QualifyStaticImageEntropyTransport
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_NEGOTIATION => {
            X86SystemsLowering::QualifyStaticImageEntropyNegotiation
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE => {
            X86SystemsLowering::QualifyStaticImageEntropyQueue
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_BUS_MASTER => {
            X86SystemsLowering::QualifyStaticImageEntropyBusMaster
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DMA_BUFFERS => {
            X86SystemsLowering::QualifyStaticImageEntropyDmaBuffers
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SPLIT_RING => {
            X86SystemsLowering::QualifyStaticImageEntropySplitRing
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DMA_PREPARATION => {
            X86SystemsLowering::QualifyStaticImageEntropyDmaPreparation
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE_ADDRESSES => {
            X86SystemsLowering::QualifyStaticImageEntropyQueueAddresses
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE_ACTIVATION => {
            X86SystemsLowering::QualifyStaticImageEntropyQueueActivation
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_REQUEST_COMPLETION => {
            X86SystemsLowering::QualifyStaticImageEntropyRequestCompletion
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SAMPLE_CAPTURE => {
            X86SystemsLowering::QualifyStaticImageEntropySampleCapture
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SAMPLE_QUALITY => {
            X86SystemsLowering::QualifyStaticImageEntropySampleQuality
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_POOL_LIFECYCLE => {
            X86SystemsLowering::QualifyStaticImageEntropyPoolLifecycle
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_GENERATOR_LIFECYCLE => {
            X86SystemsLowering::QualifyStaticImageEntropyGeneratorLifecycle
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_DELIVER_ENTROPY_INITIAL_RANDOM => {
            X86SystemsLowering::DeliverStaticImageEntropyInitialRandom
        }
        SYSTEMS_STATIC_IMAGE_PROCESS_ENTER => X86SystemsLowering::EnterStaticImageProcess,
        SYSTEMS_STATIC_IMAGE_PROCESS_RESTORE_ENTRY => {
            X86SystemsLowering::RestoreStaticImageProcessEntry
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_INITIAL_RANDOM => {
            X86SystemsLowering::RevokeStaticImageEntropyInitialRandom
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_ACQUISITION => {
            X86SystemsLowering::RevokeStaticImageEntropyAcquisition
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_HANDOFF => {
            X86SystemsLowering::RevokeStaticImageEntropyHandoff
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESTORE_PREPARED_PROCESS => {
            X86SystemsLowering::RestoreStaticImagePreparedProcess
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_RANDOM_SLOT => {
            X86SystemsLowering::RevokeStaticImageStackRandomSlot
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_LOAD_METADATA => {
            X86SystemsLowering::RemoveStaticImageStackLoadMetadata
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_HARDWARE_CAPABILITIES => {
            X86SystemsLowering::RemoveStaticImageStackHardwareCapabilities
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PLATFORM => {
            X86SystemsLowering::RemoveStaticImageStackPlatform
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_CLOCK_TICK_RATE => {
            X86SystemsLowering::RemoveStaticImageStackClockTickRate
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_SECURE_MODE => {
            X86SystemsLowering::RemoveStaticImageStackSecureMode
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_CREDENTIALS => {
            X86SystemsLowering::RemoveStaticImageStackCredentials
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_EXECUTABLE_NAME => {
            X86SystemsLowering::RemoveStaticImageStackExecutableName
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PROGRAM_HEADERS => {
            X86SystemsLowering::RemoveStaticImageStackProgramHeaders
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_ENTRY_FACT => {
            X86SystemsLowering::RemoveStaticImageStackEntryFact
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PAGESZ => {
            X86SystemsLowering::RemoveStaticImageStackPageSize
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_CLEAR_STACK_ENTRY => {
            X86SystemsLowering::ClearStaticImageStackEntry
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_LEAF => {
            X86SystemsLowering::RevokeStaticImageStackLeaf
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_COPY_PAYLOAD => {
            X86SystemsLowering::CopyStaticImagePayload
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_SEAL_PERMISSIONS => {
            X86SystemsLowering::SealStaticImagePermissions
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ATTACH_ADDRESS_SPACE => {
            X86SystemsLowering::AttachStaticImageAddressSpace
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ACTIVATE_ADDRESS_SPACE => {
            X86SystemsLowering::ActivateStaticImageAddressSpace
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESTORE_ADDRESS_SPACE => {
            X86SystemsLowering::RestoreStaticImageAddressSpace
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTRY => {
            X86SystemsLowering::AuthorizeStaticImageEntry
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_SLOT => {
            X86SystemsLowering::AuthorizeStaticImageStackSlot
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_SLOT => {
            X86SystemsLowering::RevokeStaticImageStackSlot
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTRY => {
            X86SystemsLowering::RevokeStaticImageEntry
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_DETACH_ADDRESS_SPACE => {
            X86SystemsLowering::DetachStaticImageAddressSpace
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_HIERARCHY => {
            X86SystemsLowering::ReturnStaticImageHierarchy
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_METADATA => {
            X86SystemsLowering::RevokeInitialStaticImageLeaves
        }
        SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_PAYLOAD => {
            X86SystemsLowering::ReturnStaticImagePayload
        }
        SYSTEMS_KERNEL_MAP => X86SystemsLowering::AdoptBootstrapIdentityMapping,
        SYSTEMS_KERNEL_MAPPING_STORE_BYTE => X86SystemsLowering::PlainKernelMappingStoreByte,
        SYSTEMS_KERNEL_MAPPING_LOAD_BYTE => X86SystemsLowering::PlainKernelMappingLoadByte,
        SYSTEMS_KERNEL_UNMAP => X86SystemsLowering::ConsumeKernelMapping,
        SYSTEMS_TRANSLATION_BEGIN => X86SystemsLowering::SelectBootstrapTranslationBacking,
        SYSTEMS_TRANSLATION_COMMIT => X86SystemsLowering::BuildBootstrapEquivalentTranslation,
        SYSTEMS_TRANSLATION_ACTIVATE => X86SystemsLowering::ActivateTranslationRoot,
        SYSTEMS_TRANSLATION_EDIT_BEGIN => X86SystemsLowering::BeginActiveTranslationEdit,
        SYSTEMS_TRANSLATION_EDIT_MAP => X86SystemsLowering::StageActiveTranslationMap,
        SYSTEMS_TRANSLATION_EDIT_UNMAP => X86SystemsLowering::StageActiveTranslationUnmap,
        SYSTEMS_TRANSLATION_EDIT_COMMIT => X86SystemsLowering::CommitActiveTranslationEdit,
        SYSTEMS_CRITICAL_ENTER => X86SystemsLowering::CaptureAndMaskLocalInterrupts,
        SYSTEMS_CRITICAL_RESTORE => X86SystemsLowering::RestoreLocalInterruptState,
        SYSTEMS_ATOMIC_WORD_CREATE => X86SystemsLowering::InitializeAtomicWord,
        SYSTEMS_ATOMIC_COMPARE_EXCHANGE => X86SystemsLowering::LockedCompareExchangeWord,
        SYSTEMS_ATOMIC_LOAD => X86SystemsLowering::LoadAtomicWordAcquire,
        SYSTEMS_ATOMIC_END => X86SystemsLowering::ConsumeAtomicWord,
        SYSTEMS_LOCAL_NOTIFICATION_SEND => X86SystemsLowering::SendLocalApicNotification,
        SYSTEMS_LOCAL_NOTIFICATION_WAIT => X86SystemsLowering::WaitForLocalNotification,
        SYSTEMS_LOCAL_NOTIFICATION_COMPLETE => X86SystemsLowering::CompleteLocalApicNotification,
        SYSTEMS_RESUME_LOCAL_NOTIFICATION => X86SystemsLowering::ResumeLocalNotification,
        SYSTEMS_MONOTONIC_CLOCK_NOW => X86SystemsLowering::ObserveHpetMonotonicClock,
        SYSTEMS_DEADLINE_AFTER => X86SystemsLowering::ConstructHpetDeadline,
        SYSTEMS_DEADLINE_ARM => X86SystemsLowering::ArmHpetIoApicDeadline,
        SYSTEMS_DEADLINE_WAIT => X86SystemsLowering::WaitForDeadline,
        SYSTEMS_DEADLINE_COMPLETE => X86SystemsLowering::CompleteHpetIoApicDeadline,
        SYSTEMS_RESUME_DEADLINE => X86SystemsLowering::ResumeDeadline,
        SYSTEMS_KERNEL_CONTEXT_CREATE => X86SystemsLowering::CreateKernelContext,
        SYSTEMS_KERNEL_CONTEXT_TRANSFER_UNTIL_DEADLINE => {
            X86SystemsLowering::TransferKernelContextUntilDeadline
        }
        SYSTEMS_KERNEL_CONTEXT_AWAIT_DEADLINE_PREEMPTION => {
            X86SystemsLowering::AwaitDeadlinePreemption
        }
        SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE => X86SystemsLowering::BlockOneShotCompletion,
        SYSTEMS_KERNEL_CONTEXT_PREEMPT_CURRENT => X86SystemsLowering::PreemptCurrentKernelContext,
        SYSTEMS_KERNEL_CONTEXT_TRANSFER => X86SystemsLowering::TransferKernelContext,
        SYSTEMS_KERNEL_CONTEXT_RETIRE => X86SystemsLowering::RetireKernelContext,
        SYSTEMS_KERNEL_CONTEXT_RECLAIM => X86SystemsLowering::ReclaimKernelContext,
        SYSTEMS_USER_ADDRESS_SPACE_PREPARE => X86SystemsLowering::PrepareInitialUserAddressSpace,
        SYSTEMS_USER_ADDRESS_SPACE_VERIFY_MAPPING => X86SystemsLowering::VerifyInitialUserMapping,
        SYSTEMS_USER_ADDRESS_SPACE_MAP => X86SystemsLowering::MapInitialUserFrame,
        SYSTEMS_USER_MAPPING_STORE_BYTE => X86SystemsLowering::StoreInitialUserMappingByte,
        SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE => {
            X86SystemsLowering::InstallInitialUserEntryTrampoline
        }
        SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY => {
            X86SystemsLowering::ProtectInitialUserMappingReadOnly
        }
        SYSTEMS_USER_MAPPING_LOAD_BYTE => X86SystemsLowering::LoadInitialUserMappingByte,
        SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE => {
            X86SystemsLowering::VerifyInitialUserEntryTrampoline
        }
        SYSTEMS_USER_MAPPING_PERMIT_EXECUTE => X86SystemsLowering::PermitInitialUserMappingExecute,
        SYSTEMS_USER_MAPPING_SEAL_IMMUTABLE_DATA => {
            X86SystemsLowering::SealInitialUserMappingImmutableData
        }
        SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA => {
            X86SystemsLowering::PermitInitialUserMappingWritableData
        }
        SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS => {
            X86SystemsLowering::ReserveInitialAnonymousUserMapping
        }
        SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS => {
            X86SystemsLowering::ReleaseInitialAnonymousUserMapping
        }
        SYSTEMS_USER_ADDRESS_SPACE_UNMAP => X86SystemsLowering::UnmapInitialUserFrame,
        SYSTEMS_USER_CONTEXT_PREPARE => X86SystemsLowering::PrepareInitialUserProbe,
        SYSTEMS_USER_CONTEXT_ENTER => X86SystemsLowering::EnterInitialUserContext,
        SYSTEMS_USER_CONTEXT_COMPLETE => X86SystemsLowering::CompleteInitialUserBacking,
        SYSTEMS_USER_ADDRESS_SPACE_RECLAIM => X86SystemsLowering::ReclaimInitialUserAddressSpace,
        SYSTEMS_USER_COPY_TO => X86SystemsLowering::CopyInitialUnameToUser,
        SYSTEMS_USER_COPY_FROM => X86SystemsLowering::CopyInitialTimespecFromUser,
        SYSTEMS_USER_SYSCALL_DISPATCH => X86SystemsLowering::DispatchInitialUserSyscall,
        SYSTEMS_BOOTSTRAP_STORAGE_PROVISION => X86SystemsLowering::StaticBootstrapStorage,
        SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE => X86SystemsLowering::MonotonicBootstrapAllocate,
        SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE => X86SystemsLowering::PlainBootstrapRegionStoreByte,
        SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE => X86SystemsLowering::PlainBootstrapRegionLoadByte,
        SYSTEMS_BOOTSTRAP_STORAGE_RELEASE => X86SystemsLowering::ConsumeBootstrapRegion,
        SYSTEMS_BOOTSTRAP_STORAGE_COMPLETE => X86SystemsLowering::CompleteBootstrapStorage,
        SYSTEMS_DMA_BUFFER_CREATE => X86SystemsLowering::StaticCoherentDmaBuffer,
        SYSTEMS_DMA_BUFFER_BIND_REFERENCE => X86SystemsLowering::BindIdentityTranslatedDmaReference,
        SYSTEMS_DMA_BUFFER_STORE_INTEGER => X86SystemsLowering::StoreCpuOwnedDmaInteger,
        SYSTEMS_DMA_BUFFER_PREPARE_DEVICE => X86SystemsLowering::PrepareCoherentDmaForDevice,
        SYSTEMS_DMA_BUFFER_COMPLETE_CPU => X86SystemsLowering::CompleteCoherentDmaForCpu,
        SYSTEMS_DMA_BUFFER_LOAD_INTEGER => X86SystemsLowering::LoadCpuOwnedDmaInteger,
        SYSTEMS_ENTROPY_SAMPLE_CAPTURE_DMA => X86SystemsLowering::CaptureOpaqueRawEntropySample,
        SYSTEMS_ENTROPY_POOL_MIX_SAMPLE => X86SystemsLowering::MixBlake2sEntropyPoolSample,
        SYSTEMS_ENTROPY_GENERATOR_INITIALIZE => {
            X86SystemsLowering::InitializeChaCha20EntropyGenerator
        }
        SYSTEMS_ENTROPY_GENERATOR_EXTRACT => X86SystemsLowering::ExtractChaCha20EntropyGenerator,
        SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM => {
            X86SystemsLowering::DeliverInitialAuxvRandom
        }
        SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM => {
            X86SystemsLowering::ReserveInitialUserRandom
        }
        SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE => X86SystemsLowering::BindReusableUserRandom,
        SYSTEMS_PCI_CONFIGURATION_READ_DWORD => {
            X86SystemsLowering::PciConfigurationMechanismOneReadDword
        }
        SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD => {
            X86SystemsLowering::PciConfigurationMechanismOneUpdateWord
        }
        SYSTEMS_PCI_REGION_READ => X86SystemsLowering::PciConfigurationGatewayRegionRead,
        SYSTEMS_PCI_REGION_READ_EVENTUALLY => {
            X86SystemsLowering::PciConfigurationGatewayRegionReadEventually
        }
        SYSTEMS_PCI_REGION_WRITE => X86SystemsLowering::PciConfigurationGatewayRegionWrite,
        SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE => {
            X86SystemsLowering::PciConfigurationGatewayDmaReferenceWrite
        }
        SYSTEMS_CONSOLE_WRITE => X86SystemsLowering::PolledUart16550PortIo,
        SYSTEMS_DEBUG_BREAK => X86SystemsLowering::BreakpointVector3,
        SYSTEMS_RESUME_DEBUG_BREAK => X86SystemsLowering::InterruptReturn,
        SYSTEMS_FATAL => X86SystemsLowering::InterruptsDisabledHalt,
        _ => {
            return Err(CompileError::Tool(format!(
                "systems operation `{identity}` has no mapping in provider `{X86_SYSTEMS_PROVIDER_REVISION}`"
            )));
        }
    };
    Ok(SystemsProviderOperationPlan {
        semantic_identity: match lowering {
            X86SystemsLowering::LinuxBootParamsE820 => SYSTEMS_BOOT_MEMORY_DESCRIBE,
            X86SystemsLowering::RetainedE820FrameAuthority => SYSTEMS_FRAME_ALLOCATOR_CREATE,
            X86SystemsLowering::E820StaticFrameExtentSelect => SYSTEMS_FRAMES_ALLOCATE,
            X86SystemsLowering::ConsumePhysicalFrameExtent => SYSTEMS_FRAMES_RELEASE,
            X86SystemsLowering::ConstructInitialStaticImageLeaves => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PREPARE
            }
            X86SystemsLowering::ZeroStaticImagePayload => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_PAYLOAD
            }
            X86SystemsLowering::ZeroStaticImageStackBacking => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_STACK_BACKING
            }
            X86SystemsLowering::PublishStaticImageStackLeaf => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PUBLISH_STACK_LEAF
            }
            X86SystemsLowering::InitializeStaticImageStackEntry => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_INITIALIZE_STACK_ENTRY
            }
            X86SystemsLowering::AddStaticImageStackPageSize => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PAGESZ
            }
            X86SystemsLowering::AddStaticImageStackEntryFact => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_ENTRY_FACT
            }
            X86SystemsLowering::AddStaticImageStackProgramHeaders => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PROGRAM_HEADERS
            }
            X86SystemsLowering::AddStaticImageStackExecutableName => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_EXECUTABLE_NAME
            }
            X86SystemsLowering::AddStaticImageStackCredentials => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_CREDENTIALS
            }
            X86SystemsLowering::AddStaticImageStackSecureMode => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_SECURE_MODE
            }
            X86SystemsLowering::AddStaticImageStackClockTickRate => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_CLOCK_TICK_RATE
            }
            X86SystemsLowering::AddStaticImageStackPlatform => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PLATFORM
            }
            X86SystemsLowering::AddStaticImageStackHardwareCapabilities => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_HARDWARE_CAPABILITIES
            }
            X86SystemsLowering::AddStaticImageStackLoadMetadata => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_LOAD_METADATA
            }
            X86SystemsLowering::AuthorizeStaticImageStackRandomSlot => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_RANDOM_SLOT
            }
            X86SystemsLowering::RetainStaticImagePreparedProcess => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RETAIN_PREPARED_PROCESS
            }
            X86SystemsLowering::QualifyStaticImageRetainedDisjointWork => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_RETAINED_DISJOINT_WORK
            }
            X86SystemsLowering::AuthorizeStaticImageEntropyHandoff => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTROPY_HANDOFF
            }
            X86SystemsLowering::AuthorizeStaticImageEntropyAcquisition => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTROPY_ACQUISITION
            }
            X86SystemsLowering::QualifyStaticImageEntropyDevice => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DEVICE
            }
            X86SystemsLowering::QualifyStaticImageEntropyTransport => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_TRANSPORT
            }
            X86SystemsLowering::QualifyStaticImageEntropyNegotiation => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_NEGOTIATION
            }
            X86SystemsLowering::QualifyStaticImageEntropyQueue => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE
            }
            X86SystemsLowering::QualifyStaticImageEntropyBusMaster => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_BUS_MASTER
            }
            X86SystemsLowering::QualifyStaticImageEntropyDmaBuffers => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DMA_BUFFERS
            }
            X86SystemsLowering::QualifyStaticImageEntropySplitRing => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SPLIT_RING
            }
            X86SystemsLowering::QualifyStaticImageEntropyDmaPreparation => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_DMA_PREPARATION
            }
            X86SystemsLowering::QualifyStaticImageEntropyQueueAddresses => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE_ADDRESSES
            }
            X86SystemsLowering::QualifyStaticImageEntropyQueueActivation => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE_ACTIVATION
            }
            X86SystemsLowering::QualifyStaticImageEntropyRequestCompletion => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_REQUEST_COMPLETION
            }
            X86SystemsLowering::QualifyStaticImageEntropySampleCapture => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SAMPLE_CAPTURE
            }
            X86SystemsLowering::QualifyStaticImageEntropySampleQuality => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_SAMPLE_QUALITY
            }
            X86SystemsLowering::QualifyStaticImageEntropyPoolLifecycle => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_POOL_LIFECYCLE
            }
            X86SystemsLowering::QualifyStaticImageEntropyGeneratorLifecycle => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_GENERATOR_LIFECYCLE
            }
            X86SystemsLowering::DeliverStaticImageEntropyInitialRandom => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_DELIVER_ENTROPY_INITIAL_RANDOM
            }
            X86SystemsLowering::EnterStaticImageProcess => SYSTEMS_STATIC_IMAGE_PROCESS_ENTER,
            X86SystemsLowering::RestoreStaticImageProcessEntry => {
                SYSTEMS_STATIC_IMAGE_PROCESS_RESTORE_ENTRY
            }
            X86SystemsLowering::RevokeStaticImageEntropyInitialRandom => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_INITIAL_RANDOM
            }
            X86SystemsLowering::RevokeStaticImageEntropyAcquisition => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_ACQUISITION
            }
            X86SystemsLowering::RevokeStaticImageEntropyHandoff => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTROPY_HANDOFF
            }
            X86SystemsLowering::RestoreStaticImagePreparedProcess => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESTORE_PREPARED_PROCESS
            }
            X86SystemsLowering::RevokeStaticImageStackRandomSlot => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_RANDOM_SLOT
            }
            X86SystemsLowering::RemoveStaticImageStackLoadMetadata => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_LOAD_METADATA
            }
            X86SystemsLowering::RemoveStaticImageStackHardwareCapabilities => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_HARDWARE_CAPABILITIES
            }
            X86SystemsLowering::RemoveStaticImageStackPlatform => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PLATFORM
            }
            X86SystemsLowering::RemoveStaticImageStackClockTickRate => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_CLOCK_TICK_RATE
            }
            X86SystemsLowering::RemoveStaticImageStackSecureMode => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_SECURE_MODE
            }
            X86SystemsLowering::RemoveStaticImageStackCredentials => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_CREDENTIALS
            }
            X86SystemsLowering::RemoveStaticImageStackExecutableName => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_EXECUTABLE_NAME
            }
            X86SystemsLowering::RemoveStaticImageStackProgramHeaders => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PROGRAM_HEADERS
            }
            X86SystemsLowering::RemoveStaticImageStackEntryFact => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_ENTRY_FACT
            }
            X86SystemsLowering::RemoveStaticImageStackPageSize => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PAGESZ
            }
            X86SystemsLowering::ClearStaticImageStackEntry => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_CLEAR_STACK_ENTRY
            }
            X86SystemsLowering::RevokeStaticImageStackLeaf => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_LEAF
            }
            X86SystemsLowering::CopyStaticImagePayload => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_COPY_PAYLOAD
            }
            X86SystemsLowering::SealStaticImagePermissions => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_SEAL_PERMISSIONS
            }
            X86SystemsLowering::AttachStaticImageAddressSpace => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ATTACH_ADDRESS_SPACE
            }
            X86SystemsLowering::ActivateStaticImageAddressSpace => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ACTIVATE_ADDRESS_SPACE
            }
            X86SystemsLowering::RestoreStaticImageAddressSpace => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESTORE_ADDRESS_SPACE
            }
            X86SystemsLowering::AuthorizeStaticImageEntry => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTRY
            }
            X86SystemsLowering::AuthorizeStaticImageStackSlot => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_SLOT
            }
            X86SystemsLowering::RevokeStaticImageStackSlot => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_SLOT
            }
            X86SystemsLowering::RevokeStaticImageEntry => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTRY
            }
            X86SystemsLowering::DetachStaticImageAddressSpace => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_DETACH_ADDRESS_SPACE
            }
            X86SystemsLowering::ReturnStaticImageHierarchy => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_HIERARCHY
            }
            X86SystemsLowering::RevokeInitialStaticImageLeaves => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_METADATA
            }
            X86SystemsLowering::ReturnStaticImagePayload => {
                SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_PAYLOAD
            }
            X86SystemsLowering::AdoptBootstrapIdentityMapping => SYSTEMS_KERNEL_MAP,
            X86SystemsLowering::PlainKernelMappingStoreByte => SYSTEMS_KERNEL_MAPPING_STORE_BYTE,
            X86SystemsLowering::PlainKernelMappingLoadByte => SYSTEMS_KERNEL_MAPPING_LOAD_BYTE,
            X86SystemsLowering::ConsumeKernelMapping => SYSTEMS_KERNEL_UNMAP,
            X86SystemsLowering::SelectBootstrapTranslationBacking => SYSTEMS_TRANSLATION_BEGIN,
            X86SystemsLowering::BuildBootstrapEquivalentTranslation => SYSTEMS_TRANSLATION_COMMIT,
            X86SystemsLowering::ActivateTranslationRoot => SYSTEMS_TRANSLATION_ACTIVATE,
            X86SystemsLowering::BeginActiveTranslationEdit => SYSTEMS_TRANSLATION_EDIT_BEGIN,
            X86SystemsLowering::StageActiveTranslationMap => SYSTEMS_TRANSLATION_EDIT_MAP,
            X86SystemsLowering::StageActiveTranslationUnmap => SYSTEMS_TRANSLATION_EDIT_UNMAP,
            X86SystemsLowering::CommitActiveTranslationEdit => SYSTEMS_TRANSLATION_EDIT_COMMIT,
            X86SystemsLowering::CaptureAndMaskLocalInterrupts => SYSTEMS_CRITICAL_ENTER,
            X86SystemsLowering::RestoreLocalInterruptState => SYSTEMS_CRITICAL_RESTORE,
            X86SystemsLowering::InitializeAtomicWord => SYSTEMS_ATOMIC_WORD_CREATE,
            X86SystemsLowering::LockedCompareExchangeWord => SYSTEMS_ATOMIC_COMPARE_EXCHANGE,
            X86SystemsLowering::LoadAtomicWordAcquire => SYSTEMS_ATOMIC_LOAD,
            X86SystemsLowering::ConsumeAtomicWord => SYSTEMS_ATOMIC_END,
            X86SystemsLowering::SendLocalApicNotification => SYSTEMS_LOCAL_NOTIFICATION_SEND,
            X86SystemsLowering::WaitForLocalNotification => SYSTEMS_LOCAL_NOTIFICATION_WAIT,
            X86SystemsLowering::CompleteLocalApicNotification => {
                SYSTEMS_LOCAL_NOTIFICATION_COMPLETE
            }
            X86SystemsLowering::ResumeLocalNotification => SYSTEMS_RESUME_LOCAL_NOTIFICATION,
            X86SystemsLowering::ObserveHpetMonotonicClock => SYSTEMS_MONOTONIC_CLOCK_NOW,
            X86SystemsLowering::ConstructHpetDeadline => SYSTEMS_DEADLINE_AFTER,
            X86SystemsLowering::ArmHpetIoApicDeadline => SYSTEMS_DEADLINE_ARM,
            X86SystemsLowering::WaitForDeadline => SYSTEMS_DEADLINE_WAIT,
            X86SystemsLowering::CompleteHpetIoApicDeadline => SYSTEMS_DEADLINE_COMPLETE,
            X86SystemsLowering::ResumeDeadline => SYSTEMS_RESUME_DEADLINE,
            X86SystemsLowering::CreateKernelContext => SYSTEMS_KERNEL_CONTEXT_CREATE,
            X86SystemsLowering::TransferKernelContextUntilDeadline => {
                SYSTEMS_KERNEL_CONTEXT_TRANSFER_UNTIL_DEADLINE
            }
            X86SystemsLowering::AwaitDeadlinePreemption => {
                SYSTEMS_KERNEL_CONTEXT_AWAIT_DEADLINE_PREEMPTION
            }
            X86SystemsLowering::BlockOneShotCompletion => SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE,
            X86SystemsLowering::PreemptCurrentKernelContext => {
                SYSTEMS_KERNEL_CONTEXT_PREEMPT_CURRENT
            }
            X86SystemsLowering::TransferKernelContext => SYSTEMS_KERNEL_CONTEXT_TRANSFER,
            X86SystemsLowering::RetireKernelContext => SYSTEMS_KERNEL_CONTEXT_RETIRE,
            X86SystemsLowering::ReclaimKernelContext => SYSTEMS_KERNEL_CONTEXT_RECLAIM,
            X86SystemsLowering::PrepareInitialUserAddressSpace => {
                SYSTEMS_USER_ADDRESS_SPACE_PREPARE
            }
            X86SystemsLowering::VerifyInitialUserMapping => {
                SYSTEMS_USER_ADDRESS_SPACE_VERIFY_MAPPING
            }
            X86SystemsLowering::MapInitialUserFrame => SYSTEMS_USER_ADDRESS_SPACE_MAP,
            X86SystemsLowering::StoreInitialUserMappingByte => SYSTEMS_USER_MAPPING_STORE_BYTE,
            X86SystemsLowering::InstallInitialUserEntryTrampoline => {
                SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE
            }
            X86SystemsLowering::ProtectInitialUserMappingReadOnly => {
                SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY
            }
            X86SystemsLowering::LoadInitialUserMappingByte => SYSTEMS_USER_MAPPING_LOAD_BYTE,
            X86SystemsLowering::VerifyInitialUserEntryTrampoline => {
                SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE
            }
            X86SystemsLowering::PermitInitialUserMappingExecute => {
                SYSTEMS_USER_MAPPING_PERMIT_EXECUTE
            }
            X86SystemsLowering::SealInitialUserMappingImmutableData => {
                SYSTEMS_USER_MAPPING_SEAL_IMMUTABLE_DATA
            }
            X86SystemsLowering::PermitInitialUserMappingWritableData => {
                SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA
            }
            X86SystemsLowering::ReserveInitialAnonymousUserMapping => {
                SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS
            }
            X86SystemsLowering::ReleaseInitialAnonymousUserMapping => {
                SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS
            }
            X86SystemsLowering::UnmapInitialUserFrame => SYSTEMS_USER_ADDRESS_SPACE_UNMAP,
            X86SystemsLowering::PrepareInitialUserProbe => SYSTEMS_USER_CONTEXT_PREPARE,
            X86SystemsLowering::EnterInitialUserContext => SYSTEMS_USER_CONTEXT_ENTER,
            X86SystemsLowering::CompleteInitialUserBacking => SYSTEMS_USER_CONTEXT_COMPLETE,
            X86SystemsLowering::ReclaimInitialUserAddressSpace => {
                SYSTEMS_USER_ADDRESS_SPACE_RECLAIM
            }
            X86SystemsLowering::CopyInitialUnameToUser => SYSTEMS_USER_COPY_TO,
            X86SystemsLowering::CopyInitialTimespecFromUser => SYSTEMS_USER_COPY_FROM,
            X86SystemsLowering::DispatchInitialUserSyscall => SYSTEMS_USER_SYSCALL_DISPATCH,
            X86SystemsLowering::StaticBootstrapStorage => SYSTEMS_BOOTSTRAP_STORAGE_PROVISION,
            X86SystemsLowering::MonotonicBootstrapAllocate => SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
            X86SystemsLowering::PlainBootstrapRegionStoreByte => {
                SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE
            }
            X86SystemsLowering::PlainBootstrapRegionLoadByte => SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
            X86SystemsLowering::ConsumeBootstrapRegion => SYSTEMS_BOOTSTRAP_STORAGE_RELEASE,
            X86SystemsLowering::CompleteBootstrapStorage => SYSTEMS_BOOTSTRAP_STORAGE_COMPLETE,
            X86SystemsLowering::StaticCoherentDmaBuffer => SYSTEMS_DMA_BUFFER_CREATE,
            X86SystemsLowering::BindIdentityTranslatedDmaReference => {
                SYSTEMS_DMA_BUFFER_BIND_REFERENCE
            }
            X86SystemsLowering::StoreCpuOwnedDmaInteger => SYSTEMS_DMA_BUFFER_STORE_INTEGER,
            X86SystemsLowering::PrepareCoherentDmaForDevice => SYSTEMS_DMA_BUFFER_PREPARE_DEVICE,
            X86SystemsLowering::CompleteCoherentDmaForCpu => SYSTEMS_DMA_BUFFER_COMPLETE_CPU,
            X86SystemsLowering::LoadCpuOwnedDmaInteger => SYSTEMS_DMA_BUFFER_LOAD_INTEGER,
            X86SystemsLowering::CaptureOpaqueRawEntropySample => SYSTEMS_ENTROPY_SAMPLE_CAPTURE_DMA,
            X86SystemsLowering::MixBlake2sEntropyPoolSample => SYSTEMS_ENTROPY_POOL_MIX_SAMPLE,
            X86SystemsLowering::InitializeChaCha20EntropyGenerator => {
                SYSTEMS_ENTROPY_GENERATOR_INITIALIZE
            }
            X86SystemsLowering::ExtractChaCha20EntropyGenerator => {
                SYSTEMS_ENTROPY_GENERATOR_EXTRACT
            }
            X86SystemsLowering::DeliverInitialAuxvRandom => {
                SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM
            }
            X86SystemsLowering::ReserveInitialUserRandom => {
                SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM
            }
            X86SystemsLowering::BindReusableUserRandom => SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE,
            X86SystemsLowering::PciConfigurationMechanismOneReadDword => {
                SYSTEMS_PCI_CONFIGURATION_READ_DWORD
            }
            X86SystemsLowering::PciConfigurationMechanismOneUpdateWord => {
                SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD
            }
            X86SystemsLowering::PciConfigurationGatewayRegionRead => SYSTEMS_PCI_REGION_READ,
            X86SystemsLowering::PciConfigurationGatewayRegionReadEventually => {
                SYSTEMS_PCI_REGION_READ_EVENTUALLY
            }
            X86SystemsLowering::PciConfigurationGatewayRegionWrite => SYSTEMS_PCI_REGION_WRITE,
            X86SystemsLowering::PciConfigurationGatewayDmaReferenceWrite => {
                SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE
            }
            X86SystemsLowering::PolledUart16550PortIo => SYSTEMS_CONSOLE_WRITE,
            X86SystemsLowering::BreakpointVector3 => SYSTEMS_DEBUG_BREAK,
            X86SystemsLowering::InterruptReturn => SYSTEMS_RESUME_DEBUG_BREAK,
            X86SystemsLowering::InterruptsDisabledHalt => SYSTEMS_FATAL,
        },
        lowering,
    })
}

#[cfg(test)]
mod tests {
    use topal_language::compiler::{CompilerSystemsTargetSelection, analyze_systems_for_compiler};

    use super::*;

    const SOURCE: &str = include_str!("../../../linux-kernel/kernel/arch/x86_64/toolchain-gate.t");

    fn program() -> CompilerSystemsProgram {
        analyze_systems_for_compiler(
            SOURCE,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap()
    }

    #[test]
    fn plans_the_complete_sealed_initial_provider_mapping() {
        // TOPAL-COMP-SYSTEMS-X64-001, TOPAL-SYSTEMS-MACHINE-001.
        let plan = plan_x86_64_systems_provider(&program()).unwrap();
        assert_eq!(plan.revision, X86_SYSTEMS_PROVIDER_REVISION);
        assert_eq!(plan.target, INITIAL_SYSTEMS_TARGET);
        assert_eq!(plan.board, INITIAL_SYSTEMS_BOARD);
        assert_eq!(plan.machine_cpu_model, "qemu64-v1");
        assert_eq!(plan.codegen_cpu, "x86-64");
        assert_eq!(plan.object_format, "elf64-x86-64");
        assert_eq!(plan.relocation_model, "static");
        assert_eq!(plan.code_model, "small");
        assert_eq!(plan.bootstrap_placement.capacity_bytes, 65_536);
        assert_eq!(plan.bootstrap_placement.alignment_bytes, 4096);
        assert_eq!(plan.operations.len(), 152);
        for source_only in [
            SYSTEMS_KERNEL_RUNNABLE_QUEUE_CREATE,
            SYSTEMS_KERNEL_RUNNABLE_QUEUE_ENQUEUE,
            SYSTEMS_KERNEL_RUNNABLE_QUEUE_DEQUEUE,
            SYSTEMS_KERNEL_RUNNABLE_QUEUE_CONSUME,
            SYSTEMS_KERNEL_CONTEXT_TAKE_PREEMPTED,
            SYSTEMS_KERNEL_COMPLETION_CREATE,
            SYSTEMS_KERNEL_COMPLETION_SIGNAL,
            SYSTEMS_KERNEL_COMPLETION_CONSUME,
            SYSTEMS_ENTROPY_SAMPLE_ASSESS_QUALITY,
            SYSTEMS_ENTROPY_POOL_CREATE,
            SYSTEMS_ENTROPY_POOL_CREDIT_CONTRIBUTION,
            SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS,
            SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS,
        ] {
            assert!(
                plan.operations
                    .iter()
                    .all(|operation| operation.semantic_identity != source_only),
                "source-owned runnable queue must not gain a provider lowering"
            );
        }
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_BOOT_MEMORY_DESCRIBE
                && operation.lowering == X86SystemsLowering::LinuxBootParamsE820
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE
                && operation.lowering == X86SystemsLowering::BindReusableUserRandom
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_FRAME_ALLOCATOR_CREATE
                && operation.lowering == X86SystemsLowering::RetainedE820FrameAuthority
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_FRAMES_ALLOCATE
                && operation.lowering == X86SystemsLowering::E820StaticFrameExtentSelect
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PREPARE
                && operation.lowering == X86SystemsLowering::ConstructInitialStaticImageLeaves
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_STACK_BACKING
                && operation.lowering == X86SystemsLowering::ZeroStaticImageStackBacking
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_PUBLISH_STACK_LEAF
                && operation.lowering == X86SystemsLowering::PublishStaticImageStackLeaf
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_INITIALIZE_STACK_ENTRY
                && operation.lowering == X86SystemsLowering::InitializeStaticImageStackEntry
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_PAGESZ
                && operation.lowering == X86SystemsLowering::AddStaticImageStackPageSize
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ADD_STACK_ENTRY_FACT
                && operation.lowering == X86SystemsLowering::AddStaticImageStackEntryFact
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_ENTRY_FACT
                && operation.lowering == X86SystemsLowering::RemoveStaticImageStackEntryFact
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REMOVE_STACK_PAGESZ
                && operation.lowering == X86SystemsLowering::RemoveStaticImageStackPageSize
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_CLEAR_STACK_ENTRY
                && operation.lowering == X86SystemsLowering::ClearStaticImageStackEntry
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_LEAF
                && operation.lowering == X86SystemsLowering::RevokeStaticImageStackLeaf
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_PAYLOAD
                && operation.lowering == X86SystemsLowering::ZeroStaticImagePayload
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_COPY_PAYLOAD
                && operation.lowering == X86SystemsLowering::CopyStaticImagePayload
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_SEAL_PERMISSIONS
                && operation.lowering == X86SystemsLowering::SealStaticImagePermissions
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ATTACH_ADDRESS_SPACE
                && operation.lowering == X86SystemsLowering::AttachStaticImageAddressSpace
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ACTIVATE_ADDRESS_SPACE
                && operation.lowering == X86SystemsLowering::ActivateStaticImageAddressSpace
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RESTORE_ADDRESS_SPACE
                && operation.lowering == X86SystemsLowering::RestoreStaticImageAddressSpace
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_ENTRY
                && operation.lowering == X86SystemsLowering::AuthorizeStaticImageEntry
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_AUTHORIZE_STACK_SLOT
                && operation.lowering == X86SystemsLowering::AuthorizeStaticImageStackSlot
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_STACK_SLOT
                && operation.lowering == X86SystemsLowering::RevokeStaticImageStackSlot
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_ENTRY
                && operation.lowering == X86SystemsLowering::RevokeStaticImageEntry
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_DETACH_ADDRESS_SPACE
                && operation.lowering == X86SystemsLowering::DetachStaticImageAddressSpace
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_HIERARCHY
                && operation.lowering == X86SystemsLowering::ReturnStaticImageHierarchy
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_REVOKE_METADATA
                && operation.lowering == X86SystemsLowering::RevokeInitialStaticImageLeaves
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_STATIC_IMAGE_CONSTRUCTION_RECLAIM_PAYLOAD
                && operation.lowering == X86SystemsLowering::ReturnStaticImagePayload
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_PCI_CONFIGURATION_READ_DWORD
                && operation.lowering == X86SystemsLowering::PciConfigurationMechanismOneReadDword
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD
                && operation.lowering == X86SystemsLowering::PciConfigurationMechanismOneUpdateWord
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_DMA_BUFFER_CREATE
                && operation.lowering == X86SystemsLowering::StaticCoherentDmaBuffer
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_DMA_BUFFER_BIND_REFERENCE
                && operation.lowering == X86SystemsLowering::BindIdentityTranslatedDmaReference
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_DMA_BUFFER_STORE_INTEGER
                && operation.lowering == X86SystemsLowering::StoreCpuOwnedDmaInteger
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_DMA_BUFFER_PREPARE_DEVICE
                && operation.lowering == X86SystemsLowering::PrepareCoherentDmaForDevice
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_DMA_BUFFER_COMPLETE_CPU
                && operation.lowering == X86SystemsLowering::CompleteCoherentDmaForCpu
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_DMA_BUFFER_LOAD_INTEGER
                && operation.lowering == X86SystemsLowering::LoadCpuOwnedDmaInteger
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_ENTROPY_SAMPLE_CAPTURE_DMA
                && operation.lowering == X86SystemsLowering::CaptureOpaqueRawEntropySample
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_ENTROPY_POOL_MIX_SAMPLE
                && operation.lowering == X86SystemsLowering::MixBlake2sEntropyPoolSample
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_ENTROPY_GENERATOR_INITIALIZE
                && operation.lowering == X86SystemsLowering::InitializeChaCha20EntropyGenerator
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_ENTROPY_GENERATOR_EXTRACT
                && operation.lowering == X86SystemsLowering::ExtractChaCha20EntropyGenerator
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_PCI_REGION_READ
                && operation.lowering == X86SystemsLowering::PciConfigurationGatewayRegionRead
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_PCI_REGION_READ_EVENTUALLY
                && operation.lowering
                    == X86SystemsLowering::PciConfigurationGatewayRegionReadEventually
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_PCI_REGION_WRITE
                && operation.lowering == X86SystemsLowering::PciConfigurationGatewayRegionWrite
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE
                && operation.lowering
                    == X86SystemsLowering::PciConfigurationGatewayDmaReferenceWrite
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_FRAMES_RELEASE
                && operation.lowering == X86SystemsLowering::ConsumePhysicalFrameExtent
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_KERNEL_MAP
                && operation.lowering == X86SystemsLowering::AdoptBootstrapIdentityMapping
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_KERNEL_MAPPING_STORE_BYTE
                && operation.lowering == X86SystemsLowering::PlainKernelMappingStoreByte
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_KERNEL_MAPPING_LOAD_BYTE
                && operation.lowering == X86SystemsLowering::PlainKernelMappingLoadByte
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_KERNEL_UNMAP
                && operation.lowering == X86SystemsLowering::ConsumeKernelMapping
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_TRANSLATION_BEGIN
                && operation.lowering == X86SystemsLowering::SelectBootstrapTranslationBacking
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_TRANSLATION_COMMIT
                && operation.lowering == X86SystemsLowering::BuildBootstrapEquivalentTranslation
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_TRANSLATION_ACTIVATE
                && operation.lowering == X86SystemsLowering::ActivateTranslationRoot
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_TRANSLATION_EDIT_BEGIN
                && operation.lowering == X86SystemsLowering::BeginActiveTranslationEdit
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_TRANSLATION_EDIT_MAP
                && operation.lowering == X86SystemsLowering::StageActiveTranslationMap
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_TRANSLATION_EDIT_UNMAP
                && operation.lowering == X86SystemsLowering::StageActiveTranslationUnmap
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_TRANSLATION_EDIT_COMMIT
                && operation.lowering == X86SystemsLowering::CommitActiveTranslationEdit
        }));
        assert_critical_lowerings(&plan);
        assert_local_notification_lowerings(&plan);
        assert_clock_lowering(&plan);
        assert_deadline_lowerings(&plan);
        assert_context_lowerings(&plan);
        for (semantic_identity, lowering) in [
            (
                SYSTEMS_USER_ADDRESS_SPACE_PREPARE,
                X86SystemsLowering::PrepareInitialUserAddressSpace,
            ),
            (
                SYSTEMS_USER_ADDRESS_SPACE_MAP,
                X86SystemsLowering::MapInitialUserFrame,
            ),
            (
                SYSTEMS_USER_MAPPING_STORE_BYTE,
                X86SystemsLowering::StoreInitialUserMappingByte,
            ),
            (
                SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE,
                X86SystemsLowering::InstallInitialUserEntryTrampoline,
            ),
            (
                SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY,
                X86SystemsLowering::ProtectInitialUserMappingReadOnly,
            ),
            (
                SYSTEMS_USER_MAPPING_LOAD_BYTE,
                X86SystemsLowering::LoadInitialUserMappingByte,
            ),
            (
                SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE,
                X86SystemsLowering::VerifyInitialUserEntryTrampoline,
            ),
            (
                SYSTEMS_USER_MAPPING_PERMIT_EXECUTE,
                X86SystemsLowering::PermitInitialUserMappingExecute,
            ),
            (
                SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA,
                X86SystemsLowering::PermitInitialUserMappingWritableData,
            ),
            (
                SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS,
                X86SystemsLowering::ReserveInitialAnonymousUserMapping,
            ),
            (
                SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS,
                X86SystemsLowering::ReleaseInitialAnonymousUserMapping,
            ),
            (
                SYSTEMS_USER_ADDRESS_SPACE_UNMAP,
                X86SystemsLowering::UnmapInitialUserFrame,
            ),
            (
                SYSTEMS_USER_CONTEXT_PREPARE,
                X86SystemsLowering::PrepareInitialUserProbe,
            ),
            (
                SYSTEMS_USER_CONTEXT_ENTER,
                X86SystemsLowering::EnterInitialUserContext,
            ),
            (
                SYSTEMS_USER_CONTEXT_COMPLETE,
                X86SystemsLowering::CompleteInitialUserBacking,
            ),
            (
                SYSTEMS_USER_ADDRESS_SPACE_RECLAIM,
                X86SystemsLowering::ReclaimInitialUserAddressSpace,
            ),
            (
                SYSTEMS_USER_COPY_TO,
                X86SystemsLowering::CopyInitialUnameToUser,
            ),
            (
                SYSTEMS_USER_COPY_FROM,
                X86SystemsLowering::CopyInitialTimespecFromUser,
            ),
            (
                SYSTEMS_USER_SYSCALL_DISPATCH,
                X86SystemsLowering::DispatchInitialUserSyscall,
            ),
        ] {
            assert!(plan.operations.iter().any(|operation| {
                operation.semantic_identity == semantic_identity && operation.lowering == lowering
            }));
        }
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_CONSOLE_WRITE
                && operation.lowering == X86SystemsLowering::PolledUart16550PortIo
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_DEBUG_BREAK
                && operation.lowering == X86SystemsLowering::BreakpointVector3
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_RESUME_DEBUG_BREAK
                && operation.lowering == X86SystemsLowering::InterruptReturn
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_FATAL
                && operation.lowering == X86SystemsLowering::InterruptsDisabledHalt
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE
                && operation.lowering == X86SystemsLowering::PlainBootstrapRegionStoreByte
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE
                && operation.lowering == X86SystemsLowering::PlainBootstrapRegionLoadByte
        }));
    }

    fn assert_critical_lowerings(plan: &X86SystemsProviderPlan) {
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_CRITICAL_ENTER
                && operation.lowering == X86SystemsLowering::CaptureAndMaskLocalInterrupts
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_CRITICAL_RESTORE
                && operation.lowering == X86SystemsLowering::RestoreLocalInterruptState
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_ATOMIC_WORD_CREATE
                && operation.lowering == X86SystemsLowering::InitializeAtomicWord
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_ATOMIC_COMPARE_EXCHANGE
                && operation.lowering == X86SystemsLowering::LockedCompareExchangeWord
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_ATOMIC_LOAD
                && operation.lowering == X86SystemsLowering::LoadAtomicWordAcquire
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_ATOMIC_END
                && operation.lowering == X86SystemsLowering::ConsumeAtomicWord
        }));
    }

    fn assert_local_notification_lowerings(plan: &X86SystemsProviderPlan) {
        for (semantic_identity, lowering) in [
            (
                SYSTEMS_LOCAL_NOTIFICATION_SEND,
                X86SystemsLowering::SendLocalApicNotification,
            ),
            (
                SYSTEMS_LOCAL_NOTIFICATION_WAIT,
                X86SystemsLowering::WaitForLocalNotification,
            ),
            (
                SYSTEMS_LOCAL_NOTIFICATION_COMPLETE,
                X86SystemsLowering::CompleteLocalApicNotification,
            ),
            (
                SYSTEMS_RESUME_LOCAL_NOTIFICATION,
                X86SystemsLowering::ResumeLocalNotification,
            ),
        ] {
            assert!(plan.operations.iter().any(|operation| {
                operation.semantic_identity == semantic_identity && operation.lowering == lowering
            }));
        }
    }

    fn assert_clock_lowering(plan: &X86SystemsProviderPlan) {
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_MONOTONIC_CLOCK_NOW
                && operation.lowering == X86SystemsLowering::ObserveHpetMonotonicClock
        }));
    }

    fn assert_deadline_lowerings(plan: &X86SystemsProviderPlan) {
        for (semantic_identity, lowering) in [
            (
                SYSTEMS_DEADLINE_AFTER,
                X86SystemsLowering::ConstructHpetDeadline,
            ),
            (
                SYSTEMS_DEADLINE_COMPLETE,
                X86SystemsLowering::CompleteHpetIoApicDeadline,
            ),
        ] {
            assert!(plan.operations.iter().any(|operation| {
                operation.semantic_identity == semantic_identity && operation.lowering == lowering
            }));
        }
    }

    fn assert_context_lowerings(plan: &X86SystemsProviderPlan) {
        for (semantic_identity, lowering) in [
            (
                SYSTEMS_KERNEL_CONTEXT_CREATE,
                X86SystemsLowering::CreateKernelContext,
            ),
            (
                SYSTEMS_KERNEL_CONTEXT_TRANSFER_UNTIL_DEADLINE,
                X86SystemsLowering::TransferKernelContextUntilDeadline,
            ),
            (
                SYSTEMS_KERNEL_CONTEXT_AWAIT_DEADLINE_PREEMPTION,
                X86SystemsLowering::AwaitDeadlinePreemption,
            ),
            (
                SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE,
                X86SystemsLowering::BlockOneShotCompletion,
            ),
            (
                SYSTEMS_KERNEL_CONTEXT_PREEMPT_CURRENT,
                X86SystemsLowering::PreemptCurrentKernelContext,
            ),
            (
                SYSTEMS_KERNEL_CONTEXT_TRANSFER,
                X86SystemsLowering::TransferKernelContext,
            ),
            (
                SYSTEMS_KERNEL_CONTEXT_RETIRE,
                X86SystemsLowering::RetireKernelContext,
            ),
            (
                SYSTEMS_KERNEL_CONTEXT_RECLAIM,
                X86SystemsLowering::ReclaimKernelContext,
            ),
        ] {
            assert!(plan.operations.iter().any(|operation| {
                operation.semantic_identity == semantic_identity && operation.lowering == lowering
            }));
        }
    }

    #[test]
    fn plan_is_deterministic_and_retains_no_host_platform_identity() {
        // TOPAL-SYSTEMS-ARTIFACT-001, TOPAL-SYSTEMS-QUALIFY-001.
        let first = plan_x86_64_systems_provider(&program()).unwrap();
        let second = plan_x86_64_systems_provider(&program()).unwrap();
        assert_eq!(first, second);
        let evidence = format!("{first:?}");
        for forbidden in ["linux-gnu", "libc", "dynamic-loader"] {
            assert!(!evidence.contains(forbidden), "{evidence}");
        }
    }

    #[test]
    fn plan_rejects_programs_outside_the_qualified_semantic_profile() {
        // TOPAL-SYSTEMS-QUALIFY-001.
        let mut invalid = program();
        invalid.target.board = "another-board".into();
        let error = plan_x86_64_systems_provider(&invalid).unwrap_err();
        assert!(error.to_string().contains("E-SYSTEMS-TARGET"));
    }
}
