#!/usr/bin/env topal
# Demonstrates reversible explicit Some and typed None construction.
missing : Optional Int is None
(Some 42, Some "present", None Int, None String, missing)
