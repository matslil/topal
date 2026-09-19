#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordinary Optional Rational Lists across construction, equality,
# complete decomposition, private boundaries, products, and observations.
head-or is fn (candidate : List Optional Rational, fallback : Optional Rational) -> Optional Rational
  candidate
    Entry (first, rest) then first
    Empty then fallback

second-or is fn (candidate : List Optional Rational, fallback : Optional Rational) -> Optional Rational
  candidate
    Entry (ignored, rest) then head-or (rest, fallback)
    Empty then fallback

return-list is fn (candidate : List Optional Rational) -> List Optional Rational
  candidate

return-pair is fn (package : (List Optional Rational, Optional Rational)) -> (List Optional Rational, Optional Rational)
  package

return-record is fn (package : Record (candidate : List Optional Rational, fallback : Optional Rational)) -> Record (candidate : List Optional Rational, fallback : Optional Rational)
  package

apply-package is fn ((candidate : List Optional Rational, fallback : Optional Rational)) -> Optional Rational
  head-or (candidate, fallback)

apply-pair : Function is { (candidate, fallback) } head-or (candidate, fallback)

half : Rational is Rational (1, 2)
negative : Rational is Rational (-3, 4)
zero : Rational is Rational (0, 1)
head-fallback : Rational is Rational (7, 3)
pair-fallback : Rational is Rational (9, 2)
record-fallback : Rational is Rational (8, 5)
values : List Optional Rational is Entry (Some half, Entry (None Rational, Entry (Some negative, Empty)))
copy : List Optional Rational is Entry (Some half, Entry (None Rational, Entry (Some negative, Empty)))
different : List Optional Rational is Entry (Some half, Entry (Some zero, Entry (Some negative, Empty)))
shorter : List Optional Rational is Entry (Some half, Entry (None Rational, Empty))
empty-values : List Optional Rational is Empty
forwarded is return-list values
paired is return-pair (values, Some pair-fallback)
recorded is return-record (candidate is empty-values, fallback is Some record-fallback)

(
  head-or (forwarded, Some head-fallback),
  second-or (forwarded, Some head-fallback),
  values = copy,
  values != different,
  values != shorter,
  entry-count values,
  empty? empty-values,
  apply-package (candidate is empty-values, fallback is Some head-fallback),
  apply-pair paired,
  head-or (recorded candidate, recorded fallback),
  forwarded
)
