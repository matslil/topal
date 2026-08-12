#!/usr/bin/env topal-debug
# Demonstrates reversible range-as-predicate value and index selection while
# preserving List order, multiplicity, and ordinary String character units.
values : List Int is Entry ( 9, Entry ( 2, Entry ( 4, Entry ( 7, Entry ( 3, Empty ) ) ) ) )
(values select (2 .. 4), values select-index (1 .. 3), "Topal" select-index (1 .. 3))
