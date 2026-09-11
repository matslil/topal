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

increment-when is fn (count : Nat, accepted : Boolean) -> Nat
  accepted
    true then count + 1
    false then count

lower-bound is fn (values : List (Value : TotalOrder), sought : Value) -> Nat
  values fold 0 { count, candidate } increment-when (count, candidate < sought)

insert-ordered is fn (candidate : (Value : TotalOrder), values : List Value) -> List Value
  boundary is lower-bound (values, candidate)
  length is entry-count values
  (values select-index (0 .. boundary)) concat (one candidate) concat (values select-index (boundary .. length))

sort-step is fn (candidate : (Value : TotalOrder), sorted : List Value) -> List Value
  insert-ordered (candidate, sorted)

sort-source is fn (values : List (Value : TotalOrder)) -> List Value
  sorted : List Value is Empty
  values fold sorted { collected, candidate } sort-step (candidate, collected)

present-rational is fn (value : Rational) -> Optional Rational
  present : Optional Rational is Some value
  present

median-rational-upper is fn (lower : Rational, upper-value : Optional Rational) -> Optional Rational
  upper-value
    Some upper-entry then present-rational ((lower + upper-entry) / (Rational 2))
    None then None Rational

median-rational-lower is fn (lower-value : Optional Rational, upper-value : Optional Rational) -> Optional Rational
  lower-value
    Some lower then median-rational-upper (lower, upper-value)
    None then None Rational

median-rational-even is fn (sorted : List Rational, upper : Nat) -> Optional Rational
  lower-value is first (sorted select-index ((upper - 1) ..= (upper - 1)))
  upper-value is first (sorted select-index (upper ..= upper))
  median-rational-lower (lower-value, upper-value)

median-rational-nonempty is fn (sorted : List Rational, length : Nat) -> Optional Rational
  half-pair is length /% 2
  half is fn (quotient : Int, remainder : Int) -> Int
    quotient
  middle : Nat is Nat (half half-pair)
  length % 2 = 1
    true then first (sorted select-index (middle ..= middle))
    false then median-rational-even (sorted, middle)

median-rational is fn (values : List Rational) -> Optional Rational
  sorted is sort-source values
  length is entry-count sorted
  length = 0
    true then None Rational
    false then median-rational-nonempty (sorted, length)

median-int-upper is fn (lower : Int, upper-value : Optional Int) -> Optional Rational
  upper-value
    Some upper-entry then present-rational ((Rational (lower + upper-entry)) / (Rational 2))
    None then None Rational

median-int-lower is fn (lower-value : Optional Int, upper-value : Optional Int) -> Optional Rational
  lower-value
    Some lower then median-int-upper (lower, upper-value)
    None then None Rational

median-int-even is fn (sorted : List Int, upper : Nat) -> Optional Rational
  lower-value is first (sorted select-index ((upper - 1) ..= (upper - 1)))
  upper-value is first (sorted select-index (upper ..= upper))
  median-int-lower (lower-value, upper-value)

median-int-odd is fn (value : Optional Int) -> Optional Rational
  value
    Some present then present-rational (Rational present)
    None then None Rational

median-int-nonempty is fn (sorted : List Int, length : Nat) -> Optional Rational
  half-pair is length /% 2
  half is fn (quotient : Int, remainder : Int) -> Int
    quotient
  middle : Nat is Nat (half half-pair)
  length % 2 = 1
    true then median-int-odd (first (sorted select-index (middle ..= middle)))
    false then median-int-even (sorted, middle)

median-int is fn (values : List Int) -> Optional Rational
  sorted is sort-source values
  length is entry-count sorted
  length = 0
    true then None Rational
    false then median-int-nonempty (sorted, length)

### Return the exact median, or absence for an empty List.
pub median is fn (values : List Int) -> Optional Rational
  median-int values
### Return the exact median, or absence for an empty List.
pub median is fn (values : List Rational) -> Optional Rational
  median-rational values

unique-step is fn (selected : List (Value : Equality), candidate : Value) -> List Value
  selected contains-entry candidate
    true then selected
    false then selected append candidate

unique-source is fn (values : List (Value : Equality)) -> List Value
  values fold (Empty Value) { selected, candidate } unique-step (selected, candidate)

count-int is fn (values : List Int, sought : Int) -> Nat
  zero-count : Nat is Nat 0
  values fold zero-count { count, candidate } increment-when (count, candidate = sought)

int-frequency is fn (values : List Int, candidate : Int) -> (Int, Nat)
  (candidate, count-int (values, candidate))

histogram-int-step is fn ((values : List Int, collected : List (Int, Nat), candidate : Int)) -> List (Int, Nat)
  collected append (int-frequency (values, candidate))

histogram-int-source is fn (values : List Int) -> List (Int, Nat)
  empty-histogram : List (Int, Nat) is Empty
  (unique-source values) fold empty-histogram { collected, candidate } histogram-int-step (values, collected, candidate)

count-rational is fn (values : List Rational, sought : Rational) -> Nat
  zero-count : Nat is Nat 0
  values fold zero-count { count, candidate } increment-when (count, candidate = sought)

rational-frequency is fn (values : List Rational, candidate : Rational) -> (Rational, Nat)
  (candidate, count-rational (values, candidate))

histogram-rational-step is fn ((values : List Rational, collected : List (Rational, Nat), candidate : Rational)) -> List (Rational, Nat)
  collected append (rational-frequency (values, candidate))

histogram-rational-source is fn (values : List Rational) -> List (Rational, Nat)
  empty-histogram : List (Rational, Nat) is Empty
  (unique-source values) fold empty-histogram { collected, candidate } histogram-rational-step (values, collected, candidate)

pair-value is fn ((value : Int, count : Nat)) -> Int
  value
pair-count is fn ((value : Int, count : Nat)) -> Nat
  count
rational-pair-value is fn ((value : Rational, count : Nat)) -> Rational
  value
rational-pair-count is fn ((value : Rational, count : Nat)) -> Nat
  count

maximum-count is fn (left : Nat, right : Nat) -> Nat
  left < right
    true then right
    false then left

select-int-mode is fn ((selected : List Int, pair : (Int, Nat), maximum : Nat)) -> List Int
  (pair-count pair) = maximum
    true then selected append (pair-value pair)
    false then selected

select-rational-mode is fn ((selected : List Rational, pair : (Rational, Nat), maximum : Nat)) -> List Rational
  (rational-pair-count pair) = maximum
    true then selected append (rational-pair-value pair)
    false then selected

### Return every most-frequent value in first-occurrence order.
pub modes is fn (values : List Int) -> List Int
  histogram-values is histogram-int-source values
  maximum is histogram-values fold 0 { count, pair } maximum-count (count, pair-count pair)
  histogram-values fold (Empty Int) { selected, pair } select-int-mode (selected, pair, maximum)
### Return every most-frequent value in first-occurrence order.
pub modes is fn (values : List Rational) -> List Rational
  histogram-values is histogram-rational-source values
  maximum is histogram-values fold 0 { count, pair } maximum-count (count, rational-pair-count pair)
  histogram-values fold (Empty Rational) { selected, pair } select-rational-mode (selected, pair, maximum)

### Return first-occurrence ordered value-frequency pairs.
pub histogram is fn (values : List Int) -> List (Int, Nat)
  histogram-int-source values
### Return first-occurrence ordered value-frequency pairs.
pub histogram is fn (values : List Rational) -> List (Rational, Nat)
  histogram-rational-source values

summary-count is fn ((count : Nat, sum : Rational, squares : Rational)) -> Nat
  count
summary-sum is fn ((count : Nat, sum : Rational, squares : Rational)) -> Rational
  sum
summary-squares is fn ((count : Nat, sum : Rational, squares : Rational)) -> Rational
  squares

summary-add-source is fn (summary : (Nat, Rational, Rational), value : Rational) -> (Nat, Rational, Rational)
  ((summary-count summary) + 1, (summary-sum summary) + value, (summary-squares summary) + (value * value))

summarize-rational is fn (values : List Rational) -> (Nat, Rational, Rational)
  values fold (0, Rational 0, Rational 0) { summary, value } summary-add-source (summary, value)

summarize-int-step is fn (summary : (Nat, Rational, Rational), value : Int) -> (Nat, Rational, Rational)
  summary-add-source (summary, Rational value)

summarize-int is fn (values : List Int) -> (Nat, Rational, Rational)
  values fold (0, Rational 0, Rational 0) { summary, value } summarize-int-step (summary, value)

variance-denominator is fn (count : Nat, sample? : Boolean) -> Int
  sample?
    true then count - 1
    false then count

variance-present is fn ((summary : (Nat, Rational, Rational), count : Nat, denominator : Int)) -> Optional Rational
  sum is summary-sum summary
  squares is summary-squares summary
  present-rational ((squares - ((sum * sum) / (Rational count))) / (Rational denominator))

summary-variance-source is fn (summary : (Nat, Rational, Rational), sample? : Boolean) -> Optional Rational
  count is summary-count summary
  denominator is variance-denominator (count, sample?)
  denominator <= 0
    true then None Rational
    false then variance-present (summary, count, denominator)

### Return exact population variance, or absence for an empty List.
pub population-variance is fn (values : List Int) -> Optional Rational
  summary-variance-source (summarize-int values, false)
### Return exact population variance, or absence for an empty List.
pub population-variance is fn (values : List Rational) -> Optional Rational
  summary-variance-source (summarize-rational values, false)

### Return exact sample variance, or absence when fewer than two entries exist.
pub sample-variance is fn (values : List Int) -> Optional Rational
  summary-variance-source (summarize-int values, true)
### Return exact sample variance, or absence when fewer than two entries exist.
pub sample-variance is fn (values : List Rational) -> Optional Rational
  summary-variance-source (summarize-rational values, true)

Probability is Rational constraint { probability } (probability >= (Rational 0)) and (probability <= (Rational 1))

position-index is fn ((index : Nat, accepted : Nat)) -> Nat
  index
position-accepted is fn ((index : Nat, accepted : Nat)) -> Nat
  accepted

position-step is fn ((state : (Nat, Nat), ignored : (Value : Type), position : Rational)) -> (Nat, Nat)
  index is position-index state
  accepted is position-accepted state
  (index + 1, increment-when (accepted, (Rational index) <= position))

floor-position is fn (values : List (Value : Type), position : Rational) -> Nat
  zero : Nat is Nat 0
  final is values fold (zero, zero) { state, value } position-step (state, value, position)
  Nat ((position-accepted final) - 1)

interpolate-upper is fn ((lower : Rational, upper : Optional Rational, fraction : Rational)) -> Optional Rational
  upper
    Some value then present-rational (lower + (fraction * (value - lower)))
    None then present-rational lower

interpolate-lower is fn ((lower : Optional Rational, upper : Optional Rational, fraction : Rational)) -> Optional Rational
  lower
    Some value then interpolate-upper (value, upper, fraction)
    None then None Rational

quantile-rational-nonempty is fn ((sorted : List Rational, probability : Rational, length : Nat)) -> Optional Rational
  position is probability * (Rational (length - 1))
  lower is floor-position (sorted, position)
  fraction is position - (Rational lower)
  lower-value is first (sorted select-index (lower ..= lower))
  upper-value is first (sorted select-index ((lower + 1) ..= (lower + 1)))
  interpolate-lower (lower-value, upper-value, fraction)

quantile-rational-source is fn (values : List Rational, probability : Rational) -> Optional Rational
  checked : Probability is Probability probability
  sorted is sort-source values
  length is entry-count sorted
  length = 0
    true then None Rational
    false then quantile-rational-nonempty (sorted, checked, length)

append-rational is fn (values : List Rational, value : Int) -> List Rational
  values append (Rational value)

int-list-rationals is fn (values : List Int) -> List Rational
  empty-rationals : List Rational is Empty
  values fold empty-rationals { converted, value } append-rational (converted, value)

### Return the linearly interpolated exact quantile for an Int List and a probability in 0 ..= 1.
pub quantile is fn (values : List Int, probability : Rational) -> Optional Rational
  quantile-rational-source (int-list-rationals values, probability)
### Return the linearly interpolated exact quantile for a Rational List and a probability in 0 ..= 1.
pub quantile is fn (values : List Rational, probability : Rational) -> Optional Rational
  quantile-rational-source (values, probability)

covariance-count is fn ((count : Nat, left-sum : Rational, right-sum : Rational, products : Rational)) -> Nat
  count
covariance-left is fn ((count : Nat, left-sum : Rational, right-sum : Rational, products : Rational)) -> Rational
  left-sum
covariance-right is fn ((count : Nat, left-sum : Rational, right-sum : Rational, products : Rational)) -> Rational
  right-sum
covariance-products is fn ((count : Nat, left-sum : Rational, right-sum : Rational, products : Rational)) -> Rational
  products

covariance-step is fn ((state : (Nat, Rational, Rational, Rational), left : Rational, right : Rational)) -> (Nat, Rational, Rational, Rational)
  ((covariance-count state) + 1,
   (covariance-left state) + left,
   (covariance-right state) + right,
   (covariance-products state) + (left * right))

covariance-present is fn (state : (Nat, Rational, Rational, Rational), count : Nat) -> Optional Rational
  left-sum is covariance-left state
  right-sum is covariance-right state
  products is covariance-products state
  present-rational ((products / (Rational count)) - ((left-sum / (Rational count)) * (right-sum / (Rational count))))

covariance-source is fn (pairs : List (Rational, Rational)) -> Optional Rational
  final is pairs fold (0, Rational 0, Rational 0, Rational 0) { state, (left, right) } covariance-step (state, left, right)
  count is covariance-count final
  count <= 0
    true then None Rational
    false then covariance-present (final, count)

EqualLengths is Boolean constraint { equal? } equal? = true

### Return exact population covariance for equally sized paired Lists.
pub covariance is fn (left : List Int, right : List Int) -> Optional Rational
  checked : EqualLengths is EqualLengths ((entry-count left) = (entry-count right))
  _ is checked
  left-values : List Rational is int-list-rationals left
  right-values : List Rational is int-list-rationals right
  covariance-source (left-values zip-shortest right-values)
### Return exact population covariance for equally sized paired Lists.
pub covariance is fn (left : List Rational, right : List Rational) -> Optional Rational
  checked : EqualLengths is EqualLengths ((entry-count left) = (entry-count right))
  _ is checked
  covariance-source (left zip-shortest right)

### Produce an exact mergeable summary: count, sum, and sum of squares.
pub summarize is fn (values : List Int) -> (Nat, Rational, Rational)
  summarize-int values
### Produce an exact mergeable summary: count, sum, and sum of squares.
pub summarize is fn (values : List Rational) -> (Nat, Rational, Rational)
  summarize-rational values

### Add one exact observation to a summary without retaining the sample.
pub summary-add is fn (
  summary : (Nat, Rational, Rational),
  value : Rational
) -> (Nat, Rational, Rational)
  summary-add-source (summary, value)

### Merge summaries of disjoint sample portions exactly.
pub summary-merge is fn (
  left : (Nat, Rational, Rational),
  right : (Nat, Rational, Rational)
) -> (Nat, Rational, Rational)
  ((summary-count left) + (summary-count right),
   (summary-sum left) + (summary-sum right),
   (summary-squares left) + (summary-squares right))

### Return the exact mean represented by a summary.
pub summary-mean is fn (summary : (Nat, Rational, Rational)) -> Optional Rational
  count is summary-count summary
  count <= 0
    true then None Rational
    false then present-rational ((summary-sum summary) / (Rational count))

### Return the exact population variance represented by a summary.
pub summary-population-variance is fn (summary : (Nat, Rational, Rational)) -> Optional Rational
  summary-variance-source (summary, false)
