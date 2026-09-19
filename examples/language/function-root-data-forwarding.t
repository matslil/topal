#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that a live root value is forwarded explicitly through every
# private function frame while same-named ordinary parameters remain isolated.
read is fn (answer : Int) -> (Int, Int, String)
  (root answer, answer, root label)

relay is fn (answer : Int) -> (Int, Int, String)
  read answer

forward is fn (answer : Int) -> (Int, Int, String)
  relay answer

label is "ready"
answer is 40 + 2

forward 0
