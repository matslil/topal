//! Deterministic x86-64 provider planning for checked systems programs.

use topal_language::compiler::{
    CompilerSystemsProgram, INITIAL_SYSTEMS_BOARD, INITIAL_SYSTEMS_PROFILE, INITIAL_SYSTEMS_TARGET,
    SYSTEMS_ATOMIC_COMPARE_EXCHANGE, SYSTEMS_ATOMIC_END, SYSTEMS_ATOMIC_LOAD,
    SYSTEMS_ATOMIC_WORD_CREATE, SYSTEMS_BOOT_MEMORY_DESCRIBE, SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
    SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE, SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
    SYSTEMS_BOOTSTRAP_STORAGE_COMPLETE, SYSTEMS_BOOTSTRAP_STORAGE_PROVISION,
    SYSTEMS_BOOTSTRAP_STORAGE_RELEASE, SYSTEMS_CONSOLE_WRITE, SYSTEMS_CRITICAL_ENTER,
    SYSTEMS_CRITICAL_RESTORE, SYSTEMS_DEBUG_BREAK, SYSTEMS_FATAL, SYSTEMS_FRAME_ALLOCATOR_CREATE,
    SYSTEMS_FRAMES_ALLOCATE, SYSTEMS_FRAMES_RELEASE, SYSTEMS_KERNEL_MAP,
    SYSTEMS_KERNEL_MAPPING_LOAD_BYTE, SYSTEMS_KERNEL_MAPPING_STORE_BYTE, SYSTEMS_KERNEL_UNMAP,
    SYSTEMS_LOCAL_NOTIFICATION_COMPLETE, SYSTEMS_LOCAL_NOTIFICATION_SEND,
    SYSTEMS_LOCAL_NOTIFICATION_WAIT, SYSTEMS_MONOTONIC_CLOCK_NOW, SYSTEMS_RESUME_DEBUG_BREAK,
    SYSTEMS_RESUME_LOCAL_NOTIFICATION, SYSTEMS_TRANSLATION_ACTIVATE, SYSTEMS_TRANSLATION_BEGIN,
    SYSTEMS_TRANSLATION_COMMIT, SYSTEMS_TRANSLATION_EDIT_BEGIN, SYSTEMS_TRANSLATION_EDIT_COMMIT,
    SYSTEMS_TRANSLATION_EDIT_MAP, SYSTEMS_TRANSLATION_EDIT_UNMAP, validate_systems_program,
};
use topal_language::compiler::{
    SYSTEMS_DEADLINE_AFTER, SYSTEMS_DEADLINE_ARM, SYSTEMS_DEADLINE_COMPLETE, SYSTEMS_DEADLINE_WAIT,
    SYSTEMS_RESUME_DEADLINE,
};

use crate::CompileError;

pub const X86_SYSTEMS_PROVIDER_REVISION: &str = "topal.provider.x86_64-qemu-pc-q35/11";
pub const X86_SYSTEMS_PLATFORM_ABI: &str = "topal.systems.x86_64-bare/1";
pub const X86_SYSTEMS_DATA_LAYOUT: &str =
    "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum X86SystemsLowering {
    LinuxBootParamsE820,
    RetainedE820FrameAuthority,
    E820SingleFrameSelect,
    ConsumePhysicalFrameExtent,
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
    StaticBootstrapStorage,
    MonotonicBootstrapAllocate,
    PlainBootstrapRegionStoreByte,
    PlainBootstrapRegionLoadByte,
    ConsumeBootstrapRegion,
    CompleteBootstrapStorage,
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
            Self::E820SingleFrameSelect => "topal.provider.x86_64.frames.select-one-e820/1",
            Self::ConsumePhysicalFrameExtent => "topal.provider.x86_64.frames.consume-extent/1",
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
            Self::PolledUart16550PortIo
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
        SYSTEMS_FRAMES_ALLOCATE => X86SystemsLowering::E820SingleFrameSelect,
        SYSTEMS_FRAMES_RELEASE => X86SystemsLowering::ConsumePhysicalFrameExtent,
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
        SYSTEMS_BOOTSTRAP_STORAGE_PROVISION => X86SystemsLowering::StaticBootstrapStorage,
        SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE => X86SystemsLowering::MonotonicBootstrapAllocate,
        SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE => X86SystemsLowering::PlainBootstrapRegionStoreByte,
        SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE => X86SystemsLowering::PlainBootstrapRegionLoadByte,
        SYSTEMS_BOOTSTRAP_STORAGE_RELEASE => X86SystemsLowering::ConsumeBootstrapRegion,
        SYSTEMS_BOOTSTRAP_STORAGE_COMPLETE => X86SystemsLowering::CompleteBootstrapStorage,
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
            X86SystemsLowering::E820SingleFrameSelect => SYSTEMS_FRAMES_ALLOCATE,
            X86SystemsLowering::ConsumePhysicalFrameExtent => SYSTEMS_FRAMES_RELEASE,
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
            X86SystemsLowering::StaticBootstrapStorage => SYSTEMS_BOOTSTRAP_STORAGE_PROVISION,
            X86SystemsLowering::MonotonicBootstrapAllocate => SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
            X86SystemsLowering::PlainBootstrapRegionStoreByte => {
                SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE
            }
            X86SystemsLowering::PlainBootstrapRegionLoadByte => SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
            X86SystemsLowering::ConsumeBootstrapRegion => SYSTEMS_BOOTSTRAP_STORAGE_RELEASE,
            X86SystemsLowering::CompleteBootstrapStorage => SYSTEMS_BOOTSTRAP_STORAGE_COMPLETE,
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
        assert_eq!(plan.operations.len(), 41);
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_BOOT_MEMORY_DESCRIBE
                && operation.lowering == X86SystemsLowering::LinuxBootParamsE820
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_FRAME_ALLOCATOR_CREATE
                && operation.lowering == X86SystemsLowering::RetainedE820FrameAuthority
        }));
        assert!(plan.operations.iter().any(|operation| {
            operation.semantic_identity == SYSTEMS_FRAMES_ALLOCATE
                && operation.lowering == X86SystemsLowering::E820SingleFrameSelect
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
                SYSTEMS_DEADLINE_ARM,
                X86SystemsLowering::ArmHpetIoApicDeadline,
            ),
            (SYSTEMS_DEADLINE_WAIT, X86SystemsLowering::WaitForDeadline),
            (
                SYSTEMS_DEADLINE_COMPLETE,
                X86SystemsLowering::CompleteHpetIoApicDeadline,
            ),
            (SYSTEMS_RESUME_DEADLINE, X86SystemsLowering::ResumeDeadline),
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
        for forbidden in ["linux-gnu", "syscall", "libc", "dynamic-loader"] {
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
