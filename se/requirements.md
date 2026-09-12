# Core language requirements

The keywords **shall**, **should**, and **may** express mandatory behavior,
recommended behavior, and permitted variation respectively.

## TOPAL-REQ-MODEL-001 — Unified object model

The language shall define values, types, functions, constraints, capabilities,
effects, interfaces, modules, patterns, and protocols within one explicit object
taxonomy and recursive construction model.

## TOPAL-REQ-SAFE-001 — Defined safe behavior

Every accepted safe program shall have defined behavior for every permitted
execution. Operations that cannot satisfy their contracts shall be rejected or
produce an explicitly typed result.

## TOPAL-REQ-TOTAL-001 — Explicit termination and failure

Functions shall be total by default. Non-success outcomes shall be represented
explicitly; productive infinite computation shall use declared generator or
external-suspension semantics.

## TOPAL-REQ-CONC-001 — Race and deadlock prevention

The language shall reject safe programs whose declared resources, tasks, and
protocols cannot establish freedom from data races and internal deadlocks.

## TOPAL-REQ-DETERMINISM-001 — Scheduling independence

All executions permitted for a program shall produce the same semantic result,
apart from observations explicitly declared as permitted nondeterminism.

## TOPAL-REQ-EFFECT-001 — Observable-effect accounting

Observable interactions and affected resource identities shall be represented
in contracts sufficiently to validate ordering, independence, and containment.

## TOPAL-REQ-RESOURCE-001 — Resource and memory safety

Safe code shall access storage only through valid layouts, locations, address
ranges, lifetimes, and access capabilities, with ordering consistent with the
declared hardware and memory semantics.

## TOPAL-REQ-GENERIC-001 — Preserved generic meaning

Exported generic functions shall retain the type relationships, capability
evidence, effects, and other contracts required for an importing tool to
instantiate them without source access or semantic weakening.

## TOPAL-REQ-SERIAL-001 — Canonical native interchange

Topal's native serialization shall define versioned type descriptions,
canonical encodings where required, streaming behavior, validation, and
deterministic rejection of malformed input.

## TOPAL-REQ-TOOLS-001 — Tool conformance

The compiler, interpreter, linter, and other language tools shall implement the
applicable formal specification rules and shall identify unsupported language
revisions rather than silently changing meaning.

## TOPAL-REQ-INTEROP-001 — Execution interoperability

For the same valid program, inputs, and declared external observations, the
interpreter and compiled result shall have equivalent semantic behavior.

## TOPAL-REQ-TRACE-001 — Verifiable traceability

Normative rules and functional tests shall reference stable specification IDs;
specification rules shall trace to the design goals and requirements they
realize.

## TOPAL-REQ-SHARED-001 — Reusable toolchain layers

Language tools shall consume reusable source, lossless syntax, semantic, and
diagnostic layers rather than derive incompatible private representations.
Shared source and syntax data shall retain stable byte ranges, trivia, malformed
input, and incomplete input needed by batch tools, editor services, custom lint
rules, and static debugging. Runtime tools shall correlate execution decisions
with the same stable source identities without making application semantics
depend on observation.

## TOPAL-REQ-BEST-PRACTICE-001 — Shared programming guidance

The repository shall maintain a versioned best-practice database from which
human guidance, agent decision information, and optional lint rules are
traceably derived. Entries shall have stable owned identities, explicit status,
classification, applicability, defaults, tags, provenance, and license.
Generated projections shall remain version controlled and shall be verified
against their authoritative inputs.

## TOPAL-REQ-LINT-001 — Contained configurable linting

The Topal linter shall consume shared versioned syntax and semantic views,
produce diagnostics compatible with the interpreter and compiler, support
configuration and scoped suppression by stable best-practice identity, and
apply only explicitly selected safe rectifications. External databases and
library-supplied rules shall be supported without granting ambient authority or
automatic execution merely because a package is installed.

## TOPAL-REQ-TRANSFER-001 — Protocol-governed external interaction

External and inter-component data transfer shall use capability-authorized
endpoints whose typed operations, completions, failures, effects, lifetime, and
legal protocol transitions remain explicit. Local calls, messages, stores,
networks, and devices shall share this foundation without erasing their
distinct ordering, atomicity, reliability, or addressing semantics.

## TOPAL-REQ-DATA-VIEW-001 — Safe layered data access

Messages, packets, frames, sequences, and addressed regions shall retain their
semantic boundaries and support recursively nested representations. Validated
views over shared data shall permit bounded-copy inspection and transfer while
tracking ownership, span dependencies, mutation invalidation, resource limits,
and device-memory obligations.

## TOPAL-REQ-STORE-001 — Explicit store guarantees

File, relational, graph, document, key-value, and object stores shall share
identity, authority, transaction, snapshot, change, consistency, durability,
replication, and failure concepts where applicable without being forced through
one query or byte-stream interface.

## TOPAL-REQ-TRANSPORT-BINDING-001 — Replaceable faithful bindings

An application service may be realized over multiple local, network, or device
transports. A binding shall preserve the service protocol or reject unmet
requirements, and shall expose transport-specific correctness properties such
as addressing, scope, transaction boundaries, retry safety, security,
completion ordering, and resource constraints.

## TOPAL-REQ-CONTRACT-001 — Relational contracts

Revisioned function and state contracts shall express pure total preconditions,
postconditions, effect bounds, implementation requirements, and invariants with
unambiguous scope. Calls and public transitions shall prove their obligations;
protected safety obligations shall not be discharged by unverified trust.

## TOPAL-REQ-EVIDENCE-001 — Typed evidence provenance

Semantic and implementation evidence shall retain property, subject, static
parameters, status, producer, assumptions, language revision, and optional
architecture-model identity. Ordinary or generated source shall not mint proof
status, and implementation evidence shall not change semantic results.

## TOPAL-REQ-RESOURCE-BOUND-001 — Composable portable bounds

The language shall represent and conservatively compose portable work, span,
allocation, peak-live, retained, stack, queue, transfer, code-size, and progress
requirements. Unknown evidence shall fail a hard requirement, while an explicit
preference may use a semantically correct fallback.

## TOPAL-REQ-EXCLUSIVE-001 — Safe reuse and scoped allocation

The checker shall infer invocation-local exclusivity and enforce explicit
consumption without exposing mutable references. Named allocation regions shall
prevent dependent values from escaping except through checked move, promotion,
copy, or independence evidence and shall clean up deterministically.

## TOPAL-REQ-CONC-SYNTH-001 — Synthesized concurrency mechanisms

The language shall expose bounded interaction policies and immutable published
snapshots while keeping atomics, locks, fences, memory orders, and reclamation
mechanisms out of portable source. Only a verified selected implementation may
publish nonblocking progress evidence.

## TOPAL-REQ-TRANSACTION-001 — Structured atomic state

Transaction domains shall isolate candidate state, publish one complete commit
or none, expose conflict and abort outcomes, define cancellation winners and
same-domain nesting, and reject un-staged observable effects. Cross-domain
atomicity and durability shall require exact checked provider evidence.

## TOPAL-REQ-TIME-001 — Explicit temporal observations

Clock identity, instants, absolute deadlines, periodic release, lateness, and
timeout races shall be explicit semantic values and trace observations.
Relative timeouts shall create and propagate one deadline. Physical timing
claims shall fail closed without approved implementation evidence.

## TOPAL-REQ-STATIC-FLOW-001 — Checked static-rate flow

Closed static-rate flow graphs shall have checked balance and causality,
deterministically derived schedules and finite buffers, and identical logical
traces under interpreted and optimized execution.

## TOPAL-REQ-EFFECT-HANDLER-001 — Typed lexical handling

Effect protocols shall resolve handlers lexically, require complete typed
implementations, subtract handled effects, and use affine one-shot resumptions
with deterministic cleanup. Multiple resumption shall require verified safety
evidence.

## TOPAL-REQ-SPECIALIZE-001 — Verifiable specialization

A hard specialization requirement shall be satisfied only by compiler-owned or
checked-artifact code-shape evidence. Implementation plans shall be typed,
reproducible, read-only outside the compiler, and unable to change program
meaning.

## TOPAL-REQ-LAYOUT-COMPOSE-001 — Compositional representation

Semantic shapes and multidimensional, blocked, component-organized, sparse,
view, and conversion layouts shall have checked coverage, arithmetic,
lifetimes, access, and canonicalization. Foreign layout descriptions shall
receive no unchecked ABI or address authority.

## TOPAL-REQ-INFOFLOW-001 — Confidentiality and integrity flow

Information policies shall verify their label lattice, propagate explicit and
implicit control flow through values, effects, and messages, and permit
declassification or endorsement only with exact unforgeable scoped authority.

## TOPAL-REQ-ARCH-EVIDENCE-001 — Opaque architecture-evidence seam

Hard target-dependent requirements shall consume typed provider evidence
without exposing target facts to ordinary semantic computation. Architecture,
scheduler, device, fault-domain, and foreign-ABI schemas remain separately
deferred and shall not be inferred from this seam.

## TOPAL-REQ-COMPILER-001 — Correct native compilation

At its unoptimized reference level, the compiler shall preserve the observable
semantics of every accepted source program and shall reject every feature
outside its explicitly versioned implementation subset. Optional optimization
shall not repair, weaken, or change source meaning.

## TOPAL-REQ-NATIVE-PLATFORM-001 — Freestanding platform boundary

A native Topal executable shall depend only on the interfaces explicitly
selected for its target. It shall not implicitly require a C or C++ standard
library, foreign-language runtime, startup object, or dynamic loader. Direct
operating-system mechanisms shall be isolated in a versioned platform layer
with typed inputs, results, failures, effects, and target identity.

## TOPAL-REQ-NATIVE-ABI-001 — Target-qualified ABI lowering

Every machine-code artifact shall record its target triple, data layout,
object format, CPU baseline, platform ABI, and Topal private-ABI revision.
Target-specific calling and layout decisions shall be derived only from a
qualified target implementation. A foreign boundary shall use an explicit
adapter and shall not expose a private Topal aggregate representation.

## TOPAL-REQ-NATIVE-ARTIFACT-001 — Validated library code-generation metadata

A compiled library shall retain the semantic GEIR and public interface
information needed for source-independent checking and instantiation, plus
target-qualified native slices and reproducible dependency, toolchain,
provenance, and debug mappings. LLVM IR or bitcode shall not become the stable
library compatibility boundary.

## TOPAL-REQ-NATIVE-DEBUG-001 — Source-level native debugging

An unoptimized native artifact shall retain sufficient source, scope, function,
parameter, local-value, type, and unwind information for deterministic GDB
breakpoints, stepping, backtraces, and value inspection. Optimized artifacts
shall state their reduced variable-availability guarantees.

## TOPAL-REQ-LLVM-001 — Qualified LLVM use

The compiler shall version-check LLVM, verify every generated module, delegate
target-independent and target-dependent transformations only where LLVM's
semantics match Topal's proved facts, and record the disposition of applicable
LLVM facilities. A deferred or deliberately unused facility shall have a
documented reason.
