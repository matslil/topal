#!/usr/bin/env topal
use language (
  version is v0.1
)

### Revision of the representation-independent text parsing namespace.
pub revision is 1

### Parse one complete signed ASCII decimal integer.
pub int is fn (text : String) -> Optional Int
  string-parse-int text

### Extract signed ASCII decimal integers in source order.
pub signed-integers is fn (text : String) -> List Int
  string-signed-integers text

### Extract unsigned ASCII decimal integers in source order.
pub unsigned-integers is fn (text : String) -> List Nat
  string-unsigned-integers text

### Extract every nonempty row of signed decimal integers.
pub integer-rows is fn (text : String) -> List (List Int)
  string-integer-rows text

### Read fixed-width vertical decimal columns, preserving blank separators.
pub vertical-integers is fn (text : String) -> List (Optional Int)
  string-vertical-integers text

### Convert an ASCII decimal digit String into its individual values.
pub decimal-digits is fn (text : String) -> List Nat
  string-decimal-digits text

### Format an exact integer in canonical base-ten notation.
pub decimal is fn (value : Int) -> String
  int-decimal-string value

### Materialize Unicode Characters without exposing encoded units.
pub character-list is fn (text : String) -> List Character
  string-characters text

### Construct a String from complete Unicode Characters.
pub string is fn (values : List Character) -> String
  character-list-string values
