#!/usr/bin/env topal
use language (
  version is v0.1
)

# Executes the first standard-library vertical slice through the same package
# tree consumed by source tools. The derived function itself remains in the
# ordinary published source under fundamental/ordering.t.
min is fundamental ordering min
max is fundamental ordering max
min-max is fundamental ordering min-max
sign is numeric exact sign
distance is numeric exact distance
optional-map is fundamental optional map
absent? is fundamental optional absent?
optional-zip is fundamental optional zip
result-map is fundamental result map
divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  left / right
gcd is numeric exact gcd
even? is numeric exact even?
reciprocal is numeric exact reciprocal

(min (4, 2), max (4.5, 2.5), min ((1, 2), (1, 3)), min-max (7, 3), sign -9, sign -0.5, distance (-4, 5), distance (-0.5, 1.0), optional-map ((Some 4), { value } value + 1), absent? (None String), optional-zip ((Some 2), (Some "items")), result-map ((8.0 divide 2.0), { value } value + 1.0), gcd (-54, 24), even? -4, reciprocal 2.0)
