#!/usr/bin/env topal
# Demonstrates reversible preservation of a nominal Optional Int through a
# yielded Some alternative and a distinct final None alternative.
optional is generator ( initial : Optional Int )
  yields Optional Int
  resumes Unit
  -> Optional Int

  _ is yield initial
  None Int

generated is optional (Some 7)
generated foreach { candidate }
  _ is candidate = (Some 7)
