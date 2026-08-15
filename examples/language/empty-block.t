#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that an empty lexical block evaluates to Unit and that bindings
# inside a nonempty block produce its final value without escaping the block.
empty is {}
{
  local is 41
  (empty, local + 1)
}
