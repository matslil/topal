#!/usr/bin/env topal
use language (
  version is v0.1
)
use library std (
  version is v0.1
)

# Count available ingredient IDs covered by at least one fresh inclusive range.
split is std pattern split
lines is std text lines
unsigned-integers is std parse unsigned-integers
no-ranges : List (Int, Int) is Empty
section-break is text"

"text

required-section is fn (candidate : Optional String) -> String
  candidate
    Some value then value
    None then ""
required-section-callable is required-section
remaining-sections is fn (candidate : Optional (List String)) -> List String
  candidate
    Some value then value
    None then Empty String
remaining-sections-callable is remaining-sections
required-nat is fn (candidate : Optional Nat) -> Int
  candidate
    Some value then value
    None then 0
required-nat-callable is required-nat
remaining-nats is fn (candidate : Optional (List Nat)) -> List Nat
  candidate
    Some value then value
    None then Empty Nat
remaining-nats-callable is remaining-nats

as-range is fn (line : String) -> (Int, Int)
  values is unsigned-integers line
  lower is required-nat-callable (first values)
  upper is required-nat-callable (first (remaining-nats-callable (rest values)))
  (lower, upper)
as-range-callable is as-range

append-range is fn (ranges : List (Int, Int), line : String) -> List (Int, Int)
  ranges append (as-range-callable line)
append-range-callable is append-range

contains-id is fn (found : Boolean, (range : (Int, Int), id : Int)) -> Boolean
  check is fn (lower : Int, upper : Int) -> Boolean
    (lower <= id) and (id <= upper)
  found or (check range)
contains-id-callable is contains-id

count-id is fn (state : (Int, List (Int, Int)), id : Int) -> (Int, List (Int, Int))
  count-of is fn (count : Int, ranges : List (Int, Int)) -> Int
    count
  ranges-of is fn (count : Int, ranges : List (Int, Int)) -> List (Int, Int)
    ranges
  ranges is ranges-of state
  found is ranges fold false { selected, range } contains-id-callable (selected, (range, id))
  increment is fn (accepted : Boolean, count : Int) -> Int
    accepted
      true then count + 1
      false then count
  (increment (found, count-of state), ranges)
count-id-callable is count-id

solve is fn (input : String) -> Int
  sections is split (input, section-break)
  range-section is required-section-callable (first sections)
  id-section is required-section-callable (first (remaining-sections-callable (rest sections)))
  ranges is (lines range-section) fold no-ranges { selected, line } append-range-callable (selected, line)
  ids is unsigned-integers id-section
  final is ids fold (0, ranges) { state, id } count-id-callable (state, id)
  count-of is fn (count : Int, ranges : List (Int, Int)) -> Int
    count
  count-of final
