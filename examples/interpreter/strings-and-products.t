#!/usr/bin/env topal
# Demonstrates literal-preserving tagged strings and positional products.
message is text"Topal strings preserve "quotes",
newlines, and {braces}."text
flags is (true, false)
(
  message,
  flags,
  (),
)
