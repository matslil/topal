#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible direct calls through bound anonymous function values,
# including positional-product argument binding for a multi-parameter value.
increment is { value } value + 1
combine is { left, right } left * 10 + right
(increment 41, combine (4, 2))
