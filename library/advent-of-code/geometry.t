#!/usr/bin/env topal
use language (
  version is v0.1
)

### Revision of the Advent of Code finite-geometry namespace.
pub revision is 1

point3-x is fn ((x : Int, y : Int, z : Int)) -> Int
  x
point3-y is fn ((x : Int, y : Int, z : Int)) -> Int
  y
point3-z is fn ((x : Int, y : Int, z : Int)) -> Int
  z

point3-at is fn (points : List (Int, Int, Int), index : Int) -> Optional (Int, Int, Int)
  first (points select-index (index ..= index))

distance3 is fn (left : (Int, Int, Int), right : (Int, Int, Int)) -> Int
  dx is (point3-x right) - (point3-x left)
  dy is (point3-y right) - (point3-y left)
  dz is (point3-z right) - (point3-z left)
  (dx * dx) + (dy * dy) + (dz * dz)

append-point-pair-right is fn ((pairs : List (Int, Int, Int), left-index : Int, right-index : Int, left : (Int, Int, Int), right : Optional (Int, Int, Int))) -> List (Int, Int, Int)
  right
    Some point then pairs append (distance3 (left, point), left-index, right-index)
    None then pairs

append-point-pair-left is fn ((points : List (Int, Int, Int), pairs : List (Int, Int, Int), left-index : Int, right-index : Int, left : Optional (Int, Int, Int))) -> List (Int, Int, Int)
  left
    Some point then append-point-pair-right (pairs, left-index, right-index, point, point3-at (points, right-index))
    None then pairs

append-point-pair is fn ((points : List (Int, Int, Int), left-index : Int, pairs : List (Int, Int, Int), right-index : Int)) -> List (Int, Int, Int)
  left-index < right-index
    true then append-point-pair-left (points, pairs, left-index, right-index, point3-at (points, left-index))
    false then pairs

append-pairs-for-left is fn ((points : List (Int, Int, Int), indexes : List Int, pairs : List (Int, Int, Int), left-index : Int)) -> List (Int, Int, Int)
  indexes fold pairs { selected, right-index } append-point-pair (points, left-index, selected, right-index)

point-pairs is fn (points : List (Int, Int, Int)) -> List (Int, Int, Int)
  length is entry-count points
  indexes is collect (0 iterate ({ index } index + 1) take-while ({ index } index < length))
  empty-pairs : List (Int, Int, Int) is Empty
  indexes fold empty-pairs { pairs, left-index } append-pairs-for-left (points, indexes, pairs, left-index)

pair-distance is fn ((distance : Int, left : Int, right : Int)) -> Int
  distance
pair-left is fn ((distance : Int, left : Int, right : Int)) -> Int
  left
pair-right is fn ((distance : Int, left : Int, right : Int)) -> Int
  right

pair-equal-distance-before? is fn (left : (Int, Int, Int), right : (Int, Int, Int)) -> Boolean
  (pair-left left) < (pair-left right)
    true then true
    false then ((pair-left left) = (pair-left right)) and ((pair-right left) < (pair-right right))

pair-tie-before? is fn ((left : (Int, Int, Int), right : (Int, Int, Int), tied? : Boolean)) -> Boolean
  tied?
    false then false
    true then pair-equal-distance-before? (left, right)

pair-before? is fn (left : (Int, Int, Int), right : (Int, Int, Int)) -> Boolean
  left-distance is pair-distance left
  right-distance is pair-distance right
  left-distance < right-distance
    true then true
    false then pair-tie-before? (left, right, left-distance = right-distance)

pair-lower-bound-step is fn ((sought : (Int, Int, Int), count : Nat, candidate : (Int, Int, Int))) -> Nat
  pair-before? (candidate, sought)
    true then count + 1
    false then count

insert-pair is fn (sorted : List (Int, Int, Int), candidate : (Int, Int, Int)) -> List (Int, Int, Int)
  boundary is sorted fold (Nat 0) { count, present } pair-lower-bound-step (candidate, count, present)
  length is entry-count sorted
  (sorted select-index (0 .. boundary)) concat (one candidate) concat (sorted select-index (boundary .. length))

sort-pairs is fn (pairs : List (Int, Int, Int)) -> List (Int, Int, Int)
  empty-pairs : List (Int, Int, Int) is Empty
  pairs fold empty-pairs { sorted, candidate } insert-pair (sorted, candidate)

label-at is fn (labels : List Int, index : Int) -> Optional Int
  first (labels select-index (index ..= index))

optional-label is fn (label : Optional Int) -> Int
  label
    Some present then present
    None then 0

relabel-step is fn ((from : Int, to : Int, labels : List Int, label : Int)) -> List Int
  label = from
    true then labels append to
    false then labels append label

relabel-all is fn ((labels : List Int, from : Int, to : Int)) -> List Int
  empty-labels : List Int is Empty
  labels fold empty-labels { updated, label } relabel-step (from, to, updated, label)

join-labels is fn ((labels : List Int, left : Int, right : Int)) -> List Int
  left-label is optional-label (label-at (labels, left))
  right-label is optional-label (label-at (labels, right))
  left-label = right-label
    true then labels
    false then relabel-all (labels, right-label, left-label)

join-pair is fn (labels : List Int, pair : (Int, Int, Int)) -> List Int
  join-labels (labels, pair-left pair, pair-right pair)

initial-labels is fn (points : List (Int, Int, Int)) -> List Int
  length is entry-count points
  collect (0 iterate ({ index } index + 1) take-while ({ index } index < length))

count-label is fn ((sought : Int, count : Nat, label : Int)) -> Nat
  label = sought
    true then count + 1
    false then count

label-size is fn (labels : List Int, sought : Int) -> Nat
  labels fold (Nat 0) { count, label } count-label (sought, count, label)

unique-label-step is fn (selected : List Int, label : Int) -> List Int
  selected contains-entry label
    true then selected
    false then selected append label

append-label-size is fn ((labels : List Int, sizes : List Nat, label : Int)) -> List Nat
  sizes append (label-size (labels, label))

component-sizes is fn (labels : List Int) -> List Nat
  empty-labels : List Int is Empty
  unique is labels fold empty-labels { selected, label } unique-label-step (selected, label)
  empty-sizes : List Nat is Empty
  unique fold empty-sizes { sizes, label } append-label-size (labels, sizes, label)

size-bound is fn ((candidate : Nat, count : Nat, present : Nat)) -> Nat
  present > candidate
    true then count + 1
    false then count

insert-size is fn (sorted : List Nat, candidate : Nat) -> List Nat
  boundary is sorted fold (Nat 0) { count, present } size-bound (candidate, count, present)
  length is entry-count sorted
  ((sorted select-index (0 .. boundary)) append candidate) concat (sorted select-index (boundary .. length))

three-size-product is fn (sizes : List Int) -> Int
  sorted : List Nat is (component-sizes sizes) fold (Empty Nat) { ordered, size } insert-size (ordered, size)
  (sorted select-index (0 .. 3)) fold 1 { product, size } product * size

nearest-source is fn (points : List (Int, Int, Int), connections : Nat) -> Int
  selected is (sort-pairs (point-pairs points)) select-index (0 .. connections)
  labels is selected fold (initial-labels points) { current, pair } join-pair (current, pair)
  three-size-product labels

### Join the requested number of nearest 3D point pairs and multiply the three largest component sizes.
pub nearest-component-product is fn (points : List (Int, Int, Int), connections : Nat) -> Int
  nearest-source (points, connections)

connection-labels is fn ((labels : List Int, answer : Int)) -> List Int
  labels
connection-answer is fn ((labels : List Int, answer : Int)) -> Int
  answer

point-x-optional is fn (point : Optional (Int, Int, Int)) -> Int
  point
    Some present then point3-x present
    None then 0

connection-step is fn ((points : List (Int, Int, Int), state : (List Int, Int), pair : (Int, Int, Int))) -> (List Int, Int)
  labels is connection-labels state
  left is pair-left pair
  right is pair-right pair
  joined? is (optional-label (label-at (labels, left))) = (optional-label (label-at (labels, right)))
  joined?
    true then state
    false then (join-labels (labels, left, right), (point-x-optional (point3-at (points, left))) * (point-x-optional (point3-at (points, right))))

final-connection-source is fn (points : List (Int, Int, Int)) -> Int
  final is (sort-pairs (point-pairs points)) fold (initial-labels points, 0) { state, pair } connection-step (points, state, pair)
  connection-answer final

### Complete nearest-first 3D clustering and multiply the x coordinates of the final joining pair.
pub final-connection-x-product is fn (points : List (Int, Int, Int)) -> Int
  final-connection-source points

point-x is fn ((x : Int, y : Int)) -> Int
  x
point-y is fn ((x : Int, y : Int)) -> Int
  y

rectangle-area is fn (left : (Int, Int), right : (Int, Int)) -> Int
  ((absolute ((point-x right) - (point-x left))) + 1) * ((absolute ((point-y right) - (point-y left))) + 1)

maximum-area is fn (current : Int, candidate : Int) -> Int
  current < candidate
    true then candidate
    false then current

largest-with-left is fn (points : List (Int, Int), left : (Int, Int)) -> Int
  points fold 0 { largest, right } maximum-area (largest, rectangle-area (left, right))

largest-point-step is fn ((points : List (Int, Int), largest : Int, left : (Int, Int))) -> Int
  maximum-area (largest, largest-with-left (points, left))

largest-point-source is fn (points : List (Int, Int)) -> Int
  points fold 0 { largest, left } largest-point-step (points, largest, left)

### Return the largest inclusive axis-aligned rectangle having two supplied 2D points as corners.
pub largest-point-rectangle is fn (points : List (Int, Int)) -> Int
  largest-point-source points

minimum-int is fn (left : Int, right : Int) -> Int
  left < right
    true then left
    false then right

maximum-int is fn (left : Int, right : Int) -> Int
  left > right
    true then left
    false then right

vertical-crosses? is fn ((left : (Int, Int), right : (Int, Int), edge-start : (Int, Int), edge-end : (Int, Int))) -> Boolean
  minimum-x is minimum-int (point-x left, point-x right)
  maximum-x is maximum-int (point-x left, point-x right)
  minimum-y is minimum-int (point-y left, point-y right)
  maximum-y is maximum-int (point-y left, point-y right)
  inside-x is ((point-x edge-start) > minimum-x) and ((point-x edge-start) < maximum-x)
  overlaps-low is (minimum-int (point-y edge-start, point-y edge-end)) < maximum-y
  overlaps-high is (maximum-int (point-y edge-start, point-y edge-end)) > minimum-y
  inside-x and (overlaps-low and overlaps-high)

horizontal-crosses? is fn ((left : (Int, Int), right : (Int, Int), edge-start : (Int, Int), edge-end : (Int, Int))) -> Boolean
  minimum-x is minimum-int (point-x left, point-x right)
  maximum-x is maximum-int (point-x left, point-x right)
  minimum-y is minimum-int (point-y left, point-y right)
  maximum-y is maximum-int (point-y left, point-y right)
  inside-y is ((point-y edge-start) > minimum-y) and ((point-y edge-start) < maximum-y)
  overlaps-low is (minimum-int (point-x edge-start, point-x edge-end)) < maximum-x
  overlaps-high is (maximum-int (point-x edge-start, point-x edge-end)) > minimum-x
  inside-y and (overlaps-low and overlaps-high)

edge-crosses? is fn ((left : (Int, Int), right : (Int, Int), edge-start : (Int, Int), edge-end : (Int, Int))) -> Boolean
  (vertical-crosses? (left, right, edge-start, edge-end)) or (horizontal-crosses? (left, right, edge-start, edge-end))

point-at is fn (points : List (Int, Int), index : Nat) -> Optional (Int, Int)
  first (points select-index (index ..= index))

edge-valid-end is fn ((left : (Int, Int), right : (Int, Int), edge-start : (Int, Int), edge-end : Optional (Int, Int))) -> Boolean
  edge-end
    Some present then not (edge-crosses? (left, right, edge-start, present))
    None then true

edge-valid-start is fn ((points : List (Int, Int), left : (Int, Int), right : (Int, Int), index : Nat, edge-start : Optional (Int, Int))) -> Boolean
  length is entry-count points
  next is (index + 1) % length
  edge-start
    Some present then edge-valid-end (left, right, present, point-at (points, next))
    None then true

rectangle-edge-valid? is fn ((points : List (Int, Int), left : (Int, Int), right : (Int, Int), valid? : Boolean, index : Nat)) -> Boolean
  valid? and (edge-valid-start (points, left, right, index, point-at (points, index)))

contained-candidate-valid? is fn ((points : List (Int, Int), left : (Int, Int), right : (Int, Int))) -> Boolean
  length is entry-count points
  indexes is collect (0 iterate ({ index } index + 1) take-while ({ index } index < length))
  indexes fold true { valid?, index } rectangle-edge-valid? (points, left, right, valid?, index)

contained-with-left-step is fn ((points : List (Int, Int), left : (Int, Int), largest : Int, right : (Int, Int))) -> Int
  contained-candidate-valid? (points, left, right)
    true then maximum-area (largest, rectangle-area (left, right))
    false then largest

contained-with-left is fn (points : List (Int, Int), left : (Int, Int)) -> Int
  points fold 0 { largest, right } contained-with-left-step (points, left, largest, right)

contained-point-step is fn ((points : List (Int, Int), largest : Int, left : (Int, Int))) -> Int
  maximum-area (largest, contained-with-left (points, left))

largest-contained-source is fn (points : List (Int, Int)) -> Int
  points fold 0 { largest, left } contained-point-step (points, largest, left)

### Return the largest inclusive rectangle contained by the closed orthogonal polygon in vertex order.
pub largest-contained-rectangle is fn (vertices : List (Int, Int)) -> Int
  largest-contained-source vertices
