#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates the qualified GeneratorErrorCode identity. This constructs a code
# value only; it does not close a continuation or define an Error.domain.
closed is lang generator generator-closed
(closed, closed = (lang generator generator-closed))
