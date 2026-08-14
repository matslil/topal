#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates preservation of the language-defined nominal Comparison identity
# through a yielded Less alternative and a distinct final Greater alternative.
order is generator ( initial : Comparison )
  yields Comparison
  resumes Unit
  -> Comparison

  _ is yield initial
  3 <=> 2

generated is order (1 <=> 2)
generated foreach { comparison }
  _ is comparison = (1 <=> 2)
