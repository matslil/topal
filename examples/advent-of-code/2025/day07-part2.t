#!/usr/bin/env topal
use language (
  version is v0.1
)
use library std (
  version is v0.1
)

# Duplicate beam columns represent independent timelines, so multiplicity is exact.
lines is std text lines
character-list is std parse character-list
enumerate is std sequence enumerate

select-marked is fn (arguments : (List Int, Character), (index : Nat, value : Character)) -> (List Int, Character)
  columns-of is fn (columns : List Int, marker : Character) -> List Int
    columns
  marker-of is fn (columns : List Int, marker : Character) -> Character
    marker
  columns is columns-of arguments
  marker is marker-of arguments
  value = marker
    true then (columns append index, marker)
    false then arguments

marked-columns is fn (row : String, marker : Character) -> List Int
  final is (enumerate (character-list row)) fold ((Empty Int), marker) { state, pair } select-marked (state, pair)
  columns-of is fn (columns : List Int, ignored : Character) -> List Int
    columns
  columns-of final

advance is fn (beams : List Int, row : String) -> List Int
  splitters is marked-columns (row, "^")
  add-beam is fn (result : List Int, column : Int) -> List Int
    splitters contains-entry column
      true then (result append (column - 1)) append (column + 1)
      false then result append column
  beams fold (Empty Int) { result, column } add-beam (result, column)

solve is fn (input : String) -> Int
  rows is lines input
  starts is rows fold (Empty Int) { found, row } found concat (marked-columns (row, "S"))
  timelines is rows fold starts { beams, row } advance (beams, row)
  entry-count timelines
