# Interprocedural and feedback-directed optimizations

## CO-IF-001 — Inlining and partial inlining

**Found in.** LLVM supplies full and partial production inliners; GCC enables
small, indirect, and partial inlining at higher optimization levels.[^if1][^if2]
Go's profile-guided optimizer uses hotness to make more aggressive inlining
decisions.[^if3]

**Problem.** Calls add control and convention overhead and hide constants,
types, effects, and surrounding context from intraprocedural passes.

**Approach.** Substitute a callee body into its caller, or inline only a hot or
guarded part while retaining a call for the remainder. Re-run simplification
and specialization on the exposed body.

**Assumptions and evidence.** Requires exact call target, compatible semantics
and ABI, recursion controls, body availability, and a cost estimate based on
size, hotness, call overhead, downstream simplification, registers, and cache.

**Limitations and interactions.** Inlining can multiply code, compile time,
debug complexity, and register pressure; it can harm instruction-cache locality
or prevent independent compilation. Refusing it may block constant propagation,
devirtualization chains, and allocation elimination.

**Characteristic weighting.** Speed-oriented policies value hot call removal
and enabled simplification; size-oriented policies inline only when the result
shrinks or exposes larger deletion. Cold paths, recursive growth, and target
instruction-cache pressure lower the budget.

**Topal relevance.** Topal owns semantic specialization and can expose exact
private call graphs. LLVM should decide ordinary low-level inlining from exact
signatures, with Topal-proved attributes retained.

## CO-IF-002 — Devirtualization and guarded direct calls

**Found in.** GCC exposes direct and speculative devirtualization in optimized
pipelines.[^if2] Go PGO replaces hot indirect interface calls with guarded
direct calls and can then inline the direct target.[^if4]

**Problem.** Dynamic dispatch costs an indirect branch and prevents target-
specific reasoning about the callee.

**Approach.** Prove a unique target, or test a likely receiver/callee identity
and issue a direct call on the common path with the original dispatch as
fallback.

**Assumptions and evidence.** Closed-world devirtualization needs complete type
or implementation knowledge. Speculation needs a valid identity test, preserved
fallback, representative profile, and target branch/indirect-call costs.

**Limitations and interactions.** Open-world loading, separate compilation, or
reflection can invalidate uniqueness assumptions. Guards and cloned paths grow
code and can mispredict when profiles drift. A direct target often matters
mainly because it enables inlining and specialization.

**Characteristic weighting.** Compare indirect-call and missed-inlining cost
with guard, fallback, code-size, cache, and misprediction costs. Confidence and
hotness should be recorded rather than collapsed into target capability.

**Topal relevance.** Topal's evidence and artifact identities can distinguish
closed implementation sets from open boundaries. Guarded plans must never treat
profile likelihood as semantic proof.

## CO-IF-003 — Function cloning and argument specialization

**Found in.** LLVM documents argument promotion and interprocedural constant
propagation; GCC exposes interprocedural constant propagation, scalar
replacement, value-range propagation, and cloning.[^if1][^if2]

**Problem.** One general function retains branches, generic representation, or
indirection even though important callers provide known values, ranges, types,
or evidence.

**Approach.** Create a private clone parameterized by stable call-site facts,
substitute those facts, simplify the body, and route matching calls to it while
retaining a general version where needed.

**Assumptions and evidence.** Facts must be exact for every redirected call and
must include relevant effects, representation, ownership, and target identity.
Profitability needs call frequency, simplification gain, clone count, and code
cache estimates.

**Limitations and interactions.** Too many value combinations cause code
explosion and compile-time growth. Separate artifacts may not expose bodies or
may require a stable unspecialized interface. Specialization can reduce sharing
and harm instruction locality.

**Characteristic weighting.** Value removed branches, unboxing, and enabled
vectorization against clone size, compile time, cache pressure, artifact
reusability, and dispatch overhead. Cap versions per semantic function.

**Topal relevance.** Topal explicitly requires compiler-owned specialization
evidence for hard code-shape guarantees. This is a central implementation-plan
operation; LLVM may additionally specialize machine-level arguments.

## CO-IF-004 — Escape analysis and scalar replacement

**Found in.** HotSpot classifies allocations by escape state and removes scalar-
replaceable heap objects and associated locks.[^if5] LLVM's scalar-replacement
and argument-promotion passes similarly split aggregates and promote memory to
SSA values.[^if1]

**Problem.** Temporary aggregate allocation introduces heap traffic, lifetime
management, indirection, and synchronization even when identity never escapes.

**Approach.** Prove the object does not escape relevant scope, replace fields
with independent scalar SSA values, remove allocation and locking, or place
storage in a cheaper bounded region.

**Assumptions and evidence.** Needs whole-use escape, alias, identity, lifetime,
weak-reference, finalization, exception/failure, concurrency, and debug
analysis. Field access and construction must be decomposable without changing
layout-observable boundaries.

**Limitations and interactions.** Opaque calls and storage defeat proof.
Partial escape creates difficult merge points. Scalarization increases SSA
values and registers and may be worse for large aggregates or ABI crossings.

**Characteristic weighting.** Balance allocation, collection, locks, cache and
indirection savings against register pressure, spills, code size, analysis cost,
debug observability, and aggregate-copy instructions available on the target.

**Topal relevance.** Immutable values plus ownership/exclusivity evidence are a
strong basis, but Topal resource guarantees decide whether allocation is
observable. Selected representation stays private and target-qualified.

## CO-IF-005 — Link-time and whole-program optimization

**Found in.** GCC LTO merges serialized GIMPLE across compilation units and
performs ordinary interprocedural optimization with linker visibility
information.[^if2][^if6] LLVM supports full and ThinLTO over LLVM bitcode.[^if7]

**Problem.** Separate compilation hides call targets, implementations, global
uses, and visibility, limiting cross-module optimization.

**Approach.** Retain validated intermediate representation and summaries until
link or a distributed thin-link phase, compute a program-wide call/visibility
view, import selected bodies, internalize symbols, and optimize across units.

**Assumptions and evidence.** Requires exact artifact and target compatibility,
trusted IR validation, symbol-resolution rules, complete external-visibility
information, reproducible cache keys, and bounded import/partition policy.

**Limitations and interactions.** Full LTO increases link memory, time, and
incremental rebuild scope. Thin summaries are scalable but less complete.
Dynamic loading, foreign symbols, and public ABI boundaries constrain
internalization.

**Characteristic weighting.** Trade runtime and code-size gains against build
latency, peak compiler memory, cacheability, parallelism, artifact portability,
and debugging. Thin/import budgets target high-value hot edges.

**Topal relevance.** Canonical GEIR is the semantic boundary; LLVM bitcode may
be only a rebuildable, exact-toolchain native payload. Whole-program plans must
retain evidence, target, and dependency identity.

## CO-IF-006 — Profile-guided optimization

**Found in.** GCC supports instrumentation and sampled feedback for branch,
call, value, layout, and inlining decisions.[^if2] Go accepts representative
CPU profiles to guide inlining and devirtualization.[^if3][^if4]

**Problem.** Static heuristics cannot reliably know which paths, calls, values,
or functions dominate a deployed workload.

**Approach.** Collect execution counts or samples, match them to a particular
program identity, and bias legal transformations such as layout, inlining,
cloning, vectorization, and cold-code separation.

**Assumptions and evidence.** Profiles must be compatible, sufficiently
representative, and mapped despite source/build changes. A profile supplies
likelihood and frequency, never semantic truth. Reproducible builds need exact
profile identity and deterministic use rules.

**Limitations and interactions.** Collection adds operational cost and may leak
workload information. Stale or narrow profiles overfit and regress other uses.
Instrumentation perturbs execution; sampling is less exact. Unobserved paths
still require correct general code.

**Characteristic weighting.** Weight hot-path time and layout heavily while
controlling code size, startup, collection/build cost, profile confidence,
tail latency, and worst-case behavior. Safety and resource ceilings remain hard
constraints independent of frequency.

**Topal relevance.** Profile identity belongs in the implementation plan and
artifact metadata. Topal can use feedback for profitability only after semantic
legality; LLVM's standard PGO pipeline can consume emitted profiles later.

## Sources

[^if1]: LLVM Project, [LLVM's Analysis and Transform Passes](https://llvm.org/docs/Passes.html).
[^if2]: GNU Project, [Optimize Options](https://gcc.gnu.org/onlinedocs/gcc/Optimize-Options.html).
[^if3]: Go Project, [Profile-guided optimization](https://go.dev/doc/pgo).
[^if4]: Go Project, [Profile-guided optimization in Go 1.21](https://go.dev/blog/pgo).
[^if5]: Oracle, [Java HotSpot Virtual Machine Performance Enhancements](https://docs.oracle.com/en/java/javase/17/vm/java-hotspot-virtual-machine-performance-enhancements.html).
[^if6]: GNU Project, [LTO Overview](https://gcc.gnu.org/onlinedocs/gccint/LTO-Overview.html).
[^if7]: LLVM Project, [LLVM Link Time Optimization: Design and Implementation](https://llvm.org/docs/LinkTimeOptimization.html).
