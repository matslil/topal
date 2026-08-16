#!/usr/bin/env topal
use language (
  version is v0.1
)
# Select the lesser or greater of two values through their shared TotalOrder
# evidence without imposing a concrete representation.
pub min is fn (left : (Value : TotalOrder), right : Value) -> Value
  left
    <= right then left
    otherwise right

pub max is fn (left : (Value : TotalOrder), right : Value) -> Value
  left
    >= right then left
    otherwise right

pub min-max is fn (left : (Value : TotalOrder), right : Value) -> (Value, Value)
  left
    <= right then (left, right)
    otherwise (right, left)
