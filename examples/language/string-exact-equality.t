#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact preserved-sequence String equality and derived Optional
# String equality without canonical normalization.
composed is "é"
decomposed is "é"
preserve is fn (value : String) -> String
  value
round-trip is preserve "Topal"
(composed = decomposed, composed != decomposed, round-trip = "Topal", "" = "", (Some "ok") = (Some "ok"), (Some "ok") != (Some "no"), (None String) = (None String), (Some "ok") = (None String))
