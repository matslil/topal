#!/usr/bin/env topal
# Demonstrates inferred anonymous functions in their contextual collection uses:
# map transforms each entry, select retains matching entries, and fold carries
# an explicitly initialized state from left to right.
values : List Int is Entry ( 1, Entry ( 2, Entry ( 3, Empty ) ) )
(values map { value } value * 2, values select { value } value > 1, values fold 0 { sum, value } sum + value)
