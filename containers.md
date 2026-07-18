# Containers and algebraic data

Topal's container model is built from the same small set of recursively
composable concepts as the rest of the language. Products, sums, recursion,
constraints, and finite indexes provide the semantic foundation. Collection
capabilities provide shared algorithms without forcing every value into one
universal container hierarchy.

## Algebraic foundation

Products combine values that are present together. Sums select one of several
possible values:

```text
Tuple ( A, B, C )   = A * B * C
Variant ( A, B, C ) = A + B + C
Option T            = Unit + T
Result T E          = T + E
```

Records are labeled products and tagged unions are labeled sums. They do not
require separate composition mechanisms. A list additionally uses recursion:

```text
List T = Empty | Entry ( T, List T )
```

This is a semantic construction, not a required storage representation. The
compiler may represent a list as a tree, flat buffer, shared slice, or another
equivalent form when no source-level observation changes.

A tuple is fundamentally a product rather than a homogeneous collection. A
homogeneous tuple may support sequence algorithms, but a heterogeneous tuple
cannot generally be mapped, folded, or selected using one entry type.

## Sequences and arrays

A sequence preserves entry order and multiplicity. Lists and arrays share
sequence operations such as traversal, selection, and indexed access where the
required index evidence is available.

A fixed-size array has the conceptual form:

```text
Array N T = Index N -> T
```

`Index N` contains evidence that an index is within the array's bounds, making
access total. An array can also be understood as a sequence constrained to have
`N` entries:

```text
Array N T = Sequence T where entry-count = N
```

These views are compatible: the first emphasizes total indexing and the second
emphasizes reuse of sequence algorithms. Neither requires contiguous storage or
constant-time access as an observable language guarantee. Hardware layouts and
encoded arrays can add explicit representation constraints when those properties
matter at a boundary.

## Sets, maps, and bags

A set is a finite unordered collection whose members are unique. A map is a set
of key/value products constrained so that a key occurs at most once:

```text
Map K V = Set ( K, V ) where each K occurs at most once
```

Consequently, map entries use the ordinary product model, and generic set
operations can be reused when their laws remain meaningful. Lookup and other
key-oriented algorithms are derived from the uniqueness constraint.

A bag, or multiset, retains multiplicity without introducing order. It can be
constructed from a map of values to positive occurrence counts:

```text
Bag T = Map T PositiveNat
```

Removing an entry decrements its count and removes the association when the
count would reach zero. This keeps one canonical representation for an absent
entry.

`Set T` can alternatively be viewed as `Map T Unit`, but this is an equivalence
rather than a second foundational definition. The primary construction remains
a map as a constrained set of associations, avoiding circular definitions.

## Collection capabilities

Shared behavior is expressed through small capabilities rather than nominal
inheritance from a single `Container` type. The exact capability names remain
provisional, but the useful distinctions are:

```text
Foldable Entry
  fold
  entry-count
  empty?

Sequence Entry
  get Index
  select-index
  preserves order and multiplicity

Unique Member
  contains
  insert
  remove

Associative Key Value
  get Key
  keys
  values
  entries
```

Lists and arrays are sequences. Sets provide uniqueness, bags provide
multiplicity without order, and maps provide key association with unique keys.
All finite collections can support folding and explicit entry counting where
their entry type is uniform.

Ordering is independent of key association. An ordered map preserves a declared
key or insertion order by satisfying both associative and ordered traversal
capabilities; an ordinary map does not acquire an arbitrary observable order.

## Derived structures

Many familiar structures are compositions or interfaces, not additional core
container kinds:

- Stacks, queues, and deques are sequence operation disciplines.
- Priority queues are collections paired with an ordering and priority-removal
  algorithms.
- Matrices are arrays indexed by products of finite indexes.
- Graphs can be maps from vertices to sets of vertices or edges.
- Trees are ordinary recursive products and variants.
- Strings remain semantic text values rather than arrays of bytes or characters.
- Ranges remain convex predicates and constraints rather than containers.
- Streams and generators are productive computations rather than immutable
  container values.

The core named collection families are therefore `List`, `Array`, `Set`, `Map`,
and `Bag`. `Tuple` and `Variant` are the more fundamental product and sum
constructions from which collections and other user-defined data can be built.

## Design principles

- Products, sums, recursion, constraints, and finite indexes define structure.
- More specific collections retain the laws of the simpler structures from
  which they are constructed.
- Capabilities share algorithms without erasing semantic differences in order,
  uniqueness, multiplicity, or indexing.
- Storage layout and performance strategies remain compiler choices unless a
  program explicitly requests representation constraints.
- Specialized structures are composed from core collections and algorithms
  instead of multiplying primitive container kinds.
