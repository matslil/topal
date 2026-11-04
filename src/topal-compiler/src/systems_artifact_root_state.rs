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
    StaticProcessExecSnapshot,
    StaticProcessDescriptorTable,
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
            Self::StaticProcessExecSnapshot => X86_SYSTEMS_STATIC_PROCESS_EXEC_SNAPSHOT_SYMBOL,
            Self::StaticProcessDescriptorTable => {
                X86_SYSTEMS_STATIC_PROCESS_DESCRIPTOR_TABLE_SYMBOL
            }
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
            | Self::StaticProcessBrkReserve
            | Self::StaticProcessExecSnapshot
            | Self::StaticProcessDescriptorTable => SymbolKind::Data,
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

#[derive(Clone, Copy)]
enum RootLibraryDependency {
    RegionApply,
    RegionFinalize,
    Code,
    ImageCode,
    DispatchCode,
    ProcessIdentityCode,
    RetainedIdentityCode,
    StructuredIdentityCode,
    RetainedState,
    Arena,
}

#[derive(Default)]
struct RootEncoder {
    bootstrap_capacity: u64,
    region_library: bool,
    source_bound_library: bool,
    source_library_export: &'static str,
    numeric_library: bool,
    image_library: bool,
    dispatch_library: bool,
    dispatch_policy: Option<crate::systems_numeric_library::NumericPolicy>,
    process_identity_library: bool,
    retained_identity: bool,
    structured_identity: bool,
    library_relocations: Vec<(u64, RootLibraryDependency)>,
    source_library_relocations: Vec<u64>,
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
    bootstrap_region_bytes: Option<u64>,
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
    static_process_exec_snapshot: SymbolId,
    static_process_descriptor_table: SymbolId,
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

include!("systems_root_encoder_memory.rs");

include!("systems_root_encoder_stack_metadata.rs");

include!("systems_root_encoder_entropy_qualification.rs");

include!("systems_root_encoder_entropy_devices.rs");

include!("systems_root_encoder_translation_completion.rs");

include!("systems_root_encoder_devices.rs");

include!("systems_root_encoder_contexts.rs");

include!("systems_root_encoder_user_memory.rs");
