// TOPAL-SYSTEMS-KERNEL-IMAGE-PREPARE-001: source supplies no native geometry.
fn analyze_kernel_image_backing_sequence(
    source: &SourceText,
    statements: &[Statement],
    context: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CheckedBootstrapRegionDecision, Diagnostic> {
    let (subject, owner, success, failure_name, failure, span) =
        result_sequence(source, statements, "kernel-image backing reservation")?;
    let Expression::Application { items, .. } = subject else {
        return Err(translation_diagnostic(source, span, "kernel-image backing requires its allocator context"));
    };
    let [receiver, kernel, image, reserve, backing, Expression::Product { fields, .. }] = items.as_slice() else {
        return Err(translation_diagnostic(source, span, "kernel-image backing requires current-kernel and provider-selected page policy"));
    };
    let tables = identifier_is(source, reserve, "prepare") && identifier_is(source, backing, "tables");
    let raw = identifier_is(source, reserve, "reserve") && identifier_is(source, backing, "backing");
    if !identifier_is(source, receiver, context)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, image, "image")
        || !(tables || raw)
        || fields.len() != 2
    {
        return Err(translation_diagnostic(source, span, "kernel-image backing requires its exact allocator, image and page policy"));
    }
    let mut selected_image = false;
    let mut selected_policy = false;
    for field in fields {
        let label = field.label.ok_or_else(|| translation_diagnostic(source, field.value.span(), "kernel-image backing parameters must be named"))?;
        let expected = match source.slice(label) {
            "image" if !selected_image => { selected_image = true; "current-kernel" }
            "page-policy" if !selected_policy => { selected_policy = true; "provider-selected" }
            _ => return Err(translation_diagnostic(source, label, "unknown or duplicate kernel-image backing parameter")),
        };
        if !identifier_is(source, &field.value, expected) {
            return Err(translation_diagnostic(source, field.value.span(), format!("kernel-image backing requires `{expected}`")));
        }
    }
    let failure_message = translation_failure(source, failure, &failure_name, "kernel-image backing failure must enter fatal")?;
    let body = translation_block(source, success, span, "kernel-image backing requires an affine discard block")?;
    let [reserved_marker, discard, tail @ ..] = body else {
        return Err(translation_diagnostic(source, span, "kernel-image backing requires observation, exact discard and continuation"));
    };
    let reserved_marker = parse_static_image_marker(source, reserved_marker, context,
        if tables { "TOPAL_KERNEL_IMAGE_TABLES_PREPARED" } else { "TOPAL_KERNEL_IMAGE_BACKING_RESERVED" },
        "kernel-image preparation requires its exact observation")?;
    if tables && let Statement::Expression(Expression::Application { items, .. }) = discard
        && items.get(3).is_some_and(|word| identifier_is(source, word, "retain"))
    {
        if !tail.is_empty() {
            return Err(translation_diagnostic(source, span, "scoped retention is the terminal preparation consumer"));
        }
        return analyze_kernel_image_retention(source, body, context, &owner, storage, failure_message, reserved_marker);
    }
    let [discarded_marker, rest @ ..] = tail else {
        return Err(translation_diagnostic(source,span,"immediate discard requires its observation and continuation"));
    };
    let Statement::Expression(Expression::Application { items, .. }) = discard else {
        return Err(translation_diagnostic(source, statement_span(discard), "kernel-image backing requires its consuming discard"));
    };
    let words = [context, "kernel", "image", "discard", if tables { "tables" } else { "backing" }, owner.as_str()];
    if items.len() != words.len() || !items.iter().zip(words).all(|(item, word)| identifier_is(source, item, word)) {
        return Err(translation_diagnostic(source, statement_span(discard), "kernel-image backing discard requires the exact allocator and live owner"));
    }
    let discarded_marker = parse_static_image_marker(source, discarded_marker, context,
        if tables { "TOPAL_KERNEL_IMAGE_TABLES_DISCARDED" } else { "TOPAL_KERNEL_IMAGE_BACKING_DISCARDED" },
        "kernel-image preparation requires its exact discard observation")?;
    let (next, ..) = result_sequence(source, rest, "translation construction after image backing")?;
    parse_translation_begin(source, next, context)?;
    let mut checked = analyze_translation_sequence(source, rest, context, storage)?;
    checked.operations.splice(0..0, [
        if tables { CompilerSystemsOperation::PrepareKernelImageTables { failure_message } }
        else { CompilerSystemsOperation::ReserveKernelImageBacking { failure_message } },
        reserved_marker,
        if tables { CompilerSystemsOperation::DiscardKernelImageTables }
        else { CompilerSystemsOperation::DiscardKernelImageBacking },
        discarded_marker,
    ]);
    Ok(checked)
}

fn analyze_kernel_image_retention(
    source: &SourceText,
    body: &[Statement],
    context: &str,
    owner: &str,
    storage: &CompilerBootstrapStorageDescriptor,
    failure_message: String,
    prepared: CompilerSystemsOperation,
) -> Result<CheckedBootstrapRegionDecision, Diagnostic> {
    if owner==context {
        return Err(translation_diagnostic(source,statement_span(&body[0]),"retention context and prepared owner must have distinct names"));
    }
    let [_, Statement::Expression(Expression::Application { items, span })] = body else {
        return Err(translation_diagnostic(source, Span::new(0,0), "retention requires one scoped consumer after preparation observation"));
    };
    let [receiver,kernel,image,retain,tables,value,during,Expression::Block { statements, .. }] = items.as_slice() else {
        return Err(translation_diagnostic(source,*span,"retention requires the exact owner and a checked `during` block"));
    };
    if ![receiver,kernel,image,retain,tables,value,during].into_iter()
        .zip([context,"kernel","image","retain","tables",owner,"during"])
        .all(|(item,word)|identifier_is(source,item,word))
    {
        return Err(translation_diagnostic(source,*span,"retention requires the exact context and prepared table owner"));
    }
    let [marker,continuation @ ..] = statements.as_slice() else {
        return Err(translation_diagnostic(source,*span,"retention requires observation and an ordinary boot continuation"));
    };
    let marker=parse_static_image_marker(source,marker,context,"TOPAL_KERNEL_IMAGE_TABLES_RETAINED","retention requires its exact observation")?;
    let mut checked=analyze_translation_sequence(source,continuation,context,storage)?;
    checked.operations.splice(0..0,[
        CompilerSystemsOperation::PrepareKernelImageTables { failure_message },prepared,
        CompilerSystemsOperation::RetainKernelImageTables,marker,
    ]);
    checked.operations.extend([
        CompilerSystemsOperation::DiscardRetainedKernelImageTables,
        CompilerSystemsOperation::ConsoleWrite { text:"TOPAL_KERNEL_IMAGE_RETAINED_TABLES_DISCARDED".into() },
    ]);
    Ok(checked)
}
