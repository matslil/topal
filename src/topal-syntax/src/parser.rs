use topal_source::{SourceText, Span};

use crate::{Lexed, SyntaxDiagnostic, Token, TokenKind};

include!("parser/ast.rs");
include!("parser/statements.rs");
include!("parser/decisions_and_expressions.rs");
include!("parser/statement_span.rs");
#[cfg(test)]
mod tests {
    include!("parser/tests/retains_generic_application_order.rs");
}
