#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible closed exact Rational-to-Int narrowing.
fifty : Int is 100 / 2
negative-three : Int is -9 / 3
(fifty, negative-three)
