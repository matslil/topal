use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::Serialize;
use topal_best_practices::{Catalog, CatalogEntry};
use topal_language::{Session, Value as LanguageValue};
use topal_source::{Diagnostic, Severity, SourceText, Span};
use topal_syntax::{
    CallableKind, DecisionMatcher, Expression, Statement, SyntaxDiagnostic, lex, parse,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    best_practice: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    best_practice_version: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rule_version: Option<&'a str>,
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
            best_practice: best_practice.map(|context| context.identity.as_str()),
            best_practice_version: best_practice.map(|context| context.version.as_str()),
            rule_version: best_practice.map(|context| context.rule_version.as_str()),
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
                    "startColumn": diagnostic.column
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
        has_error |= lint_source(path, &catalog, &options.overrides, &mut emitter)?;
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
    println!("class: {}", entry.class);
    println!("enabled: {}", policy.enabled);
    println!("severity: {}", policy.severity.label());
    println!("language: {} {}", entry.language, entry.language_versions);
    println!("tags: {}", entry.tags.join(", "));
    Ok(())
}

fn lint_source(
    path: &Path,
    catalog: &Catalog,
    overrides: &[Override],
    emitter: &mut Emitter,
) -> Result<bool, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("cannot read source {}: {error}", path.display()))?;
    let report = analyze_text(&text, catalog, overrides)?;
    for diagnostic in &report.diagnostics {
        emitter.emit(diagnostic, path)?;
    }
    Ok(report.has_errors)
}

/// Diagnostics produced by one in-memory linter analysis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LintReport {
    pub diagnostics: Vec<Diagnostic>,
    pub has_errors: bool,
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
    let catalog = Catalog::builtin();
    let overrides = controls
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
        .collect::<Result<Vec<_>, String>>()?;
    validate_overrides(&catalog, &overrides)?;
    analyze_text(text, &catalog, &overrides)
}

fn analyze_text(
    text: &str,
    catalog: &Catalog,
    overrides: &[Override],
) -> Result<LintReport, String> {
    let mut findings = Vec::new();
    let source = match SourceText::new(text) {
        Ok(source) => source,
        Err(error) => {
            findings.push(source_diagnostic(
                text,
                error.span,
                error.code,
                error.message,
            ));
            return Ok(LintReport {
                diagnostics: findings,
                has_errors: true,
            });
        }
    };
    let lexed = lex(&source);
    let parsed = parse(&source, &lexed);
    let mut seen = BTreeSet::new();
    let mut has_error = false;
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
        let selected_version = selected_language_version(&source, &parsed.statements);
        for entry in &catalog.entries {
            let entry_policy = policy(entry, overrides)?;
            if !entry_policy.enabled {
                continue;
            }
            if !entry_applies(entry, selected_version)? {
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
                    let diagnostic = match entry_policy.severity {
                        Severity::Warning => Diagnostic::warning(
                            &rule.diagnostic_code,
                            rule_finding.line,
                            rule_finding.column,
                            rule_finding.message,
                        ),
                        Severity::Error => Diagnostic::error(
                            &rule.diagnostic_code,
                            rule_finding.line,
                            rule_finding.column,
                            rule_finding.message,
                        ),
                    }
                    .with_best_practice_suggestion(
                        &entry.identity,
                        &entry.version,
                        &rule.version,
                        rule_finding.suggestion,
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
    })
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
            topal_task_declaration_order(rule_source, &rule.entry_point, source, statements)
        }
        "task-state-machine/1" => {
            topal_task_state_machine(rule_source, &rule.entry_point, source, statements)
        }
        _ => unreachable!("view checked above"),
    }
}

fn topal_task_state_machine(
    rule_source: &str,
    entry_point: &str,
    source: &SourceText,
    statements: &[Statement],
) -> Result<Vec<RuleFinding>, String> {
    let mut findings = Vec::new();
    visit_topal_task_state_machine(rule_source, entry_point, source, statements, &mut findings)?;
    Ok(findings)
}

fn visit_topal_task_state_machine(
    rule_source: &str,
    entry_point: &str,
    source: &SourceText,
    statements: &[Statement],
    findings: &mut Vec<RuleFinding>,
) -> Result<(), String> {
    for statement in statements {
        match statement {
            Statement::Published { declaration, .. } => visit_topal_task_state_machine(
                rule_source,
                entry_point,
                source,
                std::slice::from_ref(declaration.as_ref()),
                findings,
            )?,
            Statement::Implementation {
                name, declarations, ..
            } => {
                if is_task_definition(source, declarations) {
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
                        let position = source.position(name.start);
                        findings.push(RuleFinding {
                            line: position.line,
                            column: position.column,
                            message: "stateful task declares no explicit message transition",
                            suggestion: "update task-owned state in the handler for each state-changing event",
                        });
                    }
                }
                visit_topal_task_state_machine(
                    rule_source,
                    entry_point,
                    source,
                    declarations,
                    findings,
                )?;
            }
            Statement::Function { body, .. }
            | Statement::Generator { body, .. }
            | Statement::Foreach { body, .. } => {
                visit_topal_task_state_machine(rule_source, entry_point, source, body, findings)?;
            }
            Statement::InterfaceImplementation { declarations, .. } => {
                visit_topal_task_state_machine(
                    rule_source,
                    entry_point,
                    source,
                    declarations,
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
    rule_source: &str,
    entry_point: &str,
    source: &SourceText,
    statements: &[Statement],
) -> Result<Vec<RuleFinding>, String> {
    let mut findings = Vec::new();
    visit_topal_task_order(rule_source, entry_point, source, statements, &mut findings)?;
    Ok(findings)
}

fn visit_topal_task_order(
    rule_source: &str,
    entry_point: &str,
    source: &SourceText,
    statements: &[Statement],
    findings: &mut Vec<RuleFinding>,
) -> Result<(), String> {
    for statement in statements {
        match statement {
            Statement::Published { declaration, .. } => visit_topal_task_order(
                rule_source,
                entry_point,
                source,
                std::slice::from_ref(declaration.as_ref()),
                findings,
            )?,
            Statement::Implementation { declarations, .. } => {
                if is_task_definition(source, declarations) {
                    check_topal_task_order(
                        rule_source,
                        entry_point,
                        source,
                        declarations,
                        findings,
                    )?;
                }
                visit_topal_task_order(rule_source, entry_point, source, declarations, findings)?;
            }
            Statement::Function { body, .. }
            | Statement::Generator { body, .. }
            | Statement::Foreach { body, .. } => {
                visit_topal_task_order(rule_source, entry_point, source, body, findings)?;
            }
            Statement::InterfaceImplementation { declarations, .. } => {
                visit_topal_task_order(rule_source, entry_point, source, declarations, findings)?;
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
            let position = source.position(span.start);
            findings.push(RuleFinding {
                line: position.line,
                column: position.column,
                message: "task declaration is outside the recommended lifecycle section",
                suggestion: expected,
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

fn selected_language_version<'a>(
    source: &'a SourceText,
    statements: &'a [Statement],
) -> Option<&'a str> {
    statements.first().and_then(|statement| match statement {
        Statement::LanguageSelection { version, .. } => Some(source.slice(*version)),
        _ => None,
    })
}

fn entry_applies(entry: &CatalogEntry, selected_version: Option<&str>) -> Result<bool, String> {
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
    line: usize,
    column: usize,
    message: &'static str,
    suggestion: &'static str,
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
    use super::*;

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
    fn applicability_uses_the_selected_source_language_version() {
        let mut entry = Catalog::builtin()
            .entries
            .into_iter()
            .find(|entry| entry.identity.ends_with("declaration-order"))
            .unwrap();
        assert!(entry_applies(&entry, Some("v0.1")).unwrap());
        assert!(entry_applies(&entry, Some("v0.2")).unwrap());
        assert!(!entry_applies(&entry, None).unwrap());

        entry.status.kind = "obsolete".into();
        entry.status.since_language_version = Some("v0.2".into());
        entry.status.explanation = Some("covered by the language".into());
        assert!(entry_applies(&entry, Some("v0.1")).unwrap());
        assert!(!entry_applies(&entry, Some("v0.2")).unwrap());
        assert!(!entry_applies(&entry, Some("v0.3")).unwrap());
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
