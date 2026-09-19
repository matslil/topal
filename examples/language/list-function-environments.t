#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact finite Lists of Function identities and immutable
# environments across private parameters, results, decisions, and products.
context-offset is 40

increment is fn (value : Int) -> Int
  value + 1

return-list is fn (candidate : List Function) -> List Function
  candidate

return-tuple is fn (package : (List Function, Int)) -> (List Function, Int)
  package

apply-first is fn (candidate : List Function, value : Int) -> Int
  candidate
    Entry (operation, remaining) then operation value
    Empty then 0

apply-second is fn (candidate : List Function, value : Int) -> Int
  candidate
    Entry (ignored, remaining) then apply-first (remaining, value)
    Empty then 0

has-entry is fn (candidate : List Function) -> Int
  candidate
    Entry (operation, remaining) then 1
    Empty then 0

apply-package is fn ((candidate : List Function, value : Int)) -> Int
  apply-second (candidate, value)

apply-record is fn (package : Record (candidate : List Function, value : Int)) -> Int
  apply-second (package candidate, package value)

apply-tuple : Function is { (candidate, value) } apply-second (candidate, value)

make-list is fn (offset : Int) -> List Function
  increase is fn (value : Int) -> Int
    value + offset + @ context-offset + (root live-offset)
  Entry (increment, Entry (increase, Empty))

make-record is fn (offset : Int, value : Int) -> Record (candidate : List Function, value : Int)
  (candidate is make-list offset, value is value)

named-list is fn (_ : Unit) -> List Function
  Entry (increment, Empty)

symbolic-list is fn (_ : Unit) -> List Function
  Entry (+, Empty)

anonymous-list is fn (offset : Int) -> List Function
  Entry ({ value } value + offset, Empty)

live-offset is 1
first is make-list 1
second is make-list 2
forwarded is return-list (make-list 3)
package is make-record (4, 1)
empty : List Function is Empty

(
  apply-second (first, 1),
  apply-second (second, 1),
  apply-second (forwarded, 1),
  apply-first (anonymous-list 4, 1),
  apply-first (named-list (), 41),
  symbolic-list (),
  apply-record package,
  apply-package (candidate is make-list 5, value is 1),
  apply-tuple (return-tuple (make-list 6, 1)),
  has-entry first,
  has-entry empty,
  first,
  empty
)
