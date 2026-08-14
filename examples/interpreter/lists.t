#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates homogeneous List construction and total decomposition, followed
# by prepend, append, concat, entry-count, empty?, uncons, and equality.
decide-first is fn ( values : List Int ) -> Optional Int
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
reversed is combined reverse
(decide-first combined, first combined, rest combined, first empty-values, rest empty-values, entry-count combined, empty? combined, empty? empty-values, values = copy, uncons combined, first reversed, reversed reverse = combined)
