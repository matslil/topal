#!/usr/bin/env topal
# Demonstrates recursive List classifiers: an outer List retains inner Lists of
# positional products through construction, function boundaries, and equality.
preserve is fn ( values : List List (Int, String) ) -> List List (Int, String)
  values

pairs : List (Int, String) is Entry ( (7, "seven"), Empty )
nested : List List (Int, String) is Entry ( pairs, Empty )
copy : List List (Int, String) is Entry ( pairs, Empty )
preserved is preserve nested
(first preserved, entry-count preserved, preserved = copy)
