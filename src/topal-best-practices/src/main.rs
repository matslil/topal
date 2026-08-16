use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const GENERATOR_VERSION: &str = "topal-best-practices/6";
const SCHEMA_VERSION: u64 = 6;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BestPractice {
    schema: u64,
    identity: Vec<String>,
    version: String,
    status: Status,
    class: String,
    title: String,
    summary: String,
    recommendation: String,
    rationale: String,
    language: String,
    language_versions: String,
    required_features: Vec<String>,
    excluded_features: Vec<String>,
    default_enabled: bool,
    default_severity: String,
    checkability: String,
    confidence: Option<String>,
    rectification: String,
    tags: Vec<Vec<String>>,
    exceptions: Vec<String>,
    examples: Vec<String>,
    specification_rules: Vec<String>,
    lint_rule: Option<RuleAttachment>,
    provenance: String,
    license: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Status {
    kind: String,
    since_language_version: Option<String>,
    explanation: Option<String>,
    replacement: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RuleAttachment {
    engine: String,
    entry_point: String,
    version: String,
    stage: String,
    view: String,
    diagnostic_code: String,
    source: String,
}

struct Entry {
    directory: PathBuf,
    record_path: PathBuf,
    guidance_path: PathBuf,
    record_source: String,
    guidance: String,
    rule_source: Option<String>,
    record: BestPractice,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), String> {
    let command = env::args().nth(1).unwrap_or_else(|| "check".into());
    let root = workspace_root()?;
    let entries = load_entries(&root)?;
    match command.as_str() {
        "generate" => generate(&root, &entries),
        "check" => check(&root, &entries),
        _ => Err("usage: topal-best-practices [generate|check]".into()),
    }
}

fn workspace_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|error| format!("cannot locate workspace root: {error}"))
}

fn load_entries(root: &Path) -> Result<Vec<Entry>, String> {
    let mut records = Vec::new();
    visit_records(&root.join("best-practices/entries"), &mut records)?;
    records.sort();
    if records.is_empty() {
        return Err("best-practice database contains no entries".into());
    }
    records
        .into_iter()
        .map(|record_path| load_entry(root, record_path))
        .collect()
}

fn visit_records(directory: &Path, records: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(directory)
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?
    {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.is_dir() {
            visit_records(&path, records)?;
        } else if path
            .file_name()
            .is_some_and(|name| name == "best-practice.json")
        {
            records.push(path);
        }
    }
    Ok(())
}

fn load_entry(root: &Path, record_path: PathBuf) -> Result<Entry, String> {
    let directory = record_path.parent().unwrap().to_owned();
    let guidance_path = directory.join("guidance.md");
    let record_source = fs::read_to_string(&record_path)
        .map_err(|error| format!("cannot read {}: {error}", record_path.display()))?;
    let guidance = fs::read_to_string(&guidance_path)
        .map_err(|error| format!("cannot read {}: {error}", guidance_path.display()))?;
    let record: BestPractice = serde_json::from_str(&record_source)
        .map_err(|error| format!("invalid {}: {error}", record_path.display()))?;
    validate(root, &directory, &record)?;
    let rule_source = match record.lint_rule.as_ref() {
        Some(rule) => Some(
            fs::read_to_string(root.join(&rule.source))
                .map_err(|error| format!("cannot read lint rule {}: {error}", rule.source))?,
        ),
        None => None,
    };
    Ok(Entry {
        directory,
        record_path,
        guidance_path,
        record_source,
        guidance,
        rule_source,
        record,
    })
}

fn validate(root: &Path, directory: &Path, record: &BestPractice) -> Result<(), String> {
    if record.schema != SCHEMA_VERSION {
        return Err(format!(
            "unsupported best-practice schema {}",
            record.schema
        ));
    }
    if record.identity.len() < 3 || record.identity.iter().any(String::is_empty) {
        return Err("best-practice identity must be a structured nonempty path".into());
    }
    if record.identity.get(1).map(String::as_str) != Some("best-practice") {
        return Err(
            "best-practice identity must place `best-practice` after its owner namespace".into(),
        );
    }
    let stored_path = directory
        .strip_prefix(root.join("best-practices/entries"))
        .map_err(|error| error.to_string())?;
    let identity_path: PathBuf = record
        .identity
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != 1)
        .map(|(_, component)| component)
        .collect();
    if stored_path != identity_path {
        return Err(format!(
            "best-practice identity `{}` does not match storage path {}",
            identity(record),
            stored_path.display()
        ));
    }
    validate_status(record)?;
    validate_class(record)?;
    if record.language.is_empty() || record.language_versions.is_empty() {
        return Err("best-practice applicability requires language and versions".into());
    }
    if record.tags.is_empty()
        || record
            .tags
            .iter()
            .any(|tag| tag.len() < 2 || tag.iter().any(String::is_empty))
    {
        return Err("best-practice tags must be structured nonempty paths".into());
    }
    validate_checkability(record)?;
    if !matches!(
        record.rectification.as_str(),
        "unavailable" | "suggestion" | "automatic"
    ) {
        return Err("unknown best-practice rectification".into());
    }
    for example in &record.examples {
        if !root.join(example).is_file() {
            return Err(format!("missing best-practice example `{example}`"));
        }
    }
    if record.provenance.is_empty() || record.license.is_empty() {
        return Err("best-practice provenance and license are required".into());
    }
    if let Some(rule) = &record.lint_rule {
        if rule.engine != "topal" {
            return Err(format!("unknown lint-rule engine `{}`", rule.engine));
        }
        let path = Path::new(&rule.source);
        if path.is_absolute()
            || path
                .components()
                .any(|component| matches!(component, std::path::Component::ParentDir))
            || !root.join(path).is_file()
        {
            return Err(
                "Topal lint-rule source must be an existing repository-relative file".into(),
            );
        }
        if rule.entry_point.is_empty() || rule.version.is_empty() {
            return Err("lint-rule entry point and version cannot be empty".into());
        }
        if !valid_view(&rule.view) {
            return Err("lint-rule view must use a stable `name/version` identity".into());
        }
        if !matches!(
            rule.stage.as_str(),
            "tokens" | "syntax" | "semantic" | "trace"
        ) {
            return Err(format!("unknown lint-rule stage `{}`", rule.stage));
        }
        if !rule.diagnostic_code.starts_with("L-")
            || !rule.diagnostic_code.chars().all(|character| {
                character.is_ascii_uppercase() || character.is_ascii_digit() || character == '-'
            })
        {
            return Err("lint-rule diagnostic code must use stable `L-...` form".into());
        }
    }
    Ok(())
}

fn valid_view(view: &str) -> bool {
    view.rsplit_once('/').is_some_and(|(name, version)| {
        !name.is_empty()
            && name.chars().all(|character| {
                character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
            })
            && version.parse::<u64>().is_ok_and(|version| version > 0)
    })
}

fn validate_status(record: &BestPractice) -> Result<(), String> {
    if record.status.kind == "proposed" && record.default_enabled {
        return Err("a proposed best-practice cannot be enabled by default".into());
    }
    match record.status.kind.as_str() {
        "proposed" | "active" => {
            if record.status.since_language_version.is_some() {
                return Err("only obsolete status records `since_language_version`".into());
            }
        }
        "obsolete" => {
            if record.status.since_language_version.is_none() {
                return Err("obsolete status requires `since_language_version`".into());
            }
            if record
                .status
                .explanation
                .as_deref()
                .is_none_or(str::is_empty)
            {
                return Err("obsolete status requires an explanation".into());
            }
        }
        "deprecated" => {
            if record.status.since_language_version.is_some() {
                return Err("deprecated status is independent of a language version".into());
            }
            if record
                .status
                .explanation
                .as_deref()
                .is_none_or(str::is_empty)
            {
                return Err("deprecated status requires an explanation".into());
            }
        }
        other => return Err(format!("unknown best-practice status `{other}`")),
    }
    Ok(())
}

fn validate_class(record: &BestPractice) -> Result<(), String> {
    let normal = match record.class.as_str() {
        "template" | "recommended" => "warning",
        "best-practice" => "error",
        other => return Err(format!("unknown best-practice class `{other}`")),
    };
    if record.default_severity != normal {
        return Err(format!(
            "{} normally requires default severity {normal}",
            record.class
        ));
    }
    if record.class == "recommended" && record.exceptions.is_empty() {
        return Err("a recommended best-practice documents its exceptions".into());
    }
    Ok(())
}

fn validate_checkability(record: &BestPractice) -> Result<(), String> {
    if !matches!(
        record.checkability.as_str(),
        "guidance-only" | "heuristic" | "semantic" | "formally-decidable"
    ) {
        return Err("unknown best-practice checkability".into());
    }
    match (record.checkability.as_str(), record.confidence.as_deref()) {
        ("heuristic", Some(confidence)) if !confidence.is_empty() => {}
        ("heuristic", _) => {
            return Err("a heuristic best-practice explains its confidence".into());
        }
        (_, None) => {}
        _ => return Err("only a heuristic best-practice records confidence".into()),
    }
    if record.lint_rule.is_some()
        && record.default_severity == "error"
        && !matches!(
            record.checkability.as_str(),
            "semantic" | "formally-decidable"
        )
    {
        return Err(
            "an executable default-error best-practice requires semantic or formally-decidable checkability"
                .into(),
        );
    }
    Ok(())
}

fn generate(root: &Path, entries: &[Entry]) -> Result<(), String> {
    for entry in entries {
        let (human_path, agent_path) = generated_paths(root, entry)?;
        fs::create_dir_all(human_path.parent().unwrap()).map_err(|error| error.to_string())?;
        fs::create_dir_all(agent_path.parent().unwrap()).map_err(|error| error.to_string())?;
        fs::write(&human_path, human_projection(entry)).map_err(|error| error.to_string())?;
        fs::write(&agent_path, agent_projection(entry)?).map_err(|error| error.to_string())?;
    }
    fs::write(catalog_path(root), catalog_projection(entries)?)
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn check(root: &Path, entries: &[Entry]) -> Result<(), String> {
    for entry in entries {
        let (human_path, agent_path) = generated_paths(root, entry)?;
        check_projection(&human_path, &human_projection(entry))?;
        check_projection(&agent_path, &agent_projection(entry)?)?;
    }
    check_projection(&catalog_path(root), &catalog_projection(entries)?)?;
    reject_unexpected(root, entries)?;
    Ok(())
}

fn catalog_path(root: &Path) -> PathBuf {
    root.join("best-practices/generated/lint-catalog.json")
}

fn reject_unexpected(root: &Path, entries: &[Entry]) -> Result<(), String> {
    let mut expected = vec![catalog_path(root)];
    for entry in entries {
        let (human, agent) = generated_paths(root, entry)?;
        expected.extend([human, agent]);
    }
    expected.sort();
    let generated = root.join("best-practices/generated");
    let mut actual = Vec::new();
    visit_files(&generated, &mut actual)?;
    actual.sort();
    if actual != expected {
        return Err("generated best-practice output contains missing or unexpected files; run generate and remove obsolete projections".into());
    }
    Ok(())
}

fn visit_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(directory)
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?
    {
        let path = entry.map_err(|error| error.to_string())?.path();
        if path.is_dir() {
            visit_files(&path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}

fn check_projection(path: &Path, expected: &str) -> Result<(), String> {
    let actual = fs::read_to_string(path).map_err(|_| {
        format!(
            "missing generated projection {}; run generate",
            path.display()
        )
    })?;
    if actual != expected {
        return Err(format!(
            "stale generated projection {}; run generate",
            path.display()
        ));
    }
    Ok(())
}

fn generated_paths(root: &Path, entry: &Entry) -> Result<(PathBuf, PathBuf), String> {
    let relative = entry
        .directory
        .strip_prefix(root.join("best-practices/entries"))
        .map_err(|error| error.to_string())?;
    Ok((
        root.join("best-practices/generated/human")
            .join(relative)
            .with_extension("md"),
        root.join("best-practices/generated/agents")
            .join(relative)
            .with_extension("json"),
    ))
}

fn digest(entry: &Entry) -> String {
    let mut digest = Sha256::new();
    digest.update(entry.record_source.as_bytes());
    digest.update([0]);
    digest.update(entry.guidance.as_bytes());
    if let Some(rule_source) = &entry.rule_source {
        digest.update([0]);
        digest.update(rule_source.as_bytes());
    }
    format!("{:x}", digest.finalize())
}

fn identity(record: &BestPractice) -> String {
    record.identity.join(" ")
}

fn human_projection(entry: &Entry) -> String {
    let record = &entry.record;
    format!(
        "<!-- Generated by {GENERATOR_VERSION}; do not edit.\nSource: {} and {}\nSchema: {}; best-practice version: {}; SHA-256: {}\n-->\n\n# {}\n\n**Identity:** `{}`  \n**Status:** `{}`  \n**Class:** `{}`  \n**Default:** {} / `{}`  \n**Checkability:** `{}`  \n**Confidence:** {}  \n**Rectification:** `{}`  \n**Lint rule:** {}\n\n{}\n",
        relative_source(&entry.record_path),
        relative_source(&entry.guidance_path),
        record.schema,
        record.version,
        digest(entry),
        record.title,
        identity(record),
        status_label(&record.status),
        record.class,
        if record.default_enabled {
            "enabled"
        } else {
            "disabled"
        },
        record.default_severity,
        record.checkability,
        record.confidence.as_deref().unwrap_or("not applicable"),
        record.rectification,
        rule_label(record.lint_rule.as_ref()),
        entry.guidance.trim()
    )
}

fn rule_label(rule: Option<&RuleAttachment>) -> String {
    rule.map_or_else(
        || "none".into(),
        |rule| {
            format!(
                "`{}:{}` / `{}` / `{}`",
                rule.engine, rule.entry_point, rule.version, rule.diagnostic_code
            )
        },
    )
}

fn relative_source(path: &Path) -> String {
    path.strip_prefix(workspace_root().expect("workspace root"))
        .unwrap_or(path)
        .display()
        .to_string()
}

fn status_label(status: &Status) -> String {
    match &status.since_language_version {
        Some(version) => format!("{} since {version}", status.kind),
        None => status.kind.clone(),
    }
}

#[derive(Serialize)]
struct AgentProjection<'a> {
    generated_by: &'static str,
    schema: u64,
    authoritative_sha256: String,
    identity: String,
    version: &'a str,
    status: &'a Status,
    class: &'a str,
    title: &'a str,
    summary: &'a str,
    recommendation: &'a str,
    rationale: &'a str,
    applicability: AgentApplicability<'a>,
    exceptions: &'a [String],
    default_enabled: bool,
    default_severity: &'a str,
    checkability: &'a str,
    confidence: Option<&'a str>,
    rectification: &'a str,
    tags: Vec<String>,
    examples: &'a [String],
    lint_rule: Option<RuleProjection<'a>>,
}

#[derive(Serialize)]
struct AgentApplicability<'a> {
    language: &'a str,
    versions: &'a str,
    required_features: &'a [String],
    excluded_features: &'a [String],
}

fn agent_projection(entry: &Entry) -> Result<String, String> {
    let record = &entry.record;
    let projection = AgentProjection {
        generated_by: GENERATOR_VERSION,
        schema: record.schema,
        authoritative_sha256: digest(entry),
        identity: identity(record),
        version: &record.version,
        status: &record.status,
        class: &record.class,
        title: &record.title,
        summary: &record.summary,
        recommendation: &record.recommendation,
        rationale: &record.rationale,
        applicability: AgentApplicability {
            language: &record.language,
            versions: &record.language_versions,
            required_features: &record.required_features,
            excluded_features: &record.excluded_features,
        },
        exceptions: &record.exceptions,
        default_enabled: record.default_enabled,
        default_severity: &record.default_severity,
        checkability: &record.checkability,
        confidence: record.confidence.as_deref(),
        rectification: &record.rectification,
        tags: record.tags.iter().map(|tag| tag.join(" ")).collect(),
        examples: &record.examples,
        lint_rule: projected_rule(entry),
    };
    serde_json::to_string_pretty(&projection)
        .map(|json| format!("{json}\n"))
        .map_err(|error| error.to_string())
}

#[derive(Serialize)]
struct CatalogProjection<'a> {
    generated_by: &'static str,
    schema: u64,
    entries: Vec<CatalogEntry<'a>>,
}

#[derive(Serialize)]
struct CatalogEntry<'a> {
    identity: String,
    version: &'a str,
    authoritative_sha256: String,
    status: &'a Status,
    class: &'a str,
    recommendation: &'a str,
    checkability: &'a str,
    confidence: Option<&'a str>,
    default_enabled: bool,
    default_severity: &'a str,
    language: &'a str,
    language_versions: &'a str,
    required_features: &'a [String],
    excluded_features: &'a [String],
    tags: Vec<String>,
    lint_rule: Option<RuleProjection<'a>>,
}

fn catalog_projection(entries: &[Entry]) -> Result<String, String> {
    let projection = CatalogProjection {
        generated_by: GENERATOR_VERSION,
        schema: SCHEMA_VERSION,
        entries: entries
            .iter()
            .map(|entry| CatalogEntry {
                identity: identity(&entry.record),
                version: &entry.record.version,
                authoritative_sha256: digest(entry),
                status: &entry.record.status,
                class: &entry.record.class,
                recommendation: &entry.record.recommendation,
                checkability: &entry.record.checkability,
                confidence: entry.record.confidence.as_deref(),
                default_enabled: entry.record.default_enabled,
                default_severity: &entry.record.default_severity,
                language: &entry.record.language,
                language_versions: &entry.record.language_versions,
                required_features: &entry.record.required_features,
                excluded_features: &entry.record.excluded_features,
                tags: entry.record.tags.iter().map(|tag| tag.join(" ")).collect(),
                lint_rule: projected_rule(entry),
            })
            .collect(),
    };
    serde_json::to_string_pretty(&projection)
        .map(|json| format!("{json}\n"))
        .map_err(|error| error.to_string())
}

#[derive(Serialize)]
struct RuleProjection<'a> {
    engine: &'a str,
    entry_point: &'a str,
    version: &'a str,
    stage: &'a str,
    view: &'a str,
    diagnostic_code: &'a str,
    source_sha256: String,
    source_text: &'a str,
}

fn projected_rule(entry: &Entry) -> Option<RuleProjection<'_>> {
    entry.record.lint_rule.as_ref().map(|rule| RuleProjection {
        engine: &rule.engine,
        entry_point: &rule.entry_point,
        version: &rule.version,
        stage: &rule.stage,
        view: &rule.view,
        diagnostic_code: &rule.diagnostic_code,
        source_sha256: {
            let source = entry
                .rule_source
                .as_ref()
                .expect("Topal rule source loaded during entry validation");
            let mut digest = Sha256::new();
            digest.update(source.as_bytes());
            format!("{:x}", digest.finalize())
        },
        source_text: entry
            .rule_source
            .as_deref()
            .expect("Topal rule source loaded during entry validation"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn first_entry() -> (PathBuf, Entry) {
        let root = workspace_root().unwrap();
        let mut entries = load_entries(&root).unwrap();
        (root, entries.remove(0))
    }

    #[test]
    fn committed_projections_match_authoritative_entries() {
        let root = workspace_root().unwrap();
        let entries = load_entries(&root).unwrap();
        check(&root, &entries).unwrap();
    }

    #[test]
    fn proposed_entry_cannot_be_enabled() {
        let (root, entry) = first_entry();
        let mut record = entry.record.clone();
        record.default_enabled = true;
        assert!(
            validate(&root, &entry.directory, &record)
                .unwrap_err()
                .contains("proposed")
        );
    }

    #[test]
    fn obsolete_entry_requires_version_and_explanation() {
        let (root, entry) = first_entry();
        let mut record = entry.record.clone();
        record.status.kind = "obsolete".into();
        assert!(
            validate(&root, &entry.directory, &record)
                .unwrap_err()
                .contains("since_language_version")
        );
        record.status.since_language_version = Some("v0.2".into());
        assert!(
            validate(&root, &entry.directory, &record)
                .unwrap_err()
                .contains("explanation")
        );
    }

    #[test]
    fn identity_must_match_owned_storage_path() {
        let (root, entry) = first_entry();
        let mut record = entry.record.clone();
        record.identity.push("different".into());
        assert!(
            validate(&root, &entry.directory, &record)
                .unwrap_err()
                .contains("storage path")
        );
    }

    #[test]
    fn executable_default_error_requires_sound_checkability() {
        let (root, entry) = first_entry();
        let mut record = entry.record.clone();
        record.class = "best-practice".into();
        record.default_severity = "error".into();
        record.checkability = "heuristic".into();
        record.confidence = Some("the syntax view may produce false positives".into());
        assert!(
            validate(&root, &entry.directory, &record)
                .unwrap_err()
                .contains("requires semantic or formally-decidable")
        );
    }
}
