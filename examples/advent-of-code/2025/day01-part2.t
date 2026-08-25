#!/usr/bin/env topal
use language (
  version is v0.1
)
use library std (
  version is v0.1
)

# Count every passage through zero while applying each dial rotation.
lines is std text lines
drop is std sequence drop
starts-with? is std pattern starts-with?
parse-int is std parse int

required-int is fn (candidate : Optional Int) -> Int
  candidate
    Some value then value
    None then 0

quotient-from is fn (quotient : Int, remainder : Int) -> Int
  quotient
floor-hundred is fn (value : Int) -> Int
  quotient-from (value /% 100)

signed-distance is fn (left : Boolean, distance : Int) -> Int
  left
    true then negate distance
    false then distance

crossing-count is fn (left : Boolean, (position : Int, distance : Int)) -> Int
  left
    true then (floor-hundred (position - 1)) - (floor-hundred (position - distance - 1))
    false then (floor-hundred (position + distance)) - (floor-hundred position)

turn is fn (state : (Int, Int), instruction : String) -> (Int, Int)
  position-extract is fn (position : Int, count : Int) -> Int
    position
  count-extract is fn (position : Int, count : Int) -> Int
    count
  position is position-extract state
  count is count-extract state
  distance is required-int (parse-int (drop (instruction, 1)))
  left is starts-with? (instruction, "L")
  delta is signed-distance (left, distance)
  crossings is crossing-count (left, (position, distance))
  ((position + delta) % 100, count + crossings)

answer is fn (state : (Int, Int)) -> Int
  count-extract is fn (position : Int, count : Int) -> Int
    count
  count-extract state

solve is fn (input : String) -> Int
  answer ((lines input) fold (50, 0) { state, instruction } turn (state, instruction))
