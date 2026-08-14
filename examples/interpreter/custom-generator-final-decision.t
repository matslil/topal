#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a final decision evaluated only after the Boolean yield resumes.
# Its selected String action becomes the generator's distinct final return.
describe is generator ( initial : Boolean )
  yields Boolean
  resumes Unit
  -> String

  _ is yield initial
  initial
    true then "accepted"
    otherwise "rejected"

generated is describe true
generated foreach { value }
  _ is not value
