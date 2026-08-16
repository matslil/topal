#!/usr/bin/env topal
use language (
  version is v0.1
)

# Demonstrates that Unit helpers preserve the Unit value and do not manufacture
# the distinct Completed evidence value.
pub keep is fn (value : Unit) -> Unit
  value
