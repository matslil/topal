#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates inert first-class Effect Lists across construction, structural
# equality, total decomposition, private boundaries, products, and observations.
head-or is fn (candidate : List Effect, fallback : Effect) -> Effect
  candidate
    Entry (first, rest) then first
    Empty then fallback

second-or is fn (candidate : List Effect, fallback : Effect) -> Effect
  candidate
    Entry (ignored, rest) then head-or (rest, fallback)
    Empty then fallback

return-list is fn (candidate : List Effect) -> List Effect
  candidate

return-pair is fn (package : (List Effect, Effect)) -> (List Effect, Effect)
  package

return-record is fn (package : Record (candidate : List Effect, fallback : Effect)) -> Record (candidate : List Effect, fallback : Effect)
  package

apply-package is fn ((candidate : List Effect, fallback : Effect)) -> Effect
  head-or (candidate, fallback)

apply-pair : Function is { (candidate, fallback) } head-or (candidate, fallback)

empty-effect : Effect is Effects ()
values : List Effect is Entry (empty-effect, Entry (empty-effect, Empty))
copy : List Effect is Entry (empty-effect, Entry (empty-effect, Empty))
different : List Effect is Entry (empty-effect, Empty)
empty-values : List Effect is Empty
forwarded is return-list values
paired is return-pair (values, empty-effect)
recorded is return-record (candidate is empty-values, fallback is empty-effect)

(
  head-or (forwarded, empty-effect),
  second-or (forwarded, empty-effect),
  values = copy,
  values != different,
  entry-count values,
  empty? empty-values,
  apply-package (candidate is empty-values, fallback is empty-effect),
  apply-pair paired,
  head-or (recorded candidate, recorded fallback),
  forwarded
)
