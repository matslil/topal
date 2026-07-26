# Remaining fundamental design decisions

The semantic foundations define generic evidence, effects, execution, structured
tasks, scoped resources, typed error vocabularies, and foreign boundaries.
Several choices deliberately remain open because they affect grammar,
ergonomics, or the trusted implementation boundary rather than following
uniquely from Topal's existing principles.

## Recently settled foundations

The following questions from the initial audit are no longer open:

- [Constraints and capabilities](abstractions.md) are distinct kinds.
  Constraints retain a base type and limit its values through a predicate;
  capabilities make static interface and law promises.
- Constraint construction is object-first, as in
  `Integer constraint { value } ...`. The inferred anonymous function is the
  predicate.
- Chained classification proceeds from left to right. A function header such
  as `values : C : Sortable` first classifies `values` as `C` and then requires
  `C` to satisfy `Sortable`.
- Function headers perform static type matching. Construction syntax can bind
  components such as `X` and `Y` from `Tuple ( X, Y )`, and the function body
  is the implicit successful branch of the header match.
- Returning a captured complete type such as `C` preserves the precise
  relationship between the input and result, including nominal identity,
  constraints, static sizes, and other parameters.
- Overload resolution is ordered like pattern matching. Declarations under one
  name in one namespace are tested in source order and the first applicable
  input header wins. Capability strength, conversion quality, and the expected
  result type do not reorder them. Optional diagnostics may identify surprising
  overlap or shadowing.
- Imported scopes remain qualified and do not merge overload sets. A qualified
  call selects its namespace before searching that namespace's declarations;
  scope aliases preserve the same order. Explicit cross-namespace overload-set
  composition has no current surface syntax.
- The [initial capability vocabulary](capabilities.md) now defines comparison,
  collection observation and construction, keyed association, combination, and
  function-law capabilities.
- Possible compiler-generated tests, symbolic proof tables, capability-law
  verification, and task/protocol proofs are recorded as
  [future work](FUTURE.md), not current language guarantees.

## Surface grammar

The grammar must select compatible spellings for:

- capability declarations, implementations, and explicitly selected evidence;
- type-construction patterns beyond homogeneous `Container Value`, including
  constructions such as `Map ( Key, Value )`;
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
unambiguous. Function headers already bind generic type components through
static matching rather than through a separate generic-parameter list.

## Type-pattern applicability

Construction matching and chained classification establish the core generic
model. Source order now selects the first applicable overload, so neither
concrete types nor stronger capability patterns receive an implicit specificity
rank. Matching an individual declaration still needs precise rules for:

- repeated names which require definitionally equal matched objects;
- partial type constructions such as matching `Array N Value` as
  `Container Value`;
- opaque or nominal types which publish capabilities without publishing their
  construction;
- open record patterns which retain additional fields; and
- when evidence forgetting and lossless conversion make one header applicable.

The result type does not participate in header matching. Matching remains static
and must not introduce runtime reflection or dispatch unless an interface
explicitly requests it. The compiler may optionally diagnose overlapping,
conversion-preempted, or provably shadowed declarations, but declaration order
remains authoritative.

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
comparison, collection, and function-law names. Formatting, parsing, checked
construction, and other library-specific capabilities still need vocabularies
in their respective designs.

The multi-component matcher for `Keyed Container Key Value` and similar
capabilities also needs a final spelling. It must bind `Key` and `Value` from
the actual container construction rather than equating unrelated constructions
such as `Map ( Key, Value )` and `Tuple ( Key, Value )`.

## Law evidence before automated proof

The [future verification design](FUTURE.md) describes symbolic proof tables,
finite exhaustive verification, induction, and independently checked proof
certificates. Before that system exists, the language still needs a conservative
rule for function-law evidence used by optimizations.

The main choices are:

- initially allow law evidence only for compiler-defined operations;
- permit explicitly trusted user or foreign claims; or
- accept user proof terms in a smaller initial proof language.

Passing sampled tests cannot establish `Associative`, `Commutative`, `Identity`,
or `Idempotent`, because those capabilities may authorize reordering and
parallel execution. Any trusted alternative must remain visible in source,
static introspection, compiled metadata, and diagnostics.

## Effect annotations and handlers

Effects are inferred for ordinary implementations. An `Interface` contains
function and generator interaction shapes rather than the inferred effects of
one implementation. Applying it to a concrete context, packaged value, task, or
endpoint constructs implementation evidence whose compiled contract includes
the inferred effects. Callers retain that evidence whenever the selected
implementation is known.

It remains to decide the surface spelling for optional effect upper bounds and
for accepting a dynamically selected implementation under a common effect
bound. Foreign implementations must declare effects which the compiler cannot
infer.

The initial design handles effects through application composition, tasks,
protocols, constructed contexts, and foreign adapters. A future general handler
construct should be added only if concrete use cases cannot be expressed
cleanly through those boundaries. If added, its treatment of continuation
linearity, task state, and effect resource identities needs a separate design.

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
