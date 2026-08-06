#!/usr/bin/env topal
# Demonstrates transferring a named linear Character generator into a function
# parameter. The callee consumes the only continuation with foreach.
consume is fn (generated : Generator Character Unit Unit) -> Unit
  generated foreach { character }
    _ is String character
generated is characters "á👩‍🔬🇸🇪"
consume generated
