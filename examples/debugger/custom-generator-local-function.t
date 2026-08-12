#!/usr/bin/env topal
# Demonstrates reversible retention of generator-local enum and function
# declarations across Boolean suspension and Unit resumption.
describe is generator ( initial : Boolean )
  yields Boolean
  resumes Unit
  -> String

  Choice is Enum ( Accepted, Rejected )
  label is fn ( value : Choice ) -> String
    value
      Accepted then "accepted"
      Rejected then "rejected"
  _ is yield initial
  label Accepted

generated is describe true
generated foreach { value }
  _ is not value
