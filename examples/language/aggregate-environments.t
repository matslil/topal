#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact Tuple, Record, and nominal Sum values forwarded through
# private acyclic and independently proof-backed recursive environments.
Token is Union
  Number : Int
  Label : String

context-pair is (40, "context")
context-record is (amount is 2, enabled is true)
context-token is Label "context-sum"

select-context-pair is fn (value : Int) -> (Int, String)
  value
    <= 0 then @ context-pair
    otherwise select-context-pair (value - 1)

forward-context-pair is fn (value : Int) -> (Int, String)
  select-context-pair value

select-context-record is fn (value : Int) -> Record (amount : Int, enabled : Boolean)
  value
    <= 0 then @ context-record
    otherwise select-context-record (value - 1)

forward-context-record is fn (value : Int) -> Record (amount : Int, enabled : Boolean)
  select-context-record value

select-root-pair is fn (value : Int) -> (Int, String)
  value
    <= 0 then root live-pair
    otherwise select-root-pair (value - 1)

forward-root-pair is fn (value : Int) -> (Int, String)
  select-root-pair value

select-root-record is fn (value : Int) -> Record (amount : Int, enabled : Boolean)
  value
    <= 0 then root live-record
    otherwise select-root-record (value - 1)

forward-root-record is fn (value : Int) -> Record (amount : Int, enabled : Boolean)
  select-root-record value

select-context-token is fn () -> Token
  @ context-token

forward-context-token is fn () -> Token
  select-context-token ()

select-root-token is fn () -> Token
  root live-token

forward-root-token is fn () -> Token
  select-root-token ()

live-pair is (7, "root")
live-record is (amount is 9, enabled is false)
live-token is Number 11
(
  forward-context-pair 2,
  forward-context-record 2,
  forward-root-pair 2,
  forward-root-record 2,
  forward-context-token (),
  forward-root-token ()
)
