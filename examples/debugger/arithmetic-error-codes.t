#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible qualified arithmetic error-code selection.
(
  lang arithmetic division-by-zero,
  lang arithmetic indeterminate,
  (lang arithmetic division-by-zero) = (lang arithmetic division-by-zero)
)
