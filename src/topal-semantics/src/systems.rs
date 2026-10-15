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
    SYSTEMS_RESUME_DEADLINE, SYSTEMS_RESUME_LOCAL_NOTIFICATION, SYSTEMS_TRANSLATION_ACTIVATE,
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
            })
    {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-USER-CONTEXT",
            "the initial user-syscall handler must dispatch the sealed read, write, brk, getpid, uname, nanosleep, and exit_group protocol",
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
    constructed_user_read_completed: bool,
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
}

const INITIAL_SINGLE_FRAME_REQUEST: PhysicalFrameRequest = PhysicalFrameRequest {
    frame_count: 1,
    alignment_frames: 1,
};

const INITIAL_MULTI_FRAME_REQUEST: PhysicalFrameRequest = PhysicalFrameRequest {
    frame_count: 3,
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
                if !self.allocator_created
                    || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !matches!(
                        *request,
                        INITIAL_SINGLE_FRAME_REQUEST | INITIAL_MULTI_FRAME_REQUEST
                    )
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-FRAMES",
                        "physical-frame allocation requires one allocator, an admitted static extent shape, and no live extent",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::Frames(*request);
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
                if !matches!(self.memory_ownership, BootstrapMemoryOwnership::Frames(_)) {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-FRAMES",
                        "physical-frame release requires one live extent",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::None;
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
                    != BootstrapMemoryOwnership::Frames(INITIAL_SINGLE_FRAME_REQUEST)
                    || self.anonymous_user_mapping_reserved
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "anonymous reservation requires one source-owned frame and no prior reservation",
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
                if self.memory_ownership
                    != BootstrapMemoryOwnership::Frames(INITIAL_SINGLE_FRAME_REQUEST)
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user mapping publication requires one disjoint source-owned frame and no live critical scope",
                    ));
                }
                self.memory_ownership =
                    BootstrapMemoryOwnership::UserMapping(INITIAL_SINGLE_FRAME_REQUEST);
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
                        != BootstrapMemoryOwnership::UserMapping(INITIAL_SINGLE_FRAME_REQUEST)
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
                    != BootstrapMemoryOwnership::UserMapping(INITIAL_SINGLE_FRAME_REQUEST)
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
                        != BootstrapMemoryOwnership::UserMapping(INITIAL_SINGLE_FRAME_REQUEST)
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
                        != BootstrapMemoryOwnership::UserMapping(INITIAL_SINGLE_FRAME_REQUEST)
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
            SystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_USER_READ_OK" => {
                if !matches!(
                    self.user_execution,
                    BootstrapUserExecutionState::CompletedWithResidentImage(_)
                ) || self.memory_ownership != BootstrapMemoryOwnership::None
                    || !self.resident_user_image_ready
                    || !self.initial_boot_resource_validated
                    || self.constructed_user_read_completed
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "constructed read witness requires exact user return with both finalized mappings resident",
                    ));
                }
                self.constructed_user_read_completed = true;
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
                self.memory_ownership =
                    BootstrapMemoryOwnership::Frames(INITIAL_SINGLE_FRAME_REQUEST);
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
                    BootstrapMemoryOwnership::Frames(INITIAL_SINGLE_FRAME_REQUEST);
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
                if !self.critical_stack.is_empty() {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-USER-CONTEXT",
                        "user entry requires the exact prepared probe and restored kernel context",
                    ));
                }
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
            BootstrapMemoryOwnership::None => {}
            BootstrapMemoryOwnership::Frames(_) => {
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
            let fresh_mapping = protocol.map_user_frame(
                prepared.as_ref().expect("checked prepared user space"),
                INITIAL_SINGLE_FRAME_REQUEST,
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
            if returned != INITIAL_SINGLE_FRAME_REQUEST {
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
                            request: INITIAL_SINGLE_FRAME_REQUEST,
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
                            request: INITIAL_SINGLE_FRAME_REQUEST,
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
                        SystemsOperation::EnterUserContext,
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_USER_READ_OK".into(),
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
                        || text == "TOPAL_KERNEL_USER_READ_OK"
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
        assert_eq!(
            model_systems_transitions(&program(SystemsDisposition::Resume)).unwrap(),
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
                    request: INITIAL_SINGLE_FRAME_REQUEST,
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
                    request: INITIAL_SINGLE_FRAME_REQUEST,
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
                    text: "TOPAL_KERNEL_USER_READ_OK".into(),
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
        let observation = early.bootstrap.handler.operations.remove(6);
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
        unsupported_request.frame_count = 2;
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
