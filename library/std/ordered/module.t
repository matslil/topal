#!/usr/bin/env topal
use language (
  version is v0.1
)

### Revision of the ordered-algorithm namespace.
pub revision is 1

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

### Find an equal Int in an ascending List using binary search.
pub binary-search is fn (values : List Int, sought : Int) -> Optional Nat
  values ordered-binary-search sought

### Find an equal Rational in an ascending List using binary search.
pub binary-search is fn (values : List Rational, sought : Rational) -> Optional Nat
  values ordered-binary-search sought

### Merge two ascending Int Lists into one ascending List.
pub merge is fn (left : List Int, right : List Int) -> List Int
  left ordered-merge right

### Merge two ascending Rational Lists into one ascending List.
pub merge is fn (left : List Rational, right : List Rational) -> List Rational
  left ordered-merge right

### Return at most `count` smallest Int entries in ascending order.
pub smallest is fn (values : List Int, count : Nat) -> List Int
  values ordered-smallest count

### Return at most `count` smallest Rational entries in ascending order.
pub smallest is fn (values : List Rational, count : Nat) -> List Rational
  values ordered-smallest count

### Return the zero-based nth smallest Int, or absence.
pub nth is fn (values : List Int, index : Nat) -> Optional Int
  values ordered-nth index

### Return the zero-based nth smallest Rational, or absence.
pub nth is fn (values : List Rational, index : Nat) -> Optional Rational
  values ordered-nth index

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
