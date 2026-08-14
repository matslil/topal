#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible entry and return through a Nat-classified function.
identity is fn (value : Nat) -> Nat
  value
identity 42
