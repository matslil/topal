#!/usr/bin/env topal
use language (
  version is v0.1
)

### Revision of the exact combinatorial-count namespace.
pub revision is 1

### Return the number of permutations of `count` distinct entries.
pub factorial is fn (count : Nat) -> Nat
  count
    <= 1 then 1
    otherwise count * (factorial (count - 1))

### Return the number of subsets of a finite List.
pub subset-count is fn (values : List (Value : Type)) -> Nat
  2 ^ (entry-count values)

permutations-implementation is fn (values : List (Value : Type)) -> List (List Value)
  list-permutations values
permutations-implementation-callable is permutations-implementation

### Return every positional permutation in deterministic source order.
pub permutations is fn (values : List (Value : Type)) -> List (List Value)
  permutations-implementation-callable values

combinations-implementation is fn (values : List (Value : Type), count : Nat) -> List (List Value)
  list-combinations (values, count)
combinations-implementation-callable is combinations-implementation

### Return every order-preserving positional selection of `count` entries.
pub combinations is fn (values : List (Value : Type), count : Nat) -> List (List Value)
  combinations-implementation-callable (values, count)

subsets-implementation is fn (values : List (Value : Type)) -> List (List Value)
  list-subsets values
subsets-implementation-callable is subsets-implementation

### Return every positional subset, beginning with the empty subset.
pub subsets is fn (values : List (Value : Type)) -> List (List Value)
  subsets-implementation-callable values

cartesian-implementation is fn (
  left : List (Left : Type),
  right : List (Right : Type)
) -> List (Left, Right)
  list-cartesian-product (left, right)
cartesian-implementation-callable is cartesian-implementation

### Return the left-major Cartesian product of two finite Lists.
pub cartesian-product is fn (
  left : List (Left : Type),
  right : List (Right : Type)
) -> List (Left, Right)
  cartesian-implementation-callable (left, right)
