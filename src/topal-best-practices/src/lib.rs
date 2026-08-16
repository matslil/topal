//! Versioned best-practice catalog model shared by Topal tools.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const SUPPORTED_SCHEMA: u64 = 5;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub generated_by: String,
    pub schema: u64,
    pub entries: Vec<CatalogEntry>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogEntry {
    pub identity: String,
    pub version: String,
    pub authoritative_sha256: String,
    pub status: Status,
    pub class: String,
    pub recommendation: String,
    pub default_enabled: bool,
    pub default_severity: String,
    pub language: String,
    pub language_versions: String,
    pub required_features: Vec<String>,
    pub excluded_features: Vec<String>,
    pub tags: Vec<String>,
    pub lint_rule: Option<RuleAttachment>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Status {
    pub kind: String,
    pub since_language_version: Option<String>,
    pub explanation: Option<String>,
    pub replacement: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuleAttachment {
    pub engine: String,
    pub entry_point: String,
    pub version: String,
    pub stage: String,
    pub view: String,
    pub diagnostic_code: String,
    pub source_sha256: String,
    pub source_text: String,
}

impl Catalog {
    /// Decode and minimally validate a generated catalog.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed JSON, unsupported schema revisions,
    /// duplicate identities, or entries without structured identities.
    pub fn from_json(source: &str) -> Result<Self, String> {
        let mut catalog: Self = serde_json::from_str(source)
            .map_err(|error| format!("invalid best-practice catalog: {error}"))?;
        if catalog.schema != SUPPORTED_SCHEMA {
            return Err(format!(
                "unsupported best-practice catalog schema {}",
                catalog.schema
            ));
        }
        catalog
            .entries
            .sort_by(|left, right| left.identity.cmp(&right.identity));
        for entry in &catalog.entries {
            if entry.identity.split(' ').count() < 3 || entry.identity.contains("  ") {
                return Err(format!(
                    "best-practice catalog contains invalid identity `{}`",
                    entry.identity
                ));
            }
            validate_entry(entry)?;
        }
        for pair in catalog.entries.windows(2) {
            if pair[0].identity == pair[1].identity {
                return Err(format!(
                    "best-practice catalog repeats identity `{}`",
                    pair[0].identity
                ));
            }
        }
        Ok(catalog)
    }

    /// Return the catalog generated from repository-owned entries.
    ///
    /// # Panics
    ///
    /// Panics only when a repository build contains a stale invalid generated
    /// catalog, which is guarded by the catalog conformance test.
    #[must_use]
    pub fn builtin() -> Self {
        Self::from_json(include_str!(
            "../../../best-practices/generated/lint-catalog.json"
        ))
        .expect("committed built-in best-practice catalog must be valid")
    }

    /// Add one explicitly selected external catalog.
    ///
    /// # Errors
    ///
    /// Returns an error if the external catalog collides with any already
    /// loaded stable identity.
    pub fn merge(&mut self, external: Self) -> Result<(), String> {
        if external.schema != SUPPORTED_SCHEMA {
            return Err(format!(
                "unsupported best-practice catalog schema {}",
                external.schema
            ));
        }
        for entry in &external.entries {
            validate_entry(entry)?;
        }
        self.entries.extend(external.entries);
        self.entries
            .sort_by(|left, right| left.identity.cmp(&right.identity));
        for pair in self.entries.windows(2) {
            if pair[0].identity == pair[1].identity {
                return Err(format!(
                    "best-practice identity `{}` is supplied by more than one catalog",
                    pair[0].identity
                ));
            }
        }
        Ok(())
    }
}

fn validate_entry(entry: &CatalogEntry) -> Result<(), String> {
    if entry.status.kind == "proposed" && entry.default_enabled {
        return Err(format!(
            "proposed best-practice `{}` cannot be enabled by default",
            entry.identity
        ));
    }
    if !matches!(
        entry.class.as_str(),
        "template" | "recommended" | "best-practice"
    ) {
        return Err(format!("unknown best-practice class `{}`", entry.class));
    }
    if entry.recommendation.is_empty() {
        return Err(format!(
            "best-practice `{}` has no recommendation guidance",
            entry.identity
        ));
    }
    validate_status(entry)?;
    if !matches!(entry.default_severity.as_str(), "warning" | "error") {
        return Err(format!(
            "unknown default severity `{}` for {}",
            entry.default_severity, entry.identity
        ));
    }
    if entry.tags.is_empty() {
        return Err(format!("best-practice `{}` has no tags", entry.identity));
    }
    if let Some(rule) = &entry.lint_rule {
        if rule.engine != "topal" {
            return Err(format!("unknown lint-rule engine `{}`", rule.engine));
        }
        if rule.source_text.is_empty() || rule.source_sha256 != sha256(&rule.source_text) {
            return Err("Topal lint-rule attachment requires authenticated embedded source".into());
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
            return Err(format!(
                "invalid lint-rule diagnostic code `{}`",
                rule.diagnostic_code
            ));
        }
    }
    Ok(())
}

fn validate_status(entry: &CatalogEntry) -> Result<(), String> {
    match entry.status.kind.as_str() {
        "proposed" | "active" => {
            if entry.status.since_language_version.is_some() {
                return Err(format!(
                    "best-practice `{}` records an obsolete-since version without obsolete status",
                    entry.identity
                ));
            }
        }
        "obsolete" => {
            if entry
                .status
                .since_language_version
                .as_deref()
                .is_none_or(str::is_empty)
            {
                return Err(format!(
                    "obsolete best-practice `{}` requires a language version",
                    entry.identity
                ));
            }
            if entry
                .status
                .explanation
                .as_deref()
                .is_none_or(str::is_empty)
            {
                return Err(format!(
                    "obsolete best-practice `{}` requires an explanation",
                    entry.identity
                ));
            }
        }
        "deprecated" => {
            if entry.status.since_language_version.is_some() {
                return Err(format!(
                    "deprecated best-practice `{}` cannot use a language-version cutoff",
                    entry.identity
                ));
            }
            if entry
                .status
                .explanation
                .as_deref()
                .is_none_or(str::is_empty)
            {
                return Err(format!(
                    "deprecated best-practice `{}` requires an explanation",
                    entry.identity
                ));
            }
        }
        other => return Err(format!("unknown best-practice status `{other}`")),
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

fn sha256(source: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(source.as_bytes());
    format!("{:x}", digest.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_catalog_is_valid_and_sorted() {
        let catalog = Catalog::builtin();
        assert_eq!(catalog.schema, SUPPORTED_SCHEMA);
        assert!(!catalog.entries.is_empty());
        assert!(
            catalog
                .entries
                .is_sorted_by(|left, right| left.identity <= right.identity)
        );
        assert!(
            catalog
                .entries
                .iter()
                .all(|entry| !entry.recommendation.is_empty())
        );
    }

    #[test]
    fn duplicate_external_identity_is_rejected() {
        let mut catalog = Catalog::builtin();
        assert!(
            catalog
                .merge(Catalog::builtin())
                .unwrap_err()
                .contains("more than one")
        );
    }

    #[test]
    fn malformed_external_rule_attachment_is_rejected() {
        let mut catalog = Catalog::builtin();
        let entry = catalog
            .entries
            .iter_mut()
            .find(|entry| entry.lint_rule.is_some())
            .unwrap();
        entry.lint_rule.as_mut().unwrap().engine = "ambient-process".into();
        let source = serde_json::to_string(&catalog).unwrap();
        assert!(
            Catalog::from_json(&source)
                .unwrap_err()
                .contains("unknown lint-rule engine")
        );
    }

    #[test]
    fn modified_embedded_rule_source_is_rejected() {
        let mut catalog = Catalog::builtin();
        let rule = catalog
            .entries
            .iter_mut()
            .find_map(|entry| entry.lint_rule.as_mut())
            .unwrap();
        rule.source_text.push_str("# untrusted modification\n");
        let source = serde_json::to_string(&catalog).unwrap();
        assert!(
            Catalog::from_json(&source)
                .unwrap_err()
                .contains("authenticated embedded source")
        );
    }

    #[test]
    fn malformed_read_only_view_identity_is_rejected() {
        let mut catalog = Catalog::builtin();
        let rule = catalog
            .entries
            .iter_mut()
            .find_map(|entry| entry.lint_rule.as_mut())
            .unwrap();
        rule.view = "../ambient/0".into();
        let source = serde_json::to_string(&catalog).unwrap();
        assert!(
            Catalog::from_json(&source)
                .unwrap_err()
                .contains("stable `name/version`")
        );
    }

    #[test]
    fn malformed_external_lifecycle_metadata_is_rejected() {
        let mut catalog = Catalog::builtin();
        catalog.entries[0].status.kind = "obsolete".into();
        catalog.entries[0].status.since_language_version = None;
        catalog.entries[0].status.explanation = Some("the language now enforces this".into());
        let source = serde_json::to_string(&catalog).unwrap();
        assert!(
            Catalog::from_json(&source)
                .unwrap_err()
                .contains("requires a language version")
        );

        catalog.entries[0].status.kind = "deprecated".into();
        catalog.entries[0].status.since_language_version = Some("v0.2".into());
        let source = serde_json::to_string(&catalog).unwrap();
        assert!(
            Catalog::from_json(&source)
                .unwrap_err()
                .contains("cannot use a language-version cutoff")
        );
    }
}
