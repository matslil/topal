#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible Unicode character indexing and Optional outcomes. An
# Optional decision preserves a present Character and handles an absent index.
describe is fn (candidate : Optional Character) -> String
  candidate
    Some character then String character
    None then "missing"
text is "á👩‍🔬🇸🇪"
(text character-at 0, text character-at 1, text character-at 2, text character-at -1, text character-at 3, describe (text character-at 1), describe (text character-at 3))
