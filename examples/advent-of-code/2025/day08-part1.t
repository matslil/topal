#!/usr/bin/env topal
use language (
  version is v0.1
)
use library std (
  version is v0.1
)

parse is std parse integer-triples
component-product is std geometry nearest-component-product

solve is fn (input : String) -> Int
  component-product (parse input, 1000)
