#!/usr/bin/env topal
# Demonstrates reversible operand evaluation before comparison selection.
within-next is fn (value : Int, limit : Int) -> Boolean
  value
    < limit + 1 then true
    otherwise false
(5 within-next 5, 6 within-next 5)
