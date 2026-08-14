#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible debugging following one message transaction.
# Its trace connects send, receive, state replacement, and response.
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

counter is counter-service 40
counter increment 2
counter current ()
