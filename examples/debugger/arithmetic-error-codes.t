#!/usr/bin/env topal
# Demonstrates reversible qualified arithmetic error-code selection.
(
  lang arithmetic division-by-zero,
  lang arithmetic indeterminate,
  (lang arithmetic division-by-zero) = (lang arithmetic division-by-zero)
)
