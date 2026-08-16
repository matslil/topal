#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates the task declaration-order template: private state comes first,
# followed by start, ordinary message handlers, and terminate lifecycle cleanup.
OrderedCounter is Task (queue-size is 4, identity is ordered-counter)

ordered-counter-service is OrderedCounter
  count : Nat
  start is fn (initial : Nat) -> Completed
    @ count is initial
    Completed
  increment is fn (_ : MessageContext, amount : Nat) -> Unit
    @ count is @ count + amount
  current is fn (_ : MessageContext, _ : Unit) -> Result (Nat, ())
    @ count
  terminate is fn (_ : String) -> Unit
    ()

ordered-counter is ordered-counter-service 1
ordered-counter increment 2
ordered-counter current ()
