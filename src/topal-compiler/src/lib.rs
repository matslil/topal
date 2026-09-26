//! Correctness-first native compilation for the currently admitted Topal slice.

mod artifact;
mod codegen;
mod toolchain;

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

pub use artifact::{
    DigestEntry, ExportEntry, NATIVE_ABI, NATIVE_ARTIFACT_SCHEMA, NativeArtifactMetadata,
    NativeSlice, PLATFORM_ABI,
};
pub use toolchain::LlvmTools;
use topal_source::Diagnostic;

pub const TARGET_TRIPLE: &str = "x86_64-unknown-linux-gnu";
pub const DATA_LAYOUT: &str =
    "e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128";
pub const LLVM_MAJOR: u32 = 22;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Emit {
    LlvmIr,
    Object,
    Executable,
}

impl Emit {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::LlvmIr => "llvm-ir",
            Self::Object => "object",
            Self::Executable => "executable",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CompileOptions {
    pub source_name: String,
    pub output: PathBuf,
    pub emit: Emit,
    pub llvm_tools: Option<PathBuf>,
    pub library_root: PathBuf,
}

#[derive(Debug)]
pub enum CompileError {
    Diagnostic(Diagnostic),
    Tool(String),
    Io(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Diagnostic(diagnostic) => diagnostic.fmt(formatter),
            Self::Tool(message) | Self::Io(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for CompileError {}

/// Compile source and publish the output with its canonical metadata sidecar.
///
/// # Errors
///
/// Returns a source diagnostic, an LLVM tool failure, or an I/O failure. LLVM
/// failures occur before the requested output path is published.
pub fn compile_source(
    source: &str,
    options: &CompileOptions,
) -> Result<NativeArtifactMetadata, CompileError> {
    let modules = selected_source_modules(source, &options.library_root)?;
    let program = topal_language::analyze_for_compiler_with_modules(source, &modules)
        .map_err(CompileError::Diagnostic)?;
    let llvm = codegen::emit_llvm(&program, &options.source_name);
    toolchain::materialize(&program, llvm.as_bytes(), options)
}

fn selected_source_modules(
    source: &str,
    library_root: &Path,
) -> Result<Vec<topal_language::CompilerSourceModule>, CompileError> {
    let mut modules = Vec::new();
    for library in ["advent-of-code", "std"] {
        if !topal_language::declares_library(source, library) {
            continue;
        }
        let directory = library_root.join(library);
        collect_selected_source_modules(source, library, &directory, &directory, &mut modules)?;
    }
    modules.sort_by(|left, right| left.identity.cmp(&right.identity));
    Ok(modules)
}

fn collect_selected_source_modules(
    application: &str,
    library: &str,
    root: &Path,
    directory: &Path,
    modules: &mut Vec<topal_language::CompilerSourceModule>,
) -> Result<(), CompileError> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(CompileError::Io(format!(
                "cannot read library directory {}: {error}",
                directory.display()
            )));
        }
    };
    let mut paths = entries
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| CompileError::Io(error.to_string()))?;
    paths.sort();
    for path in paths {
        if path.is_dir() {
            collect_selected_source_modules(application, library, root, &path, modules)?;
            continue;
        }
        if path.extension().is_none_or(|extension| extension != "t")
            || path.file_name().is_some_and(|name| {
                matches!(
                    name.to_str(),
                    Some("application.t" | "package.t" | "library.t" | "module.t")
                )
            })
        {
            continue;
        }
        let relative = path.strip_prefix(root).map_err(|error| {
            CompileError::Io(format!(
                "cannot identify library module {}: {error}",
                path.display()
            ))
        })?;
        let mut identity = vec![library.to_owned()];
        identity.extend(
            relative
                .parent()
                .into_iter()
                .flat_map(Path::components)
                .map(|component| component.as_os_str().to_string_lossy().into_owned()),
        );
        identity.push(
            path.file_stem()
                .expect("selected .t module has a stem")
                .to_string_lossy()
                .into_owned(),
        );
        if !topal_language::references_module(application, &identity) {
            continue;
        }
        let module_source = fs::read_to_string(&path).map_err(|error| {
            CompileError::Io(format!(
                "cannot read library module {}: {error}",
                path.display()
            ))
        })?;
        modules.push(topal_language::CompilerSourceModule {
            identity,
            source_name: path.display().to_string(),
            source: module_source,
        });
    }
    Ok(())
}

#[must_use]
pub fn metadata_path(output: &Path) -> PathBuf {
    let mut name = output.as_os_str().to_owned();
    name.push(".topal.json");
    PathBuf::from(name)
}
