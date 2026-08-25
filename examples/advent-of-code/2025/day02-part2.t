#!/usr/bin/env topal
use language (
  version is v0.1
)
use library std (
  version is v0.1
)

# Generate every decimal prefix repeated at least twice, then deduplicate IDs.
split is std pattern split
unsigned-integers is std parse unsigned-integers
decimal is std parse decimal
parse-int is std parse int
repeat is std repeat
range-values is std sequence values
unique is std sequence unique
no-ranges : List (Int, Int) is Empty

required-int is fn (candidate : Optional Int) -> Int
  candidate
    Some value then value
    None then 0

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
  (lower-value (first values), lower-value (first (upper-list (rest values))))

append-range is fn (ranges : List (Int, Int), text : String) -> List (Int, Int)
  ranges append (range-pair text)

upper-end is fn (range : (Int, Int)) -> Int
  select is fn (lower : Int, upper : Int) -> Int
    upper
  select range

larger is fn (left : Int, right : Int) -> Int
  left >= right
    true then left
    false then right

contains-id is fn (range : (Int, Int), id : Int) -> Boolean
  check is fn (lower : Int, upper : Int) -> Boolean
    (lower <= id) and (id <= upper)
  check range

find-id is fn (state : (Boolean, Int), range : (Int, Int)) -> (Boolean, Int)
  found-value is fn (found : Boolean, id : Int) -> Boolean
    found
  id-value is fn (found : Boolean, id : Int) -> Int
    id
  found is found-value state
  id is id-value state
  (found or (contains-id (range, id)), id)

contained? is fn (ranges : List (Int, Int), id : Int) -> Boolean
  final is ranges fold (false, id) { state, range } find-id (state, range)
  select is fn (found : Boolean, value : Int) -> Boolean
    found
  select final

append-when is fn (values : List Int, (accepted : Boolean, candidate : Int)) -> List Int
  accepted
    true then values append candidate
    false then values

candidates-for is fn (
  ranges : List (Int, Int),
  (maximum-digits : Nat, prefix : Int)
) -> List Int
  text is decimal prefix
  add-repeat is fn (values : List Int, repetitions : Int) -> List Int
    id is required-int (parse-int (repeat (text, repetitions)))
    append-when (values, (contained? (ranges, id), id))
  (range-values (2 ..= maximum-digits)) fold (Empty Int) { values, repetitions } add-repeat (values, repetitions)

append-prefix is fn (
  state : (List Int, List (Int, Int), Nat),
  prefix : Int
) -> (List Int, List (Int, Int), Nat)
  values-of is fn ((values : List Int, ranges : List (Int, Int), digits : Nat)) -> List Int
    values
  ranges-of is fn ((values : List Int, ranges : List (Int, Int), digits : Nat)) -> List (Int, Int)
    ranges
  digits-of is fn ((values : List Int, ranges : List (Int, Int), digits : Nat)) -> Nat
    digits
  values is values-of state
  ranges is ranges-of state
  digits is digits-of state
  (values concat (candidates-for (ranges, (digits, prefix))), ranges, digits)

values-of-state is fn ((values : List Int, ranges : List (Int, Int), digits : Nat)) -> List Int
  values

solve is fn (input : String) -> Int
  ranges is (split (input, ",")) fold no-ranges { selected, text } append-range (selected, text)
  maximum is ranges fold 0 { selected, range } larger (selected, upper-end range)
  digits is entry-count (decimal maximum)
  quotient is fn (value : Int, remainder : Int) -> Int
    value
  prefix-digits is quotient ((digits + 1) /% 2)
  limit is (10 ^ prefix-digits) - 1
  state is (range-values (1 ..= limit)) fold ((Empty Int), ranges, digits) { selected, prefix } append-prefix (selected, prefix)
  (unique (values-of-state state)) fold 0 { total, value } total + value
