#!/usr/bin/env topal
use language (
  version is v0.1,
  features is ( lint )
)
# Demonstrates constructing the lint language variant. It exposes the static,
# authority-free `lang lint` namespace without granting debugger capabilities.
lang lint
