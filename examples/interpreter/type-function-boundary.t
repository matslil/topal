#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a Type value crossing an ordinary function boundary.
identity-type is fn (kind : Type) -> Type
  kind
identity-type Int
