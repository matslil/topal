#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates returns from every action of complete Optional decisions.

explicit is fn (candidate : Optional Int) -> Int
  candidate
    None then { return 40 }
    Some payload then { return payload + 1 }
  1000

some-fallback is fn (candidate : Optional Int) -> Int
  candidate
    Some payload then { return payload + 2 }
    otherwise { return 44 }
  1000

none-fallback is fn (candidate : Optional Int) -> Int
  candidate
    None then { return 45 }
    otherwise { return 1001 }
  1000

(explicit (Some 42), explicit (None Int), some-fallback (None Int), none-fallback (None Int))
