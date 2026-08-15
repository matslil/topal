#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates closure of an unconsumed transferred custom-generator parameter.
# The caller loses ownership; function exit closes the callee's suspended state.
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
