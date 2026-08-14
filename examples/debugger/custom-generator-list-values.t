#!/usr/bin/env topal-debug
use language (
  version is v0.1
)
# Demonstrates reversible List transfer through generator suspension and
# ordinary function ownership boundaries before its distinct final return.
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
