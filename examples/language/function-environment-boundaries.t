#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact defining-context and live-root environments through
# private Function parameters, results, and Function-containing aggregates.
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

apply-int is fn (operation : Function, value : Int) -> Int
  operation value

forward-int is fn (operation : Function, value : Int) -> Int
  apply-int (operation, value)

return-operation is fn (operation : Function) -> Function
  operation

apply-record is fn (package : Record (context-operation : Function, root-operation : Function, pair-operation : Function, value : Int)) -> (Int, String, Int, String, (Int, String), (Int, String))
  (
    (package context-operation) (package value),
    (package context-operation) "selected",
    (package root-operation) (package value),
    (package root-operation) "selected",
    (package pair-operation) (package value),
    (package pair-operation) "selected"
  )

forward-record is fn (package : Record (context-operation : Function, root-operation : Function, pair-operation : Function, value : Int)) -> Record (context-operation : Function, root-operation : Function, pair-operation : Function, value : Int)
  package

apply-one is fn (package : Record (operation : Function, value : Int)) -> Int
  (package operation) (package value)

make-anonymous is fn () -> Function
  operation : Function is { value } value + @ context-number + (root live-number)
  return-operation operation

make-anonymous-record is fn () -> Record (operation : Function, value : Int)
  operation : Function is { value } value + @ context-number + (root live-number)
  (operation is operation, value is 2)

use-nested is fn () -> (Int, Int)
  increase is fn (value : Int) -> Int
    value + @ context-number + (root live-number)
  package is (operation is increase, value is 3)
  (forward-int (increase, 2), apply-one package)

live-number is 7
live-label is "root"
live-pair is (7, "root-pair")

context-operation is return-operation read-context
root-operation is return-operation read-root
pair-operation is return-operation read-pair
package is (
  context-operation is read-context,
  root-operation is read-root,
  pair-operation is read-pair,
  value is 2
)
anonymous-operation is make-anonymous ()
anonymous-package is make-anonymous-record ()

(
  context-operation 2,
  context-operation "selected",
  root-operation 2,
  root-operation "selected",
  pair-operation 0,
  pair-operation "selected",
  apply-record (forward-record package),
  anonymous-operation 1,
  apply-one anonymous-package,
  use-nested ()
)
