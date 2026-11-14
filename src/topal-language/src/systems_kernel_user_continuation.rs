// User backing, mappings, entropy and entry after worker completion checks.
// Kept in the parent systems namespace; diagnostics and operation order remain.
fn analyze_kernel_user_continuation(
    source: &SourceText,
    user_allocation: &[Statement],
    completion_resumed: String,
    mut operations: Vec<CompilerSystemsOperation>,
) -> Result<CheckedBootstrapRegionDecision, Diagnostic> {
    let (subject, user_frames, success, _problem_name, failure, span) =
        result_sequence(source, user_allocation, "user backing allocation")?;
    let backing_request = parse_physical_frame_allocation(source, subject, &completion_resumed)?;
    if backing_request
        != (CompilerPhysicalFrameRequest {
            frame_count: 3,
            alignment_frames: 1,
        })
    {
        return Err(user_context_diagnostic(
            source,
            subject.span(),
            "initial user backing requires exactly three frames aligned to one frame",
        ));
    }
    let CompilerSystemsDisposition::Fatal {
        message: backing_failure,
    } = action_disposition(source, failure, &completion_resumed)?
    else {
        return Err(user_context_diagnostic(
            source,
            failure.span(),
            "user backing allocation failure must preserve the allocator context and enter `fatal`",
        ));
    };
    let user_statements = action_block(source, success, "user backing allocation success")?;
    let [prepare_address_space] = user_statements else {
        return Err(user_context_diagnostic(
            source,
            span,
            "user backing success must begin with one affine user-address-space preparation scope",
        ));
    };
    let (user_space, address_space_continuation) = parse_user_address_space_prepare(
        source,
        prepare_address_space,
        &completion_resumed,
        &user_frames,
    )?;
    let (scratch_subject, scratch_frames, scratch_success, _scratch_problem, scratch_failure, _) =
        result_sequence(
            source,
            address_space_continuation,
            "disjoint frame allocation",
        )?;
    let scratch_request =
        parse_physical_frame_allocation(source, scratch_subject, &completion_resumed)?;
    if scratch_request
        != (CompilerPhysicalFrameRequest {
            frame_count: 2,
            alignment_frames: 1,
        })
    {
        return Err(user_context_diagnostic(
            source,
            scratch_subject.span(),
            "the disjoint executable extent requires exactly two frames aligned to one frame",
        ));
    }
    let CompilerSystemsDisposition::Fatal {
        message: scratch_failure_message,
    } = action_disposition(source, scratch_failure, &completion_resumed)?
    else {
        return Err(user_context_diagnostic(
            source,
            scratch_failure.span(),
            "disjoint frame allocation failure must preserve the allocator context and enter `fatal`",
        ));
    };
    let scratch_statements =
        action_block(source, scratch_success, "disjoint frame allocation success")?;
    let [map_mapping, scratch_after_map @ ..] = scratch_statements else {
        return Err(user_context_diagnostic(
            source,
            span,
            "disjoint frame allocation success must map, store, load, unmap, and release its exact extent before consuming the prepared user address space",
        ));
    };
    let (mapped_scratch, map_continuation) = parse_user_address_space_map(
        source,
        map_mapping,
        &completion_resumed,
        &user_space,
        &scratch_frames,
        "probe-code",
    )?;
    let after_map =
        select_user_mapping_continuation(source, span, scratch_after_map, map_continuation, "map")?;
    let [store_mapping, scratch_after_store @ ..] = after_map else {
        return Err(user_context_diagnostic(
            source,
            span,
            "code mapping publication must be followed by entry installation, protection, verification, and executable promotion",
        ));
    };
    let (stored_scratch, store_continuation) = parse_user_mapping_entry_trampoline(
        source,
        store_mapping,
        &completion_resumed,
        &mapped_scratch,
        "install",
    )?;
    let after_store = select_user_mapping_continuation(
        source,
        span,
        scratch_after_store,
        store_continuation,
        "store",
    )?;
    let [protect_mapping, scratch_after_protect @ ..] = after_store else {
        return Err(user_context_diagnostic(
            source,
            span,
            "entry-trampoline installation must be followed by protection, verification, and executable promotion",
        ));
    };
    let (protected_scratch, protect_continuation) = parse_user_mapping_protection(
        source,
        protect_mapping,
        &completion_resumed,
        &stored_scratch,
    )?;
    let after_protect = select_user_mapping_continuation(
        source,
        span,
        scratch_after_protect,
        protect_continuation,
        "protect",
    )?;
    let [load_mapping, scratch_after_load @ ..] = after_protect else {
        return Err(user_context_diagnostic(
            source,
            span,
            "protected code mapping must have its entry trampoline verified before executable promotion",
        ));
    };
    let (verified_scratch, load_continuation) = parse_user_mapping_entry_trampoline(
        source,
        load_mapping,
        &completion_resumed,
        &protected_scratch,
        "verify",
    )?;
    let after_load = select_user_mapping_continuation(
        source,
        span,
        scratch_after_load,
        load_continuation,
        "load",
    )?;
    let [execute_mapping, scratch_after_execute @ ..] = after_load else {
        return Err(user_context_diagnostic(
            source,
            span,
            "entry-trampoline verification must be followed by execute promotion",
        ));
    };
    let (executable_scratch, execute_continuation) = parse_user_mapping_execute_promotion(
        source,
        execute_mapping,
        &completion_resumed,
        &verified_scratch,
    )?;
    let after_execute = select_user_mapping_continuation(
        source,
        span,
        scratch_after_execute,
        execute_continuation,
        "execute promotion",
    )?;
    let (data_subject, data_frames, data_success, _data_problem, data_failure, data_span) =
        result_sequence(source, after_execute, "data frame allocation")?;
    let data_request = parse_physical_frame_allocation(source, data_subject, &completion_resumed)?;
    if data_request
        != (CompilerPhysicalFrameRequest {
            frame_count: 1,
            alignment_frames: 1,
        })
    {
        return Err(user_context_diagnostic(
            source,
            data_subject.span(),
            "resident immutable data requires exactly one separately allocated frame",
        ));
    }
    let CompilerSystemsDisposition::Fatal {
        message: data_failure_message,
    } = action_disposition(source, data_failure, &completion_resumed)?
    else {
        return Err(user_context_diagnostic(
            source,
            data_failure.span(),
            "data frame allocation failure must preserve the allocator context and enter `fatal`",
        ));
    };
    let data_statements = action_block(source, data_success, "data frame allocation success")?;
    let [map_mapping_again, after_second_map @ ..] = data_statements else {
        return Err(user_context_diagnostic(
            source,
            data_span,
            "data frame allocation success must construct and revoke immutable data before executable code",
        ));
    };
    let (mapped_scratch_again, map_again_continuation) = parse_user_address_space_map(
        source,
        map_mapping_again,
        &completion_resumed,
        &user_space,
        &data_frames,
        "probe-data",
    )?;
    let after_second_map = select_user_mapping_continuation(
        source,
        span,
        after_second_map,
        map_again_continuation,
        "second map",
    )?;
    let [store_mapping_again, after_second_store @ ..] = after_second_map else {
        return Err(user_context_diagnostic(
            source,
            span,
            "second user mapping publication must be followed by store, protect, load, and unmap",
        ));
    };
    let (stored_scratch_again, store_again_continuation) = parse_user_mapping_refinement(
        source,
        store_mapping_again,
        &completion_resumed,
        &mapped_scratch_again,
        "store",
    )?;
    let after_second_store = select_user_mapping_continuation(
        source,
        span,
        after_second_store,
        store_again_continuation,
        "second store",
    )?;
    let [protect_mapping_again, after_second_protect @ ..] = after_second_store else {
        return Err(user_context_diagnostic(
            source,
            span,
            "second user mapping store must be followed by protect, load, and unmap",
        ));
    };
    let (protected_scratch_again, protect_again_continuation) = parse_user_mapping_protection(
        source,
        protect_mapping_again,
        &completion_resumed,
        &stored_scratch_again,
    )?;
    let after_second_protect = select_user_mapping_continuation(
        source,
        span,
        after_second_protect,
        protect_again_continuation,
        "second protect",
    )?;
    let [load_mapping_again, after_second_load @ ..] = after_second_protect else {
        return Err(user_context_diagnostic(
            source,
            span,
            "second protected user mapping must be loaded before unmap",
        ));
    };
    let (verified_scratch_again, load_again_continuation) = parse_user_mapping_refinement(
        source,
        load_mapping_again,
        &completion_resumed,
        &protected_scratch_again,
        "load",
    )?;
    let after_second_load = select_user_mapping_continuation(
        source,
        span,
        after_second_load,
        load_again_continuation,
        "second load",
    )?;
    let [seal_mapping_again, after_second_seal @ ..] = after_second_load else {
        return Err(user_context_diagnostic(
            source,
            span,
            "second user mapping load must be followed by writable-data promotion and unmap",
        ));
    };
    let (writable_data_scratch_again, seal_again_continuation) =
        parse_user_mapping_writable_data_promotion(
            source,
            seal_mapping_again,
            &completion_resumed,
            &verified_scratch_again,
        )?;
    let after_second_seal = select_user_mapping_continuation(
        source,
        span,
        after_second_seal,
        seal_again_continuation,
        "second writable-data promotion",
    )?;
    let (
        anonymous_subject,
        anonymous_frames,
        anonymous_success,
        _anonymous_problem,
        anonymous_failure,
        anonymous_span,
    ) = result_sequence(source, after_second_seal, "anonymous frame allocation")?;
    let anonymous_request =
        parse_physical_frame_allocation(source, anonymous_subject, &completion_resumed)?;
    if anonymous_request
        != (CompilerPhysicalFrameRequest {
            frame_count: 34,
            alignment_frames: 1,
        })
    {
        return Err(user_context_diagnostic(
            source,
            anonymous_subject.span(),
            "the bounded anonymous reservation requires exactly 35 frames aligned to one frame",
        ));
    }
    let CompilerSystemsDisposition::Fatal {
        message: anonymous_failure_message,
    } = action_disposition(source, anonymous_failure, &completion_resumed)?
    else {
        return Err(user_context_diagnostic(
            source,
            anonymous_failure.span(),
            "anonymous frame allocation failure must preserve the allocator context and enter `fatal`",
        ));
    };
    let anonymous_statements = action_block(
        source,
        anonymous_success,
        "anonymous frame allocation success",
    )?;
    let [reserve_anonymous, anonymous_continuation @ ..] = anonymous_statements else {
        return Err(user_context_diagnostic(
            source,
            anonymous_span,
            "anonymous frame allocation success must reserve its exact frame before user entry",
        ));
    };
    let (anonymous_page, reserve_continuation) = parse_user_address_space_reserve_anonymous(
        source,
        reserve_anonymous,
        &completion_resumed,
        &user_space,
        &anonymous_frames,
    )?;
    let after_anonymous_reserve = select_user_mapping_continuation(
        source,
        anonymous_span,
        anonymous_continuation,
        reserve_continuation,
        "anonymous reservation",
    )?;
    let [
        image_ready_marker_statement,
        boot_resource_marker_statement,
        prepare_user,
        after_prepare_user @ ..,
    ] = after_anonymous_reserve
    else {
        return Err(user_context_diagnostic(
            source,
            span,
            "second writable-data user mapping must publish readiness before user preparation",
        ));
    };
    let image_ready_marker = analyze_operation(
        source,
        image_ready_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if image_ready_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_READY".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(image_ready_marker_statement),
            "resident user image requires its exact readiness marker before teardown",
        ));
    }
    let boot_resource_marker = analyze_operation(
        source,
        boot_resource_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if boot_resource_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_BOOT_RESOURCE_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(boot_resource_marker_statement),
            "initial user image requires its exact boot-resource marker before user preparation",
        ));
    }
    let (user_probe, probe_continuation) =
        parse_user_probe_prepare(source, prepare_user, &completion_resumed, &user_space)?;
    let user_continuation = match (after_prepare_user.is_empty(), probe_continuation.is_empty()) {
        (true, false) => probe_continuation,
        (false, true) => after_prepare_user,
        _ => {
            return Err(user_context_diagnostic(
                source,
                span,
                "prepared user address space must transfer exactly one continuation into its user probe",
            ));
        }
    };
    let [
        queue_enable_write_statement,
        queue_enable_read_statement,
        driver_ok_write_statement,
        driver_ok_read_statement,
        queue_activation_marker_statement,
        queue_notification_statement,
        queue_notification_marker_statement,
        completion_signal_statement,
        completion_signal_marker_statement,
        complete_control_statement,
        used_flags_statement,
        used_index_statement,
        used_identifier_statement,
        used_length_statement,
        used_ring_marker_statement,
        complete_payload_statement,
        payload_completion_marker_statement,
        entropy_sample_statement,
        entropy_sample_marker_statement,
        entropy_quality_statement,
        entropy_quality_marker_statement,
        entropy_pool_statement,
        entropy_pool_marker_statement,
        entropy_mix_statement,
        entropy_mix_marker_statement,
        entropy_credit_statement,
        entropy_credit_marker_statement,
        entropy_generator_statement,
        entropy_generator_marker_statement,
        entropy_extract_statement,
        entropy_extract_marker_statement,
        entropy_delivery_statement,
        auxv_random_marker_statement,
        entropy_getrandom_extract_statement,
        entropy_getrandom_reservation_statement,
        entropy_getrandom_binding_statement,
        entropy_getrandom_small_request_statement,
        entropy_getrandom_large_request_statement,
        enter_user,
        multipage_executable_marker_statement,
        user_getrandom_marker_statement,
        user_getrandom_repeated_marker_statement,
        user_getrandom_full_output_marker_statement,
        user_getrandom_small_request_marker_statement,
        user_getrandom_large_request_marker_statement,
        user_read_marker_statement,
        user_close_marker_statement,
        user_dup_marker_statement,
        user_fcntl_marker_statement,
        user_fstat_marker_statement,
        user_tcgets_marker_statement,
        user_winsize_marker_statement,
        user_lseek_marker_statement,
        user_copy_to_marker_statement,
        user_copy_from_marker_statement,
        user_marker_statement,
        entry_marker_statement,
        stack_marker_statement,
        auxv_marker_statement,
        auxv_entry_marker_statement,
        auxv_execfn_marker_statement,
        auxv_credentials_marker_statement,
        elf_marker_statement,
        external_elf_marker_statement,
        external_elf_copy_marker_statement,
        runtime_extent_marker_statement,
        runtime_entry_marker_statement,
        runtime_data_marker_statement,
        external_bss_marker_statement,
        data_marker_statement,
        writable_data_marker_statement,
        image_copy_to_marker_statement,
        image_copy_from_marker_statement,
        image_syscall_marker_statement,
        retained_marker_statement,
        release_anonymous,
        release_anonymous_frame,
        unmap_mapping_again,
        release_data,
        unmap_code,
        release_code,
        mapping_marker_statement,
        disjoint_marker_statement,
        complete_user,
        reclaim_user_space,
        release_user_frames,
        backing_marker_statement,
        disposition,
    ] = user_continuation
    else {
        return Err(user_context_diagnostic(
            source,
            span,
            "user probe must return with its image resident before ordered teardown, completion, reclaim, and disposition",
        ));
    };
    let user_completed =
        parse_user_context_enter(source, enter_user, &completion_resumed, &user_probe)?;
    let multipage_executable_marker = analyze_operation(
        source,
        multipage_executable_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if multipage_executable_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_MULTIPAGE_EXEC_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(multipage_executable_marker_statement),
            "returned user execution requires its exact multipage-executable marker",
        ));
    }
    let user_getrandom_marker = analyze_operation(
        source,
        user_getrandom_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_getrandom_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_GETRANDOM_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_getrandom_marker_statement),
            "returned user execution requires its exact bounded getrandom marker",
        ));
    }
    let user_getrandom_repeated_marker = analyze_operation(
        source,
        user_getrandom_repeated_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_getrandom_repeated_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_GETRANDOM_REPEATED_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_getrandom_repeated_marker_statement),
            "returned user execution requires its exact repeated getrandom marker",
        ));
    }
    let user_getrandom_full_output_marker = analyze_operation(
        source,
        user_getrandom_full_output_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_getrandom_full_output_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_GETRANDOM_FULL_OUTPUT_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_getrandom_full_output_marker_statement),
            "returned user execution requires its exact full-output getrandom marker",
        ));
    }
    let user_getrandom_small_request_marker = analyze_operation(
        source,
        user_getrandom_small_request_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_getrandom_small_request_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_GETRANDOM_SMALL_REQUEST_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_getrandom_small_request_marker_statement),
            "returned user execution requires its exact small-request getrandom marker",
        ));
    }
    let user_getrandom_large_request_marker = analyze_operation(
        source,
        user_getrandom_large_request_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_getrandom_large_request_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_GETRANDOM_LARGE_REQUEST_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_getrandom_large_request_marker_statement),
            "returned user execution requires its exact large-request getrandom marker",
        ));
    }
    let user_read_marker = analyze_operation(
        source,
        user_read_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_read_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_READ_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_read_marker_statement),
            "returned user execution requires its exact constructed read marker",
        ));
    }
    let user_close_marker = analyze_operation(
        source,
        user_close_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_close_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_CLOSE_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_close_marker_statement),
            "returned user execution requires its exact descriptor-close marker",
        ));
    }
    let user_dup_marker = analyze_operation(
        source,
        user_dup_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_dup_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_DUP_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_dup_marker_statement),
            "returned user execution requires its exact descriptor-duplication marker",
        ));
    }
    let user_fcntl_marker = analyze_operation(
        source,
        user_fcntl_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_fcntl_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_FCNTL_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_fcntl_marker_statement),
            "returned user execution requires its exact descriptor-metadata marker",
        ));
    }
    let user_fstat_marker = analyze_operation(
        source,
        user_fstat_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_fstat_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_FSTAT_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_fstat_marker_statement),
            "returned user execution requires its exact descriptor-stat marker",
        ));
    }
    let user_tcgets_marker = analyze_operation(
        source,
        user_tcgets_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_tcgets_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_TCGETS_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_tcgets_marker_statement),
            "returned user execution requires its exact terminal-attributes marker",
        ));
    }
    let user_winsize_marker = analyze_operation(
        source,
        user_winsize_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_winsize_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_WINSIZE_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_winsize_marker_statement),
            "returned user execution requires its exact console-geometry marker",
        ));
    }
    let user_lseek_marker = analyze_operation(
        source,
        user_lseek_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_lseek_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_LSEEK_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(user_lseek_marker_statement),
            "returned user execution requires its exact console-seek marker",
        ));
    }
    let user_copy_to_marker = analyze_operation(
        source,
        user_copy_to_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_copy_to_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_COPY_TO_OK".into(),
        })
    {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(user_copy_to_marker_statement),
            "user copy_to_user recovery requires its exact marker",
        ));
    }
    let user_copy_from_marker = analyze_operation(
        source,
        user_copy_from_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_copy_from_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_COPY_FROM_OK".into(),
        })
    {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(user_copy_from_marker_statement),
            "user copy_from_user recovery requires its exact marker",
        ));
    }
    let user_marker = analyze_operation(
        source,
        user_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if user_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_SYSCALL_OK".into(),
        })
    {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(user_marker_statement),
            "user syscall round trip requires its exact marker",
        ));
    }
    let entry_marker = analyze_operation(
        source,
        entry_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entry_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_ENTRY_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entry_marker_statement),
            "returned user execution requires its exact constructed-image entry marker",
        ));
    }
    let stack_marker = analyze_operation(
        source,
        stack_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if stack_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_STACK_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(stack_marker_statement),
            "returned user execution requires its exact constructed-image initial-stack marker",
        ));
    }
    let auxv_marker = analyze_operation(
        source,
        auxv_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if auxv_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_AUXV_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(auxv_marker_statement),
            "returned user execution requires its exact constructed-image auxiliary-vector marker",
        ));
    }
    let auxv_entry_marker = analyze_operation(
        source,
        auxv_entry_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if auxv_entry_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_AUXV_ENTRY_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(auxv_entry_marker_statement),
            "returned user execution requires its exact constructed-image auxiliary-vector entry marker",
        ));
    }
    let auxv_execfn_marker = analyze_operation(
        source,
        auxv_execfn_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if auxv_execfn_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_AUXV_EXECFN_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(auxv_execfn_marker_statement),
            "returned user execution requires its exact constructed-image auxiliary-vector executable-name marker",
        ));
    }
    let auxv_credentials_marker = analyze_operation(
        source,
        auxv_credentials_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if auxv_credentials_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_AUXV_CREDENTIALS_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(auxv_credentials_marker_statement),
            "returned user execution requires its exact constructed-image auxiliary-vector credentials marker",
        ));
    }
    let elf_marker = analyze_operation(
        source,
        elf_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if elf_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_ELF_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(elf_marker_statement),
            "returned user execution requires its exact constructed-image ELF identity marker",
        ));
    }
    let external_elf_marker = analyze_operation(
        source,
        external_elf_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if external_elf_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(external_elf_marker_statement),
            "returned user execution requires its exact external ELF-load marker",
        ));
    }
    let external_elf_copy_marker = analyze_operation(
        source,
        external_elf_copy_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if external_elf_copy_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_EXTERNAL_COPY_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(external_elf_copy_marker_statement),
            "returned user execution requires its exact external ELF-copy marker",
        ));
    }
    let runtime_extent_marker = analyze_operation(
        source,
        runtime_extent_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if runtime_extent_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_RUNTIME_EXTENT_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(runtime_extent_marker_statement),
            "returned user execution requires its exact runtime-derived executable-extent marker",
        ));
    }
    let runtime_entry_marker = analyze_operation(
        source,
        runtime_entry_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if runtime_entry_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_RUNTIME_ENTRY_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(runtime_entry_marker_statement),
            "returned user execution requires its exact runtime-derived ELF-entry marker",
        ));
    }
    let runtime_data_marker = analyze_operation(
        source,
        runtime_data_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if runtime_data_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_RUNTIME_DATA_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(runtime_data_marker_statement),
            "returned user execution requires its exact runtime-derived data/BSS-extent marker",
        ));
    }
    let external_bss_marker = analyze_operation(
        source,
        external_bss_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if external_bss_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_BSS_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(external_bss_marker_statement),
            "returned user execution requires its exact external ELF data/BSS marker",
        ));
    }
    let data_marker = analyze_operation(
        source,
        data_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if data_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_DATA_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(data_marker_statement),
            "returned user execution requires its exact constructed-image data marker",
        ));
    }
    let writable_data_marker = analyze_operation(
        source,
        writable_data_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if writable_data_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_WRITABLE_DATA_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(writable_data_marker_statement),
            "returned user execution requires its exact writable-data marker",
        ));
    }
    let image_copy_to_marker = analyze_operation(
        source,
        image_copy_to_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if image_copy_to_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_COPY_TO_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(image_copy_to_marker_statement),
            "returned user execution requires its exact constructed-image copy-to-user marker",
        ));
    }
    let image_copy_from_marker = analyze_operation(
        source,
        image_copy_from_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if image_copy_from_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_COPY_FROM_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(image_copy_from_marker_statement),
            "returned user execution requires its exact constructed-image copy-from-user marker",
        ));
    }
    let image_syscall_marker = analyze_operation(
        source,
        image_syscall_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if image_syscall_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_SYSCALL_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(image_syscall_marker_statement),
            "returned user execution requires its exact constructed-image syscall marker",
        ));
    }
    let retained_marker = analyze_operation(
        source,
        retained_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if retained_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_RETAINED".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(retained_marker_statement),
            "returned user execution requires its exact retained-image marker before teardown",
        ));
    }
    let (returned_anonymous, release_anonymous_continuation) =
        parse_user_address_space_release_anonymous(
            source,
            release_anonymous,
            &completion_resumed,
            &user_space,
            &anonymous_page,
        )?;
    if !release_anonymous_continuation.is_empty() {
        return Err(user_context_diagnostic(
            source,
            statement_span(release_anonymous),
            "anonymous release continuation must remain after returned user execution",
        ));
    }
    parse_physical_frame_release(
        source,
        release_anonymous_frame,
        &completion_resumed,
        &returned_anonymous,
    )?;
    let (returned_scratch_again, unmap_again_continuation) = parse_user_address_space_unmap(
        source,
        unmap_mapping_again,
        &completion_resumed,
        &user_space,
        &writable_data_scratch_again,
    )?;
    if !unmap_again_continuation.is_empty() {
        return Err(user_context_diagnostic(
            source,
            statement_span(unmap_mapping_again),
            "writable-data unmap continuation must remain after returned user execution",
        ));
    }
    parse_physical_frame_release(
        source,
        release_data,
        &completion_resumed,
        &returned_scratch_again,
    )?;
    let (returned_code, code_unmap_continuation) = parse_user_address_space_unmap(
        source,
        unmap_code,
        &completion_resumed,
        &user_space,
        &executable_scratch,
    )?;
    if !code_unmap_continuation.is_empty() {
        return Err(user_context_diagnostic(
            source,
            statement_span(unmap_code),
            "executable-code unmap continuation must remain after returned user execution",
        ));
    }
    parse_physical_frame_release(source, release_code, &completion_resumed, &returned_code)?;
    let mapping_marker = analyze_operation(
        source,
        mapping_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if mapping_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_MAPPING_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(mapping_marker_statement),
            "user mapping round trip requires its exact marker",
        ));
    }
    let disjoint_marker = analyze_operation(
        source,
        disjoint_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if disjoint_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_FRAME_DISJOINT_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(disjoint_marker_statement),
            "disjoint frame release requires its exact marker",
        ));
    }
    let revoked_space =
        parse_user_execution_complete(source, complete_user, &completion_resumed, &user_completed)?;
    let returned_frames = parse_user_address_space_reclaim(
        source,
        reclaim_user_space,
        &completion_resumed,
        &revoked_space,
    )?;
    parse_physical_frame_release(
        source,
        release_user_frames,
        &completion_resumed,
        &returned_frames,
    )?;
    let backing_marker = analyze_operation(
        source,
        backing_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if backing_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_BACKING_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(backing_marker_statement),
            "user backing release requires its exact marker",
        ));
    }
    let mut queue_activation_operations = Vec::new();
    for (statement, request) in [
        queue_enable_write_statement,
        queue_enable_read_statement,
        driver_ok_write_statement,
        driver_ok_read_statement,
    ]
    .into_iter()
    .zip(CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION)
    {
        let operation = analyze_operation(
            source,
            statement,
            CompilerSystemsContextKind::Bootstrap,
            &completion_resumed,
        )?;
        if operation != (CompilerSystemsOperation::AccessPciRegionRegister { request }) {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "user teardown must be followed by the exact virtio queue-activation sequence",
            ));
        }
        queue_activation_operations.push(operation);
    }
    let queue_activation_marker = analyze_operation(
        source,
        queue_activation_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if queue_activation_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_VIRTIO_QUEUE_ACTIVE".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(queue_activation_marker_statement),
            "virtio queue activation requires its exact completion marker",
        ));
    }
    let queue_notification = analyze_operation(
        source,
        queue_notification_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if queue_notification
        != (CompilerSystemsOperation::AccessPciRegionRegister {
            request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION[0],
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(queue_notification_statement),
            "verified virtio queue activation must be followed by the exact queue-zero notification",
        ));
    }
    let queue_notification_marker = analyze_operation(
        source,
        queue_notification_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if queue_notification_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_VIRTIO_QUEUE_NOTIFIED".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(queue_notification_marker_statement),
            "virtio queue notification requires its exact completion marker",
        ));
    }
    let completion_signal = analyze_operation(
        source,
        completion_signal_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if completion_signal
        != (CompilerSystemsOperation::AccessPciRegionRegister {
            request: CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL[0],
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(completion_signal_statement),
            "virtio queue notification must be followed by the exact completion-signal observation",
        ));
    }
    let completion_signal_marker = analyze_operation(
        source,
        completion_signal_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if completion_signal_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_VIRTIO_COMPLETION_SIGNAL".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(completion_signal_marker_statement),
            "virtio completion-signal observation requires its exact marker",
        ));
    }
    let complete_control = analyze_operation(
        source,
        complete_control_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if complete_control
        != (CompilerSystemsOperation::CompleteDmaBufferForCpu {
            request: CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[1],
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(complete_control_statement),
            "virtio used-ring validation requires exact control-buffer CPU completion",
        ));
    }
    let used_ring_statements = [
        used_flags_statement,
        used_index_statement,
        used_identifier_statement,
        used_length_statement,
    ];
    let mut used_ring_operations = Vec::with_capacity(4);
    for (statement, request) in used_ring_statements
        .iter()
        .zip(CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING)
    {
        let operation = analyze_operation(
            source,
            statement,
            CompilerSystemsContextKind::Bootstrap,
            &completion_resumed,
        )?;
        if operation != (CompilerSystemsOperation::LoadDmaBufferIntegerEquals { request }) {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "virtio used-ring validation requires the exact ordered control-field reads",
            ));
        }
        used_ring_operations.push(operation);
    }
    let used_ring_marker = analyze_operation(
        source,
        used_ring_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if used_ring_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_VIRTIO_USED_RING_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(used_ring_marker_statement),
            "virtio used-ring validation requires its exact marker",
        ));
    }
    let complete_payload = analyze_operation(
        source,
        complete_payload_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if complete_payload
        != (CompilerSystemsOperation::CompleteDmaBufferForCpu {
            request: CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG[0],
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(complete_payload_statement),
            "virtio used-ring validation must be followed by exact payload-buffer CPU completion",
        ));
    }
    let payload_completion_marker = analyze_operation(
        source,
        payload_completion_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if payload_completion_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_VIRTIO_PAYLOAD_CPU_OWNED".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(payload_completion_marker_statement),
            "virtio payload completion requires its exact marker",
        ));
    }
    let entropy_sample = analyze_operation(
        source,
        entropy_sample_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_sample
        != (CompilerSystemsOperation::CaptureEntropySample {
            request: CompilerEntropySampleCaptureRequest::INITIAL_Q35_VIRTIO_RNG,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_sample_statement),
            "virtio payload completion must be followed by exact uncredited entropy-sample capture",
        ));
    }
    let entropy_sample_marker = analyze_operation(
        source,
        entropy_sample_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_sample_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_ENTROPY_SAMPLE_CAPTURED".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_sample_marker_statement),
            "entropy-sample capture requires its exact marker",
        ));
    }
    let entropy_quality = analyze_operation(
        source,
        entropy_quality_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_quality
        != (CompilerSystemsOperation::AssessEntropySampleQuality {
            request: CompilerEntropySampleQualityRequest::INITIAL_Q35_VIRTIO_RNG,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_quality_statement),
            "entropy-sample capture must be followed by exact Q35 quality assessment",
        ));
    }
    let entropy_quality_marker = analyze_operation(
        source,
        entropy_quality_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_quality_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_ENTROPY_SAMPLE_CREDITED".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_quality_marker_statement),
            "entropy-sample quality assessment requires its exact marker",
        ));
    }
    let entropy_pool = analyze_operation(
        source,
        entropy_pool_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_pool
        != (CompilerSystemsOperation::CreateEntropyPool {
            request: CompilerEntropyPoolRequest::INITIAL_Q35,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_pool_statement),
            "entropy-sample assessment must be followed by exact empty-pool creation",
        ));
    }
    let entropy_pool_marker = analyze_operation(
        source,
        entropy_pool_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_pool_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_ENTROPY_POOL_CREATED".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_pool_marker_statement),
            "entropy-pool creation requires its exact marker",
        ));
    }
    let entropy_mix = analyze_operation(
        source,
        entropy_mix_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_mix
        != (CompilerSystemsOperation::MixEntropyPoolSample {
            request: CompilerEntropyPoolMixRequest::INITIAL_Q35,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_mix_statement),
            "empty-pool creation must be followed by the exact first sample mix",
        ));
    }
    let entropy_mix_marker = analyze_operation(
        source,
        entropy_mix_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_mix_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_ENTROPY_POOL_MIXED".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_mix_marker_statement),
            "entropy-pool mixing requires its exact marker",
        ));
    }
    let entropy_credit = analyze_operation(
        source,
        entropy_credit_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_credit
        != (CompilerSystemsOperation::CreditEntropyPoolContribution {
            request: CompilerEntropyPoolCreditRequest::INITIAL_Q35,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_credit_statement),
            "the first sample mix must be followed by its exact contribution-credit transfer",
        ));
    }
    let entropy_credit_marker = analyze_operation(
        source,
        entropy_credit_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_credit_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_ENTROPY_POOL_CREDITED".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_credit_marker_statement),
            "entropy-credit transfer requires its exact marker",
        ));
    }
    let entropy_generator = analyze_operation(
        source,
        entropy_generator_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_generator
        != (CompilerSystemsOperation::InitializeEntropyGenerator {
            request: CompilerEntropyGeneratorRequest::INITIAL_Q35,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_generator_statement),
            "entropy credit must be followed by exact generation-one CRNG initialization",
        ));
    }
    let entropy_generator_marker = analyze_operation(
        source,
        entropy_generator_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_generator_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_ENTROPY_GENERATOR_READY".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_generator_marker_statement),
            "entropy-generator initialization requires its exact marker",
        ));
    }
    let entropy_extract = analyze_operation(
        source,
        entropy_extract_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_extract
        != (CompilerSystemsOperation::ExtractEntropyGenerator {
            request: CompilerEntropyGeneratorExtractRequest::INITIAL_Q35,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_extract_statement),
            "ready generation one must be followed by the exact first CRNG extraction",
        ));
    }
    let entropy_extract_marker = analyze_operation(
        source,
        entropy_extract_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_extract_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_ENTROPY_OUTPUT_READY".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_extract_marker_statement),
            "entropy-generator extraction requires its exact marker",
        ));
    }
    let entropy_delivery = analyze_operation(
        source,
        entropy_delivery_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_delivery
        != (CompilerSystemsOperation::DeliverInitialAuxvRandom {
            request: CompilerEntropyOutputDeliveryRequest::INITIAL_AUXV_RANDOM,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_delivery_statement),
            "first CRNG extraction must be followed by exact affine initial AT_RANDOM delivery",
        ));
    }
    let auxv_random_marker = analyze_operation(
        source,
        auxv_random_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if auxv_random_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_USER_IMAGE_AUXV_RANDOM_OK".into(),
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(auxv_random_marker_statement),
            "initial AT_RANDOM delivery requires its exact completion marker before user entry",
        ));
    }
    let entropy_getrandom_extract = analyze_operation(
        source,
        entropy_getrandom_extract_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_getrandom_extract
        != (CompilerSystemsOperation::ExtractEntropyGenerator {
            request: CompilerEntropyGeneratorExtractRequest::INITIAL_Q35_GETRANDOM,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_getrandom_extract_statement),
            "initial AT_RANDOM delivery must be followed by the exact second CRNG extraction",
        ));
    }
    let entropy_getrandom_reservation = analyze_operation(
        source,
        entropy_getrandom_reservation_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_getrandom_reservation
        != (CompilerSystemsOperation::ReserveInitialUserRandom {
            request: CompilerEntropyOutputReservationRequest::INITIAL_GETRANDOM,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_getrandom_reservation_statement),
            "the second CRNG output must be reserved by the exact initial getrandom capability before user entry",
        ));
    }
    let entropy_getrandom_binding = analyze_operation(
        source,
        entropy_getrandom_binding_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_getrandom_binding
        != (CompilerSystemsOperation::BindReusableUserRandom {
            request: CompilerEntropyUserRandomServiceRequest::INITIAL_GETRANDOM,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_getrandom_binding_statement),
            "the initial reservation and generation-three generator must bind the exact reusable getrandom service before user entry",
        ));
    }
    let entropy_getrandom_small_request = analyze_operation(
        source,
        entropy_getrandom_small_request_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_getrandom_small_request
        != (CompilerSystemsOperation::ExpandSmallUserRandomRequests {
            request: CompilerEntropyUserRandomSmallRequest::INITIAL_GETRANDOM,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_getrandom_small_request_statement),
            "the reusable getrandom service must receive its exact bounded small-request refinement before user entry",
        ));
    }
    let entropy_getrandom_large_request = analyze_operation(
        source,
        entropy_getrandom_large_request_statement,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    if entropy_getrandom_large_request
        != (CompilerSystemsOperation::ExpandLargeUserRandomRequests {
            request: CompilerEntropyUserRandomSmallRequest::LARGE_GETRANDOM,
        })
    {
        return Err(user_context_diagnostic(
            source,
            statement_span(entropy_getrandom_large_request_statement),
            "the small-request getrandom service must receive its exact bounded large-request refinement before user entry",
        ));
    }
    let disposition = analyze_disposition(
        source,
        disposition,
        CompilerSystemsContextKind::Bootstrap,
        &completion_resumed,
    )?;
    operations.extend([
        CompilerSystemsOperation::AllocatePhysicalFrames {
            request: backing_request,
            failure_message: backing_failure,
        },
        CompilerSystemsOperation::PrepareUserAddressSpace,
        CompilerSystemsOperation::AllocatePhysicalFrames {
            request: scratch_request,
            failure_message: scratch_failure_message,
        },
        CompilerSystemsOperation::MapUserAddressSpace,
        CompilerSystemsOperation::InstallUserMappingEntryTrampoline,
        CompilerSystemsOperation::ProtectUserMappingReadOnly,
        CompilerSystemsOperation::VerifyUserMappingEntryTrampoline,
        CompilerSystemsOperation::PermitUserMappingExecute,
        CompilerSystemsOperation::AllocatePhysicalFrames {
            request: data_request,
            failure_message: data_failure_message,
        },
        CompilerSystemsOperation::MapUserAddressSpace,
        CompilerSystemsOperation::StoreUserMappingByte,
        CompilerSystemsOperation::ProtectUserMappingReadOnly,
        CompilerSystemsOperation::LoadUserMappingByte,
        CompilerSystemsOperation::PermitUserMappingWritableData,
        CompilerSystemsOperation::AllocatePhysicalFrames {
            request: anonymous_request,
            failure_message: anonymous_failure_message,
        },
        CompilerSystemsOperation::ReserveAnonymousUserMapping,
        image_ready_marker,
        boot_resource_marker,
        CompilerSystemsOperation::PrepareUserProbe {
            process_identity: 1,
        },
    ]);
    operations.extend(queue_activation_operations);
    operations.push(queue_activation_marker);
    operations.push(queue_notification);
    operations.push(queue_notification_marker);
    operations.push(completion_signal);
    operations.push(completion_signal_marker);
    operations.push(complete_control);
    operations.extend(used_ring_operations);
    operations.push(used_ring_marker);
    operations.push(complete_payload);
    operations.push(payload_completion_marker);
    operations.push(entropy_sample);
    operations.push(entropy_sample_marker);
    operations.push(entropy_quality);
    operations.push(entropy_quality_marker);
    operations.push(entropy_pool);
    operations.push(entropy_pool_marker);
    operations.push(entropy_mix);
    operations.push(entropy_mix_marker);
    operations.push(entropy_credit);
    operations.push(entropy_credit_marker);
    operations.push(entropy_generator);
    operations.push(entropy_generator_marker);
    operations.push(entropy_extract);
    operations.push(entropy_extract_marker);
    operations.push(entropy_delivery);
    operations.push(auxv_random_marker);
    operations.push(entropy_getrandom_extract);
    operations.push(entropy_getrandom_reservation);
    operations.push(entropy_getrandom_binding);
    operations.push(entropy_getrandom_small_request);
    operations.push(entropy_getrandom_large_request);
    operations.extend([
        CompilerSystemsOperation::EnterUserContext,
        multipage_executable_marker,
        user_getrandom_marker,
        user_getrandom_repeated_marker,
        user_getrandom_full_output_marker,
        user_getrandom_small_request_marker,
        user_getrandom_large_request_marker,
        user_read_marker,
        user_close_marker,
        user_dup_marker,
        user_fcntl_marker,
        user_fstat_marker,
        user_tcgets_marker,
        user_winsize_marker,
        user_lseek_marker,
        user_copy_to_marker,
        user_copy_from_marker,
        user_marker,
        entry_marker,
        stack_marker,
        auxv_marker,
        auxv_entry_marker,
        auxv_execfn_marker,
        auxv_credentials_marker,
        elf_marker,
        external_elf_marker,
        external_elf_copy_marker,
        runtime_extent_marker,
        runtime_entry_marker,
        runtime_data_marker,
        external_bss_marker,
        data_marker,
        writable_data_marker,
        image_copy_to_marker,
        image_copy_from_marker,
        image_syscall_marker,
        retained_marker,
        CompilerSystemsOperation::ReleaseAnonymousUserMapping,
        CompilerSystemsOperation::ReleasePhysicalFrames,
        CompilerSystemsOperation::UnmapUserAddressSpace,
        CompilerSystemsOperation::ReleasePhysicalFrames,
        CompilerSystemsOperation::UnmapUserAddressSpace,
        CompilerSystemsOperation::ReleasePhysicalFrames,
        mapping_marker,
        disjoint_marker,
        CompilerSystemsOperation::CompleteUserExecution,
        CompilerSystemsOperation::ReclaimUserAddressSpace,
        CompilerSystemsOperation::ReleasePhysicalFrames,
        backing_marker,
    ]);
    Ok(CheckedBootstrapRegionDecision {
        operations,
        disposition,
    })
}
