fn analyze_boot_memory_decision(
    source: &SourceText,
    body: &[Statement],
    entered_context: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CheckedBootstrapRegionDecision, Diagnostic> {
    let [
        Statement::Expression(Expression::DecisionTable {
            subject,
            rules,
            span,
        }),
    ] = body
    else {
        return Err(boot_memory_diagnostic(
            source,
            body.first().map_or(Span::new(0, 0), statement_span),
            "the bootstrap handler must begin by exhaustively deciding `context boot describe memory`",
        ));
    };
    let Expression::Application {
        items,
        span: subject_span,
    } = subject.as_ref()
    else {
        return Err(boot_memory_diagnostic(
            source,
            subject.span(),
            "boot-memory decision subject must be `context boot describe memory`",
        ));
    };
    let [context, boot, describe, memory] = items.as_slice() else {
        return Err(boot_memory_diagnostic(
            source,
            *subject_span,
            "boot-memory decision subject must be `context boot describe memory`",
        ));
    };
    if !identifier_is(source, context, entered_context)
        || !identifier_is(source, boot, "boot")
        || !identifier_is(source, describe, "describe")
        || !identifier_is(source, memory, "memory")
    {
        return Err(boot_memory_diagnostic(
            source,
            *subject_span,
            "boot-memory description requires the live entered bootstrap context",
        ));
    }
    let (described_name, success, failure_name, failure) =
        boot_memory_result_actions(source, rules, *span)?;
    let failure_disposition = action_disposition(source, failure, &failure_name)?;
    let CompilerSystemsDisposition::Fatal {
        message: failure_message,
    } = failure_disposition
    else {
        return Err(boot_memory_diagnostic(
            source,
            failure.span(),
            "boot-memory failure context admits only `fatal`",
        ));
    };
    let Expression::Block {
        statements,
        span: success_span,
    } = success
    else {
        return Err(boot_memory_diagnostic(
            source,
            success.span(),
            "boot-memory success requires a block using its memory-described context",
        ));
    };
    if statements.len() < 3 {
        return Err(boot_memory_diagnostic(
            source,
            *success_span,
            "boot-memory success must continue with the refined context",
        ));
    }
    let (operations, allocation) = statements.split_at(statements.len() - 3);
    let mut checked_operations =
        vec![CompilerSystemsOperation::DescribeBootMemory { failure_message }];
    for operation in operations {
        checked_operations.push(analyze_operation(
            source,
            operation,
            CompilerSystemsContextKind::Bootstrap,
            &described_name,
        )?);
    }
    let checked = analyze_bootstrap_region_sequence(source, allocation, &described_name, storage)?;
    checked_operations.extend(checked.operations);
    Ok(CheckedBootstrapRegionDecision {
        operations: checked_operations,
        disposition: checked.disposition,
    })
}

fn boot_memory_result_actions<'a>(
    source: &SourceText,
    rules: &'a [DecisionRule],
    span: Span,
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
                return Err(boot_memory_diagnostic(
                    source,
                    rule.span,
                    "boot-memory description requires exactly one `Ok described` and one `Error failure` action",
                ));
            }
        }
    }
    let (Some((ok_name, ok_action)), Some((error_name, error_action))) = (ok, error) else {
        return Err(boot_memory_diagnostic(
            source,
            span,
            "boot-memory description requires exhaustive `Ok` and `Error` actions",
        ));
    };
    Ok((ok_name, ok_action, error_name, error_action))
}

fn boot_memory_diagnostic(
    source: &SourceText,
    span: Span,
    message: impl Into<String>,
) -> Diagnostic {
    source_diagnostic(source, "E-SYSTEMS-BOOT-MEMORY", span, message)
}
