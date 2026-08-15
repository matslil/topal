#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates inert first-class Effect expressions retained in a List.
rows : List Effect is Entry (Effects (), Empty)
rows
