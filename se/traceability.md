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
| `TOPAL-REQ-MODEL-001` | `TOPAL-TYPE-KIND-001` through `TOPAL-TYPE-MATCH-001`, `TOPAL-NUM-SYMBOL-001` |
| `TOPAL-REQ-SAFE-001` | `TOPAL-TYPE-SOUND-001`, `TOPAL-NUM-INT-001`, `TOPAL-NUM-ADD-001`, `TOPAL-NUM-NEG-001`, `TOPAL-NUM-SUB-001`, `TOPAL-MEM-LOCATION-001`, `TOPAL-MEM-PLAIN-001`, `TOPAL-CONC-RACE-001` |
| `TOPAL-REQ-TOTAL-001` | `TOPAL-TYPE-TOTAL-001`, `TOPAL-CONC-PROGRESS-001` |
| `TOPAL-REQ-CONC-001` | `TOPAL-CONC-DEADLOCK-001`, `TOPAL-CONC-RACE-001` |
| `TOPAL-REQ-DETERMINISM-001` | `TOPAL-NUM-INT-001`, `TOPAL-NUM-ADD-001`, `TOPAL-NUM-NEG-001`, `TOPAL-NUM-SUB-001`, `TOPAL-MEM-OPT-001`, `TOPAL-CONC-DETERMINISM-001` |
| `TOPAL-REQ-EFFECT-001` | `TOPAL-GIR-EFFECT-001`, `TOPAL-MEM-HARDWARE-001`, `TOPAL-CONC-ORDER-001` |
| `TOPAL-REQ-RESOURCE-001` | `TOPAL-MEM-LOCATION-001` through `TOPAL-MEM-LIFETIME-001` |
| `TOPAL-REQ-GENERIC-001` | `TOPAL-GIR-PURPOSE-001` through `TOPAL-GIR-COMPAT-001` |
| `TOPAL-REQ-SERIAL-001` | `TOPAL-SER-SCOPE-001` through `TOPAL-SER-CANON-001` |
| `TOPAL-REQ-TOOLS-001` | `TOPAL-SYN-SOURCE-001` through `TOPAL-SYN-DIAG-001`, `TOPAL-NUM-SYMBOL-001`, `TOPAL-GIR-VALID-001` |
| `TOPAL-REQ-INTEROP-001` | `TOPAL-TYPE-SOUND-001`, `TOPAL-NUM-ADD-001`, `TOPAL-NUM-NEG-001`, `TOPAL-NUM-SUB-001`, `TOPAL-MEM-OPT-001`, `TOPAL-CONC-DETERMINISM-001` |
| `TOPAL-REQ-TRACE-001` | all stable `TOPAL-*` specification rules |
| `TOPAL-REQ-SHARED-001` | `TOPAL-SYN-SOURCE-001`, `TOPAL-SYN-LEX-001`, `TOPAL-SYN-INDENT-001`, `TOPAL-SYN-GRAMMAR-001` |

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
