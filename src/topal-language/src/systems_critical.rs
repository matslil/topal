fn analyze_local_interrupt_critical_sequence(
    source: &SourceText,
    statements: &[Statement],
    context_name: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CheckedBootstrapRegionDecision, Diagnostic> {
    let (subject, critical_name, success, failure_name, failure, span) =
        result_sequence(source, statements, "local-interrupt critical entry")?;
    parse_critical_enter(source, subject, context_name)?;
    let CompilerSystemsDisposition::Fatal { message: failure_message } =
        action_disposition(source, failure, &failure_name)?
    else {
        return Err(critical_diagnostic(
            source,
            failure.span(),
            "critical entry failure must enter `fatal`",
        ));
    };
    let success_statements = action_block(source, success, "critical entry success")?;
    let [marker, restore_and_tail @ ..] = success_statements else {
        return Err(critical_diagnostic(
            source,
            span,
            "critical entry success requires a scoped marker and matching restoration",
        ));
    };
    let marker = analyze_operation(
        source,
        marker,
        CompilerSystemsContextKind::Bootstrap,
        &critical_name,
    )?;
    if !matches!(marker, CompilerSystemsOperation::ConsoleWrite { .. }) {
        return Err(critical_diagnostic(
            source,
            statement_span(success_statements.first().expect("marker exists")),
            "the initial critical scope requires a nonblocking console marker",
        ));
    }
    let (restored_name, tail) =
        parse_critical_restore_scope(source, restore_and_tail, &critical_name)?;
    let (ordinary, mut checked) =
        analyze_mapping_trailing_bootstrap(source, success, tail, &restored_name, storage)?;
    let mut operations = vec![
        CompilerSystemsOperation::EnterCritical {
            domain: CompilerCriticalDomain::LocalMaskableInterrupts,
            failure_message,
        },
        marker,
        CompilerSystemsOperation::RestoreCritical {
            domain: CompilerCriticalDomain::LocalMaskableInterrupts,
        },
    ];
    for operation in ordinary {
        operations.push(analyze_operation(
            source,
            operation,
            CompilerSystemsContextKind::Bootstrap,
            &restored_name,
        )?);
    }
    operations.append(&mut checked.operations);
    Ok(CheckedBootstrapRegionDecision {
        operations,
        disposition: checked.disposition,
    })
}

fn parse_critical_enter(
    source: &SourceText,
    expression: &Expression,
    context_name: &str,
) -> Result<(), Diagnostic> {
    let Expression::Application { items, span } = expression else {
        return Err(critical_diagnostic(
            source,
            expression.span(),
            "critical entry must be `context critical enter (domain is local-maskable-interrupts)`",
        ));
    };
    let [context, critical, enter, Expression::Product { fields, .. }] = items.as_slice() else {
        return Err(critical_diagnostic(
            source,
            *span,
            "critical entry must be `context critical enter (domain is local-maskable-interrupts)`",
        ));
    };
    let [ProductField {
        label: Some(label),
        value,
        ..
    }] = fields.as_slice()
    else {
        return Err(critical_diagnostic(
            source,
            *span,
            "critical entry requires exactly one named `domain`",
        ));
    };
    if !identifier_is(source, context, context_name)
        || !identifier_is(source, critical, "critical")
        || !identifier_is(source, enter, "enter")
        || source.slice(*label) != "domain"
        || !identifier_is(source, value, "local-maskable-interrupts")
    {
        return Err(critical_diagnostic(
            source,
            *span,
            "critical entry requires the current affine context and `local-maskable-interrupts` domain",
        ));
    }
    Ok(())
}

fn parse_critical_restore_scope<'a>(
    source: &SourceText,
    statements: &'a [Statement],
    critical_name: &str,
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
            return Err(critical_diagnostic(
                source,
                statements.first().map_or(Span::new(0, 0), statement_span),
                "critical scope must bind the context returned by restoration",
            ));
        }
    };
    let [critical, restore] = items.as_slice() else {
        return Err(critical_diagnostic(
            source,
            span,
            "critical restoration must be `restored is critical restore`",
        ));
    };
    if !identifier_is(source, critical, critical_name) || !identifier_is(source, restore, "restore")
    {
        return Err(critical_diagnostic(
            source,
            span,
            "critical restoration requires the live affine critical context",
        ));
    }
    Ok((source.slice(name).to_owned(), tail))
}

fn critical_diagnostic(source: &SourceText, span: Span, message: impl Into<String>) -> Diagnostic {
    source_diagnostic(source, "E-SYSTEMS-CRITICAL", span, message)
}
