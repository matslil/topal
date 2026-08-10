#!/usr/bin/env topal
# Demonstrates reversible selection of the qualified generator-closed code rule,
# independently of the Error domain and generator provenance.
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
