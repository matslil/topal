#!/usr/bin/env topal
# Demonstrates passing a symbolic callable through a higher-order Function input.
apply-pair is fn (operation : Function) -> Int
  operation (20, 22)
apply-pair +
