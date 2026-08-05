#!/usr/bin/env topal
# Demonstrates explicit canonical decomposition without changing the input.
preserved is "é"
decomposed is preserved normalize NFD
different-after is decomposed != preserved
(preserved, decomposed, different-after)
