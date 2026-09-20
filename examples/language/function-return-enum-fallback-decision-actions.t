#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates returns from every action of a final-fallback Enum decision.

Color is Enum (Red, Green, Blue)

choose is fn (value : Color) -> Int
  value
    Red then { return 40 }
    Green then { return 41 }
    otherwise { return 42 }
  1000
(choose Red, choose Green, choose Blue)
