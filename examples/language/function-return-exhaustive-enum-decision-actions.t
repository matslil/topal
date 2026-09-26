#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates returns from every action of an exhaustive Enum decision.

Color is Enum (Red, Green, Blue)

choose is fn (value : Color) -> Int
  value
    Blue then { return 45 }
    Red then { return 43 }
    Green then { return 44 }
  1000
(choose Red, choose Green, choose Blue)
