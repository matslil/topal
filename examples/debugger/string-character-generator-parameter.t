#!/usr/bin/env topal
# Demonstrates reversible history across linear generator parameter transfer.
consume is fn (generated : Generator Character Unit Unit) -> Unit
  generated foreach { character }
    _ is String character
generated is characters "á👩‍🔬🇸🇪"
consume generated
