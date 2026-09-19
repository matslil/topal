#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordinary Unit Lists across construction, equality, complete
# decomposition, private boundaries, products, and observations.
head-or is fn (candidate : List Unit, fallback : Unit) -> Unit
  candidate
    Entry (first, rest) then first
    Empty then fallback

second-or is fn (candidate : List Unit, fallback : Unit) -> Unit
  candidate
    Entry (ignored, rest) then head-or (rest, fallback)
    Empty then fallback

return-list is fn (candidate : List Unit) -> List Unit
  candidate

return-pair is fn (package : (List Unit, Unit)) -> (List Unit, Unit)
  package

return-record is fn (package : Record (candidate : List Unit, fallback : Unit)) -> Record (candidate : List Unit, fallback : Unit)
  package

apply-package is fn ((candidate : List Unit, fallback : Unit)) -> Unit
  head-or (candidate, fallback)

apply-pair : Function is { (candidate, fallback) } head-or (candidate, fallback)

unit : Unit is ()
values : List Unit is Entry (unit, Entry (unit, Entry (unit, Empty)))
copy : List Unit is Entry (unit, Entry (unit, Entry (unit, Empty)))
different : List Unit is Entry (unit, Entry (unit, Empty))
empty-values : List Unit is Empty
forwarded is return-list values
paired is return-pair (values, unit)
recorded is return-record (candidate is empty-values, fallback is unit)

(
  head-or (forwarded, unit),
  second-or (forwarded, unit),
  values = copy,
  values != different,
  entry-count values,
  empty? empty-values,
  apply-package (candidate is empty-values, fallback is unit),
  apply-pair paired,
  head-or (recorded candidate, recorded fallback),
  forwarded
)
