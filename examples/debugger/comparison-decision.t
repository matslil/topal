#!/usr/bin/env topal
# Demonstrates reversible comparison matching and fallback selection.
minimum is fn (left : Int, right : Int) -> Int
  left
    < right then left
    otherwise right
(minimum (42, 50), minimum (60, 50))
