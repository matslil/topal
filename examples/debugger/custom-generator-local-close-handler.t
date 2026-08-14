#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible restoration of local nominal and function state while
# generator abandonment executes its generator-closed result action.
handle-close is generator ( initial : Character )
  yields Character
  resumes Unit
  -> Unit

  CloseChoice is Enum ( Closed, Continued )
  cleanup is fn ( choice : CloseChoice ) -> Unit
    ()
  resume-result is yield initial
  resume-result
    Error ( code is lang generator generator-closed ) then cleanup Closed
    Error problem then ()
    Ok resumed then cleanup Continued

abandon is fn ( initial : Character ) -> Unit
  generated is handle-close initial
  ()

abandon "T"
