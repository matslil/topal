#!/usr/bin/env topal
use language (
  version is v0.1
)
# Demonstrates an unsigned little endian layout, an MMIO address range,
# an aligned offset, checked location construction, and ordered write/read.
UInt32LE is (
  storage-size is 32[b],
  encoding is UnsignedBinary,
  endian is Little,
  access is ReadWrite
) Layout Nat
# Text, product, sum, and sequence layouts use the same closed construction.
Utf8Text is (
  storage-size is 64[b],
  encoding is Utf8,
  length is NoLength,
  termination is NoTerminator
) Layout String
HeaderLayout is (packing is Natural) Layout (
  first is UInt32LE,
  second is UInt32LE
)
MaybeNatLayout is (
  storage-size is 64[b],
  encoding is Tagged,
  tag-layout is UInt32LE,
  tags is (none is 0, some is 1),
  payload-placement is AfterTag
) Layout Optional Nat
PairArrayLayout is (
  storage-size is 64[b],
  element-layout is UInt32LE,
  stride is 4[B]
) Layout Array 2 Nat
DeviceAddresses is AddressRange (
  caching is Uncached,
  minimum-access-size is 32[b],
  medium is MMIO
)
device is DeviceAddresses (0x40000000 .. 0x4000ffff)
DeviceOffset is AddressOffset (range is device, alignment is 4)
control-offset is DeviceOffset 32
ControlLocation is Location UInt32LE
control is ControlLocation control-offset
stored is UInt32LE 42
control write stored
read control
