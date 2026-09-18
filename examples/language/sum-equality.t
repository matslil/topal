#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates derived equality for nominal sums. The tag selects the only
# active payload which participates in equality.
Inner is Union
  Mark
  Code : Int

Token is Union
  Stop
  Number : Int
  Label : String
  Pair : (Int, String)
  Details : Record (code : Int, text : String)
  Wrapped : Inner

Choice is Variant (Int, String)

same-token is fn (left : Token, right : Token) -> Boolean
  left = right

same-choice is fn (left : Choice, right : Choice) -> Boolean
  left = right

(
  same-token (Stop, Stop),
  same-token (Stop, Number 0),
  same-token (Number 7, Number 7),
  same-token (Number 7, Number 8),
  same-token (Label "seven", Label "seven"),
  same-token (Pair (7, "seven"), Pair (7, "seven")),
  same-token (Details (code is 7, text is "seven"), Details (text is "seven", code is 7)),
  same-token (Wrapped Mark, Wrapped (Code 9)),
  same-choice (Choice at 1 "same", Choice at 1 "same"),
  same-choice (Choice at 0 1, Choice at 1 "1"),
  (Choice at 1 "same") != (Choice at 1 "different")
)
