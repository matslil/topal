fn audit_provider_entropy_authority(file: &object::File<'_>) {
    let authorize_random_slot = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_STACK_RANDOM_SLOT_SYMBOL,
    );
    let revoke_random_slot = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_REVOKE_STACK_RANDOM_SLOT_SYMBOL,
    );
    let retain_prepared_process = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_RETAIN_PREPARED_PROCESS_SYMBOL,
    );
    let restore_prepared_process = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_RESTORE_PREPARED_PROCESS_SYMBOL,
    );
    let qualify_retained_disjoint_work = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_RETAINED_DISJOINT_WORK_SYMBOL,
    );
    let authorize_entropy_handoff = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_ENTROPY_HANDOFF_SYMBOL,
    );
    let authorize_entropy_acquisition = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_AUTHORIZE_ENTROPY_ACQUISITION_SYMBOL,
    );
    let qualify_entropy_device = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_DEVICE_SYMBOL,
    );
    let qualify_entropy_transport = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_TRANSPORT_SYMBOL,
    );
    let qualify_entropy_negotiation = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_NEGOTIATION_SYMBOL,
    );
    let qualify_entropy_queue =
        symbol_bytes(&file, X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_QUEUE_SYMBOL);
    let qualify_entropy_bus_master = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_BUS_MASTER_SYMBOL,
    );
    let qualify_entropy_dma_buffers = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_DMA_BUFFERS_SYMBOL,
    );
    let qualify_entropy_split_ring = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_SPLIT_RING_SYMBOL,
    );
    let qualify_entropy_dma_preparation = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_DMA_PREPARATION_SYMBOL,
    );
    let qualify_entropy_queue_addresses = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_QUEUE_ADDRESSES_SYMBOL,
    );
    let qualify_entropy_queue_activation = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_QUEUE_ACTIVATION_SYMBOL,
    );
    let qualify_entropy_request_completion = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_REQUEST_COMPLETION_SYMBOL,
    );
    let qualify_entropy_sample_capture = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_SAMPLE_CAPTURE_SYMBOL,
    );
    let qualify_entropy_sample_quality = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_SAMPLE_QUALITY_SYMBOL,
    );
    let qualify_entropy_pool_lifecycle = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_POOL_LIFECYCLE_SYMBOL,
    );
    let qualify_entropy_generator_lifecycle = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_QUALIFY_ENTROPY_GENERATOR_LIFECYCLE_SYMBOL,
    );
    let revoke_entropy_acquisition = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTROPY_ACQUISITION_SYMBOL,
    );
    let revoke_entropy_handoff = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTROPY_HANDOFF_SYMBOL,
    );
    for bytes in [
        authorize_random_slot,
        retain_prepared_process,
        qualify_retained_disjoint_work,
        authorize_entropy_handoff,
        authorize_entropy_acquisition,
        qualify_entropy_device,
        qualify_entropy_transport,
        qualify_entropy_negotiation,
        qualify_entropy_queue,
        qualify_entropy_bus_master,
        qualify_entropy_dma_buffers,
        qualify_entropy_split_ring,
        qualify_entropy_dma_preparation,
        qualify_entropy_queue_addresses,
        qualify_entropy_queue_activation,
        qualify_entropy_request_completion,
        qualify_entropy_sample_capture,
        qualify_entropy_sample_quality,
        qualify_entropy_pool_lifecycle,
        qualify_entropy_generator_lifecycle,
        revoke_entropy_acquisition,
        revoke_entropy_handoff,
        restore_prepared_process,
        revoke_random_slot,
    ] {
        assert_eq!(
            bytes
                .windows(11)
                .filter(|window| {
                    *window == [0x31, 0xc0, 0xb9, 0xd2, 0x01, 0, 0, 0xfc, 0xf3, 0x48, 0xaf]
                })
                .count(),
            1,
            "random-slot and prepared-process authority must completely verify unchanged zero regions"
        );
        for offset in [0x0fd0_u32, 0x0fd8_u32] {
            assert!(
                bytes.windows(8).any(|window| {
                    window[..3] == [0x49, 0x83, 0xbb]
                        && window[3..7] == offset.to_le_bytes()
                        && window[7] == 0
                }),
                "random-slot and prepared-process authority must verify both future object qwords as zero"
            );
        }
        assert!(
            !bytes.windows(3).any(|window| window == [0x49, 0x89, 0x83]),
            "random-slot and prepared-process authority must not mutate the deterministic stack"
        );
        assert!(
            !bytes.windows(3).any(|window| window == [0x0f, 0xae, 0xf8]),
            "read-only random-slot authority must not issue a store fence"
        );
        assert!(
            !bytes.windows(3).any(|window| window == [0x0f, 0x01, 0x3f]),
            "inactive random-slot authority must not invalidate an active address"
        );
    }

    let deliver_initial_random = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_DELIVER_ENTROPY_INITIAL_RANDOM_SYMBOL,
    );
    let revoke_initial_random = symbol_bytes(
        &file,
        X86_SYSTEMS_STATIC_IMAGE_REVOKE_ENTROPY_INITIAL_RANDOM_SYMBOL,
    );
    for offset in [0x0e70_u32, 0x0e78, 0x0fc8, 0x0fd0] {
        assert!(
            deliver_initial_random
                .windows(7)
                .any(|window| window[3..7] == offset.to_le_bytes()),
            "initial-random delivery must address every object and auxiliary-vector qword"
        );
        assert!(
            revoke_initial_random
                .windows(7)
                .any(|window| window[3..7] == offset.to_le_bytes()),
            "initial-random inverse must address every object and auxiliary-vector qword"
        );
    }
    assert!(
        deliver_initial_random
            .windows(8)
            .any(|window| { window == 0x007f_fe70_u64.to_le_bytes() })
    );
    assert!(
        revoke_initial_random
            .windows(8)
            .any(|window| { window == 0x007f_fe70_u64.to_le_bytes() })
    );
    for offset in [
        168_u32, 176, 192, 200, 208, 216, 224, 232, 240, 248, 256, 264, 272,
    ] {
        assert!(
            deliver_initial_random
                .windows(7)
                .any(|window| window[3..7] == offset.to_le_bytes()),
            "reusable initial-random delivery must address every private service field"
        );
    }
}
