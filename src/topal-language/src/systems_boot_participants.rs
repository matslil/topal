// TOPAL-SYSTEMS-BOOT-PARTICIPANT-SOURCE-001: only the initial retained scope
// calls this parser. Native entry/controller/register/geometry stays private.
fn begins_boot_participant_initialization(source: &SourceText, statements: &[Statement]) -> bool {
    let Some(Statement::Expression(expression)) = statements.first() else {
        return false;
    };
    let subject = if let Expression::DecisionTable { subject, .. } = expression {
        subject.as_ref()
    } else {
        expression
    };
    matches!(subject, Expression::Application { items, .. }
        if items.get(1).is_some_and(|word|identifier_is(source,word,"boot")))
}

fn boot_participant_diagnostic(
    source: &SourceText,
    span: Span,
    message: impl Into<String>,
) -> Diagnostic {
    source_diagnostic(source, "E-SYSTEMS-BOOT-PARTICIPANTS", span, message)
}

fn analyze_boot_participant_initialization(
    source: &SourceText,
    statements: &[Statement],
    context: &str,
    storage: &CompilerBootstrapStorageDescriptor,
) -> Result<CheckedBootstrapRegionDecision, Diagnostic> {
    let (subject, owner, success, failure_name, failure, span) =
        result_sequence(source, statements, "boot participant initialization")?;
    let Expression::Application { items, .. } = subject else {
        return Err(boot_participant_diagnostic(
            source,
            span,
            "participant initialization requires its boot allocator",
        ));
    };
    let [
        receiver,
        boot,
        participants,
        initialize,
        Expression::Product { fields, .. },
    ] = items.as_slice()
    else {
        return Err(boot_participant_diagnostic(
            source,
            span,
            "participant initialization requires only a provider-selected typed entry",
        ));
    };
    if ![receiver, boot, participants, initialize]
        .into_iter()
        .zip([context, "boot", "participants", "initialize"])
        .all(|(word, expected)| identifier_is(source, word, expected))
        || fields.len() != 1
        || fields[0]
            .label
            .is_none_or(|label| source.slice(label) != "entry")
        || !identifier_is(source, &fields[0].value, "provider-selected")
        || owner == context
    {
        return Err(boot_participant_diagnostic(
            source,
            span,
            "initialization requires its exact allocator, provider-selected entry and distinct arrival owner",
        ));
    }
    let failure_message = translation_failure(
        source,
        failure,
        &failure_name,
        "participant initialization failure must enter fatal",
    )?;
    let body = translation_block(
        source,
        success,
        span,
        "participant initialization requires its consuming success block",
    )?;
    let [
        Statement::Expression(Expression::Application {
            items,
            span: consume_span,
        }),
        marker,
        ordinary @ ..,
    ] = body
    else {
        return Err(boot_participant_diagnostic(
            source,
            span,
            "arrival success requires lifetime retention, observation and boot continuation",
        ));
    };
    if items.len() != 6
        || !items
            .iter()
            .zip([
                context,
                "boot",
                "participants",
                "retain",
                owner.as_str(),
                "for-kernel-lifetime",
            ])
            .all(|(word, expected)| identifier_is(source, word, expected))
    {
        return Err(boot_participant_diagnostic(
            source,
            *consume_span,
            "participant retention requires the exact allocator, live arrival owner and kernel lifetime",
        ));
    }
    let marker = parse_static_image_marker(
        source,
        marker,
        context,
        "TOPAL_BOOT_PARTICIPANTS_ARRIVED",
        "participant retention requires its exact arrival observation",
    )?;
    let mut checked = analyze_translation_sequence(source, ordinary, context, storage)?;
    checked.operations.splice(
        0..0,
        [
            CompilerSystemsOperation::InitializeBootParticipants {
                entry: CompilerBootParticipantEntry::InitialProviderSelected,
                failure_message,
            },
            CompilerSystemsOperation::RetainBootParticipantsForKernelLifetime,
            marker,
        ],
    );
    Ok(checked)
}
