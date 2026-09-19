#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordinary Boolean Lists across construction, structural equality,
# total decomposition, private boundaries, products, and observations.
head-or is fn (candidate : List Boolean, fallback : Boolean) -> Boolean
  candidate
    Entry (first, rest) then first
    Empty then fallback

second-or is fn (candidate : List Boolean, fallback : Boolean) -> Boolean
  candidate
    Entry (ignored, rest) then head-or (rest, fallback)
    Empty then fallback

return-list is fn (candidate : List Boolean) -> List Boolean
  candidate

return-pair is fn (package : (List Boolean, Boolean)) -> (List Boolean, Boolean)
  package

return-record is fn (package : Record (candidate : List Boolean, fallback : Boolean)) -> Record (candidate : List Boolean, fallback : Boolean)
  package

apply-package is fn ((candidate : List Boolean, fallback : Boolean)) -> Boolean
  head-or (candidate, fallback)

apply-pair : Function is { (candidate, fallback) } head-or (candidate, fallback)

values : List Boolean is Entry (true, Entry (false, Empty))
copy : List Boolean is Entry (true, Entry (false, Empty))
different : List Boolean is Entry (true, Entry (true, Empty))
empty-values : List Boolean is Empty
forwarded is return-list values
paired is return-pair (values, false)
recorded is return-record (candidate is empty-values, fallback is false)

(
  head-or (forwarded, false),
  second-or (forwarded, true),
  values = copy,
  values != different,
  entry-count values,
  empty? empty-values,
  apply-package (candidate is empty-values, fallback is true),
  apply-pair paired,
  head-or (recorded candidate, recorded fallback),
  forwarded
)
