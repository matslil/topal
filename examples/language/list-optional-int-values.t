#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordinary Optional Int Lists across construction, equality,
# complete decomposition, private boundaries, products, and observations.
head-or is fn (candidate : List Optional Int, fallback : Optional Int) -> Optional Int
  candidate
    Entry (first, rest) then first
    Empty then fallback

second-or is fn (candidate : List Optional Int, fallback : Optional Int) -> Optional Int
  candidate
    Entry (ignored, rest) then head-or (rest, fallback)
    Empty then fallback

return-list is fn (candidate : List Optional Int) -> List Optional Int
  candidate

return-pair is fn (package : (List Optional Int, Optional Int)) -> (List Optional Int, Optional Int)
  package

return-record is fn (package : Record (candidate : List Optional Int, fallback : Optional Int)) -> Record (candidate : List Optional Int, fallback : Optional Int)
  package

apply-package is fn ((candidate : List Optional Int, fallback : Optional Int)) -> Optional Int
  head-or (candidate, fallback)

apply-pair : Function is { (candidate, fallback) } head-or (candidate, fallback)

values : List Optional Int is Entry (Some 1, Entry (None Int, Entry (Some (-2), Empty)))
copy : List Optional Int is Entry (Some 1, Entry (None Int, Entry (Some (-2), Empty)))
different : List Optional Int is Entry (Some 1, Entry (Some 0, Entry (Some (-2), Empty)))
shorter : List Optional Int is Entry (Some 1, Entry (None Int, Empty))
empty-values : List Optional Int is Empty
forwarded is return-list values
paired is return-pair (values, Some 9)
recorded is return-record (candidate is empty-values, fallback is Some 8)

(
  head-or (forwarded, Some 7),
  second-or (forwarded, Some 7),
  values = copy,
  values != different,
  values != shorter,
  entry-count values,
  empty? empty-values,
  apply-package (candidate is empty-values, fallback is Some 7),
  apply-pair paired,
  head-or (recorded candidate, recorded fallback),
  forwarded
)
