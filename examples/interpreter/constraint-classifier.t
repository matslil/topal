#!/usr/bin/env topal
# Demonstrates a named constraint as a first-class Constraint object.
Positive is Int constraint { value } value > 0
rule : Constraint is Positive
rule
