#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates product destructuring in a contextually typed anonymous
# function while preserving the exact Int field classifiers.
pairs : List (Int, Int) is Entry ((2, 3), Entry ((5, 7), Empty))
pairs map { (left, right) } left + right
