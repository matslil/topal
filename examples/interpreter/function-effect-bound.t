#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates an explicit empty upper bound on a function's inferred effects.
# The bound belongs to the function contract and is retained by static view.
identity is fn ( value : Int ) -> Int
  : Effects ()
  value
signature is lang view identity
identity 42
