#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible product-classified generator ownership through an
# ordinary function result and parameter before traversal and result binding.
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
