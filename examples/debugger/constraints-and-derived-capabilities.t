#!/usr/bin/env topal-debug
# Demonstrates reversible constraint validation, retained evidence, implicit
# evidence-forgetting conversion, and equality/order inherited from Int.
Positive is Int constraint { value } value > 0
first : Positive is Positive 3
second : Positive is Positive 5
(first = (Positive 3), first < second, first + 2)
