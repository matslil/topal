# Static introspection

Topal provides static introspection over first-class language objects. Static
algorithms may inspect the semantic structure of types, algorithms, scopes,
constraints, effects, protocols, and declarations without retaining reflection
metadata in the executable.

Introspection is a compiler facility, not an implicit ability of every runtime
value. Its operations and descriptor types belong to the compiler-provided
`lang` scope:

```topal
description is lang view Person
identity is lang identity Person
source is lang declaration Person
context is lang context
```

The qualified spelling deliberately distinguishes inspection of language
objects from ordinary application and data selection. Names such as `view`,
`identity`, `declaration`, `context`, and `public-members` are not introduced
unqualified into source files.

## Static phase

Every introspection operation is static. Its input must be a statically known
language object, and its result exists only during compilation unless ordinary
code deliberately converts selected information into runtime data:

```topal
describe is fn static ( subject : Type ) -> String
  lang view subject
    lang RecordType fields then describe-record fields
    lang UnionType alternatives then describe-union alternatives
    lang OpaqueType identity then "opaque {identity}"
    otherwise "other type"
```

`lang view subject` does not inspect an arbitrary runtime value to discover its
type. Runtime code instead uses its statically known type to obtain or derive
an ordinary algorithm:

```topal
serializer is serializer-for Person
bytes is serializer person
```

No type names, field descriptors, source locations, or other introspection data
are retained at runtime merely because static introspection is available.

## Typed views

Objects retain their distinct kinds. `lang view` is overloaded by the kind of
its input rather than returning one universal reflection value:

```topal
lang view : fn static ( Type ) -> lang TypeView
lang view : fn static ( Algorithm ) -> lang AlgorithmView
lang view : fn static ( Scope ) -> lang ScopeView
lang view : fn static ( Constraint T ) -> lang ConstraintView T
lang view : fn static ( Effect ) -> lang EffectView
lang view : fn static ( Protocol ) -> lang ProtocolView
```

A value of one kind cannot be inspected through another kind's view. A type
view cannot accidentally be treated as an algorithm view, and neither is an
untyped map of strings to arbitrary values.

Views are ordinary algebraic static values. Static algorithms inspect them
with normal pattern matching and transform them with normal composition. Topal
does not introduce a separate macro language or textual source substitution.

## Type views

The initial semantic type view contains the fundamental algebraic forms:

```topal
lang TypeView is Union
  lang PrimitiveType : lang PrimitiveDescriptor
  lang TupleType : lang ComponentStructure
  lang RecordType : lang FieldStructure
  lang VariantType : lang ComponentStructure
  lang UnionType : lang AlternativeStructure
  lang RefinedType : lang RefinementDescriptor
  lang AlgorithmType : lang AlgorithmSignature
  lang OpaqueType : lang Identity Type
```

These alternatives describe semantic construction and matching, not the syntax
which originally declared the type. A future surface construct which lowers to
a record presents as `lang RecordType`; it does not require every introspection
algorithm to recognize another source-level spelling.

A field or alternative descriptor retains typed objects:

```topal
lang Field is Record
  label : lang Label
  type : Type

lang Alternative is Record
  label : lang Label
  payload : Type
```

The field type is a first-class `Type`, not text containing a type name.
Likewise, labels use the static `lang Label` type rather than runtime `String`.

### Dependent structures

Record fields may depend on earlier fields:

```topal
Interval is Record
  start : Integer
  end : ( > start ) Integer
```

Consequently, `lang FieldStructure` is an ordered dependent structure rather
than `List (lang Field)`. Inspecting `end` retains its reference to the
preceding `start` field and the evidence required by its constraint.

Convenience operations may produce an independent list only when that loses no
information:

```topal
lang independent-fields :
  fn static ( Type ) -> Optional (List (lang Field))
```

`lang independent-fields Interval` produces `None`. The fundamental
`lang view` remains lossless for both dependent and independent records.

### Recursive structures

Views preserve recursive identity rather than expanding a recursive type
without end. When a component refers to an enclosing or previously visited
type, its descriptor carries that type's `lang Identity Type`. Static folds
over views must handle such references explicitly.

## Algorithm views

An algorithm view exposes its declared semantic contract:

```topal
lang AlgorithmView is Record
  identity : lang Identity Algorithm
  input : lang PatternType
  output : Type
  staticness : lang Staticness
  effects : lang EffectSet
  environments : lang EnvironmentRequirementSet
```

It does not expose compiler-generated machine instructions, optimization
choices, closure layout, or mutable implementation state. The body is not a
general source abstract syntax tree.

Algorithm inspection supports static derivation and verification:

```topal
accepts-errors is fn static ( algorithm : Algorithm ) -> Boolean
  signature is lang view algorithm
  signature output compatible-with Result
```

Whether implementation control flow becomes available to specialized compiler
facilities, such as exhaustive unit-test coverage, is separate from ordinary
static introspection. `lang view` does not bypass implementation visibility.

## Scope views

`lang view` of a scope exposes only the names visible at the inspection site:

```topal
members is lang public-members logging
```

Conceptually:

```topal
lang ScopeView is Record
  identity : lang Identity Scope
  members : lang MemberStructure

lang Member is Record
  name : lang Name
  object : lang SomeObject
```

`lang SomeObject` is an existential package retaining the member object's kind
and identity. A static matcher must establish that kind before applying a
kind-specific operation.

Member ordering is deterministic and defined by the selected language revision.
It does not depend on filesystem enumeration order, hash-map order, or compiler
parallelism.

## Visibility and opacity

Introspection observes the same interface ordinary source can observe. It
never makes private declarations, fields, alternatives, constraints, effects,
or scope members visible across a module boundary.

A published type whose construction or matching structure remains private
produces:

```topal
lang OpaqueType identity
```

The identity may be compared or used as the input to generic construction, but
the hidden structure cannot be recovered. A scope view likewise includes only
members published to the inspection site.

This rule applies transitively. Publishing a scope binding does not allow
`lang public-members` to reveal private objects reachable inside that scope.

## Structural and declaration views

Semantic structure and source declaration metadata are distinct:

```topal
structure is lang view Person
declaration is lang declaration Person
```

`lang view` answers what an object semantically is and how it composes.
`lang declaration` describes the visible declaration which introduced or
published a name:

```topal
lang DeclarationView is Record
  name : Optional (lang Name)
  canonical-path : Optional (lang Path)
  documentation : Optional String
  license : Optional License
  copyright : Optional Copyright
  language : lang LanguageContext
```

Aliases may therefore have different declaration views while referring to the
same semantic object. Declaration inspection does not expose private source
text or an unrestricted syntax tree. Metadata absent from the visible
declaration remains absent from its view.

## Names, labels, and paths

Introspection uses typed static names:

```topal
lang Name
lang Label
lang Path
```

They provide exact equality, canonical formatting, and safe qualification.
They are not interchangeable with runtime strings. Conversion to `String` is
available for generated documentation and diagnostics. Constructing a name
from text requires validation:

```topal
label : Result (lang Label)
label is lang label "age"
```

With a statically known string, validation occurs during compilation. Generated
field selection and construction consume the validated label object rather
than performing textual runtime lookup.

## Identity and relations

Introspection distinguishes several relations which must not be inferred by
manually comparing views:

```topal
A lang same-object B
A lang equivalent-type B
A lang compatible-with B
A lang same-layout B
```

- `lang same-object` compares stable language-object identity.
- `lang equivalent-type` compares semantic inhabitants and constraints.
- `lang compatible-with` applies the compatibility rules of the relevant
  artifact or boundary.
- `lang same-layout` compares explicit external representations.

These are qualified introspection operations even in infix position. Recursive,
dependent, nominal, and versioned objects make naïve recursive comparison of
their descriptors insufficient.

## Representation remains separate

A semantic `TypeView` does not expose field offsets, padding, byte order,
alignment, or compiler-selected runtime representation. External
representation belongs to the explicit layout model:

```topal
layout-description is lang view-layout packet-layout
```

`lang view-layout` accepts a `Layout T` and returns a typed
`lang LayoutView T`. Ordinary `lang view T` cannot be used to infer storage
properties which the type's semantic interface does not promise.

## Construction and derivation

Introspection becomes useful through typed construction rather than source
rewriting. Language-provided static constructors correspond to lossless view
forms:

```topal
lang record-type :
  fn static ( lang FieldStructure ) -> Type

lang union-type :
  fn static ( lang AlternativeStructure ) -> Type
```

Static algorithms can fold a view to derive ordinary algorithms:

```topal
serializer-for is fn static ( subject : Type ) ->
  fn ( subject ) -> Result Bytes

  lang view subject
    lang RecordType fields then record-serializer fields
    lang UnionType alternatives then union-serializer alternatives
    lang PrimitiveType primitive then primitive-serializer primitive
    lang OpaqueType identity then require-published-serializer identity
    otherwise unsupported-serializer subject
```

The returned algorithm is kind-checked in the usual way. It is not generated
source text, does not capture names textually, and cannot construct an invalid
field access.

For every public lossless structural view, reconstructing that view produces a
semantically equivalent object:

```text
lang reconstruct (lang view object) lang equivalent-type object
```

Reconstruction need not preserve nominal identity. An opaque view cannot be
reconstructed because its hidden structure was deliberately not exposed.

## Language context

The selected language revision and variants are statically introspectable:

```topal
context is lang context
```

Conceptually:

```topal
lang LanguageContext is Record
  revision : lang LanguageRevision
  variants : Set (lang LanguageVariant)
```

This reports the exact `use lang` selection of the current source file. It does
not report the compiler implementation version, claim compatibility with a
range of language revisions, or permit a build target to change the source
program's semantics.

`lang context` becomes available only after bootstrap language selection has
been parsed. It cannot influence the interpretation of the `use lang`
declaration which selected it.

## Language revisions

The alternatives and guarantees of introspection views belong to an immutable
language revision. Static code written under that revision sees its closed
`lang TypeView` and other view types. New surface syntax should normally lower
to an existing semantic form; a genuinely new semantic form requires a new
language revision.

Files selecting different revisions may exchange published objects only when
their interfaces establish the required interoperability. Each file inspects
an imported object through the view vocabulary and visibility available under
its own selected revision.

## Initial scope

The first introspection revision should provide:

- type views for primitive, tuple, record, variant, union, refined, algorithm,
  and opaque types;
- lossless dependent and recursive field structures;
- algorithm signatures including staticness, effects, and environment
  requirements;
- deterministic enumeration of visible scope members;
- declaration metadata already visible at the inspection site;
- typed static names, labels, paths, and identities;
- exact selected language revision and variant inspection;
- explicit layout inspection separate from semantic type inspection; and
- typed construction sufficient to derive algorithms from inspected types.

It should not initially provide:

- runtime type discovery;
- dynamic invocation by textual name;
- automatic reflection metadata retention;
- unrestricted source abstract syntax trees or source rewriting;
- private implementation or declaration inspection;
- compiler intermediate representation or optimization inspection; or
- target- or compiler-version-dependent program behavior.
