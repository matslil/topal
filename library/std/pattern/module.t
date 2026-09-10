#!/usr/bin/env topal
use language (
  version is v0.1
)

### Revision of the exact pattern-algorithm namespace.
pub revision is 1

### Test whether text begins with an exact String pattern.
pub starts-with? is fn (text : String, pattern : String) -> Boolean
  text-characters is collect (characters text)
  pattern-characters is collect (characters pattern)
  length is entry-count pattern-characters
  (text-characters select-index (0 .. length)) = pattern-characters

### Test whether text ends with an exact String pattern.
pub ends-with? is fn (text : String, pattern : String) -> Boolean
  text-characters is collect (characters text)
  pattern-characters is collect (characters pattern)
  text-length is entry-count text-characters
  pattern-length is entry-count pattern-characters
  pattern-length > text-length
    true then false
    false then (text-characters select-index ((text-length - pattern-length) .. text-length)) = pattern-characters

### Test whether text contains an exact consecutive String pattern.
pub contains? is fn (text : String, pattern : String) -> Boolean
  (collect (characters text)) contains-sequence (collect (characters pattern))

### Test whether a List contains an exact consecutive List pattern.
pub contains? is fn (
  values : List (Value : Equality),
  pattern : List Value
) -> Boolean
  values contains-sequence pattern

### Test whether a List contains a possibly gapped ordered pattern.
pub subsequence? is fn (
  values : List (Value : Equality),
  pattern : List Value
) -> Boolean
  values contains-subsequence pattern

NonemptyPattern is String constraint { pattern } (entry-count (collect (characters pattern))) > 0

find-step is fn ((text : List Character, pattern : List Character, indexes : List Nat, index : Nat)) -> List Nat
  length is entry-count pattern
  (text select-index (index .. (index + length))) = pattern
    true then indexes append index
    false then indexes

find-source is fn (text : String, pattern : String) -> List Nat
  checked : NonemptyPattern is NonemptyPattern pattern
  _ is checked
  text-characters is collect (characters text)
  pattern-characters is collect (characters pattern)
  pattern-length is entry-count pattern-characters
  text-length is entry-count text-characters
  candidates is collect (0 iterate ({ index } index + 1) take-while ({ index } index + pattern-length <= text-length))
  indexes : List Nat is Empty
  candidates fold indexes { found, index } find-step (text-characters, pattern-characters, found, index)

### Count overlapping exact String occurrences at Character boundaries.
pub count is fn (text : String, pattern : String) -> Nat
  entry-count (find-source (text, pattern))

### Return every overlapping exact-match Character index.
pub find-all is fn (text : String, pattern : String) -> List Nat
  find-source (text, pattern)

split-parts is fn ((parts : List String, start : Nat)) -> List String
  parts

split-start is fn ((parts : List String, start : Nat)) -> Nat
  start

split-step is fn ((text : String, pattern-length : Nat, state : (List String, Nat), index : Nat)) -> (List String, Nat)
  parts is split-parts state
  start is split-start state
  index < start
    true then state
    false then (parts append (text select-index (start .. index)), index + pattern-length)

### Split text at every nonoverlapping exact pattern occurrence.
pub split is fn (text : String, pattern : String) -> List String
  indexes is find-source (text, pattern)
  pattern-length is entry-count (collect (characters pattern))
  final is indexes fold ((Empty String), 0) { state, index } split-step (text, pattern-length, state, index)
  (split-parts final) append (text select-index ((split-start final) .. (entry-count text)))

joined-text is fn ((joined : String, first? : Boolean)) -> String
  joined

joined-first? is fn ((joined : String, first? : Boolean)) -> Boolean
  first?

join-step is fn ((separator : String, state : (String, Boolean), part : String)) -> (String, Boolean)
  joined is joined-text state
  first? is joined-first? state
  first?
    true then (part, false)
    false then (joined concat separator concat part, false)

join-text is fn (parts : List String, separator : String) -> String
  final is parts fold ("", true) { state, part } join-step (separator, state, part)
  joined-text final

### Replace every nonoverlapping exact String pattern from left to right.
pub replace-all is fn (
  text : String,
  (pattern : String, replacement : String)
) -> String
  join-text (split (text, pattern), replacement)

optional-boolean is fn (value : Optional Boolean) -> Boolean
  value
    Some present then present
    None then false

boolean-at is fn (values : List Boolean, index : Nat) -> Boolean
  optional-boolean (first (values select-index (index ..= index)))

last-boolean is fn (values : List Boolean) -> Boolean
  length is entry-count values
  length = 0
    true then false
    false then boolean-at (values, length - 1)

initial-row-step is fn (row : List Boolean, ignored : Character) -> List Boolean
  row append false

initial-glob-row is fn (text : List Character) -> List Boolean
  row : List Boolean is one true
  text fold row { values, ignored } initial-row-step (values, ignored)

glob-row-values is fn ((values : List Boolean, previous : List Boolean, index : Nat)) -> List Boolean
  values
glob-row-previous is fn ((values : List Boolean, previous : List Boolean, index : Nat)) -> List Boolean
  previous
glob-row-index is fn ((values : List Boolean, previous : List Boolean, index : Nat)) -> Nat
  index

glob-star-step is fn (state : (List Boolean, List Boolean, Nat), ignored : Character) -> (List Boolean, List Boolean, Nat)
  index is glob-row-index state
  matched is (boolean-at (glob-row-previous state, index + 1)) or (last-boolean (glob-row-values state))
  ((glob-row-values state) append matched, glob-row-previous state, index + 1)

glob-character-matches? is fn (pattern : Character, candidate : Character) -> Boolean
  (pattern = "?") or (pattern = candidate)

glob-character-step is fn ((pattern : Character, state : (List Boolean, List Boolean, Nat), candidate : Character)) -> (List Boolean, List Boolean, Nat)
  index is glob-row-index state
  matched is (glob-character-matches? (pattern, candidate)) and (boolean-at (glob-row-previous state, index))
  ((glob-row-values state) append matched, glob-row-previous state, index + 1)

glob-star-row is fn (text : List Character, previous : List Boolean) -> List Boolean
  initial : List Boolean is one (boolean-at (previous, 0))
  final is text fold (initial, previous, Nat 0) { state, candidate } glob-star-step (state, candidate)
  glob-row-values final

glob-character-row is fn ((text : List Character, previous : List Boolean, pattern : Character)) -> List Boolean
  initial : List Boolean is one false
  final is text fold (initial, previous, Nat 0) { state, candidate } glob-character-step (pattern, state, candidate)
  glob-row-values final

glob-pattern-step is fn ((text : List Character, previous : List Boolean, pattern : Character)) -> List Boolean
  pattern = "*"
    true then glob-star-row (text, previous)
    false then glob-character-row (text, previous, pattern)

glob-source is fn (text : String, pattern : String) -> Boolean
  text-characters is collect (characters text)
  pattern-characters is collect (characters pattern)
  final is pattern-characters fold (initial-glob-row text-characters) { previous, candidate } glob-pattern-step (text-characters, previous, candidate)
  last-boolean final

### Match a complete String using `*` and `?` Character wildcards.
pub glob? is fn (text : String, pattern : String) -> Boolean
  glob-source (text, pattern)

### Test whether any exact String pattern occurs.
contains-any-step is fn ((found : Boolean, text : String, pattern : String)) -> Boolean
  found or (contains? (text, pattern))

pub contains-any? is fn (text : String, patterns : List String) -> Boolean
  patterns fold false { found, pattern } contains-any-step (found, text, pattern)
