#!/usr/bin/env topal
use language (
  version is v0.1
)

### Revision of the representation-independent text parsing namespace.
pub revision is 1

parse-value is fn ((value : Int, valid? : Boolean)) -> Int
  value
parse-valid? is fn ((value : Int, valid? : Boolean)) -> Boolean
  valid?

parse-present-digit is fn (state : (Int, Boolean), digit : Nat) -> (Int, Boolean)
  ((parse-value state) * 10 + digit, parse-valid? state)

parse-digit-step is fn (state : (Int, Boolean), character : Character) -> (Int, Boolean)
  ascii-decimal-digit character
    Some digit then parse-present-digit (state, digit)
    None then (parse-value state, false)

present-int is fn (value : Int) -> Optional Int
  present : Optional Int is Some value
  present

finish-int is fn ((state : (Int, Boolean), sign : Int, digit-count : Nat)) -> Optional Int
  valid is (parse-valid? state) and (digit-count > 0)
  valid
    true then present-int (sign * (parse-value state))
    false then None Int

sign-from-character is fn (character : Character) -> Int
  character = "-"
    true then -1
    false then 1

parse-sign is fn (characters : List Character) -> Int
  first characters
    Some character then sign-from-character character
    None then 1

start-from-character is fn (character : Character) -> Nat
  (character = "-") or (character = "+")
    true then 1
    false then 0

parse-start is fn (characters : List Character) -> Nat
  first characters
    Some character then start-from-character character
    None then 0

### Parse one complete signed ASCII decimal integer.
pub int is fn (text : String) -> Optional Int
  characters is collect (characters text)
  start is parse-start characters
  digits is characters select-index (start .. (entry-count characters))
  state is digits fold (0, true) { parsed, character } parse-digit-step (parsed, character)
  finish-int (state, parse-sign characters, entry-count digits)

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

### Extract rows containing exactly two signed decimal integers.
pub integer-pairs is fn (text : String) -> List (Int, Int)
  string-integer-pairs text

### Extract rows containing exactly three signed decimal integers.
pub integer-triples is fn (text : String) -> List (Int, Int, Int)
  string-integer-triples text

append-present-digit is fn (digits : List Nat, digit : Optional Nat) -> List Nat
  digit
    Some value then digits append value
    None then digits

append-decimal-digit is fn (digits : List Nat, character : Character) -> List Nat
  append-present-digit (digits, ascii-decimal-digit character)

DecimalText is String constraint { text } ascii-decimal-text? text

### Convert an ASCII decimal digit String into its individual values.
pub decimal-digits is fn (text : String) -> List Nat
  checked : DecimalText is DecimalText text
  _ is checked
  digit-values : List Nat is Empty
  (collect (characters text)) fold digit-values { digits, character } append-decimal-digit (digits, character)

digit-string is fn (digit : Nat) -> String
  digit
    = 0 then "0"
    = 1 then "1"
    = 2 then "2"
    = 3 then "3"
    = 4 then "4"
    = 5 then "5"
    = 6 then "6"
    = 7 then "7"
    = 8 then "8"
    otherwise "9"

quotient-value is fn ((quotient : Int, remainder : Int)) -> Int
  quotient
remainder-value is fn ((quotient : Int, remainder : Int)) -> Int
  remainder

prepend-decimal-digit is fn (text : String, value : Nat) -> String
  divided is value /% 10
  digit : Nat is Nat (remainder-value divided)
  (digit-string digit) concat text

decimal-positive-nonzero is fn (value : Nat) -> String
  places is collect (value iterate ({ current } quotient-value (current /% 10)) take-while ({ current } current > 0))
  places fold "" { text, place } prepend-decimal-digit (text, place)

decimal-positive is fn (value : Nat) -> String
  value = 0
    true then "0"
    false then decimal-positive-nonzero value

### Format an exact integer in canonical base-ten notation.
pub decimal is fn (value : Int) -> String
  value < 0
    true then "-" concat (decimal-positive (absolute value))
    false then decimal-positive value

### Materialize Unicode Characters without exposing encoded units.
pub character-list is fn (text : String) -> List Character
  collect (characters text)

### Construct a String from complete Unicode Characters.
pub string is fn (values : List Character) -> String
  values fold "" { text, character } text concat character
