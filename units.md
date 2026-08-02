# Quantities, affine points, and units

Topal combines exact or approximate numbers with dimensioned units. Linear
dimensions describe quantities such as distance and duration. Affine dimensions
describe points such as absolute temperatures together with compiler-derived
linear differences. Units determine scale and, for derived affine coordinates,
origin; they do not change the arithmetic semantics of the underlying number.

## Dimensions

Capitalized language constructions distinguish linear and affine dimensions:

```topal
Distance is Dimension
Duration is Dimension
Temperature is AffineDimension
```

A linear dimension directly classifies quantities:

```topal
length : Distance
elapsed : Duration
```

An affine dimension directly classifies points. Its associated linear
difference classifier is constructed by language-defined `Delta`:

```topal
outside : Temperature
change : Delta Temperature
```

`Delta` accepts an `AffineDimension` or one of its affine `MeasurementUnit`
objects and produces the corresponding linear dimension or unit classifier. It
is a static
classifier construction rather than a capability. Applying it to an ordinary
linear dimension or unit is invalid because that object is already linear.

## One MeasurementUnit construction

`MeasurementUnit` is the single capitalized construction for fundamental,
derived, linear, and affine units. Its argument shape and referenced dimension
determine which form is being declared.

A fundamental linear unit names a `Dimension`:

```topal
Metre is MeasurementUnit (
  symbol is m,
  prefixes is SI,
  dimension is Distance
)
```

A fundamental affine unit names an `AffineDimension` and establishes its
canonical coordinate origin and scale:

```topal
Kelvin is MeasurementUnit (
  symbol is K,
  prefixes is SI,
  dimension is Temperature
)
```

A derived linear unit supplies a dimensioned scale quantity. Its dimension is
inferred, so it cannot also provide `dimension`:

```topal
Inch is MeasurementUnit (
  symbol is in,
  prefixes is none,
  scale is 0.0254[m]
)
```

A unit enabled for a prefix family does not need explicit declarations for its
prefixed forms. For example, SI-enabled `Metre` automatically supplies symbolic
`cm` and named `Centimetre`; declaring either spelling again is a collision even
if the explicit scale would be definitionally equal.

A derived affine unit supplies a fundamental or derived affine reference, a
dimensionless scale, and an offset expressed in the reference's delta unit:

```topal
Celsius is MeasurementUnit (
  symbol is °C,
  prefixes is none,
  reference is Kelvin,
  scale is 1,
  offset is 273.15[ΔK]
)
```

Its coordinate conversion is exact when the underlying number can represent
the result:

```text
reference coordinate = source coordinate * scale + offset
```

The parameter forms are mutually exclusive:

- `dimension : Dimension` declares a fundamental linear unit;
- `dimension : AffineDimension` declares its fundamental affine unit;
- one dimensioned `scale` declares a derived linear unit; and
- affine `reference`, dimensionless `scale`, and delta-valued `offset` together
  declare a derived affine unit.

Every declaration supplies `symbol` and `prefixes` exactly once. A fundamental
dimension has exactly one fundamental unit. A scale must be nonzero and finite,
affine references must share one affine dimension, and derived declarations
must not form cycles. Invalid mixtures are rejected rather than interpreted by
argument order.

## Compiler-derived delta units

Every affine unit automatically provides its linear delta unit. Lowercase
`delta` is the language-defined named-unit operator inside `[]`:

```topal
5[delta Celsius]
5[delta Kelvin]
```

The symbolic form is derived by prefixing the language-defined capital delta
symbol `Δ` to the complete unit symbol:

```topal
5[Δ°C]
5[ΔK]
```

There is no `delta-symbol` declaration. The compiler owns this derivation and
rejects an explicit symbol or prefix combination which would collide with it.
The delta unit has the point unit's scale and no offset.

At the classifier level, `Delta` is capitalized because it constructs a linear
dimension classifier:

```topal
difference : Delta Temperature is 5[Δ°C]
```

Inside brackets, `delta` is lowercase to distinguish the unit operator from a
named unit atom:

```topal
difference : Delta Temperature is 5[delta Celsius]
```

## Quantity expressions

A bracketed unit expression following a number constructs a linear quantity or
an affine point:

```topal
9.81[N]
250[g]
5[kg]
20[°C]
5[Δ°C]
```

The brackets are structural syntax rather than part of the numeric token.
Whitespace around delimiters is accepted. Ordinary mathematical operators
retain Topal's normal whitespace rules, so these are valid:

```topal
9.81[m / (s ^ 2)]
25[N * m]
100[kg * m / (s ^ 2)]
```

but `1[kg*m]` is invalid because `*` is not a delimiter and requires surrounding
whitespace.

One unit-expression grammar accepts two non-mixing atom vocabularies. Symbolic
mode uses declared symbols and enabled symbolic prefixes:

```topal
1[kg * m / (s ^ 2)]
5[Δ°C]
```

Named mode uses complete static unit names and lowercase `delta`:

```topal
1[Kilogram * Metre / (Second ^ 2)]
5[delta Celsius]
```

Unicode symbols do not require ASCII aliases. For example, angular degree and
Celsius remain distinct complete symbols while their named forms stay readable:

```topal
90[°]
90[Degree]
20[°C]
20[Celsius]
```

Both modes use the same `*`, `/`, `^`, and parenthesized grouping. There are no
parallel prose operators such as `per`, `squared`, or `power`. The first unit
atom selects the mode; every later atom must use the same vocabulary. Operators,
parentheses, and whitespace are neutral. These are consequently invalid:

```topal
1[kg * Metre]
1[Kilogram * m]
5[delta °C]
5[Δ Celsius]
```

Enabled language-defined prefixes derive both symbolic and complete named atoms
without changing the mode. Symbolic derivation concatenates the prefix symbol
and complete unit symbol, while named derivation constructs one capitalized
identifier from the long prefix and lowercased unit stem:

```text
c    + m      -> cm
Centi + metre -> Centimetre

Milli + second -> Millisecond
Kilo  + gram   -> Kilogram
Kibi  + byte   -> Kibibyte
```

The examples above show word components, not source tokens. Inside `[]`, each
derived name is one atom with no whitespace:

```topal
1[Centimetre]
5[Millisecond]
2[Kilogram]
4[Kibibyte]
```

Forms such as `[Centi Metre]` and `[Milli Second]` are invalid. The prefix is
not an independent unit atom, and juxtaposition is not multiplication. This
also keeps a prefixed symbol distinct from an explicit product:

```topal
1[ms]      # Millisecond.
1[m * s]   # Metre multiplied by Second.
```

The selected language version supplies canonical long prefix names, their
capitalization derivation, and factors. Enabling prefixes reserves every
derived symbolic and named spelling. A module rejects an explicit declaration,
import, or compiler-derived delta form which collides with any reserved atom;
source or import order never chooses an interpretation.

## Linear arithmetic

Fundamental and derived linear units normalize to dimensions and exact scale
factors. For example:

```topal
Newton is MeasurementUnit (
  symbol is N,
  prefixes is SI,
  scale is 1[kg * m / (s ^ 2)]
)
```

The checker can prove:

```topal
2[kg] * 4[m] / (1[s] ^ 2) = 8[N]
2[kg] + 500[g] = 2.5[kg]
8[N] + 2[kg * m / (s ^ 2)] = 10[N]
```

Addition, subtraction, comparison, and equality require compatible dimensions.
Multiplication, division, and integer powers derive dimension products. An
expected type, explicit conversion, or formatting choice may select a preferred
compatible unit without changing the semantic quantity.

## Affine arithmetic

An affine unit constructs a point by default; its delta unit constructs a
linear difference:

```topal
outside : Temperature is 20[°C]
inside : Temperature is 25[°C]
step : Delta Temperature is 5[Δ°C]
```

The language-defined operations are:

```text
Affine - Affine       -> Delta Affine
Affine + Delta Affine -> Affine
Delta Affine + Affine -> Affine
Affine - Delta Affine -> Affine

Delta Affine + Delta Affine -> Delta Affine
Delta Affine - Delta Affine -> Delta Affine
```

Adding two affine points, multiplying or dividing a point, and raising a point
to a power are invalid. Points have no implicit numeric zero. A delta cannot
become a point implicitly because it carries no origin; add it to an existing
point. Conversely, subtract an origin point to obtain a delta:

```topal
freezing : Temperature is 0[°C]
boiling : Temperature is freezing + 100[Δ°C]
difference : Delta Temperature is boiling - freezing
```

Converting between point units preserves one point and applies scale plus
offset. Converting between delta units preserves one difference and applies
scale only:

```topal
celsius : Temperature is 20[°C]
kelvin : Temperature is celsius as Kelvin

change-celsius : Delta Temperature is 5[Δ°C]
change-kelvin : Delta Temperature is change-celsius as Delta Kelvin
```

There is no separate `Point` wrapper or `at` construction. The affine dimension
is the point classifier, and its unit literal constructs a point directly.

## Prefixes and symbols

Prefix meanings are language-defined and case-sensitive. Unit declarations
select `none`, one family such as `SI` or `Binary`, or a product such as
`( SI, Binary )`. Programs may define units but cannot redefine prefix families
or their factors.

The parser resolves one complete symbolic or named atom from the declarations
and reserved derived forms in scope. There is no precedence rule between an
explicit atom and a prefixed interpretation because declarations which would
create both are rejected. Unit symbols are case-sensitive: `m`, `M`, `b`, and
`B` are distinct. Prefixes scale units but never change their dimensions or turn
points into deltas.

For data amounts:

```topal
DataAmount is Dimension

Bit is MeasurementUnit (
  symbol is b,
  prefixes is ( SI, Binary ),
  dimension is DataAmount
)

Byte is MeasurementUnit (
  symbol is B,
  prefixes is ( SI, Binary ),
  scale is 8[b]
)
```

Consequently, `1[B]` equals `8[b]` and `1[KiB]` equals `1_024[B]`.

## Context-dependent conversions

Fixed affine conversion does not describe exchange rates, civil time zones,
historical calendars, sensor calibration, or other conversions which depend on
time, external state, policy, or effects. Those remain ordinary explicit
functions. Affine units provide only a fixed scale and offset relative to one
declared reference.
