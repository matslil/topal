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
    let mut checked = analyze_local_interrupt_critical_sequence(
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
