# Quantities and units

This document records the provisional model for measured quantities in Topal.
A quantity combines a number with a unit. Units carry dimensions and scale;
they do not change the arithmetic semantics of the underlying numeric type.

## Quantity expressions

A unit expression in square brackets following a number constructs a quantity:

```topal
9.81[N]
250[g]
5[kg]
4[KiB]
```

The brackets are structural syntax rather than part of the numeric token.
Whitespace around and within them is accepted, while the formatter uses the
compact spelling above. Thus `9.81 [ N ]` and `9.81[N]` have the same meaning.

Unit expressions may multiply, divide, and raise units to integer powers:

```topal
10[m]
3[s]
9.81[m / ( s ^ 2 )]
25[N * m]
100[kg * m / ( s ^ 2 )]
```

The exact operator spelling remains provisional. Operations on quantities
combine their unit expressions along with their numeric values.

## Dimensions and derived units

Every unit has a dimension and a scale relative to a canonical unit for that
dimension. Fundamental units establish the dimensions from which derived units
are composed. A derived unit declaration establishes an equality rather than
merely giving the compiler an optimization hint.

For example, newton is the unit of force and is definitionally equivalent to
kilogram metre per second squared. Provisional declaration syntax is:

```topal
Newton is unit (
  symbol N ,
  dimension Mass * Length / ( Time ^ 2 ) ,
  scale 1[kg * m / ( s ^ 2 )] ,
  prefixes allowed
)
```

The checker normalizes dimension products and scale factors. It can therefore
prove this equality statically:

```topal
2[kg] * 4[m] / ( 1[s] ^ 2 ) = 8[N]
```

Conceptually, both sides normalize to `8[kg * m / ( s ^ 2 )]`. The inferred
result need not be displayed using the derived symbol. An expected type,
explicit conversion, or formatting choice may select `N`:

```topal
force : N
force is 2[kg] * 4[m] / ( 1[s] ^ 2 )
```

Addition, subtraction, comparison, and equality require compatible dimensions.
Scale conversion is exact whenever the underlying number type can represent the
result:

```topal
2[kg] + 500[g] = 2.5[kg]
8[N] + 2[kg * m / ( s ^ 2 )] = 10[N]
2[kg] + 4[m] # Invalid: mass and length have incompatible dimensions.
```

Multiplication and division instead derive new dimensions, so `2[kg] * 4[m]`
is the valid quantity `8[kg * m]`.

## Prefixes

Prefix meanings are language-defined and case-sensitive. Programs may define
units, but cannot redefine prefixes. Topal uses the standard SI spellings such
as `m`, `k`, `M`, and `G`, and the standard binary spellings `Ki`, `Mi`, `Gi`,
and so on:

```topal
1[kg] = 1_000[g]
1[kN] = 1_000[N]
1[KiB] = 1_024[B]
```

A unit declaration controls whether prefixes may be applied to it. The parser
first resolves a complete declared unit symbol and only then tries a recognized
prefix followed by a unit symbol. A module must reject declarations that would
make a quantity's unit spelling ambiguous.

Unit symbols are case-sensitive: `m`, `M`, and `N` are distinct. Prefixes scale
units but never change their dimensions.

## Custom units

Programs may declare new fundamental or derived units, give them symbols, and
choose whether language-defined prefixes are accepted. Declarations must state
enough dimension and scale information for the checker to normalize quantities
without relying on their textual unit spellings.

Simple multiplicative scaling does not describe every measurement system.
Affine units such as degrees Celsius additionally require an offset, and units
with context-dependent conversions require explicit conversion algorithms.
Their declaration syntax and arithmetic rules remain undecided.

Mass and force remain distinct dimensions. Grams and kilograms measure mass;
newtons measure force. Informal uses of “weight” do not make values in grams
compatible with values in newtons.
