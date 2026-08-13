#!/usr/bin/env topal
# Demonstrates traversal-control construction. Continue carries a next state;
# the fold action returns Finish immediately, leaving later entries unvisited.
values : List Int is Entry (1, Entry (2, Entry (100, Empty)))
controls is (Continue 1, Finish 2)
stop is { state, value } Finish (state + value)
(values fold 0 stop, controls)
