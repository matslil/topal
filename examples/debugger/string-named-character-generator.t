#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible history around an explicitly classified linear
# named-generator consumption.
generated : Generator Character Unit Unit is characters "á👩‍🔬🇸🇪"
generated foreach { character }
  _ is String character
