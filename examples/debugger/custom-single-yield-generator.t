#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible declaration, start, yield, Unit resume, and return for
# a custom single-yield generator.
once is generator ( initial : Character )
  yields Character
  resumes Unit
  -> Unit

  _ is yield initial
  ()

generated is once "T"
generated foreach { character }
  _ is String character
