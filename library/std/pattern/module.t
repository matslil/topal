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

### Replace every nonoverlapping exact String pattern from left to right.
pub replace-all is fn (
  text : String,
  (pattern : String, replacement : String)
) -> String
  string-replace-all (text, pattern, replacement)

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

### Split text at every nonoverlapping exact pattern occurrence.
pub split is fn (text : String, pattern : String) -> List String
  string-split-exact (text, pattern)

### Match a complete String using `*` and `?` Character wildcards.
pub glob? is fn (text : String, pattern : String) -> Boolean
  string-glob-matches (text, pattern)

### Test whether a Unicode regular expression occurs in text.
pub regex-contains? is fn (text : String, pattern : String) -> Boolean
  string-regex-contains (text, pattern)

### Test whether any exact String pattern occurs.
contains-any-step is fn ((found : Boolean, text : String, pattern : String)) -> Boolean
  found or (contains? (text, pattern))

pub contains-any? is fn (text : String, patterns : List String) -> Boolean
  patterns fold false { found, pattern } contains-any-step (found, text, pattern)
