#!/usr/bin/env topal-debug
use language (
  version is v0.1
)
# Demonstrates reversible modular construction, reduction, wrapping, equality,
# and canonical-representative ordering for ModNat and ModInt.
ByteCounter is ModNat (0 .. 255)
SignedByte is ModInt ((-128) .. 127)
((ByteCounter 255) + (ByteCounter 1), (-1) modulo ByteCounter, (SignedByte 127) + (SignedByte 1))
