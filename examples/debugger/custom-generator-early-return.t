#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible generator start and final Unit return with no yield or
# resumption transition between them.
nothing is generator ( initial : Character )
  yields Character
  resumes Unit
  -> Unit

  ()

generated is nothing "T"
generated foreach { character }
  _ is String character
