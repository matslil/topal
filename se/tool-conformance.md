# Source-tool conformance matrix

This matrix refines `TOPAL-REQ-TOOLS-001` for the accepted `design-0`
specification. Each row applies to every stable rule in the named specification
file. It makes applicability explicit for the interpreter, language server,
linter, and source debugger instead of treating a mention anywhere in the
traceability ledger as proof for every tool.

The dispositions are:

- **direct** — the tool owns observable implementation and functional evidence;
- **shared** — the tool consumes the listed shared implementation and its
  tool-facing corpus verifies that the feature is accepted without divergence;
- **boundary** — the rule is compiler- or artifact-only and the tool enforces a
  tested rejection boundary; and
- **not-applicable** — the rule has no behavior in that tool under its approved
  tool requirements.

A shared disposition does not claim that an analysis-only tool executes runtime
semantics. It means the tool recognizes the same source representation and does
not introduce a private interpretation. Runtime results remain directly
applicable to the interpreter and debugger.

| Specification | Interpreter | LSP | Linter | Debugger | Evidence |
| --- | --- | --- | --- | --- | --- |
| `spec/abstractions.md` | direct | shared | shared | shared | `topal-semantics`; interpreter and debugger functional suites; LSP and linter source corpora |
| `spec/best-practices.md` | not-applicable | direct | direct | not-applicable | `topal-best-practices`; `topal-linter`; LSP lint adapter tests |
| `spec/concurrency-model.md` | shared | not-applicable | shared | shared | `topal-semantics`; task source tests; contained task-rule views; reversible message tests |
| `spec/containers.md` | direct | shared | shared | shared | `topal-language`; cross-tool source corpora |
| `spec/data-transfer-packages.md` | boundary | shared | shared | boundary | explicit host-capability boundary; package source corpora; debugger replay boundary |
| `spec/debugger-scripting.md` | not-applicable | shared | not-applicable | direct | `topal-debugger`; debugger scripts; LSP debug-variant completion tests |
| `spec/decisions.md` | direct | shared | shared | shared | `topal-syntax`; `topal-language`; cross-tool source corpora |
| `spec/diagnostics.md` | shared | direct | direct | shared | `topal-source`; tool adapter and functional suites |
| `spec/effects.md` | shared | shared | shared | shared | `topal-semantics`; effect source corpus and debugger history tests |
| `spec/functions.md` | direct | shared | shared | shared | `topal-language`; cross-tool source and standard-library corpora |
| `spec/generators.md` | direct | shared | shared | shared | `topal-language`; cross-tool source corpora and close-history tests |
| `spec/generic-ir.md` | boundary | boundary | not-applicable | boundary | `topal-geir` compiler-only boundary matrix |
| `spec/memory-model.md` | shared | not-applicable | not-applicable | shared | `topal-semantics` memory-model tests; shared execution boundary |
| `spec/modules.md` | direct | shared | shared | shared | shared module loader; directory-application and library corpus tests |
| `spec/numbers.md` | direct | shared | shared | shared | `topal-language`; cross-tool source corpora |
| `spec/ranges.md` | direct | shared | shared | shared | `topal-language`; cross-tool source corpora |
| `spec/resources.md` | shared | not-applicable | shared | shared | `topal-semantics` ownership tests; shared semantic views and execution |
| `spec/serialization.md` | direct | shared | shared | shared | `topal-serialization`; source serialization and checked-location corpora |
| `spec/source-documentation.md` | shared | shared | not-applicable | direct | `topal-syntax`; `topal-language`; LSP corpus; debugger help tests |
| `spec/standard-library.md` | direct | shared | shared | shared | shared library application and cross-tool conformance suites |
| `spec/strings.md` | direct | shared | shared | shared | `topal-source`; `topal-language`; cross-tool source corpora |
| `spec/syntax.md` | shared | direct | shared | shared | `topal-source`; `topal-syntax`; all four source-tool corpora |
| `spec/tasks.md` | direct | shared | shared | shared | task examples, contained rule views, and reversible transaction tests |
| `spec/tracing.md` | direct | not-applicable | shared | direct | `topal-semantics`; interpreter test traces; supplied lint trace views; debugger history |
| `spec/type-system.md` | direct | shared | shared | shared | `topal-semantics`; `topal-language`; cross-tool source corpora |

The repository conformance test expands these domain rows to every stable rule,
requires one row per stable specification file, validates every disposition,
and requires a nonempty evidence description for each domain. Adding a rule or
specification domain therefore cannot silently inherit an unreviewed tool
disposition.
