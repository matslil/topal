#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates settled modular numeric families: checked canonical construction,
# explicit reduction, wrapping arithmetic, canonical display, equality, and
# ordering for unsigned and signed representative ranges.
ByteCounter is ModNat (0 ..= 255)
SignedByte is ModInt ((-128) ..= 127)
(
  (ByteCounter 255) + (ByteCounter 1),
  (SignedByte 127) + (SignedByte 1),
  (-1) modulo ByteCounter,
  128 modulo SignedByte,
  (ByteCounter 1) < (ByteCounter 2),
  (SignedByte (-1)) = (255 modulo SignedByte)
)
