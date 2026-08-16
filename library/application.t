#!/usr/bin/env topal
use language (
  version is v0.1
)

# Executes the first standard-library vertical slice through the same package
# tree consumed by source tools. The derived function itself remains in the
# ordinary published source under fundamental/ordering.t.
minimum is fundamental ordering minimum
maximum is fundamental ordering maximum
between-inclusive is fundamental ordering between-inclusive
implies is fundamental boolean implies
keep-unit is fundamental unit keep
optional-int-or is fundamental optional-result optional-int-or
result-rational-or is fundamental optional-result result-rational-or
result-rational-failed is fundamental optional-result result-rational-failed
sign is numeric exact sign
distance is numeric exact distance
square is numeric exact square
intersect is numeric ranges intersect
range-contains is numeric ranges contains
ByteRing is numeric ranges ByteRing

divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  left / right
failed-division is divide (1.0, 0.0)
(minimum (4, 2), maximum (4, 2), between-inclusive (3, 2 .. 4), implies (false, false), keep-unit (), optional-int-or (None Int, 7), result-rational-or (failed-division, 5.0), result-rational-failed failed-division, sign -9, distance (-4, 5), square 1.5, intersect (0 .. 5, 3 .. 8), range-contains (3 .. 8, 9), (-1) modulo ByteRing)
