#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates abandonment of a suspended custom generator at function exit.
# Error.domain is root; root.pause-once is retained separately as provenance.
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
