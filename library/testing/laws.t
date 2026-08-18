#!/usr/bin/env topal
use language (
  version is v0.1
)

# Executable laws for optimization candidates. A compiler may substitute an
# implementation only for the exact declaration identity and must preserve
# these ordinary-source observations.
pub min-commutes is fn (left : Int, right : Int) -> Boolean
  (left <= right then left otherwise right) = (right <= left then right otherwise left)

pub nfc-idempotent is fn (text : String) -> Boolean
  ((text normalize NFC) normalize NFC) = (text normalize NFC)
