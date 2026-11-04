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
    pub kernel_library: Option<crate::SystemsSourceLibraryRequirement>,
    pub region_library: Option<crate::SystemsSourceLibraryRequirement>,
    pub file_position_library: Option<crate::SystemsSourceLibraryRequirement>,
    pub ram_page_pool_library: Option<crate::SystemsSourceLibraryRequirement>,
    pub time_library: Option<crate::SystemsSourceLibraryRequirement>,
    pub image_library: Option<crate::SystemsSourceLibraryRequirement>,
    pub dispatch_library: Option<crate::SystemsSourceLibraryRequirement>,
    pub process_identity_library: Option<crate::SystemsSourceLibraryRequirement>,
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
    BorrowBootstrapRegionForLibrary {
        generation: u64,
    },
    RequireRegionLibraryReclamation,
    ResumeBootstrapRegionAfterLibrary {
        /// Runtime acceptance obligation; no source result is predicted.
        require_true: bool,
    },
    CheckDynamicBootstrapByte {
        offset_bytes: u64,
        expected: u8,
    },
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
            Self::BorrowBootstrapRegionForLibrary { .. }
            | Self::RequireRegionLibraryReclamation
            | Self::ResumeBootstrapRegionAfterLibrary { .. } => SYSTEMS_REGION_LIBRARY_APPLY,
            Self::CheckDynamicBootstrapByte { .. } => SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
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
    let invocation_count = program
        .bootstrap
        .handler
        .operations
        .iter()
        .filter(|operation| matches!(operation, SystemsOperation::ApplyBootstrapRegionLibrary))
        .count();
    if invocation_count != usize::from(program.region_library.is_some()) {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-REGION-ROOT",
            "region-library requires exactly one bootstrap invocation",
        ));
    }
    if let Some(library) = &program.region_library {
        library.validate()?;
        if !matches!(
            library.interface,
            crate::SystemsSourceLibraryInterface::BorrowedByteRegionV1
                | crate::SystemsSourceLibraryInterface::BorrowedByteRegionInstanceActionV1
        ) {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-REGION-ROOT",
                "region-library requires a qualified Unit or instance-action entry",
            ));
        }
    }
    if let Some(library) = &program.ram_page_pool_library {
        library.validate()?;
        if library.interface
            != crate::SystemsSourceLibraryInterface::BorrowedByteRegionInstanceActionV1
        {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-REGION-PAGE",
                "ram-page-pool-library requires the qualified region instance action interface",
            ));
        }
    }
    if let Some(library) = &program.file_position_library {
        library.validate()?;
        if !matches!(
            library.interface,
            crate::SystemsSourceLibraryInterface::BorrowedByteRegionActionV1
                | crate::SystemsSourceLibraryInterface::BorrowedByteRegionFramedActionV1
                | crate::SystemsSourceLibraryInterface::BorrowedByteRegionInstanceActionV1
        ) {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-REGION-ACTION",
                "file-position-library requires a qualified region action interface",
            ));
        }
    }
    if let Some(library) = &program.kernel_library {
        library.validate()?;
        if !matches!(
            library.interface,
            crate::SystemsSourceLibraryInterface::BootOpenDecisionV1
                | crate::SystemsSourceLibraryInterface::FileOpenDecisionV1
        ) {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-LIBRARY",
                "kernel-library requires a qualified finite open interface",
            ));
        }
    }
    if let Some(library) = &program.time_library {
        library.validate()?;
        if library.interface != crate::SystemsSourceLibraryInterface::RelativeSleepDeadlineV1 {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-LIBRARY",
                "time-library requires relative-sleep-deadline-v1",
            ));
        }
    }
    if let Some(library) = &program.image_library {
        library.validate()?;
        if library.interface != crate::SystemsSourceLibraryInterface::StaticElf64HeaderV1 {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-LIBRARY",
                "image-library requires static-elf64-header-v1",
            ));
        }
    }
    if let Some(library) = &program.process_identity_library {
        library.validate()?;
        if !matches!(
            library.interface,
            crate::SystemsSourceLibraryInterface::ProcessIdentityTransitionV1
                | crate::SystemsSourceLibraryInterface::RetainedProcessIdentityV1
                | crate::SystemsSourceLibraryInterface::StructuredProcessIdentityV1
        ) {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-LIBRARY",
                "process-identity-library requires process-identity-transition-v1",
            ));
        }
    }
    if let Some(library) = &program.dispatch_library {
        library.validate()?;
        if !matches!(
            library.interface,
            crate::SystemsSourceLibraryInterface::StaticLinuxDispatchV1
                | crate::SystemsSourceLibraryInterface::StaticLinuxDispatchV2
                | crate::SystemsSourceLibraryInterface::StaticLinuxDispatchV3
                | crate::SystemsSourceLibraryInterface::StaticLinuxDispatchV4
        ) {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-LIBRARY",
                "dispatch-library requires static-linux-dispatch-v1",
            ));
        }
    }
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
                    | SystemsOperation::ApplyBootstrapRegionLibrary
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
