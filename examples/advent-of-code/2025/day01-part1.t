#!/usr/bin/env topal
use language (
  version is v0.1
)
use library std (
  version is v0.1
)

# Count rotations which leave the circular dial at zero.
lines is std text lines
drop is std sequence drop
starts-with? is std pattern starts-with?
parse-int is std parse int

required-int is fn (candidate : Optional Int) -> Int
  candidate
    Some value then value
    None then 0
required-int-callable is required-int

signed-distance is fn (left : Boolean, distance : Int) -> Int
  left
    true then negate distance
    false then distance
signed-distance-callable is signed-distance

increment-at-zero is fn (count : Int, position : Int) -> Int
  position = 0
    true then count + 1
    false then count
increment-at-zero-callable is increment-at-zero

turn is fn (state : (Int, Int), instruction : String) -> (Int, Int)
  position-extract is fn (position : Int, count : Int) -> Int
    position
  count-extract is fn (position : Int, count : Int) -> Int
    count
  position is position-extract state
  count is count-extract state
  distance is required-int-callable (parse-int (drop (instruction, 1)))
  delta is signed-distance-callable (starts-with? (instruction, "L"), distance)
  next is (position + delta) % 100
  next-count is increment-at-zero-callable (count, next)
  (next, next-count)
turn-callable is turn

answer is fn (state : (Int, Int)) -> Int
  count-extract is fn (position : Int, count : Int) -> Int
    count
  count-extract state
answer-callable is answer

solve is fn (input : String) -> Int
  answer-callable ((lines input) fold (50, 0) { state, instruction } turn-callable (state, instruction))
