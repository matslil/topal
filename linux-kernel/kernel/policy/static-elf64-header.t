use language (version is v0.1)
use library support (version is v0.1)
# The adapter supplies copied semantic bytes and bounded caller facts.
pub header-admitted? is fn ((bytes : Array (64, Int), machine : Nat, maximum : Nat)) -> Boolean
  support elf admitted? (bytes, machine, maximum)
