#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordinary String/Int-pair Lists across construction, equality,
# complete decomposition, private boundaries, products, and observations.
head-or is fn (candidate : List (String, Int), fallback : (String, Int)) -> (String, Int)
  candidate
    Entry (first, rest) then first
    Empty then fallback

second-or is fn (candidate : List (String, Int), fallback : (String, Int)) -> (String, Int)
  candidate
    Entry (ignored, rest) then head-or (rest, fallback)
    Empty then fallback

return-list is fn (candidate : List (String, Int)) -> List (String, Int)
  candidate

return-pair is fn (package : (List (String, Int), (String, Int))) -> (List (String, Int), (String, Int))
  package

return-record is fn (package : Record (candidate : List (String, Int), fallback : (String, Int))) -> Record (candidate : List (String, Int), fallback : (String, Int))
  package

apply-package is fn ((candidate : List (String, Int), fallback : (String, Int))) -> (String, Int)
  head-or (candidate, fallback)

apply-pair : Function is { (candidate, fallback) } head-or (candidate, fallback)

large : Int is 340282366920938463463374607431768211456
negative : Int is -170141183460469231731687303715884105728
values : List (String, Int) is Entry (("one", 1), Entry (("räv", -4), Entry (("", large), Empty)))
copy : List (String, Int) is Entry (("one", 1), Entry (("räv", -4), Entry (("", large), Empty)))
different-string : List (String, Int) is Entry (("one", 1), Entry (("fox", -4), Entry (("", large), Empty)))
different-int : List (String, Int) is Entry (("one", 1), Entry (("räv", negative), Entry (("", large), Empty)))
shorter : List (String, Int) is Entry (("one", 1), Entry (("räv", -4), Empty))
empty-values : List (String, Int) is Empty
forwarded is return-list values
paired is return-pair (values, ("nine", 9))
recorded is return-record (candidate is empty-values, fallback is ("eleven", 11))

(
  head-or (forwarded, ("seven", 7)),
  second-or (forwarded, ("seven", 7)),
  values = copy,
  values != different-string,
  values != different-int,
  values != shorter,
  entry-count values,
  empty? empty-values,
  apply-package (candidate is empty-values, fallback is ("seven", 7)),
  apply-pair paired,
  head-or (recorded candidate, recorded fallback),
  forwarded
)
