#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates private Tuple parameters, including nesting, multi-parameter
# calls, source-ordered overload selection, and a typed discard.
retain is fn (value : ((Int, Boolean), String)) -> ((Int, Boolean), String)
  value

choose is fn (value : (Int, String), condition : Boolean) -> (Int, String)
  condition
    true then value
    false then (0, "fallback")

tuple-first is fn (value : (Int, String)) -> String
  "tuple"
tuple-first is fn (number : Int, text : String) -> String
  "fields"

fields-first is fn (number : Int, text : String) -> String
  "fields"
fields-first is fn (value : (Int, String)) -> String
  "tuple"

discard-pair is fn static (_ : (Effect, Completed)) -> String
  "discarded"

(retain ((42, true), "nested"), choose ((7, "kept"), true), choose ((9, "lost"), false), tuple-first (1, "one"), fields-first (2, "two"), discard-pair (Effects (), Completed))
