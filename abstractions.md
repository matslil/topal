# Generic abstraction and semantic capabilities

Topal generic code uses classification and matching rather than a separate
generic-parameter language. A function input pattern can bind the complete
type of its value, match the construction of that type, and require capabilities
of the matched objects. The function body is the implicit successful branch of
that match.

Constraints, capabilities, and resource complexity guarantees produce evidence,
compose with matchers, and participate in static introspection. They remain
different kinds of object:

- a constraint limits the permitted values of one base type; and
- a capability makes semantic promises about an existing type, function, or
  other static object; while
- a resource complexity guarantee bounds how an implementation's execution work
  or allocation grows with measures of its inputs or represented values.

The third kind is defined in
[resource complexity guarantees](performance.md). It may be combined with a
capability, but forgetting it loses only a performance specialization and does
not forget the capability's semantic promise.

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

## Capabilities compose promises

A capability does not remove values from a type. It states that an object of a
particular kind satisfies semantic promises. The selected Topal version
supplies the initial atomic capabilities and defines what existing ordinary
operations and laws each promises. Conceptual vocabulary such as:

```text
Sequence A
  Element : Type
  Index : Type
  get : A, Index -> Element
  entries : A -> Traversal Element
```

means that `Sequence A` promises the availability and relationships of those
already-existing ordinary functions and static objects. It does not define
them, contain their implementations, or introduce a namespace owned by `A`.
`get` remains an ordinary overload whose applicability is established by the
evidence.

Source code cannot declare a new atomic semantic promise. It may bind
conjunctions and alternatives of existing capabilities:

```topal
Searchable is Foldable and Membership
Serializable is JsonSerializable or BinarySerializable
```

`and` forms evidence containing every promise. `or` retains evidence for the
available alternative or alternatives rather than selecting one by import or
discovery order. Nested expressions flatten, duplicate capabilities collapse
by canonical identity, and conjunction order is irrelevant.

Ordinary static functions may construct parameterized capability expressions,
but their results must still be combinations of existing capabilities:

```topal
Searchable is fn static (
  Container : Type,
  Value : Type
) -> Capability
  Foldable ( Container, Value )
    and Membership ( Container, Value )
```

A capability supplies the kinds of the objects it classifies. Consequently,
`Value : TotalOrder` already establishes `Value : Type`; spelling
`Value : Type : TotalOrder` is valid but redundant. A bare unclassified name
still introduces nothing.

Evidence can arise in three ways:

- the language derives it from a fundamental construction and the evidence of
  its components;
- a declaration makes a claim using an existing capability; or
- a static matcher establishes it from already available evidence.

Claims never supply an implementation. Every operation involved is an ordinary
function declared independently. Merely having functions with suitable names
is insufficient: accidental structural similarity must not silently assert
laws such as associativity, ordering, uniqueness, or losslessness.

A claim applies an existing promise directly to its subject:

```topal
Associative addition
TotalOrder Customer
```

The first makes a promise about the already-declared `addition` function. The
second makes `Customer` eligible for generic `TotalOrder` code and refers to the
ordinary comparison vocabulary fixed by that atomic capability. Neither
declaration contains executable code. A concrete `compare (Customer, Customer)`
overload may exist without the second claim when only that specialized function
needs to compare customers.

## Coherence and claim ownership

Each canonical object-capability pair has at most one interpretation in a
compiled context. For a type capability this means, for example, that
`Equality Money` has one interpretation: there is no second explicit or
implicit `Equality Money` value with different semantics. The rule applies
equally when the classified object is a function, construction, or other static
object.

A claim may be declared only in the canonical definition context of the
capability or of the object which satisfies it. Ownership extends across the
defining package's own module organization; it does not extend to an unrelated
package merely because that package imports both definitions. Consequently, an
adapter which owns neither an external type nor an external capability cannot
attach the capability directly to that type.

Third-party integration instead introduces an owned specialization or type
construction and claims the capability for that new canonical type. The
specialization can expose a canonical lossless conversion or evidence-forgetting
relation to the general boundary type where their semantics permit it. For
example, a database adapter can define `UuidParameter`, retain its underlying
UUID value, claim the database parameter capability in the adapter's context,
and allow it wherever the general parameter contract is accepted.
Construction at the boundary makes the adaptation visible without changing
the capabilities of the external UUID type.

Different enduring semantics likewise use different canonical types,
specializations, or capability parameters. Independent promises compose: text
may provide `CaseInsensitive`, `Language Swedish`, both, or neither.
Language-sensitive case operations require both capabilities, while the
universal case operations require only `CaseInsensitive`. Neither capability
replaces the exact canonical equality of `String`. A choice which belongs only
to one operation, such as selecting a collation for one sort, is an explicit
strategy input or named operation and does not establish another capability
interpretation for the original type.

Derived evidence obeys the same coherence rule:

1. An explicit owner claim is canonical and suppresses derivation.
2. Otherwise exactly one applicable derivation may construct the evidence.
3. Several applicable derivations for the same pair are a compile-time error.
4. The capability or object owner resolves that error by declaring the
   canonical claim explicitly.

Derivation order, import order, and filesystem discovery never select among
competing paths. Compiled interfaces retain the canonical evidence identity,
its source attribution, required evidence, inferred effects, and any additional
promises or optimization properties.

## Type patterns in function headers

A function header is a static matcher as well as an ordinary value pattern.
Chained classification is read from left to right:

```topal
sort is fn (
  values : ( C : Sortable )
) -> C
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
  pair : Tuple (
    X : Type,
    Y : Type
  )
) -> Tuple ( Y, X )

  pair
    ( x, y ) then ( y, x )
```

The explicitly classified pattern statically binds `X` and `Y`; the decision
table separately decomposes the runtime pair. For labeled products, labels
belong to the scope of their respective record types and field selection
remains total:

```topal
swap-record is fn (
  record : Record (
    left : ( X : Type ),
    right : ( Y : Type )
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
reflection or dynamic dispatch. A new pattern binding states its classification
explicitly; a later bare occurrence refers to the exact object already bound.
For example:

```topal
same-shaped-pairs is fn (
  left : Tuple (
    A : Type,
    B : Type
  ),
  right : Tuple ( A, B )
) -> Boolean
```

`Object` is the top classification when a construction genuinely accepts any
language object. It preserves the captured object's actual kind; kind-specific
operations first refine it to `Type`, `Function`, `Capability`, or another
appropriate classification. A bare unbound name is never introduced
implicitly.

Opaque types expose only published construction and capability evidence, and
overlapping overload headers are resolved by their source declaration order.

## Discarded and retained construction parameters

`_` may occupy one construction-parameter binding and discard its local name:

```topal
array : Array (
  _ : Nat,
  Int
)
```

This accepts every array size with the exact element type `Int`. `_` remains a
discard identifier rather than a general wildcard: the parameter must exist
and satisfy `Nat`, but its identity receives no source binding. To use the
parameter, the pattern names it:

```topal
array : Array (
  array-size : Nat,
  Int
)
```

Discarding or omitting a local name never removes the parameter from the
complete matched type. An `Array (12, Int)` matched by the first pattern remains
exactly `Array (12, Int)` in type identity, capability evidence, introspection,
and compiled metadata. A result which needs to name the size must capture it or
capture the complete input type.

## Visibility and open records

Matching uses only nominal identity, construction views, fields, and capability
evidence visible in the lexical context. Code in a type's defining private
scope may match its private construction. External code cannot match an opaque
type through hidden representation, and `lang view` reports the same
visibility-respecting semantic view rather than bypassing the boundary.

An open record pattern accepts an anonymous structural record containing at
least its declared visible fields:

```topal
identifier-of is fn (
  value : Record (
    id : Identifier,
    ...
  )
) -> Identifier

  value id
```

Additional fields remain part of the complete input type but are not locally
named by `...`. The marker does not capture a record row or grant generic
reconstruction authority. Field order outside the stated fields does not affect
the match.

A nominal record does not satisfy an open structural pattern merely because its
visible fields happen to have matching names and types. Its owner must
explicitly publish the corresponding structural view or a semantic capability.
Private fields never participate outside their scope, and an opaque type never
matches through hidden fields. Reconstructing a nominal record continues to
require owner-published construction or replacement evidence so an open pattern
cannot bypass invariants.

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
sort is fn (
  values : ( C : Sortable )
) -> C
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
  mapping : Map (
    K : Type,
    V : Type
  )
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

## Capability components

An atomic capability supplied by the selected language version may relate
several existing objects. Those component objects belong to its canonical
evidence rather than to a global type namespace and may have any static kind:

```text
Element A : Type
Index A : Type
effects function : Set Effect
identity operation : lang Identity Function
```

Selecting a component requires evidence identifying the capability
interpretation. Coherence makes the short conceptual spelling `Element A`
unambiguous for a known `A`. When the identity of `A` or its evidence is
existentially packaged, selection retains that packaged evidence rather than
searching for an alternative interpretation.

Type-valued components may depend on static values and identities. `Index A`,
for example, retains the identity and bound of a concrete array type rather
than weakening to the index type of every array.

### Multi-component capability matching

A capability with several component objects groups them into one explicit
component argument. Chained classification inserts the complete classified
object as the capability's first argument:

```topal
lookup-value is fn (
  mapping : (
    C : Keyed (
      Key : Type,
      Value : Type
    )
  ),
  key : Key
) -> Option Value

  mapping lookup key
```

Conceptually, the canonical evidence is:

```text
Keyed C (
  Key,
  Value
)
```

This respects the zero-to-two operand rule: `C` is the first operand and the
key/value component product is the second. `Key` and `Value` come from the
canonical `Keyed C` evidence for this exact `C`; the matcher does not search for
unrelated types with suitable capabilities or infer them from a coincidentally
similar construction.

An opaque `UserDirectory` may therefore publish `Keyed` evidence associating
`UserId` and `User` without exposing whether its representation is a map, tree,
database, or remote service. Conversely, `Map (Key, Value)` and
`Tuple (Key, Value)` remain unrelated constructions despite accepting similarly
shaped parameter products.

Components use ordinary explicit bindings and discards:

```topal
mapping : (
  C : Keyed (
    _ : Type,
    Value : Type
  )
)
```

Additional component promises chain normally, as in `Key : TotalOrder`.
Discarded components retain the kinds supplied by `Keyed` and remain in the
capability evidence and complete classified object.

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
other fundamental operations. Programmers may declare law evidence for any
operation they own or can lawfully implement evidence for. The compiler
attempts a sound verification when one is available and classifies the result:

- **verified** evidence has a compiler proof or exhaustive verification over a
  proven finite domain;
- **trusted-unverified** evidence is a programmer claim which the compiler can
  neither prove nor refute; and
- a **refuted** claim is rejected, with a counterexample when one is available.

Trusted-unverified evidence is accepted and may authorize the same
transformations as verified evidence, because the programmer has explicitly
assumed responsibility for the law. The compiler emits `unverified-law` at the
claim by default. Suppressing that warning does not relabel the evidence.
Sampled or generated tests may refute a law but successful samples do not
verify it.

Capabilities required for the compiler's own safety or totality guarantees are
an exception. In particular, `Decreases` evidence must be verified and cannot
be introduced as trusted-unverified evidence.

Compiled evidence retains the exact operation identity, law, relevant static
parameters, verification status, declaration provenance, and verification
method. Consumers may reject trusted-unverified evidence by build policy, but
otherwise rely on the published claim without warning at every use.

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

Overload declarations are ordinarily tested in source order. For each
declaration, header matching may use evidence forgetting and then a lossless
semantic conversion. The first applicable declaration is selected, even when a
later declaration would require a shorter conversion or match the input
exactly. Declaration order is the explicit precedence; conversion quality,
capability satisfaction, and the output type do not reorder candidates.

An explicit call-site resource `Prefer` construction may rank already
applicable implementations by their retained complexity evidence before this
source-order step. It changes neither header applicability nor conversions and
falls back to source order for unsupported, equivalent, or incomparable
preferences. The complete model is defined in
[resource complexity guarantees](performance.md).

The compiler may optionally diagnose when an earlier conversion preempts a
later exact match and report the conversion path and capability evidence which
made the earlier declaration applicable.

Evidence forgetting may satisfy a concrete classifier but never changes an
already captured complete object. Repeated pattern names require definitional
equality before conversion and cannot be made equal by forgetting evidence. A
generic header which captures `C` continues to operate on and return that
original complete type.

A concrete header which cannot match the original input may use one canonical
lossless conversion. Its body receives the declared destination type; the
caller's immutable source value and type remain unchanged. A header which needs
to retain the source instead captures its complete type and separately requires
lossless-conversion evidence. Conversion does not participate in structural
unification or repeated-name equality. Several otherwise valid conversion paths
make the match fail until the caller selects one explicitly.

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

The semantic model does not decide the spelling for opening an existential
package.

Those choices should be made together with the final grammar. They must not
change the classifications, evidence, coherence, or conversion rules above.
