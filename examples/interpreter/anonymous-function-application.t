#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates direct application of anonymous function values. Unary input is
# supplied directly; several inputs are packaged in one positional product.
increment is { value } value + 1
combine is { left, right } left * 10 + right
(increment 41, combine (4, 2))
