#!/usr/bin/env topal
use language (
  version is v0.1
)

# Executable laws for optimization candidates. A compiler may substitute an
# implementation only for the exact declaration identity and must preserve
# these ordinary-source observations.
pub min-commutes is fn (left : Int, right : Int) -> Boolean
  min is fundamental ordering min
  min (left, right) = min (right, left)

pub nfc-idempotent is fn (text : String) -> Boolean
  nfc is text unicode nfc
  nfc (nfc text) = nfc text
