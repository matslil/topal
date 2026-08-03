# Layouts and addressed storage

This document records the provisional model for describing external storage and
memory-mapped I/O. Semantic values remain immutable. Layouts describe their
external representation, while address ranges, offsets, and locations describe
where that representation can be accessed. The compiler may use direct loads,
direct stores, lazy conversion, copying, or in-place updates when these choices
preserve the source semantics.

Attributes use the ordinary parenthesized `name is value` construction syntax.
Their schema comes from `AddressRange`, `AddressOffset`, `Layout T`, or the
selected encoding family. Curly braces are not used because they already
introduce short function bodies.

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

For example, `( caching is Uncached ) AddressRange ( 0 .. 255 )` constructs a
complete range directly. `AddressRange ( caching is Uncached )` instead
constructs a subtype which can be reused to construct several ranges. This
asymmetry keeps the complete expression attribute-first while leaving subtype
construction as an ordinary one-parameter application.

## Address ranges

An `AddressRange` value stores an ordinary `Nat .. Nat` range. Its subtype's
attribute map initially supports:

Schema tables in this document place a field name and its accepted classifier
in columns; they are not source expressions. Source constructions always place
`is` between the field name and value.

```text
caching             Cached | Uncached
minimum-access-size size in bits or bytes
medium              Memory | MMIO
```

For example:

```topal
DeviceAddresses is AddressRange (
  caching is Uncached,
  minimum-access-size is 32[b],
  medium is MMIO
)

device is DeviceAddresses (
  0x4000_0000 .. 0x4000_FFFF
)
```

`caching` describes the platform or hardware cache policy. It does not by itself
decide whether the compiler may reuse, omit, or combine storage access; that
follows from the range's medium and the semantics of the operation.

`medium` distinguishes stable memory, such as a protocol-packet buffer, from
memory-mapped I/O. A `Memory` range supports immutable layout-backed values and
the usual representation-preserving optimizations. An `MMIO` range exposes
fallible hardware reads and writes as observable effects.

`minimum-access-size` describes the smallest physical transaction supported by
the range. A layout may be smaller, but accessing it then requires a containing
transaction whose layout determines every accessed bit. The compiler must not
invent a read-modify-write operation when doing so could introduce an illegal
or observable read.

All location reads and writes within one `MMIO` address range occur in source
effect order. They cannot be duplicated, removed, combined, or reordered. The
initial model may conservatively preserve ordering between different MMIO
ranges as well; more precise cross-range ordering remains future work. Access
to a `Memory` range instead follows ordinary immutable value semantics, under
which the compiler may optimize storage access when the result is unchanged.

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
  range is device,
  alignment is 4
)

control-offset is DeviceOffset 0x20
```

Because offsets are measured in bytes, alignment is also measured in bytes and
the `bytes` name is optional. Thus `alignment is 4` means four-byte alignment.
Construction proves that the offset belongs to the associated range and
satisfies the alignment. Applying a layout later additionally proves that the
complete stored representation fits within the range.

The range identity remains part of the offset subtype. Offsets associated with
different range values are distinct even if their numeric values, bounds, and
attributes happen to match.

## Layouts

A layout describes the stored representation of an immutable semantic value.
The fields accepted by `Layout T` are determined by `T`; its attribute map is
not open. Every layout supports these common fields where meaningful:

```text
storage-size   complete size in bits or bytes
encoding       one encoding admitted by T
endian         Little | Big
access         ReadWrite | ReadOnly | WriteOnly | Reserved
alignment      required byte alignment
```

`access` defaults to `ReadWrite`. `storage-size` is required when width cannot
be derived and optional when the encoding or component layouts determine one
exact size. `alignment` defaults to one byte. `endian` is required when several
byte orders are possible and rejected for a one-byte or order-independent
representation. `encoding` is required unless the other fields and `T` admit
exactly one representation.

The same field name always has the meaning specified here. An encoding which
orders bits inside a storage unit uses `bit-order`; it does not give `endian` a
second meaning. Missing, unknown, inapplicable, or inconsistent fields are
compile errors.

The complete initial field vocabulary is indexed here. A field is defined once
in the referenced schema and reused by every listed type family:

| Field | Defined by |
| --- | --- |
| `storage-size`, `encoding`, `endian`, `access`, `alignment` | every applicable `Layout T` |
| `bit-order`, `unit-size`, `canonical`, `length` | shared encoding vocabulary |
| `false-pattern`, `true-pattern` | `BooleanBits` |
| `bias` | `BiasedBinary` |
| `numerator-layout`, `denominator-layout` | `Ratio` |
| `integer-layout`, `quantum` | `ScaledInteger` |
| `exponent-bits`, `fraction-bits`, `exponent-bias`, `subnormal`, `infinity`, `signed-zero`, `nan` | `IeeeBinary`, `IeeeDecimal` |
| `termination`, `padding` | text encodings |
| `packing`, `field-order`, component `layout` and `offset` | tuple and record layouts |
| `tag-layout`, `tags`, `payload-placement` | `Tagged` sums and enums |
| `element-layout`, `stride`, `entry-layout`, `ordering` | arrays and collections |
| `measurement-unit` | measured numeric layouts |

Attributes precede the semantic type:

```topal
UInt32LE is (
  storage-size is 32[b],
  encoding is UnsignedBinary,
  endian is Little,
  access is ReadWrite
) Layout Nat
```

The combination of `storage-size is 32[b]` and `encoding is UnsignedBinary`
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
  alignment is 4,
  access is ReadWrite
) Layout (
  kind is UInt8,
  length is UInt16BE,
  sequence is UInt32BE
)
```

The semantic product is derived from the semantic types provided by its field
layouts. `storage-size` is optional for a container layout: when absent, its
size is inferred from its field sizes, placement, padding, and alignment. When
present, the declared size is checked against the derived layout. Container
alignment and packing attributes determine field placement as specified below.

## Encoding construction and validation

An encoding is a typed mapping between semantic values and stored bit patterns.
The layout's right-hand type determines which encoding families are admitted;
the selected family then determines its closed set of fields. Encoding fields
do not repeat total size, byte order, alignment, or access.

Reading validates both the bit pattern and the semantic type. Writing validates
representability and emits the encoding's canonical pattern. Neither operation
silently truncates, wraps, saturates, rounds, normalizes, or substitutes a value
unless the selected encoding explicitly says so. A field uniquely derivable
from the semantic type may be omitted; if supplied for clarity, it must agree.

The shared encoding fields are:

```text
bit-order       MostSignificantFirst | LeastSignificantFirst
unit-size       size of one encoded storage unit in bits
canonical       family-specific choice when several patterns decode equally
length          NoLength | Prefix ( Layout Nat ) | Fixed Nat | Remainder
```

`bit-order` orders bits inside each `unit-size`; layout `endian` orders multiple
units. `canonical` is available only for a family with equivalent patterns.
Reads may accept every valid pattern, while writes always use the canonical one.
`length` always describes encoded entries or code units, never bytes.
`NoLength` is available when another rule determines the boundary; `Prefix`
stores the count first, `Fixed` declares it statically, and `Remainder` uses a
containing fixed-size representation's complete remainder.

## Fundamental scalar schemas

`Layout Unit` has inferred `storage-size is 0[b]` and encoding `Empty`. It may be
a zero-size component but cannot independently name an addressable location.

`Layout Boolean` admits `BooleanBits` with these required fields:

```text
false-pattern   Bits width
true-pattern    Bits width
```

The distinct patterns must match `storage-size`; every other pattern is
invalid. Boolean storage is therefore not assumed to be numeric zero and one.

`Layout ( Bits width )` admits `RawBits`; `storage-size` is exactly `width` and
is inferred. `Byte` is the `Bits 8` storage unit. `Bytes` uses the sequence
schema below. Endianness never reverses uninterpreted bits or bytes.

### Integer encodings

`Layout Nat`, `Layout Int`, constrained integers, `ModNat`, and `ModInt` admit
the applicable families:

```text
UnsignedBinary
TwosComplement
OnesComplement ( canonical is PositiveZero )
OnesComplement ( canonical is NegativeZero )
SignMagnitude ( canonical is PositiveZero )
SignMagnitude ( canonical is NegativeZero )
BiasedBinary ( bias is 127 )
```

`UnsignedBinary` requires nonnegative represented values. `BiasedBinary` stores
`value + bias` as unsigned binary. The other named families have their ordinary
signed meanings. Width comes from `storage-size`. A pattern outside a semantic
constraint is invalid, and a value outside the encoded range cannot be written.
Modular values use canonical representatives; layout conversion never performs
modular reduction and unused bit patterns remain invalid.

These integer encoding families represent only finite values. Reading them
therefore always produces `Finite` evidence; writing an unconstrained `Int` or
`Nat` fails when its value is infinite. A layout may state `Finite Int` or
`Finite Nat` when rejecting infinity should be visible in its semantic subtype.

### Rational and fixed point encodings

`Layout Rational` admits `Ratio`:

```text
numerator-layout     Layout Int
denominator-layout   Layout Nat
canonical            ReducedPositiveDenominator
```

Both component layouts are required. Zero denominators are invalid. The
canonical policy requires a positive denominator and relatively prime parts;
reads reject alternatives and writes emit that unique representation.
`Ratio` represents finite rationals only, so a successful read carries `Finite`
evidence and writing a rational infinity fails.

A layout of a `FixedPoint` type admits `ScaledInteger`:

```text
integer-layout       Layout Int | Layout Nat
quantum              positive Rational
```

The integer layout is required. `quantum` is inferred from the fixed-point type
and, when written explicitly, must match it. Stored integer times quantum is the
semantic value. No rounding is implied.
`ScaledInteger` represents finite fixed-point values only. As with integer and
ratio layouts, a successful read proves `Finite` and an infinity cannot be
written.

### Approximate encodings

An approximate type admits `IeeeBinary` or `IeeeDecimal` when compatible with
its radix and arithmetic policy:

```text
exponent-bits       positive Nat
fraction-bits       Nat
exponent-bias       Nat
subnormal           Gradual | Absent
infinity            Signed
signed-zero         Preserve | Discard
nan                  Invalid
```

Widths and bias are required unless uniquely derived from the approximate type
and total size. The policy fields must agree with that type. `nan` is currently
fixed to `Invalid`: raw `Bits` can preserve a NaN pattern, but decoding it as a
number fails. Approximate types contain signed infinities, so an IEEE encoding
must represent them. A layout of `Finite ApproxType` uses the same encoding but
rejects infinity patterns on read and infinite values on write. Signed zero
becomes ordinary zero plus direction evidence when preserved.

Layouts do not round exact values. An approximate value is produced through
exact checked type construction or an explicitly named rounding function before
the layout stores it. Layout interpretation only validates and stores that
already constructed value.

For example, assuming `Binary64` names the corresponding semantic `Approx`
policy, its little-endian IEEE layout is:

```topal
IeeeBinary64LE is (
  storage-size is 64[b],
  encoding is IeeeBinary (
    exponent-bits is 11,
    fraction-bits is 52,
    exponent-bias is 1023,
    subnormal is Gradual,
    infinity is Signed,
    signed-zero is Preserve,
    nan is Invalid
  ),
  endian is Little
) Layout Binary64
```

## Text schemas

`Layout String` and `Layout Character` admit `Utf8`, `Utf16`, `Utf32`, their
normalization variants, and `Ascii`, as defined in
[the string model](strings.md#encodings-and-storage-formats). `Character`
additionally validates that decoding produces one semantic character.

Text encodings accept:

```text
length              NoLength | Prefix ( Layout Nat ) | Fixed Nat | Remainder
termination         NoTerminator | CodeUnit value
padding             NoPadding | CodeUnit value
```

Length defaults to `NoLength`, and termination defaults to `NoTerminator`, but
fixed storage size, a nonzero fixed length, a length prefix, a remainder, or a
terminator must make the boundary knowable. Short content in fixed storage
requires padding, which is not decoded as text. A terminator or padding value
must be one representable code unit and cannot also occur as unescaped content.

`Utf16` and `Utf32` require layout `endian`; `Utf8` and `Ascii` reject it.
Normalization variants normalize on write and validate on read. Malformed,
truncated, unterminated, incorrectly padded, or normalization-inconsistent text
fails validation. Layouts of `Encoded E` and `Bytes` use sequence framing and
do not decode their payload.

## Product schemas

Tuple and record layouts contain a layout for every component and accept:

```text
packing             Natural | Packed
field-order         Declared | explicit list of all field identities
```

Natural packing is the default and inserts minimum alignment padding. `Packed`
inserts none and is invalid when a component cannot safely be accessed at its
resulting alignment. Declared field order is the default. Each component
association accepts a layout directly, as in `kind is UInt8`, or an expanded
entry with these fields:

```text
layout              layout of the component's semantic type
offset              size in bits or bytes from the product start
```

`layout` is required in the expanded form and `offset` is optional:

```topal
HeaderLayout is (
  packing is Packed
) Layout (
  kind is UInt8,
  length is (
    layout is UInt16BE,
    offset is 2[B]
  )
)
```

An explicit offset overrides inferred placement. Components cannot overlap.
Gaps are reserved padding with the preservation semantics described above. A
product `storage-size`, when supplied, describes the complete representation
and must contain every component.

## Sum and enum schemas

Variants, unions, and enums admit `Tagged`:

```text
tag-layout          Layout Nat | Layout Int | Layout Bits
tags                complete alternative-identity to tag-value map
payload-placement   AfterTag | Overlay
```

The first two fields are required. Tag values must be distinct and representable;
unassigned patterns are invalid. Payload placement defaults to `AfterTag`.
`Overlay` gives every alternative the same aligned payload position. Each
alternative supplies its payload layout; a `Unit` alternative occupies no
payload. Total size covers the tag and largest alternative.

`Optional T` and `Result ( T, Codes )` use this same sum schema; they do
not introduce private tag fields. Their alternatives and payload layouts are
selected in the ordinary `tags` map.

## Sequence and collection schemas

`Array count T` may repeat one homogeneous layout:

```text
element-layout      Layout T
stride              size in bits or bytes
```

Stride defaults to the aligned element size. It may reserve more space but may
not overlap the following element or violate alignment.

Variable-size `List`, `Bytes`, `Encoded E`, `Set`, `Bag`, and `Map` layouts use:

```text
length              NoLength | Prefix ( Layout Nat ) | Fixed Nat | Remainder
element-layout      Layout T
entry-layout        product layout
ordering            Preserved | Canonical
```

`NoLength` is rejected for these variable-size collections. Homogeneous
collections require `element-layout`; maps require an `entry-layout`
containing key and value. Exactly one is present. `length` is required unless a
containing fixed-size representation supplies an unambiguous remainder.
Sequences infer `Preserved`. Unordered collections require `Canonical`, using
the total encoded-bit order of complete entries so equal semantic values have
one representation. Duplicate or noncanonical set/map entries and
noncanonical bag order are invalid on read.

## Constraints, quantities, and nominal types

A constrained value uses a layout admitted by its base type and validates the
constraint on read. A nominal type may reuse its base layout only when its
declaration publishes that representation; layouts cannot bypass abstraction.

A measured numeric quantity uses an applicable numeric layout plus:

```text
measurement-unit    compatible MeasurementUnit
```

The required field selects the stored unit. Conversion follows the semantic
numeric type's explicit rules. Affine points apply the unit offset; a
`Delta Dimension` uses its compiler-derived delta unit without that offset.

## Types without layouts

Functions, tasks, endpoints, resources, capabilities, environments, static
identifiers, types themselves, and opaque objects without a published value
representation do not admit `Layout T`. Native Topal serialization may still
describe them, but a portable byte layout cannot reconstruct authority,
behavior, compiler identity, or private state. This separates
[native serialization](serialization.md) from programmer-selected external
representation.

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
added when a function needs the concrete address.

## Reading and writing

MMIO uses ordinary fallible operations whose types are determined by the
location's layout:

```topal
value : UInt32LE is read control
control write value
```

Conceptually:

```text
read  : Location L -> Result ( L, LayoutErrorCode ) where L is a readable Layout T
write : ( Location L, L ) -> Result ( Unit, LayoutErrorCode ) where L is a writable Layout T
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

## Future foreign boundary declarations

A future foreign declaration would belong to a sandbox adapter and associate
an external symbol or callback entry inside that sandbox with a declared
boundary protocol. It would explicitly declare:

- the sandbox, ABI, and external symbol identity;
- a layout for every externally represented input and output;
- copied or serialized ownership and destruction behavior;
- explicitly granted resource capabilities;
- fallibility and error translation;
- whether the sandbox operation may suspend or send replies; and
- the task protocol through which each sandbox message enters Topal.

Layout decoding, constrained integer construction, text decoding, and protocol
validation occur at the sandbox boundary and return `Result` when external data
may be invalid. Foreign code receives no borrowed Topal value, raw continuation,
task internals, unrestricted callback, or ambient process resource.

The adapter exposes only copied or serialized values and explicitly granted
handles. Effects on a granted file, device, endpoint, or other resource retain
that identity in Topal's dependency graph. Programmer claims can add semantic
or optimization evidence but cannot bypass the sandbox or validation.

Foreign callbacks become declared sandbox messages delivered to a typed task
capability. They do not enter as arbitrary calls on an external thread.
Ordinary task isolation, termination, and effect ordering then apply. Future
language-specific adapters may establish stronger direct-call promises from
their own safety and interface systems.

Foreign integration, its declaration grammar, and its ABI catalogs are not part
of the current language commitment. ABI families may later belong to selected
language features rather than the portable bootstrap grammar.
