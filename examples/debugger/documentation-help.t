#!/usr/bin/env topal
use language (
  version is v0.1
)

# Demonstrate source-level declaration documentation in debugger help.
### Return the documented answer used by the debugger example.
pub documented-answer is fn () -> Int
  42

documented-answer ()
