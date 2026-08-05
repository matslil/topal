#!/usr/bin/env topal
# Demonstrates reversible selection between exhaustive Boolean literal cases.
describe-flag is fn (flag : Boolean) -> String
  flag
    true then "enabled"
    false then "disabled"
(describe-flag true, describe-flag false)
