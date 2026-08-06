#!/usr/bin/env topal
# Demonstrates reversible execution through Character-classified values.
identity is fn (value : Character) -> Character
  value
composed : Character is "á"
(identity "🙂", identity composed)
