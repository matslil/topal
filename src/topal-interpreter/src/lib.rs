//! Application-service boundary for deterministic Topal interpretation.
//!
//! The shared language crate owns evaluation semantics. This crate coordinates
//! a persistent session with explicit module and application inputs while
//! leaving terminal I/O, argument parsing, and process exit policy to adapters.

use std::collections::BTreeSet;
use std::fmt;
use std::path::{Path, PathBuf};

use topal_language::Diagnostic;
use topal_language::interpreter::{Session, Value};
use topal_language::modules::{declares_library, load_module_tree};
use topal_language::tracing::TraceSink;

/// One explicit source-evaluation request.
#[derive(Clone, Copy, Debug)]
pub struct EvaluationRequest<'a> {
    pub source: &'a str,
    pub source_name: &'a str,
    pub library_root: Option<&'a Path>,
    pub application_input: Option<&'a str>,
}

/// Failure at the interpreter application-service boundary.
#[derive(Debug)]
pub enum InterpreterError {
    Diagnostic {
        source_name: String,
        diagnostic: Box<Diagnostic>,
    },
    Module(String),
}

impl fmt::Display for InterpreterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Diagnostic {
                source_name,
                diagnostic,
            } => formatter.write_str(&diagnostic.render(source_name)),
            Self::Module(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for InterpreterError {}

/// Reusable source interpreter application service.
///
/// A service owns exactly one persistent semantic session. Host adapters may
/// submit multiple requests intentionally; no global state or terminal stream
/// is consulted by this type.
#[derive(Clone, Default)]
pub struct Interpreter {
    session: Session,
    loaded_module_roots: BTreeSet<PathBuf>,
}

impl Interpreter {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a package/module tree into the service's persistent session.
    ///
    /// # Errors
    ///
    /// Returns a path-qualified module loading or semantic error.
    pub fn load_modules(
        &mut self,
        directory: &Path,
        trace: &mut impl TraceSink,
    ) -> Result<(), InterpreterError> {
        if self.loaded_module_roots.contains(directory) {
            return Ok(());
        }
        load_module_tree(&mut self.session, directory, trace).map_err(InterpreterError::Module)?;
        self.loaded_module_roots.insert(directory.to_owned());
        Ok(())
    }

    /// Evaluate source and optionally invoke its conventional String `solve`
    /// entry point with an explicit application input.
    ///
    /// When `library_root` is present, declared built-in libraries are loaded
    /// before source evaluation. When `application_input` is absent, the source
    /// result is returned directly. The caller owns any policy that requires an
    /// input file for a named application.
    ///
    /// # Errors
    ///
    /// Returns shared source diagnostics with the supplied source identity, or
    /// a module-loading failure.
    pub fn evaluate(
        &mut self,
        request: EvaluationRequest<'_>,
        trace: &mut impl TraceSink,
    ) -> Result<Value, InterpreterError> {
        if let Some(library_root) = request.library_root
            && (declares_library(request.source, "std")
                || declares_library(request.source, "advent-of-code"))
        {
            self.load_modules(library_root, trace)?;
        }
        let value = self
            .session
            .evaluate_source_file(request.source, trace)
            .map_err(|diagnostic| InterpreterError::Diagnostic {
                source_name: request.source_name.to_owned(),
                diagnostic: Box::new(diagnostic),
            })?;
        if let Some(input) = request.application_input {
            let expression = format!("solve {}", Value::String(input.to_owned()));
            self.session
                .evaluate(&expression, trace)
                .map_err(|diagnostic| InterpreterError::Diagnostic {
                    source_name: request.source_name.to_owned(),
                    diagnostic: Box::new(diagnostic),
                })
        } else {
            Ok(value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_returns_source_result_without_terminal_io() {
        let mut interpreter = Interpreter::new();
        let value = interpreter
            .evaluate(
                EvaluationRequest {
                    source: "use language (version is v0.1)\n40 + 2\n",
                    source_name: "answer.t",
                    library_root: None,
                    application_input: None,
                },
                &mut Vec::new(),
            )
            .unwrap();
        assert_eq!(value.to_string(), "42");
    }

    #[test]
    fn service_invokes_solve_with_only_explicit_input() {
        let mut interpreter = Interpreter::new();
        let value = interpreter
            .evaluate(
                EvaluationRequest {
                    source: "use language (version is v0.1)\nsolve is fn (input : String) -> String\n  input\n()\n",
                    source_name: "application.t",
                    library_root: None,
                    application_input: Some("hello"),
                },
                &mut Vec::new(),
            )
            .unwrap();
        assert_eq!(value.to_string(), "\"hello\"");
    }
}
