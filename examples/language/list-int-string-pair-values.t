#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordinary Int/String-pair Lists across construction, equality,
# complete decomposition, private boundaries, products, and observations.
head-or is fn (candidate : List (Int, String), fallback : (Int, String)) -> (Int, String)
  candidate
    Entry (first, rest) then first
    Empty then fallback

second-or is fn (candidate : List (Int, String), fallback : (Int, String)) -> (Int, String)
  candidate
    Entry (ignored, rest) then head-or (rest, fallback)
    Empty then fallback

return-list is fn (candidate : List (Int, String)) -> List (Int, String)
  candidate

return-pair is fn (package : (List (Int, String), (Int, String))) -> (List (Int, String), (Int, String))
  package

return-record is fn (package : Record (candidate : List (Int, String), fallback : (Int, String))) -> Record (candidate : List (Int, String), fallback : (Int, String))
  package

apply-package is fn ((candidate : List (Int, String), fallback : (Int, String))) -> (Int, String)
  head-or (candidate, fallback)

apply-pair : Function is { (candidate, fallback) } head-or (candidate, fallback)

large : Int is 340282366920938463463374607431768211456
negative : Int is -170141183460469231731687303715884105728
values : List (Int, String) is Entry ((1, "one"), Entry ((-4, "räv"), Entry ((large, ""), Empty)))
copy : List (Int, String) is Entry ((1, "one"), Entry ((-4, "räv"), Entry ((large, ""), Empty)))
different-int : List (Int, String) is Entry ((1, "one"), Entry ((negative, "räv"), Entry ((large, ""), Empty)))
different-string : List (Int, String) is Entry ((1, "one"), Entry ((-4, "fox"), Entry ((large, ""), Empty)))
shorter : List (Int, String) is Entry ((1, "one"), Entry ((-4, "räv"), Empty))
empty-values : List (Int, String) is Empty
forwarded is return-list values
paired is return-pair (values, (9, "nine"))
recorded is return-record (candidate is empty-values, fallback is (11, "eleven"))

(
  head-or (forwarded, (7, "seven")),
  second-or (forwarded, (7, "seven")),
  values = copy,
  values != different-int,
  values != different-string,
  values != shorter,
  entry-count values,
  empty? empty-values,
  apply-package (candidate is empty-values, fallback is (7, "seven")),
  apply-pair paired,
  head-or (recorded candidate, recorded fallback),
  forwarded
)
