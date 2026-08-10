#!/usr/bin/env topal
# Demonstrates reversible delivery and handling of generator-closed inside the
# abandoned custom continuation, with domain and provenance kept separate.
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
