# Core-language completion plan

**Completion record:** all ten phases are implemented for the `v0.1`
(`design-0`) standard-library development baseline. The terminal ledger and
bootstrap handoff are maintained by Phase 10 acceptance tests.

This plan closes the gap between Topal's approved `design-0` language and the
executable toolchain needed before broad standard-library development. It does
not introduce language semantics. Human-readable design in `docs/`, the system
requirements in `se/`, and normative rules in `spec/` remain authoritative.
Deliberately deferred work in `FUTURE.md` is outside the completion boundary.

## Completion boundary

The core language is complete for standard-library development when every
non-deferred `design-0` rule has an explicit disposition and every rule that is
observable in a source tool has shared implementation and conformance evidence.
A disposition is one of:

- **runtime** — implemented by the shared execution layer and observable in the
  interpreter and debugger;
- **static** — implemented by shared syntax or semantic analysis and observable
  in the interpreter, debugger, and language server where applicable;
- **artifact** — implemented and validated by a shared representation or codec;
- **compiler-only** — intentionally not executed by the interpreter, with the
  boundary enforced by shared validation; or
- **deferred** — explicitly assigned to `FUTURE.md` or provisional by an
  authoritative design document.

An `unsupported` diagnostic is a temporary disposition, not completion, for a
non-deferred rule applicable to the invoked tool.

## Pull-request series

Each numbered entry is one cohesive pull request. Within an entry, independently
reviewable bullets are separate commits. Every language increment updates the
shared frontend and semantics, interpreter, debugger, language server, examples,
tests, and traceability wherever the feature affects them.

1. **Conformance ledger and closure gates**
   - Inventory stable specification rules and record their tool disposition.
   - Validate that every stable rule has an owner, disposition, and evidence or
     an explicit completion phase.
   - Add a repository check which prevents silent loss of coverage.

2. **Frontend, type system, and execution closure**
   - Replace remaining implemented-subset grammar restrictions for approved
     expressions, declarations, patterns, and blocks.
   - Complete classifier, constraint, conversion, function, decision, totality,
     and recursion semantics shared by source tools.
   - Preserve actionable diagnostics, semantic traces, and debugger history for
     every new execution transition.

3. **Core values, numbers, ranges, containers, errors, and generators**
   - Close all non-deferred primitive and exact-number rules.
   - Complete algebraic values, collection construction and operations, and
     range behavior required independently of a standard library.
   - Complete structured result/error propagation and generator lifecycle rules.

4. **Modules, packages, and constructed contexts**
   - Implement source-tree module discovery, canonical names, visibility, and
     qualified resolution.
   - Implement package/application/library metadata and language-context
     selection required to run multi-file programs.
   - Implement constructed contexts, `use`, and dependency injection boundaries.

5. **Generic abstractions, capabilities, and interfaces**
   - Implement type patterns, constraints, generic instantiation, and evidence.
   - Implement capability declarations, derivation, coherence, and selection.
   - Implement function and message interfaces with context and packaged
     implementations.

6. **Effects, resources, locations, and memory semantics**
   - Implement effect rows, inference, polymorphism, ordering, and containment.
   - Implement affine resource movement, deterministic cleanup, and lifetimes.
   - Implement semantic locations and checked access while preserving the
     specified memory-event model.

7. **Tasks, message transactions, and concurrency**
   - Implement structured task creation, lifetime, cancellation, and failure.
   - Implement request/reply and stream message protocols with backpressure.
   - Implement deterministic scheduling evidence and debugger stepping that
     follows a message transaction as one logical call path.

8. **Layouts and serialization**
   - Implement scalar, product, sum, sequence, and text layout construction.
   - Implement checked reading and writing with access, endian, packing, and
     absence policies.
   - Implement canonical versioned serialization, safe deserialization, golden
     vectors, and malformed-input coverage.

9. **Static introspection and generic artifacts**
   - Implement typed introspection views, visibility, identity, and relations.
   - Implement validated canonical generic intermediate artifacts and evidence
     preservation.
   - Enforce the interpreter/compiler boundary for compiler-only operations with
     shared deterministic diagnostics.

10. **Core-language acceptance and standard-library handoff**
    - Eliminate temporary unsupported dispositions for applicable non-deferred
      rules and close the conformance ledger.
    - Run cross-tool examples, trace comparisons, malformed-source/property
      suites, concurrency litmus tests, and serialization compatibility tests.
    - Publish the supported core revision and a standard-library bootstrap
      contract without adding library policy to the language core.

## Stack maintenance

The first pull request is based on current `main`. Each successor is developed
from the exact head of its predecessor and uses that predecessor as its review
base while it is open. Whenever a predecessor merges, the successor is rebased
onto updated `main`, force-pushed with lease, and its pull-request base is moved
to `main`. Before opening another successor, every open pull request in this
series must be reported mergeable by GitHub and its diff must contain only its
own phase.

## Per-phase acceptance

Every phase must pass formatting, warning-free linting, the full workspace test
suite, focused conformance tests, and `git diff --check`. A PR description must
record its risk assessment, governing stable IDs, validation, deliberate
deferrals, and any approved design interpretation. A phase is not complete
until its branch is published as a mergeable pull request.
