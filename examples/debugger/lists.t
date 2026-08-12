#!/usr/bin/env topal-debug
# Demonstrates reversible List construction, fundamental operations including
# total uncons, equality, and a decision binding the first value and remaining List.
first is fn ( values : List Int ) -> Optional Int
  values
    Empty then None Int
    Entry ( value, rest ) then Some value

values : List Int is Entry ( 7, Entry ( 8, Empty ) )
copy : List Int is Entry ( 7, Entry ( 8, Empty ) )
empty-values is empty List Int
suffix is one 10
prepended is values prepend 6
appended is prepended append 9
combined is appended concat suffix
(first combined, entry-count combined, empty? combined, empty? empty-values, values = copy, uncons combined)
