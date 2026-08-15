#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that a function body may call a completely typed function whose
# declaration appears later in the same scope.
render is fn (text : String) -> String
  decorate text
decorate is fn (text : String) -> String
  "[" concat text concat "]"
render "Topal"
