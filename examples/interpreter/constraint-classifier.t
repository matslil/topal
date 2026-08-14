#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a named constraint as a first-class Constraint object.
Positive is Int constraint { value } value > 0
rule : Constraint is Positive
rule
