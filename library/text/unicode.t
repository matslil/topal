#!/usr/bin/env topal
use language (
  version is v0.1
)

# Demonstrates explicit Unicode policy: normalization and case conversion never
# occur implicitly, and character count remains distinct from UTF-8 byte count.
pub nfc is fn (text : String) -> String
  text normalize NFC

pub caseless-equal is fn (left : String, right : String) -> Boolean
  (case-fold left) = (case-fold right)

pub character-length is fn (text : String) -> Nat
  character-count text

pub utf8-length is fn (text : String) -> Nat
  utf8-byte-count text
