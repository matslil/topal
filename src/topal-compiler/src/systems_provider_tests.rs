use topal_language::compiler::{CompilerSystemsTargetSelection, analyze_systems_for_compiler};

use super::*;

const SOURCE: &str = include_str!("../../../linux-kernel/labs/qemu/x86_64/pre-frame-kernel-root.t");

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
    assert_eq!(plan.operations.len(), 153);
    assert_eq!(
        plan.region_library_interface.as_deref(),
        Some("borrowed-byte-region-instance-action-v1")
    );
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
            && operation.lowering == X86SystemsLowering::PciConfigurationGatewayRegionReadEventually
    }));
    assert!(plan.operations.iter().any(|operation| {
        operation.semantic_identity == SYSTEMS_PCI_REGION_WRITE
            && operation.lowering == X86SystemsLowering::PciConfigurationGatewayRegionWrite
    }));
    assert!(plan.operations.iter().any(|operation| {
        operation.semantic_identity == SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE
            && operation.lowering == X86SystemsLowering::PciConfigurationGatewayDmaReferenceWrite
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
