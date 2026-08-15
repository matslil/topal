#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates binding a Character generator and consuming its linear
# continuation once with foreach. Its explicit classifier records all three
# directions; debugger snapshots do not duplicate the continuation.
generated : Generator Character Unit Unit is characters "á👩‍🔬🇸🇪"
generated foreach { character }
  _ is String character
