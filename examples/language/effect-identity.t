#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates canonical equality of empty effect rows.
(Effects ()) = (Effects ())
