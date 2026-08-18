#!/usr/bin/env topal
use language (
  version is v0.1
)

# Explicit Unicode policy: normalization and case conversion never occur
# implicitly.
pub nfc is fn (text : String) -> String
  text normalize NFC

pub nfd is fn (text : String) -> String
  text normalize NFD

pub canonical-equal is fn (left : String, right : String) -> Boolean
  left canonically-equals right

pub caseless-equal is fn (left : String, right : String) -> Boolean
  (case-fold left) = (case-fold right)

pub starts-with? is fn (text : String, prefix : String) -> Boolean
  string-starts-with (text, prefix)

pub ends-with? is fn (text : String, suffix : String) -> Boolean
  string-ends-with (text, suffix)

pub contains? is fn (text : String, fragment : String) -> Boolean
  string-contains (text, fragment)

pub trim is fn (text : String) -> String
  string-trim text

pub replace-all is fn (
  (
    text : String,
    pattern : String,
    replacement : String
  )
) -> String
  string-replace-all (text, pattern, replacement)

pub repeat is fn (text : String, count : Nat) -> String
  string-repeat (text, count)
