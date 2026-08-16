use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::Serialize;
use topal_best_practices::{Catalog, CatalogEntry};
use topal_source::{SourceText, Span};
use topal_syntax::{SyntaxDiagnostic, lex, parse};

const USAGE: &str = "usage: topal-lint [OPTIONS] SOURCE...\n\
  --list                         list selected best-practices\n\
  --explain ID                   explain one stable identity\n\
  --catalog PATH                 explicitly load an external catalog\n\
  --enable SELECTOR              enable an identity, namespace:PATH, or tag:ID\n\
  --disable SELECTOR             disable an identity, namespace:PATH, or tag:ID\n\
  --severity SELECTOR=LEVEL      set warning, error, or off\n\
  --format terminal|json         select finding presentation";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Severity {
    Warning,
    Error,
}

impl Severity {
    fn parse(value: &str) -> Result<Option<Self>, String> {
        match value {
            "warning" => Ok(Some(Self::Warning)),
            "error" => Ok(Some(Self::Error)),
            "off" => Ok(None),
            _ => Err(format!("unknown lint severity `{value}`")),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Format {
    Terminal,
    Json,
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Policy {
    enabled: bool,
    severity: Severity,
}

#[derive(Serialize)]
struct Finding<'a> {
    severity: Severity,
    code: &'a str,
    message: &'a str,
    source: &'a str,
    line: usize,
    column: usize,
}

fn main() -> ExitCode {
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
    let mut has_error = false;
    for path in &options.sources {
        has_error |= lint_source(path, format)?;
    }
    Ok(has_error)
}

fn parse_options(arguments: impl Iterator<Item = String>) -> Result<Options, String> {
    let mut options = Options::default();
    let mut arguments = arguments.peekable();
    let mut order = 0;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
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
                    severity: match Severity::parse(severity)? {
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
    let default_severity = Severity::parse(&entry.default_severity)?
        .ok_or_else(|| "catalog default severity cannot be off".to_string())?;
    let mut policy = Policy {
        enabled: entry.default_enabled,
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

fn lint_source(path: &Path, format: Format) -> Result<bool, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("cannot read source {}: {error}", path.display()))?;
    let source = match SourceText::new(&text) {
        Ok(source) => source,
        Err(error) => {
            let finding = finding(path, &text, error.span, error.code, error.message);
            emit(&finding, format)?;
            return Ok(true);
        }
    };
    let lexed = lex(&source);
    let parsed = parse(&source, &lexed);
    let mut seen = BTreeSet::new();
    let mut diagnostics = Vec::new();
    diagnostics.extend(lexed.diagnostics.iter());
    diagnostics.extend(parsed.diagnostics.iter());
    for diagnostic in diagnostics {
        if seen.insert((diagnostic.code, diagnostic.span.start, diagnostic.span.end)) {
            emit_syntax(path, &source, diagnostic, format)?;
        }
    }
    Ok(!seen.is_empty())
}

fn emit_syntax(
    path: &Path,
    source: &SourceText,
    diagnostic: &SyntaxDiagnostic,
    format: Format,
) -> Result<(), String> {
    let position = source.position(diagnostic.span.start);
    let finding = Finding {
        severity: Severity::Error,
        code: diagnostic.code,
        message: &diagnostic.message,
        source: path.to_str().unwrap_or("<source>"),
        line: position.line,
        column: position.column,
    };
    emit(&finding, format)
}

fn finding<'a>(
    path: &'a Path,
    text: &str,
    span: Span,
    code: &'a str,
    message: &'a str,
) -> Finding<'a> {
    let (line, column) = byte_position(text, span.start);
    Finding {
        severity: Severity::Error,
        code,
        message,
        source: path.to_str().unwrap_or("<source>"),
        line,
        column,
    }
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

fn emit(finding: &Finding<'_>, format: Format) -> Result<(), String> {
    match format {
        Format::Terminal => eprintln!(
            "{}[{}]: {}\n --> {}:{}:{}",
            finding.severity.label(),
            finding.code,
            finding.message,
            finding.source,
            finding.line,
            finding.column
        ),
        Format::Json => println!(
            "{}",
            serde_json::to_string(finding).map_err(|error| error.to_string())?
        ),
    }
    Ok(())
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
}
