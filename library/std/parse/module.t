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

signed-values is fn ((values : List Int, value : Int, sign : Int, active? : Boolean)) -> List Int
  values
signed-value is fn ((values : List Int, value : Int, sign : Int, active? : Boolean)) -> Int
  value
signed-sign is fn ((values : List Int, value : Int, sign : Int, active? : Boolean)) -> Int
  sign
signed-active? is fn ((values : List Int, value : Int, sign : Int, active? : Boolean)) -> Boolean
  active?

signed-digit is fn (state : (List Int, Int, Int, Boolean), digit : Nat) -> (List Int, Int, Int, Boolean)
  (signed-values state, (signed-value state) * 10 + digit, signed-sign state, true)

finish-signed-token is fn (state : (List Int, Int, Int, Boolean)) -> List Int
  signed-active? state
    true then (signed-values state) append ((signed-sign state) * (signed-value state))
    false then signed-values state

next-sign is fn (character : Character) -> Int
  character = "-"
    true then -1
    false then 1

signed-separator is fn (state : (List Int, Int, Int, Boolean), character : Character) -> (List Int, Int, Int, Boolean)
  (finish-signed-token state, 0, next-sign character, false)

signed-step is fn (state : (List Int, Int, Int, Boolean), character : Character) -> (List Int, Int, Int, Boolean)
  ascii-decimal-digit character
    Some digit then signed-digit (state, digit)
    None then signed-separator (state, character)

signed-source is fn (text : String) -> List Int
  empty-values : List Int is Empty
  final is (collect (characters text)) fold (empty-values, 0, 1, false) { state, character } signed-step (state, character)
  finish-signed-token final

### Extract signed ASCII decimal integers in source order.
pub signed-integers is fn (text : String) -> List Int
  signed-source text

unsigned-values is fn ((values : List Nat, value : Nat, active? : Boolean)) -> List Nat
  values
unsigned-value is fn ((values : List Nat, value : Nat, active? : Boolean)) -> Nat
  value
unsigned-active? is fn ((values : List Nat, value : Nat, active? : Boolean)) -> Boolean
  active?

unsigned-digit is fn (state : (List Nat, Nat, Boolean), digit : Nat) -> (List Nat, Nat, Boolean)
  (unsigned-values state, (unsigned-value state) * 10 + digit, true)

finish-unsigned-token is fn (state : (List Nat, Nat, Boolean)) -> List Nat
  unsigned-active? state
    true then (unsigned-values state) append (unsigned-value state)
    false then unsigned-values state

unsigned-separator is fn (state : (List Nat, Nat, Boolean)) -> (List Nat, Nat, Boolean)
  (finish-unsigned-token state, Nat 0, false)

unsigned-step is fn (state : (List Nat, Nat, Boolean), character : Character) -> (List Nat, Nat, Boolean)
  ascii-decimal-digit character
    Some digit then unsigned-digit (state, digit)
    None then unsigned-separator state

unsigned-source is fn (text : String) -> List Nat
  empty-values : List Nat is Empty
  zero : Nat is Nat 0
  final is (collect (characters text)) fold (empty-values, zero, false) { state, character } unsigned-step (state, character)
  finish-unsigned-token final

### Extract unsigned ASCII decimal integers in source order.
pub unsigned-integers is fn (text : String) -> List Nat
  unsigned-source text

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

line-content-with-previous is fn (text : String, index : Nat) -> Nat
  previous is first (collect (characters (text select-index ((index - 1) .. index))))
  optional-carriage-return? previous
    true then index - 1
    false then index

line-content-end is fn ((text : String, start : Nat, index : Nat)) -> Nat
  index > start
    true then line-content-with-previous (text, index)
    false then index

append-line is fn ((text : String, state : (List String, Nat, Nat))) -> (List String, Nat, Nat)
  index is line-index state
  finish is line-content-end (text, line-start state, index)
  ((line-values state) append (text select-index ((line-start state) .. finish)), index + 1, index + 1)

line-step is fn ((text : String, state : (List String, Nat, Nat), character : Character)) -> (List String, Nat, Nat)
  unicode-line-feed-character character
    true then append-line (text, state)
    false then (line-values state, line-start state, (line-index state) + 1)

finish-lines is fn (text : String, state : (List String, Nat, Nat)) -> List String
  start is line-start state
  length is line-index state
  start < length
    true then (line-values state) append (text select-index (start .. length))
    false then line-values state

lines-source is fn (text : String) -> List String
  empty-lines : List String is Empty
  final is (collect (characters text)) fold (empty-lines, Nat 0, Nat 0) { state, character } line-step (text, state, character)
  finish-lines (text, final)

append-nonempty-row is fn (rows : List List Int, line : String) -> List List Int
  values is signed-source line
  (entry-count values) > 0
    true then rows append values
    false then rows

integer-rows-source is fn (text : String) -> List List Int
  empty-rows : List List Int is Empty
  (lines-source text) fold empty-rows { rows, line } append-nonempty-row (rows, line)

### Extract every nonempty row of signed decimal integers.
pub integer-rows is fn (text : String) -> List List Int
  integer-rows-source text

maximum-nat is fn (left : Nat, right : Nat) -> Nat
  left < right
    true then right
    false then left

line-width-step is fn (width : Nat, line : String) -> Nat
  maximum-nat (width, entry-count (collect (characters line)))

number-lines is fn (lines : List String) -> List String
  count is entry-count lines
  count = 0
    true then lines
    false then lines select-index (0 .. (count - 1))

column-value is fn ((value : Int, present? : Boolean)) -> Int
  value
column-present? is fn ((value : Int, present? : Boolean)) -> Boolean
  present?

column-digit is fn (state : (Int, Boolean), digit : Nat) -> (Int, Boolean)
  ((column-value state) * 10 + digit, true)

column-optional-digit is fn (state : (Int, Boolean), digit : Optional Nat) -> (Int, Boolean)
  digit
    Some value then column-digit (state, value)
    None then state

column-character is fn (state : (Int, Boolean), character : Optional Character) -> (Int, Boolean)
  character
    Some value then column-optional-digit (state, ascii-decimal-digit value)
    None then state

column-step is fn ((column : Nat, state : (Int, Boolean), line : String)) -> (Int, Boolean)
  character is first (collect (characters (line select-index (column ..= column))))
  column-character (state, character)

present-column is fn (state : (Int, Boolean)) -> Optional Int
  column-present? state
    true then present-int (column-value state)
    false then None Int

parse-column is fn (lines : List String, column : Nat) -> Optional Int
  final is lines fold (0, false) { state, line } column-step (column, state, line)
  present-column final

append-column is fn ((lines : List String, values : List Optional Int, column : Nat)) -> List Optional Int
  values append (parse-column (lines, column))

vertical-source is fn (text : String) -> List Optional Int
  lines is lines-source text
  width : Nat is lines fold (Nat 0) { maximum, line } line-width-step (maximum, line)
  data-lines is number-lines lines
  columns is collect (0 iterate ({ column } column + 1) take-while ({ column } column < width))
  empty-values : List Optional Int is Empty
  columns fold empty-values { values, column } append-column (data-lines, values, column)

### Read fixed-width vertical decimal columns, preserving blank separators.
pub vertical-integers is fn (text : String) -> List Optional Int
  vertical-source text

append-pair-tail is fn ((pairs : List (Int, Int), left : Int, right : Int, tail : List Int)) -> List (Int, Int)
  tail
    Empty then pairs append (left, right)
    Entry (ignored, rest) then pairs

append-pair-rest is fn ((pairs : List (Int, Int), left : Int, rest : List Int)) -> List (Int, Int)
  rest
    Entry (right, tail) then append-pair-tail (pairs, left, right, tail)
    Empty then pairs

append-pair-row is fn (pairs : List (Int, Int), row : List Int) -> List (Int, Int)
  row
    Entry (left, rest) then append-pair-rest (pairs, left, rest)
    Empty then pairs

append-triple-tail is fn ((triples : List (Int, Int, Int), first-value : Int, second : Int, third : Int, tail : List Int)) -> List (Int, Int, Int)
  tail
    Empty then triples append (first-value, second, third)
    Entry (ignored, rest) then triples

append-triple-third is fn ((triples : List (Int, Int, Int), first-value : Int, second : Int, rest : List Int)) -> List (Int, Int, Int)
  rest
    Entry (third, tail) then append-triple-tail (triples, first-value, second, third, tail)
    Empty then triples

append-triple-rest is fn ((triples : List (Int, Int, Int), first-value : Int, rest : List Int)) -> List (Int, Int, Int)
  rest
    Entry (second, tail) then append-triple-third (triples, first-value, second, tail)
    Empty then triples

append-triple-row is fn (triples : List (Int, Int, Int), row : List Int) -> List (Int, Int, Int)
  row
    Entry (first-value, rest) then append-triple-rest (triples, first-value, rest)
    Empty then triples

### Extract rows containing exactly two signed decimal integers.
pub integer-pairs is fn (text : String) -> List (Int, Int)
  empty-pairs : List (Int, Int) is Empty
  (integer-rows-source text) fold empty-pairs { pairs, row } append-pair-row (pairs, row)

### Extract rows containing exactly three signed decimal integers.
pub integer-triples is fn (text : String) -> List (Int, Int, Int)
  empty-triples : List (Int, Int, Int) is Empty
  (integer-rows-source text) fold empty-triples { triples, row } append-triple-row (triples, row)

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
