#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible validation of an Enum function signature.
Color is Enum (Red, Green, Blue)
identity is fn (value : Color) -> Color
  value
(identity Red, identity Green)
