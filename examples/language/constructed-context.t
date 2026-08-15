#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates explicit selection of an immutable defining-context member.
offset is 40
add-offset is fn (value : Int) -> Int
  value + @ offset
add-offset 2
