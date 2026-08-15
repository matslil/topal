#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates lazy unbounded generated traversal. Construction captures the
# unary next function but does not invoke it or materialize an intermediate List.
numbers is 0 iterate { value } value + 1
numbers
