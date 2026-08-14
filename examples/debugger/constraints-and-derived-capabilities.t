#!/usr/bin/env topal-debug
use language (
  version is v0.1
)
# Demonstrates reversible constraint validation, retained evidence, implicit
# evidence-forgetting conversion, and equality/order inherited from Int.
Positive is Int constraint { value } value > 0
first : Positive is Positive 3
second : Positive is Positive 5
(first = (Positive 3), first < second, first + 2)
