#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible execution through Character-classified values.
identity is fn (value : Character) -> Character
  value
composed : Character is "á"
# String construction forgets the Character constraint but preserves text.
(String (identity "🙂"), String composed)
