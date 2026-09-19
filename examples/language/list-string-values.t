#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordinary String Lists across construction, structural equality,
# total decomposition, private boundaries, products, and observations.
head-or is fn (candidate : List String, fallback : String) -> String
  candidate
    Entry (first, rest) then first
    Empty then fallback

second-or is fn (candidate : List String, fallback : String) -> String
  candidate
    Entry (ignored, rest) then head-or (rest, fallback)
    Empty then fallback

return-list is fn (candidate : List String) -> List String
  candidate

return-pair is fn (package : (List String, String)) -> (List String, String)
  package

return-record is fn (package : Record (candidate : List String, fallback : String)) -> Record (candidate : List String, fallback : String)
  package

apply-package is fn ((candidate : List String, fallback : String)) -> String
  head-or (candidate, fallback)

apply-pair : Function is { (candidate, fallback) } head-or (candidate, fallback)

values : List String is Entry ("Top", Entry ("al", Empty))
copy : List String is Entry ("Top", Entry ("al", Empty))
different : List String is Entry ("Top", Entry ("az", Empty))
empty-values : List String is Empty
forwarded is return-list values
paired is return-pair (values, "pair")
recorded is return-record (candidate is empty-values, fallback is "record")

(
  head-or (forwarded, "missing"),
  second-or (forwarded, "missing"),
  values = copy,
  values != different,
  entry-count values,
  empty? empty-values,
  apply-package (candidate is empty-values, fallback is "package"),
  apply-pair paired,
  head-or (recorded candidate, recorded fallback),
  forwarded
)
