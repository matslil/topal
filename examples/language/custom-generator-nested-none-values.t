#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that an absent Optional retains its full positional-product
# payload classifier while crossing generator input, yield, and final return.
absent is generator ( initial : Optional (Int, String) )
  yields Optional (Int, String)
  resumes Unit
  -> Optional (Int, String)

  _ is yield initial
  None (Int, String)

generated is absent (None (Int, String))
generated foreach { candidate }
  _ is candidate = (None (Int, String))
