#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible language-defined Comparison flow across suspension,
# preserving nominal identity separately from the Less and Greater alternatives.
order is generator ( initial : Comparison )
  yields Comparison
  resumes Unit
  -> Comparison

  _ is yield initial
  3 <=> 2

generated is order (1 <=> 2)
generated foreach { comparison }
  _ is comparison = (1 <=> 2)
