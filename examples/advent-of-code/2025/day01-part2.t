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
required-int-callable is required-int

quotient-from is fn (quotient : Int, remainder : Int) -> Int
  quotient
quotient-from-callable is quotient-from
floor-hundred is fn (value : Int) -> Int
  quotient-from-callable (value /% 100)
floor-hundred-callable is floor-hundred

signed-distance is fn (left : Boolean, distance : Int) -> Int
  left
    true then negate distance
    false then distance
signed-distance-callable is signed-distance

crossing-count is fn (left : Boolean, (position : Int, distance : Int)) -> Int
  left
    true then (floor-hundred-callable (position - 1)) - (floor-hundred-callable (position - distance - 1))
    false then (floor-hundred-callable (position + distance)) - (floor-hundred-callable position)
crossing-count-callable is crossing-count

turn is fn (state : (Int, Int), instruction : String) -> (Int, Int)
  position-extract is fn (position : Int, count : Int) -> Int
    position
  count-extract is fn (position : Int, count : Int) -> Int
    count
  position is position-extract state
  count is count-extract state
  distance is required-int-callable (parse-int (drop (instruction, 1)))
  left is starts-with? (instruction, "L")
  delta is signed-distance-callable (left, distance)
  crossings is crossing-count-callable (left, (position, distance))
  ((position + delta) % 100, count + crossings)
turn-callable is turn

answer is fn (state : (Int, Int)) -> Int
  count-extract is fn (position : Int, count : Int) -> Int
    count
  count-extract state
answer-callable is answer

solve is fn (input : String) -> Int
  answer-callable ((lines input) fold (50, 0) { state, instruction } turn-callable (state, instruction))
