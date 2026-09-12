#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact Int arithmetic beyond machine-word bounds, including
# carry, borrow, sign normalization, multiplication, equality, and ordering.
large is 123456789012345678901234567890
other is 987654321098765432109876543210
sum is large + other
difference is large - other
opposite is negate large
cancelled is large + opposite
product is large * other
(large, opposite, absolute difference, sum, difference, cancelled, product, negate product, large = large, large != other, large < other, opposite < cancelled)
