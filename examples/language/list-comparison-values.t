#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordinary Comparison Lists across construction, equality, complete
# decomposition, private boundaries, products, and observations.
head-or is fn (candidate : List Comparison, fallback : Comparison) -> Comparison
  candidate
    Entry (first, rest) then first
    Empty then fallback

second-or is fn (candidate : List Comparison, fallback : Comparison) -> Comparison
  candidate
    Entry (ignored, rest) then head-or (rest, fallback)
    Empty then fallback

return-list is fn (candidate : List Comparison) -> List Comparison
  candidate

return-pair is fn (package : (List Comparison, Comparison)) -> (List Comparison, Comparison)
  package

return-record is fn (package : Record (candidate : List Comparison, fallback : Comparison)) -> Record (candidate : List Comparison, fallback : Comparison)
  package

apply-package is fn ((candidate : List Comparison, fallback : Comparison)) -> Comparison
  head-or (candidate, fallback)

apply-pair : Function is { (candidate, fallback) } head-or (candidate, fallback)

less : Comparison is 1 <=> 2
equal : Comparison is 2 <=> 2
greater : Comparison is 3 <=> 2
values : List Comparison is Entry (less, Entry (equal, Entry (greater, Empty)))
copy : List Comparison is Entry (less, Entry (equal, Entry (greater, Empty)))
different : List Comparison is Entry (less, Entry (greater, Entry (equal, Empty)))
empty-values : List Comparison is Empty
forwarded is return-list values
paired is return-pair (values, equal)
recorded is return-record (candidate is empty-values, fallback is greater)

(
  head-or (forwarded, greater),
  second-or (forwarded, greater),
  values = copy,
  values != different,
  entry-count values,
  empty? empty-values,
  apply-package (candidate is empty-values, fallback is equal),
  apply-pair paired,
  head-or (recorded candidate, recorded fallback),
  forwarded
)
