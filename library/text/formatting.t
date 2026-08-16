#!/usr/bin/env topal
use language (
  version is v0.1
)

# Demonstrates explicit, locale-independent presentation and structured parsing.
# Unknown Boolean text is absence rather than a guessed value.
pub display-character is fn (value : Character) -> String
  String value

pub parse-boolean is fn (source : String) -> Optional Boolean
  source
    = "true" then Some true
    = "false" then Some false
    otherwise None
