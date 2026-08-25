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

### Return at most the first `count` List entries.
pub take is fn (values : List (Value : Type), count : Nat) -> List Value
  bound is clamp-count (count, entry-count values)
  values select-index (0 .. bound)

### Return the List entries after at most `count` leading entries.
pub drop is fn (values : List (Value : Type), count : Nat) -> List Value
  length is entry-count values
  bound is clamp-count (count, length)
  values select-index (bound .. length)

### Split a List at a clamped boundary.
pub split-at is fn (values : List (Value : Type), count : Nat) -> (List Value, List Value)
  length is entry-count values
  bound is clamp-count (count, length)
  (values select-index (0 .. bound), values select-index (bound .. length))

### Return at most the first `count` Characters of a String.
pub take is fn (value : String, count : Nat) -> String
  bound is clamp-count (count, entry-count value)
  value select-index (0 .. bound)

### Return the String after at most `count` leading Characters.
pub drop is fn (value : String, count : Nat) -> String
  length is entry-count value
  bound is clamp-count (count, length)
  value select-index (bound .. length)

### Split a String at a clamped Character boundary.
pub split-at is fn (value : String, count : Nat) -> (String, String)
  length is entry-count value
  bound is clamp-count (count, length)
  (value select-index (0 .. bound), value select-index (bound .. length))

retain-step is fn (
  collected : List (Value : Type),
  (accepted : Boolean, candidate : Value)
) -> List Value
  accepted
    true then collected append candidate
    false then collected

### Retain List entries accepted by a predicate, preserving order.
pub retain is fn (
  values : List (Value : Type),
  predicate : fn (Value) -> Boolean
) -> List Value
  values fold (Empty Value) { collected, candidate } retain-step (collected, (predicate candidate, candidate))

### Retain List entries rejected by a predicate, preserving order.
pub reject is fn (
  values : List (Value : Type),
  predicate : fn (Value) -> Boolean
) -> List Value
  values fold (Empty Value) { collected, candidate } retain-step (collected, (not (predicate candidate), candidate))

unique-step is fn (selected : List (Value : Equality), candidate : Value) -> List Value
  selected contains-entry candidate
    true then selected
    false then selected append candidate

### Remove repeated equal entries, retaining each first occurrence.
pub unique is fn (values : List (Value : Equality)) -> List Value
  values fold (Empty Value) { selected, candidate } unique-step (selected, candidate)

enumerate-step is fn (candidate : (Value : Type), indexed : List (Nat, Value)) -> List (Nat, Value)
  indexed append (Nat (entry-count indexed), candidate)

enumerate-source is fn (values : List (Value : Type)) -> List (Nat, Value)
  indexed : List (Nat, Value) is Empty
  values fold indexed { collected, candidate } enumerate-step (candidate, collected)

present-index is fn (index : Nat) -> Optional Nat
  present : Optional Nat is Some index
  present

index-when is fn (accepted : Boolean, index : Nat) -> Optional Nat
  accepted
    true then present-index index
    false then None Nat

index-step is fn ((found : Optional Nat, index : Nat, candidate : (Value : Equality), sought : Value)) -> Optional Nat
  found
    Some present then found
    None then index-when (candidate = sought, index)

### Return the first index containing an equal value, or absence.
pub index-of is fn (values : List (Value : Equality), sought : Value) -> Optional Nat
  (enumerate-source values) fold (None Nat) { found, (index, candidate) } index-step (found, index, candidate, sought)

last-index-step is fn ((found : Optional Nat, index : Nat, candidate : (Value : Equality), sought : Value)) -> Optional Nat
  candidate = sought
    true then present-index index
    false then found

### Return the last index containing an equal value, or absence.
pub last-index-of is fn (values : List (Value : Equality), sought : Value) -> Optional Nat
  (enumerate-source values) fold (None Nat) { found, (index, candidate) } last-index-step (found, index, candidate, sought)

### Rotate entries left, wrapping by the List entry count.
rotate-left-nonempty is fn (values : List (Value : Type), count : Nat) -> List Value
  length is entry-count values
  boundary is count % length
  (values select-index (boundary .. length)) concat (values select-index (0 .. boundary))

pub rotate-left is fn (values : List (Value : Type), count : Nat) -> List Value
  length is entry-count values
  length = 0
    true then values
    false then rotate-left-nonempty (values, count)

### Rotate entries right, wrapping by the List entry count.
rotate-right-nonempty is fn (values : List (Value : Type), count : Nat) -> List Value
  length is entry-count values
  rotate-left (values, length - (count % length))

pub rotate-right is fn (values : List (Value : Type), count : Nat) -> List Value
  length is entry-count values
  length = 0
    true then values
    false then rotate-right-nonempty (values, count)

### Divide a List into nonempty consecutive Lists of at most `size` entries.
chunks-implementation is fn (values : List (Value : Type), size : Nat) -> List List Value
  values list-chunks size

pub chunks is fn (values : List (Value : Type), size : Nat) -> List List Value
  chunks-implementation (values, size)

### Return every consecutive List window with exactly `size` entries.
windows-implementation is fn (values : List (Value : Type), size : Nat) -> List List Value
  values list-windows size

pub windows is fn (values : List (Value : Type), size : Nat) -> List List Value
  windows-implementation (values, size)

### Pair each entry with its zero-based index.
pub enumerate is fn (values : List (Value : Type)) -> List (Nat, Value)
  enumerate-source values

### Group adjacent equal entries into nonempty runs.
group-runs-implementation is fn (values : List (Value : Equality)) -> List List Value
  values list-group-runs

pub group-runs is fn (values : List (Value : Equality)) -> List List Value
  group-runs-implementation values

zip-step is fn ((left : (Left : Type), right : List (Right : Type), collected : List (Left, Right), index : Nat)) -> List (Left, Right)
  first (right select-index (index ..= index))
    Some present then collected append (left, present)
    None then collected

### Pair entries until either List is exhausted.
pub zip is fn (
  left : List (Left : Type),
  right : List (Right : Type)
) -> List (Left, Right)
  collected : List (Left, Right) is Empty
  (enumerate-source left) fold collected { pairs, (index, candidate) } zip-step (candidate, right, pairs, index)

range-start is fn (range : Range Int) -> Int
  range-lower-inclusive? range
    true then range-lower range
    false then (range-lower range) + 1

range-finish is fn (range : Range Int) -> Int
  range-upper-inclusive? range
    true then range-upper range
    false then (range-upper range) - 1

### Materialize a finite Int range in ascending order.
pub values is fn (range : Range Int) -> List Int
  start is range-start range
  finish is range-finish range
  start > finish
    true then Empty Int
    false then collect (start iterate ({ value } value + 1) take-while ({ value } value <= finish))

### Transpose homogeneous rows through the shortest row boundary.
pub transpose is fn (rows : List (List (Value : Type))) -> List (List Value)
  list-transpose-shortest rows
