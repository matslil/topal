# Traceability

This matrix connects system goals to core requirements and formal specification
domains. Test and implementation columns will be added with those artifacts.

| Goal | Requirements |
| --- | --- |
| `TOPAL-GOAL-COMPOSE-001` | `TOPAL-REQ-MODEL-001`, `TOPAL-REQ-GENERIC-001` |
| `TOPAL-GOAL-SAFE-001` | `TOPAL-REQ-SAFE-001`, `TOPAL-REQ-TOTAL-001`, `TOPAL-REQ-CONC-001`, `TOPAL-REQ-RESOURCE-001` |
| `TOPAL-GOAL-DETERMINISTIC-001` | `TOPAL-REQ-DETERMINISM-001`, `TOPAL-REQ-INTEROP-001` |
| `TOPAL-GOAL-EXPLICIT-001` | `TOPAL-REQ-EFFECT-001`, `TOPAL-REQ-RESOURCE-001`, `TOPAL-REQ-SERIAL-001` |
| `TOPAL-GOAL-ZEROCOST-001` | `TOPAL-REQ-DETERMINISM-001`, `TOPAL-REQ-RESOURCE-001` |
| `TOPAL-GOAL-PRECISE-001` | `TOPAL-REQ-GENERIC-001`, `TOPAL-REQ-SERIAL-001`, `TOPAL-REQ-TOOLS-001`, `TOPAL-REQ-INTEROP-001` |
| `TOPAL-GOAL-EVOLVE-001` | `TOPAL-REQ-TRACE-001`, `TOPAL-REQ-TOOLS-001` |
| `TOPAL-GOAL-TOOLCHAIN-001` | `TOPAL-REQ-SHARED-001`, `TOPAL-REQ-TOOLS-001`, `TOPAL-REQ-INTEROP-001` |

| Requirement | Governing specification rules |
| --- | --- |
| `TOPAL-REQ-MODEL-001` | `TOPAL-TYPE-KIND-001` through `TOPAL-TYPE-MATCH-001`, including `TOPAL-TYPE-BOOLEAN-001` and `TOPAL-TYPE-EQUALITY-001`, `TOPAL-NUM-SYMBOL-001`, `TOPAL-NUM-INT-RATIONAL-CONVERT-001` |
| `TOPAL-REQ-SAFE-001` | `TOPAL-TYPE-SOUND-001`, `TOPAL-NUM-INT-001`, `TOPAL-NUM-ADD-001`, `TOPAL-NUM-NEG-001`, `TOPAL-NUM-SUB-001`, `TOPAL-NUM-MUL-001`, `TOPAL-NUM-RATIONAL-001`, `TOPAL-NUM-DIV-001`, `TOPAL-NUM-DIVZERO-001`, `TOPAL-NUM-POW-001`, `TOPAL-NUM-INT-RATIONAL-CONVERT-001`, `TOPAL-MEM-LOCATION-001`, `TOPAL-MEM-PLAIN-001`, `TOPAL-CONC-RACE-001` |
| `TOPAL-REQ-TOTAL-001` | `TOPAL-TYPE-TOTAL-001`, `TOPAL-CONC-PROGRESS-001` |
| `TOPAL-REQ-CONC-001` | `TOPAL-CONC-DEADLOCK-001`, `TOPAL-CONC-RACE-001` |
| `TOPAL-REQ-DETERMINISM-001` | `TOPAL-NUM-INT-001`, `TOPAL-NUM-ADD-001`, `TOPAL-NUM-NEG-001`, `TOPAL-NUM-SUB-001`, `TOPAL-NUM-MUL-001`, `TOPAL-NUM-RATIONAL-001`, `TOPAL-NUM-DIV-001`, `TOPAL-NUM-POW-001`, `TOPAL-MEM-OPT-001`, `TOPAL-CONC-DETERMINISM-001` |
| `TOPAL-REQ-EFFECT-001` | `TOPAL-GIR-EFFECT-001`, `TOPAL-MEM-HARDWARE-001`, `TOPAL-CONC-ORDER-001` |
| `TOPAL-REQ-RESOURCE-001` | `TOPAL-MEM-LOCATION-001` through `TOPAL-MEM-LIFETIME-001` |
| `TOPAL-REQ-GENERIC-001` | `TOPAL-GIR-PURPOSE-001` through `TOPAL-GIR-COMPAT-001` |
| `TOPAL-REQ-SERIAL-001` | `TOPAL-SER-SCOPE-001` through `TOPAL-SER-CANON-001` |
| `TOPAL-REQ-TOOLS-001` | `TOPAL-SYN-SOURCE-001` through `TOPAL-SYN-DIAG-001`, including `TOPAL-SYN-UNICODE-001`, `TOPAL-NUM-SYMBOL-001`, `TOPAL-GIR-VALID-001` |
| `TOPAL-REQ-INTEROP-001` | `TOPAL-TYPE-SOUND-001`, `TOPAL-NUM-ADD-001`, `TOPAL-NUM-NEG-001`, `TOPAL-NUM-SUB-001`, `TOPAL-NUM-MUL-001`, `TOPAL-NUM-RATIONAL-001`, `TOPAL-NUM-DIV-001`, `TOPAL-NUM-POW-001`, `TOPAL-NUM-INT-RATIONAL-CONVERT-001`, `TOPAL-MEM-OPT-001`, `TOPAL-CONC-DETERMINISM-001` |
| `TOPAL-REQ-TRACE-001` | all stable `TOPAL-*` specification rules |
| `TOPAL-REQ-SHARED-001` | `TOPAL-SYN-SOURCE-001`, `TOPAL-SYN-UNICODE-001`, `TOPAL-SYN-LEX-001`, `TOPAL-SYN-INDENT-001`, `TOPAL-SYN-GRAMMAR-001` |

## Maintenance rules

- Never reuse a retired stable ID for different meaning.
- Record all applicable relationships, not only one convenient parent.
- A requirement without a validating scenario is incomplete.
- A normative specification rule without a requirement or approved design
  source is suspect and must be reviewed.
- A functional test without a specification-rule reference is incomplete.

## Implementation coverage

| Tool requirement | Specification rules | Functional evidence | Implementation |
| --- | --- | --- | --- |
| `TOPAL-INTP-MODE-001` | `TOPAL-SYN-SOURCE-001`, `TOPAL-SYN-NUM-001`, `TOPAL-SYN-GRAMMAR-001` | `src/topal-interpreter/tests/cli.rs` | `topal-language`, `topal-interpreter` |
| `TOPAL-INTP-MODE-002` | `TOPAL-SYN-NUM-001`, `TOPAL-SYN-GRAMMAR-001` | `src/topal-interpreter/tests/cli.rs` | `topal-language`, `topal-interpreter` |
| `TOPAL-INTP-MODE-003` | `TOPAL-SYN-SOURCE-001`, `TOPAL-SYN-NUM-001`, `TOPAL-SYN-GRAMMAR-001` | `src/topal-interpreter/tests/cli.rs` | `topal-language`, `topal-interpreter` |
| `TOPAL-INTP-SUBSET-001` | `TOPAL-SYN-GRAMMAR-001`, `TOPAL-REQ-TOOLS-001` | `src/topal-interpreter/tests/cli.rs` | `topal-language` |
| `TOPAL-INTP-SUBSET-002` | `TOPAL-SYN-BIND-001`, `TOPAL-SYN-GRAMMAR-001` | `src/topal-interpreter/tests/cli.rs` | `topal-language` |
| `TOPAL-INTP-SUBSET-003` | `TOPAL-SYN-NUM-001`, `TOPAL-NUM-LITERAL-001` | `src/topal-interpreter/tests/cli.rs` | `topal-language` |
| `TOPAL-REQ-SHARED-001` | `TOPAL-SYN-SOURCE-001`, `TOPAL-SYN-LEX-001`, `TOPAL-SYN-GRAMMAR-001` | unit tests in `topal-source` and `topal-syntax` | `topal-source`, `topal-syntax` |
| `TOPAL-INTP-SUBSET-004` | `TOPAL-SYN-GRAMMAR-001`, `TOPAL-TYPE-CALL-001`, `TOPAL-NUM-ADD-001` | `src/topal-interpreter/tests/cli.rs` | `topal-source`, `topal-syntax`, `topal-language` |
| `TOPAL-INTP-SUBSET-005` | `TOPAL-SYN-GRAMMAR-001`, `TOPAL-TYPE-CALL-001`, `TOPAL-NUM-NEG-001`, `TOPAL-NUM-SUB-001` | `src/topal-interpreter/tests/cli.rs` | `topal-syntax`, `topal-language` |
| `TOPAL-INTP-SUBSET-006` | `TOPAL-SYN-GRAMMAR-001`, `TOPAL-TYPE-CALL-001`, `TOPAL-NUM-MUL-001` | `src/topal-interpreter/tests/cli.rs` | `topal-syntax`, `topal-language` |
| `TOPAL-INTP-SUBSET-007` | `TOPAL-TYPE-CALL-001`, `TOPAL-NUM-RATIONAL-001`, `TOPAL-NUM-DIV-001`, `TOPAL-NUM-DIVZERO-001` | `src/topal-interpreter/tests/cli.rs` | `topal-syntax`, `topal-language` |
| `TOPAL-INTP-SUBSET-008` | `TOPAL-NUM-DIVZERO-001`, `TOPAL-TYPE-CALL-001` | `src/topal-interpreter/tests/cli.rs` | `topal-language`, `topal-interpreter` |
| `TOPAL-INTP-SUBSET-009` | `TOPAL-SYN-GRAMMAR-001`, `TOPAL-TYPE-CALL-001`, `TOPAL-NUM-POW-001` | `src/topal-interpreter/tests/cli.rs` | `topal-syntax`, `topal-language` |
| `TOPAL-INTP-SUBSET-010` | `TOPAL-SYN-NUM-001`, `TOPAL-NUM-RATIONAL-001`, `TOPAL-NUM-RATIONAL-LITERAL-001` | `src/topal-interpreter/tests/cli.rs` | `topal-syntax`, `topal-language` |
| `TOPAL-INTP-SUBSET-011` | `TOPAL-TYPE-CALL-001`, `TOPAL-NUM-RAT-NEG-001` through `TOPAL-NUM-RAT-DIV-001`, `TOPAL-NUM-DIVZERO-001` | `src/topal-interpreter/tests/cli.rs` | `topal-language`, `topal-interpreter` |
| `TOPAL-INTP-SUBSET-012` | `TOPAL-TYPE-CONVERT-001`, `TOPAL-NUM-INT-RATIONAL-CONVERT-001`, `TOPAL-TYPE-CALL-001` | `src/topal-interpreter/tests/cli.rs` | `topal-language`, `topal-interpreter` |
| `TOPAL-INTP-SUBSET-013` | `TOPAL-SYN-STRING-001`, `TOPAL-SYN-GRAMMAR-001` | `src/topal-interpreter/tests/cli.rs` | `topal-source`, `topal-syntax`, `topal-language` |
| `TOPAL-INTP-SUBSET-014` | `TOPAL-SYN-UNICODE-001`, `TOPAL-SYN-LEX-001`, `TOPAL-SYN-STRING-001` | `src/topal-interpreter/tests/cli.rs` and unit tests in `topal-source` and `topal-syntax` | `topal-source`, `topal-syntax`, `topal-language`, `topal-interpreter` |
| `TOPAL-INTP-SUBSET-015` | `TOPAL-SYN-GRAMMAR-001`, `TOPAL-TYPE-PRODUCT-001` | `src/topal-interpreter/tests/cli.rs` and unit tests in `topal-syntax` | `topal-syntax`, `topal-language`, `topal-interpreter` |
| `TOPAL-INTP-SUBSET-016` | `TOPAL-SYN-GRAMMAR-001`, `TOPAL-TYPE-PRODUCT-001` | `src/topal-interpreter/tests/cli.rs` and unit tests in `topal-syntax` | `topal-syntax`, `topal-language`, `topal-interpreter` |
| `TOPAL-INTP-SUBSET-017` | `TOPAL-SYN-INDENT-001`, `TOPAL-SYN-GRAMMAR-001` | `src/topal-interpreter/tests/cli.rs` and unit tests in `topal-syntax` | `topal-syntax`, `topal-language`, `topal-interpreter` |
| `TOPAL-INTP-SUBSET-018` | `TOPAL-SYN-LEX-001`, `TOPAL-SYN-GRAMMAR-001`, `TOPAL-TYPE-BOOLEAN-001` | `src/topal-interpreter/tests/cli.rs` and unit tests in `topal-syntax` | `topal-syntax`, `topal-language`, `topal-interpreter` |
| `TOPAL-INTP-SUBSET-019` | `TOPAL-SYN-GRAMMAR-001`, `TOPAL-TYPE-CALL-001`, `TOPAL-TYPE-EQUALITY-001`, `TOPAL-NUM-INT-RATIONAL-CONVERT-001` | `src/topal-interpreter/tests/cli.rs` | `topal-syntax`, `topal-language`, `topal-interpreter` |
| `TOPAL-INTP-SUBSET-020` | `TOPAL-SYN-LEX-001`, `TOPAL-TYPE-CALL-001`, `TOPAL-TYPE-EQUALITY-001` | `src/topal-interpreter/tests/cli.rs` and unit tests in `topal-syntax` | `topal-syntax`, `topal-language`, `topal-interpreter` |
| `TOPAL-INTP-DIAG-001` | `TOPAL-REQ-TOOLS-001`, `TOPAL-REQ-SHARED-001` | `src/topal-interpreter/tests/cli.rs` and unit tests in `topal-language` | `topal-language`, `topal-interpreter` |
| `TOPAL-INTP-SUBSET-021` | `TOPAL-SYN-LEX-001`, `TOPAL-TYPE-CALL-001`, `TOPAL-NUM-COMPARE-001`, `TOPAL-NUM-INT-RATIONAL-CONVERT-001` | `src/topal-interpreter/tests/cli.rs` and unit tests in `topal-syntax` | `topal-syntax`, `topal-language`, `topal-interpreter` |
| `TOPAL-INTP-SUBSET-022` | `TOPAL-TYPE-ORDERING-001`, `TOPAL-NUM-COMPARE-001`, `TOPAL-NUM-INT-RATIONAL-CONVERT-001` | `src/topal-interpreter/tests/cli.rs` | `topal-language`, `topal-interpreter` |
| `TOPAL-LSP-PROTOCOL-001` | `TOPAL-REQ-TOOLS-001` | `src/topal-lsp/tests/protocol.rs` | `topal-lsp` |
| `TOPAL-LSP-SYNC-001` | `TOPAL-REQ-SHARED-001` | unit tests in `topal-lsp` and `src/topal-lsp/tests/protocol.rs` | `topal-lsp` |
| `TOPAL-LSP-DIAG-001` | `TOPAL-SYN-SOURCE-001`, `TOPAL-SYN-LEX-001`, `TOPAL-SYN-GRAMMAR-001`, `TOPAL-REQ-SHARED-001` | unit tests in `topal-lsp` and `src/topal-lsp/tests/protocol.rs` | `topal-source`, `topal-syntax`, `topal-lsp` |
| `TOPAL-LSP-TOKENS-001` | `TOPAL-SYN-LEX-001`, `TOPAL-SYN-STRING-001`, `TOPAL-REQ-SHARED-001` | unit tests in `topal-lsp` | `topal-source`, `topal-syntax`, `topal-lsp` |
| `TOPAL-INTP-EXAMPLE-001` | implemented `TOPAL-SYN-*`, `TOPAL-TYPE-*`, and `TOPAL-NUM-*` subset rules | `examples/interpreter/*.t` via `src/topal-interpreter/tests/cli.rs` | `topal-interpreter` |
| `TOPAL-LSP-FEATURE-001` | `TOPAL-REQ-SHARED-001` and each implemented feature rule | `examples/interpreter/*.t` via unit tests in `topal-lsp` | `topal-source`, `topal-syntax`, `topal-lsp` |

The approved source-debugger requirements are staged for implementation in
`src/topal-debugger/se-requirements.md`. Their functional-evidence and
implementation entries shall be added incrementally when the corresponding
debugger capabilities land; `TOPAL-DEBUG-MESSAGE-001` remains intentionally
pending until message passing exists in the shared execution machine.

| Tool requirement | Specification rules | Functional evidence | Implementation |
| --- | --- | --- | --- |
| `TOPAL-DEBUG-EXEC-001`, `TOPAL-DEBUG-TRACE-001` | implemented semantic trace rule IDs | unit tests in `topal-language`; `examples/debugger/basic-history.t` | `topal-language::ExecutionHistory` |
| `TOPAL-DEBUG-CONTROL-001` (transition navigation foundation) | implemented semantic trace rule IDs | `src/topal-debugger/tests/cli.rs`; `examples/debugger/basic-history.t` | `topal-debugger` |
| `TOPAL-DEBUG-MODE-001` | debugger command contract | `src/topal-debugger/tests/cli.rs`; `examples/debugger/basic-history.debug` | `topal-debugger` |
| `TOPAL-DEBUG-REVERSE-001` (immutable binding checkpoints) | `TOPAL-SYN-BIND-001` and implemented value rules | unit tests in `topal-language`; `src/topal-debugger/tests/cli.rs` | `topal-language::ExecutionState`, `topal-debugger` |
| `TOPAL-DEBUG-CONTROL-001` (source stepping) | `TOPAL-SYN-GRAMMAR-001`, `TOPAL-SYN-BIND-001` | unit tests in `topal-language`; `src/topal-debugger/tests/cli.rs` | `topal-language::ExecutionHistory`, `topal-debugger` |
| `TOPAL-DEBUG-CONTROL-001` (breakpoints and continue) | `TOPAL-SYN-GRAMMAR-001` | `src/topal-debugger/tests/cli.rs`; `examples/debugger/basic-history.debug` | `topal-language::ExecutionHistory`, `topal-debugger` |
| `TOPAL-DEBUG-CONTROL-001` (binding watchpoints) | `TOPAL-SYN-BIND-001` | `src/topal-debugger/tests/cli.rs`; `examples/debugger/basic-history.debug` | `topal-debugger` |
| `TOPAL-DEBUG-CONTROL-001` (named checkpoints) | implemented execution rules | `src/topal-debugger/tests/cli.rs`; `examples/debugger/basic-history.debug` | `topal-language::ExecutionHistory`, `topal-debugger` |
| `TOPAL-DEBUG-CONTROL-001` (top-level next, finish, and backtrace) | `TOPAL-SYN-GRAMMAR-001` | `src/topal-debugger/tests/cli.rs`; `examples/debugger/basic-history.debug` | `topal-language::ExecutionHistory`, `topal-debugger` |
| `TOPAL-DEBUG-MODE-001` (interactive prompts and strict scripts) | debugger command contract | `src/topal-debugger/tests/cli.rs` | `topal-debugger` |
| `TOPAL-DEBUG-TRACE-001` (`why` decision inspection) | stable rule ID carried by each transition | `src/topal-debugger/tests/cli.rs`; `examples/debugger/basic-history.debug` | `topal-debugger` |
| `TOPAL-DEBUG-EXEC-001` (resumable source execution) | implemented statement and expression rules | unit tests in `topal-language`; existing interpreter functional suites | `topal-language::Execution`, `topal-language::Session` |
| `TOPAL-DEBUG-EXEC-001` (live debugger control) | implemented statement and expression rules | `src/topal-debugger/tests/cli.rs`; `examples/debugger/live-execution.debug` | `topal-debugger`, `topal-language::Execution` |
| `TOPAL-DEBUG-FAILURE-001` | implemented diagnostic and execution rules | `src/topal-debugger/tests/cli.rs`; `examples/debugger/failing-history.t`; `examples/debugger/failing-history.debug` | `topal-debugger`, `topal-language::ExecutionHistory` |
| `TOPAL-DEBUG-CONTROL-001` (expression checkpoints) | implemented expression rules | `src/topal-debugger/tests/cli.rs`; `examples/debugger/expression-stepping.t`; `examples/debugger/expression-stepping.debug` | `topal-language`, `topal-debugger` |
| `TOPAL-DEBUG-CONTROL-001` (historical expression inspection) | implemented expression rules | `src/topal-debugger/tests/cli.rs`; `examples/debugger/expression-inspection.debug` | `topal-language::Session::inspect`, `topal-debugger` |
| `TOPAL-INTP-SUBSET-023` | `TOPAL-SYN-LEX-001`, `TOPAL-SYN-BIND-001` | unit tests in `topal-syntax` and `topal-language`; `src/topal-interpreter/tests/cli.rs`; `examples/interpreter/bindings-and-discard.t` | `topal-syntax`, `topal-language`, `topal-interpreter`, `topal-lsp`, `topal-debugger` |
| `TOPAL-INTP-SUBSET-024` | `TOPAL-SYN-GRAMMAR-001`, `TOPAL-TYPE-PRODUCT-001` | unit tests in `topal-syntax` and `topal-language`; `src/topal-interpreter/tests/cli.rs`; `examples/interpreter/strings-and-products.t` | `topal-syntax`, `topal-language`, `topal-interpreter`, `topal-lsp`, `topal-debugger` |
| uniform product-field rule | `TOPAL-SYN-GRAMMAR-001`, `TOPAL-TYPE-PRODUCT-001` | unit tests in `topal-syntax` and `topal-language`; `src/topal-interpreter/tests/cli.rs` | `topal-syntax`, `topal-language`, `topal-interpreter`, `topal-lsp` |
| `TOPAL-INTP-SUBSET-025` | `TOPAL-SYN-GRAMMAR-001`, `TOPAL-TYPE-PRODUCT-001` | unit tests in `topal-language`; `src/topal-interpreter/tests/cli.rs`; `examples/interpreter/strings-and-products.t` | `topal-syntax`, `topal-language`, `topal-interpreter`, `topal-lsp`, `topal-debugger` |
| `TOPAL-INTP-SUBSET-026` | `TOPAL-SYN-STRING-001`, `TOPAL-TYPE-CALL-001`, `TOPAL-STRING-CONCAT-001`, `TOPAL-STRING-LITERAL-COMPOSE-001` | unit tests in `topal-language`; `src/topal-interpreter/tests/cli.rs`; `examples/interpreter/strings-and-products.t` | `topal-syntax`, `topal-language`, `topal-interpreter`, `topal-lsp`, `topal-debugger` |
| `TOPAL-INTP-SUBSET-027` | `TOPAL-TYPE-CALL-001`, `TOPAL-STRING-EMPTY-001` | unit tests in `topal-language`; `src/topal-interpreter/tests/cli.rs`; `examples/interpreter/strings-and-products.t` | `topal-syntax`, `topal-language`, `topal-interpreter`, `topal-lsp`, `topal-debugger` |
| `TOPAL-INTP-SUBSET-028` | `TOPAL-SYN-UNICODE-001`, `TOPAL-TYPE-CALL-001`, `TOPAL-STRING-CHARACTER-COUNT-001` | unit tests in `topal-source` and `topal-language`; `src/topal-interpreter/tests/cli.rs`; `examples/interpreter/strings-and-products.t` | `topal-source`, `topal-language`, `topal-interpreter`, `topal-lsp`, `topal-debugger` |
| `TOPAL-INTP-SUBSET-029` | `TOPAL-TYPE-PRODUCT-001`, `TOPAL-TYPE-EQUALITY-001` | unit tests in `topal-language`; `src/topal-interpreter/tests/cli.rs`; `examples/interpreter/equality-and-ordering.t` | `topal-language`, `topal-interpreter`, `topal-lsp`, `topal-debugger` |
| `TOPAL-INTP-SUBSET-030` | `TOPAL-TYPE-CALL-001`, `TOPAL-STRING-CHARACTER-COUNT-001`, `TOPAL-STRING-ENTRY-COUNT-001` | unit tests in `topal-language`; `src/topal-interpreter/tests/cli.rs`; `examples/interpreter/strings-and-products.t` | `topal-language`, `topal-interpreter`, `topal-lsp`, `topal-debugger` |
| `TOPAL-INTP-SUBSET-031` | `TOPAL-TYPE-CALL-001`, `TOPAL-STRING-UTF8-BYTE-COUNT-001` | unit tests in `topal-language`; `src/topal-interpreter/tests/cli.rs`; `examples/interpreter/string-utf8-byte-count.t` | `topal-language`, `topal-interpreter`, `topal-lsp`, `topal-debugger` |
| `TOPAL-LSP-COMPLETION-001` | implemented named root operations | unit tests in `topal-lsp`; `src/topal-lsp/tests/protocol.rs` | `topal-lsp` |
| `TOPAL-INTP-SUBSET-033` | `TOPAL-SYN-UNICODE-001`, `TOPAL-TYPE-CALL-001`, `TOPAL-STRING-NORMALIZE-NFC-001` | unit tests in `topal-source` and `topal-language`; `src/topal-interpreter/tests/cli.rs`; `examples/interpreter/string-normalization.t` | `topal-source`, `topal-language`, `topal-interpreter`, `topal-lsp`, `topal-debugger` |
| `TOPAL-INTP-SUBSET-032` | `TOPAL-SYN-LEX-001`, `TOPAL-TYPE-CALL-001`, `TOPAL-STRING-EMPTY-PREDICATE-001` | unit tests in `topal-syntax` and `topal-language`; `src/topal-interpreter/tests/cli.rs`; `examples/interpreter/strings-and-products.t` | `topal-syntax`, `topal-language`, `topal-interpreter`, `topal-lsp`, `topal-debugger` |
| `TOPAL-INTP-SUBSET-034` | `TOPAL-SYN-GRAMMAR-001`, `TOPAL-TYPE-CALL-001`, `TOPAL-FUNCTION-STATIC-NULLARY-001` | unit tests in `topal-syntax` and `topal-language`; `src/topal-interpreter/tests/cli.rs`; `src/topal-debugger/tests/cli.rs`; `examples/interpreter/static-nullary-functions.t`; `examples/debugger/static-function-call.t` | `topal-syntax`, `topal-language`, `topal-interpreter`, `topal-lsp`, `topal-debugger` |
| `TOPAL-INTP-SUBSET-035` | `TOPAL-SYN-UNICODE-001`, `TOPAL-TYPE-CALL-001`, `TOPAL-STRING-NORMALIZE-NFD-001` | unit tests in `topal-source` and `topal-language`; `src/topal-interpreter/tests/cli.rs`; `examples/interpreter/string-normalization-nfd.t` | `topal-source`, `topal-language`, `topal-interpreter`, `topal-lsp`, `topal-debugger` |
| `TOPAL-INTP-SUBSET-036` | `TOPAL-SYN-GRAMMAR-001`, `TOPAL-TYPE-CALL-001`, `TOPAL-FUNCTION-STATIC-UNARY-001` | unit tests in `topal-syntax` and `topal-language`; `src/topal-interpreter/tests/cli.rs`; `src/topal-debugger/tests/cli.rs`; `examples/interpreter/static-nullary-functions.t`; `examples/debugger/static-unary-function.t` | `topal-syntax`, `topal-language`, `topal-interpreter`, `topal-lsp`, `topal-debugger` |
| `TOPAL-INTP-SUBSET-037` | `TOPAL-SYN-GRAMMAR-001`, `TOPAL-TYPE-CALL-001`, `TOPAL-FUNCTION-STATIC-BINARY-001` | unit tests in `topal-syntax` and `topal-language`; `src/topal-interpreter/tests/cli.rs`; `src/topal-debugger/tests/cli.rs`; `examples/interpreter/static-product-functions.t`; `examples/debugger/static-product-function.t` | `topal-syntax`, `topal-language`, `topal-interpreter`, `topal-lsp`, `topal-debugger` |
| `TOPAL-INTP-SUBSET-038` | `TOPAL-SYN-GRAMMAR-001`, `TOPAL-SYN-BIND-001`, `TOPAL-FUNCTION-BLOCK-001` | unit tests in `topal-syntax` and `topal-language`; `src/topal-interpreter/tests/cli.rs`; `src/topal-debugger/tests/cli.rs`; `examples/interpreter/static-product-functions.t`; `examples/debugger/static-product-function.t` | `topal-syntax`, `topal-language`, `topal-interpreter`, `topal-lsp`, `topal-debugger` |
| `TOPAL-INTP-SUBSET-039` | `TOPAL-SYN-GRAMMAR-001`, `TOPAL-FUNCTION-RETURN-001` | unit tests in `topal-syntax` and `topal-language`; `src/topal-interpreter/tests/cli.rs`; `src/topal-debugger/tests/cli.rs`; `examples/interpreter/function-return.t`; `examples/debugger/function-return.t` | `topal-syntax`, `topal-language`, `topal-interpreter`, `topal-lsp`, `topal-debugger` |
| `TOPAL-INTP-SUBSET-040` | `TOPAL-SYN-GRAMMAR-001`, `TOPAL-TYPE-CALL-001`, `TOPAL-FUNCTION-ORDINARY-001` | unit tests in `topal-syntax` and `topal-language`; `src/topal-interpreter/tests/cli.rs`; `src/topal-debugger/tests/cli.rs`; `examples/interpreter/ordinary-functions.t`; `examples/debugger/ordinary-function.t` | `topal-syntax`, `topal-language`, `topal-interpreter`, `topal-lsp`, `topal-debugger` |
| `TOPAL-INTP-SUBSET-041` | `TOPAL-TYPE-CALL-001`, `TOPAL-FUNCTION-CALL-CHAIN-001` | unit tests in `topal-language`; `src/topal-interpreter/tests/cli.rs`; `src/topal-debugger/tests/cli.rs`; `examples/interpreter/function-call-chains.t`; `examples/debugger/function-call-chain.t` | `topal-language`, `topal-interpreter`, `topal-lsp`, `topal-debugger` |
| `TOPAL-INTP-SUBSET-042` | `TOPAL-SYN-BIND-001`, `TOPAL-FUNCTION-BLOCK-001`, `TOPAL-FUNCTION-LOCAL-SCOPE-001` | unit tests in `topal-language`; `src/topal-interpreter/tests/cli.rs`; `src/topal-debugger/tests/cli.rs`; `examples/interpreter/function-local-shadowing.t`; `examples/debugger/function-local-shadowing.t` | `topal-language`, `topal-interpreter`, `topal-lsp`, `topal-debugger` |
| `TOPAL-INTP-SUBSET-043` | `TOPAL-TYPE-CALL-001`, `TOPAL-FUNCTION-OVERLOAD-001` | unit tests in `topal-language`; `src/topal-interpreter/tests/cli.rs`; `src/topal-debugger/tests/cli.rs`; `examples/interpreter/function-overloads.t`; `examples/debugger/function-overloads.t` | `topal-language`, `topal-interpreter`, `topal-lsp`, `topal-debugger` |
