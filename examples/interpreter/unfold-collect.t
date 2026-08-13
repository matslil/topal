#!/usr/bin/env topal
# Demonstrates finite unfold consumption. List uncons keeps the internal seed as
# List Int while yielding Int entries, and None ends collection without an entry.
values : List Int is Entry (4, Entry (5, Entry (6, Empty)))
generated is values unfold { remaining } uncons remaining
collect generated
