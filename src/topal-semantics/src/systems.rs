//! Architecture-independent reference model for the initial systems profile.

use std::fmt;

use crate::{
    AtomicCompareExchangeResult, AtomicOrder, AtomicWordLocation, AtomicWordRequest,
    BootstrapRegion, BootstrapStorageDescriptor, BootstrapStorageRequest, BootstrapStorageState,
    CriticalDomain, KernelMappingRequest, LocalInterruptMaskState, LocalNotificationProtocol,
    PhysicalFrameRequest, SYSTEMS_ATOMIC_COMPARE_EXCHANGE, SYSTEMS_ATOMIC_END, SYSTEMS_ATOMIC_LOAD,
    SYSTEMS_ATOMIC_WORD_CREATE, SYSTEMS_BOOT_MEMORY_DESCRIBE, SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
    SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE, SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
    SYSTEMS_BOOTSTRAP_STORAGE_PROVISION, SYSTEMS_BOOTSTRAP_STORAGE_RELEASE, SYSTEMS_CRITICAL_ENTER,
    SYSTEMS_CRITICAL_RESTORE, SYSTEMS_DEADLINE_AFTER, SYSTEMS_DEADLINE_ARM,
    SYSTEMS_DEADLINE_COMPLETE, SYSTEMS_DEADLINE_WAIT, SYSTEMS_FRAME_ALLOCATOR_CREATE,
    SYSTEMS_FRAMES_ALLOCATE, SYSTEMS_FRAMES_RELEASE, SYSTEMS_KERNEL_CONTEXT_CREATE,
    SYSTEMS_KERNEL_CONTEXT_RECLAIM, SYSTEMS_KERNEL_CONTEXT_RETIRE, SYSTEMS_KERNEL_CONTEXT_TRANSFER,
    SYSTEMS_KERNEL_MAP, SYSTEMS_KERNEL_MAPPING_LOAD_BYTE, SYSTEMS_KERNEL_MAPPING_STORE_BYTE,
    SYSTEMS_KERNEL_THREAD_ENTRY, SYSTEMS_KERNEL_UNMAP, SYSTEMS_LOCAL_NOTIFICATION_COMPLETE,
    SYSTEMS_LOCAL_NOTIFICATION_SEND, SYSTEMS_LOCAL_NOTIFICATION_WAIT, SYSTEMS_MONOTONIC_CLOCK_NOW,
    SYSTEMS_RESUME_DEADLINE, SYSTEMS_RESUME_LOCAL_NOTIFICATION, SYSTEMS_TRANSLATION_ACTIVATE,
    SYSTEMS_TRANSLATION_BEGIN, SYSTEMS_TRANSLATION_COMMIT, SYSTEMS_TRANSLATION_EDIT_BEGIN,
    SYSTEMS_TRANSLATION_EDIT_COMMIT, SYSTEMS_TRANSLATION_EDIT_MAP, SYSTEMS_TRANSLATION_EDIT_UNMAP,
    TranslationEditKind, TranslationMappingRequest, TranslationUpdateRequest,
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemsContextKind {
    Bootstrap,
    DebugBreak,
    LocalNotificationInterrupt,
    DeadlineInterrupt,
    KernelThread,
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
    TransferKernelContext,
    ReclaimKernelContext,
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
            Self::TransferKernelContext => SYSTEMS_KERNEL_CONTEXT_TRANSFER,
            Self::ReclaimKernelContext => SYSTEMS_KERNEL_CONTEXT_RECLAIM,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SystemsDisposition {
    Resume,
    RetireKernelContextToCaller,
    Fatal { message: String },
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
            Self::RetireKernelContextToCaller => SYSTEMS_KERNEL_CONTEXT_RETIRE,
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
    pub terminal_thread: SystemsEntry,
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
    EndDeadlineWait {
        event_identity: u64,
    },
    CreateKernelContext {
        context_identity: u64,
    },
    TransferKernelContext {
        context_identity: u64,
        caller_identity: u64,
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
            Self::CreateKernelContext { .. } => SYSTEMS_KERNEL_CONTEXT_CREATE,
            Self::TransferKernelContext { .. } => SYSTEMS_KERNEL_CONTEXT_TRANSFER,
            Self::EnterKernelThread { .. } => SYSTEMS_KERNEL_THREAD_ENTRY,
            Self::RetireKernelContextToCaller { .. } => SYSTEMS_KERNEL_CONTEXT_RETIRE,
            Self::ResumeKernelContextCaller { .. } => SYSTEMS_KERNEL_CONTEXT_TRANSFER,
            Self::ReclaimKernelContext { .. } => SYSTEMS_KERNEL_CONTEXT_RECLAIM,
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
        &program.terminal_thread,
        SystemsEntryKind::ResumedKernelThread,
        SystemsContextKind::KernelThread,
    )?;
    let names = [
        &program.bootstrap.handler.name,
        &program.debug_break.handler.name,
        &program.local_notification.handler.name,
        &program.deadline_notification.handler.name,
        &program.kernel_thread.handler.name,
        &program.terminal_thread.handler.name,
    ];
    if names
        .iter()
        .enumerate()
        .any(|(index, name)| names.iter().skip(index + 1).any(|other| *name == *other))
    {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-ENTRY-SET",
            "bootstrap, debug-break, local-notification, deadline, cooperative-thread, and terminal-thread entries require distinct handlers",
        ));
    }
    validate_local_notification_handler(&program.local_notification.handler)?;
    validate_deadline_handler(&program.deadline_notification.handler)?;
    validate_kernel_thread_handler(&program.kernel_thread.handler, false)?;
    validate_kernel_thread_handler(&program.terminal_thread.handler, true)?;
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
                    | SystemsOperation::ReclaimKernelContext
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
            SystemsDisposition::Resume | SystemsDisposition::Fatal { .. },
        )
        | (SystemsContextKind::KernelThread, SystemsDisposition::RetireKernelContextToCaller) => {}
        (SystemsContextKind::Bootstrap, SystemsDisposition::Resume)
        | (
            SystemsContextKind::Bootstrap
            | SystemsContextKind::DebugBreak
            | SystemsContextKind::LocalNotificationInterrupt
            | SystemsContextKind::DeadlineInterrupt,
            SystemsDisposition::RetireKernelContextToCaller,
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
            "systems handler effects must equal its sorted, deduplicated semantic identities",
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
        || handler.disposition != SystemsDisposition::Resume
    {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-DEADLINE-EVENT",
            "the deadline handler must consume completion authority before resume",
        ));
    }
    Ok(())
}

fn validate_kernel_thread_handler(
    handler: &SystemsHandler,
    terminal: bool,
) -> Result<(), SystemsModelError> {
    let expected = if terminal {
        vec![SystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_CONTEXT_TERMINAL_ENTERED".into(),
        }]
    } else {
        vec![
            SystemsOperation::ConsoleWrite {
                text: "TOPAL_KERNEL_CONTEXT_COOPERATIVE_ENTERED".into(),
            },
            SystemsOperation::TransferKernelContext,
            SystemsOperation::ConsoleWrite {
                text: "TOPAL_KERNEL_CONTEXT_COOPERATIVE_RESUMED".into(),
            },
        ]
    };
    if handler.operations != expected
        || handler.disposition != SystemsDisposition::RetireKernelContextToCaller
    {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-CONTEXT-TRANSFER",
            "cooperative and terminal kernel-thread handlers must implement their exact handoff protocols",
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
    CooperativeCreated,
    BothSuspended,
    CooperativeYielded,
    TerminalCompleted,
    TerminalReclaimed,
    CooperativeCompleted,
    Reclaimed,
}

#[derive(Default, Eq, PartialEq)]
enum BootstrapMemoryOwnership {
    #[default]
    None,
    Frames,
    Mapping,
    ProvisionalMapping,
    ProvisionalFrames,
}

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
                    || request.frame_count == 0
                    || request.alignment_frames == 0
                    || !request.alignment_frames.is_power_of_two()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-FRAMES",
                        "physical-frame allocation requires one allocator, a valid request, and no live extent",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::Frames;
            }
            SystemsOperation::ReleasePhysicalFrames => {
                if self.memory_ownership != BootstrapMemoryOwnership::Frames {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-FRAMES",
                        "physical-frame release requires one live extent",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::None;
            }
            SystemsOperation::MapKernelFrames { request, .. } => {
                if self.memory_ownership != BootstrapMemoryOwnership::Frames
                    || *request != KernelMappingRequest::initial_read_write()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-MAPPING",
                        "kernel mapping requires one live extent and the sealed mapping policy",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::Mapping;
            }
            SystemsOperation::KernelMappingStoreByte { offset_bytes, .. }
            | SystemsOperation::KernelMappingLoadByteEquals { offset_bytes, .. } => {
                if self.memory_ownership != BootstrapMemoryOwnership::Mapping
                    || *offset_bytes >= 4096
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-MAPPING",
                        "kernel mapping byte access requires one live mapping and an in-bounds offset",
                    ));
                }
            }
            SystemsOperation::UnmapKernelFrames => {
                if self.memory_ownership != BootstrapMemoryOwnership::Mapping {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-MAPPING",
                        "kernel unmap requires one live mapping",
                    ));
                }
                self.memory_ownership = BootstrapMemoryOwnership::Frames;
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
            SystemsOperation::CreateKernelContext => {
                if self.deadline != BootstrapDeadlineState::Completed
                    || !self.critical_stack.is_empty()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "kernel-context creation requires the completed deadline lifecycle and one restored processor context",
                    ));
                }
                self.kernel_context = match self.kernel_context {
                    BootstrapKernelContextState::Fresh => {
                        BootstrapKernelContextState::CooperativeCreated
                    }
                    BootstrapKernelContextState::CooperativeCreated => {
                        BootstrapKernelContextState::BothSuspended
                    }
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-CONTEXT-TRANSFER",
                            "the cooperative profile creates exactly two contexts before dispatch",
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
                    BootstrapKernelContextState::BothSuspended => {
                        BootstrapKernelContextState::CooperativeYielded
                    }
                    BootstrapKernelContextState::CooperativeYielded => {
                        BootstrapKernelContextState::TerminalCompleted
                    }
                    BootstrapKernelContextState::TerminalReclaimed => {
                        BootstrapKernelContextState::CooperativeCompleted
                    }
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-CONTEXT-TRANSFER",
                            "kernel-context transfers must follow cooperative, terminal, cooperative FIFO order",
                        ));
                    }
                };
            }
            SystemsOperation::ReclaimKernelContext => {
                self.kernel_context = match self.kernel_context {
                    BootstrapKernelContextState::TerminalCompleted => {
                        BootstrapKernelContextState::TerminalReclaimed
                    }
                    BootstrapKernelContextState::CooperativeCompleted => {
                        BootstrapKernelContextState::Reclaimed
                    }
                    _ => {
                        return Err(SystemsModelError::new(
                            "E-SYSTEMS-CONTEXT-TRANSFER",
                            "kernel-context reclaim requires the matching completed transfer",
                        ));
                    }
                };
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
                        && self.memory_ownership == BootstrapMemoryOwnership::Frames =>
                {
                    self.translation = BootstrapTranslationState::MapEdit;
                }
                TranslationEditKind::Unmap
                    if self.translation == BootstrapTranslationState::MappedActive
                        && self.memory_ownership == BootstrapMemoryOwnership::Mapping =>
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
                if self.translation != BootstrapTranslationState::MapEdit
                    || self.memory_ownership != BootstrapMemoryOwnership::Frames
                    || *request != TranslationMappingRequest::initial_read_write()
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-TRANSLATION-EDIT",
                        "translation edit map requires the sealed policy, map edit, and live extent",
                    ));
                }
                self.translation = BootstrapTranslationState::MapStaged;
                self.memory_ownership = BootstrapMemoryOwnership::ProvisionalMapping;
            }
            SystemsOperation::UnmapTranslationMapping => {
                if self.translation != BootstrapTranslationState::UnmapEdit
                    || self.memory_ownership != BootstrapMemoryOwnership::Mapping
                {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-TRANSLATION-EDIT",
                        "translation edit unmap requires the matching edit and live mapping",
                    ));
                }
                self.translation = BootstrapTranslationState::UnmapStaged;
                self.memory_ownership = BootstrapMemoryOwnership::ProvisionalFrames;
            }
            SystemsOperation::CommitTranslationEdit { kind, .. } => match kind {
                TranslationEditKind::Map
                    if self.translation == BootstrapTranslationState::MapStaged
                        && self.memory_ownership
                            == BootstrapMemoryOwnership::ProvisionalMapping =>
                {
                    self.translation = BootstrapTranslationState::MappedActive;
                    self.memory_ownership = BootstrapMemoryOwnership::Mapping;
                }
                TranslationEditKind::Unmap
                    if self.translation == BootstrapTranslationState::UnmapStaged
                        && self.memory_ownership == BootstrapMemoryOwnership::ProvisionalFrames =>
                {
                    self.translation = BootstrapTranslationState::RemovedActive;
                    self.memory_ownership = BootstrapMemoryOwnership::Frames;
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
            BootstrapMemoryOwnership::Frames => {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-FRAMES-LIVE",
                    "bootstrap handler consumes its context while a physical-frame extent remains live",
                ));
            }
            BootstrapMemoryOwnership::Mapping => {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-MAPPING-LIVE",
                    "bootstrap handler consumes its context while a kernel mapping remains live",
                ));
            }
            BootstrapMemoryOwnership::ProvisionalMapping
            | BootstrapMemoryOwnership::ProvisionalFrames => {
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
        if self.kernel_context != BootstrapKernelContextState::Reclaimed {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-CONTEXT-TRANSFER-LIVE",
                "bootstrap completion requires both cooperative contexts to retire and be reclaimed",
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
        SystemsOperation::TransferKernelContext => {
            if context_stacks.is_empty() {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "kernel-context transfer requires one live suspended context stack",
                ));
            }
        }
        SystemsOperation::ReclaimKernelContext => {
            let index = match state {
                BootstrapKernelContextState::TerminalReclaimed => 1,
                BootstrapKernelContextState::Reclaimed => 0,
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
            &program.terminal_thread.handler,
            &mut storage,
            &mut region,
            &mut context_stacks,
            &mut kernel_context_state,
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

#[allow(clippy::too_many_arguments)]
fn model_kernel_context_transition(
    operation: &SystemsOperation,
    cooperative_handler: &SystemsHandler,
    terminal_handler: &SystemsHandler,
    storage: &mut BootstrapStorageState,
    region: &mut Option<BootstrapRegion>,
    context_stacks: &mut Vec<BootstrapRegion>,
    state: &mut BootstrapKernelContextState,
    transitions: &mut Vec<SystemsTransition>,
) -> Result<bool, SystemsModelError> {
    match operation {
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
                    BootstrapKernelContextState::CooperativeCreated
                }
                BootstrapKernelContextState::CooperativeCreated => {
                    BootstrapKernelContextState::BothSuspended
                }
                _ => {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "kernel-context model creates exactly two worker contexts",
                    ));
                }
            };
            transitions.push(SystemsTransition::CreateKernelContext { context_identity });
        }
        SystemsOperation::TransferKernelContext => match *state {
            BootstrapKernelContextState::BothSuspended => {
                let [
                    SystemsOperation::ConsoleWrite { text: entered },
                    SystemsOperation::TransferKernelContext,
                    SystemsOperation::ConsoleWrite { .. },
                ] = cooperative_handler.operations.as_slice()
                else {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-CONTEXT-TRANSFER",
                        "cooperative handler must enter, hand back once, and resume",
                    ));
                };
                transitions.extend([
                    SystemsTransition::TransferKernelContext {
                        context_identity: 1,
                        caller_identity: 1,
                    },
                    SystemsTransition::EnterKernelThread {
                        context_identity: 1,
                    },
                    SystemsTransition::ConsoleWrite {
                        text: entered.clone(),
                    },
                    SystemsTransition::TransferKernelContext {
                        context_identity: 0,
                        caller_identity: 2,
                    },
                    SystemsTransition::ResumeKernelContextCaller { caller_identity: 1 },
                ]);
                *state = BootstrapKernelContextState::CooperativeYielded;
            }
            BootstrapKernelContextState::CooperativeYielded => {
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
                        context_identity: 2,
                        caller_identity: 3,
                    },
                    SystemsTransition::EnterKernelThread {
                        context_identity: 2,
                    },
                    SystemsTransition::ConsoleWrite { text: text.clone() },
                    SystemsTransition::RetireKernelContextToCaller {
                        context_identity: 2,
                        caller_identity: 3,
                    },
                    SystemsTransition::ResumeKernelContextCaller { caller_identity: 3 },
                ]);
                *state = BootstrapKernelContextState::TerminalCompleted;
            }
            BootstrapKernelContextState::TerminalReclaimed => {
                let [_, _, SystemsOperation::ConsoleWrite { text }] =
                    cooperative_handler.operations.as_slice()
                else {
                    unreachable!("validated cooperative handler shape")
                };
                transitions.extend([
                    SystemsTransition::TransferKernelContext {
                        context_identity: 1,
                        caller_identity: 4,
                    },
                    SystemsTransition::ResumeKernelContextCaller { caller_identity: 2 },
                    SystemsTransition::ConsoleWrite { text: text.clone() },
                    SystemsTransition::RetireKernelContextToCaller {
                        context_identity: 1,
                        caller_identity: 4,
                    },
                    SystemsTransition::ResumeKernelContextCaller { caller_identity: 4 },
                ]);
                *state = BootstrapKernelContextState::CooperativeCompleted;
            }
            _ => {
                return Err(SystemsModelError::new(
                    "E-SYSTEMS-CONTEXT-TRANSFER",
                    "kernel-context model requires cooperative, terminal, cooperative FIFO order",
                ));
            }
        },
        SystemsOperation::ReclaimKernelContext => {
            let (stack_index, context_identity, next_state) = match *state {
                BootstrapKernelContextState::TerminalCompleted => {
                    (1, 2, BootstrapKernelContextState::TerminalReclaimed)
                }
                BootstrapKernelContextState::CooperativeCompleted => {
                    (0, 1, BootstrapKernelContextState::Reclaimed)
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
        | SystemsOperation::TransferKernelContext
        | SystemsOperation::ReclaimKernelContext => {
            unreachable!("kernel-context operations are modeled by the caller")
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
            SYSTEMS_DEADLINE_ARM.into(),
            SYSTEMS_DEADLINE_WAIT.into(),
            SYSTEMS_DEBUG_BREAK.into(),
            SYSTEMS_FRAME_ALLOCATOR_CREATE.into(),
            SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE.into(),
            SYSTEMS_KERNEL_CONTEXT_CREATE.into(),
            SYSTEMS_KERNEL_CONTEXT_TRANSFER.into(),
            SYSTEMS_KERNEL_CONTEXT_RECLAIM.into(),
        ];
        bootstrap_effects.sort();
        let mut local_notification_effects = vec![
            SYSTEMS_RESUME_LOCAL_NOTIFICATION.into(),
            SYSTEMS_LOCAL_NOTIFICATION_COMPLETE.into(),
        ];
        local_notification_effects.sort();
        let mut deadline_effects = vec![
            SYSTEMS_RESUME_DEADLINE.into(),
            SYSTEMS_DEADLINE_COMPLETE.into(),
        ];
        deadline_effects.sort();
        let mut kernel_thread_effects = vec![
            SYSTEMS_CONSOLE_WRITE.into(),
            SYSTEMS_KERNEL_CONTEXT_TRANSFER.into(),
            SYSTEMS_KERNEL_CONTEXT_RETIRE.into(),
        ];
        kernel_thread_effects.sort();
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
                        SystemsOperation::ArmDeadline,
                        SystemsOperation::WaitDeadline,
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
                        SystemsOperation::TransferKernelContext,
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_CONTEXT_COOPERATIVE_SUSPENDED".into(),
                        },
                        SystemsOperation::TransferKernelContext,
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_CONTEXT_TERMINAL_RETIRED".into(),
                        },
                        SystemsOperation::ReclaimKernelContext,
                        SystemsOperation::TransferKernelContext,
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_CONTEXT_COOPERATIVE_RETIRED".into(),
                        },
                        SystemsOperation::ReclaimKernelContext,
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
                    disposition: SystemsDisposition::Resume,
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
                            text: "TOPAL_KERNEL_CONTEXT_COOPERATIVE_ENTERED".into(),
                        },
                        SystemsOperation::TransferKernelContext,
                        SystemsOperation::ConsoleWrite {
                            text: "TOPAL_KERNEL_CONTEXT_COOPERATIVE_RESUMED".into(),
                        },
                    ],
                    disposition: SystemsDisposition::RetireKernelContextToCaller,
                    effects: kernel_thread_effects,
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
                | SystemsOperation::TransferKernelContext
                | SystemsOperation::ReclaimKernelContext => false,
                SystemsOperation::ConsoleWrite { text }
                    if text.starts_with("TOPAL_KERNEL_CONTEXT_") =>
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
                SystemsTransition::ArmDeadline { event_identity: 1 },
                SystemsTransition::BeginDeadlineWait { event_identity: 1 },
                SystemsTransition::ObserveDeadline { event_identity: 1 },
                SystemsTransition::EnterDeadlineInterrupt { event_identity: 1 },
                SystemsTransition::CompleteDeadlineInterrupt { event_identity: 1 },
                SystemsTransition::ResumeDeadlineInterrupt { event_identity: 1 },
                SystemsTransition::EndDeadlineWait { event_identity: 1 },
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
                SystemsTransition::TransferKernelContext {
                    context_identity: 1,
                    caller_identity: 1,
                },
                SystemsTransition::EnterKernelThread {
                    context_identity: 1,
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_CONTEXT_COOPERATIVE_ENTERED".into(),
                },
                SystemsTransition::TransferKernelContext {
                    context_identity: 0,
                    caller_identity: 2,
                },
                SystemsTransition::ResumeKernelContextCaller { caller_identity: 1 },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_CONTEXT_COOPERATIVE_SUSPENDED".into(),
                },
                SystemsTransition::TransferKernelContext {
                    context_identity: 2,
                    caller_identity: 3,
                },
                SystemsTransition::EnterKernelThread {
                    context_identity: 2,
                },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_CONTEXT_TERMINAL_ENTERED".into(),
                },
                SystemsTransition::RetireKernelContextToCaller {
                    context_identity: 2,
                    caller_identity: 3,
                },
                SystemsTransition::ResumeKernelContextCaller { caller_identity: 3 },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_CONTEXT_TERMINAL_RETIRED".into(),
                },
                SystemsTransition::ReclaimKernelContext {
                    context_identity: 2,
                },
                SystemsTransition::TransferKernelContext {
                    context_identity: 1,
                    caller_identity: 4,
                },
                SystemsTransition::ResumeKernelContextCaller { caller_identity: 2 },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_CONTEXT_COOPERATIVE_RESUMED".into(),
                },
                SystemsTransition::RetireKernelContextToCaller {
                    context_identity: 1,
                    caller_identity: 4,
                },
                SystemsTransition::ResumeKernelContextCaller { caller_identity: 4 },
                SystemsTransition::ConsoleWrite {
                    text: "TOPAL_KERNEL_CONTEXT_COOPERATIVE_RETIRED".into(),
                },
                SystemsTransition::ReclaimKernelContext {
                    context_identity: 1,
                },
                SystemsTransition::Fatal {
                    message: "done".into(),
                },
            ]
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
    fn deadline_requires_exact_construction_arm_wait_and_completion() {
        // TOPAL-SEM-SYSTEMS-001, TOPAL-SYSTEMS-DEADLINE-EVENT-001.
        let mut wrong_duration = program(SystemsDisposition::Resume);
        wrong_duration.bootstrap.handler.operations[8] = SystemsOperation::ConstructDeadline {
            duration_nanoseconds: 2_000_000,
        };
        assert_eq!(
            validate_systems_program(&wrong_duration).unwrap_err().code,
            "E-SYSTEMS-DEADLINE-EVENT"
        );

        let mut missing_wait = program(SystemsDisposition::Resume);
        remove_context_lifecycle(&mut missing_wait);
        missing_wait
            .bootstrap
            .handler
            .operations
            .retain(|operation| !matches!(operation, SystemsOperation::WaitDeadline));
        refresh_bootstrap_effects(&mut missing_wait);
        assert_eq!(
            validate_systems_program(&missing_wait).unwrap_err().code,
            "E-SYSTEMS-DEADLINE-EVENT-LIVE"
        );

        let mut incomplete = program(SystemsDisposition::Resume);
        incomplete.deadline_notification.handler.operations.clear();
        incomplete.deadline_notification.handler.effects = vec![SYSTEMS_RESUME_DEADLINE.into()];
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
            SYSTEMS_KERNEL_CONTEXT_TRANSFER.into(),
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
            frame_count: 1,
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

        program
            .bootstrap
            .handler
            .operations
            .retain(|operation| !matches!(operation, SystemsOperation::ReleasePhysicalFrames));
        program
            .bootstrap
            .handler
            .effects
            .retain(|effect| effect != SYSTEMS_FRAMES_RELEASE);
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
