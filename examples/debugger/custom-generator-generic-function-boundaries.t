#!/usr/bin/env topal
# Demonstrates reversible Generator Int Unit String ownership transfer through
# an ordinary function result and parameter before traversal and final return.
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
