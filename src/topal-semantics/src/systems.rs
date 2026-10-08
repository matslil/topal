//! Architecture-independent reference model for the initial systems profile.

use std::fmt;

use crate::{
    BootstrapRegion, BootstrapStorageDescriptor, BootstrapStorageRequest, BootstrapStorageState,
    KernelMappingRequest, PhysicalFrameRequest, SYSTEMS_BOOT_MEMORY_DESCRIBE,
    SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE, SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE,
    SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE, SYSTEMS_BOOTSTRAP_STORAGE_PROVISION,
    SYSTEMS_BOOTSTRAP_STORAGE_RELEASE, SYSTEMS_FRAME_ALLOCATOR_CREATE, SYSTEMS_FRAMES_ALLOCATE,
    SYSTEMS_FRAMES_RELEASE, SYSTEMS_KERNEL_MAP, SYSTEMS_KERNEL_MAPPING_LOAD_BYTE,
    SYSTEMS_KERNEL_MAPPING_STORE_BYTE, SYSTEMS_KERNEL_UNMAP,
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemsContextKind {
    Bootstrap,
    DebugBreak,
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
    BootstrapRelease,
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
            Self::ConsoleWrite { .. } => SYSTEMS_CONSOLE_WRITE,
            Self::DebugBreak => SYSTEMS_DEBUG_BREAK,
            Self::BootstrapAllocate { .. } => SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
            Self::BootstrapStoreByte { .. } => SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE,
            Self::BootstrapLoadByteEquals { .. } => SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
            Self::BootstrapRelease => SYSTEMS_BOOTSTRAP_STORAGE_RELEASE,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SystemsDisposition {
    Resume,
    Fatal { message: String },
}

impl SystemsDisposition {
    #[must_use]
    pub const fn semantic_identity(&self) -> &'static str {
        match self {
            Self::Resume => SYSTEMS_RESUME_DEBUG_BREAK,
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
    ConsoleWrite {
        text: String,
    },
    ObserveDebugBreak,
    EnterDebugBreak,
    ResumeDebugBreak,
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
            Self::ConsoleWrite { .. } => SYSTEMS_CONSOLE_WRITE,
            Self::ObserveDebugBreak => SYSTEMS_DEBUG_BREAK,
            Self::EnterDebugBreak => "topal.systems.entry.synchronous.debug-break/1",
            Self::ResumeDebugBreak => SYSTEMS_RESUME_DEBUG_BREAK,
            Self::AllocateBootstrapRegion { .. } => SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
            Self::StoreBootstrapByte { .. } => SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE,
            Self::LoadBootstrapByte { .. } => SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
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
    if program.bootstrap.handler.name == program.debug_break.handler.name {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-ENTRY-SET",
            "bootstrap and debug-break entries require distinct handlers",
        ));
    }
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
    if required_context == SystemsContextKind::DebugBreak
        && entry.handler.operations.iter().any(|operation| {
            matches!(
                operation,
                SystemsOperation::DescribeBootMemory { .. }
                    | SystemsOperation::CreateFrameAllocator { .. }
                    | SystemsOperation::AllocatePhysicalFrames { .. }
                    | SystemsOperation::ReleasePhysicalFrames
                    | SystemsOperation::MapKernelFrames { .. }
                    | SystemsOperation::KernelMappingStoreByte { .. }
                    | SystemsOperation::KernelMappingLoadByteEquals { .. }
                    | SystemsOperation::UnmapKernelFrames
                    | SystemsOperation::DebugBreak
                    | SystemsOperation::BootstrapAllocate { .. }
                    | SystemsOperation::BootstrapStoreByte { .. }
                    | SystemsOperation::BootstrapLoadByteEquals { .. }
                    | SystemsOperation::BootstrapRelease
            )
        })
    {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-OPERATION",
            "debug-break is not admitted by a live debug-break context",
        ));
    }
    match (required_context, &entry.handler.disposition) {
        (SystemsContextKind::Bootstrap, SystemsDisposition::Fatal { .. })
        | (
            SystemsContextKind::DebugBreak,
            SystemsDisposition::Resume | SystemsDisposition::Fatal { .. },
        ) => {}
        (SystemsContextKind::Bootstrap, SystemsDisposition::Resume) => {
            return Err(SystemsModelError::new(
                "E-SYSTEMS-DISPOSITION",
                "resume is admitted only by a live debug-break context",
            ));
        }
    }

    let mut expected_effects = entry
        .handler
        .operations
        .iter()
        .map(|operation| operation.semantic_identity().to_owned())
        .collect::<Vec<_>>();
    expected_effects.push(entry.handler.disposition.semantic_identity().to_owned());
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

#[derive(Default)]
struct BootstrapAuthorityState {
    memory_described: bool,
    allocator_created: bool,
    memory_ownership: BootstrapMemoryOwnership,
}

#[derive(Default, Eq, PartialEq)]
enum BootstrapMemoryOwnership {
    #[default]
    None,
    Frames,
    Mapping,
}

impl BootstrapAuthorityState {
    fn observe(
        &mut self,
        index: usize,
        operation: &SystemsOperation,
    ) -> Result<bool, SystemsModelError> {
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

    fn complete(self) -> Result<(), SystemsModelError> {
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
        }
        Ok(())
    }
}

fn validate_bootstrap_storage_operations(
    program: &SystemsProgram,
) -> Result<(), SystemsModelError> {
    let mut storage = BootstrapStorageState::new(
        program.bootstrap_storage.clone(),
        "systems-program-validation",
    )?;
    let mut region: Option<BootstrapRegion> = None;
    let mut authority = BootstrapAuthorityState::default();
    for (index, operation) in program.bootstrap.handler.operations.iter().enumerate() {
        if authority.observe(index, operation)? {
            continue;
        }
        match operation {
            SystemsOperation::BootstrapAllocate { request } => {
                if region.is_some() {
                    return Err(SystemsModelError::new(
                        "E-SYSTEMS-STORAGE-LIFETIME",
                        "the initial executable slice admits one live bootstrap region",
                    ));
                }
                region = Some(storage.allocate(*request).map_err(|code| {
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
            SystemsOperation::DescribeBootMemory { .. }
            | SystemsOperation::CreateFrameAllocator { .. }
            | SystemsOperation::AllocatePhysicalFrames { .. }
            | SystemsOperation::ReleasePhysicalFrames => {
                unreachable!("authority operations continue above")
            }
            SystemsOperation::MapKernelFrames { .. }
            | SystemsOperation::KernelMappingStoreByte { .. }
            | SystemsOperation::KernelMappingLoadByteEquals { .. }
            | SystemsOperation::UnmapKernelFrames => {
                unreachable!("mapping authority operations continue above")
            }
            SystemsOperation::ConsoleWrite { .. } | SystemsOperation::DebugBreak => {}
        }
    }
    authority.complete()?;
    if region.is_some() {
        return Err(SystemsModelError::new(
            "E-SYSTEMS-STORAGE-LIVE",
            "bootstrap handler consumes its context while a region remains live",
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
    let mut transitions = vec![
        SystemsTransition::ProvisionBootstrapStorage {
            capacity_bytes: program.bootstrap_storage.capacity_bytes,
            alignment_bytes: program.bootstrap_storage.alignment_bytes,
        },
        SystemsTransition::EnterBootstrap,
    ];
    let mut storage = BootstrapStorageState::new(
        program.bootstrap_storage.clone(),
        "systems-transition-model",
    )?;
    let mut region: Option<BootstrapRegion> = None;
    for operation in &program.bootstrap.handler.operations {
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
            SystemsOperation::ConsoleWrite { text } => {
                transitions.push(SystemsTransition::ConsoleWrite { text: text.clone() });
            }
            SystemsOperation::DebugBreak => {
                if model_debug_break(&program.debug_break.handler, &mut transitions) {
                    return Ok(transitions);
                }
            }
            _ => {
                if model_bootstrap_storage_operation(
                    operation,
                    &mut storage,
                    &mut region,
                    &mut transitions,
                )? {
                    return Ok(transitions);
                }
            }
        }
    }
    if let SystemsDisposition::Fatal { message } = &program.bootstrap.handler.disposition {
        transitions.push(SystemsTransition::Fatal {
            message: message.clone(),
        });
    }
    Ok(transitions)
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
    }
}

fn model_bootstrap_storage_operation(
    operation: &SystemsOperation,
    storage: &mut BootstrapStorageState,
    region: &mut Option<BootstrapRegion>,
    transitions: &mut Vec<SystemsTransition>,
) -> Result<bool, SystemsModelError> {
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
                    ],
                    disposition: SystemsDisposition::Fatal {
                        message: "done".into(),
                    },
                    effects: vec![
                        SYSTEMS_BOOT_MEMORY_DESCRIBE.into(),
                        SYSTEMS_CONSOLE_WRITE.into(),
                        SYSTEMS_FATAL.into(),
                        SYSTEMS_DEBUG_BREAK.into(),
                        SYSTEMS_FRAME_ALLOCATOR_CREATE.into(),
                    ],
                },
            },
            debug_break: SystemsEntry {
                kind: SystemsEntryKind::SynchronousExceptionDebugBreak,
                handler: SystemsHandler {
                    name: "debug".into(),
                    context: SystemsContextKind::DebugBreak,
                    operations: Vec::new(),
                    effects: vec![debug_disposition.semantic_identity().into()],
                    disposition: debug_disposition,
                },
            },
        }
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
                SystemsTransition::Fatal {
                    message: "done".into(),
                },
            ]
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
        program.bootstrap.handler.effects = vec![
            SYSTEMS_BOOT_MEMORY_DESCRIBE.into(),
            SYSTEMS_CONSOLE_WRITE.into(),
            SYSTEMS_FATAL.into(),
            SYSTEMS_DEBUG_BREAK.into(),
            SYSTEMS_FRAME_ALLOCATOR_CREATE.into(),
            SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE.into(),
            SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE.into(),
            SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE.into(),
            SYSTEMS_BOOTSTRAP_STORAGE_RELEASE.into(),
        ];
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
        if let SystemsOperation::BootstrapStoreByte { offset_bytes, .. } =
            &mut program.bootstrap.handler.operations[5]
        {
            *offset_bytes = 64;
        }
        assert_eq!(
            validate_systems_program(&program).unwrap_err().code,
            "E-SYSTEMS-STORAGE-BOUNDS"
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
