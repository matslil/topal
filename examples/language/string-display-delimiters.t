#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates dynamic String return values and canonical display delimiters when
# contents contain quotes, including a collision with the preferred text tag.
render is fn (flag : Boolean) -> String
  flag
    true then text"He said "hello"."text
    false then outer"This contains "text delimiters"text."outer
(render true, render false)
