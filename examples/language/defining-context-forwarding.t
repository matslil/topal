#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that defining-context values are forwarded explicitly through
# every private function frame while same-named parameters remain isolated.
offset is 40
label is "ready"

read is fn (offset : Int) -> (Int, Int, String)
  (@ offset, offset, @ label)

relay is fn (offset : Int) -> (Int, Int, String)
  read offset

forward is fn (offset : Int) -> (Int, Int, String)
  relay offset

forward 2
