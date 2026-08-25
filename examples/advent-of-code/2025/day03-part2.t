#!/usr/bin/env topal
use language (
  version is v0.1
)
use library std (
  version is v0.1
)

# Greedily select the lexicographically largest twelve-digit subsequence.
lines is std text lines
digits is std parse decimal-digits
take is std sequence take
drop is std sequence drop
range-values is std sequence values

required-index is fn (candidate : Optional Nat) -> Nat
  candidate
    Some value then value
    None then 0

larger is fn (left : Nat, right : Nat) -> Nat
  left >= right
    true then left
    false then right

remaining-of is fn ((remaining : Int, values : List Nat, accumulated : Int)) -> Int
  remaining
values-of is fn ((remaining : Int, values : List Nat, accumulated : Int)) -> List Nat
  values
accumulated-of is fn ((remaining : Int, values : List Nat, accumulated : Int)) -> Int
  accumulated

pick-step is fn (state : (Int, List Nat, Int), iteration : Int) -> (Int, List Nat, Int)
  _ is iteration
  remaining is remaining-of state
  values is values-of state
  accumulated is accumulated-of state
  available is (entry-count values) - remaining + 1
  selected is (take (values, available)) fold 0 { maximum, value } larger (maximum, value)
  index is required-index (values list-index-of selected)
  (remaining - 1, drop (values, index + 1), accumulated * 10 + selected)

pick is fn (count : Nat, values : List Nat) -> Int
  final is (range-values (1 ..= count)) fold (count, values, 0) { state, iteration } pick-step (state, iteration)
  accumulated-of final

add-bank is fn (total : Int, line : String) -> Int
  total + (pick (12, digits line))

solve is fn (input : String) -> Int
  (lines input) fold 0 { total, line } add-bank (total, line)
