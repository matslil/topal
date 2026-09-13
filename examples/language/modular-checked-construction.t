#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates a reusable named finite range plus checked construction whose
# result depends on a runtime Int value.
ByteRange is 0 ..= 255
ByteCounter is ModNat ByteRange
construct is fn (value : Int) -> Result (ByteCounter, lang arithmetic ArithmeticErrorCode)
  ByteCounter value
accepted is construct 255
rejected is construct 256
(
  accepted,
  rejected,
  rejected code,
  rejected domain,
  rejected source
)
