#!/usr/bin/env topal
# Demonstrates that an empty lexical block evaluates to Unit and that bindings
# inside a nonempty block produce its final value without escaping the block.
empty is {}
{
  local is 41
  (empty, local + 1)
}
