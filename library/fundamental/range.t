#!/usr/bin/env topal
use language (
  version is v0.1
)

# Ranges remain convex predicates. These functions observe and combine bounds;
# they do not imply that members can be enumerated.
pub lower-bound is fn (interval : Range (Value : TotalOrder)) -> Value
  range-lower interval

pub upper-bound is fn (interval : Range (Value : TotalOrder)) -> Value
  range-upper interval

pub bounds is fn (interval : Range (Value : TotalOrder)) -> (Value, Value)
  (range-lower interval, range-upper interval)

pub intersection is fn (
  left : Range (Value : TotalOrder),
  right : Range Value
) -> Range Value
  left and right

pub overlaps? is fn (
  left : Range (Value : TotalOrder),
  right : Range Value
) -> Boolean
  not (empty? (left and right))

range-min is fn (left : (Value : TotalOrder), right : Value) -> Value
  left
    <= right then left
    otherwise right
range-min-callable is range-min

range-max is fn (left : (Value : TotalOrder), right : Value) -> Value
  left
    >= right then left
    otherwise right
range-max-callable is range-max

pub hull is fn (
  left : Range (Value : TotalOrder),
  right : Range Value
) -> Range Value
  lower is range-min-callable (range-lower left, range-lower right)
  upper is range-max-callable (range-upper left, range-upper right)
  lower .. upper

pub adjacent? is fn (left : Range Int, right : Range Int) -> Boolean
  ((range-upper left) + 1 = (range-lower right)) or ((range-upper right) + 1 = (range-lower left))
