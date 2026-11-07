fn parse_pci_region_dma_reference_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerPciRegionDmaReferenceRequest, Diagnostic> {
    if fields.len() != 14 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-PUBLISH",
            span,
            "PCI-region DMA-reference publication requires exactly fourteen region, buffer, and domain fields",
        ));
    }
    let width = field_natural(source, &fields[9], "width-bytes")?;
    let request = CompilerPciRegionDmaReferenceRequest {
        segment: u16::try_from(field_natural(source, &fields[0], "segment")?).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PUBLISH",
                span,
                "PCI segment exceeds 16 bits",
            )
        })?,
        bus: u8::try_from(field_natural(source, &fields[1], "bus")?).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PUBLISH",
                span,
                "PCI bus exceeds 8 bits",
            )
        })?,
        device: u8::try_from(field_natural(source, &fields[2], "device")?).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PUBLISH",
                span,
                "PCI device exceeds 8 bits",
            )
        })?,
        function: u8::try_from(field_natural(source, &fields[3], "function")?).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PUBLISH",
                span,
                "PCI function exceeds 8 bits",
            )
        })?,
        access_capability_offset_bytes: u8::try_from(field_natural(
            source,
            &fields[4],
            "access-capability-offset-bytes",
        )?)
        .map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PUBLISH",
                span,
                "PCI access-capability offset exceeds 8 bits",
            )
        })?,
        bar: u8::try_from(field_natural(source, &fields[5], "bar")?).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PUBLISH",
                span,
                "PCI BAR exceeds 8 bits",
            )
        })?,
        region_offset_bytes: u32::try_from(field_natural(
            source,
            &fields[6],
            "region-offset-bytes",
        )?)
        .map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PUBLISH",
                span,
                "PCI region offset exceeds 32 bits",
            )
        })?,
        region_length_bytes: u32::try_from(field_natural(
            source,
            &fields[7],
            "region-length-bytes",
        )?)
        .map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PUBLISH",
                span,
                "PCI region length exceeds 32 bits",
            )
        })?,
        register_offset_bytes: u32::try_from(field_natural(
            source,
            &fields[8],
            "register-offset-bytes",
        )?)
        .map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PUBLISH",
                span,
                "PCI register offset exceeds 32 bits",
            )
        })?,
        width_bytes: u8::try_from(width).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PUBLISH",
                span,
                "DMA-reference width exceeds 8 bits",
            )
        })?,
        buffer_identity: field_natural(source, &fields[10], "buffer-identity")?,
        buffer_offset_bytes: field_natural(source, &fields[11], "buffer-offset-bytes")?,
        device_identity: field_natural(source, &fields[12], "device-identity")?,
        translation_domain_identity: field_natural(
            source,
            &fields[13],
            "translation-domain-identity",
        )?,
    };
    if !request.is_well_formed()
        || !CompilerPciRegionDmaReferenceRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ADDRESSES
            .contains(&request)
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-PUBLISH",
            span,
            "the initial Q35 profile admits only the three sealed virtio-rng queue-address publications",
        ));
    }
    Ok(request)
}

fn analyze_disposition(
    source: &SourceText,
    statement: &Statement,
    context: CompilerSystemsContextKind,
    context_name: &str,
) -> Result<CompilerSystemsDisposition, Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DISPOSITION",
            statement_span(statement),
            "a systems handler must end in one context-consuming disposition",
        ));
    };
    if matches!(
        context,
        CompilerSystemsContextKind::DebugBreak
            | CompilerSystemsContextKind::LocalNotificationInterrupt
            | CompilerSystemsContextKind::DeadlineInterrupt
    ) && let [receiver, resume] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, resume, "resume")
    {
        return Ok(CompilerSystemsDisposition::Resume);
    }
    if context == CompilerSystemsContextKind::DeadlineInterrupt
        && let [receiver, preempt, current, kernel, context_word] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, preempt, "preempt")
        && identifier_is(source, current, "current")
        && identifier_is(source, kernel, "kernel")
        && identifier_is(source, context_word, "context")
    {
        return Ok(CompilerSystemsDisposition::PreemptKernelContext);
    }
    if let [receiver, fatal, Expression::String(message)] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, fatal, "fatal")
    {
        let message = parse_string(source.slice(*message)).ok_or_else(|| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DISPOSITION",
                *message,
                "invalid fatal text",
            )
        })?;
        return Ok(CompilerSystemsDisposition::Fatal {
            message: message.into(),
        });
    }
    Err(source_diagnostic(
        source,
        "E-SYSTEMS-DISPOSITION",
        *span,
        "final operation is not a disposition admitted by this entry context",
    ))
}

fn clauses_are_empty(clauses: &FunctionClauses) -> bool {
    clauses.requires.is_none()
        && clauses.effects.is_none()
        && clauses.guarantees.is_none()
        && clauses.result_binding.is_none()
        && clauses.ensures.is_none()
}

fn affine_context_error(source: &SourceText, span: Span) -> Diagnostic {
    source_diagnostic(
        source,
        "E-SYSTEMS-AFFINE-CONTEXT",
        span,
        "a systems context may occur only as the receiver of an admitted operation and may not be rebound, stored, passed, captured, or returned",
    )
}

fn identifier_is(source: &SourceText, expression: &Expression, expected: &str) -> bool {
    matches!(expression, Expression::Identifier(span) if source.slice(*span) == expected)
}

fn source_diagnostic(
    source: &SourceText,
    code: impl Into<String>,
    span: Span,
    message: impl Into<String>,
) -> Diagnostic {
    let start = span.start.min(source.as_str().len());
    let end = span.end.min(source.as_str().len()).max(start);
    let position = source.position(start);
    let line = source
        .as_str()
        .lines()
        .nth(position.line.saturating_sub(1))
        .map(ToOwned::to_owned);
    let width = source.slice(Span::new(start, end)).chars().count().max(1);
    Diagnostic::error(code, position.line, position.column, message)
        .with_source_span(span)
        .with_source_excerpt(line, width)
}

fn statement_span(statement: &Statement) -> Span {
    match statement {
        Statement::LanguageSelection { span, .. }
        | Statement::LibrarySelection { span, .. }
        | Statement::Published { span, .. }
        | Statement::DiagnosticControl { span, .. }
        | Statement::Implementation { span, .. }
        | Statement::StateField { name: span, .. }
        | Statement::ContextAssignment { span, .. }
        | Statement::Function { span, .. }
        | Statement::Generator { span, .. }
        | Statement::Union { span, .. }
        | Statement::Interface { span, .. }
        | Statement::InterfaceImplementation { span, .. }
        | Statement::Foreach { span, .. }
        | Statement::Discard { span, .. } => *span,
        Statement::Binding { name, value, .. } => Span::new(name.start, value.span().end),
        Statement::Return { keyword, value } => Span::new(keyword.start, value.span().end),
        Statement::Expression(expression) => expression.span(),
    }
}
