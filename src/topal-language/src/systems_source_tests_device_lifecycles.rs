#[test]
fn requires_the_complete_ordered_virtio_pci_transport_observation() {
    // TOPAL-SYSTEMS-PCI-REGION-001, TOPAL-SYSTEMS-VIRTIO-PCI-NEGOTIATION-001,
    // TOPAL-SYSTEMS-VIRTIO-PCI-QUEUE-DISCOVERY-001.
    let mut malformed = CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[0];
    malformed.register_offset_bytes = 1;
    malformed.width_bytes = 4;
    assert!(!malformed.is_well_formed());
    malformed = CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION[0];
    malformed.region_length_bytes = 20;
    assert!(!malformed.is_well_formed());
    malformed = CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY[0];
    malformed.value = 65_536;
    assert!(!malformed.is_well_formed());

    let program = analyze_systems_for_compiler(
        &memory_source(),
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let indices = program
        .bootstrap
        .handler
        .operations
        .iter()
        .enumerate()
        .filter_map(|(index, operation)| {
            matches!(
                operation,
                CompilerSystemsOperation::AccessPciRegionRegister { .. }
            )
            .then_some(index)
        })
        .collect::<Vec<_>>();
    assert_eq!(indices.len(), 27);

    let mut missing = program.clone();
    missing.bootstrap.handler.operations.remove(indices[17]);
    assert_eq!(
        validate_systems_program(&missing).unwrap_err().code,
        "E-SYSTEMS-PCI-REGION"
    );

    let mut duplicated = program.clone();
    duplicated.bootstrap.handler.operations[indices[17]] =
        duplicated.bootstrap.handler.operations[indices[16]].clone();
    assert_eq!(
        validate_systems_program(&duplicated).unwrap_err().code,
        "E-SYSTEMS-PCI-REGION"
    );

    let mut reordered = program;
    reordered
        .bootstrap
        .handler
        .operations
        .swap(indices[17], indices[18]);
    assert_eq!(
        validate_systems_program(&reordered).unwrap_err().code,
        "E-SYSTEMS-PCI-REGION"
    );
}

#[test]
fn requires_the_exact_pci_bus_master_authorization() {
    // TOPAL-SYSTEMS-PCI-CONFIG-UPDATE-001.
    let mut malformed =
        CompilerPciConfigurationWordUpdateRequest::INITIAL_Q35_VIRTIO_RNG_BUS_MASTER;
    malformed.mask = 0;
    assert!(!malformed.is_well_formed());
    malformed = CompilerPciConfigurationWordUpdateRequest::INITIAL_Q35_VIRTIO_RNG_BUS_MASTER;
    malformed.expected_masked_value = 1;
    assert!(!malformed.is_well_formed());

    let program = analyze_systems_for_compiler(
        &memory_source(),
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let index = program
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::UpdatePciConfigurationWord { .. }
            )
        })
        .unwrap();

    let mut missing = program.clone();
    missing.bootstrap.handler.operations.remove(index);
    missing.bootstrap.handler.operations.retain(|operation| {
        !is_queue_finalization(operation)
            && !matches!(
                operation,
                CompilerSystemsOperation::CreateDmaBuffer { .. }
                    | CompilerSystemsOperation::BindDmaBufferReference { .. }
                    | CompilerSystemsOperation::StoreDmaBufferInteger { .. }
                    | CompilerSystemsOperation::PrepareDmaBufferForDevice { .. }
                    | CompilerSystemsOperation::PublishPciRegionDmaReference { .. }
            )
    });
    missing.bootstrap.handler.effects.retain(|effect| {
        *effect != SYSTEMS_PCI_CONFIGURATION_UPDATE_WORD
            && *effect != SYSTEMS_DMA_BUFFER_CREATE
            && *effect != SYSTEMS_DMA_BUFFER_BIND_REFERENCE
            && *effect != SYSTEMS_DMA_BUFFER_STORE_INTEGER
            && *effect != SYSTEMS_DMA_BUFFER_PREPARE_DEVICE
            && *effect != SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE
            && *effect != SYSTEMS_PCI_REGION_READ_EVENTUALLY
            && *effect != SYSTEMS_DMA_BUFFER_COMPLETE_CPU
            && *effect != SYSTEMS_DMA_BUFFER_LOAD_INTEGER
    });
    remove_entropy_sample_capture(&mut missing);
    assert_eq!(
        validate_systems_program(&missing).unwrap_err().code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut duplicated = program.clone();
    duplicated.bootstrap.handler.operations.insert(
        index,
        duplicated.bootstrap.handler.operations[index].clone(),
    );
    assert_eq!(
        validate_systems_program(&duplicated).unwrap_err().code,
        "E-SYSTEMS-PCI-CONFIG-UPDATE"
    );

    let mut substituted = program;
    if let CompilerSystemsOperation::UpdatePciConfigurationWord { request } =
        &mut substituted.bootstrap.handler.operations[index]
    {
        request.desired_masked_value = 0;
    } else {
        unreachable!();
    }
    assert_eq!(
        validate_systems_program(&substituted).unwrap_err().code,
        "E-SYSTEMS-PCI-CONFIG-UPDATE"
    );
}

#[test]
fn requires_the_exact_cpu_owned_dma_buffers() {
    // TOPAL-SYSTEMS-DMA-BUFFER-001.
    let mut malformed = CompilerDmaBufferRequest::INITIAL_Q35_VIRTIO_RNG[0];
    malformed.alignment_bytes = 0;
    assert!(!malformed.is_well_formed());

    let program = analyze_systems_for_compiler(
        &memory_source(),
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let indices = program
        .bootstrap
        .handler
        .operations
        .iter()
        .enumerate()
        .filter_map(|(index, operation)| {
            matches!(operation, CompilerSystemsOperation::CreateDmaBuffer { .. }).then_some(index)
        })
        .collect::<Vec<_>>();
    assert_eq!(indices.len(), 2);

    let mut missing = program.clone();
    missing.bootstrap.handler.operations.remove(indices[1]);
    missing.bootstrap.handler.operations.retain(|operation| {
        !is_queue_finalization(operation)
            && !matches!(
                operation,
                CompilerSystemsOperation::BindDmaBufferReference { .. }
                    | CompilerSystemsOperation::StoreDmaBufferInteger { .. }
                    | CompilerSystemsOperation::PrepareDmaBufferForDevice { .. }
                    | CompilerSystemsOperation::PublishPciRegionDmaReference { .. }
            )
    });
    missing.bootstrap.handler.effects.retain(|effect| {
        *effect != SYSTEMS_DMA_BUFFER_BIND_REFERENCE
            && *effect != SYSTEMS_DMA_BUFFER_STORE_INTEGER
            && *effect != SYSTEMS_DMA_BUFFER_PREPARE_DEVICE
            && *effect != SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE
            && *effect != SYSTEMS_PCI_REGION_READ_EVENTUALLY
            && *effect != SYSTEMS_DMA_BUFFER_COMPLETE_CPU
            && *effect != SYSTEMS_DMA_BUFFER_LOAD_INTEGER
    });
    remove_entropy_sample_capture(&mut missing);
    assert_eq!(
        validate_systems_program(&missing).unwrap_err().code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut duplicated = program.clone();
    duplicated.bootstrap.handler.operations.insert(
        indices[1],
        duplicated.bootstrap.handler.operations[indices[0]].clone(),
    );
    assert_eq!(
        validate_systems_program(&duplicated).unwrap_err().code,
        "E-SYSTEMS-DMA-BUFFER"
    );

    let mut reordered = program;
    reordered
        .bootstrap
        .handler
        .operations
        .swap(indices[0], indices[1]);
    assert_eq!(
        validate_systems_program(&reordered).unwrap_err().code,
        "E-SYSTEMS-DMA-BUFFER"
    );
}

#[test]
fn requires_the_exact_cpu_owned_split_ring_construction() {
    // TOPAL-SYSTEMS-DMA-CONSTRUCT-001.
    let mut malformed = CompilerDmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING[0];
    malformed.width_bytes = 3;
    assert!(!malformed.is_well_formed());

    let program = analyze_systems_for_compiler(
        &memory_source(),
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let bind_index = program
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::BindDmaBufferReference { .. }
            )
        })
        .unwrap();
    let store_indices = program
        .bootstrap
        .handler
        .operations
        .iter()
        .enumerate()
        .filter_map(|(index, operation)| {
            matches!(
                operation,
                CompilerSystemsOperation::StoreDmaBufferInteger { .. }
            )
            .then_some(index)
        })
        .collect::<Vec<_>>();
    assert_eq!(store_indices.len(), 4);

    let mut missing = program.clone();
    missing
        .bootstrap
        .handler
        .operations
        .remove(store_indices[3]);
    missing.bootstrap.handler.operations.retain(|operation| {
        !is_queue_finalization(operation)
            && !matches!(
                operation,
                CompilerSystemsOperation::PrepareDmaBufferForDevice { .. }
                    | CompilerSystemsOperation::PublishPciRegionDmaReference { .. }
            )
    });
    missing.bootstrap.handler.effects.retain(|effect| {
        *effect != SYSTEMS_DMA_BUFFER_PREPARE_DEVICE
            && *effect != SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE
            && *effect != SYSTEMS_PCI_REGION_READ_EVENTUALLY
            && *effect != SYSTEMS_DMA_BUFFER_COMPLETE_CPU
            && *effect != SYSTEMS_DMA_BUFFER_LOAD_INTEGER
    });
    remove_entropy_sample_capture(&mut missing);
    assert_eq!(
        validate_systems_program(&missing).unwrap_err().code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut duplicated = program.clone();
    duplicated.bootstrap.handler.operations.insert(
        store_indices[1],
        duplicated.bootstrap.handler.operations[store_indices[0]].clone(),
    );
    assert_eq!(
        validate_systems_program(&duplicated).unwrap_err().code,
        "E-SYSTEMS-DMA-CONSTRUCT"
    );

    let mut reordered = program;
    reordered
        .bootstrap
        .handler
        .operations
        .swap(bind_index, store_indices[0]);
    assert_eq!(
        validate_systems_program(&reordered).unwrap_err().code,
        "E-SYSTEMS-DMA-CONSTRUCT"
    );
}

#[test]
fn requires_the_exact_dma_device_preparation_sequence() {
    // TOPAL-SYSTEMS-DMA-PREPARE-001.
    let mut malformed = CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0];
    malformed.buffer_identity = 0;
    assert!(!malformed.is_well_formed());

    let program = analyze_systems_for_compiler(
        &memory_source(),
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let indices = program
        .bootstrap
        .handler
        .operations
        .iter()
        .enumerate()
        .filter_map(|(index, operation)| {
            matches!(
                operation,
                CompilerSystemsOperation::PrepareDmaBufferForDevice { .. }
            )
            .then_some(index)
        })
        .collect::<Vec<_>>();
    assert_eq!(indices.len(), 2);

    let mut missing = program.clone();
    missing.bootstrap.handler.operations.remove(indices[1]);
    missing.bootstrap.handler.operations.retain(|operation| {
        !is_queue_finalization(operation)
            && !matches!(
                operation,
                CompilerSystemsOperation::PublishPciRegionDmaReference { .. }
            )
    });
    missing.bootstrap.handler.effects.retain(|effect| {
        *effect != SYSTEMS_PCI_REGION_WRITE_DMA_REFERENCE
            && *effect != SYSTEMS_PCI_REGION_READ_EVENTUALLY
            && *effect != SYSTEMS_DMA_BUFFER_COMPLETE_CPU
            && *effect != SYSTEMS_DMA_BUFFER_LOAD_INTEGER
    });
    remove_entropy_sample_capture(&mut missing);
    assert_eq!(
        validate_systems_program(&missing).unwrap_err().code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut duplicated = program.clone();
    duplicated.bootstrap.handler.operations.insert(
        indices[1],
        duplicated.bootstrap.handler.operations[indices[0]].clone(),
    );
    assert_eq!(
        validate_systems_program(&duplicated).unwrap_err().code,
        "E-SYSTEMS-DMA-PREPARE"
    );

    let mut reordered = program;
    reordered
        .bootstrap
        .handler
        .operations
        .swap(indices[0], indices[1]);
    assert_eq!(
        validate_systems_program(&reordered).unwrap_err().code,
        "E-SYSTEMS-DMA-PREPARE"
    );
}

#[test]
fn requires_the_exact_virtio_queue_address_publication_sequence() {
    // TOPAL-SYSTEMS-DMA-PUBLISH-001.
    let mut malformed =
        CompilerPciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES[0];
    malformed.width_bytes = 4;
    assert!(!malformed.is_well_formed());

    let program = analyze_systems_for_compiler(
        &memory_source(),
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let indices = program
        .bootstrap
        .handler
        .operations
        .iter()
        .enumerate()
        .filter_map(|(index, operation)| {
            matches!(
                operation,
                CompilerSystemsOperation::PublishPciRegionDmaReference { .. }
            )
            .then_some(index)
        })
        .collect::<Vec<_>>();
    assert_eq!(indices.len(), 3);

    let mut missing = program.clone();
    missing.bootstrap.handler.operations.remove(indices[2]);
    missing
        .bootstrap
        .handler
        .operations
        .retain(|operation| !is_queue_finalization(operation));
    missing.bootstrap.handler.effects.retain(|effect| {
        *effect != SYSTEMS_PCI_REGION_READ_EVENTUALLY
            && *effect != SYSTEMS_DMA_BUFFER_COMPLETE_CPU
            && *effect != SYSTEMS_DMA_BUFFER_LOAD_INTEGER
    });
    remove_entropy_sample_capture(&mut missing);
    assert_eq!(
        validate_systems_program(&missing).unwrap_err().code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut duplicated = program.clone();
    duplicated.bootstrap.handler.operations.insert(
        indices[1],
        duplicated.bootstrap.handler.operations[indices[0]].clone(),
    );
    assert_eq!(
        validate_systems_program(&duplicated).unwrap_err().code,
        "E-SYSTEMS-DMA-PUBLISH"
    );

    let mut reordered = program;
    reordered
        .bootstrap
        .handler
        .operations
        .swap(indices[0], indices[1]);
    assert_eq!(
        validate_systems_program(&reordered).unwrap_err().code,
        "E-SYSTEMS-DMA-PUBLISH"
    );
}

#[test]
fn requires_the_exact_virtio_queue_activation_sequence() {
    // TOPAL-SYSTEMS-VIRTIO-PCI-QUEUE-ACTIVATE-001.
    let program = analyze_systems_for_compiler(
        &memory_source(),
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let indices = program
        .bootstrap
        .handler
        .operations
        .iter()
        .enumerate()
        .filter_map(|(index, operation)| {
            matches!(
                operation,
                CompilerSystemsOperation::AccessPciRegionRegister { request }
                    if CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
                        .contains(request)
            )
            .then_some(index)
        })
        .collect::<Vec<_>>();
    assert_eq!(indices.len(), 4);

    let mut missing = program.clone();
    missing.bootstrap.handler.operations.remove(indices[3]);
    missing.bootstrap.handler.operations.retain(|operation| {
        !matches!(
            operation,
            CompilerSystemsOperation::AccessPciRegionRegister { request }
                if CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION
                    .contains(request)
                    || CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL
                        .contains(request)
        ) && !matches!(
            operation,
            CompilerSystemsOperation::CompleteDmaBufferForCpu { .. }
                | CompilerSystemsOperation::LoadDmaBufferIntegerEquals { .. }
        )
    });
    missing.bootstrap.handler.effects.retain(|effect| {
        *effect != SYSTEMS_PCI_REGION_READ_EVENTUALLY
            && *effect != SYSTEMS_DMA_BUFFER_COMPLETE_CPU
            && *effect != SYSTEMS_DMA_BUFFER_LOAD_INTEGER
    });
    remove_entropy_sample_capture(&mut missing);
    assert_eq!(
        validate_systems_program(&missing).unwrap_err().code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut duplicated = program.clone();
    duplicated.bootstrap.handler.operations.insert(
        indices[1],
        duplicated.bootstrap.handler.operations[indices[0]].clone(),
    );
    assert_eq!(
        validate_systems_program(&duplicated).unwrap_err().code,
        "E-SYSTEMS-PCI-REGION"
    );

    let mut reordered = program;
    reordered
        .bootstrap
        .handler
        .operations
        .swap(indices[0], indices[1]);
    assert_eq!(
        validate_systems_program(&reordered).unwrap_err().code,
        "E-SYSTEMS-PCI-REGION"
    );
}

#[test]
fn requires_the_exact_virtio_queue_notification() {
    // TOPAL-SYSTEMS-VIRTIO-PCI-QUEUE-NOTIFY-001.
    let program = analyze_systems_for_compiler(
        &memory_source(),
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let index = program
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::AccessPciRegionRegister { request }
                    if CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION
                        .contains(request)
            )
        })
        .unwrap();

    let mut missing = program.clone();
    missing.bootstrap.handler.operations.remove(index);
    missing.bootstrap.handler.operations.retain(|operation| {
        !matches!(
            operation,
            CompilerSystemsOperation::AccessPciRegionRegister { request }
                if CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL
                    .contains(request)
        ) && !matches!(
            operation,
            CompilerSystemsOperation::CompleteDmaBufferForCpu { .. }
                | CompilerSystemsOperation::LoadDmaBufferIntegerEquals { .. }
        )
    });
    missing.bootstrap.handler.effects.retain(|effect| {
        *effect != SYSTEMS_PCI_REGION_READ_EVENTUALLY
            && *effect != SYSTEMS_DMA_BUFFER_COMPLETE_CPU
            && *effect != SYSTEMS_DMA_BUFFER_LOAD_INTEGER
    });
    remove_entropy_sample_capture(&mut missing);
    assert_eq!(
        validate_systems_program(&missing).unwrap_err().code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut duplicated = program.clone();
    duplicated.bootstrap.handler.operations.insert(
        index,
        duplicated.bootstrap.handler.operations[index].clone(),
    );
    assert_eq!(
        validate_systems_program(&duplicated).unwrap_err().code,
        "E-SYSTEMS-PCI-REGION"
    );

    let mut substituted = program;
    if let CompilerSystemsOperation::AccessPciRegionRegister { request } =
        &mut substituted.bootstrap.handler.operations[index]
    {
        request.value = 1;
    } else {
        unreachable!();
    }
    assert_eq!(
        validate_systems_program(&substituted).unwrap_err().code,
        "E-SYSTEMS-PCI-REGION"
    );
}

#[test]
fn requires_the_exact_virtio_completion_signal() {
    // TOPAL-SYSTEMS-VIRTIO-PCI-COMPLETION-SIGNAL-001.
    let program = analyze_systems_for_compiler(
        &memory_source(),
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let index = program
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::AccessPciRegionRegister { request }
                    if CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL
                        .contains(request)
            )
        })
        .unwrap();

    let mut missing = program.clone();
    missing.bootstrap.handler.operations.remove(index);
    missing.bootstrap.handler.operations.retain(|operation| {
        !matches!(
            operation,
            CompilerSystemsOperation::CompleteDmaBufferForCpu { .. }
                | CompilerSystemsOperation::LoadDmaBufferIntegerEquals { .. }
        )
    });
    missing.bootstrap.handler.effects.retain(|effect| {
        *effect != SYSTEMS_PCI_REGION_READ_EVENTUALLY
            && *effect != SYSTEMS_DMA_BUFFER_COMPLETE_CPU
            && *effect != SYSTEMS_DMA_BUFFER_LOAD_INTEGER
    });
    remove_entropy_sample_capture(&mut missing);
    assert_eq!(
        validate_systems_program(&missing).unwrap_err().code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut duplicated = program.clone();
    duplicated.bootstrap.handler.operations.insert(
        index,
        duplicated.bootstrap.handler.operations[index].clone(),
    );
    assert_eq!(
        validate_systems_program(&duplicated).unwrap_err().code,
        "E-SYSTEMS-PCI-REGION"
    );

    let mut substituted = program;
    if let CompilerSystemsOperation::AccessPciRegionRegister { request } =
        &mut substituted.bootstrap.handler.operations[index]
    {
        request.value = 0;
    } else {
        unreachable!();
    }
    assert_eq!(
        validate_systems_program(&substituted).unwrap_err().code,
        "E-SYSTEMS-PCI-REGION"
    );
}

#[test]
fn requires_the_exact_virtio_used_ring_validation() {
    // TOPAL-SYSTEMS-DMA-COMPLETE-001, TOPAL-SYSTEMS-VIRTIO-PCI-USED-RING-001.
    let mut malformed = CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING[0];
    malformed.width_bytes = 3;
    assert!(!malformed.is_well_formed());

    let program = analyze_systems_for_compiler(
        &memory_source(),
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let complete_index = program
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::CompleteDmaBufferForCpu { .. }
            )
        })
        .unwrap();
    let read_indices = program
        .bootstrap
        .handler
        .operations
        .iter()
        .enumerate()
        .filter_map(|(index, operation)| {
            matches!(
                operation,
                CompilerSystemsOperation::LoadDmaBufferIntegerEquals { .. }
            )
            .then_some(index)
        })
        .collect::<Vec<_>>();
    assert_eq!(read_indices.len(), 4);

    let mut missing = program.clone();
    missing.bootstrap.handler.operations.remove(read_indices[3]);
    missing.bootstrap.handler.operations.retain(|operation| {
        !matches!(
            operation,
            CompilerSystemsOperation::CompleteDmaBufferForCpu { request }
                if *request == CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0]
        ) && !matches!(
            operation,
            CompilerSystemsOperation::CaptureEntropySample { .. }
        )
    });
    remove_entropy_sample_capture(&mut missing);
    assert_eq!(
        validate_systems_program(&missing).unwrap_err().code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut duplicated = program.clone();
    duplicated.bootstrap.handler.operations.insert(
        read_indices[1],
        duplicated.bootstrap.handler.operations[read_indices[0]].clone(),
    );
    assert_eq!(
        validate_systems_program(&duplicated).unwrap_err().code,
        "E-SYSTEMS-DMA-USED-RING"
    );

    let mut reordered = program.clone();
    reordered
        .bootstrap
        .handler
        .operations
        .swap(read_indices[0], read_indices[1]);
    assert_eq!(
        validate_systems_program(&reordered).unwrap_err().code,
        "E-SYSTEMS-DMA-USED-RING"
    );

    let mut substituted = program;
    if let CompilerSystemsOperation::CompleteDmaBufferForCpu { request } =
        &mut substituted.bootstrap.handler.operations[complete_index]
    {
        *request = CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0];
    } else {
        unreachable!();
    }
    assert_eq!(
        validate_systems_program(&substituted).unwrap_err().code,
        "E-SYSTEMS-DMA-COMPLETE"
    );
}

#[test]
fn requires_the_exact_virtio_payload_completion() {
    // TOPAL-SYSTEMS-DMA-COMPLETE-001,
    // TOPAL-SYSTEMS-VIRTIO-PCI-PAYLOAD-COMPLETE-001.
    let program = analyze_systems_for_compiler(
        &memory_source(),
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let indices = program
        .bootstrap
        .handler
        .operations
        .iter()
        .enumerate()
        .filter_map(|(index, operation)| {
            matches!(
                operation,
                CompilerSystemsOperation::CompleteDmaBufferForCpu { .. }
            )
            .then_some(index)
        })
        .collect::<Vec<_>>();
    assert_eq!(indices.len(), 2);

    let mut missing = program.clone();
    missing.bootstrap.handler.operations.remove(indices[1]);
    missing.bootstrap.handler.operations.retain(|operation| {
        !matches!(
            operation,
            CompilerSystemsOperation::CaptureEntropySample { .. }
        )
    });
    remove_entropy_sample_capture(&mut missing);
    assert_eq!(
        validate_systems_program(&missing).unwrap_err().code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut duplicated = program.clone();
    duplicated.bootstrap.handler.operations.insert(
        indices[1],
        duplicated.bootstrap.handler.operations[indices[1]].clone(),
    );
    assert_eq!(
        validate_systems_program(&duplicated).unwrap_err().code,
        "E-SYSTEMS-DMA-COMPLETE"
    );

    let mut reordered = program.clone();
    let last_read = reordered
            .bootstrap
            .handler
            .operations
            .iter()
            .position(|operation| {
                matches!(
                    operation,
                    CompilerSystemsOperation::LoadDmaBufferIntegerEquals { request }
                        if *request == CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING[3]
                )
            })
            .unwrap();
    reordered
        .bootstrap
        .handler
        .operations
        .swap(last_read, indices[1]);
    assert_eq!(
        validate_systems_program(&reordered).unwrap_err().code,
        "E-SYSTEMS-DMA-COMPLETE"
    );

    let mut substituted = program;
    if let CompilerSystemsOperation::CompleteDmaBufferForCpu { request } =
        &mut substituted.bootstrap.handler.operations[indices[1]]
    {
        *request = CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[1];
    } else {
        unreachable!();
    }
    assert_eq!(
        validate_systems_program(&substituted).unwrap_err().code,
        "E-SYSTEMS-DMA-COMPLETE"
    );
}

#[test]
fn requires_the_exact_uncredited_entropy_sample_capture() {
    // TOPAL-SYSTEMS-ENTROPY-SAMPLE-001,
    // TOPAL-SYSTEMS-VIRTIO-PCI-ENTROPY-CAPTURE-001.
    let mut malformed = CompilerEntropySampleCaptureRequest::INITIAL_Q35_VIRTIO_RNG;
    malformed.sample_identity = 0;
    assert!(!malformed.is_well_formed());

    let program = analyze_systems_for_compiler(
        &memory_source(),
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let index = program
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::CaptureEntropySample { .. }
            )
        })
        .unwrap();

    let mut missing = program.clone();
    missing.bootstrap.handler.operations.remove(index);
    remove_entropy_sample_capture(&mut missing);
    assert_eq!(
        validate_systems_program(&missing).unwrap_err().code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut duplicated = program.clone();
    duplicated.bootstrap.handler.operations.insert(
        index,
        duplicated.bootstrap.handler.operations[index].clone(),
    );
    assert_eq!(
        validate_systems_program(&duplicated).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-SAMPLE"
    );

    let mut reordered = program.clone();
    let payload_completion = reordered
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::CompleteDmaBufferForCpu { request }
                    if *request == CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0]
            )
        })
        .unwrap();
    reordered
        .bootstrap
        .handler
        .operations
        .swap(payload_completion, index);
    assert_eq!(
        validate_systems_program(&reordered).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-SAMPLE"
    );

    let mut substituted = program;
    if let CompilerSystemsOperation::CaptureEntropySample { request } =
        &mut substituted.bootstrap.handler.operations[index]
    {
        request.byte_count = 31;
    } else {
        unreachable!();
    }
    assert_eq!(
        validate_systems_program(&substituted).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-SAMPLE"
    );
}

#[test]
fn requires_the_exact_trusted_entropy_sample_quality() {
    // TOPAL-SYSTEMS-ENTROPY-QUALITY-001,
    // TOPAL-SYSTEMS-VIRTIO-PCI-ENTROPY-QUALITY-001.
    let mut malformed = CompilerEntropySampleQualityRequest::INITIAL_Q35_VIRTIO_RNG;
    malformed.quality_parts_per_1024 = 1023;
    assert!(!malformed.is_well_formed());

    let program = analyze_systems_for_compiler(
        &memory_source(),
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let index = program
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::AssessEntropySampleQuality { .. }
            )
        })
        .unwrap();

    let mut missing = program.clone();
    missing.bootstrap.handler.operations.remove(index);
    missing.bootstrap.handler.operations.retain(|operation| {
        !matches!(
            operation,
            CompilerSystemsOperation::CreateEntropyPool { .. }
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
    missing.bootstrap.handler.effects.retain(|effect| {
        *effect != SYSTEMS_ENTROPY_SAMPLE_ASSESS_QUALITY
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
    assert_eq!(
        validate_systems_program(&missing).unwrap_err().code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut duplicated = program.clone();
    duplicated.bootstrap.handler.operations.insert(
        index,
        duplicated.bootstrap.handler.operations[index].clone(),
    );
    assert_eq!(
        validate_systems_program(&duplicated).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-QUALITY"
    );

    let mut reordered = program.clone();
    let capture = reordered
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::CaptureEntropySample { .. }
            )
        })
        .unwrap();
    reordered.bootstrap.handler.operations.swap(capture, index);
    assert_eq!(
        validate_systems_program(&reordered).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-QUALITY"
    );

    let mut substituted = program;
    if let CompilerSystemsOperation::AssessEntropySampleQuality { request } =
        &mut substituted.bootstrap.handler.operations[index]
    {
        request.quality_parts_per_1024 = 512;
        request.credited_entropy_bits = 128;
    } else {
        unreachable!();
    }
    assert_eq!(
        validate_systems_program(&substituted).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-QUALITY"
    );
}

#[test]
fn requires_the_exact_empty_entropy_pool() {
    // TOPAL-SYSTEMS-ENTROPY-POOL-001,
    // TOPAL-SYSTEMS-VIRTIO-PCI-ENTROPY-POOL-001.
    let mut malformed = CompilerEntropyPoolRequest::INITIAL_Q35;
    malformed.initial_credited_entropy_bits = 1;
    assert!(!malformed.is_well_formed());

    let program = analyze_systems_for_compiler(
        &memory_source(),
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let index = program
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::CreateEntropyPool { .. }
            )
        })
        .unwrap();

    let mut missing = program.clone();
    missing.bootstrap.handler.operations.remove(index);
    missing.bootstrap.handler.operations.retain(|operation| {
        !matches!(
            operation,
            CompilerSystemsOperation::MixEntropyPoolSample { .. }
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
    missing.bootstrap.handler.effects.retain(|effect| {
        *effect != SYSTEMS_ENTROPY_POOL_CREATE
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
    assert_eq!(
        validate_systems_program(&missing).unwrap_err().code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut duplicated = program.clone();
    duplicated.bootstrap.handler.operations.insert(
        index,
        duplicated.bootstrap.handler.operations[index].clone(),
    );
    assert_eq!(
        validate_systems_program(&duplicated).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-POOL"
    );

    let mut reordered = program.clone();
    let assessment = reordered
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::AssessEntropySampleQuality { .. }
            )
        })
        .unwrap();
    reordered
        .bootstrap
        .handler
        .operations
        .swap(assessment, index);
    assert_eq!(
        validate_systems_program(&reordered).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-POOL"
    );

    let mut substituted = program;
    if let CompilerSystemsOperation::CreateEntropyPool { request } =
        &mut substituted.bootstrap.handler.operations[index]
    {
        request.credit_capacity_bits = 128;
    } else {
        unreachable!();
    }
    assert_eq!(
        validate_systems_program(&substituted).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-POOL"
    );
}

#[test]
fn requires_the_exact_first_entropy_pool_mix() {
    // TOPAL-SYSTEMS-ENTROPY-MIX-001,
    // TOPAL-SYSTEMS-VIRTIO-PCI-ENTROPY-MIX-001.
    let mut malformed = CompilerEntropyPoolMixRequest::INITIAL_Q35;
    malformed.byte_count = 31;
    assert!(!malformed.is_well_formed());

    let program = analyze_systems_for_compiler(
        &memory_source(),
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let index = program
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::MixEntropyPoolSample { .. }
            )
        })
        .unwrap();

    let mut missing = program.clone();
    missing.bootstrap.handler.operations.remove(index);
    missing.bootstrap.handler.operations.retain(|operation| {
        !matches!(
            operation,
            CompilerSystemsOperation::CreditEntropyPoolContribution { .. }
                | CompilerSystemsOperation::InitializeEntropyGenerator { .. }
                | CompilerSystemsOperation::ExtractEntropyGenerator { .. }
                | CompilerSystemsOperation::DeliverInitialAuxvRandom { .. }
                | CompilerSystemsOperation::ReserveInitialUserRandom { .. }
                | CompilerSystemsOperation::BindReusableUserRandom { .. }
                | CompilerSystemsOperation::ExpandSmallUserRandomRequests { .. }
                | CompilerSystemsOperation::ExpandLargeUserRandomRequests { .. }
        )
    });
    missing.bootstrap.handler.effects.retain(|effect| {
        *effect != SYSTEMS_ENTROPY_POOL_MIX_SAMPLE
            && *effect != SYSTEMS_ENTROPY_POOL_CREDIT_CONTRIBUTION
            && *effect != SYSTEMS_ENTROPY_GENERATOR_INITIALIZE
            && *effect != SYSTEMS_ENTROPY_GENERATOR_EXTRACT
            && *effect != SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM
            && *effect != SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM
            && *effect != SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE
            && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS
            && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS
    });
    assert_eq!(
        validate_systems_program(&missing).unwrap_err().code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut duplicated = program.clone();
    duplicated.bootstrap.handler.operations.insert(
        index,
        duplicated.bootstrap.handler.operations[index].clone(),
    );
    assert_eq!(
        validate_systems_program(&duplicated).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-MIX"
    );

    let mut reordered = program.clone();
    let pool = reordered
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::CreateEntropyPool { .. }
            )
        })
        .unwrap();
    reordered.bootstrap.handler.operations.swap(pool, index);
    assert_eq!(
        validate_systems_program(&reordered).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-MIX"
    );

    let mut substituted = program;
    if let CompilerSystemsOperation::MixEntropyPoolSample { request } =
        &mut substituted.bootstrap.handler.operations[index]
    {
        request.conditioner_identity = 2;
    } else {
        unreachable!();
    }
    assert_eq!(
        validate_systems_program(&substituted).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-MIX"
    );
}

#[test]
fn requires_the_exact_first_entropy_credit_transfer() {
    // TOPAL-SYSTEMS-ENTROPY-CREDIT-001,
    // TOPAL-SYSTEMS-VIRTIO-PCI-ENTROPY-CREDIT-001.
    let mut malformed = CompilerEntropyPoolCreditRequest::INITIAL_Q35;
    malformed.resulting_credited_entropy_bits = 255;
    assert!(!malformed.is_well_formed());

    let program = analyze_systems_for_compiler(
        &memory_source(),
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let index = program
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::CreditEntropyPoolContribution { .. }
            )
        })
        .unwrap();

    let mut missing = program.clone();
    missing.bootstrap.handler.operations.remove(index);
    missing.bootstrap.handler.operations.retain(|operation| {
        !matches!(
            operation,
            CompilerSystemsOperation::InitializeEntropyGenerator { .. }
                | CompilerSystemsOperation::ExtractEntropyGenerator { .. }
                | CompilerSystemsOperation::DeliverInitialAuxvRandom { .. }
                | CompilerSystemsOperation::ReserveInitialUserRandom { .. }
                | CompilerSystemsOperation::BindReusableUserRandom { .. }
                | CompilerSystemsOperation::ExpandSmallUserRandomRequests { .. }
                | CompilerSystemsOperation::ExpandLargeUserRandomRequests { .. }
        )
    });
    missing.bootstrap.handler.effects.retain(|effect| {
        *effect != SYSTEMS_ENTROPY_POOL_CREDIT_CONTRIBUTION
            && *effect != SYSTEMS_ENTROPY_GENERATOR_INITIALIZE
            && *effect != SYSTEMS_ENTROPY_GENERATOR_EXTRACT
            && *effect != SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM
            && *effect != SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM
            && *effect != SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE
            && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS
            && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS
    });
    assert_eq!(
        validate_systems_program(&missing).unwrap_err().code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut duplicated = program.clone();
    duplicated.bootstrap.handler.operations.insert(
        index,
        duplicated.bootstrap.handler.operations[index].clone(),
    );
    assert_eq!(
        validate_systems_program(&duplicated).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-CREDIT"
    );

    let mut reordered = program.clone();
    let mixing = reordered
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::MixEntropyPoolSample { .. }
            )
        })
        .unwrap();
    reordered.bootstrap.handler.operations.swap(mixing, index);
    assert_eq!(
        validate_systems_program(&reordered).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-CREDIT"
    );

    let mut substituted = program;
    if let CompilerSystemsOperation::CreditEntropyPoolContribution { request } =
        &mut substituted.bootstrap.handler.operations[index]
    {
        request.conditioner_origin_identity = 2;
    } else {
        unreachable!();
    }
    assert_eq!(
        validate_systems_program(&substituted).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-CREDIT"
    );
}

#[test]
fn requires_the_exact_initial_entropy_generator() {
    // TOPAL-SYSTEMS-ENTROPY-GENERATOR-001,
    // TOPAL-SYSTEMS-VIRTIO-PCI-ENTROPY-GENERATOR-001.
    let mut malformed = CompilerEntropyGeneratorRequest::INITIAL_Q35;
    malformed.key_byte_count = 31;
    assert!(!malformed.is_well_formed());
    let program = analyze_systems_for_compiler(
        &memory_source(),
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let index = program
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::InitializeEntropyGenerator { .. }
            )
        })
        .unwrap();
    let mut missing = program.clone();
    missing.bootstrap.handler.operations.remove(index);
    missing.bootstrap.handler.operations.retain(|operation| {
        !matches!(
            operation,
            CompilerSystemsOperation::ExtractEntropyGenerator { .. }
                | CompilerSystemsOperation::DeliverInitialAuxvRandom { .. }
                | CompilerSystemsOperation::ReserveInitialUserRandom { .. }
                | CompilerSystemsOperation::BindReusableUserRandom { .. }
                | CompilerSystemsOperation::ExpandSmallUserRandomRequests { .. }
                | CompilerSystemsOperation::ExpandLargeUserRandomRequests { .. }
        )
    });
    missing.bootstrap.handler.effects.retain(|effect| {
        *effect != SYSTEMS_ENTROPY_GENERATOR_INITIALIZE
            && *effect != SYSTEMS_ENTROPY_GENERATOR_EXTRACT
            && *effect != SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM
            && *effect != SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM
            && *effect != SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE
            && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS
            && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS
    });
    assert_eq!(
        validate_systems_program(&missing).unwrap_err().code,
        "E-SYSTEMS-USER-CONTEXT"
    );
    let mut duplicated = program.clone();
    duplicated.bootstrap.handler.operations.insert(
        index,
        duplicated.bootstrap.handler.operations[index].clone(),
    );
    assert_eq!(
        validate_systems_program(&duplicated).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-GENERATOR"
    );
    let mut reordered = program.clone();
    let credit = reordered
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::CreditEntropyPoolContribution { .. }
            )
        })
        .unwrap();
    reordered.bootstrap.handler.operations.swap(credit, index);
    assert_eq!(
        validate_systems_program(&reordered).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-GENERATOR"
    );
    let mut substituted = program;
    if let CompilerSystemsOperation::InitializeEntropyGenerator { request } =
        &mut substituted.bootstrap.handler.operations[index]
    {
        request.algorithm_identity = 2;
    } else {
        unreachable!();
    }
    assert_eq!(
        validate_systems_program(&substituted).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-GENERATOR"
    );
}

#[test]
fn requires_the_exact_first_entropy_generator_extraction() {
    // TOPAL-SYSTEMS-ENTROPY-EXTRACT-001,
    // TOPAL-SYSTEMS-VIRTIO-PCI-ENTROPY-EXTRACT-001.
    let mut malformed = CompilerEntropyGeneratorExtractRequest::INITIAL_Q35;
    malformed.block_byte_count = 63;
    assert!(!malformed.is_well_formed());
    let program = analyze_systems_for_compiler(
        &memory_source(),
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let index = program
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::ExtractEntropyGenerator { .. }
            )
        })
        .unwrap();
    let mut missing = program.clone();
    missing.bootstrap.handler.operations.remove(index);
    missing.bootstrap.handler.operations.retain(|operation| {
        !matches!(
            operation,
            CompilerSystemsOperation::ExtractEntropyGenerator { .. }
                | CompilerSystemsOperation::DeliverInitialAuxvRandom { .. }
                | CompilerSystemsOperation::ReserveInitialUserRandom { .. }
                | CompilerSystemsOperation::BindReusableUserRandom { .. }
                | CompilerSystemsOperation::ExpandSmallUserRandomRequests { .. }
                | CompilerSystemsOperation::ExpandLargeUserRandomRequests { .. }
        )
    });
    missing.bootstrap.handler.effects.retain(|effect| {
        *effect != SYSTEMS_ENTROPY_GENERATOR_EXTRACT
            && *effect != SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM
            && *effect != SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM
            && *effect != SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE
            && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS
            && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS
    });
    assert_eq!(
        validate_systems_program(&missing).unwrap_err().code,
        "E-SYSTEMS-USER-CONTEXT"
    );
    let mut duplicated = program.clone();
    duplicated.bootstrap.handler.operations.insert(
        index,
        duplicated.bootstrap.handler.operations[index].clone(),
    );
    assert_eq!(
        validate_systems_program(&duplicated).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-EXTRACT"
    );
    let mut reordered = program.clone();
    let initialization = reordered
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::InitializeEntropyGenerator { .. }
            )
        })
        .unwrap();
    reordered
        .bootstrap
        .handler
        .operations
        .swap(initialization, index);
    assert_eq!(
        validate_systems_program(&reordered).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-EXTRACT"
    );
    let mut substituted = program;
    if let CompilerSystemsOperation::ExtractEntropyGenerator { request } =
        &mut substituted.bootstrap.handler.operations[index]
    {
        request.counter = 1;
    } else {
        unreachable!();
    }
    assert_eq!(
        validate_systems_program(&substituted).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-EXTRACT"
    );
}

#[test]
fn requires_affine_initial_auxv_random_delivery_before_user_entry() {
    // TOPAL-SYSTEMS-ENTROPY-DELIVER-001,
    // TOPAL-SYSTEMS-USER-AUXV-RANDOM-001.
    let mut malformed = CompilerEntropyOutputDeliveryRequest::INITIAL_AUXV_RANDOM;
    malformed.delivered_byte_count = 15;
    assert!(!malformed.is_well_formed());
    let program = analyze_systems_for_compiler(
        &memory_source(),
        &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
    )
    .unwrap();
    let delivery = program
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| {
            matches!(
                operation,
                CompilerSystemsOperation::DeliverInitialAuxvRandom { .. }
                    | CompilerSystemsOperation::ReserveInitialUserRandom { .. }
            )
        })
        .unwrap();
    let entry = program
        .bootstrap
        .handler
        .operations
        .iter()
        .position(|operation| matches!(operation, CompilerSystemsOperation::EnterUserContext))
        .unwrap();
    assert!(delivery < entry);

    let mut missing = program.clone();
    missing.bootstrap.handler.operations.remove(delivery);
    missing.bootstrap.handler.operations.retain(|operation| {
        !matches!(
            operation,
            CompilerSystemsOperation::ExtractEntropyGenerator { request }
                if *request == CompilerEntropyGeneratorExtractRequest::INITIAL_Q35_GETRANDOM
        ) && !matches!(
            operation,
            CompilerSystemsOperation::ReserveInitialUserRandom { .. }
                | CompilerSystemsOperation::BindReusableUserRandom { .. }
                | CompilerSystemsOperation::ExpandSmallUserRandomRequests { .. }
                | CompilerSystemsOperation::ExpandLargeUserRandomRequests { .. }
        )
    });
    missing.bootstrap.handler.effects.retain(|effect| {
        *effect != SYSTEMS_ENTROPY_OUTPUT_DELIVER_INITIAL_AUXV_RANDOM
            && *effect != SYSTEMS_ENTROPY_OUTPUT_RESERVE_INITIAL_USER_RANDOM
            && *effect != SYSTEMS_ENTROPY_USER_RANDOM_BIND_REUSABLE
            && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_SMALL_REQUESTS
            && *effect != SYSTEMS_ENTROPY_USER_RANDOM_EXPAND_LARGE_REQUESTS
    });
    assert_eq!(
        validate_systems_program(&missing).unwrap_err().code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut duplicated = program.clone();
    duplicated.bootstrap.handler.operations.insert(
        delivery,
        duplicated.bootstrap.handler.operations[delivery].clone(),
    );
    assert_eq!(
        validate_systems_program(&duplicated).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-DELIVER"
    );

    let mut reordered = program.clone();
    reordered.bootstrap.handler.operations.swap(delivery, entry);
    assert_eq!(
        validate_systems_program(&reordered).unwrap_err().code,
        "E-SYSTEMS-USER-CONTEXT"
    );

    let mut substituted = program;
    if let CompilerSystemsOperation::DeliverInitialAuxvRandom { request } =
        &mut substituted.bootstrap.handler.operations[delivery]
    {
        request.auxiliary_tag = 24;
    } else {
        unreachable!();
    }
    assert_eq!(
        validate_systems_program(&substituted).unwrap_err().code,
        "E-SYSTEMS-ENTROPY-DELIVER"
    );
}

#[test]
fn rejects_unbounded_or_non_affine_bootstrap_region_forms() {
    // TOPAL-SYSTEMS-AUTHORITY-001, TOPAL-SYSTEMS-STORAGE-001.
    for (source, code, expected) in [
        (
            memory_source().replace(
                "region byte store (offset-bytes is 0, value is 90)",
                "region byte store (offset-bytes is 64, value is 90)",
            ),
            "E-SYSTEMS-STORAGE",
            "outside the static region size",
        ),
        (
            memory_source().replace(
                "region byte store (offset-bytes is 0, value is 90)",
                "region byte store (offset-bytes is 0, value is 256)",
            ),
            "E-SYSTEMS-STORAGE",
            "from 0 through 255",
        ),
        (
            memory_source().replacen(
                "restored bootstrap release region",
                "restored console write \"not released\"",
                1,
            ),
            "E-SYSTEMS-STORAGE",
            "release requires",
        ),
        (
            memory_source().replace(
                "restored fatal \"toolchain gate allocation failed\"",
                "region byte load (offset-bytes is 0)",
            ),
            "E-SYSTEMS-DISPOSITION",
            "final operation is not a disposition",
        ),
    ] {
        let error = analyze_systems_for_compiler(
            &source,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err();
        assert_eq!(error.code, code);
        assert!(error.message.contains(expected), "{}", error.message);
    }
}

#[test]
fn requires_the_sealed_affine_atomic_word_lifecycle() {
    // TOPAL-SYSTEMS-ATOMIC-001, TOPAL-SYSTEMS-ORDER-001,
    // TOPAL-SYSTEMS-QUALIFY-001.
    for (source, expected) in [
        (
            memory_source().replace("domain is cpu-shared", "domain is device"),
            "requires `cpu-shared`",
        ),
        (
            memory_source().replace("offset-bytes is 8", "offset-bytes is 9"),
            "requires aligned offset 8",
        ),
        (
            memory_source().replace(
                "success-order is acquire-release",
                "success-order is release",
            ),
            "initial compare/exchange requires",
        ),
        (
            memory_source().replace("failure-order is acquire", "failure-order is release"),
            "initial compare/exchange requires",
        ),
        (
            memory_source().replacen("region is atomic end", "wrong-region is atomic end", 1),
            "restore the original region binding",
        ),
        (
            memory_source().replace("Observed actual then", "Exchanged actual then"),
            "`Exchanged`, `Observed` order",
        ),
    ] {
        let error = analyze_systems_for_compiler(
            &source,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err();
        assert_eq!(error.code, "E-SYSTEMS-ATOMIC");
        assert!(error.message.contains(expected), "{}", error.message);
    }
}

#[test]
fn rejects_hosted_and_mismatched_target_profiles() {
    // TOPAL-SYSTEMS-FEATURE-001, TOPAL-SYSTEMS-QUALIFY-001.
    let hosted = CompilerSystemsTargetSelection {
        target: "x86_64-unknown-linux-gnu".into(),
        board: INITIAL_SYSTEMS_BOARD.into(),
        profile: INITIAL_SYSTEMS_PROFILE.into(),
    };
    assert_eq!(
        analyze_systems_for_compiler(SOURCE, &hosted)
            .unwrap_err()
            .code,
        "E-SYSTEMS-TARGET"
    );
    let wrong_board = CompilerSystemsTargetSelection {
        target: INITIAL_SYSTEMS_TARGET.into(),
        board: "another-board".into(),
        profile: INITIAL_SYSTEMS_PROFILE.into(),
    };
    assert_eq!(
        analyze_systems_for_compiler(SOURCE, &wrong_board)
            .unwrap_err()
            .code,
        "E-SYSTEMS-TARGET"
    );
    let hosted_compiler = analyze_for_compiler(SOURCE).unwrap_err();
    assert_eq!(hosted_compiler.code, "E-COMPILER-UNSUPPORTED");
    assert!(
        hosted_compiler
            .message
            .contains("`systems` language feature")
    );
}

#[test]
fn rejects_context_escape_unknown_operations_and_missing_dispositions() {
    // TOPAL-SYSTEMS-AUTHORITY-001, TOPAL-SYSTEMS-DISPOSITION-001.
    let escaped = SOURCE.replace(
        "restored console write \"TOPAL_KERNEL_BOOT\"",
        "saved is translated",
    );
    assert_eq!(
        analyze_systems_for_compiler(
            &escaped,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err()
        .code,
        "E-SYSTEMS-AFFINE-CONTEXT"
    );

    let unknown = SOURCE.replace("restored debug break", "restored machine instruction");
    assert_eq!(
        analyze_systems_for_compiler(
            &unknown,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err()
        .code,
        "E-SYSTEMS-OPERATION"
    );

    let missing = SOURCE.replace(
        "  context resume",
        "  context console write \"missing disposition\"",
    );
    assert_eq!(
        analyze_systems_for_compiler(
            &missing,
            &CompilerSystemsTargetSelection::initial_x86_64_qemu(),
        )
        .unwrap_err()
        .code,
        "E-SYSTEMS-DISPOSITION"
    );
}
