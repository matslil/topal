#!/usr/bin/env topal
# Demonstrates reversible materialization of a take-while-bounded iterate
# generator while retaining yield order and the Int element classifier.
digits is collect (0 iterate ({ value } value + 1) take-while ({ value } value < 5))
digits
