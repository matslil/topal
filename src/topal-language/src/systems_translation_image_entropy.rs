fn parse_static_image_entropy_negotiation(
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
                "static-image entropy-transport negotiation requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-transport negotiation requires named parameters",
        ));
    };
    if items.len() != 8
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], "qualify")
        || !identifier_is(source, &items[5], "entropy")
        || !identifier_is(source, &items[6], "negotiation")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-transport negotiation requires its exact owner, transport identity, and policy",
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
                "static-image entropy-transport negotiation parameters must be named",
            ));
        };
        let Expression::Identifier(identity) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy-transport negotiation parameters must be identifiers",
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
                    "unknown static-image entropy-transport negotiation parameter",
                ));
            }
        }
    }
    if owner != Some(owner_name)
        || transport != Some("initial-virtio-rng-pci-transport")
        || policy != Some("bounded-static-elf-entropy-transport-negotiation")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-transport negotiation requires the exact owner, initial entropy transport, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_queue(
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
                "static-image entropy-queue discovery requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-queue discovery requires named parameters",
        ));
    };
    if items.len() != 8
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], "qualify")
        || !identifier_is(source, &items[5], "entropy")
        || !identifier_is(source, &items[6], "queue")
        || fields.len() != 4
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-queue discovery requires its exact owner, transport identity, queue identity, and policy",
        ));
    }
    let mut owner = None;
    let mut transport = None;
    let mut queue = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy-queue parameters must be named",
            ));
        };
        let Expression::Identifier(identity) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy-queue parameters must be identifiers",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "transport" => transport = Some(source.slice(*identity)),
            "queue" => queue = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image entropy-queue parameter",
                ));
            }
        }
    }
    if owner != Some(owner_name)
        || transport != Some("initial-virtio-rng-pci-transport")
        || queue != Some("initial-virtio-rng-queue-zero")
        || policy != Some("bounded-static-elf-entropy-queue-discovery")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-queue discovery requires the exact owner, initial entropy transport, queue zero, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_bus_master(
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
                "static-image entropy bus-master qualification requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy bus-master qualification requires named parameters",
        ));
    };
    if items.len() != 9
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], "qualify")
        || !identifier_is(source, &items[5], "entropy")
        || !identifier_is(source, &items[6], "bus")
        || !identifier_is(source, &items[7], "master")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy bus-master qualification requires its exact owner, device identity, and policy",
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
                "static-image entropy bus-master parameters must be named",
            ));
        };
        let Expression::Identifier(identity) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy bus-master parameters must be identifiers",
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
                    "unknown static-image entropy bus-master parameter",
                ));
            }
        }
    }
    if owner != Some(owner_name)
        || device != Some("initial-virtio-rng-device")
        || policy != Some("bounded-static-elf-entropy-bus-master")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy bus-master qualification requires the exact owner, initial entropy device, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_dma_buffers(
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
                "static-image entropy DMA-buffer coexistence requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy DMA-buffer coexistence requires named parameters",
        ));
    };
    if items.len() != 9
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], "qualify")
        || !identifier_is(source, &items[5], "entropy")
        || !identifier_is(source, &items[6], "dma")
        || !identifier_is(source, &items[7], "buffers")
        || fields.len() != 4
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy DMA-buffer coexistence requires its exact owner, device, translation domain, and policy",
        ));
    }
    let mut owner = None;
    let mut device = None;
    let mut domain = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy DMA-buffer parameters must be named",
            ));
        };
        let Expression::Identifier(identity) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy DMA-buffer parameters must be identifiers",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "device" => device = Some(source.slice(*identity)),
            "domain" => domain = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image entropy DMA-buffer parameter",
                ));
            }
        }
    }
    if owner != Some(owner_name)
        || device != Some("initial-virtio-rng-device")
        || domain != Some("initial-dma-domain")
        || policy != Some("bounded-static-elf-entropy-dma-buffer-coexistence")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy DMA-buffer coexistence requires the exact owner, initial entropy device, DMA domain, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_split_ring(
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
                "static-image entropy split-ring coexistence requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy split-ring coexistence requires named parameters",
        ));
    };
    if items.len() != 9
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], "qualify")
        || !identifier_is(source, &items[5], "entropy")
        || !identifier_is(source, &items[6], "split")
        || !identifier_is(source, &items[7], "ring")
        || fields.len() != 5
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy split-ring coexistence requires its exact owner, device, translation domain, queue, and policy",
        ));
    }
    let mut owner = None;
    let mut device = None;
    let mut domain = None;
    let mut queue = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy split-ring parameters must be named",
            ));
        };
        let Expression::Identifier(identity) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy split-ring parameters must be identifiers",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "device" => device = Some(source.slice(*identity)),
            "domain" => domain = Some(source.slice(*identity)),
            "queue" => queue = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image entropy split-ring parameter",
                ));
            }
        }
    }
    if owner != Some(owner_name)
        || device != Some("initial-virtio-rng-device")
        || domain != Some("initial-dma-domain")
        || queue != Some("initial-virtio-rng-queue-zero")
        || policy != Some("bounded-static-elf-entropy-split-ring-coexistence")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy split-ring coexistence requires the exact owner, initial entropy device, DMA domain, queue, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_dma_preparation(
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
                "static-image entropy DMA-preparation coexistence requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy DMA-preparation coexistence requires named parameters",
        ));
    };
    if items.len() != 9
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], "qualify")
        || !identifier_is(source, &items[5], "entropy")
        || !identifier_is(source, &items[6], "dma")
        || !identifier_is(source, &items[7], "preparation")
        || fields.len() != 5
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy DMA-preparation coexistence requires its exact owner, device, translation domain, queue, and policy",
        ));
    }
    let mut owner = None;
    let mut device = None;
    let mut domain = None;
    let mut queue = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy DMA-preparation parameters must be named",
            ));
        };
        let Expression::Identifier(identity) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy DMA-preparation parameters must be identifiers",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "device" => device = Some(source.slice(*identity)),
            "domain" => domain = Some(source.slice(*identity)),
            "queue" => queue = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image entropy DMA-preparation parameter",
                ));
            }
        }
    }
    if owner != Some(owner_name)
        || device != Some("initial-virtio-rng-device")
        || domain != Some("initial-dma-domain")
        || queue != Some("initial-virtio-rng-queue-zero")
        || policy != Some("bounded-static-elf-entropy-dma-preparation-coexistence")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy DMA-preparation coexistence requires the exact owner, initial entropy device, DMA domain, queue, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_queue_addresses(
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
                "static-image entropy queue-address coexistence requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy queue-address coexistence requires named parameters",
        ));
    };
    if items.len() != 9
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], "qualify")
        || !identifier_is(source, &items[5], "entropy")
        || !identifier_is(source, &items[6], "queue")
        || !identifier_is(source, &items[7], "addresses")
        || fields.len() != 6
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy queue-address coexistence requires its exact owner, device, translation domain, queue, transport, and policy",
        ));
    }
    let mut owner = None;
    let mut device = None;
    let mut domain = None;
    let mut queue = None;
    let mut transport = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy queue-address parameters must be named",
            ));
        };
        let Expression::Identifier(identity) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy queue-address parameters must be identifiers",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "device" => device = Some(source.slice(*identity)),
            "domain" => domain = Some(source.slice(*identity)),
            "queue" => queue = Some(source.slice(*identity)),
            "transport" => transport = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image entropy queue-address parameter",
                ));
            }
        }
    }
    if owner != Some(owner_name)
        || device != Some("initial-virtio-rng-device")
        || domain != Some("initial-dma-domain")
        || queue != Some("initial-virtio-rng-queue-zero")
        || transport != Some("initial-virtio-rng-pci-transport")
        || policy != Some("bounded-static-elf-entropy-queue-address-coexistence")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy queue-address coexistence requires the exact owner, initial entropy device, DMA domain, queue, transport, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_queue_activation(
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
                "static-image entropy queue-activation coexistence requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy queue-activation coexistence requires named parameters",
        ));
    };
    if items.len() != 9
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], "qualify")
        || !identifier_is(source, &items[5], "entropy")
        || !identifier_is(source, &items[6], "queue")
        || !identifier_is(source, &items[7], "activation")
        || fields.len() != 6
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy queue-activation coexistence requires its exact owner, device, translation domain, queue, transport, and policy",
        ));
    }
    let mut owner = None;
    let mut device = None;
    let mut domain = None;
    let mut queue = None;
    let mut transport = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy queue-activation parameters must be named",
            ));
        };
        let Expression::Identifier(identity) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy queue-activation parameters must be identifiers",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "device" => device = Some(source.slice(*identity)),
            "domain" => domain = Some(source.slice(*identity)),
            "queue" => queue = Some(source.slice(*identity)),
            "transport" => transport = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image entropy queue-activation parameter",
                ));
            }
        }
    }
    if owner != Some(owner_name)
        || device != Some("initial-virtio-rng-device")
        || domain != Some("initial-dma-domain")
        || queue != Some("initial-virtio-rng-queue-zero")
        || transport != Some("initial-virtio-rng-pci-transport")
        || policy != Some("bounded-static-elf-entropy-empty-queue-activation-coexistence")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy queue-activation coexistence requires the exact owner, initial entropy device, DMA domain, queue, transport, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_request_completion(
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
                "static-image entropy request-completion coexistence requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy request-completion coexistence requires named parameters",
        ));
    };
    if items.len() != 9
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], "qualify")
        || !identifier_is(source, &items[5], "entropy")
        || !identifier_is(source, &items[6], "request")
        || !identifier_is(source, &items[7], "completion")
        || fields.len() != 6
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy request-completion coexistence requires its exact owner, device, translation domain, queue, transport, and policy",
        ));
    }
    let mut owner = None;
    let mut device = None;
    let mut domain = None;
    let mut queue = None;
    let mut transport = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy request-completion parameters must be named",
            ));
        };
        let Expression::Identifier(identity) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy request-completion parameters must be identifiers",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "device" => device = Some(source.slice(*identity)),
            "domain" => domain = Some(source.slice(*identity)),
            "queue" => queue = Some(source.slice(*identity)),
            "transport" => transport = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image entropy request-completion parameter",
                ));
            }
        }
    }
    if owner != Some(owner_name)
        || device != Some("initial-virtio-rng-device")
        || domain != Some("initial-dma-domain")
        || queue != Some("initial-virtio-rng-queue-zero")
        || transport != Some("initial-virtio-rng-pci-transport")
        || policy != Some("bounded-static-elf-entropy-request-completion-coexistence")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy request-completion coexistence requires the exact owner, initial entropy device, DMA domain, queue, transport, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_sample_capture(
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
                "static-image entropy-sample capture coexistence requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-sample capture coexistence requires named parameters",
        ));
    };
    if items.len() != 9
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], "qualify")
        || !identifier_is(source, &items[5], "entropy")
        || !identifier_is(source, &items[6], "sample")
        || !identifier_is(source, &items[7], "capture")
        || fields.len() != 7
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-sample capture coexistence requires its exact owner, device, translation domain, queue, transport, sample, and policy",
        ));
    }
    let mut owner = None;
    let mut device = None;
    let mut domain = None;
    let mut queue = None;
    let mut transport = None;
    let mut sample = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy-sample capture parameters must be named",
            ));
        };
        let Expression::Identifier(identity) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy-sample capture parameters must be identifiers",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "device" => device = Some(source.slice(*identity)),
            "domain" => domain = Some(source.slice(*identity)),
            "queue" => queue = Some(source.slice(*identity)),
            "transport" => transport = Some(source.slice(*identity)),
            "sample" => sample = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image entropy-sample capture parameter",
                ));
            }
        }
    }
    if owner != Some(owner_name)
        || device != Some("initial-virtio-rng-device")
        || domain != Some("initial-dma-domain")
        || queue != Some("initial-virtio-rng-queue-zero")
        || transport != Some("initial-virtio-rng-pci-transport")
        || sample != Some("initial-virtio-rng-sample")
        || policy != Some("bounded-static-elf-entropy-sample-capture-coexistence")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-sample capture coexistence requires the exact owner, initial entropy device, DMA domain, queue, transport, sample, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_sample_quality(
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
                "static-image entropy-sample quality coexistence requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-sample quality coexistence requires named parameters",
        ));
    };
    if items.len() != 9
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], "qualify")
        || !identifier_is(source, &items[5], "entropy")
        || !identifier_is(source, &items[6], "sample")
        || !identifier_is(source, &items[7], "quality")
        || fields.len() != 9
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-sample quality coexistence requires its exact owner, transport resources, sample, source, evidence, and policy",
        ));
    }
    let mut values = std::collections::BTreeMap::new();
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy-sample quality parameters must be named",
            ));
        };
        let Expression::Identifier(identity) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy-sample quality parameters must be identifiers",
            ));
        };
        if values
            .insert(source.slice(label), source.slice(*identity))
            .is_some()
        {
            return Err(translation_edit_diagnostic(
                source,
                label,
                "duplicate static-image entropy-sample quality parameter",
            ));
        }
    }
    let exact = [
        ("owner", owner_name),
        ("device", "initial-virtio-rng-device"),
        ("domain", "initial-dma-domain"),
        ("queue", "initial-virtio-rng-queue-zero"),
        ("transport", "initial-virtio-rng-pci-transport"),
        ("sample", "initial-virtio-rng-sample"),
        ("source", "initial-virtio-rng-source"),
        ("evidence", "initial-q35-virtio-rng-trust-evidence"),
        (
            "policy",
            "bounded-static-elf-entropy-sample-quality-coexistence",
        ),
    ];
    if values.len() != exact.len()
        || exact
            .iter()
            .any(|(label, value)| values.get(label) != Some(value))
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-sample quality coexistence requires the exact owner, transport resources, sample, source, evidence, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_pool_lifecycle(
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
                "static-image entropy-pool lifecycle coexistence requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-pool lifecycle coexistence requires named parameters",
        ));
    };
    if items.len() != 9
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], "qualify")
        || !identifier_is(source, &items[5], "entropy")
        || !identifier_is(source, &items[6], "pool")
        || !identifier_is(source, &items[7], "lifecycle")
        || fields.len() != 12
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-pool lifecycle coexistence requires its exact owner, transport resources, entropy resources, and policy",
        ));
    }
    let mut values = std::collections::BTreeMap::new();
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy-pool lifecycle parameters must be named",
            ));
        };
        let Expression::Identifier(identity) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy-pool lifecycle parameters must be identifiers",
            ));
        };
        if values
            .insert(source.slice(label), source.slice(*identity))
            .is_some()
        {
            return Err(translation_edit_diagnostic(
                source,
                label,
                "duplicate static-image entropy-pool lifecycle parameter",
            ));
        }
    }
    let exact = [
        ("owner", owner_name),
        ("device", "initial-virtio-rng-device"),
        ("domain", "initial-dma-domain"),
        ("queue", "initial-virtio-rng-queue-zero"),
        ("transport", "initial-virtio-rng-pci-transport"),
        ("sample", "initial-virtio-rng-sample"),
        ("source", "initial-virtio-rng-source"),
        ("evidence", "initial-q35-virtio-rng-trust-evidence"),
        ("pool", "initial-entropy-pool"),
        ("conditioner", "initial-blake2s-conditioner"),
        ("contribution", "initial-entropy-contribution"),
        (
            "policy",
            "bounded-static-elf-entropy-pool-lifecycle-coexistence",
        ),
    ];
    if values.len() != exact.len()
        || exact
            .iter()
            .any(|(label, value)| values.get(label) != Some(value))
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-pool lifecycle coexistence requires the exact owner, transport resources, entropy resources, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_generator_lifecycle(
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
                "static-image entropy-generator lifecycle coexistence requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-generator lifecycle coexistence requires named parameters",
        ));
    };
    if items.len() != 9
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], "qualify")
        || !identifier_is(source, &items[5], "entropy")
        || !identifier_is(source, &items[6], "generator")
        || !identifier_is(source, &items[7], "lifecycle")
        || fields.len() != 15
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-generator lifecycle coexistence requires its exact owner, transport resources, entropy resources, generator resources, and policy",
        ));
    }
    let mut values = std::collections::BTreeMap::new();
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy-generator lifecycle parameters must be named",
            ));
        };
        let Expression::Identifier(identity) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entropy-generator lifecycle parameters must be identifiers",
            ));
        };
        if values
            .insert(source.slice(label), source.slice(*identity))
            .is_some()
        {
            return Err(translation_edit_diagnostic(
                source,
                label,
                "duplicate static-image entropy-generator lifecycle parameter",
            ));
        }
    }
    let exact = [
        ("owner", owner_name),
        ("device", "initial-virtio-rng-device"),
        ("domain", "initial-dma-domain"),
        ("queue", "initial-virtio-rng-queue-zero"),
        ("transport", "initial-virtio-rng-pci-transport"),
        ("sample", "initial-virtio-rng-sample"),
        ("source", "initial-virtio-rng-source"),
        ("evidence", "initial-q35-virtio-rng-trust-evidence"),
        ("pool", "initial-entropy-pool"),
        ("conditioner", "initial-blake2s-conditioner"),
        ("contribution", "initial-entropy-contribution"),
        ("generator", "initial-entropy-generator"),
        ("algorithm", "chacha20"),
        ("output", "initial-entropy-output"),
        (
            "policy",
            "bounded-static-elf-entropy-generator-lifecycle-coexistence",
        ),
    ];
    if values.len() != exact.len()
        || exact
            .iter()
            .any(|(label, value)| values.get(label) != Some(value))
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entropy-generator lifecycle coexistence requires the exact owner, transport resources, entropy resources, generator resources, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_initial_random_delivery(
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
                "static-image initial-random delivery requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image initial-random delivery requires named parameters",
        ));
    };
    if items.len() != 9
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], "deliver")
        || !identifier_is(source, &items[5], "entropy")
        || !identifier_is(source, &items[6], "initial")
        || !identifier_is(source, &items[7], "random")
        || fields.len() != 17
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image initial-random delivery requires its exact owner, entropy resources, interface, delivery, and policy",
        ));
    }
    let mut values = std::collections::BTreeMap::new();
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image initial-random delivery parameters must be named",
            ));
        };
        let Expression::Identifier(identity) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image initial-random delivery parameters must be identifiers",
            ));
        };
        if values
            .insert(source.slice(label), source.slice(*identity))
            .is_some()
        {
            return Err(translation_edit_diagnostic(
                source,
                label,
                "duplicate static-image initial-random delivery parameter",
            ));
        }
    }
    let exact = [
        ("owner", owner_name),
        ("device", "initial-virtio-rng-device"),
        ("domain", "initial-dma-domain"),
        ("queue", "initial-virtio-rng-queue-zero"),
        ("transport", "initial-virtio-rng-pci-transport"),
        ("sample", "initial-virtio-rng-sample"),
        ("source", "initial-virtio-rng-source"),
        ("evidence", "initial-q35-virtio-rng-trust-evidence"),
        ("pool", "initial-entropy-pool"),
        ("conditioner", "initial-blake2s-conditioner"),
        ("contribution", "initial-entropy-contribution"),
        ("generator", "initial-entropy-generator"),
        ("algorithm", "chacha20"),
        ("output", "initial-entropy-output"),
        ("interface", "initial-linux-random-interface"),
        ("delivery", "initial-auxv-random-delivery"),
        (
            "policy",
            "bounded-static-elf-initial-random-delivery-coexistence",
        ),
    ];
    if values.len() != exact.len()
        || exact
            .iter()
            .any(|(label, value)| values.get(label) != Some(value))
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image initial-random delivery requires the exact owner, entropy resources, interface, delivery, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entropy_initial_random_revocation(
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
                "static-image initial-random revocation requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image initial-random revocation requires named parameters",
        ));
    };
    if items.len() != 9
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], "revoke")
        || !identifier_is(source, &items[5], "entropy")
        || !identifier_is(source, &items[6], "initial")
        || !identifier_is(source, &items[7], "random")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image initial-random revocation requires its exact owner, delivery, and policy",
        ));
    }
    let mut values = std::collections::BTreeMap::new();
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image initial-random revocation parameters must be named",
            ));
        };
        let Expression::Identifier(identity) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image initial-random revocation parameters must be identifiers",
            ));
        };
        if values
            .insert(source.slice(label), source.slice(*identity))
            .is_some()
        {
            return Err(translation_edit_diagnostic(
                source,
                label,
                "duplicate static-image initial-random revocation parameter",
            ));
        }
    }
    let exact = [
        ("owner", owner_name),
        ("delivery", "initial-auxv-random-delivery"),
        (
            "policy",
            "bounded-static-elf-initial-random-delivery-qualification-rollback",
        ),
    ];
    if values.len() != exact.len()
        || exact
            .iter()
            .any(|(label, value)| values.get(label) != Some(value))
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image initial-random revocation requires the exact owner, delivery, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

include!("systems_translation_image_process_restore.rs");
