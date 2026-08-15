#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a Generator Int Unit String returned from one ordinary function,
# transferred into another, traversed there, and propagated as a final String.
numbers is generator ( initial : Int )
  yields Int
  resumes Unit
  -> String

  _ is yield initial
  "done"

make is fn ( initial : Int ) -> Generator Int Unit String
  numbers initial

consume is fn ( generated : Generator Int Unit String ) -> String
  result : String is generated foreach { value }
    _ is value + 1
  result

generated is make 7
consume generated
