fn analyze_kernel_mapping_sequence(
    source: &SourceText,
    statements: &[Statement],
    allocator_context: &str,
    extent_name: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CheckedBootstrapRegionDecision, Diagnostic> {
    let (subject, mapping_name, success, _problem_name, failure, span) =
        result_sequence(source, statements, "kernel mapping")?;
    let request = parse_kernel_mapping_request(source, subject, allocator_context, extent_name)?;
    let failure_disposition = action_disposition(source, failure, allocator_context)?;
    let CompilerSystemsDisposition::Fatal {
        message: failure_message,
    } = failure_disposition
    else {
        return Err(mapping_diagnostic(
            source,
            failure.span(),
            "kernel-mapping failure must consume the allocator context through `fatal`",
        ));
    };
    let Expression::Block {
        statements: success_statements,
        ..
    } = success
    else {
        return Err(mapping_diagnostic(
            source,
            span,
            "kernel-mapping success requires a block using and unmapping its affine capability",
        ));
    };
    let [store, load, comparison, true_rule, false_rule] = success_statements.as_slice() else {
        return Err(mapping_diagnostic(
            source,
            success.span(),
            "kernel-mapping success requires byte store, byte load, and an exhaustive equality decision",
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
    for offset in [store_offset, load_offset] {
        if offset >= 4096 {
            return Err(mapping_diagnostic(
                source,
                success.span(),
                format!("kernel-mapping byte offset {offset} is outside the one-frame mapping"),
            ));
        }
    }

    let true_statements = mapping_action_block(source, true_action, "successful mapping comparison")?;
    let (returned_extent, true_tail) = parse_kernel_unmap_scope(
        source,
        true_statements,
        allocator_context,
        &mapping_name,
    )?;
    if true_tail.len() < 3 {
        return Err(mapping_diagnostic(
            source,
            true_action.span(),
            "successful mapping comparison requires unmap, marker, frame release, and bootstrap allocation decision",
        ));
    }
    let [marker, release, rest @ ..] = true_tail else {
        unreachable!("length was checked")
    };
    let marker_span = statement_span(marker);
    let marker = analyze_operation(
        source,
        marker,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    let CompilerSystemsOperation::ConsoleWrite { .. } = marker else {
        return Err(mapping_diagnostic(
            source,
            marker_span,
            "successful kernel mapping must publish its marker after unmap",
        ));
    };
    parse_physical_frame_release(source, release, allocator_context, &returned_extent)?;
    let (ordinary, checked) = if matches!(
        rest.last(),
        Some(Statement::Expression(Expression::DecisionTable { .. }))
    ) {
        let (bootstrap, ordinary) = rest.split_last().expect("mapping tail is non-empty");
        (
            ordinary,
            analyze_bootstrap_region_decision(source, bootstrap, allocator_context, storage)?,
        )
    } else {
        if rest.len() < 3 {
            return Err(mapping_diagnostic(
                source,
                true_action.span(),
                "successful mapping comparison requires a bootstrap allocation decision",
            ));
        }
        let (ordinary, bootstrap) = rest.split_at(rest.len() - 3);
        (
            ordinary,
            analyze_bootstrap_region_sequence(source, bootstrap, allocator_context, storage)?,
        )
    };
    let mut trailing = Vec::new();
    for operation in ordinary {
        trailing.push(analyze_operation(
            source,
            operation,
            CompilerSystemsContextKind::Bootstrap,
            allocator_context,
        )?);
    }
    trailing.extend(checked.operations);

    let false_statements =
        mapping_action_block(source, false_action, "failed mapping comparison")?;
    let (failed_extent, false_tail) = parse_kernel_unmap_scope(
        source,
        false_statements,
        allocator_context,
        &mapping_name,
    )?;
    let [release, disposition] = false_tail else {
        return Err(mapping_diagnostic(
            source,
            false_action.span(),
            "failed mapping comparison requires unmap, frame release, and fatal disposition",
        ));
    };
    parse_physical_frame_release(source, release, allocator_context, &failed_extent)?;
    let failure = analyze_disposition(
        source,
        disposition,
        CompilerSystemsContextKind::Bootstrap,
        allocator_context,
    )?;
    let CompilerSystemsDisposition::Fatal {
        message: mismatch_message,
    } = failure
    else {
        return Err(mapping_diagnostic(
            source,
            statement_span(disposition),
            "failed mapping comparison must enter the fatal disposition",
        ));
    };

    let mut operations = vec![
        CompilerSystemsOperation::MapKernelFrames {
            request,
            failure_message,
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
        CompilerSystemsOperation::UnmapKernelFrames,
        marker,
        CompilerSystemsOperation::ReleasePhysicalFrames,
    ];
    operations.extend(trailing);
    Ok(CheckedBootstrapRegionDecision {
        operations,
        disposition: checked.disposition,
    })
}

fn parse_kernel_mapping_request(
    source: &SourceText,
    expression: &Expression,
    allocator_context: &str,
    extent_name: &str,
) -> Result<CompilerKernelMappingRequest, Diagnostic> {
    let Expression::Application { items, span } = expression else {
        return Err(mapping_diagnostic(
            source,
            expression.span(),
            "kernel-mapping subject must be `context kernel map frames (...)`",
        ));
    };
    let [context, kernel, map, frames, Expression::Product { fields, .. }] = items.as_slice()
    else {
        return Err(mapping_diagnostic(
            source,
            *span,
            "kernel-mapping subject must be `context kernel map frames (...)`",
        ));
    };
    if !identifier_is(source, context, allocator_context)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, map, "map")
        || !identifier_is(source, frames, extent_name)
    {
        return Err(mapping_diagnostic(
            source,
            *span,
            "kernel map requires the live allocator context and its affine frame extent",
        ));
    }
    let mut rights = None;
    let mut execution = None;
    let mut memory_kind = None;
    for field in fields {
        let label = field.label.ok_or_else(|| {
            mapping_diagnostic(source, field.value.span(), "kernel-map parameters must be named")
        })?;
        let Expression::Identifier(value) = &field.value else {
            return Err(mapping_diagnostic(
                source,
                field.value.span(),
                "kernel-map policy values must be static systems identities",
            ));
        };
        let (destination, expected) = match source.slice(label) {
            "rights" => (&mut rights, "read-write"),
            "execution" => (&mut execution, "denied"),
            "memory-kind" => (&mut memory_kind, "normal"),
            other => {
                return Err(mapping_diagnostic(
                    source,
                    label,
                    format!("unknown kernel-map parameter `{other}`"),
                ));
            }
        };
        if source.slice(*value) != expected {
            return Err(mapping_diagnostic(
                source,
                *value,
                format!("the initial kernel-map `{}` policy must be `{expected}`", source.slice(label)),
            ));
        }
        if destination.replace(()).is_some() {
            return Err(mapping_diagnostic(
                source,
                label,
                format!("kernel-map parameter `{}` is duplicated", source.slice(label)),
            ));
        }
    }
    if rights.is_none() || execution.is_none() || memory_kind.is_none() || fields.len() != 3 {
        return Err(mapping_diagnostic(
            source,
            *span,
            "kernel map requires exactly `rights`, `execution`, and `memory-kind`",
        ));
    }
    Ok(CompilerKernelMappingRequest {
        rights: CompilerKernelMappingRights::ReadWrite,
        execution: CompilerKernelExecutionPolicy::Denied,
        memory_kind: CompilerKernelMemoryKind::Normal,
    })
}

fn parse_mapping_store(
    source: &SourceText,
    statement: &Statement,
    mapping_name: &str,
) -> Result<(u64, u8), Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(mapping_diagnostic(
            source,
            statement_span(statement),
            "mapping success action must begin with `mapping byte store (...)`",
        ));
    };
    let [mapping, byte, store, Expression::Product { fields, .. }] = items.as_slice() else {
        return Err(mapping_diagnostic(
            source,
            *span,
            "mapping success action must begin with `mapping byte store (...)`",
        ));
    };
    if !identifier_is(source, mapping, mapping_name)
        || !identifier_is(source, byte, "byte")
        || !identifier_is(source, store, "store")
        || fields.len() != 2
    {
        return Err(mapping_diagnostic(
            source,
            *span,
            "byte store requires the affine mapping and exactly `offset-bytes` and `value`",
        ));
    }
    let offset = mapping_named_natural(source, fields, "offset-bytes")?;
    let value = u8::try_from(mapping_named_natural(source, fields, "value")?).map_err(|_| {
        mapping_diagnostic(source, *span, "byte store value must be a `Nat` from 0 through 255")
    })?;
    Ok((offset, value))
}

fn parse_mapping_load(
    source: &SourceText,
    statement: &Statement,
    mapping_name: &str,
) -> Result<(String, u64), Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(mapping_diagnostic(
            source,
            statement_span(statement),
            "mapping byte load must bind its `Nat` result",
        ));
    };
    let [mapping, byte, load, Expression::Product { fields, .. }] = items.as_slice() else {
        return Err(mapping_diagnostic(
            source,
            *span,
            "mapping byte load must be `observed : Nat is mapping byte load (...)`",
        ));
    };
    if source.slice(*classifier) != "Nat"
        || !identifier_is(source, mapping, mapping_name)
        || !identifier_is(source, byte, "byte")
        || !identifier_is(source, load, "load")
        || fields.len() != 1
    {
        return Err(mapping_diagnostic(
            source,
            *span,
            "byte load requires the affine mapping and one static `offset-bytes` field",
        ));
    }
    Ok((
        source.slice(*name).to_owned(),
        mapping_named_natural(source, fields, "offset-bytes")?,
    ))
}

fn parse_mapping_comparison<'a>(
    source: &SourceText,
    statement: &Statement,
    true_rule: &'a Statement,
    false_rule: &'a Statement,
    loaded_name: &str,
) -> Result<(u8, &'a Expression, &'a Expression), Diagnostic> {
    let Statement::Expression(Expression::Application { items, .. }) = statement else {
        return Err(mapping_diagnostic(
            source,
            statement_span(statement),
            "loaded mapping byte requires an exhaustive equality decision",
        ));
    };
    let [loaded, Expression::Callable { kind: CallableKind::Equal, .. }, expected] =
        items.as_slice()
    else {
        return Err(mapping_diagnostic(
            source,
            statement_span(statement),
            "loaded mapping byte decision must use equality with a static byte",
        ));
    };
    if !identifier_is(source, loaded, loaded_name) {
        return Err(mapping_diagnostic(
            source,
            loaded.span(),
            "mapping byte comparison must use the immediately preceding load binding",
        ));
    }
    let expected = u8::try_from(mapping_natural(source, expected)?).map_err(|_| {
        mapping_diagnostic(source, expected.span(), "byte comparison value must be from 0 through 255")
    })?;
    Ok((
        expected,
        parse_mapping_boolean_rule(source, true_rule, true)?,
        parse_mapping_boolean_rule(source, false_rule, false)?,
    ))
}

fn parse_mapping_boolean_rule<'a>(
    source: &SourceText,
    statement: &'a Statement,
    expected: bool,
) -> Result<&'a Expression, Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(mapping_diagnostic(
            source,
            statement_span(statement),
            "mapping byte equality requires explicit `true` and `false` actions",
        ));
    };
    let [Expression::Boolean(value), then, action] = items.as_slice() else {
        return Err(mapping_diagnostic(source, *span, "mapping byte equality requires explicit `true` and `false` actions"));
    };
    if (source.slice(*value) == "true") != expected || !identifier_is(source, then, "then") {
        return Err(mapping_diagnostic(source, *span, "mapping byte equality actions must appear once in `true`, `false` order"));
    }
    Ok(action)
}

fn parse_kernel_unmap_scope<'a>(
    source: &SourceText,
    statements: &'a [Statement],
    allocator_context: &str,
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
            return Err(mapping_diagnostic(
                source,
                statements.first().map_or(Span::new(0, 0), statement_span),
                "kernel unmap must bind the returned affine frame extent",
            ));
        }
    };
    let [context, kernel, unmap, mapping] = items.as_slice() else {
        return Err(mapping_diagnostic(source, span, "unmap must be `frames is context kernel unmap mapping`"));
    };
    if !identifier_is(source, context, allocator_context)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, unmap, "unmap")
        || !identifier_is(source, mapping, mapping_name)
    {
        return Err(mapping_diagnostic(
            source,
            span,
            "kernel unmap requires the live allocator context and its affine mapping",
        ));
    }
    Ok((source.slice(name).to_owned(), tail))
}

fn mapping_named_natural(
    source: &SourceText,
    fields: &[ProductField],
    name: &str,
) -> Result<u64, Diagnostic> {
    let mut found = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(mapping_diagnostic(source, field.value.span(), "mapping byte access parameters must be named"));
        };
        if source.slice(label) == name {
            if found.replace(mapping_natural(source, &field.value)?).is_some() {
                return Err(mapping_diagnostic(source, label, format!("mapping byte access parameter `{name}` is duplicated")));
            }
        } else if !matches!(source.slice(label), "offset-bytes" | "value") {
            return Err(mapping_diagnostic(source, label, format!("unknown mapping byte access parameter `{}`", source.slice(label))));
        }
    }
    found.ok_or_else(|| mapping_diagnostic(source, fields.first().map_or(Span::new(0, 0), |field| field.value.span()), format!("mapping byte access requires `{name}`")))
}

fn mapping_natural(source: &SourceText, value: &Expression) -> Result<u64, Diagnostic> {
    let Expression::Integer(span) = value else {
        return Err(mapping_diagnostic(source, value.span(), "mapping byte values must be static natural-number literals"));
    };
    source.slice(*span).replace('_', "").parse::<u64>().map_err(|_| {
        mapping_diagnostic(source, *span, "mapping byte value is outside the supported natural-number range")
    })
}

fn mapping_action_block<'a>(
    source: &SourceText,
    action: &'a Expression,
    description: &str,
) -> Result<&'a [Statement], Diagnostic> {
    let Expression::Block { statements, .. } = action else {
        return Err(mapping_diagnostic(source, action.span(), format!("{description} requires a block")));
    };
    Ok(statements)
}

fn mapping_diagnostic(source: &SourceText, span: Span, message: impl Into<String>) -> Diagnostic {
    source_diagnostic(source, "E-SYSTEMS-MAPPING", span, message)
}
