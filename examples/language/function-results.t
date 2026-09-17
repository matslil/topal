#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a private Function result retaining the callable selected by a
# specialized invocation, for both named and symbolic function values.
increment is fn (value : Int) -> Int
  value + 1

select is fn (operation : Function) -> Function
  operation

selected is select increment
addition is select +
returned is selected
(returned 41, addition (20, 22))
