#!/usr/bin/env topal
# Demonstrates that a comparison matcher accepts a complete operand expression;
# structural `then` terminates `limit + 1` rather than becoming an operand.
within-next is fn (value : Int, limit : Int) -> Boolean
  value
    < limit + 1 then true
    otherwise false
(5 within-next 5, 6 within-next 5)
