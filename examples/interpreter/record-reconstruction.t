#!/usr/bin/env topal
# Demonstrates immutable record reconstruction: `with` replaces the selected
# field in a new product while the original product remains unchanged.
person is (name is "Ada", age is 36)
updated is person with (age is person age + 1)
(person age, updated name, updated age)
