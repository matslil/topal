#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates transferring a suspended custom continuation into a function.
# The caller binding is consumed and the callee traverses that same state once.
pause-once is generator ( initial : Character )
  yields Character
  resumes Unit
  -> Unit

  _ is yield initial
  ()

consume is fn ( generated : Generator Character Unit Unit ) -> Unit
  generated foreach { character }
    _ is String character

generated is pause-once "T"
consume generated
