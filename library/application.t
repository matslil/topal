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

(min (4, 2), max (4.5, 2.5), min ((1, 2), (1, 3)), min-max (7, 3), sign -9, sign -0.5, distance (-4, 5), distance (-0.5, 1.0))
