#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact Maps from String keys to Function identities and immutable
# environments across collision policies, private boundaries, and products.
context-offset is 40

increment is fn (value : Int) -> Int
  value + 1

return-map is fn (candidate : Map (String, Function)) -> Map (String, Function)
  candidate

return-tuple is fn (package : (Map (String, Function), Int)) -> (Map (String, Function), Int)
  package

apply-increment is fn (candidate : Map (String, Function), value : Int) -> Int
  map-lookup (candidate, "increment")
    Some operation then operation value
    None then 0

apply-increase is fn (candidate : Map (String, Function), value : Int) -> Int
  map-lookup (candidate, "increase")
    Some operation then operation value
    None then 0

apply-only is fn (candidate : Map (String, Function), value : Int) -> Int
  map-lookup (candidate, "only")
    Some operation then operation value
    None then 0

apply-symbolic is fn (candidate : Map (String, Function)) -> Int
  map-lookup (candidate, "+")
    Some operation then operation (1, 2)
    None then 0

apply-operation is fn (candidate : Map (String, Function), value : Int) -> Int
  map-lookup (candidate, "operation")
    Some operation then operation value
    None then 0

missing is fn (candidate : Map (String, Function)) -> Int
  map-lookup (candidate, "missing")
    Some ignored then 1
    None then 0

apply-package is fn ((candidate : Map (String, Function), value : Int)) -> Int
  apply-increase (candidate, value)

apply-record is fn (package : Record (candidate : Map (String, Function), value : Int)) -> Int
  apply-increase (package candidate, package value)

apply-tuple : Function is { (candidate, value) } apply-increase (candidate, value)

make-map is fn (offset : Int) -> Map (String, Function)
  increase is fn (value : Int) -> Int
    value + offset + @ context-offset + (root live-offset)
  pairs : List (String, Function) is Entry (("increment", increment), Entry (("increase", increase), Empty))
  collect-map pairs resolving keep-last

make-record is fn (offset : Int, value : Int) -> Record (candidate : Map (String, Function), value : Int)
  (candidate is make-map offset, value is value)

named-map is fn (_ : Unit) -> Map (String, Function)
  pairs : List (String, Function) is Entry (("only", increment), Empty)
  collect-map pairs resolving reject

symbolic-map is fn (_ : Unit) -> Map (String, Function)
  pairs : List (String, Function) is Entry (("+", +), Empty)
  collect-map pairs resolving keep-first

anonymous-map is fn (offset : Int) -> Map (String, Function)
  pairs : List (String, Function) is Entry (("only", { value } value + offset), Empty)
  collect-map pairs resolving keep-last

keep-first-map is fn (offset : Int) -> Map (String, Function)
  increase is fn (value : Int) -> Int
    value + offset
  pairs : List (String, Function) is Entry (("operation", increment), Entry (("operation", increase), Empty))
  collect-map pairs resolving keep-first

keep-last-map is fn (offset : Int) -> Map (String, Function)
  increase is fn (value : Int) -> Int
    value + offset
  pairs : List (String, Function) is Entry (("operation", increment), Entry (("operation", increase), Empty))
  collect-map pairs resolving keep-last

live-offset is 1
first is make-map 1
second is make-map 2
forwarded is return-map (make-map 3)
package is make-record (4, 1)

(
  apply-increment (first, 1),
  apply-increase (first, 1),
  apply-increase (second, 1),
  apply-increase (forwarded, 1),
  apply-only (anonymous-map 4, 1),
  apply-only (named-map (), 41),
  apply-symbolic (symbolic-map ()),
  apply-record package,
  apply-package (candidate is make-map 5, value is 1),
  apply-tuple (return-tuple (make-map 6, 1)),
  apply-operation (keep-first-map 7, 1),
  apply-operation (keep-last-map 7, 1),
  entry-count first,
  missing first,
  empty? first,
  first
)
