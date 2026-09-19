#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordinary fundamental Type Lists across construction, equality,
# complete decomposition, private boundaries, products, and observations.
head-or is fn (candidate : List Type, fallback : Type) -> Type
  candidate
    Entry (first, rest) then first
    Empty then fallback

second-or is fn (candidate : List Type, fallback : Type) -> Type
  candidate
    Entry (ignored, rest) then head-or (rest, fallback)
    Empty then fallback

return-list is fn (candidate : List Type) -> List Type
  candidate

return-pair is fn (package : (List Type, Type)) -> (List Type, Type)
  package

return-record is fn (package : Record (candidate : List Type, fallback : Type)) -> Record (candidate : List Type, fallback : Type)
  package

apply-package is fn ((candidate : List Type, fallback : Type)) -> Type
  head-or (candidate, fallback)

apply-pair : Function is { (candidate, fallback) } head-or (candidate, fallback)

values : List Type is Entry (Boolean, Entry (Int, Entry (Nat, Entry (Rational, Entry (String, Entry (Unit, Entry (Scope, Empty)))))))
copy : List Type is Entry (Boolean, Entry (Int, Entry (Nat, Entry (Rational, Entry (String, Entry (Unit, Entry (Scope, Empty)))))))
different : List Type is Entry (Boolean, Entry (Int, Entry (Nat, Entry (Rational, Entry (String, Entry (Unit, Entry (Boolean, Empty)))))))
shorter : List Type is Entry (Boolean, Entry (Int, Entry (Nat, Entry (Rational, Entry (String, Entry (Unit, Empty))))))
empty-values : List Type is Empty
forwarded is return-list values
paired is return-pair (values, Scope)
recorded is return-record (candidate is empty-values, fallback is Unit)

(
  head-or (forwarded, String),
  second-or (forwarded, String),
  values = copy,
  values != different,
  values != shorter,
  entry-count values,
  empty? empty-values,
  apply-package (candidate is empty-values, fallback is String),
  apply-pair paired,
  head-or (recorded candidate, recorded fallback),
  forwarded
)
