#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates finite traversal by user-perceived Unicode characters. Collecting
# the unchanged Characters reconstructs the exact preserved scalar sequence.
# The generator resumes with Unit and returns Unit after its final yield.
characters "á👩‍🔬🇸🇪" collect String
