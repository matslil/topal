#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates structural Record parameters, results, decisions, nesting, field
# selection, and construction-order display across private function boundaries.
make-person is fn (name : String, active : Boolean) -> Record (name : String, active : Boolean)
  (name is name, active is active)

retain-person is fn (person : Record (active : Boolean, name : String)) -> Record (active : Boolean, name : String)
  person

choose-person is fn (condition : Boolean) -> Record (active : Boolean, name : String)
  condition
    true then (active is true, name is "first")
    false then (name is "second", active is false)

Choice is Enum (Left, Right)

choose-ordered is fn (value : Int) -> Record (label : String, value : Int)
  value
    < 0 then (value is value, label is "negative")
    otherwise (label is "nonnegative", value is value)

choose-comparison is fn (ordering : Comparison) -> Record (label : String, value : Int)
  ordering
    Less then (label is "less", value is -1)
    Equal then (value is 0, label is "equal")
    Greater then (label is "greater", value is 1)

choose-enum is fn (choice : Choice) -> Record (label : String, value : Int)
  choice
    Left then (value is 0, label is "left")
    Right then (label is "right", value is 1)

choose-optional is fn (candidate : Optional Int) -> Record (label : String, value : Int)
  candidate
    Some value then (value is value, label is "some")
    None then (label is "none", value is 0)

divide is fn (denominator : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  1.0 / denominator

choose-result is fn (denominator : Rational) -> Record (label : String, value : Rational)
  divide denominator
    Ok value then (label is "ok", value is value)
    Error problem then (value is 0.0, label is "error")

retain-wrapper is fn (wrapper : Record (person : Record (active : Boolean, name : String), score : Int)) -> Record (person : Record (active : Boolean, name : String), score : Int)
  wrapper

made is make-person ("Ada", true)
retained is retain-person (
  active is false,
  name is "Grace"
)
wrapped is retain-wrapper (
  score is 42,
  person is (
    name is "Lin",
    active is true
  )
)
(made, retained, choose-person true, choose-person false, wrapped, retained name, choose-ordered (negate 2), choose-ordered 2, choose-comparison (1 <=> 2), choose-enum Right, choose-optional (Some 7), choose-optional (None Int), choose-result 2.0, choose-result 0.0)
