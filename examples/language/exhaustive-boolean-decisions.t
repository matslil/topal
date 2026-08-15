#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a complete Boolean decision table whose explicit `true` and
# `false` cases make an `otherwise` rule unnecessary.
describe-flag is fn (flag : Boolean) -> String
  flag
    true then "enabled"
    false then "disabled"
(describe-flag true, describe-flag false)
