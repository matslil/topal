#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible bindings whose names use an emoji and an embedded
# operator character under the broad Unicode identifier profile.
🙂 is 40
left+right is 2
🙂 + left+right
