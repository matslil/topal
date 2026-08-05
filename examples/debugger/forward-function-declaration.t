#!/usr/bin/env topal
# Demonstrates reversible entry into a function declared later in source order.
render is fn (text : String) -> String
  decorate text
decorate is fn (text : String) -> String
  "[" concat text concat "]"
render "Topal"
