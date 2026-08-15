#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that `_ is expression` evaluates and discards without binding `_`.
_ is 20 + 22
visible is 7
((), visible)
