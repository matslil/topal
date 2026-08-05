#!/usr/bin/env topal
# Demonstrates reversible enum declaration, resolution, display, and equality.
Color is Enum (Red, Green, Blue)
(Red, Green, Red = Red, Red = Green)
