use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::Serialize;
use topal_best_practices::{Catalog, CatalogEntry};
use topal_language::{Session, Value as LanguageValue};
use topal_source::{Diagnostic, Severity, SourceText, Span};
use topal_syntax::{
    CallableKind, DecisionMatcher, DiagnosticControlKind, Expression, Statement, SyntaxDiagnostic,
    lex, parse,
};

include!("lib/frontend_and_engine.rs");
include!("lib/design_pattern_rules.rs");
#[cfg(test)]
mod tests {
    include!("lib/tests/every_language_example_uses_the_shared_linter_frontend.rs");
}
