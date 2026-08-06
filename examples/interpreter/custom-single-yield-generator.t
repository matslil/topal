#!/usr/bin/env topal
# Demonstrates a custom generator declaration. Applying `once` binds its
# Character input, yields that value once, accepts Unit resumption, then returns
# Unit. Direct foreach consumes the fresh linear generator.
once is generator ( initial : Character )
  yields Character
  resumes Unit
  -> Unit

  _ is yield initial
  ()

generated is once "T"
generated foreach { character }
  _ is String character
