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

### Split text into lines without retaining line terminators.
pub lines is fn (text : String) -> List String
  string-lines text

### Split text on Unicode whitespace and omit empty pieces.
pub words is fn (text : String) -> List String
  string-words text

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
