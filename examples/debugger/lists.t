#!/usr/bin/env topal-debug
# Demonstrates reversible List construction, structural equality, and an
# exhaustive Empty/Entry decision that binds the first value and remaining List.
first is fn ( values : List Int ) -> Optional Int
  values
    Empty then None Int
    Entry ( value, rest ) then Some value

values : List Int is Entry ( 7, Entry ( 8, Empty ) )
copy : List Int is Entry ( 7, Entry ( 8, Empty ) )
(first values, values = copy)
