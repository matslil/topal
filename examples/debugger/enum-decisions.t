#!/usr/bin/env topal
# Demonstrates reversible consideration and selection of exhaustive Enum cases.
Color is Enum (Red, Green)
name is fn (value : Color) -> String
  value
    Red then "red"
    Green then "green"
(name Red, name Green)
