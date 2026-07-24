# Generic abstraction and semantic capabilities

Topal generic code uses the same first-class objects, constraints, and evidence
as ordinary code. It does not introduce a separate template language, class
hierarchy, or method-lookup system. A generic declaration classifies the objects
it abstracts over and states the evidence required to use them.

This document defines the semantic model. The exact surface spelling for
explicit generic parameters and capability declarations remains provisional.

## Generic parameters

A generic parameter is a statically available input to a declaration. Its
classification states its kind:

```text
T : Type
N : Nat
C : Constraint T
comparison : TotalOrder T
```

Types, static values, constraints, layouts, effects, protocols, and algorithms
can consequently all be generic parameters. A parameter may depend on an
earlier parameter, but not on a later one. This is the same ordered dependency
rule used by record fields.

The conceptual signature:

```text
get : forall A where A satisfies Sequence.
      A, Index A -> Element A
```

has three relevant static objects: the particular type `A`, evidence that `A`
satisfies `Sequence`, and the associated objects `Index A` and `Element A`
selected through that evidence. They do not become runtime operands merely
because they are explicit parts of the declaration.

Generic arguments are inferred when classifications of ordinary operands or an
expected result determine one answer. Code supplies them explicitly when they
cannot be inferred or when several valid objects remain. Inference never uses a
runtime value, an algorithm body, or the required output type to choose between
otherwise applicable overloads.

A generic declaration is checked once against its declared parameters and
evidence. Instantiation may specialize representation and execution, but it
does not defer ordinary type checking in the manner of textual templates.

## Capabilities are constraints with evidence

A capability states that objects of a particular kind support a semantic
operation or law. Satisfying a capability produces evidence which contains its
associated objects and algorithms:

```text
Sequence A
  Element : Type
  Index : Type
  get : A, Index -> Element
  entries : A -> Traversal Element
```

The capability is not a namespace owned by `A`. `get` remains an ordinary
overload whose applicability is established by the evidence. This preserves
Topal's independent algorithm composition and avoids introducing methods.

Capabilities may require other capabilities and may refine their associated
objects. For example, `TotalOrder T` requires `Equality T`, while a writable
layout requires the operations and access evidence of a readable addressed
layout plus permission to write.

Evidence can arise in three ways:

- the language derives it from a fundamental construction and the evidence of
  its components;
- a declaration explicitly supplies the required objects and algorithms; or
- a constraint check produces evidence for a particular value or refined type.

Merely having algorithms with suitable names is insufficient. Accidental
structural similarity must not silently assert laws such as associativity,
ordering, uniqueness, or losslessness. The compiler may derive structural
operations for products, sums, and finite recursion when every component
provides the required evidence.

There must be at most one implicit capability implementation for the same
capability and type in one compiled context. Alternative comparisons,
serializations, layouts, or reduction laws are ordinary explicit evidence
values passed to the operation which uses them.

## Associated objects

An associated object belongs to capability evidence rather than to a global
type namespace. It may have any static kind:

```text
Element A : Type
Index A : Type
effects algorithm : Set Effect
identity operation : lang Identity Algorithm
```

Selecting an associated object requires evidence identifying the capability
instance. The short conceptual spelling `Element A` is valid when exactly one
applicable instance is available. Otherwise the evidence must be named
explicitly.

Associated types may depend on static values and identities. `Index A`, for
example, retains the identity and bound of a concrete array type rather than
weakening to the index type of every array.

## Existential results

An operation may produce a value whose exact static parameter depends on a
runtime result:

```text
select : Array N T, ( T -> Boolean )
      -> exists M where M <= N. Array M T
```

The result packages the hidden parameter, its value, and the evidence relating
it to the visible parameters. Pattern matching may introduce the hidden name
and evidence in a nested scope. Code which does not need them can forget the
package to a weaker visible capability such as `Sequence T`.

Existential packaging is never an unchecked cast. Opening a package creates a
fresh identity which cannot be equated with another hidden parameter without
evidence.

## Equality, ordering, and keys

Equality is a capability, not an operation available for every object:

```text
Equality T
  equal : T, T -> Boolean
```

Its evidence promises reflexivity, symmetry, transitivity, and compatibility
with the observable value semantics of `T`. `=` and `!=` use this capability.
The language derives equality for tuples, records, variants, unions, and finite
recursive values when every observed component has equality.

Algorithms, continuations, task capabilities, environment endpoints, external
resources, and opaque values do not receive value equality automatically.
A type may expose an explicit stable identity with equality when identity is
part of its public semantics. Possessing two capabilities for the same task,
for example, does not otherwise make task identity observable.

Partial and total ordering are distinct capabilities. Sorting and ordered maps
require an explicit total order; numeric types whose exceptional values prevent
a total order expose only the weaker capability unless a policy supplies one.
Alternative culturally sensitive string comparisons are explicit comparison
objects rather than implicit `String` ordering.

Map and set membership requires stable equality and a compatible key strategy.
Hashing and tree ordering are replaceable implementation strategies and are not
observable properties of `Map` or `Set`. A specialized representation may
require `Hash T` or `TotalOrder T`, but converting it to the ordinary semantic
collection forgets that representation evidence.

## Law evidence

Parallel reduction and other transformations whose legality depends on
algebraic laws accept evidence for those laws:

```text
Associative operation
Commutative operation
Identity operation value
```

The compiler derives law evidence for language-defined exact arithmetic and
other fundamental operations. User declarations may provide evidence, and the
compiler verifies it when a decidable proof is available. An unverified foreign
claim is trusted boundary code and is identified as such in compiled metadata.

Law evidence is not inferred from an algorithm's name. An overload called
`sum`, for example, is not assumed associative for an approximate numeric type.

## Type identity and transparency

A type alias gives another name to the same type object. A type declaration
constructs a distinct nominal type, even when its visible structure is equal to
that of another declaration. Instantiating one generic type declaration with
definitionally equal parameters produces the same type identity; instantiating
different declarations does not.

Records and unions declared as types are nominal at module boundaries.
Anonymous records and positional products are structural. A declared type may
explicitly expose a lossless conversion to or from its representation without
making the two types identical.

An opaque public type exposes its identity and declared public capabilities but
not its representation. Static introspection follows the same rule and cannot
recover private structure. Constraints refine a base type and attach evidence;
forgetting the constraint returns the same underlying value and is not a change
of nominal identity.

Recursive type identity is established by the declaration being defined, not by
infinite structural expansion. Static introspection and serialization retain
that identity when traversing recursive structure.

## Conversion relations

Topal distinguishes conversions by the guarantee they provide:

- **Evidence forgetting** removes a refinement, static guarantee, or capability
  view without changing the value. It is implicit.
- **Lossless conversion** changes the semantic type while preserving all
  information. It may be implicit only when one canonical conversion exists and
  overload selection remains unambiguous.
- **Checked conversion** validates a value and produces `Result` plus evidence
  on success.
- **Lossy conversion** discards or rounds information and is always explicit,
  with a named policy where more than one result is reasonable.
- **Representation interpretation** reads or writes an external layout and is
  an effectful boundary operation, never an ordinary conversion.

Implicit conversions are considered only after exact type matches during
overload resolution. A call is ambiguous when two candidates require equally
strong conversions; the output type does not break the tie. The compiler must
report the candidate conversions rather than silently selecting one.

Algorithm inputs are contravariant and outputs covariant only where the
conversion involved is implicit and effect, fallibility, and static guarantees
are preserved. Mutable external locations, resumable generator inputs, and
message protocol directions are invariant unless their defining capability
proves a safe variance relation.

## Derivation and visibility

Derived evidence has the visibility of the least-visible declaration on which
it depends. Publishing a type does not accidentally publish equality,
serialization, construction, or representation evidence based on private
fields. A module explicitly publishes the capabilities intended to form its
interface.

Static introspection can construct derived implementations from visible
semantic views. Generated implementations are ordinary declarations in the
compiled interface, with stable identities, dependency information, effects,
and source attribution.

## Surface syntax still to choose

The semantic model does not decide:

- the punctuation introducing explicit generic parameter lists;
- the declaration spelling for capabilities and their implementations;
- the spelling for opening an existential package; or
- whether common capability bounds have a compact `where` form.

Those choices should be made together with the final grammar. They must not
change the classifications, evidence, coherence, or conversion rules above.
