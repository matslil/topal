# Functional composition and DSL patterns

## FD-01 — Bulk map/filter/reduce pipeline

**Good fit.** Many independent elements undergo the same pure transformation,
followed by associative aggregation or partitioning.

**References.** Google's MapReduce model separates map and reduce so a runtime
can partition, schedule, and recover work.[^fd1] Futhark shows that bulk
operators, fusion, and flattening can compile pure array code competitively.[^fd2]

**Hardware assumptions.** Scales from SIMD CPUs to GPUs and clusters; efficient
reduction needs target-appropriate hierarchy and associative operations.

**Problem.** Element-by-element imperative control hides independence and
forces programmers to implement scheduling, synchronization, and partitioning.

**Structure.** Express transformations as pure bulk combinators; attach
associativity/commutativity evidence to reductions; fuse compatible stages and
partition only at execution boundaries.

**Limitations.** Skew, communication, intermediate materialization, and
non-associative floating point can dominate. Not every irregular dependency
fits the model.

**Core-language support required.** Higher-order functions, immutable arrays,
shape information, pure/effect classification, associative and commutative
evidence, parallel reduction semantics, fusion, and target specialization.

## FD-02 — Parser combinators

**Good fit.** A grammar benefits from reusable sequencing, choice, repetition,
and domain-specific error reporting inside a general-purpose language.

**References.** Hutton and Meijer model parsers as functions composed by
higher-order grammar operators.[^fd3]

**Hardware assumptions.** None; CPU execution is typical.

**Problem.** Hand-written recursive descent repeats control/error plumbing,
while an external generator separates grammar fragments from ordinary code.

**Structure.** Make a parser a first-class value from input state to success or
structured failure, then compose parsers with sequencing, alternative,
mapping, and repetition.

**Limitations.** Naive backtracking can be exponential, retain input, allocate
closures, and produce confusing errors. Left recursion requires special
treatment.

**Core-language support required.** First-class generic functions, algebraic
results, recursion, lazy or explicit input state, controlled backtracking, and
enough specialization/inlining to remove combinator and closure overhead.

## FD-03 — Algebraic effect handler

**Good fit.** Libraries need composable abstractions for state, exceptions,
iteration, async operations, or other effects without fixing one runtime
implementation.

**References.** Koka's row-typed algebraic effects separate operation
interfaces from handlers and describe type-directed selective-CPS lowering.[^fd4]

**Hardware assumptions.** None, but continuation representation and stack
support are target-specific.

**Problem.** Monolithic runtimes and ad-hoc callbacks make effects hard to
compose, type, interpret, or optimize.

**Structure.** Declare effect operations; an enclosing handler supplies their
meaning and may resume the captured continuation according to defined linearity.

**Limitations.** Multi-shot continuations can copy stacks and complicate
resources; handler order affects meaning; unrestricted control is difficult for
accelerators and WCET analysis.

**Core-language support required.** Extensible effect rows, typed handlers,
precise resumption multiplicity, continuation/resource cleanup semantics,
effect polymorphism, and an efficient direct/selective-CPS lowering contract.

## FD-04 — Embedded DSL with explicit interpreter

**Good fit.** A domain needs a small, analyzable vocabulary and several
interpretations such as execution, simulation, validation, optimization, code
generation, or documentation.

**References.** Spinellis catalogs implementation patterns for domain-specific
languages.[^fd5] Carette, Kiselyov, and Shan show typed, compositional “finally
tagless” embeddings with multiple interpreters.[^fd6]

**Hardware assumptions.** None; a backend may target CPU, GPU, NPU, DSP, SQL,
or a protocol engine.

**Problem.** Direct execution entangles domain meaning with one platform;
string-based code generation loses types and source structure.

**Structure.** Represent domain operations as typed constructors or an abstract
interface. Interpret the resulting program into one or more target semantics.

**Limitations.** Deep embeddings add an IR and boilerplate; shallow/tagless
forms can make whole-program inspection harder. Source locations and effects
may be lost unless preserved deliberately.

**Core-language support required.** Algebraic data and/or higher-kinded generic
interfaces, first-class functions, modules, source/effect metadata, exhaustive
interpretation, and typed target or code values.

## FD-05 — Multi-stage specialization

**Good fit.** Static configuration, shapes, protocols, or domain programs can
be evaluated once to generate a small residual hot path.

**References.** MetaML formalizes typed multi-stage programming, cross-stage
persistence, and explicit staging annotations.[^fd7]

**Hardware assumptions.** None; especially useful for kernels, DSP filters,
serializers, parsers, and fixed deployment configurations.

**Problem.** A generic abstraction repeats interpretation and dynamic checks
that are constant for a deployment.

**Structure.** Mark values and computation stages, evaluate static work, and
produce typed residual code specialized for the remaining dynamic inputs.

**Limitations.** Code size and compile time grow; uncontrolled staging can
duplicate work, leak static secrets, or make debugging difficult. Runtime code
generation may violate safety/certification constraints.

**Core-language support required.** A phase distinction, static evaluation,
typed quotation/generation or guaranteed partial evaluation, binding hygiene,
cross-stage persistence rules, resource bounds, and reproducible generated IR.

## FD-06 — Persistent immutable update

**Good fit.** Old and new versions must coexist, be shared safely, or support
snapshotting, rollback, and deterministic concurrency.

**References.** Okasaki explains path copying and structural sharing for
persistent functional data structures.[^fd8]

**Hardware assumptions.** None; pointer-rich structures favor cache-rich CPUs
more than GPUs/DSPs unless flattened.

**Problem.** Copying an entire structure on each functional update wastes time
and memory; mutation destroys snapshots and complicates sharing.

**Structure.** Copy only nodes on the changed path and share all unaffected
substructure between versions.

**Limitations.** Extra indirection, allocation, reference management, and poor
locality can dominate. Amortized bounds may be unsuitable for hard real time;
flat arrays and accelerators may require another representation.

**Core-language support required.** Immutable recursive algebraic data,
structural sharing with safe lifetime management, generic containers,
complexity evidence, and representation specialization or uniqueness-based
in-place update when an old version is dead.

## Sources

[^fd1]: J. Dean and S. Ghemawat, [“MapReduce: Simplified Data Processing on Large Clusters”](https://research.google/pubs/mapreduce-simplified-data-processing-on-large-clusters/), OSDI, 2004.
[^fd2]: T. Henriksen et al., [“Futhark: Purely Functional GPU-Programming with Nested Parallelism and In-Place Array Updates”](https://futhark-lang.org/publications/pldi17.pdf), PLDI, 2017.
[^fd3]: G. Hutton and E. Meijer, [“Monadic Parser Combinators”](https://nottingham-repository.worktribe.com/output/1024440/monadic-parser-combinators), NOTTCS-TR-96-4, 1996.
[^fd4]: D. Leijen, [“Type Directed Compilation of Row-Typed Algebraic Effects”](https://www.microsoft.com/en-us/research/publication/type-directed-compilation-row-typed-algebraic-effects/), POPL, 2017.
[^fd5]: D. Spinellis, [“Notable Design Patterns for Domain-Specific Languages”](https://doi.org/10.1016/S0164-1212(00)00089-3), *Journal of Systems and Software*, 2001.
[^fd6]: J. Carette, O. Kiselyov, and C.-c. Shan, [“Finally Tagless, Partially Evaluated”](https://www.cas.mcmaster.ca/~carette/publications/APLAS.pdf), APLAS, 2007.
[^fd7]: W. Taha and T. Sheard, [“MetaML and Multi-Stage Programming with Explicit Annotations”](https://doi.org/10.1016/S0304-3975(00)00053-0), *Theoretical Computer Science*, 2000.
[^fd8]: C. Okasaki, [“Persistence”](https://www.cambridge.org/core/books/abs/purely-functional-data-structures/persistence/BEB36D6BF24898A7CA3A188DA5C35ED1), in *Purely Functional Data Structures*, Cambridge University Press, 1998.
