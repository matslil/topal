#!/usr/bin/env topal
use language (
  version is v0.1
)
use library std (
  version is v0.1
)

# Count rolls with fewer than four occupied neighbors in the eight-cell ring.
lines is std text lines
character-list is std parse character-list
enumerate is std sequence enumerate
decimal is std parse decimal
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
collect-row-callable is collect-row

points-from is fn (input : String) -> List (Int, Int)
  final is (lines input) fold (no-points, 0) { state, line } collect-row-callable (state, line)
  points-of is fn (points : List (Int, Int), row : Int) -> List (Int, Int)
    points
  points-of final
points-from-callable is points-from

point-key is fn (point : (Int, Int)) -> String
  x-of is fn (x : Int, y : Int) -> Int
    x
  y-of is fn (x : Int, y : Int) -> Int
    y
  (decimal (x-of point)) concat "," concat (decimal (y-of point))
point-key-callable is point-key

accessible is fn (occupied : List String, point : (Int, Int)) -> Boolean
  neighbor is fn (point : (Int, Int), offset : (Int, Int)) -> (Int, Int)
    x-of is fn (x : Int, y : Int) -> Int
      x
    y-of is fn (x : Int, y : Int) -> Int
      y
    ((x-of point) + (x-of offset), (y-of point) + (y-of offset))
  count-neighbor is fn (count : Int, offset : (Int, Int)) -> Int
    occupied contains-entry (point-key-callable (neighbor (point, offset)))
      true then count + 1
      false then count
  (offsets fold 0 { count, offset } count-neighbor (count, offset)) < 4
accessible-callable is accessible

count-accessible is fn (state : (Int, List String), point : (Int, Int)) -> (Int, List String)
  count-of is fn (count : Int, occupied : List String) -> Int
    count
  occupied-of is fn (count : Int, occupied : List String) -> List String
    occupied
  occupied is occupied-of state
  increment is fn (accepted : Boolean, count : Int) -> Int
    accepted
      true then count + 1
      false then count
  next-count is increment (accessible-callable (occupied, point), count-of state)
  (next-count, occupied)
count-accessible-callable is count-accessible

solve is fn (input : String) -> Int
  points is points-from-callable input
  keys is points fold (Empty String) { selected, point } selected append (point-key-callable point)
  occupied is keys
  final is points fold (0, occupied) { state, point } count-accessible-callable (state, point)
  count-of is fn (count : Int, set : List String) -> Int
    count
  count-of final
