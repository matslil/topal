#!/usr/bin/env topal
use language (
  version is v0.1
)

# Lazy constructors retain generator ownership: values are produced only when
# the consumer resumes the continuation, and abandoning it still closes it.
pub count-from is fn ( initial : Int ) -> Generator Int Unit Unit
  initial iterate ({ value } value + 1)
