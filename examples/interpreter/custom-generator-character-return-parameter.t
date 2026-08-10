#!/usr/bin/env topal
# Demonstrates transferring all generator directions through a function
# parameter: Character yield, Unit resume, and distinct final Character return.
yield-return is generator ( initial : Character )
  yields Character
  resumes Unit
  -> Character

  _ is yield initial
  "R"

consume is fn ( generated : Generator Character Unit Character ) -> Character
  generated foreach { character }
    _ is String character

generated is yield-return "Y"
consume generated
