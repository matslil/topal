#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact overload-selected defining-context and live-root capture
# sets through direct, acyclic, cross-overload, and proven recursive calls.
context-number is 40
context-label is "context"
context-pair is (2, "context-pair")

choose-context is fn (value : Int) -> Int
  value
    <= 0 then @ context-number
    otherwise choose-context (value - 1)

choose-context is fn (value : String) -> String
  @ context-label

forward-context-number is fn () -> Int
  choose-context 2

forward-context-label is fn () -> String
  choose-context "selected"

choose-pair is fn (value : Int) -> (Int, String)
  @ context-pair

choose-pair is fn (value : String) -> (Int, String)
  root live-pair

forward-context-pair is fn () -> (Int, String)
  choose-pair 0

forward-root-pair is fn () -> (Int, String)
  choose-pair "selected"

choose-root is fn (value : Int) -> Int
  value
    <= 0 then root live-number
    otherwise choose-root (value - 1)

choose-root is fn (value : String) -> String
  root live-label

forward-root-number is fn () -> Int
  choose-root 2

forward-root-label is fn () -> String
  choose-root "selected"

cross is fn (value : Int) -> Int
  @ context-number + (root live-number)

cross is fn (value : String) -> Int
  cross 0

forward-cross is fn () -> Int
  cross "selected"

choose-product is fn (value : (Int, String)) -> Int
  @ context-number

choose-product is fn (value : Record (amount : Int, label : String)) -> String
  root live-label

forward-product-tuple is fn () -> Int
  choose-product (1, "tuple")

forward-product-record is fn () -> String
  choose-product (amount is 1, label is "record")

live-pair is (7, "root-pair")
live-number is 7
live-label is "root"
(
  forward-context-number (),
  forward-context-label (),
  forward-context-pair (),
  forward-root-pair (),
  forward-root-number (),
  forward-root-label (),
  forward-cross (),
  forward-product-tuple (),
  forward-product-record ()
)
