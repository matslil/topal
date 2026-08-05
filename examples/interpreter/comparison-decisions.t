#!/usr/bin/env topal
# Demonstrates an ordered comparison decision table implementing minimum. The
# subject becomes the left comparison operand and only the selected action runs.
minimum is fn (left : Int, right : Int) -> Int
  left
    < right then left
    otherwise right
(minimum (42, 50), minimum (60, 50))
