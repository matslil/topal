#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact repeated-name matching for nominal sums. Only the active
# alternative payload participates in pattern identity.
Inner is Union
  Mark
  Code : Int

Token is Union
  Stop
  Number : Int
  Label : String
  Pair : (Int, String)
  Wrapped : Inner

repeat : Function is { value, value } value

stop : Token is Stop
number : Token is Number 7
label : Token is Label "seven"
pair : Token is Pair (7, "seven")
wrapped : Token is Wrapped (Code 9)

(
  repeat (stop, stop),
  repeat (number, number),
  repeat (label, label),
  repeat (pair, pair),
  repeat (wrapped, wrapped)
)
