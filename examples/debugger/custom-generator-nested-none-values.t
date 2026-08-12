#!/usr/bin/env topal
# Demonstrates reversible absent Optional state retaining the complete
# (Int, String) payload classifier despite carrying no payload value.
absent is generator ( initial : Optional (Int, String) )
  yields Optional (Int, String)
  resumes Unit
  -> Optional (Int, String)

  _ is yield initial
  None (Int, String)

generated is absent (None (Int, String))
generated foreach { candidate }
  _ is candidate = (None (Int, String))
