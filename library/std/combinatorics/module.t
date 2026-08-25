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

positions is fn (length : Nat) -> List Int
  collect (0 iterate ({ value } value + 1) take-while ({ value } value <= length))

insert-everywhere is fn (candidate : (Value : Type), values : List Value) -> List (List Value)
  (positions (entry-count values)) map { index } values insert-at index candidate

permutation-step is fn (candidate : (Value : Type), permutations : List (List Value)) -> List (List Value)
  collected is permutations select-index (0 .. 0)
  permutations fold collected { expanded, permutation } expanded concat (insert-everywhere (candidate, permutation))

### Return every positional permutation in deterministic source order.
pub permutations is fn (values : List (Value : Type)) -> List (List Value)
  empty-permutation : List Value is Empty
  values fold (one empty-permutation) { permutations, candidate } permutation-step (candidate, permutations)

extend-subsets is fn (candidate : (Value : Type), subsets : List (List Value)) -> List (List Value)
  subsets concat (subsets map { subset } subset append candidate)

subsets-source is fn (values : List (Value : Type)) -> List (List Value)
  empty-subset : List Value is Empty
  values fold (one empty-subset) { subsets, candidate } extend-subsets (candidate, subsets)

### Return every order-preserving positional selection of `count` entries.
pub combinations is fn (values : List (Value : Type), count : Nat) -> List (List Value)
  (subsets-source values) select { subset } (entry-count subset) = count

### Return every positional subset, beginning with the empty subset.
pub subsets is fn (values : List (Value : Type)) -> List (List Value)
  subsets-source values

pair-with is fn (left : (Left : Type), right : List (Right : Type)) -> List (Left, Right)
  collected : List (Left, Right) is Empty
  right fold collected { pairs, candidate } pairs append (left, candidate)

### Return the left-major Cartesian product of two finite Lists.
pub cartesian-product is fn (
  left : List (Left : Type),
  right : List (Right : Type)
) -> List (Left, Right)
  collected : List (Left, Right) is Empty
  left fold collected { pairs, candidate } pairs concat (pair-with (candidate, right))
