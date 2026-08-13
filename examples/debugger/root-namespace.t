#!/usr/bin/env topal
# Demonstrates reversible root namespace and qualified terminal resolution before
# the selected function is entered.
increment is fn (value : Int) -> Int
  value + 1

(root, root increment 41)
