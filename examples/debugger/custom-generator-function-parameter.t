#!/usr/bin/env topal
# Demonstrates reversible linear transfer of a suspended custom continuation
# from its caller binding into an ordinary function parameter.
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
