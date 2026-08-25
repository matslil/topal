#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that an explicit measure selects one argument of a larger
# recursive state. The
# checker proves the count decreases while the accumulated value may change.
repeat-add is fn (count : Nat, total : Int) -> Int : Decreases count
  count
    <= 0 then total
    otherwise repeat-add (count - 1, total + 3)

repeat-add (4, 0)
