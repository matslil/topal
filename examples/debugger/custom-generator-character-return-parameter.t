#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible parameter transfer, yield, Unit resume, and distinct
# Character return through the consuming ordinary function.
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
