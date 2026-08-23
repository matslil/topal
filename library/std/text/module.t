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

### Test whether a String contains only Unicode whitespace or is empty.
pub blank? is fn (text : String) -> Boolean
  trimmed is string-trim text
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
  string-join (values, separator)
