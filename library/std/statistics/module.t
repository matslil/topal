#!/usr/bin/env topal
use language (
  version is v0.1
)

### Revision of the exact descriptive-statistics namespace.
pub revision is 1

### Return the exact arithmetic mean of Int entries, or absence for an empty List.
pub mean is fn (values : List Int) -> Optional Rational
  count is entry-count values
  total is values fold 0 { sum, value } sum + value
  count
    <= 0 then None Rational
    otherwise Some ((Rational total) / (Rational count))

### Return the exact arithmetic mean of Rational entries, or absence for an empty List.
pub mean is fn (values : List Rational) -> Optional Rational
  count is entry-count values
  total is values fold (Rational 0) { sum, value } sum + value
  count
    <= 0 then None Rational
    otherwise Some (total / (Rational count))
