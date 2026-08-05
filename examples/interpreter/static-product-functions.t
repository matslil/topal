#!/usr/bin/env topal
# Demonstrates a static binary function whose infix application binds two
# independently typed operands in declaration order.
add is fn static (
  left : Int,
  right : Int
) -> Int
  left + right
20 add 22
