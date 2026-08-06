#!/usr/bin/env topal
# Demonstrates reversible explicit Some and typed None construction.
missing : Optional Int is None
preserve is fn (candidate : Optional Int) -> Optional Int
  candidate
absent is fn () -> Optional Int
  None
describe is fn (candidate : Optional Int) -> String
  candidate
    Some payload then "present"
    None then "absent"
(Some 42, Some "present", None Int, None String, missing, preserve (Some 7), preserve (None Int), absent (), describe (Some 7), describe missing)
