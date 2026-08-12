#!/usr/bin/env topal
# Demonstrates homogeneous List construction and total decomposition, followed
# by prepend, append, concat, entry-count, empty?, and structural equality.
first is fn ( values : List Int ) -> Optional Int
  values
    Empty then None Int
    Entry ( value, rest ) then Some value

values : List Int is Entry ( 7, Entry ( 8, Empty ) )
copy : List Int is Entry ( 7, Entry ( 8, Empty ) )
empty-values : List Int is Empty
suffix : List Int is Entry ( 10, Empty )
prepended is values prepend 6
appended is prepended append 9
combined is appended concat suffix
(first combined, entry-count combined, empty? combined, empty? empty-values, values = copy)
