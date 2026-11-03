fn parse_user_mapping_refinement<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    mapping_name: &str,
    operation: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, declarations) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => (name, items, span, declarations.as_slice()),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (name, items, span, &[][..]),
        _ => {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "user mapping access must bind its refined affine mapping",
            ));
        }
    };
    let [receiver, user, mapping, access, byte, live_mapping] = items.as_slice() else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user mapping access requires the exact live mapping refinement",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, mapping, "mapping")
        || !identifier_is(source, access, operation)
        || !identifier_is(source, byte, "byte")
        || !identifier_is(source, live_mapping, mapping_name)
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            format!("user mapping {operation} requires the exact preceding mapping refinement"),
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn parse_user_mapping_entry_trampoline<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    mapping_name: &str,
    operation: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, declarations) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => (name, items, span, declarations.as_slice()),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (name, items, span, &[][..]),
        _ => {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "user entry-trampoline operation must bind its affine mapping refinement",
            ));
        }
    };
    let [
        receiver,
        user,
        mapping,
        action,
        entry,
        trampoline,
        live_mapping,
    ] = items.as_slice()
    else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user entry-trampoline operation requires the exact preceding mapping refinement",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, mapping, "mapping")
        || !identifier_is(source, action, operation)
        || !identifier_is(source, entry, "entry")
        || !identifier_is(source, trampoline, "trampoline")
        || !identifier_is(source, live_mapping, mapping_name)
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            format!(
                "user mapping {operation} entry trampoline requires the exact preceding mapping refinement"
            ),
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn parse_user_mapping_protection<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    mapping_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, declarations) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => (name, items, span, declarations.as_slice()),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (name, items, span, &[][..]),
        _ => {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "user mapping protection must bind its read-only affine refinement",
            ));
        }
    };
    let [receiver, user, mapping, protect, read, only, stored_mapping] = items.as_slice() else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user mapping protection requires the exact stored mapping refinement",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, mapping, "mapping")
        || !identifier_is(source, protect, "protect")
        || !identifier_is(source, read, "read")
        || !identifier_is(source, only, "only")
        || !identifier_is(source, stored_mapping, mapping_name)
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user mapping protect read only requires the exact stored mapping refinement",
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn parse_user_mapping_execute_promotion<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    mapping_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, declarations) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => (name, items, span, declarations.as_slice()),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (name, items, span, &[][..]),
        _ => {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "user mapping execute promotion must bind its affine refinement",
            ));
        }
    };
    let [receiver, user, mapping, permit, execute, verified_mapping] = items.as_slice() else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user mapping execute promotion requires the exact verified mapping refinement",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, mapping, "mapping")
        || !identifier_is(source, permit, "permit")
        || !identifier_is(source, execute, "execute")
        || !identifier_is(source, verified_mapping, mapping_name)
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user mapping permit execute requires the exact verified mapping refinement",
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn parse_user_mapping_writable_data_promotion<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    mapping_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, declarations) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => (name, items, span, declarations.as_slice()),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (name, items, span, &[][..]),
        _ => {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "user mapping writable-data promotion must bind its affine refinement",
            ));
        }
    };
    let [
        receiver,
        user,
        mapping,
        permit,
        writable,
        data,
        verified_mapping,
    ] = items.as_slice()
    else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user mapping writable-data promotion requires the exact verified mapping refinement",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, mapping, "mapping")
        || !identifier_is(source, permit, "permit")
        || !identifier_is(source, writable, "writable")
        || !identifier_is(source, data, "data")
        || !identifier_is(source, verified_mapping, mapping_name)
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user mapping permit writable data requires the second verified refinement",
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn parse_user_address_space_reserve_anonymous<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    address_space_name: &str,
    backing_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, declarations) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => (name, items, span, declarations.as_slice()),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (name, items, span, &[][..]),
        _ => {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "anonymous user-page reservation must bind its affine reservation",
            ));
        }
    };
    let [
        receiver,
        user,
        address,
        space,
        reserve,
        anonymous,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "anonymous user-page reservation requires named address-space, backing, and layout parameters",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, address, "address")
        || !identifier_is(source, space, "space")
        || !identifier_is(source, reserve, "reserve")
        || !identifier_is(source, anonymous, "anonymous")
        || fields.len() != 3
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "the bounded anonymous reservation requires its exact prepared space, frame, and layout",
        ));
    }
    let mut address_space = false;
    let mut backing = false;
    let mut layout = false;
    for field in fields {
        let Some(label) = field.label else {
            return Err(user_context_diagnostic(
                source,
                field.value.span(),
                "anonymous reservation parameters must be named",
            ));
        };
        match source.slice(label) {
            "address-space" => {
                address_space = identifier_is(source, &field.value, address_space_name)
            }
            "backing" => backing = identifier_is(source, &field.value, backing_name),
            "layout" => {
                layout = identifier_is(source, &field.value, "post-random-anonymous-growth-extent")
            }
            _ => {}
        }
    }
    if !address_space || !backing || !layout {
        return Err(user_context_diagnostic(
            source,
            *span,
            "anonymous reservation must consume the exact extent into `post-random-anonymous-growth-extent`",
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn parse_user_address_space_release_anonymous<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    address_space_name: &str,
    reservation_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, declarations) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => (name, items, span, declarations.as_slice()),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (name, items, span, &[][..]),
        _ => {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "anonymous user-page release must bind its returned affine frame",
            ));
        }
    };
    let [
        receiver,
        user,
        address,
        space,
        release,
        anonymous,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "anonymous user-page release requires named address-space and reservation parameters",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, address, "address")
        || !identifier_is(source, space, "space")
        || !identifier_is(source, release, "release")
        || !identifier_is(source, anonymous, "anonymous")
        || fields.len() != 2
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "the bounded anonymous release requires its exact prepared space and reservation",
        ));
    }
    let mut address_space = false;
    let mut reservation = false;
    for field in fields {
        let Some(label) = field.label else {
            return Err(user_context_diagnostic(
                source,
                field.value.span(),
                "anonymous release parameters must be named",
            ));
        };
        match source.slice(label) {
            "address-space" => {
                address_space = identifier_is(source, &field.value, address_space_name)
            }
            "reservation" => reservation = identifier_is(source, &field.value, reservation_name),
            _ => {}
        }
    }
    if !address_space || !reservation {
        return Err(user_context_diagnostic(
            source,
            *span,
            "anonymous release must consume the exact live reservation",
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn parse_user_address_space_unmap<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    address_space_name: &str,
    mapping_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, declarations) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => (name, items, span, declarations.as_slice()),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (name, items, span, &[][..]),
        _ => {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "user mapping revocation must bind the returned affine extent",
            ));
        }
    };
    let [
        receiver,
        user,
        address,
        space,
        unmap,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user mapping revocation requires named address-space and mapping parameters",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, address, "address")
        || !identifier_is(source, space, "space")
        || !identifier_is(source, unmap, "unmap")
        || fields.len() != 2
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "the initial user unmap requires the exact prepared space and verified mapping",
        ));
    }
    let mut address_space = None;
    let mut mapping_identity = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(user_context_diagnostic(
                source,
                field.value.span(),
                "user mapping revocation parameters must be named",
            ));
        };
        let Expression::Identifier(value) = field.value else {
            return Err(user_context_diagnostic(
                source,
                field.value.span(),
                "user mapping revocation values must use live affine identities",
            ));
        };
        match source.slice(label) {
            "address-space" => address_space = Some(source.slice(value)),
            "mapping" => mapping_identity = Some(source.slice(value)),
            _ => {
                return Err(user_context_diagnostic(
                    source,
                    label,
                    "unknown user mapping revocation parameter",
                ));
            }
        }
    }
    if address_space != Some(address_space_name) || mapping_identity != Some(mapping_name) {
        return Err(user_context_diagnostic(
            source,
            *span,
            "the initial user unmap requires the exact prepared space and verified mapping",
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn select_user_mapping_continuation<'a>(
    source: &SourceText,
    span: Span,
    following: &'a [Statement],
    nested: &'a [Statement],
    operation: &str,
) -> Result<&'a [Statement], Diagnostic> {
    match (following.is_empty(), nested.is_empty()) {
        (true, false) => Ok(nested),
        (false, true) => Ok(following),
        _ => Err(user_context_diagnostic(
            source,
            span,
            format!("user mapping {operation} must transfer exactly one continuation"),
        )),
    }
}

fn parse_user_probe_prepare<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    address_space_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, declarations) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => (name, items, span, declarations.as_slice()),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (name, items, span, &[][..]),
        _ => {
            return Err(user_context_diagnostic(
                source,
                statement_span(statement),
                "user probe preparation must bind one classified suspended user context",
            ));
        }
    };
    let [
        receiver,
        user,
        probe,
        prepare,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user probe preparation must use the sealed process-identity request",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, probe, "probe")
        || !identifier_is(source, prepare, "prepare")
        || fields.len() != 2
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "the initial suspended user probe requires process identity one and exact processor, address-space, thread, and ABI classifiers",
        ));
    }
    let mut process_identity = None;
    let mut address_space = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(user_context_diagnostic(
                source,
                field.value.span(),
                "user probe preparation parameters must be named",
            ));
        };
        match source.slice(label) {
            "process-identity" => {
                process_identity = Some(parse_storage_natural(source, &field.value)?);
            }
            "address-space" => {
                let Expression::Identifier(name) = field.value else {
                    return Err(user_context_diagnostic(
                        source,
                        field.value.span(),
                        "user probe address space must be the live opaque prepared space",
                    ));
                };
                address_space = Some(source.slice(name));
            }
            _ => {
                return Err(user_context_diagnostic(
                    source,
                    label,
                    "unknown user probe preparation parameter",
                ));
            }
        }
    }
    if process_identity != Some(1) || address_space != Some(address_space_name) {
        return Err(user_context_diagnostic(
            source,
            *span,
            "the initial suspended user probe requires process identity one and its exact prepared address space",
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn parse_user_context_enter(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    probe_name: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: None,
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(user_context_diagnostic(
            source,
            statement_span(statement),
            "user entry must bind one classified completed user execution",
        ));
    };
    let [receiver, user, context_word, enter, probe] = items.as_slice() else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user entry must be `context user context enter probe`",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, context_word, "context")
        || !identifier_is(source, enter, "enter")
        || !identifier_is(source, probe, probe_name)
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user entry must consume the exact suspended probe and preserve all identities",
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_user_execution_complete(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    completed_name: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: None,
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(user_context_diagnostic(
            source,
            statement_span(statement),
            "user execution completion must consume the terminal result",
        ));
    };
    let [receiver, user, execution, complete, completed] = items.as_slice() else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user completion must be `context user execution complete completed`",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, execution, "execution")
        || !identifier_is(source, complete, "complete")
        || !identifier_is(source, completed, completed_name)
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user completion must consume the exact terminal execution result",
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_user_address_space_reclaim(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    revoked_name: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: None,
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(user_context_diagnostic(
            source,
            statement_span(statement),
            "user-address-space reclaim must bind the returned affine frame extent",
        ));
    };
    let [receiver, user, address, space, reclaim, revoked] = items.as_slice() else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user-address-space reclaim must consume one revoked space",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, address, "address")
        || !identifier_is(source, space, "space")
        || !identifier_is(source, reclaim, "reclaim")
        || !identifier_is(source, revoked, revoked_name)
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user-address-space reclaim must consume the exact revoked space",
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_kernel_completion_create(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "kernel-completion creation must bind its affine one-shot state",
        ));
    };
    let [context, kernel, completion, create] = items.as_slice() else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "kernel-completion creation requires its live dispatcher",
        ));
    };
    if source.slice(*classifier) != "KernelCompletion InitialProcessor"
        || !identifier_is(source, context, context_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, completion, "completion")
        || !identifier_is(source, create, "create")
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "kernel-completion creation requires the exact affine classifier and live dispatcher",
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_kernel_context_transfer_until_completion(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    worker_name: &str,
    completion_name: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "completion-bound transfer must bind its exact blocked outcome",
        ));
    };
    let [
        context,
        kernel,
        context_word,
        transfer,
        worker,
        until,
        completion,
    ] = items.as_slice()
    else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "completion-bound transfer requires a selected worker and one-shot completion",
        ));
    };
    if source.slice(*classifier) != "BlockedKernelContextTransfer InitialProcessor"
        || !identifier_is(source, context, context_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, context_word, "context")
        || !identifier_is(source, transfer, "transfer")
        || !identifier_is(source, worker, worker_name)
        || !identifier_is(source, until, "until")
        || !identifier_is(source, completion, completion_name)
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "completion-bound transfer requires the exact dispatcher, selected worker, and completion identity",
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_signal_blocked_kernel_completion(
    source: &SourceText,
    statement: &Statement,
    blocked_name: &str,
    completion_name: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "completion signal must bind the exact awakened suspended context",
        ));
    };
    let [blocked, kernel, completion, signal, completion_value] = items.as_slice() else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "completion signal must consume the blocked outcome and matching signal endpoint",
        ));
    };
    if source.slice(*classifier) != "SuspendedKernelContext InitialProcessor"
        || !identifier_is(source, blocked, blocked_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, completion, "completion")
        || !identifier_is(source, signal, "signal")
        || !identifier_is(source, completion_value, completion_name)
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "completion signal requires the matching blocked context and one-shot completion",
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_kernel_completion_consume(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    completion_name: &str,
) -> Result<(), Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "completed one-shot state must be consumed before disposition",
        ));
    };
    let [context, kernel, completion, consume, completion_value] = items.as_slice() else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "completion consumption requires the live dispatcher and completed state",
        ));
    };
    if !identifier_is(source, context, context_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, completion, "completion")
        || !identifier_is(source, consume, "consume")
        || !identifier_is(source, completion_value, completion_name)
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "completion consumption requires the exact completed one-shot identity",
        ));
    }
    Ok(())
}

fn parse_runnable_queue_create(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "runnable-queue creation must bind its exact bounded affine classifier",
        ));
    };
    let [
        context,
        kernel,
        runnable,
        queue,
        create,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "runnable-queue creation requires one named capacity",
        ));
    };
    let [field] = fields.as_slice() else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "the initial runnable queue requires exactly `capacity is 3`",
        ));
    };
    let capacity_is_three = field
        .label
        .is_some_and(|label| source.slice(label) == "capacity")
        && matches!(&field.value, Expression::Integer(value) if source.slice(*value) == "3");
    if source.slice(*classifier) != "BoundedKernelRunnableQueue InitialProcessor"
        || !identifier_is(source, context, context_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, runnable, "runnable")
        || !identifier_is(source, queue, "queue")
        || !identifier_is(source, create, "create")
        || !capacity_is_three
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "the initial runnable queue requires its live context, capacity three, and exact bounded classifier",
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_runnable_queue_enqueue(
    source: &SourceText,
    statement: &Statement,
    queue_name: &str,
    context_name: &str,
) -> Result<(), Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "runnable enqueue must refine the affine queue in place",
        ));
    };
    let [queue, kernel, runnable, enqueue, context] = items.as_slice() else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "runnable enqueue must consume one queue and one suspended context",
        ));
    };
    if !identifier_is(source, queue, queue_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, runnable, "runnable")
        || !identifier_is(source, enqueue, "enqueue")
        || !identifier_is(source, context, context_name)
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "runnable enqueue must refine the queue and consume the selected suspended context",
        ));
    }
    Ok(())
}

fn parse_runnable_queue_dequeue(
    source: &SourceText,
    statement: &Statement,
    queue_name: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "runnable dequeue must bind one suspended context",
        ));
    };
    let [queue, kernel, runnable, dequeue] = items.as_slice() else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "runnable dequeue must consume the current queue state",
        ));
    };
    if source.slice(*classifier) != "SuspendedKernelContext InitialProcessor"
        || !identifier_is(source, queue, queue_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, runnable, "runnable")
        || !identifier_is(source, dequeue, "dequeue")
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "runnable dequeue must select the FIFO head from the exact queue binding",
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_runnable_queue_consume(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    queue_name: &str,
) -> Result<(), Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "empty runnable-queue consumption must be an explicit operation",
        ));
    };
    let [
        context,
        kernel,
        runnable,
        queue,
        consume,
        empty,
        queue_value,
    ] = items.as_slice()
    else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "empty runnable-queue consumption requires the live context and queue",
        ));
    };
    if !identifier_is(source, context, context_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, runnable, "runnable")
        || !identifier_is(source, queue, "queue")
        || !identifier_is(source, consume, "consume")
        || !identifier_is(source, empty, "empty")
        || !identifier_is(source, queue_value, queue_name)
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "runnable-queue consumption requires the exact empty queue before disposition",
        ));
    }
    Ok(())
}

fn kernel_context_create_fields(
    source: &SourceText,
    fields: &[ProductField],
    stack_name: &str,
    entry_name: &str,
) -> Result<bool, Diagnostic> {
    let mut stack = None;
    let mut entry = None;
    for field in fields {
        let Some(label) = field.label else {
            return Ok(false);
        };
        match source.slice(label) {
            "stack" if stack.is_none() => stack = Some(&field.value),
            "entry" if entry.is_none() => entry = Some(&field.value),
            _ => return Ok(false),
        }
    }
    Ok(
        matches!(stack, Some(value) if identifier_is(source, value, stack_name))
            && matches!(entry, Some(value) if identifier_is(source, value, entry_name)),
    )
}

fn parse_kernel_context_transfer(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    worker_name: &str,
    result_classifier: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            format!("kernel-context transfer must bind `{result_classifier}`"),
        ));
    };
    let [context, kernel, context_word, transfer, worker] = items.as_slice() else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "kernel-context transfer must consume one suspended context",
        ));
    };
    if source.slice(*classifier) != result_classifier
        || !identifier_is(source, context, context_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, context_word, "context")
        || !identifier_is(source, transfer, "transfer")
        || !identifier_is(source, worker, worker_name)
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "kernel-context transfer requires the live caller and matching suspended context",
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_kernel_context_transfer_until_deadline(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    worker_name: &str,
    deadline_name: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "deadline-preemptible transfer must bind its exact affine outcome",
        ));
    };
    let [
        context,
        kernel,
        context_word,
        transfer,
        worker,
        until,
        deadline,
    ] = items.as_slice()
    else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "deadline-preemptible transfer must consume one selected context and deadline",
        ));
    };
    if source.slice(*classifier)
        != "PreemptedKernelContextTransfer InitialProcessor InitialMonotonicClock"
        || !identifier_is(source, context, context_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, context_word, "context")
        || !identifier_is(source, transfer, "transfer")
        || !identifier_is(source, worker, worker_name)
        || !identifier_is(source, until, "until")
        || !identifier_is(source, deadline, deadline_name)
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "deadline-preemptible transfer requires the live dispatcher, FIFO-selected context, and exact deadline",
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_take_preempted_kernel_context(
    source: &SourceText,
    statement: &Statement,
    preempted_name: &str,
) -> Result<String, Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "taking a preempted outcome must bind one suspended kernel context",
        ));
    };
    let [receiver, kernel, context_word, take, preempted] = items.as_slice() else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "taking a preempted outcome must consume the exact affine result",
        ));
    };
    if source.slice(*classifier) != "SuspendedKernelContext InitialProcessor"
        || !identifier_is(source, receiver, preempted_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, context_word, "context")
        || !identifier_is(source, take, "take")
        || !identifier_is(source, preempted, "preempted")
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "taking preemption requires the exact completed outcome before runnable enqueue",
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_kernel_context_reclaim<'a>(
    source: &SourceText,
    statement: &'a Statement,
    completed_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let Statement::Implementation {
        name,
        classifier: Expression::Application { items, span },
        declarations,
        ..
    } = statement
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "kernel-context reclaim must bind the resumed caller and continue into its final disposition",
        ));
    };
    let [completed, kernel, context, reclaim] = items.as_slice() else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "kernel-context reclaim must consume one completed transfer",
        ));
    };
    if !identifier_is(source, completed, completed_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, context, "context")
        || !identifier_is(source, reclaim, "reclaim")
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "kernel-context reclaim must consume the matching completed transfer",
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn parse_deadline_after(
    source: &SourceText,
    statement: &Statement,
    instant_name: &str,
) -> Result<(String, CompilerSystemsOperation), Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(deadline_diagnostic(
            source,
            statement_span(statement),
            "deadline construction must bind `Deadline InitialMonotonicClock`",
        ));
    };
    let [
        instant,
        deadline,
        after,
        Expression::Measured { value, unit, .. },
    ] = items.as_slice()
    else {
        return Err(deadline_diagnostic(
            source,
            *span,
            "deadline construction must be `deadline : Deadline InitialMonotonicClock is instant deadline after 1[ms]`",
        ));
    };
    if source.slice(*classifier) != "Deadline InitialMonotonicClock"
        || !identifier_is(source, instant, instant_name)
        || !identifier_is(source, deadline, "deadline")
        || !identifier_is(source, after, "after")
        || source.slice(*value) != "1"
        || source.slice(*unit) != "ms"
    {
        return Err(deadline_diagnostic(
            source,
            *span,
            "the initial deadline must derive from the second observation and exact duration 1[ms]",
        ));
    }
    Ok((
        source.slice(*name).to_owned(),
        CompilerSystemsOperation::ConstructDeadline {
            duration_nanoseconds: 1_000_000,
        },
    ))
}

fn parse_monotonic_clock_now(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
) -> Result<(String, CompilerSystemsOperation), Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(monotonic_clock_diagnostic(
            source,
            statement_span(statement),
            "clock observation must bind `Instant InitialMonotonicClock`",
        ));
    };
    let [context, monotonic, clock, now] = items.as_slice() else {
        return Err(monotonic_clock_diagnostic(
            source,
            *span,
            "clock observation must be `instant : Instant InitialMonotonicClock is context monotonic clock now`",
        ));
    };
    if source.slice(*classifier) != "Instant InitialMonotonicClock"
        || !identifier_is(source, context, context_name)
        || !identifier_is(source, monotonic, "monotonic")
        || !identifier_is(source, clock, "clock")
        || !identifier_is(source, now, "now")
    {
        return Err(monotonic_clock_diagnostic(
            source,
            *span,
            "clock observation requires the resumed context and initial monotonic clock classifier",
        ));
    }
    Ok((
        source.slice(*name).to_owned(),
        CompilerSystemsOperation::ObserveMonotonicClock,
    ))
}

fn parse_atomic_exchanged_failure(
    source: &SourceText,
    false_action: &Expression,
    context_name: &str,
    region_name: &str,
    atomic_name: &str,
) -> Result<String, Diagnostic> {
    let failure = action_block(source, false_action, "failed atomic load")?;
    let [end] = failure else {
        return Err(atomic_diagnostic(
            source,
            false_action.span(),
            "failed atomic load requires atomic end, region release, and fatal disposition",
        ));
    };
    let failure_continuation = parse_atomic_end(source, end, atomic_name, region_name)?;
    let [release, disposition] = failure_continuation else {
        return Err(atomic_diagnostic(
            source,
            false_action.span(),
            "failed atomic load requires region release and fatal disposition after atomic end",
        ));
    };
    parse_region_release(source, release, context_name, region_name)?;
    let CompilerSystemsDisposition::Fatal {
        message: load_failure_message,
    } = analyze_disposition(
        source,
        disposition,
        CompilerSystemsContextKind::Bootstrap,
        context_name,
    )?
    else {
        return Err(atomic_diagnostic(
            source,
            statement_span(disposition),
            "failed atomic load must enter the fatal disposition",
        ));
    };
    Ok(load_failure_message)
}

fn parse_local_notification_send(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
) -> Result<(String, CompilerSystemsOperation), Diagnostic> {
    let (name, items, span) = affine_application_binding(source, statement, "send")?;
    let [context, local, notification, send] = items else {
        return Err(local_interrupt_diagnostic(
            source,
            span,
            "send must bind `pending is context local notification send`",
        ));
    };
    if !identifier_is(source, context, context_name)
        || !identifier_is(source, local, "local")
        || !identifier_is(source, notification, "notification")
        || !identifier_is(source, send, "send")
    {
        return Err(local_interrupt_diagnostic(
            source,
            span,
            "local-notification send must consume the current processor context",
        ));
    }
    Ok((
        source.slice(name).to_owned(),
        CompilerSystemsOperation::SendLocalNotification,
    ))
}

fn parse_local_notification_wait(
    source: &SourceText,
    statement: &Statement,
    pending_name: &str,
) -> Result<(String, CompilerSystemsOperation), Diagnostic> {
    let (name, items, span) = affine_application_binding(source, statement, "wait")?;
    let [pending, local, notification, wait] = items else {
        return Err(local_interrupt_diagnostic(
            source,
            span,
            "wait must bind `resumed is pending local notification wait`",
        ));
    };
    if !identifier_is(source, pending, pending_name)
        || !identifier_is(source, local, "local")
        || !identifier_is(source, notification, "notification")
        || !identifier_is(source, wait, "wait")
    {
        return Err(local_interrupt_diagnostic(
            source,
            span,
            "local-notification wait must consume the matching pending session",
        ));
    }
    Ok((
        source.slice(name).to_owned(),
        CompilerSystemsOperation::WaitLocalNotification,
    ))
}

fn parse_atomic_load(
    source: &SourceText,
    statement: &Statement,
    atomic_name: &str,
) -> Result<(String, CompilerAtomicOrder), Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(atomic_diagnostic(
            source,
            statement_span(statement),
            "atomic load must bind its `Nat` result",
        ));
    };
    let [atomic, load, Expression::Product { fields, .. }] = items.as_slice() else {
        return Err(atomic_diagnostic(
            source,
            *span,
            "atomic load must be `observed : Nat is atomic load (order is acquire)`",
        ));
    };
    if source.slice(*classifier) != "Nat"
        || !identifier_is(source, atomic, atomic_name)
        || !identifier_is(source, load, "load")
        || fields.len() != 1
    {
        return Err(atomic_diagnostic(
            source,
            *span,
            "atomic load requires the live location and one named order",
        ));
    }
    Ok((
        source.slice(*name).to_owned(),
        atomic_order_field(source, fields, "order")?,
    ))
}

fn parse_atomic_equality(
    source: &SourceText,
    statement: &Statement,
    loaded_name: &str,
) -> Result<u64, Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(atomic_diagnostic(
            source,
            statement_span(statement),
            "atomic load requires an exhaustive equality decision",
        ));
    };
    let [
        loaded,
        Expression::Callable {
            kind: CallableKind::Equal,
            ..
        },
        expected,
    ] = items.as_slice()
    else {
        return Err(atomic_diagnostic(
            source,
            *span,
            "atomic load decision must compare its binding with a static natural",
        ));
    };
    if !identifier_is(source, loaded, loaded_name) {
        return Err(atomic_diagnostic(
            source,
            loaded.span(),
            "atomic comparison must use the immediately preceding load binding",
        ));
    }
    parse_storage_natural(source, expected)
}

fn parse_atomic_end<'a>(
    source: &SourceText,
    statement: &'a Statement,
    atomic_name: &str,
    region_name: &str,
) -> Result<&'a [Statement], Diagnostic> {
    let Statement::Implementation {
        name,
        classifier: Expression::Application { items, span },
        declarations,
        ..
    } = statement
    else {
        return Err(atomic_diagnostic(
            source,
            statement_span(statement),
            "atomic lifecycle must return its region through `region is atomic end`",
        ));
    };
    let [atomic, end] = items.as_slice() else {
        return Err(atomic_diagnostic(
            source,
            *span,
            "atomic lifecycle must return its region through `region is atomic end`",
        ));
    };
    if source.slice(*name) != region_name
        || !identifier_is(source, atomic, atomic_name)
        || !identifier_is(source, end, "end")
    {
        return Err(atomic_diagnostic(
            source,
            *span,
            "atomic end must consume the live location and restore the original region binding",
        ));
    }
    Ok(declarations)
}

fn atomic_natural_field(
    source: &SourceText,
    fields: &[ProductField],
    name: &str,
) -> Result<u64, Diagnostic> {
    let field = unique_atomic_field(source, fields, name)?;
    parse_storage_natural(source, &field.value)
}

fn atomic_identity_field(
    source: &SourceText,
    fields: &[ProductField],
    name: &str,
    required: &str,
) -> Result<String, Diagnostic> {
    let field = unique_atomic_field(source, fields, name)?;
    let Expression::Identifier(value) = field.value else {
        return Err(atomic_diagnostic(
            source,
            field.value.span(),
            format!("atomic parameter `{name}` must be a static identity"),
        ));
    };
    let value = source.slice(value).to_owned();
    if value != required {
        return Err(atomic_diagnostic(
            source,
            field.value.span(),
            format!("atomic parameter `{name}` requires `{required}`"),
        ));
    }
    Ok(value)
}

fn atomic_order_field(
    source: &SourceText,
    fields: &[ProductField],
    name: &str,
) -> Result<CompilerAtomicOrder, Diagnostic> {
    let field = unique_atomic_field(source, fields, name)?;
    let Expression::Identifier(value) = field.value else {
        return Err(atomic_diagnostic(
            source,
            field.value.span(),
            format!("atomic order `{name}` must be a static identity"),
        ));
    };
    match source.slice(value) {
        "atomic-only" => Ok(CompilerAtomicOrder::AtomicOnly),
        "acquire" => Ok(CompilerAtomicOrder::Acquire),
        "release" => Ok(CompilerAtomicOrder::Release),
        "acquire-release" => Ok(CompilerAtomicOrder::AcquireRelease),
        "sequential" => Ok(CompilerAtomicOrder::Sequential),
        other => Err(atomic_diagnostic(
            source,
            value,
            format!("unknown atomic order `{other}`"),
        )),
    }
}

fn unique_atomic_field<'a>(
    source: &SourceText,
    fields: &'a [ProductField],
    name: &str,
) -> Result<&'a ProductField, Diagnostic> {
    let mut found = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(atomic_diagnostic(
                source,
                field.value.span(),
                "atomic parameters must be named",
            ));
        };
        if source.slice(label) == name && found.replace(field).is_some() {
            return Err(atomic_diagnostic(
                source,
                label,
                format!("atomic parameter `{name}` is duplicated"),
            ));
        }
    }
    found.ok_or_else(|| {
        atomic_diagnostic(
            source,
            fields
                .first()
                .map_or(Span::new(0, 0), |field| field.value.span()),
            format!("atomic operation requires `{name}`"),
        )
    })
}

fn parse_boolean_rule<'a>(
    source: &SourceText,
    statement: &'a Statement,
    expected: bool,
) -> Result<&'a Expression, Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(storage_diagnostic(
            source,
            statement_span(statement),
            "byte equality requires explicit `true` and `false` actions",
        ));
    };
    let [Expression::Boolean(value), then, action] = items.as_slice() else {
        return Err(storage_diagnostic(
            source,
            *span,
            "byte equality requires explicit `true` and `false` actions",
        ));
    };
    if (source.slice(*value) == "true") != expected || !identifier_is(source, then, "then") {
        return Err(storage_diagnostic(
            source,
            *span,
            "byte equality actions must appear once in `true`, `false` order",
        ));
    }
    Ok(action)
}

fn action_block<'a>(
    source: &SourceText,
    action: &'a Expression,
    description: &str,
) -> Result<&'a [Statement], Diagnostic> {
    let Expression::Block { statements, .. } = action else {
        return Err(storage_diagnostic(
            source,
            action.span(),
            format!("{description} requires a block"),
        ));
    };
    Ok(statements)
}

fn action_disposition(
    source: &SourceText,
    action: &Expression,
    context_name: &str,
) -> Result<CompilerSystemsDisposition, Diagnostic> {
    if let Expression::Block { statements, .. } = action {
        let [statement] = statements.as_slice() else {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-DISPOSITION",
                action.span(),
                "a systems failure block must contain exactly one context-consuming disposition",
            ));
        };
        return analyze_disposition(
            source,
            statement,
            CompilerSystemsContextKind::Bootstrap,
            context_name,
        );
    }
    let statement = Statement::Expression(action.clone());
    analyze_disposition(
        source,
        &statement,
        CompilerSystemsContextKind::Bootstrap,
        context_name,
    )
}
