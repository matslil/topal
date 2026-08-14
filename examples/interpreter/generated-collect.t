#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates explicit materialization of a finite generated traversal as a
# List. The rejected boundary value 5 is not included in the result.
digits is collect (0 iterate ({ value } value + 1) take-while ({ value } value < 5))
digits
