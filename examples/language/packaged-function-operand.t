#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates one packaged function operand. The labeled call omits fallback,
# so the declaration's field default is evaluated in the invocation scope.
sum is fn (
  (
    value : Int,
    fallback : Int default 2
  )
) -> Int
  value + fallback
sum (value is 40)
