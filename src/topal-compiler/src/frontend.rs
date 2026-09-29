//! Source discovery and checked-program construction.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use topal_c_abi::{AccessLibrary, CValue};
use topal_language::compiler::{
    CompilerCFunction, CompilerCValue, CompilerProgram, CompilerType,
    analyze_for_compiler_with_modules,
};
use topal_language::modules::{declared_libraries, select_source_modules};

use crate::CompileError;
use crate::DigestEntry;

const BUILT_IN_LIBRARIES: &[&str] = &["advent-of-code", "std"];

/// A program accepted by the shared compiler frontend.
///
/// Keeping this wrapper private to the compiler pipeline prevents unchecked
/// source or raw module collections from reaching a native backend stage.
pub(crate) struct CheckedProgram {
    program: CompilerProgram,
    static_archives: Vec<PathBuf>,
    foreign_dependencies: Vec<DigestEntry>,
}

impl CheckedProgram {
    pub(crate) fn program(&self) -> &CompilerProgram {
        &self.program
    }

    pub(crate) fn static_archives(&self) -> &[PathBuf] {
        &self.static_archives
    }

    pub(crate) fn foreign_dependencies(&self) -> &[DigestEntry] {
        &self.foreign_dependencies
    }
}

/// Discover referenced modules and construct the checked compiler model.
pub(crate) fn check(source: &str, library_root: &Path) -> Result<CheckedProgram, CompileError> {
    let mut libraries = BUILT_IN_LIBRARIES
        .iter()
        .map(|identity| (*identity).to_owned())
        .collect::<Vec<_>>();
    for identity in declared_libraries(source) {
        if !libraries.contains(&identity) {
            libraries.push(identity);
        }
    }
    let library_names = libraries.iter().map(String::as_str).collect::<Vec<_>>();
    let modules = select_source_modules(source, library_root, &library_names)
        .map_err(|error| CompileError::Io(error.to_string()))?;
    let access_libraries = load_access_libraries(&modules)?;
    let mut program =
        analyze_for_compiler_with_modules(source, &modules).map_err(CompileError::Diagnostic)?;
    attach_foreign_functions(&mut program, &access_libraries)?;
    let static_archives = access_libraries
        .values()
        .map(|(_, archive)| archive.clone())
        .collect();
    let mut foreign_dependencies = access_libraries
        .values()
        .flat_map(|(library, _)| {
            [
                DigestEntry {
                    identity: format!("c-abi.{}.header", library.identity),
                    sha256: library.header.sha256.clone(),
                },
                DigestEntry {
                    identity: format!("c-abi.{}.static-archive", library.identity),
                    sha256: library.static_archive.sha256.clone(),
                },
            ]
        })
        .collect::<Vec<_>>();
    foreign_dependencies.sort_by(|left, right| left.identity.cmp(&right.identity));
    Ok(CheckedProgram {
        program,
        static_archives,
        foreign_dependencies,
    })
}

fn load_access_libraries(
    modules: &[topal_language::modules::SourceModule],
) -> Result<BTreeMap<String, (AccessLibrary, PathBuf)>, CompileError> {
    let mut libraries = BTreeMap::new();
    for module in modules {
        if !module.source.contains("features is ( abi )") {
            continue;
        }
        let source_path = Path::new(&module.source_name);
        let manifest_path = source_path.with_file_name("module.topal-c-abi.json");
        let bytes = fs::read(&manifest_path).map_err(|error| {
            CompileError::Io(format!(
                "cannot read C access-library manifest {}: {error}",
                manifest_path.display()
            ))
        })?;
        let manifest = AccessLibrary::decode(&bytes).map_err(CompileError::Tool)?;
        if module.source != manifest.topal_source() {
            return Err(CompileError::Tool(format!(
                "Topal ABI source {} is not the canonical rendering of {}",
                source_path.display(),
                manifest_path.display()
            )));
        }
        let identity = module.identity.first().ok_or_else(|| {
            CompileError::Tool("C access-library module has no library identity".into())
        })?;
        if &manifest.identity != identity {
            return Err(CompileError::Tool(format!(
                "C manifest identity `{}` does not match selected library `{identity}`",
                manifest.identity
            )));
        }
        let directory = manifest_path.parent().unwrap_or_else(|| Path::new("."));
        verify_artifact(directory, &manifest.header.file, &manifest.header.sha256)?;
        let archive = verify_artifact(
            directory,
            &manifest.static_archive.file,
            &manifest.static_archive.sha256,
        )?;
        if libraries
            .insert(identity.clone(), (manifest, archive))
            .is_some()
        {
            return Err(CompileError::Tool(format!(
                "C access library `{identity}` has more than one selected ABI module"
            )));
        }
    }
    Ok(libraries)
}

fn verify_artifact(directory: &Path, file: &str, expected: &str) -> Result<PathBuf, CompileError> {
    let path = directory.join(file);
    let canonical_directory = fs::canonicalize(directory).map_err(|error| {
        CompileError::Io(format!(
            "cannot resolve C ABI directory {}: {error}",
            directory.display()
        ))
    })?;
    let canonical_path = fs::canonicalize(&path).map_err(|error| {
        CompileError::Io(format!(
            "cannot resolve C ABI artifact {}: {error}",
            path.display()
        ))
    })?;
    if !canonical_path.starts_with(&canonical_directory) {
        return Err(CompileError::Tool(format!(
            "C ABI artifact {} escapes its access-library directory",
            path.display()
        )));
    }
    let bytes = fs::read(&canonical_path).map_err(|error| {
        CompileError::Io(format!(
            "cannot read C ABI artifact {}: {error}",
            path.display()
        ))
    })?;
    let actual = format!("{:x}", Sha256::digest(&bytes));
    if actual != expected {
        return Err(CompileError::Tool(format!(
            "C ABI artifact {} has digest {actual}, expected {expected}",
            path.display()
        )));
    }
    Ok(canonical_path)
}

fn attach_foreign_functions(
    program: &mut CompilerProgram,
    libraries: &BTreeMap<String, (AccessLibrary, PathBuf)>,
) -> Result<(), CompileError> {
    for function in &mut program.functions {
        let Some(identity) = function
            .module_identity
            .as_ref()
            .and_then(|identity| identity.first())
        else {
            continue;
        };
        let Some((library, _)) = libraries.get(identity) else {
            continue;
        };
        let source_name = function
            .source_name
            .rsplit('.')
            .next()
            .unwrap_or(&function.source_name);
        let declaration = library
            .functions
            .iter()
            .find(|candidate| candidate.topal_name == source_name)
            .ok_or_else(|| {
                CompileError::Tool(format!(
                    "generated ABI module function `{identity} {}` is absent from its manifest",
                    function.source_name
                ))
            })?;
        if declaration.parameters.len() != function.parameters.len()
            || !function
                .parameters
                .iter()
                .all(|parameter| parameter.value_type == CompilerType::Int)
            || function.result_type != compiler_type(declaration.result)
        {
            return Err(CompileError::Tool(format!(
                "generated Topal signature for `{identity} {}` disagrees with its C manifest",
                function.source_name
            )));
        }
        function.foreign = Some(CompilerCFunction {
            library_identity: identity.clone(),
            external_symbol: declaration.symbol.clone(),
            parameters: declaration
                .parameters
                .iter()
                .map(|parameter| compiler_c_value(parameter.value))
                .collect(),
            result: compiler_c_value(declaration.result),
        });
    }
    Ok(())
}

const fn compiler_c_value(value: CValue) -> CompilerCValue {
    match value {
        CValue::Void => CompilerCValue::Void,
        CValue::SignedInt32 => CompilerCValue::SignedInt32,
    }
}

fn compiler_type(value: CValue) -> CompilerType {
    match value {
        CValue::Void => CompilerType::Unit,
        CValue::SignedInt32 => CompilerType::Int,
    }
}
