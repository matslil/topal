#!/usr/bin/env topal
use language (
  version is v0.1
)

# Demonstrates ordered List algorithms as ordinary composition over the shared
# traversal vocabulary. These operations preserve order and are finite because
# their concrete input is List.
pub map-int is fn (values : List Int, transform : Function) -> List Int
  values map transform

pub filter-int is fn (values : List Int, predicate : Function) -> List Int
  values select predicate

pub take is fn (values : List Int, count : Nat) -> List Int
  values take count

pub drop is fn (values : List Int, count : Nat) -> List Int
  values drop count
