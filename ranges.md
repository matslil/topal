# Range and selection model

This document records the provisional range model for Topal. Ranges build on
predicates and constraints rather than introducing an unrelated container or
iteration concept. Collection selections produce ordinary collection values;
the compiler may retain stronger provenance facts when they are useful.

## Ranges are specialized predicates

A predicate decides whether a value belongs to a set. A range is a predicate
whose accepted values form one convex interval in a chosen total order:

```text
a <= b <= c
range accepts a
range accepts c
----------------
range accepts b
```

Conceptually:

```topal
Predicate T
Range T is Convex Predicate T
```

This applies to domains such as integers, rationals, decimals, strings,
indexes, and ordered map keys. The order is part of the range's meaning. A
partially ordered domain does not provide an unambiguous interval, and a map key
range must use the same ordering as the map.

A range supports ordinary predicate operations:

```topal
value in interval
interval contains value
```

It does not imply that its members can be enumerated. There is no natural way
to visit every rational or string in an interval. Enumeration belongs to a
separate progression or generator that additionally supplies direction, a step,
or successor evidence.

## Bounds and inclusivity

Comparison predicates express whether each finite bound is included:

```topal
range ( >= 0 and <= 10 ) # closed at both ends
range ( > 0 and < 10 )   # open at both ends
range ( >= 0 and < 10 )  # closed below, open above
range ( > 0 and <= 10 )  # open below, closed above
```

The concise inclusive syntax is sugar for the corresponding predicate:

```topal
0 .. 10
range ( >= 0 and <= 10 )
```

The internal representation may expose included and excluded bounds for
inspection and efficient implementation, but separate inclusivity flags are
not fundamental to source construction. Open bounds must not be normalized by
adding or subtracting one: that fails at finite extremes and does not apply to
dense domains such as `Rational` and `Decimal`.

## Unbounded ranges

One-sided ranges use predicate sections directly, so the remaining finite bound
states its inclusivity without placeholder syntax:

```topal
range ( >= 0 )  # includes 0 and has no maximum
range ( > 0 )   # excludes 0 and has no maximum
range ( <= 10 ) # has no minimum and includes 10
range ( < 10 )  # has no minimum and excludes 10
```

When the expected kind is already `Range T`, the explicit `range` construction
may be inferred. Bare forms such as `0 ..` and `.. 10` are avoided because they
look like incomplete binary applications and do not say whether the finite
endpoint is included.

A range with neither bound names its ordered domain:

```topal
range Int
range String
```

It accepts every value in that domain. A singleton accepts one value, while
contradictory bounds describe the empty range; neither requires a special
failure case.

## Constructing ranges from predicates

Every range is usable as a predicate. The reverse conversion is available only
when the predicate is known to be convex in the relevant order:

```topal
Nonnegative is range ( >= 0 )
Small is range ( >= 0 and <= 10 )
```

Simple comparison sections let the compiler establish this statically. `even`
and `< 0 or > 10` do not describe single ranges. An opaque or dynamically
supplied predicate requires proof or checked construction before it can be
treated as a range. Testing finitely many values cannot establish that an
arbitrary predicate is convex over an infinite domain.

Range intersection remains a range. A union is a range only when its accepted
regions join without a gap in the domain's order. Complementation may produce
zero, one, or two ranges, so these operations retain their general predicate
result unless stronger evidence is available.

## Ranges as constraints

Because ranges are predicates, they participate in Topal's ordinary constraint
and refined-type model:

```topal
ByteRange is 0 .. 255
ByteValue is ByteRange Nat
```

Successful validation produces reusable membership evidence. Decision-table
branches can establish bounds whether the source was written as a range or as
comparison predicates:

```topal
index
  >= 0 and < collection entry-count then collection get index
  otherwise return error OutOfBounds
```

## Selection and slices

A collection can be selected using a predicate over its values or indexed
entries. Selection preserves source order and occurrences:

```topal
values select (> 0)
values select-index ( >= start and < end )
```

Value selection and position selection are distinct when values repeat. A
predicate over an indexed entry can combine both:

```topal
values select
  fn ( entry : IndexedEntry T ) -> Boolean
    entry index even and entry value > 0
```

In all cases, the observable result is an ordinary collection of the same
semantic kind. Topal code cannot tell whether the compiler represents it as a
copy, a shared view, a tree, a fused traversal, or another equivalent form.

A slice is an inferred constraint describing how a result relates to its
source, rather than a separate observable collection type. It need not be
restricted to consecutive entries. Conceptually:

```text
SliceOf source
SelectionOf source predicate
RangeSelectionOf source index-range
```

`SliceOf` means a sequence subset: it preserves source order and multiplicity,
which mathematical set inclusion would lose. `SelectionOf` additionally records
the predicate used to choose entries. `RangeSelectionOf` is the stronger case
in which a convex index predicate chooses a consecutive region. Each implies
the relation above it, while the visible value remains `List T`, `String`, or
the corresponding source collection type.

These facts are most naturally produced by construction. Recovering whether an
arbitrary existing list is a subsequence of another can be costly and ambiguous
with repeated equal values. The compiler need not reconstruct provenance after
it has been discarded.

## Boundaries and text units

For a collection containing `count` entries, element indexes and selection
boundaries have related but different valid ranges:

```text
element indexes       >= 0 and < count
selection boundaries  >= 0 and <= count
```

Selecting indexes `>= start and < end` represents the consecutive region
between the two boundaries. It permits an empty selection when `start = end`, a
selection ending after the last entry, and adjacent regions that meet without
overlap.

String selection must also identify its unit. Scalar, character, and encoded
byte positions are not interchangeable, and a character selection cannot split
an extended grapheme cluster. The result is still an ordinary `String`; retained
boundary and source evidence permits safe reuse and optimization.

## Observable meaning and optimization

Provenance constraints give the compiler useful facts without exposing storage
decisions. It may share storage, copy a region, fuse a following traversal,
avoid an intermediate collection, derive a range selection's result count, or
remove repeated bounds checks using retained evidence.

These choices cannot affect equality, collection contents, ordering, or any
other source-level observation. Programmers may explicitly request provenance
evidence for specialized algorithms, but ordinary selection does not require
them to name or manage it.

The stable design principles are:

- A range is a convex predicate in a chosen total order.
- Predicate operators express inclusive, exclusive, and absent bounds.
- A range describes membership, not enumeration.
- Every range is a constraint; only provably convex predicates are ranges.
- Selection returns an ordinary collection and preserves source order.
- Slice and selection relationships are inferred provenance constraints, not
  observable storage representations.
