#!/usr/bin/env topal
# Demonstrates reversible unfold stepping and collection with distinct List Int
# seed state and Int yielded values.
values : List Int is Entry (4, Entry (5, Entry (6, Empty)))
generated is values unfold { remaining } uncons remaining
collect generated
