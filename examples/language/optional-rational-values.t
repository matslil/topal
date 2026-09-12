#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates Optional Rational construction, contextual absence, function
# passage, decisions, display, and derived canonical equality.
preserve is fn (candidate : Optional Rational) -> Optional Rational
  candidate
describe is fn (candidate : Optional Rational) -> String
  candidate
    Some value then "some Rational"
    None then "no Rational"
seven-halves is Rational (7, 2)
present is Some 3.5
missing : Optional Rational is None
(preserve present, preserve missing, present = (Some seven-halves), missing = (None Rational), present != missing, present != (Some 7.0), (present, missing) = ((Some seven-halves), (None Rational)), describe present, describe missing)
