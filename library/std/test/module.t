#!/usr/bin/env topal
use language (
  version is v0.1
)

### Revision of the Topal test-description namespace.
pub revision is 1

### Describe one application source, input file, and exact expected stdout file.
pub application is fn (
  source : String,
  (input : String, expected-stdout : String)
) -> (String, String, String, String)
  ("application-test", source, input, expected-stdout)
