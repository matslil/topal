#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates that static introspection is visible to source-level stepping
# without turning an ordinary runtime value into reflection metadata.
identity is lang identity Int
view is lang view Int
(identity, view, lang version)
