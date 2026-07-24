# Remaining fundamental design decisions

The semantic foundations define generic evidence, effects, execution, structured
tasks, scoped resources, typed error vocabularies, and foreign boundaries.
Several choices deliberately remain open because they affect grammar,
ergonomics, or the trusted implementation boundary rather than following
uniquely from Topal's existing principles.

## Surface grammar

The grammar must select compatible spellings for:

- capability declarations, implementations, and explicitly selected evidence;
- type-construction patterns beyond homogeneous `Container Value`;
- existential package opening;
- explicit effect bounds and effect-row parameters;
- mutually recursive declaration groups and decreasing measures;
- immutable record reconstruction and qualified task-field replacement;
- task scopes, child construction, waiting, cancellation, and selection;
- explicit resource scopes and ownership transfer from them;
- public error vocabulary bounds; and
- foreign symbols, ABIs, and trusted declarations.

These should be designed together so that indentation, recursive
classification, prefix application, and the zero-to-two operand rule remain
unambiguous. Algorithm headers already bind generic type components through
static matching rather than through a separate generic-parameter list.

## Capability organization

Capability satisfaction is coherent within a compiled context, but the module
rules still need to choose where an implementation may be declared. The main
alternatives are:

- only beside the capability or the satisfying type;
- in any module, with explicit import and conflict rejection; or
- as an ordinary named evidence value, with only one explicitly selected as
  the implicit default.

The third is the most compositional, while the first gives the strongest
protection against distant conflicts. This decision also determines how
libraries publish derived equality, ordering, parsing, and collection evidence.

The [initial capability vocabulary](capabilities.md) selects the fundamental
comparison, collection, and algorithm-law names. Formatting, parsing, checked
construction, and other library-specific capabilities still need vocabularies
in their respective designs.

## Proof checking

Topal must define which user claims require machine-checkable proof and which
may enter as trusted evidence. Termination measures and structural derivations
are intended to be checked. General algebraic laws such as associativity are
not decidable for arbitrary algorithms.

Choices include restricting implicit law evidence to compiler-derived
operations, accepting proof terms in a selected logic, or allowing visibly
trusted user assertions. Whichever rule is chosen must preserve the distinction
between verified evidence and a trusted boundary claim in introspection and
compiled metadata.

## Effect annotations and handlers

Private effects are inferred and public compiled contracts are stable, but it
remains to decide whether public source declarations must spell their complete
effect upper bound or may rely on an interface-generation step.

The initial design handles effects through application composition, tasks,
protocols, environments, and foreign adapters. A future general handler
construct should be added only if concrete use cases cannot be expressed
cleanly through those boundaries. If added, its treatment of continuation
linearity, task state, and effect resource identities needs a separate design.

## Error vocabulary precision

A public result can constrain its errors. The remaining choice is whether the
normal unit of constraint is:

- an entire error domain;
- selected codes within a domain; or
- named sets which may combine domains and codes.

Named sets are the most flexible but add interface objects and versioning rules.
Domain-only contracts are simpler but may be too broad for recovery-oriented
APIs. This should be tested against several file, parsing, network, and
application-boundary designs before selecting syntax.

## Cancellation and external time

Cancellation is cooperative and scoped. Protocols still need precise defaults
for guaranteed, best-effort, and unsupported remote cancellation, including
what completion evidence means after a cancellation race.

Clock-provided timeout alternatives avoid ambient time. The standard clock and
timer protocols, representation of deadlines, and deterministic testing rules
remain to be designed. They should reuse quantities and units rather than
introduce untyped duration numbers.

## Resource transfer and non-owning capabilities

Explicit resource scopes establish deterministic cleanup. The type-level form
of a permitted ownership transfer from one scope to another remains open.
Possible designs include a linear transfer capability, returning a new scoped
owner object, or a specialized acquire/commit operation.

The public operations for non-owning back references also need names and a
precise interaction with task and endpoint capabilities. Their semantics must
not expose reference counts or collection timing.

## Foreign ABI catalog

The portable model states what every foreign declaration must describe.
Individual language features still need to define supported ABI families,
primitive ABI layouts, callback adapters, error conventions, and platform
linkage metadata. Those catalogs are platform specifications rather than part
of the bootstrap language.

## Performance contracts

Core collections deliberately avoid promising a representation. It remains to
decide whether generic capabilities may state checked complexity or storage
guarantees, such as random access, contiguous storage, bounded allocation, or
real-time execution.

These guarantees are useful for systems programming but require a vocabulary
whose evidence survives optimization and foreign boundaries. They should not be
inferred merely from names such as `Array`, `Map`, or `List`.
