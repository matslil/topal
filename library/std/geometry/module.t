#!/usr/bin/env topal
use language (
  version is v0.1
)

### Revision of the exact finite geometry algorithm namespace.
pub revision is 1

### Join the requested number of nearest 3D point pairs and multiply the three largest component sizes.
pub nearest-component-product is fn (points : List (Int, Int, Int), connections : Nat) -> Int
  geometry-nearest-component-product (points, connections)

### Complete nearest-first 3D clustering and multiply the x coordinates of the final joining pair.
pub final-connection-x-product is fn (points : List (Int, Int, Int)) -> Int
  geometry-final-connection-x-product points

### Return the largest inclusive axis-aligned rectangle having two supplied 2D points as corners.
pub largest-point-rectangle is fn (points : List (Int, Int)) -> Int
  geometry-largest-point-rectangle points

### Return the largest such rectangle contained by the closed orthogonal polygon in vertex order.
pub largest-contained-rectangle is fn (vertices : List (Int, Int)) -> Int
  geometry-largest-contained-rectangle vertices
