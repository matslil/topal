#!/usr/bin/env topal
# Demonstrates that Character classifies one user-perceived Unicode character,
# including a grapheme made from several preserved scalar values.
identity is fn (value : Character) -> Character
  value
composed : Character is "á"
(identity "🙂", identity composed)
