#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible Scope classification and retained namespace identity.
answer is 42
api : Scope is root
api answer
