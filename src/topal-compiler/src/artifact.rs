use std::collections::BTreeSet;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use topal_geir::LEGACY_ARTIFACT_REVISION;
use topal_language::CompilerProgram;

use crate::{DATA_LAYOUT, LLVM_MAJOR, TARGET_TRIPLE};

pub const NATIVE_ARTIFACT_SCHEMA: &str = "topal.native-artifact/1";
pub const NATIVE_ABI: &str = "topal-native/2";
pub const PLATFORM_ABI: &str = "linux-x86_64-syscall/1";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DigestEntry {
    pub identity: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExportEntry {
    pub semantic_identity: String,
    pub machine_symbol: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeSlice {
    pub kind: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NativeArtifactMetadata {
    pub schema: String,
    pub language_revision: String,
    pub geir_revision: u64,
    pub manifest_revision: u64,
    pub native_abi: String,
    pub platform_abi: String,
    pub target_triple: String,
    pub data_layout: String,
    pub object_format: String,
    pub cpu: String,
    pub features: Vec<String>,
    pub llvm_major: u32,
    pub llvm_version: String,
    pub compiler: String,
    pub optimization: u8,
    pub debug_format: String,
    pub source_name: String,
    pub source_sha256: String,
    pub interface_sha256: String,
    pub build_identity: String,
    pub dependencies: Vec<DigestEntry>,
    pub exports: Vec<ExportEntry>,
    pub platform_requirements: Vec<String>,
    pub evidence: Vec<DigestEntry>,
    pub debug_prefix_map: Vec<(String, String)>,
    pub provenance: Vec<String>,
    pub native_slices: Vec<NativeSlice>,
}

impl NativeArtifactMetadata {
    #[must_use]
    pub fn for_program(
        program: &CompilerProgram,
        source_name: &str,
        output_kind: &str,
        output: &[u8],
        llvm_version: &str,
    ) -> Self {
        let source_sha256 = sha256(program.source.as_str().as_bytes());
        let interface_sha256 = sha256(b"topal.native-artifact/1:empty-interface");
        let build_inputs = [
            "topal.native-build/1".to_owned(),
            program.language_version.to_string(),
            source_name.to_owned(),
            source_sha256.clone(),
            TARGET_TRIPLE.to_owned(),
            DATA_LAYOUT.to_owned(),
            NATIVE_ABI.to_owned(),
            PLATFORM_ABI.to_owned(),
            llvm_version.to_owned(),
            env!("CARGO_PKG_VERSION").to_owned(),
            output_kind.to_owned(),
        ];
        let build_identity = sha256(build_inputs.join("\0").as_bytes());
        Self {
            schema: NATIVE_ARTIFACT_SCHEMA.into(),
            language_revision: program.language_version.to_string(),
            geir_revision: LEGACY_ARTIFACT_REVISION,
            manifest_revision: 1,
            native_abi: NATIVE_ABI.into(),
            platform_abi: PLATFORM_ABI.into(),
            target_triple: TARGET_TRIPLE.into(),
            data_layout: DATA_LAYOUT.into(),
            object_format: "elf64-x86-64".into(),
            cpu: "x86-64".into(),
            features: Vec::new(),
            llvm_major: LLVM_MAJOR,
            llvm_version: llvm_version.into(),
            compiler: format!("topalc/{}", env!("CARGO_PKG_VERSION")),
            optimization: 0,
            debug_format: "dwarf-v5-full".into(),
            source_name: source_name.into(),
            source_sha256,
            interface_sha256,
            build_identity,
            dependencies: Vec::new(),
            exports: Vec::new(),
            platform_requirements: vec!["topal.platform.linux-x86_64/1".into()],
            evidence: Vec::new(),
            debug_prefix_map: Vec::new(),
            provenance: Vec::new(),
            native_slices: vec![NativeSlice {
                kind: output_kind.into(),
                sha256: sha256(output),
            }],
        }
    }

    /// Validate a metadata document before its native slice is consumed.
    ///
    /// # Errors
    ///
    /// Returns the exact incompatible or noncanonical field.
    #[allow(clippy::too_many_lines)] // The schema contract remains auditable in one validator.
    pub fn validate(&self) -> Result<(), String> {
        let exact = [
            (self.schema.as_str(), NATIVE_ARTIFACT_SCHEMA, "schema"),
            (self.language_revision.as_str(), "v0.1", "language_revision"),
            (self.native_abi.as_str(), NATIVE_ABI, "native_abi"),
            (self.platform_abi.as_str(), PLATFORM_ABI, "platform_abi"),
            (self.target_triple.as_str(), TARGET_TRIPLE, "target_triple"),
            (self.data_layout.as_str(), DATA_LAYOUT, "data_layout"),
            (self.object_format.as_str(), "elf64-x86-64", "object_format"),
            (self.cpu.as_str(), "x86-64", "cpu"),
            (self.debug_format.as_str(), "dwarf-v5-full", "debug_format"),
        ];
        for (actual, expected, field) in exact {
            if actual != expected {
                return Err(format!(
                    "unsupported {field}: expected `{expected}`, found `{actual}`"
                ));
            }
        }
        if self.geir_revision != LEGACY_ARTIFACT_REVISION
            || self.manifest_revision != 1
            || self.llvm_major != LLVM_MAJOR
            || self.optimization != 0
        {
            return Err("unsupported artifact, GEIR, LLVM, or optimization revision".into());
        }
        if self
            .llvm_version
            .strip_prefix(&format!("{LLVM_MAJOR}."))
            .is_none_or(str::is_empty)
        {
            return Err(format!(
                "unsupported exact LLVM version `{}`",
                self.llvm_version
            ));
        }
        for (field, entries) in [
            (
                "dependencies",
                self.dependencies
                    .iter()
                    .map(|entry| entry.identity.as_str())
                    .collect::<Vec<_>>(),
            ),
            (
                "exports",
                self.exports
                    .iter()
                    .map(|entry| entry.semantic_identity.as_str())
                    .collect::<Vec<_>>(),
            ),
            (
                "evidence",
                self.evidence
                    .iter()
                    .map(|entry| entry.identity.as_str())
                    .collect::<Vec<_>>(),
            ),
        ] {
            require_sorted_unique(field, &entries)?;
        }
        require_sorted_unique(
            "platform_requirements",
            &self
                .platform_requirements
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        )?;
        require_sorted_unique(
            "features",
            &self.features.iter().map(String::as_str).collect::<Vec<_>>(),
        )?;
        require_sorted_unique(
            "native_slices",
            &self
                .native_slices
                .iter()
                .map(|slice| slice.kind.as_str())
                .collect::<Vec<_>>(),
        )?;
        require_sorted_unique(
            "provenance",
            &self
                .provenance
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        )?;
        require_sorted_unique(
            "debug_prefix_map",
            &self
                .debug_prefix_map
                .iter()
                .map(|(source, _)| source.as_str())
                .collect::<Vec<_>>(),
        )?;
        if self.platform_requirements != ["topal.platform.linux-x86_64/1"] {
            return Err("native artifact lacks its exact Linux platform requirement".into());
        }
        if self.native_slices.is_empty()
            || self
                .native_slices
                .iter()
                .any(|slice| !matches!(slice.kind.as_str(), "llvm-ir" | "object" | "executable"))
        {
            return Err("native artifact contains an unsupported native slice set".into());
        }
        if self.source_name.is_empty() || !self.compiler.starts_with("topalc/") {
            return Err("native artifact source or compiler identity is empty".into());
        }
        for digest in self
            .dependencies
            .iter()
            .map(|entry| entry.sha256.as_str())
            .chain(self.evidence.iter().map(|entry| entry.sha256.as_str()))
            .chain(self.native_slices.iter().map(|entry| entry.sha256.as_str()))
            .chain([
                self.source_sha256.as_str(),
                self.interface_sha256.as_str(),
                self.build_identity.as_str(),
            ])
        {
            if digest.len() != 64
                || !digest
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            {
                return Err(format!("invalid SHA-256 digest `{digest}`"));
            }
        }
        Ok(())
    }

    /// Validate a selected native payload against its declared digest.
    ///
    /// # Errors
    ///
    /// Returns an error when the manifest has no unique matching slice or the
    /// payload digest differs.
    pub fn validate_slice(&self, kind: &str, payload: &[u8]) -> Result<(), String> {
        self.validate()?;
        let slices = self
            .native_slices
            .iter()
            .filter(|slice| slice.kind == kind)
            .collect::<Vec<_>>();
        let [slice] = slices.as_slice() else {
            return Err(format!(
                "native artifact must contain exactly one `{kind}` slice"
            ));
        };
        if slice.sha256 != sha256(payload) {
            return Err(format!(
                "native `{kind}` slice digest does not match payload"
            ));
        }
        Ok(())
    }

    /// Encode the one canonical JSON representation.
    ///
    /// # Errors
    ///
    /// Returns validation or serialization failure.
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut bytes = serde_json::to_vec_pretty(self).map_err(|error| error.to_string())?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    /// Decode and require canonical bytes.
    ///
    /// # Errors
    ///
    /// Rejects unknown fields, incompatible values, and noncanonical JSON.
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let metadata: Self = serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
        metadata.validate()?;
        if metadata.encode()? != bytes {
            return Err("native artifact metadata is not canonical".into());
        }
        Ok(metadata)
    }
}

fn require_sorted_unique(field: &str, values: &[&str]) -> Result<(), String> {
    if values.windows(2).any(|pair| pair[0] >= pair[1])
        || values.len() != values.iter().copied().collect::<BTreeSet<_>>().len()
    {
        return Err(format!("{field} must be strictly sorted and unique"));
    }
    Ok(())
}

#[must_use]
pub fn sha256(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(encoded, "{byte:02x}").expect("writing to a String cannot fail");
    }
    encoded
}

#[cfg(test)]
mod tests {
    use topal_language::analyze_for_compiler;

    use super::*;

    #[test]
    fn native_metadata_round_trips_only_in_canonical_form() {
        let program = analyze_for_compiler("use language (version is v0.1)\n()\n").unwrap();
        let metadata = NativeArtifactMetadata::for_program(
            &program,
            "unit.t",
            "llvm-ir",
            b"ir",
            "22.1.6-test",
        );
        let bytes = metadata.encode().unwrap();
        assert_eq!(NativeArtifactMetadata::decode(&bytes).unwrap(), metadata);
        assert!(metadata.validate_slice("llvm-ir", b"ir").is_ok());
        assert!(metadata.validate_slice("llvm-ir", b"changed").is_err());
        assert!(NativeArtifactMetadata::decode(bytes.strip_suffix(b"\n").unwrap()).is_err());
    }

    #[test]
    fn native_metadata_rejects_target_and_dependency_ambiguity() {
        let program = analyze_for_compiler("use language (version is v0.1)\n()\n").unwrap();
        let mut metadata = NativeArtifactMetadata::for_program(
            &program,
            "unit.t",
            "llvm-ir",
            b"ir",
            "22.1.6-test",
        );
        metadata.target_triple = "x86_64-pc-windows-msvc".into();
        assert!(metadata.validate().is_err());
        metadata.target_triple = TARGET_TRIPLE.into();
        metadata.native_abi = "topal-native/1".into();
        assert!(metadata.validate().is_err());
        metadata.native_abi = NATIVE_ABI.into();
        metadata.dependencies = vec![
            DigestEntry {
                identity: "same".into(),
                sha256: "0".repeat(64),
            },
            DigestEntry {
                identity: "same".into(),
                sha256: "1".repeat(64),
            },
        ];
        assert!(metadata.validate().is_err());
    }
}
