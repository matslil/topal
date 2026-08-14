#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible enum declaration, resolution, display, and equality.
Color is Enum (Red, Green, Blue)
(Red, Green, Red = Red, Red = Green)
