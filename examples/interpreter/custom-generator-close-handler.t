#!/usr/bin/env topal
# Demonstrates a generator handling abandonment. The suspended yield binding
# receives generator-closed with Error.domain root, selects Error, and returns.
handle-close is generator ( initial : Character )
  yields Character
  resumes Unit
  -> Unit

  resume-result is yield initial
  resume-result
    Error problem then ()
    Ok resumed then ()

abandon is fn ( initial : Character ) -> Unit
  generated is handle-close initial
  ()

abandon "T"
