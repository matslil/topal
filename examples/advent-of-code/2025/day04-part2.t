#!/usr/bin/env topal
use language (
  version is v0.1
)
use library std (
  version is v0.1
)

# Repeatedly remove every roll with fewer than four occupied neighbors.
lines is std text lines
character-list is std parse character-list
enumerate is std sequence enumerate
decimal is std parse decimal
range-values is std sequence values
no-points : List (Int, Int) is Empty
offsets : List (Int, Int) is Entry ((-1, -1), Entry ((0, -1), Entry ((1, -1), Entry ((-1, 0), Entry ((1, 0), Entry ((-1, 1), Entry ((0, 1), Entry ((1, 1), Empty))))))))

collect-row is fn (state : (List (Int, Int), Int), line : String) -> (List (Int, Int), Int)
  points-of is fn (points : List (Int, Int), row : Int) -> List (Int, Int)
    points
  row-of is fn (points : List (Int, Int), row : Int) -> Int
    row
  points is points-of state
  current-row is row-of state
  row-characters is character-list line
  collect-cell is fn (selected : List (Int, Int), cell : (Nat, Character)) -> List (Int, Int)
    column-of is fn (column : Nat, character : Character) -> Nat
      column
    character-of is fn (column : Nat, character : Character) -> Character
      character
    character-of cell
      = "@" then selected append (column-of cell, current-row)
      otherwise selected
  ((enumerate row-characters) fold points { selected, cell } collect-cell (selected, cell), current-row + 1)

points-from is fn (input : String) -> List (Int, Int)
  final is (lines input) fold (no-points, 0) { state, line } collect-row (state, line)
  points-of is fn (points : List (Int, Int), row : Int) -> List (Int, Int)
    points
  points-of final

point-key is fn (point : (Int, Int)) -> String
  x-of is fn (x : Int, y : Int) -> Int
    x
  y-of is fn (x : Int, y : Int) -> Int
    y
  (decimal (x-of point)) concat "," concat (decimal (y-of point))

accessible is fn (occupied : List String, point : (Int, Int)) -> Boolean
  neighbor is fn (point : (Int, Int), offset : (Int, Int)) -> (Int, Int)
    x-of is fn (x : Int, y : Int) -> Int
      x
    y-of is fn (x : Int, y : Int) -> Int
      y
    ((x-of point) + (x-of offset), (y-of point) + (y-of offset))
  count-neighbor is fn (count : Int, offset : (Int, Int)) -> Int
    occupied contains-entry (point-key (neighbor (point, offset)))
      true then count + 1
      false then count
  (offsets fold 0 { count, offset } count-neighbor (count, offset)) < 4

points-of-state is fn (points : List (Int, Int), removed : Int) -> List (Int, Int)
  points
removed-of-state is fn (points : List (Int, Int), removed : Int) -> Int
  removed

remove-round is fn (state : (List (Int, Int), Int), iteration : Int) -> (List (Int, Int), Int)
  _ is iteration
  points is points-of-state state
  occupied is points fold (Empty String) { selected, point } selected append (point-key point)
  retain is fn (selected : List (Int, Int), point : (Int, Int)) -> List (Int, Int)
    accessible (occupied, point)
      true then selected
      false then selected append point
  remaining is points fold no-points { selected, point } retain (selected, point)
  (remaining, (removed-of-state state) + (entry-count points) - (entry-count remaining))

solve is fn (input : String) -> Int
  points is points-from input
  final is (range-values (1 ..= (entry-count points))) fold (points, 0) { state, iteration } remove-round (state, iteration)
  removed-of-state final
