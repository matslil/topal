#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordinary Character Lists across construction, structural
# equality, total decomposition, private boundaries, products, and observations.
head-or is fn (candidate : List Character, fallback : Character) -> Character
  candidate
    Entry (first, rest) then first
    Empty then fallback

second-or is fn (candidate : List Character, fallback : Character) -> Character
  candidate
    Entry (ignored, rest) then head-or (rest, fallback)
    Empty then fallback

return-list is fn (candidate : List Character) -> List Character
  candidate

return-pair is fn (package : (List Character, Character)) -> (List Character, Character)
  package

return-record is fn (package : Record (candidate : List Character, fallback : Character)) -> Record (candidate : List Character, fallback : Character)
  package

apply-package is fn ((candidate : List Character, fallback : Character)) -> Character
  head-or (candidate, fallback)

apply-pair : Function is { (candidate, fallback) } head-or (candidate, fallback)

values : List Character is Entry ("Å", Entry ("👩‍💻", Empty))
copy : List Character is Entry ("Å", Entry ("👩‍💻", Empty))
different : List Character is Entry ("Å", Entry ("Ω", Empty))
empty-values : List Character is Empty
missing : Character is "?"
pair-fallback : Character is "P"
record-fallback : Character is "R"
package-fallback : Character is "K"
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
