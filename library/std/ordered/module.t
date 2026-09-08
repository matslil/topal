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

sort-source is fn (values : List (Value : TotalOrder)) -> List Value
  values stable-sort

### Return Int entries in ascending order, preserving the order of equal entries.
pub sort is fn (values : List Int) -> List Int
  values stable-sort

### Return Rational entries in ascending order, preserving equal-entry order.
pub sort is fn (values : List Rational) -> List Rational
  values stable-sort

### Return Int entries in descending order, preserving the order of equal entries.
pub sort-descending is fn (values : List Int) -> List Int
  values stable-sort-descending

### Return Rational entries in descending order, preserving equal-entry order.
pub sort-descending is fn (values : List Rational) -> List Rational
  values stable-sort-descending

present-index is fn (boundary : Nat) -> Optional Nat
  present : Optional Nat is Some boundary
  present

midpoint is fn (lower : Nat, upper : Nat) -> Nat
  quotient is fn (value : Int, remainder : Int) -> Int
    value
  Nat (lower + (quotient ((upper - lower) /% 2)))

half is fn (value : Int) -> Int
  quotient is fn (result : Int, remainder : Int) -> Int
    result
  quotient (value /% 2)
search-depth is fn (length : Nat) -> Nat
  halves : List Int is collect (length iterate ({ value } half value) take-while ({ value } value > 0))
  (entry-count halves) + 1

search-values is fn ((values : List Int, sought : Int, lower : Nat, upper : Nat, found : Int)) -> List Int
  values
search-values is fn ((values : List Rational, sought : Rational, lower : Nat, upper : Nat, found : Int)) -> List Rational
  values
search-sought is fn ((values : List Int, sought : Int, lower : Nat, upper : Nat, found : Int)) -> Int
  sought
search-sought is fn ((values : List Rational, sought : Rational, lower : Nat, upper : Nat, found : Int)) -> Rational
  sought
search-lower is fn ((values : List Int, sought : Int, lower : Nat, upper : Nat, found : Int)) -> Nat
  lower
search-lower is fn ((values : List Rational, sought : Rational, lower : Nat, upper : Nat, found : Int)) -> Nat
  lower
search-upper is fn ((values : List Int, sought : Int, lower : Nat, upper : Nat, found : Int)) -> Nat
  upper
search-upper is fn ((values : List Rational, sought : Rational, lower : Nat, upper : Nat, found : Int)) -> Nat
  upper
search-found is fn ((values : List Int, sought : Int, lower : Nat, upper : Nat, found : Int)) -> Int
  found
search-found is fn ((values : List Rational, sought : Rational, lower : Nat, upper : Nat, found : Int)) -> Int
  found

int-search-present is fn (state : (List Int, Int, Nat, Nat, Int), candidate : Int) -> (List Int, Int, Nat, Nat, Int)
  middle is midpoint (search-lower state, search-upper state)
  candidate
    = (search-sought state) then (search-values state, search-sought state, search-lower state, search-upper state, middle)
    < (search-sought state) then (search-values state, search-sought state, middle + 1, search-upper state, -2)
    otherwise (search-values state, search-sought state, search-lower state, middle, -2)
int-search-candidate is fn (state : (List Int, Int, Nat, Nat, Int), candidate : Optional Int) -> (List Int, Int, Nat, Nat, Int)
  candidate
    Some present then int-search-present (state, present)
    None then (search-values state, search-sought state, search-lower state, search-upper state, -1)
int-search-nonempty is fn (state : (List Int, Int, Nat, Nat, Int)) -> (List Int, Int, Nat, Nat, Int)
  middle is midpoint (search-lower state, search-upper state)
  int-search-candidate (state, first ((search-values state) select-index (middle ..= middle)))
int-search-active-step is fn (state : (List Int, Int, Nat, Nat, Int)) -> (List Int, Int, Nat, Nat, Int)
  (search-lower state) < (search-upper state)
    true then int-search-nonempty state
    false then (search-values state, search-sought state, search-lower state, search-upper state, -1)
int-search-step is fn (state : (List Int, Int, Nat, Nat, Int)) -> (List Int, Int, Nat, Nat, Int)
  (search-found state) = -2
    true then int-search-active-step state
    false then state
int-search-loop is fn (
  state : (List Int, Int, Nat, Nat, Int),
  remaining : Nat
) -> (List Int, Int, Nat, Nat, Int) : Decreases remaining
  remaining
    <= 0 then state
    otherwise int-search-loop (int-search-step state, remaining - 1)

rational-search-present is fn (state : (List Rational, Rational, Nat, Nat, Int), candidate : Rational) -> (List Rational, Rational, Nat, Nat, Int)
  middle is midpoint (search-lower state, search-upper state)
  candidate
    = (search-sought state) then (search-values state, search-sought state, search-lower state, search-upper state, middle)
    < (search-sought state) then (search-values state, search-sought state, middle + 1, search-upper state, -2)
    otherwise (search-values state, search-sought state, search-lower state, middle, -2)
rational-search-candidate is fn (state : (List Rational, Rational, Nat, Nat, Int), candidate : Optional Rational) -> (List Rational, Rational, Nat, Nat, Int)
  candidate
    Some present then rational-search-present (state, present)
    None then (search-values state, search-sought state, search-lower state, search-upper state, -1)
rational-search-nonempty is fn (state : (List Rational, Rational, Nat, Nat, Int)) -> (List Rational, Rational, Nat, Nat, Int)
  middle is midpoint (search-lower state, search-upper state)
  rational-search-candidate (state, first ((search-values state) select-index (middle ..= middle)))
rational-search-active-step is fn (state : (List Rational, Rational, Nat, Nat, Int)) -> (List Rational, Rational, Nat, Nat, Int)
  (search-lower state) < (search-upper state)
    true then rational-search-nonempty state
    false then (search-values state, search-sought state, search-lower state, search-upper state, -1)
rational-search-step is fn (state : (List Rational, Rational, Nat, Nat, Int)) -> (List Rational, Rational, Nat, Nat, Int)
  (search-found state) = -2
    true then rational-search-active-step state
    false then state
rational-search-loop is fn (
  state : (List Rational, Rational, Nat, Nat, Int),
  remaining : Nat
) -> (List Rational, Rational, Nat, Nat, Int) : Decreases remaining
  remaining
    <= 0 then state
    otherwise rational-search-loop (rational-search-step state, remaining - 1)

optional-found-index is fn (found : Int) -> Optional Nat
  found >= 0
    true then present-index (Nat found)
    false then None Nat

### Find an equal Int in an ascending List.
pub binary-search is fn (values : List Int, sought : Int) -> Optional Nat
  final is int-search-loop ((values, sought, Nat 0, entry-count values, -2), search-depth (entry-count values))
  optional-found-index (search-found final)

### Find an equal Rational in an ascending List.
pub binary-search is fn (values : List Rational, sought : Rational) -> Optional Nat
  final is rational-search-loop ((values, sought, Nat 0, entry-count values, -2), search-depth (entry-count values))
  optional-found-index (search-found final)

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
