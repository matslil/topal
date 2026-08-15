#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates nominal payload-free enum declaration, value display, and
# equality between alternatives of the same enum.
Color is Enum (Red, Green, Blue)
(Red, Green, Red = Red, Red = Green)
