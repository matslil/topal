#!/usr/bin/env topal
use language (version is v0.1)

### Revision of the finite additive-machine planning namespace.
pub revision is 1

line-values is fn ((values : List String, start : Nat, index : Nat)) -> List String
  values
line-start is fn ((values : List String, start : Nat, index : Nat)) -> Nat
  start
line-index is fn ((values : List String, start : Nat, index : Nat)) -> Nat
  index
append-line is fn ((text : String, state : (List String, Nat, Nat))) -> (List String, Nat, Nat)
  index is line-index state
  ((line-values state) append (text select-index ((line-start state) .. index)), index + 1, index + 1)
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
manual-lines is fn (text : String) -> List String
  empty-lines : List String is Empty
  final is (collect (characters text)) fold (empty-lines, Nat 0, Nat 0) { state, character } line-step (text, state, character)
  finish-lines (text, final)

indicator-values is fn ((values : List Nat, inside? : Boolean)) -> List Nat
  values
indicator-inside? is fn ((values : List Nat, inside? : Boolean)) -> Boolean
  inside?
indicator-content is fn (state : (List Nat, Boolean), character : Character) -> (List Nat, Boolean)
  character
    = "#" then ((indicator-values state) append (Nat 1), true)
    = "." then ((indicator-values state) append (Nat 0), true)
    otherwise state
indicator-nonsymbol is fn (state : (List Nat, Boolean), character : Character) -> (List Nat, Boolean)
  indicator-inside? state
    true then indicator-content (state, character)
    false then state
indicator-symbol is fn (state : (List Nat, Boolean), character : Character) -> (List Nat, Boolean)
  character
    = "[" then (indicator-values state, true)
    = "]" then (indicator-values state, false)
    otherwise indicator-nonsymbol (state, character)
indicator-target is fn (line : String) -> List Nat
  empty-values : List Nat is Empty
  final is (collect (characters line)) fold (empty-values, false) { state, character } indicator-symbol (state, character)
  indicator-values final

button-groups is fn ((buttons : List List Nat, current : List Nat, value : Nat, digit? : Boolean, inside? : Boolean)) -> List List Nat
  buttons
button-current is fn ((buttons : List List Nat, current : List Nat, value : Nat, digit? : Boolean, inside? : Boolean)) -> List Nat
  current
button-value is fn ((buttons : List List Nat, current : List Nat, value : Nat, digit? : Boolean, inside? : Boolean)) -> Nat
  value
button-digit? is fn ((buttons : List List Nat, current : List Nat, value : Nat, digit? : Boolean, inside? : Boolean)) -> Boolean
  digit?
button-inside? is fn ((buttons : List List Nat, current : List Nat, value : Nat, digit? : Boolean, inside? : Boolean)) -> Boolean
  inside?
finish-button-number is fn (state : (List List Nat, List Nat, Nat, Boolean, Boolean)) -> (List List Nat, List Nat, Nat, Boolean, Boolean)
  button-digit? state
    true then (button-groups state, (button-current state) append (button-value state), Nat 0, false, button-inside? state)
    false then state
button-open is fn (state : (List List Nat, List Nat, Nat, Boolean, Boolean)) -> (List List Nat, List Nat, Nat, Boolean, Boolean)
  empty-current : List Nat is Empty
  (button-groups state, empty-current, Nat 0, false, true)
button-close-finished is fn (state : (List List Nat, List Nat, Nat, Boolean, Boolean)) -> (List List Nat, List Nat, Nat, Boolean, Boolean)
  ((button-groups state) append (button-current state), button-current state, Nat 0, false, false)
button-close is fn (state : (List List Nat, List Nat, Nat, Boolean, Boolean)) -> (List List Nat, List Nat, Nat, Boolean, Boolean)
  button-close-finished (finish-button-number state)
button-symbol is fn (state : (List List Nat, List Nat, Nat, Boolean, Boolean), character : Character) -> (List List Nat, List Nat, Nat, Boolean, Boolean)
  character
    = "(" then button-open state
    = "," then finish-button-number state
    = ")" then button-close state
    otherwise state
button-decimal is fn (state : (List List Nat, List Nat, Nat, Boolean, Boolean), digit : Nat) -> (List List Nat, List Nat, Nat, Boolean, Boolean)
  button-inside? state
    true then (button-groups state, button-current state, (button-value state) * 10 + digit, true, true)
    false then state
button-step is fn (state : (List List Nat, List Nat, Nat, Boolean, Boolean), character : Character) -> (List List Nat, List Nat, Nat, Boolean, Boolean)
  ascii-decimal-digit character
    Some digit then button-decimal (state, digit)
    None then button-symbol (state, character)
machine-buttons is fn (line : String) -> List List Nat
  empty-buttons : List List Nat is Empty
  empty-current : List Nat is Empty
  final is (collect (characters line)) fold (empty-buttons, empty-current, Nat 0, false, false) { state, character } button-step (state, character)
  button-groups final

append-selection is fn ((button : List Nat, added : List List List Nat, selection : List List Nat)) -> List List List Nat
  added append (selection append button)
extend-selections is fn (selections : List List List Nat, button : List Nat) -> List List List Nat
  empty-added : List List List Nat is Empty
  added is selections fold empty-added { values, selection } append-selection (button, values, selection)
  selections concat added
button-selections is fn (buttons : List List Nat) -> List List List Nat
  empty-selection : List List Nat is Empty
  selections : List List List Nat is one empty-selection
  buttons fold selections { values, button } extend-selections (values, button)

toggle-values is fn ((values : List Nat, position : Int)) -> List Nat
  values
toggle-position is fn ((values : List Nat, position : Int)) -> Int
  position
toggle-value is fn (value : Nat) -> Nat
  value = 0
    true then 1
    false then 0
toggle-step is fn ((index : Nat, state : (List Nat, Int), value : Nat)) -> (List Nat, Int)
  position is toggle-position state
  position = index
    true then ((toggle-values state) append (toggle-value value), position + 1)
    false then ((toggle-values state) append value, position + 1)
toggle-index is fn (values : List Nat, index : Nat) -> List Nat
  empty-values : List Nat is Empty
  final is values fold (empty-values, 0) { state, value } toggle-step (index, state, value)
  toggle-values final
apply-button is fn (values : List Nat, button : List Nat) -> List Nat
  button fold values { state, index } toggle-index (state, index)
apply-selection is fn (target : List Nat, selection : List List Nat) -> List Nat
  empty-values : List Nat is Empty
  initial is target fold empty-values { values, ignored } values append (Nat 0)
  selection fold initial { values, button } apply-button (values, button)
nat-list-equal? is fn (left : List Nat, right : List Nat) -> Boolean
  ((entry-count left) = (entry-count right)) and (left contains-sequence right)

minimum-value is fn ((value : Int, found? : Boolean)) -> Int
  value
minimum-found? is fn ((value : Int, found? : Boolean)) -> Boolean
  found?
minimum-selection-step is fn ((target : List Nat, state : (Int, Boolean), selection : List List Nat)) -> (Int, Boolean)
  matches? is nat-list-equal? (apply-selection (target, selection), target)
  count is entry-count selection
  better? is (not (minimum-found? state)) or (count < (minimum-value state))
  matches? and better?
    true then (count, true)
    false then state
minimum-indicator-line is fn (line : String) -> Int
  target is indicator-target line
  selections is button-selections (machine-buttons line)
  final is selections fold (0, false) { state, selection } minimum-selection-step (target, state, selection)
  minimum-value final
minimum-indicator-total-step is fn (total : Int, line : String) -> Int
  total + (minimum-indicator-line line)

### Sum minimum presses for binary indicators described by bracket targets and indexed buttons.
pub minimum-indicator-presses is fn (manual : String) -> Int
  (manual-lines manual) fold 0 { total, line } minimum-indicator-total-step (total, line)

counter-values is fn ((values : List Nat, value : Nat, digit? : Boolean, inside? : Boolean)) -> List Nat
  values
counter-value is fn ((values : List Nat, value : Nat, digit? : Boolean, inside? : Boolean)) -> Nat
  value
counter-digit? is fn ((values : List Nat, value : Nat, digit? : Boolean, inside? : Boolean)) -> Boolean
  digit?
counter-inside? is fn ((values : List Nat, value : Nat, digit? : Boolean, inside? : Boolean)) -> Boolean
  inside?
finish-counter-number is fn (state : (List Nat, Nat, Boolean, Boolean)) -> (List Nat, Nat, Boolean, Boolean)
  counter-digit? state
    true then ((counter-values state) append (counter-value state), Nat 0, false, counter-inside? state)
    false then state
counter-open is fn (state : (List Nat, Nat, Boolean, Boolean)) -> (List Nat, Nat, Boolean, Boolean)
  empty-values : List Nat is Empty
  (empty-values, Nat 0, false, true)
counter-close is fn (state : (List Nat, Nat, Boolean, Boolean)) -> (List Nat, Nat, Boolean, Boolean)
  finished is finish-counter-number state
  (counter-values finished, Nat 0, false, false)
counter-symbol is fn (state : (List Nat, Nat, Boolean, Boolean), character : Character) -> (List Nat, Nat, Boolean, Boolean)
  character
    = "{" then counter-open state
    = "," then finish-counter-number state
    = "}" then counter-close state
    otherwise state
counter-decimal is fn (state : (List Nat, Nat, Boolean, Boolean), digit : Nat) -> (List Nat, Nat, Boolean, Boolean)
  counter-inside? state
    true then (counter-values state, (counter-value state) * 10 + digit, true, true)
    false then state
counter-step is fn (state : (List Nat, Nat, Boolean, Boolean), character : Character) -> (List Nat, Nat, Boolean, Boolean)
  ascii-decimal-digit character
    Some digit then counter-decimal (state, digit)
    None then counter-symbol (state, character)
counter-target is fn (line : String) -> List Nat
  empty-values : List Nat is Empty
  final is (collect (characters line)) fold (empty-values, Nat 0, false, false) { state, character } counter-step (state, character)
  counter-values final

optional-positive? is fn (value : Optional Nat) -> Boolean
  value
    Some present then present > 0
    None then false
counter-at is fn (values : List Nat, index : Nat) -> Optional Nat
  first (values select-index (index ..= index))
button-usable-step is fn ((values : List Nat, usable? : Boolean, index : Nat)) -> Boolean
  usable? and (optional-positive? (counter-at (values, index)))
button-usable? is fn (values : List Nat, button : List Nat) -> Boolean
  button fold true { usable?, index } button-usable-step (values, usable?, index)

decrement-step is fn ((index : Nat, state : (List Nat, Int), value : Nat)) -> (List Nat, Int)
  position is toggle-position state
  position = index
    true then ((toggle-values state) append (value - 1), position + 1)
    false then ((toggle-values state) append value, position + 1)
decrement-index is fn (values : List Nat, index : Nat) -> List Nat
  empty-values : List Nat is Empty
  final is values fold (empty-values, 0) { state, value } decrement-step (index, state, value)
  toggle-values final
decrement-button is fn (values : List Nat, button : List Nat) -> List Nat
  button fold values { state, index } decrement-index (state, index)

counter-zero-step is fn (zero? : Boolean, value : Nat) -> Boolean
  zero? and (value = 0)
counter-zero? is fn (values : List Nat) -> Boolean
  values fold true { zero?, value } counter-zero-step (zero?, value)

visited-match-step is fn ((sought : List Nat, found? : Boolean, candidate : List Nat)) -> Boolean
  found? or (nat-list-equal? (sought, candidate))
visited-contains? is fn (visited : List List Nat, sought : List Nat) -> Boolean
  visited fold false { found?, candidate } visited-match-step (sought, found?, candidate)

queue-values is fn ((values : List Nat, presses : Int)) -> List Nat
  values
queue-presses is fn ((values : List Nat, presses : Int)) -> Int
  presses

search-queue is fn ((queue : List (List Nat, Int), visited : List List Nat, answer : List Int)) -> List (List Nat, Int)
  queue
search-visited is fn ((queue : List (List Nat, Int), visited : List List Nat, answer : List Int)) -> List List Nat
  visited
search-answer is fn ((queue : List (List Nat, Int), visited : List List Nat, answer : List Int)) -> List Int
  answer

enqueue-usable-successor is fn ((entry : (List Nat, Int), state : (List (List Nat, Int), List List Nat, List Int), successor : List Nat)) -> (List (List Nat, Int), List List Nat, List Int)
  visited-contains? (search-visited state, successor)
    true then state
    false then ((search-queue state) append (successor, (queue-presses entry) + 1), (search-visited state) append successor, search-answer state)

enqueue-successor is fn ((entry : (List Nat, Int), state : (List (List Nat, Int), List List Nat, List Int), button : List Nat)) -> (List (List Nat, Int), List List Nat, List Int)
  values is queue-values entry
  button-usable? (values, button)
    false then state
    true then enqueue-usable-successor (entry, state, decrement-button (values, button))

expand-counter-entry is fn ((buttons : List List Nat, state : (List (List Nat, Int), List List Nat, List Int), entry : (List Nat, Int))) -> (List (List Nat, Int), List List Nat, List Int)
  buttons fold state { next, button } enqueue-successor (entry, next, button)

counter-entry-result is fn ((buttons : List List Nat, state : (List (List Nat, Int), List List Nat, List Int), entry : (List Nat, Int))) -> (List (List Nat, Int), List List Nat, List Int)
  counter-zero? (queue-values entry)
    true then (search-queue state, search-visited state, one (queue-presses entry))
    false then expand-counter-entry (buttons, state, entry)

counter-optional-entry is fn ((buttons : List List Nat, state : (List (List Nat, Int), List List Nat, List Int), entry : Optional (List Nat, Int))) -> (List (List Nat, Int), List List Nat, List Int)
  entry
    Some present then counter-entry-result (buttons, state, present)
    None then state

counter-search-once is fn ((buttons : List List Nat, state : (List (List Nat, Int), List List Nat, List Int))) -> (List (List Nat, Int), List List Nat, List Int)
  (entry-count (search-answer state)) > 0
    true then state
    false then counter-optional-entry (buttons, ((search-queue state) select-index (1 .. (entry-count (search-queue state))), search-visited state, search-answer state), first (search-queue state))

state-space-step is fn (count : Int, value : Nat) -> Int
  count * (value + 1)
state-space is fn (target : List Nat) -> Int
  target fold 1 { count, value } state-space-step (count, value)

counter-answer is fn (answer : List Int) -> Int
  first answer
    Some present then present
    None then 0

minimum-counter-line is fn (line : String) -> Int
  target is counter-target line
  buttons is machine-buttons line
  initial-entry is (target, 0)
  initial-queue : List (List Nat, Int) is one initial-entry
  initial-visited : List List Nat is one target
  empty-answer : List Int is Empty
  bounds is collect (0 iterate ({ index } index + 1) take-while ({ index } index < (state-space target)))
  final is bounds fold (initial-queue, initial-visited, empty-answer) { state, ignored } counter-search-once (buttons, state)
  counter-answer (search-answer final)

minimum-counter-total-step is fn (total : Int, line : String) -> Int
  total + (minimum-counter-line line)

### Sum minimum presses for nonnegative exact counters described by brace targets and indexed buttons.
pub minimum-counter-presses is fn (manual : String) -> Int
  (manual-lines manual) fold 0 { total, line } minimum-counter-total-step (total, line)
