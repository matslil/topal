#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible entry into a function declared later in source order.
render is fn (text : String) -> String
  decorate text
decorate is fn (text : String) -> String
  "[" concat text concat "]"
render "Topal"
