#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates return propagation after selecting Map collection policy.

answer is fn (value : Int) -> Int
  abandoned : Map (String, Int) is collect-map { return value + 1 } resolving keep-last
  1000
answer 41
