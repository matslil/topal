#!/usr/bin/env topal
use language (
  version is v0.1
)
use library std (
  version is v0.1
)

# Parse the ordinary horizontal numbers, transpose problems, and evaluate them.
lines is std text lines
words is std text words
parse-int is std parse int
integer-rows is std parse integer-rows
take is std sequence take
transpose is std sequence transpose
zip is std sequence zip

required-string is fn (candidate : Optional String) -> String
  candidate
    Some value then value
    None then ""
required-int is fn (candidate : Optional Int) -> Int
  candidate
    Some value then value
    None then 0

evaluate is fn (problem : (List Int, String)) -> Int
  values-of is fn (values : List Int, operation : String) -> List Int
    values
  operation-of is fn (values : List Int, operation : String) -> String
    operation
  values is values-of problem
  operation-of problem
    = "*" then values fold 1 { total, value } total * value
    otherwise values fold 0 { total, value } total + value

solve is fn (input : String) -> Int
  rows is lines input
  operation-line is required-string (first (rows reverse))
  _ is take (rows, (entry-count rows) - 1)
  number-rows is integer-rows input
  problems is transpose number-rows
  operators is words operation-line
  (zip (problems, operators)) fold 0 { total, problem } total + (evaluate problem)
