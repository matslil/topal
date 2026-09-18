#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates exact repeated-name matching for already-admitted immutable
# aggregate values without conversion, re-evaluation, or a pattern runtime.
make-pair is fn (number : Int, text : String) -> (Int, String)
  (number, text)

repeat-tuple : Function is { value, value } value
repeat-record : Function is { value, value } value
repeat-optional : Function is { value, value } value
repeat-list : Function is { value, value } value

pair is make-pair (7, "seven")
person is (active is true, name is "Ada")
candidate : Optional Int is Some 9
numbers : List Int is Entry (1, Entry (2, Empty))

(repeat-tuple (pair, pair), repeat-record (person, person), repeat-optional (candidate, candidate), repeat-list (numbers, numbers))
