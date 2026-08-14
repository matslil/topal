#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates inert Effect values packaged in an ordinary product.
pair : (Effect, Effect) is (Effects (), Effects ())
pair
