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
    if !identifier_is(source, receiver, context)
        || !identifier_is(source, kernel, "kernel")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, reserve, "reserve")
        || !identifier_is(source, backing, "backing")
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
    let [reserved_marker, discard, discarded_marker, rest @ ..] = body else {
        return Err(translation_diagnostic(source, span, "kernel-image backing requires observation, exact discard and continuation"));
    };
    let reserved_marker = parse_static_image_marker(source, reserved_marker, context,
        "TOPAL_KERNEL_IMAGE_BACKING_RESERVED", "kernel-image backing requires its exact reservation observation")?;
    let Statement::Expression(Expression::Application { items, .. }) = discard else {
        return Err(translation_diagnostic(source, statement_span(discard), "kernel-image backing requires its consuming discard"));
    };
    let words = [context, "kernel", "image", "discard", "backing", owner.as_str()];
    if items.len() != words.len() || !items.iter().zip(words).all(|(item, word)| identifier_is(source, item, word)) {
        return Err(translation_diagnostic(source, statement_span(discard), "kernel-image backing discard requires the exact allocator and live owner"));
    }
    let discarded_marker = parse_static_image_marker(source, discarded_marker, context,
        "TOPAL_KERNEL_IMAGE_BACKING_DISCARDED", "kernel-image backing requires its exact discard observation")?;
    let (next, ..) = result_sequence(source, rest, "translation construction after image backing")?;
    parse_translation_begin(source, next, context)?;
    let mut checked = analyze_translation_sequence(source, rest, context, storage)?;
    checked.operations.splice(0..0, [
        CompilerSystemsOperation::ReserveKernelImageBacking { failure_message },
        reserved_marker,
        CompilerSystemsOperation::DiscardKernelImageBacking,
        discarded_marker,
    ]);
    Ok(checked)
}
