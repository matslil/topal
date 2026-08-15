#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates publishing a declaration and resolving it explicitly through the
# live root namespace without flattening the qualified path.
pub answer is 42
root answer
