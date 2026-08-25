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

median-int is fn (values : List Int) -> Optional Rational
  statistics-median values
median-rational is fn (values : List Rational) -> Optional Rational
  statistics-median values

### Return the exact median, or absence for an empty List.
pub median is fn (values : List Int) -> Optional Rational
  median-int values
pub median is fn (values : List Rational) -> Optional Rational
  median-rational values

### Return every most-frequent value in first-occurrence order.
pub modes is fn (values : List Int) -> List Int
  statistics-modes values
pub modes is fn (values : List Rational) -> List Rational
  statistics-modes values

### Return first-occurrence ordered value-frequency pairs.
pub histogram is fn (values : List Int) -> List (Int, Nat)
  statistics-histogram values
pub histogram is fn (values : List Rational) -> List (Rational, Nat)
  statistics-histogram values

### Return exact population variance, or absence for an empty List.
pub population-variance is fn (values : List Int) -> Optional Rational
  statistics-population-variance values
pub population-variance is fn (values : List Rational) -> Optional Rational
  statistics-population-variance values

### Return exact sample variance, or absence when fewer than two entries exist.
pub sample-variance is fn (values : List Int) -> Optional Rational
  statistics-sample-variance values
pub sample-variance is fn (values : List Rational) -> Optional Rational
  statistics-sample-variance values

### Return the linearly interpolated exact quantile for a probability in 0 ..= 1.
pub quantile is fn (values : List Int, probability : Rational) -> Optional Rational
  statistics-quantile (values, probability)
pub quantile is fn (values : List Rational, probability : Rational) -> Optional Rational
  statistics-quantile (values, probability)

### Return exact population covariance for equally sized paired Lists.
pub covariance is fn (left : List Int, right : List Int) -> Optional Rational
  statistics-covariance (left, right)
pub covariance is fn (left : List Rational, right : List Rational) -> Optional Rational
  statistics-covariance (left, right)

### Produce an exact mergeable summary: count, sum, and sum of squares.
pub summarize is fn (values : List Int) -> (Nat, Rational, Rational)
  statistics-summary values
pub summarize is fn (values : List Rational) -> (Nat, Rational, Rational)
  statistics-summary values

### Add one exact observation to a summary without retaining the sample.
pub summary-add is fn (
  summary : (Nat, Rational, Rational),
  value : Rational
) -> (Nat, Rational, Rational)
  statistics-summary-add (summary, value)

### Merge summaries of disjoint sample portions exactly.
pub summary-merge is fn (
  left : (Nat, Rational, Rational),
  right : (Nat, Rational, Rational)
) -> (Nat, Rational, Rational)
  statistics-summary-merge (left, right)

### Return the exact mean represented by a summary.
pub summary-mean is fn (summary : (Nat, Rational, Rational)) -> Optional Rational
  statistics-summary-mean summary

### Return the exact population variance represented by a summary.
pub summary-population-variance is fn (summary : (Nat, Rational, Rational)) -> Optional Rational
  statistics-summary-variance summary
