#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact defining-context and live-root environments through
# retained local named Function aliases and non-escaping nested functions.
context-number is 40
context-label is "context"
context-pair is (2, "context-pair")

read-context is fn (value : Int) -> Int
  value + @ context-number

read-context is fn (value : String) -> String
  @ context-label

read-root is fn (value : Int) -> Int
  value + (root live-number)

read-root is fn (value : String) -> String
  root live-label

read-pair is fn (value : Int) -> (Int, String)
  @ context-pair

read-pair is fn (value : String) -> (Int, String)
  root live-pair

alias-values is fn () -> (Int, String, Int, String, (Int, String), (Int, String))
  context-operation is read-context
  context-chain is context-operation
  root-operation is read-root
  root-chain is root-operation
  pair-operation is read-pair
  (
    context-chain 2,
    context-chain "selected",
    root-chain 2,
    root-chain "selected",
    pair-operation 0,
    pair-operation "selected"
  )

nested-values is fn () -> (Int, Int)
  nested-context is fn (value : Int) -> Int
    value + @ context-number
  nested-root is fn (value : Int) -> Int
    value + (root live-number)
  context-operation is nested-context
  root-operation is nested-root
  (context-operation 2, root-operation 3)

live-number is 7
live-label is "root"
live-pair is (7, "root-pair")
(alias-values (), nested-values ())
