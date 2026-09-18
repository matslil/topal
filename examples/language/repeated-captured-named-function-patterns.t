#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that a repeated captured nested Function value compares its
# stable declaration identity and represented capture payload before use.
compare-nested is fn (offset : Int) -> Int
  increase is fn (value : Int) -> Int
    value + offset
  repeat : Function is { operation, operation } operation 41
  repeat (increase, increase)

compare-nested 1
