# Object and type taxonomy

Topal exposes values, types, functions, constraints, capabilities, and other
compile-time entities through one object model. This does not place them in one
runtime inheritance tree. Each object has a kind which determines where it can
be used and which static operations can inspect it.

This document defines the common names used when classifying objects and value
types. More specialized documents define their construction and operations.

## Objects and kinds

`Object` is the top classification used when an interface may return or store a
language object without statically knowing its more specific kind. Every
language object is an `Object`, including a value, type, function, constraint,
or capability. `Object` is not a universal runtime value type, and classifying
something as `Object` does not erase the object's kind. A caller must establish
that kind before applying a kind-specific operation.

The principal kinds discussed by the initial model are:

```text
Object
  Value
  Type
  Function
    Predicate T
  Constraint T
  Capability
```

- A **value** is runtime data classified by a type, possibly with retained
  constraint or identity evidence. `Value` is a metamodel classification used
  by generic descriptions; it is not a source type to which ordinary values
  are implicitly converted. “Values” is ordinary prose for multiple values,
  not a separate classification.
- A **type** describes a set of values and their semantic operations. Types are
  themselves static objects of kind `Type`.
- A **function** is constructed with `fn` and maps classified inputs to a
  declared output. Calling one may produce runtime values or other statically
  determined objects.
- A **predicate** is a pure, total function returning `Boolean`. `Predicate T`
  is therefore a function classification, not another fundamental object kind.
  It accepts values of `T`; ranges are predicates with additional convexity
  evidence.
- A **constraint** combines a base type with a predicate and classifies values
  for which that predicate holds. Successful classification retains evidence;
  forgetting it recovers the unchanged base value. Constraints remain distinct
  static objects because they establish reusable classification evidence rather
  than merely returning `Boolean`.
- A **capability** promises that an object of a particular kind provides named
  operations, associated objects, and laws. Capability satisfaction supplies
  static evidence; it is not a value conversion or a nominal supertype.

Kinds remain distinct even though their objects are first-class. A `Type`
cannot be passed where a runtime `Value` is required, and a `Capability` cannot
be used as the type of every value satisfying it. Typed
[static introspection](introspection.md) preserves these distinctions.

## Passing predicates and constraints

Because functions are first-class, an ordinary higher-order function can accept
a predicate:

```topal
select is fn (
  values : List T,
  accepts : Predicate T
) -> List T
```

The predicate may be named, composed, or supplied anonymously. Its purity and
totality let `select` use the Boolean result without introducing hidden effects
or failure:

```topal
positive : Predicate Int
even : Predicate Int

positive-and-even is positive and even
values select positive-and-even
```

A function can also accept a constraint. A constraint which participates in a
parameter or result classification is normally a static input, because its
identity must be known while the function's type is checked:

```topal
describe-constraint is fn static (
  requirement : Constraint T
) -> String

element-constraint is fn static (
  requirement : Constraint T
) -> Constraint (List T)
```

Applying a statically known constraint to a value can return that same value
with evidence for the particular constraint. The constraint object and the
evidence that a value satisfies it are distinct objects.

A dynamically selected constraint can still validate a value, but the caller
cannot name that constraint in an ordinary static result type. Success must
package the selected constraint identity, the original value, and its evidence
existentially:

```text
validate : T, dynamic Constraint T
        -> Result (exists C : Constraint T. T with evidence C)
```

Code which needs only runtime acceptance can instead accept `Predicate T` and
receive `Boolean`. Code which needs a reusable refined classification accepts a
static `Constraint T`. This phase distinction keeps dynamic validation
meaningful without pretending that runtime selection created compile-time
knowledge.

## Fundamental value classifications

`Boolean` is the two-valued logical type with values `true` and `false`.
Predicates and logical operations return `Boolean`; it does not implicitly
coerce to or from numeric values.

`Unit` is the type with exactly one value, written `()`. It represents the
absence of additional information, including the result of a function called
only for its effects and the payload of a sum alternative which carries no
data.

`String` represents semantic Unicode text. Its detailed value, character,
normalization, indexing, and encoding model is defined in
[strings](strings.md).

`Number` is the generic classification for numeric types, not one additional
numeric representation. A matcher requiring `Number` accepts a numeric type
while retaining its complete type and laws. It therefore does not erase an
`Int`, `Decimal`, or `Approx` value to a boxed common number, and it does not
promise that every numeric operation has the same result type or algebraic
laws. The concrete domains include `Int`, `Nat`, `Rational`, `Decimal`,
`Approx`, their applicable extended forms, modular numbers, and measured
numeric quantities. Their semantics and conversions are defined in the
[number model](numbers.md).

The initial numeric vocabulary uses `Approx` and `ExtendedApprox`.
`Approximate` and `ExtendedApproximate` are descriptive English, not additional
type names.

`Comparison` and `PartialComparison` are enum result types for total and partial
ordering. `Comparison` contains `Less`, `Equal`, and `Greater`;
`PartialComparison` additionally contains `Incomparable`. Their precise use by
`TotalOrder` and `PartialOrder` is defined in the
[capability vocabulary](capabilities.md#value-comparison).

## Products, sums, and enums

`Tuple` and `Record` construct positional and labeled products. `Variant` and
`Union` construct positional and labeled sums. These are structural
relationships, not branches beneath a nominal `Container` type. Their
construction, matching, and identity rules are defined in
[containers and algebraic data](containers.md).

An `Enum` is a nominal union whose alternatives have labels but no payloads.
Conceptually:

```topal
Color is Enum (
  Red,
  Green,
  Blue
)
```

has the sum shape:

```topal
Union (
  Red : Unit,
  Green : Unit,
  Blue : Unit
)
```

The `Enum` declaration retains the intent that each alternative is a complete
named value. It supports exhaustive matching and enum-specific introspection,
display, and serialization without adding a separate sum mechanism. Once any
alternative carries a payload, the declaration is a general `Union` rather
than an enum.

## Collections, container matching, and sequence evidence

The core collection families are `List`, `Array`, `Set`, `Map`, and `Bag`.
They are distinct type constructions which preserve different laws concerning
order, multiplicity, indexing, and key association.

`Container` is the name bound by a homogeneous type-construction matcher such
as `Container Value`. It represents the matched construction, for example
`List` or a partially applied `Array N`, rather than a universal nominal type
inhabited by every collection. Maps and heterogeneous products do not
necessarily match that single-element form.

`Sequence Container Value` is a capability. It promises finite traversal which
preserves order and multiplicity. Lists, arrays, and strings provide sequence
evidence, but a value does not convert to a `Sequence` value when using that
evidence. Positional access is a separate `Indexed` capability.

Shared collection behavior is defined in the
[capability vocabulary](capabilities.md), while construction and matching are
defined in [containers](containers.md) and
[generic abstraction](abstractions.md).

## Index types

`Index A` is an associated type selected from indexing evidence for a concrete
type `A`. For a finite array-derived type it is a refinement of `Nat` which
retains both bounds evidence and the identity of the indexed domain:

```topal
Row is Array RowCount T
row-index : Index Row
```

Forgetting that evidence implicitly produces `Nat`. Constructing `Index Row`
from an unchecked `Nat` requires validation. `Index Row` is not a subtype of a
universal `Index Array`, because an unrefined array does not identify one bound
or nominal index domain. Generic functions preserve `A` and its associated
`Index A` instead.

The complete array-bound model is described under
[array-bound index types](containers.md#array-bound-index-types).
