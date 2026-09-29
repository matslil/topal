//! Source discovery and checked-program construction.

use std::path::Path;

use topal_language::compiler::{CompilerProgram, analyze_for_compiler_with_modules};
use topal_language::modules::select_source_modules;

use crate::CompileError;

const BUILT_IN_LIBRARIES: &[&str] = &["advent-of-code", "std"];

/// A program accepted by the shared compiler frontend.
///
/// Keeping this wrapper private to the compiler pipeline prevents unchecked
/// source or raw module collections from reaching a native backend stage.
pub(crate) struct CheckedProgram(CompilerProgram);

impl CheckedProgram {
    pub(crate) fn program(&self) -> &CompilerProgram {
        &self.0
    }
}

/// Discover referenced modules and construct the checked compiler model.
pub(crate) fn check(source: &str, library_root: &Path) -> Result<CheckedProgram, CompileError> {
    let modules = select_source_modules(source, library_root, BUILT_IN_LIBRARIES)
        .map_err(|error| CompileError::Io(error.to_string()))?;
    analyze_for_compiler_with_modules(source, &modules)
        .map(CheckedProgram)
        .map_err(CompileError::Diagnostic)
}
