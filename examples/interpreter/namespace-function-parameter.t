#!/usr/bin/env topal
# Demonstrates passing a namespace through the general Scope function boundary.
answer is 42
read-answer is fn (api : Scope) -> Int
  api answer
read-answer root
