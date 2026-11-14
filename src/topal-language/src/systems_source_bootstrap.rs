fn analyze_bootstrap_region_sequence(
    source: &SourceText,
    statements: &[Statement],
    context_name: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CheckedBootstrapRegionDecision, Diagnostic> {
    let [Statement::Expression(subject), ok_rule, error_rule] = statements else {
        return Err(storage_diagnostic(
            source,
            statements.first().map_or(Span::new(0, 0), statement_span),
            "bootstrap allocation requires a subject and exhaustive `Ok`/`Error` actions",
        ));
    };
    let request = parse_bootstrap_allocation(source, subject, context_name, storage)?;
    let (ok_binding, ok_action) = flattened_result_action(source, ok_rule, false)?;
    let (_, error_action) = flattened_result_action(source, error_rule, true)?;
    analyze_bootstrap_region_actions(
        source,
        request,
        &ok_binding,
        ok_action,
        error_action,
        context_name,
        storage,
    )
}

fn flattened_result_action<'a>(
    source: &SourceText,
    statement: &'a Statement,
    error: bool,
) -> Result<(String, &'a Expression), Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(storage_diagnostic(
            source,
            statement_span(statement),
            "bootstrap allocation requires exhaustive `Ok` and `Error` actions",
        ));
    };
    let [variant, binding, then, action] = items.as_slice() else {
        return Err(storage_diagnostic(
            source,
            *span,
            "nested bootstrap allocation actions must use explicit blocks",
        ));
    };
    let expected = if error { "Error" } else { "Ok" };
    let Expression::Identifier(binding_span) = binding else {
        return Err(storage_diagnostic(
            source,
            binding.span(),
            "bootstrap allocation result action requires one payload binding",
        ));
    };
    if !identifier_is(source, variant, expected) || !identifier_is(source, then, "then") {
        return Err(storage_diagnostic(
            source,
            *span,
            "bootstrap allocation actions must appear once in `Ok`, `Error` order",
        ));
    }
    if error {
        let Expression::Block { statements, .. } = action else {
            return Err(storage_diagnostic(
                source,
                action.span(),
                "nested bootstrap allocation error action requires a block",
            ));
        };
        let [Statement::Expression(disposition)] = statements.as_slice() else {
            return Err(storage_diagnostic(
                source,
                action.span(),
                "bootstrap allocation error block requires exactly one fatal disposition",
            ));
        };
        return Ok((source.slice(*binding_span).to_owned(), disposition));
    }
    Ok((source.slice(*binding_span).to_owned(), action))
}

fn analyze_bootstrap_region_actions(
    source: &SourceText,
    request: CompilerBootstrapStorageRequest,
    ok_binding: &str,
    ok_action: &Expression,
    error_action: &Expression,
    context_name: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CheckedBootstrapRegionDecision, Diagnostic> {
    let error_disposition = action_disposition(source, error_action, context_name)?;
    if !matches!(error_disposition, CompilerSystemsDisposition::Fatal { .. }) {
        return Err(storage_diagnostic(
            source,
            error_action.span(),
            "bootstrap allocation failure must consume the context through `fatal`",
        ));
    }
    let Expression::Block {
        statements,
        span: success_span,
    } = ok_action
    else {
        return Err(storage_diagnostic(
            source,
            ok_action.span(),
            "bootstrap allocation success requires a block using and releasing its region",
        ));
    };
    let (store, invocation, load, comparison, true_rule, false_rule) = match statements.as_slice() {
        [store, load, comparison, true_rule, false_rule] => {
            (store, None, load, comparison, true_rule, false_rule)
        }
        [store, invocation, load, comparison, true_rule, false_rule] => (
            store,
            Some(invocation),
            load,
            comparison,
            true_rule,
            false_rule,
        ),
        _ => {
            return Err(storage_diagnostic(
                source,
                *success_span,
                "region success requires byte store, optional library apply, byte load and an exhaustive equality decision",
            ));
        }
    };
    if let Some(invocation) = invocation {
        let Statement::Expression(Expression::Application { items, .. }) = invocation else {
            return Err(storage_diagnostic(
                source,
                statement_span(invocation),
                "region library apply requires a direct source invocation",
            ));
        };
        if request.byte_count != 64
            || items.len() != 3
            || !identifier_is(source, &items[0], ok_binding)
            || !identifier_is(source, &items[1], "library")
            || !identifier_is(source, &items[2], "apply")
        {
            return Err(storage_diagnostic(
                source,
                statement_span(invocation),
                "region library apply requires its live 64-byte plain owner",
            ));
        }
    }
    let (store_offset, value) = parse_region_store(source, store, ok_binding)?;
    let (loaded_name, load_offset) = parse_region_load(source, load, ok_binding)?;
    let comparison = parse_region_comparison(
        source,
        comparison,
        true_rule,
        false_rule,
        RegionComparisonContext {
            context_name,
            region_name: ok_binding,
            loaded_name: &loaded_name,
            region_request: request,
            storage,
        },
    )?;
    for offset in [store_offset, load_offset] {
        if offset >= request.byte_count {
            return Err(storage_diagnostic(
                source,
                *success_span,
                format!(
                    "bootstrap byte offset {offset} is outside the static region size {}",
                    request.byte_count
                ),
            ));
        }
    }
    let mut operations = vec![
        CompilerSystemsOperation::BootstrapAllocate { request },
        CompilerSystemsOperation::BootstrapStoreByte {
            offset_bytes: store_offset,
            value,
        },
    ];
    if invocation.is_some() {
        operations.push(CompilerSystemsOperation::ApplyBootstrapRegionLibrary);
    }
    operations.push(CompilerSystemsOperation::BootstrapLoadByteEquals {
        offset_bytes: load_offset,
        expected: comparison.expected,
        failure_message: comparison.failure_message,
    });
    operations.extend(comparison.success_operations);
    Ok(CheckedBootstrapRegionDecision {
        operations,
        disposition: comparison.success_disposition,
    })
}

fn parse_bootstrap_allocation(
    source: &SourceText,
    expression: &Expression,
    context_name: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CompilerBootstrapStorageRequest, Diagnostic> {
    let Expression::Application { items, span } = expression else {
        return Err(storage_diagnostic(
            source,
            expression.span(),
            "allocation decision subject must be `context bootstrap allocate (...)`",
        ));
    };
    let [
        context,
        bootstrap,
        allocate,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(storage_diagnostic(
            source,
            *span,
            "allocation decision subject must be `context bootstrap allocate (...)`",
        ));
    };
    if !identifier_is(source, context, context_name)
        || !identifier_is(source, bootstrap, "bootstrap")
        || !identifier_is(source, allocate, "allocate")
    {
        return Err(storage_diagnostic(
            source,
            *span,
            "allocation requires the live bootstrap context",
        ));
    }
    let mut byte_count = None;
    let mut alignment_bytes = None;
    let mut placement = None;
    for field in fields {
        let label = field.label.ok_or_else(|| {
            storage_diagnostic(
                source,
                field.value.span(),
                "bootstrap allocation parameters must be named",
            )
        })?;
        match source.slice(label) {
            "byte-count" => {
                insert_once(
                    source,
                    label,
                    &mut byte_count,
                    parse_storage_natural(source, &field.value)?,
                )?;
            }
            "alignment-bytes" => {
                insert_once(
                    source,
                    label,
                    &mut alignment_bytes,
                    parse_storage_natural(source, &field.value)?,
                )?;
            }
            "placement" => {
                let Expression::Identifier(value) = &field.value else {
                    return Err(storage_diagnostic(
                        source,
                        field.value.span(),
                        "bootstrap allocation placement must be a static systems identity",
                    ));
                };
                let parsed = match source.slice(*value) {
                    "bootstrap-reclaimable" => {
                        CompilerBootstrapStoragePlacement::BootstrapReclaimable
                    }
                    "page-aligned-table" => CompilerBootstrapStoragePlacement::PageAlignedTable,
                    other => {
                        return Err(storage_diagnostic(
                            source,
                            *value,
                            format!("unknown bootstrap allocation placement `{other}`"),
                        ));
                    }
                };
                insert_once(source, label, &mut placement, parsed)?;
            }
            other => {
                return Err(storage_diagnostic(
                    source,
                    label,
                    format!("unknown bootstrap allocation parameter `{other}`"),
                ));
            }
        }
    }
    let request = CompilerBootstrapStorageRequest {
        byte_count: byte_count.ok_or_else(|| missing_allocation_fields(source, *span))?,
        alignment_bytes: alignment_bytes.ok_or_else(|| missing_allocation_fields(source, *span))?,
        placement: placement.ok_or_else(|| missing_allocation_fields(source, *span))?,
    };
    validate_static_allocation(source, *span, storage, request)?;
    Ok(request)
}

fn validate_static_allocation(
    source: &SourceText,
    span: Span,
    storage: &CompilerBootstrapStorageDescriptor,
    request: CompilerBootstrapStorageRequest,
) -> Result<(), Diagnostic> {
    let mut model = CompilerBootstrapStorageState::new(storage.clone(), "source-check")
        .map_err(|error| storage_diagnostic(source, span, error.message))?;
    model.allocate(request).map_err(|code| {
        storage_diagnostic(
            source,
            span,
            format!("static bootstrap allocation cannot succeed: {code:?}"),
        )
    })?;
    Ok(())
}

fn insert_once<T>(
    source: &SourceText,
    label: Span,
    destination: &mut Option<T>,
    value: T,
) -> Result<(), Diagnostic> {
    if destination.replace(value).is_some() {
        return Err(storage_diagnostic(
            source,
            label,
            format!(
                "bootstrap allocation parameter `{}` is duplicated",
                source.slice(label)
            ),
        ));
    }
    Ok(())
}

fn missing_allocation_fields(source: &SourceText, span: Span) -> Diagnostic {
    storage_diagnostic(
        source,
        span,
        "bootstrap allocation requires exactly `byte-count`, `alignment-bytes`, and `placement`",
    )
}

fn result_actions<'a>(
    source: &SourceText,
    rules: &'a [DecisionRule],
    span: Span,
) -> Result<(String, &'a Expression, &'a Expression), Diagnostic> {
    let mut ok = None;
    let mut error = None;
    for rule in rules {
        match rule.matcher {
            DecisionMatcher::Result {
                error: false,
                binding,
                ..
            } if ok.is_none() => ok = Some((source.slice(binding).to_owned(), &rule.action)),
            DecisionMatcher::Result { error: true, .. } if error.is_none() => {
                error = Some(&rule.action);
            }
            _ => {
                return Err(storage_diagnostic(
                    source,
                    rule.span,
                    "bootstrap allocation requires exactly one `Ok region` and one `Error problem` action",
                ));
            }
        }
    }
    let (Some((binding, ok)), Some(error)) = (ok, error) else {
        return Err(storage_diagnostic(
            source,
            span,
            "bootstrap allocation requires exhaustive `Ok` and `Error` actions",
        ));
    };
    Ok((binding, ok, error))
}

fn raw_result_action<'a>(
    source: &SourceText,
    statement: &'a Statement,
    expected: &str,
) -> Result<(String, &'a Expression), Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "nested context allocation requires explicit `Ok` and `Error` actions",
        ));
    };
    let [constructor, Expression::Identifier(binding), then, action] = items.as_slice() else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "nested context allocation action must bind its result before `then`",
        ));
    };
    if !identifier_is(source, constructor, expected) || !identifier_is(source, then, "then") {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            format!("nested context allocation requires the `{expected}` action in order"),
        ));
    }
    Ok((source.slice(*binding).to_owned(), action))
}

fn parse_region_store(
    source: &SourceText,
    statement: &Statement,
    region_name: &str,
) -> Result<(u64, u8), Diagnostic> {
    let Statement::Expression(Expression::Application { items, span }) = statement else {
        return Err(storage_diagnostic(
            source,
            statement_span(statement),
            "region success action must begin with `region byte store (...)`",
        ));
    };
    let [region, byte, store, Expression::Product { fields, .. }] = items.as_slice() else {
        return Err(storage_diagnostic(
            source,
            *span,
            "region success action must begin with `region byte store (...)`",
        ));
    };
    if !identifier_is(source, region, region_name)
        || !identifier_is(source, byte, "byte")
        || !identifier_is(source, store, "store")
    {
        return Err(storage_diagnostic(
            source,
            *span,
            "byte store requires the affine region bound by the `Ok` action",
        ));
    }
    let offset = named_static_natural(source, fields, "offset-bytes")?;
    let value = named_static_natural(source, fields, "value")?;
    if fields.len() != 2 {
        return Err(storage_diagnostic(
            source,
            *span,
            "byte store requires exactly `offset-bytes` and `value`",
        ));
    }
    let value = u8::try_from(value).map_err(|_| {
        storage_diagnostic(
            source,
            *span,
            "byte store value must be a `Nat` from 0 through 255",
        )
    })?;
    Ok((offset, value))
}

fn parse_region_load(
    source: &SourceText,
    statement: &Statement,
    region_name: &str,
) -> Result<(String, u64), Diagnostic> {
    let Statement::Binding {
        name,
        classifier: Some(classifier),
        value: Expression::Application { items, span },
    } = statement
    else {
        return Err(storage_diagnostic(
            source,
            statement_span(statement),
            "byte load must bind its `Nat` result",
        ));
    };
    let [region, byte, load, Expression::Product { fields, .. }] = items.as_slice() else {
        return Err(storage_diagnostic(
            source,
            *span,
            "byte load must be `observed : Nat is region byte load (...)`",
        ));
    };
    if source.slice(*classifier) != "Nat"
        || !identifier_is(source, region, region_name)
        || !identifier_is(source, byte, "byte")
        || !identifier_is(source, load, "load")
        || fields.len() != 1
    {
        return Err(storage_diagnostic(
            source,
            *span,
            "byte load requires the affine region and one static `offset-bytes` field",
        ));
    }
    Ok((
        source.slice(*name).to_owned(),
        named_static_natural(source, fields, "offset-bytes")?,
    ))
}

fn named_static_natural(
    source: &SourceText,
    fields: &[ProductField],
    name: &str,
) -> Result<u64, Diagnostic> {
    let mut found = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(storage_diagnostic(
                source,
                field.value.span(),
                "byte access parameters must be named",
            ));
        };
        if source.slice(label) == name {
            if found
                .replace(parse_storage_natural(source, &field.value)?)
                .is_some()
            {
                return Err(storage_diagnostic(
                    source,
                    label,
                    format!("byte access parameter `{name}` is duplicated"),
                ));
            }
        } else if !matches!(source.slice(label), "offset-bytes" | "value") {
            return Err(storage_diagnostic(
                source,
                label,
                format!("unknown byte access parameter `{}`", source.slice(label)),
            ));
        }
    }
    found.ok_or_else(|| {
        storage_diagnostic(
            source,
            fields
                .first()
                .map_or(Span::new(0, 0), |field| field.value.span()),
            format!("byte access requires `{name}`"),
        )
    })
}

struct CheckedRegionComparison {
    expected: u8,
    success_operations: Vec<CompilerSystemsOperation>,
    failure_message: String,
    success_disposition: CompilerSystemsDisposition,
}

#[derive(Clone, Copy)]
struct RegionComparisonContext<'a> {
    context_name: &'a str,
    region_name: &'a str,
    loaded_name: &'a str,
    region_request: CompilerBootstrapStorageRequest,
    storage: &'a CompilerBootstrapStorageDescriptor,
}

fn parse_region_comparison(
    source: &SourceText,
    statement: &Statement,
    true_rule: &Statement,
    false_rule: &Statement,
    context: RegionComparisonContext<'_>,
) -> Result<CheckedRegionComparison, Diagnostic> {
    let Statement::Expression(Expression::Application { items, .. }) = statement else {
        return Err(storage_diagnostic(
            source,
            statement_span(statement),
            "loaded byte requires an exhaustive equality decision",
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
        return Err(storage_diagnostic(
            source,
            statement_span(statement),
            "loaded byte decision subject must use equality with a static byte",
        ));
    };
    if !identifier_is(source, loaded, context.loaded_name) {
        return Err(storage_diagnostic(
            source,
            loaded.span(),
            "byte comparison must use the immediately preceding load binding",
        ));
    }
    let expected = u8::try_from(parse_storage_natural(source, expected)?).map_err(|_| {
        storage_diagnostic(
            source,
            expected.span(),
            "byte comparison value must be from 0 through 255",
        )
    })?;
    let true_action = parse_boolean_rule(source, true_rule, true)?;
    let false_action = parse_boolean_rule(source, false_rule, false)?;
    let true_statements = action_block(source, true_action, "successful byte comparison")?;
    let atomic = parse_atomic_word_sequence(
        source,
        true_statements,
        context.context_name,
        context.region_name,
        context.region_request,
        context.storage,
    )?;

    let false_statements = action_block(source, false_action, "failed byte comparison")?;
    let [release, disposition] = false_statements else {
        return Err(storage_diagnostic(
            source,
            false_action.span(),
            "failed byte comparison requires region release and fatal disposition",
        ));
    };
    parse_region_release(source, release, context.context_name, context.region_name)?;
    let failure = analyze_disposition(
        source,
        disposition,
        CompilerSystemsContextKind::Bootstrap,
        context.context_name,
    )?;
    let CompilerSystemsDisposition::Fatal {
        message: failure_message,
    } = failure
    else {
        return Err(storage_diagnostic(
            source,
            statement_span(disposition),
            "failed byte comparison must enter the fatal disposition",
        ));
    };
    Ok(CheckedRegionComparison {
        expected,
        success_operations: atomic.operations,
        failure_message,
        success_disposition: atomic.disposition,
    })
}

struct CheckedAtomicSequence {
    operations: Vec<CompilerSystemsOperation>,
    disposition: CompilerSystemsDisposition,
}

fn parse_atomic_word_sequence(
    source: &SourceText,
    statements: &[Statement],
    context_name: &str,
    region_name: &str,
    region_request: CompilerBootstrapStorageRequest,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CheckedAtomicSequence, Diagnostic> {
    let [create] = statements else {
        return Err(atomic_diagnostic(
            source,
            statements.first().map_or(Span::new(0, 0), statement_span),
            "atomic word lifecycle requires create and exhaustive `Exchanged`/`Observed` compare/exchange actions",
        ));
    };
    let (atomic_name, request, continuation) =
        parse_atomic_word_create(source, create, region_name)?;
    if request != CompilerAtomicWordRequest::initial()
        || request.offset_bytes % 8 != 0
        || request
            .offset_bytes
            .checked_add(8)
            .is_none_or(|end| end > region_request.byte_count)
    {
        return Err(atomic_diagnostic(
            source,
            statement_span(create),
            "the initial atomic word requires aligned offset 8, initial value 41, cpu-shared domain, and eight in-bounds bytes",
        ));
    }
    let [
        Statement::Expression(Expression::DecisionTable {
            subject: compare_exchange,
            rules,
            span,
        }),
    ] = continuation
    else {
        return Err(atomic_diagnostic(
            source,
            statement_span(create),
            "atomic creation must continue directly into one exhaustive compare/exchange decision",
        ));
    };
    let (expected, desired, success_order, failure_order) =
        parse_atomic_compare_exchange(source, compare_exchange, &atomic_name)?;
    if (expected, desired, success_order, failure_order)
        != (
            41,
            42,
            CompilerAtomicOrder::AcquireRelease,
            CompilerAtomicOrder::Acquire,
        )
    {
        return Err(atomic_diagnostic(
            source,
            compare_exchange.span(),
            "the initial compare/exchange requires expected 41, desired 42, acquire-release success, and acquire failure",
        ));
    }
    let (exchanged_action, observed_action) = atomic_result_actions(source, rules, *span)?;
    let exchanged = parse_atomic_exchanged_action(
        source,
        exchanged_action,
        context_name,
        region_name,
        &atomic_name,
        storage,
    )?;
    let compare_failure_message = parse_atomic_observed_action(
        source,
        observed_action,
        context_name,
        region_name,
        &atomic_name,
    )?;

    let mut operations = vec![
        CompilerSystemsOperation::AtomicWordCreate { request },
        CompilerSystemsOperation::AtomicCompareExchangeEquals {
            expected,
            desired,
            success_order,
            failure_order,
            failure_message: compare_failure_message,
        },
        CompilerSystemsOperation::AtomicLoadEquals {
            order: exchanged.load_order,
            expected: exchanged.expected,
            failure_message: exchanged.load_failure_message,
        },
        CompilerSystemsOperation::AtomicWordEnd,
    ];
    operations.extend(exchanged.success_operations);
    Ok(CheckedAtomicSequence {
        operations,
        disposition: exchanged.success_disposition,
    })
}

fn parse_atomic_observed_action(
    source: &SourceText,
    action: &Expression,
    context_name: &str,
    region_name: &str,
    atomic_name: &str,
) -> Result<String, Diagnostic> {
    let statements = action_block(source, action, "observed compare/exchange")?;
    let [end] = statements else {
        return Err(atomic_diagnostic(
            source,
            action.span(),
            "observed compare/exchange requires atomic end, region release, and fatal disposition",
        ));
    };
    let continuation = parse_atomic_end(source, end, atomic_name, region_name)?;
    let [release, disposition] = continuation else {
        return Err(atomic_diagnostic(
            source,
            action.span(),
            "observed compare/exchange requires region release and fatal disposition after atomic end",
        ));
    };
    parse_region_release(source, release, context_name, region_name)?;
    let CompilerSystemsDisposition::Fatal { message } = analyze_disposition(
        source,
        disposition,
        CompilerSystemsContextKind::Bootstrap,
        context_name,
    )?
    else {
        return Err(atomic_diagnostic(
            source,
            statement_span(disposition),
            "observed compare/exchange must enter the fatal disposition",
        ));
    };
    Ok(message)
}

fn parse_atomic_word_create<'a>(
    source: &SourceText,
    statement: &'a Statement,
    region_name: &str,
) -> Result<(String, CompilerAtomicWordRequest, &'a [Statement]), Diagnostic> {
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
            "atomic creation must bind `atomic is region atomic word create (...)`",
        ));
    };
    let [
        region,
        atomic,
        word,
        create,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(atomic_diagnostic(
            source,
            *span,
            "atomic creation must bind `atomic is region atomic word create (...)`",
        ));
    };
    if !identifier_is(source, region, region_name)
        || !identifier_is(source, atomic, "atomic")
        || !identifier_is(source, word, "word")
        || !identifier_is(source, create, "create")
    {
        return Err(atomic_diagnostic(
            source,
            *span,
            "atomic word creation must consume the live ordinary region",
        ));
    }
    let offset_bytes = atomic_natural_field(source, fields, "offset-bytes")?;
    let initial_value = atomic_natural_field(source, fields, "initial-value")?;
    let domain = atomic_identity_field(source, fields, "domain", "cpu-shared")?;
    if fields.len() != 3 {
        return Err(atomic_diagnostic(
            source,
            *span,
            "atomic word creation requires exactly `offset-bytes`, `initial-value`, and `domain`",
        ));
    }
    Ok((
        source.slice(*name).to_owned(),
        CompilerAtomicWordRequest {
            offset_bytes,
            initial_value,
            domain: match domain.as_str() {
                "cpu-shared" => CompilerAtomicDomain::CpuShared,
                _ => unreachable!("identity helper checks the sealed domain"),
            },
        },
        declarations,
    ))
}

fn parse_atomic_compare_exchange(
    source: &SourceText,
    expression: &Expression,
    atomic_name: &str,
) -> Result<(u64, u64, CompilerAtomicOrder, CompilerAtomicOrder), Diagnostic> {
    let Expression::Application { items, span } = expression else {
        return Err(atomic_diagnostic(
            source,
            expression.span(),
            "compare/exchange must use the live atomic location",
        ));
    };
    let [
        atomic,
        compare,
        exchange,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(atomic_diagnostic(
            source,
            *span,
            "compare/exchange must be `atomic compare exchange (...)`",
        ));
    };
    if !identifier_is(source, atomic, atomic_name)
        || !identifier_is(source, compare, "compare")
        || !identifier_is(source, exchange, "exchange")
        || fields.len() != 4
    {
        return Err(atomic_diagnostic(
            source,
            *span,
            "compare/exchange requires the live atomic location and four named parameters",
        ));
    }
    Ok((
        atomic_natural_field(source, fields, "expected")?,
        atomic_natural_field(source, fields, "desired")?,
        atomic_order_field(source, fields, "success-order")?,
        atomic_order_field(source, fields, "failure-order")?,
    ))
}

fn atomic_result_actions<'a>(
    source: &SourceText,
    rules: &'a [DecisionRule],
    span: Span,
) -> Result<(&'a Expression, &'a Expression), Diagnostic> {
    let [exchanged, observed] = rules else {
        return Err(atomic_diagnostic(
            source,
            span,
            "compare/exchange requires exactly one `Exchanged` and one `Observed` action",
        ));
    };
    match (&exchanged.matcher, &observed.matcher) {
        (
            DecisionMatcher::Union {
                alternative: exchanged_name,
                ..
            },
            DecisionMatcher::Union {
                alternative: observed_name,
                ..
            },
        ) if source.slice(*exchanged_name) == "Exchanged"
            && source.slice(*observed_name) == "Observed" =>
        {
            Ok((&exchanged.action, &observed.action))
        }
        _ => Err(atomic_diagnostic(
            source,
            span,
            "compare/exchange actions must appear once in `Exchanged`, `Observed` order and bind the observed value",
        )),
    }
}

struct CheckedAtomicExchanged {
    load_order: CompilerAtomicOrder,
    expected: u64,
    load_failure_message: String,
    success_operations: Vec<CompilerSystemsOperation>,
    success_disposition: CompilerSystemsDisposition,
}

fn parse_atomic_exchanged_action(
    source: &SourceText,
    action: &Expression,
    context_name: &str,
    region_name: &str,
    atomic_name: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CheckedAtomicExchanged, Diagnostic> {
    let statements = action_block(source, action, "exchanged compare/exchange")?;
    let [load, comparison, true_rule, false_rule] = statements else {
        return Err(atomic_diagnostic(
            source,
            action.span(),
            "exchanged compare/exchange requires acquire load and exhaustive equality actions",
        ));
    };
    let (loaded_name, load_order) = parse_atomic_load(source, load, atomic_name)?;
    let expected = parse_atomic_equality(source, comparison, &loaded_name)?;
    let true_action = parse_boolean_rule(source, true_rule, true)?;
    let false_action = parse_boolean_rule(source, false_rule, false)?;
    let (success_operations, success_disposition) = parse_atomic_exchanged_success(
        source,
        true_action,
        context_name,
        region_name,
        atomic_name,
        storage,
    )?;
    let load_failure_message = parse_atomic_exchanged_failure(
        source,
        false_action,
        context_name,
        region_name,
        atomic_name,
    )?;
    Ok(CheckedAtomicExchanged {
        load_order,
        expected,
        load_failure_message,
        success_operations,
        success_disposition,
    })
}

#[allow(clippy::too_many_lines)] // The sealed atomic-to-interrupt-to-deadline success path is audited together.
fn parse_atomic_exchanged_success(
    source: &SourceText,
    true_action: &Expression,
    context_name: &str,
    region_name: &str,
    atomic_name: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<(Vec<CompilerSystemsOperation>, CompilerSystemsDisposition), Diagnostic> {
    let success = action_block(source, true_action, "successful atomic load")?;
    let [end] = success else {
        return Err(atomic_diagnostic(
            source,
            true_action.span(),
            "successful atomic load requires atomic end, atomic and memory markers, region release, and final disposition",
        ));
    };
    let success_continuation = parse_atomic_end(source, end, atomic_name, region_name)?;
    let [
        atomic_console,
        memory_console,
        release,
        send,
        wait,
        interrupt_console,
        first_clock,
        second_clock,
        time_console,
        deadline,
        context_decision,
    ] = success_continuation
    else {
        return Err(atomic_diagnostic(
            source,
            true_action.span(),
            "successful atomic load requires atomic and memory markers, region release, local-notification completion, two monotonic-clock observations, one deadline-preemptible context lifecycle, and final disposition after atomic end",
        ));
    };
    let atomic_marker = analyze_operation(
        source,
        atomic_console,
        CompilerSystemsContextKind::Bootstrap,
        context_name,
    )?;
    let memory_marker = analyze_operation(
        source,
        memory_console,
        CompilerSystemsContextKind::Bootstrap,
        context_name,
    )?;
    if !matches!(atomic_marker, CompilerSystemsOperation::ConsoleWrite { .. })
        || !matches!(memory_marker, CompilerSystemsOperation::ConsoleWrite { .. })
    {
        return Err(atomic_diagnostic(
            source,
            true_action.span(),
            "successful atomic load markers must be console writes",
        ));
    }
    parse_region_release(source, release, context_name, region_name)?;
    let (pending_name, send_operation) = parse_local_notification_send(source, send, context_name)?;
    let (resumed_name, wait_operation) =
        parse_local_notification_wait(source, wait, &pending_name)?;
    let interrupt_marker = analyze_operation(
        source,
        interrupt_console,
        CompilerSystemsContextKind::Bootstrap,
        &resumed_name,
    )?;
    if interrupt_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_INTERRUPT_OK".into(),
        })
    {
        return Err(local_interrupt_diagnostic(
            source,
            statement_span(interrupt_console),
            "local-notification wait must be followed by the exact interrupt success marker",
        ));
    }
    let (first_name, first_clock_operation) =
        parse_monotonic_clock_now(source, first_clock, &resumed_name)?;
    let (second_name, second_clock_operation) =
        parse_monotonic_clock_now(source, second_clock, &resumed_name)?;
    if first_name == second_name {
        return Err(monotonic_clock_diagnostic(
            source,
            statement_span(second_clock),
            "monotonic-clock observations require distinct instant bindings",
        ));
    }
    let time_marker = analyze_operation(
        source,
        time_console,
        CompilerSystemsContextKind::Bootstrap,
        &resumed_name,
    )?;
    if time_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_TIME_OK".into(),
        })
    {
        return Err(monotonic_clock_diagnostic(
            source,
            statement_span(time_console),
            "the two clock observations must be followed by the exact time success marker",
        ));
    }
    let (deadline_name, deadline_operation) = parse_deadline_after(source, deadline, &second_name)?;
    let checked_context = analyze_kernel_context_decision(
        source,
        context_decision,
        &resumed_name,
        &deadline_name,
        storage,
    )?;
    let mut operations = vec![
        atomic_marker,
        memory_marker,
        CompilerSystemsOperation::BootstrapRelease,
        send_operation,
        wait_operation,
        interrupt_marker,
        first_clock_operation,
        second_clock_operation,
        time_marker,
        deadline_operation,
    ];
    operations.extend(checked_context.operations);
    Ok((operations, checked_context.disposition))
}

fn analyze_kernel_context_decision(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    deadline_name: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CheckedBootstrapRegionDecision, Diagnostic> {
    let Statement::Expression(Expression::DecisionTable {
        subject,
        rules,
        span,
    }) = statement
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(statement),
            "kernel-context stack allocation requires a subject and exhaustive `Ok`/`Error` actions",
        ));
    };
    let request = parse_bootstrap_allocation(source, subject, context_name, storage)?;
    if request
        != (CompilerBootstrapStorageRequest {
            byte_count: 16_384,
            alignment_bytes: 16,
            placement: CompilerBootstrapStoragePlacement::BootstrapReclaimable,
        })
    {
        return Err(kernel_context_diagnostic(
            source,
            subject.span(),
            "the initial kernel context requires one 16 KiB, 16-byte-aligned bootstrap-reclaimable stack",
        ));
    }
    let (stack_name, ok_action, error_action) = result_actions(source, rules, *span)?;
    let CompilerSystemsDisposition::Fatal { .. } =
        action_disposition(source, error_action, context_name)?
    else {
        return Err(kernel_context_diagnostic(
            source,
            error_action.span(),
            "kernel-context stack allocation failure must enter the fatal disposition",
        ));
    };
    let statements = action_block(source, ok_action, "kernel-context stack allocation")?;
    let [
        create_completion,
        create_cooperative,
        completion_subject_statement,
        completion_ok_statement,
        completion_error_statement,
    ] = statements
    else {
        return Err(kernel_context_diagnostic(
            source,
            ok_action.span(),
            "context success requires completion creation, the preemptible worker, and the completion-worker stack allocation",
        ));
    };
    let completion_name = parse_kernel_completion_create(source, create_completion, context_name)?;
    let cooperative_name = parse_kernel_context_create(
        source,
        create_cooperative,
        context_name,
        &stack_name,
        "kernel-thread",
    )?;
    let Statement::Expression(completion_subject) = completion_subject_statement else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(completion_subject_statement),
            "preemptible context creation must be followed by the completion-worker stack allocation",
        ));
    };
    let completion_request =
        parse_bootstrap_allocation(source, completion_subject, context_name, storage)?;
    if completion_request != request {
        return Err(kernel_context_diagnostic(
            source,
            completion_subject.span(),
            "all worker contexts require disjoint 16 KiB bootstrap-reclaimable stacks",
        ));
    }
    let (completion_stack_name, completion_ok) =
        raw_result_action(source, completion_ok_statement, "Ok")?;
    let (_, completion_error) = raw_result_action(source, completion_error_statement, "Error")?;
    let CompilerSystemsDisposition::Fatal { .. } =
        action_disposition(source, completion_error, context_name)?
    else {
        return Err(kernel_context_diagnostic(
            source,
            completion_error.span(),
            "completion-worker stack allocation failure must enter the fatal disposition",
        ));
    };
    let completion_statements = action_block(
        source,
        completion_ok,
        "completion kernel-context stack allocation",
    )?;
    let [
        create_completion_worker,
        terminal_subject_statement,
        terminal_ok_statement,
        terminal_error_statement,
    ] = completion_statements
    else {
        return Err(kernel_context_diagnostic(
            source,
            completion_ok.span(),
            "completion context success requires its create followed by the terminal stack allocation",
        ));
    };
    let completion_worker_name = parse_kernel_context_create(
        source,
        create_completion_worker,
        context_name,
        &completion_stack_name,
        "completion-thread",
    )?;
    let Statement::Expression(terminal_subject) = terminal_subject_statement else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(terminal_subject_statement),
            "completion context creation must be followed by the terminal stack allocation",
        ));
    };
    let terminal_request =
        parse_bootstrap_allocation(source, terminal_subject, context_name, storage)?;
    if terminal_request != request {
        return Err(kernel_context_diagnostic(
            source,
            terminal_subject.span(),
            "all worker contexts require disjoint 16 KiB bootstrap-reclaimable stacks",
        ));
    }
    let (terminal_stack_name, terminal_ok) =
        raw_result_action(source, terminal_ok_statement, "Ok")?;
    let (_, terminal_error) = raw_result_action(source, terminal_error_statement, "Error")?;
    let CompilerSystemsDisposition::Fatal { .. } =
        action_disposition(source, terminal_error, context_name)?
    else {
        return Err(kernel_context_diagnostic(
            source,
            terminal_error.span(),
            "terminal stack allocation failure must enter the fatal disposition",
        ));
    };
    let terminal_statements = action_block(source, terminal_ok, "terminal stack allocation")?;
    let [
        create_terminal,
        create_queue,
        enqueue_cooperative,
        enqueue_completion,
        enqueue_terminal,
        dequeue_cooperative,
        preemptible_transfer,
        take_preempted,
        preemption_marker_statement,
        enqueue_preempted,
        dequeue_completion,
        transfer_completion_until_signal,
        dequeue_terminal,
        transfer_terminal,
        terminal_marker_statement,
        reclaim_terminal,
    ] = terminal_statements
    else {
        return Err(kernel_context_diagnostic(
            source,
            terminal_ok.span(),
            "the FIFO dispatcher requires three workers, deadline preemption, completion blocking, exact source order, retirement, and reclamation",
        ));
    };
    let terminal_name = parse_kernel_context_create(
        source,
        create_terminal,
        context_name,
        &terminal_stack_name,
        "terminal-thread",
    )?;
    let queue_name = parse_runnable_queue_create(source, create_queue, context_name)?;
    parse_runnable_queue_enqueue(source, enqueue_cooperative, &queue_name, &cooperative_name)?;
    parse_runnable_queue_enqueue(
        source,
        enqueue_completion,
        &queue_name,
        &completion_worker_name,
    )?;
    parse_runnable_queue_enqueue(source, enqueue_terminal, &queue_name, &terminal_name)?;
    let selected_cooperative =
        parse_runnable_queue_dequeue(source, dequeue_cooperative, &queue_name)?;
    let preempted_name = parse_kernel_context_transfer_until_deadline(
        source,
        preemptible_transfer,
        context_name,
        &selected_cooperative,
        deadline_name,
    )?;
    let preempted_worker =
        parse_take_preempted_kernel_context(source, take_preempted, &preempted_name)?;
    let preemption_marker = analyze_operation(
        source,
        preemption_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        context_name,
    )?;
    if preemption_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_DEADLINE_PREEMPTED".into(),
        })
    {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(preemption_marker_statement),
            "the deadline-preemptible transfer requires its exact success marker",
        ));
    }
    parse_runnable_queue_enqueue(source, enqueue_preempted, &queue_name, &preempted_worker)?;
    let selected_completion =
        parse_runnable_queue_dequeue(source, dequeue_completion, &queue_name)?;
    let blocked_completion = parse_kernel_context_transfer_until_completion(
        source,
        transfer_completion_until_signal,
        context_name,
        &selected_completion,
        &completion_name,
    )?;
    let selected_terminal = parse_runnable_queue_dequeue(source, dequeue_terminal, &queue_name)?;
    let terminal_completed = parse_kernel_context_transfer(
        source,
        transfer_terminal,
        context_name,
        &selected_terminal,
        "CompletedKernelContextTransfer InitialProcessor",
    )?;
    let terminal_marker = analyze_operation(
        source,
        terminal_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        context_name,
    )?;
    if terminal_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_CONTEXT_TERMINAL_RETIRED".into(),
        })
    {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(terminal_marker_statement),
            "terminal retirement requires its exact marker",
        ));
    }
    let (after_terminal, after_terminal_statements) =
        parse_kernel_context_reclaim(source, reclaim_terminal, &terminal_completed)?;
    let [
        signal_completion,
        enqueue_completion,
        dequeue_cooperative_final,
        transfer_cooperative,
        cooperative_marker_statement,
        reclaim_cooperative,
    ] = after_terminal_statements
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(reclaim_terminal),
            "terminal reclamation must signal and enqueue the blocked worker before restoring the preempted worker",
        ));
    };
    let awakened_completion = parse_signal_blocked_kernel_completion(
        source,
        signal_completion,
        &blocked_completion,
        &completion_name,
    )?;
    parse_runnable_queue_enqueue(
        source,
        enqueue_completion,
        &queue_name,
        &awakened_completion,
    )?;
    let selected_cooperative_final =
        parse_runnable_queue_dequeue(source, dequeue_cooperative_final, &queue_name)?;
    let cooperative_completed = parse_kernel_context_transfer(
        source,
        transfer_cooperative,
        &after_terminal,
        &selected_cooperative_final,
        "CompletedKernelContextTransfer InitialProcessor",
    )?;
    let cooperative_marker = analyze_operation(
        source,
        cooperative_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &after_terminal,
    )?;
    if cooperative_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_CONTEXT_PREEMPTIBLE_RETIRED".into(),
        })
    {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(cooperative_marker_statement),
            "preemptible-worker retirement requires its exact marker",
        ));
    }
    let (resumed_name, continuation) =
        parse_kernel_context_reclaim(source, reclaim_cooperative, &cooperative_completed)?;
    let [
        dequeue_completion_final,
        transfer_completion,
        completion_marker_statement,
        reclaim_completion,
    ] = continuation
    else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(reclaim_cooperative),
            "preemptible reclamation must resume and reclaim the awakened completion worker",
        ));
    };
    let selected_completion_final =
        parse_runnable_queue_dequeue(source, dequeue_completion_final, &queue_name)?;
    let completion_completed = parse_kernel_context_transfer(
        source,
        transfer_completion,
        &resumed_name,
        &selected_completion_final,
        "CompletedKernelContextTransfer InitialProcessor",
    )?;
    let completion_marker = analyze_operation(
        source,
        completion_marker_statement,
        CompilerSystemsContextKind::Bootstrap,
        &resumed_name,
    )?;
    if completion_marker
        != (CompilerSystemsOperation::ConsoleWrite {
            text: "TOPAL_KERNEL_CONTEXT_COMPLETION_BLOCKING_RETIRED".into(),
        })
    {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(completion_marker_statement),
            "completion-worker retirement requires its exact marker",
        ));
    }
    let (completion_resumed, completion_continuation) =
        parse_kernel_context_reclaim(source, reclaim_completion, &completion_completed)?;
    let [consume_completion, consume_queue, user_allocation @ ..] = completion_continuation else {
        return Err(kernel_context_diagnostic(
            source,
            statement_span(reclaim_completion),
            "completion-worker reclamation must consume completion and empty queue, complete one user syscall round trip, then reach disposition",
        ));
    };
    parse_kernel_completion_consume(
        source,
        consume_completion,
        &completion_resumed,
        &completion_name,
    )?;
    parse_runnable_queue_consume(source, consume_queue, &completion_resumed, &queue_name)?;
    let operations = vec![
        CompilerSystemsOperation::BootstrapAllocate { request },
        CompilerSystemsOperation::CreateKernelCompletion,
        CompilerSystemsOperation::CreateKernelContext,
        CompilerSystemsOperation::BootstrapAllocate {
            request: completion_request,
        },
        CompilerSystemsOperation::CreateKernelContext,
        CompilerSystemsOperation::BootstrapAllocate {
            request: terminal_request,
        },
        CompilerSystemsOperation::CreateKernelContext,
        CompilerSystemsOperation::CreateRunnableQueue { capacity: 3 },
        CompilerSystemsOperation::EnqueueRunnableContext,
        CompilerSystemsOperation::EnqueueRunnableContext,
        CompilerSystemsOperation::EnqueueRunnableContext,
        CompilerSystemsOperation::DequeueRunnableContext,
        CompilerSystemsOperation::TransferKernelContextUntilDeadline,
        CompilerSystemsOperation::TakePreemptedKernelContext,
        preemption_marker,
        CompilerSystemsOperation::EnqueueRunnableContext,
        CompilerSystemsOperation::DequeueRunnableContext,
        CompilerSystemsOperation::TransferKernelContextUntilCompletion,
        CompilerSystemsOperation::DequeueRunnableContext,
        CompilerSystemsOperation::TransferKernelContext,
        terminal_marker,
        CompilerSystemsOperation::ReclaimKernelContext,
        CompilerSystemsOperation::SignalBlockedKernelCompletion,
        CompilerSystemsOperation::EnqueueRunnableContext,
        CompilerSystemsOperation::DequeueRunnableContext,
        CompilerSystemsOperation::TransferKernelContext,
        cooperative_marker,
        CompilerSystemsOperation::ReclaimKernelContext,
        CompilerSystemsOperation::DequeueRunnableContext,
        CompilerSystemsOperation::TransferKernelContext,
        completion_marker,
        CompilerSystemsOperation::ReclaimKernelContext,
        CompilerSystemsOperation::ConsumeKernelCompletion,
        CompilerSystemsOperation::ConsumeEmptyRunnableQueue,
    ];
    analyze_kernel_user_continuation(source, user_allocation, completion_resumed, operations)
}

include!("systems_kernel_user_continuation.rs");

fn parse_kernel_context_create(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    stack_name: &str,
    entry_name: &str,
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
            "kernel-context creation must bind `SuspendedKernelContext InitialProcessor`",
        ));
    };
    let [
        context,
        kernel,
        context_word,
        create,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            "kernel-context creation requires named `stack` and `entry` arguments",
        ));
    };
    if source.slice(*classifier) != "SuspendedKernelContext InitialProcessor"
        || !identifier_is(source, context, context_name)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, context_word, "context")
        || !identifier_is(source, create, "create")
        || !kernel_context_create_fields(source, fields, stack_name, entry_name)?
    {
        return Err(kernel_context_diagnostic(
            source,
            *span,
            format!(
                "kernel-context creation requires the live context, allocated stack, and typed `{entry_name}` entry"
            ),
        ));
    }
    Ok(source.slice(*name).to_owned())
}

fn parse_user_address_space_prepare<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    backing_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let Statement::Implementation {
        name,
        classifier: Expression::Application { items, span },
        declarations,
        ..
    } = statement
    else {
        return Err(user_context_diagnostic(
            source,
            statement_span(statement),
            "user-address-space preparation must bind one opaque affine space",
        ));
    };
    let [
        receiver,
        user,
        address,
        space,
        prepare,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user-address-space preparation requires named `backing` and `layout` parameters",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, address, "address")
        || !identifier_is(source, space, "space")
        || !identifier_is(source, prepare, "prepare")
        || fields.len() != 2
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "the initial user address space requires the exact three-frame backing and sealed initial-probe layout",
        ));
    }
    let mut backing = None;
    let mut layout = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(user_context_diagnostic(
                source,
                field.value.span(),
                "user-address-space preparation parameters must be named",
            ));
        };
        let Expression::Identifier(value) = field.value else {
            return Err(user_context_diagnostic(
                source,
                field.value.span(),
                "user-address-space preparation values must use the live backing and sealed layout identity",
            ));
        };
        match source.slice(label) {
            "backing" => backing = Some(source.slice(value)),
            "layout" => layout = Some(source.slice(value)),
            _ => {
                return Err(user_context_diagnostic(
                    source,
                    label,
                    "unknown user-address-space preparation parameter",
                ));
            }
        }
    }
    if backing != Some(backing_name) || layout != Some("initial-probe") {
        return Err(user_context_diagnostic(
            source,
            *span,
            "the initial user address space requires the exact three-frame backing and sealed initial-probe layout",
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}

fn parse_user_address_space_map<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    address_space_name: &str,
    backing_name: &str,
    layout_name: &str,
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
                "user mapping publication must bind one fresh affine mapping",
            ));
        }
    };
    let [
        receiver,
        user,
        address,
        space,
        map,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(user_context_diagnostic(
            source,
            *span,
            "user mapping publication requires named address-space, backing, and layout parameters",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, user, "user")
        || !identifier_is(source, address, "address")
        || !identifier_is(source, space, "space")
        || !identifier_is(source, map, "map")
        || fields.len() != 3
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "the initial user mapping requires the exact prepared space, disjoint backing, and probe-guard layout",
        ));
    }
    let mut address_space = None;
    let mut backing = None;
    let mut layout = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(user_context_diagnostic(
                source,
                field.value.span(),
                "user mapping publication parameters must be named",
            ));
        };
        let Expression::Identifier(value) = field.value else {
            return Err(user_context_diagnostic(
                source,
                field.value.span(),
                "user mapping publication values must use live affine identities",
            ));
        };
        match source.slice(label) {
            "address-space" => address_space = Some(source.slice(value)),
            "backing" => backing = Some(source.slice(value)),
            "layout" => layout = Some(source.slice(value)),
            _ => {
                return Err(user_context_diagnostic(
                    source,
                    label,
                    "unknown user mapping publication parameter",
                ));
            }
        }
    }
    if address_space != Some(address_space_name)
        || backing != Some(backing_name)
        || layout != Some(layout_name)
    {
        return Err(user_context_diagnostic(
            source,
            *span,
            "the initial user mapping requires the exact prepared space, disjoint backing, and probe-guard layout",
        ));
    }
    Ok((source.slice(*name).to_owned(), declarations))
}
