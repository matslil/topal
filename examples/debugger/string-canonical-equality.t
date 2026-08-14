#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible exact and canonical Unicode String comparisons.
composed is "é"
decomposed is "é"
(composed = decomposed, composed canonically-equals decomposed, composed canonically-equals "e")
