# Initial capability vocabulary

Capabilities promise semantic operations and laws provided by types,
functions, or other static objects. They do not constrain the permitted values
of a type and do not expose an implementation representation.

This document defines the initial standard vocabulary. Capability matching and
its distinction from value constraints are described in
[generic abstraction and semantic capabilities](abstractions.md).

## Non-owning access

### Weak

```text
Weak Value
  access : Weak Value -> Result ( Value, WeakErrorCode )
```

`Weak Value` constructs a non-owning reference associated with `Value`.
Creating, copying, retaining, or destroying the weak reference does not extend
the target's lifetime. Access atomically attempts to retain an ordinary
`Value`; failure returns the language-defined `weak-unavailable` error.

Applying a weak value to a block retains the target once, binds the ordinary
value for the complete block, and releases it afterward. The retained value may
escape through an ordinary move, but the compiler continues to reject possible
owning cycles involving external resources. Task endpoints have their own
lifetime and `task-terminated` semantics and do not use `Weak`.

## Value comparison

Comparison results are ordinary enum values:

```topal
Comparison is Enum (
  Less,
  Equal,
  Greater
)

PartialComparison is Enum (
  Less,
  Equal,
  Greater,
  Incomparable
)
```

`Comparison` records the result of a total-order comparison.
`PartialComparison` additionally records that neither operand precedes the
other. There is one canonical lossless conversion from `Comparison` to
`PartialComparison`; the reverse conversion is checked because
`Incomparable` has no total-order result.

### Equality

```text
Equality Value
  equal : Value, Value -> Boolean
```

`Equality` promises reflexivity, symmetry, and transitivity and enables `=` and
`!=`. The language derives it for tuples, records, variants, unions, and finite
recursive values when every observed component provides `Equality`.

Functions, continuations, task capabilities, context-provided endpoints, external
resources, and opaque values do not receive equality automatically. A type may
separately expose a stable identity when identity comparison belongs to its
public semantics.

### PartialOrder

```text
PartialOrder Value
  Equality Value
  compare : Value, Value -> PartialComparison
```

The `Equal` result of `compare` agrees with `Equality`. `Incomparable` means
that neither value precedes the other; it is distinct from inequality.

### TotalOrder

```text
TotalOrder Value
  PartialOrder Value
  compare : Value, Value -> Comparison
```

`TotalOrder` refines the comparison result type, making the absence of
`Incomparable` statically visible. Its `Comparison` result converts losslessly
to `PartialComparison` wherever only `PartialOrder` evidence is required.
`TotalOrder` supports ranges, sorting, ordered collections, minimum, and
maximum. Tuple ordering is derived lexicographically when every component
provides `TotalOrder`. Records require an explicit ordering because field
declaration order need not have domain meaning.

## Collection observation

### Counted

```text
Counted Container
  entry-count : Container -> Nat
  empty? : Container -> Boolean
```

`Counted` promises a finite exact entry count. It says nothing about entry type,
order, or access complexity.

### Foldable

```text
Foldable Container Value
  fold
```

`Foldable` promises finite elimination over homogeneous entries. It does not
imply a stable order; folding an unordered collection therefore requires an
operation whose laws make order unobservable or an explicitly selected order.

`Foldable` and `Counted` remain separate because elimination and exact counting
are distinct promises, even though ordinary finite containers normally provide
both.

### Sequence

```text
Sequence Container Value
  Foldable Container Value
  traversal preserves order and multiplicity
```

`Sequence` promises a stable entry order and retained multiplicity. It does not
promise positional access or constant-time operations. Lists, arrays, and
strings provide `Sequence`; sets and ordinary maps do not.

### Indexed

```text
Indexed Container Value
  Sequence Container Value
  Index : Type
  get : Container, Index -> Value
```

`Indexed` promises total positional access using the associated `Index`.
Unchecked numeric access remains a separate operation returning `Option` or
`Result`.

The initial vocabulary deliberately uses `Indexed` rather than `RandomAccess`.
The latter normally promises complexity, while `Indexed` makes only a semantic
access guarantee.

### Membership

```text
Membership Container Value
  contains : Container, Value -> Boolean
```

`Membership` promises a membership test compatible with `Equality Value`.
It does not expose whether the implementation uses hashing, ordering, or a
linear scan.

## Collection construction

### Empty

```text
Empty Container
  empty : Container
```

`Empty` promises a canonical empty value. A fixed nonzero array or a non-empty
constrained collection does not provide it.

### Singleton

```text
Singleton Container Value
  one : Value -> Container
```

`Singleton` promises construction from exactly one entry.

### Collectible

```text
Collectible Container Value
  collect : Traversal Value -> Container
```

`Collectible` promises that every finite traversal of matching values can
construct the container. A type whose size, uniqueness, or other invariant can
reject an arbitrary traversal does not provide this capability; it exposes an
ordinary checked construction operation instead.

### Replaceable

```text
Replaceable Container Value
  Index : Type
  replace : Container, ( Index, Value ) -> Container
```

`Replaceable` promises immutable positional replacement while preserving the
complete container type. Its `Index` agrees with `Indexed` when both
capabilities are present. The compiler may implement replacement in place when
the old value is no longer observable.

## Keyed and unique collections

### Keyed

```text
Keyed Container Key Value
  get : Container, Key -> Option Value
  keys
  values
  entries
```

`Keyed` promises association between keys and values. A map matcher obtains
`Key` and `Value` from the actual `Map ( Key, Value )` construction; it does not
reinterpret a map as a unary homogeneous container.

### Associable

```text
Associable Container Key Value
  Keyed Container Key Value
  associate : Container, ( Key, Value ) -> Container
  remove-key : Container, Key -> Container
```

`Associable` promises immutable insertion or replacement by key and removal by
key.

### Unique

```text
Unique Container Value
  Membership Container Value
  insert : Container, Value -> Container
  remove : Container, Value -> Container
  inserting an existing value does not add another occurrence
```

`Unique` states a semantic law. It does not imply a hash-table representation.

## Collection combination

### Concatenable

```text
Concatenable Container
  concatenate : Container, Container -> Container
```

`Concatenable` promises ordered combination and is normally provided by
sequences. Unordered sets and maps use their own algebra and collision policies
rather than pretending concatenation has one meaning.

## Function laws

Law capabilities apply to function objects rather than to every value of an
operand type:

```text
Associative Operation
Commutative Operation
Identity Operation Value
Idempotent Operation
Decreases Function Measures
```

`Associative` promises:

```topal
( a operation b ) operation c = a operation ( b operation c )
```

`Commutative` promises:

```topal
a operation b = b operation a
```

`Identity` promises:

```topal
identity operation value = value
value operation identity = value
```

`Idempotent` promises:

```topal
value operation value = value
```

`Decreases` is termination evidence for a complete function. Its associated
`Measures` are one or more pure expressions over the function inputs:

```topal
search is fn (
  active : List Node,
  deferred : List Node
) -> Optional Node : Decreases ( active size + deferred size )
```

The compiler proves that every recursive call strictly reduces the complete
measure. Multiple expressions form a lexicographic product: a later component
is considered only when every earlier component remains equal. Each component
must inhabit a compiler-known well-founded order. Measures may use arithmetic,
field projections, collection sizes, and pure functions which the termination
checker can reason about.

`Decreases` classifies the complete function rather than any individual
parameter because it relates the inputs of one invocation to those of the next
recursive invocation. It follows the completed return type in a declaration.
The compiler infers equivalent termination evidence whenever its standard
analysis succeeds. Source-level evidence is needed only to guide a proof or to
state an opaque, interface, or higher-order contract whose implementation is
not available.

Unlike an optimization law, `Decreases` cannot fall back to
`trusted-unverified`: unresolved evidence would undermine Topal's totality
guarantee. A definition supplying or implementing the capability must prove
every recursive edge decreases its declared measure.

The algebraic laws above permit parallel reduction and other transformations
whose evaluation order would otherwise be observable. They are never inferred
from a function's name.

## Effect relationships

Effect and resource identities use capabilities to retain relationships which
ordinary value flow does not make visible:

```text
DependsOn Dependent Prerequisite

Independent Identities
Conflicts Identities
Aliases Identities
MayAlias Identities
```

`DependsOn` is binary and directional. `DependsOn A B` means that the relevant
execution of `A` requires the corresponding execution of `B` to complete first,
forming the dependency edge `B -> A`. The compiler may derive transitive
ordering without publishing a separate capability for every edge. A dependency
cycle with no suspension or protocol boundary is an invalid execution graph.

The other capabilities each take one static list of identities:

- `Independent` promises pairwise independence;
- `Conflicts` promises that every pair has potentially observable conflicting
  interactions;
- `Aliases` promises that every member denotes the same underlying resource;
  and
- `MayAlias` records that any pair may denote the same resource and therefore
  must not be treated as independent without stronger evidence.

List order and duplicate identities do not change these capabilities. Empty and
single-entry lists satisfy their pairwise requirements vacuously. `Aliases` is
positive identity evidence, whereas `MayAlias` preserves uncertainty. When the
compiler can prove neither exact aliasing nor independence, `MayAlias` is the
safe default. The compiler rejects relationship evidence whose laws are jointly
inconsistent for the same identities and interactions.

These capabilities may classify functions, callbacks, resources, effects, task
interactions, and other static identities. The compiler derives them from data
flow, shared resource parameters, effect declarations, and protocol order when
possible. Programmer declarations follow the ordinary verified and
trusted-unverified evidence rules. Adding an unnecessary `DependsOn` edge is
conservative; incorrectly claiming `Independent` can change observable
behavior.

## Standard composites

Common conjunctions receive names when the combined meaning is independently
useful. Sorting uses the homogeneous container-construction matcher:

```topal
Sortable is
  ( Indexed and Replaceable )
  Container ( TotalOrder Value )

sort is fn (
  values : ( C : Type : Sortable )
) -> C
  sorting-implementation values
```

The matcher obtains `Value` from the construction of the complete type `C`,
rather than inferring an unrelated type. Returning `C` preserves every static
size, constraint, and nominal identity of the input.

Other useful composites can be defined from the initial vocabulary:

```text
Searchable =
  Foldable Container ( Equality Value )

ReconstructibleSequence =
  ( Sequence and Replaceable ) Container Value

SetLike =
  ( Foldable and Membership and Unique )
  Container ( Equality Value )
```

These names do not introduce new primitive operations. A map-oriented composite
matches `Map ( Key, Value )` directly and combines `Keyed` and `Associable`;
its associated `Key` and `Value` components use the explicit grouped matcher
defined in [generic abstraction](abstractions.md#multi-component-capability-matching).

## Standard-library extensions

The following capabilities are useful but do not need to be part of the initial
language vocabulary:

- `Insertable` and `Removable`, whose operations may change size or violate a
  constraint;
- `SetAlgebra`, providing union, intersection, difference, and symmetric
  difference;
- `Mergeable`, whose map operation requires collision-policy or disjointness
  evidence; and
- formatting, parsing, and specialized construction capabilities introduced by
  their respective libraries.

They should be composed from the same capability and evidence model rather than
given separate dispatch mechanisms.

## Deliberate omissions

The initial vocabulary does not include:

- `Hashable`, because hashing is a replaceable implementation strategy;
- `Copy` or `Clone`, because source API does not expose ownership
  representation;
- `Iterator`, because folds and generators provide traversal;
- `Mutable`, `Send`, or `Sync`, because Topal uses immutable values, effects,
  and task isolation;
- universal formatting, string conversion, or object identity;
- `RandomAccess`, until complexity guarantees have a formal evidence model; or
- one broad `Numeric` capability, because individual operations and their laws
  compose more accurately.
