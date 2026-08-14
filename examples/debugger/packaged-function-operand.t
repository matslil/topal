#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates reversible binding of supplied and defaulted packaged fields.
sum is fn ( ( value : Int, fallback : Int default 2 ) ) -> Int
  value + fallback
sum (value is 40)
