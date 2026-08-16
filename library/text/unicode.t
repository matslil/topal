#!/usr/bin/env topal
use language (
  version is v0.1
)

# Explicit Unicode policy: normalization and case conversion never occur
# implicitly.
pub nfc is fn (text : String) -> String
  text normalize NFC

pub caseless-equal is fn (left : String, right : String) -> Boolean
  (case-fold left) = (case-fold right)
