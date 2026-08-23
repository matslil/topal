#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ranges as reusable convex predicates for value selection and
# zero-based position selection. Results remain ordinary Lists and Strings;
# retained SliceOf/SelectionOf provenance is observable only through traces.
values : List Int is Entry ( 9, Entry ( 2, Entry ( 4, Entry ( 7, Entry ( 3, Empty ) ) ) ) )
text : String is "Topal"
(values select (2 ..= 4), values select-index (1 .. 4), text select-index (1 .. 4))
