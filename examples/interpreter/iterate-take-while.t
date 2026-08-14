#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a lazy finite prefix of an iterate generator. take-while captures
# its predicate without evaluating either generator function during construction.
digits is 0 iterate ({ value } value + 1) take-while ({ value } value < 10)
digits
