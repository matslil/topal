#!/usr/bin/env topal
# Demonstrates exact mixed-number equality, preserved strings, and tuple ordering.
same-exact-value is 1 = 1.0
different-text is "é" != "é"
ordered is (1, (2, 3)) < (1.0, (2, 4))
(same-exact-value, different-text, ordered)
