#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a List crossing generator input, yield, suspension, final-return,
# ordinary function result, and ordinary function parameter boundaries.
relay is generator ( initial : List Int )
  yields List Int
  resumes Unit
  -> List Int

  _ is yield initial
  initial append 9

make is fn ( initial : List Int ) -> Generator List Int Unit List Int
  relay initial

consume is fn ( generated : Generator List Int Unit List Int ) -> List Int
  result : List Int is generated foreach { values }
    _ is entry-count values
  result

generated is make (one 7)
consume generated
