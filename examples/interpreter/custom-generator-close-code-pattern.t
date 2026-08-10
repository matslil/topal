#!/usr/bin/env topal
# Demonstrates matching generator-closed by its qualified nominal code. Neither
# lexical Error.domain nor root.handle-close-code provenance selects this rule.
handle-close-code is generator ( initial : Character )
  yields Character
  resumes Unit
  -> Unit

  resume-result is yield initial
  resume-result
    Error ( code is lang generator generator-closed ) then ()
    Error problem then ()
    Ok resumed then ()

abandon is fn ( initial : Character ) -> Unit
  generated is handle-close-code initial
  ()

abandon "T"
