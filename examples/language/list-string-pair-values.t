#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordinary String-pair Lists across construction, equality,
# complete decomposition, private boundaries, products, and observations.
head-or is fn (candidate : List (String, String), fallback : (String, String)) -> (String, String)
  candidate
    Entry (first, rest) then first
    Empty then fallback

second-or is fn (candidate : List (String, String), fallback : (String, String)) -> (String, String)
  candidate
    Entry (ignored, rest) then head-or (rest, fallback)
    Empty then fallback

return-list is fn (candidate : List (String, String)) -> List (String, String)
  candidate

return-pair is fn (package : (List (String, String), (String, String))) -> (List (String, String), (String, String))
  package

return-record is fn (package : Record (candidate : List (String, String), fallback : (String, String))) -> Record (candidate : List (String, String), fallback : (String, String))
  package

apply-package is fn ((candidate : List (String, String), fallback : (String, String))) -> (String, String)
  head-or (candidate, fallback)

apply-pair : Function is { (candidate, fallback) } head-or (candidate, fallback)

values : List (String, String) is Entry (("one", "first"), Entry (("räv", "andra"), Entry (("", "最後"), Empty)))
copy : List (String, String) is Entry (("one", "first"), Entry (("räv", "andra"), Entry (("", "最後"), Empty)))
different-left : List (String, String) is Entry (("one", "first"), Entry (("fox", "andra"), Entry (("", "最後"), Empty)))
different-right : List (String, String) is Entry (("one", "first"), Entry (("räv", "second"), Entry (("", "最後"), Empty)))
shorter : List (String, String) is Entry (("one", "first"), Entry (("räv", "andra"), Empty))
empty-values : List (String, String) is Empty
forwarded is return-list values
paired is return-pair (values, ("nine", "ninth"))
recorded is return-record (candidate is empty-values, fallback is ("eleven", "elfte"))

(
  head-or (forwarded, ("seven", "seventh")),
  second-or (forwarded, ("seven", "seventh")),
  values = copy,
  values != different-left,
  values != different-right,
  values != shorter,
  entry-count values,
  empty? empty-values,
  apply-package (candidate is empty-values, fallback is ("seven", "seventh")),
  apply-pair paired,
  head-or (recorded candidate, recorded fallback),
  forwarded
)
