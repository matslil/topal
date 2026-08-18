#!/usr/bin/env topal
use language (
  version is v0.1
)

# Derived finite algorithms build on the primitive fold/map/select vocabulary.
# They preserve exact element classifiers and never assume a storage layout.
pub any? is fn (
  values : List (Value : Type),
  predicate : fn (Value) -> Boolean
) -> Boolean
  values fold false { found, value } found or (predicate value)

pub all? is fn (
  values : List (Value : Type),
  predicate : fn (Value) -> Boolean
) -> Boolean
  values fold true { accepted, value } accepted and (predicate value)

pub none? is fn (
  values : List (Value : Type),
  predicate : fn (Value) -> Boolean
) -> Boolean
  not (values fold false { found, value } found or (predicate value))

increment-if is fn (count : Int, accepted : Boolean) -> Int
  accepted
    true then count + 1
    false then count
increment-if-callable is increment-if

pub count-where is fn (
  values : List (Value : Type),
  predicate : fn (Value) -> Boolean
) -> Int
  values fold 0 { count, value } increment-if-callable (count, predicate value)

present-if is fn (
  accepted : Boolean,
  value : (Value : Type)
) -> Optional Value
  accepted
    true then Some value
    false then None Value
present-if-callable is present-if

find-step is fn (
  (
    found : Optional (Value : Type),
    accepted : Boolean,
    value : Value
  )
) -> Optional Value
  found
    Some payload then found
    None then present-if-callable (accepted, value)
find-step-callable is find-step

pub find is fn (
  values : List (Value : Type),
  predicate : fn (Value) -> Boolean
) -> Optional Value
  values fold (None Value) { found, value } find-step-callable (found, predicate value, value)
