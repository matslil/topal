#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordinary Completed Lists across construction, equality, complete
# decomposition, private boundaries, products, and observations.
head-or is fn (candidate : List Completed, fallback : Completed) -> Completed
  candidate
    Entry (first, rest) then first
    Empty then fallback

second-or is fn (candidate : List Completed, fallback : Completed) -> Completed
  candidate
    Entry (ignored, rest) then head-or (rest, fallback)
    Empty then fallback

return-list is fn (candidate : List Completed) -> List Completed
  candidate

return-pair is fn (package : (List Completed, Completed)) -> (List Completed, Completed)
  package

return-record is fn (package : Record (candidate : List Completed, fallback : Completed)) -> Record (candidate : List Completed, fallback : Completed)
  package

apply-package is fn ((candidate : List Completed, fallback : Completed)) -> Completed
  head-or (candidate, fallback)

apply-pair : Function is { (candidate, fallback) } head-or (candidate, fallback)

completed : Completed is Completed
values : List Completed is Entry (completed, Entry (completed, Entry (completed, Empty)))
copy : List Completed is Entry (completed, Entry (completed, Entry (completed, Empty)))
different : List Completed is Entry (completed, Entry (completed, Empty))
empty-values : List Completed is Empty
forwarded is return-list values
paired is return-pair (values, completed)
recorded is return-record (candidate is empty-values, fallback is completed)

(
  head-or (forwarded, completed),
  second-or (forwarded, completed),
  values = copy,
  values != different,
  entry-count values,
  empty? empty-values,
  apply-package (candidate is empty-values, fallback is completed),
  apply-pair paired,
  head-or (recorded candidate, recorded fallback),
  forwarded
)
