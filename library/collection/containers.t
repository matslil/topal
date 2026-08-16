#!/usr/bin/env topal
use language (
  version is v0.1
)

# Demonstrates explicit construction policies for the fundamental containers.
# Set removes duplicates, Bag retains multiplicity, Array retains order, and
# Map requires a duplicate-key resolution policy.
pub array-count is fn (values : List Int) -> Nat
  entry-count (values collect Array)

pub unique-count is fn (values : List Int) -> Nat
  entry-count (collect-set values)

pub occurrence-count is fn (values : List Int) -> Nat
  entry-count (collect-bag values)

pub map-count is fn (entries : List (String, Int)) -> Nat
  entry-count (collect-map entries resolving keep-last)
