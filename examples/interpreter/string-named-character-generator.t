#!/usr/bin/env topal
# Demonstrates binding a Character generator and consuming its linear
# continuation once with foreach. Debugger snapshots do not duplicate it.
generated is characters "á👩‍🔬🇸🇪"
generated foreach { character }
  String character
