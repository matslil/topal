#!/usr/bin/env topal
use language (
  version is v0.1
)

### Revision of the finite sequence-algorithm namespace.
pub revision is 1

clamp-count is fn (count : Nat, length : Nat) -> Nat
  count
    <= length then count
    otherwise length
clamp-count-callable is clamp-count

### Return at most the first `count` List entries.
pub take is fn (values : List (Value : Type), count : Nat) -> List Value
  bound is clamp-count-callable (count, entry-count values)
  values select-index (0 .. bound)

### Return the List entries after at most `count` leading entries.
pub drop is fn (values : List (Value : Type), count : Nat) -> List Value
  length is entry-count values
  bound is clamp-count-callable (count, length)
  values select-index (bound .. length)

### Split a List at a clamped boundary.
pub split-at is fn (values : List (Value : Type), count : Nat) -> (List Value, List Value)
  length is entry-count values
  bound is clamp-count-callable (count, length)
  (values select-index (0 .. bound), values select-index (bound .. length))

### Return at most the first `count` Characters of a String.
pub take is fn (value : String, count : Nat) -> String
  bound is clamp-count-callable (count, entry-count value)
  value select-index (0 .. bound)

### Return the String after at most `count` leading Characters.
pub drop is fn (value : String, count : Nat) -> String
  length is entry-count value
  bound is clamp-count-callable (count, length)
  value select-index (bound .. length)

### Split a String at a clamped Character boundary.
pub split-at is fn (value : String, count : Nat) -> (String, String)
  length is entry-count value
  bound is clamp-count-callable (count, length)
  (value select-index (0 .. bound), value select-index (bound .. length))

retain-step is fn (
  collected : List (Value : Type),
  (accepted : Boolean, candidate : Value)
) -> List Value
  accepted
    true then collected append candidate
    false then collected
retain-step-callable is retain-step

### Retain List entries accepted by a predicate, preserving order.
pub retain is fn (
  values : List (Value : Type),
  predicate : fn (Value) -> Boolean
) -> List Value
  values fold (Empty Value) { collected, candidate } retain-step-callable (collected, (predicate candidate, candidate))

### Retain List entries rejected by a predicate, preserving order.
pub reject is fn (
  values : List (Value : Type),
  predicate : fn (Value) -> Boolean
) -> List Value
  values fold (Empty Value) { collected, candidate } retain-step-callable (collected, (not (predicate candidate), candidate))

unique-step is fn (selected : List (Value : Equality), candidate : Value) -> List Value
  selected contains-entry candidate
    true then selected
    false then selected append candidate
unique-step-callable is unique-step

### Remove repeated equal entries, retaining each first occurrence.
pub unique is fn (values : List (Value : Equality)) -> List Value
  values fold (Empty Value) { selected, candidate } unique-step-callable (selected, candidate)
