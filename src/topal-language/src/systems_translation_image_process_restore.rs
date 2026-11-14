fn parse_static_image_process_entry(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    owner_name: &str,
    enter: bool,
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
                "static-image process entry transition requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image process entry transition requires named parameters",
        ));
    };
    let shape_matches = if enter {
        items.len() == 6
            && identifier_is(source, &items[0], context_name)
            && identifier_is(source, &items[1], "static")
            && identifier_is(source, &items[2], "image")
            && identifier_is(source, &items[3], "process")
            && identifier_is(source, &items[4], "enter")
            && fields.len() == 5
    } else {
        items.len() == 7
            && identifier_is(source, &items[0], context_name)
            && identifier_is(source, &items[1], "static")
            && identifier_is(source, &items[2], "image")
            && identifier_is(source, &items[3], "process")
            && identifier_is(source, &items[4], "restore")
            && identifier_is(source, &items[5], "entry")
            && fields.len() == 2
    };
    if !shape_matches {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image process entry transition requires its exact owner, ABI, terminal result, and policy",
        ));
    }
    let mut values = std::collections::BTreeMap::new();
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image process entry parameters must be named",
            ));
        };
        let Expression::Identifier(identity) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image process entry parameters must be identifiers",
            ));
        };
        if values
            .insert(source.slice(label), source.slice(*identity))
            .is_some()
        {
            return Err(translation_edit_diagnostic(
                source,
                label,
                "duplicate static-image process entry parameter",
            ));
        }
    }
    let exact = if enter {
        vec![
            ("owner", owner_name),
            ("abi", "linux-x86-64-process"),
            (
                "startup",
                "initial-brk-query-growth-x64-fs-base-clear-tid-empty-robust-list-rseq-fallback-stack-limit-readlinkat-fallback-getrandom-repeated-brk-query-growth-page-alignment-relro-mprotect-process-name-getuid-config-stat-miss-credential-tail-applet-newline-write-shell-getpid-sigchld-disposition-getppid-uname-root-getcwd-tcgets-miss-signal-defaults-quit-ignore-sealed-echo-command-console-fstat-dev-tty-open-miss-standard-terminal-probes-terminal-warning-mailpath-stat-miss-banner-post-banner-terminal-probe-post-banner-root-getcwd-post-banner-geteuid-profile-open-profile-fstat-profile-read-profile-close-prompt-refill-input-pwd-command-cd-root",
            ),
            ("terminal", "exit-group-zero"),
            ("policy", "bounded-static-elf-shell-cd-root-command"),
        ]
    } else {
        vec![
            ("owner", owner_name),
            (
                "policy",
                "bounded-static-elf-process-entry-qualification-rollback",
            ),
        ]
    };
    if values.len() != exact.len()
        || exact
            .iter()
            .any(|(label, value)| values.get(label) != Some(value))
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image process entry transition requires the exact owner, ABI, terminal result, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_retained_disjoint_work(
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
                "static-image retained disjoint work requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image retained disjoint work requires named parameters",
        ));
    };
    if items.len() != 9
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], "qualify")
        || !identifier_is(source, &items[5], "retained")
        || !identifier_is(source, &items[6], "disjoint")
        || !identifier_is(source, &items[7], "work")
        || fields.len() != 4
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image retained disjoint work requires its exact owner, one-frame request, and policy",
        ));
    }
    let mut owner = None;
    let mut frame_count = None;
    let mut alignment_frames = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image retained disjoint-work parameters must be named",
            ));
        };
        match source.slice(label) {
            "owner" | "policy" => {
                let Expression::Identifier(identity) = &field.value else {
                    return Err(translation_edit_diagnostic(
                        source,
                        field.value.span(),
                        "static-image retained disjoint-work identities must be identifiers",
                    ));
                };
                if source.slice(label) == "owner" {
                    owner = Some(source.slice(*identity));
                } else {
                    policy = Some(source.slice(*identity));
                }
            }
            "frame-count" => frame_count = Some(parse_storage_natural(source, &field.value)?),
            "alignment-frames" => {
                alignment_frames = Some(parse_storage_natural(source, &field.value)?)
            }
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image retained disjoint-work parameter",
                ));
            }
        }
    }
    if owner != Some(owner_name)
        || frame_count != Some(1)
        || alignment_frames != Some(1)
        || policy != Some("bounded-static-elf-retained-disjoint-work")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image retained disjoint work requires the exact owner, one-frame request, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_prepared_process_transition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    owner_name: &str,
    backing_name: Option<&str>,
    retain: bool,
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
                "static-image prepared-process transition requires one exact affine owner",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image prepared-process transition requires named parameters",
        ));
    };
    let verb = if retain { "retain" } else { "restore" };
    if items.len() != 8
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], verb)
        || !identifier_is(source, &items[5], "prepared")
        || !identifier_is(source, &items[6], "process")
        || fields.len() != if retain { 4 } else { 2 }
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image prepared-process transition requires its exact owner, retained value, and policy",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut value = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image prepared-process parameters must be named",
            ));
        };
        let Expression::Identifier(identity) = &field.value else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image prepared-process parameters must be identifiers",
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
                    "unknown static-image prepared-process parameter",
                ));
            }
        }
    }
    let expected_value = retain.then_some("bounded-static-elf-initial-process");
    let expected_policy = if retain {
        "bounded-static-elf-pre-entropy-retention"
    } else {
        "bounded-static-elf-retention-qualification-rollback"
    };
    if owner != Some(owner_name)
        || backing != backing_name
        || value != expected_value
        || policy != Some(expected_policy)
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image prepared-process transition requires the exact owner, backing, value, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_random_slot_transition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
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
                "static-image random-slot transition must refine one exact stack entry",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image random-slot transition requires named parameters",
        ));
    };
    let verb = if authorize { "authorize" } else { "revoke" };
    if items.len() != 9
        || !identifier_is(source, &items[0], context_name)
        || !identifier_is(source, &items[1], "static")
        || !identifier_is(source, &items[2], "image")
        || !identifier_is(source, &items[3], "construction")
        || !identifier_is(source, &items[4], verb)
        || !identifier_is(source, &items[5], "stack")
        || !identifier_is(source, &items[6], "random")
        || !identifier_is(source, &items[7], "slot")
        || fields.len() != if authorize { 5 } else { 3 }
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image random-slot transition requires its exact owner, backing, value, extent, and policy",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut value = None;
    let mut extent_bytes = None;
    let mut policy = None;
    for field in fields {
        let Some(label) = field.label else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image random-slot parameters must be named",
            ));
        };
        match source.slice(label) {
            "owner" | "backing" | "value" | "policy" => {
                let Expression::Identifier(identity) = &field.value else {
                    return Err(translation_edit_diagnostic(
                        source,
                        field.value.span(),
                        "static-image random-slot identity parameters must be identifiers",
                    ));
                };
                match source.slice(label) {
                    "owner" => owner = Some(source.slice(*identity)),
                    "backing" => backing = Some(source.slice(*identity)),
                    "value" => value = Some(source.slice(*identity)),
                    "policy" => policy = Some(source.slice(*identity)),
                    _ => unreachable!(),
                }
            }
            "extent-bytes" => extent_bytes = Some(parse_storage_natural(source, &field.value)?),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image random-slot parameter",
                ));
            }
        }
    }
    let expected_value = authorize.then_some("initial-auxv-random-object");
    let expected_extent = authorize.then_some(16);
    let expected_policy = if authorize {
        "bounded-linux-auxv-random-before-null"
    } else {
        "bounded-stack-random-slot-qualification-rollback"
    };
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || value != expected_value
        || extent_bytes != expected_extent
        || policy != Some(expected_policy)
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image random-slot transition requires the exact owner, backing, 16-byte initial random object, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_program_headers_addition(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
) -> Result<String, Diagnostic> {
    parse_static_image_stack_program_headers_transition(
        source,
        statement,
        context_name,
        construction_name,
        backing_name,
        true,
    )
}

fn parse_static_image_stack_program_headers_removal(
    source: &SourceText,
    statement: &Statement,
    context_name: &str,
    construction_name: &str,
    backing_name: &str,
) -> Result<String, Diagnostic> {
    parse_static_image_stack_program_headers_transition(
        source,
        statement,
        context_name,
        construction_name,
        backing_name,
        false,
    )
}

fn parse_static_image_stack_program_headers_transition(
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
                "static-image program-header transition must refine one exact stack entry",
            ));
        }
    };
    let Some(Expression::Product { fields, .. }) = items.last() else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image program-header transition requires named parameters",
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
        || !identifier_is(source, &items[6], "program")
        || !identifier_is(source, &items[7], "headers")
        || fields.len() != if add { 4 } else { 3 }
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image program-header transition requires its exact owner, backing, value, and policy",
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
                "static-image program-header parameters must be named identities",
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
                    "unknown static-image program-header parameter",
                ));
            }
        }
    }
    let expected_value = add.then_some("resident-static-elf-program-headers");
    let expected_policy = if add {
        "bounded-linux-auxv-before-null"
    } else {
        "bounded-stack-program-headers-qualification-rollback"
    };
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || value != expected_value
        || policy != Some(expected_policy)
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image program-header transition requires the exact owner, backing, resident metadata value, and policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_entry_fact_removal(
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
                "static-image AT_ENTRY removal must refine one entry-fact stack entry",
            ));
        }
    };
    let [
        receiver,
        static_word,
        image,
        construction,
        remove,
        stack,
        entry,
        fact,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_ENTRY removal requires its exact owner, backing, and rollback policy",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, remove, "remove")
        || !identifier_is(source, stack, "stack")
        || !identifier_is(source, entry, "entry")
        || !identifier_is(source, fact, "fact")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_ENTRY removal requires its exact owner, backing, and rollback policy",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(identity)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image AT_ENTRY removal parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "backing" => backing = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image AT_ENTRY removal parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || policy != Some("bounded-stack-entry-fact-qualification-rollback")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_ENTRY removal requires the exact owner, backing, and rollback policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_pagesz_removal(
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
                "static-image AT_PAGESZ removal must refine one extended stack entry",
            ));
        }
    };
    let [
        receiver,
        static_word,
        image,
        construction,
        remove,
        stack,
        pagesz,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_PAGESZ removal requires its exact owner, backing, and rollback policy",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, remove, "remove")
        || !identifier_is(source, stack, "stack")
        || !identifier_is(source, pagesz, "pagesz")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_PAGESZ removal requires its exact owner, backing, and rollback policy",
        ));
    }
    let mut owner = None;
    let mut backing = None;
    let mut policy = None;
    for field in fields {
        let (Some(label), Expression::Identifier(identity)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image AT_PAGESZ removal parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*identity)),
            "backing" => backing = Some(source.slice(*identity)),
            "policy" => policy = Some(source.slice(*identity)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image AT_PAGESZ removal parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || policy != Some("bounded-stack-pagesz-qualification-rollback")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image AT_PAGESZ removal requires the exact owner, backing, and rollback policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_stack_entry_clearing(
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
                    "static-image stack-entry clearing cannot contain nested operations",
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
                "static-image stack-entry clearing must refine one initialized stack entry",
            ));
        }
    };
    let [
        receiver,
        static_word,
        image,
        construction,
        clear,
        stack,
        entry,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-entry clearing requires its exact owner, backing, and rollback policy",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, clear, "clear")
        || !identifier_is(source, stack, "stack")
        || !identifier_is(source, entry, "entry")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-entry clearing requires its exact owner, backing, and rollback policy",
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
                "static-image stack-entry clearing parameters must be named identities",
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
                    "unknown static-image stack-entry clearing parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name)
        || backing != Some(backing_name)
        || policy != Some("bounded-stack-entry-qualification-rollback")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image stack-entry clearing requires the exact owner, backing, and rollback policy",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_permission_seal(
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
                    "static-image permission sealing cannot contain nested operations",
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
                "static-image permission sealing must refine the live populated construction",
            ));
        }
    };
    let [
        receiver,
        static_word,
        image,
        construction,
        seal,
        permissions,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image permission sealing requires its exact owner and plan",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, seal, "seal")
        || !identifier_is(source, permissions, "permissions")
        || fields.len() != 2
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image permission sealing requires its exact owner and plan",
        ));
    }
    let mut owner = None;
    let mut plan = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image permission-seal parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "plan" => plan = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image permission-seal parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name) || plan != Some("bounded-static-elf-wx-relro") {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image permission sealing requires the exact populated construction and W^X/RELRO plan",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_payload_copy(
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
                    "static-image payload copy cannot contain nested operations",
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
                "static-image payload copy must refine the live zeroed construction",
            ));
        }
    };
    let [
        receiver,
        static_word,
        image,
        construction,
        copy,
        payload,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image payload copy requires its exact owner, source, and plan",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, copy, "copy")
        || !identifier_is(source, payload, "payload")
        || fields.len() != 3
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image payload copy requires its exact owner, source, and plan",
        ));
    }
    let mut owner = None;
    let mut resource = None;
    let mut plan = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image payload-copy parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "source" => resource = Some(source.slice(*value)),
            "plan" => plan = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image payload-copy parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name)
        || resource != Some("retained-static-user-elf")
        || plan != Some("bounded-static-elf-four-loads")
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image payload copy requires the exact zeroed construction, retained static ELF, and four-load plan",
        ));
    }
    Ok(source.slice(name).to_owned())
}

fn parse_static_image_payload_zero<'a>(
    source: &SourceText,
    statement: &'a Statement,
    context_name: &str,
    construction_name: &str,
) -> Result<(String, &'a [Statement]), Diagnostic> {
    let (name, items, span, declarations) = match statement {
        Statement::Implementation {
            name,
            classifier: Expression::Application { items, span },
            declarations,
            ..
        } => (*name, items, *span, declarations.as_slice()),
        Statement::Binding {
            name,
            classifier: None,
            value: Expression::Application { items, span },
        } => (*name, items, *span, &[][..]),
        _ => {
            return Err(translation_edit_diagnostic(
                source,
                statement_span(statement),
                "static-image payload zeroing must refine the live affine construction",
            ));
        }
    };
    let [
        receiver,
        static_word,
        image,
        construction,
        zero,
        payload,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image payload zeroing requires its exact owner and complete extent",
        ));
    };
    if !identifier_is(source, receiver, context_name)
        || !identifier_is(source, static_word, "static")
        || !identifier_is(source, image, "image")
        || !identifier_is(source, construction, "construction")
        || !identifier_is(source, zero, "zero")
        || !identifier_is(source, payload, "payload")
        || fields.len() != 2
    {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image payload zeroing requires its exact owner and complete extent",
        ));
    }
    let mut owner = None;
    let mut extent = None;
    for field in fields {
        let (Some(label), Expression::Identifier(value)) = (field.label, &field.value) else {
            return Err(translation_edit_diagnostic(
                source,
                field.value.span(),
                "static-image payload-zeroing parameters must be named identities",
            ));
        };
        match source.slice(label) {
            "owner" => owner = Some(source.slice(*value)),
            "extent" => extent = Some(source.slice(*value)),
            _ => {
                return Err(translation_edit_diagnostic(
                    source,
                    label,
                    "unknown static-image payload-zeroing parameter",
                ));
            }
        }
    }
    if owner != Some(construction_name) || extent != Some("complete") {
        return Err(translation_edit_diagnostic(
            source,
            span,
            "static-image payload zeroing requires the exact live construction and complete extent",
        ));
    }
    Ok((source.slice(name).to_owned(), declarations))
}
