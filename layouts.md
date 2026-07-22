# Layouts and addressed storage

This document records the provisional model for describing external storage and
memory-mapped I/O. Semantic values remain immutable. Layouts describe their
external representation, while address ranges, offsets, and locations describe
where that representation can be accessed. The compiler may use direct loads,
direct stores, lazy conversion, copying, or in-place updates when these choices
preserve the source semantics.

The examples show attribute maps as parenthesized lists of named entries. The
exact map-literal syntax remains undecided. Curly braces are not used because
they already introduce short function bodies.

## Construction and reusable subtypes

Full construction places the attribute map first, followed by the constructor
and its ordinary value or semantic type. Reusable subtype construction instead
applies the constructor to the attribute map as its single parameter. The
resulting subtype then accepts the ordinary value or type:

```text
attributes AddressRange ( Nat .. Nat ) -> address-range value
AddressRange attributes                -> address-range subtype
address-range-subtype ( Nat .. Nat )   -> address-range value

attributes AddressOffset Nat          -> address-offset value
AddressOffset attributes              -> address-offset subtype
address-offset-subtype Nat             -> address-offset value

attributes Layout T                   -> layout subtype of Layout and T
Layout attributes                     -> reusable layout subtype
layout-subtype T                       -> layout subtype of Layout and T

Location layout                       -> location subtype
location-subtype AddressOffset         -> location value
```

For example, `( caching Uncached ) AddressRange ( 0 .. 255 )` constructs a
complete range directly. `AddressRange ( caching Uncached )` instead constructs
a subtype which can be reused to construct several ranges. This asymmetry keeps
the complete expression attribute-first while leaving subtype construction as
an ordinary one-parameter application.

## Address ranges

An `AddressRange` value stores an ordinary `Nat .. Nat` range. Its subtype's
attribute map initially supports:

```text
caching             Cached | Uncached
minimum-access-size size in bits or bytes
```

For example:

```topal
DeviceAddresses is AddressRange (
  caching Uncached ,
  minimum-access-size bits 32
)

device is DeviceAddresses (
  0x4000_0000 .. 0x4000_FFFF
)
```

`caching` describes the platform or hardware cache policy. It never permits the
compiler to reuse, omit, or combine explicit location reads or writes.

`minimum-access-size` describes the smallest physical transaction supported by
the range. A layout may be smaller, but accessing it then requires a containing
transaction whose layout determines every accessed bit. The compiler must not
invent a read-modify-write operation when doing so could introduce an illegal
or observable read.

All location reads and writes within one address range occur in source effect
order. They cannot be duplicated, removed, combined, or reordered. The initial
model may conservatively preserve ordering between different ranges as well;
more precise cross-range ordering remains future work.

## Address offsets

An `AddressOffset` value stores a `Nat` byte offset. Its subtype's attribute map
initially supports:

```text
range       AddressRange value
alignment   byte count
```

For example:

```topal
DeviceOffset is AddressOffset (
  range device ,
  alignment 4
)

control-offset is DeviceOffset 0x20
```

Because offsets are measured in bytes, alignment is also measured in bytes and
the `bytes` name is optional. Thus `alignment 4` means four-byte alignment.
Construction proves that the offset belongs to the associated range and
satisfies the alignment. Applying a layout later additionally proves that the
complete stored representation fits within the range.

The range identity remains part of the offset subtype. Offsets associated with
different range values are distinct even if their numeric values, bounds, and
attributes happen to match.

## Layouts

A layout describes the stored representation of an immutable semantic value.
Its attribute map initially supports:

```text
storage-size   size in bits or bytes
encoding       encoding accepted by the semantic type
endian         Little | Big, when applicable
access         ReadWrite | ReadOnly | WriteOnly | Reserved
alignment      byte count, for container layouts
```

Attributes precede the semantic type:

```topal
UInt32LE is (
  storage-size bits 32 ,
  encoding UnsignedBinary ,
  endian Little ,
  access ReadWrite
) Layout Nat
```

The combination of `storage-size bits 32` and `encoding UnsignedBinary`
specifies the 32-bit unsigned binary representation. Width is not duplicated in
the encoding name. Similarly, a signed integer layout can combine a storage
size with `TwosComplement`, and an approximate-number layout can combine one
with an IEEE encoding.

Storage size describes the complete stored value. For a fixed-width integer it
therefore selects the integer width; for a string it describes the space for
the whole encoded string rather than the width of one code unit.

Endianness is independent of encoding. It is absent when an encoding or a
one-byte representation has no byte-order choice.

`Layout T` is both a subtype of `Layout` and a subtype of `T`. Ordinary Topal
code therefore observes the immutable semantics of `T`, while the compiler
retains layout evidence for direct access, lazy endian conversion, and safe
in-place optimization. Casting a layout value `as T` forgets the layout
evidence without changing the semantic value.

Access controls which location operations are available. `Reserved` is only for
padding within a container layout. Its bits are not exposed as a semantic field
and ordinary code cannot inspect or change them. Reading a container preserves
the bits as hidden layout evidence; writing that layout value back emits the
same bits. Functional updates to accessible fields preserve them as well.

`Reserved` does not declare an expected read value or a value to manufacture on
write. A newly constructed semantic container consequently lacks the hidden
bits needed to construct that complete layout value. It can only be written
after combining it with an existing layout value whose padding can be
preserved. An in-place update may simply leave those bits untouched. If a
format requires validation, clearing, setting, or any behavior other than
preservation, the programmer exposes an accessible field and implements that
policy explicitly.

Container layouts contain layouts rather than plain semantic field types. The
attribute map still precedes the contained layout:

```topal
HeaderLayout is (
  storage-size bytes 8 ,
  alignment 4 ,
  access ReadWrite
) Layout (
  kind UInt8 ,
  length UInt16BE ,
  sequence UInt32BE
)
```

The semantic product is derived from the semantic types provided by its field
layouts. Container alignment and packing attributes determine field placement.
The precise field-map and packing syntax remains to be designed; ordinary
parentheses remain available wherever explicit grouping is required.

## Locations

Applying `Location` to a layout constructs a location subtype. Applying that
subtype to an address offset constructs a location value:

```topal
ControlLocation is Location UInt32LE
control is ControlLocation control-offset
```

Construction checks that the layout fits after the offset, that the offset's
alignment satisfies the layout, and that its address range supports the
required physical access size. A location consequently carries all information
needed to find and interpret its storage:

```text
address range + byte offset + layout
```

The absolute address remains derivable rather than separately stored. The range
start and numeric offset can be recovered through their declared `as` views and
added when an algorithm needs the concrete address.

## Reading and writing

MMIO uses ordinary fallible operations whose types are determined by the
location's layout:

```topal
value : UInt32LE is read control
control write value
```

Conceptually:

```text
read  : Location L -> Result L where L is a readable Layout T
write : ( Location L, L ) -> Result Unit where L is a writable Layout T
```

Because `L` is a subtype of `T`, a caller which only needs the semantic value
may instead classify the read result as `T`. Retaining `L` also retains the
specific storage-size, encoding, endian, access, and any opaque padding
evidence. A semantic `T` can be encoded or validated to produce `L` when the
layout has no unavailable reserved bits.

Both operations are fallible because the hardware transaction may fail
independently of static bounds and representation checks. A read additionally
may receive bits which the layout cannot decode, while a write may receive a
semantic value which its layout cannot represent.

Each invocation is an observable ordered effect. Even for a cached range, two
source-level reads perform two hardware reads. Immutable values returned by
earlier reads remain valid snapshots, but do not replace later reads.

For ordinary packet or memory storage, the same location information lets the
compiler access fields without materializing a complete decoded container. A
functional update may overwrite the encoded storage in place when uniqueness,
lifetime, and alias analysis prove that the previous immutable value is no
longer observable.
