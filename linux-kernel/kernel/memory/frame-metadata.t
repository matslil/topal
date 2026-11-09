use language (version is v0.1, features is (systems))

# TK-KERNEL-FRAME-POLICY-001. Ordinary policy over an exclusive metadata owner.
# Numeric indexes/results are facts; the caller retains all frame authority.
word is fn ((region : KernelByteRegion, offset : Nat)) -> Nat
  invalid : Nat is 18446744073709551615
  region unsigned load (offset-bytes is offset, byte-count is 8, byte-order is little-endian)
    Some value then value
    None then invalid

write is fn ((region : KernelByteRegion, offset : Nat, value : Nat)) -> Boolean
  region unsigned store (offset-bytes is offset, byte-count is 8, byte-order is little-endian, value is value)

both is fn ((first : Boolean, second : Boolean)) -> Boolean
  first
    true then second
    false then false

cell is fn (index : Nat) -> Nat
  128 + (index * 24)

complete is fn ((region : KernelByteRegion, result : Nat)) -> Boolean
  value is write (region, 88, result)
  phase is write (region, 40, 4)
  both (value, phase)

skip is fn ((region : KernelByteRegion, cursor : Nat)) -> Boolean
  run is write (region, 64, 0)
  next is write (region, 48, cursor + 1)
  both (run, next)

select-span is fn ((region : KernelByteRegion, start : Nat)) -> Boolean
  cursor is write (region, 48, start)
  length is write (region, 64, 0)
  result is write (region, 88, start + 1)
  phase is write (region, 40, 1)
  both (both (cursor, length), both (result, phase))

maybe-select is fn ((region : KernelByteRegion, start : Nat, length : Nat)) -> Boolean
  length >= (word (region, 8))
    true then select-span (region, start)
    false then true

extend-span is fn ((region : KernelByteRegion, cursor : Nat, start : Nat, length : Nat, provenance : Nat, physical : Nat)) -> Boolean
  a is write (region, 56, start)
  b is write (region, 64, length)
  c is write (region, 72, provenance)
  d is write (region, 80, physical)
  e is write (region, 48, cursor + 1)
  stored is both (both (a, b), both (c, both (d, e)))
  stored
    true then maybe-select (region, start, length)
    false then false

aligned is fn ((physical : Nat, alignment : Nat)) -> Boolean
  alignment = 0
    true then false
    false then (physical % alignment) = 0

start-span is fn ((region : KernelByteRegion, cursor : Nat, provenance : Nat, physical : Nat)) -> Boolean
  aligned (physical, (word (region, 16)) + 1)
    true then extend-span (region, cursor, cursor, 1, provenance, physical)
    false then skip (region, cursor)

free-cell is fn ((region : KernelByteRegion, cursor : Nat, provenance : Nat, physical : Nat)) -> Boolean
  length is word (region, 64)
  same is both (provenance = (word (region, 72)), physical = ((word (region, 80)) + 1))
  both (length > 0, same)
    true then extend-span (region, cursor, word (region, 56), length + 1, provenance, physical)
    false then start-span (region, cursor, provenance, physical)

search-cell is fn ((region : KernelByteRegion, cursor : Nat)) -> Boolean
  base is cell cursor
  identity is word (region, base)
  provenance is word (region, base + 8)
  physical is word (region, base + 16)
  both (identity = 0, provenance > 0)
    true then free-cell (region, cursor, provenance, physical)
    false then skip (region, cursor)

search is fn (region : KernelByteRegion) -> Boolean
  cursor is word (region, 48)
  cursor < (word (region, 0))
    true then search-cell (region, cursor)
    false then complete (region, 0)

maybe-complete is fn ((region : KernelByteRegion, processed : Nat)) -> Boolean
  processed = (word (region, 8))
    true then complete (region, word (region, 88))
    false then true

advance-commit is fn ((region : KernelByteRegion, cursor : Nat, processed : Nat)) -> Boolean
  next is write (region, 48, cursor + 1)
  length is write (region, 64, processed + 1)
  both (next, length)
    true then maybe-complete (region, processed + 1)
    false then false

commit-cell is fn ((region : KernelByteRegion, cursor : Nat, identity : Nat)) -> Boolean
  write (region, cell cursor, identity)
    true then advance-commit (region, cursor, word (region, 64))
    false then false

allocate-cell is fn ((region : KernelByteRegion, cursor : Nat)) -> Boolean
  base is cell cursor
  free is (word (region, base)) = 0
  same is (word (region, base + 8)) = (word (region, 72))
  both (free, same)
    true then commit-cell (region, cursor, word (region, 24))
    false then false

release-cell is fn ((region : KernelByteRegion, cursor : Nat)) -> Boolean
  base is cell cursor
  own is (word (region, base)) = (word (region, 24))
  same is both ((word (region, base + 8)) = (word (region, 32)), (word (region, 32)) > 0)
  both (own, same)
    true then commit-cell (region, cursor, 0)
    false then false

commit-selected is fn ((region : KernelByteRegion, cursor : Nat, allocating : Boolean)) -> Boolean
  allocating
    true then allocate-cell (region, cursor)
    false then release-cell (region, cursor)

commit is fn ((region : KernelByteRegion, allocating : Boolean)) -> Boolean
  cursor is word (region, 48)
  valid is both (cursor < (word (region, 0)), (word (region, 64)) < (word (region, 8)))
  valid
    true then commit-selected (region, cursor, allocating)
    false then false

begin-release is fn (region : KernelByteRegion) -> Boolean
  start is word (region, 56)
  cursor is write (region, 48, start)
  length is write (region, 64, 0)
  result is write (region, 88, start + 1)
  phase is write (region, 40, 3)
  both (both (cursor, length), both (result, phase))

maybe-begin-release is fn ((region : KernelByteRegion, processed : Nat)) -> Boolean
  processed = (word (region, 8))
    true then begin-release region
    false then true

validated-cell is fn ((region : KernelByteRegion, cursor : Nat, physical : Nat)) -> Boolean
  processed is word (region, 64)
  a is write (region, 48, cursor + 1)
  b is write (region, 64, processed + 1)
  c is write (region, 80, physical)
  both (a, both (b, c))
    true then maybe-begin-release (region, processed + 1)
    false then false

validate-cell is fn ((region : KernelByteRegion, cursor : Nat)) -> Boolean
  base is cell cursor
  own is (word (region, base)) = (word (region, 24))
  same is (word (region, base + 8)) = (word (region, 32))
  physical is word (region, base + 16)
  first is (word (region, 64)) = 0
  contiguous is physical = ((word (region, 80)) + 1)
  valid is both (both (own, same), not (both (not first, not contiguous)))
  valid
    true then validated-cell (region, cursor, physical)
    false then false

validate-release is fn (region : KernelByteRegion) -> Boolean
  cursor is word (region, 48)
  interval is both (cursor < (word (region, 0)), (word (region, 64)) < (word (region, 8)))
  valid is both (interval, (word (region, 32)) > 0)
  valid
    true then validate-cell (region, cursor)
    false then false

phase-step is fn ((region : KernelByteRegion, phase : Nat)) -> Boolean
  phase
    <= 0 then search region
    <= 1 then commit (region, true)
    <= 2 then validate-release region
    <= 3 then commit (region, false)
    <= 4 then true
    otherwise false

step is fn (region : KernelByteRegion) -> Boolean
  size is region byte count
  cells is word (region, 0)
  count is word (region, 8)
  identity is word (region, 24)
  geometry is both (size >= (128 + (cells * 24)), count > 0)
  issuance is both (identity > 0, identity < 18446744073709551615)
  both (geometry, issuance)
    true then phase-step (region, word (region, 40))
    false then false

step-status is fn (region : KernelByteRegion) -> Nat
  zero : Nat is 0
  invalid : Nat is 18446744073709551615
  step region
    true then zero
    false then invalid

next-status is fn ((region : KernelByteRegion, status : Nat)) -> Nat
  invalid : Nat is 18446744073709551615
  status = 0
    true then step-status region
    false then invalid

finish is fn ((region : KernelByteRegion, status : Nat)) -> Nat
  invalid : Nat is 18446744073709551615
  ready is both (status = 0, (word (region, 40)) = 4)
  ready
    true then word (region, 88)
    false then invalid

stop-limit is fn ((region : KernelByteRegion, remaining : Nat, status : Nat)) -> Nat
  zero : Nat is 0
  running is both (status = 0, not ((word (region, 40)) = 4))
  running
    true then zero
    false then remaining

pub frame-policy is fn ((region : KernelByteRegion, remaining : Nat, stop : Nat, status : Nat)) -> Nat : Decreases remaining
  remaining
    <= 0 then finish (region, status)
    <= stop then finish (region, status)
    otherwise frame-policy (region, remaining - 1, stop-limit (region, remaining, status), next-status (region, status))
