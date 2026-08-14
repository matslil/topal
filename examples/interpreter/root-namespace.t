#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates explicit root namespace qualification. The qualified path selects
# the declared function before applying its ordinary Int operand.
increment is fn (value : Int) -> Int
  value + 1

(root, root increment 41)
