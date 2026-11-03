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
    let (expected, true_action, false_action) =
        parse_mapping_comparison(source, comparison, true_rule, false_rule, &loaded_name)?;
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
    let mismatch_message =
        analyze_translation_edit_unmap_failure(source, false_action, &edited_name, &mapping_name)?;
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
    let mut checked = analyze_bounded_multi_frame_sequence(source, rest, &unmapped_name, storage)?;
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
    let mut checked = analyze_static_elf_frame_sequence(source, rest, allocator_context, storage)?;
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

include!("systems_translation_static_image.rs");
include!("systems_translation_image_requests.rs");
include!("systems_translation_image_entropy.rs");
include!("systems_translation_image_lifecycle.rs");
