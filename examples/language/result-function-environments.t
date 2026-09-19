#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact Function identities and immutable environments carried by
# successful Result values across private parameters, results, and products.
context-offset is 40

increment is fn (value : Int) -> Int
  value + 1

return-result is fn (candidate : Result (Function, lang arithmetic ArithmeticErrorCode)) -> Result (Function, lang arithmetic ArithmeticErrorCode)
  candidate

project-result is fn (candidate : Result (Function, lang arithmetic ArithmeticErrorCode)) -> Result (Function, lang arithmetic ArithmeticErrorCode)
  operation : Function is candidate
  operation

return-tuple is fn (package : (Result (Function, lang arithmetic ArithmeticErrorCode), Int)) -> (Result (Function, lang arithmetic ArithmeticErrorCode), Int)
  package

apply-result is fn (candidate : Result (Function, lang arithmetic ArithmeticErrorCode), value : Int) -> Int
  candidate
    Ok operation then operation value
    Error problem then 0

has-operation is fn (candidate : Result (Function, lang arithmetic ArithmeticErrorCode)) -> Int
  candidate
    Ok ignored then 1
    Error problem then 0

apply-package is fn ((candidate : Result (Function, lang arithmetic ArithmeticErrorCode), value : Int)) -> Int
  candidate
    Ok operation then operation value
    Error problem then 0

apply-record is fn (package : Record (candidate : Result (Function, lang arithmetic ArithmeticErrorCode), value : Int)) -> Int
  package candidate
    Ok operation then operation (package value)
    Error problem then 0

apply-tuple : Function is { (candidate, value) } apply-result (candidate, value)

make-result is fn (offset : Int) -> Result (Function, lang arithmetic ArithmeticErrorCode)
  increase is fn (value : Int) -> Int
    value + offset + @ context-offset + (root live-offset)
  increase

make-record is fn (offset : Int, value : Int) -> Record (candidate : Result (Function, lang arithmetic ArithmeticErrorCode), value : Int)
  (candidate is make-result offset, value is value)

make-fallible is fn (offset : Int, denominator : Rational) -> Result (Function, lang arithmetic ArithmeticErrorCode)
  quotient : Rational is 1.0 / denominator
  increase is fn (value : Int) -> Int
    value + offset + @ context-offset + (root live-offset)
  increase

named-result is fn (_ : Unit) -> Result (Function, lang arithmetic ArithmeticErrorCode)
  increment

symbolic-result is fn (_ : Unit) -> Result (Function, lang arithmetic ArithmeticErrorCode)
  +

anonymous-result is fn (offset : Int) -> Result (Function, lang arithmetic ArithmeticErrorCode)
  { value } value + offset

live-offset is 1
first is make-result 1
second is make-result 2
forwarded is return-result (make-result 3)
package is make-record (4, 1)
successful is make-fallible (7, 2.0)
failed is make-fallible (8, 0.0)
projected is project-result (make-result 7)
projected-failure is project-result failed

(
  apply-result (first, 1),
  apply-result (second, 1),
  apply-result (forwarded, 1),
  apply-result (anonymous-result 4, 1),
  apply-result (named-result (), 41),
  symbolic-result (),
  apply-record package,
  apply-package (candidate is make-result 5, value is 1),
  apply-tuple (return-tuple (make-result 6, 1)),
  apply-result (successful, 1),
  apply-result (projected, 1),
  apply-result (failed, 1),
  has-operation failed,
  has-operation projected-failure,
  first,
  projected-failure
)
