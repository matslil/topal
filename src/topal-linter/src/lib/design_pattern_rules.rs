fn evaluate_topal_boolean_rule(
    rule_source: &str,
    entry_point: &str,
    left: bool,
    right: bool,
) -> Result<bool, String> {
    evaluate_topal_rule_application(rule_source, entry_point, left, right)
}

fn topal_task_declaration_order(
    entry: &CatalogEntry,
    rule_source: &str,
    entry_point: &str,
    source: &SourceText,
    statements: &[Statement],
) -> Result<Vec<RuleFinding>, String> {
    let mut findings = Vec::new();
    visit_topal_task_order(
        entry,
        rule_source,
        entry_point,
        source,
        statements,
        &mut ApplicabilityContext::default(),
        &mut findings,
    )?;
    Ok(findings)
}

fn visit_topal_task_order(
    entry: &CatalogEntry,
    rule_source: &str,
    entry_point: &str,
    source: &SourceText,
    statements: &[Statement],
    context: &mut ApplicabilityContext,
    findings: &mut Vec<RuleFinding>,
) -> Result<(), String> {
    for statement in statements {
        match statement {
            Statement::LanguageSelection {
                version, features, ..
            } => context.select(source, *version, features),
            Statement::Published { declaration, .. } => visit_topal_task_order(
                entry,
                rule_source,
                entry_point,
                source,
                std::slice::from_ref(declaration.as_ref()),
                context,
                findings,
            )?,
            Statement::Implementation { declarations, .. } => {
                let features = task_features(source, declarations, context);
                if is_task_definition(source, declarations)
                    && entry_applies(entry, context.version.as_deref(), &features)?
                {
                    check_topal_task_order(
                        rule_source,
                        entry_point,
                        source,
                        declarations,
                        findings,
                    )?;
                }
                visit_topal_task_order(
                    entry,
                    rule_source,
                    entry_point,
                    source,
                    declarations,
                    &mut context.clone(),
                    findings,
                )?;
            }
            Statement::Function { body, .. }
            | Statement::Generator { body, .. }
            | Statement::Foreach { body, .. } => {
                visit_topal_task_order(
                    entry,
                    rule_source,
                    entry_point,
                    source,
                    body,
                    &mut context.clone(),
                    findings,
                )?;
            }
            Statement::InterfaceImplementation { declarations, .. } => {
                visit_topal_task_order(
                    entry,
                    rule_source,
                    entry_point,
                    source,
                    declarations,
                    &mut context.clone(),
                    findings,
                )?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn check_topal_task_order(
    rule_source: &str,
    entry_point: &str,
    source: &SourceText,
    declarations: &[Statement],
    findings: &mut Vec<RuleFinding>,
) -> Result<(), String> {
    let mut previous = None;
    for declaration in declarations {
        let Some((phase, span, expected)) = declaration_phase(source, declaration) else {
            continue;
        };
        if let Some(previous_phase) = previous
            && !evaluate_topal_phase_rule(rule_source, entry_point, previous_phase, phase)?
        {
            findings.push(RuleFinding {
                span,
                message: "task declaration is outside the recommended lifecycle section",
                suggestion: expected,
                rectification: None,
            });
        }
        previous = Some(phase);
    }
    Ok(())
}

fn evaluate_topal_phase_rule(
    rule_source: &str,
    entry_point: &str,
    previous: u8,
    current: u8,
) -> Result<bool, String> {
    evaluate_topal_rule_application(rule_source, entry_point, previous, current)
}

fn evaluate_topal_rule_application(
    rule_source: &str,
    entry_point: &str,
    left: impl std::fmt::Display,
    right: impl std::fmt::Display,
) -> Result<bool, String> {
    let program = format!("{}\n{left} {entry_point} {right}\n", rule_source.trim_end());
    let value = Session::new()
        .evaluate_source_file(&program, &mut std::io::sink())
        .map_err(|diagnostic| format!("Topal lint rule failed: {diagnostic}"))?;
    match value {
        LanguageValue::Boolean(decision) => Ok(decision),
        other => Err(format!(
            "Topal lint rule `{entry_point}` returned {other}, expected Boolean"
        )),
    }
}

#[derive(Clone, Default)]
struct ApplicabilityContext {
    version: Option<String>,
    selected_features: BTreeSet<String>,
}

impl ApplicabilityContext {
    fn select(&mut self, source: &SourceText, version: Span, features: &[Span]) {
        self.version = Some(source.slice(version).to_owned());
        self.selected_features = features
            .iter()
            .map(|feature| source.slice(*feature).to_owned())
            .collect();
    }
}

fn task_features(
    source: &SourceText,
    declarations: &[Statement],
    context: &ApplicabilityContext,
) -> BTreeSet<String> {
    let mut features = context.selected_features.clone();
    features.insert("task".into());
    if declarations.iter().any(|declaration| match declaration {
        Statement::Function { parameters, .. } | Statement::Generator { parameters, .. } => {
            parameters
                .iter()
                .any(|parameter| source.slice(parameter.classifier) == "MessageContext")
        }
        _ => false,
    }) {
        features.insert("message".into());
    }
    features
}

fn entry_applies(
    entry: &CatalogEntry,
    selected_version: Option<&str>,
    features: &BTreeSet<String>,
) -> Result<bool, String> {
    if entry.language != "topal" {
        return Ok(false);
    }
    let Some(selected_version) = selected_version else {
        return Ok(false);
    };
    let selected = parse_version(selected_version)?;
    let version_matches = if let Some(minimum) = entry.language_versions.strip_prefix(">=") {
        selected >= parse_version(minimum)?
    } else {
        selected == parse_version(&entry.language_versions)?
    };
    if !version_matches {
        return Ok(false);
    }
    if !entry
        .required_features
        .iter()
        .all(|feature| features.contains(feature))
        || entry
            .excluded_features
            .iter()
            .any(|feature| features.contains(feature))
    {
        return Ok(false);
    }
    if entry.status.kind == "obsolete" {
        let cutoff = entry
            .status
            .since_language_version
            .as_deref()
            .ok_or_else(|| {
                format!(
                    "obsolete best-practice `{}` has no language-version cutoff",
                    entry.identity
                )
            })?;
        return Ok(selected < parse_version(cutoff)?);
    }
    Ok(true)
}

fn parse_version(version: &str) -> Result<(u64, u64), String> {
    let value = version
        .strip_prefix('v')
        .ok_or_else(|| format!("invalid catalog language version `{version}`"))?;
    let (major, minor) = value
        .split_once('.')
        .ok_or_else(|| format!("invalid catalog language version `{version}`"))?;
    if minor.contains('.') {
        return Err(format!("invalid catalog language version `{version}`"));
    }
    Ok((
        major
            .parse()
            .map_err(|_| format!("invalid catalog language version `{version}`"))?,
        minor
            .parse()
            .map_err(|_| format!("invalid catalog language version `{version}`"))?,
    ))
}

struct RuleFinding {
    span: Span,
    message: &'static str,
    suggestion: &'static str,
    rectification: Option<SourceEdit>,
}

fn is_task_definition(source: &SourceText, declarations: &[Statement]) -> bool {
    declarations
        .iter()
        .any(|declaration| matches!(declaration, Statement::StateField { .. }))
        && declarations.iter().any(|declaration| {
            matches!(declaration, Statement::Function { name, .. } if source.slice(*name) == "start")
        })
}

fn declaration_phase(
    source: &SourceText,
    declaration: &Statement,
) -> Option<(u8, Span, &'static str)> {
    match declaration {
        Statement::StateField { name, .. } => Some((
            0,
            *name,
            "move the state field before `start` and all message handlers",
        )),
        Statement::Function { name, .. } if source.slice(*name) == "start" => Some((
            1,
            *name,
            "place `start` after state fields and before ordinary handlers",
        )),
        Statement::Function { name, .. } if source.slice(*name) == "terminate" => Some((
            3,
            *name,
            "place `terminate` after every ordinary message handler",
        )),
        Statement::Function { name, .. } | Statement::Generator { name, .. } => Some((
            2,
            *name,
            "place ordinary handlers after `start` and before `terminate`",
        )),
        _ => None,
    }
}

fn shared_syntax_diagnostic(source: &SourceText, diagnostic: &SyntaxDiagnostic) -> Diagnostic {
    let position = source.position(diagnostic.span.start);
    Diagnostic::error(
        diagnostic.code,
        position.line,
        position.column,
        &diagnostic.message,
    )
    .with_source_span(diagnostic.span)
    .with_source_excerpt(
        source
            .as_str()
            .lines()
            .nth(position.line - 1)
            .map(str::to_owned),
        diagnostic.span.end.saturating_sub(diagnostic.span.start),
    )
}

fn source_diagnostic(
    text: &str,
    span: Span,
    code: impl Into<String>,
    message: impl Into<String>,
) -> Diagnostic {
    let (line, column) = byte_position(text, span.start);
    Diagnostic::error(code, line, column, message)
        .with_source_span(span)
        .with_source_excerpt(
            text.lines().nth(line - 1).map(str::to_owned),
            span.end.saturating_sub(span.start),
        )
}

fn byte_position(text: &str, offset: usize) -> (usize, usize) {
    let prefix = &text[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix
        .rsplit_once('\n')
        .map_or(prefix, |(_, tail)| tail)
        .chars()
        .count()
        + 1;
    (line, column)
}
