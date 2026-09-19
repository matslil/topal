#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact Function identities and immutable environments carried by
# selected nominal Sum payloads across private parameters, results, and products.
Operation is Union
  Apply : Function
  Unavailable

Choice is Variant (Function, Int)

context-offset is 40

increment is fn (value : Int) -> Int
  value + 1

return-operation is fn (candidate : Operation) -> Operation
  candidate

apply-operation is fn (candidate : Operation, value : Int) -> Int
  candidate
    Apply operation then operation value
    Unavailable then 0

has-operation is fn (candidate : Operation) -> Int
  candidate
    Apply operation then 1
    otherwise 0

apply-package is fn ((candidate : Operation, value : Int)) -> Int
  candidate
    Apply operation then operation value
    Unavailable then 0

apply-choice is fn (candidate : Choice, value : Int) -> Int
  candidate
    Choice at 0 operation then operation value
    Choice at 1 number then number

make-operation is fn (offset : Int) -> Operation
  increase is fn (value : Int) -> Int
    value + offset + @ context-offset + (root live-offset)
  Apply increase

make-choice is fn (offset : Int) -> Choice
  increase is fn (value : Int) -> Int
    value + offset + @ context-offset + (root live-offset)
  Choice at 0 increase

anonymous-offset is 4
anonymous-operation : Operation is Apply { value } value + anonymous-offset
named-operation : Operation is Apply increment
symbolic-operation : Operation is Apply +
missing : Operation is Unavailable

same-operation : Function is { candidate, candidate } 1

live-offset is 1
first is make-operation 1
second is make-operation 2
forwarded is return-operation (make-operation 3)
(
  apply-operation (first, 1),
  apply-operation (second, 1),
  apply-operation (forwarded, 1),
  apply-operation (anonymous-operation, 1),
  apply-operation (named-operation, 41),
  symbolic-operation,
  apply-package (candidate is make-operation 4, value is 1),
  apply-package (make-operation 5, 1),
  apply-choice (make-choice 6, 1),
  has-operation missing,
  same-operation (first, first),
  first,
  missing
)
