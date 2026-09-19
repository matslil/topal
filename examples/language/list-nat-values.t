#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordinary Nat Lists across construction, structural equality,
# total decomposition, private boundaries, products, and observations.
head-or is fn (candidate : List Nat, fallback : Nat) -> Nat
  candidate
    Entry (first, rest) then first
    Empty then fallback

second-or is fn (candidate : List Nat, fallback : Nat) -> Nat
  candidate
    Entry (ignored, rest) then head-or (rest, fallback)
    Empty then fallback

return-list is fn (candidate : List Nat) -> List Nat
  candidate

return-pair is fn (package : (List Nat, Nat)) -> (List Nat, Nat)
  package

return-record is fn (package : Record (candidate : List Nat, fallback : Nat)) -> Record (candidate : List Nat, fallback : Nat)
  package

apply-package is fn ((candidate : List Nat, fallback : Nat)) -> Nat
  head-or (candidate, fallback)

apply-pair : Function is { (candidate, fallback) } head-or (candidate, fallback)

positive-infinity : Nat is +Infinity
values : List Nat is Entry (0, Entry (123456789012345678901234567890, Entry (positive-infinity, Empty)))
copy : List Nat is Entry (0, Entry (123456789012345678901234567890, Entry (positive-infinity, Empty)))
different : List Nat is Entry (0, Entry (123456789012345678901234567890, Entry (2, Empty)))
empty-values : List Nat is Empty
missing : Nat is 9
pair-fallback : Nat is 7
record-fallback : Nat is 6
package-fallback : Nat is 5
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
