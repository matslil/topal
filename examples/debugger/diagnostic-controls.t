#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that diagnostic controls remain reversible source statements
# while leaving the controlled program value unchanged.
lang disable-warning example-warning
value is 41
value + 1
