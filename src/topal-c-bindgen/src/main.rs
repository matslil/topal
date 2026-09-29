//! Generate a checked Topal access library from a C header and ELF binary.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Output};

use serde_json::Value;
use sha2::{Digest, Sha256};
use topal_c_abi::{
    AccessLibrary, CEffect, CValue, Function, InputArtifact, PLATFORM_ABI, Parameter, SCHEMA,
    SHARED_SCHEMA, SharedAccessLibrary, SharedObjectArtifact, TARGET,
};

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
    let arguments = Arguments::parse(env::args().skip(1))?;
    if arguments.output.exists() {
        return Err(format!(
            "output directory {} already exists",
            arguments.output.display()
        ));
    }
    let clang_version = clang_version(&arguments.clang)?;
    let ast = clang_ast(&arguments)?;
    let functions = extract_functions(&ast)?;
    let verified_binary = verify_binary(&arguments.llvm_tools, &arguments.binary, &functions)?;

    let parent = arguments.output.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    let staging = parent.join(format!(
        ".topal-c-bindgen-{}-{}",
        std::process::id(),
        arguments.identity
    ));
    fs::create_dir(&staging)
        .map_err(|error| format!("cannot create {}: {error}", staging.display()))?;
    let result = generate(
        &arguments,
        &clang_version,
        functions,
        &verified_binary,
        &staging,
    );
    if result.is_ok() {
        fs::rename(&staging, &arguments.output).map_err(|error| {
            format!(
                "cannot publish {} as {}: {error}",
                staging.display(),
                arguments.output.display()
            )
        })?;
    } else {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

struct Arguments {
    identity: String,
    header: PathBuf,
    binary: BinaryInput,
    output: PathBuf,
    clang: PathBuf,
    llvm_tools: PathBuf,
    extra_clang_args: Vec<String>,
}

enum BinaryInput {
    StaticArchive(PathBuf),
    SharedObject(PathBuf),
}

enum VerifiedBinary {
    StaticArchive,
    SharedObject { soname: String },
}

impl Arguments {
    fn parse(arguments: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut identity = None;
        let mut header = None;
        let mut archive = None;
        let mut shared_object = None;
        let mut output = None;
        let mut clang = None;
        let mut llvm_tools = None;
        let mut extra_clang_args = Vec::new();
        let mut arguments = arguments.peekable();
        while let Some(argument) = arguments.next() {
            let value = |arguments: &mut std::iter::Peekable<_>, option: &str| {
                arguments
                    .next()
                    .ok_or_else(|| format!("{option} requires a value"))
            };
            match argument.as_str() {
                "--library" => identity = Some(value(&mut arguments, "--library")?),
                "--header" => header = Some(PathBuf::from(value(&mut arguments, "--header")?)),
                "--archive" => {
                    archive = Some(PathBuf::from(value(&mut arguments, "--archive")?));
                }
                "--shared-object" => {
                    shared_object = Some(PathBuf::from(value(&mut arguments, "--shared-object")?));
                }
                "--output" => output = Some(PathBuf::from(value(&mut arguments, "--output")?)),
                "--clang" => clang = Some(PathBuf::from(value(&mut arguments, "--clang")?)),
                "--llvm-tools" => {
                    llvm_tools = Some(PathBuf::from(value(&mut arguments, "--llvm-tools")?));
                }
                "--clang-arg" => extra_clang_args.push(value(&mut arguments, "--clang-arg")?),
                "--help" | "-h" => {
                    println!(
                        "Usage: topal-c-bindgen --library NAME --header FILE (--archive FILE | --shared-object FILE) --output DIR --clang CLANG-22 --llvm-tools DIR [--clang-arg ARG]"
                    );
                    std::process::exit(0);
                }
                _ => return Err(format!("unknown argument `{argument}`")),
            }
        }
        let binary = match (archive, shared_object) {
            (Some(path), None) => BinaryInput::StaticArchive(path),
            (None, Some(path)) => BinaryInput::SharedObject(path),
            _ => return Err("exactly one of --archive or --shared-object is required".into()),
        };
        Ok(Self {
            identity: identity.ok_or("--library is required")?,
            header: header.ok_or("--header is required")?,
            binary,
            output: output.ok_or("--output is required")?,
            clang: clang.ok_or("--clang is required")?,
            llvm_tools: llvm_tools.ok_or("--llvm-tools is required")?,
            extra_clang_args,
        })
    }
}

fn clang_version(clang: &Path) -> Result<String, String> {
    let output = command_output(Command::new(clang).arg("--version"))?;
    let version = String::from_utf8_lossy(&output.stdout);
    let first = version.lines().next().unwrap_or_default().trim();
    let major = first
        .split_whitespace()
        .find_map(|part| part.split('.').next().filter(|part| *part == "22"));
    if major.is_none() {
        return Err(format!(
            "topal-c-bindgen requires Clang 22; `{}` reported `{first}`",
            clang.display()
        ));
    }
    Ok(first.to_owned())
}

fn clang_ast(arguments: &Arguments) -> Result<Value, String> {
    let mut command = Command::new(&arguments.clang);
    command
        .arg("--target")
        .arg(TARGET)
        .args([
            "-x",
            "c-header",
            "-std=c17",
            "-ffreestanding",
            "-fsyntax-only",
        ])
        .args(&arguments.extra_clang_args)
        .args(["-Xclang", "-ast-dump=json"])
        .arg(&arguments.header);
    let output = command_output(&mut command)?;
    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("Clang returned an invalid JSON AST: {error}"))
}

fn extract_functions(ast: &Value) -> Result<Vec<Function>, String> {
    let mut declarations = Vec::new();
    visit_declarations(ast, &mut declarations)?;
    let mut by_symbol = BTreeMap::new();
    for function in declarations {
        if let Some(previous) = by_symbol.insert(function.symbol.clone(), function.clone())
            && previous != function
        {
            return Err(format!(
                "C symbol `{}` has conflicting declarations",
                function.symbol
            ));
        }
    }
    let functions = by_symbol.into_values().collect::<Vec<_>>();
    if functions.is_empty() {
        return Err("the C header declares no supported functions".into());
    }
    Ok(functions)
}

fn visit_declarations(node: &Value, functions: &mut Vec<Function>) -> Result<(), String> {
    if node.get("kind").and_then(Value::as_str) == Some("FunctionDecl")
        && !node
            .get("isImplicit")
            .and_then(Value::as_bool)
            .unwrap_or(false)
    {
        functions.push(parse_function(node)?);
        return Ok(());
    }
    if let Some(children) = node.get("inner").and_then(Value::as_array) {
        for child in children {
            visit_declarations(child, functions)?;
        }
    }
    Ok(())
}

fn parse_function(node: &Value) -> Result<Function, String> {
    let symbol = node
        .get("name")
        .and_then(Value::as_str)
        .ok_or("Clang FunctionDecl has no name")?;
    if node
        .get("variadic")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Err(format!(
            "C function `{symbol}` is variadic; variadics are not supported"
        ));
    }
    let has_const_contract = node
        .get("inner")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .any(|child| child.get("kind").and_then(Value::as_str) == Some("ConstAttr"));
    if !has_const_contract {
        return Err(format!(
            "C function `{symbol}` has no Clang `const` effect contract; the first ABI slice admits only functions declared `__attribute__((const))`"
        ));
    }
    let function_type = node
        .pointer("/type/qualType")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("C function `{symbol}` has no qualified type"))?;
    let result = function_type
        .split_once('(')
        .map(|(result, _)| result.trim())
        .ok_or_else(|| format!("C function `{symbol}` has an invalid type `{function_type}`"))?;
    let result = c_value(result, true, symbol)?;
    let mut parameters = Vec::new();
    for (index, parameter) in node
        .get("inner")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|child| child.get("kind").and_then(Value::as_str) == Some("ParmVarDecl"))
        .enumerate()
    {
        let name = parameter
            .get("name")
            .and_then(Value::as_str)
            .filter(|name| !name.is_empty())
            .map_or_else(|| format!("argument-{}", index + 1), topal_identifier);
        let value = parameter
            .pointer("/type/qualType")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("C function `{symbol}` parameter has no qualified type"))?;
        parameters.push(Parameter {
            name,
            value: c_value(value, false, symbol)?,
        });
    }
    Ok(Function {
        topal_name: topal_identifier(symbol),
        symbol: symbol.to_owned(),
        parameters,
        result,
        effect: CEffect::NoObservableEffect,
    })
}

fn c_value(spelling: &str, result: bool, function: &str) -> Result<CValue, String> {
    match spelling.trim() {
        "void" if result => Ok(CValue::Void),
        "int" => Ok(CValue::SignedInt32),
        unsupported => Err(format!(
            "C function `{function}` uses unsupported type `{unsupported}`; the first ABI slice supports only `int` parameters and `int` or `void` results"
        )),
    }
}

fn topal_identifier(symbol: &str) -> String {
    symbol.replace('_', "-")
}

fn verify_binary(
    tools: &Path,
    binary: &BinaryInput,
    functions: &[Function],
) -> Result<VerifiedBinary, String> {
    match binary {
        BinaryInput::StaticArchive(archive) => {
            verify_archive(tools, archive, functions)?;
            Ok(VerifiedBinary::StaticArchive)
        }
        BinaryInput::SharedObject(shared_object) => {
            let soname = verify_shared_object(tools, shared_object, functions)?;
            Ok(VerifiedBinary::SharedObject { soname })
        }
    }
}

fn verify_archive(tools: &Path, archive: &Path, functions: &[Function]) -> Result<(), String> {
    let readobj = command_output(
        Command::new(tools.join("llvm-readobj"))
            .arg("--file-headers")
            .arg(archive),
    )?;
    let description = String::from_utf8_lossy(&readobj.stdout);
    if !description.contains("elf64-x86-64") && !description.contains("x86_64") {
        return Err(format!(
            "static archive {} does not contain qualified x86-64 ELF objects",
            archive.display()
        ));
    }
    let symbols = command_output(
        Command::new(tools.join("llvm-nm"))
            .args(["--defined-only", "--extern-only"])
            .arg(archive),
    )?;
    let symbols = String::from_utf8_lossy(&symbols.stdout)
        .split_whitespace()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    for function in functions {
        if !symbols.contains(&function.symbol) {
            return Err(format!(
                "static archive {} does not define C symbol `{}`",
                archive.display(),
                function.symbol
            ));
        }
    }
    Ok(())
}

fn verify_shared_object(
    tools: &Path,
    shared_object: &Path,
    functions: &[Function],
) -> Result<String, String> {
    let readobj = command_output(
        Command::new(tools.join("llvm-readobj"))
            .args([
                "--file-header",
                "--program-headers",
                "--dynamic-table",
                "--needed-libs",
            ])
            .arg(shared_object),
    )?;
    let description = String::from_utf8_lossy(&readobj.stdout);
    if !description.contains("Format: elf64-x86-64") || !description.contains("Type: SharedObject")
    {
        return Err(format!(
            "shared object {} is not a qualified x86-64 ELF shared object",
            shared_object.display()
        ));
    }
    let soname = description
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("0x000000000000000E SONAME")
                .and_then(|suffix| suffix.split('[').nth(1))
                .and_then(|suffix| suffix.strip_suffix(']'))
                .map(str::to_owned)
        })
        .ok_or_else(|| {
            format!(
                "shared object {} has no explicit SONAME",
                shared_object.display()
            )
        })?;
    let needed = description
        .split_once("NeededLibraries [")
        .and_then(|(_, suffix)| suffix.split_once(']'))
        .map_or("", |(entries, _)| entries)
        .trim();
    if !needed.is_empty() {
        return Err(format!(
            "shared object {} has undeclared dynamic dependencies: {}",
            shared_object.display(),
            needed.replace('\n', ", ")
        ));
    }
    if [" INIT ", " INIT_ARRAY ", " FINI ", " FINI_ARRAY "]
        .iter()
        .any(|tag| description.contains(tag))
    {
        return Err(format!(
            "shared object {} contains a load or unload initializer",
            shared_object.display()
        ));
    }
    let undefined = command_output(
        Command::new(tools.join("llvm-nm"))
            .args(["--undefined-only", "--extern-only"])
            .arg(shared_object),
    )?;
    if !String::from_utf8_lossy(&undefined.stdout).trim().is_empty() {
        return Err(format!(
            "shared object {} contains unresolved external symbols",
            shared_object.display()
        ));
    }
    verify_symbols(tools, shared_object, functions, "shared object")?;
    Ok(soname)
}

fn verify_symbols(
    tools: &Path,
    binary: &Path,
    functions: &[Function],
    kind: &str,
) -> Result<(), String> {
    let symbols = command_output(
        Command::new(tools.join("llvm-nm"))
            .args(["--defined-only", "--extern-only"])
            .arg(binary),
    )?;
    let symbols = String::from_utf8_lossy(&symbols.stdout)
        .split_whitespace()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    for function in functions {
        if !symbols.contains(&function.symbol) {
            return Err(format!(
                "{kind} {} does not define C symbol `{}`",
                binary.display(),
                function.symbol
            ));
        }
    }
    Ok(())
}

fn generate(
    arguments: &Arguments,
    clang_version: &str,
    functions: Vec<Function>,
    verified_binary: &VerifiedBinary,
    staging: &Path,
) -> Result<(), String> {
    let header_file = "interface.h";
    let header = fs::read(&arguments.header)
        .map_err(|error| format!("cannot read {}: {error}", arguments.header.display()))?;
    let binary_path = match &arguments.binary {
        BinaryInput::StaticArchive(path) | BinaryInput::SharedObject(path) => path,
    };
    let binary = fs::read(binary_path)
        .map_err(|error| format!("cannot read {}: {error}", binary_path.display()))?;
    fs::write(staging.join(header_file), &header)
        .map_err(|error| format!("cannot copy C header: {error}"))?;
    let header_artifact = InputArtifact {
        file: header_file.into(),
        sha256: digest(&header),
    };
    let (binary_file, source, manifest) = match (&arguments.binary, verified_binary) {
        (BinaryInput::StaticArchive(_), VerifiedBinary::StaticArchive) => {
            let binary_file = format!("lib{}.a", arguments.identity);
            let library = AccessLibrary {
                schema: SCHEMA.into(),
                identity: arguments.identity.clone(),
                version: "v0.1".into(),
                target: TARGET.into(),
                platform_abi: PLATFORM_ABI.into(),
                clang_version: clang_version.into(),
                header: header_artifact,
                static_archive: InputArtifact {
                    file: binary_file.clone(),
                    sha256: digest(&binary),
                },
                functions,
            };
            (binary_file, library.topal_source(), library.encode()?)
        }
        (BinaryInput::SharedObject(_), VerifiedBinary::SharedObject { soname }) => {
            let library = SharedAccessLibrary {
                schema: SHARED_SCHEMA.into(),
                identity: arguments.identity.clone(),
                version: "v0.1".into(),
                target: TARGET.into(),
                platform_abi: PLATFORM_ABI.into(),
                clang_version: clang_version.into(),
                header: header_artifact,
                shared_object: SharedObjectArtifact {
                    file: soname.clone(),
                    sha256: digest(&binary),
                    soname: soname.clone(),
                },
                functions,
            };
            (soname.clone(), library.topal_source(), library.encode()?)
        }
        _ => return Err("internal C binary verification mismatch".into()),
    };
    fs::write(staging.join(&binary_file), &binary)
        .map_err(|error| format!("cannot copy C binary: {error}"))?;
    fs::write(staging.join("module.t"), source)
        .map_err(|error| format!("cannot write generated Topal access library: {error}"))?;
    fs::write(staging.join("module.topal-c-abi.json"), manifest)
        .map_err(|error| format!("cannot write C access-library manifest: {error}"))?;
    Ok(())
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn command_output(command: &mut Command) -> Result<Output, String> {
    let display = format!("{command:?}");
    let output = command
        .output()
        .map_err(|error| format!("cannot execute {display}: {error}"))?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(format!(
            "command failed ({display}):\n{}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_closed_int_function_subset() {
        let ast: Value = serde_json::from_str(
            r#"{"kind":"TranslationUnitDecl","inner":[{"kind":"FunctionDecl","name":"add_i32","type":{"qualType":"int (int, int)"},"inner":[{"kind":"ParmVarDecl","name":"left","type":{"qualType":"int"}},{"kind":"ParmVarDecl","name":"right","type":{"qualType":"int"}},{"kind":"ConstAttr"}]}]}"#,
        )
        .unwrap();
        let functions = extract_functions(&ast).unwrap();
        assert_eq!(functions[0].topal_name, "add-i32");
        assert_eq!(functions[0].parameters.len(), 2);
        assert_eq!(functions[0].result, CValue::SignedInt32);
    }

    #[test]
    fn rejects_pointer_without_guessing_ownership() {
        let ast: Value = serde_json::from_str(
            r#"{"kind":"FunctionDecl","name":"read","type":{"qualType":"int (int *)"},"inner":[{"kind":"ParmVarDecl","name":"value","type":{"qualType":"int *"}},{"kind":"ConstAttr"}]}"#,
        )
        .unwrap();
        assert!(extract_functions(&ast).unwrap_err().contains("int *"));
    }

    #[test]
    fn rejects_a_function_without_an_explicit_effect_contract() {
        let ast: Value = serde_json::from_str(
            r#"{"kind":"FunctionDecl","name":"stateful","type":{"qualType":"int (int)"},"inner":[{"kind":"ParmVarDecl","name":"value","type":{"qualType":"int"}}]}"#,
        )
        .unwrap();
        assert!(
            extract_functions(&ast)
                .unwrap_err()
                .contains("no Clang `const` effect contract")
        );
    }

    #[test]
    fn renders_the_complete_topal_abi_model() {
        let library = AccessLibrary {
            schema: SCHEMA.into(),
            identity: "math".into(),
            version: "v0.1".into(),
            target: TARGET.into(),
            platform_abi: PLATFORM_ABI.into(),
            clang_version: "clang version 22.0.0".into(),
            header: InputArtifact {
                file: "interface.h".into(),
                sha256: "1".repeat(64),
            },
            static_archive: InputArtifact {
                file: "libmath.a".into(),
                sha256: "2".repeat(64),
            },
            functions: vec![Function {
                topal_name: "clear".into(),
                symbol: "clear".into(),
                parameters: Vec::new(),
                result: CValue::Void,
                effect: CEffect::NoObservableEffect,
            }],
        };
        let source = library.topal_source();
        assert!(source.contains("features is ( abi )"));
        assert!(source.contains("platform-abi is \"sysv-amd64\""));
        assert!(source.contains("result-layout is \"void\""));
        assert!(source.contains("effect is \"no-observable-effect\""));
        assert!(source.contains("pub clear is fn () -> Unit"));
    }
}
