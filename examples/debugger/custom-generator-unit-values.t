#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible payload-free suspension and resumption using Unit in
# every generator direction.
pulse is generator ( initial : Unit )
  yields Unit
  resumes Unit
  -> Unit

  _ is yield initial
  ()

generated is pulse ()
generated foreach { signal }
  signal
