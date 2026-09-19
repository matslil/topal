#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordinary arithmetic ErrorCode Lists across construction,
# equality, complete decomposition, private boundaries, products, and observations.
head-or is fn (candidate : List ErrorCode, fallback : ErrorCode) -> ErrorCode
  candidate
    Entry (first, rest) then first
    Empty then fallback

second-or is fn (candidate : List ErrorCode, fallback : ErrorCode) -> ErrorCode
  candidate
    Entry (ignored, rest) then head-or (rest, fallback)
    Empty then fallback

return-list is fn (candidate : List ErrorCode) -> List ErrorCode
  candidate

return-pair is fn (package : (List ErrorCode, ErrorCode)) -> (List ErrorCode, ErrorCode)
  package

return-record is fn (package : Record (candidate : List ErrorCode, fallback : ErrorCode)) -> Record (candidate : List ErrorCode, fallback : ErrorCode)
  package

apply-package is fn ((candidate : List ErrorCode, fallback : ErrorCode)) -> ErrorCode
  head-or (candidate, fallback)

apply-pair : Function is { (candidate, fallback) } head-or (candidate, fallback)

out-of-range : ErrorCode is lang arithmetic out-of-range
not-representable : ErrorCode is lang arithmetic not-representable
division-by-zero : ErrorCode is lang arithmetic division-by-zero
indeterminate : ErrorCode is lang arithmetic indeterminate
values : List ErrorCode is Entry (out-of-range, Entry (not-representable, Entry (division-by-zero, Entry (indeterminate, Empty))))
copy : List ErrorCode is Entry (out-of-range, Entry (not-representable, Entry (division-by-zero, Entry (indeterminate, Empty))))
different : List ErrorCode is Entry (out-of-range, Entry (division-by-zero, Entry (not-representable, Entry (indeterminate, Empty))))
empty-values : List ErrorCode is Empty
forwarded is return-list values
paired is return-pair (values, division-by-zero)
recorded is return-record (candidate is empty-values, fallback is indeterminate)

(
  head-or (forwarded, indeterminate),
  second-or (forwarded, indeterminate),
  values = copy,
  values != different,
  entry-count values,
  empty? empty-values,
  apply-package (candidate is empty-values, fallback is division-by-zero),
  apply-pair paired,
  head-or (recorded candidate, recorded fallback),
  forwarded
)
