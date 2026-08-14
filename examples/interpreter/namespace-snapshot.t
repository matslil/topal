#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that namespace aliases preserve declaration visibility at capture.
answer is 41
earlier is root
later-answer is 42
(earlier answer, root later-answer)
