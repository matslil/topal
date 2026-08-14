#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that rebinding a namespace alias retains identity and members.
answer is 42
first is root
second is first
second answer
