#!/usr/bin/env topal
use language (
  version is v0.1
)
use library std (
  version is v0.1
)

# Propagate the set of energized columns row by row and count splitter events.
lines is std text lines
character-list is std parse character-list
enumerate is std sequence enumerate
unique is std sequence unique

select-splitter is fn (columns : List Int, (index : Nat, value : Character)) -> List Int
  value = "^"
    true then columns append index
    false then columns

splitter-columns is fn (row : String) -> List Int
  indexed is enumerate (character-list row)
  indexed fold (Empty Int) { columns, pair } select-splitter (columns, pair)

propagate-beam is fn ((splitters : List Int, beams : List Int), column : Int) -> List Int
  splitters contains-entry column
    true then (beams append (column - 1)) append (column + 1)
    false then beams append column

count-split is fn (splitters : List Int, (count : Int, column : Int)) -> Int
  splitters contains-entry column
    true then count + 1
    false then count

advance is fn (state : (List Int, Int), row : String) -> (List Int, Int)
  beams-of is fn (beams : List Int, events : Int) -> List Int
    beams
  events-of is fn (beams : List Int, events : Int) -> Int
    events
  beams is beams-of state
  splitters is splitter-columns row
  count-here is fn (count : Int, column : Int) -> Int
    count-split (splitters, (count, column))
  events is beams fold 0 { count, column } count-here (count, column)
  next is beams fold (Empty Int) { result, column } propagate-beam ((splitters, result), column)
  (unique next, (events-of state) + events)

collect-starts is fn (columns : List Int, row : String) -> List Int
  indexed is enumerate (character-list row)
  add-start is fn (result : List Int, (index : Nat, value : Character)) -> List Int
    value = "S"
      true then result append index
      false then result
  found is indexed fold (Empty Int) { result, pair } add-start (result, pair)
  columns concat found

solve is fn (input : String) -> Int
  rows is lines input
  starts is rows fold (Empty Int) { columns, row } collect-starts (columns, row)
  required is fn (candidate : Optional Int) -> Int
    candidate
      Some value then value
      None then 0
  start is required (first starts)
  final is rows fold (one start, 0) { state, row } advance (state, row)
  events-of is fn (beams : List Int, events : Int) -> Int
    events
  events-of final
