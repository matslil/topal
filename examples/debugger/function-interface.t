#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible declaration of a function interface shape.
Parser is Interface
  parse is fn (source : String) -> Boolean
Parser
  parse is fn (source : String) -> Boolean
    source = "ok"
parse "ok"
