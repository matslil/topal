#!/usr/bin/env topal
# Demonstrates reversible explicit Some and typed None construction.
missing : Optional Int is None
preserve is fn (candidate : Optional Int) -> Optional Int
  candidate
absent is fn () -> Optional Int
  None
(Some 42, Some "present", None Int, None String, missing, preserve (Some 7), preserve (None Int), absent ())
