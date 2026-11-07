use language (version is v0.1, features is (systems))

# Private Linux-personality encoding, independent of native register width.
quotient is fn (whole : Int, remainder : Int) -> Int
  whole

type-prefix is fn (mode : Nat) -> Nat
  parts is mode /% 4096
  whole is absolute (quotient parts)
  whole * 4096

valid-non-fifo-type is fn (prefix : Nat) -> Boolean
  prefix = 16384
    true then true
    false then prefix = 32768

valid-type is fn (prefix : Nat) -> Boolean
  prefix = 4096
    true then true
    false then valid-non-fifo-type prefix

write-time is fn ((region : KernelByteRegion, seconds : Nat)) -> Boolean
  region unsigned store (offset-bytes is 24, byte-count is 8, byte-order is little-endian, value is seconds)

write-mode-and-time is fn ((region : KernelByteRegion, mode : Nat, seconds : Nat)) -> Boolean
  region unsigned store (offset-bytes is 0, byte-count is 8, byte-order is little-endian, value is mode)
    true then write-time (region, seconds)
    false then false

preserve-type is fn ((region : KernelByteRegion, old-mode : Nat, permissions : Nat, seconds : Nat)) -> Boolean
  prefix is type-prefix old-mode
  valid-type prefix
    true then write-mode-and-time (region, prefix + permissions, seconds)
    false then false

read-mode is fn ((region : KernelByteRegion, permissions : Nat, seconds : Nat)) -> Boolean
  region unsigned load (offset-bytes is 0, byte-count is 8, byte-order is little-endian)
    Some mode then preserve-type (region, mode, permissions, seconds)
    None then false

admit-time is fn ((region : KernelByteRegion, permissions : Nat, seconds : Nat)) -> Boolean
  seconds <= 18446744073709551615
    true then read-mode (region, permissions, seconds)
    false then false

admit-permissions is fn ((region : KernelByteRegion, permissions : Nat, seconds : Nat)) -> Boolean
  permissions <= 4095
    true then admit-time (region, permissions, seconds)
    false then false

pub apply-mode is fn ((region : KernelByteRegion, permissions : Nat, seconds : Nat)) -> Boolean
  extent is region byte count
  extent >= 32
    true then admit-permissions (region, permissions, seconds)
    false then false
