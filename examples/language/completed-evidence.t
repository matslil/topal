#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that Completed is explicit zero-data completion evidence and can
# be returned by a function; it is distinct from the Unit value ().
finish-work is fn () -> Completed
  Completed

finish-work ()
