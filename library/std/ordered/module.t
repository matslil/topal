#!/usr/bin/env topal
use language (
  version is v0.1
)

### Revision of the ordered-algorithm namespace.
pub revision is 1

increment-when is fn (count : Int, accepted : Boolean) -> Int
  accepted
    true then count + 1
    false then count

### Return the insertion index before equal entries in an ascending List.
pub lower-bound is fn (values : List (Value : TotalOrder), sought : Value) -> Int
  values fold 0 { count, candidate } increment-when (count, candidate < sought)

### Return the insertion index after equal entries in an ascending List.
pub upper-bound is fn (values : List (Value : TotalOrder), sought : Value) -> Int
  values fold 0 { count, candidate } increment-when (count, candidate <= sought)

### Return the half-open index range containing entries equal to `sought`.
pub equal-range is fn (values : List (Value : TotalOrder), sought : Value) -> Range Int
  lower is lower-bound (values, sought)
  upper is upper-bound (values, sought)
  lower .. upper

insert-ordered is fn (candidate : (Value : TotalOrder), values : List Value) -> List Value
  boundary is lower-bound (values, candidate)
  length is entry-count values
  (values select-index (0 .. boundary)) concat (one candidate) concat (values select-index (boundary .. length))

sort-step is fn (candidate : (Value : TotalOrder), sorted : List Value) -> List Value
  insert-ordered (candidate, sorted)

sort-source is fn (values : List (Value : TotalOrder)) -> List Value
  sorted : List Value is Empty
  values fold sorted { collected, candidate } sort-step (candidate, collected)

### Return Int entries in ascending order, preserving the order of equal entries.
pub sort is fn (values : List Int) -> List Int
  sort-source values

### Return Rational entries in ascending order, preserving equal-entry order.
pub sort is fn (values : List Rational) -> List Rational
  sort-source values

### Return Int entries in descending order, preserving the order of equal entries.
pub sort-descending is fn (values : List Int) -> List Int
  (sort-source values) reverse

### Return Rational entries in descending order, preserving equal-entry order.
pub sort-descending is fn (values : List Rational) -> List Rational
  (sort-source values) reverse

present-index is fn (boundary : Nat) -> Optional Nat
  present : Optional Nat is Some boundary
  present

index-if-equal is fn ((candidate : (Value : TotalOrder), sought : Value, boundary : Nat)) -> Optional Nat
  candidate = sought
    true then present-index boundary
    false then None Nat

search-source is fn (values : List (Value : TotalOrder), sought : Value) -> Optional Nat
  boundary is lower-bound (values, sought)
  first (values select-index (boundary ..= boundary))
    Some candidate then index-if-equal (candidate, sought, boundary)
    None then None Nat

### Find an equal Int in an ascending List.
pub binary-search is fn (values : List Int, sought : Int) -> Optional Nat
  search-source (values, sought)

### Find an equal Rational in an ascending List.
pub binary-search is fn (values : List Rational, sought : Rational) -> Optional Nat
  search-source (values, sought)

### Merge two ascending Int Lists into one ascending List.
pub merge is fn (left : List Int, right : List Int) -> List Int
  sort-source (left concat right)

### Merge two ascending Rational Lists into one ascending List.
pub merge is fn (left : List Rational, right : List Rational) -> List Rational
  sort-source (left concat right)

bounded-count is fn (count : Nat, length : Nat) -> Nat
  count <= length
    true then count
    false then length

take-smallest is fn (values : List (Value : TotalOrder), count : Nat) -> List Value
  length is entry-count values
  boundary is bounded-count (count, length)
  (sort-source values) select-index (0 .. boundary)

### Return at most `count` smallest Int entries in ascending order.
pub smallest is fn (values : List Int, count : Nat) -> List Int
  take-smallest (values, count)

### Return at most `count` smallest Rational entries in ascending order.
pub smallest is fn (values : List Rational, count : Nat) -> List Rational
  take-smallest (values, count)

nth-source is fn (values : List (Value : TotalOrder), index : Nat) -> Optional Value
  first ((sort-source values) select-index (index ..= index))

### Return the zero-based nth smallest Int, or absence.
pub nth is fn (values : List Int, index : Nat) -> Optional Int
  nth-source (values, index)

### Return the zero-based nth smallest Rational, or absence.
pub nth is fn (values : List Rational, index : Nat) -> Optional Rational
  nth-source (values, index)
