#!/usr/bin/env topal
use language (
  version is v0.1
)
# Publishes one module member while retaining an unexported implementation value.
private-value is 40
pub answer is private-value + 2
