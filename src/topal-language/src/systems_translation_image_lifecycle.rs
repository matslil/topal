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
    let [
        receiver,
        static_word,
        image,
        construction,
        prepare,
        Expression::Product { fields, .. },
    ] = items.as_slice()
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
    let [
        receiver,
        static_word,
        image,
        construction,
        action,
        Expression::Product { fields, .. },
    ] = items.as_slice()
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
) -> Result<
    (
        String,
        String,
        &'a Expression,
        Vec<CompilerSystemsOperation>,
    ),
    Diagnostic,
> {
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
    let (returned_frames, commit_statements) =
        parse_translation_edit_unmap_scope(source, begin_statements, &edit_name, mapping_name)?;
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
    let [
        edit,
        kernel,
        map,
        frames,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
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
            translation_edit_diagnostic(
                source,
                field.value.span(),
                "translation edit map parameters must be named",
            )
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
                format!(
                    "the initial translation edit map `{}` policy must be `{expected}`",
                    source.slice(label)
                ),
            ));
        }
        if destination.replace(()).is_some() {
            return Err(translation_edit_diagnostic(
                source,
                label,
                format!(
                    "translation edit map parameter `{}` is duplicated",
                    source.slice(label)
                ),
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
        [
            Statement::Implementation {
                name,
                classifier: Expression::Application { items, span },
                declarations,
                ..
            },
        ] => (*name, items, *span, declarations.as_slice()),
        [
            Statement::Binding {
                name,
                classifier: None,
                value: Expression::Application { items, span },
            },
            tail @ ..,
        ] => (*name, items, *span, tail),
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
