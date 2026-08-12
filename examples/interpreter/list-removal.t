#!/usr/bin/env topal
# Demonstrates value-based List removal: remove-first deletes only the earliest
# equal entry, while remove-all deletes every equal entry and preserves order.
values : List Int is Entry ( 1, Entry ( 2, Entry ( 3, Entry ( 2, Entry ( 4, Empty ) ) ) ) )
(values remove-first 2, values remove-all 2, values remove-all 9)
