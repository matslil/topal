#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact Function identities and immutable environments carried by
# present Optional values across private parameters, results, and containers.
context-offset is 40

increment is fn (value : Int) -> Int
  value + 1

return-optional is fn (candidate : Optional Function) -> Optional Function
  candidate

return-tuple is fn (package : (Optional Function, Int)) -> (Optional Function, Int)
  package

apply-optional is fn (candidate : Optional Function, value : Int) -> Int
  candidate
    Some operation then operation value
    None then 0

has-operation is fn (candidate : Optional Function) -> Int
  candidate
    Some ignored then 1
    None then 0

apply-package is fn ((candidate : Optional Function, value : Int)) -> Int
  candidate
    Some operation then operation value
    None then 0

apply-record is fn (package : Record (candidate : Optional Function, value : Int)) -> Int
  package candidate
    Some operation then operation (package value)
    None then 0

apply-tuple : Function is { (candidate, value) } apply-optional (candidate, value)

make-optional is fn (offset : Int) -> Optional Function
  increase is fn (value : Int) -> Int
    value + offset + @ context-offset + (root live-offset)
  Some increase

make-record is fn (offset : Int, value : Int) -> Record (candidate : Optional Function, value : Int)
  increase is fn (operand : Int) -> Int
    operand + offset + @ context-offset + (root live-offset)
  (candidate is Some increase, value is value)

anonymous-offset is 4
anonymous-optional : Optional Function is Some { value } value + anonymous-offset
named-optional : Optional Function is Some increment
symbolic-optional : Optional Function is Some +
missing : Optional Function is None

same-optional : Function is { candidate, candidate } 1

live-offset is 1
first is make-optional 1
second is make-optional 2
forwarded is return-optional (make-optional 3)
package is make-record (4, 1)

(
  apply-optional (first, 1),
  apply-optional (second, 1),
  apply-optional (forwarded, 1),
  apply-optional (anonymous-optional, 1),
  apply-optional (named-optional, 41),
  symbolic-optional,
  apply-record package,
  apply-package (candidate is make-optional 5, value is 1),
  apply-tuple (return-tuple (make-optional 6, 1)),
  has-operation missing,
  same-optional (first, first),
  first,
  missing
)
