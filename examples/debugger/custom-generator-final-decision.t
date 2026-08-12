#!/usr/bin/env topal
# Demonstrates reversible ordering from Boolean suspension through Unit
# resumption and final decision selection to the distinct String return.
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
