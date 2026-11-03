use super::*;
use crate::analyze_for_compiler;

const SOURCE: &str = include_str!("../../../linux-kernel/kernel/arch/x86_64/toolchain-gate.t");
const STORAGE_DECLARATION: &str = "  bootstrap-storage is lang systems bounded-bootstrap-storage (\n    capacity-bytes is 65536,\n    alignment-bytes is 4096\n  ),\n";

fn memory_source() -> String {
    SOURCE.to_owned()
}

#[test]
fn root_region_requires_exact_declaration_and_one_owned_call() {
    // TOPAL-SYSTEMS-REGION-ROOT-BINDING-001, -CALL-001.
    let target = CompilerSystemsTargetSelection::initial_x86_64_qemu();
    let valid = analyze_systems_for_compiler(SOURCE, &target).unwrap();
    assert_eq!(
        valid.region_library.as_ref().unwrap().interface,
        CompilerSystemsSourceLibraryInterface::BorrowedByteRegionInstanceActionV1
    );
    assert_eq!(
        valid
            .bootstrap
            .handler
            .operations
            .iter()
            .filter(|op| matches!(op, CompilerSystemsOperation::ApplyBootstrapRegionLibrary))
            .count(),
        1
    );
    let start = SOURCE.find("  region-library is").unwrap();
    let end = start + SOURCE[start..].find("  ),\n").unwrap() + 5;
    let declaration = &SOURCE[start..end];
    let legacy = declaration
        .replace(
            "borrowed-byte-region-instance-action-v1",
            "borrowed-byte-region-v1",
        )
        .replace("bootstrap-byte-copy-frame.t", "bootstrap-byte-copy.t");
    let legacy =
        analyze_systems_for_compiler(&SOURCE.replace(declaration, &legacy), &target).unwrap();
    assert_eq!(
        legacy.region_library.unwrap().interface,
        CompilerSystemsSourceLibraryInterface::BorrowedByteRegionV1
    );
    assert_eq!(
        valid.file_position_library.as_ref().unwrap().interface,
        CompilerSystemsSourceLibraryInterface::BorrowedByteRegionInstanceActionV1
    );
    let call_line = SOURCE
        .lines()
        .find(|line| line.trim() == "region library apply")
        .unwrap();
    for (case, invalid) in [
        SOURCE.replace(declaration, ""),
        SOURCE.replace(call_line, ""),
        SOURCE.replace(declaration, &format!("{declaration}{declaration}")),
        SOURCE.replace(
            declaration,
            &declaration.replace(
                "borrowed-byte-region-instance-action-v1",
                "borrowed-byte-region-framed-action-v1",
            ),
        ),
        SOURCE.replace(
            declaration,
            &declaration.replace(
                "    export is",
                "    instance is root-region,\n    export is",
            ),
        ),
        SOURCE.replace(
            declaration,
            &declaration.replace(
                "borrowed-byte-region-instance-action-v1",
                "relative-sleep-deadline-v1",
            ),
        ),
        SOURCE.replacen("region library apply", "other library apply", 1),
        SOURCE.replace(call_line, &format!("{call_line}\n{call_line}")),
        SOURCE.replace(declaration, &declaration.replace("4096", "2048")),
        SOURCE.replace(declaration, &declaration.replace("65536", "32768")),
        SOURCE.replace(
            declaration,
            &declaration.replace(
                "    runtime-limit-bytes",
                "    state-limit-bytes is 4096,\n    runtime-limit-bytes",
            ),
        ),
        SOURCE.replace(
            declaration,
            &declaration.replace("policy/bootstrap-byte-copy-frame.t", "../outside.t"),
        ),
    ]
    .into_iter()
    .enumerate()
    {
        assert!(
            analyze_systems_for_compiler(&invalid, &target).is_err(),
            "accepted invalid root region source case {case}"
        );
    }
    let absent = SOURCE
        .replace(declaration, "")
        .replace(&format!("{call_line}\n"), "")
        .replace(
            "observed : Nat is region byte load (offset-bytes is 1)",
            "observed : Nat is region byte load (offset-bytes is 0)",
        );
    assert!(
        analyze_systems_for_compiler(&absent, &target)
            .unwrap()
            .region_library
            .is_none()
    );
}

fn is_queue_finalization(operation: &CompilerSystemsOperation) -> bool {
    matches!(
        operation,
        CompilerSystemsOperation::AccessPciRegionRegister { request }
            if CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
                .contains(request)
                || CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION
                    .contains(request)
                || CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL
                    .contains(request)
    ) || matches!(
        operation,
        CompilerSystemsOperation::CompleteDmaBufferForCpu { .. }
            | CompilerSystemsOperation::LoadDmaBufferIntegerEquals { .. }
            | CompilerSystemsOperation::CaptureEntropySample { .. }
            | CompilerSystemsOperation::AssessEntropySampleQuality { .. }
            | CompilerSystemsOperation::CreateEntropyPool { .. }
            | CompilerSystemsOperation::MixEntropyPoolSample { .. }
            | CompilerSystemsOperation::CreditEntropyPoolContribution { .. }
            | CompilerSystemsOperation::InitializeEntropyGenerator { .. }
            | CompilerSystemsOperation::ExtractEntropyGenerator { .. }
            | CompilerSystemsOperation::DeliverInitialAuxvRandom { .. }
            | CompilerSystemsOperation::ReserveInitialUserRandom { .. }
            | CompilerSystemsOperation::BindReusableUserRandom { .. }
            | CompilerSystemsOperation::ExpandSmallUserRandomRequests { .. }
            | CompilerSystemsOperation::ExpandLargeUserRandomRequests { .. }
    )
}

fn remove_entropy_sample_capture(program: &mut CompilerSystemsProgram) {
    program.bootstrap.handler.operations.retain(|operation| {
        !matches!(
            operation,
            CompilerSystemsOperation::CaptureEntropySample { .. }
                | CompilerSystemsOperation::AssessEntropySampleQuality { .. }
                | CompilerSystemsOperation::CreateEntropyPool { .. }
                | CompilerSystemsOperation::MixEntropyPoolSample { .. }
                | CompilerSystemsOperation::CreditEntropyPoolContribution { .. }
                | CompilerSystemsOperation::InitializeEntropyGenerator { .. }
                | CompilerSystemsOperation::ExtractEntropyGenerator { .. }
                | CompilerSystemsOperation::DeliverInitialAuxvRandom { .. }
                | CompilerSystemsOperation::ReserveInitialUserRandom { .. }
                | CompilerSystemsOperation::BindReusableUserRandom { .. }
                | CompilerSystemsOperation::ExpandSmallUserRandomRequests { .. }
                | CompilerSystemsOperation::ExpandLargeUserRandomRequests { .. }
        )
    });
    program.bootstrap.handler.effects.retain(|effect| {
        *effect != SYSTEMS_ENTROPY_SAMPLE_CAPTURE_DMA
            && *effect != SYSTEMS_ENTROPY_SAMPLE_ASSESS_QUALITY
            && *effect != SYSTEMS_ENTROPY_POOL_CREATE
            && *effect != SYSTEMS_ENTROPY_POOL_MIX_SAMPLE
            && *effect != SYSTEMS_ENTROPY_POOL_CREDIT_CONTRIBUTION
            && *effect != SYSTEMS_ENTROPY_GENERATOR_INITIALIZE
            && *effect != SYSTEMS_ENTROPY_GENERATOR_EXTRACT
            && *effect != SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM
            && *effect != SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM
            && *effect != SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE
            && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS
            && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS
    });
}

macro_rules! assert_initial_transition_shape {
        ($actual:expr, $legacy_expected:expr) => {{
            let transitions = $actual;
            let _ = stringify!($legacy_expected);
            assert!(matches!(
                transitions.first(),
                Some(CompilerSystemsTransition::ProvisionBootstrapStorage { .. })
            ));
            assert!(matches!(
                transitions.last(),
                Some(CompilerSystemsTransition::Fatal { message })
                    if message == "toolchain gate complete"
            ));
            let delivery = transitions
                .iter()
                .position(|transition| matches!(
                    transition,
                    CompilerSystemsTransition::DeliverInitialAuxvRandom { request }
                        if *request == CompilerEntropyOutputDeliveryRequest::INITIAL_AUXV_RANDOM
                ))
                .expect("modeled initial AT_RANDOM delivery");
            let entry = transitions
                .iter()
                .position(|transition| matches!(
                    transition,
                    CompilerSystemsTransition::EnterUserExecution { .. }
                ))
                .expect("modeled initial user entry");
            let reservation = transitions
                .iter()
                .position(|transition| matches!(
                    transition,
                    CompilerSystemsTransition::ReserveInitialUserRandom { request }
                        if *request == CompilerEntropyOutputReservationRequest::INITIAL_GETRANDOM
                ))
                .expect("modeled initial getrandom reservation");
            let binding = transitions
                .iter()
                .position(|transition| matches!(
                    transition,
                    CompilerSystemsTransition::BindReusableUserRandom { request }
                        if *request == CompilerEntropyUserRandomServiceRequest::INITIAL_GETRANDOM
                ))
                .expect("modeled reusable getrandom binding");
            assert!(delivery < reservation && reservation < binding && binding < entry);
        }};
    }

#[test]
#[allow(clippy::too_many_lines)] // One assertion retains the complete ordered systems trace.
fn checks_the_initial_artifact_and_models_entry_transitions() {
    // TOPAL-SYSTEMS-VOCABULARY-001, TOPAL-SYSTEMS-ENTRY-001,
    // TOPAL-SYSTEMS-DISPOSITION-001, TOPAL-SYSTEMS-OBSERVATION-001,
    // TOPAL-SYSTEMS-FRAMES-001, TOPAL-SYSTEMS-MONOTONIC-CLOCK-001,
    // TOPAL-SYSTEMS-DEADLINE-EVENT-001.
    let program = analyze_systems_for_compiler(
        SOURCE,
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    assert_eq!(program.bootstrap.kind, CompilerSystemsEntryKind::Bootstrap);
    assert_eq!(program.bootstrap_storage.capacity_bytes, 65_536);
    assert_eq!(program.bootstrap_storage.alignment_bytes, 4096);
    assert_eq!(
        program.debug_break.kind,
        CompilerSystemsEntryKind::SynchronousExceptionDebugBreak
    );
    assert_eq!(
        program.bootstrap.handler.effects,
        [
            SYSTEMS_ATOMIC_COMPARE_EXCHANGE,
            SYSTEMS_ATOMIC_WORD_CREATE,
            SYSTEMS_ATOMIC_END,
            SYSTEMS_ATOMIC_LOAD,
            SYSTEMS_BOOT_MEMORY_DESCRIBE,
            SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE,
            SYSTEMS_KERNEL_COMPLETION_CONSUME,
            SYSTEMS_KERNEL_COMPLETION_CREATE,
            SYSTEMS_KERNEL_COMPLETION_SIGNAL,
            SYSTEMS_KERNEL_CONTEXT_CREATE,
            SYSTEMS_KERNEL_CONTEXT_RECLAIM,
            SYSTEMS_KERNEL_RUNNABLE_QUEUE_CONSUME,
            SYSTEMS_KERNEL_RUNNABLE_QUEUE_CREATE,
            SYSTEMS_KERNEL_RUNNABLE_QUEUE_DEQUEUE,
            SYSTEMS_KERNEL_RUNNABLE_QUEUE_ENQUEUE,
            SYSTEMS_KERNEL_CONTEXT_TAKE_PREEMPTED,
            SYSTEMS_KERNEL_CONTEXT_TRANSFER_UNTIL_DEADLINE,
            SYSTEMS_KERNEL_CONTEXT_TRANSFER,
            SYSTEMS_USER_CONTEXT_COMPLETE,
            SYSTEMS_USER_CONTEXT_ENTER,
            SYSTEMS_USER_CONTEXT_PREPARE,
            SYSTEMS_CRITICAL_ENTER,
            SYSTEMS_CRITICAL_RESTORE,
            SYSTEMS_CONSOLE_WRITE,
            SYSTEMS_PCI_CONFIGURATION_READ_DWORD,
            SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD,
            SYSTEMS_PCI_REGION_READ_EVENTUALLY,
            SYSTEMS_PCI_REGION_READ,
            SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE,
            SYSTEMS_PCI_REGION_WRITE,
            SYSTEMS_FATAL,
            SYSTEMS_DMA_BUFFER_BIND_REFERENCE,
            SYSTEMS_DMA_BUFFER_COMPLETE_CPU,
            SYSTEMS_DMA_BUFFER_CREATE,
            SYSTEMS_DMA_BUFFER_LOAD_INTEGER,
            SYSTEMS_DMA_BUFFER_PREPARE_DEVICE,
            SYSTEMS_DMA_BUFFER_STORE_INTEGER,
            SYSTEMS_ENTROPY_GENERATOR_EXTRACT,
            SYSTEMS_ENTROPY_GENERATOR_INITIALIZE,
            SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM,
            SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM,
            SYSTEMS_ENTROPY_POOL_CREATE,
            SYSTEMS_ENTROPY_POOL_CREDIT_CONTRIBUTION,
            SYSTEMS_ENTROPY_POOL_MIX_SAMPLE,
            SYSTEMS_ENTROPY_SAMPLE_ASSESS_QUALITY,
            SYSTEMS_ENTROPY_SAMPLE_CAPTURE_DMA,
            SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE,
            SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS,
            SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS,
            SYSTEMS_FRAMES_ALLOCATE,
            SYSTEMS_FRAMES_RELEASE,
            SYSTEMS_LOCAL_NOTIFICATION_SEND,
            SYSTEMS_LOCAL_NOTIFICATION_WAIT,
            SYSTEMS_DEBUG_BREAK,
            SYSTEMS_KERNEL_MAPPING_LOAD_BYTE,
            SYSTEMS_KERNEL_MAP,
            SYSTEMS_KERNEL_MAPPING_STORE_BYTE,
            SYSTEMS_KERNEL_UNMAP,
            SYSTEMS_FRAME_ALLOCATOR_CREATE,
            SYSTEMS_USER_ADDRESS_SPACE_MAP,
            SYSTEMS_USER_ADDRESS_SPACE_PREPARE,
            SYSTEMS_USER_ADDRESS_SPACE_RECLAIM,
            SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS,
            SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS,
            SYSTEMS_USER_ADDRESS_SPACE_UNMAP,
            SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE,
            SYSTEMS_USER_MAPPING_LOAD_BYTE,
            SYSTEMS_USER_MAPPING_PERMIT_EXECUTE,
            SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA,
            SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY,
            SYSTEMS_USER_MAPPING_STORE_BYTE,
            SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE,
            SYSTEMS_REGION_LIBRARY_APPLY,
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
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE_ACTIVATION,
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE_ADDRESSES,
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE,
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
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_STACK_BACKING,
            SYSTEMS_STATIC_IMAGE_PROCESS_ENTER,
            SYSTEMS_STATIC_IMAGE_PROCESS_RESTORE_ENTRY,
            SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
            SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE,
            SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
            SYSTEMS_BOOTSTRAP_STORAGE_RELEASE,
            SYSTEMS_DEADLINE_AFTER,
            SYSTEMS_MONOTONIC_CLOCK_NOW,
            SYSTEMS_TRANSLATION_ACTIVATE,
            SYSTEMS_TRANSLATION_BEGIN,
            SYSTEMS_TRANSLATION_COMMIT,
            SYSTEMS_TRANSLATION_EDIT_BEGIN,
            SYSTEMS_TRANSLATION_EDIT_COMMIT,
            SYSTEMS_TRANSLATION_EDIT_MAP,
            SYSTEMS_TRANSLATION_EDIT_UNMAP,
        ]
    );
    assert_initial_transition_shape!(
        model_systems_transitions(&program).unwrap(),
        vec![
            CompilerSystemsTransition::ProvisionBootstrapStorage {
                capacity_bytes: 65_536,
                alignment_bytes: 4096,
            },
            CompilerSystemsTransition::EnterBootstrap,
            CompilerSystemsTransition::DescribeBootMemory,
            CompilerSystemsTransition::CreateFrameAllocator,
            CompilerSystemsTransition::AllocatePhysicalFrames {
                request: CompilerPhysicalFrameRequest {
                    frame_count: 1,
                    alignment_frames: 1,
                },
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_FRAME_ALLOCATED".into(),
            },
            CompilerSystemsTransition::MapKernelFrames {
                request: CompilerKernelMappingRequest::initial_read_write(),
            },
            CompilerSystemsTransition::StoreKernelMappingByte {
                offset_bytes: 0,
                value: 165,
            },
            CompilerSystemsTransition::LoadKernelMappingByte {
                offset_bytes: 0,
                value: 165,
            },
            CompilerSystemsTransition::UnmapKernelFrames,
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_FRAME_MAPPED".into(),
            },
            CompilerSystemsTransition::ReleasePhysicalFrames,
            CompilerSystemsTransition::BeginTranslationUpdate {
                request: CompilerTranslationUpdateRequest::initial_bootstrap_equivalent(),
            },
            CompilerSystemsTransition::CommitTranslationUpdate,
            CompilerSystemsTransition::ActivateTranslationSpace,
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_TRANSLATION_ACTIVE".into(),
            },
            CompilerSystemsTransition::AllocatePhysicalFrames {
                request: CompilerPhysicalFrameRequest {
                    frame_count: 1,
                    alignment_frames: 1,
                },
            },
            CompilerSystemsTransition::BeginTranslationEdit {
                kind: CompilerTranslationEditKind::Map,
            },
            CompilerSystemsTransition::MapTranslationFrames {
                request: CompilerTranslationMappingRequest::initial_read_write(),
            },
            CompilerSystemsTransition::CommitTranslationEdit {
                kind: CompilerTranslationEditKind::Map,
            },
            CompilerSystemsTransition::StoreKernelMappingByte {
                offset_bytes: 0,
                value: 60,
            },
            CompilerSystemsTransition::LoadKernelMappingByte {
                offset_bytes: 0,
                value: 60,
            },
            CompilerSystemsTransition::BeginTranslationEdit {
                kind: CompilerTranslationEditKind::Unmap,
            },
            CompilerSystemsTransition::UnmapTranslationMapping,
            CompilerSystemsTransition::CommitTranslationEdit {
                kind: CompilerTranslationEditKind::Unmap,
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_TRANSLATION_EDITED".into(),
            },
            CompilerSystemsTransition::ReleasePhysicalFrames,
            CompilerSystemsTransition::AllocatePhysicalFrames {
                request: CompilerPhysicalFrameRequest {
                    frame_count: 3,
                    alignment_frames: 1,
                },
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_FRAME_EXTENT_OK".into(),
            },
            CompilerSystemsTransition::ReleasePhysicalFrames,
            CompilerSystemsTransition::EnterCritical {
                domain: CompilerCriticalDomain::LocalMaskableInterrupts,
                nesting_identity: 1,
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_INTERRUPTS_MASKED".into(),
            },
            CompilerSystemsTransition::RestoreCritical {
                domain: CompilerCriticalDomain::LocalMaskableInterrupts,
                nesting_identity: 1,
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_MEMORY_DESCRIBED".into(),
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 1,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[0],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 2,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[1],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 3,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[2],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 4,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[3],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 5,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[4],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 6,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[5],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 7,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[6],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 8,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[7],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 9,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[8],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 10,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[9],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 11,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[10],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 12,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[11],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 13,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[12],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 14,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[13],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 15,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[14],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 16,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[15],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 17,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[16],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 18,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[17],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 19,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[18],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 20,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[19],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 21,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[20],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 22,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[21],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 23,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[22],
            },
            CompilerSystemsTransition::ObservePciConfigurationDword {
                transaction_identity: 24,
                request: CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[23],
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_PCI_VIRTIO_RNG_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_VIRTIO_PCI_CAPABILITIES_OK".into(),
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 1,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[0],
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 2,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[1],
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 3,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[2],
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 4,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[3],
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 5,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[4],
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 6,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[5],
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 7,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[6],
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 8,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[7],
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 9,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[8],
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 10,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[9],
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 11,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[10],
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 12,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[11],
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 13,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[12],
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 14,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[13],
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 15,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[14],
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 16,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[15],
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_VIRTIO_PCI_NEGOTIATED".into(),
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 17,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY
                    [0],
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 18,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY
                    [1],
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 19,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY
                    [2],
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 20,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY
                    [3],
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 21,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY
                    [4],
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_VIRTIO_QUEUE_DISCOVERED".into(),
            },
            CompilerSystemsTransition::UpdatePciConfigurationWord {
                transaction_identity: 1,
                request:
                    CompilerPciConfigurationWordUpdateRequest::INITIAL_Q35_VIRTIO_RNG_BUS_MASTER,
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_PCI_BUS_MASTER_OK".into(),
            },
            CompilerSystemsTransition::CreateDmaBuffer {
                request: CompilerDmaBufferRequest::INITIAL_Q35_VIRTIO_RNG[0],
            },
            CompilerSystemsTransition::CreateDmaBuffer {
                request: CompilerDmaBufferRequest::INITIAL_Q35_VIRTIO_RNG[1],
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_DMA_BUFFERS_OK".into(),
            },
            CompilerSystemsTransition::BindDmaBufferReference {
                request: CompilerDmaBufferReferenceRequest::INITIAL_Q35_VIRTIO_RNG_DESCRIPTOR,
            },
            CompilerSystemsTransition::StoreDmaBufferInteger {
                request: CompilerDmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING[0],
            },
            CompilerSystemsTransition::StoreDmaBufferInteger {
                request: CompilerDmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING[1],
            },
            CompilerSystemsTransition::StoreDmaBufferInteger {
                request: CompilerDmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING[2],
            },
            CompilerSystemsTransition::StoreDmaBufferInteger {
                request: CompilerDmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING[3],
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_VIRTIO_SPLIT_RING_OK".into(),
            },
            CompilerSystemsTransition::PrepareDmaBufferForDevice {
                request: CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0],
            },
            CompilerSystemsTransition::PrepareDmaBufferForDevice {
                request: CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[1],
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_DMA_DEVICE_OWNED".into(),
            },
            CompilerSystemsTransition::PublishPciRegionDmaReference {
                transaction_identity: 22,
                request:
                    CompilerPciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES[0],
            },
            CompilerSystemsTransition::PublishPciRegionDmaReference {
                transaction_identity: 23,
                request:
                    CompilerPciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES[1],
            },
            CompilerSystemsTransition::PublishPciRegionDmaReference {
                transaction_identity: 24,
                request:
                    CompilerPciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES[2],
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_VIRTIO_QUEUE_ADDRESSES_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_BOOT".into(),
            },
            CompilerSystemsTransition::ObserveDebugBreak,
            CompilerSystemsTransition::EnterDebugBreak,
            CompilerSystemsTransition::ResumeDebugBreak,
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_FAULT_RESUMED".into(),
            },
            CompilerSystemsTransition::AllocateBootstrapRegion {
                request: CompilerBootstrapStorageRequest {
                    byte_count: 64,
                    alignment_bytes: 8,
                    placement: CompilerBootstrapStoragePlacement::BootstrapReclaimable,
                },
            },
            CompilerSystemsTransition::StoreBootstrapByte {
                offset_bytes: 0,
                value: 90,
            },
            CompilerSystemsTransition::BorrowBootstrapRegionForLibrary { generation: 1 },
            CompilerSystemsTransition::RequireRegionLibraryReclamation,
            CompilerSystemsTransition::ResumeBootstrapRegionAfterLibrary { require_true: true },
            CompilerSystemsTransition::CheckDynamicBootstrapByte {
                offset_bytes: 1,
                expected: 90,
            },
            CompilerSystemsTransition::CreateAtomicWord {
                request: CompilerAtomicWordRequest::initial(),
            },
            CompilerSystemsTransition::CompareExchangeAtomicWord {
                expected: 41,
                desired: 42,
                observed: 41,
                exchanged: true,
                success_order: CompilerAtomicOrder::AcquireRelease,
                failure_order: CompilerAtomicOrder::Acquire,
            },
            CompilerSystemsTransition::LoadAtomicWord {
                value: 42,
                order: CompilerAtomicOrder::Acquire,
            },
            CompilerSystemsTransition::EndAtomicWord,
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_ATOMIC_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_MEMORY_OK".into(),
            },
            CompilerSystemsTransition::ReleaseBootstrapRegion,
            CompilerSystemsTransition::SendLocalNotification { event_identity: 1 },
            CompilerSystemsTransition::BeginLocalNotificationWait { event_identity: 1 },
            CompilerSystemsTransition::ObserveLocalNotification { event_identity: 1 },
            CompilerSystemsTransition::EnterLocalNotificationInterrupt { event_identity: 1 },
            CompilerSystemsTransition::CompleteLocalNotificationInterrupt { event_identity: 1 },
            CompilerSystemsTransition::ResumeLocalNotificationInterrupt { event_identity: 1 },
            CompilerSystemsTransition::EndLocalNotificationWait { event_identity: 1 },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_INTERRUPT_OK".into(),
            },
            CompilerSystemsTransition::ObserveMonotonicClock {
                observation_identity: 1,
            },
            CompilerSystemsTransition::ObserveMonotonicClock {
                observation_identity: 2,
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_TIME_OK".into(),
            },
            CompilerSystemsTransition::ConstructDeadline {
                source_observation_identity: 2,
                duration_nanoseconds: 1_000_000,
            },
            CompilerSystemsTransition::AllocateBootstrapRegion {
                request: CompilerBootstrapStorageRequest {
                    byte_count: 16_384,
                    alignment_bytes: 16,
                    placement: CompilerBootstrapStoragePlacement::BootstrapReclaimable,
                },
            },
            CompilerSystemsTransition::CreateKernelCompletion {
                completion_identity: 1,
            },
            CompilerSystemsTransition::CreateKernelContext {
                context_identity: 1,
            },
            CompilerSystemsTransition::AllocateBootstrapRegion {
                request: CompilerBootstrapStorageRequest {
                    byte_count: 16_384,
                    alignment_bytes: 16,
                    placement: CompilerBootstrapStoragePlacement::BootstrapReclaimable,
                },
            },
            CompilerSystemsTransition::CreateKernelContext {
                context_identity: 2,
            },
            CompilerSystemsTransition::AllocateBootstrapRegion {
                request: CompilerBootstrapStorageRequest {
                    byte_count: 16_384,
                    alignment_bytes: 16,
                    placement: CompilerBootstrapStoragePlacement::BootstrapReclaimable,
                },
            },
            CompilerSystemsTransition::CreateKernelContext {
                context_identity: 3,
            },
            CompilerSystemsTransition::CreateRunnableQueue { capacity: 3 },
            CompilerSystemsTransition::EnqueueRunnableContext {
                context_identity: 1,
            },
            CompilerSystemsTransition::EnqueueRunnableContext {
                context_identity: 2,
            },
            CompilerSystemsTransition::EnqueueRunnableContext {
                context_identity: 3,
            },
            CompilerSystemsTransition::DequeueRunnableContext {
                context_identity: 1,
            },
            CompilerSystemsTransition::ArmDeadline { event_identity: 1 },
            CompilerSystemsTransition::TransferKernelContextUntilDeadline {
                context_identity: 1,
                caller_identity: 1,
                event_identity: 1,
            },
            CompilerSystemsTransition::EnterKernelThread {
                context_identity: 1,
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_ENTERED".into(),
            },
            CompilerSystemsTransition::AwaitDeadlinePreemption {
                context_identity: 1,
            },
            CompilerSystemsTransition::ObserveDeadline { event_identity: 1 },
            CompilerSystemsTransition::EnterDeadlineInterrupt { event_identity: 1 },
            CompilerSystemsTransition::CompleteDeadlineInterrupt { event_identity: 1 },
            CompilerSystemsTransition::PreemptKernelContext {
                event_identity: 1,
                context_identity: 1,
                caller_identity: 1,
            },
            CompilerSystemsTransition::ResumePreemptionDispatcher { caller_identity: 1 },
            CompilerSystemsTransition::TakePreemptedKernelContext {
                context_identity: 1,
                event_identity: 1,
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_DEADLINE_PREEMPTED".into(),
            },
            CompilerSystemsTransition::EnqueueRunnableContext {
                context_identity: 1,
            },
            CompilerSystemsTransition::DequeueRunnableContext {
                context_identity: 2,
            },
            CompilerSystemsTransition::TransferKernelContextUntilCompletion {
                context_identity: 2,
                caller_identity: 2,
                completion_identity: 1,
            },
            CompilerSystemsTransition::EnterKernelThread {
                context_identity: 2,
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_ENTERED".into(),
            },
            CompilerSystemsTransition::AwaitKernelCompletion {
                context_identity: 2,
                completion_identity: 1,
            },
            CompilerSystemsTransition::BlockKernelContext {
                context_identity: 2,
                caller_identity: 2,
                completion_identity: 1,
            },
            CompilerSystemsTransition::ResumeCompletionDispatcher { caller_identity: 2 },
            CompilerSystemsTransition::DequeueRunnableContext {
                context_identity: 3,
            },
            CompilerSystemsTransition::TransferKernelContext {
                context_identity: 3,
                caller_identity: 3,
            },
            CompilerSystemsTransition::EnterKernelThread {
                context_identity: 3,
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_CONTEXT_TERMINAL_ENTERED".into(),
            },
            CompilerSystemsTransition::RetireKernelContextToCaller {
                context_identity: 3,
                caller_identity: 3,
            },
            CompilerSystemsTransition::ResumeKernelContextCaller { caller_identity: 3 },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_CONTEXT_TERMINAL_RETIRED".into(),
            },
            CompilerSystemsTransition::ReclaimKernelContext {
                context_identity: 3,
            },
            CompilerSystemsTransition::SignalBlockedKernelCompletion {
                context_identity: 2,
                caller_identity: 2,
                completion_identity: 1,
            },
            CompilerSystemsTransition::EnqueueRunnableContext {
                context_identity: 2,
            },
            CompilerSystemsTransition::DequeueRunnableContext {
                context_identity: 1,
            },
            CompilerSystemsTransition::TransferKernelContext {
                context_identity: 1,
                caller_identity: 4,
            },
            CompilerSystemsTransition::RestorePreemptedKernelContext {
                context_identity: 1,
                caller_identity: 4,
                event_identity: 1,
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_RESUMED".into(),
            },
            CompilerSystemsTransition::RetireKernelContextToCaller {
                context_identity: 1,
                caller_identity: 4,
            },
            CompilerSystemsTransition::ResumeKernelContextCaller { caller_identity: 4 },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_RETIRED".into(),
            },
            CompilerSystemsTransition::ReclaimKernelContext {
                context_identity: 1,
            },
            CompilerSystemsTransition::DequeueRunnableContext {
                context_identity: 2,
            },
            CompilerSystemsTransition::TransferKernelContext {
                context_identity: 2,
                caller_identity: 5,
            },
            CompilerSystemsTransition::RestoreBlockedKernelContext {
                context_identity: 2,
                caller_identity: 5,
                completion_identity: 1,
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_RESUMED".into(),
            },
            CompilerSystemsTransition::RetireKernelContextToCaller {
                context_identity: 2,
                caller_identity: 5,
            },
            CompilerSystemsTransition::ResumeKernelContextCaller { caller_identity: 5 },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_RETIRED".into(),
            },
            CompilerSystemsTransition::ReclaimKernelContext {
                context_identity: 2,
            },
            CompilerSystemsTransition::ConsumeKernelCompletion {
                completion_identity: 1,
            },
            CompilerSystemsTransition::ConsumeEmptyRunnableQueue,
            CompilerSystemsTransition::AllocatePhysicalFrames {
                request: CompilerPhysicalFrameRequest {
                    frame_count: 3,
                    alignment_frames: 1,
                },
            },
            CompilerSystemsTransition::PrepareUserAddressSpace {
                address_space_identity: 1,
            },
            CompilerSystemsTransition::AllocatePhysicalFrames {
                request: CompilerPhysicalFrameRequest {
                    frame_count: 2,
                    alignment_frames: 1,
                },
            },
            CompilerSystemsTransition::MapUserAddressSpace {
                address_space_identity: 1,
                mapping_identity: 1,
            },
            CompilerSystemsTransition::InstallUserMappingEntryTrampoline {
                mapping_identity: 1,
            },
            CompilerSystemsTransition::ProtectUserMappingReadOnly {
                mapping_identity: 1,
            },
            CompilerSystemsTransition::VerifyUserMappingEntryTrampoline {
                mapping_identity: 1,
            },
            CompilerSystemsTransition::PermitUserMappingExecute {
                mapping_identity: 1,
            },
            CompilerSystemsTransition::AllocatePhysicalFrames {
                request: CompilerPhysicalFrameRequest {
                    frame_count: 1,
                    alignment_frames: 1,
                },
            },
            CompilerSystemsTransition::MapUserAddressSpace {
                address_space_identity: 1,
                mapping_identity: 2,
            },
            CompilerSystemsTransition::StoreUserMappingByte {
                mapping_identity: 2,
            },
            CompilerSystemsTransition::ProtectUserMappingReadOnly {
                mapping_identity: 2,
            },
            CompilerSystemsTransition::LoadUserMappingByte {
                mapping_identity: 2,
            },
            CompilerSystemsTransition::PermitUserMappingWritableData {
                mapping_identity: 2,
            },
            CompilerSystemsTransition::AllocatePhysicalFrames {
                request: CompilerPhysicalFrameRequest {
                    frame_count: 34,
                    alignment_frames: 1,
                },
            },
            CompilerSystemsTransition::ReserveAnonymousUserMapping {
                address_space_identity: 1,
                reservation_identity: 3,
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_IMAGE_READY".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_BOOT_RESOURCE_OK".into(),
            },
            CompilerSystemsTransition::PrepareUserProbe {
                process_identity: 1,
                probe_identity: 1,
            },
            CompilerSystemsTransition::EnterUserExecution {
                process_identity: 1,
                kernel_continuation_identity: 1,
            },
            CompilerSystemsTransition::EnterUserSyscall {
                entry_identity: 1,
                number: 39,
                arguments: [0; 6],
            },
            CompilerSystemsTransition::ReturnToUser {
                entry_identity: 1,
                result: 1,
            },
            CompilerSystemsTransition::EnterUserSyscall {
                entry_identity: 2,
                number: 63,
                arguments: [6_299_136, 0, 0, 0, 0, 0],
            },
            CompilerSystemsTransition::BeginUserTransfer {
                entry_identity: 2,
                transfer_identity: 1,
                candidate: 6_299_136,
                extent: 390,
            },
            CompilerSystemsTransition::CopyToUserComplete {
                transfer_identity: 1,
                copied: 390,
            },
            CompilerSystemsTransition::EndUserTransfer {
                entry_identity: 2,
                transfer_identity: 1,
            },
            CompilerSystemsTransition::ReturnToUser {
                entry_identity: 2,
                result: 0,
            },
            CompilerSystemsTransition::EnterUserSyscall {
                entry_identity: 3,
                number: 63,
                arguments: [6_299_392, 0, 0, 0, 0, 0],
            },
            CompilerSystemsTransition::BeginUserTransfer {
                entry_identity: 3,
                transfer_identity: 2,
                candidate: 6_299_392,
                extent: 390,
            },
            CompilerSystemsTransition::CopyToUserFault {
                transfer_identity: 2,
                copied_prefix: 256,
            },
            CompilerSystemsTransition::EndUserTransfer {
                entry_identity: 3,
                transfer_identity: 2,
            },
            CompilerSystemsTransition::ReturnToUser {
                entry_identity: 3,
                result: u64::MAX - 13,
            },
            CompilerSystemsTransition::EnterUserSyscall {
                entry_identity: 4,
                number: 35,
                arguments: [6_299_632, 0, 0, 0, 0, 0],
            },
            CompilerSystemsTransition::BeginUserTransfer {
                entry_identity: 4,
                transfer_identity: 3,
                candidate: 6_299_632,
                extent: 16,
            },
            CompilerSystemsTransition::CopyFromUserComplete {
                transfer_identity: 3,
                copied: 16,
                seconds: 0,
                nanoseconds: 0,
            },
            CompilerSystemsTransition::EndUserTransfer {
                entry_identity: 4,
                transfer_identity: 3,
            },
            CompilerSystemsTransition::ReturnToUser {
                entry_identity: 4,
                result: 0,
            },
            CompilerSystemsTransition::EnterUserSyscall {
                entry_identity: 5,
                number: 35,
                arguments: [6_299_640, 0, 0, 0, 0, 0],
            },
            CompilerSystemsTransition::BeginUserTransfer {
                entry_identity: 5,
                transfer_identity: 4,
                candidate: 6_299_640,
                extent: 16,
            },
            CompilerSystemsTransition::CopyFromUserFault {
                transfer_identity: 4,
                copied_prefix: 8,
            },
            CompilerSystemsTransition::EndUserTransfer {
                entry_identity: 5,
                transfer_identity: 4,
            },
            CompilerSystemsTransition::ReturnToUser {
                entry_identity: 5,
                result: u64::MAX - 13,
            },
            CompilerSystemsTransition::EnterUserSyscall {
                entry_identity: 6,
                number: 231,
                arguments: [0; 6],
            },
            CompilerSystemsTransition::TerminateUser {
                entry_identity: 6,
                status: 0,
            },
            CompilerSystemsTransition::ResumeUserKernelContinuation {
                kernel_continuation_identity: 1,
                status: 0,
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_READ_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_CLOSE_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_DUP_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_FCNTL_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_FSTAT_OK".into(),
            },
            CompilerSystemsOperation::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_TCGETS_OK".into(),
            },
            CompilerSystemsOperation::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_WINSIZE_OK".into(),
            },
            CompilerSystemsOperation::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_LSEEK_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_COPY_TO_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_COPY_FROM_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_SYSCALL_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_IMAGE_ENTRY_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_IMAGE_STACK_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_IMAGE_AUXV_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_IMAGE_ELF_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_COPY_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_IMAGE_RUNTIME_EXTENT_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_IMAGE_RUNTIME_ENTRY_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_IMAGE_RUNTIME_DATA_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_IMAGE_BSS_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_IMAGE_DATA_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_IMAGE_WRITABLE_DATA_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_IMAGE_COPY_TO_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_IMAGE_COPY_FROM_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_IMAGE_SYSCALL_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_IMAGE_RETAINED".into(),
            },
            CompilerSystemsTransition::ReleaseAnonymousUserMapping {
                address_space_identity: 1,
                reservation_identity: 3,
            },
            CompilerSystemsTransition::ReleasePhysicalFrames,
            CompilerSystemsTransition::UnmapUserAddressSpace {
                address_space_identity: 1,
                mapping_identity: 2,
            },
            CompilerSystemsTransition::ReleasePhysicalFrames,
            CompilerSystemsTransition::UnmapUserAddressSpace {
                address_space_identity: 1,
                mapping_identity: 1,
            },
            CompilerSystemsTransition::ReleasePhysicalFrames,
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_MAPPING_OK".into(),
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_FRAME_DISJOINT_OK".into(),
            },
            CompilerSystemsTransition::CompleteUserExecution {
                process_identity: 1,
            },
            CompilerSystemsTransition::ReclaimUserAddressSpace {
                address_space_identity: 1,
            },
            CompilerSystemsTransition::ReleasePhysicalFrames,
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_USER_BACKING_OK".into(),
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 25,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
                    [0],
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 26,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
                    [1],
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 27,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
                    [2],
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 28,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
                    [3],
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_VIRTIO_QUEUE_ACTIVE".into(),
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 29,
                request:
                    CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION[0],
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_VIRTIO_QUEUE_NOTIFIED".into(),
            },
            CompilerSystemsTransition::AccessPciRegionRegister {
                transaction_identity: 30,
                request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL
                    [0],
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_VIRTIO_COMPLETION_SIGNAL".into(),
            },
            CompilerSystemsTransition::CompleteDmaBufferForCpu {
                request: CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[1],
            },
            CompilerSystemsTransition::LoadDmaBufferIntegerEquals {
                request: CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING[0],
            },
            CompilerSystemsTransition::LoadDmaBufferIntegerEquals {
                request: CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING[1],
            },
            CompilerSystemsTransition::LoadDmaBufferIntegerEquals {
                request: CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING[2],
            },
            CompilerSystemsTransition::LoadDmaBufferIntegerEquals {
                request: CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING[3],
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_VIRTIO_USED_RING_OK".into(),
            },
            CompilerSystemsTransition::CompleteDmaBufferForCpu {
                request: CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0],
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_VIRTIO_PAYLOAD_CPU_OWNED".into(),
            },
            CompilerSystemsTransition::CaptureEntropySample {
                request: CompilerEntropySampleCaptureRequest::INITIAL_Q35_VIRTIO_RNG,
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_ENTROPY_SAMPLE_CAPTURED".into(),
            },
            CompilerSystemsTransition::AssessEntropySampleQuality {
                request: CompilerEntropySampleQualityRequest::INITIAL_Q35_VIRTIO_RNG,
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_ENTROPY_SAMPLE_CREDITED".into(),
            },
            CompilerSystemsTransition::CreateEntropyPool {
                request: CompilerEntropyPoolRequest::INITIAL_Q35,
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_ENTROPY_POOL_CREATED".into(),
            },
            CompilerSystemsTransition::MixEntropyPoolSample {
                request: CompilerEntropyPoolMixRequest::INITIAL_Q35,
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_ENTROPY_POOL_MIXED".into(),
            },
            CompilerSystemsTransition::CreditEntropyPoolContribution {
                request: CompilerEntropyPoolCreditRequest::INITIAL_Q35,
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_ENTROPY_POOL_CREDITED".into(),
            },
            CompilerSystemsTransition::InitializeEntropyGenerator {
                request: CompilerEntropyGeneratorRequest::INITIAL_Q35,
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_ENTROPY_GENERATOR_READY".into(),
            },
            CompilerSystemsTransition::ExtractEntropyGenerator {
                request: CompilerEntropyGeneratorExtractRequest::INITIAL_Q35,
            },
            CompilerSystemsTransition::ConsoleWrite {
                text: "TOPAL_KERNEL_ENTROPY_OUTPUT_READY".into(),
            },
            CompilerSystemsTransition::Fatal {
                message: "toolchain gate complete".into(),
            },
        ]
    );
}

#[test]
fn checks_affine_bootstrap_region_access_and_dynamic_source_guard() {
    // TOPAL-SYSTEMS-STORAGE-001, TOPAL-SYSTEMS-QUALIFY-001,
    // TOPAL-SYSTEMS-FRAMES-001, TOPAL-SYSTEMS-STATIC-ELF-MAPPING-RESOURCES-001,
    // TOPAL-SYSTEMS-STATIC-ELF-LEAVES-001.
    let program = analyze_systems_for_compiler(
        &memory_source(),
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    assert_eq!(program.bootstrap.handler.operations.len(), 423);
    assert_eq!(
        program.bootstrap.handler.effects,
        [
            SYSTEMS_ATOMIC_COMPARE_EXCHANGE,
            SYSTEMS_ATOMIC_WORD_CREATE,
            SYSTEMS_ATOMIC_END,
            SYSTEMS_ATOMIC_LOAD,
            SYSTEMS_BOOT_MEMORY_DESCRIBE,
            SYSTEMS_KERNEL_COMPLETION_BLOCK_ONCE,
            SYSTEMS_KERNEL_COMPLETION_CONSUME,
            SYSTEMS_KERNEL_COMPLETION_CREATE,
            SYSTEMS_KERNEL_COMPLETION_SIGNAL,
            SYSTEMS_KERNEL_CONTEXT_CREATE,
            SYSTEMS_KERNEL_CONTEXT_RECLAIM,
            SYSTEMS_KERNEL_RUNNABLE_QUEUE_CONSUME,
            SYSTEMS_KERNEL_RUNNABLE_QUEUE_CREATE,
            SYSTEMS_KERNEL_RUNNABLE_QUEUE_DEQUEUE,
            SYSTEMS_KERNEL_RUNNABLE_QUEUE_ENQUEUE,
            SYSTEMS_KERNEL_CONTEXT_TAKE_PREEMPTED,
            SYSTEMS_KERNEL_CONTEXT_TRANSFER_UNTIL_DEADLINE,
            SYSTEMS_KERNEL_CONTEXT_TRANSFER,
            SYSTEMS_USER_CONTEXT_COMPLETE,
            SYSTEMS_USER_CONTEXT_ENTER,
            SYSTEMS_USER_CONTEXT_PREPARE,
            SYSTEMS_CRITICAL_ENTER,
            SYSTEMS_CRITICAL_RESTORE,
            SYSTEMS_CONSOLE_WRITE,
            SYSTEMS_PCI_CONFIGURATION_READ_DWORD,
            SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD,
            SYSTEMS_PCI_REGION_READ_EVENTUALLY,
            SYSTEMS_PCI_REGION_READ,
            SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE,
            SYSTEMS_PCI_REGION_WRITE,
            SYSTEMS_FATAL,
            SYSTEMS_DMA_BUFFER_BIND_REFERENCE,
            SYSTEMS_DMA_BUFFER_COMPLETE_CPU,
            SYSTEMS_DMA_BUFFER_CREATE,
            SYSTEMS_DMA_BUFFER_LOAD_INTEGER,
            SYSTEMS_DMA_BUFFER_PREPARE_DEVICE,
            SYSTEMS_DMA_BUFFER_STORE_INTEGER,
            SYSTEMS_ENTROPY_GENERATOR_EXTRACT,
            SYSTEMS_ENTROPY_GENERATOR_INITIALIZE,
            SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM,
            SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM,
            SYSTEMS_ENTROPY_POOL_CREATE,
            SYSTEMS_ENTROPY_POOL_CREDIT_CONTRIBUTION,
            SYSTEMS_ENTROPY_POOL_MIX_SAMPLE,
            SYSTEMS_ENTROPY_SAMPLE_ASSESS_QUALITY,
            SYSTEMS_ENTROPY_SAMPLE_CAPTURE_DMA,
            SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE,
            SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS,
            SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS,
            SYSTEMS_FRAMES_ALLOCATE,
            SYSTEMS_FRAMES_RELEASE,
            SYSTEMS_LOCAL_NOTIFICATION_SEND,
            SYSTEMS_LOCAL_NOTIFICATION_WAIT,
            SYSTEMS_DEBUG_BREAK,
            SYSTEMS_KERNEL_MAPPING_LOAD_BYTE,
            SYSTEMS_KERNEL_MAP,
            SYSTEMS_KERNEL_MAPPING_STORE_BYTE,
            SYSTEMS_KERNEL_UNMAP,
            SYSTEMS_FRAME_ALLOCATOR_CREATE,
            SYSTEMS_USER_ADDRESS_SPACE_MAP,
            SYSTEMS_USER_ADDRESS_SPACE_PREPARE,
            SYSTEMS_USER_ADDRESS_SPACE_RECLAIM,
            SYSTEMS_USER_ADDRESS_SPACE_RELEASE_ANONYMOUS,
            SYSTEMS_USER_ADDRESS_SPACE_RESERVE_ANONYMOUS,
            SYSTEMS_USER_ADDRESS_SPACE_UNMAP,
            SYSTEMS_USER_MAPPING_INSTALL_ENTRY_TRAMPOLINE,
            SYSTEMS_USER_MAPPING_LOAD_BYTE,
            SYSTEMS_USER_MAPPING_PERMIT_EXECUTE,
            SYSTEMS_USER_MAPPING_PERMIT_WRITABLE_DATA,
            SYSTEMS_USER_MAPPING_PROTECT_READ_ONLY,
            SYSTEMS_USER_MAPPING_STORE_BYTE,
            SYSTEMS_USER_MAPPING_VERIFY_ENTRY_TRAMPOLINE,
            SYSTEMS_REGION_LIBRARY_APPLY,
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
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE_ACTIVATION,
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE_ADDRESSES,
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_QUALIFY_ENTROPY_QUEUE,
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
            SYSTEMS_STATIC_IMAGE_CONSTRUCTION_ZERO_STACK_BACKING,
            SYSTEMS_STATIC_IMAGE_PROCESS_ENTER,
            SYSTEMS_STATIC_IMAGE_PROCESS_RESTORE_ENTRY,
            SYSTEMS_BOOTSTRAP_REGION_LOAD_BYTE,
            SYSTEMS_BOOTSTRAP_REGION_STORE_BYTE,
            SYSTEMS_BOOTSTRAP_STORAGE_ALLOCATE,
            SYSTEMS_BOOTSTRAP_STORAGE_RELEASE,
            SYSTEMS_DEADLINE_AFTER,
            SYSTEMS_MONOTONIC_CLOCK_NOW,
            SYSTEMS_TRANSLATION_ACTIVATE,
            SYSTEMS_TRANSLATION_BEGIN,
            SYSTEMS_TRANSLATION_COMMIT,
            SYSTEMS_TRANSLATION_EDIT_BEGIN,
            SYSTEMS_TRANSLATION_EDIT_COMMIT,
            SYSTEMS_TRANSLATION_EDIT_MAP,
            SYSTEMS_TRANSLATION_EDIT_UNMAP,
        ]
    );
    let transitions = model_systems_transitions(&program).unwrap();
    assert!(
        transitions.contains(&CompilerSystemsTransition::StoreBootstrapByte {
            offset_bytes: 0,
            value: 90,
        })
    );
    assert!(
        transitions.contains(&CompilerSystemsTransition::CheckDynamicBootstrapByte {
            offset_bytes: 1,
            expected: 90,
        })
    );
    assert!(transitions.contains(&CompilerSystemsTransition::ReleaseBootstrapRegion));
    assert!(
        transitions.contains(&CompilerSystemsTransition::MapKernelFrames {
            request: CompilerKernelMappingRequest::initial_read_write(),
        })
    );
    assert!(
        transitions.contains(&CompilerSystemsTransition::StoreKernelMappingByte {
            offset_bytes: 0,
            value: 165,
        })
    );
    assert!(
        transitions.contains(&CompilerSystemsTransition::LoadKernelMappingByte {
            offset_bytes: 0,
            value: 165,
        })
    );
    assert!(transitions.contains(&CompilerSystemsTransition::UnmapKernelFrames));
    assert!(
        transitions.contains(&CompilerSystemsTransition::ConsoleWrite {
            text: "TOPAL_KERNEL_MEMORY_OK".into(),
        })
    );
}

#[test]
fn requires_the_sealed_bounded_pci_configuration_observation() {
    // TOPAL-SYSTEMS-PCI-CONFIG-001, TOPAL-SYSTEMS-VIRTIO-PCI-DISCOVERY-001.
    for source in [
            memory_source().replace("expected-value is 272898804", "expected-value is 0"),
            memory_source().replace(
                "register-offset-bytes is 0,\n                                                                    expected-value",
                "register-offset-bytes is 2,\n                                                                    expected-value",
            ),
            memory_source().replacen("device is 5", "device is 32", 1),
        ] {
            let error = analyze_systems_for_compiler(
                &source,
                &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
            )
            .unwrap_err();
            assert_eq!(error.code, "E-SYSTEMS-PCI-CONFIG");
        }
}

#[test]
fn requires_the_complete_ordered_virtio_pci_capability_discovery() {
    // TOPAL-SYSTEMS-VIRTIO-PCI-DISCOVERY-001.
    let program = analyze_systems_for_compiler(
        &memory_source(),
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let pci_indices = program
        .bootstrap
        .handler
        .operations
        .iter()
        .enumerate()
        .filter_map(|(index, operation)| {
            matches!(
                operation,
                CompilerSystemsOperation::ObservePciConfigurationDword { .. }
            )
            .then_some(index)
        })
        .collect::<Vec<_>>();
    assert_eq!(pci_indices.len(), 24);

    let mut missing = program.clone();
    missing.bootstrap.handler.operations.remove(pci_indices[1]);
    assert_eq!(
        validate_systems_program(&missing).unwrap_err().code,
        "E-SYSTEMS-PCI-CONFIG"
    );

    let mut duplicated = program.clone();
    duplicated.bootstrap.handler.operations[pci_indices[1]] =
        duplicated.bootstrap.handler.operations[pci_indices[0]].clone();
    assert_eq!(
        validate_systems_program(&duplicated).unwrap_err().code,
        "E-SYSTEMS-PCI-CONFIG"
    );

    let mut reordered = program;
    reordered
        .bootstrap
        .handler
        .operations
        .swap(pci_indices[1], pci_indices[2]);
    assert_eq!(
        validate_systems_program(&reordered).unwrap_err().code,
        "E-SYSTEMS-PCI-CONFIG"
    );
}
