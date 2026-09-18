#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact namespace snapshots as complete package fields with
# reordered labeled calls, positional parity, and a closed root default.
answer is 40 + 2

increment is fn (value : Int) -> Int
  value + 1

api : Scope is root

make-value is fn () -> Int
  41

observe is fn ((scope : Scope, value : Int)) -> (Int, Int)
  (scope answer, value + 1)

observe-default is fn ((scope : Scope default root, value : Int)) -> (Int, Int)
  (scope answer, value + 1)

(
  observe (value is make-value (), scope is api),
  observe (api, 41),
  observe-default (value is make-value ())
)
