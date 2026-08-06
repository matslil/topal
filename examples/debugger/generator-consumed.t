#!/usr/bin/env topal
# Demonstrates the diagnostic produced by attempting to traverse the same
# named linear generator twice. Construct a fresh generator for another pass.
generated is characters "Topal"
generated foreach { character }
  String character
generated foreach { character }
  String character
