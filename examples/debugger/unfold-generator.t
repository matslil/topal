#!/usr/bin/env topal
# Demonstrates reversible lazy unfold construction without invoking the captured
# uncons step or materializing its future yielded values.
values : List Int is Entry (4, Entry (5, Entry (6, Empty)))
generated is values unfold { remaining } uncons remaining
generated
