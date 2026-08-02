# Number model

This document records the provisional number model for Topal. Numeric types
describe mathematical or algebraic behavior, constraints restrict values, and
encodings describe storage. The compiler may select efficient machine
representations whenever it proves that observable semantics are preserved.
Numbers may be combined with dimensioned units to form values described by the
[quantity and unit model](units.md).

## Exact integers

`Int` contains arbitrary-precision mathematical integers together with
`-Infinity` and `+Infinity`. Its finite values have no intrinsic bound, overflow,
or storage width:

```topal
Int
```

Integer literal radix prefixes, optional digit grouping, and the
whitespace-sensitive spelling of negative literals are defined in
[the numeric literal syntax](syntax.md#numeric-literals).

`Nat` is its nonnegative refinement:

```topal
Nat is >= 0 Int
```

The names provide the familiar signed/nonnegative distinction, but neither type
implies a sign bit or fixed storage. The constraint excludes `-Infinity` and
retains `+Infinity`. Finite `Nat` values use mathematical arithmetic rather than
machine-style unsigned overflow.

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
Temperature is ( Finite and >= -273 ) Int
```

If an exact operation produces a result outside a refinement, the result still
exists as `Int` or `Nat`; assigning it back to the refined type requires proof
or explicit validation. Constraints never silently truncate, saturate, or wrap.
Any finite upper or lower bound excludes the infinity beyond that bound. A
one-sided constraint otherwise retains the infinity on its unbounded side;
combine it with `Finite` when that value is not intended.

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
Result ( ByteCounter, ArithmeticErrorCode )
```

Failure uses the numeric construction domain's `out-of-range` error code.

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

## Fixed-width bits

`Bits width` is a fixed-width sequence of bits. The width is a positive `Nat`
known statically as part of the type:

```topal
Bits 1
Bits 8
Bits 32
```

Unlike `Int`, `Nat`, `ModInt`, and `ModNat`, `Bits` has no intrinsic numeric
interpretation. A value of `Bits 32` is the same sequence whether an external
layout later interprets it as an unsigned integer, a two's-complement signed
integer, an IEEE floating-point number, or four encoded characters. Numeric
conversion or interpretation is consequently explicit.

The fundamental pairwise bit operations require equal widths and preserve that
width:

```text
bit-and : ( Bits width , Bits width ) -> Bits width
bit-or  : ( Bits width , Bits width ) -> Bits width
bit-xor : ( Bits width , Bits width ) -> Bits width
bit-not : Bits width -> Bits width
```

Their provisional application syntax is:

```topal
left bit-and right
left bit-or right
left bit-xor right
bit-not value
```

The `bit-` prefix distinguishes these operations from logical predicates and
other uses of `and` and `or`. `bit-xor` follows the same naming scheme even
where an unprefixed `xor` would not otherwise be ambiguous.

Logical shifts retain the width, discard bits shifted out of the value, and
insert zero bits. A shift by the width or more produces all zero bits:

```text
shift-left  : ( Bits width , Nat ) -> Bits width
shift-right : ( Bits width , Nat ) -> Bits width
```

```topal
value shift-left count
value shift-right count
```

`Bits` does not provide an arithmetic right shift because it has no sign. An
encoded signed number must be decoded to an appropriate numeric type, or use an
explicit operation which supplies the signed interpretation.

Rotations also retain the width, but wrap bits around instead of discarding
them. Rotation counts are reduced modulo the width:

```text
rotate-left  : ( Bits width , Nat ) -> Bits width
rotate-right : ( Bits width , Nat ) -> Bits width
```

```topal
value rotate-left count
value rotate-right count
```

The fixed width makes every result well-defined. In contrast, applying
`bit-not`, a shift, or a rotation directly to arbitrary-precision `Int` or
`Nat` would require an otherwise unobservable choice of width. Arbitrary
modular arithmetic also does not imply a bit representation: a
`ModNat ( 0 .. 9 )` has ten values while four bits have sixteen patterns. Code
which needs both numeric and bit operations converts explicitly between the
numeric value and a `Bits` value using a chosen representation.

## Ordering modular values

Ordinary comparison uses canonical representatives. Thus `-1 < 1` is true for
a `ModInt`, while the corresponding `ModNat` residue `255` is greater than `1`.

Residues are fundamentally cyclic and have no intrinsic linear order. Protocols
using wrapping sequence numbers or counters need an explicit cyclic comparison
with a valid maximum distance rather than overloading ordinary `<`:

```topal
left cyclic-before right
```

## Exact non-integers and fixed point

Topal should distinguish exact domains from finite approximations rather than
using one `Float` type for all non-integers.

The finite values of `Rational` are exact ratios of arbitrary-precision
integers; the type additionally contains signed infinities:

```topal
Rational ( 1 , 3 )
```

Finite addition and multiplication are exact, associative, and commutative.
Numerators and denominators may grow and therefore consume increasing
resources. Operations involving infinities follow the common rules below and
can be indeterminate.

Fractional decimal literals construct exact `Rational` values by default:

```topal
0.1     # Rational ( 1, 10 )
12.50   # Rational ( 25, 2 )
1.25e3  # Rational 1250
```

Their decimal point, exponent, and digit-grouping rules are defined in
[the numeric literal syntax](syntax.md#numeric-literals). Base-prefixed
literals are integers rather than alternate spellings of rational values.
Trailing fractional zeroes do not change numeric identity; formatting policy
chooses how many digits to display.

`FixedPoint` represents exact finite values on a statically declared scale and
also contains signed infinities. A finite value has an arbitrary-precision
integer coefficient semantically, so scale does not imply overflow or a machine
width. Its construction accepts exactly one of two scale forms. Conventional
radix fixed point declares the number of fractional digits, with radix ten as
the default:

```topal
CentAmount is FixedPoint (
  fractional-digits is 2
)

BinaryFraction is FixedPoint (
  radix is 2,
  fractional-digits is 8
)
```

The exact quantum is `1 / (radix ^ fractional-digits)`. `CentAmount` therefore
contains multiples of `1 / 100`, while `BinaryFraction` contains multiples of
`1 / 256`. `radix` must be at least two and `fractional-digits` is a `Nat`.

The alternative form names an arbitrary positive rational quantum directly:

```topal
NickelAmount is FixedPoint (
  quantum is 0.05
)
```

`quantum` is mutually exclusive with both `radix` and `fractional-digits`.
Construction and conversion require the value to be an exact integer multiple
of the resulting quantum:

```topal
price : CentAmount is 12.34  # Statically verified implicit conversion.
invalid : CentAmount is 12.345 # Compile error.
```

A dynamic finite rational becomes `Result ( FixedType, ArithmeticErrorCode )`
unless available constraints prove exact representability. A rational infinity
maps to the same fixed-point infinity without inspecting the quantum. Rounding
is never implicit:

```topal
price is amount round (
  to is CentAmount,
  using is NearestEven
)
```

Addition and subtraction of one complete fixed-point type remain in that type;
multiplication by an integer also preserves it. Multiplying fixed-point values
with quanta `A` and `B` may derive `FixedPoint ( quantum is A * B )`. Arithmetic
between incompatible fixed-point scales otherwise promotes to `Rational` unless
a statically proven lossless common fixed-point type is required by context.
Division always follows the exact rational rule described below.

Fixed point describes semantic granularity, not storage. Layouts independently
select encoded width, signedness, scale, and overflow validation. Currency is
also separate: applications combine a fixed-point amount with a currency type,
unit, or record rather than treating all two-place values as interchangeable.

## Infinity and finite constraints

Topal's ordinary ordered scalar numeric domains contain their applicable
infinities. This is part of the numeric type rather than a parallel `Extended`
family:

```text
Nat          finite nonnegative values and +Infinity
Int          finite integers, -Infinity, and +Infinity
Rational     finite ratios, -Infinity, and +Infinity
FixedPoint   finite multiples of its quantum, -Infinity, and +Infinity
Approx       finite approximations, -Infinity, and +Infinity
```

`Nat` excludes `-Infinity` through its existing nonnegative constraint.
`ModNat`, `ModInt`, and bounded numeric constraints remain inherently finite:
their definitions admit only their finite canonical values. A range with a
finite lower and upper bound likewise proves finiteness. `Bits` is not a numeric
domain and has no infinity.

`+Infinity` and `-Infinity` are language numeric constants. Their expected
numeric type determines the value; `-Infinity` cannot satisfy `Nat` or another
nonnegative constraint. Without sufficient context, the compiler requires an
explicit numeric classification rather than choosing one domain.

`Finite N` is the language-defined constraint family for a numeric type `N`. It
retains the complete underlying numeric type and operations but excludes both
infinities where present:

```topal
count : Finite Nat
balance : Finite Rational
sample : Finite Binary64
```

Constructing `Finite N` from a dynamic `N` validates the existing value and
returns the ordinary constraint-validation `Result`. Forgetting the constraint
is implicit and lossless. `Finite` is a value constraint, not a storage policy,
approximation mode, capability, or separate arithmetic implementation.

Operations preserve the strongest provable constraint. Addition, subtraction,
and multiplication of finite exact values remain finite. Exact division of
finite values by a proven nonzero finite divisor remains finite. Approximate
arithmetic may overflow to an infinity, so finite approximate operands do not
alone prove a finite result; the result retains `Finite` only when range and
policy analysis proves that overflow is impossible. Constraints and compiler
proofs apply the same rule to generic numeric code.

Arithmetic on infinities is defined only where the numeric domain gives an
unambiguous result. Adding a finite value to an infinity preserves that
infinity; multiplying an infinity by a nonzero value follows its sign; and
ordering places negative infinity below every finite value and positive
infinity above every finite value. Forms such as `0 * Infinity`,
`Infinity - Infinity` with equal signs, `Infinity / Infinity`, and `0 / 0` are
indeterminate. They report the numeric domain's `indeterminate` arithmetic error
rather than producing a NaN-like semantic value.

The former names `ExtendedNat`, `ExtendedInt`, `ExtendedRational`, and
`ExtendedApprox` are removed. Code which requires finite inputs says so with
`Finite`; code using the unconstrained numeric type accepts its infinities.

## Zero directionality

Exact numeric domains have one zero. `Int`, `Nat`, `Rational`, and `FixedPoint`
do not gain a second numeric value named `-0`. Topal may instead carry
directional evidence with a calculation:

```text
zero FromBelow
zero Exact
zero FromAbove
```

This evidence records the side from which a calculation reached zero. It is not
part of the underlying value's sign, equality, or hash, so ordinary zero retains
the algebraic laws of its numeric type. A spelling such as `-0` may denote zero
with `FromBelow` evidence where a directional value is expected; converting it
to a plain exact number intentionally discards that evidence.

Directionality can accompany calculations over basic numeric types. An
operation that reaches a singularity may then produce an infinite result in
that ordinary numeric domain:

```text
positive / zero FromAbove -> +Infinity
positive / zero FromBelow -> -Infinity
negative / zero FromAbove -> -Infinity
negative / zero FromBelow -> +Infinity
positive / zero Exact     -> DivisionByZero
zero Exact / zero Exact   -> Indeterminate
```

For the dense `Rational` domain, direction can describe an ordinary one-sided
limit. Integers and fixed-point domains are discrete, so direction on an `Int`,
`Nat`, or `FixedPoint` instead records calculation provenance, rounding, or an
extrapolation; it is not a claim that distinct values in that domain lie
arbitrarily close to zero. `Nat` calculations cannot approach zero from below
while remaining in the `Nat` domain.

Direction is useful at nonzero boundaries as well. Retaining `FromBelow` or
`FromAbove` on a calculation approaching `10` lets a later subtraction expose
the corresponding direction at zero. Treating it as calculation evidence
rather than a special zero representation makes this behavior general.

## Approximate numbers

`Approx` is the provisional name for finite-precision approximate arithmetic. It is
more explicit about semantics than `Float`, while named IEEE formats remain
available for storage and interoperability.

An approximate type declares its radix, significand precision, finite exponent
range, subnormal behavior, and rounding rule:

```topal
Approx (
  radix is 2 ,
  precision is 53 ,
  minimum-normal-exponent is -1022 ,
  maximum-exponent is 1023 ,
  subnormal is Gradual ,
  rounding is NearestEven
)
```

`precision` counts radix digits in the significand. The exponent fields bound
finite normal values, while `subnormal` is `Gradual` or `Absent`. Every `Approx`
type also contains signed infinities. A finite operation whose rounded magnitude
exceeds `maximum-exponent` produces the corresponding infinity; a tiny result
follows the declared subnormal and rounding policies. `Finite ApproxType`
excludes those infinity results as an ordinary constraint.

Aliases can describe standard formats:

```topal
Binary32
Binary64
IeeeDecimal128
```

Exact literals do not silently become binary approximations. Conversion is
explicit:

```topal
0.1 approximate Binary64
```

The exact spelling remains provisional.

NaN is not a value of `Approx`. An operation which would
produce NaN under IEEE arithmetic instead reports an `indeterminate` arithmetic
error at that operation. An IEEE encoding may contain a NaN bit pattern so it
can be preserved at an external boundary, but decoding that pattern as a
semantic approximate number fails validation. Raw encoding inspection may
still expose its classification and payload when interoperability requires it.

Finite-precision arithmetic may round a tiny negative value to zero. Its
negative polarity maps to the same `FromBelow` evidence used by exact and
symbolic calculations, rather than changing meaning according to physical
encoding. IEEE formats preserve and emit this evidence for interoperability;
an approximate type whose policy discards it does so explicitly.

## Precision and arithmetic laws

Finite precision alone does not make approximate addition associative. Rounding
after each operation can make these produce different results:

```topal
( a + b ) + c
a + ( b + c )
c + b + a
```

Increasing precision reduces error but cannot remove this property for all
inputs. Infinities and directional zero evidence add further algebraic
distinctions.

Topal therefore associates algebraic law evidence with operations and operand
types:

```text
Finite Int addition        associative, commutative, identity 0
Finite Rational addition   associative, commutative, identity 0
same-type finite FixedPoint addition associative, commutative, identity 0
modular addition           associative, commutative, identity 0
ordinary Approx addition   deterministic only for a defined evaluation order
```

The compiler may reorder or parallelize a reduction only when the selected
operation provides the necessary laws.

## Reproducible approximate sums

Order-independent approximate results require semantics stronger than ordinary
finite-precision addition. Topal can support explicit alternatives:

- Accumulate exact intermediate values and round once at the requested boundary.
- Use a specified reproducible superaccumulator or binned summation function.
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
functions make that policy explicit:

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
  width is 32,
  signed is TwosComplement,
  endian is Little
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
encoding           mapping between semantic values and external bit patterns
storage-size       complete chosen representation including overhead
```

Width and byte order are layout fields shared across encoding families. The
complete type-directed field vocabulary is defined in
[layouts and addressed storage](layouts.md#encoding-construction-and-validation).

For example, `ModNat ( 0 .. 9 )` may be stored in a byte, packed into four bits,
or encoded as an ASCII digit without changing its modulo-10 arithmetic. A
`ModNat ( 0 .. 255 )` may occupy one byte in an encoded array and a full machine
register during computation.

As with strings, representation becomes observable only at an explicit storage,
encoding, hardware, or foreign-language boundary. The compiler otherwise has
freedom to choose and change representations.

## Fundamental operations

Numeric types expose operations according to their algebraic capabilities. The
fundamental numeric vocabulary is:

```topal
left = right
left != right
left <=> right
left < right
left <= right
left > right
left >= right
zero NumberType
one NumberType
negate value
absolute value
left + right
left - right
left * right
left / right
left % right
left /% right
left ^ right
value convert NumberType
```

`=` is the fundamental equality operation and returns `Boolean`; `!=` is its
derived negation. Equality does not require ordering. Totally ordered numeric
domains additionally provide three-way comparison:

```topal
left <=> right -> Comparison

Comparison
  Less
  Equal
  Greater
```

The familiar Boolean ordering operators are derived from `<=>`: `<` selects
`Less`, `>` selects `Greater`, and the inclusive forms additionally select
`Equal`. They remain standard operators because they state the intended
predicate directly. Comparison chains have no special evaluation rule: write
`a < b and b < c`, rather than `a < b < c`, consistently with ordinary
left-to-right application.

A domain with only a partial order exposes `compare` returning
`PartialComparison`:

```topal
left compare right -> PartialComparison

PartialComparison
  Less
  Equal
  Greater
  Incomparable
```

It does not provide `<=>` as a total comparison. `Comparison` converts
losslessly to `PartialComparison`, while converting `Incomparable` to
`Comparison` fails validation. The result types and their relationship to
`PartialOrder` and `TotalOrder` are defined in the
[capability vocabulary](capabilities.md#value-comparison).

`/` is exact division. Dividing values from `Nat`, `Int`, `Rational`, or
`FixedPoint` produces `Rational`; it never silently rounds or truncates to the
operand domain. A statically verified lossless conversion may immediately
satisfy a narrower expected type:

```topal
fifty : Int is 100 / 2
third : Rational is 1.0 / 3.0
```

The first declaration is valid because the compiler proves that the exact
rational result is the integer `50`. With dynamic operands, conversion to `Int`
is infallible only when evidence proves a nonzero divisor and exact divisibility;
otherwise zero division and failed integer conversion are reported through the
ordinary composed `Result` vocabularies.

`%` and `/%` are the compact Euclidean operations for discrete numeric domains.
`%` produces the modulo, while `/%` produces the quotient and modulo together:

```topal
17 % 5    # 2
17 /% 5   # ( 3 , 2 )
-17 /% 5  # ( -4 , 3 )
```

For `a /% b = ( q , r )`, where `b` is nonzero, the result is uniquely defined
by:

```text
a = b * q + r
0 <= r < absolute b
```

Consequently, `%` always produces the canonical nonnegative modulo, including
when either operand is negative. `^` is exponentiation where the operand domain
supports the requested exponent; its detailed result typing is
capability-specific.

Directional zero allows an ordinary numeric domain to divide a nonzero value
by that zero and produce the corresponding infinity, as described under zero
directionality. Directionless zero still reports
`division-by-zero`, and indeterminate forms such as `0 / 0` report
`indeterminate` rather than producing NaN.

The discrete `%` and `/%` operations never produce infinity. They report the
numeric domain's `division-by-zero` arithmetic error for a zero divisor. A
statically evident invalid zero divisor is rejected during compilation. If a
divisor is proven nonzero, an operation has its ordinary result type; otherwise
its type is `Result` of that result type, and the caller handles or propagates
the error according to the standard error model.

`absolute` produces a value's nonnegative magnitude. Prefix `-` is the compact
syntax for `negate`; binary `-` remains subtraction.

Subtraction is fundamental independently of negation. `Nat` supports subtraction
even though it is not closed under negation. The operation derives the strongest
result constraint justified by its operands and available ordering evidence:

```text
Nat - Nat where right <= left -> Nat
Nat - Nat                       -> Int
```

Consequently, subtraction of statically known literals retains a natural result
when possible:

```topal
5 - 2  # Nat 3
2 - 5  # Int -3
```

Where a domain is closed under negation, subtraction additionally has the
derived algebraic law:

```topal
a - b = a + ( negate b )
```

This is one subtraction operation with evidence-sensitive result typing, not a
saturating natural subtraction. Saturation remains an explicit arithmetic
policy. Ordering predicates derive from `<=>`. The Euclidean `%` and `/%`
operations provide discrete numeric domains' modulo and combined integer
division. Truncating quotient and signed-remainder operations are not part of
the initial model.

Not every numeric type supports every operation, and an operation need not
return its operand type. `Nat`, for example, is not closed under negation, and
exact division of two integers may produce a rational result. Rounding,
approximation, saturation, wrapping, roots, transcendental functions, and bit
operations are standard or capability-specific functions rather than
universal numeric fundamentals. Exponentiation has the common `^` spelling but
remains capability-specific rather than being supported by every numeric type.

## Provisional hierarchy

```topal
Int
Nat
Rational
FixedPoint specification
Approx specification

ModInt range
ModNat range
Bits width

Range ( minimum , maximum )
```

The stable design principles are:

- Exact arithmetic is the default.
- `Nat` is a nonnegative exact integer, not a machine unsigned integer.
- Constraints restrict values without changing operations.
- `ModInt` and `ModNat` use ranges to define modular arithmetic and canonical
  representatives.
- `Bits` is fixed-width and has no intrinsic numeric interpretation.
- Approximation, rounding, saturation, and wrapping are never implicit.
- Algebraic laws determine whether the compiler may reorder operations.
- Encodings and storage layouts remain boundary concerns.
