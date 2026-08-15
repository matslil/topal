#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that yielded values and the final return are independent. The
# generator yields `Y`; after foreach consumes it, the expression produces `R`.
yield-then-return is generator ( initial : Character )
  yields Character
  resumes Unit
  -> Character

  _ is yield initial
  "R"

generated is yield-then-return "Y"
generated foreach { character }
  _ is String character
