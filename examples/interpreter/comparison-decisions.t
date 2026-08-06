#!/usr/bin/env topal
# Demonstrates an ordered comparison decision table implementing minimum. The
# subject becomes the left comparison operand, only the selected action runs,
# and the exhaustive otherwise fallback is necessarily the final rule.
minimum is fn (left : Int, right : Int) -> Int
  left
    < right then left
    otherwise right
(42 minimum 50, 60 minimum 50)
