#!/usr/bin/env topal
# Demonstrates a Type value crossing an ordinary function boundary.
identity-type is fn (kind : Type) -> Type
  kind
identity-type Int
