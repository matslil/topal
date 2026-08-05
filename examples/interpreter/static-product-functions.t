#!/usr/bin/env topal
# Demonstrates a static function whose positional-product input binds two
# independently typed parameters in declaration order.
add is fn static (
  left : Int,
  right : Int
) -> Int
  left + right
add (20, 22)
