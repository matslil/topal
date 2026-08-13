#!/usr/bin/env topal
# Demonstrates reversible construction of a bounded generated traversal without
# eagerly invoking its next or take-while functions.
digits is 0 iterate ({ value } value + 1) take-while ({ value } value < 10)
digits
