use language (version is v0.1, features is (systems))

# Encoding is explicit and independent of the architecture word size.
pub store-u64-le is fn ((region : KernelByteRegion, value : Nat)) -> Boolean
  region unsigned store (offset-bytes is 0, byte-count is 8, byte-order is little-endian, value is value)
