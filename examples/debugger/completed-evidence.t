#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible construction and return of explicit Completed
# evidence, distinct from the Unit value ().
finish-work is fn () -> Completed
  Completed

finish-work ()
