#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordered generator overload selection by complete initial-input
# shape: the unary overload yields Int, while the binary overload yields String.
select is generator ( value : Int )
  yields Int
  resumes Unit
  -> String

  _ is yield value
  "unary"

select is generator ( value : Int, suffix : String )
  yields String
  resumes Unit
  -> String

  _ is value + 1
  _ is yield suffix
  "binary"

unary-generated is select 7
unary-result : String is unary-generated foreach { value }
  _ is value + 1
binary-generated is select (7, "item")
binary-result : String is binary-generated foreach { value }
  _ is empty? value
(unary-result, binary-result)
