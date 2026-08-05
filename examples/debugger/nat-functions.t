#!/usr/bin/env topal
# Demonstrates reversible entry and return through a Nat-classified function.
identity is fn (value : Nat) -> Nat
  value
identity 42
