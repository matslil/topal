#!/usr/bin/env topal
# Demonstrates reversible qualified lookup through a Scope function parameter.
answer is 42
read-answer is fn (api : Scope) -> Int
  api answer
read-answer root
