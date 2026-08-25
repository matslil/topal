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
select-marked-callable is select-marked

marked-columns is fn (row : String, marker : Character) -> List Int
  final is (enumerate (character-list row)) fold ((Empty Int), marker) { state, pair } select-marked-callable (state, pair)
  columns-of is fn (columns : List Int, ignored : Character) -> List Int
    columns
  columns-of final
marked-columns-callable is marked-columns

advance is fn (beams : List Int, row : String) -> List Int
  splitters is marked-columns-callable (row, "^")
  add-beam is fn (result : List Int, column : Int) -> List Int
    splitters contains-entry column
      true then (result append (column - 1)) append (column + 1)
      false then result append column
  add-beam-callable is add-beam
  beams fold (Empty Int) { result, column } add-beam-callable (result, column)
advance-callable is advance

solve is fn (input : String) -> Int
  rows is lines input
  starts is rows fold (Empty Int) { found, row } found concat (marked-columns-callable (row, "S"))
  timelines is rows fold starts { beams, row } advance-callable (beams, row)
  entry-count timelines
