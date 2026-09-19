#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordinary Int-pair Lists across construction, equality, complete
# decomposition, private boundaries, products, and observations.
head-or is fn (candidate : List (Int, Int), fallback : (Int, Int)) -> (Int, Int)
  candidate
    Entry (first, rest) then first
    Empty then fallback

second-or is fn (candidate : List (Int, Int), fallback : (Int, Int)) -> (Int, Int)
  candidate
    Entry (ignored, rest) then head-or (rest, fallback)
    Empty then fallback

return-list is fn (candidate : List (Int, Int)) -> List (Int, Int)
  candidate

return-pair is fn (package : (List (Int, Int), (Int, Int))) -> (List (Int, Int), (Int, Int))
  package

return-record is fn (package : Record (candidate : List (Int, Int), fallback : (Int, Int))) -> Record (candidate : List (Int, Int), fallback : (Int, Int))
  package

apply-package is fn ((candidate : List (Int, Int), fallback : (Int, Int))) -> (Int, Int)
  head-or (candidate, fallback)

apply-pair : Function is { (candidate, fallback) } head-or (candidate, fallback)

large : Int is 340282366920938463463374607431768211456
negative : Int is -170141183460469231731687303715884105728
values : List (Int, Int) is Entry ((1, 2), Entry ((3, -4), Entry ((large, negative), Empty)))
copy : List (Int, Int) is Entry ((1, 2), Entry ((3, -4), Entry ((large, negative), Empty)))
different : List (Int, Int) is Entry ((1, 2), Entry ((3, 5), Entry ((large, negative), Empty)))
shorter : List (Int, Int) is Entry ((1, 2), Entry ((3, -4), Empty))
empty-values : List (Int, Int) is Empty
forwarded is return-list values
paired is return-pair (values, (9, 10))
recorded is return-record (candidate is empty-values, fallback is (11, 12))

(
  head-or (forwarded, (7, 8)),
  second-or (forwarded, (7, 8)),
  values = copy,
  values != different,
  values != shorter,
  entry-count values,
  empty? empty-values,
  apply-package (candidate is empty-values, fallback is (7, 8)),
  apply-pair paired,
  head-or (recorded candidate, recorded fallback),
  forwarded
)
