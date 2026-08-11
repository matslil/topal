#!/usr/bin/env topal
# Demonstrates String as the yield direction. Foreach receives each complete
# String unchanged, tests it, and resumes the continuation with Unit.
texts is generator ( initial : String )
  yields String
  resumes Unit
  -> Unit

  _ is yield initial
  _ is yield ""
  ()

generated is texts "Topal"
generated foreach { text }
  _ is empty? text
