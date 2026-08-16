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

(minimum (4, 2), maximum (4, 2), between-inclusive (3, 2 .. 4), implies (false, false), keep-unit ())
