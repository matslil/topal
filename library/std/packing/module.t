#!/usr/bin/env topal
use language (version is v0.1)

### Revision of the exact finite polyomino-packing namespace.
pub revision is 1

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
description-lines is fn (text : String) -> List String
  empty-lines : List String is Empty
  final is (collect (characters text)) fold (empty-lines, Nat 0, Nat 0) { state, character } line-step (text, state, character)
  finish-lines (text, final)

section-values is fn ((sections : List List String, current : List String)) -> List List String
  sections
section-current is fn ((sections : List List String, current : List String)) -> List String
  current
finish-section is fn (state : (List List String, List String)) -> (List List String, List String)
  empty-current : List String is Empty
  (entry-count (section-current state)) > 0
    true then ((section-values state) append (section-current state), empty-current)
    false then state
section-step is fn (state : (List List String, List String), line : String) -> (List List String, List String)
  (entry-count (collect (characters line))) = 0
    true then finish-section state
    false then (section-values state, (section-current state) append line)
description-sections is fn (text : String) -> List List String
  empty-sections : List List String is Empty
  empty-current : List String is Empty
  final is (description-lines text) fold (empty-sections, empty-current) { state, line } section-step (state, line)
  section-values (finish-section final)

number-values is fn ((values : List Int, value : Int, active? : Boolean)) -> List Int
  values
number-value is fn ((values : List Int, value : Int, active? : Boolean)) -> Int
  value
number-active? is fn ((values : List Int, value : Int, active? : Boolean)) -> Boolean
  active?
finish-number is fn (state : (List Int, Int, Boolean)) -> (List Int, Int, Boolean)
  number-active? state
    true then ((number-values state) append (number-value state), 0, false)
    false then state
number-digit is fn (state : (List Int, Int, Boolean), digit : Nat) -> (List Int, Int, Boolean)
  (number-values state, (number-value state) * 10 + digit, true)
number-step is fn (state : (List Int, Int, Boolean), character : Character) -> (List Int, Int, Boolean)
  ascii-decimal-digit character
    Some digit then number-digit (state, digit)
    None then finish-number state
unsigned-values is fn (text : String) -> List Int
  empty-values : List Int is Empty
  final is (collect (characters text)) fold (empty-values, 0, false) { state, character } number-step (state, character)
  number-values (finish-number final)

point-values is fn ((points : List (Int, Int), column : Int)) -> List (Int, Int)
  points
point-column is fn ((points : List (Int, Int), column : Int)) -> Int
  column
point-step is fn ((row : Int, state : (List (Int, Int), Int), character : Character)) -> (List (Int, Int), Int)
  column is point-column state
  character = "#"
    true then ((point-values state) append (row, column), column + 1)
    false then (point-values state, column + 1)
append-shape-row is fn ((row : Int, points : List (Int, Int), line : String)) -> List (Int, Int)
  final is (collect (characters line)) fold (points, 0) { state, character } point-step (row, state, character)
  point-values final
shape-row-state-points is fn ((points : List (Int, Int), row : Int)) -> List (Int, Int)
  points
shape-row-state-row is fn ((points : List (Int, Int), row : Int)) -> Int
  row
shape-row-step is fn (state : (List (Int, Int), Int), line : String) -> (List (Int, Int), Int)
  row is shape-row-state-row state
  (append-shape-row (row, shape-row-state-points state, line), row + 1)
shape-section is fn (section : List String) -> List (Int, Int)
  rows is section select-index (1 .. (entry-count section))
  empty-points : List (Int, Int) is Empty
  final is rows fold (empty-points, 0) { state, line } shape-row-step (state, line)
  shape-row-state-points final

first-line is fn (section : List String) -> String
  first section
    Some line then line
    None then ""
region-section? is fn (section : List String) -> Boolean
  (collect (characters (first-line section))) contains-entry "x"

parsed-shapes is fn ((shapes : List List (Int, Int), regions : List (Int, Int, List Int))) -> List List (Int, Int)
  shapes
parsed-regions is fn ((shapes : List List (Int, Int), regions : List (Int, Int, List Int))) -> List (Int, Int, List Int)
  regions

region-tail is fn ((regions : List (Int, Int, List Int), width : Int, height : Int, tail : List Int)) -> (List List (Int, Int), List (Int, Int, List Int))
  empty-shapes : List List (Int, Int) is Empty
  (empty-shapes, regions append (width, height, tail))
region-second is fn ((regions : List (Int, Int, List Int), width : Int, rest : List Int)) -> (List List (Int, Int), List (Int, Int, List Int))
  empty-shapes : List List (Int, Int) is Empty
  rest
    Entry (height, tail) then region-tail (regions, width, height, tail)
    Empty then (empty-shapes, regions)
parse-region-line is fn (regions : List (Int, Int, List Int), line : String) -> (List List (Int, Int), List (Int, Int, List Int))
  empty-shapes : List List (Int, Int) is Empty
  unsigned-values line
    Entry (width, rest) then region-second (regions, width, rest)
    Empty then (empty-shapes, regions)
append-region-line is fn (state : (List List (Int, Int), List (Int, Int, List Int)), line : String) -> (List List (Int, Int), List (Int, Int, List Int))
  parsed is parse-region-line (parsed-regions state, line)
  (parsed-shapes state, parsed-regions parsed)

parse-section is fn (state : (List List (Int, Int), List (Int, Int, List Int)), section : List String) -> (List List (Int, Int), List (Int, Int, List Int))
  region-section? section
    true then section fold state { parsed, line } append-region-line (parsed, line)
    false then ((parsed-shapes state) append (shape-section section), parsed-regions state)

parse-description is fn (text : String) -> (List List (Int, Int), List (Int, Int, List Int))
  empty-shapes : List List (Int, Int) is Empty
  empty-regions : List (Int, Int, List Int) is Empty
  (description-sections text) fold (empty-shapes, empty-regions) { state, section } parse-section (state, section)

point-x is fn ((x : Int, y : Int)) -> Int
  x
point-y is fn ((x : Int, y : Int)) -> Int
  y
minimum-int is fn (left : Int, right : Int) -> Int
  left < right
    true then left
    false then right
normalize-min-x is fn (minimum : Int, point : (Int, Int)) -> Int
  minimum-int (minimum, point-x point)
normalize-min-y is fn (minimum : Int, point : (Int, Int)) -> Int
  minimum-int (minimum, point-y point)
translate-normal is fn ((minimum-x : Int, minimum-y : Int, normalized : List (Int, Int), point : (Int, Int))) -> List (Int, Int)
  normalized append ((point-x point) - minimum-x, (point-y point) - minimum-y)
normalize-shape is fn (points : List (Int, Int)) -> List (Int, Int)
  minimum-x is points fold 0 { minimum, point } normalize-min-x (minimum, point)
  minimum-y is points fold 0 { minimum, point } normalize-min-y (minimum, point)
  empty-points : List (Int, Int) is Empty
  points fold empty-points { normalized, point } translate-normal (minimum-x, minimum-y, normalized, point)

flip-x is fn (flip? : Boolean, x : Int) -> Int
  flip?
    true then 0 - x
    false then x

transform-point is fn ((flip? : Boolean, rotation : Int, point : (Int, Int))) -> (Int, Int)
  x is point-x point
  y is point-y point
  flipped-x is flip-x (flip?, x)
  rotation
    = 0 then (flipped-x, y)
    = 1 then (y, 0 - flipped-x)
    = 2 then (0 - flipped-x, 0 - y)
    otherwise (0 - y, flipped-x)
transform-step is fn ((flip? : Boolean, rotation : Int, transformed : List (Int, Int), point : (Int, Int))) -> List (Int, Int)
  transformed append (transform-point (flip?, rotation, point))
transform-shape is fn ((points : List (Int, Int), setting : (Boolean, Int))) -> List (Int, Int)
  flip? is fn ((flip? : Boolean, rotation : Int)) -> Boolean
    flip?
  rotation is fn ((flip? : Boolean, rotation : Int)) -> Int
    rotation
  empty-points : List (Int, Int) is Empty
  transformed is points fold empty-points { values, point } transform-step (flip? setting, rotation setting, values, point)
  normalize-shape transformed

point-list-equal? is fn (left : List (Int, Int), right : List (Int, Int)) -> Boolean
  ((entry-count left) = (entry-count right)) and (left contains-sequence right)
orientation-present-step is fn ((sought : List (Int, Int), found? : Boolean, candidate : List (Int, Int))) -> Boolean
  found? or (point-list-equal? (sought, candidate))
orientation-present? is fn (orientations : List List (Int, Int), sought : List (Int, Int)) -> Boolean
  orientations fold false { found?, candidate } orientation-present-step (sought, found?, candidate)
append-orientation is fn ((shape : List (Int, Int), orientations : List List (Int, Int), setting : (Boolean, Int))) -> List List (Int, Int)
  candidate is transform-shape (shape, setting)
  orientation-present? (orientations, candidate)
    true then orientations
    false then orientations append candidate
shape-orientations is fn (shape : List (Int, Int)) -> List List (Int, Int)
  settings : List (Boolean, Int) is Entry ((false, 0), Entry ((false, 1), Entry ((false, 2), Entry ((false, 3), Entry ((true, 0), Entry ((true, 1), Entry ((true, 2), Entry ((true, 3), Empty))))))))
  empty-orientations : List List (Int, Int) is Empty
  settings fold empty-orientations { values, setting } append-orientation (shape, values, setting)

maximum-int is fn (left : Int, right : Int) -> Int
  left > right
    true then left
    false then right
shape-width-step is fn (width : Int, point : (Int, Int)) -> Int
  maximum-int (width, (point-x point) + 1)
shape-height-step is fn (height : Int, point : (Int, Int)) -> Int
  maximum-int (height, (point-y point) + 1)

translate-point is fn ((x : Int, y : Int, translated : List (Int, Int), point : (Int, Int))) -> List (Int, Int)
  translated append ((point-x point) + x, (point-y point) + y)
translate-shape is fn ((shape : List (Int, Int), x : Int, y : Int)) -> List (Int, Int)
  empty-points : List (Int, Int) is Empty
  shape fold empty-points { translated, point } translate-point (x, y, translated, point)

placement-free-step is fn ((occupied : List (Int, Int), free? : Boolean, point : (Int, Int))) -> Boolean
  free? and (not (occupied contains-entry point))
placement-free? is fn (occupied : List (Int, Int), placement : List (Int, Int)) -> Boolean
  placement fold true { free?, point } placement-free-step (occupied, free?, point)

configuration-present-step is fn ((sought : List (Int, Int), found? : Boolean, candidate : List (Int, Int))) -> Boolean
  found? or (point-list-equal? (sought, candidate))
configuration-present? is fn (configurations : List List (Int, Int), sought : List (Int, Int)) -> Boolean
  configurations fold false { found?, candidate } configuration-present-step (sought, found?, candidate)
append-placement is fn ((configurations : List List (Int, Int), occupied : List (Int, Int), placement : List (Int, Int))) -> List List (Int, Int)
  valid? is placement-free? (occupied, placement)
  combined is occupied concat placement
  valid? and (not (configuration-present? (configurations, combined)))
    true then configurations append combined
    false then configurations

place-y is fn ((shape : List (Int, Int), occupied : List (Int, Int), x : Int, configurations : List List (Int, Int), y : Int)) -> List List (Int, Int)
  append-placement (configurations, occupied, translate-shape (shape, x, y))
place-x is fn ((shape : List (Int, Int), occupied : List (Int, Int), ys : List Int, configurations : List List (Int, Int), x : Int)) -> List List (Int, Int)
  ys fold configurations { values, y } place-y (shape, occupied, x, values, y)
place-orientation is fn ((width : Int, height : Int, occupied : List (Int, Int), configurations : List List (Int, Int), shape : List (Int, Int))) -> List List (Int, Int)
  shape-width is shape fold 0 { value, point } shape-width-step (value, point)
  shape-height is shape fold 0 { value, point } shape-height-step (value, point)
  xs is collect (0 iterate ({ x } x + 1) take-while ({ x } x + shape-width <= width))
  ys is collect (0 iterate ({ y } y + 1) take-while ({ y } y + shape-height <= height))
  xs fold configurations { values, x } place-x (shape, occupied, ys, values, x)
place-occupied is fn ((width : Int, height : Int, orientations : List List (Int, Int), configurations : List List (Int, Int), occupied : List (Int, Int))) -> List List (Int, Int)
  orientations fold configurations { values, shape } place-orientation (width, height, occupied, values, shape)
expand-configurations is fn ((width : Int, height : Int, orientations : List List (Int, Int), current : List List (Int, Int))) -> List List (Int, Int)
  empty-configurations : List List (Int, Int) is Empty
  current fold empty-configurations { values, occupied } place-occupied (width, height, orientations, values, occupied)

shape-at is fn (shapes : List List (Int, Int), index : Int) -> Optional (List (Int, Int))
  first (shapes select-index (index ..= index))
orientations-for is fn (shape : Optional (List (Int, Int))) -> List List (Int, Int)
  empty-orientations : List List (Int, Int) is Empty
  shape
    Some present then shape-orientations present
    None then empty-orientations

piece-shape is fn ((shape-index : Int, quantity-index : Int)) -> Int
  shape-index
piece-count is fn ((shape-index : Int, quantity-index : Int)) -> Int
  quantity-index
append-pieces is fn ((pieces : List Int, state : (Int, Int), quantity : Int)) -> (List Int, (Int, Int))
  shape-index is piece-shape state
  count is piece-count state
  indexes is collect (0 iterate ({ index } index + 1) take-while ({ index } index < quantity))
  appended is indexes fold pieces { values, ignored } values append shape-index
  (appended, (shape-index + 1, count + 1))

pieces-values is fn ((pieces : List Int, state : (Int, Int))) -> List Int
  pieces
pieces-state is fn ((pieces : List Int, state : (Int, Int))) -> (Int, Int)
  state
region-pieces-step is fn (state : (List Int, (Int, Int)), quantity : Int) -> (List Int, (Int, Int))
  append-pieces (pieces-values state, pieces-state state, quantity)
region-pieces is fn (quantities : List Int) -> List Int
  empty-pieces : List Int is Empty
  final is quantities fold (empty-pieces, (0, 0)) { state, quantity } region-pieces-step (state, quantity)
  pieces-values final

fit-piece is fn ((width : Int, height : Int, shapes : List List (Int, Int), configurations : List List (Int, Int), piece : Int)) -> List List (Int, Int)
  expand-configurations (width, height, orientations-for (shape-at (shapes, piece)), configurations)
region-width is fn ((width : Int, height : Int, quantities : List Int)) -> Int
  width
region-height is fn ((width : Int, height : Int, quantities : List Int)) -> Int
  height
region-quantities is fn ((width : Int, height : Int, quantities : List Int)) -> List Int
  quantities
region-fits? is fn (shapes : List List (Int, Int), region : (Int, Int, List Int)) -> Boolean
  width is region-width region
  height is region-height region
  empty-occupied : List (Int, Int) is Empty
  configurations : List List (Int, Int) is one empty-occupied
  final is (region-pieces (region-quantities region)) fold configurations { current, piece } fit-piece (width, height, shapes, current, piece)
  (entry-count final) > 0
count-region is fn ((shapes : List List (Int, Int), count : Int, region : (Int, Int, List Int))) -> Int
  region-fits? (shapes, region)
    true then count + 1
    false then count

fitting-source is fn (description : String) -> Int
  parsed is parse-description description
  (parsed-regions parsed) fold 0 { count, region } count-region (parsed-shapes parsed, count, region)

### Count rectangular regions that admit the requested free polyomino multiset.
pub fitting-region-count is fn (description : String) -> Int
  fitting-source description
