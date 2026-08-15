#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a function transferring a fresh Character generator to its
# caller, which binds and consumes the returned linear continuation once.
generate is fn (text : String) -> Generator Character Unit Unit
  characters text
generated is generate "á👩‍🔬🇸🇪"
generated foreach { character }
  _ is String character
