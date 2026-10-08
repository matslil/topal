fn analyze_translation_sequence(
    source: &SourceText,
    statements: &[Statement],
    allocator_context: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CheckedBootstrapRegionDecision, Diagnostic> {
    let (subject, update_name, success, failure_name, failure, span) =
        result_sequence(source, statements, "translation construction")?;
    let request = parse_translation_begin(source, subject, allocator_context)?;
    let begin_failure = translation_failure(
        source,
        failure,
        &failure_name,
        "translation construction failure must enter `fatal`",
    )?;
    let success_statements = translation_block(
        source,
        success,
        span,
        "translation construction success requires a commit decision",
    )?;
    let (subject, space_name, commit_success, commit_failure_name, commit_failure, commit_span) =
        result_sequence(source, success_statements, "translation commit")?;
    parse_translation_unary(
        source,
        subject,
        allocator_context,
        "commit",
        &update_name,
    )?;
    let commit_failure = translation_failure(
        source,
        commit_failure,
        &commit_failure_name,
        "translation commit failure must enter `fatal`",
    )?;
    let commit_statements = translation_block(
        source,
        commit_success,
        commit_span,
        "translation commit success requires an activation decision",
    )?;
    let (
        subject,
        translated_name,
        activate_success,
        activate_failure_name,
        activate_failure,
        activate_span,
    ) = result_sequence(source, commit_statements, "translation activation")?;
    parse_translation_unary(
        source,
        subject,
        allocator_context,
        "activate",
        &space_name,
    )?;
    let activate_failure = translation_failure(
        source,
        activate_failure,
        &activate_failure_name,
        "translation activation failure must enter `fatal`",
    )?;
    let activate_statements = translation_block(
        source,
        activate_success,
        activate_span,
        "translation activation success requires a refined-context block",
    )?;
    let [marker, rest @ ..] = activate_statements else {
        return Err(translation_diagnostic(
            source,
            activate_success.span(),
            "translation activation success must publish its marker before continuing",
        ));
    };
    let marker_span = statement_span(marker);
    let marker = analyze_operation(
        source,
        marker,
        CompilerSystemsContextKind::Bootstrap,
        &translated_name,
    )?;
    let CompilerSystemsOperation::ConsoleWrite { .. } = marker else {
        return Err(translation_diagnostic(
            source,
            marker_span,
            "translation activation success must publish its marker through the refined context",
        ));
    };
    let (ordinary, mut checked) = analyze_mapping_trailing_bootstrap(
        source,
        activate_success,
        rest,
        &translated_name,
        storage,
    )?;
    let mut operations = vec![
        CompilerSystemsOperation::BeginTranslationUpdate {
            request,
            failure_message: begin_failure,
        },
        CompilerSystemsOperation::CommitTranslationUpdate {
            failure_message: commit_failure,
        },
        CompilerSystemsOperation::ActivateTranslationSpace {
            failure_message: activate_failure,
        },
        marker,
    ];
    for operation in ordinary {
        operations.push(analyze_operation(
            source,
            operation,
            CompilerSystemsContextKind::Bootstrap,
            &translated_name,
        )?);
    }
    operations.append(&mut checked.operations);
    Ok(CheckedBootstrapRegionDecision {
        operations,
        disposition: checked.disposition,
    })
}

fn parse_translation_begin(
    source: &SourceText,
    expression: &Expression,
    allocator_context: &str,
) -> Result<CompilerTranslationUpdateRequest, Diagnostic> {
    let Expression::Application { items, span } = expression else {
        return Err(translation_diagnostic(
            source,
            expression.span(),
            "translation construction must be `context translation begin (...)`",
        ));
    };
    let [context, translation, begin, Expression::Product { fields, .. }] = items.as_slice()
    else {
        return Err(translation_diagnostic(
            source,
            *span,
            "translation construction must be `context translation begin (...)`",
        ));
    };
    if !identifier_is(source, context, allocator_context)
        || !identifier_is(source, translation, "translation")
        || !identifier_is(source, begin, "begin")
        || fields.len() != 2
    {
        return Err(translation_diagnostic(
            source,
            *span,
            "translation begin requires the live allocator and exactly `template` and `page-policy`",
        ));
    }
    let mut template = false;
    let mut page_policy = false;
    for field in fields {
        let label = field.label.ok_or_else(|| {
            translation_diagnostic(
                source,
                field.value.span(),
                "translation begin parameters must be named",
            )
        })?;
        let expected = match source.slice(label) {
            "template" if !template => {
                template = true;
                "bootstrap-equivalent"
            }
            "page-policy" if !page_policy => {
                page_policy = true;
                "provider-selected"
            }
            "template" | "page-policy" => {
                return Err(translation_diagnostic(
                    source,
                    label,
                    format!("translation begin parameter `{}` is duplicated", source.slice(label)),
                ));
            }
            other => {
                return Err(translation_diagnostic(
                    source,
                    label,
                    format!("unknown translation begin parameter `{other}`"),
                ));
            }
        };
        if !identifier_is(source, &field.value, expected) {
            return Err(translation_diagnostic(
                source,
                field.value.span(),
                format!("translation begin `{}` must be `{expected}`", source.slice(label)),
            ));
        }
    }
    Ok(CompilerTranslationUpdateRequest {
        template: CompilerTranslationTemplate::BootstrapEquivalent,
        page_policy: CompilerTranslationPagePolicy::ProviderSelected,
    })
}

fn parse_translation_unary(
    source: &SourceText,
    expression: &Expression,
    allocator_context: &str,
    operation: &str,
    resource: &str,
) -> Result<(), Diagnostic> {
    let Expression::Application { items, span } = expression else {
        return Err(translation_diagnostic(
            source,
            expression.span(),
            format!("translation {operation} requires its affine resource"),
        ));
    };
    let [context, translation, actual_operation, actual_resource] = items.as_slice() else {
        return Err(translation_diagnostic(
            source,
            *span,
            format!("translation {operation} requires its affine resource"),
        ));
    };
    if !identifier_is(source, context, allocator_context)
        || !identifier_is(source, translation, "translation")
        || !identifier_is(source, actual_operation, operation)
        || !identifier_is(source, actual_resource, resource)
    {
        return Err(translation_diagnostic(
            source,
            *span,
            format!("translation {operation} requires the live context and its affine resource"),
        ));
    }
    Ok(())
}

fn translation_failure(
    source: &SourceText,
    action: &Expression,
    failure_name: &str,
    message: &str,
) -> Result<String, Diagnostic> {
    let CompilerSystemsDisposition::Fatal { message: fatal } =
        action_disposition(source, action, failure_name)?
    else {
        return Err(translation_diagnostic(source, action.span(), message));
    };
    Ok(fatal)
}

fn translation_block<'a>(
    source: &SourceText,
    action: &'a Expression,
    span: Span,
    message: &str,
) -> Result<&'a [Statement], Diagnostic> {
    let Expression::Block { statements, .. } = action else {
        return Err(translation_diagnostic(source, span, message));
    };
    Ok(statements)
}

fn translation_diagnostic(
    source: &SourceText,
    span: Span,
    message: impl Into<String>,
) -> Diagnostic {
    source_diagnostic(source, "E-SYSTEMS-TRANSLATION", span, message)
}
