#!/usr/bin/env topal
use language (
  version is v0.1
)
use library std (
  version is v0.1
)

parse is std parse integer-triples
final-product is std geometry final-connection-x-product

solve is fn (input : String) -> Int
  final-product (parse input)
