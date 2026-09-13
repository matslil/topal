#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates recursively decomposed Tuple results from every admitted decision
# family while preserving the selected fields and their source classifiers.
Choice is Enum (Left, Right)

choose-boolean is fn (condition : Boolean) -> (Int, String)
  condition
    true then (42, "true")
    false then (0, "false")

choose-ordered is fn (value : Int) -> (Int, String)
  value
    < 0 then (value, "negative")
    otherwise (value, "nonnegative")

choose-comparison is fn (ordering : Comparison) -> (Int, String)
  ordering
    Less then (-1, "less")
    Equal then (0, "equal")
    Greater then (1, "greater")

choose-enum is fn (choice : Choice) -> (Int, String)
  choice
    Left then (0, "left")
    Right then (1, "right")

choose-optional is fn (candidate : Optional Int) -> (Int, String)
  candidate
    Some value then (value, "some")
    None then (0, "none")

divide is fn (denominator : Rational) -> Result (Rational, lang arithmetic ArithmeticErrorCode)
  1.0 / denominator

choose-result is fn (denominator : Rational) -> (Rational, String)
  divide denominator
    Ok value then (value, "ok")
    Error problem then (0.0, "error")

(choose-boolean true, choose-boolean false, choose-ordered (negate 2), choose-ordered 2, choose-comparison (1 <=> 2), choose-enum Right, choose-optional (Some 7), choose-optional (None Int), choose-result 2.0, choose-result 0.0)
