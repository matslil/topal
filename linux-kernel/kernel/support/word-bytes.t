use language (version is v0.1, features is (systems))

# Explicit internal byte encoding; ordinary arithmetic and checked byte stores.
quotient is fn (whole : Int, remainder : Int) -> Int
  whole

remainder is fn (whole : Int, rest : Int) -> Int
  rest

store-byte-7 is fn ((region : KernelByteRegion, value : Int)) -> Boolean
  parts is value /% 256
  byte is absolute (remainder parts)
  region byte store (offset-bytes is 7, value is byte)
    true then true
    false then false

store-byte-6 is fn ((region : KernelByteRegion, value : Int)) -> Boolean
  parts is value /% 256
  byte is absolute (remainder parts)
  region byte store (offset-bytes is 6, value is byte)
    true then store-byte-7 (region, quotient parts)
    false then false

store-byte-5 is fn ((region : KernelByteRegion, value : Int)) -> Boolean
  parts is value /% 256
  byte is absolute (remainder parts)
  region byte store (offset-bytes is 5, value is byte)
    true then store-byte-6 (region, quotient parts)
    false then false

store-byte-4 is fn ((region : KernelByteRegion, value : Int)) -> Boolean
  parts is value /% 256
  byte is absolute (remainder parts)
  region byte store (offset-bytes is 4, value is byte)
    true then store-byte-5 (region, quotient parts)
    false then false

store-byte-3 is fn ((region : KernelByteRegion, value : Int)) -> Boolean
  parts is value /% 256
  byte is absolute (remainder parts)
  region byte store (offset-bytes is 3, value is byte)
    true then store-byte-4 (region, quotient parts)
    false then false

store-byte-2 is fn ((region : KernelByteRegion, value : Int)) -> Boolean
  parts is value /% 256
  byte is absolute (remainder parts)
  region byte store (offset-bytes is 2, value is byte)
    true then store-byte-3 (region, quotient parts)
    false then false

store-byte-1 is fn ((region : KernelByteRegion, value : Int)) -> Boolean
  parts is value /% 256
  byte is absolute (remainder parts)
  region byte store (offset-bytes is 1, value is byte)
    true then store-byte-2 (region, quotient parts)
    false then false

store-byte-0 is fn ((region : KernelByteRegion, value : Int)) -> Boolean
  parts is value /% 256
  byte is absolute (remainder parts)
  region byte store (offset-bytes is 0, value is byte)
    true then store-byte-1 (region, quotient parts)
    false then false

store-word is fn ((region : KernelByteRegion, value : Nat)) -> Boolean
  value <= 18446744073709551615
    true then store-byte-0 (region, value)
    false then false

pub store-u64-le is fn ((region : KernelByteRegion, value : Nat)) -> Boolean
  extent is region byte count
  extent >= 8
    true then store-word (region, value)
    false then false
