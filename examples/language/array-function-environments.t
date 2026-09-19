#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact fixed-size Arrays of Function identities and immutable
# environments across private parameters, results, indexing, and products.
context-offset is 40

increment is fn (value : Int) -> Int
  value + 1

return-array is fn (candidate : Array (2, Function)) -> Array (2, Function)
  candidate

return-tuple is fn (package : (Array (2, Function), Int)) -> (Array (2, Function), Int)
  package

apply-first is fn (candidate : Array (2, Function), value : Int) -> Int
  array-at? (candidate, 0)
    Some operation then operation value
    None then 0

apply-second is fn (candidate : Array (2, Function), value : Int) -> Int
  array-at? (candidate, 1)
    Some operation then operation value
    None then 0

apply-only is fn (candidate : Array (1, Function), value : Int) -> Int
  array-at? (candidate, 0)
    Some operation then operation value
    None then 0

missing is fn (candidate : Array (2, Function)) -> Int
  array-at? (candidate, 2)
    Some ignored then 1
    None then 0

apply-package is fn ((candidate : Array (2, Function), value : Int)) -> Int
  apply-second (candidate, value)

apply-record is fn (package : Record (candidate : Array (2, Function), value : Int)) -> Int
  apply-second (package candidate, package value)

apply-tuple : Function is { (candidate, value) } apply-second (candidate, value)

make-array is fn (offset : Int) -> Array (2, Function)
  increase is fn (value : Int) -> Int
    value + offset + @ context-offset + (root live-offset)
  values : List Function is Entry (increment, Entry (increase, Empty))
  values collect Array

make-record is fn (offset : Int, value : Int) -> Record (candidate : Array (2, Function), value : Int)
  (candidate is make-array offset, value is value)

named-array is fn (_ : Unit) -> Array (1, Function)
  values : List Function is Entry (increment, Empty)
  values collect Array

symbolic-array is fn (_ : Unit) -> Array (1, Function)
  values : List Function is Entry (+, Empty)
  values collect Array

anonymous-array is fn (offset : Int) -> Array (1, Function)
  values : List Function is Entry ({ value } value + offset, Empty)
  values collect Array

empty-array is fn (_ : Unit) -> Array (0, Function)
  values : List Function is Empty
  values collect Array

live-offset is 1
first is make-array 1
second is make-array 2
forwarded is return-array (make-array 3)
package is make-record (4, 1)
empty is empty-array ()

(
  apply-first (first, 1),
  apply-second (first, 1),
  apply-second (second, 1),
  apply-second (forwarded, 1),
  apply-only (anonymous-array 4, 1),
  apply-only (named-array (), 41),
  symbolic-array (),
  apply-record package,
  apply-package (candidate is make-array 5, value is 1),
  apply-tuple (return-tuple (make-array 6, 1)),
  entry-count first,
  missing first,
  empty? empty,
  first,
  empty
)
