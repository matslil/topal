#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible transfer and function-exit closure of an unconsumed
# custom-generator parameter, with domain and provenance represented separately.
pause-once is generator ( initial : Character )
  yields Character
  resumes Unit
  -> Unit

  _ is yield initial
  ()

ignore is fn ( generated : Generator Character Unit Unit ) -> Unit
  ()

generated is pause-once "T"
ignore generated
