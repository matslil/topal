#!/usr/bin/env topal
use language (
  version is v0.1
)

### Revision of the text-algorithm namespace.
pub revision is 1

### Test canonical Unicode equality while retaining case significance.
pub canonical-equal? is fn (left : String, right : String) -> Boolean
  left canonically-equals right

### Test Unicode default caseless equality without locale policy.
pub caseless-equal? is fn (left : String, right : String) -> Boolean
  (case-fold left) = (case-fold right)

unicode-whitespace? is fn (character : Character) -> Boolean
  unicode-whitespace-character character

leading-count is fn ((count : Nat, leading? : Boolean)) -> Nat
  count
leading-active? is fn ((count : Nat, leading? : Boolean)) -> Boolean
  leading?

leading-step is fn (state : (Nat, Boolean), character : Character) -> (Nat, Boolean)
  active? is leading-active? state
  whitespace? is unicode-whitespace? character
  active? and whitespace?
    true then ((leading-count state) + 1, true)
    false then (leading-count state, false)

trim-boundary is fn (characters : List Character) -> Nat
  zero : Nat is Nat 0
  final is characters fold (zero, true) { state, character } leading-step (state, character)
  leading-count final

trim-source is fn (text : String) -> String
  characters is collect (characters text)
  leading is trim-boundary characters
  trailing is trim-boundary ((collect (characters text)) reverse)
  text select-index (leading .. ((entry-count characters) - trailing))

### Test whether a String contains only Unicode whitespace or is empty.
pub blank? is fn (text : String) -> Boolean
  trimmed is trim-source text
  empty? trimmed

### Return text normalized to Unicode NFC.
pub nfc is fn (text : String) -> String
  text normalize NFC

### Return text normalized to Unicode NFD.
pub nfd is fn (text : String) -> String
  text normalize NFD

line-values is fn ((values : List String, start : Nat, index : Nat)) -> List String
  values
line-start is fn ((values : List String, start : Nat, index : Nat)) -> Nat
  start
line-index is fn ((values : List String, start : Nat, index : Nat)) -> Nat
  index

optional-carriage-return? is fn (value : Optional Character) -> Boolean
  value
    Some previous then unicode-carriage-return-character previous
    None then false

previous-is-carriage-return? is fn ((text : String, start : Nat, index : Nat)) -> Boolean
  index > start
    true then optional-carriage-return? (first (collect (characters (text select-index ((index - 1) .. index)))))
    false then false

line-content-end is fn ((text : String, start : Nat, index : Nat)) -> Nat
  previous-is-carriage-return? (text, start, index)
    true then index - 1
    false then index

append-line is fn ((text : String, state : (List String, Nat, Nat))) -> (List String, Nat, Nat)
  index is line-index state
  finish is line-content-end (text, line-start state, index)
  ((line-values state) append (text select-index ((line-start state) .. finish)), index + 1, index + 1)

advance-line is fn (state : (List String, Nat, Nat)) -> (List String, Nat, Nat)
  (line-values state, line-start state, (line-index state) + 1)

line-step is fn ((text : String, state : (List String, Nat, Nat), character : Character)) -> (List String, Nat, Nat)
  unicode-line-feed-character character
    true then append-line (text, state)
    false then advance-line state

finish-lines is fn (text : String, state : (List String, Nat, Nat)) -> List String
  start is line-start state
  length is line-index state
  start < length
    true then (line-values state) append (text select-index (start .. length))
    false then line-values state

word-values is fn ((values : List String, start : Nat, index : Nat, inside? : Boolean)) -> List String
  values
word-start is fn ((values : List String, start : Nat, index : Nat, inside? : Boolean)) -> Nat
  start
word-index is fn ((values : List String, start : Nat, index : Nat, inside? : Boolean)) -> Nat
  index
word-inside? is fn ((values : List String, start : Nat, index : Nat, inside? : Boolean)) -> Boolean
  inside?

word-on-whitespace is fn ((text : String, state : (List String, Nat, Nat, Boolean))) -> (List String, Nat, Nat, Boolean)
  index is word-index state
  word-inside? state
    true then ((word-values state) append (text select-index ((word-start state) .. index)), index + 1, index + 1, false)
    false then (word-values state, index + 1, index + 1, false)

word-on-content is fn (state : (List String, Nat, Nat, Boolean)) -> (List String, Nat, Nat, Boolean)
  index is word-index state
  word-inside? state
    true then (word-values state, word-start state, index + 1, true)
    false then (word-values state, index, index + 1, true)

word-step is fn ((text : String, state : (List String, Nat, Nat, Boolean), character : Character)) -> (List String, Nat, Nat, Boolean)
  unicode-whitespace? character
    true then word-on-whitespace (text, state)
    false then word-on-content state

finish-words is fn (text : String, state : (List String, Nat, Nat, Boolean)) -> List String
  word-inside? state
    true then (word-values state) append (text select-index ((word-start state) .. (word-index state)))
    false then word-values state

### Split text into lines without retaining line terminators.
pub lines is fn (text : String) -> List String
  characters is collect (characters text)
  empty-lines : List String is Empty
  final is characters fold (empty-lines, Nat 0, Nat 0) { state, character } line-step (text, state, character)
  finish-lines (text, final)

### Split text on Unicode whitespace and omit empty pieces.
pub words is fn (text : String) -> List String
  characters is collect (characters text)
  empty-words : List String is Empty
  final is characters fold (empty-words, Nat 0, Nat 0, false) { state, character } word-step (text, state, character)
  finish-words (text, final)

### Join String entries with one exact separator.
pub join is fn (values : List String, separator : String) -> String
  joined-text is fn ((text : String, first? : Boolean)) -> String
    text
  joined-first? is fn ((text : String, first? : Boolean)) -> Boolean
    first?
  join-step is fn ((state : (String, Boolean), value : String, separator : String)) -> (String, Boolean)
    joined-first? state
      true then (value, false)
      false then ((joined-text state) concat separator concat value, false)
  final is values fold ("", true) { state, value } join-step (state, value, separator)
  joined-text final
