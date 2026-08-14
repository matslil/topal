#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible lazy construction of an iterate generator without
# invoking its captured next function.
numbers is 0 iterate { value } value + 1
numbers
