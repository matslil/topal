#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates ordinary nominal modular Lists across construction, equality,
# complete decomposition, private boundaries, Tuple packages, and observations.
ByteCounter is ModNat (0 ..= 255)
SignedByte is ModInt ((-128) ..= 127)

head-or is fn (candidate : List ByteCounter, fallback : ByteCounter) -> ByteCounter
  candidate
    Entry (first, rest) then first
    Empty then fallback

second-or is fn (candidate : List ByteCounter, fallback : ByteCounter) -> ByteCounter
  candidate
    Entry (ignored, rest) then head-or (rest, fallback)
    Empty then fallback

return-list is fn (candidate : List ByteCounter) -> List ByteCounter
  candidate

return-pair is fn (package : (List ByteCounter, ByteCounter)) -> (List ByteCounter, ByteCounter)
  package

apply-package is fn ((candidate : List ByteCounter, fallback : ByteCounter)) -> ByteCounter
  head-or (candidate, fallback)

apply-pair : Function is { (candidate, fallback) } head-or (candidate, fallback)

values : List ByteCounter is Entry (ByteCounter 0, Entry (ByteCounter 255, Entry (ByteCounter 42, Empty)))
copy : List ByteCounter is Entry (ByteCounter 0, Entry (ByteCounter 255, Entry (ByteCounter 42, Empty)))
different : List ByteCounter is Entry (ByteCounter 0, Entry (ByteCounter 255, Entry (ByteCounter 43, Empty)))
shorter : List ByteCounter is Entry (ByteCounter 0, Entry (ByteCounter 255, Empty))
empty-values : List ByteCounter is Empty
forwarded is return-list values
paired is return-pair (values, ByteCounter 9)

(
  head-or (forwarded, ByteCounter 7),
  second-or (forwarded, ByteCounter 7),
  values = copy,
  values != different,
  values != shorter,
  entry-count values,
  empty? empty-values,
  apply-package (candidate is empty-values, fallback is ByteCounter 7),
  apply-pair paired,
  head-or (empty-values, ByteCounter 8),
  forwarded
)
