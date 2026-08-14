#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible function return and linear generator consumption.
generate is fn (text : String) -> Generator Character Unit Unit
  characters text
generated is generate "á👩‍🔬🇸🇪"
generated foreach { character }
  _ is String character
