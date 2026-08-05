#!/usr/bin/env topal
# Demonstrates numeric, string, tuple, and derived labeled-record comparisons.
same-exact-value is 1 = 1.0
different-text is "é" != "é"
ordered is (1, (2, 3)) < (1.0, (2, 4))
same-record is (name is "Ada", score is 1) = (name is "Ada", score is 1.0)
(same-exact-value, different-text, ordered, same-record)
