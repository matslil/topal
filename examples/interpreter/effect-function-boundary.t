#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates an inert Effect value crossing a function boundary.
identity-effect is fn (effects : Effect) -> Effect
  effects
identity-effect (Effects ())
