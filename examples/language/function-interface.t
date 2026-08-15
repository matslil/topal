#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates an implementation-independent function interface declaration.
Parser is Interface
  parse is fn (source : String) -> Boolean
Parser
  parse is fn (source : String) -> Boolean
    source = "ok"
parse "ok"
