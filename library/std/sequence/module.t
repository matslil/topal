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

### Return the first index containing an equal value, or absence.
index-of-implementation is fn (values : List (Value : Equality), sought : Value) -> Optional Nat
  values list-index-of sought

pub index-of is fn (values : List (Value : Equality), sought : Value) -> Optional Nat
  index-of-implementation (values, sought)

### Return the last index containing an equal value, or absence.
last-index-of-implementation is fn (values : List (Value : Equality), sought : Value) -> Optional Nat
  values list-last-index-of sought

pub last-index-of is fn (values : List (Value : Equality), sought : Value) -> Optional Nat
  last-index-of-implementation (values, sought)

### Rotate entries left, wrapping by the List entry count.
rotate-left-implementation is fn (values : List (Value : Type), count : Nat) -> List Value
  values list-rotate-left count

pub rotate-left is fn (values : List (Value : Type), count : Nat) -> List Value
  rotate-left-implementation (values, count)

### Rotate entries right, wrapping by the List entry count.
rotate-right-implementation is fn (values : List (Value : Type), count : Nat) -> List Value
  values list-rotate-right count

pub rotate-right is fn (values : List (Value : Type), count : Nat) -> List Value
  rotate-right-implementation (values, count)

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
enumerate-implementation is fn (values : List (Value : Type)) -> List (Nat, Value)
  values list-enumerate

pub enumerate is fn (values : List (Value : Type)) -> List (Nat, Value)
  enumerate-implementation values

### Pair Characters with indexes without erasing their exact classifier.
pub enumerate is fn (values : List Character) -> List (Nat, Character)
  values list-enumerate

### Group adjacent equal entries into nonempty runs.
group-runs-implementation is fn (values : List (Value : Equality)) -> List List Value
  values list-group-runs

pub group-runs is fn (values : List (Value : Equality)) -> List List Value
  group-runs-implementation values

### Pair entries until either List is exhausted.
zip-shortest-implementation is fn (
  left : List (Left : Type),
  right : List (Right : Type)
) -> List (Left, Right)
  left list-zip-shortest right

pub zip is fn (
  left : List (Left : Type),
  right : List (Right : Type)
) -> List (Left, Right)
  zip-shortest-implementation (left, right)

### Materialize a finite Int range in ascending order.
pub values is fn (range : Range Int) -> List Int
  range-integers range

### Transpose homogeneous rows through the shortest row boundary.
pub transpose is fn (rows : List (List (Value : Type))) -> List (List Value)
  list-transpose-shortest rows
