#!/usr/bin/env topal
# Demonstrates that diagnostic controls remain reversible source statements
# while leaving the controlled program value unchanged.
lang disable-warning example-warning
value is 41
value + 1
