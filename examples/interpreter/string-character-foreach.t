#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates direct traversal of a Unit-resumed Character generator. Each
# grapheme is converted to String in the body; exhaustion returns Unit.
characters "á👩‍🔬🇸🇪" foreach { character }
  _ is String character
