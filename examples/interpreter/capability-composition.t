#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates conjunction and alternatives of atomic capability promises.
Comparable is Equality and Ordering
Searchable is Foldable and Membership
ComparableOrSearchable : Capability is Comparable or Searchable
ComparableOrSearchable
