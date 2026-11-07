fn analyze_static_elf_frame_sequence(
    source: &SourceText,
    statements: &[Statement],
    allocator_context: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CheckedBootstrapRegionDecision, Diagnostic> {
    let (subject, extent_name, success, _problem_name, failure, span) =
        result_sequence(source, statements, "static ELF frame allocation")?;
    let request = parse_physical_frame_allocation(source, subject, allocator_context)?;
    if request
        != (CompilerPhysicalFrameRequest {
            frame_count: 503,
            alignment_frames: 1,
        })
    {
        return Err(translation_edit_diagnostic(
            source,
            subject.span(),
            "the static ELF frame probe requires exactly 503 frames aligned to one frame",
        ));
    }
    let CompilerSystemsDisposition::Fatal {
        message: failure_message,
    } = action_disposition(source, failure, allocator_context)?
    else {
        return Err(translation_edit_diagnostic(
            source,
            failure.span(),
            "static ELF frame allocation failure must consume the allocator context through `fatal`",
        ));
    };
    let success_statements = translation_block(
        source,
        success,
        span,
        "static ELF frame allocation success requires its exact marker and nested metadata allocation",
    )?;
    let [marker, nested @ ..] = success_statements else {
        return Err(translation_edit_diagnostic(
            source,
            success.span(),
            "static ELF frame allocation success requires its exact marker and nested metadata allocation",
        ));
    };
    let marker = analyze_operation(
        source,
        marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_FRAME_EXTENT_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            statement_span(success_statements.first().expect("marker exists")),
            "static ELF frame allocation must publish its exact marker before release",
        ));
    }

    let (
        metadata_subject,
        metadata_name,
        metadata_success,
        _problem_name,
        metadata_failure,
        metadata_span,
    ) = result_sequence(source, nested, "static ELF mapping metadata allocation")?;
    let metadata_request =
        parse_physical_frame_allocation(source, metadata_subject, allocator_context)?;
    if metadata_request
        != (CompilerPhysicalFrameRequest {
            frame_count: 1,
            alignment_frames: 1,
        })
    {
        return Err(translation_edit_diagnostic(
            source,
            metadata_subject.span(),
            "static ELF mapping metadata requires exactly one frame aligned to one frame",
        ));
    }
    let CompilerSystemsDisposition::Fatal {
        message: metadata_failure_message,
    } = action_disposition(source, metadata_failure, allocator_context)?
    else {
        return Err(translation_edit_diagnostic(
            source,
            metadata_failure.span(),
            "static ELF metadata allocation failure must consume the allocator context through `fatal`",
        ));
    };
    let metadata_statements = translation_block(
        source,
        metadata_success,
        metadata_span,
        "static ELF mapping resources require their exact marker and metadata-then-payload release",
    )?;
    let [resources_marker, construction @ ..] = metadata_statements else {
        return Err(translation_edit_diagnostic(
            source,
            metadata_success.span(),
            "static ELF mapping resources require their exact marker and construction preparation",
        ));
    };
    let resources_marker = analyze_operation(
        source,
        resources_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &resources_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_MAPPING_RESOURCES_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            statement_span(metadata_statements.first().expect("marker exists")),
            "static ELF mapping resources must publish their exact marker before release",
        ));
    }

    let (
        construction_subject,
        construction_name,
        construction_success,
        _problem_name,
        construction_failure,
        construction_span,
    ) = result_sequence(source, construction, "static ELF initial leaf construction")?;
    parse_static_image_construction_prepare(
        source,
        construction_subject,
        allocator_context,
        &extent_name,
        &metadata_name,
    )?;
    let CompilerSystemsDisposition::Fatal {
        message: construction_failure_message,
    } = action_disposition(source, construction_failure, allocator_context)?
    else {
        return Err(translation_edit_diagnostic(
            source,
            construction_failure.span(),
            "static ELF construction failure must consume the allocator context through `fatal`",
        ));
    };
    let constructed = translation_block(
        source,
        construction_success,
        construction_span,
        "static ELF initial leaves require witness, revocation, and exact resource return",
    )?;
    let [leaves_marker, zero] = constructed else {
        return Err(translation_edit_diagnostic(
            source,
            construction_success.span(),
            "static ELF initial leaves require witness, revocation, and exact resource return",
        ));
    };
    let leaves_marker = analyze_operation(
        source,
        leaves_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &leaves_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_INITIAL_LEAVES_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            statement_span(constructed.first().expect("marker exists")),
            "static ELF initial leaves must publish their exact marker before revocation",
        ));
    }
    let (zeroed_construction_name, after_zero) =
        parse_static_image_payload_zero(source, zero, allocator_context, &construction_name)?;
    let [
        zero_marker,
        copy,
        copy_marker,
        seal,
        seal_marker,
        hierarchy_reserve,
        hierarchy_marker,
        attach,
        attach_marker,
        activate,
        active_marker,
        restore,
        restored_marker,
        authorize_entry,
        entry_marker,
        authorize_stack_slot,
        stack_slot_marker,
        stack_backing_reserve,
        stack_backing_marker,
        zero_stack_backing,
        stack_backing_zeroed_marker,
        publish_stack_leaf,
        stack_leaf_published_marker,
        initialize_stack_entry,
        stack_entry_initialized_marker,
        add_stack_pagesz,
        stack_pagesz_marker,
        add_stack_entry_fact,
        stack_entry_fact_marker,
        add_stack_program_headers,
        stack_program_headers_marker,
        add_stack_executable_name,
        stack_executable_name_marker,
        add_stack_credentials,
        stack_credentials_marker,
        add_stack_secure_mode,
        stack_secure_mode_marker,
        add_stack_clock_tick_rate,
        stack_clock_tick_rate_marker,
        add_stack_platform,
        stack_platform_marker,
        add_stack_hardware_capabilities,
        stack_hardware_capabilities_marker,
        add_stack_load_metadata,
        stack_load_metadata_marker,
        authorize_stack_random_slot,
        stack_random_slot_authorized_marker,
        retain_prepared_process,
        prepared_process_retained_marker,
        qualify_retained_disjoint_work,
        retained_disjoint_work_marker,
        authorize_entropy_handoff,
        entropy_handoff_authorized_marker,
        authorize_entropy_acquisition,
        entropy_acquisition_authorized_marker,
        qualify_entropy_device,
        entropy_device_marker,
        qualify_entropy_transport,
        entropy_transport_marker,
        qualify_entropy_negotiation,
        entropy_negotiation_marker,
        qualify_entropy_queue,
        entropy_queue_marker,
        qualify_entropy_bus_master,
        entropy_bus_master_marker,
        qualify_entropy_dma_buffers,
        entropy_dma_buffers_marker,
        qualify_entropy_split_ring,
        entropy_split_ring_marker,
        qualify_entropy_dma_preparation,
        entropy_dma_preparation_marker,
        qualify_entropy_queue_addresses,
        entropy_queue_addresses_marker,
        qualify_entropy_queue_activation,
        entropy_queue_activation_marker,
        qualify_entropy_request_completion,
        entropy_request_completion_marker,
        qualify_entropy_sample_capture,
        entropy_sample_capture_marker,
        qualify_entropy_sample_quality,
        entropy_sample_quality_marker,
        qualify_entropy_pool_lifecycle,
        entropy_pool_lifecycle_marker,
        qualify_entropy_generator_lifecycle,
        entropy_generator_lifecycle_marker,
        deliver_entropy_initial_random,
        entropy_initial_random_delivered_marker,
        enter_static_process,
        static_process_initial_brk_query_marker,
        static_process_initial_brk_growth_marker,
        static_process_initial_x64_fs_base_marker,
        static_process_initial_clear_tid_marker,
        static_process_initial_empty_robust_list_marker,
        static_process_initial_rseq_fallback_marker,
        static_process_initial_stack_limit_marker,
        static_process_initial_readlinkat_fallback_marker,
        static_process_initial_getrandom_marker,
        static_process_reusable_getrandom_marker,
        static_process_post_random_brk_query_marker,
        static_process_post_random_brk_growth_marker,
        static_process_post_random_brk_page_alignment_marker,
        static_process_relro_mprotect_marker,
        static_process_initial_process_name_marker,
        static_process_initial_getuid_marker,
        static_process_initial_config_stat_profile_marker,
        static_process_initial_credential_tail_profile_marker,
        static_process_exited_marker,
        restore_static_process_entry,
        static_process_entry_restored_marker,
        revoke_entropy_initial_random,
        entropy_initial_random_revoked_marker,
        revoke_entropy_acquisition,
        entropy_acquisition_revoked_marker,
        revoke_entropy_handoff,
        entropy_handoff_revoked_marker,
        restore_prepared_process,
        prepared_process_restored_marker,
        revoke_stack_random_slot,
        stack_random_slot_revoked_marker,
        remove_stack_load_metadata,
        stack_load_metadata_revoked_marker,
        remove_stack_hardware_capabilities,
        stack_hardware_capabilities_revoked_marker,
        remove_stack_platform,
        stack_platform_revoked_marker,
        remove_stack_clock_tick_rate,
        stack_clock_tick_rate_revoked_marker,
        remove_stack_secure_mode,
        stack_secure_mode_revoked_marker,
        remove_stack_credentials,
        stack_credentials_revoked_marker,
        remove_stack_executable_name,
        executable_name_revoked_marker,
        remove_stack_program_headers,
        stack_program_headers_revoked_marker,
        remove_stack_entry_fact,
        stack_entry_fact_revoked_marker,
        remove_stack_pagesz,
        stack_pagesz_revoked_marker,
        clear_stack_entry,
        stack_entry_cleared_marker,
        revoke_stack_leaf,
        stack_leaf_revoked_marker,
        stack_backing_release,
        revoke_stack_slot,
        stack_slot_revoked_marker,
        revoke_entry,
        entry_revoked_marker,
        detach,
        detach_marker,
        hierarchy_reclaim,
        hierarchy_release,
        revoke,
        metadata_release,
        reclaim,
        payload_release,
        rest @ ..,
    ] = after_zero
    else {
        return Err(translation_edit_diagnostic(
            source,
            statement_span(zero),
            "static ELF payload zeroing requires its exact witness and population",
        ));
    };
    let zero_marker = analyze_operation(
        source,
        zero_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &zero_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_PAYLOAD_ZEROED_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            statement_span(after_zero.first().expect("marker exists")),
            "static ELF payload zeroing must publish its exact marker before revocation",
        ));
    }
    let populated_construction_name = parse_static_image_payload_copy(
        source,
        copy,
        allocator_context,
        &zeroed_construction_name,
    )?;
    let copy_marker = parse_static_image_marker(
        source,
        copy_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_PAYLOAD_COPIED_OK",
        "static ELF payload copy must publish its exact marker before revocation",
    )?;
    let sealed_construction_name = parse_static_image_permission_seal(
        source,
        seal,
        allocator_context,
        &populated_construction_name,
    )?;
    let seal_marker = parse_static_image_marker(
        source,
        seal_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_PERMISSIONS_OK",
        "static ELF permission sealing must publish its exact marker before revocation",
    )?;
    let hierarchy_name = parse_static_image_hierarchy_reservation(
        source,
        hierarchy_reserve,
        allocator_context,
        &sealed_construction_name,
    )?;
    let hierarchy_marker = parse_static_image_marker(
        source,
        hierarchy_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_HIERARCHY_EXTENT_OK",
        "static ELF hierarchy allocation must publish its exact marker before release",
    )?;
    let attached_construction_name = parse_static_image_address_space_attach(
        source,
        attach,
        allocator_context,
        &sealed_construction_name,
        &hierarchy_name,
    )?;
    let attach_marker = parse_static_image_marker(
        source,
        attach_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_OK",
        "static ELF address-space attachment must publish its exact marker before detachment",
    )?;
    let active_construction_name = parse_static_image_address_space_transition(
        source,
        activate,
        allocator_context,
        &attached_construction_name,
        "activate",
        "bounded-static-elf-kernel-continuation",
    )?;
    let active_marker = parse_static_image_marker(
        source,
        active_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_ACTIVE_OK",
        "static ELF address-space activation must publish its exact marker before restoration",
    )?;
    let restored_construction_name = parse_static_image_address_space_transition(
        source,
        restore,
        allocator_context,
        &active_construction_name,
        "restore",
        "bounded-bootstrap-kernel-continuation",
    )?;
    let restored_marker = parse_static_image_marker(
        source,
        restored_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_RESTORED_OK",
        "static ELF address-space restoration must publish its exact marker before detachment",
    )?;
    let entry_authorized_construction_name = parse_static_image_entry_transition(
        source,
        authorize_entry,
        allocator_context,
        &restored_construction_name,
        "authorize",
        "validated-static-elf-entry",
    )?;
    let entry_marker = parse_static_image_marker(
        source,
        entry_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ENTRY_AUTHORITY_OK",
        "static ELF entry authorization must publish its exact marker before revocation",
    )?;
    let stack_slot_authorized_construction_name = parse_static_image_stack_slot_transition(
        source,
        authorize_stack_slot,
        allocator_context,
        &entry_authorized_construction_name,
        "authorize",
        "bounded-linux-initial-stack-slot",
    )?;
    let stack_slot_marker = parse_static_image_marker(
        source,
        stack_slot_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_STACK_SLOT_AUTHORITY_OK",
        "static ELF stack-slot authorization must publish its exact marker before revocation",
    )?;
    let stack_backing_name = parse_static_image_stack_backing_reservation(
        source,
        stack_backing_reserve,
        allocator_context,
        &stack_slot_authorized_construction_name,
    )?;
    let stack_backing_marker = parse_static_image_marker(
        source,
        stack_backing_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_EXTENT_OK",
        "static ELF stack backing must publish its exact marker before whole release",
    )?;
    let zeroed_stack_backing_construction_name = parse_static_image_stack_backing_zero(
        source,
        zero_stack_backing,
        allocator_context,
        &stack_slot_authorized_construction_name,
        &stack_backing_name,
    )?;
    let stack_backing_zeroed_marker = parse_static_image_marker(
        source,
        stack_backing_zeroed_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_ZEROED_OK",
        "static ELF stack-backing zeroing must publish its exact marker before whole release",
    )?;
    let published_stack_leaf_construction_name = parse_static_image_stack_leaf_publication(
        source,
        publish_stack_leaf,
        allocator_context,
        &zeroed_stack_backing_construction_name,
        &stack_backing_name,
    )?;
    let stack_leaf_published_marker = parse_static_image_marker(
        source,
        stack_leaf_published_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_PUBLISHED_OK",
        "static ELF stack-leaf publication must publish its exact marker before revocation",
    )?;
    let initialized_stack_entry_construction_name = parse_static_image_stack_entry_initialization(
        source,
        initialize_stack_entry,
        allocator_context,
        &published_stack_leaf_construction_name,
        &stack_backing_name,
    )?;
    let stack_entry_initialized_marker = parse_static_image_marker(
        source,
        stack_entry_initialized_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_INITIALIZED_OK",
        "static ELF stack-entry initialization must publish its exact marker before clearing",
    )?;
    let pagesz_stack_entry_construction_name = parse_static_image_stack_pagesz_addition(
        source,
        add_stack_pagesz,
        allocator_context,
        &initialized_stack_entry_construction_name,
        &stack_backing_name,
    )?;
    let stack_pagesz_marker = parse_static_image_marker(
        source,
        stack_pagesz_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_AUXV_PAGESZ_OK",
        "static ELF AT_PAGESZ addition must publish its exact marker before removal",
    )?;
    let entry_fact_stack_construction_name = parse_static_image_stack_entry_fact_addition(
        source,
        add_stack_entry_fact,
        allocator_context,
        &pagesz_stack_entry_construction_name,
        &stack_backing_name,
    )?;
    let stack_entry_fact_marker = parse_static_image_marker(
        source,
        stack_entry_fact_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_AUXV_ENTRY_OK",
        "static ELF AT_ENTRY addition must publish its exact marker before removal",
    )?;
    let program_header_stack_construction_name = parse_static_image_stack_program_headers_addition(
        source,
        add_stack_program_headers,
        allocator_context,
        &entry_fact_stack_construction_name,
        &stack_backing_name,
    )?;
    let stack_program_headers_marker = parse_static_image_marker(
        source,
        stack_program_headers_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_AUXV_PROGRAM_HEADERS_OK",
        "static ELF program-header addition must publish its exact marker before removal",
    )?;
    let executable_name_stack_construction_name =
        parse_static_image_stack_executable_name_transition(
            source,
            add_stack_executable_name,
            allocator_context,
            &program_header_stack_construction_name,
            &stack_backing_name,
            true,
        )?;
    let stack_executable_name_marker = parse_static_image_marker(
        source,
        stack_executable_name_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_AUXV_EXECFN_OK",
        "static ELF AT_EXECFN addition must publish its exact marker before removal",
    )?;
    let credential_stack_construction_name = parse_static_image_stack_credentials_transition(
        source,
        add_stack_credentials,
        allocator_context,
        &executable_name_stack_construction_name,
        &stack_backing_name,
        true,
    )?;
    let stack_credentials_marker = parse_static_image_marker(
        source,
        stack_credentials_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_AUXV_CREDENTIALS_OK",
        "static ELF credential addition must publish its exact marker before removal",
    )?;
    let secure_mode_stack_construction_name = parse_static_image_stack_secure_mode_transition(
        source,
        add_stack_secure_mode,
        allocator_context,
        &credential_stack_construction_name,
        &stack_backing_name,
        true,
    )?;
    let stack_secure_mode_marker = parse_static_image_marker(
        source,
        stack_secure_mode_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_AUXV_SECURE_OK",
        "static ELF AT_SECURE addition must publish its exact marker before removal",
    )?;
    let clock_tick_rate_stack_construction_name =
        parse_static_image_stack_clock_tick_rate_transition(
            source,
            add_stack_clock_tick_rate,
            allocator_context,
            &secure_mode_stack_construction_name,
            &stack_backing_name,
            true,
        )?;
    let stack_clock_tick_rate_marker = parse_static_image_marker(
        source,
        stack_clock_tick_rate_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_AUXV_CLKTCK_OK",
        "static ELF AT_CLKTCK addition must publish its exact marker before removal",
    )?;
    let platform_stack_construction_name = parse_static_image_stack_platform_transition(
        source,
        add_stack_platform,
        allocator_context,
        &clock_tick_rate_stack_construction_name,
        &stack_backing_name,
        true,
    )?;
    let stack_platform_marker = parse_static_image_marker(
        source,
        stack_platform_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_AUXV_PLATFORM_OK",
        "static ELF AT_PLATFORM addition must publish its exact marker before removal",
    )?;
    let hardware_capabilities_stack_construction_name =
        parse_static_image_stack_hardware_capabilities_transition(
            source,
            add_stack_hardware_capabilities,
            allocator_context,
            &platform_stack_construction_name,
            &stack_backing_name,
            true,
        )?;
    let stack_hardware_capabilities_marker = parse_static_image_marker(
        source,
        stack_hardware_capabilities_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_AUXV_HWCAP_OK",
        "static ELF hardware-capability addition must publish its exact marker before removal",
    )?;
    let load_metadata_stack_construction_name = parse_static_image_stack_load_metadata_transition(
        source,
        add_stack_load_metadata,
        allocator_context,
        &hardware_capabilities_stack_construction_name,
        &stack_backing_name,
        true,
    )?;
    let stack_load_metadata_marker = parse_static_image_marker(
        source,
        stack_load_metadata_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_AUXV_LOAD_METADATA_OK",
        "static ELF load-metadata addition must publish its exact marker before removal",
    )?;
    let random_slot_authorized_stack_construction_name =
        parse_static_image_stack_random_slot_transition(
            source,
            authorize_stack_random_slot,
            allocator_context,
            &load_metadata_stack_construction_name,
            &stack_backing_name,
            true,
        )?;
    let stack_random_slot_authorized_marker = parse_static_image_marker(
        source,
        stack_random_slot_authorized_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_RANDOM_SLOT_AUTHORIZED_OK",
        "static ELF random-slot authorization must publish its exact marker before revocation",
    )?;
    let retained_process_name = parse_static_image_prepared_process_transition(
        source,
        retain_prepared_process,
        allocator_context,
        &random_slot_authorized_stack_construction_name,
        Some(&stack_backing_name),
        true,
    )?;
    let prepared_process_retained_marker = parse_static_image_marker(
        source,
        prepared_process_retained_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_PREPARED_PROCESS_RETAINED_OK",
        "static ELF prepared-process retention must publish its exact marker before restoration",
    )?;
    let retained_process_after_disjoint_work_name = parse_static_image_retained_disjoint_work(
        source,
        qualify_retained_disjoint_work,
        allocator_context,
        &retained_process_name,
    )?;
    let retained_disjoint_work_marker = parse_static_image_marker(
        source,
        retained_disjoint_work_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_RETAINED_DISJOINT_WORK_OK",
        "static ELF retained disjoint work must publish its exact marker before restoration",
    )?;
    let entropy_pending_process_name = parse_static_image_entropy_handoff(
        source,
        authorize_entropy_handoff,
        allocator_context,
        &retained_process_after_disjoint_work_name,
        true,
    )?;
    let entropy_handoff_authorized_marker = parse_static_image_marker(
        source,
        entropy_handoff_authorized_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ENTROPY_HANDOFF_AUTHORIZED_OK",
        "static ELF entropy handoff authorization must publish its exact marker before revocation",
    )?;
    let entropy_acquiring_process_name = parse_static_image_entropy_acquisition(
        source,
        authorize_entropy_acquisition,
        allocator_context,
        &entropy_pending_process_name,
        true,
    )?;
    let entropy_acquisition_authorized_marker = parse_static_image_marker(
        source,
        entropy_acquisition_authorized_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ENTROPY_ACQUISITION_AUTHORIZED_OK",
        "static ELF entropy acquisition authorization must publish its exact marker before revocation",
    )?;
    let entropy_acquiring_process_after_device_name = parse_static_image_entropy_device(
        source,
        qualify_entropy_device,
        allocator_context,
        &entropy_acquiring_process_name,
    )?;
    let entropy_device_marker = parse_static_image_marker(
        source,
        entropy_device_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ENTROPY_DEVICE_OK",
        "static ELF entropy-device qualification must publish its exact marker before acquisition revocation",
    )?;
    let entropy_acquiring_process_after_transport_name = parse_static_image_entropy_transport(
        source,
        qualify_entropy_transport,
        allocator_context,
        &entropy_acquiring_process_after_device_name,
    )?;
    let entropy_transport_marker = parse_static_image_marker(
        source,
        entropy_transport_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ENTROPY_TRANSPORT_OK",
        "static ELF entropy-transport qualification must publish its exact marker before acquisition revocation",
    )?;
    let entropy_acquiring_process_after_negotiation_name = parse_static_image_entropy_negotiation(
        source,
        qualify_entropy_negotiation,
        allocator_context,
        &entropy_acquiring_process_after_transport_name,
    )?;
    let entropy_negotiation_marker = parse_static_image_marker(
        source,
        entropy_negotiation_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ENTROPY_NEGOTIATION_OK",
        "static ELF entropy-transport negotiation must publish its exact marker before acquisition revocation",
    )?;
    let entropy_acquiring_process_after_queue_name = parse_static_image_entropy_queue(
        source,
        qualify_entropy_queue,
        allocator_context,
        &entropy_acquiring_process_after_negotiation_name,
    )?;
    let entropy_queue_marker = parse_static_image_marker(
        source,
        entropy_queue_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_OK",
        "static ELF entropy-queue discovery must publish its exact marker before acquisition revocation",
    )?;
    let entropy_acquiring_process_after_bus_master_name = parse_static_image_entropy_bus_master(
        source,
        qualify_entropy_bus_master,
        allocator_context,
        &entropy_acquiring_process_after_queue_name,
    )?;
    let entropy_bus_master_marker = parse_static_image_marker(
        source,
        entropy_bus_master_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ENTROPY_BUS_MASTER_OK",
        "static ELF entropy bus-master qualification must publish its exact marker before acquisition revocation",
    )?;
    let entropy_acquiring_process_after_dma_buffers_name = parse_static_image_entropy_dma_buffers(
        source,
        qualify_entropy_dma_buffers,
        allocator_context,
        &entropy_acquiring_process_after_bus_master_name,
    )?;
    let entropy_dma_buffers_marker = parse_static_image_marker(
        source,
        entropy_dma_buffers_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ENTROPY_DMA_BUFFERS_OK",
        "static ELF entropy DMA-buffer coexistence must publish its exact marker before acquisition revocation",
    )?;
    let entropy_acquiring_process_after_split_ring_name = parse_static_image_entropy_split_ring(
        source,
        qualify_entropy_split_ring,
        allocator_context,
        &entropy_acquiring_process_after_dma_buffers_name,
    )?;
    let entropy_split_ring_marker = parse_static_image_marker(
        source,
        entropy_split_ring_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ENTROPY_SPLIT_RING_OK",
        "static ELF entropy split-ring coexistence must publish its exact marker before acquisition revocation",
    )?;
    let entropy_acquiring_process_after_dma_preparation_name =
        parse_static_image_entropy_dma_preparation(
            source,
            qualify_entropy_dma_preparation,
            allocator_context,
            &entropy_acquiring_process_after_split_ring_name,
        )?;
    let entropy_dma_preparation_marker = parse_static_image_marker(
        source,
        entropy_dma_preparation_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ENTROPY_DMA_PREPARATION_OK",
        "static ELF entropy DMA-preparation coexistence must publish its exact marker before acquisition revocation",
    )?;
    let entropy_acquiring_process_after_queue_addresses_name =
        parse_static_image_entropy_queue_addresses(
            source,
            qualify_entropy_queue_addresses,
            allocator_context,
            &entropy_acquiring_process_after_dma_preparation_name,
        )?;
    let entropy_queue_addresses_marker = parse_static_image_marker(
        source,
        entropy_queue_addresses_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_ADDRESSES_OK",
        "static ELF entropy queue-address coexistence must publish its exact marker before acquisition revocation",
    )?;
    let entropy_acquiring_process_after_queue_activation_name =
        parse_static_image_entropy_queue_activation(
            source,
            qualify_entropy_queue_activation,
            allocator_context,
            &entropy_acquiring_process_after_queue_addresses_name,
        )?;
    let entropy_queue_activation_marker = parse_static_image_marker(
        source,
        entropy_queue_activation_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ENTROPY_QUEUE_ACTIVATION_OK",
        "static ELF entropy queue-activation coexistence must publish its exact marker before acquisition revocation",
    )?;
    let entropy_acquiring_process_after_request_completion_name =
        parse_static_image_entropy_request_completion(
            source,
            qualify_entropy_request_completion,
            allocator_context,
            &entropy_acquiring_process_after_queue_activation_name,
        )?;
    let entropy_request_completion_marker = parse_static_image_marker(
        source,
        entropy_request_completion_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ENTROPY_REQUEST_COMPLETION_OK",
        "static ELF entropy request-completion coexistence must publish its exact marker before acquisition revocation",
    )?;
    let entropy_acquiring_process_after_sample_capture_name =
        parse_static_image_entropy_sample_capture(
            source,
            qualify_entropy_sample_capture,
            allocator_context,
            &entropy_acquiring_process_after_request_completion_name,
        )?;
    let entropy_sample_capture_marker = parse_static_image_marker(
        source,
        entropy_sample_capture_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ENTROPY_SAMPLE_CAPTURE_OK",
        "static ELF entropy-sample capture coexistence must publish its exact marker before acquisition revocation",
    )?;
    let entropy_acquiring_process_after_sample_quality_name =
        parse_static_image_entropy_sample_quality(
            source,
            qualify_entropy_sample_quality,
            allocator_context,
            &entropy_acquiring_process_after_sample_capture_name,
        )?;
    let entropy_sample_quality_marker = parse_static_image_marker(
        source,
        entropy_sample_quality_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ENTROPY_SAMPLE_QUALITY_OK",
        "static ELF entropy-sample quality coexistence must publish its exact marker before acquisition revocation",
    )?;
    let entropy_acquiring_process_after_pool_lifecycle_name =
        parse_static_image_entropy_pool_lifecycle(
            source,
            qualify_entropy_pool_lifecycle,
            allocator_context,
            &entropy_acquiring_process_after_sample_quality_name,
        )?;
    let entropy_pool_lifecycle_marker = parse_static_image_marker(
        source,
        entropy_pool_lifecycle_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ENTROPY_POOL_LIFECYCLE_OK",
        "static ELF entropy-pool lifecycle coexistence must publish its exact marker before acquisition revocation",
    )?;
    let entropy_acquiring_process_after_generator_lifecycle_name =
        parse_static_image_entropy_generator_lifecycle(
            source,
            qualify_entropy_generator_lifecycle,
            allocator_context,
            &entropy_acquiring_process_after_pool_lifecycle_name,
        )?;
    let entropy_generator_lifecycle_marker = parse_static_image_marker(
        source,
        entropy_generator_lifecycle_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ENTROPY_GENERATOR_LIFECYCLE_OK",
        "static ELF entropy-generator lifecycle coexistence must publish its exact marker before acquisition revocation",
    )?;
    let randomized_process_name = parse_static_image_entropy_initial_random_delivery(
        source,
        deliver_entropy_initial_random,
        allocator_context,
        &entropy_acquiring_process_after_generator_lifecycle_name,
    )?;
    let entropy_initial_random_delivered_marker = parse_static_image_marker(
        source,
        entropy_initial_random_delivered_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_INITIAL_RANDOM_DELIVERED_OK",
        "static ELF initial-random delivery must publish its exact marker before qualification revocation",
    )?;
    let completed_process_name = parse_static_image_process_entry(
        source,
        enter_static_process,
        allocator_context,
        &randomized_process_name,
        true,
    )?;
    let static_process_initial_brk_query_marker = parse_static_image_marker(
        source,
        static_process_initial_brk_query_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_INITIAL_BRK_QUERY_OK",
        "static ELF process entry must publish its exact initial-brk-query marker before its exit marker",
    )?;
    let static_process_initial_brk_growth_marker = parse_static_image_marker(
        source,
        static_process_initial_brk_growth_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_INITIAL_BRK_GROWTH_OK",
        "static ELF process entry must publish its exact initial-brk-growth marker after its query marker and before its exit marker",
    )?;
    let static_process_initial_x64_fs_base_marker = parse_static_image_marker(
        source,
        static_process_initial_x64_fs_base_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_INITIAL_X64_FS_BASE_OK",
        "static ELF process entry must publish its exact initial-x64-FS-base marker after growth and before its exit marker",
    )?;
    let static_process_initial_clear_tid_marker = parse_static_image_marker(
        source,
        static_process_initial_clear_tid_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_INITIAL_CLEAR_TID_OK",
        "static ELF process entry must publish its exact initial-clear-TID marker after FS installation and before its exit marker",
    )?;
    let static_process_initial_empty_robust_list_marker = parse_static_image_marker(
        source,
        static_process_initial_empty_robust_list_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_INITIAL_EMPTY_ROBUST_LIST_OK",
        "static ELF process entry must publish its exact initial-empty-robust-list marker after clear-TID registration and before its exit marker",
    )?;
    let static_process_initial_rseq_fallback_marker = parse_static_image_marker(
        source,
        static_process_initial_rseq_fallback_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_INITIAL_RSEQ_FALLBACK_OK",
        "static ELF process entry must publish its exact initial-rseq-fallback marker after robust-list registration and before its exit marker",
    )?;
    let static_process_initial_stack_limit_marker = parse_static_image_marker(
        source,
        static_process_initial_stack_limit_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_INITIAL_STACK_LIMIT_OK",
        "static ELF process entry must publish its exact initial-stack-limit marker after rseq fallback and before its exit marker",
    )?;
    let static_process_initial_readlinkat_fallback_marker = parse_static_image_marker(
        source,
        static_process_initial_readlinkat_fallback_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_INITIAL_READLINKAT_FALLBACK_OK",
        "static ELF process entry must publish its exact initial-readlinkat-fallback marker after the stack-limit query and before its exit marker",
    )?;
    let static_process_initial_getrandom_marker = parse_static_image_marker(
        source,
        static_process_initial_getrandom_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_INITIAL_GETRANDOM_OK",
        "static ELF process entry must publish its exact initial-getrandom marker after the readlinkat fallback and before its exit marker",
    )?;
    let static_process_reusable_getrandom_marker = parse_static_image_marker(
        source,
        static_process_reusable_getrandom_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_REUSABLE_GETRANDOM_OK",
        "static ELF process entry must publish its exact reusable-getrandom marker after the initial request and before its exit marker",
    )?;
    let static_process_post_random_brk_query_marker = parse_static_image_marker(
        source,
        static_process_post_random_brk_query_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_QUERY_OK",
        "static ELF process entry must publish its exact post-random-brk-query marker after reusable getrandom and before its exit marker",
    )?;
    let static_process_post_random_brk_growth_marker = parse_static_image_marker(
        source,
        static_process_post_random_brk_growth_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_GROWTH_OK",
        "static ELF process entry must publish its exact post-random-brk-growth marker after the current-break query and before its exit marker",
    )?;
    let static_process_post_random_brk_page_alignment_marker = parse_static_image_marker(
        source,
        static_process_post_random_brk_page_alignment_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_POST_RANDOM_BRK_PAGE_ALIGNMENT_OK",
        "static ELF process entry must publish its exact post-random-brk-page-alignment marker after cross-boundary growth and before its exit marker",
    )?;
    let static_process_relro_mprotect_marker = parse_static_image_marker(
        source,
        static_process_relro_mprotect_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_RELRO_MPROTECT_OK",
        "static ELF process entry must publish its exact RELRO-mprotect marker after break alignment and before its exit marker",
    )?;
    let static_process_initial_process_name_marker = parse_static_image_marker(
        source,
        static_process_initial_process_name_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_INITIAL_PROCESS_NAME_OK",
        "static ELF process entry must publish its exact initial-process-name marker after RELRO confirmation and before its exit marker",
    )?;
    let static_process_initial_getuid_marker = parse_static_image_marker(
        source,
        static_process_initial_getuid_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_INITIAL_GETUID_OK",
        "static ELF process entry must publish its exact initial-getuid marker after the process-name marker and before its exit marker",
    )?;
    let static_process_initial_config_stat_profile_marker = parse_static_image_marker(
        source,
        static_process_initial_config_stat_profile_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_INITIAL_CONFIG_STAT_PROFILE_OK",
        "static ELF process entry must publish its exact initial-config-stat-profile marker after the getuid marker and before its exit marker",
    )?;
    let static_process_initial_credential_tail_profile_marker = parse_static_image_marker(
        source,
        static_process_initial_credential_tail_profile_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_INITIAL_CREDENTIAL_TAIL_PROFILE_OK",
        "static ELF process entry must publish its exact initial-credential-tail-profile marker after the configuration-stat marker and before its exit marker",
    )?;
    let static_process_exited_marker = parse_static_image_marker(
        source,
        static_process_exited_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_PROCESS_EXITED_OK",
        "static ELF process entry must publish its exact marker before qualification restoration",
    )?;
    let randomized_process_after_entry_restoration_name = parse_static_image_process_entry(
        source,
        restore_static_process_entry,
        allocator_context,
        &completed_process_name,
        false,
    )?;
    let static_process_entry_restored_marker = parse_static_image_marker(
        source,
        static_process_entry_restored_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_PROCESS_ENTRY_RESTORED_OK",
        "static ELF process-entry restoration must publish its exact marker before initial-random revocation",
    )?;
    let entropy_acquiring_process_after_initial_random_revocation_name =
        parse_static_image_entropy_initial_random_revocation(
            source,
            revoke_entropy_initial_random,
            allocator_context,
            &randomized_process_after_entry_restoration_name,
        )?;
    let entropy_initial_random_revoked_marker = parse_static_image_marker(
        source,
        entropy_initial_random_revoked_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_INITIAL_RANDOM_REVOKED_OK",
        "static ELF initial-random qualification revocation must publish its exact marker before acquisition revocation",
    )?;
    let entropy_pending_process_after_acquisition_rollback_name =
        parse_static_image_entropy_acquisition(
            source,
            revoke_entropy_acquisition,
            allocator_context,
            &entropy_acquiring_process_after_initial_random_revocation_name,
            false,
        )?;
    let entropy_acquisition_revoked_marker = parse_static_image_marker(
        source,
        entropy_acquisition_revoked_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ENTROPY_ACQUISITION_REVOKED_OK",
        "static ELF entropy acquisition revocation must publish its exact marker before handoff revocation",
    )?;
    let retained_process_after_entropy_rollback_name = parse_static_image_entropy_handoff(
        source,
        revoke_entropy_handoff,
        allocator_context,
        &entropy_pending_process_after_acquisition_rollback_name,
        false,
    )?;
    let entropy_handoff_revoked_marker = parse_static_image_marker(
        source,
        entropy_handoff_revoked_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ENTROPY_HANDOFF_REVOKED_OK",
        "static ELF entropy handoff revocation must publish its exact marker before restoration",
    )?;
    let random_slot_authorized_stack_restored_construction_name =
        parse_static_image_prepared_process_transition(
            source,
            restore_prepared_process,
            allocator_context,
            &retained_process_after_entropy_rollback_name,
            None,
            false,
        )?;
    let prepared_process_restored_marker = parse_static_image_marker(
        source,
        prepared_process_restored_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_PREPARED_PROCESS_RESTORED_OK",
        "static ELF prepared-process restoration must publish its exact marker before random-slot revocation",
    )?;
    let load_metadata_stack_restored_construction_name =
        parse_static_image_stack_random_slot_transition(
            source,
            revoke_stack_random_slot,
            allocator_context,
            &random_slot_authorized_stack_restored_construction_name,
            &stack_backing_name,
            false,
        )?;
    let stack_random_slot_revoked_marker = parse_static_image_marker(
        source,
        stack_random_slot_revoked_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_RANDOM_SLOT_REVOKED_OK",
        "static ELF random-slot revocation must publish its exact marker before load-metadata removal",
    )?;
    let hardware_capabilities_stack_restored_construction_name =
        parse_static_image_stack_load_metadata_transition(
            source,
            remove_stack_load_metadata,
            allocator_context,
            &load_metadata_stack_restored_construction_name,
            &stack_backing_name,
            false,
        )?;
    let stack_load_metadata_revoked_marker = parse_static_image_marker(
        source,
        stack_load_metadata_revoked_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_AUXV_LOAD_METADATA_REVOKED_OK",
        "static ELF load-metadata removal must publish its exact marker before hardware-capability removal",
    )?;
    let platform_stack_restored_construction_name =
        parse_static_image_stack_hardware_capabilities_transition(
            source,
            remove_stack_hardware_capabilities,
            allocator_context,
            &hardware_capabilities_stack_restored_construction_name,
            &stack_backing_name,
            false,
        )?;
    let stack_hardware_capabilities_revoked_marker = parse_static_image_marker(
        source,
        stack_hardware_capabilities_revoked_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_AUXV_HWCAP_REVOKED_OK",
        "static ELF hardware-capability removal must publish its exact marker before AT_PLATFORM removal",
    )?;
    let clock_tick_rate_stack_restored_construction_name =
        parse_static_image_stack_platform_transition(
            source,
            remove_stack_platform,
            allocator_context,
            &platform_stack_restored_construction_name,
            &stack_backing_name,
            false,
        )?;
    let stack_platform_revoked_marker = parse_static_image_marker(
        source,
        stack_platform_revoked_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_AUXV_PLATFORM_REVOKED_OK",
        "static ELF AT_PLATFORM removal must publish its exact marker before AT_CLKTCK removal",
    )?;
    let secure_mode_stack_restored_construction_name =
        parse_static_image_stack_clock_tick_rate_transition(
            source,
            remove_stack_clock_tick_rate,
            allocator_context,
            &clock_tick_rate_stack_restored_construction_name,
            &stack_backing_name,
            false,
        )?;
    let stack_clock_tick_rate_revoked_marker = parse_static_image_marker(
        source,
        stack_clock_tick_rate_revoked_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_AUXV_CLKTCK_REVOKED_OK",
        "static ELF AT_CLKTCK removal must publish its exact marker before AT_SECURE removal",
    )?;
    let credential_stack_restored_construction_name =
        parse_static_image_stack_secure_mode_transition(
            source,
            remove_stack_secure_mode,
            allocator_context,
            &secure_mode_stack_restored_construction_name,
            &stack_backing_name,
            false,
        )?;
    let stack_secure_mode_revoked_marker = parse_static_image_marker(
        source,
        stack_secure_mode_revoked_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_AUXV_SECURE_REVOKED_OK",
        "static ELF AT_SECURE removal must publish its exact marker before credential removal",
    )?;
    let executable_name_stack_restored_construction_name =
        parse_static_image_stack_credentials_transition(
            source,
            remove_stack_credentials,
            allocator_context,
            &credential_stack_restored_construction_name,
            &stack_backing_name,
            false,
        )?;
    let stack_credentials_revoked_marker = parse_static_image_marker(
        source,
        stack_credentials_revoked_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_AUXV_CREDENTIALS_REVOKED_OK",
        "static ELF credential removal must publish its exact marker before AT_EXECFN removal",
    )?;
    let program_header_stack_restored_construction_name =
        parse_static_image_stack_executable_name_transition(
            source,
            remove_stack_executable_name,
            allocator_context,
            &executable_name_stack_restored_construction_name,
            &stack_backing_name,
            false,
        )?;
    let stack_executable_name_revoked_marker = executable_name_revoked_marker;
    let stack_executable_name_revoked_marker = parse_static_image_marker(
        source,
        stack_executable_name_revoked_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_AUXV_EXECFN_REVOKED_OK",
        "static ELF AT_EXECFN removal must publish its exact marker before program-header removal",
    )?;
    let entry_fact_stack_restored_construction_name =
        parse_static_image_stack_program_headers_removal(
            source,
            remove_stack_program_headers,
            allocator_context,
            &program_header_stack_restored_construction_name,
            &stack_backing_name,
        )?;
    let stack_program_headers_revoked_marker = parse_static_image_marker(
        source,
        stack_program_headers_revoked_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_AUXV_PROGRAM_HEADERS_REVOKED_OK",
        "static ELF program-header removal must publish its exact marker before AT_ENTRY removal",
    )?;
    let pagesz_stack_entry_restored_construction_name =
        parse_static_image_stack_entry_fact_removal(
            source,
            remove_stack_entry_fact,
            allocator_context,
            &entry_fact_stack_restored_construction_name,
            &stack_backing_name,
        )?;
    let stack_entry_fact_revoked_marker = parse_static_image_marker(
        source,
        stack_entry_fact_revoked_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_AUXV_ENTRY_REVOKED_OK",
        "static ELF AT_ENTRY removal must publish its exact marker before AT_PAGESZ removal",
    )?;
    let initialized_stack_entry_restored_construction_name =
        parse_static_image_stack_pagesz_removal(
            source,
            remove_stack_pagesz,
            allocator_context,
            &pagesz_stack_entry_restored_construction_name,
            &stack_backing_name,
        )?;
    let stack_pagesz_revoked_marker = parse_static_image_marker(
        source,
        stack_pagesz_revoked_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_AUXV_PAGESZ_REVOKED_OK",
        "static ELF AT_PAGESZ removal must publish its exact marker before stack-entry clearing",
    )?;
    let published_stack_leaf_restored_construction_name = parse_static_image_stack_entry_clearing(
        source,
        clear_stack_entry,
        allocator_context,
        &initialized_stack_entry_restored_construction_name,
        &stack_backing_name,
    )?;
    let stack_entry_cleared_marker = parse_static_image_marker(
        source,
        stack_entry_cleared_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_CLEARED_OK",
        "static ELF stack-entry clearing must publish its exact marker before leaf revocation",
    )?;
    let zeroed_stack_backing_restored_construction_name = parse_static_image_stack_leaf_revocation(
        source,
        revoke_stack_leaf,
        allocator_context,
        &published_stack_leaf_restored_construction_name,
        &stack_backing_name,
    )?;
    let stack_leaf_revoked_marker = parse_static_image_marker(
        source,
        stack_leaf_revoked_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_REVOKED_OK",
        "static ELF stack-leaf revocation must publish its exact marker before backing release",
    )?;
    parse_physical_frame_release(
        source,
        stack_backing_release,
        allocator_context,
        &stack_backing_name,
    )?;
    let stack_slot_restored_construction_name = parse_static_image_stack_slot_transition(
        source,
        revoke_stack_slot,
        allocator_context,
        &zeroed_stack_backing_restored_construction_name,
        "revoke",
        "bounded-stack-slot-qualification-rollback",
    )?;
    let stack_slot_revoked_marker = parse_static_image_marker(
        source,
        stack_slot_revoked_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_STACK_SLOT_AUTHORITY_REVOKED_OK",
        "static ELF stack-slot revocation must publish its exact marker before entry revocation",
    )?;
    let entry_restored_construction_name = parse_static_image_entry_transition(
        source,
        revoke_entry,
        allocator_context,
        &stack_slot_restored_construction_name,
        "revoke",
        "bounded-entry-qualification-rollback",
    )?;
    let entry_revoked_marker = parse_static_image_marker(
        source,
        entry_revoked_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ENTRY_AUTHORITY_REVOKED_OK",
        "static ELF entry revocation must publish its exact marker before detachment",
    )?;
    let detached_construction_name = parse_static_image_address_space_detach(
        source,
        detach,
        allocator_context,
        &entry_restored_construction_name,
    )?;
    let detach_marker = parse_static_image_marker(
        source,
        detach_marker,
        allocator_context,
        "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_DETACHED_OK",
        "static ELF address-space detachment must publish its exact marker before hierarchy reclaim",
    )?;
    let (returned_hierarchy, after_hierarchy_reclaim) = parse_static_image_resource_return(
        source,
        hierarchy_reclaim,
        allocator_context,
        &detached_construction_name,
        "reclaim",
        "hierarchy",
    )?;
    if !after_hierarchy_reclaim.is_empty() {
        return Err(translation_edit_diagnostic(
            source,
            statement_span(hierarchy_reclaim),
            "static ELF hierarchy reclamation must return exactly one affine resource",
        ));
    }
    parse_physical_frame_release(
        source,
        hierarchy_release,
        allocator_context,
        &returned_hierarchy,
    )?;
    let (returned_metadata, after_revoke) = parse_static_image_resource_return(
        source,
        revoke,
        allocator_context,
        &detached_construction_name,
        "revoke",
        "metadata",
    )?;
    if !after_revoke.is_empty() {
        return Err(translation_edit_diagnostic(
            source,
            statement_span(revoke),
            "static ELF revocation must return exactly one affine resource",
        ));
    }
    parse_physical_frame_release(
        source,
        metadata_release,
        allocator_context,
        &returned_metadata,
    )?;
    let (returned_payload, after_reclaim) = parse_static_image_resource_return(
        source,
        reclaim,
        allocator_context,
        &detached_construction_name,
        "reclaim",
        "payload",
    )?;
    if !after_reclaim.is_empty() {
        return Err(translation_edit_diagnostic(
            source,
            statement_span(reclaim),
            "static ELF payload reclamation must return exactly one affine resource",
        ));
    }
    parse_physical_frame_release(
        source,
        payload_release,
        allocator_context,
        &returned_payload,
    )?;
    let mut checked =
        analyze_local_interrupt_critical_sequence(source, rest, allocator_context, storage)?;
    let mut operations = vec![
        CompilerSystemsOperation::AllocatePhysicalFrames {
            request,
            failure_message,
        },
        marker,
        CompilerSystemsOperation::AllocatePhysicalFrames {
            request: metadata_request,
            failure_message: metadata_failure_message,
        },
        resources_marker,
        CompilerSystemsOperation::PrepareStaticImageConstruction {
            failure_message: construction_failure_message,
        },
        leaves_marker,
        CompilerSystemsOperation::ZeroStaticImageConstructionPayload,
        zero_marker,
        CompilerSystemsOperation::CopyStaticImageConstructionPayload,
        copy_marker,
        CompilerSystemsOperation::SealStaticImageConstructionPermissions,
        seal_marker,
        CompilerSystemsOperation::AllocatePhysicalFrames {
            request: CompilerPhysicalFrameRequest {
                frame_count: 5,
                alignment_frames: 1,
            },
            failure_message: "static ELF hierarchy frame reservation failed".into(),
        },
        hierarchy_marker,
        CompilerSystemsOperation::AttachStaticImageAddressSpace,
        attach_marker,
        CompilerSystemsOperation::ActivateStaticImageAddressSpace,
        active_marker,
        CompilerSystemsOperation::RestoreStaticImageAddressSpace,
        restored_marker,
        CompilerSystemsOperation::AuthorizeStaticImageEntry,
        entry_marker,
        CompilerSystemsOperation::AuthorizeStaticImageStackSlot,
        stack_slot_marker,
        CompilerSystemsOperation::AllocatePhysicalFrames {
            request: CompilerPhysicalFrameRequest {
                frame_count: 2,
                alignment_frames: 1,
            },
            failure_message: "static ELF stack-backing frame reservation failed".into(),
        },
        stack_backing_marker,
        CompilerSystemsOperation::ZeroStaticImageStackBacking,
        stack_backing_zeroed_marker,
        CompilerSystemsOperation::PublishStaticImageStackLeaf,
        stack_leaf_published_marker,
        CompilerSystemsOperation::InitializeStaticImageStackEntry,
        stack_entry_initialized_marker,
        CompilerSystemsOperation::AddStaticImageStackPageSize,
        stack_pagesz_marker,
        CompilerSystemsOperation::AddStaticImageStackEntryFact,
        stack_entry_fact_marker,
        CompilerSystemsOperation::AddStaticImageStackProgramHeaders,
        stack_program_headers_marker,
        CompilerSystemsOperation::AddStaticImageStackExecutableName,
        stack_executable_name_marker,
        CompilerSystemsOperation::AddStaticImageStackCredentials,
        stack_credentials_marker,
        CompilerSystemsOperation::AddStaticImageStackSecureMode,
        stack_secure_mode_marker,
        CompilerSystemsOperation::AddStaticImageStackClockTickRate,
        stack_clock_tick_rate_marker,
        CompilerSystemsOperation::AddStaticImageStackPlatform,
        stack_platform_marker,
        CompilerSystemsOperation::AddStaticImageStackHardwareCapabilities,
        stack_hardware_capabilities_marker,
        CompilerSystemsOperation::AddStaticImageStackLoadMetadata,
        stack_load_metadata_marker,
        CompilerSystemsOperation::AuthorizeStaticImageStackRandomSlot,
        stack_random_slot_authorized_marker,
        CompilerSystemsOperation::RetainStaticImagePreparedProcess,
        prepared_process_retained_marker,
        CompilerSystemsOperation::QualifyStaticImageRetainedDisjointWork,
        retained_disjoint_work_marker,
        CompilerSystemsOperation::AuthorizeStaticImageEntropyHandoff,
        entropy_handoff_authorized_marker,
        CompilerSystemsOperation::AuthorizeStaticImageEntropyAcquisition,
        entropy_acquisition_authorized_marker,
        CompilerSystemsOperation::QualifyStaticImageEntropyDevice,
        entropy_device_marker,
        CompilerSystemsOperation::QualifyStaticImageEntropyTransport,
        entropy_transport_marker,
        CompilerSystemsOperation::QualifyStaticImageEntropyNegotiation,
        entropy_negotiation_marker,
        CompilerSystemsOperation::QualifyStaticImageEntropyQueue,
        entropy_queue_marker,
        CompilerSystemsOperation::QualifyStaticImageEntropyBusMaster,
        entropy_bus_master_marker,
        CompilerSystemsOperation::QualifyStaticImageEntropyDmaBuffers,
        entropy_dma_buffers_marker,
        CompilerSystemsOperation::QualifyStaticImageEntropySplitRing,
        entropy_split_ring_marker,
        CompilerSystemsOperation::QualifyStaticImageEntropyDmaPreparation,
        entropy_dma_preparation_marker,
        CompilerSystemsOperation::QualifyStaticImageEntropyQueueAddresses,
        entropy_queue_addresses_marker,
        CompilerSystemsOperation::QualifyStaticImageEntropyQueueActivation,
        entropy_queue_activation_marker,
        CompilerSystemsOperation::QualifyStaticImageEntropyRequestCompletion,
        entropy_request_completion_marker,
        CompilerSystemsOperation::QualifyStaticImageEntropySampleCapture,
        entropy_sample_capture_marker,
        CompilerSystemsOperation::QualifyStaticImageEntropySampleQuality,
        entropy_sample_quality_marker,
        CompilerSystemsOperation::QualifyStaticImageEntropyPoolLifecycle,
        entropy_pool_lifecycle_marker,
        CompilerSystemsOperation::QualifyStaticImageEntropyGeneratorLifecycle,
        entropy_generator_lifecycle_marker,
        CompilerSystemsOperation::DeliverStaticImageEntropyInitialRandom,
        entropy_initial_random_delivered_marker,
        CompilerSystemsOperation::EnterStaticImageProcess,
        static_process_initial_brk_query_marker,
        static_process_initial_brk_growth_marker,
        static_process_initial_x64_fs_base_marker,
        static_process_initial_clear_tid_marker,
        static_process_initial_empty_robust_list_marker,
        static_process_initial_rseq_fallback_marker,
        static_process_initial_stack_limit_marker,
        static_process_initial_readlinkat_fallback_marker,
        static_process_initial_getrandom_marker,
        static_process_reusable_getrandom_marker,
        static_process_post_random_brk_query_marker,
        static_process_post_random_brk_growth_marker,
        static_process_post_random_brk_page_alignment_marker,
        static_process_relro_mprotect_marker,
        static_process_initial_process_name_marker,
        static_process_initial_getuid_marker,
        static_process_initial_config_stat_profile_marker,
        static_process_initial_credential_tail_profile_marker,
        static_process_exited_marker,
        CompilerSystemsOperation::RestoreStaticImageProcessEntry,
        static_process_entry_restored_marker,
        CompilerSystemsOperation::RevokeStaticImageEntropyInitialRandom,
        entropy_initial_random_revoked_marker,
        CompilerSystemsOperation::RevokeStaticImageEntropyAcquisition,
        entropy_acquisition_revoked_marker,
        CompilerSystemsOperation::RevokeStaticImageEntropyHandoff,
        entropy_handoff_revoked_marker,
        CompilerSystemsOperation::RestoreStaticImagePreparedProcess,
        prepared_process_restored_marker,
        CompilerSystemsOperation::RevokeStaticImageStackRandomSlot,
        stack_random_slot_revoked_marker,
        CompilerSystemsOperation::RemoveStaticImageStackLoadMetadata,
        stack_load_metadata_revoked_marker,
        CompilerSystemsOperation::RemoveStaticImageStackHardwareCapabilities,
        stack_hardware_capabilities_revoked_marker,
        CompilerSystemsOperation::RemoveStaticImageStackPlatform,
        stack_platform_revoked_marker,
        CompilerSystemsOperation::RemoveStaticImageStackClockTickRate,
        stack_clock_tick_rate_revoked_marker,
        CompilerSystemsOperation::RemoveStaticImageStackSecureMode,
        stack_secure_mode_revoked_marker,
        CompilerSystemsOperation::RemoveStaticImageStackCredentials,
        stack_credentials_revoked_marker,
        CompilerSystemsOperation::RemoveStaticImageStackExecutableName,
        stack_executable_name_revoked_marker,
        CompilerSystemsOperation::RemoveStaticImageStackProgramHeaders,
        stack_program_headers_revoked_marker,
        CompilerSystemsOperation::RemoveStaticImageStackEntryFact,
        stack_entry_fact_revoked_marker,
        CompilerSystemsOperation::RemoveStaticImageStackPageSize,
        stack_pagesz_revoked_marker,
        CompilerSystemsOperation::ClearStaticImageStackEntry,
        stack_entry_cleared_marker,
        CompilerSystemsOperation::RevokeStaticImageStackLeaf,
        stack_leaf_revoked_marker,
        CompilerSystemsOperation::ReleasePhysicalFrames,
        CompilerSystemsOperation::RevokeStaticImageStackSlot,
        stack_slot_revoked_marker,
        CompilerSystemsOperation::RevokeStaticImageEntry,
        entry_revoked_marker,
        CompilerSystemsOperation::DetachStaticImageAddressSpace,
        detach_marker,
        CompilerSystemsOperation::ReclaimStaticImageConstructionHierarchy,
        CompilerSystemsOperation::ReleasePhysicalFrames,
        CompilerSystemsOperation::RevokeStaticImageConstructionMetadata,
        CompilerSystemsOperation::ReleasePhysicalFrames,
        CompilerSystemsOperation::ReclaimStaticImageConstructionPayload,
        CompilerSystemsOperation::ReleasePhysicalFrames,
    ];
    operations.append(&mut checked.operations);
    Ok(CheckedBootstrapRegionDecision {
        operations,
        disposition: checked.disposition,
    })
}

fn parse_static_image_marker(
    source: &SourceText,
    statement: &Statement,
    allocator_context: &str,
    expected: &str,
    failure_message: &str,
) -> Result<CompilerSystemsOperation, Diagnostic> {
    let span = statement_span(statement);
    let marker = analyze_operation(
        source,
        statement,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(&marker, CompilerSystemsOperation::ConsoleWrite { text } if text == expected) {
        return Err(translation_edit_diagnostic(source, span, failure_message));
    }
    Ok(marker)
}
