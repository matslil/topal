# Standard-library implementation plan

This plan turns the approved standard-library architecture into a stacked pull
request series. It refines the bootstrap contract in
[`standard-library-bootstrap.md`](standard-library-bootstrap.md) without giving
library code privileges unavailable to an ordinary versioned Topal package.

## Architecture

The authoritative library is Topal source. A shared package loader constructs
one versioned, visibility-checked semantic module graph. The interpreter and
source debugger execute definitions from that graph; the compiler lowers the
same definitions and identities to validated GEIR. Cached checked-source and
GEIR artifacts are derived data and must be rejected when their source,
dependency, language revision, Unicode revision, or artifact revision differs.

The bootstrap core retains only irreducible semantics: literals and fundamental
representations, algebraic construction and matching, representation-dependent
operations, exact arithmetic primitives, pinned Unicode primitives, task and
resource primitives, static evidence, introspection, and fundamental trace
events. Algorithms expressible without hidden representation access belong in
ordinary library source. Compiler substitution is optional and must remain
observationally equivalent to that source implementation.

Platform facilities are separate explicit packages. They are not part of the
fundamental-type library and enter through published capabilities, effects,
protocols, checked layouts, and ordinary package construction.

## Source organization

The initial package is organized by semantic responsibility:

```text
library/
├── package.t
├── library.t
├── fundamental/
├── numeric/
├── text/
├── collection/
├── iteration/
└── testing/
```

Physical layout does not bypass Topal publication. `package.t` constructs the
package context, `library.t` publishes the supported facade, child `module.t`
files assemble their interfaces, and ordinary files retain private-by-default
visibility. The public package identity and final facade name are selected in
the package-foundation PR so they are reviewed independently of later APIs.

## Pull-request stack

### 1. Package foundation and first vertical slice

- Add the ordinary versioned package and facade.
- Move filesystem/module-graph loading behind a shared API usable by source
  tools and the future compiler.
- Define dependency, language-version, Unicode-version, source-hash, and
  artifact-revision cache keys.
- Add one derived fundamental-type function and prove that interpreter,
  debugger, LSP, linter, and GEIR use the same declaration identity.

### 2. Differential conformance framework

- Execute every library example with the interpreter and checked debugger.
- Lower the same declarations through the compiler frontend to validated GEIR.
- Compare results, diagnostics, applicable semantic decisions, and trace data.
- Add Topal-authored law tests, malformed-package tests, and version mismatch
  tests before the public surface expands.

### 3. Boolean, Unit, ordering, and comparison

- Add derived Boolean predicates and composition.
- Add utilities over the three-way ordering result, including `min`, `max`,
  range predicates, and generic comparison laws.
- Add Unit/completion traversal helpers without conflating `Unit` and
  `Completed`.

### 4. Optional, Result, Error, and decisions

- Add mapping, chaining, filtering, fallback, recovery, traversal, and
  aggregation functions.
- Preserve explicit error vocabularies, occurrence domains, causes, details,
  and provenance.
- Require callers to supply the semantic error when converting absence into
  failure.

### 5. Exact numbers

- Extend `Int`, `Nat`, and `Rational` with sign, absolute value, bounds,
  distance, Euclidean quotient/remainder, GCD, LCM, powers, and fraction
  utilities.
- Add checked conversions and reusable finite, positive, nonzero, and bounded
  constraints.
- Keep parsing and formatting separate from arithmetic semantics.

### 6. Modular numbers, ranges, and indexes

- Add explicit modular construction and conversion helpers.
- Add range containment, intersection, splitting, and traversal.
- Add finite-index construction and checked `Nat` boundary conversion.
- Preserve the distinction between exact constraints and modular arithmetic.

### 7. Character, String, and Unicode

- Add classification, search, prefix/suffix, split/join, trim, replacement,
  normalization-aware, casing, and case-folding algorithms.
- Preserve the distinction between user-perceived characters, exact Unicode
  sequences, and encoded bytes.
- Pin every conformance vector to the selected Unicode revision.

### 8. Bytes, encodings, and binary conversion

- Add byte-sequence construction and traversal.
- Add checked text encoding/decoding and explicit malformed-input results.
- Integrate with layouts and native serialization and add round-trip and
  malformed-data properties.

### 9. List and generic sequence algorithms

- Add map, filter, fold, reduce, scan, collect, flatten, zip, search,
  partition, grouping, sorting, and generator-based lazy variants.
- Target the narrowest applicable capability instead of concrete `List`
  representation.
- Specify termination, short-circuit behavior, order, multiplicity, and
  complexity guarantees.

### 10. Array, Map, Set, and Bag

- Add checked array access and transformations.
- Add map lookup/update/merge, set algebra, and bag multiplicity operations.
- Retain immutable source semantics while allowing proven in-place compiler
  implementation.
- Derive equality, ordering, hashing, traversal, and serialization only when
  their required evidence exists.

### 11. Formatting, parsing, and display

- Add explicit interfaces and implementations for fundamental types.
- Keep locale and presentation policy explicit.
- Return structured parse failures and retain a tool bootstrap formatter which
  does not depend on a successfully loaded standard library.

### 12. Optimization contracts and compiler substitutions

- Publish laws and capability evidence for optimizable functions.
- Recognize exact structural identities rather than source spellings.
- Compare specialized compiled execution against the ordinary Topal reference
  implementation.
- Keep every substitution optional for correctness.

## Per-PR evidence

Every increment updates the shared library source, interpreter, scripted
debugger, LSP, linter, compiler/GEIR path, commented examples, tests,
specification, and traceability wherever applicable. Each public operation has:

- an exact type and error contract;
- laws and capability requirements;
- interpreter and compiler-facing conformance evidence;
- boundary, malformed-input, and version tests;
- a commented shared example; and
- complexity and allocation guarantees when observable.

The interpreter implementation is the initial executable reference, not a
separate native library definition. Native acceleration and compiler lowering
are tested against the ordinary source behavior.

## Stack maintenance

The first implementation PR is based on the head containing this plan. Each
successor is based on the exact head of its predecessor and uses that branch as
its review base while both are open. When a predecessor merges, its successor
is rebased onto updated `main`, force-pushed with lease, and retargeted to
`main`. Before opening another PR, every open PR in the series must report
mergeable and the new diff must contain only its own phase.

Every phase passes formatting, warning-free workspace linting, the complete
workspace test suite, focused conformance tests, and diff validation. A phase
is complete only after publication as a mergeable PR.
