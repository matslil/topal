#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that a generator may return Unit before its first yield. Foreach
# therefore invokes no action and produces the generator's final Unit directly.
nothing is generator ( initial : Character )
  yields Character
  resumes Unit
  -> Unit

  ()

generated is nothing "T"
generated foreach { character }
  _ is String character
