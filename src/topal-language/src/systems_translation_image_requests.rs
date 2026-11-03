fn parse_static_image_hierarchy_reservation(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image hierarchy reservation cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image hierarchy reservation must bind one affine extent",
            ));
        }
    };
    let [
        receiver,
        static_word,
        image,
        construction,
        reserve,
        hierarchy,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image hierarchy reservation requires its exact owner and five-frame shape",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, reserve, "reserve")
        || !identifier_is(source, hierarchy, "hierarchy")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image hierarchy reservation requires its exact owner and five-frame shape",
        ));
    }
    let mut owner = None;
    let mut frame_count = None;
    let mut alignment_frames = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image hierarchy-reservation parameters must be named",
            ));
        };
        match source.slice(label) {
            "owner" => {
                let Expression::Identifier(value) = field.value else {
                    return Err(translation_edit_diagnostic(
                        source,
                        field.value.span(),
                        "static-image hierarchy owner must be the live sealed identity",
                    ));
                };
                owner = Some(source.slice(value));
            }
            "frame-count" => frame_count = Some(parse_storage_natural(source, &field.value)?),
            "alignment-frames" => {
                alignment_frames = Some(parse_storage_natural(source, &field.value)?);
            }
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image hierarchy-reservation parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name) || frame_count != Some(5) || alignment_frames != Some(1) {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image hierarchy reservation requires the exact sealed construction and five unit-aligned frames",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_address_space_attach(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    hierarchy_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image address-space attachment cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image address-space attachment must refine one sealed construction",
            ));
        }
    };
    let [
        receiver,
        static_word,
        image,
        construction,
        attach,
        address,
        space,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image address-space attachment requires its exact owner, hierarchy, and layout",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, attach, "attach")
        || !identifier_is(source, address, "address")
        || !identifier_is(source, space, "space")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image address-space attachment requires its exact owner, hierarchy, and layout",
        ));
    }
    let mut owner = None;
    let mut hierarchy = None;
    let mut layout = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image address-space parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "hierarchy" => hierarchy = Some(source.slice(*value)),
            "layout" => layout = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image address-space attachment parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name)
        || hierarchy != Some(hierarchy_name)
        || layout != Some("bounded-static-elf-inactive-address-space")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image address-space attachment requires the exact sealed construction, hierarchy extent, and inactive layout",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_address_space_detach(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image address-space detachment cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image address-space detachment must refine one attached construction",
            ));
        }
    };
    let [
        receiver,
        static_word,
        image,
        construction,
        detach,
        address,
        space,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image address-space detachment requires its exact owner",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, detach, "detach")
        || !identifier_is(source, address, "address")
        || !identifier_is(source, space, "space")
        || fields.len() != 1
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image address-space detachment requires its exact owner",
        ));
    }
    let field = &fields[0];
    let (Some(label), Expression::Identifier(owner)) = (field.label, &field.value) else {
        return Err(translation_edit_diagnostic(
            source,
            field.value.span(),
            "static-image address-space detachment owner must be a named identity",
        ));
    };
    if source.slice(label) != "owner" || source.slice(*owner) != construction_name {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image address-space detachment requires the exact restored construction",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_address_space_transition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    operation_name: &str,
    policy_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image address-space transition cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image address-space transition must refine one live construction",
            ));
        }
    };
    let [
        receiver,
        static_word,
        image,
        construction,
        operation,
        address,
        space,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image address-space transition requires its exact owner and policy",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, operation, operation_name)
        || !identifier_is(source, address, "address")
        || !identifier_is(source, space, "space")
        || fields.len() != 2
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image address-space transition requires its exact owner and policy",
        ));
    }
    let mut owner = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image address-space transition parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "policy" => policy = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image address-space transition parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name) || policy != Some(policy_name) {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image address-space transition requires the exact owner and continuation policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_entry_transition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    operation_name: &str,
    policy_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image entry transition cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image entry transition must refine one live construction",
            ));
        }
    };
    let [
        receiver,
        static_word,
        image,
        construction,
        operation,
        entry,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entry transition requires its exact owner and policy",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, operation, operation_name)
        || !identifier_is(source, entry, "entry")
        || fields.len() != 2
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entry transition requires its exact owner and policy",
        ));
    }
    let mut owner = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image entry transition parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "policy" => policy = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image entry transition parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name) || policy != Some(policy_name) {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image entry transition requires the exact owner and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_slot_transition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    operation_name: &str,
    policy_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image stack-slot transition cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image stack-slot transition must refine one live construction",
            ));
        }
    };
    let [
        receiver,
        static_word,
        image,
        construction,
        operation,
        stack,
        slot,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-slot transition requires its exact owner and policy",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, operation, operation_name)
        || !identifier_is(source, stack, "stack")
        || !identifier_is(source, slot, "slot")
        || fields.len() != 2
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-slot transition requires its exact owner and policy",
        ));
    }
    let mut owner = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image stack-slot transition parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "policy" => policy = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image stack-slot transition parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name) || policy != Some(policy_name) {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-slot transition requires the exact owner and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_backing_reservation(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image stack-backing reservation cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image stack-backing reservation must bind one affine extent",
            ));
        }
    };
    let [
        receiver,
        static_word,
        image,
        construction,
        reserve,
        stack,
        backing,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-backing reservation requires its exact owner and two-frame shape",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, reserve, "reserve")
        || !identifier_is(source, stack, "stack")
        || !identifier_is(source, backing, "backing")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-backing reservation requires its exact owner and two-frame shape",
        ));
    }
    let mut owner = None;
    let mut frame_count = None;
    let mut alignment_frames = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image stack-backing reservation parameters must be named",
            ));
        };
        match source.slice(label) {
            "owner" => {
                let Expression::Identifier(value) = field.value else {
                    return Err(translation_edit_diagnostic(
                        source,
                        field.value.span(),
                        "static-image stack-backing owner must be the live slot identity",
                    ));
                };
                owner = Some(source.slice(value));
            }
            "frame-count" => frame_count = Some(parse_storage_natural(source, &field.value)?),
            "alignment-frames" => {
                alignment_frames = Some(parse_storage_natural(source, &field.value)?);
            }
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image stack-backing reservation parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name) || frame_count != Some(2) || alignment_frames != Some(1) {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-backing reservation requires the exact slot-authorized construction and two unit-aligned frames",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_backing_zero(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image stack-backing zeroing cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image stack-backing zeroing must refine one live construction",
            ));
        }
    };
    let [
        receiver,
        static_word,
        image,
        construction,
        zero,
        stack,
        backing,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-backing zeroing requires its exact owner, backing, and complete extent",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, zero, "zero")
        || !identifier_is(source, stack, "stack")
        || !identifier_is(source, backing, "backing")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-backing zeroing requires its exact owner, backing, and complete extent",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut extent = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image stack-backing zeroing parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "backing" => backing = Some(source.slice(*value)),
            "extent" => extent = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image stack-backing zeroing parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || extent != Some("complete")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-backing zeroing requires the exact owner, backing, and complete extent",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_leaf_publication(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image stack-leaf publication cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image stack-leaf publication must refine one live construction",
            ));
        }
    };
    let [
        receiver,
        static_word,
        image,
        construction,
        publish,
        stack,
        leaf,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-leaf publication requires its exact owner, backing, access, and complete extent",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, publish, "publish")
        || !identifier_is(source, stack, "stack")
        || !identifier_is(source, leaf, "leaf")
        || fields.len() != 4
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-leaf publication requires its exact owner, backing, access, and complete extent",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut access = None;
    let mut extent = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image stack-leaf publication parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "backing" => backing = Some(source.slice(*value)),
            "access" => access = Some(source.slice(*value)),
            "extent" => extent = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image stack-leaf publication parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || access != Some("writable-non-executable")
        || extent != Some("complete")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-leaf publication requires the exact owner, backing, RW/NX access, and complete extent",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_leaf_revocation(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image stack-leaf revocation cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image stack-leaf revocation must refine one live construction",
            ));
        }
    };
    let [
        receiver,
        static_word,
        image,
        construction,
        revoke,
        stack,
        leaf,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-leaf revocation requires its exact owner, backing, and rollback policy",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, revoke, "revoke")
        || !identifier_is(source, stack, "stack")
        || !identifier_is(source, leaf, "leaf")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-leaf revocation requires its exact owner, backing, and rollback policy",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image stack-leaf revocation parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "backing" => backing = Some(source.slice(*value)),
            "policy" => policy = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image stack-leaf revocation parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || policy != Some("bounded-stack-leaf-qualification-rollback")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-leaf revocation requires the exact owner, backing, and rollback policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_entry_initialization(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
) -> Result<String, Diagnostic> {
    let (name, items, span) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => {
            if !declarations.is_empty() {
                return Err(translation_edit_diagnostic(
                    source,
                    statement_span(statement),
                    "static-image stack-entry initialization cannot contain nested operations",
                ));
            }
            (*name, items, *span)
        }
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image stack-entry initialization must refine one published stack leaf",
            ));
        }
    };
    let [
        receiver,
        static_word,
        image,
        construction,
        initialize,
        stack,
        entry,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-entry initialization requires its exact owner, backing, profile, and complete extent",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, initialize, "initialize")
        || !identifier_is(source, stack, "stack")
        || !identifier_is(source, entry, "entry")
        || fields.len() != 4
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-entry initialization requires its exact owner, backing, profile, and complete extent",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut profile = None;
    let mut extent = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image stack-entry initialization parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "backing" => backing = Some(source.slice(*value)),
            "profile" => profile = Some(source.slice(*value)),
            "extent" => extent = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image stack-entry initialization parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || profile != Some("bounded-static-elf-busybox-sh")
        || extent != Some("complete")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-entry initialization requires the exact owner, backing, sealed profile, and complete extent",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_pagesz_addition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
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
                "static-image AT_PAGESZ addition must refine one initialized stack entry",
            ));
        }
    };
    let [
        receiver,
        static_word,
        image,
        construction,
        add,
        stack,
        pagesz,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_PAGESZ addition requires its exact owner, backing, value, and policy",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, add, "add")
        || !identifier_is(source, stack, "stack")
        || !identifier_is(source, pagesz, "pagesz")
        || fields.len() != 4
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_PAGESZ addition requires its exact owner, backing, value, and policy",
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
                "static-image AT_PAGESZ addition parameters must be named identities",
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
                    "unknown static-image AT_PAGESZ addition parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || value != Some("provider-base-page")
        || policy != Some("bounded-linux-auxv-before-null")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_PAGESZ addition requires the exact owner, backing, provider page value, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_entry_fact_addition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
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
                "static-image AT_ENTRY addition must refine one page-size-qualified stack entry",
            ));
        }
    };
    let [
        receiver,
        static_word,
        image,
        construction,
        add,
        stack,
        entry,
        fact,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_ENTRY addition requires its exact owner, backing, entry value, and policy",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, add, "add")
        || !identifier_is(source, stack, "stack")
        || !identifier_is(source, entry, "entry")
        || !identifier_is(source, fact, "fact")
        || fields.len() != 4
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_ENTRY addition requires its exact owner, backing, entry value, and policy",
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
                "static-image AT_ENTRY addition parameters must be named identities",
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
                    "unknown static-image AT_ENTRY addition parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || value != Some("authorized-static-elf-entry")
        || policy != Some("bounded-linux-auxv-before-null")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_ENTRY addition requires the exact owner, backing, authorized entry value, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_executable_name_transition(
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
                "static-image executable-name transition must refine one exact stack entry",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image executable-name transition requires named parameters",
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
        || !identifier_is(source, &items[6], "executable")
        || !identifier_is(source, &items[7], "name")
        || fields.len() != if add { 4 } else { 3 }
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image executable-name transition requires its exact owner, backing, value, and policy",
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
                "static-image executable-name parameters must be named identities",
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
                    "unknown static-image executable-name parameter",
                ));
            }
        }
    }
    let expected_value = add.then_some("initial-executable-name");
    let expected_policy = if add {
        "bounded-linux-auxv-before-null"
    } else {
        "bounded-stack-executable-name-qualification-rollback"
    };
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || value != expected_value
        || policy != Some(expected_policy)
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image executable-name transition requires the exact owner, backing, executable name, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_credentials_transition(
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
                "static-image credential transition must refine one exact stack entry",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image credential transition requires named parameters",
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
        || !identifier_is(source, &items[6], "credentials")
        || fields.len() != if add { 4 } else { 3 }
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image credential transition requires its exact owner, backing, value, and policy",
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
                "static-image credential parameters must be named identities",
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
                    "unknown static-image credential parameter",
                ));
            }
        }
    }
    let expected_value = add.then_some("initial-process-credentials");
    let expected_policy = if add {
        "bounded-linux-auxv-before-null"
    } else {
        "bounded-stack-credentials-qualification-rollback"
    };
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || value != expected_value
        || policy != Some(expected_policy)
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image credential transition requires the exact owner, backing, process credentials, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_secure_mode_transition(
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
                "static-image secure-mode transition must refine one exact stack entry",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image secure-mode transition requires named parameters",
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
        || !identifier_is(source, &items[6], "secure")
        || !identifier_is(source, &items[7], "mode")
        || fields.len() != if add { 4 } else { 3 }
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image secure-mode transition requires its exact owner, backing, value, and policy",
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
                "static-image secure-mode parameters must be named identities",
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
                    "unknown static-image secure-mode parameter",
                ));
            }
        }
    }
    let expected_value = add.then_some("initial-execution-security");
    let expected_policy = if add {
        "bounded-linux-auxv-before-null"
    } else {
        "bounded-stack-secure-mode-qualification-rollback"
    };
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || value != expected_value
        || policy != Some(expected_policy)
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image secure-mode transition requires the exact owner, backing, execution security, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_clock_tick_rate_transition(
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
                "static-image clock-tick-rate transition must refine one exact stack entry",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image clock-tick-rate transition requires named parameters",
        ));
    };
    let verb = if add { "add" } else { "remove" };
    if items.len() != 10
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], verb)
        || !identifier_is(source, &items[5], "stack")
        || !identifier_is(source, &items[6], "clock")
        || !identifier_is(source, &items[7], "tick")
        || !identifier_is(source, &items[8], "rate")
        || fields.len() != if add { 4 } else { 3 }
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image clock-tick-rate transition requires its exact owner, backing, value, and policy",
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
                "static-image clock-tick-rate parameters must be named identities",
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
                    "unknown static-image clock-tick-rate parameter",
                ));
            }
        }
    }
    let expected_value = add.then_some("linux-user-hz");
    let expected_policy = if add {
        "bounded-linux-auxv-before-null"
    } else {
        "bounded-stack-clock-tick-rate-qualification-rollback"
    };
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || value != expected_value
        || policy != Some(expected_policy)
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image clock-tick-rate transition requires the exact owner, backing, Linux userspace accounting rate, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

include!("systems_translation_image_handoff.rs");
