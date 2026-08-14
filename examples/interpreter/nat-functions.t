#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates Nat parameter and result classification preserving an exact,
# nonnegative Int value through an ordinary function call.
identity is fn (value : Nat) -> Nat
  value
identity 42
