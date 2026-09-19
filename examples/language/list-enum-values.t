#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordinary payload-free nominal Enum Lists across construction,
# equality, complete decomposition, private boundaries, and observations.
Color is Enum (Red, Green, Blue)

head-or is fn (candidate : List Color, fallback : Color) -> Color
  candidate
    Entry (first, rest) then first
    Empty then fallback

second-or is fn (candidate : List Color, fallback : Color) -> Color
  candidate
    Entry (ignored, rest) then head-or (rest, fallback)
    Empty then fallback

return-list is fn (candidate : List Color) -> List Color
  candidate

return-pair is fn (package : (List Color, Color)) -> (List Color, Color)
  package

return-record is fn (package : Record (candidate : List Color, fallback : Color)) -> Record (candidate : List Color, fallback : Color)
  package

apply-package is fn ((candidate : List Color, fallback : Color)) -> Color
  head-or (candidate, fallback)

apply-pair : Function is { (candidate, fallback) } head-or (candidate, fallback)

values : List Color is Entry (Red, Entry (Green, Entry (Blue, Empty)))
copy : List Color is Entry (Red, Entry (Green, Entry (Blue, Empty)))
different : List Color is Entry (Red, Entry (Blue, Entry (Green, Empty)))
shorter : List Color is Entry (Red, Entry (Green, Empty))
empty-values : List Color is Empty
forwarded is return-list values
paired is return-pair (values, Blue)
recorded is return-record (candidate is empty-values, fallback is Green)

(
  head-or (forwarded, Blue),
  second-or (forwarded, Blue),
  values = copy,
  values != different,
  values != shorter,
  entry-count values,
  empty? empty-values,
  apply-package (candidate is empty-values, fallback is Blue),
  apply-pair paired,
  head-or (recorded candidate, recorded fallback),
  forwarded
)
