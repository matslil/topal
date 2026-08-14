#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates one-statement and scoped warning controls without changing the
# value or execution decisions of the controlled statements.
lang disable-warning example-warning
value is 41
lang push-disable-warning example-warning
lang pop-disable-warning example-warning
value + 1
