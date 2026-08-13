#!/usr/bin/env topal
# Demonstrates reversible yield, Unit resumption, and termination of a bounded
# generated traversal before its first rejected candidate.
digits is 0 iterate ({ value } value + 1) take-while ({ value } value < 5)
completed is digits foreach { digit }
  _ is digit
completed
