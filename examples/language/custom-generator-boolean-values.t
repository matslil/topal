#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates independent Boolean generator directions: true is the initial
# input and yielded value, while the final return is the distinct value false.
invert is generator ( initial : Boolean )
  yields Boolean
  resumes Unit
  -> Boolean

  _ is yield initial
  not initial

generated is invert true
generated foreach { value }
  _ is not value
