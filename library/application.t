#!/usr/bin/env topal
use language (
  version is v0.1
)

# Executes the first standard-library vertical slice through the same package
# tree consumed by source tools. The derived function itself remains in the
# ordinary published source under fundamental/ordering.t.
minimum is fundamental ordering minimum
maximum is fundamental ordering maximum
implies is fundamental boolean implies
sign is numeric exact sign
distance is numeric exact distance

(minimum (4, 2), maximum (4, 2), implies (false, false), sign -9, distance (-4, 5))
