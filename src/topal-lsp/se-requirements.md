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

## TOPAL-LSP-COMPLETION-001 — Implemented root-operation completion

The language server shall advertise and answer standard full-document
completion requests with a deterministic list of named root operations
implemented by the current language subset. Each item shall identify the
operation as a function and describe its implemented call shape. Completion
shall not advertise planned operations before their interpreter semantics land.
When an operation accepts a finite set of implemented static arguments, its
detail shall name those arguments so completion does not imply support for
planned alternatives.

## TOPAL-LSP-VARIANT-001 — Domain-specific language contexts

The language server shall preserve language feature selections, recognize the
`debug` and `lint` variants without treating their vocabulary as global, and
provide completion for `lang debug` or the `lang lint` namespace only in the
applicable constructed context.
Debugger-script examples shall receive the same shared syntax diagnostics and
semantic tokens as other Topal source. `lang trace` completion shall be added
when its source construction is executable rather than advertised prematurely.

## TOPAL-LSP-COMPILER-BOUNDARY-001 — Analysis-only artifact boundary

The language server may inspect typed static views but shall not export, lower,
or optimize GEIR artifacts. Requests through shared embedding interfaces shall
return `E-COMPILER-ONLY` without executing source or retaining runtime
reflection metadata.

## TOPAL-LSP-LINT-001 — Shared live best-practice findings

Initialization options may explicitly enable or disable best-practices by
stable identity, namespace, or tag through
`initializationOptions.lint.enable` and `.disable`, and may override selector
severity through `.severity`. The language server shall reject malformed or
unknown selectors and levels during initialization and shall run the shared
in-memory linter engine on every open or full-content change. Published
findings shall preserve lint severity, stable code, UTF-16 location,
best-practice and rule versions, and structured rectification data. Proposed
entries shall remain disabled unless explicitly enabled.

External catalog paths may be supplied explicitly through
`initializationOptions.lint.catalogs`. The language server shall load and
validate their generated JSON projections before accepting lint selectors.
Loading a catalog shall not grant its contained rule access to the catalog
path, editor process, filesystem, or other ambient authority.
