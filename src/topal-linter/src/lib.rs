use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::Serialize;
use topal_best_practices::{Catalog, CatalogEntry};
use topal_language::{Session, Value as LanguageValue};
use topal_source::{Diagnostic, Severity, SourceText, Span};
use topal_syntax::{
    CallableKind, DecisionMatcher, DiagnosticControlKind, Expression, Statement, SyntaxDiagnostic,
    lex, parse,
};

const MAX_RULE_SOURCE_BYTES: usize = 16 * 1024;
const MAX_RULE_EXPRESSION_NODES: usize = 128;
const MAX_RULE_INTEGER_BYTES: usize = 64;

const USAGE: &str = "usage: topal-lint [OPTIONS] SOURCE...\n\
  --check-rule PATH              validate a Topal lint-variant rule module\n\
  --entry-point NAME             rule function for --check-rule (default: rule)\n\
  --list                         list selected best-practices\n\
  --explain ID                   explain one stable identity\n\
  --catalog PATH                 explicitly load an external catalog\n\
  --enable SELECTOR              enable an identity, namespace:PATH, or tag:ID\n\
  --disable SELECTOR             disable an identity, namespace:PATH, or tag:ID\n\
  --severity SELECTOR=LEVEL      set warning, error, or off\n\
  --fix                          apply eligible automatic rectifications\n\
  --format terminal|json|sarif   select finding presentation";

fn parse_severity(value: &str) -> Result<Option<Severity>, String> {
    match value {
        "warning" => Ok(Some(Severity::Warning)),
        "error" => Ok(Some(Severity::Error)),
        "off" => Ok(None),
        _ => Err(format!("unknown lint severity `{value}`")),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Format {
    Terminal,
    Json,
    Sarif,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Selector {
    Identity(String),
    Namespace(String),
    Tag(String),
}

impl Selector {
    fn parse(text: &str) -> Result<Self, String> {
        if let Some(value) = text.strip_prefix("namespace:") {
            require_selector(value).map(Self::Namespace)
        } else if let Some(value) = text.strip_prefix("tag:") {
            require_selector(value).map(Self::Tag)
        } else {
            require_selector(text).map(Self::Identity)
        }
    }

    fn matches(&self, entry: &CatalogEntry) -> bool {
        match self {
            Self::Identity(identity) => entry.identity == *identity,
            Self::Namespace(namespace) => {
                entry.identity == *namespace
                    || entry
                        .identity
                        .strip_prefix(namespace)
                        .is_some_and(|rest| rest.starts_with(' '))
            }
            Self::Tag(tag) => entry.tags.iter().any(|candidate| candidate == tag),
        }
    }

    const fn rank(&self) -> u8 {
        match self {
            Self::Tag(_) => 0,
            Self::Namespace(_) => 1,
            Self::Identity(_) => 2,
        }
    }
}

fn require_selector(value: &str) -> Result<String, String> {
    if value.trim().is_empty() {
        Err("lint selector cannot be empty".into())
    } else {
        Ok(value.trim().into())
    }
}

#[derive(Clone, Debug)]
struct Override {
    selector: Selector,
    enabled: Option<bool>,
    severity: SeveritySetting,
    order: usize,
}

#[derive(Clone, Copy, Debug, Default)]
enum SeveritySetting {
    #[default]
    Keep,
    Set(Severity),
    Off,
}

#[derive(Default)]
struct Options {
    help: bool,
    list: bool,
    explain: Option<String>,
    catalogs: Vec<PathBuf>,
    overrides: Vec<Override>,
    format: Option<Format>,
    fix: bool,
    sources: Vec<PathBuf>,
    check_rule: Option<PathBuf>,
    entry_point: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Policy {
    enabled: bool,
    severity: Severity,
}

#[derive(Serialize)]
struct JsonDiagnostic<'a> {
    severity: &'static str,
    code: &'a str,
    message: &'a str,
    source: &'a str,
    line: usize,
    column: usize,
    end_line: usize,
    end_column: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    best_practice: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    best_practice_version: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rule_version: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    checkability: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    confidence: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    help: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rectification: Option<JsonRectification<'a>>,
}

#[derive(Serialize)]
struct JsonRectification<'a> {
    kind: &'static str,
    message: &'a str,
}

impl<'a> JsonDiagnostic<'a> {
    fn from_diagnostic(diagnostic: &'a Diagnostic, source: &'a str) -> Self {
        let best_practice = diagnostic.best_practice.as_ref();
        Self {
            severity: diagnostic.severity.label(),
            code: &diagnostic.code,
            message: &diagnostic.message,
            source,
            line: diagnostic.line,
            column: diagnostic.column,
            end_line: diagnostic.line,
            end_column: diagnostic.column + diagnostic.marker_width,
            best_practice: best_practice.map(|context| context.identity.as_str()),
            best_practice_version: best_practice.map(|context| context.version.as_str()),
            rule_version: best_practice.map(|context| context.rule_version.as_str()),
            checkability: best_practice.and_then(|context| context.checkability.as_deref()),
            confidence: best_practice.and_then(|context| context.confidence.as_deref()),
            help: diagnostic.help.as_deref(),
            rectification: best_practice.and_then(|context| {
                context
                    .suggestion
                    .as_deref()
                    .map(|message| JsonRectification {
                        kind: "suggestion",
                        message,
                    })
            }),
        }
    }
}

struct Emitter {
    format: Format,
    sarif: Vec<(String, Diagnostic)>,
}

impl Emitter {
    const fn new(format: Format) -> Self {
        Self {
            format,
            sarif: Vec::new(),
        }
    }

    fn emit(&mut self, diagnostic: &Diagnostic, path: &Path) -> Result<(), String> {
        let source = path.to_str().unwrap_or("<source>");
        match self.format {
            Format::Terminal => eprintln!("{}", diagnostic.render(source)),
            Format::Json => println!(
                "{}",
                serde_json::to_string(&JsonDiagnostic::from_diagnostic(diagnostic, source))
                    .map_err(|error| error.to_string())?
            ),
            Format::Sarif => self.sarif.push((source.to_owned(), diagnostic.clone())),
        }
        Ok(())
    }

    fn finish(self) -> Result<(), String> {
        if self.format != Format::Sarif {
            return Ok(());
        }
        let results = self
            .sarif
            .iter()
            .map(|(source, diagnostic)| sarif_result(source, diagnostic))
            .collect::<Vec<_>>();
        let report = serde_json::json!({
            "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
            "version": "2.1.0",
            "runs": [{
                "tool": { "driver": { "name": "topal-lint" } },
                "results": results
            }]
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?
        );
        Ok(())
    }
}

fn sarif_result(source: &str, diagnostic: &Diagnostic) -> serde_json::Value {
    let mut properties = serde_json::Map::new();
    if let Some(best_practice) = &diagnostic.best_practice {
        properties.insert(
            "bestPractice".into(),
            serde_json::Value::String(best_practice.identity.clone()),
        );
        properties.insert(
            "bestPracticeVersion".into(),
            serde_json::Value::String(best_practice.version.clone()),
        );
        properties.insert(
            "ruleVersion".into(),
            serde_json::Value::String(best_practice.rule_version.clone()),
        );
        if let Some(checkability) = &best_practice.checkability {
            properties.insert(
                "checkability".into(),
                serde_json::Value::String(checkability.to_string()),
            );
        }
        if let Some(confidence) = &best_practice.confidence {
            properties.insert(
                "confidence".into(),
                serde_json::Value::String(confidence.to_string()),
            );
        }
        if let Some(suggestion) = &best_practice.suggestion {
            properties.insert(
                "rectification".into(),
                serde_json::json!({ "kind": "suggestion", "message": suggestion }),
            );
        }
    }
    let mut result = serde_json::json!({
        "ruleId": diagnostic.code,
        "level": diagnostic.severity.label(),
        "message": { "text": diagnostic.message },
        "locations": [{
            "physicalLocation": {
                "artifactLocation": { "uri": source },
                "region": {
                    "startLine": diagnostic.line,
                    "startColumn": diagnostic.column,
                    "endLine": diagnostic.line,
                    "endColumn": diagnostic.column + diagnostic.marker_width
                }
            }
        }],
        "properties": properties
    });
    if let Some(help) = &diagnostic.help {
        result["help"] = serde_json::json!({ "text": help });
    }
    result
}

#[must_use]
pub fn main_entry() -> ExitCode {
    match run(env::args().skip(1)) {
        Ok(has_error) => ExitCode::from(u8::from(has_error)),
        Err(message) => {
            eprintln!("error: {message}\n\n{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn run(arguments: impl Iterator<Item = String>) -> Result<bool, String> {
    let options = parse_options(arguments)?;
    if options.help {
        return Ok(false);
    }
    if let Some(path) = &options.check_rule {
        if options.list
            || options.explain.is_some()
            || !options.sources.is_empty()
            || !options.catalogs.is_empty()
            || !options.overrides.is_empty()
            || options.fix
        {
            return Err(
                "--check-rule cannot be combined with catalog queries or source linting".into(),
            );
        }
        validate_rule_module(path, options.entry_point.as_deref().unwrap_or("rule"))?;
        return Ok(false);
    }
    if options.entry_point.is_some() {
        return Err("--entry-point requires --check-rule".into());
    }
    let catalog = load_catalogs(&options.catalogs)?;
    validate_overrides(&catalog, &options.overrides)?;
    if options.list {
        list_entries(&catalog, &options.overrides)?;
    }
    if let Some(identity) = &options.explain {
        explain_entry(&catalog, identity, &options.overrides)?;
    }
    if options.sources.is_empty() {
        if options.list || options.explain.is_some() {
            return Ok(false);
        }
        return Err("at least one source path is required".into());
    }
    let format = options.format.unwrap_or(Format::Terminal);
    let mut emitter = Emitter::new(format);
    let mut has_error = false;
    for path in &options.sources {
        has_error |= lint_source(
            path,
            &catalog,
            &options.overrides,
            options.fix,
            &mut emitter,
        )?;
    }
    emitter.finish()?;
    Ok(has_error)
}

fn parse_options(arguments: impl Iterator<Item = String>) -> Result<Options, String> {
    let mut options = Options::default();
    let mut arguments = arguments.peekable();
    let mut order = 0;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--check-rule" => {
                options.check_rule = Some(next_value(&mut arguments, "--check-rule")?.into());
            }
            "--entry-point" => {
                options.entry_point = Some(next_value(&mut arguments, "--entry-point")?);
            }
            "--list" => options.list = true,
            "--explain" => options.explain = Some(next_value(&mut arguments, "--explain")?),
            "--catalog" => options
                .catalogs
                .push(next_value(&mut arguments, "--catalog")?.into()),
            "--enable" | "--disable" => {
                let selector = Selector::parse(&next_value(&mut arguments, &argument)?)?;
                options.overrides.push(Override {
                    selector,
                    enabled: Some(argument == "--enable"),
                    severity: SeveritySetting::Keep,
                    order,
                });
                order += 1;
            }
            "--severity" => {
                let setting = next_value(&mut arguments, "--severity")?;
                let (selector, severity) = setting
                    .rsplit_once('=')
                    .ok_or_else(|| "--severity requires SELECTOR=LEVEL".to_string())?;
                options.overrides.push(Override {
                    selector: Selector::parse(selector)?,
                    enabled: None,
                    severity: match parse_severity(severity)? {
                        Some(severity) => SeveritySetting::Set(severity),
                        None => SeveritySetting::Off,
                    },
                    order,
                });
                order += 1;
            }
            "--fix" => options.fix = true,
            "--format" => {
                options.format = Some(match next_value(&mut arguments, "--format")?.as_str() {
                    "terminal" => Format::Terminal,
                    "json" => Format::Json,
                    "sarif" => Format::Sarif,
                    value => return Err(format!("unknown output format `{value}`")),
                });
            }
            "--help" | "-h" => {
                println!("{USAGE}");
                options.help = true;
                return Ok(options);
            }
            _ if argument.starts_with('-') => return Err(format!("unknown option `{argument}`")),
            _ => options.sources.push(argument.into()),
        }
    }
    Ok(options)
}

fn validate_rule_module(path: &Path, entry_point: &str) -> Result<(), String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("cannot read lint rule {}: {error}", path.display()))?;
    validate_rule_text(&text, &path.display().to_string(), entry_point, None)
}

fn validate_rule_text(
    text: &str,
    source_name: &str,
    entry_point: &str,
    expected_parameters: Option<&[&str]>,
) -> Result<(), String> {
    if text.len() > MAX_RULE_SOURCE_BYTES {
        return Err(format!(
            "L-RULE-RESOURCE: lint rule source exceeds {MAX_RULE_SOURCE_BYTES} bytes"
        ));
    }
    let source =
        SourceText::new(text).map_err(|error| format!("{}: {}", error.code, error.message))?;
    let lexed = lex(&source);
    let parsed = parse(&source, &lexed);
    if let Some(diagnostic) = lexed.diagnostics.first().or(parsed.diagnostics.first()) {
        let position = source.position(diagnostic.span.start);
        return Err(format!(
            "{}:{}:{}: {}: {}",
            source_name, position.line, position.column, diagnostic.code, diagnostic.message
        ));
    }
    let Some(Statement::LanguageSelection {
        version, features, ..
    }) = parsed.statements.first()
    else {
        return Err("L-RULE-LANGUAGE: lint rule modules begin with `use language`".into());
    };
    if source.slice(*version) != "v0.1" {
        return Err(format!(
            "L-RULE-VERSION: unsupported lint rule language version `{}`",
            source.slice(*version)
        ));
    }
    let selected: BTreeSet<_> = features.iter().map(|span| source.slice(*span)).collect();
    if !selected.contains("lint") {
        return Err("L-RULE-VARIANT: lint rule modules select `features is ( lint )`".into());
    }
    if selected.contains("debug") {
        return Err("L-RULE-AUTHORITY: lint rule modules cannot select the `debug` feature".into());
    }
    validate_contained_rule(&source, &parsed.statements, entry_point)?;
    let mut found = None;
    visit_rule_functions(
        &source,
        &parsed.statements[1..],
        entry_point,
        expected_parameters,
        &mut found,
    )?;
    match found {
        Some((true, true)) => Ok(()),
        Some((false, _)) => Err(format!(
            "L-RULE-STATIC: lint rule entry point `{entry_point}` must be static"
        )),
        Some((true, false)) => Err(format!(
            "L-RULE-SIGNATURE: lint rule entry point `{entry_point}` does not match view parameters and Boolean result"
        )),
        None => Err(format!(
            "L-RULE-ENTRY: lint rule module does not declare entry point `{entry_point}`"
        )),
    }
}

fn validate_contained_rule(
    source: &SourceText,
    statements: &[Statement],
    entry_point: &str,
) -> Result<(), String> {
    let [
        Statement::LanguageSelection { .. },
        Statement::Function { name, body, .. },
    ] = statements
    else {
        return Err(
            "L-RULE-CONTAINMENT: a lint rule module contains only its language selection and entry function"
                .into(),
        );
    };
    if source.slice(*name) != entry_point {
        return Err(format!(
            "L-RULE-ENTRY: lint rule module does not declare entry point `{entry_point}`"
        ));
    }
    let mut remaining = MAX_RULE_EXPRESSION_NODES;
    for statement in body {
        let expression = match statement {
            Statement::Expression(expression) => expression,
            Statement::Return { value, .. } => value,
            _ => {
                return Err(
                    "L-RULE-CONTAINMENT: lint rule bodies contain only pure result expressions"
                        .into(),
                );
            }
        };
        validate_rule_expression(source, expression, &mut remaining)?;
    }
    Ok(())
}

fn validate_rule_expression(
    source: &SourceText,
    expression: &Expression,
    remaining: &mut usize,
) -> Result<(), String> {
    *remaining = remaining.checked_sub(1).ok_or_else(|| {
        format!("L-RULE-RESOURCE: lint rule exceeds {MAX_RULE_EXPRESSION_NODES} expression nodes")
    })?;
    match expression {
        Expression::Unit(_)
        | Expression::Boolean(_)
        | Expression::Identifier(_)
        | Expression::Callable {
            kind:
                CallableKind::Equal
                | CallableKind::NotEqual
                | CallableKind::Less
                | CallableKind::Greater
                | CallableKind::LessEqual
                | CallableKind::Compare
                | CallableKind::GreaterEqual,
            ..
        } => Ok(()),
        Expression::Integer(span) if source.slice(*span).len() <= MAX_RULE_INTEGER_BYTES => Ok(()),
        Expression::Integer(_) => Err(format!(
            "L-RULE-RESOURCE: lint rule integer literal exceeds {MAX_RULE_INTEGER_BYTES} bytes"
        )),
        Expression::Application { items, .. } => {
            for item in items {
                validate_rule_expression(source, item, remaining)?;
            }
            Ok(())
        }
        Expression::DecisionTable { subject, rules, .. } => {
            validate_rule_expression(source, subject, remaining)?;
            for rule in rules {
                if !matches!(
                    rule.matcher,
                    DecisionMatcher::Boolean { .. } | DecisionMatcher::Otherwise(_)
                ) {
                    return Err(
                        "L-RULE-CONTAINMENT: this lint view supports only Boolean decisions".into(),
                    );
                }
                validate_rule_expression(source, &rule.action, remaining)?;
            }
            Ok(())
        }
        _ => Err(
            "L-RULE-CONTAINMENT: lint rule expression is outside the bounded pure subset".into(),
        ),
    }
}

fn visit_rule_functions(
    source: &SourceText,
    statements: &[Statement],
    entry_point: &str,
    expected_parameters: Option<&[&str]>,
    found: &mut Option<(bool, bool)>,
) -> Result<(), String> {
    for statement in statements {
        match statement {
            Statement::Published { declaration, .. } => visit_rule_functions(
                source,
                std::slice::from_ref(declaration.as_ref()),
                entry_point,
                expected_parameters,
                found,
            )?,
            Statement::Function {
                name,
                is_static,
                parameters,
                result,
                ..
            } if source.slice(*name) == entry_point => {
                if found.is_some() {
                    return Err(format!(
                        "L-RULE-ENTRY: lint rule entry point `{entry_point}` is ambiguous"
                    ));
                }
                let signature_matches = expected_parameters.is_none_or(|expected| {
                    parameters.len() == expected.len()
                        && parameters
                            .iter()
                            .zip(expected)
                            .all(|(parameter, expected)| {
                                source.slice(parameter.classifier) == *expected
                            })
                        && source.slice(*result) == "Boolean"
                });
                *found = Some((*is_static, signature_matches));
            }
            _ => {}
        }
    }
    Ok(())
}

fn next_value(
    arguments: &mut impl Iterator<Item = String>,
    option: &str,
) -> Result<String, String> {
    arguments
        .next()
        .ok_or_else(|| format!("{option} requires a value"))
}

fn load_catalogs(paths: &[PathBuf]) -> Result<Catalog, String> {
    let mut catalog = Catalog::builtin();
    for path in paths {
        let source = fs::read_to_string(path)
            .map_err(|error| format!("cannot read catalog {}: {error}", path.display()))?;
        catalog.merge(Catalog::from_json(&source)?)?;
    }
    Ok(catalog)
}

fn validate_overrides(catalog: &Catalog, overrides: &[Override]) -> Result<(), String> {
    for setting in overrides {
        if !catalog
            .entries
            .iter()
            .any(|entry| setting.selector.matches(entry))
        {
            return Err(format!(
                "lint selector matches no best-practice: {:?}",
                setting.selector
            ));
        }
    }
    Ok(())
}

fn policy(entry: &CatalogEntry, overrides: &[Override]) -> Result<Policy, String> {
    let default_severity = parse_severity(&entry.default_severity)?
        .ok_or_else(|| "catalog default severity cannot be off".to_string())?;
    let mut policy = Policy {
        enabled: entry.default_enabled && entry.status.kind != "deprecated",
        severity: default_severity,
    };
    let mut matching: Vec<_> = overrides
        .iter()
        .filter(|setting| setting.selector.matches(entry))
        .collect();
    matching.sort_by_key(|setting| (setting.selector.rank(), setting.order));
    for setting in matching {
        if let Some(enabled) = setting.enabled {
            policy.enabled = enabled;
        }
        match setting.severity {
            SeveritySetting::Keep => {}
            SeveritySetting::Set(severity) => policy.severity = severity,
            SeveritySetting::Off => policy.enabled = false,
        }
    }
    Ok(policy)
}

fn list_entries(catalog: &Catalog, overrides: &[Override]) -> Result<(), String> {
    for entry in &catalog.entries {
        let policy = policy(entry, overrides)?;
        println!(
            "{}\t{}\t{}\t{}\t{}",
            entry.identity,
            entry.version,
            entry.status.kind,
            if policy.enabled {
                "enabled"
            } else {
                "disabled"
            },
            policy.severity.label()
        );
    }
    Ok(())
}

fn explain_entry(catalog: &Catalog, identity: &str, overrides: &[Override]) -> Result<(), String> {
    let entry = catalog
        .entries
        .iter()
        .find(|entry| entry.identity == identity)
        .ok_or_else(|| format!("unknown best-practice identity `{identity}`"))?;
    let policy = policy(entry, overrides)?;
    println!("identity: {}", entry.identity);
    println!("version: {}", entry.version);
    println!("status: {}", entry.status.kind);
    if let Some(version) = &entry.status.since_language_version {
        println!("obsolete since: {version}");
    }
    if let Some(explanation) = &entry.status.explanation {
        println!("status explanation: {explanation}");
    }
    if let Some(replacement) = &entry.status.replacement {
        println!("replacement: {}", replacement.join(" "));
    }
    println!("class: {}", entry.class);
    println!("recommendation: {}", entry.recommendation);
    println!("checkability: {}", entry.checkability);
    if let Some(confidence) = &entry.confidence {
        println!("confidence: {confidence}");
    }
    println!("enabled: {}", policy.enabled);
    println!("severity: {}", policy.severity.label());
    println!("language: {} {}", entry.language, entry.language_versions);
    println!(
        "required features: {}",
        display_values(&entry.required_features)
    );
    println!(
        "excluded features: {}",
        display_values(&entry.excluded_features)
    );
    println!("tags: {}", entry.tags.join(", "));
    if let Some(rule) = &entry.lint_rule {
        println!(
            "rule: {} {} {} {} {} {}",
            rule.engine,
            rule.entry_point,
            rule.version,
            rule.stage,
            rule.view,
            rule.diagnostic_code
        );
    } else {
        println!("rule: none");
    }
    Ok(())
}

fn display_values(values: &[String]) -> String {
    if values.is_empty() {
        "none".into()
    } else {
        values.join(", ")
    }
}

fn lint_source(
    path: &Path,
    catalog: &Catalog,
    overrides: &[Override],
    fix: bool,
    emitter: &mut Emitter,
) -> Result<bool, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("cannot read source {}: {error}", path.display()))?;
    let mut report = analyze_text(&text, catalog, overrides)?;
    if fix && report.rectifications.iter().any(SourceEdit::is_eligible) {
        let changed = apply_source_edits(&text, &report.rectifications)?;
        require_clean_syntax(&changed)?;
        report = analyze_text(&changed, catalog, overrides)?;
        if report.rectifications.iter().any(SourceEdit::is_eligible) {
            return Err(format!(
                "automatic rectification did not converge for {}",
                path.display()
            ));
        }
        replace_source_atomically(path, &changed)?;
    }
    for diagnostic in &report.diagnostics {
        emitter.emit(diagnostic, path)?;
    }
    Ok(report.has_errors)
}

/// Apply eligible rectifications transactionally in memory.
///
/// # Errors
///
/// Returns an error when a span is outside the source, splits a UTF-8 scalar,
/// or overlaps another eligible edit. Review-required edits are not applied.
pub fn apply_source_edits(text: &str, edits: &[SourceEdit]) -> Result<String, String> {
    let mut eligible = edits
        .iter()
        .filter(|edit| edit.is_eligible())
        .collect::<Vec<_>>();
    eligible.sort_by_key(|edit| (edit.span.start, edit.span.end));
    let mut previous_end = 0;
    for edit in &eligible {
        if edit.span.start > edit.span.end
            || edit.span.end > text.len()
            || !text.is_char_boundary(edit.span.start)
            || !text.is_char_boundary(edit.span.end)
        {
            return Err("automatic rectification has an invalid source span".into());
        }
        if edit.span.start < previous_end {
            return Err("automatic rectifications overlap".into());
        }
        previous_end = edit.span.end;
    }
    let mut changed = text.to_owned();
    for edit in eligible.into_iter().rev() {
        changed.replace_range(edit.span.start..edit.span.end, &edit.replacement);
    }
    Ok(changed)
}

fn require_clean_syntax(text: &str) -> Result<(), String> {
    let source = SourceText::new(text)
        .map_err(|error| format!("rectified source is invalid: {}", error.message))?;
    let lexed = lex(&source);
    let parsed = parse(&source, &lexed);
    if let Some(diagnostic) = lexed.diagnostics.first().or(parsed.diagnostics.first()) {
        return Err(format!(
            "rectified source failed reparse with {}: {}",
            diagnostic.code, diagnostic.message
        ));
    }
    Ok(())
}

fn replace_source_atomically(path: &Path, text: &str) -> Result<(), String> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("source path {} has no file name", path.display()))?;
    let temporary = path.with_file_name(format!(".{file_name}.topal-fix-{}", std::process::id()));
    let metadata = fs::metadata(path)
        .map_err(|error| format!("cannot inspect source {}: {error}", path.display()))?;
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(|error| format!("cannot create rectification transaction: {error}"))?;
    let result = (|| {
        output
            .write_all(text.as_bytes())
            .map_err(|error| format!("cannot write rectified source: {error}"))?;
        output
            .sync_all()
            .map_err(|error| format!("cannot synchronize rectified source: {error}"))?;
        fs::set_permissions(&temporary, metadata.permissions())
            .map_err(|error| format!("cannot preserve source permissions: {error}"))?;
        fs::rename(&temporary, path)
            .map_err(|error| format!("cannot replace source {}: {error}", path.display()))
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

/// Diagnostics produced by one in-memory linter analysis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LintReport {
    pub diagnostics: Vec<Diagnostic>,
    pub has_errors: bool,
    pub rectifications: Vec<SourceEdit>,
}

/// A half-open replacement in normalized UTF-8 source coordinates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceEdit {
    pub span: Span,
    pub replacement: String,
    pub safety: RectificationSafety,
}

impl SourceEdit {
    const fn is_eligible(&self) -> bool {
        !matches!(self.safety, RectificationSafety::ReviewRequired)
    }
}

/// Evidence level controlling whether `--fix` may apply an edit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RectificationSafety {
    PresentationOnly,
    SyntaxPreserving,
    SemanticsProven,
    ReviewRequired,
}

/// One ordered lint-policy control shared by command, editor, and embedding
/// adapters. Selectors use the same identity, `namespace:`, and `tag:` forms
/// as the command-line interface.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LintControl {
    Enable(String),
    Disable(String),
    Severity {
        selector: String,
        severity: Severity,
    },
    Off(String),
}

/// A contained lint engine with a catalog assembled explicitly by its host.
/// Catalog JSON is parsed and authenticated before any source is inspected;
/// lint rules receive no access to the paths or storage used by the host.
pub struct LintEngine {
    catalog: Catalog,
}

impl Default for LintEngine {
    fn default() -> Self {
        Self::builtin()
    }
}

impl LintEngine {
    #[must_use]
    pub fn builtin() -> Self {
        Self {
            catalog: Catalog::builtin(),
        }
    }

    /// Add one explicitly supplied external catalog projection.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed or unsupported catalogs, unauthenticated
    /// rule attachments, or stable identities already owned by another loaded
    /// catalog.
    pub fn add_catalog_json(&mut self, source: &str) -> Result<(), String> {
        self.catalog.merge(Catalog::from_json(source)?)
    }

    /// Lint source using this engine's catalogs and ordered controls.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed or unknown selectors, or when a selected
    /// contained rule cannot be admitted or executed.
    pub fn lint_text(&self, text: &str, controls: &[LintControl]) -> Result<LintReport, String> {
        let overrides = policy_overrides(controls)?;
        validate_overrides(&self.catalog, &overrides)?;
        analyze_text(text, &self.catalog, &overrides)
    }
}

/// Lint normalized in-memory Topal source with explicitly enabled built-in
/// best-practices. Proposed entries remain disabled unless named here.
///
/// # Errors
///
/// Returns an error when a requested identity is unknown or a selected rule
/// cannot be admitted or executed in its contained view.
pub fn lint_text(text: &str, enabled_identities: &[&str]) -> Result<LintReport, String> {
    let controls = enabled_identities
        .iter()
        .map(|identity| LintControl::Enable((*identity).to_owned()))
        .collect::<Vec<_>>();
    lint_text_with_controls(text, &controls)
}

/// Lint normalized in-memory Topal source using ordered selector controls.
///
/// # Errors
///
/// Returns an error when a selector is malformed or unknown, or when a
/// selected rule cannot be admitted or executed in its contained view.
pub fn lint_text_with_controls(text: &str, controls: &[LintControl]) -> Result<LintReport, String> {
    LintEngine::builtin().lint_text(text, controls)
}

fn policy_overrides(controls: &[LintControl]) -> Result<Vec<Override>, String> {
    controls
        .iter()
        .enumerate()
        .map(|(order, control)| match control {
            LintControl::Enable(selector) => Ok(Override {
                selector: Selector::parse(selector)?,
                enabled: Some(true),
                severity: SeveritySetting::Keep,
                order,
            }),
            LintControl::Disable(selector) => Ok(Override {
                selector: Selector::parse(selector)?,
                enabled: Some(false),
                severity: SeveritySetting::Keep,
                order,
            }),
            LintControl::Severity { selector, severity } => Ok(Override {
                selector: Selector::parse(selector)?,
                enabled: None,
                severity: SeveritySetting::Set(*severity),
                order,
            }),
            LintControl::Off(selector) => Ok(Override {
                selector: Selector::parse(selector)?,
                enabled: None,
                severity: SeveritySetting::Off,
                order,
            }),
        })
        .collect()
}

fn analyze_text(
    text: &str,
    catalog: &Catalog,
    overrides: &[Override],
) -> Result<LintReport, String> {
    let mut findings = Vec::new();
    let source = match normalized_lint_source(text) {
        Ok(source) => source,
        Err(report) => return Ok(report),
    };
    let lexed = lex(&source);
    let parsed = parse(&source, &lexed);
    let mut seen = BTreeSet::new();
    let mut has_error = false;
    let mut rectifications = Vec::new();
    let mut diagnostics = Vec::new();
    diagnostics.extend(lexed.diagnostics.iter());
    diagnostics.extend(parsed.diagnostics.iter());
    for diagnostic in diagnostics {
        if seen.insert((diagnostic.code, diagnostic.span.start, diagnostic.span.end)) {
            findings.push(shared_syntax_diagnostic(&source, diagnostic));
            has_error = true;
        }
    }
    if seen.is_empty() {
        for entry in &catalog.entries {
            let entry_policy = policy(entry, overrides)?;
            if !entry_policy.enabled {
                continue;
            }
            if let Some(rule) = &entry.lint_rule {
                let rule_findings = match rule.engine.as_str() {
                    "topal" => topal_rule(entry, &source, &parsed.statements)?,
                    other => {
                        return Err(format!(
                            "lint rule {} uses unsupported engine `{other}`",
                            entry.identity
                        ));
                    }
                };
                for rule_finding in rule_findings {
                    if diagnostic_suppresses(
                        &source,
                        &parsed.statements,
                        &entry.identity,
                        rule_finding.span,
                    ) {
                        continue;
                    }
                    if let Some(rectification) = rule_finding.rectification.clone() {
                        rectifications.push(rectification);
                    }
                    let position = source.position(rule_finding.span.start);
                    let diagnostic = match entry_policy.severity {
                        Severity::Warning => Diagnostic::warning(
                            &rule.diagnostic_code,
                            position.line,
                            position.column,
                            rule_finding.message,
                        ),
                        Severity::Error => Diagnostic::error(
                            &rule.diagnostic_code,
                            position.line,
                            position.column,
                            rule_finding.message,
                        ),
                    }
                    .with_source_span(rule_finding.span)
                    .with_source_excerpt(
                        source
                            .as_str()
                            .lines()
                            .nth(position.line - 1)
                            .map(str::to_owned),
                        source.slice(rule_finding.span).chars().count(),
                    )
                    .with_help(&entry.recommendation)
                    .with_best_practice_suggestion(
                        &entry.identity,
                        &entry.version,
                        &rule.version,
                        rule_finding.suggestion,
                    )
                    .with_best_practice_checkability(
                        &entry.checkability,
                        entry.confidence.as_deref(),
                    );
                    findings.push(diagnostic);
                    has_error |= entry_policy.severity == Severity::Error;
                }
            }
        }
    }
    Ok(LintReport {
        diagnostics: findings,
        has_errors: has_error,
        rectifications,
    })
}

fn normalized_lint_source(text: &str) -> Result<SourceText, LintReport> {
    SourceText::new(text).map_err(|error| LintReport {
        diagnostics: vec![source_diagnostic(
            text,
            error.span,
            error.code,
            error.message,
        )],
        has_errors: true,
        rectifications: Vec::new(),
    })
}

fn diagnostic_suppresses(
    source: &SourceText,
    statements: &[Statement],
    identity: &str,
    finding: Span,
) -> bool {
    diagnostic_suppresses_in_context(source, statements, identity, finding, &[])
}

fn diagnostic_suppresses_in_context(
    source: &SourceText,
    statements: &[Statement],
    identity: &str,
    finding: Span,
    inherited: &[String],
) -> bool {
    let mut stack = inherited.to_vec();
    let mut pending = None::<String>;
    for statement in statements {
        if let Statement::DiagnosticControl {
            operation,
            identity: components,
            ..
        } = statement
        {
            let controlled = components
                .iter()
                .map(|component| source.slice(*component))
                .collect::<Vec<_>>()
                .join(" ");
            match operation {
                DiagnosticControlKind::DisableNext => pending = Some(controlled),
                DiagnosticControlKind::Push => stack.push(controlled),
                DiagnosticControlKind::Pop => {
                    stack.pop();
                }
            }
            continue;
        }

        let span = lint_statement_span(statement);
        let covers_finding = span.start <= finding.start && finding.end <= span.end;
        let suppressed =
            stack.iter().any(|active| active == identity) || pending.as_deref() == Some(identity);
        pending = None;
        if covers_finding && suppressed {
            return true;
        }
        if covers_finding
            && nested_statements(statement).is_some_and(|nested| {
                diagnostic_suppresses_in_context(source, nested, identity, finding, &stack)
            })
        {
            return true;
        }
    }
    false
}

fn nested_statements(statement: &Statement) -> Option<&[Statement]> {
    match statement {
        Statement::Published { declaration, .. } => Some(std::slice::from_ref(declaration)),
        Statement::Implementation { declarations, .. }
        | Statement::InterfaceImplementation { declarations, .. } => Some(declarations),
        Statement::Function { body, .. }
        | Statement::Generator { body, .. }
        | Statement::Foreach { body, .. } => Some(body),
        _ => None,
    }
}

const fn lint_statement_span(statement: &Statement) -> Span {
    match statement {
        Statement::LanguageSelection { span, .. }
        | Statement::Published { span, .. }
        | Statement::DiagnosticControl { span, .. }
        | Statement::Implementation { span, .. }
        | Statement::ContextAssignment { span, .. }
        | Statement::Function { span, .. }
        | Statement::Generator { span, .. }
        | Statement::Union { span, .. }
        | Statement::Interface { span, .. }
        | Statement::InterfaceImplementation { span, .. }
        | Statement::Foreach { span, .. }
        | Statement::Discard { span, .. } => *span,
        Statement::Binding { name, value, .. } => Span::new(name.start, value.span().end),
        Statement::StateField { name, classifier } => Span::new(name.start, classifier.end),
        Statement::Return { keyword, value } => Span::new(keyword.start, value.span().end),
        Statement::Expression(expression) => expression.span(),
    }
}

fn topal_rule(
    entry: &CatalogEntry,
    source: &SourceText,
    statements: &[Statement],
) -> Result<Vec<RuleFinding>, String> {
    let rule = entry.lint_rule.as_ref().expect("caller checks attachment");
    let rule_source = &rule.source_text;
    let expected_parameters: &[&str] = match rule.view.as_str() {
        "task-declaration-order/1" => &["Int", "Int"],
        "task-state-machine/1" => &["Boolean", "Boolean"],
        other => {
            return Err(format!(
                "best-practice {} requires unsupported read-only view `{other}`",
                entry.identity
            ));
        }
    };
    validate_rule_text(
        rule_source,
        "<embedded lint rule>",
        &rule.entry_point,
        Some(expected_parameters),
    )?;
    match rule.view.as_str() {
        "task-declaration-order/1" => {
            topal_task_declaration_order(entry, rule_source, &rule.entry_point, source, statements)
        }
        "task-state-machine/1" => {
            topal_task_state_machine(entry, rule_source, &rule.entry_point, source, statements)
        }
        _ => unreachable!("view checked above"),
    }
}

fn topal_task_state_machine(
    entry: &CatalogEntry,
    rule_source: &str,
    entry_point: &str,
    source: &SourceText,
    statements: &[Statement],
) -> Result<Vec<RuleFinding>, String> {
    let mut findings = Vec::new();
    visit_topal_task_state_machine(
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

fn visit_topal_task_state_machine(
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
            Statement::Published { declaration, .. } => visit_topal_task_state_machine(
                entry,
                rule_source,
                entry_point,
                source,
                std::slice::from_ref(declaration.as_ref()),
                context,
                findings,
            )?,
            Statement::Implementation {
                name, declarations, ..
            } => {
                let features = task_features(source, declarations, context);
                if is_task_definition(source, declarations)
                    && entry_applies(entry, context.version.as_deref(), &features)?
                {
                    let has_state = declarations
                        .iter()
                        .any(|declaration| matches!(declaration, Statement::StateField { .. }));
                    let has_transition = declarations.iter().any(|declaration| match declaration {
                        Statement::Function { name, body, .. }
                            if !matches!(source.slice(*name), "start" | "terminate") =>
                        {
                            contains_context_assignment(body)
                        }
                        Statement::Generator { body, .. } => contains_context_assignment(body),
                        _ => false,
                    });
                    if !evaluate_topal_boolean_rule(
                        rule_source,
                        entry_point,
                        has_state,
                        has_transition,
                    )? {
                        findings.push(RuleFinding {
                            span: *name,
                            message: "stateful task declares no explicit message transition",
                            suggestion: "update task-owned state in the handler for each state-changing event",
                            rectification: None,
                        });
                    }
                }
                visit_topal_task_state_machine(
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
                visit_topal_task_state_machine(
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
                visit_topal_task_state_machine(
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

fn contains_context_assignment(statements: &[Statement]) -> bool {
    statements.iter().any(|statement| match statement {
        Statement::ContextAssignment { .. } => true,
        Statement::Published { declaration, .. } => {
            contains_context_assignment(std::slice::from_ref(declaration.as_ref()))
        }
        Statement::Function { body, .. }
        | Statement::Generator { body, .. }
        | Statement::Foreach { body, .. } => contains_context_assignment(body),
        Statement::Implementation { declarations, .. }
        | Statement::InterfaceImplementation { declarations, .. } => {
            contains_context_assignment(declarations)
        }
        _ => false,
    })
}

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

#[cfg(test)]
mod tests {
    //! Shared-engine conformance evidence for TOPAL-SYN-SOURCE-001,
    //! TOPAL-SYN-GRAMMAR-001, TOPAL-BEST-PRACTICE-RULE-CONTAINMENT-001,
    //! and TOPAL-BEST-PRACTICE-RECTIFICATION-001.

    use super::*;
    use std::path::Path;

    #[test]
    fn every_language_example_uses_the_shared_linter_frontend() {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/language");
        let mut examples = std::fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "t"))
            .collect::<Vec<_>>();
        examples.sort();
        assert_eq!(examples.len(), 192);
        for example in examples {
            let source = std::fs::read_to_string(&example).unwrap();
            let report = lint_text(&source, &[]).unwrap();
            assert!(
                !report.has_errors,
                "{} produced linter errors: {:?}",
                example.display(),
                report.diagnostics
            );
        }
    }

    #[test]
    fn automatic_edits_are_ordered_and_review_required_edits_are_ignored() {
        let edits = [
            SourceEdit {
                span: Span::new(4, 5),
                replacement: "B".into(),
                safety: RectificationSafety::SyntaxPreserving,
            },
            SourceEdit {
                span: Span::new(0, 1),
                replacement: "A".into(),
                safety: RectificationSafety::SemanticsProven,
            },
            SourceEdit {
                span: Span::new(2, 3),
                replacement: "ignored".into(),
                safety: RectificationSafety::ReviewRequired,
            },
        ];
        assert_eq!(apply_source_edits("01245", &edits).unwrap(), "A124B");
    }

    #[test]
    fn automatic_edits_reject_overlap_and_invalid_utf8_boundaries() {
        let edit = |span| SourceEdit {
            span,
            replacement: String::new(),
            safety: RectificationSafety::PresentationOnly,
        };
        assert!(
            apply_source_edits("abcd", &[edit(Span::new(0, 2)), edit(Span::new(1, 3))])
                .unwrap_err()
                .contains("overlap")
        );
        assert!(
            apply_source_edits("å", &[edit(Span::new(1, 2))])
                .unwrap_err()
                .contains("invalid source span")
        );
    }

    #[test]
    fn rectified_candidate_must_reparse_before_replacement() {
        assert!(require_clean_syntax("value is 1").is_ok());
        assert!(require_clean_syntax("value is #").is_err());
    }

    #[test]
    fn accepted_candidate_replaces_the_complete_file_transactionally() {
        let path = std::env::temp_dir().join(format!(
            "topal-lint-rectification-transaction-{}.t",
            std::process::id()
        ));
        fs::write(&path, "value is 1\n").unwrap();
        replace_source_atomically(&path, "value is 2\n").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "value is 2\n");
        fs::remove_file(path).unwrap();
    }

    fn entry() -> CatalogEntry {
        Catalog::builtin().entries.remove(0)
    }

    #[test]
    fn exact_identity_overrides_namespace_and_tag_regardless_of_order() {
        let entry = entry();
        let settings = vec![
            Override {
                selector: Selector::Identity(entry.identity.clone()),
                enabled: Some(true),
                severity: SeveritySetting::Set(Severity::Error),
                order: 0,
            },
            Override {
                selector: Selector::Namespace("lang".into()),
                enabled: Some(false),
                severity: SeveritySetting::Set(Severity::Warning),
                order: 1,
            },
            Override {
                selector: Selector::Tag(entry.tags[0].clone()),
                enabled: Some(false),
                severity: SeveritySetting::Keep,
                order: 2,
            },
        ];
        assert_eq!(
            policy(&entry, &settings).unwrap(),
            Policy {
                enabled: true,
                severity: Severity::Error
            }
        );
    }

    #[test]
    fn later_setting_wins_within_one_specificity() {
        let entry = entry();
        let settings = vec![
            Override {
                selector: Selector::Identity(entry.identity.clone()),
                enabled: Some(true),
                severity: SeveritySetting::Keep,
                order: 0,
            },
            Override {
                selector: Selector::Identity(entry.identity.clone()),
                enabled: Some(false),
                severity: SeveritySetting::Keep,
                order: 1,
            },
        ];
        assert!(!policy(&entry, &settings).unwrap().enabled);
    }

    #[test]
    fn topal_rule_decides_adjacent_phase_order() {
        let entry = Catalog::builtin()
            .entries
            .into_iter()
            .find(|entry| entry.identity.ends_with("declaration-order"))
            .unwrap();
        let rule = entry.lint_rule.unwrap();
        let source = rule.source_text;
        assert!(evaluate_topal_phase_rule(&source, &rule.entry_point, 0, 1).unwrap());
        assert!(evaluate_topal_phase_rule(&source, &rule.entry_point, 2, 3).unwrap());
        assert!(!evaluate_topal_phase_rule(&source, &rule.entry_point, 2, 1).unwrap());
        assert!(!evaluate_topal_phase_rule(&source, &rule.entry_point, 3, 0).unwrap());
    }

    #[test]
    fn topal_rule_decides_whether_state_has_a_message_transition() {
        let entry = Catalog::builtin()
            .entries
            .into_iter()
            .find(|entry| entry.identity.ends_with("state-machine"))
            .unwrap();
        let rule = entry.lint_rule.unwrap();
        assert!(
            evaluate_topal_boolean_rule(&rule.source_text, &rule.entry_point, false, false)
                .unwrap()
        );
        assert!(
            evaluate_topal_boolean_rule(&rule.source_text, &rule.entry_point, true, true).unwrap()
        );
        assert!(
            !evaluate_topal_boolean_rule(&rule.source_text, &rule.entry_point, true, false)
                .unwrap()
        );
    }

    #[test]
    fn unsupported_read_only_view_revision_is_rejected_before_rule_execution() {
        let mut entry = Catalog::builtin()
            .entries
            .into_iter()
            .find(|entry| entry.identity.ends_with("state-machine"))
            .unwrap();
        entry.lint_rule.as_mut().unwrap().view = "task-state-machine/2".into();
        let source = SourceText::new("use language ( version is v0.1 )\n").unwrap();
        let lexed = lex(&source);
        let parsed = parse(&source, &lexed);
        let Err(error) = topal_rule(&entry, &source, &parsed.statements) else {
            panic!("unsupported view revision was accepted");
        };
        assert!(error.contains("unsupported read-only view"));
    }

    #[test]
    fn in_memory_api_matches_shared_frontend_and_rule_diagnostics() {
        let syntax = lint_text("use language ( version is v0.1 )\nvalue is #\n", &[]).unwrap();
        assert!(syntax.has_errors);
        assert!(
            syntax
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "E-UNKNOWN-TOKEN")
        );

        let task = "use language (\n  version is v0.1\n)\nCounter is Task (queue-size is 2)\nservice is Counter\n  count : Nat\n  start is fn (initial : Nat) -> Completed\n    @ count is initial\n    Completed\n  current is fn (_ : MessageContext, _ : Unit) -> Nat\n    @ count\n";
        assert!(lint_text(task, &[]).unwrap().diagnostics.is_empty());
        let report = lint_text(task, &["lang best-practice task state-machine"]).unwrap();
        assert!(!report.has_errors);
        let diagnostic = report
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.code == "L-TASK-STATE-MACHINE")
            .unwrap();
        assert_eq!(diagnostic.severity, Severity::Warning);
        assert!(
            diagnostic
                .best_practice
                .as_ref()
                .unwrap()
                .suggestion
                .is_some()
        );
    }

    #[test]
    fn structured_source_control_suppresses_regardless_of_configured_severity() {
        let task = "use language (\n  version is v0.1\n)\nCounter is Task (queue-size is 2)\nlang disable-diagnostic ( lang best-practice task state-machine )\nservice is Counter\n  count : Nat\n  start is fn (initial : Nat) -> Completed\n    @ count is initial\n    Completed\n  current is fn (_ : MessageContext, _ : Unit) -> Nat\n    @ count\n";
        let controls = [
            LintControl::Enable("lang best-practice task state-machine".into()),
            LintControl::Severity {
                selector: "lang best-practice task state-machine".into(),
                severity: Severity::Error,
            },
        ];
        let report = lint_text_with_controls(task, &controls).unwrap();
        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);

        let scoped = task.replace(
            "lang disable-diagnostic ( lang best-practice task state-machine )",
            "lang push-disable-diagnostic ( lang best-practice task state-machine )",
        ) + "lang pop-disable-diagnostic ( lang best-practice task state-machine )\n";
        let report = lint_text_with_controls(&scoped, &controls).unwrap();
        assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    }

    #[test]
    fn in_memory_controls_share_selector_precedence_and_severity() {
        let source = "use language (\n  version is v0.1\n)\nCounter is Task (queue-size is 2)\nservice is Counter\n  count : Nat\n  start is fn (initial : Nat) -> Completed\n    @ count is initial\n    Completed\n  current is fn (_ : MessageContext, _ : Unit) -> Nat\n    @ count\n";
        let identity = "lang best-practice task state-machine";
        let report = lint_text_with_controls(
            source,
            &[
                LintControl::Enable("tag:lang best-practice tag architecture".into()),
                LintControl::Severity {
                    selector: identity.into(),
                    severity: Severity::Error,
                },
            ],
        )
        .unwrap();
        assert!(report.has_errors);
        assert_eq!(report.diagnostics[0].severity, Severity::Error);

        let report = lint_text_with_controls(
            source,
            &[
                LintControl::Enable("tag:lang best-practice tag architecture".into()),
                LintControl::Disable(identity.into()),
            ],
        )
        .unwrap();
        assert!(report.diagnostics.is_empty());
    }

    #[test]
    fn in_memory_engine_requires_explicit_noncolliding_external_catalogs() {
        let mut external = Catalog::builtin();
        external.entries.truncate(1);
        external.entries[0].identity = "org.example best-practice task order".into();
        let source = serde_json::to_string(&external).unwrap();
        let mut engine = LintEngine::builtin();
        engine.add_catalog_json(&source).unwrap();
        assert!(
            engine
                .lint_text(
                    "use language ( version is v0.1 )\nvalue is 1\n",
                    &[LintControl::Enable(
                        "org.example best-practice task order".into()
                    )]
                )
                .is_ok()
        );

        let duplicate = serde_json::to_string(&Catalog::builtin()).unwrap();
        assert!(
            engine
                .add_catalog_json(&duplicate)
                .unwrap_err()
                .contains("more than one catalog")
        );
    }

    #[test]
    fn applicability_uses_the_selected_source_language_version() {
        let mut entry = Catalog::builtin()
            .entries
            .into_iter()
            .find(|entry| entry.identity.ends_with("declaration-order"))
            .unwrap();
        let features = BTreeSet::from(["task".to_string()]);
        assert!(entry_applies(&entry, Some("v0.1"), &features).unwrap());
        assert!(entry_applies(&entry, Some("v0.2"), &features).unwrap());
        assert!(!entry_applies(&entry, None, &features).unwrap());

        entry.status.kind = "obsolete".into();
        entry.status.since_language_version = Some("v0.2".into());
        entry.status.explanation = Some("covered by the language".into());
        assert!(entry_applies(&entry, Some("v0.1"), &features).unwrap());
        assert!(!entry_applies(&entry, Some("v0.2"), &features).unwrap());
        assert!(!entry_applies(&entry, Some("v0.3"), &features).unwrap());
    }

    #[test]
    fn applicability_tracks_each_selected_context_and_used_feature() {
        let mut external = Catalog::builtin();
        external
            .entries
            .retain(|entry| entry.identity.ends_with("declaration-order"));
        external.entries[0].identity = "org.example best-practice selected task order".into();
        external.entries[0].required_features = vec!["task".into(), "experimental".into()];
        external.entries[0].excluded_features = vec!["legacy".into()];
        let mut engine = LintEngine::builtin();
        engine
            .add_catalog_json(&serde_json::to_string(&external).unwrap())
            .unwrap();
        let source = "use language ( version is v0.1 )\nFirst is Task (queue-size is 1)\nfirst is First\n  start is fn (initial : Nat) -> Completed\n    Completed\n  count : Nat\nuse language ( version is v0.1, features is ( experimental ) )\nSecond is Task (queue-size is 1)\nsecond is Second\n  start is fn (initial : Nat) -> Completed\n    Completed\n  count : Nat\nuse language ( version is v0.1, features is ( experimental, legacy ) )\nThird is Task (queue-size is 1)\nthird is Third\n  start is fn (initial : Nat) -> Completed\n    Completed\n  count : Nat\n";
        let report = engine
            .lint_text(
                source,
                &[LintControl::Enable(
                    "org.example best-practice selected task order".into(),
                )],
            )
            .unwrap();
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(report.diagnostics[0].line, 12);
    }

    #[test]
    fn deprecated_entries_are_off_by_default_but_remain_selectable() {
        let mut entry = entry();
        entry.status.kind = "deprecated".into();
        entry.status.explanation = Some("superseded guidance".into());
        entry.default_enabled = true;
        assert!(!policy(&entry, &[]).unwrap().enabled);

        let settings = [Override {
            selector: Selector::Identity(entry.identity.clone()),
            enabled: Some(true),
            severity: SeveritySetting::Keep,
            order: 0,
        }];
        assert!(policy(&entry, &settings).unwrap().enabled);
    }
}
