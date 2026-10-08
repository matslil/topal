fn analyze_frame_allocator_sequence(
    source: &SourceText,
    statements: &[Statement],
    described_context: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CheckedBootstrapRegionDecision, Diagnostic> {
    let (subject, ok_binding, ok_action, failure_binding, failure_action, span) =
        result_sequence(source, statements, "frame-allocator creation")?;
    parse_frame_allocator_creation(source, subject, described_context)?;
    let failure_disposition = action_disposition(source, failure_action, &failure_binding)?;
    let CompilerSystemsDisposition::Fatal {
        message: failure_message,
    } = failure_disposition
    else {
        return Err(frame_diagnostic(
            source,
            failure_action.span(),
            "frame-allocator failure context admits only `fatal`",
        ));
    };
    let Expression::Block {
        statements: success_statements,
        ..
    } = ok_action
    else {
        return Err(frame_diagnostic(
            source,
            span,
            "frame-allocator success requires a block using its allocator context",
        ));
    };
    let mut checked = analyze_physical_frame_sequence(
        source,
        success_statements,
        &ok_binding,
        storage,
    )?;
    checked.operations.insert(
        0,
        CompilerSystemsOperation::CreateFrameAllocator { failure_message },
    );
    Ok(checked)
}

fn analyze_physical_frame_sequence(
    source: &SourceText,
    statements: &[Statement],
    allocator_context: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CheckedBootstrapRegionDecision, Diagnostic> {
    let (subject, extent_name, success, _problem_name, failure, span) =
        result_sequence(source, statements, "physical-frame allocation")?;
    let request = parse_physical_frame_allocation(source, subject, allocator_context)?;
    let failure_disposition = action_disposition(source, failure, allocator_context)?;
    let CompilerSystemsDisposition::Fatal {
        message: failure_message,
    } = failure_disposition
    else {
        return Err(frame_diagnostic(
            source,
            failure.span(),
            "physical-frame allocation failure must consume the allocator context through `fatal`",
        ));
    };
    let Expression::Block {
        statements: success_statements,
        ..
    } = success
    else {
        return Err(frame_diagnostic(
            source,
            span,
            "physical-frame allocation success requires a block releasing its affine extent",
        ));
    };
    if success_statements.len() < 5 {
        return Err(frame_diagnostic(
            source,
            success.span(),
            "the initial frame success action requires a marker, extent release, and bootstrap allocation decision",
        ));
    }
    let [marker, release, rest @ ..] = success_statements.as_slice() else {
        unreachable!("length was checked")
    };
    let marker = analyze_operation(
        source,
        marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    let CompilerSystemsOperation::ConsoleWrite { .. } = marker else {
        return Err(frame_diagnostic(
            source,
            statement_span(success_statements.first().unwrap()),
            "physical-frame allocation must publish its success marker before release",
        ));
    };
    parse_physical_frame_release(source, release, allocator_context, &extent_name)?;
    let (ordinary, bootstrap) = rest.split_at(rest.len() - 3);
    let mut operations = vec![
        CompilerSystemsOperation::AllocatePhysicalFrames {
            request,
            failure_message,
        },
        marker,
        CompilerSystemsOperation::ReleasePhysicalFrames,
    ];
    for operation in ordinary {
        operations.push(analyze_operation(
            source,
            operation,
            CompilerSystemsContextKind::Bootstrap,
            allocator_context,
        )?);
    }
    let checked = analyze_bootstrap_region_sequence(
        source,
        bootstrap,
        allocator_context,
        storage,
    )?;
    operations.extend(checked.operations);
    Ok(CheckedBootstrapRegionDecision {
        operations,
        disposition: checked.disposition,
    })
}

fn result_sequence<'a>(
    source: &SourceText,
    statements: &'a [Statement],
    operation: &str,
) -> Result<(&'a Expression, String, &'a Expression, String, &'a Expression, Span), Diagnostic> {
    if let [Statement::Expression(Expression::DecisionTable {
        subject,
        rules,
        span,
    })] = statements
    {
        let (ok_name, ok_action, error_name, error_action) =
            frame_result_actions(source, rules, *span, operation)?;
        return Ok((
            subject,
            ok_name,
            ok_action,
            error_name,
            error_action,
            *span,
        ));
    }
    let [Statement::Expression(subject), ok_rule, error_rule] = statements else {
        return Err(frame_diagnostic(
            source,
            statements.first().map_or(Span::new(0, 0), statement_span),
            format!("{operation} requires a subject and exhaustive `Ok`/`Error` actions"),
        ));
    };
    let (ok_name, ok_action) = frame_flattened_result_action(source, ok_rule, false, operation)?;
    let (error_name, error_action) =
        frame_flattened_result_action(source, error_rule, true, operation)?;
    Ok((
        subject,
        ok_name,
        ok_action,
        error_name,
        error_action,
        statement_span(ok_rule),
    ))
}

fn frame_result_actions<'a>(
    source: &SourceText,
    rules: &'a [DecisionRule],
    span: Span,
    operation: &str,
) -> Result<(String, &'a Expression, String, &'a Expression), Diagnostic> {
    let mut ok = None;
    let mut error = None;
    for rule in rules {
        match rule.matcher {
            DecisionMatcher::Result {
                error: false,
                binding,
                ..
            } if ok.is_none() => ok = Some((source.slice(binding).to_owned(), &rule.action)),
            DecisionMatcher::Result {
                error: true,
                binding,
                ..
            } if error.is_none() => {
                error = Some((source.slice(binding).to_owned(), &rule.action));
            }
            _ => {
                return Err(frame_diagnostic(
                    source,
                    rule.span,
                    format!("{operation} requires exactly one `Ok` and one `Error` action"),
                ));
            }
        }
    }
    let (Some((ok_name, ok_action)), Some((error_name, error_action))) = (ok, error) else {
        return Err(frame_diagnostic(
            source,
            span,
            format!("{operation} requires exhaustive `Ok` and `Error` actions"),
        ));
    };
    Ok((ok_name, ok_action, error_name, error_action))
}

fn frame_flattened_result_action<'a>(
    source: &SourceText,
    statement: &'a Statement,
    error: bool,
    operation: &str,
) -> Result<(String, &'a Expression), Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(frame_diagnostic(
            source,
            statement_span(statement),
            format!("{operation} requires exhaustive `Ok` and `Error` actions"),
        ));
    };
    let [variant, binding, then, action] = items.as_slice() else {
        return Err(frame_diagnostic(
            source,
            *span,
            format!("nested {operation} actions must use explicit blocks"),
        ));
    };
    let expected = if error { "Error" } else { "Ok" };
    let Expression::Identifier(binding_span) = binding else {
        return Err(frame_diagnostic(
            source,
            binding.span(),
            format!("{operation} result action requires one payload binding"),
        ));
    };
    if !identifier_is(source, variant, expected) || !identifier_is(source, then, "then") {
        return Err(frame_diagnostic(
            source,
            *span,
            format!("{operation} actions must appear once in `Ok`, `Error` order"),
        ));
    }
    Ok((source.slice(*binding_span).to_owned(), action))
}

fn parse_frame_allocator_creation(
    source: &SourceText,
    expression: &Expression,
    described_context: &str,
) -> Result<(), Diagnostic> {
    let Expression::Application { items, span } = expression else {
        return Err(frame_diagnostic(
            source,
            expression.span(),
            "frame-allocator decision subject must be `context memory create frame allocator`",
        ));
    };
    let [context, memory, create, frame, allocator] = items.as_slice() else {
        return Err(frame_diagnostic(
            source,
            *span,
            "frame-allocator decision subject must be `context memory create frame allocator`",
        ));
    };
    if !identifier_is(source, context, described_context)
        || !identifier_is(source, memory, "memory")
        || !identifier_is(source, create, "create")
        || !identifier_is(source, frame, "frame")
        || !identifier_is(source, allocator, "allocator")
    {
        return Err(frame_diagnostic(
            source,
            *span,
            "frame-allocator creation requires the live memory-described context",
        ));
    }
    Ok(())
}

fn parse_physical_frame_allocation(
    source: &SourceText,
    expression: &Expression,
    allocator_context: &str,
) -> Result<CompilerPhysicalFrameRequest, Diagnostic> {
    let Expression::Application { items, span } = expression else {
        return Err(frame_diagnostic(
            source,
            expression.span(),
            "frame allocation subject must be `context frames allocate (...)`",
        ));
    };
    let [context, frames, allocate, Expression::Product { fields, .. }] = items.as_slice() else {
        return Err(frame_diagnostic(
            source,
            *span,
            "frame allocation subject must be `context frames allocate (...)`",
        ));
    };
    if !identifier_is(source, context, allocator_context)
        || !identifier_is(source, frames, "frames")
        || !identifier_is(source, allocate, "allocate")
    {
        return Err(frame_diagnostic(
            source,
            *span,
            "frame allocation requires the live allocator context",
        ));
    }
    let mut frame_count = None;
    let mut alignment_frames = None;
    for field in fields {
        let label = field.label.ok_or_else(|| {
            frame_diagnostic(
                source,
                field.value.span(),
                "physical-frame allocation parameters must be named",
            )
        })?;
        match source.slice(label) {
            "frame-count" => insert_frame_once(
                source,
                label,
                &mut frame_count,
                parse_storage_natural(source, &field.value)?,
            )?,
            "alignment-frames" => insert_frame_once(
                source,
                label,
                &mut alignment_frames,
                parse_storage_natural(source, &field.value)?,
            )?,
            name => {
                return Err(frame_diagnostic(
                    source,
                    label,
                    format!("unknown physical-frame allocation parameter `{name}`"),
                ));
            }
        }
    }
    let (Some(frame_count), Some(alignment_frames)) = (frame_count, alignment_frames) else {
        return Err(frame_diagnostic(
            source,
            *span,
            "physical-frame allocation requires `frame-count` and `alignment-frames`",
        ));
    };
    if frame_count != 1 || alignment_frames != 1 {
        return Err(frame_diagnostic(
            source,
            *span,
            "the initial executable frame-allocation slice admits exactly one frame aligned to one frame",
        ));
    }
    Ok(CompilerPhysicalFrameRequest {
        frame_count,
        alignment_frames,
    })
}

fn insert_frame_once<T>(
    source: &SourceText,
    label: Span,
    destination: &mut Option<T>,
    value: T,
) -> Result<(), Diagnostic> {
    if destination.replace(value).is_some() {
        return Err(frame_diagnostic(
            source,
            label,
            format!(
                "physical-frame allocation parameter `{}` is duplicated",
                source.slice(label)
            ),
        ));
    }
    Ok(())
}

fn parse_physical_frame_release(
    source: &SourceText,
    statement: &Statement,
    allocator_context: &str,
    extent_name: &str,
) -> Result<(), Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(frame_diagnostic(
            source,
            statement_span(statement),
            "physical-frame extent must be consumed by `context frames release extent`",
        ));
    };
    let [context, frames, release, extent] = items.as_slice() else {
        return Err(frame_diagnostic(
            source,
            *span,
            "physical-frame extent must be consumed by `context frames release extent`",
        ));
    };
    if !identifier_is(source, context, allocator_context)
        || !identifier_is(source, frames, "frames")
        || !identifier_is(source, release, "release")
        || !identifier_is(source, extent, extent_name)
    {
        return Err(frame_diagnostic(
            source,
            *span,
            "physical-frame release requires its live allocator and affine extent",
        ));
    }
    Ok(())
}

fn frame_diagnostic(source: &SourceText, span: Span, message: impl Into<String>) -> Diagnostic {
    source_diagnostic(source, "E-SYSTEMS-FRAMES", span, message)
}
