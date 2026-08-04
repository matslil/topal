#!/usr/bin/env topal
# Demonstrates that `_ is expression` evaluates and discards without binding `_`.
_ is 20 + 22
visible is 7
((), visible)
