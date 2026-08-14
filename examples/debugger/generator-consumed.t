#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates the diagnostic produced by attempting to traverse the same
# named linear generator twice. Construct a fresh generator for another pass.
generated is characters "Topal"
generated foreach { character }
  _ is String character
generated foreach { character }
  _ is String character
