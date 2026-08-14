#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible immutable record reconstruction and confirms that the
# original labeled product remains unchanged.
person is (name is "Ada", age is 36)
updated is person with (age is person age + 1)
(person age, updated name, updated age)
