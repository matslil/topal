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

### Return the first index containing an equal value, or absence.
index-of-implementation is fn (values : List (Value : Equality), sought : Value) -> Optional Nat
  values list-index-of sought
index-of-implementation-callable is index-of-implementation

pub index-of is fn (values : List (Value : Equality), sought : Value) -> Optional Nat
  index-of-implementation-callable (values, sought)

### Return the last index containing an equal value, or absence.
last-index-of-implementation is fn (values : List (Value : Equality), sought : Value) -> Optional Nat
  values list-last-index-of sought
last-index-of-implementation-callable is last-index-of-implementation

pub last-index-of is fn (values : List (Value : Equality), sought : Value) -> Optional Nat
  last-index-of-implementation-callable (values, sought)

### Rotate entries left, wrapping by the List entry count.
rotate-left-implementation is fn (values : List (Value : Type), count : Nat) -> List Value
  values list-rotate-left count
rotate-left-implementation-callable is rotate-left-implementation

pub rotate-left is fn (values : List (Value : Type), count : Nat) -> List Value
  rotate-left-implementation-callable (values, count)

### Rotate entries right, wrapping by the List entry count.
rotate-right-implementation is fn (values : List (Value : Type), count : Nat) -> List Value
  values list-rotate-right count
rotate-right-implementation-callable is rotate-right-implementation

pub rotate-right is fn (values : List (Value : Type), count : Nat) -> List Value
  rotate-right-implementation-callable (values, count)

### Divide a List into nonempty consecutive Lists of at most `size` entries.
chunks-implementation is fn (values : List (Value : Type), size : Nat) -> List List Value
  values list-chunks size
chunks-implementation-callable is chunks-implementation

pub chunks is fn (values : List (Value : Type), size : Nat) -> List List Value
  chunks-implementation-callable (values, size)

### Return every consecutive List window with exactly `size` entries.
windows-implementation is fn (values : List (Value : Type), size : Nat) -> List List Value
  values list-windows size
windows-implementation-callable is windows-implementation

pub windows is fn (values : List (Value : Type), size : Nat) -> List List Value
  windows-implementation-callable (values, size)

### Pair each entry with its zero-based index.
enumerate-implementation is fn (values : List (Value : Type)) -> List (Nat, Value)
  values list-enumerate
enumerate-implementation-callable is enumerate-implementation

pub enumerate is fn (values : List (Value : Type)) -> List (Nat, Value)
  enumerate-implementation-callable values

### Group adjacent equal entries into nonempty runs.
group-runs-implementation is fn (values : List (Value : Equality)) -> List List Value
  values list-group-runs
group-runs-implementation-callable is group-runs-implementation

pub group-runs is fn (values : List (Value : Equality)) -> List List Value
  group-runs-implementation-callable values

### Pair entries until either List is exhausted.
zip-shortest-implementation is fn (
  left : List (Left : Type),
  right : List (Right : Type)
) -> List (Left, Right)
  left list-zip-shortest right
zip-shortest-implementation-callable is zip-shortest-implementation

pub zip is fn (
  left : List (Left : Type),
  right : List (Right : Type)
) -> List (Left, Right)
  zip-shortest-implementation-callable (left, right)

### Materialize a finite Int range in ascending order.
pub values is fn (range : Range Int) -> List Int
  range-integers range
