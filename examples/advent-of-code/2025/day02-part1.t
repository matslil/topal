#!/usr/bin/env topal
use language (
  version is v0.1
)
use library std (
  version is v0.1
)

# Generate doubled decimal prefixes instead of scanning the large ID ranges.
split is std pattern split
unsigned-integers is std parse unsigned-integers
decimal is std parse decimal
parse-int is std parse int
repeat is std repeat
range-values is std sequence values
no-ranges : List (Int, Int) is Empty

required-int is fn (candidate : Optional Int) -> Int
  candidate
    Some value then value
    None then 0
required-int-callable is required-int

range-pair is fn (text : String) -> (Int, Int)
  values is unsigned-integers text
  lower-value is fn (candidate : Optional Nat) -> Int
    candidate
      Some value then value
      None then 0
  upper-list is fn (candidate : Optional (List Nat)) -> List Nat
    candidate
      Some value then value
      None then Empty Nat
  lower-value-callable is lower-value
  upper-list-callable is upper-list
  (lower-value-callable (first values), lower-value-callable (first (upper-list-callable (rest values))))
range-pair-callable is range-pair

append-range is fn (ranges : List (Int, Int), text : String) -> List (Int, Int)
  ranges append (range-pair-callable text)
append-range-callable is append-range

upper-end is fn (range : (Int, Int)) -> Int
  select is fn (lower : Int, upper : Int) -> Int
    upper
  select range
upper-end-callable is upper-end

larger is fn (left : Int, right : Int) -> Int
  left >= right
    true then left
    false then right
larger-callable is larger

contains-id is fn (range : (Int, Int), id : Int) -> Boolean
  check is fn (lower : Int, upper : Int) -> Boolean
    (lower <= id) and (id <= upper)
  check range
contains-id-callable is contains-id

retain-id is fn (state : (Boolean, Int), range : (Int, Int)) -> (Boolean, Int)
  found-value is fn (found : Boolean, id : Int) -> Boolean
    found
  id-value is fn (found : Boolean, id : Int) -> Int
    id
  found is found-value state
  id is id-value state
  (found or (contains-id-callable (range, id)), id)
retain-id-callable is retain-id

id-if-contained is fn (ranges : List (Int, Int), id : Int) -> Int
  state is ranges fold (false, id) { found, range } retain-id-callable (found, range)
  choose is fn (found : Boolean, value : Int) -> Int
    found
      true then value
      false then 0
  choose state
id-if-contained-callable is id-if-contained

candidate-sum is fn (ranges : List (Int, Int), prefix : Int) -> Int
  text is decimal prefix
  id is required-int-callable (parse-int (repeat (text, 2)))
  id-if-contained-callable (ranges, id)
candidate-sum-callable is candidate-sum

add-candidate is fn (state : (Int, List (Int, Int)), prefix : Int) -> (Int, List (Int, Int))
  sum-value is fn (sum : Int, ranges : List (Int, Int)) -> Int
    sum
  ranges-value is fn (sum : Int, ranges : List (Int, Int)) -> List (Int, Int)
    ranges
  ranges is ranges-value state
  ((sum-value state) + (candidate-sum-callable (ranges, prefix)), ranges)
add-candidate-callable is add-candidate

answer-value is fn (state : (Int, List (Int, Int))) -> Int
  select is fn (sum : Int, ranges : List (Int, Int)) -> Int
    sum
  select state
answer-value-callable is answer-value

solve is fn (input : String) -> Int
  ranges is (split (input, ",")) fold no-ranges { selected, text } append-range-callable (selected, text)
  maximum is ranges fold 0 { selected, range } larger-callable (selected, upper-end-callable range)
  digits is entry-count (decimal maximum)
  quotient is fn (value : Int, remainder : Int) -> Int
    value
  prefix-digits is quotient ((digits + 1) /% 2)
  limit is (10 ^ prefix-digits) - 1
  answer-value-callable ((range-values (1 ..= limit)) fold (0, ranges) { state, prefix } add-candidate-callable (state, prefix))
