//! Versioned, target-qualified metadata shared by C interface producers and consumers.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::{Component, Path};

use serde::{Deserialize, Serialize};

pub const SCHEMA: &str = "topal-c-abi/1";
pub const SHARED_SCHEMA: &str = "topal-c-abi-shared/1";
pub const TARGET: &str = "x86_64-unknown-linux-gnu";
pub const PLATFORM_ABI: &str = "sysv-amd64";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CValue {
    Void,
    SignedInt32,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CEffect {
    NoObservableEffect,
}

impl CValue {
    #[must_use]
    pub const fn topal_type(self) -> &'static str {
        match self {
            Self::Void => "Unit",
            Self::SignedInt32 => "Int",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Parameter {
    pub name: String,
    pub value: CValue,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Function {
    pub topal_name: String,
    pub symbol: String,
    pub parameters: Vec<Parameter>,
    pub result: CValue,
    pub effect: CEffect,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InputArtifact {
    pub file: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SharedObjectArtifact {
    pub file: String,
    pub sha256: String,
    pub soname: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AccessLibrary {
    pub schema: String,
    pub identity: String,
    pub version: String,
    pub target: String,
    pub platform_abi: String,
    pub clang_version: String,
    pub header: InputArtifact,
    pub static_archive: InputArtifact,
    pub functions: Vec<Function>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SharedAccessLibrary {
    pub schema: String,
    pub identity: String,
    pub version: String,
    pub target: String,
    pub platform_abi: String,
    pub clang_version: String,
    pub header: InputArtifact,
    pub shared_object: SharedObjectArtifact,
    pub functions: Vec<Function>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccessLibraryManifest {
    Static(AccessLibrary),
    Shared(SharedAccessLibrary),
}

impl AccessLibraryManifest {
    /// Decode either version-one static or shared C access-library metadata.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed JSON and unknown schemas.
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let value: serde_json::Value = serde_json::from_slice(bytes)
            .map_err(|error| format!("invalid C access-library manifest: {error}"))?;
        match value.get("schema").and_then(serde_json::Value::as_str) {
            Some(SCHEMA) => AccessLibrary::decode(bytes).map(Self::Static),
            Some(SHARED_SCHEMA) => SharedAccessLibrary::decode(bytes).map(Self::Shared),
            Some(schema) => Err(format!("unsupported C access-library schema `{schema}`")),
            None => Err("C access-library manifest has no schema".into()),
        }
    }

    #[must_use]
    pub fn identity(&self) -> &str {
        match self {
            Self::Static(library) => &library.identity,
            Self::Shared(library) => &library.identity,
        }
    }

    #[must_use]
    pub fn header(&self) -> &InputArtifact {
        match self {
            Self::Static(library) => &library.header,
            Self::Shared(library) => &library.header,
        }
    }

    #[must_use]
    pub fn functions(&self) -> &[Function] {
        match self {
            Self::Static(library) => &library.functions,
            Self::Shared(library) => &library.functions,
        }
    }

    #[must_use]
    pub fn topal_source(&self) -> String {
        match self {
            Self::Static(library) => library.topal_source(),
            Self::Shared(library) => library.topal_source(),
        }
    }
}

impl AccessLibrary {
    /// Decode and validate an access-library manifest.
    ///
    /// # Errors
    ///
    /// Returns a stable description when JSON, schema, target, or declarations
    /// are invalid.
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let library: Self = serde_json::from_slice(bytes)
            .map_err(|error| format!("invalid C access-library manifest: {error}"))?;
        library.validate()?;
        Ok(library)
    }

    /// Encode a validated manifest in deterministic pretty JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if validation or JSON serialization fails.
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut bytes = serde_json::to_vec_pretty(self)
            .map_err(|error| format!("cannot encode C access-library manifest: {error}"))?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    /// Validate the closed first-version C ABI model.
    ///
    /// # Errors
    ///
    /// Rejects an incompatible schema or target and every ambiguous name.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != SCHEMA {
            return Err(format!(
                "unsupported C access-library schema `{}`; expected `{SCHEMA}`",
                self.schema
            ));
        }
        validate_artifact("header", &self.header)?;
        validate_artifact("static archive", &self.static_archive)?;
        validate_common(
            &self.identity,
            &self.version,
            &self.target,
            &self.platform_abi,
            &self.clang_version,
            &self.functions,
        )
    }

    /// Render the canonical human-readable Topal ABI access library.
    ///
    /// The JSON manifest is a machine transport for this same model. Consumers
    /// compare both representations before trusting either one.
    #[must_use]
    pub fn topal_source(&self) -> String {
        let mut source = format!(
            "use language (\n  version is v0.1,\n  features is ( abi )\n)\n# Generated checked C ABI access library. Do not edit independently of its manifest.\nabi-library is (\n  schema is \"{}\",\n  identity is \"{}\",\n  target is \"{}\",\n  platform-abi is \"{}\",\n  object-format is \"elf64-x86-64\",\n  binary-kind is \"static-archive\",\n  clang-version is \"{}\",\n  header-sha256 is \"{}\",\n  static-archive-sha256 is \"{}\"\n)\n",
            self.schema,
            self.identity,
            self.target,
            self.platform_abi,
            self.clang_version,
            self.header.sha256,
            self.static_archive.sha256
        );
        render_functions(&mut source, &self.functions);
        source
    }
}

impl SharedAccessLibrary {
    /// Decode and validate a shared access-library manifest.
    ///
    /// # Errors
    ///
    /// Returns a stable description when JSON, schema, target, or declarations
    /// are invalid.
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let library: Self = serde_json::from_slice(bytes)
            .map_err(|error| format!("invalid C access-library manifest: {error}"))?;
        library.validate()?;
        Ok(library)
    }

    /// Encode a validated manifest in deterministic pretty JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if validation or JSON serialization fails.
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut bytes = serde_json::to_vec_pretty(self)
            .map_err(|error| format!("cannot encode C access-library manifest: {error}"))?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    /// Validate the closed shared-library C ABI model.
    ///
    /// # Errors
    ///
    /// Rejects an incompatible schema, target, SONAME, or declaration.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != SHARED_SCHEMA {
            return Err(format!(
                "unsupported C access-library schema `{}`; expected `{SHARED_SCHEMA}`",
                self.schema
            ));
        }
        validate_artifact(
            "shared object",
            &InputArtifact {
                file: self.shared_object.file.clone(),
                sha256: self.shared_object.sha256.clone(),
            },
        )?;
        let path = Path::new(&self.shared_object.file);
        if path.components().count() != 1
            || path.file_name().and_then(|name| name.to_str())
                != Some(self.shared_object.soname.as_str())
            || !valid_soname(&self.shared_object.soname)
        {
            return Err(
                "C shared object must use one safe filename identical to its SONAME".into(),
            );
        }
        validate_artifact("header", &self.header)?;
        validate_common(
            &self.identity,
            &self.version,
            &self.target,
            &self.platform_abi,
            &self.clang_version,
            &self.functions,
        )
    }

    /// Render the canonical human-readable shared C ABI access library.
    #[must_use]
    pub fn topal_source(&self) -> String {
        let mut source = format!(
            "use language (\n  version is v0.1,\n  features is ( abi )\n)\n# Generated checked C ABI access library. Do not edit independently of its manifest.\nabi-library is (\n  schema is \"{}\",\n  identity is \"{}\",\n  target is \"{}\",\n  platform-abi is \"{}\",\n  object-format is \"elf64-x86-64\",\n  binary-kind is \"shared-object\",\n  clang-version is \"{}\",\n  header-sha256 is \"{}\",\n  shared-object-sha256 is \"{}\",\n  soname is \"{}\"\n)\n",
            self.schema,
            self.identity,
            self.target,
            self.platform_abi,
            self.clang_version,
            self.header.sha256,
            self.shared_object.sha256,
            self.shared_object.soname
        );
        render_functions(&mut source, &self.functions);
        source
    }
}

fn render_functions(source: &mut String, functions: &[Function]) {
    for (index, function) in functions.iter().enumerate() {
        let parameter_layouts = function
            .parameters
            .iter()
            .map(|parameter| c_value_name(parameter.value))
            .collect::<Vec<_>>()
            .join(",");
        let _ = write!(
            source,
            "abi-function-{} is (\n  topal-name is \"{}\",\n  symbol is \"{}\",\n  calling-convention is \"c\",\n  variadic is \"false\",\n  parameter-layouts is \"{}\",\n  result-layout is \"{}\",\n  effect is \"no-observable-effect\",\n  unwind is \"forbidden\",\n  transfer is \"copied-value\"\n)\n",
            index + 1,
            function.topal_name,
            function.symbol,
            parameter_layouts,
            c_value_name(function.result)
        );
        let parameters = function
            .parameters
            .iter()
            .map(|parameter| format!("{} : {}", parameter.name, parameter.value.topal_type()))
            .collect::<Vec<_>>()
            .join(", ");
        let _ = write!(
            source,
            "pub {} is fn ({parameters}) -> {}\n  {}\n",
            function.topal_name,
            function.result.topal_type(),
            if function.result == CValue::Void {
                "()"
            } else {
                "0"
            }
        );
    }
}

fn validate_common(
    identity: &str,
    version: &str,
    target: &str,
    platform_abi: &str,
    clang_version: &str,
    functions: &[Function],
) -> Result<(), String> {
    if target != TARGET || platform_abi != PLATFORM_ABI {
        return Err(format!(
            "C access library targets `{target}`/`{platform_abi}`; expected `{TARGET}`/`{PLATFORM_ABI}`"
        ));
    }
    if !valid_identifier(identity) || version != "v0.1" {
        return Err("C access-library identity must be a Topal identifier at version v0.1".into());
    }
    if clang_version.is_empty()
        || clang_version
            .bytes()
            .any(|byte| matches!(byte, b'"' | b'\\' | b'\n' | b'\r'))
    {
        return Err("C access-library Clang identity is empty or not source-safe".into());
    }
    if functions.is_empty() {
        return Err("C access library exports no supported functions".into());
    }
    let mut topal_names = BTreeSet::new();
    let mut symbols = BTreeSet::new();
    for function in functions {
        if !valid_identifier(&function.topal_name)
            || function.topal_name.starts_with("abi-")
            || !valid_c_symbol(&function.symbol)
            || !topal_names.insert(&function.topal_name)
            || !symbols.insert(&function.symbol)
        {
            return Err(format!(
                "C function `{}` has an invalid or duplicate Topal name or C symbol",
                function.topal_name
            ));
        }
        let mut parameters = BTreeSet::new();
        for parameter in &function.parameters {
            if parameter.value == CValue::Void
                || !valid_identifier(&parameter.name)
                || !parameters.insert(&parameter.name)
            {
                return Err(format!(
                    "C function `{}` has an invalid or duplicate parameter `{}`",
                    function.topal_name, parameter.name
                ));
            }
        }
    }
    Ok(())
}

const fn c_value_name(value: CValue) -> &'static str {
    match value {
        CValue::Void => "void",
        CValue::SignedInt32 => "signed-int-32",
    }
}

fn validate_artifact(kind: &str, artifact: &InputArtifact) -> Result<(), String> {
    let path = Path::new(&artifact.file);
    if artifact.file.is_empty()
        || path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
        || artifact.sha256.len() != 64
        || !artifact.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(format!(
            "C access-library {kind} must use a relative file and a SHA-256 digest"
        ));
    }
    Ok(())
}

fn valid_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

fn valid_c_symbol(value: &str) -> bool {
    if value == "_start" {
        return false;
    }
    let mut bytes = value.bytes();
    bytes
        .next()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || byte == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn valid_soname(value: &str) -> bool {
    value.starts_with("lib")
        && value.contains(".so")
        && !value.contains("..")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_void_parameters() {
        let library = AccessLibrary {
            schema: SCHEMA.into(),
            identity: "math".into(),
            version: "v0.1".into(),
            target: TARGET.into(),
            platform_abi: PLATFORM_ABI.into(),
            clang_version: "22.0.0".into(),
            header: artifact("math.h"),
            static_archive: artifact("libmath.a"),
            functions: vec![Function {
                topal_name: "clear".into(),
                symbol: "clear".into(),
                parameters: vec![Parameter {
                    name: "unused".into(),
                    value: CValue::Void,
                }],
                result: CValue::Void,
                effect: CEffect::NoObservableEffect,
            }],
        };
        assert!(library.validate().unwrap_err().contains("invalid"));
    }

    #[test]
    fn shared_schema_round_trips_with_canonical_soname() {
        let library = SharedAccessLibrary {
            schema: SHARED_SCHEMA.into(),
            identity: "math".into(),
            version: "v0.1".into(),
            target: TARGET.into(),
            platform_abi: PLATFORM_ABI.into(),
            clang_version: "clang version 22.0.0".into(),
            header: artifact("interface.h"),
            shared_object: SharedObjectArtifact {
                file: "libmath.so".into(),
                sha256: "1".repeat(64),
                soname: "libmath.so".into(),
            },
            functions: vec![Function {
                topal_name: "clear".into(),
                symbol: "clear".into(),
                parameters: Vec::new(),
                result: CValue::Void,
                effect: CEffect::NoObservableEffect,
            }],
        };
        let encoded = library.encode().unwrap();
        assert_eq!(
            AccessLibraryManifest::decode(&encoded).unwrap(),
            AccessLibraryManifest::Shared(library.clone())
        );
        assert!(library.topal_source().contains("soname is \"libmath.so\""));
    }

    fn artifact(file: &str) -> InputArtifact {
        InputArtifact {
            file: file.into(),
            sha256: "0".repeat(64),
        }
    }
}
