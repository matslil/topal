fn parse_region_release(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    region_name: &str,
) -> Result<(), Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(storage_diagnostic(
            source,
            statement_span(statement),
            "bootstrap region must be released exactly once",
        ));
    };
    let [context, bootstrap, release, region] = items.as_slice() else {
        return Err(storage_diagnostic(
            source,
            *span,
            "release must be `context bootstrap release region`",
        ));
    };
    if !identifier_is(source, context, context_name)
        || !identifier_is(source, bootstrap, "bootstrap")
        || !identifier_is(source, release, "release")
        || !identifier_is(source, region, region_name)
    {
        return Err(storage_diagnostic(
            source,
            *span,
            "release requires the live bootstrap context and its affine region",
        ));
    }
    Ok(())
}

fn analyze_operation(
    source: &SourceText,
    statement: &Statement,
    context: CompilerSystemsContextKind,
    context_name: &str,
) -> Result<CompilerSystemsOperation, Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(affine_context_error(source, statement_span(statement)));
    };
    if let [receiver, console, write, Expression::String(text)] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, console, "console")
        && identifier_is(source, write, "write")
    {
        let text = parse_string(source.slice(*text)).ok_or_else(|| {
            source_diagnostic(
                source,
                "E-SYSTEMS-CONSOLE",
                *text,
                "invalid static console text",
            )
        })?;
        return Ok(CompilerSystemsOperation::ConsoleWrite { text: text.into() });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            pci,
            region,
            register,
            read,
            eventually,
            equals,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, pci, "pci")
        && identifier_is(source, region, "region")
        && identifier_is(source, register, "register")
        && identifier_is(source, read, "read")
        && identifier_is(source, eventually, "eventually")
        && identifier_is(source, equals, "equals")
    {
        let request = parse_pci_region_register_request(
            source,
            fields,
            *span,
            CompilerPciRegionRegisterDirection::ReadEventuallyEquals,
        )?;
        return Ok(CompilerSystemsOperation::AccessPciRegionRegister { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            pci,
            configuration,
            dword,
            read,
            equals,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, pci, "pci")
        && identifier_is(source, configuration, "configuration")
        && identifier_is(source, dword, "dword")
        && identifier_is(source, read, "read")
        && identifier_is(source, equals, "equals")
    {
        let request = parse_pci_configuration_dword_request(source, fields, *span)?;
        if !CompilerPciConfigurationDwordRequest::INITIAL_Q35_VIRTIO_RNG_DISCOVERY
            .contains(&request)
        {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG",
                *span,
                "the initial Q35 profile admits only requests from the sealed segment-zero 00:05.0 virtio-rng capability-discovery sequence",
            ));
        }
        return Ok(CompilerSystemsOperation::ObservePciConfigurationDword { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            pci,
            configuration,
            word,
            update,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, pci, "pci")
        && identifier_is(source, configuration, "configuration")
        && identifier_is(source, word, "word")
        && identifier_is(source, update, "update")
    {
        let request = parse_pci_configuration_word_update_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::UpdatePciConfigurationWord { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            dma,
            buffer,
            create,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, dma, "dma")
        && identifier_is(source, buffer, "buffer")
        && identifier_is(source, create, "create")
    {
        let request = parse_dma_buffer_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::CreateDmaBuffer { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            dma,
            buffer,
            bind,
            reference,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, dma, "dma")
        && identifier_is(source, buffer, "buffer")
        && identifier_is(source, bind, "bind")
        && identifier_is(source, reference, "reference")
    {
        let request = parse_dma_buffer_reference_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::BindDmaBufferReference { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            dma,
            buffer,
            integer,
            store,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, dma, "dma")
        && identifier_is(source, buffer, "buffer")
        && identifier_is(source, integer, "integer")
        && identifier_is(source, store, "store")
    {
        let request = parse_dma_buffer_integer_store_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::StoreDmaBufferInteger { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            dma,
            buffer,
            integer,
            read,
            equals,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, dma, "dma")
        && identifier_is(source, buffer, "buffer")
        && identifier_is(source, integer, "integer")
        && identifier_is(source, read, "read")
        && identifier_is(source, equals, "equals")
    {
        let request = parse_dma_buffer_integer_load_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::LoadDmaBufferIntegerEquals { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            dma,
            buffer,
            prepare,
            device,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, dma, "dma")
        && identifier_is(source, buffer, "buffer")
        && identifier_is(source, prepare, "prepare")
        && identifier_is(source, device, "device")
    {
        let request = parse_dma_buffer_preparation_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::PrepareDmaBufferForDevice { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            dma,
            buffer,
            complete,
            cpu,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, dma, "dma")
        && identifier_is(source, buffer, "buffer")
        && identifier_is(source, complete, "complete")
        && identifier_is(source, cpu, "cpu")
    {
        let request = parse_dma_buffer_preparation_request(source, fields, *span)?;
        if !CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG.contains(&request) {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-DMA-COMPLETE",
                *span,
                "the initial Q35 profile admits only completion of its sealed virtio-rng DMA buffers",
            ));
        }
        return Ok(CompilerSystemsOperation::CompleteDmaBufferForCpu { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            sample,
            capture,
            dma,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, sample, "sample")
        && identifier_is(source, capture, "capture")
        && identifier_is(source, dma, "dma")
    {
        let request = parse_entropy_sample_capture_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::CaptureEntropySample { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            sample,
            assess,
            quality,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, sample, "sample")
        && identifier_is(source, assess, "assess")
        && identifier_is(source, quality, "quality")
    {
        let request = parse_entropy_sample_quality_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::AssessEntropySampleQuality { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            pool,
            create,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, pool, "pool")
        && identifier_is(source, create, "create")
    {
        let request = parse_entropy_pool_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::CreateEntropyPool { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            pool,
            mix,
            sample,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, pool, "pool")
        && identifier_is(source, mix, "mix")
        && identifier_is(source, sample, "sample")
    {
        let request = parse_entropy_pool_mix_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::MixEntropyPoolSample { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            pool,
            credit,
            contribution,
            Expression::Product { fields, span },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, pool, "pool")
        && identifier_is(source, credit, "credit")
        && identifier_is(source, contribution, "contribution")
    {
        let request = parse_entropy_pool_credit_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::CreditEntropyPoolContribution { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            generator,
            initialize,
            Expression::Product { fields, span },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, generator, "generator")
        && identifier_is(source, initialize, "initialize")
    {
        let request = parse_entropy_generator_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::InitializeEntropyGenerator { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            generator,
            extract,
            Expression::Product { fields, span },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, generator, "generator")
        && identifier_is(source, extract, "extract")
    {
        let request = parse_entropy_generator_extract_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::ExtractEntropyGenerator { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            output,
            deliver,
            initial,
            auxv,
            random,
            Expression::Product { fields, span },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, output, "output")
        && identifier_is(source, deliver, "deliver")
        && identifier_is(source, initial, "initial")
        && identifier_is(source, auxv, "auxv")
        && identifier_is(source, random, "random")
    {
        let request = parse_entropy_output_delivery_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::DeliverInitialAuxvRandom { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            output,
            reserve,
            initial,
            user,
            random,
            Expression::Product { fields, span },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, output, "output")
        && identifier_is(source, reserve, "reserve")
        && identifier_is(source, initial, "initial")
        && identifier_is(source, user, "user")
        && identifier_is(source, random, "random")
    {
        let request = parse_entropy_output_reservation_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::ReserveInitialUserRandom { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            user,
            random,
            bind,
            reusable,
            Expression::Product { fields, span },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, user, "user")
        && identifier_is(source, random, "random")
        && identifier_is(source, bind, "bind")
        && identifier_is(source, reusable, "reusable")
    {
        let request = parse_entropy_user_random_service_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::BindReusableUserRandom { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            user,
            random,
            expand,
            small,
            requests,
            Expression::Product { fields, span },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, user, "user")
        && identifier_is(source, random, "random")
        && identifier_is(source, expand, "expand")
        && identifier_is(source, small, "small")
        && identifier_is(source, requests, "requests")
    {
        let request = parse_entropy_user_random_small_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::ExpandSmallUserRandomRequests { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            entropy,
            user,
            random,
            expand,
            large,
            requests,
            Expression::Product { fields, span },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, entropy, "entropy")
        && identifier_is(source, user, "user")
        && identifier_is(source, random, "random")
        && identifier_is(source, expand, "expand")
        && identifier_is(source, large, "large")
        && identifier_is(source, requests, "requests")
    {
        let request = parse_entropy_user_random_large_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::ExpandLargeUserRandomRequests { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            pci,
            region,
            write,
            dma,
            reference,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, pci, "pci")
        && identifier_is(source, region, "region")
        && identifier_is(source, write, "write")
        && identifier_is(source, dma, "dma")
        && identifier_is(source, reference, "reference")
    {
        let request = parse_pci_region_dma_reference_request(source, fields, *span)?;
        return Ok(CompilerSystemsOperation::PublishPciRegionDmaReference { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            pci,
            region,
            register,
            direction,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, pci, "pci")
        && identifier_is(source, region, "region")
        && identifier_is(source, register, "register")
        && identifier_is(source, direction, "write")
    {
        let request = parse_pci_region_register_request(
            source,
            fields,
            *span,
            CompilerPciRegionRegisterDirection::Write,
        )?;
        return Ok(CompilerSystemsOperation::AccessPciRegionRegister { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [
            receiver,
            pci,
            region,
            register,
            read,
            equals,
            Expression::Product { fields, .. },
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, pci, "pci")
        && identifier_is(source, region, "region")
        && identifier_is(source, register, "register")
        && identifier_is(source, read, "read")
        && identifier_is(source, equals, "equals")
    {
        let request = parse_pci_region_register_request(
            source,
            fields,
            *span,
            CompilerPciRegionRegisterDirection::ReadEquals,
        )?;
        return Ok(CompilerSystemsOperation::AccessPciRegionRegister { request });
    }
    if context == CompilerSystemsContextKind::Bootstrap
        && let [receiver, debug, break_operation] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, debug, "debug")
        && identifier_is(source, break_operation, "break")
    {
        return Ok(CompilerSystemsOperation::DebugBreak);
    }
    if context == CompilerSystemsContextKind::KernelThread
        && let [
            receiver,
            kernel,
            context_word,
            await_word,
            deadline,
            preemption,
        ] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, kernel, "kernel")
        && identifier_is(source, context_word, "context")
        && identifier_is(source, await_word, "await")
        && identifier_is(source, deadline, "deadline")
        && identifier_is(source, preemption, "preemption")
    {
        return Ok(CompilerSystemsOperation::AwaitDeadlinePreemption);
    }
    if context == CompilerSystemsContextKind::KernelThread
        && let [receiver, kernel, completion, await_word] = items.as_slice()
        && identifier_is(source, receiver, context_name)
        && identifier_is(source, kernel, "kernel")
        && identifier_is(source, completion, "completion")
        && identifier_is(source, await_word, "await")
    {
        return Ok(CompilerSystemsOperation::AwaitKernelCompletion);
    }
    Err(source_diagnostic(
        source,
        "E-SYSTEMS-OPERATION",
        *span,
        "operation is not admitted by this live systems entry context",
    ))
}

fn parse_pci_configuration_dword_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerPciConfigurationDwordRequest, Diagnostic> {
    const LABELS: [&str; 6] = [
        "segment",
        "bus",
        "device",
        "function",
        "register-offset-bytes",
        "expected-value",
    ];
    if fields.len() != LABELS.len() {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-PCI-CONFIG",
            span,
            "PCI configuration dword observation requires exactly segment, bus, device, function, register-offset-bytes, and expected-value",
        ));
    }
    let mut values = [0_u64; 6];
    for ((field, expected_label), value) in fields.iter().zip(LABELS).zip(&mut values) {
        let Some(label) = field.label else {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG",
                field.value.span(),
                format!("PCI configuration dword observation requires named `{expected_label}`"),
            ));
        };
        if source.slice(label) != expected_label {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG",
                label,
                format!(
                    "PCI configuration dword observation requires `{expected_label}` in source order"
                ),
            ));
        }
        *value = parse_storage_natural(source, &field.value)?;
    }
    let request = CompilerPciConfigurationDwordRequest {
        segment: u16::try_from(values[0]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG",
                span,
                "PCI segment exceeds 65535",
            )
        })?,
        bus: u8::try_from(values[1]).map_err(|_| {
            source_diagnostic(source, "E-SYSTEMS-PCI-CONFIG", span, "PCI bus exceeds 255")
        })?,
        device: u8::try_from(values[2]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG",
                span,
                "PCI device exceeds 31",
            )
        })?,
        function: u8::try_from(values[3]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG",
                span,
                "PCI function exceeds 7",
            )
        })?,
        register_offset_bytes: u8::try_from(values[4]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG",
                span,
                "PCI register offset exceeds 252",
            )
        })?,
        expected_value: u32::try_from(values[5]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG",
                span,
                "PCI expected value exceeds 32 bits",
            )
        })?,
    };
    if !request.is_well_formed() {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-PCI-CONFIG",
            span,
            "PCI configuration request requires device 0..31, function 0..7, and a four-byte-aligned register offset in 0..252",
        ));
    }
    Ok(request)
}

fn parse_pci_configuration_word_update_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerPciConfigurationWordUpdateRequest, Diagnostic> {
    const LABELS: [&str; 8] = [
        "segment",
        "bus",
        "device",
        "function",
        "register-offset-bytes",
        "mask",
        "expected-masked-value",
        "desired-masked-value",
    ];
    if fields.len() != LABELS.len() {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-PCI-CONFIG-UPDATE",
            span,
            "PCI configuration-word update requires exactly the sealed function, offset, mask, pre-state, and post-state fields",
        ));
    }
    let mut values = [0_u64; 8];
    for ((field, expected_label), value) in fields.iter().zip(LABELS).zip(&mut values) {
        let Some(label) = field.label else {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG-UPDATE",
                field.value.span(),
                format!("PCI configuration-word update requires named `{expected_label}`"),
            ));
        };
        if source.slice(label) != expected_label {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG-UPDATE",
                label,
                format!(
                    "PCI configuration-word update requires `{expected_label}` in source order"
                ),
            ));
        }
        *value = parse_storage_natural(source, &field.value)?;
    }
    let request = CompilerPciConfigurationWordUpdateRequest {
        segment: u16::try_from(values[0]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG-UPDATE",
                span,
                "PCI segment exceeds 65535",
            )
        })?,
        bus: u8::try_from(values[1]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG-UPDATE",
                span,
                "PCI bus exceeds 255",
            )
        })?,
        device: u8::try_from(values[2]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG-UPDATE",
                span,
                "PCI device exceeds 31",
            )
        })?,
        function: u8::try_from(values[3]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG-UPDATE",
                span,
                "PCI function exceeds 7",
            )
        })?,
        register_offset_bytes: u8::try_from(values[4]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG-UPDATE",
                span,
                "PCI configuration-word offset exceeds 255",
            )
        })?,
        mask: u16::try_from(values[5]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG-UPDATE",
                span,
                "PCI configuration-word mask exceeds 65535",
            )
        })?,
        expected_masked_value: u16::try_from(values[6]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG-UPDATE",
                span,
                "PCI configuration-word expected value exceeds 65535",
            )
        })?,
        desired_masked_value: u16::try_from(values[7]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-CONFIG-UPDATE",
                span,
                "PCI configuration-word desired value exceeds 65535",
            )
        })?,
    };
    if !request.is_well_formed()
        || request != CompilerPciConfigurationWordUpdateRequest::INITIAL_Q35_VIRTIO_RNG_BUS_MASTER
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-PCI-CONFIG-UPDATE",
            span,
            "the initial Q35 profile admits only the sealed virtio-rng bus-master command transition",
        ));
    }
    Ok(request)
}

fn parse_dma_buffer_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerDmaBufferRequest, Diagnostic> {
    if fields.len() != 8 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-BUFFER",
            span,
            "DMA-buffer creation requires exactly identity, device, translation-domain, size, alignment, direction, visibility, and role fields",
        ));
    }
    let direction_value = field_static_string(source, &fields[5], "direction")?;
    let direction = match direction_value.as_str() {
        "bidirectional" => CompilerDmaDirection::Bidirectional,
        "device-to-memory" => CompilerDmaDirection::DeviceToMemory,
        _ => {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-DMA-BUFFER",
                span,
                "DMA-buffer direction must be `bidirectional` or `device-to-memory`",
            ));
        }
    };
    let visibility_value = field_static_string(source, &fields[6], "visibility")?;
    let visibility = match visibility_value.as_str() {
        "provider-managed" => CompilerDmaVisibilityPolicy::ProviderManaged,
        _ => {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-DMA-BUFFER",
                span,
                "DMA-buffer visibility must be `provider-managed`",
            ));
        }
    };
    let role_value = field_static_string(source, &fields[7], "role")?;
    let role = match role_value.as_str() {
        "virtqueue-control" => CompilerDmaBufferRole::VirtqueueControl,
        "entropy-payload" => CompilerDmaBufferRole::EntropyPayload,
        _ => {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-DMA-BUFFER",
                span,
                "DMA-buffer role must be `virtqueue-control` or `entropy-payload`",
            ));
        }
    };
    let request = CompilerDmaBufferRequest {
        buffer_identity: field_natural(source, &fields[0], "buffer-identity")?,
        device_identity: field_natural(source, &fields[1], "device-identity")?,
        translation_domain_identity: field_natural(
            source,
            &fields[2],
            "translation-domain-identity",
        )?,
        byte_count: field_natural(source, &fields[3], "byte-count")?,
        alignment_bytes: field_natural(source, &fields[4], "alignment-bytes")?,
        direction,
        visibility,
        role,
    };
    if !request.is_well_formed()
        || !CompilerDmaBufferRequest::INITIAL_Q35_VIRTIO_RNG.contains(&request)
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-BUFFER",
            span,
            "the initial Q35 profile admits only the two sealed virtio-rng DMA-buffer requests",
        ));
    }
    Ok(request)
}

fn parse_dma_buffer_reference_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerDmaBufferReferenceRequest, Diagnostic> {
    if fields.len() != 5 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-CONSTRUCT",
            span,
            "DMA-reference binding requires destination identity/offset, referenced identity/offset, and width",
        ));
    }
    let width = field_natural(source, &fields[4], "width-bytes")?;
    let request = CompilerDmaBufferReferenceRequest {
        destination_buffer_identity: field_natural(
            source,
            &fields[0],
            "destination-buffer-identity",
        )?,
        destination_offset_bytes: field_natural(source, &fields[1], "destination-offset-bytes")?,
        referenced_buffer_identity: field_natural(
            source,
            &fields[2],
            "referenced-buffer-identity",
        )?,
        referenced_offset_bytes: field_natural(source, &fields[3], "referenced-offset-bytes")?,
        width_bytes: u8::try_from(width).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-CONSTRUCT",
                span,
                "DMA-reference width exceeds 255",
            )
        })?,
    };
    if !request.is_well_formed()
        || request != CompilerDmaBufferReferenceRequest::INITIAL_Q35_VIRTIO_RNG_DESCRIPTOR
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-CONSTRUCT",
            span,
            "the initial Q35 profile admits only the sealed virtio-rng descriptor reference",
        ));
    }
    Ok(request)
}

fn parse_dma_buffer_integer_store_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerDmaBufferIntegerStoreRequest, Diagnostic> {
    if fields.len() != 4 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-CONSTRUCT",
            span,
            "DMA integer store requires buffer identity, offset, width, and value",
        ));
    }
    let width = field_natural(source, &fields[2], "width-bytes")?;
    let request = CompilerDmaBufferIntegerStoreRequest {
        buffer_identity: field_natural(source, &fields[0], "buffer-identity")?,
        offset_bytes: field_natural(source, &fields[1], "offset-bytes")?,
        width_bytes: u8::try_from(width).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-CONSTRUCT",
                span,
                "DMA integer-store width exceeds 255",
            )
        })?,
        value: field_natural(source, &fields[3], "value")?,
    };
    if !request.is_well_formed()
        || !CompilerDmaBufferIntegerStoreRequest::INITIAL_Q35_VIRTIO_RNG_SPLIT_RING
            .contains(&request)
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-CONSTRUCT",
            span,
            "the initial Q35 profile admits only stores from the sealed virtio-rng split-ring construction",
        ));
    }
    Ok(request)
}

fn parse_dma_buffer_preparation_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerDmaBufferPreparationRequest, Diagnostic> {
    if fields.len() != 6 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-PREPARE",
            span,
            "DMA device preparation requires buffer, device, translation-domain, direction, visibility, and role fields",
        ));
    }
    let direction = match field_static_string(source, &fields[3], "direction")?.as_str() {
        "bidirectional" => CompilerDmaDirection::Bidirectional,
        "device-to-memory" => CompilerDmaDirection::DeviceToMemory,
        _ => {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PREPARE",
                span,
                "DMA preparation direction must be `bidirectional` or `device-to-memory`",
            ));
        }
    };
    let visibility = match field_static_string(source, &fields[4], "visibility")?.as_str() {
        "provider-managed" => CompilerDmaVisibilityPolicy::ProviderManaged,
        _ => {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PREPARE",
                span,
                "DMA preparation visibility must be `provider-managed`",
            ));
        }
    };
    let role = match field_static_string(source, &fields[5], "role")?.as_str() {
        "virtqueue-control" => CompilerDmaBufferRole::VirtqueueControl,
        "entropy-payload" => CompilerDmaBufferRole::EntropyPayload,
        _ => {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-DMA-PREPARE",
                span,
                "DMA preparation role must be `virtqueue-control` or `entropy-payload`",
            ));
        }
    };
    let request = CompilerDmaBufferPreparationRequest {
        buffer_identity: field_natural(source, &fields[0], "buffer-identity")?,
        device_identity: field_natural(source, &fields[1], "device-identity")?,
        translation_domain_identity: field_natural(
            source,
            &fields[2],
            "translation-domain-identity",
        )?,
        direction,
        visibility,
        role,
    };
    if !request.is_well_formed()
        || !CompilerDmaBufferPreparationRequest::INITIAL_Q35_VIRTIO_RNG.contains(&request)
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-PREPARE",
            span,
            "the initial Q35 profile admits only the sealed payload and control DMA-preparation requests",
        ));
    }
    Ok(request)
}

fn parse_dma_buffer_integer_load_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerDmaBufferIntegerLoadRequest, Diagnostic> {
    if fields.len() != 4 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-USED-RING",
            span,
            "DMA integer read requires buffer identity, offset, width, and expected value",
        ));
    }
    let width = field_natural(source, &fields[2], "width-bytes")?;
    let request = CompilerDmaBufferIntegerLoadRequest {
        buffer_identity: field_natural(source, &fields[0], "buffer-identity")?,
        offset_bytes: field_natural(source, &fields[1], "offset-bytes")?,
        width_bytes: u8::try_from(width).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-DMA-USED-RING",
                span,
                "DMA integer-read width exceeds 255",
            )
        })?,
        expected_value: field_natural(source, &fields[3], "expected-value")?,
    };
    if !request.is_well_formed()
        || !CompilerDmaBufferIntegerLoadRequest::INITIAL_Q35_VIRTIO_RNG_USED_RING.contains(&request)
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-DMA-USED-RING",
            span,
            "the initial Q35 profile admits only the sealed virtio-rng used-ring reads",
        ));
    }
    Ok(request)
}

fn parse_entropy_sample_capture_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropySampleCaptureRequest, Diagnostic> {
    if fields.len() != 10 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-SAMPLE",
            span,
            "entropy-sample capture requires sample, buffer, range, device, domain, direction, visibility, role, and credit fields",
        ));
    }
    let direction = match field_static_string(source, &fields[6], "direction")?.as_str() {
        "device-to-memory" => CompilerDmaDirection::DeviceToMemory,
        _ => {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-ENTROPY-SAMPLE",
                span,
                "entropy-sample DMA direction must be `device-to-memory`",
            ));
        }
    };
    let visibility = match field_static_string(source, &fields[7], "visibility")?.as_str() {
        "provider-managed" => CompilerDmaVisibilityPolicy::ProviderManaged,
        _ => {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-ENTROPY-SAMPLE",
                span,
                "entropy-sample DMA visibility must be `provider-managed`",
            ));
        }
    };
    let role = match field_static_string(source, &fields[8], "role")?.as_str() {
        "entropy-payload" => CompilerDmaBufferRole::EntropyPayload,
        _ => {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-ENTROPY-SAMPLE",
                span,
                "entropy-sample DMA role must be `entropy-payload`",
            ));
        }
    };
    let request = CompilerEntropySampleCaptureRequest {
        sample_identity: field_natural(source, &fields[0], "sample-identity")?,
        buffer_identity: field_natural(source, &fields[1], "buffer-identity")?,
        offset_bytes: field_natural(source, &fields[2], "offset-bytes")?,
        byte_count: field_natural(source, &fields[3], "byte-count")?,
        device_identity: field_natural(source, &fields[4], "device-identity")?,
        translation_domain_identity: field_natural(
            source,
            &fields[5],
            "translation-domain-identity",
        )?,
        direction,
        visibility,
        role,
        credited_entropy_bits: field_natural(source, &fields[9], "credited-entropy-bits")?,
    };
    if !request.is_well_formed()
        || request != CompilerEntropySampleCaptureRequest::INITIAL_Q35_VIRTIO_RNG
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-SAMPLE",
            span,
            "the initial Q35 profile admits only the sealed 32-byte uncredited virtio-rng sample capture",
        ));
    }
    Ok(request)
}

fn parse_entropy_sample_quality_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropySampleQualityRequest, Diagnostic> {
    if fields.len() != 6 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-QUALITY",
            span,
            "entropy-sample quality assessment requires sample, source, trust evidence, extent, quality, and credit fields",
        ));
    }
    let request = CompilerEntropySampleQualityRequest {
        sample_identity: field_natural(source, &fields[0], "sample-identity")?,
        source_identity: field_natural(source, &fields[1], "source-identity")?,
        trust_evidence_identity: field_natural(source, &fields[2], "trust-evidence-identity")?,
        byte_count: field_natural(source, &fields[3], "byte-count")?,
        quality_parts_per_1024: field_natural(
            source,
            &fields[4],
            "quality-parts-per-thousand-twenty-four",
        )?,
        credited_entropy_bits: field_natural(source, &fields[5], "credited-entropy-bits")?,
    };
    if !request.is_well_formed()
        || request != CompilerEntropySampleQualityRequest::INITIAL_Q35_VIRTIO_RNG
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-QUALITY",
            span,
            "the initial Q35 profile admits only the sealed fully credited 32-byte virtio-rng sample assessment",
        ));
    }
    Ok(request)
}

fn parse_entropy_pool_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropyPoolRequest, Diagnostic> {
    if fields.len() != 3 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-POOL",
            span,
            "entropy-pool creation requires identity, credit-capacity, and initial-credit fields",
        ));
    }
    let request = CompilerEntropyPoolRequest {
        pool_identity: field_natural(source, &fields[0], "pool-identity")?,
        credit_capacity_bits: field_natural(source, &fields[1], "credit-capacity-bits")?,
        initial_credited_entropy_bits: field_natural(
            source,
            &fields[2],
            "initial-credited-entropy-bits",
        )?,
    };
    if !request.is_well_formed() || request != CompilerEntropyPoolRequest::INITIAL_Q35 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-POOL",
            span,
            "the initial Q35 profile admits only the sealed empty 256-bit entropy pool",
        ));
    }
    Ok(request)
}

fn parse_entropy_pool_mix_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropyPoolMixRequest, Diagnostic> {
    if fields.len() != 6 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-MIX",
            span,
            "entropy-pool mixing requires pool, sample, conditioner, contribution, extent, and contribution-credit fields",
        ));
    }
    let request = CompilerEntropyPoolMixRequest {
        pool_identity: field_natural(source, &fields[0], "pool-identity")?,
        sample_identity: field_natural(source, &fields[1], "sample-identity")?,
        conditioner_identity: field_natural(source, &fields[2], "conditioner-identity")?,
        contribution_identity: field_natural(source, &fields[3], "contribution-identity")?,
        byte_count: field_natural(source, &fields[4], "byte-count")?,
        contribution_entropy_bits: field_natural(source, &fields[5], "contribution-entropy-bits")?,
    };
    if !request.is_well_formed() || request != CompilerEntropyPoolMixRequest::INITIAL_Q35 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-MIX",
            span,
            "the initial Q35 profile admits only the sealed first BLAKE2s-256 entropy-pool mix",
        ));
    }
    Ok(request)
}

fn parse_entropy_pool_credit_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropyPoolCreditRequest, Diagnostic> {
    if fields.len() != 7 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-CREDIT",
            span,
            "entropy-credit transfer requires pool, contribution, origin, prior, transfer, capacity, and result fields",
        ));
    }
    let request = CompilerEntropyPoolCreditRequest {
        pool_identity: field_natural(source, &fields[0], "pool-identity")?,
        contribution_identity: field_natural(source, &fields[1], "contribution-identity")?,
        conditioner_origin_identity: field_natural(
            source,
            &fields[2],
            "conditioner-origin-identity",
        )?,
        prior_credited_entropy_bits: field_natural(
            source,
            &fields[3],
            "prior-credited-entropy-bits",
        )?,
        contribution_entropy_bits: field_natural(source, &fields[4], "contribution-entropy-bits")?,
        credit_capacity_bits: field_natural(source, &fields[5], "credit-capacity-bits")?,
        resulting_credited_entropy_bits: field_natural(
            source,
            &fields[6],
            "resulting-credited-entropy-bits",
        )?,
    };
    if !request.is_well_formed() || request != CompilerEntropyPoolCreditRequest::INITIAL_Q35 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-CREDIT",
            span,
            "the initial Q35 profile admits only the sealed zero-to-256 contribution transfer",
        ));
    }
    Ok(request)
}

fn parse_entropy_generator_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropyGeneratorRequest, Diagnostic> {
    if fields.len() != 7 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-GENERATOR",
            span,
            "entropy-generator initialization requires pool, generator, algorithm, generation, key extent, required credit, and observed credit fields",
        ));
    }
    let request = CompilerEntropyGeneratorRequest {
        pool_identity: field_natural(source, &fields[0], "pool-identity")?,
        generator_identity: field_natural(source, &fields[1], "generator-identity")?,
        algorithm_identity: field_natural(source, &fields[2], "algorithm-identity")?,
        generation: field_natural(source, &fields[3], "generation")?,
        key_byte_count: field_natural(source, &fields[4], "key-byte-count")?,
        required_entropy_bits: field_natural(source, &fields[5], "required-entropy-bits")?,
        observed_pool_entropy_bits: field_natural(
            source,
            &fields[6],
            "observed-pool-entropy-bits",
        )?,
    };
    if !request.is_well_formed() || request != CompilerEntropyGeneratorRequest::INITIAL_Q35 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-GENERATOR",
            span,
            "the initial Q35 profile admits only the sealed generation-one 32-byte ChaCha20 initialization",
        ));
    }
    Ok(request)
}

fn parse_entropy_generator_extract_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropyGeneratorExtractRequest, Diagnostic> {
    if fields.len() != 10 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-EXTRACT",
            span,
            "entropy-generator extraction requires generator, algorithm, generations, output, block, split, counter, and nonce fields",
        ));
    }
    let request = CompilerEntropyGeneratorExtractRequest {
        generator_identity: field_natural(source, &fields[0], "generator-identity")?,
        algorithm_identity: field_natural(source, &fields[1], "algorithm-identity")?,
        prior_generation: field_natural(source, &fields[2], "prior-generation")?,
        resulting_generation: field_natural(source, &fields[3], "resulting-generation")?,
        output_identity: field_natural(source, &fields[4], "output-identity")?,
        block_byte_count: field_natural(source, &fields[5], "block-byte-count")?,
        output_offset_bytes: field_natural(source, &fields[6], "output-offset-bytes")?,
        output_byte_count: field_natural(source, &fields[7], "output-byte-count")?,
        counter: field_natural(source, &fields[8], "counter")?,
        nonce_identity: field_natural(source, &fields[9], "nonce-identity")?,
    };
    if !request.is_well_formed()
        || (request != CompilerEntropyGeneratorExtractRequest::INITIAL_Q35
            && request != CompilerEntropyGeneratorExtractRequest::INITIAL_Q35_GETRANDOM)
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-EXTRACT",
            span,
            "the initial Q35 profile admits only its two sealed consecutive 32-byte ChaCha20 extractions",
        ));
    }
    Ok(request)
}

fn parse_entropy_output_reservation_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropyOutputReservationRequest, Diagnostic> {
    if fields.len() != 12 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-RESERVE",
            span,
            "initial getrandom reservation requires output, extent, process, interface, service, flag-policy, and single-use fields",
        ));
    }
    let request = CompilerEntropyOutputReservationRequest {
        output_identity: field_natural(source, &fields[0], "output-identity")?,
        source_byte_count: field_natural(source, &fields[1], "source-byte-count")?,
        selected_offset_bytes: field_natural(source, &fields[2], "selected-offset-bytes")?,
        maximum_delivery_byte_count: field_natural(
            source,
            &fields[3],
            "maximum-delivery-byte-count",
        )?,
        discarded_offset_bytes: field_natural(source, &fields[4], "discarded-offset-bytes")?,
        discarded_byte_count: field_natural(source, &fields[5], "discarded-byte-count")?,
        process_identity: field_natural(source, &fields[6], "process-identity")?,
        interface_identity: field_natural(source, &fields[7], "interface-identity")?,
        service_identity: field_natural(source, &fields[8], "service-identity")?,
        supported_flag_mask: field_natural(source, &fields[9], "supported-flag-mask")?,
        incompatible_flag_mask: field_natural(source, &fields[10], "incompatible-flag-mask")?,
        single_use: field_natural(source, &fields[11], "single-use")? == 1,
    };
    if !request.is_well_formed()
        || request != CompilerEntropyOutputReservationRequest::INITIAL_GETRANDOM
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-RESERVE",
            span,
            "the initial profile admits only the sealed single-use 16-byte getrandom reservation from output two",
        ));
    }
    Ok(request)
}

fn parse_entropy_user_random_service_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropyUserRandomServiceRequest, Diagnostic> {
    if fields.len() != 13 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-USER-SERVICE",
            span,
            "reusable getrandom binding requires generator, generation, output, extents, process, interface, service, flag-policy, first-output, and reusable fields",
        ));
    }
    let request = CompilerEntropyUserRandomServiceRequest {
        generator_identity: field_natural(source, &fields[0], "generator-identity")?,
        generation: field_natural(source, &fields[1], "generation")?,
        initial_output_identity: field_natural(source, &fields[2], "initial-output-identity")?,
        block_byte_count: field_natural(source, &fields[3], "block-byte-count")?,
        output_byte_count: field_natural(source, &fields[4], "output-byte-count")?,
        maximum_delivery_byte_count: field_natural(
            source,
            &fields[5],
            "maximum-delivery-byte-count",
        )?,
        process_identity: field_natural(source, &fields[6], "process-identity")?,
        interface_identity: field_natural(source, &fields[7], "interface-identity")?,
        service_identity: field_natural(source, &fields[8], "service-identity")?,
        supported_flag_mask: field_natural(source, &fields[9], "supported-flag-mask")?,
        incompatible_flag_mask: field_natural(source, &fields[10], "incompatible-flag-mask")?,
        reserved_first_output: field_natural(source, &fields[11], "reserved-first-output")? == 1,
        reusable: field_natural(source, &fields[12], "reusable")? == 1,
    };
    if !request.is_well_formed()
        || request != CompilerEntropyUserRandomServiceRequest::INITIAL_GETRANDOM
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-USER-SERVICE",
            span,
            "the initial profile admits only its sealed generation-three reusable 32-byte getrandom service",
        ));
    }
    Ok(request)
}

fn parse_entropy_user_random_small_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropyUserRandomSmallRequest, Diagnostic> {
    if fields.len() != 8 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-USER-SMALL-REQUEST",
            span,
            "small-request getrandom refinement requires process, interface, service, algorithm, output, count, delivery, and completion fields",
        ));
    }
    let request = CompilerEntropyUserRandomSmallRequest {
        process_identity: field_natural(source, &fields[0], "process-identity")?,
        interface_identity: field_natural(source, &fields[1], "interface-identity")?,
        service_identity: field_natural(source, &fields[2], "service-identity")?,
        algorithm_identity: field_natural(source, &fields[3], "algorithm-identity")?,
        output_byte_count: field_natural(source, &fields[4], "output-byte-count")?,
        maximum_output_count: field_natural(source, &fields[5], "maximum-output-count")?,
        maximum_delivery_byte_count: field_natural(
            source,
            &fields[6],
            "maximum-delivery-byte-count",
        )?,
        complete_request: field_natural(source, &fields[7], "complete-request")? == 1,
    };
    if !request.is_well_formed()
        || request != CompilerEntropyUserRandomSmallRequest::INITIAL_GETRANDOM
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-USER-SMALL-REQUEST",
            span,
            "the initial profile admits only its sealed eight-output 256-byte complete getrandom service",
        ));
    }
    Ok(request)
}

fn parse_entropy_user_random_large_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropyUserRandomSmallRequest, Diagnostic> {
    if fields.len() != 8 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-USER-LARGE-REQUEST",
            span,
            "large-request getrandom refinement requires process, interface, service, algorithm, output, count, delivery, and completion fields",
        ));
    }
    let request = CompilerEntropyUserRandomSmallRequest {
        process_identity: field_natural(source, &fields[0], "process-identity")?,
        interface_identity: field_natural(source, &fields[1], "interface-identity")?,
        service_identity: field_natural(source, &fields[2], "service-identity")?,
        algorithm_identity: field_natural(source, &fields[3], "algorithm-identity")?,
        output_byte_count: field_natural(source, &fields[4], "output-byte-count")?,
        maximum_output_count: field_natural(source, &fields[5], "maximum-output-count")?,
        maximum_delivery_byte_count: field_natural(
            source,
            &fields[6],
            "maximum-delivery-byte-count",
        )?,
        complete_request: field_natural(source, &fields[7], "complete-request")? == 1,
    };
    if !request.is_well_formed()
        || request != CompilerEntropyUserRandomSmallRequest::LARGE_GETRANDOM
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-USER-LARGE-REQUEST",
            span,
            "the initial profile admits only its sealed 16-output 512-byte complete getrandom service",
        ));
    }
    Ok(request)
}

fn parse_entropy_output_delivery_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
) -> Result<CompilerEntropyOutputDeliveryRequest, Diagnostic> {
    if fields.len() != 10 {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-DELIVER",
            span,
            "initial AT_RANDOM delivery requires output, source, selected, discarded, address-space, interface, tag, and single-use fields",
        ));
    }
    let request = CompilerEntropyOutputDeliveryRequest {
        output_identity: field_natural(source, &fields[0], "output-identity")?,
        source_byte_count: field_natural(source, &fields[1], "source-byte-count")?,
        selected_offset_bytes: field_natural(source, &fields[2], "selected-offset-bytes")?,
        delivered_byte_count: field_natural(source, &fields[3], "delivered-byte-count")?,
        discarded_offset_bytes: field_natural(source, &fields[4], "discarded-offset-bytes")?,
        discarded_byte_count: field_natural(source, &fields[5], "discarded-byte-count")?,
        address_space_identity: field_natural(source, &fields[6], "address-space-identity")?,
        interface_identity: field_natural(source, &fields[7], "interface-identity")?,
        auxiliary_tag: field_natural(source, &fields[8], "auxiliary-tag")?,
        single_use: field_natural(source, &fields[9], "single-use")? == 1,
    };
    if !request.is_well_formed()
        || request != CompilerEntropyOutputDeliveryRequest::INITIAL_AUXV_RANDOM
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-ENTROPY-DELIVER",
            span,
            "the initial profile admits only single-use delivery of the first 16 bytes from output one to Linux AT_RANDOM with the remainder discarded",
        ));
    }
    Ok(request)
}

fn parse_pci_region_register_request(
    source: &SourceText,
    fields: &[ProductField],
    span: Span,
    direction: CompilerPciRegionRegisterDirection,
) -> Result<CompilerPciRegionRegisterRequest, Diagnostic> {
    const LABELS: [&str; 11] = [
        "segment",
        "bus",
        "device",
        "function",
        "access-capability-offset-bytes",
        "bar",
        "region-offset-bytes",
        "region-length-bytes",
        "register-offset-bytes",
        "width-bytes",
        "value",
    ];
    if fields.len() != LABELS.len() {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-PCI-REGION",
            span,
            "PCI region-register access requires exactly the sealed function, capability, BAR, region, register, width, and value fields",
        ));
    }
    let mut values = [0_u64; 11];
    for ((field, expected_label), value) in fields.iter().zip(LABELS).zip(&mut values) {
        let Some(label) = field.label else {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                field.value.span(),
                format!("PCI region-register access requires named `{expected_label}`"),
            ));
        };
        if source.slice(label) != expected_label {
            return Err(source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                label,
                format!("PCI region-register access requires `{expected_label}` in source order"),
            ));
        }
        *value = parse_storage_natural(source, &field.value)?;
    }
    let request = CompilerPciRegionRegisterRequest {
        segment: u16::try_from(values[0]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                span,
                "PCI segment exceeds 65535",
            )
        })?,
        bus: u8::try_from(values[1]).map_err(|_| {
            source_diagnostic(source, "E-SYSTEMS-PCI-REGION", span, "PCI bus exceeds 255")
        })?,
        device: u8::try_from(values[2]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                span,
                "PCI device exceeds 31",
            )
        })?,
        function: u8::try_from(values[3]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                span,
                "PCI function exceeds 7",
            )
        })?,
        access_capability_offset_bytes: u8::try_from(values[4]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                span,
                "PCI access capability offset exceeds 255",
            )
        })?,
        bar: u8::try_from(values[5]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                span,
                "PCI BAR index exceeds 255",
            )
        })?,
        region_offset_bytes: u32::try_from(values[6]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                span,
                "PCI region offset exceeds 32 bits",
            )
        })?,
        region_length_bytes: u32::try_from(values[7]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                span,
                "PCI region length exceeds 32 bits",
            )
        })?,
        register_offset_bytes: u32::try_from(values[8]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                span,
                "PCI register offset exceeds 32 bits",
            )
        })?,
        width_bytes: u8::try_from(values[9]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                span,
                "PCI register width exceeds 255",
            )
        })?,
        direction,
        value: u32::try_from(values[10]).map_err(|_| {
            source_diagnostic(
                source,
                "E-SYSTEMS-PCI-REGION",
                span,
                "PCI register value exceeds 32 bits",
            )
        })?,
    };
    if !request.is_well_formed()
        || !(CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_NEGOTIATION
            .contains(&request)
            || CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_DISCOVERY
                .contains(&request)
            || CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_ACTIVATION
                .contains(&request)
            || CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_QUEUE_NOTIFICATION
                .contains(&request)
            || CompilerPciRegionRegisterRequest::INITIAL_Q35_VIRTIO_RNG_COMPLETION_SIGNAL
                .contains(&request))
    {
        return Err(source_diagnostic(
            source,
            "E-SYSTEMS-PCI-REGION",
            span,
            "the initial Q35 profile admits only requests from the sealed virtio-rng feature-negotiation, queue-discovery, queue-activation, queue-notification, and completion-signal sequences",
        ));
    }
    Ok(request)
}
