#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates the remaining ordered List sequence vocabulary: insertion at a
# boundary, checked regions and indexed removal, zipping policies, entry views,
# reusable traversal, predicate removal, and explicit collection.
values : List Int is Entry ( 1, Entry ( 2, Entry ( 3, Empty ) ) )
other : List Int is Entry ( 7, Entry ( 8, Empty ) )
fragments : List String is Entry ( "Top", Entry ( "al", Empty ) )

done : Unit is values foreach { value }
  _ is value

(
  values insert-at 1 9,
  values insert-at 2 (one 6),
  values split-at 2,
  values take 2,
  values drop 1,
  values remove 1,
  values remove-indexes (1 ..= 2),
  values remove-indexes { index } index = 0,
  values remove-values { value } value > 1,
  values zip-exact other,
  values zip-shortest other,
  (values, 0) zip-longest (other, 0),
  unzip (values zip-shortest other),
  values entries,
  collect values,
  fragments collect String
)
