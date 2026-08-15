#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates passing a namespace through the general Scope function boundary.
answer is 42
read-answer is fn (api : Scope) -> Int
  api answer
read-answer root
