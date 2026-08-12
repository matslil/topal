#!/usr/bin/env topal
# Demonstrates a payload-free generator boundary using Unit for input, yield,
# resumption, and final return. Foreach still invokes its action once.
pulse is generator ( initial : Unit )
  yields Unit
  resumes Unit
  -> Unit

  _ is yield initial
  ()

generated is pulse ()
generated foreach { signal }
  signal
