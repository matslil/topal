#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates recursive same-classifier positional-product equality using each
# admitted field equality, including Optional payloads and nested products.
large is 123456789012345678901234567890
left is ((), true, large, 3.5, "exact", Some "present", None String)
same is ((), true, 123456789012345678901234567890, 3.5, "exact", Some "present", None String)
different is ((), true, large, 3.5, "different", Some "present", None String)
nested-left is ((1, "one"), (Some 2, false))
nested-same is ((1, "one"), (Some 2, false))
nested-different is ((1, "one"), (Some 3, false))
(left = same, left != same, left = different, nested-left = nested-same, nested-left != nested-different)
