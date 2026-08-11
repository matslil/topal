#!/usr/bin/env topal
# Demonstrates reversible Boolean flow through generator input, suspension,
# foreach action binding, Unit resumption, and the distinct final return.
invert is generator ( initial : Boolean )
  yields Boolean
  resumes Unit
  -> Boolean

  _ is yield initial
  not initial

generated is invert true
generated foreach { value }
  _ is not value
