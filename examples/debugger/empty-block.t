#!/usr/bin/env topal
# Demonstrates reversible evaluation of empty and locally scoped blocks.
empty is {}
{
  local is 41
  (empty, local + 1)
}
