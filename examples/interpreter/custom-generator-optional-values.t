#!/usr/bin/env topal
# Demonstrates that an Optional retains both its alternative and nominal Int
# payload classifier across generator input, yield, suspension, and return.
optional is generator ( initial : Optional Int )
  yields Optional Int
  resumes Unit
  -> Optional Int

  _ is yield initial
  None Int

generated is optional (Some 7)
generated foreach { candidate }
  _ is candidate = (Some 7)
