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
