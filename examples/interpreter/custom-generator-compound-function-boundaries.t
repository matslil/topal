#!/usr/bin/env topal
# Demonstrates a generator with positional-product yield and return classifiers
# crossing ordinary function result and parameter ownership boundaries.
pairs is generator ( initial : (Int, String) )
  yields (Int, String)
  resumes Unit
  -> (Int, String)

  _ is yield initial
  (8, "done")

make is fn ( initial : (Int, String) ) -> Generator (Int, String) Unit (Int, String)
  pairs initial

consume is fn ( generated : Generator (Int, String) Unit (Int, String) ) -> (Int, String)
  result : (Int, String) is generated foreach { value }
    _ is value = (7, "item")
  result

generated is make (7, "item")
consume generated
