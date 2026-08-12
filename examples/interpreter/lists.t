#!/usr/bin/env topal
# Demonstrates homogeneous List construction with Empty and Entry, followed by
# total decomposition that binds the first entry and remaining List.
first is fn ( values : List Int ) -> Optional Int
  values
    Empty then None Int
    Entry ( value, rest ) then Some value

values : List Int is Entry ( 7, Entry ( 8, Empty ) )
copy : List Int is Entry ( 7, Entry ( 8, Empty ) )
(first values, values = copy)
