#!/usr/bin/env topal
# Demonstrates returning a live custom continuation from an ordinary function.
# Function exit transfers ownership; the caller then consumes the single yield.
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
