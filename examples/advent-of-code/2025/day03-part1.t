#!/usr/bin/env topal
use language (
  version is v0.1
)
use library std (
  version is v0.1
)

# Greedily select the lexicographically largest two-digit subsequence.
lines is std text lines
digits is std parse decimal-digits
take is std sequence take
drop is std sequence drop
range-values is std sequence values

required-index is fn (candidate : Optional Nat) -> Nat
  candidate
    Some value then value
    None then 0
required-index-callable is required-index

larger is fn (left : Nat, right : Nat) -> Nat
  left >= right
    true then left
    false then right
larger-callable is larger

remaining-of is fn ((remaining : Int, values : List Nat, accumulated : Int)) -> Int
  remaining
remaining-of-callable is remaining-of
values-of is fn ((remaining : Int, values : List Nat, accumulated : Int)) -> List Nat
  values
values-of-callable is values-of
accumulated-of is fn ((remaining : Int, values : List Nat, accumulated : Int)) -> Int
  accumulated
accumulated-of-callable is accumulated-of

pick-step is fn (state : (Int, List Nat, Int), iteration : Int) -> (Int, List Nat, Int)
  _ is iteration
  remaining is remaining-of-callable state
  values is values-of-callable state
  accumulated is accumulated-of-callable state
  available is (entry-count values) - remaining + 1
  selected is (take (values, available)) fold 0 { maximum, value } larger-callable (maximum, value)
  index is required-index-callable (values list-index-of selected)
  (remaining - 1, drop (values, index + 1), accumulated * 10 + selected)
pick-step-callable is pick-step

pick is fn (count : Nat, values : List Nat) -> Int
  final is (range-values (1 ..= count)) fold (count, values, 0) { state, iteration } pick-step-callable (state, iteration)
  accumulated-of-callable final
pick-callable is pick

add-bank is fn (total : Int, line : String) -> Int
  total + (pick-callable (2, digits line))
add-bank-callable is add-bank

solve is fn (input : String) -> Int
  (lines input) fold 0 { total, line } add-bank-callable (total, line)
