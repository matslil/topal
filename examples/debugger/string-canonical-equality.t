#!/usr/bin/env topal
# Demonstrates reversible exact and canonical Unicode String comparisons.
composed is "é"
decomposed is "é"
(composed = decomposed, composed canonically-equals decomposed, composed canonically-equals "e")
