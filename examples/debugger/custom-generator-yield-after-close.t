#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates the diagnostic for yielding after abandonment delivered
# generator-closed to the first suspended yield-result binding.
invalid-close is generator ( initial : Character )
  yields Character
  resumes Unit
  -> Unit

  resume-result is yield initial
  _ is yield initial
  ()

abandon is fn ( initial : Character ) -> Unit
  generated is invalid-close initial
  ()

abandon "T"
