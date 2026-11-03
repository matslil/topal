fn parse_static_image_stack_platform_transition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
    add: bool,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image platform transition must refine one exact stack entry",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image platform transition requires named parameters",
        ));
    };
    let verb = if add { "add" } else { "remove" };
    if items.len() != 8
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], verb)
        || !identifier_is(source, &items[5], "stack")
        || !identifier_is(source, &items[6], "platform")
        || fields.len() != if add { 4 } else { 3 }
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image platform transition requires its exact owner, backing, value, and policy",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut value = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(identity)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image platform parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "backing" => backing = Some(source.slice(*identity)),
            "value" => value = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image platform parameter",
                ));
            }
        }
    }
    let expected_value = add.then_some("native-userspace-platform");
    let expected_policy = if add {
        "bounded-linux-auxv-before-null"
    } else {
        "bounded-stack-platform-qualification-rollback"
    };
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || value != expected_value
        || policy != Some(expected_policy)
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image platform transition requires the exact owner, backing, native userspace platform, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_hardware_capabilities_transition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
    add: bool,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image hardware-capability transition must refine one exact stack entry",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image hardware-capability transition requires named parameters",
        ));
    };
    let verb = if add { "add" } else { "remove" };
    if items.len() != 9
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], verb)
        || !identifier_is(source, &items[5], "stack")
        || !identifier_is(source, &items[6], "hardware")
        || !identifier_is(source, &items[7], "capabilities")
        || fields.len() != if add { 4 } else { 3 }
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image hardware-capability transition requires its exact owner, backing, value, and policy",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut value = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(identity)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image hardware-capability parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "backing" => backing = Some(source.slice(*identity)),
            "value" => value = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image hardware-capability parameter",
                ));
            }
        }
    }
    let expected_value = add.then_some("native-userspace-hardware-capabilities");
    let expected_policy = if add {
        "bounded-linux-auxv-before-null"
    } else {
        "bounded-stack-hardware-capabilities-qualification-rollback"
    };
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || value != expected_value
        || policy != Some(expected_policy)
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image hardware-capability transition requires the exact owner, backing, native userspace capabilities, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_load_metadata_transition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
    add: bool,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image load-metadata transition must refine one exact stack entry",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image load-metadata transition requires named parameters",
        ));
    };
    let verb = if add { "add" } else { "remove" };
    if items.len() != 9
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], verb)
        || !identifier_is(source, &items[5], "stack")
        || !identifier_is(source, &items[6], "load")
        || !identifier_is(source, &items[7], "metadata")
        || fields.len() != if add { 4 } else { 3 }
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image load-metadata transition requires its exact owner, backing, value, and policy",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut value = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(identity)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image load-metadata parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "backing" => backing = Some(source.slice(*identity)),
            "value" => value = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image load-metadata parameter",
                ));
            }
        }
    }
    let expected_value = add.then_some("static-executable-load-metadata");
    let expected_policy = if add {
        "bounded-linux-auxv-before-null"
    } else {
        "bounded-stack-load-metadata-qualification-rollback"
    };
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || value != expected_value
        || policy != Some(expected_policy)
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image load-metadata transition requires the exact owner, backing, static executable metadata, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_handoff(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    owner_name: &str,
    authorize: bool,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image entropy handoff requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy handoff requires named parameters",
        ));
    };
    let verb = if authorize { "authorize" } else { "revoke" };
    if items.len() != 8
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], verb)
        || !identifier_is(source, &items[5], "entropy")
        || !identifier_is(source, &items[6], "handoff")
        || fields.len() != if authorize { 3 } else { 2 }
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy handoff requires its exact owner, process identity, and policy",
        ));
    }
    let mut owner = None;
    let mut process = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy-handoff parameters must be named",
            ));
        };
        let Expression::Identifier(identity) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy-handoff parameters must be identifiers",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "process" => process = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image entropy-handoff parameter",
                ));
            }
        }
    }
    let expected_process = authorize.then_some("bounded-static-elf-initial-process");
    let expected_policy = if authorize {
        "bounded-static-elf-initial-random-delivery"
    } else {
        "bounded-static-elf-entropy-handoff-qualification-rollback"
    };
    if owner != Some(owner_name) || process != expected_process || policy != Some(expected_policy) {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy handoff requires the exact owner, process identity, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_acquisition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    owner_name: &str,
    authorize: bool,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image entropy acquisition requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy acquisition requires named parameters",
        ));
    };
    let verb = if authorize { "authorize" } else { "revoke" };
    if items.len() != 8
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], verb)
        || !identifier_is(source, &items[5], "entropy")
        || !identifier_is(source, &items[6], "acquisition")
        || fields.len() != if authorize { 4 } else { 2 }
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy acquisition requires its exact owner, output identity, generation, and policy",
        ));
    }
    let mut owner = None;
    let mut output_identity = None;
    let mut generation = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy-acquisition parameters must be named",
            ));
        };
        let Expression::Identifier(identity) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy-acquisition parameters must be identifiers",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "output" => output_identity = Some(source.slice(*identity)),
            "generation" => generation = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image entropy-acquisition parameter",
                ));
            }
        }
    }
    let expected_output = authorize.then_some("initial-entropy-output");
    let expected_generation = authorize.then_some("generation-two");
    let expected_policy = if authorize {
        "bounded-static-elf-initial-entropy-acquisition"
    } else {
        "bounded-static-elf-entropy-acquisition-qualification-rollback"
    };
    if owner != Some(owner_name)
        || output_identity != expected_output
        || generation != expected_generation
        || policy != Some(expected_policy)
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy acquisition requires the exact owner, output identity, generation, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_device(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    owner_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image entropy-device qualification requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-device qualification requires named parameters",
        ));
    };
    if items.len() != 8
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], "qualify")
        || !identifier_is(source, &items[5], "entropy")
        || !identifier_is(source, &items[6], "device")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-device qualification requires its exact owner, device identity, and policy",
        ));
    }
    let mut owner = None;
    let mut device = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy-device parameters must be named",
            ));
        };
        let Expression::Identifier(identity) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy-device parameters must be identifiers",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "device" => device = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image entropy-device parameter",
                ));
            }
        }
    }
    if owner != Some(owner_name)
        || device != Some("initial-virtio-rng-device")
        || policy != Some("bounded-static-elf-entropy-device-coexistence")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-device qualification requires the exact owner, initial entropy device, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_transport(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    owner_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } if declarations.is_empty() => (*name, items, *span),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image entropy-transport qualification requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-transport qualification requires named parameters",
        ));
    };
    if items.len() != 8
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], "qualify")
        || !identifier_is(source, &items[5], "entropy")
        || !identifier_is(source, &items[6], "transport")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-transport qualification requires its exact owner, transport identity, and policy",
        ));
    }
    let mut owner = None;
    let mut transport = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy-transport parameters must be named",
            ));
        };
        let Expression::Identifier(identity) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy-transport parameters must be identifiers",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "transport" => transport = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image entropy-transport parameter",
                ));
            }
        }
    }
    if owner != Some(owner_name)
        || transport != Some("initial-virtio-rng-pci-transport")
        || policy != Some("bounded-static-elf-entropy-transport-discovery")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-transport qualification requires the exact owner, initial entropy transport, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}
