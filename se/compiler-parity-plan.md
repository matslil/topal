# Compiler parity closure plan

This plan closes the native compiler against the executable behavior already
accepted by the interpreter. The detailed feature sequence remains in the
[compiler conformance roadmap](compiler-conformance.md); this document groups
the remaining rows into reviewable, larger pull requests and defines the point
at which compiler parity is reached.

## Parity boundary

Compiler parity means that, on the supported Linux x86-64 target, `topalc`
accepts and executes every authority-compatible Topal program owned by the
interpreter with the same observable result and diagnostics. The qualification
inventory is:

- all 306 canonical programs in `examples/language`;
- all 3 programs in `examples/data-transfer`;
- all 23 Advent of Code application tests; and
- all 25 standard-library tests.

Target-independent interpretation alone does not imply that host adapters or
external device authority are available to a freestanding native artifact.
Such cases require either the compiler's specified platform adapter or an
explicit, traced target limitation; they may not disappear silently from the
inventory. The final gate also requires shared-source output comparison,
artifact-free rejection tests, O0 freestanding ELF/DWARF checks, and bounded
resource qualification.

## Closure sequence

| Gate | Cohesive work | Exit evidence | Status |
| ---: | --- | --- | --- |
| 1 | runtime/frontend closure: remaining scalar and function forms, containers, generators, ranges, constraints, and direct execution boundaries | canonical language corpus plus newly promoted external cases execute identically | in progress |
| 2 | program composition: external `use`, standard library selection, applications, build graph, public interfaces, and separate compilation metadata | data-transfer, standard-library, and application entry points resolve without interpreter-only loading | planned |
| 3 | effects and platform semantics: nonempty effects, resources, tasks/streams, layouts, locations, scheduling, time, and adapters | authority-compatible platform tests compile and execute; target limitations are explicit | planned |
| 4 | assurance surfaces: contracts/evidence, open serialization, introspection, contexts, and information flow | remaining standard-library and transfer assurance tests have native evidence | planned |
| 5 | zero-gap qualification: rule disposition audit, whole inventory comparison, optimized-level disposition, and resource baselines | no unexplained interpreter/compiler acceptance gap remains | planned |

Each gate may use more than one PR when reviewability requires it. After every
PR, this plan and the detailed roadmap must identify the still-open rows and the
next executable blocker. New compiler-specific semantics are not permitted: a
gap that exposes missing or contradictory language design stops for a human
decision under the repository authority rules.

## Current inventory

The compiler executes the complete 306-program canonical language corpus.
Gate 1 now includes constraint application over every compiler-admitted
fundamental scalar base. The first external blockers are composition-related:
data-transfer programs require external standard-library selection, and the
application and standard-library suites require their library/test application
contexts. These form Gate 2 once the remaining adjacent Gate 1 runtime forms
have been promoted.
