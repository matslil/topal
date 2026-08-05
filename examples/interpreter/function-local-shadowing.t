#!/usr/bin/env topal
# Demonstrates that a function-local binding can shadow a captured outer name,
# while the outer binding remains unchanged after the call returns.
value is 40
answer is fn () -> Int
  value is 42
  value
(answer (), value)
