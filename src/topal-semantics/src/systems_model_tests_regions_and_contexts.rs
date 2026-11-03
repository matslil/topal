use super::*;
use crate::BootstrapStoragePlacement;

#[test]
fn opaque_region_effects_remove_byte_predictions_until_explicit_overwrite() {
    // TOPAL-SYSTEMS-REGION-ROOT-CALL-001.
    let mut storage = BootstrapStorageState::new(
        BootstrapStorageDescriptor {
            capacity_bytes: 128,
            alignment_bytes: 8,
        },
        "opaque-root",
    )
    .unwrap();
    let request = BootstrapStorageRequest {
        byte_count: 64,
        alignment_bytes: 8,
        placement: BootstrapStoragePlacement::BootstrapReclaimable,
    };
    let owner = storage.allocate(request).unwrap();
    storage.store_byte(&owner, 0, 90).unwrap();
    assert_eq!(storage.known_byte(&owner, 0).unwrap(), Some(90));
    assert_eq!(storage.opaque_library_effect(&owner).unwrap(), 1);
    assert_eq!(storage.known_byte(&owner, 0).unwrap(), None);
    assert_eq!(storage.known_byte(&owner, 1).unwrap(), None);
    let before = format!("{storage:?}");
    assert_eq!(
        storage.load_byte(&owner, 0).unwrap_err().code,
        "E-SYSTEMS-STORAGE-UNKNOWN"
    );
    assert_eq!(format!("{storage:?}"), before);
    assert!(storage.known_byte(&owner, 64).is_err());
    storage.store_byte(&owner, 0, 17).unwrap();
    assert_eq!(storage.known_byte(&owner, 0).unwrap(), Some(17));
    assert_eq!(storage.known_byte(&owner, 1).unwrap(), None);
    storage.atomic_store_word(&owner, 8, 41).unwrap();
    assert_eq!(storage.atomic_load_word(&owner, 8).unwrap(), 41);
    assert!(storage.atomic_load_word(&owner, 0).is_err());
    storage.release(owner).unwrap();
    let fresh = storage.allocate(request).unwrap();
    assert_eq!(storage.known_byte(&fresh, 0).unwrap(), Some(0));
    assert_eq!(storage.opaque_library_effect(&fresh).unwrap(), 2);
    storage.release(fresh).unwrap();
    storage.complete_bootstrap().unwrap();
}

#[test]
fn root_region_call_records_dynamic_guard_and_cleanup_obligations() {
    // TOPAL-SYSTEMS-REGION-ROOT-BINDING-001, -CALL-001, -RECLAIM-001.
    let mut value = program(SystemsDisposition::Resume);
    value.region_library = Some(crate::SystemsSourceLibraryRequirement {
        source: "policy/region.t".into(),
        export: "apply".into(),
        interface: crate::SystemsSourceLibraryInterface::BorrowedByteRegionV1,
        stack_limit_bytes: 4096,
        runtime_limit_bytes: Some(65536),
        state_limit_bytes: None,
    });
    value.bootstrap.handler.operations.extend([
        SystemsOperation::BootstrapAllocate {
            request: BootstrapStorageRequest {
                byte_count: 64,
                alignment_bytes: 8,
                placement: BootstrapStoragePlacement::BootstrapReclaimable,
            },
        },
        SystemsOperation::BootstrapStoreByte {
            offset_bytes: 0,
            value: 90,
        },
        SystemsOperation::ApplyBootstrapRegionLibrary,
        SystemsOperation::BootstrapLoadByteEquals {
            offset_bytes: 1,
            expected: 90,
            failure_message: "source result mismatch".into(),
        },
        SystemsOperation::BootstrapRelease,
    ]);
    refresh_bootstrap_effects(&mut value);
    let trace = model_systems_transitions(&value).unwrap();
    let position =
        |predicate: fn(&SystemsTransition) -> bool| trace.iter().position(predicate).unwrap();
    let borrow = position(|step| {
        matches!(
            step,
            SystemsTransition::BorrowBootstrapRegionForLibrary { .. }
        )
    });
    let cleanup =
        position(|step| matches!(step, SystemsTransition::RequireRegionLibraryReclamation));
    let resume = position(|step| {
        matches!(
            step,
            SystemsTransition::ResumeBootstrapRegionAfterLibrary {
                require_true: false
            }
        )
    });
    let guard = position(|step| {
        matches!(
            step,
            SystemsTransition::CheckDynamicBootstrapByte {
                offset_bytes: 1,
                expected: 90
            }
        )
    });
    assert!(borrow < cleanup && cleanup < resume && resume < guard);
    let mut instance = value.clone();
    instance.region_library.as_mut().unwrap().interface =
        crate::SystemsSourceLibraryInterface::BorrowedByteRegionInstanceActionV1;
    let instance_trace = model_systems_transitions(&instance).unwrap();
    assert_eq!(instance_trace.len(), trace.len());
    assert!(matches!(
        instance_trace[resume],
        SystemsTransition::ResumeBootstrapRegionAfterLibrary { require_true: true }
    ));
    assert_eq!(instance_trace[guard], trace[guard]); // actual bytes remain unknown
    assert!(!trace.iter().any(|step| matches!(
        step,
        SystemsTransition::LoadBootstrapByte {
            offset_bytes: 1,
            ..
        }
    )));
    assert!(!trace.iter().any(|step| matches!(step, SystemsTransition::Fatal { message } if message == "source result mismatch")));
    let mut missing = value.clone();
    missing.region_library = None;
    assert!(validate_systems_program(&missing).is_err());
    let mut duplicate = value.clone();
    duplicate
        .bootstrap
        .handler
        .operations
        .push(SystemsOperation::ApplyBootstrapRegionLibrary);
    refresh_bootstrap_effects(&mut duplicate);
    assert!(validate_systems_program(&duplicate).is_err());
    let mut late = value.clone();
    let at = late
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|step| matches!(step, SystemsOperation::ApplyBootstrapRegionLibrary))
        .unwrap();
    late.bootstrap.handler.operations.splice(
        at..at,
        [
            SystemsOperation::AtomicWordCreate {
                request: crate::AtomicWordRequest::initial(),
            },
            SystemsOperation::AtomicWordEnd,
        ],
    );
    refresh_bootstrap_effects(&mut late);
    assert_eq!(
        validate_systems_program(&late).unwrap_err().code,
        "E-SYSTEMS-REGION-ROOT"
    );
    let index = value
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|step| {
            matches!(
                step,
                SystemsOperation::BootstrapLoadByteEquals {
                    offset_bytes: 1,
                    ..
                }
            )
        })
        .unwrap();
    value.bootstrap.handler.operations.insert(
        index,
        SystemsOperation::BootstrapStoreByte {
            offset_bytes: 1,
            value: 17,
        },
    );
    refresh_bootstrap_effects(&mut value);
    let trace = model_systems_transitions(&value).unwrap();
    assert!(trace.iter().any(|step| matches!(step, SystemsTransition::Fatal { message } if message == "source result mismatch")));
    assert!(!trace.iter().any(|step| matches!(
        step,
        SystemsTransition::CheckDynamicBootstrapByte {
            offset_bytes: 1,
            ..
        }
    )));
}

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
        kernel_library: None,
        region_library: None,
        file_position_library: None,
        time_library: None,
        image_library: None,
        dispatch_library: None,
        process_identity_library: None,
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
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[0],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[1],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[2],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[3],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[4],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[5],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[6],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[7],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[8],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[9],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[10],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[11],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[12],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[13],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[14],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[15],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[16],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[17],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[18],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[19],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[20],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[21],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[22],
                    },
                    SystemsOperation::ObservePciConfigurationDword {
                        request: PciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY[23],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[0],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[1],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[2],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[3],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[4],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[5],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[6],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[7],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[8],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[9],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[10],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[11],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[12],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[13],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[14],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[15],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY
                            [0],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY
                            [1],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY
                            [2],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY
                            [3],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY
                            [4],
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
                        request: DmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING[0],
                    },
                    SystemsOperation::StoreDmaBufferInteger {
                        request: DmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING[1],
                    },
                    SystemsOperation::StoreDmaBufferInteger {
                        request: DmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING[2],
                    },
                    SystemsOperation::StoreDmaBufferInteger {
                        request: DmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING[3],
                    },
                    SystemsOperation::PrepareDmaBufferForDevice {
                        request: DmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0],
                    },
                    SystemsOperation::PrepareDmaBufferForDevice {
                        request: DmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[1],
                    },
                    SystemsOperation::PublishPciRegionDmaReference {
                        request:
                            PciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES[0],
                    },
                    SystemsOperation::PublishPciRegionDmaReference {
                        request:
                            PciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES[1],
                    },
                    SystemsOperation::PublishPciRegionDmaReference {
                        request:
                            PciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES[2],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
                            [0],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
                            [1],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
                            [2],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
                            [3],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request:
                            PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION[0],
                    },
                    SystemsOperation::AccessPciRegionRegister {
                        request: PciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL
                            [0],
                    },
                    SystemsOperation::CompleteDmaBufferForCpu {
                        request: DmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[1],
                    },
                    SystemsOperation::LoadDmaBufferIntegerEquals {
                        request: DmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING[0],
                    },
                    SystemsOperation::LoadDmaBufferIntegerEquals {
                        request: DmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING[1],
                    },
                    SystemsOperation::LoadDmaBufferIntegerEquals {
                        request: DmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING[2],
                    },
                    SystemsOperation::LoadDmaBufferIntegerEquals {
                        request: DmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING[3],
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
                request: PciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES[0],
            },
            SystemsTransition::PublishPciRegionDmaReference {
                transaction_identity: 23,
                request: PciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES[1],
            },
            SystemsTransition::PublishPciRegionDmaReference {
                transaction_identity: 24,
                request: PciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES[2],
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
