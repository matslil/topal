#!/usr/bin/env topal
# Demonstrates zero-based indexing by user-perceived Unicode characters. The
# combining sequence and emoji sequence stay whole; invalid indexes return None.
# An Optional decision can consume the result without splitting the character.
describe is fn (candidate : Optional Character) -> String
  candidate
    Some character then String character
    None then "missing"
text is "á👩‍🔬🇸🇪"
(text character-at 0, text character-at 1, text character-at 2, text character-at -1, text character-at 3, describe (text character-at 1), describe (text character-at 3))
