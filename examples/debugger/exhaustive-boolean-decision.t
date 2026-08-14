#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible selection between exhaustive Boolean literal cases.
describe-flag is fn (flag : Boolean) -> String
  flag
    true then "enabled"
    false then "disabled"
(describe-flag true, describe-flag false)
