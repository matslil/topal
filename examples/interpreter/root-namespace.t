#!/usr/bin/env topal
# Demonstrates explicit root namespace qualification. The qualified path selects
# the declared function before applying its ordinary Int operand.
increment is fn (value : Int) -> Int
  value + 1

(root, root increment 41)
