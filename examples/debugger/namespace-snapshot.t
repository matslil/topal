#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible declaration-time namespace snapshot visibility.
answer is 41
earlier is root
later-answer is 42
(earlier answer, root later-answer)
