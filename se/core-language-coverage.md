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
| `spec/syntax.md` | 13 | 2 | `topal-source`, `topal-syntax`, shared library resolver | static, runtime | complete |
| `spec/type-system.md` | 45 | 2 | `topal-language` shared semantics | static, runtime | complete |
| `spec/functions.md` | 36 | 4 | `topal-language` execution | runtime | complete |
| `spec/decisions.md` | 6 | 2 | `topal-syntax`, `topal-language` | static, runtime | complete |
| `spec/numbers.md` | 40 | 3 | `topal-language` value domains | runtime | complete |
| `spec/ranges.md` | 9 | 3 | `topal-language` value domains | runtime | complete |
| `spec/strings.md` | 26 | 3 | `topal-source`, `topal-language` | static, runtime | complete |
| `spec/containers.md` | 52 | 3 | `topal-language` value domains | runtime | complete |
| `spec/generators.md` | 27 | 3 | `topal-language` execution | runtime | complete |
| `spec/modules.md` | 11 | 4 | shared loader and `topal-language` | static, runtime | complete |
| `spec/abstractions.md` | 7 | 5 | `topal-semantics`, shared source tools | static | complete |
| `spec/effects.md` | 4 | 6 | `topal-semantics`, shared source tools | static, runtime | complete |
| `spec/resources.md` | 3 | 6 | `topal-semantics`, shared execution tools | static, runtime | complete |
| `spec/memory-model.md` | 9 | 6 | shared resource and memory semantics | static, runtime | complete |
| `spec/concurrency-model.md` | 12 | 7 | shared execution scheduler | static, runtime | complete |
| `spec/tasks.md` | 5 | 7 | `topal-syntax`, `topal-language`, shared execution tools | static, runtime | complete |
| `spec/serialization.md` | 22 | 8 | shared layout and serialization codecs | artifact, runtime | complete |
| `spec/generic-ir.md` | 11 | 9 | shared generic artifact and source-package identity model | artifact, compiler-only | complete |
| `spec/standard-library.md` | 26 | 9 | shared library loader and cross-tool conformance suites | static, runtime, artifact | complete |
| `spec/advent-of-code-library.md` | 2 | 9 | shared library loader and companion-library conformance suites | static, runtime | complete |
| `spec/tracing.md` | 4 | 9 | shared semantic tracing and adapters | static, runtime, artifact | complete |
| `spec/debugger-scripting.md` | 3 | 9 | `topal-debugger`, shared language variants | static, runtime | complete |
| `spec/source-documentation.md` | 7 | 9 | `topal-syntax`, `topal-language`, `topal-doc`, `topal-debugger` | static, presentation | complete |
| `spec/best-practices.md` | 7 | 9 | `topal-best-practices` catalog model and `topal-linter` contained executor | static, artifact | complete |
| `spec/diagnostics.md` | 2 | 9 | `topal-source`, source-facing tool adapters | static, presentation | complete |
| `spec/data-transfer-packages.md` | 5 | 10 | nested `std` namespaces and shared host boundary | static, runtime, platform-specific | planned |
| `spec/data-transfers.md` | 26 | 11 | ordinary Topal library and irreducible host boundary | static, runtime, platform-specific | planned |
| `spec/contracts-and-evidence.md` | 13 | 12 | `topal-syntax`, `topal-language`, `topal-semantics`, `topal-geir` | static, runtime, artifact | planned |
| `spec/resource-evidence.md` | 8 | 12 | `topal-semantics`, shared implementation selectors | static, artifact | planned |
| `spec/concurrency-implementations.md` | 9 | 12 | `topal-semantics`, shared execution providers | static, runtime | planned |
| `spec/time-and-dataflow.md` | 8 | 12 | `topal-semantics`, shared execution providers | static, runtime | planned |
| `spec/effect-handlers-and-plans.md` | 7 | 12 | `topal-semantics`, compiler-only implementation planner | static, runtime, compiler-only | planned |
| `spec/composite-layouts-and-information-flow.md` | 9 | 12 | `topal-semantics`, checked layout and policy providers | static, runtime, artifact | planned |
| `spec/compiler.md` | 97 | 13 | `topal-compiler`, shared checked frontend, platform runtime | static, runtime, artifact, compiler-only | planned |

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
