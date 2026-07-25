# Initial capability vocabulary

Capabilities promise semantic operations and laws provided by types,
functions, or other static objects. They do not constrain the permitted values
of a type and do not expose an implementation representation.

This document defines the initial standard vocabulary. Capability matching and
its distinction from value constraints are described in
[generic abstraction and semantic capabilities](abstractions.md).

## Value comparison

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
  compare : Value, Value -> PartialOrdering
```

`PartialOrdering` has the alternatives `Less`, `Equal`, `Greater`, and
`Incomparable`. The equality result of `compare` agrees with `Equality`.

### TotalOrder

```text
TotalOrder Value
  PartialOrder Value
  compare never produces Incomparable
```

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

These laws permit parallel reduction and other transformations whose evaluation
order would otherwise be observable. They are never inferred from an
function's name.

## Standard composites

Common conjunctions receive names when the combined meaning is independently
useful. Sorting uses the homogeneous container-construction matcher:

```topal
Sortable is
  ( Indexed and Replaceable )
  Container ( TotalOrder Value )

sort is fn ( values : C : Sortable ) -> C
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
the final multi-component matcher syntax remains to be selected.

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
