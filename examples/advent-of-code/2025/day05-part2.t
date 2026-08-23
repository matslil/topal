#!/usr/bin/env topal
use language (
  version is v0.1
)
use library std (
  version is v0.1
)

# Coalesce fresh ranges and sum their exact inclusive cardinalities.
split is std pattern split
lines is std text lines
unsigned-integers is std parse unsigned-integers
coalesce is std coalesce
no-ranges : List (Int, Int) is Empty
section-break is text"

"text

required-section is fn (candidate : Optional String) -> String
  candidate
    Some value then value
    None then ""
required-section-callable is required-section
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

range-size is fn (range : (Int, Int)) -> Int
  calculate is fn (lower : Int, upper : Int) -> Int
    upper - lower + 1
  calculate range
range-size-callable is range-size

solve is fn (input : String) -> Int
  sections is split (input, section-break)
  range-section is required-section-callable (first sections)
  ranges is (lines range-section) fold no-ranges { selected, line } append-range-callable (selected, line)
  (coalesce ranges) fold 0 { total, range } total + (range-size-callable range)
