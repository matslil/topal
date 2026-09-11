//! Correctness-first native compilation for the currently admitted Topal slice.

mod artifact;
mod codegen;
mod toolchain;

use std::fmt;
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
    let program = topal_language::analyze_for_compiler(source).map_err(CompileError::Diagnostic)?;
    let llvm = codegen::emit_llvm(&program, &options.source_name);
    toolchain::materialize(&program, llvm.as_bytes(), options)
}

#[must_use]
pub fn metadata_path(output: &Path) -> PathBuf {
    let mut name = output.as_os_str().to_owned();
    name.push(".topal.json");
    PathBuf::from(name)
}
