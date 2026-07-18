# Number model

This document records the provisional number model for Topal. Numeric types
describe mathematical or algebraic behavior, constraints restrict values, and
encodings describe storage. The compiler may select efficient machine
representations whenever it proves that observable semantics are preserved.

## Exact integers

`Int` is an arbitrary-precision mathematical integer. It has no intrinsic
minimum, maximum, overflow, or storage width:

```topal
Int
```

`Nat` is its nonnegative refinement:

```topal
Nat is >= 0 Int
```

The names provide the familiar signed/nonnegative distinction, but neither type
implies a sign bit or fixed storage. `Nat` uses mathematical arithmetic rather
than machine-style unsigned overflow.

Operations derive the strongest practical result constraints. For example,
subtracting two `Nat` values produces `Int` unless their ordering proves that
the result remains nonnegative. Adding two values in `0 .. 255` produces a value
in `0 .. 510`, not another byte-sized value.

The compiler may store a proven-small `Int` or `Nat` in a machine register and
promote it when required. This is an implementation decision and cannot change
numeric results.

## Ranges and constrained integers

Ranges refine exact numbers without changing their arithmetic:

```topal
ByteValue is ( >= 0 and <= 255 ) Nat
Percentage is ( >= 0 and <= 100 ) Nat
Temperature is ( >= -273 ) Int
```

If an exact operation produces a result outside a refinement, the result still
exists as `Int` or `Nat`; assigning it back to the refined type requires proof
or explicit validation. Constraints never silently truncate, saturate, or wrap.

Ranges are specialized convex predicates rather than numeric containers. Their
general construction, open and unbounded bounds, and relationship to collection
selection are described in [the range model](ranges.md).

## Modular integers

Wrapping changes the meaning of arithmetic and therefore belongs to a modular
numeric type rather than an ordinary constraint or effect:

```topal
ModNat range
ModInt range
```

The finite, contiguous range selects canonical representatives and determines
the modulus. It must contain zero. A `ModNat` range begins at zero, while a
`ModInt` range may include negative representatives:

```topal
ByteCounter is ModNat ( 0 .. 255 )
SignedByte is ModInt ( -128 .. 127 )
ClockHour is ModNat ( 0 .. 23 )
```

Examples:

```text
ByteCounter 255 + ByteCounter 1 = ByteCounter 0
SignedByte 127 + SignedByte 1   = SignedByte -128
ClockHour 23 + ClockHour 2      = ClockHour 1
```

`ByteCounter` and `SignedByte` both have 256 values and arithmetic modulo 256.
They differ in their canonical representatives, comparison, display, and
conversion to `Int`:

```text
residue     ModNat 0..255     ModInt -128..127
0           0                 0
127         127               127
128         128               -128
255         255               -1
```

A modular range is a parameter of `ModNat` or `ModInt`, not a refinement applied
afterward. This distinction preserves the rule that constraints restrict values
while types determine what operations mean:

```topal
ByteRange is 0 .. 255
ByteValue is ByteRange Nat
ByteCounter is ModNat ByteRange
```

`ByteValue 255 + ByteValue 1` produces the exact value `256` outside
`ByteValue`. `ByteCounter 255 + ByteCounter 1` produces `ByteCounter 0`.

## Modular construction

Validation and modular reduction are separate operations. Checked construction
rejects values outside the canonical range:

```topal
ByteCounter value
```

and conceptually returns:

```topal
Result ByteCounter OutOfRange
```

Explicit modular construction always reduces into the range:

```topal
value modulo ByteCounter
```

For an eight-bit `ModNat`:

```text
256 -> 0
257 -> 1
-1  -> 255
```

Static literals that fit can be checked during compilation.

## Bit operations

Arbitrary modular arithmetic does not always have a natural bitwise model.
`ModNat ( 0 .. 9 )` has ten values even though four bits have sixteen patterns.

Bitwise and rotation operations are available when the compiler has evidence
that the modulus is a power of two and the type has a corresponding semantic bit
width:

```topal
bit-and
bit-or
bit-xor
bit-not
shift-left
shift-right
rotate-left
rotate-right
```

This makes machine words expressible as power-of-two `ModNat` or conventional
`ModInt` ranges without claiming that every modular number is a bit vector.

Signed and nonnegative forms have identical modular addition, subtraction,
multiplication, and bit patterns when they share a power-of-two modulus. Their
interpretations differ for comparison, division, remainder, arithmetic shift,
display, and conversion to exact integers.

## Ordering modular values

Ordinary comparison uses canonical representatives. Thus `-1 < 1` is true for
a `ModInt`, while the corresponding `ModNat` residue `255` is greater than `1`.

Residues are fundamentally cyclic and have no intrinsic linear order. Protocols
using wrapping sequence numbers or counters need an explicit cyclic comparison
with a valid maximum distance rather than overloading ordinary `<`:

```topal
left cyclic-before right
```

## Exact non-integers

Topal should distinguish exact domains from finite approximations rather than
using one `Float` type for all non-integers.

`Rational` represents an exact ratio of arbitrary-precision integers:

```topal
Rational ( 1 , 3 )
```

Its addition and multiplication are exact, associative, and commutative where
the corresponding mathematical operations have those laws. Numerators and
denominators may grow and therefore consume increasing resources.

`Decimal` represents an exact finite decimal value. Decimal literals are exact
by default:

```topal
0.1
12.50
1.25e3
```

Finite decimal addition and multiplication can remain exact. Division may
produce a `Rational`, require an explicit approximation, or report that no
finite decimal result exists; the precise division API remains undecided.

## Approximate numbers

`Approx` is the provisional name for finite-precision approximate values. It is
more explicit about semantics than `Float`, while named IEEE formats remain
available for storage and interoperability.

An approximate type may include a radix, precision, rounding rule, and special
value policy:

```topal
Approx (
  radix 2 ,
  precision 53 ,
  rounding NearestEven
)
```

Aliases can describe standard formats:

```topal
Binary32
Binary64
Decimal128
```

Exact literals do not silently become binary approximations. Conversion is
explicit:

```topal
0.1 approximate Binary64
```

The exact spelling remains provisional.

## Precision and arithmetic laws

Finite precision alone does not make approximate addition associative. Rounding
after each operation can make these produce different results:

```topal
( a + b ) + c
a + ( b + c )
c + b + a
```

Increasing precision reduces error but cannot remove this property for all
inputs. IEEE special values such as NaN, infinities, and negative zero add
further algebraic distinctions.

Topal therefore associates algebraic law evidence with operations and operand
types:

```text
Int addition             associative, commutative, identity 0
Rational addition        associative, commutative, identity 0
modular addition         associative, commutative, identity 0
ordinary Approx addition deterministic only for a defined evaluation order
```

The compiler may reorder or parallelize a reduction only when the selected
operation provides the necessary laws.

## Reproducible approximate sums

Order-independent approximate results require semantics stronger than ordinary
finite-precision addition. Topal can support explicit alternatives:

- Accumulate exact intermediate values and round once at the requested boundary.
- Use a specified reproducible superaccumulator or binned summation algorithm.
- Preserve one canonical logical order even when execution is parallel.
- Return an interval or error-bearing approximation when enclosure matters more
  than identical representation.

Examples of distinct intent are:

```topal
values fold ( Binary64 0 , + )
values reproducible-sum Binary64
values exact-sum approximate Binary64
```

An ordered fold reproduces its declared order but is not invariant under source
reordering. `reproducible-sum` promises its documented permutation-independent
result. Exact accumulation followed by one rounding also gives an
order-independent sum, subject to its resource requirements.

## Arithmetic policies

When an isolated operation needs a policy different from its operand type,
algorithms make that policy explicit:

```topal
left checked-add right
left saturating-add ( right , bounds )
left wrapping-add ( right , modulus )
```

- `checked-add` returns an explicit error when a requested target range is
  exceeded.
- `saturating-add` clamps to a declared boundary.
- `wrapping-add` reduces modulo a declared modulus.

Repeated modular arithmetic should normally use `ModNat` or `ModInt`, making the
behavior visible in every value's type.

## Encodings and storage

Fixed-width, signedness, endianness, and signed bit encodings describe external
or requested representations, not the semantics of `Int` and `Nat`:

```topal
value encode (
  width 32 ,
  signed TwosComplement ,
  endian Little
)
```

Named formats can abbreviate common layouts:

```topal
UInt16BE
Int32LE
IeeeBinary32
IeeeBinary64
```

Encoding validates representability unless modular reduction or truncation is
explicitly requested. Two's-complement, one's-complement, sign-magnitude, and
biased encodings map semantic values to bit patterns but do not redefine their
arithmetic.

Arithmetic range, encoded width, and physical size remain distinct:

```text
range or modulus   numeric behavior and canonical values
encoding           external bit pattern, width, and byte order
storage-size       complete chosen representation including overhead
```

For example, `ModNat ( 0 .. 9 )` may be stored in a byte, packed into four bits,
or encoded as an ASCII digit without changing its modulo-10 arithmetic. A
`ModNat ( 0 .. 255 )` may occupy one byte in an encoded array and a full machine
register during computation.

As with strings, representation becomes observable only at an explicit storage,
encoding, hardware, or foreign-language boundary. The compiler otherwise has
freedom to choose and change representations.

## Provisional hierarchy

```topal
Int
Nat
Rational
Decimal
Approx specification

ModInt range
ModNat range

Range ( minimum , maximum )
```

The stable design principles are:

- Exact arithmetic is the default.
- `Nat` is a nonnegative exact integer, not a machine unsigned integer.
- Constraints restrict values without changing operations.
- `ModInt` and `ModNat` use ranges to define modular arithmetic and canonical
  representatives.
- Approximation, rounding, saturation, and wrapping are never implicit.
- Algebraic laws determine whether the compiler may reorder operations.
- Encodings and storage layouts remain boundary concerns.
