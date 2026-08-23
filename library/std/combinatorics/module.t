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
