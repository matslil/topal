#!/usr/bin/env topal
# Demonstrates reversible function-result ownership transfer followed by yield,
# Unit resumption, and a distinct final Character return in the caller.
yield-return is generator ( initial : Character )
  yields Character
  resumes Unit
  -> Character

  _ is yield initial
  "R"

make is fn ( initial : Character ) -> Generator Character Unit Character
  yield-return initial

generated is make "Y"
generated foreach { character }
  _ is String character
