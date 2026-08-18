#!/usr/bin/env topal
use language (
  version is v0.1
)

# Executes the first standard-library vertical slice through the same package
# tree consumed by source tools. The derived function itself remains in the
# ordinary published source under fundamental/ordering.t.
min is std min
max is std max
min-max is std min-max
sign is std sign
distance is std distance
optional-map is std map
absent? is std absent?
optional-zip is std zip
result-map is std map
divide is fn (left : Rational, right : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  left / right
gcd is std gcd
even? is std even?
reciprocal is std reciprocal
range-bounds is std bounds
range-hull is std hull
text-trim is std trim
text-repeat is std repeat
any? is std any?
find is std find
count-from is std count-from
values : List Int is Entry (1, Entry (2, Entry (3, Empty)))
generated is count-from 3
prefix is collect (generated take-while ({ value } value < 6))

(min (4, 2), max (4.5, 2.5), min ((1, 2), (1, 3)), min-max (7, 3), sign -9, sign -0.5, distance (-4, 5), distance (-0.5, 1.0), optional-map ((Some 4), { value } value + 1), absent? (None String), optional-zip ((Some 2), (Some "items")), result-map ((8.0 divide 2.0), { value } value + 1.0), gcd (-54, 24), even? -4, reciprocal 2.0, range-bounds (-2 .. 5), range-hull (0 .. 3, 2 .. 8), text-trim "  text  ", text-repeat ("ab", 2), any? (values, { value } value > 2), find (values, { value } value > 1), prefix)
