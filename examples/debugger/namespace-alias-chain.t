#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible lookup through a chain of immutable namespace aliases.
answer is 42
first is root
second is first
second answer
