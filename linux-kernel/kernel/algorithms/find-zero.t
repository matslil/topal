use language (version is v0.1, features is (systems))

# TK-DEC-282, TOPAL-SYSTEMS-TAIL-NAT-OWNER-001.
# The remaining count at the first zero identifies its position: subtract it
# from the original requested end. This is a fact, never allocation authority.
# Zero means no match. The all-ones word marks an invalid requested interval;
# an owning allocator must reject that protocol result before granting an extent.
classify-byte is fn ((occupied : Nat, remaining : Nat)) -> Nat
  zero : Nat is 0
  occupied = 0
    true then remaining
    false then zero

probe-byte is fn ((region : KernelByteRegion, offset : Nat, remaining : Nat)) -> Nat
  invalid : Nat is 18446744073709551615
  region byte load (offset-bytes is offset)
    Some occupied then classify-byte (occupied, remaining)
    None then invalid

next-position is fn ((region : KernelByteRegion, remaining : Nat, offset : Nat, state : Nat)) -> Nat
  state = 0
    true then probe-byte (region, offset, remaining)
    false then state

pub find-zero is fn ((region : KernelByteRegion, remaining : Nat, offset : Nat, state : Nat)) -> Nat : Decreases remaining
  remaining
    <= 0 then state
    otherwise find-zero (region, remaining - 1, offset + 1, next-position (region, remaining, offset, state))
