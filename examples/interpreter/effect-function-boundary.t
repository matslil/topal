#!/usr/bin/env topal
# Demonstrates an inert Effect value crossing a function boundary.
identity-effect is fn (effects : Effect) -> Effect
  effects
identity-effect (Effects ())
