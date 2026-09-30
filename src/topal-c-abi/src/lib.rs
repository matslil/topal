//! Versioned, target-qualified metadata shared by C interface producers and consumers.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Component, Path};

pub const SCHEMA: &str = "topal-c-abi/1";
pub const SHARED_SCHEMA: &str = "topal-c-abi-shared/1";
pub const TARGET: &str = "x86_64-unknown-linux-gnu";
pub const PLATFORM_ABI: &str = "sysv-amd64";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CValue {
    Void,
    SignedInt32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub value: CValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Function {
    pub topal_name: String,
    pub symbol: String,
    pub parameters: Vec<Parameter>,
    pub result: CValue,
    pub effect: CEffect,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputArtifact {
    pub file: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SharedObjectArtifact {
    pub file: String,
    pub sha256: String,
    pub soname: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
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
pub enum AccessLibraryModel {
    Static(AccessLibrary),
    Shared(SharedAccessLibrary),
}

impl AccessLibraryModel {
    /// Decode the canonical Topal source for a static or shared C access library.
    ///
    /// # Errors
    ///
    /// Returns an error for malformed, incomplete, or noncanonical source.
    pub fn decode_topal(source: &str) -> Result<Self, String> {
        let (mut library, function_records) = parse_abi_records(source)?;
        let schema = take(&mut library, "schema")?;
        let identity = take(&mut library, "identity")?;
        let version = take(&mut library, "version")?;
        let target = take(&mut library, "target")?;
        let platform_abi = take(&mut library, "platform-abi")?;
        require(&mut library, "object-format", "elf64-x86-64")?;
        let binary_kind = take(&mut library, "binary-kind")?;
        let clang_version = take(&mut library, "clang-version")?;
        let header = InputArtifact {
            file: take(&mut library, "header-file")?,
            sha256: take(&mut library, "header-sha256")?,
        };
        let functions = function_records
            .into_iter()
            .map(decode_function)
            .collect::<Result<Vec<_>, _>>()?;
        let model = match binary_kind.as_str() {
            "static-archive" => Self::Static(AccessLibrary {
                schema,
                identity,
                version,
                target,
                platform_abi,
                clang_version,
                header,
                static_archive: InputArtifact {
                    file: take(&mut library, "static-archive-file")?,
                    sha256: take(&mut library, "static-archive-sha256")?,
                },
                functions,
            }),
            "shared-object" => Self::Shared(SharedAccessLibrary {
                schema,
                identity,
                version,
                target,
                platform_abi,
                clang_version,
                header,
                shared_object: SharedObjectArtifact {
                    file: take(&mut library, "shared-object-file")?,
                    sha256: take(&mut library, "shared-object-sha256")?,
                    soname: take(&mut library, "soname")?,
                },
                functions,
            }),
            other => {
                return Err(format!(
                    "unsupported C access-library binary kind `{other}`"
                ));
            }
        };
        if let Some(name) = library.keys().next() {
            return Err(format!("unknown C access-library field `{name}`"));
        }
        match &model {
            Self::Static(library) => library.validate()?,
            Self::Shared(library) => library.validate()?,
        }
        if model.topal_source() != source {
            return Err("C access-library module is not in canonical generated form".into());
        }
        Ok(model)
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
    /// Consumers decode and validate this source directly; there is no sidecar
    /// representation which can disagree with it.
    #[must_use]
    pub fn topal_source(&self) -> String {
        let mut source = format!(
            "use language (\n  version is v0.1,\n  features is ( abi )\n)\n# Generated checked C ABI access library. The compiler requires this canonical form.\nabi-library is (\n  schema is \"{}\",\n  identity is \"{}\",\n  version is \"{}\",\n  target is \"{}\",\n  platform-abi is \"{}\",\n  object-format is \"elf64-x86-64\",\n  binary-kind is \"static-archive\",\n  clang-version is \"{}\",\n  header-file is \"{}\",\n  header-sha256 is \"{}\",\n  static-archive-file is \"{}\",\n  static-archive-sha256 is \"{}\"\n)\n",
            self.schema,
            self.identity,
            self.version,
            self.target,
            self.platform_abi,
            self.clang_version,
            self.header.file,
            self.header.sha256,
            self.static_archive.file,
            self.static_archive.sha256
        );
        render_functions(&mut source, &self.functions);
        source
    }
}

impl SharedAccessLibrary {
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
            "use language (\n  version is v0.1,\n  features is ( abi )\n)\n# Generated checked C ABI access library. The compiler requires this canonical form.\nabi-library is (\n  schema is \"{}\",\n  identity is \"{}\",\n  version is \"{}\",\n  target is \"{}\",\n  platform-abi is \"{}\",\n  object-format is \"elf64-x86-64\",\n  binary-kind is \"shared-object\",\n  clang-version is \"{}\",\n  header-file is \"{}\",\n  header-sha256 is \"{}\",\n  shared-object-file is \"{}\",\n  shared-object-sha256 is \"{}\",\n  soname is \"{}\"\n)\n",
            self.schema,
            self.identity,
            self.version,
            self.target,
            self.platform_abi,
            self.clang_version,
            self.header.file,
            self.header.sha256,
            self.shared_object.file,
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
        let parameter_names = function
            .parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
            .collect::<Vec<_>>()
            .join(",");
        let _ = write!(
            source,
            "abi-function-{} is (\n  topal-name is \"{}\",\n  symbol is \"{}\",\n  calling-convention is \"c\",\n  variadic is \"false\",\n  parameter-names is \"{}\",\n  parameter-layouts is \"{}\",\n  result-layout is \"{}\",\n  effect is \"no-observable-effect\",\n  unwind is \"forbidden\",\n  transfer is \"copied-value\"\n)\n",
            index + 1,
            function.topal_name,
            function.symbol,
            parameter_names,
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

type AbiRecord = BTreeMap<String, String>;

fn parse_abi_records(source: &str) -> Result<(AbiRecord, Vec<AbiRecord>), String> {
    let lines = source.lines().collect::<Vec<_>>();
    let mut library = None;
    let mut functions = BTreeMap::new();
    let mut index = 0;
    while index < lines.len() {
        if lines[index] == "abi-library is (" {
            if library.is_some() {
                return Err("C access-library module has duplicate `abi-library` records".into());
            }
            library = Some(parse_record(&lines, &mut index)?);
        } else if let Some(number) = lines[index]
            .strip_prefix("abi-function-")
            .and_then(|line| line.strip_suffix(" is ("))
        {
            let number = number
                .parse::<usize>()
                .map_err(|_| "C access-library function record has an invalid index")?;
            let record = parse_record(&lines, &mut index)?;
            if functions.insert(number, record).is_some() {
                return Err(format!(
                    "C access-library module has duplicate `abi-function-{number}` records"
                ));
            }
        } else {
            index += 1;
        }
    }
    let library = library.ok_or("C access-library module has no `abi-library` record")?;
    let mut ordered = Vec::with_capacity(functions.len());
    for expected in 1..=functions.len() {
        ordered.push(functions.remove(&expected).ok_or_else(|| {
            format!("C access-library module has no `abi-function-{expected}` record")
        })?);
    }
    Ok((library, ordered))
}

fn parse_record(lines: &[&str], index: &mut usize) -> Result<AbiRecord, String> {
    *index += 1;
    let mut fields = BTreeMap::new();
    while *index < lines.len() && lines[*index] != ")" {
        let line = lines[*index]
            .strip_prefix("  ")
            .ok_or("C access-library record field has invalid indentation")?;
        let line = line.strip_suffix(',').unwrap_or(line);
        let (name, value) = line
            .split_once(" is \"")
            .ok_or("C access-library record field is malformed")?;
        let value = value
            .strip_suffix('"')
            .ok_or("C access-library record field is not a string")?;
        if name.is_empty() || fields.insert(name.to_owned(), value.to_owned()).is_some() {
            return Err(format!(
                "C access-library record has invalid or duplicate field `{name}`"
            ));
        }
        *index += 1;
    }
    if *index == lines.len() {
        return Err("C access-library record is not closed".into());
    }
    *index += 1;
    Ok(fields)
}

fn decode_function(mut fields: AbiRecord) -> Result<Function, String> {
    let topal_name = take(&mut fields, "topal-name")?;
    let symbol = take(&mut fields, "symbol")?;
    require(&mut fields, "calling-convention", "c")?;
    require(&mut fields, "variadic", "false")?;
    let names = comma_values(&take(&mut fields, "parameter-names")?);
    let layouts = comma_values(&take(&mut fields, "parameter-layouts")?);
    if names.len() != layouts.len() {
        return Err(format!(
            "C function `{topal_name}` has different parameter name and layout counts"
        ));
    }
    let parameters = names
        .into_iter()
        .zip(layouts)
        .map(|(name, layout)| {
            Ok(Parameter {
                name,
                value: decode_c_value(&layout)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let result = decode_c_value(&take(&mut fields, "result-layout")?)?;
    require(&mut fields, "effect", "no-observable-effect")?;
    require(&mut fields, "unwind", "forbidden")?;
    require(&mut fields, "transfer", "copied-value")?;
    if let Some(name) = fields.keys().next() {
        return Err(format!("unknown C function field `{name}`"));
    }
    Ok(Function {
        topal_name,
        symbol,
        parameters,
        result,
        effect: CEffect::NoObservableEffect,
    })
}

fn comma_values(value: &str) -> Vec<String> {
    if value.is_empty() {
        Vec::new()
    } else {
        value.split(',').map(str::to_owned).collect()
    }
}

fn decode_c_value(value: &str) -> Result<CValue, String> {
    match value {
        "void" => Ok(CValue::Void),
        "signed-int-32" => Ok(CValue::SignedInt32),
        other => Err(format!("unsupported C ABI value layout `{other}`")),
    }
}

fn take(fields: &mut AbiRecord, name: &str) -> Result<String, String> {
    fields
        .remove(name)
        .ok_or_else(|| format!("C access-library record has no `{name}` field"))
}

fn require(fields: &mut AbiRecord, name: &str, expected: &str) -> Result<(), String> {
    let actual = take(fields, name)?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "C access-library field `{name}` is `{actual}`; expected `{expected}`"
        ))
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
        || artifact
            .file
            .bytes()
            .any(|byte| matches!(byte, b'"' | b'\\' | b'\n' | b'\r'))
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
    fn shared_schema_round_trips_through_canonical_topal_source() {
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
        assert_eq!(
            AccessLibraryModel::decode_topal(&library.topal_source()).unwrap(),
            AccessLibraryModel::Shared(library.clone())
        );
        assert!(library.topal_source().contains("soname is \"libmath.so\""));
    }

    #[test]
    fn rejects_noncanonical_or_incomplete_topal_metadata() {
        let library = AccessLibrary {
            schema: SCHEMA.into(),
            identity: "math".into(),
            version: "v0.1".into(),
            target: TARGET.into(),
            platform_abi: PLATFORM_ABI.into(),
            clang_version: "clang version 22.0.0".into(),
            header: artifact("interface.h"),
            static_archive: artifact("libmath.a"),
            functions: vec![Function {
                topal_name: "clear".into(),
                symbol: "clear".into(),
                parameters: Vec::new(),
                result: CValue::Void,
                effect: CEffect::NoObservableEffect,
            }],
        };
        let source = library.topal_source();
        assert_eq!(
            AccessLibraryModel::decode_topal(&source).unwrap(),
            AccessLibraryModel::Static(library)
        );
        assert!(
            AccessLibraryModel::decode_topal(&source.replace("  header-file", " header-file"))
                .unwrap_err()
                .contains("indentation")
        );
    }

    fn artifact(file: &str) -> InputArtifact {
        InputArtifact {
            file: file.into(),
            sha256: "0".repeat(64),
        }
    }
}
