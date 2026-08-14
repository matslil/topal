#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible custom-continuation closure with lexical root domain
# and separate root.pause-once generator provenance.
pause-once is generator ( initial : Character )
  yields Character
  resumes Unit
  -> Unit

  _ is yield initial
  ()

abandon is fn ( initial : Character ) -> Unit
  generated is pause-once initial
  ()

abandon "T"
