#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible source-level history for native serialization and
# validated deserialization without exposing raw protocol bytes as authority.
stream is v0.1 (lang serialize) (answer is 42, accepted is true)
lang deserialize stream
