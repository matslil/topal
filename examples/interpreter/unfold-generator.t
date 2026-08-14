#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates lazy unfold construction with state separate from yielded values.
# List uncons supplies Some (entry, remaining-seed) until it reaches None.
values : List Int is Entry (4, Entry (5, Entry (6, Empty)))
generated is values unfold { remaining } uncons remaining
generated
