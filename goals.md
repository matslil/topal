# Topal design goals

Topal is an experimental, general-purpose language built from a small set of
recursively composable concepts. Its source language is pure and functional,
while the compiler may use mutation, in-place updates, and parallel execution
when these preserve the program's meaning.

## Core model

- Everything exposed by the language is a first-class object, including values,
  types, algorithms, modules, patterns, constraints, effects, and protocols.
- Every object has a distinct kind. In particular, types describe data and
  algorithms transform objects; the two cannot be mixed even though algorithms
  may construct statically deterministic types, constraints, or other algorithms.
- Definitions share one recursive construction model rather than separate
  macro, template, class, and [module systems](modules.md).
- Composition is the fundamental operation. Syntax is easy for humans and tools
  to parse, evaluates left to right, and has no special precedence hierarchy.
- Typed [static introspection](introspection.md) exposes visible semantic
  structure through qualified `lang` operations. Static algorithms can inspect
  and construct language objects without a separate macro language or implicit
  runtime reflection metadata.
- Category theory and concatenative programming are sources of semantic and
  syntactic inspiration, not terminology that users must learn.

## Values, constraints, and evidence

- Source-level values are immutable. Changing state is represented by producing
  a value for a new scope; the compiler may implement this with safe mutation.
- Users can define constraints such as `CamelCase Text`. Static inputs are
  checked at compile time; dynamic inputs return `Result` from runtime
  validation. Successful validation produces reusable evidence.
- Dependent and existential types retain relationships involving runtime-known
  sizes, identities, versions, and states. Checked values are the fallback when
  such evidence cannot be represented statically.
- Bounds, [resource lifetimes](resources.md), and programmer-defined invariants
  are checked by construction. Undefined behavior is not part of the safe
  language.
- [Measured quantities](units.md) carry statically checked dimensions and
  scales; programmer-defined units compose using language-defined prefixes.

## Algorithms and effects

- Algorithms are total by default and return errors explicitly as `Result`
  values; there are no exceptions. Results use a common, contextual
  [error model](errors.md) across modules.
- Effects complement regular types by describing observable interactions and
  the resources they touch. They make ordering and parallelization constraints
  visible without hiding failure control flow.
- Typed [environments](environments.md) provide fixed diagnostic operations,
  execution context, and service capabilities without process-global variables
  or shared mutable application state.
- Infinite algorithms exist only as productive [generators](generators.md): every request either
  yields, ends, fails, cancels, or suspends for an external event in finite
  computation. Consumers determine whether the enclosing computation terminates.
- Logging and tracing can be attached without modifying the observed algorithm.
  Diagnostic effects may be unordered when their ordering is not semantic.
- Compact [unit-test tables](testing.md) supply concrete inputs, mocked
  dependency results, and expected results. The compiler verifies each row and
  certifies coverage of every feasible structural path, collapsing loops and
  structural recursion to zero versus one-or-more repetitions.

## Parallelism and communication

- Parallelism is part of the language model. Independent loop iterations are
  isolated calls and may run concurrently; dependencies are explicit to both
  programmer and compiler.
- Scheduling and performance may vary, but semantic results are deterministic.
  Ordered folds preserve order, while parallel reductions require appropriate
  algebraic laws.
- Data races and deadlocks are compile errors. Safe code has no direct mutex or
  equivalent primitive; the compiler introduces synchronization when required.
- Structured [task scopes](tasks.md), resource-aware effects, linear capabilities, and
  first-class protocols describe ownership, ordering, message passing, and
  request/reply dependencies. The compiler derives and explains the resulting
  dependency graph.
- The checker may conservatively reject unusual safe programs, but common safe
  designs should be easy to express and failures should identify the missing
  dependency, ordering rule, or proof.

## External systems and hardware

- Foreign-language, application, and network boundaries are described with
  typed protocols and explicit validation rather than unchecked conventions.
- Hardware [layouts and addressed storage](layouts.md) separate immutable
  semantic values from their encoding, access rights, address ranges, offsets,
  and locations. Access capabilities specify legal read/write sizes and
  resource identity.
- Hardware access is an ordered effect: reads and writes cannot be duplicated,
  removed, cached, or reordered unless their declarations allow it.
- Typestate protocols and linear capabilities describe device state changes and
  interference between registers or other views of the same resource.

## Sources of inspiration

- [Joy: Forth's Functional Cousin](https://www.complang.tuwien.ac.at/anton/euroforth/ef01/thomas01a.pdf) — functional concatenative composition.
- [Call-by-push-value: a subsuming paradigm](https://research.birmingham.ac.uk/en/publications/call-by-push-value-a-subsuming-paradigm/) — the separation of values and computations.
- [The Categorical Abstract Machine](https://www.sciencedirect.com/science/article/pii/0167642387900207) — categorical terms as executable code.
- [Pony reference capabilities](https://tutorial.ponylang.io/reference-capabilities/reference-capabilities.html) — object capabilities and data-race freedom.
- [Asynchronous session-based concurrency](https://arxiv.org/abs/2412.08232) — protocols and deadlock freedom by typing.
- [Idris 2 totality](https://idris2.readthedocs.io/en/stable/tutorial/typesfuns.html#totality) — total functions and dependent types.
- [Programming in ATS](https://ats-lang.sourceforge.net/DOCUMENT/INT2PROGINATS/HTML/INT2PROGINATS-BOOK-onechunk.html) — theorem-assisted resource and bounds safety.
- [The Koka language](https://koka-lang.github.io/koka/doc/book.html) — inferred effect types and functional semantics.
- [Futhark uniqueness types](https://futhark-lang.org/blog/2022-06-13-uniqueness-types.html) — safe in-place updates in a parallel functional language.
