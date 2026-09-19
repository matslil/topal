#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordinary Rational Lists across construction, equality, complete
# decomposition, private boundaries, products, and observations.
head-or is fn (candidate : List Rational, fallback : Rational) -> Rational
  candidate
    Entry (first, rest) then first
    Empty then fallback

second-or is fn (candidate : List Rational, fallback : Rational) -> Rational
  candidate
    Entry (ignored, rest) then head-or (rest, fallback)
    Empty then fallback

return-list is fn (candidate : List Rational) -> List Rational
  candidate

return-pair is fn (package : (List Rational, Rational)) -> (List Rational, Rational)
  package

return-record is fn (package : Record (candidate : List Rational, fallback : Rational)) -> Record (candidate : List Rational, fallback : Rational)
  package

apply-package is fn ((candidate : List Rational, fallback : Rational)) -> Rational
  head-or (candidate, fallback)

apply-pair : Function is { (candidate, fallback) } head-or (candidate, fallback)

half : Rational is Rational (1, 2)
large : Rational is Rational (123456789012345678901234567890, 7)
positive-infinity : Rational is +Infinity
different-last : Rational is Rational (2, 3)
values : List Rational is Entry (half, Entry (large, Entry (positive-infinity, Empty)))
copy : List Rational is Entry (half, Entry (large, Entry (positive-infinity, Empty)))
different : List Rational is Entry (half, Entry (large, Entry (different-last, Empty)))
empty-values : List Rational is Empty
missing : Rational is Rational (9, 1)
pair-fallback : Rational is Rational (7, 1)
record-fallback : Rational is Rational (6, 1)
package-fallback : Rational is Rational (5, 1)
forwarded is return-list values
paired is return-pair (values, pair-fallback)
recorded is return-record (candidate is empty-values, fallback is record-fallback)

(
  head-or (forwarded, missing),
  second-or (forwarded, missing),
  values = copy,
  values != different,
  entry-count values,
  empty? empty-values,
  apply-package (candidate is empty-values, fallback is package-fallback),
  apply-pair paired,
  head-or (recorded candidate, recorded fallback),
  forwarded
)
