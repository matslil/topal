#!/usr/bin/env topal
use language (
  version is v0.1
)

### Revision of the exact pattern-algorithm namespace.
pub revision is 1

### Test whether text begins with an exact String pattern.
pub starts-with? is fn (text : String, pattern : String) -> Boolean
  string-starts-with (text, pattern)

### Test whether text ends with an exact String pattern.
pub ends-with? is fn (text : String, pattern : String) -> Boolean
  string-ends-with (text, pattern)

### Test whether text contains an exact consecutive String pattern.
pub contains? is fn (text : String, pattern : String) -> Boolean
  string-contains (text, pattern)

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

### Count overlapping exact String occurrences at Character boundaries.
pub count is fn (text : String, pattern : String) -> Nat
  string-count-exact (text, pattern)

### Return every overlapping exact-match Character index.
pub find-all is fn (text : String, pattern : String) -> List Nat
  string-find-all (text, pattern)

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
pub contains-any? is fn (text : String, patterns : List String) -> Boolean
  string-contains-any (text, patterns)
