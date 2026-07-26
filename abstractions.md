# Generic abstraction and semantic capabilities

Topal generic code uses classification and matching rather than a separate
generic-parameter language. A function input pattern can bind the complete
type of its value, match the construction of that type, and require capabilities
of the matched objects. The function body is the implicit successful branch of
that match.

Constraints and capabilities both produce evidence, compose with matchers, and
participate in static introspection. They remain different kinds of object:

- a constraint limits the permitted values of one base type; and
- a capability promises an interface and laws provided by a type, function, or
  other static object.

## Constraints limit values

A constraint combines a base type with a predicate over values of that type.
The base type is the left operand and the inferred anonymous function is the
right operand:

```topal
Positive is Integer constraint { value }
  value > 0
```

Conceptually:

```text
Positive : Constraint Integer
```

A constraint already retains its base type, so classification does not repeat
it:

```topal
count : Positive
```

Static values are checked during compilation. A dynamic value is validated at
the classification and produces `Result` plus evidence on success. Successful
classification denotes the base value refined by that evidence; forgetting the
evidence recovers the unchanged base value.

A constraint can occupy a classified component of another type construction.
The construction uses its base type and retains the constraint evidence:

```topal
positive-values : List Positive
```

This is a list whose entries are integers carrying `Positive` evidence; it does
not make `Positive` a type or erase the distinction between their kinds.

Constraints compose as matchers:

```topal
InteriorIndex is Nat constraint { index }
  index >= 0 and index < length

NonBoundaryIndex is InteriorIndex and != 0
```

`and` retains both pieces of evidence. `or` retains evidence identifying the
successful alternative. Constraints may depend on earlier values in the same
record or pattern, which makes relationships such as `end > start` part of the
classified value.

A predicate is a pure, total `T -> Boolean` function classification rather than
a separate fundamental object kind. Ordinary higher-order functions may accept
and compose `Predicate T` values when they need a runtime decision.

A constraint is normally passed as a static input when a function uses its
identity in a parameter or result classification. Applying it successfully
produces evidence for that particular constraint. If a constraint is selected
dynamically, successful validation instead returns an existential package
containing the selected constraint identity, the unchanged base value, and its
evidence. Accepting only `Predicate T` is preferable when the caller needs a
Boolean decision rather than reusable classification evidence. The complete
distinction is defined under
[passing predicates and constraints](types.md#passing-predicates-and-constraints).

## Capabilities promise interfaces

A capability does not remove values from a type. It states that an object of a
particular kind provides semantic operations or laws. Satisfying a capability
produces static evidence containing those operations and any related objects:

```text
Sequence A
  Element : Type
  Index : Type
  get : A, Index -> Element
  entries : A -> Traversal Element
```

The capability is not a namespace owned by `A`. `get` remains an ordinary
overload whose applicability is established by the evidence. This preserves
Topal's independent function composition and avoids introducing methods.

Capabilities may require other capabilities and may refine their associated
objects. For example, `TotalOrder T` requires `Equality T`, while a writable
layout requires the operations and access evidence of a readable addressed
layout plus permission to write.

Evidence can arise in three ways:

- the language derives it from a fundamental construction and the evidence of
  its components;
- a declaration explicitly supplies the required objects and functions; or
- a static matcher establishes it from already available evidence.

Merely having functions with suitable names is insufficient. Accidental
structural similarity must not silently assert laws such as associativity,
ordering, uniqueness, or losslessness. The compiler may derive structural
operations for products, sums, and finite recursion when every component
provides the required evidence.

## Coherence and implementation ownership

Each canonical object-capability pair has at most one implementation in a
compiled context. For a type capability this means, for example, that
`Equality Money` has one interpretation: there is no second explicit or
implicit `Equality Money` value with different semantics. The rule applies
equally when the classified object is a function, construction, or other static
object.

An implementation may be declared only in the canonical definition context of
the capability or of the object which satisfies it. Ownership extends across
the defining package's own module organization; it does not extend to an
unrelated package merely because that package imports both definitions.
Consequently, an adapter which owns neither an external type nor an external
capability cannot attach the capability directly to that type.

Third-party integration instead introduces an owned specialization or type
construction and implements the capability for that new canonical type. The
specialization can expose a canonical lossless conversion or evidence-forgetting
relation to the general boundary type where their semantics permit it. For
example, a database adapter can define `UuidParameter`, retain its underlying
UUID value, implement the database parameter capability in the adapter's
context, and allow it wherever the general parameter contract is accepted.
Construction at the boundary makes the adaptation visible without changing
the capabilities of the external UUID type.

Different enduring semantics likewise use different canonical types,
specializations, or capability parameters. A case-folded string type may
provide `CaseInsensitive` and derive its canonical equality from that promise;
ordinary `String` does not thereby acquire an alternative equality. A choice
which belongs only to one operation, such as selecting a collation for one
sort, is an explicit strategy input or named operation and does not establish
another capability implementation for the original type.

Derived evidence obeys the same coherence rule:

1. An explicit owner implementation is canonical and suppresses derivation.
2. Otherwise exactly one applicable derivation may construct the evidence.
3. Several applicable derivations for the same pair are a compile-time error.
4. The capability or object owner resolves that error by declaring the
   canonical implementation explicitly.

Derivation order, import order, and filesystem discovery never select among
competing paths. Compiled interfaces retain the canonical implementation
identity, its source attribution, required evidence, inferred effects, and any
additional promises or optimization properties.

## Type patterns in function headers

A function header is a static matcher as well as an ordinary value pattern.
Chained classification is read from left to right:

```topal
sort is fn ( values : C : Sortable ) -> C
  sorting-implementation values
```

This establishes:

```text
C : Sortable
values : C
```

`C` binds the complete input type. Reusing it as the output type promises that
the function returns exactly the same type, retaining its nominal identity,
static sizes, constraints, and other parameters.

There is no `then` in a function header. If the static type and capability
match succeeds, the function body is its implicit action. If it fails, that
overload is not applicable. A decision table instead uses `then` because it
chooses a runtime action among matchers.

Type constructors use the same syntax for matching as for construction. A
header may therefore decompose a positional type and use its components in the
result:

```topal
swap is fn (
  pair : Tuple ( X, Y )
) -> Tuple ( Y, X )

  pair
    ( x, y ) then ( y, x )
```

The header statically binds `X` and `Y`; the decision table separately
decomposes the runtime pair. For labeled products, labels belong to the scope of
their respective record types and field selection remains total:

```topal
swap-record is fn (
  record : Record (
    left : X,
    right : Y
  )
) -> Record (
  left : Y,
  right : X
)

  Record (
    left is record right,
    right is record left
  )
```

The repeated labels do not shadow one another. The input `left` and `right`
belong to the input record type, while the output labels belong to the newly
constructed result type.

Header matches happen during type checking. They do not require runtime
reflection or dynamic dispatch. Repeated names must match the same object,
opaque types expose only published construction and capability evidence, and
overlapping overload headers are resolved by their source declaration order.

## Container construction patterns

A homogeneous container construction has the conceptual pattern:

```text
Container Value
```

Matching `List Integer` binds `Container` to `List` and `Value` to `Integer`.
Matching `Array N Integer` can bind `Container` to the partially constructed
`Array N`, retaining `N` in the complete matched type. `Map ( K, V )` does not
match this unary construction pattern: its type constructor accepts the
key/value product as one explicit argument rather than exposing an independently
matched `Value`.

Capabilities can describe the container construction and its entry type in one
matcher. For example:

```topal
Sortable is ( Indexed and Replaceable ) Container ( TotalOrder Value )
```

This promises indexed access and immutable replacement for the matched
container, and total ordering for the `Value` obtained from that container
construction. It does not independently guess an unrelated `Value` type.

The generic sorting signature is consequently:

```topal
sort is fn ( values : C : Sortable ) -> C
  sorting-implementation values
```

The complete `C` provides the exact input/output relationship, while matching
`Sortable` exposes `Container`, `Value`, and the promised operations inside the
body. A sort which builds a new collection can substitute an appropriate
construction capability for `Replaceable`.

Another function can match the map construction directly and use its captured
components in a different result type:

```topal
map-entries is fn (
  mapping : Map ( K, V )
) -> Set ( Tuple ( K, V ) )

  mapping entries
```

The header binds `K` and `V` from the input `Map` type. The result then
constructs the map's entry-product type explicitly. `Map ( K, V )` and
`Tuple ( K, V )` are different constructions; no structural match equates
them.

An ordinary unordered map also does not satisfy the `Sortable` pattern above.
Code can obtain its entries and order that separate collection by an explicit
key, value, or product comparison.

## Associated objects

An associated object belongs to the canonical capability evidence rather than
to a global type namespace. It may have any static kind:

```text
Element A : Type
Index A : Type
effects function : Set Effect
identity operation : lang Identity Function
```

Selecting an associated object requires evidence identifying the capability
implementation. Coherence makes the short conceptual spelling `Element A`
unambiguous for a known `A`. When the identity of `A` or its implementation is
existentially packaged, selection retains that packaged evidence rather than
searching for an alternative implementation.

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

Functions, continuations, task capabilities, context-provided endpoints, external
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

The initial comparison, collection, and function-law interfaces are collected
in the [standard capability vocabulary](capabilities.md).

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

Law evidence is not inferred from a function's name. An overload called
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

Topal does not organize all objects or value types into one inheritance tree.
`Type`, `Constraint`, `Predicate`, `Function`, and `Capability` are different
object kinds, not supertypes of the values they describe or classify. An
interface type is a `Type` construction which groups function and generator
declarations. It may be implemented directly by a source context, packaged as a
value, or implemented through task message passing. Its concrete implementation
evidence retains inferred effects and optimization properties. Likewise,
satisfying `Sequence`, `Equality`, or another capability supplies evidence that
an object provides an interface and its laws; it does not convert that object
to a capability value or to a common nominal supertype.

Compatibility between values is instead described by directed conversion
relations. Topal distinguishes them by the guarantee they provide:

- **Evidence forgetting** removes a refinement, static guarantee, or capability
  view without changing the value. It is implicit.
- **Lossless conversion** changes the semantic type while preserving all
  information. It may be implicit only when one canonical conversion exists.
- **Checked conversion** validates a value and produces `Result` plus evidence
  on success.
- **Lossy conversion** discards or rounds information and is always explicit,
  with a named policy where more than one result is reasonable.
- **Representation interpretation** reads or writes an external layout and is
  an effectful boundary operation, never an ordinary conversion.

These relations must not be inferred merely because two types have similar
operations or representations. In particular:

- A constrained value can be used as its base value by forgetting evidence.
  For example, `Nat` is the nonnegative refinement of `Int`, and `Index A`
  forgets its bound and domain evidence to become `Nat`.
- A capability requirement is satisfied by passing the original object together
  with static evidence. A `List T`, `Array N T`, or `String` can therefore meet
  a `Sequence` requirement without first converting to a `Sequence` value or a
  nominal `Container` type.
- A type construction is not a subtype relation. `List T`, `Set T`, and
  `Map ( K, V )` are distinct constructions even when they share capabilities.
  `Tuple`, `Record`, `Variant`, and `Union` similarly describe product and sum
  structure rather than positions in a container hierarchy.
- A canonical embedding between distinct numeric domains is a lossless
  conversion, not automatically a refinement. The finite-to-extended relations
  such as `Int` to `ExtendedInt` are lossless, while the reverse direction is
  checked because an infinity cannot become an `Int`. Approximation, rounding,
  saturation, wrapping, and interpretation from bits remain explicit.

Evidence forgetting is transitive: a value may forget several refinements on
its way to a visible base type. Lossless conversions may be composed implicitly
only when the complete path is canonical, preserves every intermediate
guarantee required by the destination, and has one unambiguous path. The
compiler does not search an open-ended graph for a convenient coercion. A
declared direct conversion wins over a longer composed path; two otherwise
equivalent canonical paths make the conversion ambiguous and require an
explicit choice.

Overload declarations are tested in source order. For each declaration, header
matching may use evidence forgetting and then a lossless semantic conversion.
The first applicable declaration is selected, even when a later declaration
would require a shorter conversion or match the input exactly. Declaration
order is the explicit precedence; conversion quality, capability satisfaction,
and the output type do not reorder candidates. The compiler may optionally
diagnose when an earlier conversion preempts a later exact match and report the
conversion path and capability evidence which made the earlier declaration
applicable.

Checked and lossy conversions never participate in overload resolution. An
expected input type may select a unique implicit conversion, but an expected
output type cannot change the selected overload. Generic matching retains the
complete input type whenever possible, so requiring a capability does not
prematurely erase refinements, identities, sizes, or other static parameters.

Function inputs are contravariant and outputs covariant only where the
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

- the declaration spelling for capabilities and their implementations;
- the spelling for opening an existential package;
- how a type-construction matcher exposes several independently classified
  components beyond the homogeneous `Container Value` case; or
- whether capability implementations with no implicit default need a compact
  explicit-selection form.

Those choices should be made together with the final grammar. They must not
change the classifications, evidence, coherence, or conversion rules above.
