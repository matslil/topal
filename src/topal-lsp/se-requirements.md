# Topal language server requirements

These tool requirements refine `TOPAL-REQ-TOOLS-001` and
`TOPAL-REQ-SHARED-001` for the editor-facing language server.

## TOPAL-LSP-PROTOCOL-001 — Standards-compatible transport

The language server shall communicate over standard input and output using
JSON-RPC framing compatible with Language Server Protocol 3.18. Protocol output
shall never be mixed with logging or human-oriented text. The initial server
shall implement initialize, initialized, shutdown, exit, and deterministic
method-not-found responses for unsupported requests.

## TOPAL-LSP-SYNC-001 — In-memory document authority

The language server shall support open, full-content change, and close
notifications. Open editor content is authoritative over filesystem state.
Closing a document shall clear its published diagnostics and release its
in-memory content.

## TOPAL-LSP-DIAG-001 — Shared live syntax diagnostics

On every open or full-content change, the language server shall run the shared
source, lexer, and recoverable parser layers without executing the program or
mutating an interpreter session. It shall publish every available source and
syntax diagnostic with its stable code, message, severity, and UTF-16 protocol
range. CRLF normalization shall not change the line and character range seen by
the client.

## TOPAL-LSP-TOKENS-001 — Recovery-friendly semantic tokens

The language server shall provide full-document semantic tokens derived from
the shared lossless lexer even when the document has recoverable syntax errors.
The initial legend shall distinguish variables, numbers, strings, comments,
reserved literals, and operators. Token positions and lengths shall use UTF-16;
multiline tokens shall be split into valid single-line protocol tokens without
changing their shared source spans.

## TOPAL-LSP-FEATURE-001 — Feature-increment editor coverage

Every language-feature increment shall assess and update language-server
behavior in the same reviewable series. When no protocol implementation change
is required, the increment shall still add explicit LSP conformance coverage
showing that shared diagnostics, semantic tokens, and other applicable editor
features recognize the new syntax and semantics. The LSP suite shall open every
runnable example and require it to be free of shared-frontend diagnostics.
