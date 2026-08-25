#!/usr/bin/env topal
use language (
  version is v0.1
)
use library std (
  version is v0.1
)

# Read each fixed-width digit column as one number and evaluate blank-separated problems.
lines is std text lines
vertical-integers is std parse vertical-integers
character-list is std parse character-list
zip is std sequence zip

required-string is fn (candidate : Optional String) -> String
  candidate
    Some value then value
    None then ""
required-string-callable is required-string

total-of is fn ((total : Int, values : List Int, operation : Character)) -> Int
  total
total-of-callable is total-of
values-of is fn ((total : Int, values : List Int, operation : Character)) -> List Int
  values
values-of-callable is values-of
operation-of is fn ((total : Int, values : List Int, operation : Character)) -> Character
  operation
operation-of-callable is operation-of

selected-operation is fn (current : Character, candidate : Character) -> Character
  candidate
    = "*" then candidate
    = "+" then candidate
    otherwise current
selected-operation-callable is selected-operation

evaluate is fn (values : List Int, operation : Character) -> Int
  operation
    = "*" then values fold 1 { result, value } result * value
    otherwise values fold 0 { result, value } result + value
evaluate-callable is evaluate

present-column is fn (
  state : (Int, List Int, Character),
  (value : Int, operation : Character)
) -> (Int, List Int, Character)
  (total-of-callable state,
   (values-of-callable state) append value,
   selected-operation-callable (operation-of-callable state, operation))
present-column-callable is present-column

blank-column is fn (state : (Int, List Int, Character)) -> (Int, List Int, Character)
  values is values-of-callable state
  ((total-of-callable state) + (evaluate-callable (values, operation-of-callable state)), Empty Int, "+")
blank-column-callable is blank-column

process-column is fn (
  state : (Int, List Int, Character),
  column : (Optional Int, Character)
) -> (Int, List Int, Character)
  number-of is fn (number : Optional Int, operation : Character) -> Optional Int
    number
  operation-of-column is fn (number : Optional Int, operation : Character) -> Character
    operation
  number-of column
    Some value then present-column-callable (state, (value, operation-of-column column))
    None then blank-column-callable state
process-column-callable is process-column

solve is fn (input : String) -> Int
  rows is lines input
  operation-line is required-string-callable (first (rows reverse))
  numbers is (vertical-integers input) append (None Int)
  operations is (character-list operation-line) append " "
  columns is zip (numbers, operations)
  final is columns fold (0, (Empty Int), "+") { state, column } process-column-callable (state, column)
  total-of-callable final
