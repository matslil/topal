#!/usr/bin/env topal-debug
use language (
  version is v0.1
)
# Demonstrates reversible decisions for entry, consecutive-sequence, and
# ordered-subsequence containment over immutable Lists.
values : List Int is Entry ( 1, Entry ( 2, Entry ( 3, Entry ( 2, Entry ( 4, Empty ) ) ) ) )
consecutive : List Int is Entry ( 2, Entry ( 3, Empty ) )
with-gaps : List Int is Entry ( 1, Entry ( 3, Entry ( 4, Empty ) ) )
reversed : List Int is Entry ( 3, Entry ( 1, Empty ) )
(values contains-entry 2, values contains-entry 9, values contains-sequence consecutive, values contains-subsequence with-gaps, values contains-sequence with-gaps, values contains-subsequence reversed)
