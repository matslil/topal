//! Source discovery and checked-program construction.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use topal_c_abi::{AccessLibraryManifest, CValue};
use topal_language::compiler::{
    CompilerCFunction, CompilerCValue, CompilerProgram, CompilerType,
    analyze_for_compiler_with_modules,
};
use topal_language::modules::{declared_libraries, select_source_modules};

use crate::CompileError;
use crate::DigestEntry;
use crate::standard_library::{
    StandardLibrarySlice, source_interface_digest, standard_library_manifest_path,
};

const BUILT_IN_LIBRARIES: &[&str] = &["advent-of-code", "std"];

/// A program accepted by the shared compiler frontend.
///
/// Keeping this wrapper private to the compiler pipeline prevents unchecked
/// source or raw module collections from reaching a native backend stage.
pub(crate) struct CheckedProgram {
    program: CompilerProgram,
    static_archives: Vec<StaticArchiveLink>,
    shared_objects: Vec<SharedObjectLink>,
    foreign_dependencies: Vec<DigestEntry>,
}

#[derive(Clone, Debug)]
pub(crate) struct SharedObjectLink {
    pub(crate) library_identity: String,
    pub(crate) path: PathBuf,
    pub(crate) soname: String,
    pub(crate) sha256: String,
}

#[derive(Clone, Debug)]
pub(crate) struct StaticArchiveLink {
    pub(crate) library_identity: String,
    pub(crate) path: PathBuf,
}

enum ForeignBinary {
    StaticArchive(PathBuf),
    SharedObject(SharedObjectLink),
}

struct LoadedAccessLibrary {
    manifest: AccessLibraryManifest,
    binary: ForeignBinary,
}

impl CheckedProgram {
    pub(crate) fn program(&self) -> &CompilerProgram {
        &self.program
    }

    pub(crate) fn static_archives(&self) -> &[StaticArchiveLink] {
        &self.static_archives
    }

    pub(crate) fn shared_objects(&self) -> &[SharedObjectLink] {
        &self.shared_objects
    }

    pub(crate) fn foreign_dependencies(&self) -> &[DigestEntry] {
        &self.foreign_dependencies
    }
}

/// Discover referenced modules and construct the checked compiler model.
#[allow(clippy::too_many_lines)] // Source, C ABI, and sealed Topal slice discovery remain one ordered boundary.
pub(crate) fn check(
    source: &str,
    library_root: &Path,
    standard_library: Option<&Path>,
) -> Result<CheckedProgram, CompileError> {
    let mut libraries = BUILT_IN_LIBRARIES
        .iter()
        .map(|identity| (*identity).to_owned())
        .collect::<Vec<_>>();
    let declared = declared_libraries(source);
    for identity in &declared {
        if !libraries.contains(identity) {
            libraries.push(identity.clone());
        }
    }
    let library_names = libraries.iter().map(String::as_str).collect::<Vec<_>>();
    let modules = select_source_modules(source, library_root, &library_names)
        .map_err(|error| CompileError::Io(error.to_string()))?;
    let access_libraries = load_access_libraries(&modules)?;
    let mut program =
        analyze_for_compiler_with_modules(source, &modules).map_err(CompileError::Diagnostic)?;
    attach_foreign_functions(&mut program, &access_libraries)?;
    let mut static_archives = Vec::new();
    let mut shared_objects = Vec::new();
    let mut foreign_dependencies = Vec::new();
    let mut shared_sonames = BTreeMap::new();
    for library in access_libraries.values() {
        foreign_dependencies.push(DigestEntry {
            identity: format!("c-abi.{}.header", library.manifest.identity()),
            sha256: library.manifest.header().sha256.clone(),
        });
        match &library.binary {
            ForeignBinary::StaticArchive(path) => {
                static_archives.push(StaticArchiveLink {
                    library_identity: library.manifest.identity().to_owned(),
                    path: path.clone(),
                });
                let AccessLibraryManifest::Static(manifest) = &library.manifest else {
                    unreachable!("static binary retains static manifest")
                };
                foreign_dependencies.push(DigestEntry {
                    identity: format!("c-abi.{}.static-archive", manifest.identity),
                    sha256: manifest.static_archive.sha256.clone(),
                });
            }
            ForeignBinary::SharedObject(shared) => {
                if let Some(previous) = shared_sonames.insert(&shared.soname, &shared.sha256)
                    && previous != &shared.sha256
                {
                    return Err(CompileError::Tool(format!(
                        "C shared-object SONAME `{}` resolves to conflicting digests",
                        shared.soname
                    )));
                }
                shared_objects.push(shared.clone());
                foreign_dependencies.push(DigestEntry {
                    identity: format!("c-abi.{}.shared-object", library.manifest.identity()),
                    sha256: shared.sha256.clone(),
                });
            }
        }
    }
    if let Some(path) = standard_library {
        if !declared.iter().any(|identity| identity == "std") {
            return Err(CompileError::Tool(
                "--std-library requires an explicit `use library std` selection".into(),
            ));
        }
        let manifest_path = standard_library_manifest_path(path);
        let manifest_bytes = fs::read(&manifest_path).map_err(|error| {
            CompileError::Io(format!(
                "cannot read standard-library manifest {}: {error}",
                manifest_path.display()
            ))
        })?;
        let manifest = StandardLibrarySlice::decode(&manifest_bytes)?;
        if path.file_name().and_then(|name| name.to_str()) != Some(manifest.soname.as_str()) {
            return Err(CompileError::Tool(format!(
                "standard-library slice filename must match SONAME `{}`",
                manifest.soname
            )));
        }
        let payload = fs::read(path).map_err(|error| {
            CompileError::Io(format!(
                "cannot read standard-library slice {}: {error}",
                path.display()
            ))
        })?;
        let payload_digest = format!("{:x}", Sha256::digest(&payload));
        if payload_digest != manifest.shared_object_sha256 {
            return Err(CompileError::Tool(
                "standard-library shared-object digest does not match its manifest".into(),
            ));
        }
        if source_interface_digest(library_root)? != manifest.source_interface_sha256 {
            return Err(CompileError::Tool(
                "standard-library source interface does not match the selected native slice".into(),
            ));
        }
        if let Some(previous) = shared_sonames.insert(&manifest.soname, &payload_digest)
            && previous != &payload_digest
        {
            return Err(CompileError::Tool(format!(
                "shared-object SONAME `{}` resolves to conflicting digests",
                manifest.soname
            )));
        }
        shared_objects.push(SharedObjectLink {
            library_identity: "std".into(),
            path: path.to_owned(),
            soname: manifest.soname,
            sha256: payload_digest.clone(),
        });
        foreign_dependencies.push(DigestEntry {
            identity: "topal-library.std.shared-object".into(),
            sha256: payload_digest,
        });
    }
    foreign_dependencies.sort_by(|left, right| left.identity.cmp(&right.identity));
    Ok(CheckedProgram {
        program,
        static_archives,
        shared_objects,
        foreign_dependencies,
    })
}

fn load_access_libraries(
    modules: &[topal_language::modules::SourceModule],
) -> Result<BTreeMap<String, LoadedAccessLibrary>, CompileError> {
    let mut libraries = BTreeMap::new();
    for module in modules {
        if !module.source.contains("features is ( abi )") {
            continue;
        }
        let source_path = Path::new(&module.source_name);
        let manifest = AccessLibraryManifest::decode_topal_source(&module.source)
            .map_err(CompileError::Tool)?;
        let identity = module.identity.first().ok_or_else(|| {
            CompileError::Tool("C access-library module has no library identity".into())
        })?;
        if manifest.identity() != identity {
            return Err(CompileError::Tool(format!(
                "C manifest identity `{}` does not match selected library `{identity}`",
                manifest.identity()
            )));
        }
        let directory = source_path.parent().unwrap_or_else(|| Path::new("."));
        verify_artifact(
            directory,
            &manifest.header().file,
            &manifest.header().sha256,
        )?;
        let binary = match &manifest {
            AccessLibraryManifest::Static(library) => {
                ForeignBinary::StaticArchive(verify_artifact(
                    directory,
                    &library.static_archive.file,
                    &library.static_archive.sha256,
                )?)
            }
            AccessLibraryManifest::Shared(library) => {
                let path = verify_artifact(
                    directory,
                    &library.shared_object.file,
                    &library.shared_object.sha256,
                )?;
                ForeignBinary::SharedObject(SharedObjectLink {
                    library_identity: identity.clone(),
                    path,
                    soname: library.shared_object.soname.clone(),
                    sha256: library.shared_object.sha256.clone(),
                })
            }
        };
        if libraries
            .insert(identity.clone(), LoadedAccessLibrary { manifest, binary })
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
    libraries: &BTreeMap<String, LoadedAccessLibrary>,
) -> Result<(), CompileError> {
    for function in &mut program.functions {
        let Some(identity) = function
            .module_identity
            .as_ref()
            .and_then(|identity| identity.first())
        else {
            continue;
        };
        let Some(library) = libraries.get(identity) else {
            continue;
        };
        let source_name = function
            .source_name
            .rsplit('.')
            .next()
            .unwrap_or(&function.source_name);
        let declaration = library
            .manifest
            .functions()
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
