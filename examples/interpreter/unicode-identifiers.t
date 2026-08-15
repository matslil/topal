#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that printable Unicode symbols and embedded operator characters
# are identifiers when they do not form separate, whitespace-delimited tokens.
🙂 is 40
left+right is 2
🙂 + left+right
