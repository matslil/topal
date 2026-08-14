# Core-language correction ledger

The specification-file ledger alone cannot prove completion of source-facing
behavior described by authoritative design documents. This ledger records the
independently discovered gaps and the terminal evidence required before the
`v0.1` core may again be declared complete.

`planned` means at least one required layer or invariant is missing. `complete`
requires the named implementation, cross-tool behavior, commented executable
examples, and conformance tests.

| Surface | Authority | Required terminal evidence | Status |
| --- | --- | --- | --- |
| complete function headers and effects | `docs/functions.md`, `docs/effects.md` | shared parser/type/effect behavior; interpreter, debugger, and LSP tests; commented source example; no temporary unsupported diagnostic | complete |
| tasks and message transactions | `docs/tasks.md`, `spec/concurrency-model.md` | task definition/state/lifecycle, event/request/stream execution, scheduler evidence, message-following debugger history, LSP coverage, commented examples | complete |
| static introspection | `docs/introspection.md` | typed `lang` operations, visibility/identity/relations, static-only enforcement, interpreter static execution, debugger/LSP coverage, commented examples | complete |
| external layouts and locations | `docs/layouts.md`, `spec/serialization.md` | source construction, complete initial schemas, checked location read/write, interpreter/debugger/LSP coverage, commented examples | complete |
| native serialization operations | `docs/serialization.md`, `spec/serialization.md` | source `lang serialize`/`deserialize`, structurally validated protocol values, incremental streams, resource limits, cross-tool examples and malformed tests | complete |
| GEIR interchange | `docs/introspection.md`, `spec/generic-ir.md` | canonical decoder, SHA-256 structural identities, full staged validation, compatibility and malformed-artifact tests | complete |
| acceptance accounting | `se/core-language-coverage.md`, this ledger | executable evidence checks rather than rule-ID substring checks; all rows complete; workspace and cross-tool suites green | complete |
