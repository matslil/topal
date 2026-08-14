#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates transferring a named linear Character generator into a function
# parameter. The callee consumes the only continuation with foreach.
consume is fn (generated : Generator Character Unit Unit) -> Unit
  generated foreach { character }
    _ is String character
generated is characters "á👩‍🔬🇸🇪"
consume generated
