#!/usr/bin/env topal
# Demonstrates a static binary function whose infix application binds two
# independently typed operands in declaration order. Its two-statement body
# also demonstrates an invocation-local binding followed by the result.
add is fn static (
  left : Int,
  right : Int
) -> Int
  sum is left + right
  sum
20 add 22
