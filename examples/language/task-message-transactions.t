#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a specialized task definition, typed private state, construction,
# a Unit event, and a Result request. State is selected and replaced with `@`.
Counter is Task (queue-size is 10, identity is counter)

counter-service is Counter
  count : Nat
  start is fn ( initial : Nat ) -> Completed
    @ count is initial
    Completed
  increment is fn ( _ : MessageContext, amount : Nat ) -> Unit
    @ count is @ count + amount
  current is fn ( _ : MessageContext, _ : Unit ) -> Result ( Nat, () )
    @ count
  values is generator ( _ : MessageContext, _ : Unit )
    yields Nat
    resumes Unit
    -> Result ( Unit, () )
    yield @ count
    ()
  terminate is fn ( _ : String ) -> Unit
    ()

counter is counter-service 40
counter increment 2
stream is counter values ()
stream foreach { value }
  ()
counter current ()
