#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordinary Optional String Lists across construction, equality,
# complete decomposition, private boundaries, products, and observations.
head-or is fn (candidate : List Optional String, fallback : Optional String) -> Optional String
  candidate
    Entry (first, rest) then first
    Empty then fallback

second-or is fn (candidate : List Optional String, fallback : Optional String) -> Optional String
  candidate
    Entry (ignored, rest) then head-or (rest, fallback)
    Empty then fallback

return-list is fn (candidate : List Optional String) -> List Optional String
  candidate

return-pair is fn (package : (List Optional String, Optional String)) -> (List Optional String, Optional String)
  package

return-record is fn (package : Record (candidate : List Optional String, fallback : Optional String)) -> Record (candidate : List Optional String, fallback : Optional String)
  package

apply-package is fn ((candidate : List Optional String, fallback : Optional String)) -> Optional String
  head-or (candidate, fallback)

apply-pair : Function is { (candidate, fallback) } head-or (candidate, fallback)

values : List Optional String is Entry (Some "first", Entry (None String, Entry (Some "世界", Empty)))
copy : List Optional String is Entry (Some "first", Entry (None String, Entry (Some "世界", Empty)))
different : List Optional String is Entry (Some "first", Entry (Some "", Entry (Some "世界", Empty)))
shorter : List Optional String is Entry (Some "first", Entry (None String, Empty))
empty-values : List Optional String is Empty
forwarded is return-list values
paired is return-pair (values, Some "pair")
recorded is return-record (candidate is empty-values, fallback is Some "record")

(
  head-or (forwarded, Some "fallback"),
  second-or (forwarded, Some "fallback"),
  values = copy,
  values != different,
  values != shorter,
  entry-count values,
  empty? empty-values,
  apply-package (candidate is empty-values, fallback is Some "fallback"),
  apply-pair paired,
  head-or (recorded candidate, recorded fallback),
  forwarded
)
