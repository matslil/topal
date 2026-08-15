#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates qualified ArithmeticErrorCode identities; these are code values,
# not Error.domain values.
(
  lang arithmetic division-by-zero,
  lang arithmetic indeterminate,
  (lang arithmetic division-by-zero) = (lang arithmetic division-by-zero)
)
