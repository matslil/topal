#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates direct finite traversal of a generated prefix. The foreach body
# visits 0 through 4, while the first rejected candidate 5 is never visited.
digits is 0 iterate ({ value } value + 1) take-while ({ value } value < 5)
completed is digits foreach { digit }
  _ is digit
completed
