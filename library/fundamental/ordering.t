#!/usr/bin/env topal
use language (
  version is v0.1
)
# Select the lesser or greater of two values through their shared TotalOrder
# evidence without imposing a concrete representation.
pub minimum is fn (left : (Value : TotalOrder), right : Value) -> Value
  left
    < right then left
    otherwise right

pub maximum is fn (left : (Value : TotalOrder), right : Value) -> Value
  left
    > right then left
    otherwise right
