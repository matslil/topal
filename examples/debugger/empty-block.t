#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible evaluation of empty and locally scoped blocks.
empty is {}
{
  local is 41
  (empty, local + 1)
}
