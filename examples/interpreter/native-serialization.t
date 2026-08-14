#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates version-selected native serialization, reusable partial
# application, and validation before reconstruction by lang deserialize.
serialize is lang version (lang serialize)
stream is serialize (answer is 42, accepted is true)
lang deserialize stream
