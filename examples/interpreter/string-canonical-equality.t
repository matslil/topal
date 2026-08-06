#!/usr/bin/env topal
# Demonstrates that exact String equality preserves Unicode representation,
# while canonically-equals recognizes composed and decomposed equivalents.
composed is "é"
decomposed is "é"
(composed = decomposed, composed canonically-equals decomposed, composed canonically-equals "e")
