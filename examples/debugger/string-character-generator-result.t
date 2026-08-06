#!/usr/bin/env topal
# Demonstrates reversible function return and linear generator consumption.
generate is fn (text : String) -> Generator Character Unit Unit
  characters text
generated is generate "á👩‍🔬🇸🇪"
generated foreach { character }
  String character
