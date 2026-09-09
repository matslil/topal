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

regex-token-kind is fn ((kind : Int, payload : String)) -> Int
  kind
regex-token-payload is fn ((kind : Int, payload : String)) -> String
  payload

regex-scan-tokens is fn ((tokens : List (Int, String), mode : Int, buffered : String, valid? : Boolean)) -> List (Int, String)
  tokens
regex-scan-mode is fn ((tokens : List (Int, String), mode : Int, buffered : String, valid? : Boolean)) -> Int
  mode
regex-scan-buffer is fn ((tokens : List (Int, String), mode : Int, buffered : String, valid? : Boolean)) -> String
  buffered
regex-scan-valid? is fn ((tokens : List (Int, String), mode : Int, buffered : String, valid? : Boolean)) -> Boolean
  valid?
regex-scan-append is fn (state : (List (Int, String), Int, String, Boolean), token : (Int, String)) -> (List (Int, String), Int, String, Boolean)
  ((regex-scan-tokens state) append token, 0, "", regex-scan-valid? state)
regex-scan-invalid is fn (state : (List (Int, String), Int, String, Boolean)) -> (List (Int, String), Int, String, Boolean)
  (regex-scan-tokens state, regex-scan-mode state, regex-scan-buffer state, false)

regex-backslash-present is fn (value : Optional Character) -> Character
  value
    Some character then character
    None then "."
regex-backslash-character is fn () -> Character
  regex-backslash-present (first (unicode-scalar-characters "\d"))
regex-backslash? is fn (character : Character) -> Boolean
  left is unicode-scalar-value character
  right is unicode-scalar-value (regex-backslash-character ())
  left = right
regex-quoted-metacharacter? is fn (character : Character) -> Boolean
  meta : List Character is unicode-scalar-characters ".^$|?*+()[]{}"
  (meta contains-entry character) or (regex-backslash? character)

regex-scan-escaped-other is fn (state : (List (Int, String), Int, String, Boolean), character : Character) -> (List (Int, String), Int, String, Boolean)
  regex-quoted-metacharacter? character
    true then regex-scan-append (state, (0, character))
    false then regex-scan-invalid state
regex-scan-escaped is fn (state : (List (Int, String), Int, String, Boolean), character : Character) -> (List (Int, String), Int, String, Boolean)
  value : Int is unicode-scalar-value character
  value
    = 100 then regex-scan-append (state, (2, (regex-backslash-character ()) concat character))
    = 68 then regex-scan-append (state, (2, (regex-backslash-character ()) concat character))
    = 115 then regex-scan-append (state, (2, (regex-backslash-character ()) concat character))
    = 83 then regex-scan-append (state, (2, (regex-backslash-character ()) concat character))
    = 119 then regex-scan-append (state, (2, (regex-backslash-character ()) concat character))
    = 87 then regex-scan-append (state, (2, (regex-backslash-character ()) concat character))
    otherwise regex-scan-escaped-other (state, character)

regex-scan-normal-nonbackslash is fn (state : (List (Int, String), Int, String, Boolean), character : Character) -> (List (Int, String), Int, String, Boolean)
  character
    = "." then regex-scan-append (state, (1, ""))
    = "[" then (regex-scan-tokens state, 2, "", regex-scan-valid? state)
    = "^" then regex-scan-append (state, (3, ""))
    = "$" then regex-scan-append (state, (4, ""))
    = "(" then regex-scan-append (state, (5, ""))
    = ")" then regex-scan-append (state, (6, ""))
    = "|" then regex-scan-append (state, (7, ""))
    = "*" then regex-scan-append (state, (8, ""))
    = "+" then regex-scan-append (state, (9, ""))
    = "?" then regex-scan-append (state, (10, ""))
    = "{" then (regex-scan-tokens state, 4, "", regex-scan-valid? state)
    = "}" then regex-scan-invalid state
    = "]" then regex-scan-invalid state
    otherwise regex-scan-append (state, (0, character))
regex-scan-normal is fn (state : (List (Int, String), Int, String, Boolean), character : Character) -> (List (Int, String), Int, String, Boolean)
  regex-backslash? character
    true then (regex-scan-tokens state, 1, "", regex-scan-valid? state)
    false then regex-scan-normal-nonbackslash (state, character)

regex-scan-class-escaped is fn (state : (List (Int, String), Int, String, Boolean), character : Character) -> (List (Int, String), Int, String, Boolean)
  accepted? is (regex-quoted-metacharacter? character) or (character = "-") or (character = "d") or (character = "D") or (character = "s") or (character = "S") or (character = "w") or (character = "W")
  accepted?
    true then (regex-scan-tokens state, 2, (regex-scan-buffer state) concat (regex-backslash-character ()) concat character, regex-scan-valid? state)
    false then regex-scan-invalid state
regex-scan-class-close is fn (state : (List (Int, String), Int, String, Boolean)) -> (List (Int, String), Int, String, Boolean)
  empty? (regex-scan-buffer state)
    true then regex-scan-invalid state
    false then regex-scan-append (state, (2, regex-scan-buffer state))
regex-scan-class is fn (state : (List (Int, String), Int, String, Boolean), character : Character) -> (List (Int, String), Int, String, Boolean)
  regex-backslash? character
    true then (regex-scan-tokens state, 3, regex-scan-buffer state, regex-scan-valid? state)
    false then character = "]"
      true then regex-scan-class-close state
      false then (regex-scan-tokens state, 2, (regex-scan-buffer state) concat character, regex-scan-valid? state)
regex-scan-repeat-close is fn (state : (List (Int, String), Int, String, Boolean)) -> (List (Int, String), Int, String, Boolean)
  empty? (regex-scan-buffer state)
    true then regex-scan-invalid state
    false then regex-scan-append (state, (11, regex-scan-buffer state))
regex-scan-repeat-nondigit is fn (state : (List (Int, String), Int, String, Boolean), character : Character) -> (List (Int, String), Int, String, Boolean)
  character = ","
    true then (regex-scan-tokens state, 4, (regex-scan-buffer state) concat character, regex-scan-valid? state)
    false then regex-scan-invalid state
regex-scan-repeat-other is fn (state : (List (Int, String), Int, String, Boolean), character : Character) -> (List (Int, String), Int, String, Boolean)
  ascii-decimal-digit character
    Some digit then (regex-scan-tokens state, 4, (regex-scan-buffer state) concat character, regex-scan-valid? state)
    None then regex-scan-repeat-nondigit (state, character)
regex-scan-repeat is fn (state : (List (Int, String), Int, String, Boolean), character : Character) -> (List (Int, String), Int, String, Boolean)
  character = "}"
    true then regex-scan-repeat-close state
    false then regex-scan-repeat-other (state, character)

regex-scan-step is fn (state : (List (Int, String), Int, String, Boolean), character : Character) -> (List (Int, String), Int, String, Boolean)
  regex-scan-valid? state
    false then state
    true then regex-scan-mode state
      = 0 then regex-scan-normal (state, character)
      = 1 then regex-scan-escaped (state, character)
      = 2 then regex-scan-class (state, character)
      = 3 then regex-scan-class-escaped (state, character)
      otherwise regex-scan-repeat (state, character)

regex-tokenize is fn (pattern : String) -> (List (Int, String), Boolean)
  empty-tokens : List (Int, String) is Empty
  final is (unicode-scalar-characters pattern) fold (empty-tokens, 0, "", true) { state, character } regex-scan-step (state, character)
  ((regex-scan-tokens final), (regex-scan-valid? final) and ((regex-scan-mode final) = 0))

regex-repeat-left is fn ((left : Nat, right : Nat, comma? : Boolean, left? : Boolean, right? : Boolean, valid? : Boolean)) -> Nat
  left
regex-repeat-right is fn ((left : Nat, right : Nat, comma? : Boolean, left? : Boolean, right? : Boolean, valid? : Boolean)) -> Nat
  right
regex-repeat-comma? is fn ((left : Nat, right : Nat, comma? : Boolean, left? : Boolean, right? : Boolean, valid? : Boolean)) -> Boolean
  comma?
regex-repeat-left? is fn ((left : Nat, right : Nat, comma? : Boolean, left? : Boolean, right? : Boolean, valid? : Boolean)) -> Boolean
  left?
regex-repeat-right? is fn ((left : Nat, right : Nat, comma? : Boolean, left? : Boolean, right? : Boolean, valid? : Boolean)) -> Boolean
  right?
regex-repeat-valid? is fn ((left : Nat, right : Nat, comma? : Boolean, left? : Boolean, right? : Boolean, valid? : Boolean)) -> Boolean
  valid?
regex-repeat-digit is fn (state : (Nat, Nat, Boolean, Boolean, Boolean, Boolean), digit : Nat) -> (Nat, Nat, Boolean, Boolean, Boolean, Boolean)
  regex-repeat-comma? state
    true then (regex-repeat-left state, (regex-repeat-right state) * 10 + digit, true, regex-repeat-left? state, true, regex-repeat-valid? state)
    false then ((regex-repeat-left state) * 10 + digit, regex-repeat-right state, false, true, regex-repeat-right? state, regex-repeat-valid? state)
regex-repeat-comma is fn (state : (Nat, Nat, Boolean, Boolean, Boolean, Boolean)) -> (Nat, Nat, Boolean, Boolean, Boolean, Boolean)
  regex-repeat-comma? state
    true then (regex-repeat-left state, regex-repeat-right state, true, regex-repeat-left? state, regex-repeat-right? state, false)
    false then (regex-repeat-left state, regex-repeat-right state, true, regex-repeat-left? state, regex-repeat-right? state, regex-repeat-valid? state)
regex-repeat-nondigit is fn (state : (Nat, Nat, Boolean, Boolean, Boolean, Boolean), character : Character) -> (Nat, Nat, Boolean, Boolean, Boolean, Boolean)
  character = ","
    true then regex-repeat-comma state
    false then (regex-repeat-left state, regex-repeat-right state, regex-repeat-comma? state, regex-repeat-left? state, regex-repeat-right? state, false)
regex-repeat-character is fn (state : (Nat, Nat, Boolean, Boolean, Boolean, Boolean), character : Character) -> (Nat, Nat, Boolean, Boolean, Boolean, Boolean)
  ascii-decimal-digit character
    Some digit then regex-repeat-digit (state, digit)
    None then regex-repeat-nondigit (state, character)
regex-repeat-comma-bounds is fn (state : (Nat, Nat, Boolean, Boolean, Boolean, Boolean), valid-left? : Boolean) -> (Nat, Int, Boolean)
  regex-repeat-right? state
    false then (regex-repeat-left state, -1, valid-left?)
    true then (regex-repeat-left state, regex-repeat-right state, valid-left? and ((regex-repeat-right state) >= (regex-repeat-left state)))
regex-repeat-bounds is fn (payload : String) -> (Nat, Int, Boolean)
  final is (unicode-scalar-characters payload) fold (Nat 0, Nat 0, false, false, false, true) { state, character } regex-repeat-character (state, character)
  valid-left? is (regex-repeat-valid? final) and (regex-repeat-left? final)
  regex-repeat-comma? final
    false then (regex-repeat-left final, regex-repeat-left final, valid-left?)
    true then regex-repeat-comma-bounds (final, valid-left?)

regex-postfix-output is fn ((output : List (Int, String), groups : List (Nat, Nat), alternatives : Nat, atoms : Nat, valid? : Boolean, quantified? : Boolean)) -> List (Int, String)
  output
regex-postfix-groups is fn ((output : List (Int, String), groups : List (Nat, Nat), alternatives : Nat, atoms : Nat, valid? : Boolean, quantified? : Boolean)) -> List (Nat, Nat)
  groups
regex-postfix-alternatives is fn ((output : List (Int, String), groups : List (Nat, Nat), alternatives : Nat, atoms : Nat, valid? : Boolean, quantified? : Boolean)) -> Nat
  alternatives
regex-postfix-atoms is fn ((output : List (Int, String), groups : List (Nat, Nat), alternatives : Nat, atoms : Nat, valid? : Boolean, quantified? : Boolean)) -> Nat
  atoms
regex-postfix-valid? is fn ((output : List (Int, String), groups : List (Nat, Nat), alternatives : Nat, atoms : Nat, valid? : Boolean, quantified? : Boolean)) -> Boolean
  valid?
regex-postfix-quantified? is fn ((output : List (Int, String), groups : List (Nat, Nat), alternatives : Nat, atoms : Nat, valid? : Boolean, quantified? : Boolean)) -> Boolean
  quantified?

regex-token-arity is fn (token : (Int, String)) -> Nat
  regex-token-kind token
    = 13 then 2
    = 7 then 2
    = 8 then 1
    = 9 then 1
    = 10 then 1
    otherwise 0
regex-suffix-needed is fn ((needed : Nat, count : Nat)) -> Nat
  needed
regex-suffix-count is fn ((needed : Nat, count : Nat)) -> Nat
  count
regex-suffix-step is fn (state : (Nat, Nat), token : (Int, String)) -> (Nat, Nat)
  (regex-suffix-needed state) = 0
    true then state
    false then ((regex-suffix-needed state) - 1 + (regex-token-arity token), (regex-suffix-count state) + 1)
regex-expression-suffix-count is fn (output : List (Int, String)) -> Nat
  final is (output reverse) fold (Nat 1, Nat 0) { state, token } regex-suffix-step (state, token)
  regex-suffix-count final

regex-copy-output is fn ((output : List (Int, String), present? : Boolean)) -> List (Int, String)
  output
regex-copy-present? is fn ((output : List (Int, String), present? : Boolean)) -> Boolean
  present?
regex-append-copy is fn (expression : List (Int, String), state : (List (Int, String), Boolean)) -> (List (Int, String), Boolean)
  regex-copy-present? state
    true then ((regex-copy-output state) concat expression append (13, ""), true)
    false then ((regex-copy-output state) concat expression, true)
regex-append-optional-copy is fn (expression : List (Int, String), state : (List (Int, String), Boolean)) -> (List (Int, String), Boolean)
  regex-append-copy (expression append (10, ""), state)
regex-counts is fn (count : Nat) -> List Int
  collect (0 iterate ({ value } value + 1) take-while ({ value } value < count))
regex-expand-finite-repeat is fn ((prefix : List (Int, String), expression : List (Int, String), minimum : Nat, maximum : Nat)) -> List (Int, String)
  empty-output : List (Int, String) is Empty
  mandatory is (regex-counts minimum) fold (empty-output, false) { state, ignored } regex-append-copy (expression, state)
  optional-count is maximum - minimum
  complete is (regex-counts optional-count) fold mandatory { state, ignored } regex-append-optional-copy (expression, state)
  regex-copy-present? complete
    true then prefix concat (regex-copy-output complete)
    false then prefix append (12, "")
regex-expand-unbounded-repeat is fn ((prefix : List (Int, String), expression : List (Int, String), minimum : Nat)) -> List (Int, String)
  empty-output : List (Int, String) is Empty
  mandatory is (regex-counts minimum) fold (empty-output, false) { state, ignored } regex-append-copy (expression, state)
  with-tail is regex-append-copy (expression append (8, ""), mandatory)
  prefix concat (regex-copy-output with-tail)
regex-expand-repeat-valid is fn ((output : List (Int, String), minimum : Nat, maximum : Int)) -> List (Int, String)
  count is regex-expression-suffix-count output
  prefix is output select-index (0 .. ((entry-count output) - count))
  expression is output select-index (((entry-count output) - count) .. (entry-count output))
  maximum < 0
    true then regex-expand-unbounded-repeat (prefix, expression, minimum)
    false then regex-expand-finite-repeat (prefix, expression, minimum, Nat maximum)

regex-collapse-one-concat is fn (output : List (Int, String), ignored : Int) -> List (Int, String)
  output append (13, "")
regex-collapse-concats is fn (output : List (Int, String), atoms : Nat) -> List (Int, String)
  atoms <= 1
    true then output
    false then (regex-counts (atoms - 1)) fold output { values, ignored } regex-collapse-one-concat (values, ignored)
regex-collapse-one-alternative is fn (output : List (Int, String), ignored : Int) -> List (Int, String)
  output append (7, "")
regex-collapse-alternatives is fn (output : List (Int, String), alternatives : Nat) -> List (Int, String)
  (regex-counts alternatives) fold output { values, ignored } regex-collapse-one-alternative (values, ignored)

regex-postfix-precollapse is fn (output : List (Int, String), atoms : Nat) -> List (Int, String)
  atoms > 1
    true then output append (13, "")
    false then output
regex-postfix-next-atoms is fn (atoms : Nat) -> Nat
  atoms > 1
    true then atoms
    false then atoms + 1
regex-postfix-parent-atoms is fn (atoms : Nat) -> Nat
  atoms > 1
    true then atoms - 1
    false then atoms

regex-postfix-atom is fn (state : (List (Int, String), List (Nat, Nat), Nat, Nat, Boolean, Boolean), token : (Int, String)) -> (List (Int, String), List (Nat, Nat), Nat, Nat, Boolean, Boolean)
  atoms is regex-postfix-atoms state
  collapsed is regex-postfix-precollapse (regex-postfix-output state, atoms)
  next-atoms is regex-postfix-next-atoms atoms
  (collapsed append token, regex-postfix-groups state, regex-postfix-alternatives state, next-atoms, regex-postfix-valid? state, false)
regex-postfix-open is fn (state : (List (Int, String), List (Nat, Nat), Nat, Nat, Boolean, Boolean)) -> (List (Int, String), List (Nat, Nat), Nat, Nat, Boolean, Boolean)
  atoms is regex-postfix-atoms state
  collapsed is regex-postfix-precollapse (regex-postfix-output state, atoms)
  parent-atoms is regex-postfix-parent-atoms atoms
  (collapsed, (regex-postfix-groups state) append (regex-postfix-alternatives state, parent-atoms), Nat 0, Nat 0, regex-postfix-valid? state, false)
regex-postfix-alt is fn (state : (List (Int, String), List (Nat, Nat), Nat, Nat, Boolean, Boolean)) -> (List (Int, String), List (Nat, Nat), Nat, Nat, Boolean, Boolean)
  (regex-postfix-atoms state) = 0
    true then (regex-postfix-output state, regex-postfix-groups state, regex-postfix-alternatives state, regex-postfix-atoms state, false, false)
    false then (regex-collapse-concats (regex-postfix-output state, regex-postfix-atoms state), regex-postfix-groups state, (regex-postfix-alternatives state) + 1, Nat 0, regex-postfix-valid? state, false)
regex-postfix-close-present is fn (state : (List (Int, String), List (Nat, Nat), Nat, Nat, Boolean, Boolean), parent : (Nat, Nat)) -> (List (Int, String), List (Nat, Nat), Nat, Nat, Boolean, Boolean)
  parent-alternatives is fn ((alternatives : Nat, atoms : Nat)) -> Nat
    alternatives
  parent-atoms is fn ((alternatives : Nat, atoms : Nat)) -> Nat
    atoms
  concatenated is regex-collapse-concats (regex-postfix-output state, regex-postfix-atoms state)
  completed is regex-collapse-alternatives (concatenated, regex-postfix-alternatives state)
  groups is regex-postfix-groups state
  remaining is groups select-index (0 .. ((entry-count groups) - 1))
  (completed, remaining, parent-alternatives parent, (parent-atoms parent) + 1, regex-postfix-valid? state, false)
regex-postfix-close is fn (state : (List (Int, String), List (Nat, Nat), Nat, Nat, Boolean, Boolean)) -> (List (Int, String), List (Nat, Nat), Nat, Nat, Boolean, Boolean)
  invalid? is ((regex-postfix-atoms state) = 0) or ((entry-count (regex-postfix-groups state)) = 0)
  invalid?
    true then (regex-postfix-output state, regex-postfix-groups state, regex-postfix-alternatives state, regex-postfix-atoms state, false, false)
    false then first ((regex-postfix-groups state) select-index (((entry-count (regex-postfix-groups state)) - 1) ..= ((entry-count (regex-postfix-groups state)) - 1)))
      Some parent then regex-postfix-close-present (state, parent)
      None then (regex-postfix-output state, regex-postfix-groups state, regex-postfix-alternatives state, regex-postfix-atoms state, false, false)
regex-postfix-unary is fn (state : (List (Int, String), List (Nat, Nat), Nat, Nat, Boolean, Boolean), token : (Int, String)) -> (List (Int, String), List (Nat, Nat), Nat, Nat, Boolean, Boolean)
  valid? is ((regex-postfix-atoms state) > 0) and (not (regex-postfix-quantified? state))
  valid?
    true then ((regex-postfix-output state) append token, regex-postfix-groups state, regex-postfix-alternatives state, regex-postfix-atoms state, regex-postfix-valid? state, true)
    false then (regex-postfix-output state, regex-postfix-groups state, regex-postfix-alternatives state, regex-postfix-atoms state, false, true)
regex-postfix-repeat-valid is fn (state : (List (Int, String), List (Nat, Nat), Nat, Nat, Boolean, Boolean), bounds : (Nat, Int, Boolean)) -> (List (Int, String), List (Nat, Nat), Nat, Nat, Boolean, Boolean)
  minimum is fn ((minimum : Nat, maximum : Int, valid? : Boolean)) -> Nat
    minimum
  maximum is fn ((minimum : Nat, maximum : Int, valid? : Boolean)) -> Int
    maximum
  valid is fn ((minimum : Nat, maximum : Int, valid? : Boolean)) -> Boolean
    valid?
  bounds-valid? is valid bounds
  has-atom? is (regex-postfix-atoms state) > 0
  unquantified? is not (regex-postfix-quantified? state)
  applicable? is (bounds-valid? and has-atom?) and unquantified?
  applicable?
    true then (regex-expand-repeat-valid (regex-postfix-output state, minimum bounds, maximum bounds), regex-postfix-groups state, regex-postfix-alternatives state, regex-postfix-atoms state, regex-postfix-valid? state, true)
    false then (regex-postfix-output state, regex-postfix-groups state, regex-postfix-alternatives state, regex-postfix-atoms state, false, true)
regex-postfix-token is fn (state : (List (Int, String), List (Nat, Nat), Nat, Nat, Boolean, Boolean), token : (Int, String)) -> (List (Int, String), List (Nat, Nat), Nat, Nat, Boolean, Boolean)
  regex-postfix-valid? state
    false then state
    true then regex-token-kind token
      <= 4 then regex-postfix-atom (state, token)
      = 5 then regex-postfix-open state
      = 6 then regex-postfix-close state
      = 7 then regex-postfix-alt state
      = 8 then regex-postfix-unary (state, token)
      = 9 then regex-postfix-unary (state, token)
      = 10 then regex-postfix-unary (state, token)
      = 11 then regex-postfix-repeat-valid (state, regex-repeat-bounds (regex-token-payload token))
      otherwise state
regex-postfix-finish is fn (state : (List (Int, String), List (Nat, Nat), Nat, Nat, Boolean, Boolean)) -> (List (Int, String), Boolean)
  groups-empty? is (entry-count (regex-postfix-groups state)) = 0
  valid? is (regex-postfix-valid? state) and groups-empty?
  valid?
    false then (regex-postfix-output state, false)
    true then (regex-postfix-atoms state) = 0
      true then ((regex-postfix-output state) append (12, ""), (regex-postfix-alternatives state) = 0)
      false then (regex-collapse-alternatives (regex-collapse-concats (regex-postfix-output state, regex-postfix-atoms state), regex-postfix-alternatives state), true)
regex-to-postfix-valid is fn (tokens : List (Int, String)) -> (List (Int, String), Boolean)
  empty-output : List (Int, String) is Empty
  empty-groups : List (Nat, Nat) is Empty
  final is tokens fold (empty-output, empty-groups, Nat 0, Nat 0, true, false) { state, token } regex-postfix-token (state, token)
  regex-postfix-finish final
regex-to-postfix is fn (tokenized : (List (Int, String), Boolean)) -> (List (Int, String), Boolean)
  tokens is fn ((tokens : List (Int, String), valid? : Boolean)) -> List (Int, String)
    tokens
  valid is fn ((tokens : List (Int, String), valid? : Boolean)) -> Boolean
    valid?
  valid tokenized
    true then regex-to-postfix-valid (tokens tokenized)
    false then (tokens tokenized, false)

regex-instruction-kind is fn ((kind : Int, payload : String, out : Int, out-two : Int)) -> Int
  kind
regex-instruction-payload is fn ((kind : Int, payload : String, out : Int, out-two : Int)) -> String
  payload
regex-instruction-out is fn ((kind : Int, payload : String, out : Int, out-two : Int)) -> Int
  out
regex-instruction-out-two is fn ((kind : Int, payload : String, out : Int, out-two : Int)) -> Int
  out-two
regex-fragment-start is fn ((start : Int, outs : List (Int, Int))) -> Int
  start
regex-fragment-outs is fn ((start : Int, outs : List (Int, Int))) -> List (Int, Int)
  outs

regex-patch-values is fn ((values : List (Int, String, Int, Int), index : Nat)) -> List (Int, String, Int, Int)
  values
regex-patch-index is fn ((values : List (Int, String, Int, Int), index : Nat)) -> Nat
  index
regex-patched-out is fn ((patched? : Boolean, target : Int, current : Int)) -> Int
  patched?
    true then target
    false then current
regex-patch-step is fn ((references : List (Int, Int), target : Int, state : (List (Int, String, Int, Int), Nat), instruction : (Int, String, Int, Int))) -> (List (Int, String, Int, Int), Nat)
  index is regex-patch-index state
  patch-one? is references contains-entry (index, 1)
  patch-two? is references contains-entry (index, 2)
  out is regex-patched-out (patch-one?, target, regex-instruction-out instruction)
  out-two is regex-patched-out (patch-two?, target, regex-instruction-out-two instruction)
  ((regex-patch-values state) append (regex-instruction-kind instruction, regex-instruction-payload instruction, out, out-two), index + 1)
regex-patch is fn (instructions : List (Int, String, Int, Int), (references : List (Int, Int), target : Int)) -> List (Int, String, Int, Int)
  empty-values : List (Int, String, Int, Int) is Empty
  final is instructions fold (empty-values, Nat 0) { state, instruction } regex-patch-step (references, target, state, instruction)
  regex-patch-values final

regex-compile-instructions is fn ((instructions : List (Int, String, Int, Int), fragments : List (Int, List (Int, Int)), valid? : Boolean)) -> List (Int, String, Int, Int)
  instructions
regex-compile-fragments is fn ((instructions : List (Int, String, Int, Int), fragments : List (Int, List (Int, Int)), valid? : Boolean)) -> List (Int, List (Int, Int))
  fragments
regex-compile-valid? is fn ((instructions : List (Int, String, Int, Int), fragments : List (Int, List (Int, Int)), valid? : Boolean)) -> Boolean
  valid?
regex-fragment-prefix is fn (fragments : List (Int, List (Int, Int)), removed : Nat) -> List (Int, List (Int, Int))
  fragments select-index (0 .. ((entry-count fragments) - removed))
regex-fragment-last is fn (fragments : List (Int, List (Int, Int))) -> Optional (Int, List (Int, Int))
  count is entry-count fragments
  first (fragments select-index ((count - 1) ..= (count - 1)))
regex-fragment-left is fn (fragments : List (Int, List (Int, Int))) -> Optional (Int, List (Int, Int))
  count is entry-count fragments
  first (fragments select-index ((count - 2) ..= (count - 2)))

regex-nfa-kind is fn (token-kind : Int) -> Int
  token-kind
    = 0 then 1
    = 1 then 2
    = 2 then 3
    = 3 then 4
    = 4 then 5
    otherwise 6
regex-compile-atom is fn (state : (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean), token : (Int, String)) -> (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean)
  index is entry-count (regex-compile-instructions state)
  instruction is (regex-nfa-kind (regex-token-kind token), regex-token-payload token, -1, -1)
  outs : List (Int, Int) is one (index, 1)
  ((regex-compile-instructions state) append instruction, (regex-compile-fragments state) append (index, outs), regex-compile-valid? state)

regex-compile-concat-parts is fn (state : (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean), parts : ((Int, List (Int, Int)), (Int, List (Int, Int)))) -> (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean)
  left is fn ((left : (Int, List (Int, Int)), right : (Int, List (Int, Int)))) -> (Int, List (Int, Int))
    left
  right is fn ((left : (Int, List (Int, Int)), right : (Int, List (Int, Int)))) -> (Int, List (Int, Int))
    right
  instructions is regex-patch (regex-compile-instructions state, (regex-fragment-outs (left parts), regex-fragment-start (right parts)))
  fragments is (regex-fragment-prefix (regex-compile-fragments state, 2)) append (regex-fragment-start (left parts), regex-fragment-outs (right parts))
  (instructions, fragments, regex-compile-valid? state)
regex-compile-concat-right is fn (state : (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean), pair : ((Int, List (Int, Int)), Optional (Int, List (Int, Int)))) -> (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean)
  left is fn ((left : (Int, List (Int, Int)), right : Optional (Int, List (Int, Int)))) -> (Int, List (Int, Int))
    left
  right is fn ((left : (Int, List (Int, Int)), right : Optional (Int, List (Int, Int)))) -> Optional (Int, List (Int, Int))
    right
  right pair
    Some present then regex-compile-concat-parts (state, (left pair, present))
    None then (regex-compile-instructions state, regex-compile-fragments state, false)
regex-compile-concat-left is fn (state : (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean), pair : (Optional (Int, List (Int, Int)), Optional (Int, List (Int, Int)))) -> (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean)
  left is fn ((left : Optional (Int, List (Int, Int)), right : Optional (Int, List (Int, Int)))) -> Optional (Int, List (Int, Int))
    left
  right is fn ((left : Optional (Int, List (Int, Int)), right : Optional (Int, List (Int, Int)))) -> Optional (Int, List (Int, Int))
    right
  left pair
    Some present then regex-compile-concat-right (state, (present, right pair))
    None then (regex-compile-instructions state, regex-compile-fragments state, false)
regex-compile-concat is fn (state : (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean)) -> (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean)
  fragments is regex-compile-fragments state
  (entry-count fragments) >= 2
    true then regex-compile-concat-left (state, (regex-fragment-left fragments, regex-fragment-last fragments))
    false then (regex-compile-instructions state, fragments, false)

regex-compile-alternative-parts is fn (state : (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean), parts : ((Int, List (Int, Int)), (Int, List (Int, Int)))) -> (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean)
  left is fn ((left : (Int, List (Int, Int)), right : (Int, List (Int, Int)))) -> (Int, List (Int, Int))
    left
  right is fn ((left : (Int, List (Int, Int)), right : (Int, List (Int, Int)))) -> (Int, List (Int, Int))
    right
  index is entry-count (regex-compile-instructions state)
  instruction is (7, "", regex-fragment-start (left parts), regex-fragment-start (right parts))
  outs is (regex-fragment-outs (left parts)) concat (regex-fragment-outs (right parts))
  fragments is (regex-fragment-prefix (regex-compile-fragments state, 2)) append (index, outs)
  ((regex-compile-instructions state) append instruction, fragments, regex-compile-valid? state)
regex-compile-alternative-right is fn (state : (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean), pair : ((Int, List (Int, Int)), Optional (Int, List (Int, Int)))) -> (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean)
  left is fn ((left : (Int, List (Int, Int)), right : Optional (Int, List (Int, Int)))) -> (Int, List (Int, Int))
    left
  right is fn ((left : (Int, List (Int, Int)), right : Optional (Int, List (Int, Int)))) -> Optional (Int, List (Int, Int))
    right
  right pair
    Some present then regex-compile-alternative-parts (state, (left pair, present))
    None then (regex-compile-instructions state, regex-compile-fragments state, false)
regex-compile-alternative-left is fn (state : (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean), pair : (Optional (Int, List (Int, Int)), Optional (Int, List (Int, Int)))) -> (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean)
  left is fn ((left : Optional (Int, List (Int, Int)), right : Optional (Int, List (Int, Int)))) -> Optional (Int, List (Int, Int))
    left
  right is fn ((left : Optional (Int, List (Int, Int)), right : Optional (Int, List (Int, Int)))) -> Optional (Int, List (Int, Int))
    right
  left pair
    Some present then regex-compile-alternative-right (state, (present, right pair))
    None then (regex-compile-instructions state, regex-compile-fragments state, false)
regex-compile-alternative is fn (state : (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean)) -> (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean)
  fragments is regex-compile-fragments state
  (entry-count fragments) >= 2
    true then regex-compile-alternative-left (state, (regex-fragment-left fragments, regex-fragment-last fragments))
    false then (regex-compile-instructions state, fragments, false)

regex-compile-star is fn (state : (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean), fragment : (Int, List (Int, Int))) -> (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean)
  index is entry-count (regex-compile-instructions state)
  split is (7, "", regex-fragment-start fragment, -1)
  prefix is regex-fragment-prefix (regex-compile-fragments state, 1)
  patched is regex-patch (regex-compile-instructions state, (regex-fragment-outs fragment, index))
  (patched append split, prefix append (index, one (index, 2)), regex-compile-valid? state)
regex-compile-plus is fn (state : (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean), fragment : (Int, List (Int, Int))) -> (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean)
  index is entry-count (regex-compile-instructions state)
  split is (7, "", regex-fragment-start fragment, -1)
  prefix is regex-fragment-prefix (regex-compile-fragments state, 1)
  patched is regex-patch (regex-compile-instructions state, (regex-fragment-outs fragment, index))
  (patched append split, prefix append (regex-fragment-start fragment, one (index, 2)), regex-compile-valid? state)
regex-compile-question is fn (state : (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean), fragment : (Int, List (Int, Int))) -> (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean)
  index is entry-count (regex-compile-instructions state)
  split is (7, "", regex-fragment-start fragment, -1)
  prefix is regex-fragment-prefix (regex-compile-fragments state, 1)
  ((regex-compile-instructions state) append split, prefix append (index, (regex-fragment-outs fragment) append (index, 2)), regex-compile-valid? state)
regex-compile-unary-present is fn (state : (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean), pair : ((Int, List (Int, Int)), Int)) -> (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean)
  fragment is fn ((fragment : (Int, List (Int, Int)), kind : Int)) -> (Int, List (Int, Int))
    fragment
  kind is fn ((fragment : (Int, List (Int, Int)), kind : Int)) -> Int
    kind
  kind pair
    = 8 then regex-compile-star (state, fragment pair)
    = 9 then regex-compile-plus (state, fragment pair)
    otherwise regex-compile-question (state, fragment pair)
regex-compile-unary is fn (state : (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean), kind : Int) -> (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean)
  regex-fragment-last (regex-compile-fragments state)
    Some fragment then regex-compile-unary-present (state, (fragment, kind))
    None then (regex-compile-instructions state, regex-compile-fragments state, false)

regex-compile-token-valid is fn (state : (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean), token : (Int, String)) -> (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean)
  regex-token-kind token
    <= 4 then regex-compile-atom (state, token)
    = 12 then regex-compile-atom (state, token)
    = 13 then regex-compile-concat state
    = 7 then regex-compile-alternative state
    = 8 then regex-compile-unary (state, 8)
    = 9 then regex-compile-unary (state, 9)
    = 10 then regex-compile-unary (state, 10)
    otherwise (regex-compile-instructions state, regex-compile-fragments state, false)
regex-compile-token is fn (state : (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean), token : (Int, String)) -> (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean)
  regex-compile-valid? state
    true then regex-compile-token-valid (state, token)
    false then state

regex-compile-finish-present is fn (state : (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean), fragment : (Int, List (Int, Int))) -> (List (Int, String, Int, Int), Int, Boolean)
  match-index is entry-count (regex-compile-instructions state)
  patched is regex-patch (regex-compile-instructions state, (regex-fragment-outs fragment, match-index))
  (patched append (0, "", -1, -1), regex-fragment-start fragment, regex-compile-valid? state)
regex-compile-finish is fn (state : (List (Int, String, Int, Int), List (Int, List (Int, Int)), Boolean)) -> (List (Int, String, Int, Int), Int, Boolean)
  valid-shape? is (regex-compile-valid? state) and ((entry-count (regex-compile-fragments state)) = 1)
  valid-shape?
    false then (regex-compile-instructions state, -1, false)
    true then regex-fragment-last (regex-compile-fragments state)
      Some fragment then regex-compile-finish-present (state, fragment)
      None then (regex-compile-instructions state, -1, false)
regex-compile-postfix-valid is fn (tokens : List (Int, String)) -> (List (Int, String, Int, Int), Int, Boolean)
  empty-instructions : List (Int, String, Int, Int) is Empty
  empty-fragments : List (Int, List (Int, Int)) is Empty
  final is tokens fold (empty-instructions, empty-fragments, true) { state, token } regex-compile-token (state, token)
  regex-compile-finish final
regex-compile-postfix-invalid is fn (ignored : List (Int, String)) -> (List (Int, String, Int, Int), Int, Boolean)
  empty-instructions : List (Int, String, Int, Int) is Empty
  (empty-instructions, -1, false)
regex-compile-postfix is fn (postfix : (List (Int, String), Boolean)) -> (List (Int, String, Int, Int), Int, Boolean)
  tokens is fn ((tokens : List (Int, String), valid? : Boolean)) -> List (Int, String)
    tokens
  valid is fn ((tokens : List (Int, String), valid? : Boolean)) -> Boolean
    valid?
  valid postfix
    true then regex-compile-postfix-valid (tokens postfix)
    false then regex-compile-postfix-invalid (tokens postfix)

regex-class-tokens is fn ((tokens : List (Int, String), escaped? : Boolean, valid? : Boolean)) -> List (Int, String)
  tokens
regex-class-escaped? is fn ((tokens : List (Int, String), escaped? : Boolean, valid? : Boolean)) -> Boolean
  escaped?
regex-class-valid? is fn ((tokens : List (Int, String), escaped? : Boolean, valid? : Boolean)) -> Boolean
  valid?
regex-class-escape-token is fn (character : Character) -> (Int, String)
  character
    = "d" then (1, "")
    = "D" then (2, "")
    = "s" then (3, "")
    = "S" then (4, "")
    = "w" then (5, "")
    = "W" then (6, "")
    otherwise (7, character)
regex-class-scan-normal is fn (state : (List (Int, String), Boolean, Boolean), character : Character) -> (List (Int, String), Boolean, Boolean)
  regex-backslash? character
    true then (regex-class-tokens state, true, regex-class-valid? state)
    false then ((regex-class-tokens state) append (0, character), false, regex-class-valid? state)
regex-class-scan-step is fn (state : (List (Int, String), Boolean, Boolean), character : Character) -> (List (Int, String), Boolean, Boolean)
  regex-class-escaped? state
    true then ((regex-class-tokens state) append (regex-class-escape-token character), false, regex-class-valid? state)
    false then regex-class-scan-normal (state, character)
regex-class-leading-negated? is fn (leading : Optional Character) -> Boolean
  leading
    Some character then character = "^"
    None then false
regex-class-body is fn (source : List Character, negated? : Boolean) -> List Character
  negated?
    true then source select-index (1 .. (entry-count source))
    false then source
regex-class-tokenize is fn (payload : String) -> (List (Int, String), Boolean, Boolean)
  source is unicode-scalar-characters payload
  leading is first (source select-index (0 ..= 0))
  negated? is regex-class-leading-negated? leading
  body is regex-class-body (source, negated?)
  empty-tokens : List (Int, String) is Empty
  final is body fold (empty-tokens, false, true) { state, character } regex-class-scan-step (state, character)
  scan-valid? is regex-class-valid? final
  complete? is not (regex-class-escaped? final)
  nonempty? is (entry-count (regex-class-tokens final)) > 0
  (regex-class-tokens final, negated?, (scan-valid? and complete?) and nonempty?)
regex-class-tokenized-valid? is fn ((tokens : List (Int, String), negated? : Boolean, valid? : Boolean)) -> Boolean
  valid?

regex-class-token-at is fn (tokens : List (Int, String), index : Nat) -> Optional (Int, String)
  first (tokens select-index (index ..= index))
regex-class-literal? is fn (token : (Int, String)) -> Boolean
  ((regex-token-kind token) = 0) or ((regex-token-kind token) = 7)
regex-class-special-match? is fn (token : (Int, String), candidate : Character) -> Boolean
  regex-token-kind token
    = 1 then unicode-decimal-digit-character candidate
    = 2 then not (unicode-decimal-digit-character candidate)
    = 3 then unicode-whitespace-character candidate
    = 4 then not (unicode-whitespace-character candidate)
    = 5 then unicode-word-character candidate
    = 6 then not (unicode-word-character candidate)
    otherwise false
regex-class-range-match is fn (candidate : Character, endpoints : ((Int, String), (Int, String))) -> Boolean
  left is fn ((left : (Int, String), right : (Int, String))) -> (Int, String)
    left
  right is fn ((left : (Int, String), right : (Int, String))) -> (Int, String)
    right
  lower is unicode-scalar-value (regex-token-payload (left endpoints))
  upper is unicode-scalar-value (regex-token-payload (right endpoints))
  value is unicode-scalar-value candidate
  (lower <= value) and (value <= upper)
regex-class-range-endpoints-valid? is fn (left : (Int, String), endpoint : (Int, String)) -> Boolean
  literal? is (regex-class-literal? left) and (regex-class-literal? endpoint)
  ordered? is (unicode-scalar-value (regex-token-payload left)) <= (unicode-scalar-value (regex-token-payload endpoint))
  literal? and ordered?
regex-class-range-valid-endpoint is fn (left : (Int, String), right : Optional (Int, String)) -> Boolean
  right
    Some endpoint then regex-class-range-endpoints-valid? (left, endpoint)
    None then false
regex-class-range-valid-next is fn ((tokens : List (Int, String), index : Nat, token : (Int, String), next : (Int, String))) -> Boolean
  separator? is ((regex-token-kind next) = 0) and ((regex-token-payload next) = "-")
  endpoint? is (index + 2) < (entry-count tokens)
  range? is separator? and endpoint?
  range?
    true then regex-class-range-valid-endpoint (token, regex-class-token-at (tokens, index + 2))
    false then true
regex-class-range-valid-token is fn ((tokens : List (Int, String), index : Nat, token : (Int, String))) -> Boolean
  regex-class-token-at (tokens, index + 1)
    Some next then regex-class-range-valid-next (tokens, index, token, next)
    None then true
regex-class-range-valid-index is fn (tokens : List (Int, String), index : Nat) -> Boolean
  regex-class-token-at (tokens, index)
    Some token then regex-class-range-valid-token (tokens, index, token)
    None then true
regex-class-range-valid-at is fn ((tokens : List (Int, String), valid? : Boolean, index : Int)) -> Boolean
  converted : Nat is Nat index
  valid?
    false then false
    true then regex-class-range-valid-index (tokens, converted)
regex-class-payload-valid? is fn (payload : String) -> Boolean
  tokenized is regex-class-tokenize payload
  tokens is regex-class-tokens tokenized
  base-valid? is regex-class-tokenized-valid? tokenized
  (regex-counts (entry-count tokens)) fold base-valid? { valid?, index } regex-class-range-valid-at (tokens, valid?, index)
regex-class-previous-present? is fn (tokens : List (Int, String), index : Nat) -> Boolean
  regex-class-token-at (tokens, index - 1)
    Some separator then ((regex-token-kind separator) = 0) and ((regex-token-payload separator) = "-")
    None then false
regex-class-previous-separator? is fn (tokens : List (Int, String), index : Nat) -> Boolean
  index > 0
    false then false
    true then regex-class-previous-present? (tokens, index)
regex-class-literal-match? is fn ((tokens : List (Int, String), candidate : Character, index : Nat, token : (Int, String))) -> Boolean
  regex-class-previous-separator? (tokens, index)
    true then false
    false then (regex-class-literal? token) and ((regex-token-payload token) = candidate)
regex-class-range-endpoints-match? is fn ((candidate : Character, left : (Int, String), endpoint : (Int, String))) -> Boolean
  literals? is (regex-class-literal? left) and (regex-class-literal? endpoint)
  literals? and (regex-class-range-match (candidate, (left, endpoint)))
regex-class-range-endpoint is fn ((tokens : List (Int, String), candidate : Character, index : Nat, token : (Int, String))) -> Boolean
  regex-class-token-at (tokens, index + 2)
    Some endpoint then regex-class-range-endpoints-match? (candidate, token, endpoint)
    None then false
regex-class-next-token is fn ((tokens : List (Int, String), candidate : Character, index : Nat, token : (Int, String), separator : (Int, String))) -> Boolean
  separator? is ((regex-token-kind separator) = 0) and ((regex-token-payload separator) = "-")
  endpoint? is (index + 2) < (entry-count tokens)
  range? is separator? and endpoint?
  range?
    true then regex-class-range-endpoint (tokens, candidate, index, token)
    false then regex-class-literal-match? (tokens, candidate, index, token)
regex-class-index-literal-match? is fn ((tokens : List (Int, String), candidate : Character, index : Nat, token : (Int, String))) -> Boolean
  regex-class-token-at (tokens, index + 1)
    Some separator then regex-class-next-token (tokens, candidate, index, token, separator)
    None then regex-class-literal-match? (tokens, candidate, index, token)
regex-class-index-match-present is fn ((tokens : List (Int, String), candidate : Character, index : Nat, token : (Int, String))) -> Boolean
  kind is regex-token-kind token
  (kind >= 1) and (kind <= 6)
    true then regex-class-special-match? (token, candidate)
    false then regex-class-index-literal-match? (tokens, candidate, index, token)
regex-class-index-match? is fn ((tokens : List (Int, String), candidate : Character, index : Nat)) -> Boolean
  regex-class-token-at (tokens, index)
    Some token then regex-class-index-match-present (tokens, candidate, index, token)
    None then false
regex-class-match-step is fn ((tokens : List (Int, String), candidate : Character, found : Boolean, index : Int)) -> Boolean
  found or (regex-class-index-match? (tokens, candidate, Nat index))
regex-class-matches? is fn (payload : String, candidate : Character) -> Boolean
  tokenized is regex-class-tokenize payload
  tokens is fn ((tokens : List (Int, String), negated? : Boolean, valid? : Boolean)) -> List (Int, String)
    tokens
  negated is fn ((tokens : List (Int, String), negated? : Boolean, valid? : Boolean)) -> Boolean
    negated?
  indexes is regex-counts (entry-count (tokens tokenized))
  found is indexes fold false { state, index } regex-class-match-step (tokens tokenized, candidate, state, index)
  negated tokenized
    true then not found
    false then found

regex-instruction-at-valid is fn (instructions : List (Int, String, Int, Int), index : Int) -> Optional (Int, String, Int, Int)
  converted : Nat is Nat index
  first (instructions select-index (converted ..= converted))
regex-instruction-at is fn (instructions : List (Int, String, Int, Int), index : Int) -> Optional (Int, String, Int, Int)
  index < 0
    true then None (Int, String, Int, Int)
    false then regex-instruction-at-valid (instructions, index)
regex-state-add is fn (states : List Int, index : Int) -> List Int
  (index >= 0) and (not (states contains-entry index))
    true then states append index
    false then states
regex-closure-assert is fn ((enabled? : Boolean, states : List Int, instruction : (Int, String, Int, Int))) -> List Int
  enabled?
    true then regex-state-add (states, regex-instruction-out instruction)
    false then states
regex-closure-instruction is fn ((position : Nat, length : Nat, states : List Int, instruction : (Int, String, Int, Int))) -> List Int
  regex-instruction-kind instruction
    = 4 then regex-closure-assert (position = 0, states, instruction)
    = 5 then regex-closure-assert (position = length, states, instruction)
    = 6 then regex-state-add (states, regex-instruction-out instruction)
    = 7 then regex-state-add (regex-state-add (states, regex-instruction-out instruction), regex-instruction-out-two instruction)
    otherwise states
regex-closure-state-step is fn ((instructions : List (Int, String, Int, Int), position : Nat, length : Nat, states : List Int, index : Int)) -> List Int
  regex-instruction-at (instructions, index)
    Some instruction then regex-closure-instruction (position, length, states, instruction)
    None then states
regex-closure-pass is fn ((instructions : List (Int, String, Int, Int), position : Nat, length : Nat, states : List Int)) -> List Int
  states fold states { expanded, index } regex-closure-state-step (instructions, position, length, expanded, index)
regex-closure-repeat is fn ((instructions : List (Int, String, Int, Int), position : Nat, length : Nat, states : List Int, ignored : Int)) -> List Int
  regex-closure-pass (instructions, position, length, states)
regex-closure is fn ((instructions : List (Int, String, Int, Int), position : Nat, length : Nat, states : List Int)) -> List Int
  (regex-counts (entry-count instructions)) fold states { expanded, ignored } regex-closure-repeat (instructions, position, length, expanded, ignored)
regex-consuming-match? is fn (instruction : (Int, String, Int, Int), candidate : Character) -> Boolean
  regex-instruction-kind instruction
    = 1 then (regex-instruction-payload instruction) = candidate
    = 2 then not (unicode-line-feed-character candidate)
    = 3 then regex-class-matches? (regex-instruction-payload instruction, candidate)
    otherwise false
regex-consume-present is fn ((states : List Int, candidate : Character, instruction : (Int, String, Int, Int))) -> List Int
  regex-consuming-match? (instruction, candidate)
    true then regex-state-add (states, regex-instruction-out instruction)
    false then states
regex-consume-step is fn ((instructions : List (Int, String, Int, Int), candidate : Character, states : List Int, index : Int)) -> List Int
  regex-instruction-at (instructions, index)
    Some instruction then regex-consume-present (states, candidate, instruction)
    None then states
regex-consume is fn ((instructions : List (Int, String, Int, Int), active : List Int, candidate : Character)) -> List Int
  empty : List Int is Empty
  active fold empty { states, index } regex-consume-step (instructions, candidate, states, index)
regex-has-match-step is fn ((instructions : List (Int, String, Int, Int), found : Boolean, index : Int)) -> Boolean
  regex-instruction-at (instructions, index)
    Some instruction then found or ((regex-instruction-kind instruction) = 0)
    None then found
regex-has-match? is fn (instructions : List (Int, String, Int, Int), active : List Int) -> Boolean
  active fold false { found, index } regex-has-match-step (instructions, found, index)
regex-run-active is fn ((active : List Int, position : Nat, matched? : Boolean)) -> List Int
  active
regex-run-position is fn ((active : List Int, position : Nat, matched? : Boolean)) -> Nat
  position
regex-run-matched? is fn ((active : List Int, position : Nat, matched? : Boolean)) -> Boolean
  matched?
regex-run-step is fn ((instructions : List (Int, String, Int, Int), start : Int, length : Nat, state : (List Int, Nat, Boolean), candidate : Character)) -> (List Int, Nat, Boolean)
  position is regex-run-position state
  seeded is regex-state-add (regex-run-active state, start)
  closed is regex-closure (instructions, position, length, seeded)
  matched? is (regex-run-matched? state) or (regex-has-match? (instructions, closed))
  consumed is regex-consume (instructions, closed, candidate)
  next is regex-closure (instructions, position + 1, length, consumed)
  (next, position + 1, matched? or (regex-has-match? (instructions, next)))
regex-run is fn (text : String, compiled : (List (Int, String, Int, Int), Int, Boolean)) -> Boolean
  instructions is fn ((instructions : List (Int, String, Int, Int), start : Int, valid? : Boolean)) -> List (Int, String, Int, Int)
    instructions
  start is fn ((instructions : List (Int, String, Int, Int), start : Int, valid? : Boolean)) -> Int
    start
  source is unicode-scalar-characters text
  empty : List Int is Empty
  final is source fold (empty, Nat 0, false) { state, candidate } regex-run-step (instructions compiled, start compiled, entry-count source, state, candidate)
  ending is regex-closure (instructions compiled, entry-count source, entry-count source, regex-state-add (regex-run-active final, start compiled))
  (regex-run-matched? final) or (regex-has-match? (instructions compiled, ending))

regex-pattern-token-valid? is fn (valid? : Boolean, token : (Int, String)) -> Boolean
  valid?
    false then false
    true then (regex-token-kind token) = 2
      true then regex-class-payload-valid? (regex-token-payload token)
      false then true
regex-pattern-classes-valid? is fn (tokenized : (List (Int, String), Boolean)) -> Boolean
  tokens is fn ((tokens : List (Int, String), valid? : Boolean)) -> List (Int, String)
    tokens
  valid is fn ((tokens : List (Int, String), valid? : Boolean)) -> Boolean
    valid?
  (tokens tokenized) fold (valid tokenized) { state, token } regex-pattern-token-valid? (state, token)
regex-compiled-valid? is fn ((instructions : List (Int, String, Int, Int), start : Int, valid? : Boolean)) -> Boolean
  valid?
RegexValid is Boolean constraint { valid } valid

### Test whether the capture-free regular expression matches a substring.
pub regex-contains? is fn (text : String, pattern : String) -> Boolean
  tokenized is regex-tokenize pattern
  compiled is regex-compile-postfix (regex-to-postfix tokenized)
  valid? is (regex-compiled-valid? compiled) and (regex-pattern-classes-valid? tokenized)
  checked : RegexValid is RegexValid valid?
  _ is checked
  regex-run (text, compiled)

### Test whether any exact String pattern occurs.
contains-any-step is fn ((found : Boolean, text : String, pattern : String)) -> Boolean
  found or (contains? (text, pattern))

pub contains-any? is fn (text : String, patterns : List String) -> Boolean
  patterns fold false { found, pattern } contains-any-step (found, text, pattern)
