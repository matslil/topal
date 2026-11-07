use language (version is v0.1, features is (systems))

# One owner, checked source/destination offsets and ordinary Read/Write effects.
pub copy-byte is fn ((region : KernelByteRegion, source : Nat, destination : Nat)) -> Boolean
  region byte load (offset-bytes is source)
    Some value then region byte store (offset-bytes is destination, value is value)
    None then false
