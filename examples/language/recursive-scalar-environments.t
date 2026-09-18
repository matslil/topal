#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that proof-backed mutual recursion forwards the immutable
# defining context and the live root value through every private frame.
captured is 40

cycle-even is fn (value : Int) -> (Boolean, Int, Int)
  value
    <= 0 then (true, @ captured, 0)
    otherwise cycle-odd (value - 1)

cycle-odd is fn (value : Int) -> (Boolean, Int, Int)
  value
    <= 0 then (false, 0, root live)
    otherwise cycle-even (value - 1)

live is 2
(cycle-even 3, cycle-odd 3)
