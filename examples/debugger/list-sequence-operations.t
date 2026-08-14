#!/usr/bin/env topal-debug
use language (
  version is v0.1
)
# Demonstrates reversible ordered List insertion, checked regions and removal,
# explicit zip policies, traversal, entry views, and collection.
values : List Int is Entry ( 1, Entry ( 2, Entry ( 3, Empty ) ) )
other : List Int is Entry ( 7, Entry ( 8, Empty ) )
done : Unit is values foreach { value }
  _ is value
(values insert-at 1 9, values split-at 2, values remove 1, values zip-shortest other, values entries)
