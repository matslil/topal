#!/usr/bin/env topal
# Demonstrates explicit present and absent Optional construction. None carries
# the named payload type even though display intentionally omits it.
(Some 42, Some "present", None Int, None String)
