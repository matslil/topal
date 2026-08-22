#!/usr/bin/env topal
use language (
  version is v0.1
)
use library std (
  version is v0.1
)

# Executable laws for optimization candidates. A compiler may substitute an
# implementation only for the exact declaration identity and must preserve
# these ordinary-source observations.
pub min-commutes is fn (left : Int, right : Int) -> Boolean
  (left <= right then left otherwise right) = (right <= left then right otherwise left)

pub nfc-idempotent is fn (text : String) -> Boolean
  ((text normalize NFC) normalize NFC) = (text normalize NFC)

pub exact-int-sum-reference is fn (values : List Int) -> Boolean
  (std sum values) = (values fold 0 { total, value } total + value)

pub exact-int-product-reference is fn (values : List Int) -> Boolean
  (std product values) = (values fold 1 { total, value } total * value)
