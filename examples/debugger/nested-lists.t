#!/usr/bin/env topal-debug
use language (
  version is v0.1
)
# Demonstrates reversible recursive List classifiers containing inner Lists of
# positional products across ordinary function and equality boundaries.
preserve is fn ( values : List List (Int, String) ) -> List List (Int, String)
  values

pairs : List (Int, String) is Entry ( (7, "seven"), Empty )
nested : List List (Int, String) is Entry ( pairs, Empty )
copy : List List (Int, String) is Entry ( pairs, Empty )
preserved is preserve nested
(first preserved, entry-count preserved, preserved = copy)
