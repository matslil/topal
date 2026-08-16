//! Versioned best-practice catalog model shared by Topal tools.

use serde::{Deserialize, Serialize};

pub const SUPPORTED_SCHEMA: u64 = 1;

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
    pub default_enabled: bool,
    pub default_severity: String,
    pub language: String,
    pub language_versions: String,
    pub required_features: Vec<String>,
    pub excluded_features: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Status {
    pub kind: String,
    pub since_language_version: Option<String>,
    pub explanation: Option<String>,
    pub replacement: Option<Vec<String>>,
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
}
