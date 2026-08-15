#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that Character classifies one user-perceived Unicode character,
# including a grapheme made from several preserved scalar values.
identity is fn (value : Character) -> Character
  value
composed : Character is "á"
# String construction forgets the Character constraint but preserves text.
(String (identity "🙂"), String composed)
