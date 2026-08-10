#!/usr/bin/env topal
# Demonstrates reversible ownership transfer of a suspended custom continuation
# through an ordinary function result and its later caller-side traversal.
pause-once is generator ( initial : Character )
  yields Character
  resumes Unit
  -> Unit

  _ is yield initial
  ()

make is fn ( initial : Character ) -> Generator Character Unit Unit
  pause-once initial

generated is make "T"
generated foreach { character }
  _ is String character
