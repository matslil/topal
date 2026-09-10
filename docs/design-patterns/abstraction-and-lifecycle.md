# Abstraction and lifecycle patterns

## AP-01 — Strategy through semantic capability

**Good fit.** An algorithm has several interchangeable policies, and callers
should depend on promised behavior rather than a concrete representation.

**References.** The Strategy pattern in *Design Patterns*[^ap1]; Pony's
rights-carrying reference capabilities illustrate statically meaningful
capability evidence.[^ap2]

**Hardware assumptions.** None. Static selection is especially valuable on
accelerators and small targets where dynamic dispatch is costly or unavailable.

**Problem.** Hard-coded policy prevents reuse; nominal inheritance can couple
the algorithm to representation; runtime dispatch can add indirect calls and
hide optimization facts.

**Structure.** Accept behavior as a first-class function or interface and
select an implementation using evidence of semantic laws. Permit static
specialization while retaining a correct generic fallback.

**Limitations.** Too many tiny strategies obscure control flow. Laws attached
to a strategy still need proof or trusted evidence. Dynamic choice can inhibit
inlining and device compilation.

**Core-language support required.** First-class functions or interfaces,
generic constraints, law/capability evidence, deterministic overload
selection, and specialization that can erase dictionaries or indirect calls.

## AP-02 — Adapter at a validated boundary

**Good fit.** Two components have incompatible APIs, representations, failure
models, or authority domains, especially at FFI, device, network, or storage
boundaries.

**References.** The Adapter pattern in *Design Patterns*[^ap1]; NASA requires
integrity and prerequisite checks at safety-critical boundaries.[^ap3]

**Hardware assumptions.** None, although ABI, byte order, alignment, address
space, DMA, and device protocol often make the boundary hardware-specific.

**Problem.** Letting foreign representation or authority leak inward spreads
unsafe assumptions and makes validation inconsistent.

**Structure.** One narrow component decodes and validates input, translates
errors and effects, grants only required authority, and exposes an ordinary
typed interface. Encoding performs the inverse checks on output.

**Limitations.** Adaptation can copy data, lose foreign features, or create a
semantic mismatch. It does not make an untrusted implementation safe.

**Core-language support required.** Explicit layouts/encodings, checked
conversion, result/error types, effect and authority annotations, opaque
handles, lifetime/ownership rules, and a specified foreign or device boundary.

## AP-03 — Algebraic state and exhaustive transition

**Good fit.** An object or workflow has a finite set of meaningful states and
only state-specific operations or transitions should be possible.

**References.** The State pattern in *Design Patterns*[^ap1]; Rust's official
discussion shows both an object-state encoding and a type-state alternative.[^ap4]

**Hardware assumptions.** None; finite encodings also map well to compact CPU
tags and hardware state machines.

**Problem.** Scattered flags admit impossible combinations and make omitted
cases hard to detect.

**Structure.** Represent each state as an algebraic-data alternative and make
transitions total functions over the state and input. Exhaustive matching
forces new states through every affected decision.

**Limitations.** Large cross-products can cause state explosion. Runtime-only
state checks remain possible when the type does not track the current state.

**Core-language support required.** Closed sum/product types, destructuring,
exhaustive pattern matching, unreachable-case analysis, compact tag-layout
freedom, and total/error-returning transition functions.

## AP-04 — Typestate or session-typed protocol

**Good fit.** Correctness depends on operation order: file/device lifecycle,
authentication, transactions, or multi-party communication.

**References.** Strom and Yemini introduced typestate for compile-time
operation-sequence checking.[^ap5] Session types apply a type discipline to
structured communication.[^ap6]

**Hardware assumptions.** None. Device typestate may correspond to physical
controller modes; distributed sessions assume typed endpoints.

**Problem.** Ordinary object types allow commands in invalid states, duplicate
use of one-shot authority, and abandonment of protocol obligations.

**Structure.** Index a resource or endpoint by a protocol state. Each operation
consumes the old capability and produces the next state, with branch choice
carried explicitly by data.

**Limitations.** Dynamic discovery, recovery, timeout, and multiparty failure
make protocols large. Static fidelity does not prove liveness or peer honesty.

**Core-language support required.** Linear/affine values, state-indexed types,
closed protocol descriptions, branch-sensitive type refinement, ownership
transfer, exhaustive close/cancel handling, and validated dynamic boundaries.

## AP-05 — Scoped resource acquisition and cleanup (RAII)

**Good fit.** A resource must be released exactly once on success, error,
cancellation, and early return.

**References.** The C++ Core Guidelines recommend tying resource lifetime to an
owning handle and scope exit (RAII).[^ap7] Rust's `Drop` documentation gives the
language-level deterministic cleanup model.[^ap8]

**Hardware assumptions.** None; the resource may be memory, a file, a device
lease, a lock, a task, or accelerator memory.

**Problem.** Manually paired acquire/release calls leak resources or release
them twice when control flow grows.

**Structure.** Successful acquisition returns an owning value. Lexical scope
and move rules determine one destruction point, and cleanup runs on every exit.

**Limitations.** Cleanup may fail or block; destructor order matters; cycles
and detached work complicate lifetime. Determinism of release is not a timing
deadline.

**Core-language support required.** Ownership and moves, lexical lifetimes,
deterministic finalization, structured error/cancellation unwinding,
cycle handling, and explicit fallible cleanup when failure is observable.

## AP-06 — Smart constructor plus contract

**Good fit.** Values or operations are valid only under invariants that should
be checked once and relied on throughout trusted code.

**References.** Eiffel documents Design by Contract with preconditions,
postconditions, and invariants.[^ap9] SPARK uses contracts and proof to establish
properties including absence of run-time errors.[^ap10]

**Hardware assumptions.** None. Range contracts are especially useful for
fixed-width arithmetic, buffer bounds, and device commands.

**Problem.** Rechecking invariants everywhere adds branches and complexity;
unchecked constructors admit invalid states.

**Structure.** Hide raw construction, validate or prove the invariant at one
boundary, and return a refined value carrying evidence. Operations publish
pre/postconditions and preserve the invariant.

**Limitations.** Runtime contracts cost time and may fail too late; static
proof can require annotations and solver trust. External mutation can
invalidate evidence.

**Core-language support required.** Opaque construction, refinement/constraint
types, result types, pre/postconditions, invariant propagation, proof-carrying
evidence, and a clear checked-versus-erased contract policy.

## Sources

[^ap1]: E. Gamma, R. Helm, R. Johnson, and J. Vlissides, [*Design Patterns: Elements of Reusable Object-Oriented Software*](https://www.pearson.com/en-us/subject-catalog/p/design-patterns-elements-of-reusable-object-oriented-software/P200000009480/9780201633610), Addison-Wesley, 1994.
[^ap2]: Pony, [Reference Capabilities](https://tutorial.ponylang.io/reference-capabilities/reference-capabilities.html).
[^ap3]: NASA Software Engineering Handbook, [SWE-134 — Safety Critical Software Requirements](https://swehb.nasa.gov/spaces/7150/pages/16449641/SWE-134%2B-%2BSafety%2BCritical%2BSoftware%2BRequirements?desktop=true&macroName=div).
[^ap4]: Rust Project, [Implementing an Object-Oriented Design Pattern](https://doc.rust-lang.org/stable/book/ch18-03-oo-design-patterns.html).
[^ap5]: R. Strom and S. Yemini, [“Typestate: A Programming Language Concept for Enhancing Software Reliability”](https://research.ibm.com/publications/typestate-a-programming-language-concept-for-enhancing-software-reliability), *IEEE TSE*, 1986.
[^ap6]: PLS Lab, [Session Types](https://www.pls-lab.org/Session_Types), with primary references to Honda, Vasconcelos, Kubo, Yoshida, and Carbone.
[^ap7]: Standard C++ Foundation, [C++ Core Guidelines R.1 — Manage Resources Automatically Using Resource Handles and RAII](https://isocpp.github.io/CppCoreGuidelines/CppCoreGuidelines#r1-manage-resources-automatically-using-resource-handles-and-raii-raii).
[^ap8]: Rust Project, [`std::ops::Drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html).
[^ap9]: Eiffel, [Design by Contract and Assertions](https://www.eiffel.org/doc/eiffelstudio/I2E-_Design_by_Contract_and_Assertions).
[^ap10]: AdaCore, [SPARK Reference Manual introduction](https://docs.adacore.com/spark2014-docs/html/lrm/introduction.html).
