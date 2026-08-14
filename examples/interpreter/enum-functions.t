#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a nominal Enum used as both a function parameter and result.
Color is Enum (Red, Green, Blue)
identity is fn (value : Color) -> Color
  value
(identity Red, identity Green)
