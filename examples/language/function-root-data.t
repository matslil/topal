#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates explicit root data selection from a function body while a
# same-named parameter remains isolated from the live root namespace.
read is fn (answer : Int) -> (Int, Int, String)
  (root answer, answer, root label)

make-answer is fn () -> Int
  40 + 2

label is "ready"
answer is make-answer ()

read 0
