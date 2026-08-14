# Core-language coverage ledger

This ledger assigns every stable rule in `spec/` to a completion phase and an
implementation owner. Rows apply to every `### TOPAL-*` rule in the named file;
the rule count is checked mechanically. Detailed rule-to-test links continue to
live in [`traceability.md`](traceability.md) and are extended by each phase.

`planned` means that at least one rule in the domain still lacks its terminal
evidence. A phase changes its row to `complete` only after every rule has the
listed disposition or an individually recorded, authoritative deferral.

| Specification | Rules | Phase | Owner | Terminal disposition | Status |
| --- | ---: | ---: | --- | --- | --- |
| `spec/syntax.md` | 11 | 2 | `topal-source`, `topal-syntax` | static | planned |
| `spec/type-system.md` | 45 | 2 | `topal-language` shared semantics | static, runtime | planned |
| `spec/functions.md` | 29 | 2 | `topal-language` execution | runtime | planned |
| `spec/decisions.md` | 6 | 2 | `topal-syntax`, `topal-language` | static, runtime | planned |
| `spec/numbers.md` | 40 | 3 | `topal-language` value domains | runtime | complete |
| `spec/ranges.md` | 7 | 3 | `topal-language` value domains | runtime | complete |
| `spec/strings.md` | 24 | 3 | `topal-source`, `topal-language` | static, runtime | complete |
| `spec/containers.md` | 45 | 3 | `topal-language` value domains | runtime | complete |
| `spec/generators.md` | 27 | 3 | `topal-language` execution | runtime | complete |
| `spec/modules.md` | 11 | 4 | shared loader and `topal-language` | static, runtime | complete |
| `spec/abstractions.md` | 7 | 5 | `topal-semantics`, shared source tools | static | complete |
| `spec/effects.md` | 3 | 6 | `topal-semantics`, shared source tools | static, runtime | complete |
| `spec/resources.md` | 3 | 6 | `topal-semantics`, shared execution tools | static, runtime | complete |
| `spec/memory-model.md` | 9 | 6 | shared resource and memory semantics | static, runtime | complete |
| `spec/concurrency-model.md` | 12 | 7 | shared execution scheduler | static, runtime | planned |
| `spec/serialization.md` | 16 | 8 | shared layout and serialization codecs | artifact, runtime | planned |
| `spec/generic-ir.md` | 10 | 9 | shared generic artifact model | artifact, compiler-only | planned |

## Cross-tool evidence

Runtime and static source-language rows are not complete until applicable
interpreter, debugger, and language-server behavior is covered. Artifact rows
require golden, malformed-input, and compatibility evidence. Compiler-only rows
require a shared validated boundary so the interpreter cannot accidentally
assign runtime meaning to them.

The final acceptance phase verifies that this ledger, `se/traceability.md`, the
tool requirements, examples, tests, and implementations agree. It also records
individual exceptions where a rule contains both an implemented core guarantee
and explicitly deferred design.
