#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates the general Scope classifier while retaining qualified lookup.
answer is 42
api : Scope is root
api answer
