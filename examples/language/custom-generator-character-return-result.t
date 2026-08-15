#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates returning a Character-yielding, Unit-resumed continuation whose
# distinct final Character remains available to caller-side foreach traversal.
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
