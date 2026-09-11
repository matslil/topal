#!/usr/bin/env topal
use language (
  version is v0.1
)

### Revision of the regular-expression namespace.
pub revision is 1

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

regex-group-tokens is fn ((tokens : List (Int, String), stack : List String, next : String, valid? : Boolean)) -> List (Int, String)
  tokens
regex-group-stack is fn ((tokens : List (Int, String), stack : List String, next : String, valid? : Boolean)) -> List String
  stack
regex-group-next is fn ((tokens : List (Int, String), stack : List String, next : String, valid? : Boolean)) -> String
  next
regex-group-valid? is fn ((tokens : List (Int, String), stack : List String, next : String, valid? : Boolean)) -> Boolean
  valid?
regex-token-at is fn (tokens : List (Int, String), index : Nat) -> Optional (Int, String)
  first (tokens select-index (index ..= index))
regex-noncapture-colon? is fn (tokens : List (Int, String), index : Nat) -> Boolean
  regex-token-at (tokens, index + 2)
    Some colon then ((regex-token-kind colon) = 0) and ((regex-token-payload colon) = ":")
    None then false
regex-noncapture-open-tail? is fn ((tokens : List (Int, String), index : Nat, question : (Int, String))) -> Boolean
  (regex-token-kind question) = 10
    false then false
    true then regex-noncapture-colon? (tokens, index)
regex-noncapture-after-open? is fn (tokens : List (Int, String), index : Nat) -> Boolean
  regex-token-at (tokens, index + 1)
    Some question then regex-noncapture-open-tail? (tokens, index, question)
    None then false
regex-noncapture-open? is fn ((tokens : List (Int, String), index : Nat, open : (Int, String))) -> Boolean
  (regex-token-kind open) = 5
    false then false
    true then regex-noncapture-after-open? (tokens, index)
regex-noncapture-prefix? is fn (tokens : List (Int, String), index : Nat) -> Boolean
  regex-token-at (tokens, index)
    Some open then regex-noncapture-open? (tokens, index, open)
    None then false
regex-noncapture-skip-earlier-valid? is fn (tokens : List (Int, String), index : Nat) -> Boolean
  converted : Nat is Nat (index - 2)
  regex-noncapture-prefix? (tokens, converted)
regex-noncapture-skip-earlier? is fn (tokens : List (Int, String), index : Nat) -> Boolean
  index > 1
    false then false
    true then regex-noncapture-skip-earlier-valid? (tokens, index)
regex-noncapture-skip-previous? is fn (tokens : List (Int, String), index : Nat) -> Boolean
  regex-noncapture-prefix? (tokens, index - 1)
    true then true
    false then regex-noncapture-skip-earlier? (tokens, index)
regex-noncapture-skip? is fn (tokens : List (Int, String), index : Nat) -> Boolean
  index > 0
    false then false
    true then regex-noncapture-skip-previous? (tokens, index)
regex-noncapture-token-output is fn ((tokens : List (Int, String), output : List (Int, String), index : Nat, token : (Int, String))) -> List (Int, String)
  regex-noncapture-prefix? (tokens, index)
    true then output append (16, "")
    false then output append token
regex-noncapture-append is fn ((tokens : List (Int, String), output : List (Int, String), index : Nat)) -> List (Int, String)
  regex-token-at (tokens, index)
    Some token then regex-noncapture-token-output (tokens, output, index, token)
    None then output
regex-noncapture-step is fn ((tokens : List (Int, String), output : List (Int, String), index : Int)) -> List (Int, String)
  converted : Nat is Nat index
  regex-noncapture-skip? (tokens, converted)
    true then output
    false then regex-noncapture-append (tokens, output, converted)
regex-prepare-output is fn ((output : List (Int, String), index : Nat)) -> List (Int, String)
  output
regex-prepare-index is fn ((output : List (Int, String), index : Nat)) -> Nat
  index
regex-prepare-step is fn ((tokens : List (Int, String), state : (List (Int, String), Nat), ignored : (Int, String))) -> (List (Int, String), Nat)
  index is regex-prepare-index state
  output is regex-noncapture-step (tokens, regex-prepare-output state, index)
  (output, index + 1)
regex-prepare-group-tokens is fn (tokens : List (Int, String)) -> List (Int, String)
  empty : List (Int, String) is Empty
  final is tokens fold (empty, Nat 0) { state, ignored } regex-prepare-step (tokens, state, ignored)
  regex-prepare-output final
regex-group-close-present is fn (state : (List (Int, String), List String, String, Boolean), marker : String) -> (List (Int, String), List String, String, Boolean)
  stack is regex-group-stack state
  remaining is stack select-index (0 .. ((entry-count stack) - 1))
  ((regex-group-tokens state) append (6, marker), remaining, regex-group-next state, regex-group-valid? state)
regex-group-close is fn (state : (List (Int, String), List String, String, Boolean)) -> (List (Int, String), List String, String, Boolean)
  stack is regex-group-stack state
  (entry-count stack) = 0
    true then (regex-group-tokens state, stack, regex-group-next state, false)
    false then first (stack select-index (((entry-count stack) - 1) ..= ((entry-count stack) - 1)))
      Some marker then regex-group-close-present (state, marker)
      None then (regex-group-tokens state, stack, regex-group-next state, false)
regex-group-token is fn (state : (List (Int, String), List String, String, Boolean), token : (Int, String)) -> (List (Int, String), List String, String, Boolean)
  regex-token-kind token
    = 5 then ((regex-group-tokens state) append (5, regex-group-next state), (regex-group-stack state) append (regex-group-next state), (regex-group-next state) concat "g", regex-group-valid? state)
    = 16 then ((regex-group-tokens state) append (5, ""), (regex-group-stack state) append "", regex-group-next state, regex-group-valid? state)
    = 6 then regex-group-close state
    otherwise ((regex-group-tokens state) append token, regex-group-stack state, regex-group-next state, regex-group-valid? state)
regex-annotate-groups-valid is fn (tokens : List (Int, String)) -> (List (Int, String), Boolean)
  empty-tokens : List (Int, String) is Empty
  empty-stack : List String is Empty
  final is (regex-prepare-group-tokens tokens) fold (empty-tokens, empty-stack, "g", true) { state, token } regex-group-token (state, token)
  (regex-group-tokens final, (regex-group-valid? final) and ((entry-count (regex-group-stack final)) = 0))
regex-annotate-groups is fn (tokenized : (List (Int, String), Boolean)) -> (List (Int, String), Boolean)
  tokens is fn ((tokens : List (Int, String), valid? : Boolean)) -> List (Int, String)
    tokens
  valid is fn ((tokens : List (Int, String), valid? : Boolean)) -> Boolean
    valid?
  valid tokenized
    true then regex-annotate-groups-valid (tokens tokenized)
    false then tokenized

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

regex-postfix-output is fn ((output : List (Int, String), groups : List (Nat, Nat, String, Nat), alternatives : Nat, atoms : Nat, valid? : Boolean, quantified? : Boolean)) -> List (Int, String)
  output
regex-postfix-groups is fn ((output : List (Int, String), groups : List (Nat, Nat, String, Nat), alternatives : Nat, atoms : Nat, valid? : Boolean, quantified? : Boolean)) -> List (Nat, Nat, String, Nat)
  groups
regex-postfix-alternatives is fn ((output : List (Int, String), groups : List (Nat, Nat, String, Nat), alternatives : Nat, atoms : Nat, valid? : Boolean, quantified? : Boolean)) -> Nat
  alternatives
regex-postfix-atoms is fn ((output : List (Int, String), groups : List (Nat, Nat, String, Nat), alternatives : Nat, atoms : Nat, valid? : Boolean, quantified? : Boolean)) -> Nat
  atoms
regex-postfix-valid? is fn ((output : List (Int, String), groups : List (Nat, Nat, String, Nat), alternatives : Nat, atoms : Nat, valid? : Boolean, quantified? : Boolean)) -> Boolean
  valid?
regex-postfix-quantified? is fn ((output : List (Int, String), groups : List (Nat, Nat, String, Nat), alternatives : Nat, atoms : Nat, valid? : Boolean, quantified? : Boolean)) -> Boolean
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

regex-postfix-atom is fn (state : (List (Int, String), List (Nat, Nat, String, Nat), Nat, Nat, Boolean, Boolean), token : (Int, String)) -> (List (Int, String), List (Nat, Nat, String, Nat), Nat, Nat, Boolean, Boolean)
  atoms is regex-postfix-atoms state
  collapsed is regex-postfix-precollapse (regex-postfix-output state, atoms)
  next-atoms is regex-postfix-next-atoms atoms
  (collapsed append token, regex-postfix-groups state, regex-postfix-alternatives state, next-atoms, regex-postfix-valid? state, false)
regex-postfix-open is fn (state : (List (Int, String), List (Nat, Nat, String, Nat), Nat, Nat, Boolean, Boolean), token : (Int, String)) -> (List (Int, String), List (Nat, Nat, String, Nat), Nat, Nat, Boolean, Boolean)
  atoms is regex-postfix-atoms state
  collapsed is regex-postfix-precollapse (regex-postfix-output state, atoms)
  parent-atoms is regex-postfix-parent-atoms atoms
  (collapsed, (regex-postfix-groups state) append (regex-postfix-alternatives state, parent-atoms, regex-token-payload token, entry-count collapsed), Nat 0, Nat 0, regex-postfix-valid? state, false)
regex-postfix-alt is fn (state : (List (Int, String), List (Nat, Nat, String, Nat), Nat, Nat, Boolean, Boolean)) -> (List (Int, String), List (Nat, Nat, String, Nat), Nat, Nat, Boolean, Boolean)
  (regex-postfix-atoms state) = 0
    true then (regex-postfix-output state, regex-postfix-groups state, regex-postfix-alternatives state, regex-postfix-atoms state, false, false)
    false then (regex-collapse-concats (regex-postfix-output state, regex-postfix-atoms state), regex-postfix-groups state, (regex-postfix-alternatives state) + 1, Nat 0, regex-postfix-valid? state, false)
regex-postfix-capture is fn ((prefix : List (Int, String), completed : List (Int, String), marker : String)) -> List (Int, String)
  empty? marker
    true then prefix concat completed
    false then prefix concat ((one (14, "s" concat marker)) concat completed append (13, "")) append (15, "e" concat marker) append (13, "")
regex-postfix-close-present is fn (state : (List (Int, String), List (Nat, Nat, String, Nat), Nat, Nat, Boolean, Boolean), parent : (Nat, Nat, String, Nat)) -> (List (Int, String), List (Nat, Nat, String, Nat), Nat, Nat, Boolean, Boolean)
  parent-alternatives is fn ((alternatives : Nat, atoms : Nat, marker : String, boundary : Nat)) -> Nat
    alternatives
  parent-atoms is fn ((alternatives : Nat, atoms : Nat, marker : String, boundary : Nat)) -> Nat
    atoms
  parent-marker is fn ((alternatives : Nat, atoms : Nat, marker : String, boundary : Nat)) -> String
    marker
  parent-boundary is fn ((alternatives : Nat, atoms : Nat, marker : String, boundary : Nat)) -> Nat
    boundary
  output is regex-postfix-output state
  prefix is output select-index (0 .. (parent-boundary parent))
  inner is output select-index ((parent-boundary parent) .. (entry-count output))
  concatenated is regex-collapse-concats (inner, regex-postfix-atoms state)
  completed is regex-collapse-alternatives (concatenated, regex-postfix-alternatives state)
  captured is regex-postfix-capture (prefix, completed, parent-marker parent)
  groups is regex-postfix-groups state
  remaining is groups select-index (0 .. ((entry-count groups) - 1))
  (captured, remaining, parent-alternatives parent, (parent-atoms parent) + 1, regex-postfix-valid? state, false)
regex-postfix-close is fn (state : (List (Int, String), List (Nat, Nat, String, Nat), Nat, Nat, Boolean, Boolean)) -> (List (Int, String), List (Nat, Nat, String, Nat), Nat, Nat, Boolean, Boolean)
  invalid? is ((regex-postfix-atoms state) = 0) or ((entry-count (regex-postfix-groups state)) = 0)
  invalid?
    true then (regex-postfix-output state, regex-postfix-groups state, regex-postfix-alternatives state, regex-postfix-atoms state, false, false)
    false then first ((regex-postfix-groups state) select-index (((entry-count (regex-postfix-groups state)) - 1) ..= ((entry-count (regex-postfix-groups state)) - 1)))
      Some parent then regex-postfix-close-present (state, parent)
      None then (regex-postfix-output state, regex-postfix-groups state, regex-postfix-alternatives state, regex-postfix-atoms state, false, false)
regex-postfix-unary is fn (state : (List (Int, String), List (Nat, Nat, String, Nat), Nat, Nat, Boolean, Boolean), token : (Int, String)) -> (List (Int, String), List (Nat, Nat, String, Nat), Nat, Nat, Boolean, Boolean)
  valid? is ((regex-postfix-atoms state) > 0) and (not (regex-postfix-quantified? state))
  valid?
    true then ((regex-postfix-output state) append token, regex-postfix-groups state, regex-postfix-alternatives state, regex-postfix-atoms state, regex-postfix-valid? state, true)
    false then (regex-postfix-output state, regex-postfix-groups state, regex-postfix-alternatives state, regex-postfix-atoms state, false, true)
regex-postfix-repeat-valid is fn (state : (List (Int, String), List (Nat, Nat, String, Nat), Nat, Nat, Boolean, Boolean), bounds : (Nat, Int, Boolean)) -> (List (Int, String), List (Nat, Nat, String, Nat), Nat, Nat, Boolean, Boolean)
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
regex-active-number is fn (accepted : Boolean, value : Int) -> Int
  accepted
    true then value
    false then -1
regex-postfix-token is fn (state : (List (Int, String), List (Nat, Nat, String, Nat), Nat, Nat, Boolean, Boolean), token : (Int, String)) -> (List (Int, String), List (Nat, Nat, String, Nat), Nat, Nat, Boolean, Boolean)
  kind is regex-active-number (regex-postfix-valid? state, regex-token-kind token)
  kind
    < 0 then state
    <= 4 then regex-postfix-atom (state, token)
    = 5 then regex-postfix-open (state, token)
    = 6 then regex-postfix-close state
    = 7 then regex-postfix-alt state
    = 8 then regex-postfix-unary (state, token)
    = 9 then regex-postfix-unary (state, token)
    = 10 then regex-postfix-unary (state, token)
    = 11 then regex-postfix-repeat-valid (state, regex-repeat-bounds (regex-token-payload token))
    otherwise state
regex-postfix-finish is fn (state : (List (Int, String), List (Nat, Nat, String, Nat), Nat, Nat, Boolean, Boolean)) -> (List (Int, String), Boolean)
  groups-empty? is (entry-count (regex-postfix-groups state)) = 0
  valid? is (regex-postfix-valid? state) and groups-empty?
  atoms is regex-active-number (valid?, regex-postfix-atoms state)
  atoms
    < 0 then (regex-postfix-output state, false)
    = 0 then ((regex-postfix-output state) append (12, ""), (regex-postfix-alternatives state) = 0)
    otherwise (regex-collapse-alternatives (regex-collapse-concats (regex-postfix-output state, regex-postfix-atoms state), regex-postfix-alternatives state), true)
regex-to-postfix-valid is fn (tokens : List (Int, String)) -> (List (Int, String), Boolean)
  empty-output : List (Int, String) is Empty
  empty-groups : List (Nat, Nat, String, Nat) is Empty
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
    = 14 then 8
    = 15 then 8
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
    = 14 then regex-compile-atom (state, token)
    = 15 then regex-compile-atom (state, token)
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
  end-index is entry-count (regex-compile-instructions state)
  match-index is end-index + 1
  start-index is end-index + 2
  patched is regex-patch (regex-compile-instructions state, (regex-fragment-outs fragment, end-index))
  completed is patched append (8, "e", match-index, -1) append (0, "", -1, -1) append (8, "s", regex-fragment-start fragment, -1)
  (completed, start-index, regex-compile-valid? state)
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
regex-consuming-match? is fn (instruction : (Int, String, Int, Int), candidate : Character) -> Boolean
  regex-instruction-kind instruction
    = 1 then (regex-instruction-payload instruction) = candidate
    = 2 then not (unicode-line-feed-character candidate)
    = 3 then regex-class-matches? (regex-instruction-payload instruction, candidate)
    otherwise false
regex-thread-pc is fn ((pc : Int, slots : List Int)) -> Int
  pc
regex-thread-slots is fn ((pc : Int, slots : List Int)) -> List Int
  slots
regex-thread-pc-match is fn ((pc : Int, found : Boolean, thread : (Int, List Int))) -> Boolean
  found or ((regex-thread-pc thread) = pc)
regex-thread-pc-present? is fn (threads : List (Int, List Int), pc : Int) -> Boolean
  threads fold false { found, thread } regex-thread-pc-match (pc, found, thread)
regex-thread-add is fn (threads : List (Int, List Int), thread : (Int, List Int)) -> List (Int, List Int)
  valid? is (regex-thread-pc thread) >= 0
  new? is not (regex-thread-pc-present? (threads, regex-thread-pc thread))
  valid? and new?
    true then threads append thread
    false then threads
regex-slot-output is fn ((output : List Int, index : Nat)) -> List Int
  output
regex-slot-index is fn ((output : List Int, index : Nat)) -> Nat
  index
first-int-or-negative is fn (value : Optional Int) -> Int
  value
    Some present then present
    None then -1
regex-slot-replacement is fn ((selected? : Boolean, value : Int, current : Int)) -> Int
  selected?
    true then value
    false then current
regex-slot-update-step is fn ((slots : List Int, target : Nat, value : Int, state : (List Int, Nat), ignored : Int)) -> (List Int, Nat)
  index is regex-slot-index state
  current is first-int-or-negative (first (slots select-index (index ..= index)))
  replacement is regex-slot-replacement (index = target, value, current)
  ((regex-slot-output state) append replacement, index + 1)
regex-slot-update is fn ((slots : List Int, target : Nat, value : Int)) -> List Int
  empty : List Int is Empty
  final is slots fold (empty, Nat 0) { state, ignored } regex-slot-update-step (slots, target, value, state, ignored)
  regex-slot-output final
regex-save-start? is fn (payload : String) -> Boolean
  first (unicode-scalar-characters payload)
    Some prefix then prefix = "s"
    None then false
regex-save-slot is fn (payload : String) -> Nat
  group is (entry-count (unicode-scalar-characters payload)) - 1
  regex-save-start? payload
    true then group * 2
    false then group * 2 + 1
regex-capture-closure-assert is fn ((enabled? : Boolean, threads : List (Int, List Int), thread : (Int, List Int), instruction : (Int, String, Int, Int))) -> List (Int, List Int)
  enabled?
    true then regex-thread-add (threads, (regex-instruction-out instruction, regex-thread-slots thread))
    false then threads
regex-capture-save is fn ((position : Nat, threads : List (Int, List Int), thread : (Int, List Int), instruction : (Int, String, Int, Int))) -> List (Int, List Int)
  slot is regex-save-slot (regex-instruction-payload instruction)
  slots is regex-slot-update (regex-thread-slots thread, slot, position)
  regex-thread-add (threads, (regex-instruction-out instruction, slots))
regex-capture-closure-instruction is fn ((position : Nat, length : Nat, threads : List (Int, List Int), thread : (Int, List Int), instruction : (Int, String, Int, Int))) -> List (Int, List Int)
  regex-instruction-kind instruction
    = 4 then regex-capture-closure-assert (position = 0, threads, thread, instruction)
    = 5 then regex-capture-closure-assert (position = length, threads, thread, instruction)
    = 6 then regex-thread-add (threads, (regex-instruction-out instruction, regex-thread-slots thread))
    = 7 then regex-thread-add (regex-thread-add (threads, (regex-instruction-out instruction, regex-thread-slots thread)), (regex-instruction-out-two instruction, regex-thread-slots thread))
    = 8 then regex-capture-save (position, threads, thread, instruction)
    otherwise threads
regex-capture-closure-step is fn ((instructions : List (Int, String, Int, Int), position : Nat, length : Nat, threads : List (Int, List Int), thread : (Int, List Int))) -> List (Int, List Int)
  regex-instruction-at (instructions, regex-thread-pc thread)
    Some instruction then regex-capture-closure-instruction (position, length, threads, thread, instruction)
    None then threads
regex-capture-closure-pass is fn ((instructions : List (Int, String, Int, Int), position : Nat, length : Nat, threads : List (Int, List Int))) -> List (Int, List Int)
  threads fold threads { expanded, thread } regex-capture-closure-step (instructions, position, length, expanded, thread)
regex-capture-closure-repeat is fn ((instructions : List (Int, String, Int, Int), position : Nat, length : Nat, threads : List (Int, List Int), ignored : Int)) -> List (Int, List Int)
  regex-capture-closure-pass (instructions, position, length, threads)
regex-capture-closure is fn ((instructions : List (Int, String, Int, Int), position : Nat, length : Nat, threads : List (Int, List Int))) -> List (Int, List Int)
  (regex-counts (entry-count instructions)) fold threads { expanded, ignored } regex-capture-closure-repeat (instructions, position, length, expanded, ignored)
regex-capture-consume-present is fn ((candidate : Character, threads : List (Int, List Int), thread : (Int, List Int), instruction : (Int, String, Int, Int))) -> List (Int, List Int)
  regex-consuming-match? (instruction, candidate)
    true then regex-thread-add (threads, (regex-instruction-out instruction, regex-thread-slots thread))
    false then threads
regex-capture-consume-step is fn ((instructions : List (Int, String, Int, Int), candidate : Character, threads : List (Int, List Int), thread : (Int, List Int))) -> List (Int, List Int)
  regex-instruction-at (instructions, regex-thread-pc thread)
    Some instruction then regex-capture-consume-present (candidate, threads, thread, instruction)
    None then threads
regex-capture-consume is fn ((instructions : List (Int, String, Int, Int), active : List (Int, List Int), candidate : Character)) -> List (Int, List Int)
  empty : List (Int, List Int) is Empty
  active fold empty { threads, thread } regex-capture-consume-step (instructions, candidate, threads, thread)
regex-capture-result-present? is fn ((present? : Boolean, slots : List Int)) -> Boolean
  present?
regex-capture-result-slots is fn ((present? : Boolean, slots : List Int)) -> List Int
  slots
regex-no-capture-result is fn () -> (Boolean, List Int)
  empty : List Int is Empty
  (false, empty)
regex-capture-match-instruction is fn (thread : (Int, List Int), instruction : (Int, String, Int, Int)) -> (Boolean, List Int)
  (regex-instruction-kind instruction) = 0
    true then (true, regex-thread-slots thread)
    false then regex-no-capture-result ()
regex-capture-match-none is fn (instructions : List (Int, String, Int, Int), thread : (Int, List Int)) -> (Boolean, List Int)
  regex-instruction-at (instructions, regex-thread-pc thread)
    Some instruction then regex-capture-match-instruction (thread, instruction)
    None then regex-no-capture-result ()
regex-capture-match-step is fn ((instructions : List (Int, String, Int, Int), found : (Boolean, List Int), thread : (Int, List Int))) -> (Boolean, List Int)
  regex-capture-result-present? found
    true then found
    false then regex-capture-match-none (instructions, thread)
regex-capture-match is fn (instructions : List (Int, String, Int, Int), threads : List (Int, List Int)) -> (Boolean, List Int)
  threads fold (regex-no-capture-result ()) { found, thread } regex-capture-match-step (instructions, found, thread)
regex-capture-slot-at is fn (slots : List Int, index : Nat) -> Int
  first-int-or-negative (first (slots select-index (index ..= index)))
regex-capture-longer is fn (current : List Int, candidate : List Int) -> List Int
  (regex-capture-slot-at (candidate, 1)) >= (regex-capture-slot-at (current, 1))
    true then candidate
    false then current
regex-capture-better-present is fn (current : List Int, candidate : List Int) -> List Int
  current-start is regex-capture-slot-at (current, 0)
  candidate-start is regex-capture-slot-at (candidate, 0)
  candidate-start < current-start
    true then candidate
    false then candidate-start = current-start
      true then regex-capture-longer (current, candidate)
      false then current
regex-capture-better-candidate is fn (current : (Boolean, List Int), candidate-slots : List Int) -> (Boolean, List Int)
  regex-capture-result-present? current
    false then (true, candidate-slots)
    true then (true, regex-capture-better-present (regex-capture-result-slots current, candidate-slots))
regex-capture-better is fn (current : (Boolean, List Int), candidate : (Boolean, List Int)) -> (Boolean, List Int)
  regex-capture-result-present? candidate
    false then current
    true then regex-capture-better-candidate (current, regex-capture-result-slots candidate)
regex-capture-run-threads is fn ((threads : List (Int, List Int), position : Nat, best : (Boolean, List Int))) -> List (Int, List Int)
  threads
regex-capture-run-position is fn ((threads : List (Int, List Int), position : Nat, best : (Boolean, List Int))) -> Nat
  position
regex-capture-run-best is fn ((threads : List (Int, List Int), position : Nat, best : (Boolean, List Int))) -> (Boolean, List Int)
  best
regex-capture-run-step is fn ((instructions : List (Int, String, Int, Int), start : Int, length : Nat, initial-slots : List Int, state : (List (Int, List Int), Nat, (Boolean, List Int)), candidate : Character)) -> (List (Int, List Int), Nat, (Boolean, List Int))
  position is regex-capture-run-position state
  seed is (start, initial-slots)
  seeded is regex-thread-add (regex-capture-run-threads state, seed)
  closed is regex-capture-closure (instructions, position, length, seeded)
  before is regex-capture-better (regex-capture-run-best state, regex-capture-match (instructions, closed))
  consumed is regex-capture-consume (instructions, closed, candidate)
  next is regex-capture-closure (instructions, position + 1, length, consumed)
  after is regex-capture-better (before, regex-capture-match (instructions, next))
  (next, position + 1, after)
regex-negative-slot is fn (slots : List Int, ignored : Int) -> List Int
  slots append -1
regex-initial-slots is fn (group-count : Nat) -> List Int
  empty : List Int is Empty
  (regex-counts ((group-count + 1) * 2)) fold empty { slots, ignored } regex-negative-slot (slots, ignored)
regex-group-count-step is fn (count : Nat, token : (Int, String)) -> Nat
  (regex-token-kind token) = 5
    true then count + 1
    false then count
regex-group-count is fn (tokens : List (Int, String)) -> Nat
  tokens fold (Nat 0) { count, token } regex-group-count-step (count, token)
regex-capture-run is fn ((text : String, compiled : (List (Int, String, Int, Int), Int, Boolean), group-count : Nat)) -> (Boolean, List Int)
  instructions is fn ((instructions : List (Int, String, Int, Int), start : Int, valid? : Boolean)) -> List (Int, String, Int, Int)
    instructions
  start is fn ((instructions : List (Int, String, Int, Int), start : Int, valid? : Boolean)) -> Int
    start
  source is unicode-scalar-characters text
  empty : List (Int, List Int) is Empty
  slots is regex-initial-slots group-count
  final is source fold (empty, Nat 0, regex-no-capture-result ()) { state, candidate } regex-capture-run-step (instructions compiled, start compiled, entry-count source, slots, state, candidate)
  ending-seeded is regex-thread-add (regex-capture-run-threads final, (start compiled, slots))
  ending is regex-capture-closure (instructions compiled, entry-count source, entry-count source, ending-seeded)
  regex-capture-better (regex-capture-run-best final, regex-capture-match (instructions compiled, ending))
regex-scalar-concat is fn (text : String, character : Character) -> String
  text concat character
regex-captured-value-valid is fn ((source : List Character, start : Int, finish : Int)) -> (Boolean, String)
  converted-start : Nat is Nat start
  converted-finish : Nat is Nat finish
  selected is source select-index (converted-start .. converted-finish)
  (true, selected fold "" { text, character } regex-scalar-concat (text, character))
regex-captured-value is fn ((source : List Character, slots : List Int, group : Int)) -> (Boolean, String)
  slot : Nat is Nat (group * 2)
  start is regex-capture-slot-at (slots, slot)
  finish is regex-capture-slot-at (slots, slot + 1)
  (start < 0) or (finish < start)
    true then (false, "")
    false then regex-captured-value-valid (source, start, finish)
regex-captured-value-step is fn ((source : List Character, slots : List Int, captures : List (Boolean, String), group : Int)) -> List (Boolean, String)
  captures append (regex-captured-value (source, slots, group))
regex-captured-values is fn ((source : List Character, group-count : Nat, slots : List Int)) -> List (Boolean, String)
  empty : List (Boolean, String) is Empty
  (regex-counts (group-count + 1)) fold empty { captures, group } regex-captured-value-step (source, slots, captures, group)
regex-no-captures is fn () -> (Boolean, List (Boolean, String))
  empty : List (Boolean, String) is Empty
  (false, empty)
regex-captures-present is fn ((source : List Character, group-count : Nat, result : (Boolean, List Int))) -> (Boolean, List (Boolean, String))
  regex-capture-result-present? result
    true then (true, regex-captured-values (source, group-count, regex-capture-result-slots result))
    false then regex-no-captures ()

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
regex-tokenized-tokens is fn ((tokens : List (Int, String), valid? : Boolean)) -> List (Int, String)
  tokens
RegexValid is Boolean constraint { valid } valid

### Test whether a valid regular expression matches a substring; reject invalid patterns.
pub contains? is fn (text : String, pattern : String) -> Boolean
  tokenized is regex-tokenize pattern
  compiled is regex-compile-postfix (regex-to-postfix (regex-annotate-groups tokenized))
  valid? is (regex-compiled-valid? compiled) and (regex-pattern-classes-valid? tokenized)
  checked : RegexValid is RegexValid valid?
  _ is checked
  group-count is regex-group-count (regex-prepare-group-tokens (regex-tokenized-tokens tokenized))
  regex-capture-result-present? (regex-capture-run (text, compiled, group-count))

### Return the leftmost-longest match and its groups; reject invalid patterns, and use entry zero for the whole match.
pub captures is fn (text : String, pattern : String) -> (Boolean, List (Boolean, String))
  tokenized is regex-tokenize pattern
  annotated is regex-annotate-groups tokenized
  compiled is regex-compile-postfix (regex-to-postfix annotated)
  valid? is (regex-compiled-valid? compiled) and (regex-pattern-classes-valid? tokenized)
  checked : RegexValid is RegexValid valid?
  _ is checked
  group-count is regex-group-count (regex-prepare-group-tokens (regex-tokenized-tokens tokenized))
  source is unicode-scalar-characters text
  regex-captures-present (source, group-count, regex-capture-run (text, compiled, group-count))
