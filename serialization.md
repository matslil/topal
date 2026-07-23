# Serialization

Topal provides a native binary serialization protocol for every object which
an algorithm may return. The protocol is designed for inexpensive tracing:
type structure and representation properties are written once in a stream
header, while events contain only a compact type identity and their value.

Serialization records an object's semantic representation. It does not promise
that another execution can recreate the same object. Deserialization decides,
using the receiving language implementation and safety context, whether to
construct the represented object or a generic description of it.

External trace and interchange formats are encodings of a serialization stream,
not alternative Topal serialization protocols. Only the native format can be
passed to `lang deserialize`.

## Version values

`Version` is a general language-provided type used for language, package,
protocol, schema, and application versions. A literal begins with `v`:

```topal
v1.5
v1.5.2
v1.5.2-7
```

The complete value contains four natural-number components:

```topal
Version is Record
  major : Nat
  minor : Nat
  patch : Nat
  build : Nat
```

Omitted components are zero:

```text
v1.5     = v1.5.0-0
v1.5.2   = v1.5.2-0
```

Versions compare lexicographically by major, minor, patch, and build. This
numeric type deliberately does not admit textual prerelease or build
identifiers.

`lang version` is the version of the Topal language context active at that
source location. A later `use lang topal` selection changes `lang version` only
for following declarations.

## Serialization operation

`lang serialize` is a qualified language-provided binary function. Its left
operand selects the Topal version whose semantic vocabulary will describe the
stream, and its right operand supplies the object:

```topal
stream is v1.3 (lang serialize) my-variable
```

Conceptually:

```topal
lang serialize :
  fn ( Version, T ) -> SerializationStream
```

Normal partial application constructs a convenient local operation:

```topal
serialize is v1.3 (lang serialize)
first is serialize first-value
second is serialize second-value
```

Tracing normally targets the language context of the tracing source:

```topal
serialize is lang version (lang serialize)
trace-stream is serialize events
```

Communication with a receiver supporting an older version selects it
explicitly:

```topal
serialize-for-peer is v1.3 (lang serialize)
message is serialize-for-peer request
```

The version is the left operand rather than an optional property of the output.
This makes partial application unambiguous even when the object on the right is
itself a `Version`:

```topal
stream is v1.3 (lang serialize) v2.0
```

The compiler derives serialization from the statically visible semantic
structure of the input. When both the object type and target version are known,
an object which cannot be expressed by the target version is a compile-time
error. A stream containing dynamically typed objects reports an unsupported
object through its ordinary stream failure behavior.

Feature activation at the serialization site is irrelevant to the wire format.
Feature-defined types have unique semantic identities. Serialization succeeds
when the target language version defines representations for every identity
used by the object; the stream does not record which source features happened
to be active in the producer.

## Serialization streams

`SerializationStream` is an immutable stream-producing object, not an
eagerly materialized `Bytes` value. It permits a consumer to process headers,
type definitions, and values incrementally. An implementation may retain a
compact buffer, generate bytes on demand, or fuse the producer with its
consumer without changing the program's meaning.

A logical stream begins with:

```text
serialization protocol version
language identity
target language Version
producer-native byte order
type definitions
```

It is followed by events:

```text
event type identity
encoded value
```

The protocol version specifies the binary framing and schema notation. The
language version specifies the semantic vocabulary represented by those
schemas. They are independent: several Topal versions may use the same protocol
version, and a later protocol version need not change object semantics.

All type definitions needed by a finite statically known trace occur before its
events. A physical tracing session may contain several streams, such as one per
processor, which share equivalent type definitions while retaining their own
stream headers and byte order.

### Native byte order

Serialization always writes applicable numeric representations in the
producer's native byte order. The programmer does not select an endian. The
stream header records the native order once, and a receiver swaps values when
its order differs.

This rule avoids byte reversal on the common tracing write path. It applies to
protocol numeric representations, not to byte sequences or explicitly
represented external layouts. Serializing the semantic numeric value observed
through a big-endian layout uses stream-native order; explicitly serializing
the layout's physical bytes preserves those bytes and describes their layout
separately.

## Type definitions

Type definitions describe each event and nested object once. They include the
information needed to traverse its values:

- object kind and stable semantic identity;
- primitive encoding, width, and signedness;
- record fields and their identities;
- tuple components;
- variant and union alternatives and their tags;
- sequence, set, and map element types;
- refinements required to validate reconstructed values; and
- identities of feature-defined semantic forms.

An event uses a compact reference to one of these definitions. Width,
signedness, byte order, field structure, and nested type identities are
therefore not repeated for every value.

The schema describes public semantics rather than compiler memory layouts.
Every returnable opaque object has at least a stable identity and a generic
descriptive representation; it may additionally publish the semantic state
needed for reconstruction. A compiler may change internal representations
without changing the native serialization contract for an immutable language
version.

## Objects and descriptions

Serialization applies to every object which can be returned by an algorithm.
This includes values and statically constructed objects such as `Type`. It also
includes returnable resources and capabilities, although their representations
do not grant the receiver equivalent authority.

An `Environment` is a composition context rather than an ordinary algorithm
result and consequently is not serialized as a returned object. A value
selected from an environment, such as an endpoint, may be returned and is
serialized according to its own type.

The serializer records the representation defined by the target language. It
does not classify that representation as reconstructable or descriptive. That
decision belongs exclusively to the receiving `lang deserialize`
implementation. A newer implementation may therefore reconstruct an object
from an older stream even when implementations contemporary with that stream
could only describe it.

Examples of likely outcomes include:

```text
immutable record       -> reconstructed record
structural Type        -> reconstructed Type
published nominal Type -> reconstructed Type when its identity resolves
File                    -> ObjectDescription
Endpoint P              -> ObjectDescription unless safely resolved
task capability         -> ObjectDescription
```

Deserialization must not manufacture authority merely because a representation
contains an external identity. Reopening a file or resolving an endpoint is a
separate effectful operation requiring an explicit receiving capability.

## Generic descriptions

When an object cannot be reconstructed safely, deserialization produces an
`ObjectDescription`. A description is a map describing the serialized object's
type and data:

```topal
ObjectDescription is Map ( Label, Object )
```

Map entries may contain scalars, lists, maps, type identities, and other
ordinary objects as appropriate. For example:

```topal
(
  kind -> RecordType ,
  identity -> "example.Person" ,
  fields -> [
    ( label -> "name", type -> String ),
    ( label -> "age", type -> Nat )
  ] ,
  value -> (
    name -> "Ada" ,
    age -> 37
  )
)
```

The generic map representation is intended primarily for traversal,
diagnostics, and printing. It removes the need for a distinct unknown-object
case: any schema understood by the native protocol can be exposed as a map even
when the receiver does not know how to construct its original semantic type.

An implementation may present the map lazily over the immutable stream rather
than allocating a complete tree. Type identities and unknown fields must be
preserved so that a later implementation or explicitly loaded resolver can
attempt a richer interpretation.

## Deserialization

Only a native `SerializationStream` is accepted:

```topal
object is lang deserialize stream
```

The receiver validates the complete header before consuming events:

1. it supports the serialization protocol version;
2. it understands the declared Topal language version;
3. it can parse every type definition; and
4. declared sizes and resource requirements are acceptable.

An unsupported protocol or language version rejects the stream at the header
boundary. `ObjectDescription` is not a substitute for a schema language the
receiver cannot parse.

For each event, the deserializer resolves its type definition and produces the
best safe interpretation available in the receiving implementation:

```text
recognized and safely reconstructable -> original Object
otherwise                             -> ObjectDescription
```

The generic result belongs to the top `Object` classification because an
`ObjectDescription` is itself an object. A typed convenience may require a
specific reconstructed result and fail when only a description is available.

## External encoding

CTF, JSON, Protobuf, Google trace events, and other external formats are
produced by a separate `encode` step:

```topal
serialize is lang version (lang serialize)
native is serialize events
ctf is native encode CTF
json is native encode JSON
```

These encodings are intended for storage, transport, or their existing tooling.
They are not accepted by `lang deserialize` and need not preserve every Topal
object well enough for reconstruction.

The conceptual separation does not require an intermediate native byte stream.
For example, a CTF encoder can consume the serialization schema, emit valid CTF
metadata, and write event payloads directly in native byte order. The compiler
may fuse serialization and encoding so the tracing path performs no redundant
schema construction, allocation, copying, or endian conversion.

An external encoder reports when its target format cannot represent a required
semantic form. Format-specific projection, clock definitions, packet framing,
compression, checksums, and transport policy belong to the encoder rather than
the native serialization semantics.

## Initial scope

The first serialization revision should define:

- the numeric `Version` type and `v` literals;
- version-first `lang serialize` and `lang deserialize`;
- native-endian stream headers and type tables;
- primitive, tuple, record, variant, union, refined, and finite-container
  representations;
- at least a descriptive representation for every other returnable object kind;
- object and feature identities stable within immutable language versions;
- generic map-based `ObjectDescription`;
- incremental stream production and consumption;
- deterministic resource-limit and malformed-input errors; and
- a representation interface from which a CTF encoder can write directly.

It need not initially define:

- executable closure reconstruction;
- recreation of external authority;
- automatic endpoint or resource resolution;
- random-access indexes;
- compression or integrity policies;
- a complete CTF encoder; or
- mappings to every external trace and interchange format.
