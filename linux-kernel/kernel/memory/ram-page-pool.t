use language (version is v0.1, features is (systems))
use library support (version is v0.1)

# Facts identify an operation, not a pointer or live allocation grant.
# The exclusive controller binds this region to the real eight-byte map.
allocate is fn ((pool : KernelByteRegion, slot : Nat)) -> Boolean
  slot = 0
    true then support page-slots reserve pool
    false then false

release-or-refuse is fn ((pool : KernelByteRegion, operation : Nat, slot : Nat)) -> Boolean
  operation = 1
    true then support page-slots release (pool, slot)
    false then false

dispatch is fn ((pool : KernelByteRegion, operation : Nat, slot : Nat)) -> Boolean
  operation = 0
    true then allocate (pool, slot)
    false then release-or-refuse (pool, operation, slot)

pub apply is fn ((pool : KernelByteRegion, operation : Nat, slot : Nat)) -> Boolean
  extent is pool byte count
  extent = 8
    true then dispatch (pool, operation, slot)
    false then false
