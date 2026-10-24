#[allow(clippy::too_many_lines)] // The sealed affine edit grammar stays visible as one ordered lifecycle.
fn analyze_active_translation_edit_sequence(
    source: &SourceText,
    action: &Expression,
    statements: &[Statement],
    active_context: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CheckedBootstrapRegionDecision, Diagnostic> {
    let (subject, frames_name, allocation_success, failure_name, failure, span) =
        result_sequence(source, statements, "active-translation frame allocation")?;
    let request = parse_physical_frame_allocation(source, subject, active_context)?;
    if request
        != (CompilerPhysicalFrameRequest {
            frame_count: 1,
            alignment_frames: 1,
        })
    {
        return Err(translation_edit_diagnostic(
            source,
            subject.span(),
            "the initial active-translation edit requires exactly one 4 KiB frame",
        ));
    }
    let CompilerSystemsDisposition::Fatal {
        message: allocation_failure,
    } = action_disposition(source, failure, active_context)?
    else {
        return Err(translation_edit_diagnostic(
            source,
            failure.span(),
            "active-translation frame allocation failure must enter `fatal`",
        ));
    };
    let _ = failure_name;
    let allocation_statements = translation_block(
        source,
        allocation_success,
        span,
        "active-translation frame allocation success requires a map edit",
    )?;

    let (subject, edit_name, edit_success, failure_name, failure, span) =
        result_sequence(source, allocation_statements, "translation map edit begin")?;
    parse_translation_edit_begin(source, subject, active_context)?;
    let begin_failure = translation_failure(
        source,
        failure,
        &failure_name,
        "translation map-edit begin failure must enter `fatal`",
    )?;
    let edit_statements = translation_block(
        source,
        edit_success,
        span,
        "translation map-edit success requires a provisional mapping",
    )?;

    let (subject, mapping_name, map_success, failure_name, failure, span) =
        result_sequence(source, edit_statements, "translation edit mapping")?;
    let mapping_request =
        parse_translation_edit_mapping(source, subject, &edit_name, &frames_name)?;
    let map_failure = translation_failure(
        source,
        failure,
        &failure_name,
        "translation edit mapping failure must enter `fatal`",
    )?;
    let mapping_statements = translation_block(
        source,
        map_success,
        span,
        "translation edit mapping success requires commit before access",
    )?;

    let (subject, edited_name, commit_success, failure_name, failure, span) =
        result_sequence(source, mapping_statements, "translation map edit commit")?;
    parse_translation_edit_commit(source, subject, &edit_name)?;
    let commit_failure = translation_failure(
        source,
        failure,
        &failure_name,
        "translation map-edit commit failure must enter `fatal`",
    )?;
    let committed = translation_block(
        source,
        commit_success,
        span,
        "translation map-edit commit success requires mapping access and removal",
    )?;
    let [store, load, comparison, true_rule, false_rule] = committed else {
        return Err(translation_edit_diagnostic(
            source,
            commit_success.span(),
            "committed translation mapping requires byte store, byte load, and an exhaustive equality decision",
        ));
    };
    let (store_offset, value) = parse_mapping_store(source, store, &mapping_name)?;
    let (loaded_name, load_offset) = parse_mapping_load(source, load, &mapping_name)?;
    let (expected, true_action, false_action) = parse_mapping_comparison(
        source,
        comparison,
        true_rule,
        false_rule,
        &loaded_name,
    )?;
    if store_offset >= 4096 || load_offset >= 4096 {
        return Err(translation_edit_diagnostic(
            source,
            commit_success.span(),
            "active-translation mapping byte access is outside the one-frame mapping",
        ));
    }

    let mut checked = analyze_translation_edit_unmap_success(
        source,
        true_action,
        &edited_name,
        &mapping_name,
        storage,
    )?;
    let mismatch_message = analyze_translation_edit_unmap_failure(
        source,
        false_action,
        &edited_name,
        &mapping_name,
    )?;
    let mut operations = vec![
        CompilerSystemsOperation::AllocatePhysicalFrames {
            request,
            failure_message: allocation_failure,
        },
        CompilerSystemsOperation::BeginTranslationEdit {
            kind: CompilerTranslationEditKind::Map,
            failure_message: begin_failure,
        },
        CompilerSystemsOperation::MapTranslationFrames {
            request: mapping_request,
            failure_message: map_failure,
        },
        CompilerSystemsOperation::CommitTranslationEdit {
            kind: CompilerTranslationEditKind::Map,
            failure_message: commit_failure,
        },
        CompilerSystemsOperation::KernelMappingStoreByte {
            offset_bytes: store_offset,
            value,
        },
        CompilerSystemsOperation::KernelMappingLoadByteEquals {
            offset_bytes: load_offset,
            expected,
            failure_message: mismatch_message,
        },
    ];
    operations.append(&mut checked.operations);
    let _ = action;
    Ok(CheckedBootstrapRegionDecision {
        operations,
        disposition: checked.disposition,
    })
}

fn analyze_translation_edit_unmap_success(
    source: &SourceText,
    action: &Expression,
    active_context: &str,
    mapping_name: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CheckedBootstrapRegionDecision, Diagnostic> {
    let statements = translation_block(
        source,
        action,
        action.span(),
        "successful mapping comparison requires an unmap edit block",
    )?;
    let (unmapped_name, returned_frames, commit_success, mut operations) =
        analyze_translation_edit_unmap_prefix(source, statements, active_context, mapping_name)?;
    let success_statements = translation_block(
        source,
        commit_success,
        commit_success.span(),
        "translation unmap-edit commit success requires release and continuation",
    )?;
    let [marker, release, rest @ ..] = success_statements else {
        return Err(translation_edit_diagnostic(
            source,
            commit_success.span(),
            "translation unmap-edit commit success requires marker, frame release, and bootstrap continuation",
        ));
    };
    let marker = analyze_operation(
        source,
        marker,
        CompilerSystemsContextKind::Bootstrap,
        &unmapped_name,
    )?;
    if !matches!(marker, CompilerSystemsOperation::ConsoleWrite { .. }) {
        return Err(translation_edit_diagnostic(
            source,
            statement_span(success_statements.first().expect("marker exists")),
            "translation edit success must publish its marker after unmap commit",
        ));
    }
    parse_physical_frame_release(source, release, &unmapped_name, &returned_frames)?;
    let mut checked = analyze_bounded_multi_frame_sequence(
        source,
        rest,
        &unmapped_name,
        storage,
    )?;
    operations.push(marker);
    operations.push(CompilerSystemsOperation::ReleasePhysicalFrames);
    operations.append(&mut checked.operations);
    Ok(CheckedBootstrapRegionDecision {
        operations,
        disposition: checked.disposition,
    })
}

fn analyze_bounded_multi_frame_sequence(
    source: &SourceText,
    statements: &[Statement],
    allocator_context: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CheckedBootstrapRegionDecision, Diagnostic> {
    let (subject, extent_name, success, _problem_name, failure, span) =
        result_sequence(source, statements, "bounded multi-frame allocation")?;
    let request = parse_physical_frame_allocation(source, subject, allocator_context)?;
    if request
        != (CompilerPhysicalFrameRequest {
            frame_count: 3,
            alignment_frames: 1,
        })
    {
        return Err(translation_edit_diagnostic(
            source,
            subject.span(),
            "the bounded multi-frame probe requires exactly three frames aligned to one frame",
        ));
    }
    let CompilerSystemsDisposition::Fatal {
        message: failure_message,
    } = action_disposition(source, failure, allocator_context)?
    else {
        return Err(translation_edit_diagnostic(
            source,
            failure.span(),
            "bounded multi-frame allocation failure must consume the allocator context through `fatal`",
        ));
    };
    let success_statements = translation_block(
        source,
        success,
        span,
        "bounded multi-frame allocation success requires marker, whole-extent release, and continuation",
    )?;
    let [marker, release, rest @ ..] = success_statements else {
        return Err(translation_edit_diagnostic(
            source,
            success.span(),
            "bounded multi-frame allocation success requires marker, whole-extent release, and continuation",
        ));
    };
    let marker = analyze_operation(
        source,
        marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(marker, CompilerSystemsOperation::ConsoleWrite { .. }) {
        return Err(translation_edit_diagnostic(
            source,
            statement_span(success_statements.first().expect("marker exists")),
            "bounded multi-frame allocation must publish its marker before release",
        ));
    }
    parse_physical_frame_release(source, release, allocator_context, &extent_name)?;
    let mut checked =
        analyze_static_elf_frame_sequence(source, rest, allocator_context, storage)?;
    let mut operations = vec![
        CompilerSystemsOperation::AllocatePhysicalFrames {
            request,
            failure_message,
        },
        marker,
        CompilerSystemsOperation::ReleasePhysicalFrames,
    ];
    operations.append(&mut checked.operations);
    Ok(CheckedBootstrapRegionDecision {
        operations,
        disposition: checked.disposition,
    })
}

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

    let (metadata_subject, metadata_name, metadata_success, _problem_name, metadata_failure, metadata_span) =
        result_sequence(source, nested, "static ELF mapping metadata allocation")?;
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
    let (zeroed_construction_name, after_zero) = parse_static_image_payload_zero(
        source,
        zero,
        allocator_context,
        &construction_name,
    )?;
    let [zero_marker, copy, copy_marker, seal, seal_marker, hierarchy_reserve, hierarchy_marker, attach, attach_marker, activate, active_marker, restore, restored_marker, authorize_entry, entry_marker, authorize_stack_slot, stack_slot_marker, stack_backing_reserve, stack_backing_marker, zero_stack_backing, stack_backing_zeroed_marker, publish_stack_leaf, stack_leaf_published_marker, initialize_stack_entry, stack_entry_initialized_marker, add_stack_pagesz, stack_pagesz_marker, add_stack_entry_fact, stack_entry_fact_marker, add_stack_program_headers, stack_program_headers_marker, add_stack_executable_name, stack_executable_name_marker, add_stack_credentials, stack_credentials_marker, add_stack_secure_mode, stack_secure_mode_marker, add_stack_clock_tick_rate, stack_clock_tick_rate_marker, add_stack_platform, stack_platform_marker, add_stack_hardware_capabilities, stack_hardware_capabilities_marker, add_stack_load_metadata, stack_load_metadata_marker, authorize_stack_random_slot, stack_random_slot_authorized_marker, retain_prepared_process, prepared_process_retained_marker, qualify_retained_disjoint_work, retained_disjoint_work_marker, authorize_entropy_handoff, entropy_handoff_authorized_marker, authorize_entropy_acquisition, entropy_acquisition_authorized_marker, qualify_entropy_device, entropy_device_marker, qualify_entropy_transport, entropy_transport_marker, qualify_entropy_negotiation, entropy_negotiation_marker, revoke_entropy_acquisition, entropy_acquisition_revoked_marker, revoke_entropy_handoff, entropy_handoff_revoked_marker, restore_prepared_process, prepared_process_restored_marker, revoke_stack_random_slot, stack_random_slot_revoked_marker, remove_stack_load_metadata, stack_load_metadata_revoked_marker, remove_stack_hardware_capabilities, stack_hardware_capabilities_revoked_marker, remove_stack_platform, stack_platform_revoked_marker, remove_stack_clock_tick_rate, stack_clock_tick_rate_revoked_marker, remove_stack_secure_mode, stack_secure_mode_revoked_marker, remove_stack_credentials, stack_credentials_revoked_marker, remove_stack_executable_name, stack_executable_name_revoked_marker, remove_stack_program_headers, stack_program_headers_revoked_marker, remove_stack_entry_fact, stack_entry_fact_revoked_marker, remove_stack_pagesz, stack_pagesz_revoked_marker, clear_stack_entry, stack_entry_cleared_marker, revoke_stack_leaf, stack_leaf_revoked_marker, stack_backing_release, revoke_stack_slot, stack_slot_revoked_marker, revoke_entry, entry_revoked_marker, detach, detach_marker, hierarchy_reclaim, hierarchy_release, revoke, metadata_release, reclaim, payload_release, rest @ ..] =
        after_zero
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
    let copy_marker_span = statement_span(copy_marker);
    let copy_marker = analyze_operation(
        source,
        copy_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &copy_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_PAYLOAD_COPIED_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            copy_marker_span,
            "static ELF payload copy must publish its exact marker before revocation",
        ));
    }
    let sealed_construction_name = parse_static_image_permission_seal(
        source,
        seal,
        allocator_context,
        &populated_construction_name,
    )?;
    let seal_marker_span = statement_span(seal_marker);
    let seal_marker = analyze_operation(
        source,
        seal_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &seal_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_PERMISSIONS_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            seal_marker_span,
            "static ELF permission sealing must publish its exact marker before revocation",
        ));
    }
    let hierarchy_name = parse_static_image_hierarchy_reservation(
        source,
        hierarchy_reserve,
        allocator_context,
        &sealed_construction_name,
    )?;
    let hierarchy_marker_span = statement_span(hierarchy_marker);
    let hierarchy_marker = analyze_operation(
        source,
        hierarchy_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &hierarchy_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_HIERARCHY_EXTENT_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            hierarchy_marker_span,
            "static ELF hierarchy allocation must publish its exact marker before release",
        ));
    }
    let attached_construction_name = parse_static_image_address_space_attach(
        source,
        attach,
        allocator_context,
        &sealed_construction_name,
        &hierarchy_name,
    )?;
    let attach_marker_span = statement_span(attach_marker);
    let attach_marker = analyze_operation(
        source,
        attach_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &attach_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            attach_marker_span,
            "static ELF address-space attachment must publish its exact marker before detachment",
        ));
    }
    let active_construction_name = parse_static_image_address_space_transition(
        source,
        activate,
        allocator_context,
        &attached_construction_name,
        "activate",
        "bounded-static-elf-kernel-continuation",
    )?;
    let active_marker_span = statement_span(active_marker);
    let active_marker = analyze_operation(
        source,
        active_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &active_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_ACTIVE_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            active_marker_span,
            "static ELF address-space activation must publish its exact marker before restoration",
        ));
    }
    let restored_construction_name = parse_static_image_address_space_transition(
        source,
        restore,
        allocator_context,
        &active_construction_name,
        "restore",
        "bounded-bootstrap-kernel-continuation",
    )?;
    let restored_marker_span = statement_span(restored_marker);
    let restored_marker = analyze_operation(
        source,
        restored_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &restored_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_RESTORED_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            restored_marker_span,
            "static ELF address-space restoration must publish its exact marker before detachment",
        ));
    }
    let entry_authorized_construction_name = parse_static_image_entry_transition(
        source,
        authorize_entry,
        allocator_context,
        &restored_construction_name,
        "authorize",
        "validated-static-elf-entry",
    )?;
    let entry_marker_span = statement_span(entry_marker);
    let entry_marker = analyze_operation(
        source,
        entry_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &entry_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_ENTRY_AUTHORITY_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            entry_marker_span,
            "static ELF entry authorization must publish its exact marker before revocation",
        ));
    }
    let stack_slot_authorized_construction_name = parse_static_image_stack_slot_transition(
        source,
        authorize_stack_slot,
        allocator_context,
        &entry_authorized_construction_name,
        "authorize",
        "bounded-linux-initial-stack-slot",
    )?;
    let stack_slot_marker_span = statement_span(stack_slot_marker);
    let stack_slot_marker = analyze_operation(
        source,
        stack_slot_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &stack_slot_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_STACK_SLOT_AUTHORITY_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            stack_slot_marker_span,
            "static ELF stack-slot authorization must publish its exact marker before revocation",
        ));
    }
    let stack_backing_name = parse_static_image_stack_backing_reservation(
        source,
        stack_backing_reserve,
        allocator_context,
        &stack_slot_authorized_construction_name,
    )?;
    let stack_backing_marker_span = statement_span(stack_backing_marker);
    let stack_backing_marker = analyze_operation(
        source,
        stack_backing_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &stack_backing_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_EXTENT_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            stack_backing_marker_span,
            "static ELF stack backing must publish its exact marker before whole release",
        ));
    }
    let zeroed_stack_backing_construction_name = parse_static_image_stack_backing_zero(
        source,
        zero_stack_backing,
        allocator_context,
        &stack_slot_authorized_construction_name,
        &stack_backing_name,
    )?;
    let stack_backing_zeroed_marker_span = statement_span(stack_backing_zeroed_marker);
    let stack_backing_zeroed_marker = analyze_operation(
        source,
        stack_backing_zeroed_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &stack_backing_zeroed_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_STACK_BACKING_ZEROED_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            stack_backing_zeroed_marker_span,
            "static ELF stack-backing zeroing must publish its exact marker before whole release",
        ));
    }
    let published_stack_leaf_construction_name = parse_static_image_stack_leaf_publication(
        source,
        publish_stack_leaf,
        allocator_context,
        &zeroed_stack_backing_construction_name,
        &stack_backing_name,
    )?;
    let stack_leaf_published_marker_span = statement_span(stack_leaf_published_marker);
    let stack_leaf_published_marker = analyze_operation(
        source,
        stack_leaf_published_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &stack_leaf_published_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_PUBLISHED_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            stack_leaf_published_marker_span,
            "static ELF stack-leaf publication must publish its exact marker before revocation",
        ));
    }
    let initialized_stack_entry_construction_name = parse_static_image_stack_entry_initialization(
        source,
        initialize_stack_entry,
        allocator_context,
        &published_stack_leaf_construction_name,
        &stack_backing_name,
    )?;
    let stack_entry_initialized_marker_span = statement_span(stack_entry_initialized_marker);
    let stack_entry_initialized_marker = analyze_operation(
        source,
        stack_entry_initialized_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &stack_entry_initialized_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_INITIALIZED_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            stack_entry_initialized_marker_span,
            "static ELF stack-entry initialization must publish its exact marker before clearing",
        ));
    }
    let pagesz_stack_entry_construction_name = parse_static_image_stack_pagesz_addition(
        source,
        add_stack_pagesz,
        allocator_context,
        &initialized_stack_entry_construction_name,
        &stack_backing_name,
    )?;
    let stack_pagesz_marker_span = statement_span(stack_pagesz_marker);
    let stack_pagesz_marker = analyze_operation(
        source,
        stack_pagesz_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &stack_pagesz_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_AUXV_PAGESZ_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            stack_pagesz_marker_span,
            "static ELF AT_PAGESZ addition must publish its exact marker before removal",
        ));
    }
    let entry_fact_stack_construction_name = parse_static_image_stack_entry_fact_addition(
        source,
        add_stack_entry_fact,
        allocator_context,
        &pagesz_stack_entry_construction_name,
        &stack_backing_name,
    )?;
    let stack_entry_fact_marker_span = statement_span(stack_entry_fact_marker);
    let stack_entry_fact_marker = analyze_operation(
        source,
        stack_entry_fact_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &stack_entry_fact_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_AUXV_ENTRY_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            stack_entry_fact_marker_span,
            "static ELF AT_ENTRY addition must publish its exact marker before removal",
        ));
    }
    let program_header_stack_construction_name =
        parse_static_image_stack_program_headers_addition(
            source,
            add_stack_program_headers,
            allocator_context,
            &entry_fact_stack_construction_name,
            &stack_backing_name,
        )?;
    let stack_program_headers_marker_span = statement_span(stack_program_headers_marker);
    let stack_program_headers_marker = analyze_operation(
        source,
        stack_program_headers_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &stack_program_headers_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_AUXV_PROGRAM_HEADERS_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            stack_program_headers_marker_span,
            "static ELF program-header addition must publish its exact marker before removal",
        ));
    }
    let executable_name_stack_construction_name =
        parse_static_image_stack_executable_name_transition(
            source,
            add_stack_executable_name,
            allocator_context,
            &program_header_stack_construction_name,
            &stack_backing_name,
            true,
        )?;
    let stack_executable_name_marker_span = statement_span(stack_executable_name_marker);
    let stack_executable_name_marker = analyze_operation(
        source,
        stack_executable_name_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(&stack_executable_name_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_AUXV_EXECFN_OK") {
        return Err(translation_edit_diagnostic(source, stack_executable_name_marker_span, "static ELF AT_EXECFN addition must publish its exact marker before removal"));
    }
    let credential_stack_construction_name = parse_static_image_stack_credentials_transition(
        source,
        add_stack_credentials,
        allocator_context,
        &executable_name_stack_construction_name,
        &stack_backing_name,
        true,
    )?;
    let stack_credentials_marker_span = statement_span(stack_credentials_marker);
    let stack_credentials_marker = analyze_operation(
        source,
        stack_credentials_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(&stack_credentials_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_AUXV_CREDENTIALS_OK") {
        return Err(translation_edit_diagnostic(source, stack_credentials_marker_span, "static ELF credential addition must publish its exact marker before removal"));
    }
    let secure_mode_stack_construction_name = parse_static_image_stack_secure_mode_transition(
        source,
        add_stack_secure_mode,
        allocator_context,
        &credential_stack_construction_name,
        &stack_backing_name,
        true,
    )?;
    let stack_secure_mode_marker_span = statement_span(stack_secure_mode_marker);
    let stack_secure_mode_marker = analyze_operation(
        source,
        stack_secure_mode_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(&stack_secure_mode_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_AUXV_SECURE_OK") {
        return Err(translation_edit_diagnostic(source, stack_secure_mode_marker_span, "static ELF AT_SECURE addition must publish its exact marker before removal"));
    }
    let clock_tick_rate_stack_construction_name =
        parse_static_image_stack_clock_tick_rate_transition(
            source,
            add_stack_clock_tick_rate,
            allocator_context,
            &secure_mode_stack_construction_name,
            &stack_backing_name,
            true,
        )?;
    let stack_clock_tick_rate_marker_span = statement_span(stack_clock_tick_rate_marker);
    let stack_clock_tick_rate_marker = analyze_operation(
        source,
        stack_clock_tick_rate_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(&stack_clock_tick_rate_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_AUXV_CLKTCK_OK") {
        return Err(translation_edit_diagnostic(source, stack_clock_tick_rate_marker_span, "static ELF AT_CLKTCK addition must publish its exact marker before removal"));
    }
    let platform_stack_construction_name = parse_static_image_stack_platform_transition(
        source,
        add_stack_platform,
        allocator_context,
        &clock_tick_rate_stack_construction_name,
        &stack_backing_name,
        true,
    )?;
    let stack_platform_marker_span = statement_span(stack_platform_marker);
    let stack_platform_marker = analyze_operation(source, stack_platform_marker, CompilerSystemsContextKind::Bootstrap, allocator_context)?;
    if !matches!(&stack_platform_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_AUXV_PLATFORM_OK") {
        return Err(translation_edit_diagnostic(source, stack_platform_marker_span, "static ELF AT_PLATFORM addition must publish its exact marker before removal"));
    }
    let hardware_capabilities_stack_construction_name =
        parse_static_image_stack_hardware_capabilities_transition(
            source,
            add_stack_hardware_capabilities,
            allocator_context,
            &platform_stack_construction_name,
            &stack_backing_name,
            true,
        )?;
    let stack_hardware_capabilities_marker_span =
        statement_span(stack_hardware_capabilities_marker);
    let stack_hardware_capabilities_marker = analyze_operation(
        source,
        stack_hardware_capabilities_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(&stack_hardware_capabilities_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_AUXV_HWCAP_OK") {
        return Err(translation_edit_diagnostic(source, stack_hardware_capabilities_marker_span, "static ELF hardware-capability addition must publish its exact marker before removal"));
    }
    let load_metadata_stack_construction_name =
        parse_static_image_stack_load_metadata_transition(
            source,
            add_stack_load_metadata,
            allocator_context,
            &hardware_capabilities_stack_construction_name,
            &stack_backing_name,
            true,
        )?;
    let stack_load_metadata_marker_span = statement_span(stack_load_metadata_marker);
    let stack_load_metadata_marker = analyze_operation(
        source,
        stack_load_metadata_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(&stack_load_metadata_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_AUXV_LOAD_METADATA_OK") {
        return Err(translation_edit_diagnostic(source, stack_load_metadata_marker_span, "static ELF load-metadata addition must publish its exact marker before removal"));
    }
    let random_slot_authorized_stack_construction_name =
        parse_static_image_stack_random_slot_transition(
            source,
            authorize_stack_random_slot,
            allocator_context,
            &load_metadata_stack_construction_name,
            &stack_backing_name,
            true,
        )?;
    let stack_random_slot_authorized_marker_span =
        statement_span(stack_random_slot_authorized_marker);
    let stack_random_slot_authorized_marker = analyze_operation(
        source,
        stack_random_slot_authorized_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(&stack_random_slot_authorized_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_RANDOM_SLOT_AUTHORIZED_OK") {
        return Err(translation_edit_diagnostic(source, stack_random_slot_authorized_marker_span, "static ELF random-slot authorization must publish its exact marker before revocation"));
    }
    let retained_process_name = parse_static_image_prepared_process_transition(
        source,
        retain_prepared_process,
        allocator_context,
        &random_slot_authorized_stack_construction_name,
        Some(&stack_backing_name),
        true,
    )?;
    let prepared_process_retained_marker_span = statement_span(prepared_process_retained_marker);
    let prepared_process_retained_marker = analyze_operation(
        source,
        prepared_process_retained_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(&prepared_process_retained_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_PREPARED_PROCESS_RETAINED_OK") {
        return Err(translation_edit_diagnostic(source, prepared_process_retained_marker_span, "static ELF prepared-process retention must publish its exact marker before restoration"));
    }
    let retained_process_after_disjoint_work_name =
        parse_static_image_retained_disjoint_work(
            source,
            qualify_retained_disjoint_work,
            allocator_context,
            &retained_process_name,
        )?;
    let retained_disjoint_work_marker_span = statement_span(retained_disjoint_work_marker);
    let retained_disjoint_work_marker = analyze_operation(
        source,
        retained_disjoint_work_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(&retained_disjoint_work_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_RETAINED_DISJOINT_WORK_OK") {
        return Err(translation_edit_diagnostic(source, retained_disjoint_work_marker_span, "static ELF retained disjoint work must publish its exact marker before restoration"));
    }
    let entropy_pending_process_name = parse_static_image_entropy_handoff(
        source, authorize_entropy_handoff, allocator_context,
        &retained_process_after_disjoint_work_name, true,
    )?;
    let entropy_handoff_authorized_marker_span = statement_span(entropy_handoff_authorized_marker);
    let entropy_handoff_authorized_marker = analyze_operation(source, entropy_handoff_authorized_marker, CompilerSystemsContextKind::Bootstrap, allocator_context)?;
    if !matches!(&entropy_handoff_authorized_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_ENTROPY_HANDOFF_AUTHORIZED_OK") {
        return Err(translation_edit_diagnostic(source, entropy_handoff_authorized_marker_span, "static ELF entropy handoff authorization must publish its exact marker before revocation"));
    }
    let entropy_acquiring_process_name = parse_static_image_entropy_acquisition(
        source, authorize_entropy_acquisition, allocator_context,
        &entropy_pending_process_name, true,
    )?;
    let entropy_acquisition_authorized_marker_span = statement_span(entropy_acquisition_authorized_marker);
    let entropy_acquisition_authorized_marker = analyze_operation(source, entropy_acquisition_authorized_marker, CompilerSystemsContextKind::Bootstrap, allocator_context)?;
    if !matches!(&entropy_acquisition_authorized_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_ENTROPY_ACQUISITION_AUTHORIZED_OK") {
        return Err(translation_edit_diagnostic(source, entropy_acquisition_authorized_marker_span, "static ELF entropy acquisition authorization must publish its exact marker before revocation"));
    }
    let entropy_acquiring_process_after_device_name = parse_static_image_entropy_device(
        source, qualify_entropy_device, allocator_context,
        &entropy_acquiring_process_name,
    )?;
    let entropy_device_marker_span = statement_span(entropy_device_marker);
    let entropy_device_marker = analyze_operation(source, entropy_device_marker, CompilerSystemsContextKind::Bootstrap, allocator_context)?;
    if !matches!(&entropy_device_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_ENTROPY_DEVICE_OK") {
        return Err(translation_edit_diagnostic(source, entropy_device_marker_span, "static ELF entropy-device qualification must publish its exact marker before acquisition revocation"));
    }
    let entropy_acquiring_process_after_transport_name = parse_static_image_entropy_transport(
        source, qualify_entropy_transport, allocator_context,
        &entropy_acquiring_process_after_device_name,
    )?;
    let entropy_transport_marker_span = statement_span(entropy_transport_marker);
    let entropy_transport_marker = analyze_operation(source, entropy_transport_marker, CompilerSystemsContextKind::Bootstrap, allocator_context)?;
    if !matches!(&entropy_transport_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_ENTROPY_TRANSPORT_OK") {
        return Err(translation_edit_diagnostic(source, entropy_transport_marker_span, "static ELF entropy-transport qualification must publish its exact marker before acquisition revocation"));
    }
    let entropy_acquiring_process_after_negotiation_name = parse_static_image_entropy_negotiation(
        source, qualify_entropy_negotiation, allocator_context,
        &entropy_acquiring_process_after_transport_name,
    )?;
    let entropy_negotiation_marker_span = statement_span(entropy_negotiation_marker);
    let entropy_negotiation_marker = analyze_operation(source, entropy_negotiation_marker, CompilerSystemsContextKind::Bootstrap, allocator_context)?;
    if !matches!(&entropy_negotiation_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_ENTROPY_NEGOTIATION_OK") {
        return Err(translation_edit_diagnostic(source, entropy_negotiation_marker_span, "static ELF entropy-transport negotiation must publish its exact marker before acquisition revocation"));
    }
    let entropy_pending_process_after_acquisition_rollback_name = parse_static_image_entropy_acquisition(
        source, revoke_entropy_acquisition, allocator_context,
        &entropy_acquiring_process_after_negotiation_name, false,
    )?;
    let entropy_acquisition_revoked_marker_span = statement_span(entropy_acquisition_revoked_marker);
    let entropy_acquisition_revoked_marker = analyze_operation(source, entropy_acquisition_revoked_marker, CompilerSystemsContextKind::Bootstrap, allocator_context)?;
    if !matches!(&entropy_acquisition_revoked_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_ENTROPY_ACQUISITION_REVOKED_OK") {
        return Err(translation_edit_diagnostic(source, entropy_acquisition_revoked_marker_span, "static ELF entropy acquisition revocation must publish its exact marker before handoff revocation"));
    }
    let retained_process_after_entropy_rollback_name = parse_static_image_entropy_handoff(
        source, revoke_entropy_handoff, allocator_context,
        &entropy_pending_process_after_acquisition_rollback_name, false,
    )?;
    let entropy_handoff_revoked_marker_span = statement_span(entropy_handoff_revoked_marker);
    let entropy_handoff_revoked_marker = analyze_operation(source, entropy_handoff_revoked_marker, CompilerSystemsContextKind::Bootstrap, allocator_context)?;
    if !matches!(&entropy_handoff_revoked_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_ENTROPY_HANDOFF_REVOKED_OK") {
        return Err(translation_edit_diagnostic(source, entropy_handoff_revoked_marker_span, "static ELF entropy handoff revocation must publish its exact marker before restoration"));
    }
    let random_slot_authorized_stack_restored_construction_name =
        parse_static_image_prepared_process_transition(
            source,
            restore_prepared_process,
            allocator_context,
            &retained_process_after_entropy_rollback_name,
            None,
            false,
        )?;
    let prepared_process_restored_marker_span = statement_span(prepared_process_restored_marker);
    let prepared_process_restored_marker = analyze_operation(
        source,
        prepared_process_restored_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(&prepared_process_restored_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_PREPARED_PROCESS_RESTORED_OK") {
        return Err(translation_edit_diagnostic(source, prepared_process_restored_marker_span, "static ELF prepared-process restoration must publish its exact marker before random-slot revocation"));
    }
    let load_metadata_stack_restored_construction_name =
        parse_static_image_stack_random_slot_transition(
            source,
            revoke_stack_random_slot,
            allocator_context,
            &random_slot_authorized_stack_restored_construction_name,
            &stack_backing_name,
            false,
        )?;
    let stack_random_slot_revoked_marker_span = statement_span(stack_random_slot_revoked_marker);
    let stack_random_slot_revoked_marker = analyze_operation(
        source,
        stack_random_slot_revoked_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(&stack_random_slot_revoked_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_RANDOM_SLOT_REVOKED_OK") {
        return Err(translation_edit_diagnostic(source, stack_random_slot_revoked_marker_span, "static ELF random-slot revocation must publish its exact marker before load-metadata removal"));
    }
    let hardware_capabilities_stack_restored_construction_name =
        parse_static_image_stack_load_metadata_transition(
            source,
            remove_stack_load_metadata,
            allocator_context,
            &load_metadata_stack_restored_construction_name,
            &stack_backing_name,
            false,
        )?;
    let stack_load_metadata_revoked_marker_span =
        statement_span(stack_load_metadata_revoked_marker);
    let stack_load_metadata_revoked_marker = analyze_operation(
        source,
        stack_load_metadata_revoked_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(&stack_load_metadata_revoked_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_AUXV_LOAD_METADATA_REVOKED_OK") {
        return Err(translation_edit_diagnostic(source, stack_load_metadata_revoked_marker_span, "static ELF load-metadata removal must publish its exact marker before hardware-capability removal"));
    }
    let platform_stack_restored_construction_name =
        parse_static_image_stack_hardware_capabilities_transition(
            source,
            remove_stack_hardware_capabilities,
            allocator_context,
            &hardware_capabilities_stack_restored_construction_name,
            &stack_backing_name,
            false,
        )?;
    let stack_hardware_capabilities_revoked_marker_span =
        statement_span(stack_hardware_capabilities_revoked_marker);
    let stack_hardware_capabilities_revoked_marker = analyze_operation(
        source,
        stack_hardware_capabilities_revoked_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(&stack_hardware_capabilities_revoked_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_AUXV_HWCAP_REVOKED_OK") {
        return Err(translation_edit_diagnostic(source, stack_hardware_capabilities_revoked_marker_span, "static ELF hardware-capability removal must publish its exact marker before AT_PLATFORM removal"));
    }
    let clock_tick_rate_stack_restored_construction_name =
        parse_static_image_stack_platform_transition(
            source,
            remove_stack_platform,
            allocator_context,
            &platform_stack_restored_construction_name,
            &stack_backing_name,
            false,
        )?;
    let stack_platform_revoked_marker_span = statement_span(stack_platform_revoked_marker);
    let stack_platform_revoked_marker = analyze_operation(source, stack_platform_revoked_marker, CompilerSystemsContextKind::Bootstrap, allocator_context)?;
    if !matches!(&stack_platform_revoked_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_AUXV_PLATFORM_REVOKED_OK") {
        return Err(translation_edit_diagnostic(source, stack_platform_revoked_marker_span, "static ELF AT_PLATFORM removal must publish its exact marker before AT_CLKTCK removal"));
    }
    let secure_mode_stack_restored_construction_name =
        parse_static_image_stack_clock_tick_rate_transition(
            source,
            remove_stack_clock_tick_rate,
            allocator_context,
            &clock_tick_rate_stack_restored_construction_name,
            &stack_backing_name,
            false,
        )?;
    let stack_clock_tick_rate_revoked_marker_span =
        statement_span(stack_clock_tick_rate_revoked_marker);
    let stack_clock_tick_rate_revoked_marker = analyze_operation(
        source,
        stack_clock_tick_rate_revoked_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(&stack_clock_tick_rate_revoked_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_AUXV_CLKTCK_REVOKED_OK") {
        return Err(translation_edit_diagnostic(source, stack_clock_tick_rate_revoked_marker_span, "static ELF AT_CLKTCK removal must publish its exact marker before AT_SECURE removal"));
    }
    let credential_stack_restored_construction_name =
        parse_static_image_stack_secure_mode_transition(
            source,
            remove_stack_secure_mode,
            allocator_context,
            &secure_mode_stack_restored_construction_name,
            &stack_backing_name,
            false,
        )?;
    let stack_secure_mode_revoked_marker_span = statement_span(stack_secure_mode_revoked_marker);
    let stack_secure_mode_revoked_marker = analyze_operation(
        source,
        stack_secure_mode_revoked_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(&stack_secure_mode_revoked_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_AUXV_SECURE_REVOKED_OK") {
        return Err(translation_edit_diagnostic(source, stack_secure_mode_revoked_marker_span, "static ELF AT_SECURE removal must publish its exact marker before credential removal"));
    }
    let executable_name_stack_restored_construction_name =
        parse_static_image_stack_credentials_transition(
            source,
            remove_stack_credentials,
            allocator_context,
            &credential_stack_restored_construction_name,
            &stack_backing_name,
            false,
        )?;
    let stack_credentials_revoked_marker_span = statement_span(stack_credentials_revoked_marker);
    let stack_credentials_revoked_marker = analyze_operation(
        source,
        stack_credentials_revoked_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(&stack_credentials_revoked_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_AUXV_CREDENTIALS_REVOKED_OK") {
        return Err(translation_edit_diagnostic(source, stack_credentials_revoked_marker_span, "static ELF credential removal must publish its exact marker before AT_EXECFN removal"));
    }
    let program_header_stack_restored_construction_name =
        parse_static_image_stack_executable_name_transition(
            source,
            remove_stack_executable_name,
            allocator_context,
            &executable_name_stack_restored_construction_name,
            &stack_backing_name,
            false,
        )?;
    let stack_executable_name_revoked_marker_span =
        statement_span(stack_executable_name_revoked_marker);
    let stack_executable_name_revoked_marker = analyze_operation(
        source,
        stack_executable_name_revoked_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(&stack_executable_name_revoked_marker, CompilerSystemsOperation::ConsoleWrite { text } if text == "TOPAL_KERNEL_STATIC_ELF_AUXV_EXECFN_REVOKED_OK") {
        return Err(translation_edit_diagnostic(source, stack_executable_name_revoked_marker_span, "static ELF AT_EXECFN removal must publish its exact marker before program-header removal"));
    }
    let entry_fact_stack_restored_construction_name =
        parse_static_image_stack_program_headers_removal(
            source,
            remove_stack_program_headers,
            allocator_context,
            &program_header_stack_restored_construction_name,
            &stack_backing_name,
        )?;
    let stack_program_headers_revoked_marker_span =
        statement_span(stack_program_headers_revoked_marker);
    let stack_program_headers_revoked_marker = analyze_operation(
        source,
        stack_program_headers_revoked_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &stack_program_headers_revoked_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_AUXV_PROGRAM_HEADERS_REVOKED_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            stack_program_headers_revoked_marker_span,
            "static ELF program-header removal must publish its exact marker before AT_ENTRY removal",
        ));
    }
    let pagesz_stack_entry_restored_construction_name =
        parse_static_image_stack_entry_fact_removal(
            source,
            remove_stack_entry_fact,
            allocator_context,
            &entry_fact_stack_restored_construction_name,
            &stack_backing_name,
        )?;
    let stack_entry_fact_revoked_marker_span = statement_span(stack_entry_fact_revoked_marker);
    let stack_entry_fact_revoked_marker = analyze_operation(
        source,
        stack_entry_fact_revoked_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &stack_entry_fact_revoked_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_AUXV_ENTRY_REVOKED_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            stack_entry_fact_revoked_marker_span,
            "static ELF AT_ENTRY removal must publish its exact marker before AT_PAGESZ removal",
        ));
    }
    let initialized_stack_entry_restored_construction_name =
        parse_static_image_stack_pagesz_removal(
            source,
            remove_stack_pagesz,
            allocator_context,
            &pagesz_stack_entry_restored_construction_name,
            &stack_backing_name,
        )?;
    let stack_pagesz_revoked_marker_span = statement_span(stack_pagesz_revoked_marker);
    let stack_pagesz_revoked_marker = analyze_operation(
        source,
        stack_pagesz_revoked_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &stack_pagesz_revoked_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_AUXV_PAGESZ_REVOKED_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            stack_pagesz_revoked_marker_span,
            "static ELF AT_PAGESZ removal must publish its exact marker before stack-entry clearing",
        ));
    }
    let published_stack_leaf_restored_construction_name = parse_static_image_stack_entry_clearing(
        source,
        clear_stack_entry,
        allocator_context,
        &initialized_stack_entry_restored_construction_name,
        &stack_backing_name,
    )?;
    let stack_entry_cleared_marker_span = statement_span(stack_entry_cleared_marker);
    let stack_entry_cleared_marker = analyze_operation(
        source,
        stack_entry_cleared_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &stack_entry_cleared_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_STACK_ENTRY_CLEARED_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            stack_entry_cleared_marker_span,
            "static ELF stack-entry clearing must publish its exact marker before leaf revocation",
        ));
    }
    let zeroed_stack_backing_restored_construction_name =
        parse_static_image_stack_leaf_revocation(
            source,
            revoke_stack_leaf,
            allocator_context,
            &published_stack_leaf_restored_construction_name,
            &stack_backing_name,
        )?;
    let stack_leaf_revoked_marker_span = statement_span(stack_leaf_revoked_marker);
    let stack_leaf_revoked_marker = analyze_operation(
        source,
        stack_leaf_revoked_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &stack_leaf_revoked_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_STACK_LEAF_REVOKED_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            stack_leaf_revoked_marker_span,
            "static ELF stack-leaf revocation must publish its exact marker before backing release",
        ));
    }
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
    let stack_slot_revoked_marker_span = statement_span(stack_slot_revoked_marker);
    let stack_slot_revoked_marker = analyze_operation(
        source,
        stack_slot_revoked_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &stack_slot_revoked_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_STACK_SLOT_AUTHORITY_REVOKED_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            stack_slot_revoked_marker_span,
            "static ELF stack-slot revocation must publish its exact marker before entry revocation",
        ));
    }
    let entry_restored_construction_name = parse_static_image_entry_transition(
        source,
        revoke_entry,
        allocator_context,
        &stack_slot_restored_construction_name,
        "revoke",
        "bounded-entry-qualification-rollback",
    )?;
    let entry_revoked_marker_span = statement_span(entry_revoked_marker);
    let entry_revoked_marker = analyze_operation(
        source,
        entry_revoked_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &entry_revoked_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_ENTRY_AUTHORITY_REVOKED_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            entry_revoked_marker_span,
            "static ELF entry revocation must publish its exact marker before detachment",
        ));
    }
    let detached_construction_name = parse_static_image_address_space_detach(
        source,
        detach,
        allocator_context,
        &entry_restored_construction_name,
    )?;
    let detach_marker_span = statement_span(detach_marker);
    let detach_marker = analyze_operation(
        source,
        detach_marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    if !matches!(
        &detach_marker,
        CompilerSystemsOperation::ConsoleWrite { text }
            if text == "TOPAL_KERNEL_STATIC_ELF_ADDRESS_SPACE_DETACHED_OK"
    ) {
        return Err(translation_edit_diagnostic(
            source,
            detach_marker_span,
            "static ELF address-space detachment must publish its exact marker before hierarchy reclaim",
        ));
    }
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
                frame_count: 4,
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
                frame_count: 1,
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

fn parse_static_image_hierarchy_reservation(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image hierarchy reservation cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image hierarchy reservation must bind one affine extent",
            ));
        }
    };
    let [receiver, static_word, image, construction, reserve, hierarchy, Expression::Product { fields, .. }] =
        items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image hierarchy reservation requires its exact owner and four-frame shape",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, reserve, "reserve")
        || !identifier_is(source, hierarchy, "hierarchy")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image hierarchy reservation requires its exact owner and four-frame shape",
        ));
    }
    let mut owner = None;
    let mut frame_count = None;
    let mut alignment_frames = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image hierarchy-reservation parameters must be named",
            ));
        };
        match source.slice(label) {
            "owner" => {
                let Expression::Identifier(value) = field.value else {
                    return Err(translation_edit_diagnostic(
                        source,
                        field.value.span(),
                        "static-image hierarchy owner must be the live sealed identity",
                    ));
                };
                owner = Some(source.slice(value));
            }
            "frame-count" => frame_count = Some(parse_storage_natural(source, &field.value)?),
            "alignment-frames" => {
                alignment_frames = Some(parse_storage_natural(source, &field.value)?);
            }
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image hierarchy-reservation parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name) || frame_count != Some(4) || alignment_frames != Some(1) {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image hierarchy reservation requires the exact sealed construction and four unit-aligned frames",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_address_space_attach(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    hierarchy_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image address-space attachment cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image address-space attachment must refine one sealed construction",
            ));
        }
    };
    let [receiver, static_word, image, construction, attach, address, space, Expression::Product { fields, .. }] =
        items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image address-space attachment requires its exact owner, hierarchy, and layout",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, attach, "attach")
        || !identifier_is(source, address, "address")
        || !identifier_is(source, space, "space")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image address-space attachment requires its exact owner, hierarchy, and layout",
        ));
    }
    let mut owner = None;
    let mut hierarchy = None;
    let mut layout = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image address-space parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "hierarchy" => hierarchy = Some(source.slice(*value)),
            "layout" => layout = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image address-space attachment parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name)
        || hierarchy != Some(hierarchy_name)
        || layout != Some("bounded-static-elf-inactive-address-space")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image address-space attachment requires the exact sealed construction, hierarchy extent, and inactive layout",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_address_space_detach(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image address-space detachment cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image address-space detachment must refine one attached construction",
            ));
        }
    };
    let [receiver, static_word, image, construction, detach, address, space, Expression::Product { fields, .. }] =
        items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image address-space detachment requires its exact owner",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, detach, "detach")
        || !identifier_is(source, address, "address")
        || !identifier_is(source, space, "space")
        || fields.len() != 1
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image address-space detachment requires its exact owner",
        ));
    }
    let field = &fields[0];
    let (Some(label), Expression::Identifier(owner)) = (field.label, &field.value) else {
        return Err(translation_edit_diagnostic(
            source,
            field.value.span(),
            "static-image address-space detachment owner must be a named identity",
        ));
    };
    if source.slice(label) != "owner" || source.slice(*owner) != construction_name {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image address-space detachment requires the exact restored construction",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_address_space_transition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    operation_name: &str,
    policy_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image address-space transition cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image address-space transition must refine one live construction",
            ));
        }
    };
    let [receiver, static_word, image, construction, operation, address, space, Expression::Product { fields, .. }] =
        items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image address-space transition requires its exact owner and policy",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, operation, operation_name)
        || !identifier_is(source, address, "address")
        || !identifier_is(source, space, "space")
        || fields.len() != 2
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image address-space transition requires its exact owner and policy",
        ));
    }
    let mut owner = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image address-space transition parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "policy" => policy = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image address-space transition parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name) || policy != Some(policy_name) {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image address-space transition requires the exact owner and continuation policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entry_transition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    operation_name: &str,
    policy_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image entry transition cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image entry transition must refine one live construction",
            ));
        }
    };
    let [receiver, static_word, image, construction, operation, entry, Expression::Product { fields, .. }] =
        items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entry transition requires its exact owner and policy",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, operation, operation_name)
        || !identifier_is(source, entry, "entry")
        || fields.len() != 2
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entry transition requires its exact owner and policy",
        ));
    }
    let mut owner = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entry transition parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "policy" => policy = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image entry transition parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name) || policy != Some(policy_name) {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entry transition requires the exact owner and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_slot_transition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    operation_name: &str,
    policy_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image stack-slot transition cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image stack-slot transition must refine one live construction",
            ));
        }
    };
    let [receiver, static_word, image, construction, operation, stack, slot, Expression::Product { fields, .. }] =
        items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-slot transition requires its exact owner and policy",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, operation, operation_name)
        || !identifier_is(source, stack, "stack")
        || !identifier_is(source, slot, "slot")
        || fields.len() != 2
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-slot transition requires its exact owner and policy",
        ));
    }
    let mut owner = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image stack-slot transition parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "policy" => policy = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image stack-slot transition parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name) || policy != Some(policy_name) {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-slot transition requires the exact owner and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_backing_reservation(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image stack-backing reservation cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image stack-backing reservation must bind one affine extent",
            ));
        }
    };
    let [receiver, static_word, image, construction, reserve, stack, backing, Expression::Product { fields, .. }] =
        items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-backing reservation requires its exact owner and one-frame shape",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, reserve, "reserve")
        || !identifier_is(source, stack, "stack")
        || !identifier_is(source, backing, "backing")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-backing reservation requires its exact owner and one-frame shape",
        ));
    }
    let mut owner = None;
    let mut frame_count = None;
    let mut alignment_frames = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image stack-backing reservation parameters must be named",
            ));
        };
        match source.slice(label) {
            "owner" => {
                let Expression::Identifier(value) = field.value else {
                    return Err(translation_edit_diagnostic(
                        source,
                        field.value.span(),
                        "static-image stack-backing owner must be the live slot identity",
                    ));
                };
                owner = Some(source.slice(value));
            }
            "frame-count" => frame_count = Some(parse_storage_natural(source, &field.value)?),
            "alignment-frames" => {
                alignment_frames = Some(parse_storage_natural(source, &field.value)?);
            }
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image stack-backing reservation parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name) || frame_count != Some(1) || alignment_frames != Some(1) {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-backing reservation requires the exact slot-authorized construction and one unit-aligned frame",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_backing_zero(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image stack-backing zeroing cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image stack-backing zeroing must refine one live construction",
            ));
        }
    };
    let [receiver, static_word, image, construction, zero, stack, backing, Expression::Product { fields, .. }] =
        items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-backing zeroing requires its exact owner, backing, and complete extent",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, zero, "zero")
        || !identifier_is(source, stack, "stack")
        || !identifier_is(source, backing, "backing")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-backing zeroing requires its exact owner, backing, and complete extent",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut extent = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image stack-backing zeroing parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "backing" => backing = Some(source.slice(*value)),
            "extent" => extent = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image stack-backing zeroing parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || extent != Some("complete")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-backing zeroing requires the exact owner, backing, and complete extent",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_leaf_publication(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image stack-leaf publication cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image stack-leaf publication must refine one live construction",
            ));
        }
    };
    let [receiver, static_word, image, construction, publish, stack, leaf, Expression::Product { fields, .. }] =
        items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-leaf publication requires its exact owner, backing, access, and complete extent",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, publish, "publish")
        || !identifier_is(source, stack, "stack")
        || !identifier_is(source, leaf, "leaf")
        || fields.len() != 4
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-leaf publication requires its exact owner, backing, access, and complete extent",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut access = None;
    let mut extent = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image stack-leaf publication parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "backing" => backing = Some(source.slice(*value)),
            "access" => access = Some(source.slice(*value)),
            "extent" => extent = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image stack-leaf publication parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || access != Some("writable-non-executable")
        || extent != Some("complete")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-leaf publication requires the exact owner, backing, RW/NX access, and complete extent",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_leaf_revocation(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image stack-leaf revocation cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image stack-leaf revocation must refine one live construction",
            ));
        }
    };
    let [receiver, static_word, image, construction, revoke, stack, leaf, Expression::Product { fields, .. }] =
        items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-leaf revocation requires its exact owner, backing, and rollback policy",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, revoke, "revoke")
        || !identifier_is(source, stack, "stack")
        || !identifier_is(source, leaf, "leaf")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-leaf revocation requires its exact owner, backing, and rollback policy",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image stack-leaf revocation parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "backing" => backing = Some(source.slice(*value)),
            "policy" => policy = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image stack-leaf revocation parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || policy != Some("bounded-stack-leaf-qualification-rollback")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-leaf revocation requires the exact owner, backing, and rollback policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_entry_initialization(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image stack-entry initialization cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image stack-entry initialization must refine one published stack leaf",
            ));
        }
    };
    let [receiver, static_word, image, construction, initialize, stack, entry, Expression::Product { fields, .. }] =
        items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-entry initialization requires its exact owner, backing, profile, and complete extent",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, initialize, "initialize")
        || !identifier_is(source, stack, "stack")
        || !identifier_is(source, entry, "entry")
        || fields.len() != 4
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-entry initialization requires its exact owner, backing, profile, and complete extent",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut profile = None;
    let mut extent = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image stack-entry initialization parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "backing" => backing = Some(source.slice(*value)),
            "profile" => profile = Some(source.slice(*value)),
            "extent" => extent = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image stack-entry initialization parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || profile != Some("bounded-static-elf-busybox-true")
        || extent != Some("complete")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-entry initialization requires the exact owner, backing, sealed profile, and complete extent",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_pagesz_addition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image AT_PAGESZ addition must refine one initialized stack entry",
            ));
        }
    };
    let [receiver, static_word, image, construction, add, stack, pagesz, Expression::Product { fields, .. }] =
        items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_PAGESZ addition requires its exact owner, backing, value, and policy",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, add, "add")
        || !identifier_is(source, stack, "stack")
        || !identifier_is(source, pagesz, "pagesz")
        || fields.len() != 4
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_PAGESZ addition requires its exact owner, backing, value, and policy",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut value = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(identity)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image AT_PAGESZ addition parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "backing" => backing = Some(source.slice(*identity)),
            "value" => value = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image AT_PAGESZ addition parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || value != Some("provider-base-page")
        || policy != Some("bounded-linux-auxv-before-null")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_PAGESZ addition requires the exact owner, backing, provider page value, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_entry_fact_addition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image AT_ENTRY addition must refine one page-size-qualified stack entry",
            ));
        }
    };
    let [receiver, static_word, image, construction, add, stack, entry, fact, Expression::Product { fields, .. }] =
        items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_ENTRY addition requires its exact owner, backing, entry value, and policy",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, add, "add")
        || !identifier_is(source, stack, "stack")
        || !identifier_is(source, entry, "entry")
        || !identifier_is(source, fact, "fact")
        || fields.len() != 4
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_ENTRY addition requires its exact owner, backing, entry value, and policy",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut value = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(identity)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image AT_ENTRY addition parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "backing" => backing = Some(source.slice(*identity)),
            "value" => value = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image AT_ENTRY addition parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || value != Some("authorized-static-elf-entry")
        || policy != Some("bounded-linux-auxv-before-null")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_ENTRY addition requires the exact owner, backing, authorized entry value, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_executable_name_transition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
    add: bool,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation { name, classifier: Expression::Application { items, span }, declarations, .. }
            if declarations.is_empty() => (*name, items, *span),
        Statement::Binding { name, classifier: None, value: Expression::Application { items, span } } => (*name, items, *span),
        _ => return Err(translation_edit_diagnostic(source, statement_span(statement), "static-image executable-name transition must refine one exact stack entry")),
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(source, span, "static-image executable-name transition requires named parameters"));
    };
    let verb = if add { "add" } else { "remove" };
    if items.len() != 9
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], verb)
        || !identifier_is(source, &items[5], "stack")
        || !identifier_is(source, &items[6], "executable")
        || !identifier_is(source, &items[7], "name")
        || fields.len() != if add { 4 } else { 3 }
    {
        return Err(translation_edit_diagnostic(source, span, "static-image executable-name transition requires its exact owner, backing, value, and policy"));
    }
    let mut owner = None;
    let mut backing = None;
    let mut value = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(identity)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(source, field.value.span(), "static-image executable-name parameters must be named identities"));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "backing" => backing = Some(source.slice(*identity)),
            "value" => value = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => return Err(translation_edit_diagnostic(source, label, "unknown static-image executable-name parameter")),
        }
    }
    let expected_value = add.then_some("initial-executable-name");
    let expected_policy = if add { "bounded-linux-auxv-before-null" } else { "bounded-stack-executable-name-qualification-rollback" };
    if owner != Some(construction_name) || backing != Some(backing_name) || value != expected_value || policy != Some(expected_policy) {
        return Err(translation_edit_diagnostic(source, span, "static-image executable-name transition requires the exact owner, backing, executable name, and policy"));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_credentials_transition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
    add: bool,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation { name, classifier: Expression::Application { items, span }, declarations, .. }
            if declarations.is_empty() => (*name, items, *span),
        Statement::Binding { name, classifier: None, value: Expression::Application { items, span } } => (*name, items, *span),
        _ => return Err(translation_edit_diagnostic(source, statement_span(statement), "static-image credential transition must refine one exact stack entry")),
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(source, span, "static-image credential transition requires named parameters"));
    };
    let verb = if add { "add" } else { "remove" };
    if items.len() != 8
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], verb)
        || !identifier_is(source, &items[5], "stack")
        || !identifier_is(source, &items[6], "credentials")
        || fields.len() != if add { 4 } else { 3 }
    {
        return Err(translation_edit_diagnostic(source, span, "static-image credential transition requires its exact owner, backing, value, and policy"));
    }
    let mut owner = None;
    let mut backing = None;
    let mut value = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(identity)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(source, field.value.span(), "static-image credential parameters must be named identities"));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "backing" => backing = Some(source.slice(*identity)),
            "value" => value = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => return Err(translation_edit_diagnostic(source, label, "unknown static-image credential parameter")),
        }
    }
    let expected_value = add.then_some("initial-process-credentials");
    let expected_policy = if add { "bounded-linux-auxv-before-null" } else { "bounded-stack-credentials-qualification-rollback" };
    if owner != Some(construction_name) || backing != Some(backing_name) || value != expected_value || policy != Some(expected_policy) {
        return Err(translation_edit_diagnostic(source, span, "static-image credential transition requires the exact owner, backing, process credentials, and policy"));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_secure_mode_transition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
    add: bool,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation { name, classifier: Expression::Application { items, span }, declarations, .. }
            if declarations.is_empty() => (*name, items, *span),
        Statement::Binding { name, classifier: None, value: Expression::Application { items, span } } => (*name, items, *span),
        _ => return Err(translation_edit_diagnostic(source, statement_span(statement), "static-image secure-mode transition must refine one exact stack entry")),
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(source, span, "static-image secure-mode transition requires named parameters"));
    };
    let verb = if add { "add" } else { "remove" };
    if items.len() != 9
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], verb)
        || !identifier_is(source, &items[5], "stack")
        || !identifier_is(source, &items[6], "secure")
        || !identifier_is(source, &items[7], "mode")
        || fields.len() != if add { 4 } else { 3 }
    {
        return Err(translation_edit_diagnostic(source, span, "static-image secure-mode transition requires its exact owner, backing, value, and policy"));
    }
    let mut owner = None;
    let mut backing = None;
    let mut value = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(identity)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(source, field.value.span(), "static-image secure-mode parameters must be named identities"));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "backing" => backing = Some(source.slice(*identity)),
            "value" => value = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => return Err(translation_edit_diagnostic(source, label, "unknown static-image secure-mode parameter")),
        }
    }
    let expected_value = add.then_some("initial-execution-security");
    let expected_policy = if add { "bounded-linux-auxv-before-null" } else { "bounded-stack-secure-mode-qualification-rollback" };
    if owner != Some(construction_name) || backing != Some(backing_name) || value != expected_value || policy != Some(expected_policy) {
        return Err(translation_edit_diagnostic(source, span, "static-image secure-mode transition requires the exact owner, backing, execution security, and policy"));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_clock_tick_rate_transition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
    add: bool,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation { name, classifier: Expression::Application { items, span }, declarations, .. }
            if declarations.is_empty() => (*name, items, *span),
        Statement::Binding { name, classifier: None, value: Expression::Application { items, span } } => (*name, items, *span),
        _ => return Err(translation_edit_diagnostic(source, statement_span(statement), "static-image clock-tick-rate transition must refine one exact stack entry")),
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(source, span, "static-image clock-tick-rate transition requires named parameters"));
    };
    let verb = if add { "add" } else { "remove" };
    if items.len() != 10
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], verb)
        || !identifier_is(source, &items[5], "stack")
        || !identifier_is(source, &items[6], "clock")
        || !identifier_is(source, &items[7], "tick")
        || !identifier_is(source, &items[8], "rate")
        || fields.len() != if add { 4 } else { 3 }
    {
        return Err(translation_edit_diagnostic(source, span, "static-image clock-tick-rate transition requires its exact owner, backing, value, and policy"));
    }
    let mut owner = None;
    let mut backing = None;
    let mut value = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(identity)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(source, field.value.span(), "static-image clock-tick-rate parameters must be named identities"));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "backing" => backing = Some(source.slice(*identity)),
            "value" => value = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => return Err(translation_edit_diagnostic(source, label, "unknown static-image clock-tick-rate parameter")),
        }
    }
    let expected_value = add.then_some("linux-user-hz");
    let expected_policy = if add { "bounded-linux-auxv-before-null" } else { "bounded-stack-clock-tick-rate-qualification-rollback" };
    if owner != Some(construction_name) || backing != Some(backing_name) || value != expected_value || policy != Some(expected_policy) {
        return Err(translation_edit_diagnostic(source, span, "static-image clock-tick-rate transition requires the exact owner, backing, Linux userspace accounting rate, and policy"));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_platform_transition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
    add: bool,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation { name, classifier: Expression::Application { items, span }, declarations, .. } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding { name, classifier: None, value: Expression::Application { items, span } } => (*name, items, *span),
        _ => return Err(translation_edit_diagnostic(source, statement_span(statement), "static-image platform transition must refine one exact stack entry")),
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(source, span, "static-image platform transition requires named parameters"));
    };
    let verb = if add { "add" } else { "remove" };
    if items.len() != 8 || !identifier_is(source, &items[0], context_name) || !identifier_is(source, &items[1], "static") || !identifier_is(source, &items[2], "image") || !identifier_is(source, &items[3], "construction") || !identifier_is(source, &items[4], verb) || !identifier_is(source, &items[5], "stack") || !identifier_is(source, &items[6], "platform") || fields.len() != if add { 4 } else { 3 } {
        return Err(translation_edit_diagnostic(source, span, "static-image platform transition requires its exact owner, backing, value, and policy"));
    }
    let mut owner = None;
    let mut backing = None;
    let mut value = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(identity)) = (field.label, &field.value) else { return Err(translation_edit_diagnostic(source, field.value.span(), "static-image platform parameters must be named identities")); };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "backing" => backing = Some(source.slice(*identity)),
            "value" => value = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => return Err(translation_edit_diagnostic(source, label, "unknown static-image platform parameter")),
        }
    }
    let expected_value = add.then_some("native-userspace-platform");
    let expected_policy = if add { "bounded-linux-auxv-before-null" } else { "bounded-stack-platform-qualification-rollback" };
    if owner != Some(construction_name) || backing != Some(backing_name) || value != expected_value || policy != Some(expected_policy) {
        return Err(translation_edit_diagnostic(source, span, "static-image platform transition requires the exact owner, backing, native userspace platform, and policy"));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_hardware_capabilities_transition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
    add: bool,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation { name, classifier: Expression::Application { items, span }, declarations, .. } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding { name, classifier: None, value: Expression::Application { items, span } } => (*name, items, *span),
        _ => return Err(translation_edit_diagnostic(source, statement_span(statement), "static-image hardware-capability transition must refine one exact stack entry")),
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(source, span, "static-image hardware-capability transition requires named parameters"));
    };
    let verb = if add { "add" } else { "remove" };
    if items.len() != 9 || !identifier_is(source, &items[0], context_name) || !identifier_is(source, &items[1], "static") || !identifier_is(source, &items[2], "image") || !identifier_is(source, &items[3], "construction") || !identifier_is(source, &items[4], verb) || !identifier_is(source, &items[5], "stack") || !identifier_is(source, &items[6], "hardware") || !identifier_is(source, &items[7], "capabilities") || fields.len() != if add { 4 } else { 3 } {
        return Err(translation_edit_diagnostic(source, span, "static-image hardware-capability transition requires its exact owner, backing, value, and policy"));
    }
    let mut owner = None;
    let mut backing = None;
    let mut value = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(identity)) = (field.label, &field.value) else { return Err(translation_edit_diagnostic(source, field.value.span(), "static-image hardware-capability parameters must be named identities")); };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "backing" => backing = Some(source.slice(*identity)),
            "value" => value = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => return Err(translation_edit_diagnostic(source, label, "unknown static-image hardware-capability parameter")),
        }
    }
    let expected_value = add.then_some("native-userspace-hardware-capabilities");
    let expected_policy = if add { "bounded-linux-auxv-before-null" } else { "bounded-stack-hardware-capabilities-qualification-rollback" };
    if owner != Some(construction_name) || backing != Some(backing_name) || value != expected_value || policy != Some(expected_policy) {
        return Err(translation_edit_diagnostic(source, span, "static-image hardware-capability transition requires the exact owner, backing, native userspace capabilities, and policy"));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_load_metadata_transition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
    add: bool,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation { name, classifier: Expression::Application { items, span }, declarations, .. } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding { name, classifier: None, value: Expression::Application { items, span } } => (*name, items, *span),
        _ => return Err(translation_edit_diagnostic(source, statement_span(statement), "static-image load-metadata transition must refine one exact stack entry")),
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(source, span, "static-image load-metadata transition requires named parameters"));
    };
    let verb = if add { "add" } else { "remove" };
    if items.len() != 9 || !identifier_is(source, &items[0], context_name) || !identifier_is(source, &items[1], "static") || !identifier_is(source, &items[2], "image") || !identifier_is(source, &items[3], "construction") || !identifier_is(source, &items[4], verb) || !identifier_is(source, &items[5], "stack") || !identifier_is(source, &items[6], "load") || !identifier_is(source, &items[7], "metadata") || fields.len() != if add { 4 } else { 3 } {
        return Err(translation_edit_diagnostic(source, span, "static-image load-metadata transition requires its exact owner, backing, value, and policy"));
    }
    let mut owner = None;
    let mut backing = None;
    let mut value = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(identity)) = (field.label, &field.value) else { return Err(translation_edit_diagnostic(source, field.value.span(), "static-image load-metadata parameters must be named identities")); };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "backing" => backing = Some(source.slice(*identity)),
            "value" => value = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => return Err(translation_edit_diagnostic(source, label, "unknown static-image load-metadata parameter")),
        }
    }
    let expected_value = add.then_some("static-executable-load-metadata");
    let expected_policy = if add { "bounded-linux-auxv-before-null" } else { "bounded-stack-load-metadata-qualification-rollback" };
    if owner != Some(construction_name) || backing != Some(backing_name) || value != expected_value || policy != Some(expected_policy) {
        return Err(translation_edit_diagnostic(source, span, "static-image load-metadata transition requires the exact owner, backing, static executable metadata, and policy"));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_handoff(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    owner_name: &str,
    authorize: bool,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation { name, classifier: Expression::Application { items, span }, declarations, .. } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding { name, classifier: None, value: Expression::Application { items, span } } => (*name, items, *span),
        _ => return Err(translation_edit_diagnostic(source, statement_span(statement), "static-image entropy handoff requires one exact affine owner")),
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(source, span, "static-image entropy handoff requires named parameters"));
    };
    let verb = if authorize { "authorize" } else { "revoke" };
    if items.len() != 8 || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static") || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction") || !identifier_is(source, &items[4], verb)
        || !identifier_is(source, &items[5], "entropy") || !identifier_is(source, &items[6], "handoff")
        || fields.len() != if authorize { 3 } else { 2 }
    {
        return Err(translation_edit_diagnostic(source, span, "static-image entropy handoff requires its exact owner, process identity, and policy"));
    }
    let mut owner = None;
    let mut process = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else { return Err(translation_edit_diagnostic(source, field.value.span(), "static-image entropy-handoff parameters must be named")); };
        let Expression::Identifier(identity) = &field.value else { return Err(translation_edit_diagnostic(source, field.value.span(), "static-image entropy-handoff parameters must be identifiers")); };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "process" => process = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => return Err(translation_edit_diagnostic(source, label, "unknown static-image entropy-handoff parameter")),
        }
    }
    let expected_process = authorize.then_some("bounded-static-elf-initial-process");
    let expected_policy = if authorize { "bounded-static-elf-initial-random-delivery" } else { "bounded-static-elf-entropy-handoff-qualification-rollback" };
    if owner != Some(owner_name) || process != expected_process || policy != Some(expected_policy) {
        return Err(translation_edit_diagnostic(source, span, "static-image entropy handoff requires the exact owner, process identity, and policy"));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_acquisition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    owner_name: &str,
    authorize: bool,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation { name, classifier: Expression::Application { items, span }, declarations, .. } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding { name, classifier: None, value: Expression::Application { items, span } } => (*name, items, *span),
        _ => return Err(translation_edit_diagnostic(source, statement_span(statement), "static-image entropy acquisition requires one exact affine owner")),
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(source, span, "static-image entropy acquisition requires named parameters"));
    };
    let verb = if authorize { "authorize" } else { "revoke" };
    if items.len() != 8 || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static") || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction") || !identifier_is(source, &items[4], verb)
        || !identifier_is(source, &items[5], "entropy") || !identifier_is(source, &items[6], "acquisition")
        || fields.len() != if authorize { 4 } else { 2 }
    {
        return Err(translation_edit_diagnostic(source, span, "static-image entropy acquisition requires its exact owner, output identity, generation, and policy"));
    }
    let mut owner = None;
    let mut output_identity = None;
    let mut generation = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else { return Err(translation_edit_diagnostic(source, field.value.span(), "static-image entropy-acquisition parameters must be named")); };
        let Expression::Identifier(identity) = &field.value else { return Err(translation_edit_diagnostic(source, field.value.span(), "static-image entropy-acquisition parameters must be identifiers")); };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "output" => output_identity = Some(source.slice(*identity)),
            "generation" => generation = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => return Err(translation_edit_diagnostic(source, label, "unknown static-image entropy-acquisition parameter")),
        }
    }
    let expected_output = authorize.then_some("initial-entropy-output");
    let expected_generation = authorize.then_some("generation-two");
    let expected_policy = if authorize { "bounded-static-elf-initial-entropy-acquisition" } else { "bounded-static-elf-entropy-acquisition-qualification-rollback" };
    if owner != Some(owner_name) || output_identity != expected_output || generation != expected_generation || policy != Some(expected_policy) {
        return Err(translation_edit_diagnostic(source, span, "static-image entropy acquisition requires the exact owner, output identity, generation, and policy"));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_device(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    owner_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation { name, classifier: Expression::Application { items, span }, declarations, .. } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding { name, classifier: None, value: Expression::Application { items, span } } => (*name, items, *span),
        _ => return Err(translation_edit_diagnostic(source, statement_span(statement), "static-image entropy-device qualification requires one exact affine owner")),
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(source, span, "static-image entropy-device qualification requires named parameters"));
    };
    if items.len() != 8 || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static") || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction") || !identifier_is(source, &items[4], "qualify")
        || !identifier_is(source, &items[5], "entropy") || !identifier_is(source, &items[6], "device")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(source, span, "static-image entropy-device qualification requires its exact owner, device identity, and policy"));
    }
    let mut owner = None;
    let mut device = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else { return Err(translation_edit_diagnostic(source, field.value.span(), "static-image entropy-device parameters must be named")); };
        let Expression::Identifier(identity) = &field.value else { return Err(translation_edit_diagnostic(source, field.value.span(), "static-image entropy-device parameters must be identifiers")); };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "device" => device = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => return Err(translation_edit_diagnostic(source, label, "unknown static-image entropy-device parameter")),
        }
    }
    if owner != Some(owner_name)
        || device != Some("initial-virtio-rng-device")
        || policy != Some("bounded-static-elf-entropy-device-coexistence")
    {
        return Err(translation_edit_diagnostic(source, span, "static-image entropy-device qualification requires the exact owner, initial entropy device, and policy"));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_transport(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    owner_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation { name, classifier: Expression::Application { items, span }, declarations, .. } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding { name, classifier: None, value: Expression::Application { items, span } } => (*name, items, *span),
        _ => return Err(translation_edit_diagnostic(source, statement_span(statement), "static-image entropy-transport qualification requires one exact affine owner")),
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(source, span, "static-image entropy-transport qualification requires named parameters"));
    };
    if items.len() != 8 || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static") || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction") || !identifier_is(source, &items[4], "qualify")
        || !identifier_is(source, &items[5], "entropy") || !identifier_is(source, &items[6], "transport")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(source, span, "static-image entropy-transport qualification requires its exact owner, transport identity, and policy"));
    }
    let mut owner = None;
    let mut transport = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else { return Err(translation_edit_diagnostic(source, field.value.span(), "static-image entropy-transport parameters must be named")); };
        let Expression::Identifier(identity) = &field.value else { return Err(translation_edit_diagnostic(source, field.value.span(), "static-image entropy-transport parameters must be identifiers")); };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "transport" => transport = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => return Err(translation_edit_diagnostic(source, label, "unknown static-image entropy-transport parameter")),
        }
    }
    if owner != Some(owner_name)
        || transport != Some("initial-virtio-rng-pci-transport")
        || policy != Some("bounded-static-elf-entropy-transport-discovery")
    {
        return Err(translation_edit_diagnostic(source, span, "static-image entropy-transport qualification requires the exact owner, initial entropy transport, and policy"));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_negotiation(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    owner_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation { name, classifier: Expression::Application { items, span }, declarations, .. } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding { name, classifier: None, value: Expression::Application { items, span } } => (*name, items, *span),
        _ => return Err(translation_edit_diagnostic(source, statement_span(statement), "static-image entropy-transport negotiation requires one exact affine owner")),
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(source, span, "static-image entropy-transport negotiation requires named parameters"));
    };
    if items.len() != 8 || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static") || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction") || !identifier_is(source, &items[4], "qualify")
        || !identifier_is(source, &items[5], "entropy") || !identifier_is(source, &items[6], "negotiation")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(source, span, "static-image entropy-transport negotiation requires its exact owner, transport identity, and policy"));
    }
    let mut owner = None;
    let mut transport = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else { return Err(translation_edit_diagnostic(source, field.value.span(), "static-image entropy-transport negotiation parameters must be named")); };
        let Expression::Identifier(identity) = &field.value else { return Err(translation_edit_diagnostic(source, field.value.span(), "static-image entropy-transport negotiation parameters must be identifiers")); };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "transport" => transport = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => return Err(translation_edit_diagnostic(source, label, "unknown static-image entropy-transport negotiation parameter")),
        }
    }
    if owner != Some(owner_name)
        || transport != Some("initial-virtio-rng-pci-transport")
        || policy != Some("bounded-static-elf-entropy-transport-negotiation")
    {
        return Err(translation_edit_diagnostic(source, span, "static-image entropy-transport negotiation requires the exact owner, initial entropy transport, and policy"));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_retained_disjoint_work(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    owner_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image retained disjoint work requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(source, span, "static-image retained disjoint work requires named parameters"));
    };
    if items.len() != 9
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], "qualify")
        || !identifier_is(source, &items[5], "retained")
        || !identifier_is(source, &items[6], "disjoint")
        || !identifier_is(source, &items[7], "work")
        || fields.len() != 4
    {
        return Err(translation_edit_diagnostic(source, span, "static-image retained disjoint work requires its exact owner, one-frame request, and policy"));
    }
    let mut owner = None;
    let mut frame_count = None;
    let mut alignment_frames = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(source, field.value.span(), "static-image retained disjoint-work parameters must be named"));
        };
        match source.slice(label) {
            "owner" | "policy" => {
                let Expression::Identifier(identity) = &field.value else {
                    return Err(translation_edit_diagnostic(source, field.value.span(), "static-image retained disjoint-work identities must be identifiers"));
                };
                if source.slice(label) == "owner" {
                    owner = Some(source.slice(*identity));
                } else {
                    policy = Some(source.slice(*identity));
                }
            }
            "frame-count" => frame_count = Some(parse_storage_natural(source, &field.value)?),
            "alignment-frames" => alignment_frames = Some(parse_storage_natural(source, &field.value)?),
            _ => return Err(translation_edit_diagnostic(source, label, "unknown static-image retained disjoint-work parameter")),
        }
    }
    if owner != Some(owner_name)
        || frame_count != Some(1)
        || alignment_frames != Some(1)
        || policy != Some("bounded-static-elf-retained-disjoint-work")
    {
        return Err(translation_edit_diagnostic(source, span, "static-image retained disjoint work requires the exact owner, one-frame request, and policy"));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_prepared_process_transition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    owner_name: &str,
    backing_name: Option<&str>,
    retain: bool,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image prepared-process transition requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image prepared-process transition requires named parameters",
        ));
    };
    let verb = if retain { "retain" } else { "restore" };
    if items.len() != 8
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], verb)
        || !identifier_is(source, &items[5], "prepared")
        || !identifier_is(source, &items[6], "process")
        || fields.len() != if retain { 4 } else { 2 }
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image prepared-process transition requires its exact owner, retained value, and policy",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut value = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image prepared-process parameters must be named",
            ));
        };
        let Expression::Identifier(identity) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image prepared-process parameters must be identifiers",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "backing" => backing = Some(source.slice(*identity)),
            "value" => value = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => return Err(translation_edit_diagnostic(source, label, "unknown static-image prepared-process parameter")),
        }
    }
    let expected_value = retain.then_some("bounded-static-elf-initial-process");
    let expected_policy = if retain {
        "bounded-static-elf-pre-entropy-retention"
    } else {
        "bounded-static-elf-retention-qualification-rollback"
    };
    if owner != Some(owner_name)
        || backing != backing_name
        || value != expected_value
        || policy != Some(expected_policy)
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image prepared-process transition requires the exact owner, backing, value, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_random_slot_transition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
    authorize: bool,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image random-slot transition must refine one exact stack entry",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image random-slot transition requires named parameters",
        ));
    };
    let verb = if authorize { "authorize" } else { "revoke" };
    if items.len() != 9
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], verb)
        || !identifier_is(source, &items[5], "stack")
        || !identifier_is(source, &items[6], "random")
        || !identifier_is(source, &items[7], "slot")
        || fields.len() != if authorize { 5 } else { 3 }
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image random-slot transition requires its exact owner, backing, value, extent, and policy",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut value = None;
    let mut extent_bytes = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image random-slot parameters must be named",
            ));
        };
        match source.slice(label) {
            "owner" | "backing" | "value" | "policy" => {
                let Expression::Identifier(identity) = &field.value else {
                    return Err(translation_edit_diagnostic(
                        source,
                        field.value.span(),
                        "static-image random-slot identity parameters must be identifiers",
                    ));
                };
                match source.slice(label) {
                    "owner" => owner = Some(source.slice(*identity)),
                    "backing" => backing = Some(source.slice(*identity)),
                    "value" => value = Some(source.slice(*identity)),
                    "policy" => policy = Some(source.slice(*identity)),
                    _ => unreachable!(),
                }
            }
            "extent-bytes" => extent_bytes = Some(parse_storage_natural(source, &field.value)?),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image random-slot parameter",
                ));
            }
        }
    }
    let expected_value = authorize.then_some("initial-auxv-random-object");
    let expected_extent = authorize.then_some(16);
    let expected_policy = if authorize {
        "bounded-linux-auxv-random-before-null"
    } else {
        "bounded-stack-random-slot-qualification-rollback"
    };
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || value != expected_value
        || extent_bytes != expected_extent
        || policy != Some(expected_policy)
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image random-slot transition requires the exact owner, backing, 16-byte initial random object, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_program_headers_addition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
) -> Result<String, Diagnostic> {
    parse_static_image_stack_program_headers_transition(
        source,
        statement,
        context_name,
        construction_name,
        backing_name,
        true,
    )
}

fn parse_static_image_stack_program_headers_removal(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
) -> Result<String, Diagnostic> {
    parse_static_image_stack_program_headers_transition(
        source,
        statement,
        context_name,
        construction_name,
        backing_name,
        false,
    )
}

fn parse_static_image_stack_program_headers_transition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
    add: bool,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image program-header transition must refine one exact stack entry",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image program-header transition requires named parameters",
        ));
    };
    let verb = if add { "add" } else { "remove" };
    if items.len() != 9
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], verb)
        || !identifier_is(source, &items[5], "stack")
        || !identifier_is(source, &items[6], "program")
        || !identifier_is(source, &items[7], "headers")
        || fields.len() != if add { 4 } else { 3 }
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image program-header transition requires its exact owner, backing, value, and policy",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut value = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(identity)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image program-header parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "backing" => backing = Some(source.slice(*identity)),
            "value" => value = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image program-header parameter",
                ));
            }
        }
    }
    let expected_value = add.then_some("resident-static-elf-program-headers");
    let expected_policy = if add {
        "bounded-linux-auxv-before-null"
    } else {
        "bounded-stack-program-headers-qualification-rollback"
    };
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || value != expected_value
        || policy != Some(expected_policy)
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image program-header transition requires the exact owner, backing, resident metadata value, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_entry_fact_removal(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image AT_ENTRY removal must refine one entry-fact stack entry",
            ));
        }
    };
    let [receiver, static_word, image, construction, remove, stack, entry, fact, Expression::Product { fields, .. }] =
        items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_ENTRY removal requires its exact owner, backing, and rollback policy",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, remove, "remove")
        || !identifier_is(source, stack, "stack")
        || !identifier_is(source, entry, "entry")
        || !identifier_is(source, fact, "fact")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_ENTRY removal requires its exact owner, backing, and rollback policy",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(identity)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image AT_ENTRY removal parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "backing" => backing = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image AT_ENTRY removal parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || policy != Some("bounded-stack-entry-fact-qualification-rollback")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_ENTRY removal requires the exact owner, backing, and rollback policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_pagesz_removal(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image AT_PAGESZ removal must refine one extended stack entry",
            ));
        }
    };
    let [receiver, static_word, image, construction, remove, stack, pagesz, Expression::Product { fields, .. }] =
        items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_PAGESZ removal requires its exact owner, backing, and rollback policy",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, remove, "remove")
        || !identifier_is(source, stack, "stack")
        || !identifier_is(source, pagesz, "pagesz")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_PAGESZ removal requires its exact owner, backing, and rollback policy",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(identity)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image AT_PAGESZ removal parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "backing" => backing = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image AT_PAGESZ removal parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || policy != Some("bounded-stack-pagesz-qualification-rollback")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_PAGESZ removal requires the exact owner, backing, and rollback policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_entry_clearing(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image stack-entry clearing cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image stack-entry clearing must refine one initialized stack entry",
            ));
        }
    };
    let [receiver, static_word, image, construction, clear, stack, entry, Expression::Product { fields, .. }] =
        items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-entry clearing requires its exact owner, backing, and rollback policy",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, clear, "clear")
        || !identifier_is(source, stack, "stack")
        || !identifier_is(source, entry, "entry")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-entry clearing requires its exact owner, backing, and rollback policy",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image stack-entry clearing parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "backing" => backing = Some(source.slice(*value)),
            "policy" => policy = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image stack-entry clearing parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || policy != Some("bounded-stack-entry-qualification-rollback")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-entry clearing requires the exact owner, backing, and rollback policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_permission_seal(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image permission sealing cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image permission sealing must refine the live populated construction",
            ));
        }
    };
    let [receiver, static_word, image, construction, seal, permissions, Expression::Product { fields, .. }] =
        items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image permission sealing requires its exact owner and plan",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, seal, "seal")
        || !identifier_is(source, permissions, "permissions")
        || fields.len() != 2
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image permission sealing requires its exact owner and plan",
        ));
    }
    let mut owner = None;
    let mut plan = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image permission-seal parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "plan" => plan = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image permission-seal parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name) || plan != Some("bounded-static-elf-wx-relro") {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image permission sealing requires the exact populated construction and W^X/RELRO plan",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_payload_copy(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image payload copy cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image payload copy must refine the live zeroed construction",
            ));
        }
    };
    let [receiver, static_word, image, construction, copy, payload, Expression::Product { fields, .. }] =
        items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image payload copy requires its exact owner, source, and plan",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, copy, "copy")
        || !identifier_is(source, payload, "payload")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image payload copy requires its exact owner, source, and plan",
        ));
    }
    let mut owner = None;
    let mut resource = None;
    let mut plan = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image payload-copy parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "source" => resource = Some(source.slice(*value)),
            "plan" => plan = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image payload-copy parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name)
        || resource != Some("retained-static-user-elf")
        || plan != Some("bounded-static-elf-four-loads")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image payload copy requires the exact zeroed construction, retained static ELF, and four-load plan",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_payload_zero<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    construction_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, declarations) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => (*name, items, *span, declarations.as_slice()),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span, &[][..]),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image payload zeroing must refine the live affine construction",
            ));
        }
    };
    let [receiver, static_word, image, construction, zero, payload, Expression::Product { fields, .. }] =
        items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image payload zeroing requires its exact owner and complete extent",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, zero, "zero")
        || !identifier_is(source, payload, "payload")
        || fields.len() != 2
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image payload zeroing requires its exact owner and complete extent",
        ));
    }
    let mut owner = None;
    let mut extent = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image payload-zeroing parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "extent" => extent = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image payload-zeroing parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name) || extent != Some("complete") {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image payload zeroing requires the exact live construction and complete extent",
        ));
    }
    Ok((source.slice(name).to_owned(), declarations))
}

fn parse_static_image_construction_prepare(
    source: &SourceText,
    expression: &Expression,
    context_name: &str,
    payload_name: &str,
    metadata_name: &str,
) -> Result<(), Diagnostic> {
    let Expression::Application { items, span } = expression else {
        return Err(translation_edit_diagnostic(
            source,
            expression.span(),
            "static-image construction preparation requires named opaque resources",
        ));
    };
    let [receiver, static_word, image, construction, prepare, Expression::Product { fields, .. }] =
        items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            *span,
            "static-image construction preparation requires payload, metadata, and layout",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, prepare, "prepare")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            *span,
            "static-image construction preparation requires the live context and sealed operation",
        ));
    }
    let mut payload = None;
    let mut metadata = None;
    let mut layout = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image construction parameters must be named live identities",
            ));
        };
        match source.slice(label) {
            "payload" => payload = Some(source.slice(*value)),
            "metadata" => metadata = Some(source.slice(*value)),
            "layout" => layout = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image construction parameter",
                ));
            }
        }
    }
    if payload != Some(payload_name)
        || metadata != Some(metadata_name)
        || layout != Some("bounded-static-elf-initial-rw-nx")
    {
        return Err(translation_edit_diagnostic(
            source,
            *span,
            "static-image construction requires the exact payload, metadata, and initial RW/NX layout",
        ));
    }
    Ok(())
}

fn parse_static_image_resource_return<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    construction_name: &str,
    action_name: &str,
    resource_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, declarations) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => (*name, items, *span, declarations.as_slice()),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span, &[][..]),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image resource return must bind the original affine extent",
            ));
        }
    };
    let [receiver, static_word, image, construction, action, Expression::Product { fields, .. }] =
        items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image resource return requires its exact construction and role",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, action, action_name)
        || fields.len() != 2
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image resource return requires its exact construction and role",
        ));
    }
    let mut owner = None;
    let mut resource = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image resource return parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "resource" => resource = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image resource return parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name) || resource != Some(resource_name) {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image resource return requires its exact construction and role",
        ));
    }
    Ok((source.slice(name).to_owned(), declarations))
}

fn analyze_translation_edit_unmap_failure(
    source: &SourceText,
    action: &Expression,
    active_context: &str,
    mapping_name: &str,
) -> Result<String, Diagnostic> {
    let statements = translation_block(
        source,
        action,
        action.span(),
        "failed mapping comparison requires an unmap edit block",
    )?;
    let (unmapped_name, returned_frames, commit_success, _operations) =
        analyze_translation_edit_unmap_prefix(source, statements, active_context, mapping_name)?;
    let success_statements = translation_block(
        source,
        commit_success,
        commit_success.span(),
        "failed comparison unmap commit requires frame release and fatal disposition",
    )?;
    let [release, disposition] = success_statements else {
        return Err(translation_edit_diagnostic(
            source,
            commit_success.span(),
            "failed comparison requires unmap commit, frame release, and fatal disposition",
        ));
    };
    parse_physical_frame_release(source, release, &unmapped_name, &returned_frames)?;
    let CompilerSystemsDisposition::Fatal { message } = analyze_disposition(
        source,
        disposition,
        CompilerSystemsContextKind::Bootstrap,
        &unmapped_name,
    )?
    else {
        return Err(translation_edit_diagnostic(
            source,
            statement_span(disposition),
            "failed mapping comparison must enter `fatal` after unmap commit",
        ));
    };
    Ok(message)
}

fn analyze_translation_edit_unmap_prefix<'a>(
    source: &SourceText,
    statements: &'a [Statement],
    active_context: &str,
    mapping_name: &str,
) -> Result<(String, String, &'a Expression, Vec<CompilerSystemsOperation>), Diagnostic> {
    let (subject, edit_name, begin_success, failure_name, failure, span) =
        result_sequence(source, statements, "translation unmap edit begin")?;
    parse_translation_edit_begin(source, subject, active_context)?;
    let begin_failure = translation_failure(
        source,
        failure,
        &failure_name,
        "translation unmap-edit begin failure must enter `fatal`",
    )?;
    let begin_statements = translation_block(
        source,
        begin_success,
        span,
        "translation unmap-edit success requires unmap and commit",
    )?;
    let (returned_frames, commit_statements) = parse_translation_edit_unmap_scope(
        source,
        begin_statements,
        &edit_name,
        mapping_name,
    )?;
    let (subject, unmapped_name, commit_success, failure_name, failure, _) =
        result_sequence(source, commit_statements, "translation unmap edit commit")?;
    parse_translation_edit_commit(source, subject, &edit_name)?;
    let commit_failure = translation_failure(
        source,
        failure,
        &failure_name,
        "translation unmap-edit commit failure must enter `fatal`",
    )?;
    Ok((
        unmapped_name,
        returned_frames,
        commit_success,
        vec![
            CompilerSystemsOperation::BeginTranslationEdit {
                kind: CompilerTranslationEditKind::Unmap,
                failure_message: begin_failure,
            },
            CompilerSystemsOperation::UnmapTranslationMapping,
            CompilerSystemsOperation::CommitTranslationEdit {
                kind: CompilerTranslationEditKind::Unmap,
                failure_message: commit_failure,
            },
        ],
    ))
}

fn parse_translation_edit_begin(
    source: &SourceText,
    expression: &Expression,
    active_context: &str,
) -> Result<(), Diagnostic> {
    let Expression::Application { items, span } = expression else {
        return Err(translation_edit_diagnostic(
            source,
            expression.span(),
            "translation edit begin must be `active translation edit begin`",
        ));
    };
    let [active, translation, edit, begin] = items.as_slice() else {
        return Err(translation_edit_diagnostic(
            source,
            *span,
            "translation edit begin must be `active translation edit begin`",
        ));
    };
    if !identifier_is(source, active, active_context)
        || !identifier_is(source, translation, "translation")
        || !identifier_is(source, edit, "edit")
        || !identifier_is(source, begin, "begin")
    {
        return Err(translation_edit_diagnostic(
            source,
            *span,
            "translation edit begin requires the current affine active context",
        ));
    }
    Ok(())
}

fn parse_translation_edit_mapping(
    source: &SourceText,
    expression: &Expression,
    edit_name: &str,
    frames_name: &str,
) -> Result<CompilerTranslationMappingRequest, Diagnostic> {
    let Expression::Application { items, span } = expression else {
        return Err(translation_edit_diagnostic(
            source,
            expression.span(),
            "translation edit map must consume its frame extent",
        ));
    };
    let [edit, kernel, map, frames, Expression::Product { fields, .. }] = items.as_slice() else {
        return Err(translation_edit_diagnostic(
            source,
            *span,
            "translation edit map must be `edit kernel map frames (...)`",
        ));
    };
    if !identifier_is(source, edit, edit_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, map, "map")
        || !identifier_is(source, frames, frames_name)
    {
        return Err(translation_edit_diagnostic(
            source,
            *span,
            "translation edit map requires the live edit and its affine frame extent",
        ));
    }
    let mut rights = None;
    let mut execution = None;
    let mut memory_kind = None;
    let mut placement = None;
    for field in fields {
        let label = field.label.ok_or_else(|| {
            translation_edit_diagnostic(source, field.value.span(), "translation edit map parameters must be named")
        })?;
        let Expression::Identifier(value) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "translation edit map policy values must be static systems identities",
            ));
        };
        let (destination, expected) = match source.slice(label) {
            "rights" => (&mut rights, "read-write"),
            "execution" => (&mut execution, "denied"),
            "memory-kind" => (&mut memory_kind, "normal"),
            "placement" => (&mut placement, "provider-selected"),
            other => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    format!("unknown translation edit map parameter `{other}`"),
                ));
            }
        };
        if source.slice(*value) != expected {
            return Err(translation_edit_diagnostic(
                source,
                *value,
                format!("the initial translation edit map `{}` policy must be `{expected}`", source.slice(label)),
            ));
        }
        if destination.replace(()).is_some() {
            return Err(translation_edit_diagnostic(
                source,
                label,
                format!("translation edit map parameter `{}` is duplicated", source.slice(label)),
            ));
        }
    }
    if rights.is_none()
        || execution.is_none()
        || memory_kind.is_none()
        || placement.is_none()
        || fields.len() != 4
    {
        return Err(translation_edit_diagnostic(
            source,
            *span,
            "translation edit map requires exactly `rights`, `execution`, `memory-kind`, and `placement`",
        ));
    }
    Ok(CompilerTranslationMappingRequest::initial_read_write())
}

fn parse_translation_edit_commit(
    source: &SourceText,
    expression: &Expression,
    edit_name: &str,
) -> Result<(), Diagnostic> {
    let Expression::Application { items, span } = expression else {
        return Err(translation_edit_diagnostic(
            source,
            expression.span(),
            "translation edit commit must be `edit translation commit`",
        ));
    };
    let [edit, translation, commit] = items.as_slice() else {
        return Err(translation_edit_diagnostic(
            source,
            *span,
            "translation edit commit must be `edit translation commit`",
        ));
    };
    if !identifier_is(source, edit, edit_name)
        || !identifier_is(source, translation, "translation")
        || !identifier_is(source, commit, "commit")
    {
        return Err(translation_edit_diagnostic(
            source,
            *span,
            "translation edit commit requires the current affine edit",
        ));
    }
    Ok(())
}

fn parse_translation_edit_unmap_scope<'a>(
    source: &SourceText,
    statements: &'a [Statement],
    edit_name: &str,
    mapping_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, tail) = match statements {
        [Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        }] => (*name, items, *span, declarations.as_slice()),
        [Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        }, tail @ ..] => (*name, items, *span, tail),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statements.first().map_or(Span::new(0, 0), statement_span),
                "translation edit unmap must bind its provisional frame extent",
            ));
        }
    };
    let [edit, kernel, unmap, mapping] = items.as_slice() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "translation edit unmap must be `frames is edit kernel unmap mapping`",
        ));
    };
    if !identifier_is(source, edit, edit_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, unmap, "unmap")
        || !identifier_is(source, mapping, mapping_name)
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "translation edit unmap requires the current edit and its live mapping",
        ));
    }
    Ok((source.slice(name).to_owned(), tail))
}

fn translation_edit_diagnostic(
    source: &SourceText,
    span: Span,
    message: impl Into<String>,
) -> Diagnostic {
    source_diagnostic(source, "E-SYSTEMS-TRANSLATION-EDIT", span, message)
}
