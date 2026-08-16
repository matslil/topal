#!/usr/bin/env topal
use language (
  version is v0.1
)

# Demonstrates reusable inclusive-range algorithms, checked finite-index
# conversion, and explicit modular reduction. Constraints and modular values
# remain distinct types even when they share representative bounds.
pub intersect is fn (left : Range Int, right : Range Int) -> Range Int
  left and right

pub contains is fn (bounds : Range Int, value : Int) -> Boolean
  bounds contains value

pub as-index is fn (value : Int) -> Result (Nat, lang arithmetic ArithmeticErrorCode)
  Nat value

pub ByteRing is ModNat (0 .. 255)
