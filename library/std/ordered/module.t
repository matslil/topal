#!/usr/bin/env topal
use language (
  version is v0.1
)

### Revision of the ordered-algorithm namespace.
pub revision is 1

### Return Int entries in ascending order, preserving the order of equal entries.
pub sort is fn (values : List Int) -> List Int
  values stable-sort

### Return Int entries in descending order, preserving the order of equal entries.
pub sort-descending is fn (values : List Int) -> List Int
  values stable-sort-descending

increment-when is fn (count : Int, accepted : Boolean) -> Int
  accepted
    true then count + 1
    false then count
increment-when-callable is increment-when

### Return the insertion index before equal entries in an ascending List.
pub lower-bound is fn (values : List (Value : TotalOrder), sought : Value) -> Int
  values fold 0 { count, candidate } increment-when-callable (count, candidate < sought)
lower-bound-callable is lower-bound

### Return the insertion index after equal entries in an ascending List.
pub upper-bound is fn (values : List (Value : TotalOrder), sought : Value) -> Int
  values fold 0 { count, candidate } increment-when-callable (count, candidate <= sought)
upper-bound-callable is upper-bound

### Return the half-open index range containing entries equal to `sought`.
pub equal-range is fn (values : List (Value : TotalOrder), sought : Value) -> Range Int
  lower is lower-bound-callable (values, sought)
  upper is upper-bound-callable (values, sought)
  lower .. upper
