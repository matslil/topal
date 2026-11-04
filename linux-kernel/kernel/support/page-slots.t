use language (version is v0.1, features is (systems))

reserve-if-free is fn ((pool : KernelByteRegion, slot : Nat, occupied : Nat)) -> Boolean
  occupied = 0
    true then pool byte store (offset-bytes is slot, value is 1)
    false then false

try-reserve is fn ((pool : KernelByteRegion, slot : Nat)) -> Boolean
  pool byte load (offset-bytes is slot)
    Some occupied then reserve-if-free (pool, slot, occupied)
    None then false

reserve-7 is fn (pool : KernelByteRegion) -> Boolean
  try-reserve (pool, 7)
    true then true
    false then false

reserve-6 is fn (pool : KernelByteRegion) -> Boolean
  try-reserve (pool, 6)
    true then true
    false then reserve-7 pool

reserve-5 is fn (pool : KernelByteRegion) -> Boolean
  try-reserve (pool, 5)
    true then true
    false then reserve-6 pool

reserve-4 is fn (pool : KernelByteRegion) -> Boolean
  try-reserve (pool, 4)
    true then true
    false then reserve-5 pool

reserve-3 is fn (pool : KernelByteRegion) -> Boolean
  try-reserve (pool, 3)
    true then true
    false then reserve-4 pool

reserve-2 is fn (pool : KernelByteRegion) -> Boolean
  try-reserve (pool, 2)
    true then true
    false then reserve-3 pool

reserve-1 is fn (pool : KernelByteRegion) -> Boolean
  try-reserve (pool, 1)
    true then true
    false then reserve-2 pool

pub reserve is fn (pool : KernelByteRegion) -> Boolean
  try-reserve (pool, 0)
    true then true
    false then reserve-1 pool

release-if-occupied is fn ((pool : KernelByteRegion, slot : Nat, occupied : Nat)) -> Boolean
  occupied = 1
    true then pool byte store (offset-bytes is slot, value is 0)
    false then false

release-slot is fn ((pool : KernelByteRegion, slot : Nat)) -> Boolean
  pool byte load (offset-bytes is slot)
    Some occupied then release-if-occupied (pool, slot, occupied)
    None then false

pub release is fn ((pool : KernelByteRegion, slot : Nat)) -> Boolean
  slot < 8
    true then release-slot (pool, slot)
    false then false
