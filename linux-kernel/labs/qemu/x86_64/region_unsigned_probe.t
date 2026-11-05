use language (version is v0.1, features is (systems))

write-field is fn ((region : KernelByteRegion, offset : Nat, value : Nat)) -> Boolean
  region unsigned store (offset-bytes is offset, byte-count is 8, byte-order is little-endian, value is value)

read-field is fn ((region : KernelByteRegion, offset : Nat, expected : Nat)) -> Boolean
  region unsigned load (offset-bytes is offset, byte-count is 8, byte-order is little-endian)
    Some actual then actual = expected
    None then false

readonly-field is fn ((region : KernelByteRegion, offset : Nat, value : Nat)) -> Boolean
  region byte window (offset-bytes is 0, byte-count is 8, rights is read-only)
    Some alias then write-field (alias, offset, value)
    None then false

dispatch-five is fn ((region : KernelByteRegion, offset : Nat, value : Nat, command : Nat)) -> Boolean
  command = 5
    true then write-field (region, offset, value + 57896044618658097711785492504343953926634992332820282019728792003956564819968)
    false then read-field (region, offset + 57896044618658097711785492504343953926634992332820282019728792003956564819968, value)

dispatch-four is fn ((region : KernelByteRegion, offset : Nat, value : Nat, command : Nat)) -> Boolean
  command = 4
    true then read-field (region, offset + 18446744073709551616, value)
    false then dispatch-five (region, offset, value, command)

dispatch-three is fn ((region : KernelByteRegion, offset : Nat, value : Nat, command : Nat)) -> Boolean
  command = 3
    true then write-field (region, offset, value + 18446744073709551616)
    false then dispatch-four (region, offset, value, command)

dispatch-two is fn ((region : KernelByteRegion, offset : Nat, value : Nat, command : Nat)) -> Boolean
  command = 2
    true then readonly-field (region, offset, value)
    false then dispatch-three (region, offset, value, command)

dispatch-one is fn ((region : KernelByteRegion, offset : Nat, value : Nat, command : Nat)) -> Boolean
  command = 1
    true then read-field (region, offset, value)
    false then dispatch-two (region, offset, value, command)

pub apply is fn ((region : KernelByteRegion, offset : Nat, value : Nat, command : Nat)) -> Boolean
  command = 0
    true then write-field (region, offset, value)
    false then dispatch-one (region, offset, value, command)
