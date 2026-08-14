#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible selection of the qualified generator-closed code.
closed is lang generator generator-closed
(closed, closed = (lang generator generator-closed))
