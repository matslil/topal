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
